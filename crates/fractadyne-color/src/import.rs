//! Palette importers: format → [`Gradient`].
//!
//! Layer 1 of `design/palette-import.md` §4. Each importer's whole job is to produce a segment
//! gradient; nothing here touches the GPU, and nothing past here knows what format a palette came
//! from. `.map` is first because it is the simplest and because Fractint's `default.map` is a
//! ready-made fixture with a documented structure. `.ggr` and the swatch-list formats belong
//! beside `.map` and `.ugr`.

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

// ================================================================================================
// Ultra Fractal `.ugr`
// ================================================================================================

/// Highest index an Ultra Fractal gradient uses. ⭐**400 positions, `0..=399`** — not 0–255 and not
/// 0–1, which is the first thing to get wrong about the format.
pub const UGR_INDEX_MAX: u32 = 399;

/// One named gradient out of a `.ugr` file.
///
/// ⚠A `.ugr` holds **many** gradients, so an importer must offer a LIST to choose from; loading
/// "the" gradient would silently pick one of dozens.
#[derive(Clone, Debug, PartialEq)]
pub struct UgrGradient {
    /// The block's identifier — `blatte10` in `blatte10 { ... }`. This is what a picker shows.
    pub name: String,
    /// The optional `title="..."` inside the block. Usually the same as `name`, occasionally nicer.
    pub title: Option<String>,
    /// `(index 0..=399, DISPLAY-space RGB)`, in file order.
    pub stops: Vec<(u32, [f32; 3])>,
    /// The `opacity:` section's `(index, alpha 0..1)` — parsed and carried so the data is not
    /// thrown away, though the renderer's palette has no alpha channel today.
    pub opacity: Vec<(u32, f32)>,
    /// `rotation=` in index units, i.e. `0..=399`.
    pub rotation: u32,
    /// The file's `smooth=yes|no`.
    ///
    /// ⚠**Recorded, not honoured.** Ultra Fractal's smooth mode is a spline through the control
    /// points, which is not a per-segment blend function and so is not expressible in the model
    /// this crate uses; gnofract4d has the same limitation and also imports linearly. Silently
    /// mapping it onto `Blend::Curved` would look like support for something we do not do.
    pub smooth: bool,
}

impl UgrGradient {
    /// One linear RGB segment per adjacent index pair, position = `index / 399`, then the file's
    /// `rotation` applied.
    pub fn to_gradient(&self) -> Gradient {
        let stops: Vec<(f32, [f32; 3])> = self
            .stops
            .iter()
            .map(|(i, c)| ((*i).min(UGR_INDEX_MAX) as f32 / UGR_INDEX_MAX as f32, *c))
            .collect();
        let g = Gradient::from_stops(self.title.clone().unwrap_or_else(|| self.name.clone()), &stops);
        if self.rotation == 0 {
            g
        } else {
            // ⚠**Direction unverified against Ultra Fractal itself.** Applying a stated field with
            // a possibly-wrong sign is still better than ignoring it (ignoring it is wrong for
            // certain), and `ugr_rotation_shifts_the_ring` pins the current direction so a
            // correction is one sign change plus one test edit rather than an archaeology job.
            g.rotated(self.rotation as f32 / (UGR_INDEX_MAX + 1) as f32)
        }
    }
}

