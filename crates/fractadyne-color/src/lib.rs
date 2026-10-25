//! Palettes and coloring for Fractadyne.
//!
//! Three layers, per `design/palette-import.md` §4:
//!
//! 1. [`import`] — a palette FILE (`.map` today) becomes a gradient. Format knowledge stops here.
//! 2. [`segment`] — the gradient model itself (GIMP's: per-segment colours, midpoint, blend
//!    function and colour space), a superset of every format we care about, plus the bake that
//!    turns one into the [`segment::LUT_SIZE`]-entry table the shader reads.
//! 3. This module — the built-in [`PRESETS`] and the tolerant paste parser
//!    ([`parse_palette_text`]), both of which are just short stop lists feeding layer 2.
//!
//! ⚠**Every colour here is DISPLAY-referred, and that was measured** — see [`srgb8_to_stop`].
//!
//! Presets are chosen to loop seamlessly (first and last colors match), because the palette is
//! cycled by the smooth iteration value and a mismatched pair shows as a hard seam.

pub mod import;
pub mod segment;

/// Width of the packed `[[r, g, b, pos]; N]` stop array.
///
/// ⚠**No longer a palette limit.** It was the GPU uniform's capacity, and so a hard ceiling on
/// every palette in the app — which is what made a real palette file structurally impossible to
/// represent (Fractint's `default.map` has 37 hard jumps; eight stops give seven segments).
/// Palettes now bake to a [`segment::LUT_SIZE`]-entry LUT and the shader carries no stops at all.
/// What survives is this fixed-width array as a convenient shape for a SHORT stop list: the
/// presets and the random palette animator still use it, and [`segment::Gradient::from_packed`]
/// converts it.
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

    // ⭐Same hygiene as every other importer: CR / CRLF / LF all split, and clipboard damage
    // (a smart quote, an en dash, an invisible space inside a number) repaired before parsing.
    let cleaned = fractadyne_text::clean(text);
    let text = cleaned.text.as_str();
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
/// Sampling evenly across the list (rather than truncating to the first `n`, which would import
/// only the palette's dark end) preserves the gradient's overall shape.
///
/// ⚠This used to be called with `MAX_STOPS`, which threw away everything past the EIGHTH colour of
/// every import. The LUT bake lifted that ceiling; `n` is now a generosity limit rather than a
/// hardware one, and a 256-entry `.map` body passes through untouched.
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
    // ⚠Append only: a view file and a session store a preset by its INDEX here (`palette=`,
    // `palette_idx`), so a preset inserted above another recolours every saved view that used the
    // one after it.
    // The four seasons round the ring: snow, frost, spring leaf, blossom, summer sun, harvest,
    // autumn russet, a bare winter dusk, and back to snow.
    Palette {
        name: "Four Seasons",
        stops: &[
            (0.00, [0.933, 0.957, 1.000]),
            (0.12, [0.561, 0.702, 0.851]),
            (0.25, [0.424, 0.749, 0.271]),
            (0.37, [0.969, 0.659, 0.769]),
            (0.50, [1.000, 0.847, 0.227]),
            (0.62, [0.910, 0.455, 0.102]),
            (0.75, [0.478, 0.180, 0.071]),
            (0.88, [0.227, 0.200, 0.282]),
            (1.00, [0.933, 0.957, 1.000]),
        ],
    },
    // Blues, purples and crimsons, with the gold of a crown as the highlight.
    Palette {
        name: "Royalty",
        stops: &[
            (0.00, [0.043, 0.071, 0.251]),
            (0.17, [0.141, 0.255, 0.722]),
            (0.34, [0.416, 0.184, 0.627]),
            (0.50, [0.231, 0.059, 0.361]),
            (0.64, [0.690, 0.071, 0.180]),
            (0.80, [0.949, 0.757, 0.306]),
            (0.90, [0.478, 0.039, 0.141]),
            (1.00, [0.043, 0.071, 0.251]),
        ],
    },
    // Black light and neon: each flash comes out of the dark and goes back into it.
    Palette {
        name: "Rave",
        stops: &[
            (0.00, [0.031, 0.000, 0.059]),
            (0.14, [1.000, 0.122, 0.784]),
            (0.28, [0.071, 0.000, 0.165]),
            (0.43, [0.000, 0.898, 1.000]),
            (0.57, [0.039, 0.000, 0.125]),
            (0.72, [0.651, 1.000, 0.000]),
            (0.86, [0.102, 0.000, 0.188]),
            (0.93, [1.000, 0.902, 0.000]),
            (1.00, [0.031, 0.000, 0.059]),
        ],
    },
    // Ireland: deep forest, emerald, shamrock, sea mist, stone walls and the Atlantic.
    Palette {
        name: "Emerald Isle",
        stops: &[
            (0.00, [0.016, 0.188, 0.102]),
            (0.18, [0.039, 0.541, 0.271]),
            (0.34, [0.310, 0.761, 0.227]),
            (0.50, [0.863, 0.937, 0.839]),
            (0.64, [0.557, 0.588, 0.565]),
            (0.80, [0.122, 0.353, 0.400]),
            (1.00, [0.016, 0.188, 0.102]),
        ],
    },
    // Harvest gold, avocado, burnt orange and chocolate brown on cream.
    Palette {
        name: "Seventies",
        stops: &[
            (0.00, [0.239, 0.133, 0.063]),
            (0.16, [0.769, 0.353, 0.071]),
            (0.32, [0.886, 0.655, 0.180]),
            (0.48, [0.953, 0.890, 0.722]),
            (0.62, [0.561, 0.604, 0.165]),
            (0.78, [0.353, 0.290, 0.071]),
            (0.90, [0.659, 0.282, 0.118]),
            (1.00, [0.239, 0.133, 0.063]),
        ],
    },
    // Synthwave night: purple, hot pink, a sunset orange and turquoise neon.
    Palette {
        name: "Eighties",
        stops: &[
            (0.00, [0.071, 0.016, 0.188]),
            (0.17, [0.416, 0.106, 0.690]),
            (0.33, [1.000, 0.176, 0.584]),
            (0.50, [1.000, 0.604, 0.165]),
            (0.60, [1.000, 0.941, 0.478]),
            (0.75, [0.086, 0.839, 0.784]),
            (0.88, [0.165, 0.227, 0.847]),
            (1.00, [0.071, 0.016, 0.188]),
        ],
    },
    // Fields of white, red, blue and yellow, each edged by a black line.
    Palette {
        name: "Mondrian",
        stops: &[
            (0.0000, [0.957, 0.945, 0.910]),
            (0.2564, [0.957, 0.945, 0.910]),
            (0.2564, [0.067, 0.067, 0.067]),
            (0.2872, [0.067, 0.067, 0.067]),
            (0.2872, [0.831, 0.149, 0.110]),
            (0.4667, [0.831, 0.149, 0.110]),
            (0.4667, [0.067, 0.067, 0.067]),
            (0.4974, [0.067, 0.067, 0.067]),
            (0.4974, [0.957, 0.945, 0.910]),
            (0.6256, [0.957, 0.945, 0.910]),
            (0.6256, [0.067, 0.067, 0.067]),
            (0.6564, [0.067, 0.067, 0.067]),
            (0.6564, [0.122, 0.247, 0.651]),
            (0.8103, [0.122, 0.247, 0.651]),
            (0.8103, [0.067, 0.067, 0.067]),
            (0.8410, [0.067, 0.067, 0.067]),
            (0.8410, [0.949, 0.784, 0.122]),
            (0.9692, [0.949, 0.784, 0.122]),
            (0.9692, [0.067, 0.067, 0.067]),
            (1.0000, [0.067, 0.067, 0.067]),
        ],
    },
    // Obsidian edges round each scale, deep jade and iridescent teal, glints of gold and ember.
    Palette {
        name: "Dragon Scales",
        stops: &[
            (0.00, [0.012, 0.039, 0.027]),
            (0.10, [0.043, 0.239, 0.173]),
            (0.30, [0.075, 0.541, 0.384]),
            (0.45, [0.247, 0.831, 0.690]),
            (0.58, [0.847, 0.761, 0.353]),
            (0.70, [0.604, 0.290, 0.078]),
            (0.82, [0.227, 0.055, 0.024]),
            (1.00, [0.012, 0.039, 0.027]),
        ],
    },
    // Black enamel, gunmetal, brushed steel, a chrome highlight and brass fittings.
    Palette {
        name: "Machine Age",
        stops: &[
            (0.00, [0.047, 0.051, 0.063]),
            (0.20, [0.227, 0.251, 0.282]),
            (0.40, [0.549, 0.588, 0.627]),
            (0.50, [0.949, 0.961, 0.973]),
            (0.60, [0.659, 0.690, 0.722]),
            (0.75, [0.722, 0.537, 0.180]),
            (0.88, [0.353, 0.247, 0.071]),
            (1.00, [0.047, 0.051, 0.063]),
        ],
    },
    // Earth, cast bronze, a polished gold edge, copper, and the verdigris of age.
    Palette {
        name: "Bronze Age",
        stops: &[
            (0.00, [0.141, 0.078, 0.039]),
            (0.18, [0.541, 0.353, 0.141]),
            (0.32, [0.878, 0.690, 0.384]),
            (0.46, [0.690, 0.376, 0.180]),
            (0.62, [0.290, 0.627, 0.541]),
            (0.78, [0.071, 0.235, 0.204]),
            (1.00, [0.141, 0.078, 0.039]),
        ],
    },
    // Every hue in one ring, evenly bright: all the colours together.
    Palette {
        name: "Unity",
        stops: &[
            (0.00, [0.910, 0.200, 0.247]),
            (0.16, [0.969, 0.576, 0.165]),
            (0.32, [0.961, 0.847, 0.227]),
            (0.48, [0.271, 0.722, 0.306]),
            (0.64, [0.165, 0.541, 0.847]),
            (0.80, [0.478, 0.298, 0.769]),
            (1.00, [0.910, 0.200, 0.247]),
        ],
    },
    // Clove, cinnamon, paprika, saffron, turmeric and cardamom.
    Palette {
        name: "Spice",
        stops: &[
            (0.00, [0.180, 0.078, 0.039]),
            (0.16, [0.541, 0.267, 0.125]),
            (0.32, [0.784, 0.196, 0.102]),
            (0.48, [0.949, 0.549, 0.102]),
            (0.62, [0.969, 0.776, 0.137]),
            (0.78, [0.541, 0.604, 0.275]),
            (0.90, [0.290, 0.227, 0.118]),
            (1.00, [0.180, 0.078, 0.039]),
        ],
    },
    // Eggplant, beetroot, tomato, carrot, sweetcorn, broccoli and kale.
    Palette {
        name: "Veggies",
        stops: &[
            (0.00, [0.180, 0.059, 0.212]),
            (0.14, [0.541, 0.071, 0.282]),
            (0.28, [0.878, 0.196, 0.110]),
            (0.42, [0.969, 0.510, 0.102]),
            (0.56, [0.969, 0.847, 0.290]),
            (0.72, [0.310, 0.561, 0.133]),
            (0.86, [0.090, 0.247, 0.110]),
            (1.00, [0.180, 0.059, 0.212]),
        ],
    },
    // Tesserae of lapis, ochre, terracotta, turquoise, cream and wine, set in pale grout.
    Palette {
        name: "Mosaic",
        stops: &[
            (0.0000, [0.114, 0.247, 0.561]),
            (0.1689, [0.114, 0.247, 0.561]),
            (0.1689, [0.788, 0.761, 0.690]),
            (0.1892, [0.788, 0.761, 0.690]),
            (0.1892, [0.851, 0.635, 0.227]),
            (0.3243, [0.851, 0.635, 0.227]),
            (0.3243, [0.788, 0.761, 0.690]),
            (0.3446, [0.788, 0.761, 0.690]),
            (0.3446, [0.710, 0.325, 0.180]),
            (0.5135, [0.710, 0.325, 0.180]),
            (0.5135, [0.788, 0.761, 0.690]),
            (0.5338, [0.788, 0.761, 0.690]),
            (0.5338, [0.184, 0.639, 0.627]),
            (0.6689, [0.184, 0.639, 0.627]),
            (0.6689, [0.788, 0.761, 0.690]),
            (0.6892, [0.788, 0.761, 0.690]),
            (0.6892, [0.949, 0.902, 0.784]),
            (0.8243, [0.949, 0.902, 0.784]),
            (0.8243, [0.788, 0.761, 0.690]),
            (0.8446, [0.788, 0.761, 0.690]),
            (0.8446, [0.431, 0.102, 0.180]),
            (0.9797, [0.431, 0.102, 0.180]),
            (0.9797, [0.788, 0.761, 0.690]),
            (1.0000, [0.788, 0.761, 0.690]),
        ],
    },
    // Deep water, turquoise shallows, sand, coral and a sea-anemone violet.
    Palette {
        name: "Coral Reef",
        stops: &[
            (0.00, [0.024, 0.165, 0.353]),
            (0.18, [0.082, 0.659, 0.722]),
            (0.34, [0.949, 0.886, 0.690]),
            (0.50, [1.000, 0.435, 0.369]),
            (0.64, [1.000, 0.706, 0.227]),
            (0.80, [0.478, 0.247, 0.620]),
            (1.00, [0.024, 0.165, 0.353]),
        ],
    },
    // The northern lights: green curtains and magenta veils on a night sky.
    Palette {
        name: "Aurora",
        stops: &[
            (0.00, [0.012, 0.024, 0.086]),
            (0.20, [0.059, 0.420, 0.290]),
            (0.36, [0.235, 0.941, 0.541]),
            (0.50, [0.086, 0.627, 0.627]),
            (0.66, [0.769, 0.227, 0.659]),
            (0.82, [0.165, 0.055, 0.353]),
            (1.00, [0.012, 0.024, 0.086]),
        ],
    },
    // Dusk violet, red rock, sand, bleached bone, sage brush and desert sky.
    Palette {
        name: "Desert",
        stops: &[
            (0.00, [0.227, 0.141, 0.282]),
            (0.18, [0.722, 0.290, 0.165]),
            (0.34, [0.902, 0.722, 0.471]),
            (0.48, [0.980, 0.941, 0.863]),
            (0.62, [0.561, 0.620, 0.447]),
            (0.80, [0.416, 0.627, 0.831]),
            (1.00, [0.227, 0.141, 0.282]),
        ],
    },
    // A sweet shop: raspberry, bubblegum, lavender, sky, mint and lemon.
    Palette {
        name: "Candy",
        stops: &[
            (0.00, [0.690, 0.071, 0.306]),
            (0.16, [1.000, 0.561, 0.769]),
            (0.32, [0.769, 0.659, 0.961]),
            (0.48, [0.561, 0.831, 1.000]),
            (0.64, [0.561, 0.949, 0.784]),
            (0.80, [1.000, 0.949, 0.541]),
            (1.00, [0.690, 0.071, 0.306]),
        ],
    },
    // Engineering blue with thin white lines, like a print from the drawing office.
    Palette {
        name: "Blueprint",
        stops: &[
            (0.0000, [0.051, 0.184, 0.400]),
            (0.2100, [0.114, 0.310, 0.604]),
            (0.2350, [0.949, 0.969, 1.000]),
            (0.2600, [0.141, 0.349, 0.651]),
            (0.5000, [0.071, 0.247, 0.525]),
            (0.7100, [0.114, 0.310, 0.604]),
            (0.7350, [0.949, 0.969, 1.000]),
            (0.7600, [0.141, 0.349, 0.651]),
            (1.0000, [0.051, 0.184, 0.400]),
        ],
    },
    // An old photograph: dark umber through sepia to warm paper.
    Palette {
        name: "Sepia",
        stops: &[
            (0.00, [0.118, 0.078, 0.047]),
            (0.30, [0.431, 0.290, 0.165]),
            (0.55, [0.690, 0.541, 0.361]),
            (0.78, [0.937, 0.878, 0.749]),
            (1.00, [0.118, 0.078, 0.047]),
        ],
    },
    // A thermal camera's ironbow: cold black and blue, through red and orange, to white heat.
    Palette {
        name: "Thermal",
        stops: &[
            (0.00, [0.000, 0.000, 0.000]),
            (0.14, [0.102, 0.039, 0.431]),
            (0.30, [0.541, 0.039, 0.604]),
            (0.46, [0.878, 0.133, 0.180]),
            (0.62, [1.000, 0.541, 0.000]),
            (0.78, [1.000, 0.902, 0.227]),
            (0.90, [1.000, 0.984, 0.902]),
            (1.00, [0.000, 0.000, 0.000]),
        ],
    },
    // Jewel-toned glass held in black lead: ruby, sapphire, emerald, amber, amethyst.
    Palette {
        name: "Stained Glass",
        stops: &[
            (0.0000, [0.710, 0.071, 0.165]),
            (0.1724, [0.710, 0.071, 0.165]),
            (0.1724, [0.051, 0.051, 0.051]),
            (0.2000, [0.051, 0.051, 0.051]),
            (0.2000, [0.102, 0.247, 0.722]),
            (0.3724, [0.102, 0.247, 0.722]),
            (0.3724, [0.051, 0.051, 0.051]),
            (0.4000, [0.051, 0.051, 0.051]),
            (0.4000, [0.078, 0.569, 0.353]),
            (0.5724, [0.078, 0.569, 0.353]),
            (0.5724, [0.051, 0.051, 0.051]),
            (0.6000, [0.051, 0.051, 0.051]),
            (0.6000, [0.949, 0.647, 0.086]),
            (0.7724, [0.949, 0.647, 0.086]),
            (0.7724, [0.051, 0.051, 0.051]),
            (0.8000, [0.051, 0.051, 0.051]),
            (0.8000, [0.478, 0.184, 0.659]),
            (0.9724, [0.478, 0.184, 0.659]),
            (0.9724, [0.051, 0.051, 0.051]),
            (1.0000, [0.051, 0.051, 0.051]),
        ],
    },
    // Film noir: black, charcoal, smoke and silver, with one red accent.
    Palette {
        name: "Noir",
        stops: &[
            (0.00, [0.020, 0.020, 0.024]),
            (0.25, [0.180, 0.188, 0.212]),
            (0.50, [0.478, 0.510, 0.565]),
            (0.70, [0.910, 0.925, 0.941]),
            (0.80, [0.769, 0.078, 0.118]),
            (0.88, [0.227, 0.012, 0.024]),
            (1.00, [0.020, 0.020, 0.024]),
        ],
    },
    // Cherry blossom: dark bark, deep pink, sakura, petal white and new leaves.
    Palette {
        name: "Cherry Blossom",
        stops: &[
            (0.00, [0.180, 0.102, 0.118]),
            (0.20, [0.769, 0.227, 0.431]),
            (0.38, [1.000, 0.690, 0.800]),
            (0.54, [1.000, 0.965, 0.973]),
            (0.70, [0.722, 0.847, 0.541]),
            (0.86, [0.353, 0.227, 0.196]),
            (1.00, [0.180, 0.102, 0.118]),
        ],
    },
    // Play of colour on a dark stone: electric blue, green and orange flashes.
    Palette {
        name: "Black Opal",
        stops: &[
            (0.00, [0.024, 0.031, 0.094]),
            (0.14, [0.122, 0.353, 1.000]),
            (0.28, [0.031, 0.071, 0.165]),
            (0.42, [0.165, 1.000, 0.604]),
            (0.56, [0.039, 0.102, 0.141]),
            (0.70, [1.000, 0.416, 0.165]),
            (0.84, [0.753, 0.290, 1.000]),
            (1.00, [0.024, 0.031, 0.094]),
        ],
    },
    // Provence: lavender rows, sky, sunflowers, sage and a violet dusk.
    Palette {
        name: "Lavender Fields",
        stops: &[
            (0.00, [0.180, 0.102, 0.322]),
            (0.18, [0.478, 0.353, 0.722]),
            (0.34, [0.749, 0.682, 0.910]),
            (0.50, [0.659, 0.800, 0.941]),
            (0.64, [0.969, 0.812, 0.227]),
            (0.80, [0.498, 0.604, 0.400]),
            (1.00, [0.180, 0.102, 0.322]),
        ],
    },
];

#[cfg(test)]
#[path = "../tests/unit/tests.rs"]
mod tests;
