//! Palettes and coloring for Fractadyne (M1).
//!
//! A palette is a small list of gradient stops in `0..1`. The renderer interpolates
//! between them in-shader, cycled by the smooth iteration value. Stops are chosen
//! to loop seamlessly (first and last colors match). A data-driven LUT / custom
//! gradient editor comes later (UI-DESIGN.md §6, §8); these presets are the start.

/// Max stops the GPU uniform carries (must match `fractadyne-gpu`).
pub const MAX_STOPS: usize = 8;

/// One 8-bit sRGB channel → linear. Pasted colours are written by humans and tools in sRGB
/// (`#ff8800`, `255 136 0`), while every stop in this crate is LINEAR — mixing the two silently
/// produces a washed-out gradient rather than an error, so the conversion lives here next to the
/// parser that needs it.
fn srgb8_to_linear(v: u8) -> f32 {
    let c = v as f32 / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Parse a pasted palette into linear-RGB colours, in the order given.
///
/// Deliberately format-tolerant rather than format-specific: the goal ("I found a palette on the
/// web") is defeated by a format war, so this accepts what people actually paste —
///
/// - hex, with or without `#`, 3- or 6-digit: `#ff8800`, `ff8800`, `#f80`
/// - 0–255 triples, the Fractint/KF `.map` line shape: `255 136 0`
/// - separated by newlines, commas, semicolons or spaces, in any mixture
/// - `;` and `//` line comments (the `.map` convention), ignored
///
/// Each line is classified independently: if it carries hex tokens they win, otherwise its
/// integers are taken in groups of three. That resolves the one genuine ambiguity — `255 000 000`
/// is three integers, `FF0000` is one hex colour — without asking the user to declare a format.
///
/// Returns `Err` with a human-readable reason; the caller shows it verbatim.
pub fn parse_palette_text(text: &str) -> Result<Vec<[f32; 3]>, String> {
    fn hex_token(t: &str) -> Option<[u8; 3]> {
        let h = t.strip_prefix('#').unwrap_or(t);
        let ok = |s: &str| s.chars().all(|c| c.is_ascii_hexdigit());
        match h.len() {
            6 if ok(h) => Some([
                u8::from_str_radix(&h[0..2], 16).ok()?,
                u8::from_str_radix(&h[2..4], 16).ok()?,
                u8::from_str_radix(&h[4..6], 16).ok()?,
            ]),
            // #f80 is shorthand for #ff8800 — each digit doubled, per CSS.
            3 if ok(h) => {
                let d = |i: usize| -> Option<u8> {
                    let v = u8::from_str_radix(&h[i..i + 1], 16).ok()?;
                    Some(v * 17)
                };
                Some([d(0)?, d(1)?, d(2)?])
            }
            _ => None,
        }
    }

    let mut out: Vec<[f32; 3]> = Vec::new();
    for raw in text.lines() {
        // Strip comments. `#` is NOT a comment marker here — it introduces hex.
        let line = match (raw.find(';'), raw.find("//")) {
            (Some(a), Some(b)) => &raw[..a.min(b)],
            (Some(a), None) => &raw[..a],
            (None, Some(b)) => &raw[..b],
            (None, None) => raw,
        };
        let tokens: Vec<&str> = line
            .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
            .filter(|t| !t.is_empty())
            .collect();
        if tokens.is_empty() {
            continue;
        }
        let hexes: Vec<[u8; 3]> = tokens.iter().filter_map(|t| hex_token(t)).collect();
        // A line counts as hex only if EVERY token parsed — a half-parsed line is a malformed
        // line, and silently keeping the half that worked is how a wrong palette gets imported.
        if !hexes.is_empty() && hexes.len() == tokens.len() {
            out.extend(hexes.iter().map(|c| {
                [
                    srgb8_to_linear(c[0]),
                    srgb8_to_linear(c[1]),
                    srgb8_to_linear(c[2]),
                ]
            }));
            continue;
        }
        let ints: Vec<u32> = tokens.iter().filter_map(|t| t.parse::<u32>().ok()).collect();
        if ints.len() == tokens.len() && ints.len() % 3 == 0 && !ints.is_empty() {
            if let Some(bad) = ints.iter().find(|v| **v > 255) {
                return Err(format!("{bad} is out of range — RGB values run 0–255"));
            }
            for c in ints.chunks(3) {
                out.push([
                    srgb8_to_linear(c[0] as u8),
                    srgb8_to_linear(c[1] as u8),
                    srgb8_to_linear(c[2] as u8),
                ]);
            }
            continue;
        }
        return Err(format!(
            "couldn't read \"{}\" — expected hex colours (#ff8800) or 0–255 triples (255 136 0)",
            line.trim()
        ));
    }
    if out.is_empty() {
        return Err("no colours found".to_string());
    }
    Ok(out)
}

/// Reduce a colour list to at most `n` evenly spaced stops, always keeping the first and last.
///
/// A `.map` file carries 256 baked entries and the GPU uniform carries eight, so importing one
/// is necessarily lossy; sampling evenly across the list (rather than truncating to the first
/// eight, which would import only the palette's dark end) preserves the gradient's overall shape.
pub fn resample_colors(colors: &[[f32; 3]], n: usize) -> Vec<[f32; 3]> {
    let n = n.max(1);
    if colors.len() <= n {
        return colors.to_vec();
    }
    if n == 1 {
        return vec![colors[0]];
    }
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            let idx = (t * (colors.len() - 1) as f32).round() as usize;
            colors[idx.min(colors.len() - 1)]
        })
        .collect()
}

