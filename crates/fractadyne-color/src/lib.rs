//! Palettes and coloring for Fractadyne (M1).
//!
//! A palette is a small list of gradient stops in `0..1`. The renderer interpolates
//! between them in-shader, cycled by the smooth iteration value. Stops are chosen
//! to loop seamlessly (first and last colors match). A data-driven LUT / custom
//! gradient editor comes later (UI-DESIGN.md §6, §8); these presets are the start.

/// Max stops the GPU uniform carries (must match `fractadyne-gpu`).
pub const MAX_STOPS: usize = 8;

/// One 8-bit channel → the `0..1` a stop holds. A plain divide, **not** an sRGB decode.
///
/// ⭐⭐**Stops are DISPLAY-REFERRED, and this was measured, not assumed** (2026-09-04). The
/// renderer writes palette colours straight into a **non-sRGB** framebuffer, so a stop's value IS
/// the byte the monitor shows (`fractadyne-export`'s module docs spell out why: the live view is
/// WYSIWYG and palette interpolation happens in gamma space on purpose).
///
/// This used to apply the sRGB→linear transfer, on the stated belief that "every stop in this
/// crate is LINEAR". The renderer never honoured it, so the conversion was pure loss: a pasted
/// `#808080` became the stop 0.2159, which reached the framebuffer as byte 55 and displayed as
/// **`#373737`** — every imported palette one decode too dark. Verified with a uniform custom
/// palette rendered through `--render`: stop 0.2159 → `#373737`, and the control, stop 0.502 →
/// `#808080`. The built-in presets never showed it because they were authored by eye against the
/// live view, so their numbers were already display values.
///
/// ⚠So any future palette importer (`.map`, `.ugr`, `.ggr` — see `design/palette-import.md`)
/// targets THIS space. If the renderer is ever made linear, this is one of the two places that
/// must change with it; the other is the shader's final write.
fn srgb8_to_stop(v: u8) -> f32 {
    v as f32 / 255.0
}

/// Parse a pasted palette into DISPLAY-space colours (see `srgb8_to_stop`), in the order given.
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
        // ⭐⭐**A `.map` TRIPLE OUTRANKS BARE HEX SHORTHAND.** `168 168 168` is a real line in
        // Fractint's `default.map` (the VGA light grey) and every token is also three valid hex
        // digits, so the hex-wins rule below read it as THREE `#114488` colours. Any `.map` line
        // whose three values all land in 100–255 was silently mis-imported — in the very format
        // this parser advertises. Bare 3-digit shorthand made only of DECIMAL digits is
        // vanishingly rare next to a `.map` triple, and anyone who means it can write `#128`;
        // a shorthand containing a letter (`f80`) is unambiguous and still takes the hex path.
        let pure_decimal = tokens.iter().all(|t| t.chars().all(|c| c.is_ascii_digit()));
        let map_triple = pure_decimal
            && tokens.len() % 3 == 0
            && !tokens.is_empty()
            && tokens.iter().all(|t| t.parse::<u32>().is_ok_and(|v| v <= 255));

        let hexes: Vec<[u8; 3]> = if map_triple {
            Vec::new()
        } else {
            tokens.iter().filter_map(|t| hex_token(t)).collect()
        };
        // A line counts as hex only if EVERY token parsed — a half-parsed line is a malformed
        // line, and silently keeping the half that worked is how a wrong palette gets imported.
        if !hexes.is_empty() && hexes.len() == tokens.len() {
            out.extend(hexes.iter().map(|c| {
                [
                    srgb8_to_stop(c[0]),
                    srgb8_to_stop(c[1]),
                    srgb8_to_stop(c[2]),
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
                    srgb8_to_stop(c[0] as u8),
                    srgb8_to_stop(c[1] as u8),
                    srgb8_to_stop(c[2] as u8),
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

/// A named gradient palette: ascending `(position 0..1, DISPLAY-space RGB 0..1)` stops.
///
/// ⚠Display-referred, not linear: the shader writes these values straight into a non-sRGB
/// framebuffer, so a stop IS the byte shown. See `srgb8_to_stop` for the measurement.
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
mod tests;
