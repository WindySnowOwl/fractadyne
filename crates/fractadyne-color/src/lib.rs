//! Palettes and coloring for Fractadyne.
//!
//! Three layers, per `design/palette-import.md` §4:
//!
//! 1. [`import`] — a palette FILE (`.map` today) becomes a gradient. Format knowledge stops here.
//! 2. [`segment`] — the gradient model itself (GIMP's: per-segment colors, midpoint, blend
//!    function and color space), a superset of every format we care about, plus the bake that
//!    turns one into the [`segment::LUT_SIZE`]-entry table the shader reads.
//! 3. This module — the built-in [`PRESETS`] and the tolerant paste parser
//!    ([`parse_palette_text`]), both of which are just short stop lists feeding layer 2.
//!
//! ⚠**Every color here is DISPLAY-referred, and that was measured** — see [`srgb8_to_stop`].
//!
//! Presets are chosen to loop seamlessly (first and last colors match), because the palette is
//! cycled by the smooth iteration value and a mismatched pair shows as a hard seam.

pub mod import;
pub mod segment;
pub mod tags;

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
/// renderer writes palette colors straight into a **non-sRGB** framebuffer, so a stop's value IS
/// the byte the monitor shows (`fractadyne-export`'s module docs spell out why: the live view is
/// WYSIWYG and palette interpolation happens in gamma space on purpose).
///
/// This used to apply the sRGB→linear transfer, on the stated belief that "every stop in this
/// crate is LINEAR". The renderer never honored it, so the conversion was pure loss: a pasted
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

/// Parse a pasted palette into DISPLAY-space colors (see `srgb8_to_stop`), in the order given.
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
/// is three integers, `FF0000` is one hex color — without asking the user to declare a format.
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
        // Fractint's `default.map` (the VGA light gray) and every token is also three valid hex
        // digits, so the hex-wins rule below read it as THREE `#114488` colors. Any `.map` line
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
            "couldn't read \"{}\" — expected hex colors (#ff8800) or 0–255 triples (255 136 0)",
            line.trim()
        ));
    }
    if out.is_empty() {
        return Err("no colors found".to_string());
    }
    Ok(out)
}

/// Reduce a color list to at most `n` evenly spaced stops, always keeping the first and last.
///
/// Sampling evenly across the list (rather than truncating to the first `n`, which would import
/// only the palette's dark end) preserves the gradient's overall shape.
///
/// ⚠This used to be called with `MAX_STOPS`, which threw away everything past the EIGHTH color of
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
    /// The palette's permanent identity: what view files, sessions, tours and the command line save
    /// and name it by. Lowercase ASCII letters, digits and hyphens, so it survives the Latin-1
    /// `key=value` lines of a view file and a PNG text chunk. ⛔Never changes once shipped; the
    /// display name and the list's order are free to.
    pub id: &'static str,
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

/// The holiday palettes in [`PRESETS`], in the order of the year, from Lunar New Year to New
/// Year's Eve. The palette lists show them under their own heading.
pub const HOLIDAY_PRESETS: std::ops::Range<usize> = 30..50;

/// A collection of presets the palette lists show under its own heading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetGroup {
    Holidays,
    Minerals,
    Plants,
    Animals,
    Food,
    Painters,
    Biomes,
    Places,
}

/// ⭐**Each collection's run of [`PRESETS`]**, in list order. The first thirty presets come before
/// any heading; the runs that follow are back to back to the end of the list.
pub const PRESET_GROUPS: &[(PresetGroup, std::ops::Range<usize>)] = &[
    (PresetGroup::Holidays, HOLIDAY_PRESETS),
    (PresetGroup::Minerals, 50..70),
    (PresetGroup::Plants, 70..90),
    (PresetGroup::Animals, 90..110),
    (PresetGroup::Food, 110..130),
    (PresetGroup::Painters, 130..150),
    (PresetGroup::Biomes, 150..170),
    (PresetGroup::Places, 170..190),
];

/// The collection whose run starts at preset `i`, if one does: where a list puts a heading.
pub fn preset_group_starting_at(i: usize) -> Option<PresetGroup> {
    PRESET_GROUPS.iter().find(|(_, r)| r.start == i).map(|(g, _)| *g)
}

/// ⛔**Frozen: the palette a saved NUMBER means.** Until 0.3.0-beta.27 a view file's `palette=`, the
/// session and `--palette` named a preset by its position in [`PRESETS`]. Those numbers are in files
/// people have, so each keeps meaning the palette it meant: number `n` is the preset with id
/// `LEGACY_PRESET_ORDER[n]`, wherever that preset now sits. Never reorder or extend this list; a
/// preset added later has an id and no number.
pub const LEGACY_PRESET_ORDER: &[&str] = &[
    "ember", "ice", "nebula", "grayscale", "four-seasons", "royalty", "rave", "emerald-isle",
    "seventies", "eighties", "mondrian", "dragon-scales", "machine-age", "bronze-age", "unity",
    "spice", "veggies", "mosaic", "coral-reef", "aurora", "desert", "candy", "blueprint", "sepia",
    "thermal", "stained-glass", "noir", "cherry-blossom", "black-opal", "lavender-fields",
    "lunar-new-year", "valentines-day", "mardi-gras", "holi", "nowruz", "eid-al-fitr", "easter",
    "vesak", "midsummer", "matariki", "mid-autumn", "meskel", "halloween", "day-of-the-dead",
    "diwali", "thanksgiving", "hanukkah", "christmas", "kwanzaa", "new-years-eve",
];

/// The preset with this id (ASCII case ignored), as its position in [`PRESETS`].
pub fn preset_by_id(id: &str) -> Option<usize> {
    let id = id.trim();
    PRESETS.iter().position(|p| p.id.eq_ignore_ascii_case(id))
}

/// The preset a NUMBER saved by an older build means (see [`LEGACY_PRESET_ORDER`]).
pub fn preset_by_legacy_number(n: usize) -> Option<usize> {
    LEGACY_PRESET_ORDER.get(n).and_then(|id| preset_by_id(id))
}

/// The number an older build knows preset `i` by, if it has one: what a file writes for them.
pub fn legacy_number_of(i: usize) -> Option<usize> {
    let id = PRESETS.get(i)?.id;
    LEGACY_PRESET_ORDER.iter().position(|l| *l == id)
}

/// A preset named the way a person or a file may name it: its id, its display name (ASCII case
/// ignored either way), or an older build's number for it. What `--palette` and a tour's
/// `palette =` accept.
pub fn preset_by_spec(spec: &str) -> Option<usize> {
    let s = spec.trim();
    if let Ok(n) = s.parse::<usize>() {
        return preset_by_legacy_number(n);
    }
    preset_by_id(s).or_else(|| PRESETS.iter().position(|p| p.name.eq_ignore_ascii_case(s)))
}