/// A named gradient palette: ascending `(position 0..1, linear RGB 0..1)` stops.
pub struct Palette {
    pub name: &'static str,
    pub stops: &'static [(f32, [f32; 3])],
}

impl Palette {
    /// Pack into a fixed `[[r,g,b,pos]; MAX_STOPS]` array plus the active count, for
    /// GPU upload. Unused trailing slots repeat the last stop.
    pub fn packed(&self) -> ([[f32; 4]; MAX_STOPS], u32) {
        let mut out = [[0.0f32; 4]; MAX_STOPS];
        let n = self.stops.len().clamp(1, MAX_STOPS);
        for (i, slot) in out.iter_mut().enumerate() {
            let (pos, c) = self.stops[i.min(n - 1)];
            *slot = [c[0], c[1], c[2], pos];
        }
        (out, n as u32)
    }
}

/// Built-in palettes (names match the design mockups).
pub const PRESETS: &[Palette] = &[
    Palette {
        name: "Ember",
        stops: &[
            (0.00, [0.00, 0.00, 0.00]),
            (0.15, [0.45, 0.02, 0.02]),
            (0.40, [0.90, 0.30, 0.02]),
            (0.62, [1.00, 0.70, 0.12]),
            (0.82, [1.00, 1.00, 0.82]),
            (1.00, [0.00, 0.00, 0.00]),
        ],
    },
    Palette {
        name: "Ice",
        stops: &[
            (0.00, [0.00, 0.02, 0.10]),
            (0.30, [0.00, 0.30, 0.60]),
            (0.55, [0.20, 0.70, 0.92]),
            (0.80, [0.82, 0.95, 1.00]),
            (1.00, [0.00, 0.02, 0.10]),
        ],
    },
    Palette {
        name: "Nebula",
        stops: &[
            (0.00, [0.05, 0.00, 0.10]),
            (0.25, [0.40, 0.00, 0.50]),
            (0.50, [0.90, 0.20, 0.50]),
            (0.70, [0.20, 0.60, 0.70]),
            (0.86, [0.85, 0.92, 0.72]),
            (1.00, [0.05, 0.00, 0.10]),
        ],
    },
    Palette {
        name: "Grayscale",
        stops: &[
            (0.00, [0.00, 0.00, 0.00]),
            (0.50, [1.00, 1.00, 1.00]),
            (1.00, [0.00, 0.00, 0.00]),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Each packed slot is `[r, g, b, pos]` — the order the GPU uniform expects.
    #[test]
    fn packed_slot_is_rgb_then_pos() {
        let p = Palette { name: "t", stops: &[(0.0, [0.1, 0.2, 0.3]), (1.0, [0.4, 0.5, 0.6])] };
        let (out, n) = p.packed();
        assert_eq!(n, 2);
        assert_eq!(out[0], [0.1, 0.2, 0.3, 0.0]);
        assert_eq!(out[1], [0.4, 0.5, 0.6, 1.0]);
    }

    /// Unused trailing slots repeat the last real stop (so the shader's `stop_count` bound and the
    /// padded array agree — reading past `n` still yields the terminal color, never garbage).
    #[test]
    fn trailing_slots_repeat_last_stop() {
        let p = Palette { name: "t", stops: &[(0.0, [0.0; 3]), (0.5, [1.0; 3]), (1.0, [0.2; 3])] };
        let (out, n) = p.packed();
        assert_eq!(n, 3);
        assert_eq!(out[2], [0.2, 0.2, 0.2, 1.0]);
        for slot in &out[n as usize..] {
            assert_eq!(*slot, out[2]);
        }
    }

    /// The shapes people actually paste, all accepted.
    #[test]
    fn palette_text_accepts_common_shapes() {
        // CSS-ish hex list on one line.
        let c = parse_palette_text("#ff0000, #00ff00, #0000ff").unwrap();
        assert_eq!(c.len(), 3);
        // Pure red in sRGB is pure red in linear; the green/blue channels stay at zero.
        assert!((c[0][0] - 1.0).abs() < 1e-6 && c[0][1] == 0.0 && c[0][2] == 0.0);
        // Bare hex, one per line, mixed case.
        assert_eq!(parse_palette_text("ff0000\n00FF00\n").unwrap().len(), 2);
        // 3-digit shorthand: #f00 == #ff0000.
        assert_eq!(parse_palette_text("#f00").unwrap()[0], c[0]);
        // Fractint / KF .map triples, with a trailing comment line.
        let m = parse_palette_text("255 0 0\n0 255 0\n; a comment\n").unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m[0], c[0]);
        // Several triples on one line.
        assert_eq!(parse_palette_text("255 0 0 0 255 0").unwrap().len(), 2);
    }

    /// sRGB is converted to linear, not copied through. Mid-grey is the cheap tell: 128/255 is
    /// ~0.502 in sRGB but ~0.216 in linear, and getting this wrong washes out every import.
    #[test]
    fn palette_text_converts_srgb_to_linear() {
        let c = parse_palette_text("#808080").unwrap();
        assert!((c[0][0] - 0.2159).abs() < 1e-3, "got {}", c[0][0]);
    }

    /// Malformed input is rejected with a reason rather than silently half-imported.
    #[test]
    fn palette_text_rejects_junk() {
        assert!(parse_palette_text("").is_err());
        assert!(parse_palette_text("hello there").is_err());
        assert!(parse_palette_text("300 0 0").is_err()); // out of 0-255 range
        assert!(parse_palette_text("#ff0000 nonsense").is_err()); // half-parsed line
        assert!(parse_palette_text("255 0").is_err()); // incomplete triple
    }

    /// Down-sampling keeps the ends and spans the middle, so a 256-entry .map keeps its shape
    /// instead of importing only its dark end.
    #[test]
    fn resample_keeps_endpoints() {
        let src: Vec<[f32; 3]> = (0..256).map(|i| [i as f32 / 255.0; 3]).collect();
        let out = resample_colors(&src, MAX_STOPS);
        assert_eq!(out.len(), MAX_STOPS);
        assert_eq!(out[0], src[0]);
        assert_eq!(out[MAX_STOPS - 1], src[255]);
        // Monotonic: evenly spaced samples of a ramp stay a ramp.
        assert!(out.windows(2).all(|w| w[0][0] < w[1][0]));
        // Short lists pass through untouched.
        assert_eq!(resample_colors(&src[..3], MAX_STOPS).len(), 3);
    }

    /// A single-stop palette fills every slot with that stop (count 1, no out-of-bounds).
    #[test]
    fn single_stop_fills_all_slots() {
        let p = Palette { name: "t", stops: &[(0.3, [0.7, 0.8, 0.9])] };
        let (out, n) = p.packed();
        assert_eq!(n, 1);
        assert!(out.iter().all(|s| *s == [0.7, 0.8, 0.9, 0.3]));
    }

    /// More stops than the GPU carries: the count saturates at `MAX_STOPS` and the first
    /// `MAX_STOPS` stops are kept (no panic, no overflow).
    #[test]
    fn count_clamps_to_max_stops() {
        let p = Palette {
            name: "t",
            stops: &[
                (0.0, [0.0; 3]), (0.1, [1.0; 3]), (0.2, [2.0; 3]), (0.3, [3.0; 3]),
                (0.4, [4.0; 3]), (0.5, [5.0; 3]), (0.6, [6.0; 3]), (0.7, [7.0; 3]),
                (0.8, [8.0; 3]), (0.9, [9.0; 3]), (1.0, [10.0; 3]),
            ],
        };
        let (out, n) = p.packed();
        assert_eq!(n as usize, MAX_STOPS);
        assert_eq!(out[MAX_STOPS - 1], [7.0, 7.0, 7.0, 0.7]);
    }

    /// Every shipped preset packs within bounds, fits in `MAX_STOPS`, and keeps its first stop.
    #[test]
    fn presets_pack_within_bounds() {
        for p in PRESETS {
            let (out, n) = p.packed();
            assert!((1..=MAX_STOPS as u32).contains(&n), "{}: count {n} out of range", p.name);
            assert_eq!(n as usize, p.stops.len(), "{}: all presets must fit in MAX_STOPS", p.name);
            let (pos, c) = p.stops[0];
            assert_eq!(out[0], [c[0], c[1], c[2], pos], "{}: first slot", p.name);
        }
    }
}