/// Parse an Ultra Fractal `.ugr`, returning every gradient in the file, in file order.
///
/// The format is free-form: a `name { ... }` block containing a `gradient:` section of interleaved
/// `index=`/`color=` pairs wrapped across as many lines as it likes, optional `title=`, `smooth=`
/// and `rotation=`, and a separate `opacity:` section. Whitespace and line breaks carry no meaning
/// inside a block, so this tokenises rather than reading line by line.
///
/// ⭐⭐**`color=` is a decimal integer packed BGR — `0xBBGGRR`, red is the LOW byte.** Verified from
/// gnofract4d's decoder (`icolor & 0xFF` → red, `(icolor >> 16) & 0xFF` → blue). Reading it as RGB
/// swaps red and blue on every imported gradient and **still looks plausible**, which is exactly
/// how that bug survives a review.
///
/// ⚠gnofract4d divides those bytes by **256.0**; we divide by **255.0**, so our import is a hair
/// brighter than theirs and, unlike theirs, reaches pure white.
pub fn parse_ugr(text: &str) -> Result<Vec<UgrGradient>, String> {
    let toks = ugr_tokens(text);
    let mut out: Vec<UgrGradient> = Vec::new();
    let mut i = 0usize;
    while i < toks.len() {
        // A block opens with `name {`. Anything before that is a stray token; skip it rather than
        // failing, because .ugr files in the wild carry headers and comments we do not model.
        if i + 1 >= toks.len() || toks[i + 1] != "{" {
            i += 1;
            continue;
        }
        let name = toks[i].clone();
        i += 2;
        let mut g = UgrGradient {
            name,
            title: None,
            stops: Vec::new(),
            opacity: Vec::new(),
            rotation: 0,
            smooth: false,
        };
        // `index=` applies to whichever section we are in: it pairs with the NEXT `color=` in the
        // gradient section and the next `opacity=` in the opacity section.
        let mut in_opacity = false;
        let mut pending: Option<u32> = None;
        while i < toks.len() && toks[i] != "}" {
            let t = &toks[i];
            if t == "gradient:" {
                in_opacity = false;
                pending = None;
            } else if t == "opacity:" {
                in_opacity = true;
                pending = None;
            } else if let Some(v) = t.strip_prefix("index=") {
                pending = Some(ugr_u32(v, "index")?);
            } else if let Some(v) = t.strip_prefix("color=") {
                let packed = ugr_u32(v, "color")?;
                let idx = pending.take().ok_or_else(|| {
                    format!("{}: a color= with no index= before it", g.name)
                })?;
                // BGR: red is the low byte.
                let b = |shift: u32| ((packed >> shift) & 0xFF) as f32 / 255.0;
                g.stops.push((idx, [b(0), b(8), b(16)]));
            } else if let Some(v) = t.strip_prefix("opacity=") {
                let a = ugr_u32(v, "opacity")?;
                if let Some(idx) = pending.take() {
                    g.opacity.push((idx, (a.min(255) as f32) / 255.0));
                }
            } else if let Some(v) = t.strip_prefix("rotation=") {
                g.rotation = ugr_u32(v, "rotation")? % (UGR_INDEX_MAX + 1);
            } else if let Some(v) = t.strip_prefix("smooth=") {
                if !in_opacity {
                    g.smooth = v.eq_ignore_ascii_case("yes") || v == "1";
                }
            } else if let Some(v) = t.strip_prefix("title=") {
                if !in_opacity {
                    g.title = Some(v.trim_matches('"').to_string());
                }
            }
            i += 1;
        }
        i += 1; // past the closing brace
        if g.stops.len() >= 2 {
            g.stops.sort_by_key(|(idx, _)| *idx);
            out.push(g);
        }
        // A block with fewer than two colours is not a gradient (UF files carry formula and
        // parameter blocks in the same syntax); skipping it is what lets a mixed file load.
    }
    if out.is_empty() {
        return Err(
            "no gradients found - a .ugr holds `name { gradient: index=.. color=.. }` blocks"
                .to_string(),
        );
    }
    Ok(out)
}

/// Split a `.ugr` into tokens: `{`, `}`, `gradient:`, `opacity:`, and `key=value` (a quoted value
/// keeps its spaces). `;` starts a comment.
fn ugr_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let push = |cur: &mut String, out: &mut Vec<String>| {
        if !cur.is_empty() {
            out.push(std::mem::take(cur));
        }
    };
    for line in text.lines() {
        // Comments run to end of line — but a `;` inside a quoted title is not a comment.
        let mut quoted = false;
        let line = match line.char_indices().find(|(_, c)| {
            if *c == '"' {
                quoted = !quoted;
            }
            *c == ';' && !quoted
        }) {
            Some((at, _)) => &line[..at],
            None => line,
        };
        for c in line.chars() {
            match c {
                '"' => {
                    in_quote = !in_quote;
                    cur.push(c);
                }
                '{' | '}' if !in_quote => {
                    push(&mut cur, &mut out);
                    out.push(c.to_string());
                }
                c if c.is_whitespace() && !in_quote => push(&mut cur, &mut out),
                c => cur.push(c),
            }
        }
        // A block name sits on its own line before `{`; a wrapped `index=`/`color=` run does not
        // care about the break. Either way the token ends at the newline.
        push(&mut cur, &mut out);
        in_quote = false; // a quote never spans lines in this format
    }
    out
}

fn ugr_u32(v: &str, what: &str) -> Result<u32, String> {
    v.trim()
        .parse::<u32>()
        .map_err(|_| format!("{what}={v:?} is not a whole number"))
}

// ================================================================================================
// GIMP `.ggr`
// ================================================================================================