/// Built-in palettes (names match the design mockups).
///
/// A palette is saved by its [`Palette::id`], so this list's order is only the order the lists show:
/// a preset can go wherever it reads best. Numbers from older files go through
/// [`LEGACY_PRESET_ORDER`].
pub const PRESETS: &[Palette] = &[
    Palette {
        id: "ember",
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
        id: "ice",
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
        id: "nebula",
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
        id: "grayscale",
        name: "Grayscale",
        stops: &[
            (0.00, [0.00, 0.00, 0.00]),
            (0.50, [1.00, 1.00, 1.00]),
            (1.00, [0.00, 0.00, 0.00]),
        ],
    },
    // ⚠Append only: a view file and a session store a preset by its INDEX here (`palette=`,
    // `palette_idx`), so a preset inserted above another recolors every saved view that used the
    // one after it.
    // The four seasons round the ring: snow, frost, spring leaf, blossom, summer sun, harvest,
    // autumn russet, a bare winter dusk, and back to snow.
    Palette {
        id: "four-seasons",
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
        id: "royalty",
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
        id: "rave",
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
        id: "emerald-isle",
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
        id: "seventies",
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
        id: "eighties",
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
        id: "mondrian",
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
        id: "dragon-scales",
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
        id: "machine-age",
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
        id: "bronze-age",
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
    // Every hue in one ring, evenly bright: all the colors together.
    Palette {
        id: "unity",
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
        id: "spice",
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
        id: "veggies",
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
        id: "mosaic",
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
        id: "coral-reef",
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
        id: "aurora",
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
        id: "desert",
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
        id: "candy",
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
        id: "blueprint",
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
        id: "sepia",
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
        id: "thermal",
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
        id: "stained-glass",
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
        id: "noir",
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
        id: "cherry-blossom",
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
    // Play of color on a dark stone: electric blue, green and orange flashes.
    Palette {
        id: "black-opal",
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
        id: "lavender-fields",
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
    // Lunar New Year: red lanterns and red envelopes, gold, and the pink of plum blossom.
    Palette {
        id: "lunar-new-year",
        name: "Lunar New Year",
        stops: &[
            (0.00, [0.165, 0.012, 0.024]),
            (0.14, [0.647, 0.059, 0.082]),
            (0.28, [0.890, 0.141, 0.106]),
            (0.40, [0.969, 0.788, 0.282]),
            (0.48, [1.000, 0.953, 0.769]),
            (0.56, [0.969, 0.788, 0.282]),
            (0.68, [0.847, 0.118, 0.118]),
            (0.80, [0.957, 0.604, 0.690]),
            (0.90, [0.541, 0.043, 0.071]),
            (1.00, [0.165, 0.012, 0.024]),
        ],
    },
    // Valentine's Day: deep wine, rose red, pink, blush and white lace.
    Palette {
        id: "valentines-day",
        name: "Valentine's Day",
        stops: &[
            (0.00, [0.239, 0.000, 0.067]),
            (0.18, [0.639, 0.000, 0.118]),
            (0.32, [0.878, 0.125, 0.227]),
            (0.48, [1.000, 0.478, 0.612]),
            (0.62, [1.000, 0.820, 0.863]),
            (0.72, [1.000, 0.969, 0.976]),
            (0.84, [0.839, 0.227, 0.369]),
            (1.00, [0.239, 0.000, 0.067]),
        ],
    },
    // Mardi Gras: the purple, green and gold of the New Orleans carnival, with a bead's shine.
    Palette {
        id: "mardi-gras",
        name: "Mardi Gras",
        stops: &[
            (0.00, [0.165, 0.039, 0.290]),
            (0.16, [0.416, 0.122, 0.659]),
            (0.30, [0.690, 0.478, 0.878]),
            (0.40, [0.957, 0.761, 0.102]),
            (0.50, [1.000, 0.902, 0.502]),
            (0.60, [0.122, 0.620, 0.227]),
            (0.74, [0.051, 0.353, 0.133]),
            (0.88, [0.290, 0.082, 0.502]),
            (1.00, [0.165, 0.039, 0.290]),
        ],
    },
    // Holi: clouds of colored powder, magenta, saffron, yellow, green, blue and violet.
    Palette {
        id: "holi",
        name: "Holi",
        stops: &[
            (0.00, [1.000, 0.176, 0.541]),
            (0.13, [1.000, 0.604, 0.122]),
            (0.25, [1.000, 0.878, 0.227]),
            (0.35, [0.969, 0.941, 0.902]),
            (0.46, [0.235, 0.812, 0.306]),
            (0.58, [0.122, 0.710, 0.910]),
            (0.68, [0.969, 0.941, 0.902]),
            (0.79, [0.478, 0.247, 0.847]),
            (0.90, [0.902, 0.165, 0.431]),
            (1.00, [1.000, 0.176, 0.541]),
        ],
    },
    // Nowruz, the Persian new year: the haft-sin table's wheatgrass and hyacinths, a goldfish, and
    // cobalt tiles.
    Palette {
        id: "nowruz",
        name: "Nowruz",
        stops: &[
            (0.00, [0.122, 0.290, 0.071]),
            (0.15, [0.353, 0.659, 0.165]),
            (0.28, [0.761, 0.902, 0.478]),
            (0.40, [0.910, 0.659, 0.847]),
            (0.52, [0.604, 0.290, 0.722]),
            (0.64, [0.102, 0.247, 0.604]),
            (0.76, [0.039, 0.122, 0.290]),
            (0.86, [1.000, 0.541, 0.165]),
            (1.00, [0.122, 0.290, 0.071]),
        ],
    },
    // Eid al-Fitr: a midnight sky, emerald, lantern gold and the silver of the new crescent.
    Palette {
        id: "eid-al-fitr",
        name: "Eid al-Fitr",
        stops: &[
            (0.00, [0.024, 0.078, 0.200]),
            (0.18, [0.059, 0.239, 0.400]),
            (0.32, [0.055, 0.478, 0.416]),
            (0.44, [0.910, 0.722, 0.290]),
            (0.52, [1.000, 0.945, 0.784]),
            (0.60, [0.851, 0.604, 0.169]),
            (0.74, [0.043, 0.302, 0.290]),
            (0.88, [0.788, 0.831, 0.910]),
            (1.00, [0.024, 0.078, 0.200]),
        ],
    },
    // Easter: the stripes of dyed eggs, pastel pink, yellow, mint, sky and lilac, between white
    // lines.
    Palette {
        id: "easter",
        name: "Easter",
        stops: &[
            (0.0000, [0.969, 0.714, 0.784]),
            (0.1754, [0.969, 0.714, 0.784]),
            (0.1754, [1.000, 0.992, 0.969]),
            (0.2000, [1.000, 0.992, 0.969]),
            (0.2000, [0.984, 0.898, 0.541]),
            (0.3754, [0.984, 0.898, 0.541]),
            (0.3754, [1.000, 0.992, 0.969]),
            (0.4000, [1.000, 0.992, 0.969]),
            (0.4000, [0.714, 0.922, 0.776]),
            (0.5754, [0.714, 0.922, 0.776]),
            (0.5754, [1.000, 0.992, 0.969]),
            (0.6000, [1.000, 0.992, 0.969]),
            (0.6000, [0.702, 0.847, 0.961]),
            (0.7754, [0.702, 0.847, 0.961]),
            (0.7754, [1.000, 0.992, 0.969]),
            (0.8000, [1.000, 0.992, 0.969]),
            (0.8000, [0.824, 0.741, 0.941]),
            (0.9754, [0.824, 0.741, 0.941]),
            (0.9754, [1.000, 0.992, 0.969]),
            (1.0000, [1.000, 0.992, 0.969]),
        ],
    },
    // Vesak: lanterns on a night sky, lotus pink and white, saffron, and the maroon of robes.
    Palette {
        id: "vesak",
        name: "Vesak",
        stops: &[
            (0.00, [0.051, 0.094, 0.220]),
            (0.20, [0.941, 0.541, 0.659]),
            (0.34, [0.992, 0.910, 0.933]),
            (0.46, [0.965, 0.718, 0.200]),
            (0.60, [0.910, 0.451, 0.102]),
            (0.76, [0.478, 0.122, 0.071]),
            (0.88, [0.141, 0.125, 0.310]),
            (1.00, [0.051, 0.094, 0.220]),
        ],
    },
    // Midsummer in the Nordic countries: birch bark, meadow, buttercups, wildflowers, cornflowers
    // and the pale sky of a night that never gets dark.
    Palette {
        id: "midsummer",
        name: "Midsummer",
        stops: &[
            (0.00, [0.180, 0.478, 0.196]),
            (0.14, [0.498, 0.761, 0.353]),
            (0.25, [0.961, 0.953, 0.910]),
            (0.33, [0.976, 0.847, 0.290]),
            (0.47, [0.949, 0.608, 0.753]),
            (0.61, [0.302, 0.494, 0.851]),
            (0.75, [0.663, 0.784, 0.933]),
            (0.88, [0.353, 0.604, 0.227]),
            (1.00, [0.180, 0.478, 0.196]),
        ],
    },
    // Matariki, the Māori new year: the Pleiades rising in a midwinter sky before the dawn.
    Palette {
        id: "matariki",
        name: "Matariki",
        stops: &[
            (0.00, [0.008, 0.016, 0.059]),
            (0.22, [0.043, 0.110, 0.290]),
            (0.38, [0.165, 0.357, 0.722]),
            (0.48, [0.812, 0.886, 1.000]),
            (0.52, [1.000, 1.000, 1.000]),
            (0.58, [0.561, 0.698, 1.000]),
            (0.72, [0.102, 0.180, 0.431]),
            (0.86, [0.910, 0.541, 0.290]),
            (0.94, [0.227, 0.102, 0.165]),
            (1.00, [0.008, 0.016, 0.059]),
        ],
    },
    // The Mid-Autumn Festival: the full moon, osmanthus, mooncakes and red lanterns at night.
    Palette {
        id: "mid-autumn",
        name: "Mid-Autumn",
        stops: &[
            (0.00, [0.039, 0.071, 0.188]),
            (0.20, [0.122, 0.184, 0.416]),
            (0.34, [1.000, 0.957, 0.784]),
            (0.42, [0.969, 0.827, 0.416]),
            (0.54, [0.761, 0.471, 0.165]),
            (0.66, [0.416, 0.204, 0.071]),
            (0.78, [0.910, 0.255, 0.122]),
            (0.90, [0.227, 0.063, 0.188]),
            (1.00, [0.039, 0.071, 0.188]),
        ],
    },
    // Meskel in Ethiopia: green highlands, yellow Meskel daisies, white shawls and the bonfire.
    Palette {
        id: "meskel",
        name: "Meskel",
        stops: &[
            (0.00, [0.102, 0.165, 0.039]),
            (0.16, [0.247, 0.478, 0.118]),
            (0.30, [0.969, 0.816, 0.102]),
            (0.40, [1.000, 0.953, 0.659]),
            (0.50, [0.969, 0.957, 0.918]),
            (0.62, [1.000, 0.541, 0.102]),
            (0.74, [0.784, 0.196, 0.102]),
            (0.88, [0.227, 0.071, 0.031]),
            (1.00, [0.102, 0.165, 0.039]),
        ],
    },
    // Halloween: black, a witch's purple, jack-o'-lantern orange and candlelight, and slime green.
    Palette {
        id: "halloween",
        name: "Halloween",
        stops: &[
            (0.00, [0.020, 0.012, 0.031]),
            (0.16, [0.227, 0.078, 0.345]),
            (0.30, [0.478, 0.180, 0.753]),
            (0.42, [1.000, 0.478, 0.039]),
            (0.52, [1.000, 0.690, 0.165]),
            (0.60, [0.102, 0.039, 0.016]),
            (0.74, [0.486, 0.824, 0.102]),
            (0.86, [0.122, 0.227, 0.039]),
            (1.00, [0.020, 0.012, 0.031]),
        ],
    },
    // The Day of the Dead in Mexico: marigolds, papel picado in magenta and turquoise, sugar-skull
    // white and violet.
    Palette {
        id: "day-of-the-dead",
        name: "Day of the Dead",
        stops: &[
            (0.00, [0.039, 0.020, 0.063]),
            (0.14, [1.000, 0.549, 0.000]),
            (0.26, [1.000, 0.761, 0.102]),
            (0.38, [0.878, 0.086, 0.478]),
            (0.50, [0.969, 0.953, 0.933]),
            (0.62, [0.098, 0.710, 0.690]),
            (0.74, [0.416, 0.165, 0.722]),
            (0.86, [0.165, 0.039, 0.227]),
            (1.00, [0.039, 0.020, 0.063]),
        ],
    },
    // Diwali, the festival of lights: an indigo night, rangoli magenta and turquoise, marigolds and
    // the flames of the diyas.
    Palette {
        id: "diwali",
        name: "Diwali",
        stops: &[
            (0.00, [0.071, 0.039, 0.227]),
            (0.16, [0.227, 0.086, 0.439]),
            (0.28, [0.839, 0.098, 0.478]),
            (0.40, [1.000, 0.541, 0.102]),
            (0.50, [1.000, 0.824, 0.290]),
            (0.56, [1.000, 0.965, 0.816]),
            (0.66, [1.000, 0.690, 0.125]),
            (0.78, [0.075, 0.639, 0.627]),
            (0.90, [0.102, 0.165, 0.416]),
            (1.00, [0.071, 0.039, 0.227]),
        ],
    },
    // Thanksgiving: corn husk and corn, pumpkin, cranberry, an oak leaf's brown, sage and wheat.
    Palette {
        id: "thanksgiving",
        name: "Thanksgiving",
        stops: &[
            (0.00, [0.957, 0.886, 0.722]),
            (0.14, [0.949, 0.710, 0.267]),
            (0.28, [0.910, 0.451, 0.102]),
            (0.40, [0.722, 0.145, 0.227]),
            (0.50, [0.416, 0.063, 0.125]),
            (0.62, [0.290, 0.165, 0.078]),
            (0.74, [0.604, 0.651, 0.416]),
            (0.86, [0.910, 0.839, 0.627]),
            (1.00, [0.957, 0.886, 0.722]),
        ],
    },
    // Hanukkah: blue and white, silver, and the gold of the candle flames.
    Palette {
        id: "hanukkah",
        name: "Hanukkah",
        stops: &[
            (0.00, [0.039, 0.102, 0.290]),
            (0.18, [0.122, 0.310, 0.722]),
            (0.32, [0.353, 0.541, 0.878]),
            (0.44, [0.910, 0.933, 0.973]),
            (0.52, [0.722, 0.769, 0.831]),
            (0.62, [1.000, 0.824, 0.290]),
            (0.68, [1.000, 0.957, 0.761]),
            (0.80, [0.165, 0.353, 0.784]),
            (1.00, [0.039, 0.102, 0.290]),
        ],
    },
    // Christmas: evergreen, holly red, gold and snow.
    Palette {
        id: "christmas",
        name: "Christmas",
        stops: &[
            (0.00, [0.039, 0.165, 0.071]),
            (0.16, [0.102, 0.416, 0.165]),
            (0.30, [0.784, 0.063, 0.180]),
            (0.40, [0.478, 0.039, 0.094]),
            (0.52, [0.949, 0.757, 0.306]),
            (0.60, [1.000, 0.973, 0.878]),
            (0.68, [0.969, 0.984, 1.000]),
            (0.82, [0.165, 0.541, 0.227]),
            (1.00, [0.039, 0.165, 0.071]),
        ],
    },
    // Kwanzaa: the red, black and green of the bendera, and the light of the kinara's candles.
    Palette {
        id: "kwanzaa",
        name: "Kwanzaa",
        stops: &[
            (0.00, [0.039, 0.039, 0.039]),
            (0.18, [0.710, 0.071, 0.106]),
            (0.32, [0.910, 0.251, 0.165]),
            (0.44, [0.969, 0.753, 0.290]),
            (0.56, [0.102, 0.541, 0.227]),
            (0.72, [0.043, 0.290, 0.118]),
            (0.86, [0.102, 0.039, 0.039]),
            (1.00, [0.039, 0.039, 0.039]),
        ],
    },
    // New Year's Eve: a midnight sky, champagne, silver and fireworks.
    Palette {
        id: "new-years-eve",
        name: "New Year's Eve",
        stops: &[
            (0.00, [0.020, 0.024, 0.102]),
            (0.16, [0.102, 0.122, 0.353]),
            (0.28, [0.910, 0.784, 0.478]),
            (0.36, [1.000, 0.965, 0.863]),
            (0.46, [0.753, 0.776, 0.831]),
            (0.56, [0.071, 0.078, 0.227]),
            (0.66, [1.000, 0.227, 0.541]),
            (0.74, [0.227, 0.878, 1.000]),
            (0.84, [0.102, 0.039, 0.227]),
            (1.00, [0.020, 0.024, 0.102]),
        ],
    },
    // Amethyst: a geode, from its dark violet heart through lilac crystal to white quartz and the
    // gray-brown agate of its rind.
    Palette {
        id: "amethyst",
        name: "Amethyst",
        stops: &[
            (0.00, [0.118, 0.043, 0.200]),
            (0.16, [0.294, 0.102, 0.478]),
            (0.32, [0.541, 0.310, 0.769]),
            (0.44, [0.788, 0.651, 0.925]),
            (0.52, [0.957, 0.925, 0.984]),
            (0.60, [0.725, 0.541, 0.878]),
            (0.74, [0.369, 0.165, 0.573]),
            (0.88, [0.490, 0.435, 0.408]),
            (1.00, [0.118, 0.043, 0.200]),
        ],
    },
    // Malachite: the stone's tight concentric bands of dark and bright green.
    Palette {
        id: "malachite",
        name: "Malachite",
        stops: &[
            (0.00, [0.043, 0.239, 0.149]),
            (0.12, [0.122, 0.541, 0.298]),
            (0.22, [0.055, 0.310, 0.188]),
            (0.34, [0.247, 0.749, 0.478]),
            (0.44, [0.039, 0.200, 0.125]),
            (0.56, [0.165, 0.639, 0.392]),
            (0.66, [0.498, 0.878, 0.659]),
            (0.76, [0.075, 0.376, 0.227]),
            (0.88, [0.345, 0.812, 0.557]),
            (1.00, [0.043, 0.239, 0.149]),
        ],
    },
    // Lapis lazuli: deep ultramarine with flecks of golden pyrite and veins of white calcite.
    Palette {
        id: "lapis-lazuli",
        name: "Lapis Lazuli",
        stops: &[
            (0.00, [0.031, 0.078, 0.275]),
            (0.20, [0.110, 0.227, 0.620]),
            (0.34, [0.184, 0.353, 0.839]),
            (0.40, [0.878, 0.722, 0.298]),
            (0.44, [0.114, 0.216, 0.580]),
            (0.62, [0.071, 0.149, 0.478]),
            (0.70, [0.812, 0.839, 0.902]),
            (0.76, [0.141, 0.282, 0.722]),
            (1.00, [0.031, 0.078, 0.275]),
        ],
    },
    // Turquoise: robin's-egg blue-green crossed by the brown-black webbing of its matrix.
    Palette {
        id: "turquoise",
        name: "Turquoise",
        stops: &[
            (0.00, [0.169, 0.114, 0.078]),
            (0.08, [0.369, 0.784, 0.753]),
            (0.30, [0.561, 0.878, 0.831]),
            (0.45, [0.247, 0.702, 0.690]),
            (0.52, [0.420, 0.353, 0.243]),
            (0.56, [0.286, 0.741, 0.714]),
            (0.78, [0.659, 0.925, 0.875]),
            (0.92, [0.227, 0.663, 0.651]),
            (1.00, [0.169, 0.114, 0.078]),
        ],
    },
    // Obsidian: black volcanic glass, with the gray glint and green and violet sheen of a polished
    // face.
    Palette {
        id: "obsidian",
        name: "Obsidian",
        stops: &[
            (0.00, [0.020, 0.020, 0.027]),
            (0.30, [0.082, 0.086, 0.110]),
            (0.45, [0.227, 0.247, 0.290]),
            (0.50, [0.541, 0.561, 0.600]),
            (0.55, [0.169, 0.227, 0.200]),
            (0.70, [0.047, 0.051, 0.063]),
            (0.82, [0.290, 0.235, 0.353]),
            (1.00, [0.020, 0.020, 0.027]),
        ],
    },
    // Rose quartz: soft translucent pink, with a cloudy lavender-gray.
    Palette {
        id: "rose-quartz",
        name: "Rose Quartz",
        stops: &[
            (0.00, [0.953, 0.776, 0.812]),
            (0.25, [0.976, 0.867, 0.890]),
            (0.45, [1.000, 0.957, 0.965]),
            (0.60, [0.914, 0.655, 0.714]),
            (0.70, [0.788, 0.702, 0.788]),
            (0.82, [0.851, 0.541, 0.627]),
            (1.00, [0.953, 0.776, 0.812]),
        ],
    },
    // Tiger's eye: chatoyant bands of honey gold over dark brown, each with its silky flash.
    Palette {
        id: "tigers-eye",
        name: "Tiger's Eye",
        stops: &[
            (0.00, [0.165, 0.086, 0.024]),
            (0.15, [0.420, 0.231, 0.059]),
            (0.28, [0.753, 0.541, 0.180]),
            (0.34, [0.941, 0.776, 0.416]),
            (0.40, [0.659, 0.439, 0.118]),
            (0.55, [0.231, 0.125, 0.039]),
            (0.70, [0.541, 0.353, 0.102]),
            (0.78, [0.851, 0.659, 0.290]),
            (0.86, [0.353, 0.208, 0.063]),
            (1.00, [0.165, 0.086, 0.024]),
        ],
    },
    // Azurite: deep azure crystals grown together with green malachite.
    Palette {
        id: "azurite",
        name: "Azurite",
        stops: &[
            (0.00, [0.031, 0.082, 0.310]),
            (0.20, [0.102, 0.231, 0.690]),
            (0.35, [0.247, 0.435, 0.878]),
            (0.45, [0.051, 0.165, 0.541]),
            (0.55, [0.118, 0.420, 0.290]),
            (0.65, [0.247, 0.682, 0.416]),
            (0.75, [0.059, 0.290, 0.200]),
            (0.88, [0.102, 0.231, 0.690]),
            (1.00, [0.031, 0.082, 0.310]),
        ],
    },
    // Rhodochrosite: banded raspberry and rose, with thin white bands.
    Palette {
        id: "rhodochrosite",
        name: "Rhodochrosite",
        stops: &[
            (0.00, [0.478, 0.102, 0.227]),
            (0.12, [0.761, 0.251, 0.416]),
            (0.20, [0.941, 0.541, 0.659]),
            (0.26, [0.992, 0.902, 0.925]),
            (0.32, [0.878, 0.337, 0.494]),
            (0.46, [0.620, 0.137, 0.314]),
            (0.56, [0.949, 0.627, 0.722]),
            (0.62, [1.000, 0.941, 0.957]),
            (0.70, [0.820, 0.290, 0.455]),
            (0.86, [0.557, 0.133, 0.275]),
            (1.00, [0.478, 0.102, 0.227]),
        ],
    },
    // Labradorite: a dark gray stone that flashes blue, teal, green and gold as it turns.
    Palette {
        id: "labradorite",
        name: "Labradorite",
        stops: &[
            (0.00, [0.165, 0.176, 0.200]),
            (0.18, [0.290, 0.314, 0.349]),
            (0.30, [0.122, 0.353, 0.620]),
            (0.40, [0.184, 0.690, 0.784]),
            (0.48, [0.541, 0.816, 0.627]),
            (0.54, [0.910, 0.769, 0.353]),
            (0.62, [0.290, 0.314, 0.349]),
            (0.80, [0.118, 0.125, 0.145]),
            (1.00, [0.165, 0.176, 0.200]),
        ],
    },
    // Moonstone: milky white with the blue glow that floats across it.
    Palette {
        id: "moonstone",
        name: "Moonstone",
        stops: &[
            (0.00, [0.925, 0.910, 0.941]),
            (0.16, [0.965, 0.957, 0.973]),
            (0.28, [0.722, 0.800, 0.941]),
            (0.38, [0.290, 0.471, 0.847]),
            (0.46, [0.165, 0.314, 0.690]),
            (0.54, [0.541, 0.667, 0.941]),
            (0.66, [0.933, 0.941, 0.973]),
            (0.78, [0.941, 0.847, 0.769]),
            (0.88, [0.847, 0.784, 0.784]),
            (1.00, [0.925, 0.910, 0.941]),
        ],
    },
    // Jade: jadeite from deep green to the milky pale green of its translucent edge.
    Palette {
        id: "jade",
        name: "Jade",
        stops: &[
            (0.00, [0.118, 0.353, 0.227]),
            (0.20, [0.165, 0.604, 0.353]),
            (0.36, [0.353, 0.784, 0.533]),
            (0.48, [0.831, 0.941, 0.863]),
            (0.56, [0.941, 0.980, 0.949]),
            (0.64, [0.722, 0.878, 0.769]),
            (0.78, [0.353, 0.659, 0.471]),
            (0.90, [0.180, 0.478, 0.314]),
            (1.00, [0.118, 0.353, 0.227]),
        ],
    },
    // Marble: white Carrara stone with its blue-gray veins.
    Palette {
        id: "marble",
        name: "Marble",
        stops: &[
            (0.00, [0.957, 0.957, 0.949]),
            (0.22, [0.910, 0.910, 0.902]),
            (0.30, [0.604, 0.635, 0.675]),
            (0.34, [0.243, 0.275, 0.314]),
            (0.38, [0.722, 0.745, 0.776]),
            (0.56, [0.980, 0.980, 0.980]),
            (0.66, [0.847, 0.753, 0.541]),
            (0.70, [0.941, 0.918, 0.847]),
            (0.80, [0.769, 0.792, 0.816]),
            (0.86, [0.290, 0.322, 0.361]),
            (0.90, [0.847, 0.863, 0.878]),
            (1.00, [0.957, 0.957, 0.949]),
        ],
    },
    // Granite: speckled pink feldspar, gray quartz and black mica.
    Palette {
        id: "granite",
        name: "Granite",
        stops: &[
            (0.00, [0.165, 0.157, 0.157]),
            (0.10, [0.722, 0.659, 0.627]),
            (0.16, [0.910, 0.784, 0.722]),
            (0.22, [0.290, 0.267, 0.275]),
            (0.30, [0.847, 0.816, 0.800]),
            (0.36, [0.659, 0.478, 0.416]),
            (0.44, [0.118, 0.110, 0.118]),
            (0.52, [0.784, 0.706, 0.667]),
            (0.60, [0.941, 0.902, 0.878]),
            (0.68, [0.353, 0.314, 0.322]),
            (0.78, [0.722, 0.541, 0.478]),
            (0.88, [0.541, 0.518, 0.525]),
            (1.00, [0.165, 0.157, 0.157]),
        ],
    },
    // Agate: a sliced nodule's fine bands of rust, orange and cream.
    Palette {
        id: "agate",
        name: "Agate",
        stops: &[
            (0.00, [0.416, 0.165, 0.071]),
            (0.10, [0.722, 0.329, 0.122]),
            (0.18, [0.941, 0.784, 0.604]),
            (0.24, [0.541, 0.227, 0.094]),
            (0.34, [0.910, 0.541, 0.227]),
            (0.42, [1.000, 0.945, 0.863]),
            (0.50, [0.659, 0.282, 0.110]),
            (0.62, [0.847, 0.627, 0.439]),
            (0.70, [0.353, 0.141, 0.063]),
            (0.82, [0.784, 0.416, 0.165]),
            (0.90, [0.953, 0.863, 0.753]),
            (1.00, [0.416, 0.165, 0.071]),
        ],
    },
    // Fluorite: rainbow fluorite's purple, clear, green and aqua zones.
    Palette {
        id: "fluorite",
        name: "Fluorite",
        stops: &[
            (0.00, [0.227, 0.102, 0.416]),
            (0.16, [0.478, 0.290, 0.784]),
            (0.30, [0.784, 0.690, 0.941]),
            (0.40, [0.910, 0.965, 0.941]),
            (0.50, [0.416, 0.847, 0.690]),
            (0.62, [0.165, 0.604, 0.478]),
            (0.72, [0.290, 0.753, 0.816]),
            (0.86, [0.416, 0.227, 0.690]),
            (1.00, [0.227, 0.102, 0.416]),
        ],
    },
    // Bismuth: a lab-grown hopper crystal's silver metal under its gold, magenta, blue and green
    // oxide.
    Palette {
        id: "bismuth",
        name: "Bismuth",
        stops: &[
            (0.00, [0.604, 0.627, 0.659]),
            (0.12, [0.878, 0.690, 0.188]),
            (0.24, [0.816, 0.251, 0.541]),
            (0.36, [0.290, 0.227, 0.722]),
            (0.48, [0.165, 0.541, 0.847]),
            (0.60, [0.227, 0.753, 0.627]),
            (0.72, [0.784, 0.816, 0.251]),
            (0.84, [0.878, 0.439, 0.188]),
            (1.00, [0.604, 0.627, 0.659]),
        ],
    },
    // Sulfur: bright yellow crystals on a gray volcanic crust.
    Palette {
        id: "sulfur",
        name: "Sulfur",
        stops: &[
            (0.00, [0.227, 0.227, 0.196]),
            (0.20, [0.541, 0.541, 0.416]),
            (0.35, [0.910, 0.816, 0.125]),
            (0.45, [1.000, 0.953, 0.416]),
            (0.52, [1.000, 0.984, 0.816]),
            (0.60, [0.941, 0.847, 0.102]),
            (0.72, [0.722, 0.627, 0.063]),
            (0.86, [0.353, 0.345, 0.282]),
            (1.00, [0.227, 0.227, 0.196]),
        ],
    },
    // Cinnabar: vermilion crystals in gray host rock, with a vein of white quartz.
    Palette {
        id: "cinnabar",
        name: "Cinnabar",
        stops: &[
            (0.00, [0.227, 0.204, 0.212]),
            (0.16, [0.478, 0.431, 0.439]),
            (0.30, [0.784, 0.157, 0.102]),
            (0.42, [0.910, 0.267, 0.165]),
            (0.50, [1.000, 0.478, 0.353]),
            (0.58, [0.690, 0.118, 0.078]),
            (0.72, [0.353, 0.078, 0.063]),
            (0.82, [0.863, 0.839, 0.824]),
            (0.90, [0.416, 0.376, 0.384]),
            (1.00, [0.227, 0.204, 0.212]),
        ],
    },
    // Pyrite: brassy fool's gold cubes on dark slate.
    Palette {
        id: "pyrite",
        name: "Pyrite",
        stops: &[
            (0.00, [0.133, 0.141, 0.165]),
            (0.18, [0.290, 0.290, 0.251]),
            (0.30, [0.541, 0.478, 0.227]),
            (0.40, [0.847, 0.753, 0.376]),
            (0.46, [1.000, 0.949, 0.659]),
            (0.52, [0.784, 0.659, 0.251]),
            (0.62, [0.416, 0.369, 0.188]),
            (0.78, [0.227, 0.235, 0.251]),
            (0.88, [0.604, 0.580, 0.533]),
            (1.00, [0.133, 0.141, 0.165]),
        ],
    },
    // Sunflower: golden petals around a dark seed head, and the leaves.
    Palette {
        id: "sunflower",
        name: "Sunflower",
        stops: &[
            (0.00, [0.165, 0.102, 0.031]),
            (0.14, [0.353, 0.227, 0.078]),
            (0.26, [0.941, 0.659, 0.063]),
            (0.38, [1.000, 0.824, 0.102]),
            (0.48, [1.000, 0.941, 0.478]),
            (0.56, [0.961, 0.722, 0.000]),
            (0.70, [0.290, 0.541, 0.165]),
            (0.84, [0.122, 0.290, 0.078]),
            (1.00, [0.165, 0.102, 0.031]),
        ],
    },
    // Lotus: pink and white petals with a golden seed pod, over a green pond.
    Palette {
        id: "lotus",
        name: "Lotus",
        stops: &[
            (0.00, [0.055, 0.227, 0.227]),
            (0.16, [0.227, 0.478, 0.353]),
            (0.30, [0.961, 0.627, 0.753]),
            (0.42, [0.992, 0.894, 0.933]),
            (0.50, [1.000, 1.000, 1.000]),
            (0.56, [0.941, 0.816, 0.376]),
            (0.66, [0.878, 0.478, 0.659]),
            (0.80, [0.102, 0.353, 0.353]),
            (1.00, [0.055, 0.227, 0.227]),
        ],
    },
    // Moss: a cushion of moss from deep green to sunlit chartreuse.
    Palette {
        id: "moss",
        name: "Moss",
        stops: &[
            (0.00, [0.102, 0.141, 0.063]),
            (0.20, [0.227, 0.353, 0.102]),
            (0.36, [0.435, 0.604, 0.165]),
            (0.48, [0.722, 0.847, 0.290]),
            (0.56, [0.878, 0.941, 0.541]),
            (0.64, [0.541, 0.690, 0.227]),
            (0.80, [0.165, 0.251, 0.078]),
            (1.00, [0.102, 0.141, 0.063]),
        ],
    },
    // Bamboo grove: pale green stalks with golden nodes, in morning mist.
    Palette {
        id: "bamboo",
        name: "Bamboo",
        stops: &[
            (0.00, [0.122, 0.227, 0.102]),
            (0.18, [0.290, 0.541, 0.227]),
            (0.32, [0.604, 0.784, 0.353]),
            (0.42, [0.878, 0.918, 0.627]),
            (0.48, [0.784, 0.659, 0.290]),
            (0.52, [0.541, 0.722, 0.290]),
            (0.64, [0.831, 0.878, 0.816]),
            (0.74, [0.184, 0.416, 0.165]),
            (0.86, [0.416, 0.604, 0.290]),
            (1.00, [0.122, 0.227, 0.102]),
        ],
    },
    // Maple: autumn leaves in crimson, orange and gold, with one still olive.
    Palette {
        id: "maple",
        name: "Maple",
        stops: &[
            (0.00, [0.290, 0.039, 0.031]),
            (0.18, [0.659, 0.094, 0.055]),
            (0.32, [0.910, 0.251, 0.102]),
            (0.44, [0.973, 0.541, 0.102]),
            (0.54, [1.000, 0.761, 0.227]),
            (0.62, [0.847, 0.416, 0.071]),
            (0.74, [0.541, 0.102, 0.063]),
            (0.86, [0.416, 0.416, 0.102]),
            (1.00, [0.290, 0.039, 0.031]),
        ],
    },
    // Ginkgo: fan-shaped leaves turned butter yellow, a last green one, and the gray branch.
    Palette {
        id: "ginkgo",
        name: "Ginkgo",
        stops: &[
            (0.00, [0.659, 0.722, 0.290]),
            (0.18, [0.910, 0.847, 0.290]),
            (0.34, [1.000, 0.910, 0.353]),
            (0.46, [1.000, 0.965, 0.722]),
            (0.56, [0.941, 0.816, 0.125]),
            (0.70, [0.847, 0.690, 0.094]),
            (0.84, [0.478, 0.604, 0.227]),
            (0.92, [0.416, 0.392, 0.345]),
            (1.00, [0.659, 0.722, 0.290]),
        ],
    },
    // Wisteria: hanging lilac-blue flowers fading to white, among green leaves.
    Palette {
        id: "wisteria",
        name: "Wisteria",
        stops: &[
            (0.00, [0.227, 0.165, 0.416]),
            (0.20, [0.478, 0.416, 0.784]),
            (0.34, [0.659, 0.627, 0.910]),
            (0.44, [0.847, 0.831, 0.973]),
            (0.52, [0.949, 0.941, 0.988]),
            (0.60, [0.722, 0.659, 0.910]),
            (0.72, [0.416, 0.353, 0.722]),
            (0.86, [0.353, 0.541, 0.290]),
            (1.00, [0.227, 0.165, 0.416]),
        ],
    },
    // Poppy: scarlet petals around a black center, a green stem and a summer sky.
    Palette {
        id: "poppy",
        name: "Poppy",
        stops: &[
            (0.00, [0.039, 0.039, 0.039]),
            (0.10, [0.227, 0.031, 0.024]),
            (0.22, [0.847, 0.125, 0.055]),
            (0.36, [1.000, 0.290, 0.102]),
            (0.46, [1.000, 0.541, 0.353]),
            (0.54, [0.910, 0.157, 0.055]),
            (0.66, [0.541, 0.063, 0.039]),
            (0.78, [0.290, 0.478, 0.165]),
            (0.90, [0.659, 0.784, 0.910]),
            (1.00, [0.039, 0.039, 0.039]),
        ],
    },
    // Iris: deep violet-blue petals with a yellow beard, and the sword leaves.
    Palette {
        id: "iris",
        name: "Iris",
        stops: &[
            (0.00, [0.102, 0.039, 0.290]),
            (0.18, [0.227, 0.102, 0.627]),
            (0.32, [0.416, 0.290, 0.847]),
            (0.42, [0.722, 0.659, 0.941]),
            (0.48, [0.941, 0.753, 0.125]),
            (0.52, [1.000, 0.941, 0.627]),
            (0.58, [0.541, 0.416, 0.878]),
            (0.74, [0.165, 0.078, 0.541]),
            (0.88, [0.227, 0.416, 0.227]),
            (1.00, [0.102, 0.039, 0.290]),
        ],
    },
    // Orchid: magenta and pink petals with a white throat.
    Palette {
        id: "orchid",
        name: "Orchid",
        stops: &[
            (0.00, [0.353, 0.039, 0.290]),
            (0.18, [0.659, 0.102, 0.541]),
            (0.32, [0.878, 0.314, 0.722]),
            (0.44, [0.973, 0.659, 0.878]),
            (0.52, [1.000, 0.941, 0.980]),
            (0.60, [0.910, 0.690, 0.847]),
            (0.70, [0.753, 0.188, 0.541]),
            (0.84, [0.478, 0.039, 0.353]),
            (1.00, [0.353, 0.039, 0.290]),
        ],
    },
    // Cactus flower: saguaro green and desert sand, with a hot pink bloom.
    Palette {
        id: "cactus-flower",
        name: "Cactus Flower",
        stops: &[
            (0.00, [0.165, 0.290, 0.165]),
            (0.20, [0.353, 0.541, 0.353]),
            (0.34, [0.604, 0.722, 0.604]),
            (0.44, [0.910, 0.847, 0.690]),
            (0.52, [1.000, 0.290, 0.541]),
            (0.58, [1.000, 0.690, 0.816]),
            (0.64, [0.878, 0.165, 0.416]),
            (0.76, [0.290, 0.478, 0.290]),
            (1.00, [0.165, 0.290, 0.165]),
        ],
    },
    // Succulent: an echeveria rosette's powdery blue-green, edged lilac and plum at the tips.
    Palette {
        id: "succulent",
        name: "Succulent",
        stops: &[
            (0.00, [0.227, 0.353, 0.353]),
            (0.20, [0.416, 0.604, 0.604]),
            (0.34, [0.659, 0.816, 0.784]),
            (0.46, [0.847, 0.925, 0.902]),
            (0.54, [0.784, 0.627, 0.816]),
            (0.60, [0.541, 0.353, 0.604]),
            (0.70, [0.541, 0.722, 0.690]),
            (0.86, [0.290, 0.478, 0.471]),
            (1.00, [0.227, 0.353, 0.353]),
        ],
    },
    // Tulip fields: rows of red, yellow, pink, purple and orange tulips between green.
    Palette {
        id: "tulip-fields",
        name: "Tulip Fields",
        stops: &[
            (0.0000, [0.910, 0.063, 0.165]),
            (0.1667, [0.910, 0.063, 0.165]),
            (0.1667, [0.227, 0.478, 0.165]),
            (0.2000, [0.227, 0.478, 0.165]),
            (0.2000, [1.000, 0.824, 0.102]),
            (0.3667, [1.000, 0.824, 0.102]),
            (0.3667, [0.227, 0.478, 0.165]),
            (0.4000, [0.227, 0.478, 0.165]),
            (0.4000, [1.000, 0.416, 0.690]),
            (0.5667, [1.000, 0.416, 0.690]),
            (0.5667, [0.227, 0.478, 0.165]),
            (0.6000, [0.227, 0.478, 0.165]),
            (0.6000, [0.478, 0.165, 0.722]),
            (0.7667, [0.478, 0.165, 0.722]),
            (0.7667, [0.227, 0.478, 0.165]),
            (0.8000, [0.227, 0.478, 0.165]),
            (0.8000, [1.000, 0.478, 0.102]),
            (0.9667, [1.000, 0.478, 0.102]),
            (0.9667, [0.227, 0.478, 0.165]),
            (1.0000, [0.227, 0.478, 0.165]),
        ],
    },
    // Redwood: red-brown bark, the dark canopy far above, and light through fog.
    Palette {
        id: "redwood",
        name: "Redwood",
        stops: &[
            (0.00, [0.165, 0.071, 0.039]),
            (0.20, [0.416, 0.165, 0.078]),
            (0.32, [0.659, 0.314, 0.165]),
            (0.42, [0.784, 0.471, 0.290]),
            (0.50, [0.227, 0.290, 0.165]),
            (0.62, [0.102, 0.227, 0.118]),
            (0.74, [0.416, 0.541, 0.416]),
            (0.82, [0.784, 0.816, 0.753]),
            (0.90, [0.290, 0.125, 0.063]),
            (1.00, [0.165, 0.071, 0.039]),
        ],
    },
    // Olive grove: silver-green leaves, ripe black olives and dry earth.
    Palette {
        id: "olive-grove",
        name: "Olive Grove",
        stops: &[
            (0.00, [0.227, 0.227, 0.102]),
            (0.20, [0.416, 0.478, 0.227]),
            (0.34, [0.659, 0.722, 0.541]),
            (0.44, [0.847, 0.863, 0.784]),
            (0.52, [0.541, 0.604, 0.416]),
            (0.64, [0.227, 0.165, 0.188]),
            (0.72, [0.416, 0.478, 0.165]),
            (0.84, [0.784, 0.690, 0.502]),
            (1.00, [0.227, 0.227, 0.102]),
        ],
    },
    // Fly agaric: the red cap and its white spots, the pale stalk and the forest floor.
    Palette {
        id: "fly-agaric",
        name: "Fly Agaric",
        stops: &[
            (0.00, [0.541, 0.039, 0.039]),
            (0.14, [0.847, 0.078, 0.055]),
            (0.20, [1.000, 0.227, 0.125]),
            (0.20, [1.000, 0.973, 0.933]),
            (0.25, [1.000, 0.973, 0.933]),
            (0.25, [0.910, 0.094, 0.055]),
            (0.38, [0.753, 0.063, 0.039]),
            (0.38, [1.000, 0.973, 0.933]),
            (0.42, [1.000, 0.973, 0.933]),
            (0.42, [0.847, 0.078, 0.055]),
            (0.56, [0.910, 0.125, 0.102]),
            (0.62, [0.941, 0.910, 0.847]),
            (0.70, [0.784, 0.722, 0.596]),
            (0.80, [0.227, 0.165, 0.102]),
            (0.90, [0.290, 0.102, 0.039]),
            (1.00, [0.541, 0.039, 0.039]),
        ],
    },
    // Chanterelle: apricot funnels with pale peach ridges, on a mossy forest floor.
    Palette {
        id: "chanterelle",
        name: "Chanterelle",
        stops: &[
            (0.00, [0.141, 0.125, 0.055]),
            (0.12, [0.290, 0.353, 0.133]),
            (0.22, [0.478, 0.541, 0.227]),
            (0.30, [0.847, 0.408, 0.118]),
            (0.40, [0.941, 0.541, 0.227]),
            (0.48, [0.973, 0.690, 0.478]),
            (0.54, [0.988, 0.878, 0.800]),
            (0.60, [0.941, 0.596, 0.282]),
            (0.70, [0.659, 0.314, 0.102]),
            (0.82, [0.227, 0.227, 0.094]),
            (1.00, [0.141, 0.125, 0.055]),
        ],
    },
    // Foxfire: the faint green glow of fungi on rotting wood in the dark.
    Palette {
        id: "foxfire",
        name: "Foxfire",
        stops: &[
            (0.00, [0.008, 0.024, 0.016]),
            (0.30, [0.039, 0.102, 0.063]),
            (0.44, [0.102, 0.353, 0.165]),
            (0.52, [0.416, 1.000, 0.541]),
            (0.56, [0.816, 1.000, 0.847]),
            (0.60, [0.290, 0.878, 0.439]),
            (0.70, [0.039, 0.227, 0.102]),
            (0.86, [0.020, 0.047, 0.031]),
            (1.00, [0.008, 0.024, 0.016]),
        ],
    },
    // Turkey tail: the bracket fungus's rings of brown, tan, cream and blue-gray.
    Palette {
        id: "turkey-tail",
        name: "Turkey Tail",
        stops: &[
            (0.00, [0.227, 0.141, 0.094]),
            (0.10, [0.478, 0.290, 0.165]),
            (0.20, [0.784, 0.627, 0.439]),
            (0.28, [0.941, 0.902, 0.816]),
            (0.36, [0.416, 0.439, 0.502]),
            (0.44, [0.165, 0.165, 0.196]),
            (0.54, [0.659, 0.416, 0.227]),
            (0.62, [0.910, 0.847, 0.722]),
            (0.70, [0.290, 0.353, 0.416]),
            (0.80, [0.541, 0.353, 0.227]),
            (0.90, [0.847, 0.784, 0.659]),
            (1.00, [0.227, 0.141, 0.094]),
        ],
    },
    // Lichen: gray-green crusts, orange and chartreuse patches, on gray rock.
    Palette {
        id: "lichen",
        name: "Lichen",
        stops: &[
            (0.00, [0.416, 0.439, 0.416]),
            (0.16, [0.604, 0.667, 0.541]),
            (0.28, [0.784, 0.831, 0.659]),
            (0.36, [0.910, 0.604, 0.102]),
            (0.42, [0.973, 0.753, 0.251]),
            (0.50, [0.541, 0.580, 0.541]),
            (0.62, [0.753, 0.847, 0.290]),
            (0.70, [0.659, 0.690, 0.596]),
            (0.84, [0.290, 0.306, 0.290]),
            (1.00, [0.416, 0.439, 0.416]),
        ],
    },
    // Peacock: the eye of a tail feather, its blues and greens, a bronze and gold ring and the dark
    // center.
    Palette {
        id: "peacock",
        name: "Peacock",
        stops: &[
            (0.00, [0.016, 0.125, 0.227]),
            (0.14, [0.039, 0.290, 0.541]),
            (0.26, [0.102, 0.541, 0.722]),
            (0.36, [0.165, 0.753, 0.627]),
            (0.44, [0.659, 0.784, 0.227]),
            (0.50, [0.847, 0.659, 0.227]),
            (0.54, [0.227, 0.102, 0.416]),
            (0.60, [0.102, 0.227, 0.627]),
            (0.70, [0.039, 0.416, 0.416]),
            (0.84, [0.165, 0.353, 0.102]),
            (1.00, [0.016, 0.125, 0.227]),
        ],
    },
    // Flamingo: pinks and coral, with the black of the wing tips.
    Palette {
        id: "flamingo",
        name: "Flamingo",
        stops: &[
            (0.00, [0.102, 0.078, 0.078]),
            (0.10, [0.910, 0.353, 0.478]),
            (0.26, [1.000, 0.541, 0.627]),
            (0.40, [1.000, 0.753, 0.784]),
            (0.50, [1.000, 0.941, 0.933]),
            (0.60, [1.000, 0.604, 0.541]),
            (0.74, [0.910, 0.314, 0.416]),
            (0.88, [0.416, 0.165, 0.227]),
            (1.00, [0.102, 0.078, 0.078]),
        ],
    },
    // Tiger: orange fur shading to cream, crossed by black stripes.
    Palette {
        id: "tiger",
        name: "Tiger",
        stops: &[
            (0.00, [0.847, 0.416, 0.063]),
            (0.18, [0.941, 0.565, 0.102]),
            (0.18, [0.055, 0.031, 0.024]),
            (0.24, [0.055, 0.031, 0.024]),
            (0.24, [0.961, 0.627, 0.227]),
            (0.42, [0.984, 0.847, 0.659]),
            (0.50, [1.000, 0.973, 0.933]),
            (0.58, [0.973, 0.753, 0.502]),
            (0.58, [0.055, 0.031, 0.024]),
            (0.64, [0.055, 0.031, 0.024]),
            (0.64, [0.910, 0.502, 0.102]),
            (0.82, [0.784, 0.353, 0.039]),
            (0.82, [0.055, 0.031, 0.024]),
            (0.86, [0.055, 0.031, 0.024]),
            (0.86, [0.847, 0.416, 0.063]),
            (1.00, [0.847, 0.416, 0.063]),
        ],
    },
    // Zebra: black and white stripes of uneven width.
    Palette {
        id: "zebra",
        name: "Zebra",
        stops: &[
            (0.0000, [0.957, 0.949, 0.933]),
            (0.1667, [0.957, 0.949, 0.933]),
            (0.1667, [0.063, 0.063, 0.063]),
            (0.2778, [0.063, 0.063, 0.063]),
            (0.2778, [0.957, 0.949, 0.933]),
            (0.3889, [0.957, 0.949, 0.933]),
            (0.3889, [0.063, 0.063, 0.063]),
            (0.5556, [0.063, 0.063, 0.063]),
            (0.5556, [0.957, 0.949, 0.933]),
            (0.6111, [0.957, 0.949, 0.933]),
            (0.6111, [0.063, 0.063, 0.063]),
            (0.6667, [0.063, 0.063, 0.063]),
            (0.6667, [0.957, 0.949, 0.933]),
            (0.8889, [0.957, 0.949, 0.933]),
            (0.8889, [0.063, 0.063, 0.063]),
            (1.0000, [0.063, 0.063, 0.063]),
        ],
    },
    // Poison dart frog: electric blue skin with black spots.
    Palette {
        id: "poison-dart-frog",
        name: "Poison Dart Frog",
        stops: &[
            (0.00, [0.055, 0.102, 0.627]),
            (0.20, [0.165, 0.416, 0.941]),
            (0.32, [0.416, 0.690, 1.000]),
            (0.40, [0.165, 0.416, 0.941]),
            (0.40, [0.020, 0.020, 0.039]),
            (0.48, [0.020, 0.020, 0.039]),
            (0.48, [0.102, 0.290, 0.847]),
            (0.66, [0.290, 0.604, 1.000]),
            (0.74, [0.102, 0.227, 0.753]),
            (0.74, [0.020, 0.020, 0.039]),
            (0.80, [0.020, 0.020, 0.039]),
            (0.80, [0.055, 0.102, 0.627]),
            (1.00, [0.055, 0.102, 0.627]),
        ],
    },
    // Monarch butterfly: orange wing cells, black veins and the white dots of the border.
    Palette {
        id: "monarch",
        name: "Monarch",
        stops: &[
            (0.00, [0.039, 0.031, 0.024]),
            (0.06, [0.039, 0.031, 0.024]),
            (0.06, [0.910, 0.455, 0.055]),
            (0.22, [1.000, 0.604, 0.165]),
            (0.30, [0.941, 0.478, 0.063]),
            (0.30, [0.039, 0.031, 0.024]),
            (0.34, [0.039, 0.031, 0.024]),
            (0.34, [0.961, 0.541, 0.102]),
            (0.50, [1.000, 0.690, 0.290]),
            (0.56, [0.910, 0.439, 0.102]),
            (0.56, [0.039, 0.031, 0.024]),
            (0.70, [0.039, 0.031, 0.024]),
            (0.70, [1.000, 0.980, 0.941]),
            (0.73, [1.000, 0.980, 0.941]),
            (0.73, [0.039, 0.031, 0.024]),
            (0.80, [0.039, 0.031, 0.024]),
            (0.80, [1.000, 0.980, 0.941]),
            (0.82, [1.000, 0.980, 0.941]),
            (0.82, [0.039, 0.031, 0.024]),
            (1.00, [0.039, 0.031, 0.024]),
        ],
    },
    // Blue morpho: the butterfly's iridescent electric blue, and its brown underside.
    Palette {
        id: "blue-morpho",
        name: "Blue Morpho",
        stops: &[
            (0.00, [0.020, 0.031, 0.055]),
            (0.14, [0.039, 0.165, 0.416]),
            (0.28, [0.102, 0.416, 0.910]),
            (0.38, [0.227, 0.690, 1.000]),
            (0.44, [0.659, 0.910, 1.000]),
            (0.50, [0.165, 0.541, 0.973]),
            (0.62, [0.039, 0.227, 0.659]),
            (0.76, [0.165, 0.102, 0.063]),
            (0.86, [0.416, 0.290, 0.165]),
            (1.00, [0.020, 0.031, 0.055]),
        ],
    },
    // Koi: a white carp with orange-red and black patches, in a dark green pond.
    Palette {
        id: "koi",
        name: "Koi",
        stops: &[
            (0.00, [1.000, 0.973, 0.941]),
            (0.18, [1.000, 0.973, 0.941]),
            (0.18, [1.000, 0.353, 0.102]),
            (0.38, [0.910, 0.251, 0.102]),
            (0.38, [1.000, 0.973, 0.941]),
            (0.50, [0.973, 0.941, 0.910]),
            (0.50, [0.039, 0.039, 0.055]),
            (0.58, [0.039, 0.039, 0.055]),
            (0.58, [1.000, 0.478, 0.165]),
            (0.74, [1.000, 0.353, 0.102]),
            (0.74, [0.102, 0.227, 0.290]),
            (0.90, [0.165, 0.353, 0.416]),
            (0.90, [1.000, 0.973, 0.941]),
            (1.00, [1.000, 0.973, 0.941]),
        ],
    },
    // Mandarin duck: orange sail feathers, white stripes, a purple breast, a green and teal crest
    // and chestnut flanks.
    Palette {
        id: "mandarin-duck",
        name: "Mandarin Duck",
        stops: &[
            (0.00, [0.910, 0.439, 0.102]),
            (0.12, [1.000, 0.627, 0.251]),
            (0.20, [1.000, 0.965, 0.910]),
            (0.26, [0.290, 0.102, 0.353]),
            (0.36, [0.165, 0.416, 0.541]),
            (0.44, [0.102, 0.541, 0.416]),
            (0.52, [1.000, 0.965, 0.910]),
            (0.58, [0.722, 0.329, 0.102]),
            (0.68, [0.847, 0.690, 0.439]),
            (0.80, [0.227, 0.227, 0.416]),
            (0.90, [0.784, 0.290, 0.102]),
            (1.00, [0.910, 0.439, 0.102]),
        ],
    },
    // Scarlet macaw: red, yellow, green and blue flight feathers.
    Palette {
        id: "scarlet-macaw",
        name: "Scarlet Macaw",
        stops: &[
            (0.00, [0.784, 0.063, 0.102]),
            (0.20, [1.000, 0.165, 0.102]),
            (0.30, [1.000, 0.784, 0.102]),
            (0.38, [1.000, 0.878, 0.376]),
            (0.44, [0.165, 0.659, 0.227]),
            (0.56, [0.102, 0.353, 0.847]),
            (0.66, [0.039, 0.165, 0.541]),
            (0.76, [0.227, 0.541, 1.000]),
            (0.88, [0.659, 0.039, 0.102]),
            (1.00, [0.784, 0.063, 0.102]),
        ],
    },
    // Toucan: black plumage and a white throat, the orange and yellow bill with its red tip, a blue
    // eye.
    Palette {
        id: "toucan",
        name: "Toucan",
        stops: &[
            (0.00, [0.039, 0.039, 0.039]),
            (0.20, [0.078, 0.078, 0.078]),
            (0.30, [1.000, 0.973, 0.910]),
            (0.38, [1.000, 0.824, 0.102]),
            (0.48, [1.000, 0.541, 0.063]),
            (0.56, [0.910, 0.188, 0.063]),
            (0.62, [0.227, 0.722, 0.847]),
            (0.68, [0.165, 0.604, 0.227]),
            (0.78, [0.039, 0.039, 0.039]),
            (1.00, [0.039, 0.039, 0.039]),
        ],
    },
    // Chameleon: greens turning turquoise, with yellow and orange bands.
    Palette {
        id: "chameleon",
        name: "Chameleon",
        stops: &[
            (0.00, [0.102, 0.416, 0.165]),
            (0.14, [0.290, 0.722, 0.227]),
            (0.26, [0.722, 0.910, 0.227]),
            (0.34, [1.000, 0.824, 0.227]),
            (0.42, [1.000, 0.541, 0.165]),
            (0.50, [0.165, 0.784, 0.722]),
            (0.62, [0.102, 0.541, 0.659]),
            (0.74, [0.227, 0.847, 0.416]),
            (0.86, [0.165, 0.541, 0.227]),
            (1.00, [0.102, 0.416, 0.165]),
        ],
    },
    // Jellyfish: a translucent pink and violet bell lit in the deep blue.
    Palette {
        id: "jellyfish",
        name: "Jellyfish",
        stops: &[
            (0.00, [0.008, 0.031, 0.094]),
            (0.20, [0.039, 0.102, 0.290]),
            (0.34, [0.227, 0.165, 0.541]),
            (0.44, [0.722, 0.416, 0.847]),
            (0.52, [1.000, 0.722, 0.941]),
            (0.56, [1.000, 0.941, 1.000]),
            (0.62, [0.878, 0.502, 0.816]),
            (0.74, [0.290, 0.165, 0.604]),
            (0.88, [0.024, 0.063, 0.220]),
            (1.00, [0.008, 0.031, 0.094]),
        ],
    },
    // Hummingbird: an iridescent emerald back and a ruby throat.
    Palette {
        id: "hummingbird",
        name: "Hummingbird",
        stops: &[
            (0.00, [0.039, 0.227, 0.102]),
            (0.16, [0.102, 0.541, 0.227]),
            (0.28, [0.416, 0.847, 0.290]),
            (0.36, [0.784, 0.941, 0.502]),
            (0.42, [0.102, 0.478, 0.290]),
            (0.50, [0.910, 0.039, 0.290]),
            (0.56, [1.000, 0.290, 0.478]),
            (0.62, [0.541, 0.039, 0.165]),
            (0.74, [0.784, 0.784, 0.753]),
            (0.86, [0.227, 0.353, 0.227]),
            (1.00, [0.039, 0.227, 0.102]),
        ],
    },
    // Snow leopard: smoky gray and cream fur with black rosettes.
    Palette {
        id: "snow-leopard",
        name: "Snow Leopard",
        stops: &[
            (0.00, [0.910, 0.894, 0.863]),
            (0.16, [0.784, 0.769, 0.737]),
            (0.24, [0.541, 0.525, 0.502]),
            (0.24, [0.094, 0.094, 0.102]),
            (0.30, [0.094, 0.094, 0.102]),
            (0.30, [0.659, 0.643, 0.612]),
            (0.48, [0.957, 0.941, 0.910]),
            (0.60, [0.847, 0.816, 0.769]),
            (0.60, [0.094, 0.094, 0.102]),
            (0.64, [0.094, 0.094, 0.102]),
            (0.64, [0.604, 0.588, 0.557]),
            (0.80, [0.784, 0.753, 0.706]),
            (0.88, [0.878, 0.847, 0.784]),
            (1.00, [0.910, 0.894, 0.863]),
        ],
    },
    // Clownfish: orange with white bands edged in black.
    Palette {
        id: "clownfish",
        name: "Clownfish",
        stops: &[
            (0.0000, [1.000, 0.416, 0.063]),
            (0.2941, [1.000, 0.416, 0.063]),
            (0.2941, [0.039, 0.039, 0.039]),
            (0.3235, [0.039, 0.039, 0.039]),
            (0.3235, [1.000, 0.980, 0.957]),
            (0.4706, [1.000, 0.980, 0.957]),
            (0.4706, [0.039, 0.039, 0.039]),
            (0.5000, [0.039, 0.039, 0.039]),
            (0.5000, [1.000, 0.478, 0.102]),
            (0.7941, [1.000, 0.478, 0.102]),
            (0.7941, [0.039, 0.039, 0.039]),
            (0.8235, [0.039, 0.039, 0.039]),
            (0.8235, [1.000, 0.980, 0.957]),
            (0.9706, [1.000, 0.980, 0.957]),
            (0.9706, [0.039, 0.039, 0.039]),
            (1.0000, [0.039, 0.039, 0.039]),
        ],
    },
    // Fireflies: yellow-green flashes over a meadow at twilight.
    Palette {
        id: "fireflies",
        name: "Fireflies",
        stops: &[
            (0.00, [0.020, 0.039, 0.118]),
            (0.25, [0.055, 0.102, 0.227]),
            (0.38, [0.102, 0.165, 0.165]),
            (0.46, [0.941, 1.000, 0.416]),
            (0.50, [1.000, 1.000, 0.816]),
            (0.54, [0.847, 0.910, 0.165]),
            (0.62, [0.078, 0.125, 0.094]),
            (0.80, [0.039, 0.071, 0.157]),
            (1.00, [0.020, 0.039, 0.118]),
        ],
    },
    // Kingfisher: a cobalt and turquoise back, an orange breast and a white throat.
    Palette {
        id: "kingfisher",
        name: "Kingfisher",
        stops: &[
            (0.00, [0.039, 0.165, 0.416]),
            (0.16, [0.102, 0.416, 0.847]),
            (0.28, [0.165, 0.753, 0.910]),
            (0.36, [0.541, 0.910, 1.000]),
            (0.42, [1.000, 0.541, 0.165]),
            (0.52, [0.910, 0.376, 0.102]),
            (0.60, [1.000, 0.957, 0.910]),
            (0.66, [0.784, 0.290, 0.063]),
            (0.78, [0.039, 0.290, 0.659]),
            (0.90, [0.024, 0.125, 0.314]),
            (1.00, [0.039, 0.165, 0.416]),
        ],
    },
    // Honeybee: gold and black bands, and a clear wing.
    Palette {
        id: "honeybee",
        name: "Honeybee",
        stops: &[
            (0.00, [0.973, 0.722, 0.102]),
            (0.20, [0.973, 0.722, 0.102]),
            (0.20, [0.102, 0.071, 0.031]),
            (0.30, [0.102, 0.071, 0.031]),
            (0.30, [1.000, 0.784, 0.227]),
            (0.50, [1.000, 0.784, 0.227]),
            (0.50, [0.102, 0.071, 0.031]),
            (0.60, [0.102, 0.071, 0.031]),
            (0.60, [0.910, 0.933, 0.957]),
            (0.70, [0.910, 0.933, 0.957]),
            (0.70, [0.847, 0.588, 0.102]),
            (0.90, [0.847, 0.588, 0.102]),
            (0.90, [0.102, 0.071, 0.031]),
            (1.00, [0.102, 0.071, 0.031]),
        ],
    },
    // Scarab: a beetle's metallic shell, green and gold turning copper, purple and blue.
    Palette {
        id: "scarab",
        name: "Scarab",
        stops: &[
            (0.00, [0.039, 0.102, 0.039]),
            (0.16, [0.102, 0.353, 0.165]),
            (0.28, [0.227, 0.659, 0.227]),
            (0.38, [0.659, 0.847, 0.227]),
            (0.44, [0.910, 0.784, 0.227]),
            (0.50, [0.784, 0.439, 0.165]),
            (0.58, [0.416, 0.165, 0.353]),
            (0.70, [0.102, 0.290, 0.416]),
            (0.84, [0.055, 0.227, 0.102]),
            (1.00, [0.039, 0.102, 0.039]),
        ],
    },
    // Coffee: espresso, crema and milk foam.
    Palette {
        id: "coffee",
        name: "Coffee",
        stops: &[
            (0.00, [0.102, 0.055, 0.031]),
            (0.16, [0.227, 0.125, 0.063]),
            (0.28, [0.416, 0.227, 0.102]),
            (0.38, [0.659, 0.439, 0.227]),
            (0.46, [0.847, 0.659, 0.408]),
            (0.54, [0.957, 0.902, 0.816]),
            (0.64, [1.000, 0.980, 0.949]),
            (0.72, [0.878, 0.784, 0.659]),
            (0.82, [0.541, 0.353, 0.196]),
            (0.92, [0.227, 0.125, 0.063]),
            (1.00, [0.102, 0.055, 0.031]),
        ],
    },
    // Chocolate: a bar's layers of dark, milk and white chocolate.
    Palette {
        id: "chocolate",
        name: "Chocolate",
        stops: &[
            (0.0000, [0.165, 0.078, 0.039]),
            (0.2667, [0.165, 0.078, 0.039]),
            (0.2667, [0.353, 0.180, 0.086]),
            (0.4667, [0.353, 0.180, 0.086]),
            (0.4667, [0.541, 0.306, 0.165]),
            (0.6667, [0.541, 0.306, 0.165]),
            (0.6667, [0.949, 0.894, 0.800]),
            (0.8000, [0.949, 0.894, 0.800]),
            (0.8000, [0.416, 0.227, 0.118]),
            (1.0000, [0.416, 0.227, 0.118]),
        ],
    },
    // Citrus: slices of lemon, lime, orange and pink grapefruit, and the white pith.
    Palette {
        id: "citrus",
        name: "Citrus",
        stops: &[
            (0.00, [1.000, 0.886, 0.227]),
            (0.14, [1.000, 0.965, 0.627]),
            (0.24, [0.659, 0.878, 0.227]),
            (0.34, [0.290, 0.722, 0.165]),
            (0.44, [1.000, 0.604, 0.102]),
            (0.54, [1.000, 0.753, 0.416]),
            (0.64, [1.000, 0.416, 0.353]),
            (0.74, [1.000, 0.690, 0.627]),
            (0.86, [1.000, 0.973, 0.878]),
            (1.00, [1.000, 0.886, 0.227]),
        ],
    },
    // Watermelon: dark and pale green rind, the white, the red flesh and a black seed.
    Palette {
        id: "watermelon",
        name: "Watermelon",
        stops: &[
            (0.00, [0.102, 0.290, 0.102]),
            (0.12, [0.227, 0.541, 0.165]),
            (0.20, [0.722, 0.878, 0.541]),
            (0.26, [0.973, 0.965, 0.878]),
            (0.34, [1.000, 0.541, 0.541]),
            (0.50, [1.000, 0.227, 0.290]),
            (0.60, [0.910, 0.125, 0.227]),
            (0.60, [0.078, 0.039, 0.039]),
            (0.63, [0.078, 0.039, 0.039]),
            (0.63, [0.941, 0.188, 0.247]),
            (0.80, [1.000, 0.353, 0.416]),
            (0.90, [0.165, 0.416, 0.165]),
            (1.00, [0.102, 0.290, 0.102]),
        ],
    },
    // Matcha: whisked green tea, its foam, and cream.
    Palette {
        id: "matcha",
        name: "Matcha",
        stops: &[
            (0.00, [0.165, 0.227, 0.063]),
            (0.20, [0.353, 0.478, 0.102]),
            (0.34, [0.541, 0.659, 0.227]),
            (0.46, [0.753, 0.831, 0.478]),
            (0.54, [0.910, 0.941, 0.784]),
            (0.62, [0.965, 0.941, 0.878]),
            (0.70, [0.659, 0.753, 0.353]),
            (0.84, [0.290, 0.376, 0.094]),
            (1.00, [0.165, 0.227, 0.063]),
        ],
    },
    // Sushi: nori, rice, salmon with its white fat line, tuna, pickled ginger and wasabi.
    Palette {
        id: "sushi",
        name: "Sushi",
        stops: &[
            (0.00, [0.078, 0.125, 0.059]),
            (0.15, [0.078, 0.125, 0.059]),
            (0.15, [0.980, 0.969, 0.941]),
            (0.30, [0.980, 0.969, 0.941]),
            (0.30, [1.000, 0.522, 0.322]),
            (0.45, [1.000, 0.522, 0.322]),
            (0.45, [1.000, 0.941, 0.902]),
            (0.50, [1.000, 0.941, 0.902]),
            (0.50, [0.769, 0.102, 0.173]),
            (0.65, [0.769, 0.102, 0.173]),
            (0.65, [0.961, 0.722, 0.769]),
            (0.75, [0.961, 0.722, 0.769]),
            (0.75, [0.651, 0.784, 0.251]),
            (0.85, [0.651, 0.784, 0.251]),
            (0.85, [0.980, 0.969, 0.941]),
            (1.00, [0.980, 0.969, 0.941]),
        ],
    },
    // Berries: blackberry, blueberry and its bloom, raspberry, cream and strawberry.
    Palette {
        id: "berries",
        name: "Berries",
        stops: &[
            (0.00, [0.102, 0.039, 0.165]),
            (0.14, [0.227, 0.102, 0.416]),
            (0.26, [0.353, 0.416, 0.784]),
            (0.36, [0.910, 0.125, 0.290]),
            (0.46, [1.000, 0.353, 0.478]),
            (0.54, [1.000, 0.941, 0.941]),
            (0.62, [1.000, 0.165, 0.227]),
            (0.72, [0.541, 0.039, 0.165]),
            (0.86, [0.165, 0.039, 0.227]),
            (1.00, [0.102, 0.039, 0.165]),
        ],
    },
    // Red wine: deep garnet to ruby, and the pale rim of a tilted glass.
    Palette {
        id: "red-wine",
        name: "Red Wine",
        stops: &[
            (0.00, [0.102, 0.016, 0.031]),
            (0.20, [0.290, 0.039, 0.102]),
            (0.36, [0.478, 0.063, 0.188]),
            (0.48, [0.659, 0.125, 0.290]),
            (0.56, [0.847, 0.376, 0.478]),
            (0.62, [0.541, 0.094, 0.220]),
            (0.76, [0.227, 0.024, 0.094]),
            (1.00, [0.102, 0.016, 0.031]),
        ],
    },
    // Honey: amber and gold, and the pale wax of the comb.
    Palette {
        id: "honey",
        name: "Honey",
        stops: &[
            (0.00, [0.227, 0.102, 0.016]),
            (0.20, [0.541, 0.290, 0.039]),
            (0.34, [0.816, 0.525, 0.102]),
            (0.46, [0.973, 0.722, 0.188]),
            (0.54, [1.000, 0.878, 0.541]),
            (0.60, [1.000, 0.957, 0.784]),
            (0.70, [0.941, 0.659, 0.125]),
            (0.84, [0.541, 0.290, 0.039]),
            (1.00, [0.227, 0.102, 0.016]),
        ],
    },
    // Peach: a red blush on velvet skin, golden flesh and the pit.
    Palette {
        id: "peach",
        name: "Peach",
        stops: &[
            (0.00, [0.784, 0.227, 0.165]),
            (0.16, [1.000, 0.478, 0.353]),
            (0.30, [1.000, 0.690, 0.541]),
            (0.42, [1.000, 0.847, 0.690]),
            (0.50, [1.000, 0.941, 0.847]),
            (0.58, [1.000, 0.816, 0.439]),
            (0.68, [0.973, 0.659, 0.227]),
            (0.78, [0.541, 0.227, 0.102]),
            (0.90, [0.910, 0.353, 0.227]),
            (1.00, [0.784, 0.227, 0.165]),
        ],
    },
    // Avocado: dark pebbled skin, green flesh shading to yellow, and the brown pit.
    Palette {
        id: "avocado",
        name: "Avocado",
        stops: &[
            (0.00, [0.102, 0.141, 0.063]),
            (0.14, [0.227, 0.290, 0.102]),
            (0.24, [0.416, 0.604, 0.165]),
            (0.36, [0.722, 0.847, 0.353]),
            (0.46, [0.941, 0.941, 0.627]),
            (0.54, [0.847, 0.878, 0.478]),
            (0.62, [0.541, 0.353, 0.165]),
            (0.70, [0.353, 0.204, 0.086]),
            (0.80, [0.541, 0.722, 0.227]),
            (0.90, [0.165, 0.227, 0.078]),
            (1.00, [0.102, 0.141, 0.063]),
        ],
    },
    // Pomegranate: leathery red rind, glinting jewel seeds and the white pith.
    Palette {
        id: "pomegranate",
        name: "Pomegranate",
        stops: &[
            (0.00, [0.290, 0.039, 0.063]),
            (0.16, [0.659, 0.078, 0.118]),
            (0.28, [0.910, 0.157, 0.227]),
            (0.36, [1.000, 0.416, 0.478]),
            (0.42, [1.000, 0.816, 0.831]),
            (0.48, [0.784, 0.063, 0.165]),
            (0.58, [0.973, 0.910, 0.816]),
            (0.66, [0.910, 0.784, 0.627]),
            (0.76, [0.722, 0.125, 0.165]),
            (0.88, [0.416, 0.039, 0.078]),
            (1.00, [0.290, 0.039, 0.063]),
        ],
    },
    // Neapolitan: strawberry, vanilla and chocolate ice cream.
    Palette {
        id: "neapolitan",
        name: "Neapolitan",
        stops: &[
            (0.0000, [0.969, 0.651, 0.706]),
            (0.3333, [0.969, 0.651, 0.706]),
            (0.3333, [0.984, 0.933, 0.839]),
            (0.6667, [0.984, 0.933, 0.839]),
            (0.6667, [0.416, 0.227, 0.133]),
            (1.0000, [0.416, 0.227, 0.133]),
        ],
    },
    // Pistachio: pale green nuts, the purple-red skin and the beige shell.
    Palette {
        id: "pistachio",
        name: "Pistachio",
        stops: &[
            (0.00, [0.847, 0.816, 0.690]),
            (0.16, [0.910, 0.878, 0.784]),
            (0.28, [0.722, 0.784, 0.439]),
            (0.40, [0.541, 0.659, 0.227]),
            (0.50, [0.784, 0.847, 0.541]),
            (0.58, [0.541, 0.227, 0.353]),
            (0.66, [0.416, 0.541, 0.165]),
            (0.78, [0.941, 0.918, 0.847]),
            (0.90, [0.753, 0.690, 0.565]),
            (1.00, [0.847, 0.816, 0.690]),
        ],
    },
    // Lemon meringue: toasted meringue peaks, lemon curd and the golden crust.
    Palette {
        id: "lemon-meringue",
        name: "Lemon Meringue",
        stops: &[
            (0.00, [1.000, 0.980, 0.941]),
            (0.18, [0.957, 0.886, 0.753]),
            (0.28, [0.847, 0.659, 0.408]),
            (0.34, [0.659, 0.408, 0.165]),
            (0.40, [0.973, 0.910, 0.784]),
            (0.50, [1.000, 0.941, 0.376]),
            (0.62, [1.000, 0.878, 0.165]),
            (0.72, [0.847, 0.690, 0.376]),
            (0.82, [0.722, 0.541, 0.290]),
            (0.92, [0.984, 0.957, 0.894]),
            (1.00, [1.000, 0.980, 0.941]),
        ],
    },
    // Dragon fruit: magenta skin with green scale tips, white flesh and black seeds.
    Palette {
        id: "dragon-fruit",
        name: "Dragon Fruit",
        stops: &[
            (0.00, [0.878, 0.094, 0.416]),
            (0.16, [1.000, 0.290, 0.604]),
            (0.24, [0.541, 0.784, 0.227]),
            (0.30, [0.878, 0.094, 0.416]),
            (0.36, [1.000, 0.973, 0.965]),
            (0.48, [0.957, 0.941, 0.933]),
            (0.48, [0.078, 0.063, 0.063]),
            (0.50, [0.078, 0.063, 0.063]),
            (0.50, [0.973, 0.957, 0.949]),
            (0.62, [1.000, 0.988, 0.980]),
            (0.62, [0.078, 0.063, 0.063]),
            (0.64, [0.078, 0.063, 0.063]),
            (0.64, [0.957, 0.941, 0.933]),
            (0.76, [1.000, 0.816, 0.894]),
            (0.86, [0.784, 0.063, 0.353]),
            (1.00, [0.878, 0.094, 0.416]),
        ],
    },
    // Kiwi: fuzzy brown skin, bright green flesh, the ring of black seeds and the pale core.
    Palette {
        id: "kiwi",
        name: "Kiwi",
        stops: &[
            (0.00, [0.416, 0.290, 0.165]),
            (0.14, [0.541, 0.416, 0.227]),
            (0.20, [0.353, 0.604, 0.102]),
            (0.36, [0.541, 0.816, 0.165]),
            (0.46, [0.690, 0.878, 0.290]),
            (0.46, [0.078, 0.063, 0.031]),
            (0.50, [0.078, 0.063, 0.031]),
            (0.50, [0.784, 0.910, 0.541]),
            (0.62, [0.957, 0.973, 0.863]),
            (0.74, [0.659, 0.847, 0.227]),
            (0.86, [0.290, 0.541, 0.102]),
            (1.00, [0.416, 0.290, 0.165]),
        ],
    },
    // Fig: purple-black skin, green at the stem, and the pink-red flesh.
    Palette {
        id: "fig",
        name: "Fig",
        stops: &[
            (0.00, [0.165, 0.039, 0.165]),
            (0.16, [0.290, 0.102, 0.290]),
            (0.26, [0.416, 0.227, 0.353]),
            (0.34, [0.541, 0.541, 0.290]),
            (0.42, [0.941, 0.847, 0.690]),
            (0.52, [0.910, 0.376, 0.478]),
            (0.60, [0.784, 0.165, 0.290]),
            (0.68, [0.541, 0.102, 0.165]),
            (0.78, [0.941, 0.722, 0.627]),
            (0.88, [0.227, 0.063, 0.188]),
            (1.00, [0.165, 0.039, 0.165]),
        ],
    },
    // Blood orange: crimson flesh, orange segments and the rind.
    Palette {
        id: "blood-orange",
        name: "Blood Orange",
        stops: &[
            (0.00, [0.353, 0.039, 0.039]),
            (0.20, [0.659, 0.078, 0.102]),
            (0.32, [0.847, 0.227, 0.102]),
            (0.42, [1.000, 0.478, 0.102]),
            (0.50, [1.000, 0.667, 0.227]),
            (0.56, [0.910, 0.353, 0.102]),
            (0.66, [0.541, 0.039, 0.102]),
            (0.78, [1.000, 0.541, 0.165]),
            (0.88, [0.784, 0.290, 0.063]),
            (1.00, [0.353, 0.039, 0.039]),
        ],
    },
    // Bubble tea: milk tea, taro and strawberry, with dark tapioca pearls.
    Palette {
        id: "bubble-tea",
        name: "Bubble Tea",
        stops: &[
            (0.00, [0.102, 0.055, 0.039]),
            (0.10, [0.227, 0.141, 0.094]),
            (0.20, [0.784, 0.627, 0.478]),
            (0.32, [0.910, 0.816, 0.690]),
            (0.42, [0.722, 0.604, 0.847]),
            (0.52, [0.847, 0.784, 0.941]),
            (0.60, [0.957, 0.925, 0.894]),
            (0.70, [0.941, 0.659, 0.722]),
            (0.82, [0.541, 0.416, 0.290]),
            (1.00, [0.102, 0.055, 0.039]),
        ],
    },
    // Botticelli: The Birth of Venus, its sea green, pale skin, roses, copper-gold hair and sky.
    Palette {
        id: "botticelli",
        name: "Botticelli",
        stops: &[
            (0.00, [0.165, 0.353, 0.314]),
            (0.16, [0.416, 0.604, 0.533]),
            (0.28, [0.722, 0.816, 0.753]),
            (0.38, [0.957, 0.886, 0.816]),
            (0.46, [0.910, 0.722, 0.659]),
            (0.54, [0.784, 0.478, 0.290]),
            (0.60, [0.847, 0.690, 0.376]),
            (0.70, [0.659, 0.753, 0.784]),
            (0.84, [0.290, 0.478, 0.439]),
            (1.00, [0.165, 0.353, 0.314]),
        ],
    },
    // Rembrandt: light out of darkness, umber and ochre to a golden highlight, and a deep red.
    Palette {
        id: "rembrandt",
        name: "Rembrandt",
        stops: &[
            (0.00, [0.039, 0.024, 0.016]),
            (0.20, [0.165, 0.102, 0.047]),
            (0.34, [0.353, 0.227, 0.094]),
            (0.44, [0.659, 0.455, 0.165]),
            (0.50, [0.910, 0.753, 0.439]),
            (0.54, [1.000, 0.941, 0.784]),
            (0.58, [0.784, 0.565, 0.251]),
            (0.66, [0.478, 0.165, 0.078]),
            (0.78, [0.227, 0.125, 0.063]),
            (1.00, [0.039, 0.024, 0.016]),
        ],
    },
    // Vermeer: ultramarine and lead-tin yellow against a dark wall, and a pearl.
    Palette {
        id: "vermeer",
        name: "Vermeer",
        stops: &[
            (0.00, [0.055, 0.078, 0.063]),
            (0.20, [0.102, 0.165, 0.227]),
            (0.32, [0.118, 0.290, 0.659]),
            (0.42, [0.290, 0.478, 0.847]),
            (0.50, [0.941, 0.847, 0.439]),
            (0.58, [0.973, 0.910, 0.659]),
            (0.64, [0.910, 0.894, 0.863]),
            (0.72, [0.659, 0.478, 0.290]),
            (0.86, [0.165, 0.165, 0.118]),
            (1.00, [0.055, 0.078, 0.063]),
        ],
    },
    // Hokusai: The Great Wave, Prussian blue, white foam, a beige sky and the boats.
    Palette {
        id: "hokusai",
        name: "Hokusai",
        stops: &[
            (0.00, [0.039, 0.118, 0.227]),
            (0.18, [0.102, 0.227, 0.416]),
            (0.30, [0.227, 0.416, 0.596]),
            (0.40, [0.541, 0.690, 0.784]),
            (0.48, [0.941, 0.925, 0.878]),
            (0.56, [0.878, 0.831, 0.706]),
            (0.66, [0.784, 0.659, 0.471]),
            (0.74, [0.353, 0.478, 0.596]),
            (0.86, [0.063, 0.165, 0.314]),
            (1.00, [0.039, 0.118, 0.227]),
        ],
    },
    // Hiroshige: woodblock skies shading from indigo to white, a pink and red sunset and green
    // hills.
    Palette {
        id: "hiroshige",
        name: "Hiroshige",
        stops: &[
            (0.00, [0.039, 0.078, 0.251]),
            (0.16, [0.102, 0.227, 0.541]),
            (0.28, [0.353, 0.541, 0.816]),
            (0.36, [0.784, 0.863, 0.941]),
            (0.42, [0.973, 0.910, 0.878]),
            (0.48, [0.957, 0.659, 0.565]),
            (0.54, [0.847, 0.227, 0.165]),
            (0.60, [0.941, 0.753, 0.376]),
            (0.68, [0.227, 0.478, 0.290]),
            (0.78, [0.102, 0.290, 0.227]),
            (0.88, [0.102, 0.165, 0.416]),
            (1.00, [0.039, 0.078, 0.251]),
        ],
    },
    // Wang Ximeng: A Thousand Li of Rivers and Mountains, azurite blue and malachite green on aged
    // silk.
    Palette {
        id: "wang-ximeng",
        name: "Wang Ximeng",
        stops: &[
            (0.00, [0.784, 0.659, 0.471]),
            (0.14, [0.847, 0.753, 0.596]),
            (0.26, [0.227, 0.541, 0.541]),
            (0.36, [0.102, 0.416, 0.659]),
            (0.44, [0.165, 0.290, 0.541]),
            (0.52, [0.227, 0.659, 0.541]),
            (0.62, [0.102, 0.478, 0.353]),
            (0.72, [0.541, 0.722, 0.627]),
            (0.84, [0.690, 0.541, 0.353]),
            (1.00, [0.784, 0.659, 0.471]),
        ],
    },
    // Behzad: a Persian miniature's lapis, gold, vermilion, turquoise, rose and green on paper.
    Palette {
        id: "behzad",
        name: "Behzad",
        stops: &[
            (0.00, [0.102, 0.165, 0.541]),
            (0.14, [0.847, 0.659, 0.188]),
            (0.24, [0.910, 0.251, 0.165]),
            (0.34, [0.227, 0.722, 0.690]),
            (0.44, [0.941, 0.659, 0.722]),
            (0.54, [0.227, 0.541, 0.227]),
            (0.64, [0.957, 0.925, 0.847]),
            (0.74, [0.541, 0.165, 0.416]),
            (0.86, [0.910, 0.784, 0.376]),
            (1.00, [0.102, 0.165, 0.541]),
        ],
    },
    // Turner: luminous golden haze, cream and pale blue, with a touch of rust.
    Palette {
        id: "turner",
        name: "Turner",
        stops: &[
            (0.00, [0.973, 0.910, 0.690]),
            (0.16, [1.000, 0.965, 0.847]),
            (0.28, [0.941, 0.784, 0.408]),
            (0.40, [0.910, 0.565, 0.227]),
            (0.48, [0.722, 0.329, 0.165]),
            (0.56, [0.847, 0.627, 0.376]),
            (0.66, [0.784, 0.831, 0.816]),
            (0.78, [0.957, 0.925, 0.816]),
            (0.90, [0.973, 0.847, 0.533]),
            (1.00, [0.973, 0.910, 0.690]),
        ],
    },
    // Monet: the water lilies, lilac and blue-violet water, green pads and pink flowers.
    Palette {
        id: "monet",
        name: "Monet",
        stops: &[
            (0.00, [0.290, 0.353, 0.541]),
            (0.14, [0.478, 0.541, 0.784]),
            (0.26, [0.722, 0.690, 0.878]),
            (0.36, [0.416, 0.604, 0.416]),
            (0.44, [0.659, 0.784, 0.541]),
            (0.52, [0.941, 0.753, 0.816]),
            (0.58, [1.000, 0.941, 0.941]),
            (0.66, [0.541, 0.690, 0.784]),
            (0.78, [0.353, 0.416, 0.659]),
            (0.90, [0.227, 0.353, 0.416]),
            (1.00, [0.290, 0.353, 0.541]),
        ],
    },
    // Van Gogh: The Starry Night, swirling blues, a yellow star and the dark cypress.
    Palette {
        id: "van-gogh",
        name: "Van Gogh",
        stops: &[
            (0.00, [0.039, 0.102, 0.290]),
            (0.14, [0.102, 0.227, 0.541]),
            (0.24, [0.227, 0.416, 0.784]),
            (0.32, [0.541, 0.753, 0.878]),
            (0.38, [0.941, 0.878, 0.314]),
            (0.42, [1.000, 0.973, 0.690]),
            (0.46, [0.910, 0.753, 0.125]),
            (0.54, [0.165, 0.353, 0.690]),
            (0.66, [0.063, 0.157, 0.353]),
            (0.78, [0.102, 0.227, 0.102]),
            (0.90, [0.039, 0.078, 0.251]),
            (1.00, [0.039, 0.102, 0.290]),
        ],
    },
    // Seurat: pointillist dots of complementary color, greens and oranges, blues and lilac.
    Palette {
        id: "seurat",
        name: "Seurat",
        stops: &[
            (0.00, [0.290, 0.541, 0.227]),
            (0.10, [0.910, 0.627, 0.314]),
            (0.20, [0.659, 0.784, 0.439]),
            (0.30, [0.416, 0.541, 0.784]),
            (0.40, [0.957, 0.910, 0.784]),
            (0.50, [0.784, 0.471, 0.227]),
            (0.60, [0.541, 0.722, 0.627]),
            (0.70, [0.722, 0.604, 0.816]),
            (0.80, [0.910, 0.784, 0.502]),
            (0.90, [0.227, 0.416, 0.353]),
            (1.00, [0.290, 0.541, 0.227]),
        ],
    },
    // Gauguin: Tahiti's deep greens, yellow and orange, pink and violet.
    Palette {
        id: "gauguin",
        name: "Gauguin",
        stops: &[
            (0.00, [0.165, 0.290, 0.165]),
            (0.14, [0.227, 0.478, 0.227]),
            (0.26, [0.910, 0.627, 0.125]),
            (0.36, [0.941, 0.784, 0.376]),
            (0.44, [0.878, 0.290, 0.478]),
            (0.52, [0.541, 0.165, 0.416]),
            (0.62, [0.784, 0.416, 0.165]),
            (0.72, [0.957, 0.847, 0.627]),
            (0.84, [0.102, 0.353, 0.290]),
            (1.00, [0.165, 0.290, 0.165]),
        ],
    },
    // Rousseau: a painted jungle, layer on layer of green, with a red flower and an orange fruit.
    Palette {
        id: "rousseau",
        name: "Rousseau",
        stops: &[
            (0.00, [0.039, 0.102, 0.039]),
            (0.14, [0.102, 0.290, 0.102]),
            (0.26, [0.227, 0.478, 0.165]),
            (0.36, [0.478, 0.722, 0.227]),
            (0.44, [0.784, 0.878, 0.478]),
            (0.50, [0.910, 0.165, 0.165]),
            (0.56, [1.000, 0.541, 0.165]),
            (0.62, [0.165, 0.416, 0.227]),
            (0.74, [0.055, 0.227, 0.102]),
            (0.86, [0.290, 0.541, 0.353]),
            (1.00, [0.039, 0.102, 0.039]),
        ],
    },
    // Klimt: The Kiss, gold leaf and ochre, black and white rectangles, and flowers.
    Palette {
        id: "klimt",
        name: "Klimt",
        stops: &[
            (0.00, [0.227, 0.165, 0.039]),
            (0.14, [0.659, 0.471, 0.102]),
            (0.24, [0.910, 0.753, 0.251]),
            (0.32, [1.000, 0.941, 0.627]),
            (0.38, [0.847, 0.627, 0.125]),
            (0.44, [0.078, 0.063, 0.063]),
            (0.48, [0.957, 0.941, 0.910]),
            (0.52, [0.784, 0.627, 0.188]),
            (0.60, [0.910, 0.416, 0.541]),
            (0.66, [0.416, 0.541, 0.816]),
            (0.72, [0.910, 0.753, 0.251]),
            (0.86, [0.541, 0.353, 0.078]),
            (1.00, [0.227, 0.165, 0.039]),
        ],
    },
    // Munch: The Scream, a blood-orange sky over the deep blue fjord.
    Palette {
        id: "munch",
        name: "Munch",
        stops: &[
            (0.00, [0.910, 0.353, 0.102]),
            (0.14, [1.000, 0.604, 0.227]),
            (0.24, [0.941, 0.753, 0.376]),
            (0.32, [0.784, 0.227, 0.102]),
            (0.42, [0.102, 0.227, 0.541]),
            (0.52, [0.165, 0.353, 0.604]),
            (0.60, [0.416, 0.478, 0.541]),
            (0.70, [0.784, 0.541, 0.227]),
            (0.80, [0.541, 0.227, 0.102]),
            (0.90, [1.000, 0.478, 0.165]),
            (1.00, [0.910, 0.353, 0.102]),
        ],
    },
    // Mucha: Art Nouveau posters, soft gold, sage, muted teal, dusty rose and cream.
    Palette {
        id: "mucha",
        name: "Mucha",
        stops: &[
            (0.00, [0.910, 0.863, 0.753]),
            (0.16, [0.847, 0.722, 0.471]),
            (0.28, [0.659, 0.690, 0.541]),
            (0.38, [0.416, 0.541, 0.478]),
            (0.48, [0.910, 0.753, 0.690]),
            (0.56, [0.784, 0.541, 0.478]),
            (0.66, [0.957, 0.925, 0.847]),
            (0.76, [0.722, 0.627, 0.439]),
            (0.88, [0.541, 0.604, 0.478]),
            (1.00, [0.910, 0.863, 0.753]),
        ],
    },
    // Kandinsky: bright yellow, red and blue, black lines, and pink, green and violet on cream.
    Palette {
        id: "kandinsky",
        name: "Kandinsky",
        stops: &[
            (0.00, [0.957, 0.925, 0.847]),
            (0.10, [1.000, 0.824, 0.102]),
            (0.20, [0.910, 0.157, 0.102]),
            (0.28, [0.078, 0.078, 0.078]),
            (0.34, [0.102, 0.290, 0.847]),
            (0.44, [0.541, 0.784, 0.910]),
            (0.52, [0.941, 0.541, 0.753]),
            (0.60, [0.165, 0.541, 0.290]),
            (0.68, [1.000, 0.541, 0.102]),
            (0.76, [0.078, 0.078, 0.078]),
            (0.82, [0.416, 0.165, 0.604]),
            (0.92, [0.973, 0.910, 0.627]),
            (1.00, [0.957, 0.925, 0.847]),
        ],
    },
    // Hilma af Klint: The Ten Largest, pink, peach, mustard, lavender and blue.
    Palette {
        id: "hilma-af-klint",
        name: "Hilma af Klint",
        stops: &[
            (0.00, [0.941, 0.659, 0.690]),
            (0.14, [0.973, 0.784, 0.627]),
            (0.26, [0.910, 0.690, 0.188]),
            (0.36, [0.957, 0.910, 0.816]),
            (0.46, [0.722, 0.627, 0.847]),
            (0.56, [0.416, 0.541, 0.784]),
            (0.66, [0.910, 0.478, 0.353]),
            (0.76, [0.973, 0.816, 0.847]),
            (0.88, [0.847, 0.784, 0.408]),
            (1.00, [0.941, 0.659, 0.690]),
        ],
    },
    // Amrita Sher-Gil: earthy ochre, terracotta, olive and deep red.
    Palette {
        id: "amrita-sher-gil",
        name: "Amrita Sher-Gil",
        stops: &[
            (0.00, [0.353, 0.165, 0.078]),
            (0.16, [0.659, 0.353, 0.165]),
            (0.28, [0.847, 0.627, 0.314]),
            (0.38, [0.541, 0.541, 0.227]),
            (0.48, [0.784, 0.188, 0.102]),
            (0.56, [0.478, 0.102, 0.063]),
            (0.66, [0.910, 0.784, 0.565]),
            (0.76, [0.416, 0.290, 0.165]),
            (0.88, [0.722, 0.439, 0.165]),
            (1.00, [0.353, 0.165, 0.078]),
        ],
    },
    // Torres-García: a constructivist grid of ochre, brick red, cream and slate in black lines.
    Palette {
        id: "torres-garcia",
        name: "Torres-García",
        stops: &[
            (0.0000, [0.784, 0.627, 0.376]),
            (0.1875, [0.784, 0.627, 0.376]),
            (0.1875, [0.102, 0.086, 0.071]),
            (0.2500, [0.102, 0.086, 0.071]),
            (0.2500, [0.659, 0.251, 0.165]),
            (0.4375, [0.659, 0.251, 0.165]),
            (0.4375, [0.102, 0.086, 0.071]),
            (0.5000, [0.102, 0.086, 0.071]),
            (0.5000, [0.910, 0.863, 0.769]),
            (0.6875, [0.910, 0.863, 0.769]),
            (0.6875, [0.102, 0.086, 0.071]),
            (0.7500, [0.102, 0.086, 0.071]),
            (0.7500, [0.290, 0.353, 0.416]),
            (0.9375, [0.290, 0.353, 0.416]),
            (0.9375, [0.102, 0.086, 0.071]),
            (1.0000, [0.102, 0.086, 0.071]),
        ],
    },
    // Rainforest: layered canopy greens, deep shade and shafts of misty light.
    Palette {
        id: "rainforest",
        name: "Rainforest",
        stops: &[
            (0.00, [0.016, 0.078, 0.039]),
            (0.16, [0.055, 0.227, 0.102]),
            (0.30, [0.118, 0.416, 0.165]),
            (0.42, [0.290, 0.659, 0.227]),
            (0.50, [0.659, 0.847, 0.416]),
            (0.56, [0.847, 0.933, 0.753]),
            (0.62, [0.227, 0.541, 0.227]),
            (0.76, [0.039, 0.165, 0.078]),
            (0.90, [0.102, 0.290, 0.165]),
            (1.00, [0.016, 0.078, 0.039]),
        ],
    },
    // Cloud forest: blue-green in fog, moss, and a pink bromeliad.
    Palette {
        id: "cloud-forest",
        name: "Cloud Forest",
        stops: &[
            (0.00, [0.165, 0.290, 0.227]),
            (0.20, [0.353, 0.478, 0.416]),
            (0.34, [0.604, 0.722, 0.659]),
            (0.46, [0.863, 0.910, 0.886]),
            (0.54, [0.957, 0.965, 0.957]),
            (0.62, [0.659, 0.753, 0.690]),
            (0.72, [0.416, 0.541, 0.353]),
            (0.80, [0.847, 0.541, 0.659]),
            (0.90, [0.227, 0.353, 0.290]),
            (1.00, [0.165, 0.290, 0.227]),
        ],
    },
    // Mangrove: brackish water, green leaves, tangled dark roots and the sky in the water.
    Palette {
        id: "mangrove",
        name: "Mangrove",
        stops: &[
            (0.00, [0.165, 0.165, 0.078]),
            (0.16, [0.353, 0.353, 0.165]),
            (0.28, [0.541, 0.541, 0.290]),
            (0.38, [0.227, 0.353, 0.165]),
            (0.48, [0.416, 0.604, 0.290]),
            (0.56, [0.784, 0.847, 0.659]),
            (0.64, [0.290, 0.227, 0.141]),
            (0.74, [0.478, 0.604, 0.627]),
            (0.86, [0.227, 0.290, 0.165]),
            (1.00, [0.165, 0.165, 0.078]),
        ],
    },
    // Savanna: golden grass, dark acacias, red earth and the sunset.
    Palette {
        id: "savanna",
        name: "Savanna",
        stops: &[
            (0.00, [0.910, 0.753, 0.376]),
            (0.16, [0.957, 0.847, 0.533]),
            (0.28, [0.784, 0.596, 0.220]),
            (0.38, [0.541, 0.416, 0.165]),
            (0.46, [0.227, 0.290, 0.102]),
            (0.54, [0.416, 0.478, 0.165]),
            (0.62, [0.910, 0.541, 0.165]),
            (0.72, [0.784, 0.314, 0.102]),
            (0.84, [0.847, 0.659, 0.314]),
            (1.00, [0.910, 0.753, 0.376]),
        ],
    },
    // Prairie: tallgrass green and gold, purple coneflowers and a big sky.
    Palette {
        id: "prairie",
        name: "Prairie",
        stops: &[
            (0.00, [0.353, 0.478, 0.165]),
            (0.16, [0.659, 0.722, 0.353]),
            (0.28, [0.847, 0.784, 0.471]),
            (0.38, [0.541, 0.416, 0.604]),
            (0.46, [0.784, 0.541, 0.722]),
            (0.54, [0.541, 0.722, 0.910]),
            (0.64, [0.847, 0.910, 0.957]),
            (0.74, [0.722, 0.659, 0.345]),
            (0.86, [0.478, 0.541, 0.227]),
            (1.00, [0.353, 0.478, 0.165]),
        ],
    },
    // Taiga: dark spruce, snow, a gray sky and blue shadows.
    Palette {
        id: "taiga",
        name: "Taiga",
        stops: &[
            (0.00, [0.055, 0.118, 0.086]),
            (0.18, [0.102, 0.227, 0.165]),
            (0.30, [0.180, 0.353, 0.259]),
            (0.40, [0.416, 0.541, 0.478]),
            (0.48, [0.784, 0.831, 0.847]),
            (0.54, [0.957, 0.965, 0.973]),
            (0.62, [0.659, 0.722, 0.784]),
            (0.72, [0.227, 0.353, 0.290]),
            (0.86, [0.078, 0.157, 0.118]),
            (1.00, [0.055, 0.118, 0.086]),
        ],
    },
    // Tundra in autumn: red bearberry, ochre and olive lichens, and a cold gray sky.
    Palette {
        id: "tundra",
        name: "Tundra",
        stops: &[
            (0.00, [0.541, 0.227, 0.102]),
            (0.16, [0.784, 0.392, 0.165]),
            (0.26, [0.847, 0.627, 0.314]),
            (0.36, [0.659, 0.627, 0.376]),
            (0.46, [0.416, 0.478, 0.290]),
            (0.54, [0.722, 0.753, 0.690]),
            (0.62, [0.863, 0.878, 0.894]),
            (0.70, [0.541, 0.604, 0.659]),
            (0.80, [0.353, 0.290, 0.227]),
            (0.90, [0.659, 0.290, 0.141]),
            (1.00, [0.541, 0.227, 0.102]),
        ],
    },
    // Glacier: crevasse blue, white ice, turquoise meltwater and gray moraine.
    Palette {
        id: "glacier",
        name: "Glacier",
        stops: &[
            (0.00, [0.039, 0.227, 0.416]),
            (0.16, [0.102, 0.416, 0.659]),
            (0.28, [0.290, 0.659, 0.847]),
            (0.38, [0.659, 0.863, 0.941]),
            (0.46, [0.933, 0.973, 0.988]),
            (0.54, [1.000, 1.000, 1.000]),
            (0.62, [0.784, 0.847, 0.878]),
            (0.70, [0.478, 0.541, 0.580]),
            (0.76, [0.227, 0.722, 0.753]),
            (0.86, [0.102, 0.416, 0.659]),
            (1.00, [0.039, 0.227, 0.416]),
        ],
    },
    // Alpine: a flowering meadow, gray rock, snow and a deep blue sky.
    Palette {
        id: "alpine",
        name: "Alpine",
        stops: &[
            (0.00, [0.227, 0.416, 0.165]),
            (0.14, [0.478, 0.722, 0.290]),
            (0.22, [0.941, 0.878, 0.290]),
            (0.30, [0.847, 0.290, 0.541]),
            (0.38, [0.541, 0.541, 0.541]),
            (0.48, [0.722, 0.722, 0.706]),
            (0.56, [0.973, 0.973, 0.980]),
            (0.66, [0.416, 0.627, 0.878]),
            (0.78, [0.165, 0.353, 0.659]),
            (0.90, [0.290, 0.478, 0.227]),
            (1.00, [0.227, 0.416, 0.165]),
        ],
    },
    // Heath: moorland heather, rust bracken, peat and a gray sky.
    Palette {
        id: "heath",
        name: "Heath",
        stops: &[
            (0.00, [0.227, 0.165, 0.165]),
            (0.16, [0.416, 0.227, 0.416]),
            (0.28, [0.659, 0.353, 0.659]),
            (0.38, [0.847, 0.627, 0.816]),
            (0.46, [0.722, 0.722, 0.753]),
            (0.56, [0.659, 0.392, 0.165]),
            (0.66, [0.416, 0.290, 0.165]),
            (0.76, [0.541, 0.478, 0.353]),
            (0.88, [0.353, 0.227, 0.353]),
            (1.00, [0.227, 0.165, 0.165]),
        ],
    },
    // Chaparral: sage scrub, dry golden grass, terracotta soil and a hard blue sky.
    Palette {
        id: "chaparral",
        name: "Chaparral",
        stops: &[
            (0.00, [0.541, 0.604, 0.478]),
            (0.16, [0.722, 0.753, 0.627]),
            (0.28, [0.847, 0.784, 0.565]),
            (0.38, [0.784, 0.627, 0.314]),
            (0.48, [0.722, 0.376, 0.227]),
            (0.56, [0.541, 0.290, 0.165]),
            (0.64, [0.416, 0.478, 0.353]),
            (0.74, [0.659, 0.784, 0.878]),
            (0.86, [0.659, 0.690, 0.565]),
            (1.00, [0.541, 0.604, 0.478]),
        ],
    },
    // Wetland: still blue-gray water, golden reeds, brown cattails and green.
    Palette {
        id: "wetland",
        name: "Wetland",
        stops: &[
            (0.00, [0.165, 0.227, 0.290]),
            (0.16, [0.290, 0.416, 0.478]),
            (0.28, [0.541, 0.659, 0.690]),
            (0.38, [0.784, 0.784, 0.627]),
            (0.46, [0.847, 0.722, 0.376]),
            (0.54, [0.478, 0.290, 0.165]),
            (0.62, [0.290, 0.227, 0.102]),
            (0.72, [0.416, 0.541, 0.227]),
            (0.84, [0.227, 0.353, 0.353]),
            (1.00, [0.165, 0.227, 0.290]),
        ],
    },
    // Kelp forest: sunlit teal water, golden-brown kelp and the dark below.
    Palette {
        id: "kelp-forest",
        name: "Kelp Forest",
        stops: &[
            (0.00, [0.008, 0.102, 0.141]),
            (0.16, [0.039, 0.227, 0.290]),
            (0.28, [0.102, 0.416, 0.478]),
            (0.38, [0.290, 0.690, 0.690]),
            (0.46, [0.659, 0.878, 0.847]),
            (0.52, [0.722, 0.627, 0.251]),
            (0.60, [0.478, 0.353, 0.102]),
            (0.70, [0.227, 0.227, 0.078]),
            (0.82, [0.039, 0.165, 0.227]),
            (1.00, [0.008, 0.102, 0.141]),
        ],
    },
    // Lagoon: an atoll's deep blue ring, turquoise shallows and white sand.
    Palette {
        id: "lagoon",
        name: "Lagoon",
        stops: &[
            (0.00, [0.016, 0.188, 0.416]),
            (0.16, [0.039, 0.416, 0.690]),
            (0.28, [0.102, 0.722, 0.816]),
            (0.38, [0.416, 0.910, 0.878]),
            (0.46, [0.784, 0.973, 0.941]),
            (0.52, [0.992, 0.965, 0.894]),
            (0.58, [0.941, 0.878, 0.722]),
            (0.64, [0.290, 0.816, 0.816]),
            (0.76, [0.039, 0.541, 0.784]),
            (0.90, [0.024, 0.165, 0.353]),
            (1.00, [0.016, 0.188, 0.416]),
        ],
    },
    // Deep sea: the black abyss, with blue and cyan glows and an anglerfish's red.
    Palette {
        id: "deep-sea",
        name: "Deep Sea",
        stops: &[
            (0.00, [0.000, 0.008, 0.031]),
            (0.30, [0.008, 0.039, 0.118]),
            (0.44, [0.039, 0.102, 0.227]),
            (0.50, [0.102, 0.416, 0.784]),
            (0.54, [0.290, 0.941, 1.000]),
            (0.57, [0.910, 1.000, 1.000]),
            (0.60, [0.165, 0.541, 0.910]),
            (0.70, [0.024, 0.071, 0.165]),
            (0.80, [0.227, 0.039, 0.078]),
            (0.86, [0.000, 0.008, 0.031]),
            (1.00, [0.000, 0.008, 0.031]),
        ],
    },
    // Hydrothermal vent: a black smoker, rust and sulfur, white bacterial mats and red tube worms.
    Palette {
        id: "hydrothermal-vent",
        name: "Hydrothermal Vent",
        stops: &[
            (0.00, [0.039, 0.039, 0.047]),
            (0.14, [0.180, 0.196, 0.212]),
            (0.26, [0.416, 0.290, 0.165]),
            (0.34, [0.784, 0.471, 0.102]),
            (0.42, [0.941, 0.816, 0.251]),
            (0.48, [0.973, 0.957, 0.910]),
            (0.54, [0.910, 0.125, 0.227]),
            (0.62, [0.541, 0.055, 0.102]),
            (0.74, [0.290, 0.306, 0.337]),
            (0.86, [0.078, 0.078, 0.078]),
            (1.00, [0.039, 0.039, 0.047]),
        ],
    },
    // Volcano: black basalt, molten lava from red to yellow, and gray ash.
    Palette {
        id: "volcano",
        name: "Volcano",
        stops: &[
            (0.00, [0.055, 0.039, 0.039]),
            (0.16, [0.165, 0.102, 0.094]),
            (0.26, [0.541, 0.102, 0.039]),
            (0.34, [0.910, 0.251, 0.039]),
            (0.40, [1.000, 0.541, 0.102]),
            (0.44, [1.000, 0.878, 0.353]),
            (0.48, [1.000, 0.541, 0.102]),
            (0.56, [0.659, 0.125, 0.039]),
            (0.66, [0.227, 0.141, 0.125]),
            (0.78, [0.541, 0.525, 0.510]),
            (0.88, [0.227, 0.212, 0.204]),
            (1.00, [0.055, 0.039, 0.039]),
        ],
    },
    // Canyon: layer on layer of red, orange and cream sandstone, with purple shadow.
    Palette {
        id: "canyon",
        name: "Canyon",
        stops: &[
            (0.00, [0.541, 0.165, 0.078]),
            (0.10, [0.784, 0.314, 0.165]),
            (0.18, [0.910, 0.533, 0.290]),
            (0.24, [0.957, 0.784, 0.565]),
            (0.30, [0.722, 0.314, 0.165]),
            (0.38, [0.478, 0.165, 0.102]),
            (0.44, [0.847, 0.416, 0.227]),
            (0.52, [0.941, 0.690, 0.439]),
            (0.60, [0.353, 0.165, 0.227]),
            (0.70, [0.659, 0.251, 0.118]),
            (0.80, [0.910, 0.627, 0.376]),
            (0.90, [0.784, 0.376, 0.188]),
            (1.00, [0.541, 0.165, 0.078]),
        ],
    },
    // Salt flat: a white salt crust mirroring a pink and blue dusk.
    Palette {
        id: "salt-flat",
        name: "Salt Flat",
        stops: &[
            (0.00, [0.957, 0.949, 0.933]),
            (0.12, [0.894, 0.886, 0.871]),
            (0.22, [0.722, 0.706, 0.784]),
            (0.32, [0.416, 0.596, 0.847]),
            (0.40, [0.659, 0.769, 0.925]),
            (0.48, [0.941, 0.659, 0.784]),
            (0.56, [0.973, 0.816, 0.863]),
            (0.64, [1.000, 0.973, 0.957]),
            (0.74, [0.784, 0.659, 0.847]),
            (0.84, [0.957, 0.784, 0.690]),
            (1.00, [0.957, 0.949, 0.933]),
        ],
    },
    // Cave: dark limestone, cream flowstone and a still teal pool.
    Palette {
        id: "cave",
        name: "Cave",
        stops: &[
            (0.00, [0.020, 0.024, 0.031]),
            (0.20, [0.118, 0.133, 0.149]),
            (0.32, [0.290, 0.290, 0.267]),
            (0.42, [0.604, 0.565, 0.502]),
            (0.50, [0.910, 0.878, 0.800]),
            (0.56, [0.722, 0.659, 0.533]),
            (0.64, [0.227, 0.227, 0.212]),
            (0.72, [0.055, 0.290, 0.290]),
            (0.78, [0.227, 0.690, 0.659]),
            (0.84, [0.039, 0.165, 0.165]),
            (1.00, [0.020, 0.024, 0.031]),
        ],
    },
    // Paris: zinc roofs, cream limestone, the golden hour, the iron tower and the Seine.
    Palette {
        id: "paris",
        name: "Paris",
        stops: &[
            (0.00, [0.290, 0.322, 0.376]),
            (0.14, [0.541, 0.573, 0.620]),
            (0.26, [0.847, 0.800, 0.706]),
            (0.36, [0.957, 0.910, 0.816]),
            (0.44, [0.941, 0.753, 0.471]),
            (0.52, [0.784, 0.541, 0.353]),
            (0.60, [0.416, 0.290, 0.227]),
            (0.70, [0.541, 0.478, 0.408]),
            (0.82, [0.416, 0.478, 0.541]),
            (1.00, [0.290, 0.322, 0.376]),
        ],
    },
    // Venice: canal green, white Istrian stone, ochre and terracotta plaster and faded rose.
    Palette {
        id: "venice",
        name: "Venice",
        stops: &[
            (0.00, [0.055, 0.227, 0.243]),
            (0.12, [0.118, 0.416, 0.408]),
            (0.22, [0.353, 0.659, 0.604]),
            (0.30, [0.949, 0.925, 0.878]),
            (0.38, [0.910, 0.659, 0.251]),
            (0.46, [0.784, 0.314, 0.165]),
            (0.54, [0.541, 0.165, 0.118]),
            (0.62, [0.910, 0.604, 0.478]),
            (0.70, [0.957, 0.863, 0.753]),
            (0.82, [0.165, 0.478, 0.463]),
            (1.00, [0.055, 0.227, 0.243]),
        ],
    },
    // Santorini: whitewashed walls, blue domes, a sunset and bougainvillea.
    Palette {
        id: "santorini",
        name: "Santorini",
        stops: &[
            (0.00, [0.039, 0.227, 0.541]),
            (0.14, [0.102, 0.353, 0.784]),
            (0.24, [0.416, 0.627, 0.910]),
            (0.34, [0.973, 0.973, 0.965]),
            (0.46, [1.000, 1.000, 1.000]),
            (0.54, [0.941, 0.910, 0.878]),
            (0.62, [1.000, 0.541, 0.353]),
            (0.70, [0.910, 0.227, 0.541]),
            (0.80, [0.973, 0.784, 0.627]),
            (0.90, [0.165, 0.290, 0.659]),
            (1.00, [0.039, 0.227, 0.541]),
        ],
    },
    // Chefchaouen: the blue town in the Rif, every shade of blue wall, and a terracotta pot.
    Palette {
        id: "chefchaouen",
        name: "Chefchaouen",
        stops: &[
            (0.00, [0.102, 0.290, 0.659]),
            (0.14, [0.227, 0.478, 0.847]),
            (0.26, [0.478, 0.690, 0.941]),
            (0.36, [0.722, 0.847, 0.988]),
            (0.44, [0.910, 0.949, 0.996]),
            (0.52, [0.290, 0.541, 0.878]),
            (0.62, [0.165, 0.353, 0.753]),
            (0.70, [0.784, 0.416, 0.227]),
            (0.76, [0.541, 0.753, 0.973]),
            (0.88, [0.118, 0.353, 0.722]),
            (1.00, [0.102, 0.290, 0.659]),
        ],
    },
    // Marrakech: red ochre walls, Majorelle blue, lemon, mint and spice.
    Palette {
        id: "marrakech",
        name: "Marrakech",
        stops: &[
            (0.00, [0.659, 0.251, 0.118]),
            (0.14, [0.784, 0.376, 0.227]),
            (0.24, [0.910, 0.604, 0.416]),
            (0.32, [0.102, 0.227, 0.847]),
            (0.40, [0.165, 0.353, 0.941]),
            (0.48, [0.941, 0.722, 0.102]),
            (0.56, [0.227, 0.659, 0.416]),
            (0.66, [0.541, 0.165, 0.102]),
            (0.76, [0.910, 0.690, 0.565]),
            (0.88, [0.722, 0.290, 0.141]),
            (1.00, [0.659, 0.251, 0.118]),
        ],
    },
    // Jaipur: the Pink City's terracotta-pink walls, white trim and gold.
    Palette {
        id: "jaipur",
        name: "Jaipur",
        stops: &[
            (0.00, [0.784, 0.376, 0.353]),
            (0.16, [0.910, 0.541, 0.478]),
            (0.28, [0.957, 0.690, 0.627]),
            (0.36, [1.000, 0.957, 0.925]),
            (0.44, [0.910, 0.627, 0.314]),
            (0.54, [0.847, 0.416, 0.353]),
            (0.62, [0.722, 0.290, 0.290]),
            (0.72, [0.941, 0.753, 0.690]),
            (0.84, [0.541, 0.753, 0.878]),
            (1.00, [0.784, 0.376, 0.353]),
        ],
    },
    // Istanbul: Iznik tiles, cobalt, turquoise, tomato red and green on white.
    Palette {
        id: "istanbul",
        name: "Istanbul",
        stops: &[
            (0.00, [0.957, 0.949, 0.925]),
            (0.12, [0.102, 0.227, 0.659]),
            (0.22, [0.039, 0.118, 0.416]),
            (0.30, [0.165, 0.690, 0.753]),
            (0.38, [0.957, 0.949, 0.925]),
            (0.46, [0.847, 0.157, 0.102]),
            (0.54, [0.957, 0.949, 0.925]),
            (0.62, [0.165, 0.541, 0.290]),
            (0.72, [0.102, 0.353, 0.784]),
            (0.84, [0.541, 0.878, 0.910]),
            (1.00, [0.957, 0.949, 0.925]),
        ],
    },
    // Taj Mahal: white marble turning pink and gold at dawn, red sandstone and the garden.
    Palette {
        id: "taj-mahal",
        name: "Taj Mahal",
        stops: &[
            (0.00, [0.973, 0.957, 0.933]),
            (0.14, [0.973, 0.878, 0.847]),
            (0.24, [0.941, 0.784, 0.565]),
            (0.32, [1.000, 0.980, 0.957]),
            (0.42, [0.784, 0.847, 0.910]),
            (0.50, [0.910, 0.894, 0.878]),
            (0.58, [0.659, 0.290, 0.227]),
            (0.66, [0.416, 0.165, 0.125]),
            (0.76, [0.227, 0.416, 0.227]),
            (0.86, [0.910, 0.863, 0.816]),
            (1.00, [0.973, 0.957, 0.933]),
        ],
    },
    // Petra: the rose-red city, banded sandstone from pink to deep shadow.
    Palette {
        id: "petra",
        name: "Petra",
        stops: &[
            (0.00, [0.416, 0.165, 0.141]),
            (0.12, [0.659, 0.282, 0.227]),
            (0.22, [0.847, 0.471, 0.416]),
            (0.30, [0.941, 0.690, 0.627]),
            (0.36, [0.784, 0.376, 0.314]),
            (0.44, [0.910, 0.604, 0.478]),
            (0.52, [0.541, 0.227, 0.188]),
            (0.60, [0.957, 0.816, 0.722]),
            (0.68, [0.722, 0.345, 0.282]),
            (0.78, [0.290, 0.118, 0.102]),
            (0.88, [0.784, 0.439, 0.376]),
            (1.00, [0.416, 0.165, 0.141]),
        ],
    },
    // Giza: the pyramids' limestone and sand under a deep blue sky, and their shadows.
    Palette {
        id: "giza",
        name: "Giza",
        stops: &[
            (0.00, [0.165, 0.290, 0.541]),
            (0.14, [0.416, 0.604, 0.847]),
            (0.24, [0.784, 0.863, 0.941]),
            (0.32, [0.957, 0.894, 0.753]),
            (0.42, [0.910, 0.784, 0.502]),
            (0.52, [0.784, 0.596, 0.282]),
            (0.60, [0.541, 0.416, 0.227]),
            (0.68, [0.227, 0.165, 0.102]),
            (0.78, [0.847, 0.690, 0.439]),
            (0.88, [0.941, 0.847, 0.659]),
            (1.00, [0.165, 0.290, 0.541]),
        ],
    },
    // Kyoto: vermilion torii, black lacquer, moss gardens, the golden pavilion and red maples.
    Palette {
        id: "kyoto",
        name: "Kyoto",
        stops: &[
            (0.00, [0.910, 0.251, 0.102]),
            (0.12, [1.000, 0.416, 0.165]),
            (0.20, [0.078, 0.071, 0.063]),
            (0.28, [0.165, 0.290, 0.102]),
            (0.38, [0.353, 0.541, 0.165]),
            (0.46, [0.847, 0.690, 0.188]),
            (0.54, [0.941, 0.847, 0.439]),
            (0.62, [0.910, 0.251, 0.102]),
            (0.70, [0.973, 0.941, 0.878]),
            (0.80, [0.541, 0.102, 0.063]),
            (0.90, [0.784, 0.188, 0.102]),
            (1.00, [0.910, 0.251, 0.102]),
        ],
    },
    // Tokyo by night: neon pink, cyan, yellow and purple in the dark.
    Palette {
        id: "tokyo",
        name: "Tokyo",
        stops: &[
            (0.00, [0.039, 0.031, 0.078]),
            (0.14, [0.102, 0.063, 0.188]),
            (0.24, [1.000, 0.165, 0.541]),
            (0.30, [1.000, 0.690, 0.847]),
            (0.36, [0.165, 0.102, 0.290]),
            (0.46, [0.165, 0.941, 1.000]),
            (0.52, [0.847, 1.000, 1.000]),
            (0.58, [0.227, 0.165, 0.541]),
            (0.68, [1.000, 0.816, 0.165]),
            (0.76, [0.102, 0.078, 0.157]),
            (0.88, [0.659, 0.165, 0.941]),
            (1.00, [0.039, 0.031, 0.078]),
        ],
    },
    // The Forbidden City: vermilion walls, yellow glazed roof tiles, blue-green eaves and white
    // marble.
    Palette {
        id: "forbidden-city",
        name: "Forbidden City",
        stops: &[
            (0.00, [0.659, 0.125, 0.102]),
            (0.14, [0.784, 0.188, 0.118]),
            (0.24, [0.941, 0.722, 0.102]),
            (0.32, [1.000, 0.847, 0.290]),
            (0.40, [0.165, 0.416, 0.541]),
            (0.48, [0.227, 0.604, 0.478]),
            (0.56, [0.957, 0.941, 0.910]),
            (0.64, [0.784, 0.769, 0.737]),
            (0.74, [0.541, 0.102, 0.078]),
            (0.86, [0.847, 0.251, 0.118]),
            (1.00, [0.659, 0.125, 0.102]),
        ],
    },
    // Angkor Wat: sandstone towers against a sunrise, the lotus pond and its flowers.
    Palette {
        id: "angkor-wat",
        name: "Angkor Wat",
        stops: &[
            (0.00, [0.165, 0.165, 0.227]),
            (0.14, [0.353, 0.290, 0.290]),
            (0.24, [0.659, 0.478, 0.353]),
            (0.32, [0.941, 0.627, 0.376]),
            (0.40, [0.973, 0.816, 0.627]),
            (0.48, [0.910, 0.541, 0.541]),
            (0.56, [0.722, 0.627, 0.753]),
            (0.66, [0.227, 0.353, 0.227]),
            (0.76, [0.416, 0.541, 0.353]),
            (0.86, [0.941, 0.627, 0.722]),
            (1.00, [0.165, 0.165, 0.227]),
        ],
    },
    // Machu Picchu: gray granite, green terraces, mist and the blue Andes.
    Palette {
        id: "machu-picchu",
        name: "Machu Picchu",
        stops: &[
            (0.00, [0.227, 0.290, 0.227]),
            (0.16, [0.353, 0.541, 0.227]),
            (0.28, [0.604, 0.784, 0.416]),
            (0.36, [0.541, 0.541, 0.518]),
            (0.46, [0.722, 0.722, 0.690]),
            (0.54, [0.910, 0.925, 0.925]),
            (0.62, [0.659, 0.722, 0.784]),
            (0.72, [0.290, 0.416, 0.541]),
            (0.84, [0.416, 0.478, 0.353]),
            (1.00, [0.227, 0.290, 0.227]),
        ],
    },
    // Havana: pastel turquoise, pink and yellow facades, a red vintage car and the sea.
    Palette {
        id: "havana",
        name: "Havana",
        stops: &[
            (0.00, [0.290, 0.784, 0.753]),
            (0.12, [0.541, 0.878, 0.847]),
            (0.20, [0.973, 0.659, 0.753]),
            (0.30, [1.000, 0.878, 0.541]),
            (0.38, [0.973, 0.941, 0.878]),
            (0.46, [0.910, 0.188, 0.165]),
            (0.54, [1.000, 0.541, 0.353]),
            (0.64, [0.659, 0.847, 0.941]),
            (0.72, [0.102, 0.478, 0.722]),
            (0.84, [0.941, 0.784, 0.627]),
            (1.00, [0.290, 0.784, 0.753]),
        ],
    },
    // Burano: the island's brightly painted houses, side by side, with white trim.
    Palette {
        id: "burano",
        name: "Burano",
        stops: &[
            (0.0000, [0.910, 0.157, 0.102]),
            (0.1351, [0.910, 0.157, 0.102]),
            (0.1351, [1.000, 0.824, 0.102]),
            (0.2703, [1.000, 0.824, 0.102]),
            (0.2703, [0.102, 0.416, 0.847]),
            (0.4054, [0.102, 0.416, 0.847]),
            (0.4054, [0.973, 0.965, 0.941]),
            (0.4324, [0.973, 0.965, 0.941]),
            (0.4324, [0.165, 0.659, 0.290]),
            (0.5676, [0.165, 0.659, 0.290]),
            (0.5676, [1.000, 0.478, 0.102]),
            (0.7027, [1.000, 0.478, 0.102]),
            (0.7027, [0.784, 0.102, 0.541]),
            (0.8378, [0.784, 0.102, 0.541]),
            (0.8378, [0.973, 0.965, 0.941]),
            (0.8649, [0.973, 0.965, 0.941]),
            (0.8649, [0.227, 0.784, 0.878]),
            (1.0000, [0.227, 0.784, 0.878]),
        ],
    },
    // New York: a steel-blue dusk, yellow cabs, brownstones and lit windows.
    Palette {
        id: "new-york",
        name: "New York",
        stops: &[
            (0.00, [0.055, 0.078, 0.157]),
            (0.14, [0.102, 0.165, 0.290]),
            (0.24, [0.290, 0.416, 0.541]),
            (0.34, [0.659, 0.722, 0.784]),
            (0.42, [0.973, 0.784, 0.102]),
            (0.48, [1.000, 0.902, 0.502]),
            (0.54, [0.541, 0.227, 0.141]),
            (0.62, [0.784, 0.376, 0.227]),
            (0.70, [0.941, 0.878, 0.753]),
            (0.80, [0.227, 0.290, 0.416]),
            (0.90, [0.102, 0.118, 0.188]),
            (1.00, [0.055, 0.078, 0.157]),
        ],
    },
    // Golden Gate: International Orange, fog, the bay and the Marin hills.
    Palette {
        id: "golden-gate",
        name: "Golden Gate",
        stops: &[
            (0.00, [0.753, 0.212, 0.173]),
            (0.14, [0.910, 0.314, 0.227]),
            (0.22, [0.941, 0.541, 0.416]),
            (0.30, [0.910, 0.910, 0.925]),
            (0.40, [0.784, 0.800, 0.831]),
            (0.50, [0.541, 0.604, 0.659]),
            (0.58, [0.165, 0.353, 0.541]),
            (0.68, [0.102, 0.227, 0.416]),
            (0.78, [0.416, 0.478, 0.290]),
            (0.88, [0.659, 0.251, 0.165]),
            (1.00, [0.753, 0.212, 0.173]),
        ],
    },
    // St. Basil's: the striped and checkered domes in red, white, green, gold and blue.
    Palette {
        id: "st-basils",
        name: "St. Basil's",
        stops: &[
            (0.0000, [0.784, 0.125, 0.102]),
            (0.1579, [0.784, 0.125, 0.102]),
            (0.1579, [0.973, 0.957, 0.925]),
            (0.2368, [0.973, 0.957, 0.925]),
            (0.2368, [0.165, 0.541, 0.227]),
            (0.3947, [0.165, 0.541, 0.227]),
            (0.3947, [0.910, 0.722, 0.165]),
            (0.5000, [0.910, 0.722, 0.165]),
            (0.5000, [0.102, 0.353, 0.722]),
            (0.6579, [0.102, 0.353, 0.722]),
            (0.6579, [0.973, 0.957, 0.925]),
            (0.7368, [0.973, 0.957, 0.925]),
            (0.7368, [0.784, 0.125, 0.102]),
            (0.8421, [0.784, 0.125, 0.102]),
            (0.8421, [0.165, 0.541, 0.227]),
            (0.9474, [0.165, 0.541, 0.227]),
            (0.9474, [0.910, 0.722, 0.165]),
            (1.0000, [0.910, 0.722, 0.165]),
        ],
    },
];

#[cfg(test)]
#[path = "../tests/unit/tests.rs"]
mod tests;
