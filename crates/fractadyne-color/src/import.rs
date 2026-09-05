//! Palette importers: format → [`Gradient`].
//!
//! Layer 1 of `design/palette-import.md` §4. Each importer's whole job is to produce a segment
//! gradient; nothing here touches the GPU, and nothing past here knows what format a palette came
//! from. `.map` is first because it is the simplest and because Fractint's `default.map` is a
//! ready-made fixture with a documented structure. `.ugr`, `.ggr` and the swatch-list formats
//! belong beside it.

use crate::segment::Gradient;

/// A parsed Fractint / Kalles Fraktaler `.map` palette.
#[derive(Clone, Debug, PartialEq)]
pub struct MapPalette {
    /// Entries in file order, DISPLAY-space `0..1` (see `srgb8_to_stop`).
    pub colors: Vec<[f32; 3]>,
    /// ⭐Every value is a multiple of 4 and the maximum is 252 — the signature of a **6-bit VGA**
    /// table, which is what Fractint's own `.map` files are (0–63 written out ×4).
    ///
    /// ⚠**Reported, deliberately NOT rescaled.** Rescaling 252 → 255 would arguably be more
    /// "correct" against a VGA DAC, where 63 meant full intensity. But the bar this importer is
    /// held to (`design/palette-import.md` §7) is matching *the source application's own render*,
    /// and Fractint writes its images with the same ×4, so its white pixels ARE 252. Rescaling
    /// would make every comparison against a Fractint render fail by a few percent — an error
    /// small enough to look like nothing and large enough to fail an exactness check.
    pub vga_6bit: bool,
}

impl MapPalette {
    /// The palette as **bands** — one flat segment per entry, no interpolation.
    ///
    /// ⭐⭐This is the faithful reading and the default. Fractint indexes the table by iteration
    /// count with no blending between entries, and the hard steps that produces are the classic
    /// look — `default.map` has 37 jumps of more than 60/252 between adjacent entries. Importing
    /// it as a smooth gradient does not lower the fidelity, it produces a different palette.
    pub fn bands(&self, name: impl Into<String>) -> Gradient {
        Gradient::from_bands(name, &self.colors)
    }

    /// The palette **smoothed** — evenly spaced stops with linear blending between them.
    ///
    /// Offered because a `.map` is also just an ordered colour list, and a deep zoom cycling a
    /// palette hundreds of times across a narrow band of escape values often wants the ramp. It is
    /// a choice the user makes, never a default the importer makes for them.
    pub fn smooth(&self, name: impl Into<String>) -> Gradient {
        Gradient::from_colors(name, &self.colors)
    }
}

/// Parse a Fractint / Kalles Fraktaler `.map`: up to 256 lines of `R G B`, 0–255 decimal.
///
/// Strict where it matters and tolerant where the format itself is: `;` starts a comment (the
/// `.map` convention), blank lines are skipped, and text after the third value is ignored — real
/// files carry colour names there. What is NOT tolerated is a line that does not begin with three
/// integers in 0–255, because that is the difference between reading a `.map` and reading
/// something else that happens to have numbers in it.
///
/// ⚠This is deliberately narrower than [`crate::parse_palette_text`], which is the "I found a
/// palette on the web" paste box and guesses at the shape. A file the user named as a `.map` has
/// declared its format, so a parse failure should be reported, not guessed around.
pub fn parse_map(text: &str) -> Result<MapPalette, String> {
    let mut colors: Vec<[f32; 3]> = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split(';').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut tok = line.split_whitespace();
        let mut v = [0u32; 3];
        for (i, slot) in v.iter_mut().enumerate() {
            let t = tok.next().ok_or_else(|| {
                format!("line {}: expected three values (R G B), found {i}", n + 1)
            })?;
            let parsed: u32 = t
                .parse()
                .map_err(|_| format!("line {}: {t:?} is not a number 0-255", n + 1))?;
            if parsed > 255 {
                return Err(format!("line {}: {parsed} is out of range - values run 0-255", n + 1));
            }
            *slot = parsed;
        }
        // Trailing tokens are a colour name or a comment; real .map files carry both.
        colors.push([v[0] as f32 / 255.0, v[1] as f32 / 255.0, v[2] as f32 / 255.0]);
        if colors.len() == 256 {
            break; // a .map is a 256-entry table; anything after it is not part of the palette
        }
    }
    if colors.len() < 2 {
        return Err(format!(
            "found {} colour{} - a .map file is a list of R G B lines, one per palette entry",
            colors.len(),
            if colors.len() == 1 { "" } else { "s" }
        ));
    }
    // 6-bit detection runs on the ORIGINAL bytes, recovered exactly: v/255 * 255 round-trips.
    let byte = |c: f32| (c * 255.0).round() as u32;
    let all = colors.iter().flat_map(|c| c.iter().map(|v| byte(*v)));
    let (mut multiple_of_4, mut max) = (true, 0u32);
    for b in all {
        multiple_of_4 &= b % 4 == 0;
        max = max.max(b);
    }
    Ok(MapPalette { colors, vga_6bit: multiple_of_4 && max == 252 })
}

#[cfg(test)]
mod import_tests;