/// Parse a GIMP `.ggr` gradient.
///
/// ⭐**This is the format the whole segment model was taken from**, so unlike `.map` and `.ugr` it
/// needs no lowering: a `.ggr` segment IS a [`Segment`](crate::segment::Segment), midpoint, blend
/// function, colour space and all. Nothing is approximated here.
///
/// ```text
/// GIMP Gradient
/// Name: Full saturation spectrum CCW
/// 7
/// left mid right   lr lg lb la   rr rg rb ra   blend colour [left_type right_type]
/// ```
///
/// Thirteen numbers per segment, or fifteen in GIMP 2.x files that record where each endpoint's
/// colour comes from (fixed, foreground, background); the two extra are read and ignored, because
/// "the current foreground colour" is not a thing a fractal renderer has.
///
/// ⚠**Colours are DISPLAY-space `0..1` and pass straight through.** GIMP writes what the screen
/// shows, and so does this renderer (see `srgb8_to_stop`), so a conversion here would be the same
/// mistake that made pasted palettes too dark before beta.21.
///
/// ⚠**A `+`-prefixed compressed continuation line is NOT supported.** `design/palette-import.md` §2
/// records that such a thing exists; it was not confirmed against a real file or a working parser,
/// and guessing at a compression scheme is how a palette gets silently mis-imported. A file using
/// one is REJECTED by name rather than half-read.
pub fn parse_ggr(text: &str) -> Result<Gradient, String> {
    use crate::segment::{Blend, Segment, Space};

    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));

    let head = lines.next().unwrap_or_default();
    if !head.eq_ignore_ascii_case("GIMP Gradient") {
        return Err(format!("expected a \"GIMP Gradient\" header, found {head:?}"));
    }
    // `Name:` arrived in GIMP 1.2; a file without one is still valid, and its next line is the
    // segment count. Peeking rather than assuming is what lets both load.
    let mut name = String::new();
    let mut next = lines.next().unwrap_or_default().to_string();
    if let Some(n) = next.strip_prefix("Name:") {
        name = n.trim().to_string();
        next = lines.next().unwrap_or_default().to_string();
    }
    let count: usize = next
        .parse()
        .map_err(|_| format!("expected a segment count, found {next:?}"))?;
    if count == 0 {
        return Err("the file declares 0 segments".to_string());
    }

    let mut segments: Vec<Segment> = Vec::with_capacity(count);
    for (n, line) in lines.enumerate() {
        if line.starts_with('+') {
            return Err(format!(
                "segment {}: compressed \"+\" continuation lines are not supported",
                n + 1
            ));
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        // 13 fields, or 15 with GIMP 2.x's per-endpoint colour types.
        if f.len() != 13 && f.len() != 15 {
            return Err(format!(
                "segment {}: expected 13 or 15 values, found {}",
                n + 1,
                f.len()
            ));
        }
        let num = |i: usize| -> Result<f32, String> {
            f[i].parse::<f32>()
                .map_err(|_| format!("segment {}: {:?} is not a number", n + 1, f[i]))
        };
        let int = |i: usize| -> Result<u8, String> {
            f[i].parse::<i64>()
                .map(|v| v.clamp(0, 255) as u8)
                .map_err(|_| format!("segment {}: {:?} is not a whole number", n + 1, f[i]))
        };
        let (left, mid, right) = (num(0)?, num(1)?, num(2)?);
        if !(left <= mid && mid <= right) || !(0.0..=1.0).contains(&left) || right > 1.0 {
            return Err(format!(
                "segment {}: positions {left} / {mid} / {right} are not an ascending span inside 0..1",
                n + 1
            ));
        }
        let col = |i: usize| -> Result<[f32; 4], String> {
            Ok([
                num(i)?.clamp(0.0, 1.0),
                num(i + 1)?.clamp(0.0, 1.0),
                num(i + 2)?.clamp(0.0, 1.0),
                num(i + 3)?.clamp(0.0, 1.0),
            ])
        };
        segments.push(Segment {
            left,
            mid,
            right,
            left_color: col(3)?,
            right_color: col(7)?,
            blend: Blend::from_u8(int(11)?),
            space: Space::from_u8(int(12)?),
        });
        if segments.len() == count {
            break;
        }
    }
    if segments.len() != count {
        return Err(format!(
            "the file declares {count} segments but carries {}",
            segments.len()
        ));
    }
    // Zero-width segments are legal to write and impossible to render; dropping them here keeps
    // `eval`'s divide-by-span guard from being load-bearing.
    segments.retain(|s| s.right > s.left);
    if segments.is_empty() {
        return Err("every segment is zero-width".to_string());
    }
    Ok(Gradient { name, segments })
}

// ================================================================================================
// Adobe `.ase` swatch lists
// ================================================================================================

/// Parse an Adobe Swatch Exchange (`.ase`) file into its colours, in file order.
///
/// A swatch list has **no positions and no interpolation semantics** — it is an ordered set of
/// colours, so importing one means choosing a spacing (even) and a blend (linear). That choice is
/// the caller's: this returns colours, and [`Gradient::from_colors`] makes the gradient.
///
/// The format is big-endian binary:
///
/// ```text
/// "ASEF"  u16 major  u16 minor  u32 block_count
/// per block:  u16 type  u32 body_len  body
///   type 0x0001 = colour:  u16 name_len (UTF-16 units, incl. NUL)  name  4-byte model  f32 values  u16 kind
///   type 0xC001 / 0xC002 = group start / end, skipped
/// ```
///
/// ⚠⚠**UNVERIFIED AGAINST A REAL FILE.** Every other importer here was written against a real file
/// or a working parser (`design/palette-import.md` marks those ✅); this one is written from the
/// published layout alone, because no `.ase` was to hand. It is therefore deliberately **strict** —
/// the signature, the block lengths and the colour model must all agree — so a file that does not
/// match this understanding FAILS LOUDLY instead of importing plausible-looking wrong colours.
/// ⭐The first real `.ase` anyone imports is the actual test; if it is rejected, this is why.
///
/// ⚠**LAB swatches are refused by name, not converted.** LAB → sRGB needs a white point (Adobe uses
/// D50) and a chromatic adaptation, and writing that from memory is exactly the kind of guess that
/// produces colours which look fine and are wrong. Same call as the `.ggr` `+` continuation.
pub fn parse_ase(bytes: &[u8]) -> Result<Vec<[f32; 3]>, String> {
    let need = |at: usize, n: usize| -> Result<(), String> {
        if at + n > bytes.len() {
            Err(format!("file ends early at byte {at} (wanted {n} more)"))
        } else {
            Ok(())
        }
    };
    need(0, 12)?;
    if &bytes[0..4] != b"ASEF" {
        return Err("not an Adobe .ase file (no \"ASEF\" signature)".to_string());
    }
    let u16at = |at: usize| u16::from_be_bytes([bytes[at], bytes[at + 1]]);
    let u32at = |at: usize| {
        u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    };
    let f32at = |at: usize| {
        f32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    };
    let blocks = u32at(8) as usize;
    let mut at = 12usize;

    let mut out: Vec<[f32; 3]> = Vec::new();
    let mut lab = 0usize;
    for b in 0..blocks {
        need(at, 6)?;
        let kind = u16at(at);
        let len = u32at(at + 2) as usize;
        let body = at + 6;
        need(body, len)?;
        at = body + len;
        // 0xC001/0xC002 are group start/end. A group carries no colour of its own, and flattening
        // groups is right for a gradient: the file's ORDER is the only structure that survives.
        if kind != 0x0001 {
            continue;
        }
        if len < 2 {
            return Err(format!("block {}: colour block is too short", b + 1));
        }
        // Name length counts UTF-16 code units including the terminating NUL.
        let name_units = u16at(body) as usize;
        let model_at = body + 2 + name_units * 2;
        if model_at + 4 > body + len {
            return Err(format!("block {}: name length {name_units} overruns the block", b + 1));
        }
        let model = &bytes[model_at..model_at + 4];
        let v = model_at + 4;
        let values = match model {
            b"RGB " => 3,
            b"CMYK" => 4,
            b"Gray" => 1,
            b"LAB " => {
                lab += 1;
                continue;
            }
            other => {
                return Err(format!(
                    "block {}: unknown colour model {:?}",
                    b + 1,
                    String::from_utf8_lossy(other)
                ))
            }
        };
        if v + values * 4 > body + len {
            return Err(format!("block {}: colour values overrun the block", b + 1));
        }
        let f = |i: usize| f32at(v + i * 4).clamp(0.0, 1.0);
        out.push(match values {
            3 => [f(0), f(1), f(2)],
            1 => [f(0), f(0), f(0)],
            // Naive CMYK, stated as such: no ICC profile, no black generation. An .ase carries no
            // profile, so anything cleverer would be inventing one.
            _ => {
                let k = f(3);
                [(1.0 - f(0)) * (1.0 - k), (1.0 - f(1)) * (1.0 - k), (1.0 - f(2)) * (1.0 - k)]
            }
        });
    }
    if lab > 0 && out.is_empty() {
        return Err(format!(
            "every swatch ({lab}) is LAB, which needs a colour profile this importer will not guess at"
        ));
    }
    if lab > 0 {
        return Err(format!(
            "{lab} of {} swatches are LAB, which needs a colour profile this importer will not guess at - importing the rest would silently drop them",
            out.len() + lab
        ));
    }
    if out.len() < 2 {
        return Err(format!("found {} colours - a swatch list needs at least two", out.len()));
    }
    Ok(out)
}

#[cfg(test)]
mod import_tests;
