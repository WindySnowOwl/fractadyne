//! The segment gradient model, and the LUT it bakes into.
//!
//! This is the middle layer `design/palette-import.md` §4 identified as missing: every palette
//! source we care about — our own presets, the gradient editor, `.map`, `.ugr`, `.ggr`, swatch
//! lists — maps into GIMP's segment model without loss, and nothing else here is a superset of it.
//! Importers produce a [`Gradient`]; the renderer consumes the [`Lut`] one bakes into. No format
//! knowledge lives past the importer, and no GPU knowledge lives here at all.
//!
//! ⭐**Why a LUT and not a longer stop list.** `MAX_STOPS = 8` cannot express Fractint's 37 hard
//! jumps, and raising the ceiling only moves the wall. Baking makes a flat 256-band `.map`, a
//! curved GIMP blend and an HSV-sweep segment all cost the same at render time — one indexed
//! fetch — so the shader never learns what a blend function is.
//!
//! ⚠**Colours here are DISPLAY-referred**, like every other colour in this crate: the renderer
//! writes them straight into a non-sRGB framebuffer, so a channel value IS the byte the monitor
//! shows. See `srgb8_to_stop` for the measurement that settled it. Blending therefore happens in
//! gamma space, which is what the live view already does and what matching a reference render from
//! another application requires.

/// LUT length the renderer bakes to.
///
/// ⭐**1024, not 256, and the binding constraint is palette POSITION, not colour depth**
/// (`design/palette-import.md` §5a). Endpoints arrive at 8 bits but everything downstream is f32,
/// so colour quantisation costs at most ±1/255; what actually runs out is how finely the
/// continuous smooth-iteration value can address the palette once `cycle` sweeps it many times
/// across a narrow band of escape values. 1024 × 16 B = 16 KB, comfortably inside a uniform buffer.
pub const LUT_SIZE: usize = 1024;

/// How a segment interpolates between its endpoints — GIMP's five blend functions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Blend {
    #[default]
    Linear,
    Curved,
    Sine,
    SphereIncreasing,
    SphereDecreasing,
}

/// The space a segment blends in. HSV lets one segment sweep the long way round the hue wheel,
/// which is a thing `.ggr` files do and a thing RGB interpolation cannot express.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Space {
    #[default]
    Rgb,
    /// Hue increasing (counter-clockwise on the wheel).
    HsvCcw,
    /// Hue decreasing (clockwise).
    HsvCw,
}

/// One span of a gradient: `left..right` in `0..1`, RGBA endpoints, a blend function and a space.
///
/// `mid` shifts where the blend reaches 50% without adding a stop — GIMP's midpoint. It must lie
/// in `left..right`; [`Gradient::eval`] tolerates a degenerate one rather than panicking, because
/// these arrive from parsed files.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub left: f32,
    pub mid: f32,
    pub right: f32,
    pub left_color: [f32; 4],
    pub right_color: [f32; 4],
    pub blend: Blend,
    pub space: Space,
}

impl Segment {
    /// A linear RGB segment with the midpoint centred — what every format that is "just stops"
    /// (`.ugr`, swatch lists, our presets, the gradient editor) maps to.
    pub fn linear(left: f32, right: f32, left_color: [f32; 4], right_color: [f32; 4]) -> Self {
        Self {
            left,
            mid: 0.5 * (left + right),
            right,
            left_color,
            right_color,
            blend: Blend::Linear,
            space: Space::Rgb,
        }
    }

    /// A constant-colour segment — a band.
    ///
    /// ⭐⭐This is the one that makes `.map` import faithful. Fractint indexes a table with **no**
    /// interpolation, and the 37 hard jumps in `default.map` ARE the classic look; importing it as
    /// 255 linear segments smears them into something that no longer resembles the source. Flatness
    /// is a property of the SEGMENT, never a global switch, so a file can mix bands and ramps.
    pub fn flat(left: f32, right: f32, color: [f32; 4]) -> Self {
        Self::linear(left, right, color, color)
    }

    /// Both endpoints identical — the segment contributes no gradient, only a band.
    pub fn is_flat(&self) -> bool {
        self.left_color == self.right_color
    }

    /// Colour at `t`, which the caller has already established lies in `left..=right`.
    fn eval(&self, t: f32) -> [f32; 4] {
        // GIMP normalises position and midpoint into the segment's own 0..1 before applying the
        // blend function, so the same `mid` means the same thing in a wide and a narrow segment.
        let len = self.right - self.left;
        let (mid, pos) = if len < f32::EPSILON {
            (0.5, 0.5)
        } else {
            (((self.mid - self.left) / len).clamp(0.0, 1.0), ((t - self.left) / len).clamp(0.0, 1.0))
        };
        let f = match self.blend {
            Blend::Linear => linear_factor(mid, pos),
            Blend::Curved => curved_factor(mid, pos),
            Blend::Sine => {
                let p = linear_factor(mid, pos);
                (f32::sin(-std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * p) + 1.0) / 2.0
            }
            Blend::SphereIncreasing => {
                let p = linear_factor(mid, pos) - 1.0;
                (1.0 - p * p).max(0.0).sqrt()
            }
            Blend::SphereDecreasing => {
                let p = linear_factor(mid, pos);
                1.0 - (1.0 - p * p).max(0.0).sqrt()
            }
        };
        let (a, b) = (self.left_color, self.right_color);
        // Alpha is always a straight lerp; only the colour triple respects `space`.
        let alpha = a[3] + (b[3] - a[3]) * f;
        let rgb = match self.space {
            Space::Rgb => [
                a[0] + (b[0] - a[0]) * f,
                a[1] + (b[1] - a[1]) * f,
                a[2] + (b[2] - a[2]) * f,
            ],
            Space::HsvCcw | Space::HsvCw => {
                let (lh, ls, lv) = rgb_to_hsv([a[0], a[1], a[2]]);
                let (rh, rs, rv) = rgb_to_hsv([b[0], b[1], b[2]]);
                // Hue travels one way round the wheel, wrapping through 1.0 when it has to. This
                // is the whole point of the HSV modes: the SHORT way is what RGB already gives.
                let dh = match self.space {
                    Space::HsvCcw => {
                        if lh < rh { rh - lh } else { 1.0 - lh + rh }
                    }
                    _ => {
                        if lh > rh { -(lh - rh) } else { -(lh + 1.0 - rh) }
                    }
                };
                let h = (lh + dh * f).rem_euclid(1.0);
                hsv_to_rgb(h, ls + (rs - ls) * f, lv + (rv - lv) * f)
            }
        };
        [rgb[0], rgb[1], rgb[2], alpha]
    }
}

/// GIMP's linear factor: `pos` reaches 0.5 exactly at the midpoint, linearly on each side.
fn linear_factor(mid: f32, pos: f32) -> f32 {
    if pos <= mid {
        if mid < f32::EPSILON { 0.0 } else { 0.5 * pos / mid }
    } else {
        let rest = 1.0 - mid;
        if rest < f32::EPSILON { 1.0 } else { 0.5 + 0.5 * (pos - mid) / rest }
    }
}

/// GIMP's curved factor: `pos^(log 0.5 / log mid)`, i.e. a power curve that still passes through
/// 0.5 at the midpoint. Guarded at both ends because `log(0)` and `log(1)` are both fatal here.
fn curved_factor(mid: f32, pos: f32) -> f32 {
    const EPS: f32 = 1e-4;
    let m = mid.clamp(EPS, 1.0 - EPS);
    pos.max(0.0).powf(f32::ln(0.5) / f32::ln(m))
}

/// RGB (0..1) → HSV, hue in `0..1`. Matches GIMP's convention so `.ggr` HSV segments reproduce.
fn rgb_to_hsv(c: [f32; 3]) -> (f32, f32, f32) {
    let max = c[0].max(c[1]).max(c[2]);
    let min = c[0].min(c[1]).min(c[2]);
    let d = max - min;
    let h = if d <= 0.0 {
        0.0
    } else if max == c[0] {
        ((c[1] - c[2]) / d).rem_euclid(6.0) / 6.0
    } else if max == c[1] {
        ((c[2] - c[0]) / d + 2.0) / 6.0
    } else {
        ((c[0] - c[1]) / d + 4.0) / 6.0
    };
    (h, if max <= 0.0 { 0.0 } else { d / max }, max)
}

/// HSV (hue in `0..1`) → RGB (0..1).
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let s = s.clamp(0.0, 1.0);
    let h6 = h.rem_euclid(1.0) * 6.0;
    let i = h6.floor();
    let f = h6 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    match i as i32 % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// A named gradient: segments covering `0..1`, ascending and contiguous.
///
/// The constructors below all produce full coverage; a gradient assembled by hand need not, and
/// [`Gradient::eval`] then clamps to the nearest endpoint rather than inventing a colour.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gradient {
    pub name: String,
    pub segments: Vec<Segment>,
}

impl Gradient {
    /// From `(position, RGB)` stops — our presets, the gradient editor, `.ugr`.
    ///
    /// Stops are sorted, and coverage is extended to `0..1` with flat segments if the outermost
    /// stops do not reach the ends. ⚠That extension is a deliberate behaviour change from the old
    /// shader walk, which fell back to the FIRST stop's colour for any `t` past the LAST stop —
    /// a wrap-around no one asked for. Clamping to the nearest end is what every other gradient
    /// implementation does and what the editor's own preview implies.
    pub fn from_stops(name: impl Into<String>, stops: &[(f32, [f32; 3])]) -> Self {
        let mut s: Vec<(f32, [f32; 3])> = stops.to_vec();
        s.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut segments = Vec::new();
        match s.len() {
            0 => segments.push(Segment::flat(0.0, 1.0, [0.0, 0.0, 0.0, 1.0])),
            1 => segments.push(Segment::flat(0.0, 1.0, rgba(s[0].1))),
            _ => {
                if s[0].0 > 0.0 {
                    segments.push(Segment::flat(0.0, s[0].0, rgba(s[0].1)));
                }
                for w in s.windows(2) {
                    // A zero-width span would divide by zero in `eval`; drop it and keep the
                    // later colour, which is what a sorted duplicate position means.
                    if w[1].0 > w[0].0 {
                        segments.push(Segment::linear(w[0].0, w[1].0, rgba(w[0].1), rgba(w[1].1)));
                    }
                }
                let last = s[s.len() - 1];
                if last.0 < 1.0 {
                    segments.push(Segment::flat(last.0, 1.0, rgba(last.1)));
                }
            }
        }
        Self { name: name.into(), segments }
    }

    /// From the GPU's packed `[r, g, b, pos]` rows plus the active count.
    ///
    /// That is the shape the old eight-stop uniform used, and it is still what the random-palette
    /// animator and the gradient editor produce, so this is the adapter that lets them keep their
    /// own representation while everything downstream sees one model. Rows past `n` are ignored —
    /// the packed array repeats its last real stop into the unused slots.
    pub fn from_packed(name: impl Into<String>, packed: &[[f32; 4]], n: u32) -> Self {
        let n = (n as usize).clamp(1, packed.len().max(1)).min(packed.len());
        let stops: Vec<(f32, [f32; 3])> =
            packed[..n].iter().map(|s| (s[3], [s[0], s[1], s[2]])).collect();
        Self::from_stops(name, &stops)
    }

    /// From an ordered colour list with no positions — swatch lists, pasted hex, `.ase` / `.cs`.
    /// Evenly spaced, linearly blended.
    pub fn from_colors(name: impl Into<String>, colors: &[[f32; 3]]) -> Self {
        if colors.len() < 2 {
            return Self::from_stops(name, &colors.iter().map(|c| (0.0, *c)).collect::<Vec<_>>());
        }
        let n = colors.len();
        let stops: Vec<(f32, [f32; 3])> = colors
            .iter()
            .enumerate()
            .map(|(i, c)| (i as f32 / (n - 1) as f32, *c))
            .collect();
        Self::from_stops(name, &stops)
    }

    /// From an ordered colour list read as a **lookup table** — one flat band per entry, no
    /// interpolation. This is Fractint/KF `.map` semantics, and the reason [`Segment::flat`]
    /// exists.
    pub fn from_bands(name: impl Into<String>, colors: &[[f32; 3]]) -> Self {
        if colors.is_empty() {
            return Self::from_stops(name, &[]);
        }
        let n = colors.len();
        let segments = colors
            .iter()
            .enumerate()
            .map(|(i, c)| Segment::flat(i as f32 / n as f32, (i + 1) as f32 / n as f32, rgba(*c)))
            .collect();
        Self { name: name.into(), segments }
    }

    /// The same gradient shifted along the position axis by `by`, wrapping at the ends.
    ///
    /// Ultra Fractal stores a `rotation=` with each gradient, and a palette is cycled anyway, so
    /// this is a rotation of a RING rather than a slide of a strip: a segment that ends up
    /// straddling the seam is **split in two** and the halves re-evaluated, so no colour is lost
    /// and no flat clamp is invented at the ends. Sorting alone would have silently dropped the
    /// straddling segment's far half.
    ///
    /// The gradient must cover `0..1` for the result to (`from_stops` and friends guarantee it);
    /// an uncovered one is returned unchanged rather than rotated into nonsense.
    pub fn rotated(&self, by: f32) -> Self {
        let by = if by.is_finite() { by.rem_euclid(1.0) } else { 0.0 };
        if by == 0.0 || self.segments.is_empty() {
            return self.clone();
        }
        let covers = self.segments[0].left <= 0.0
            && self.segments[self.segments.len() - 1].right >= 1.0;
        if !covers {
            return self.clone();
        }
        let mut out: Vec<Segment> = Vec::with_capacity(self.segments.len() + 1);
        for seg in &self.segments {
            let (l, r) = (seg.left + by, seg.right + by);
            if r <= 1.0 {
                out.push(Self::shifted(seg, by, seg.left, seg.right));
            } else if l >= 1.0 {
                out.push(Self::shifted(seg, by - 1.0, seg.left, seg.right));
            } else {
                // Straddles the seam: keep [left, cut) where it is and wrap [cut, right) to the
                // front, evaluating the split colour so the join is exact.
                let cut = seg.left + (1.0 - l);
                out.push(Self::shifted(seg, by, seg.left, cut));
                out.push(Self::shifted(seg, by - 1.0, cut, seg.right));
            }
        }
        out.sort_by(|a, b| a.left.total_cmp(&b.left));
        Self { name: self.name.clone(), segments: out }
    }

    /// One segment's `[from, to]` sub-span, moved by `by`. The endpoint colours are re-evaluated
    /// at the cut so a split segment's halves meet exactly; blend and space carry over, and the
    /// midpoint is re-centred because a partial span no longer has the original's midpoint in it.
    fn shifted(seg: &Segment, by: f32, from: f32, to: f32) -> Segment {
        let (a, b) = (seg.eval(from), seg.eval(to));
        let (l, r) = (from + by, to + by);
        let whole = from <= seg.left && to >= seg.right;
        Segment {
            left: l,
            mid: if whole { seg.mid + by } else { 0.5 * (l + r) },
            right: r,
            left_color: a,
            right_color: b,
            blend: if whole { seg.blend } else { Blend::Linear },
            space: if whole { seg.space } else { Space::Rgb },
        }
    }

    /// Colour at `t`. Outside the covered range, the nearest endpoint colour; `t` is clamped to
    /// `0..1` first (the caller has already taken `fract`).
    pub fn eval(&self, t: f32) -> [f32; 4] {
        let Some(first) = self.segments.first() else {
            return [0.0, 0.0, 0.0, 1.0];
        };
        let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
        if t <= first.left {
            return first.left_color;
        }
        for seg in &self.segments {
            if t <= seg.right {
                // A gap between segments (only possible in a hand-assembled gradient) resolves to
                // this segment's left colour, which is the nearest covered value.
                return if t >= seg.left { seg.eval(t) } else { seg.left_color };
            }
        }
        self.segments[self.segments.len() - 1].right_color
    }

    /// Back to `(position, RGB)` stops — each segment's left edge, plus the last segment's right.
    ///
    /// ⭐**A hard jump between two segments becomes a DUPLICATE POSITION**, which is how every
    /// gradient editor expresses an edge and what [`Self::from_stops`] reads back (a zero-width
    /// span contributes no segment, so the colour simply changes there). Without it a rotated
    /// gradient's seam — a real discontinuity, since a palette that is not seamless has one —
    /// would be quietly smoothed into a ramp across a whole segment. That was caught by the
    /// round-trip test, not by reading the code.
    ///
    /// ⚠**Exact only for a linear-RGB gradient with centred midpoints**, which is what
    /// [`Self::from_stops`], the `.ugr` importer and [`Self::rotated`] produce. A `.ggr` with
    /// curved blends, HSV sweeps or shifted midpoints is a SUPERSET of a stop list, so this drops
    /// what a stop list cannot hold. It exists because the app persists a custom palette as stops;
    /// when `.ggr` lands, that stored shape has to grow, and this is the seam where it will show.
    pub fn to_stops(&self) -> Vec<(f32, [f32; 3])> {
        let rgb = |c: [f32; 4]| [c[0], c[1], c[2]];
        let mut out: Vec<(f32, [f32; 3])> = Vec::with_capacity(self.segments.len() + 1);
        for (i, seg) in self.segments.iter().enumerate() {
            if i > 0 {
                let prev = &self.segments[i - 1];
                if prev.right_color != seg.left_color {
                    out.push((seg.left, rgb(prev.right_color)));
                }
            }
            out.push((seg.left, rgb(seg.left_color)));
        }
        if let Some(last) = self.segments.last() {
            out.push((last.right, rgb(last.right_color)));
        }
        out
    }

    /// Every segment is a band — the whole gradient is a lookup table with no ramps.
    /// [`Gradient::bake`] turns this into the LUT's `smooth` flag; see [`Lut`].
    pub fn is_flat(&self) -> bool {
        !self.segments.is_empty() && self.segments.iter().all(Segment::is_flat)
    }

    /// Evaluate into an `n`-entry table.
    ///
    /// ⭐**Entry `i` is the gradient at `(i + 0.5) / n`** — texel-centre sampling, one convention
    /// for both fetch modes. The renderer's smooth fetch is therefore `x = fract(t) * n - 0.5`
    /// with the index wrapping mod `n`, and its flat fetch is `floor(fract(t) * n)`. Sampling at
    /// `i / (n - 1)` instead would make the flat fetch land half a band off and quietly shift
    /// every `.map` import by one entry.
    pub fn bake(&self, n: usize) -> Lut {
        let n = n.max(1);
        let entries = (0..n)
            .map(|i| self.eval((i as f32 + 0.5) / n as f32))
            .collect();
        Lut { entries, smooth: !self.is_flat() }
    }
}

/// An RGB triple as RGBA with opaque alpha — the internal colour shape is RGBA because `.ggr`
/// carries per-endpoint alpha and dropping it at import would be unrecoverable.
fn rgba(c: [f32; 3]) -> [f32; 4] {
    [c[0], c[1], c[2], 1.0]
}

/// A baked palette: `entries.len()` colours plus how the renderer should fetch between them.
///
/// ⭐⭐**`smooth` is not a style preference, it is fidelity.** Interpolating between entries is what
/// gives the palette position resolution a deep view needs at a high `cycle`; nearest-fetching is
/// what keeps a `.map`'s bands hard. With 256 bands baked into 1024 entries, interpolating would
/// put a ramp across a QUARTER of the palette — the exact smear this whole design exists to avoid.
#[derive(Clone, Debug, PartialEq)]
pub struct Lut {
    pub entries: Vec<[f32; 4]>,
    pub smooth: bool,
}

impl Lut {
    /// Sample the way the shader will, so a CPU-side check and the GPU agree by construction.
    pub fn sample(&self, t: f32) -> [f32; 4] {
        let n = self.entries.len();
        if n == 0 {
            return [0.0, 0.0, 0.0, 1.0];
        }
        let t = if t.is_finite() { t.rem_euclid(1.0) } else { 0.0 };
        if !self.smooth {
            return self.entries[((t * n as f32) as usize).min(n - 1)];
        }
        let x = t * n as f32 - 0.5;
        let i = x.floor();
        let f = x - i;
        // Wrapping (not clamping) at the seam: palettes are cycled with `fract`, so t = 1 and
        // t = 0 are adjacent on screen. A seamless palette blends invisibly; a non-seamless one
        // gets a one-entry ramp where it used to get a hard jump — 1/1024 of the sweep.
        let a = self.entries[(i as i64).rem_euclid(n as i64) as usize];
        let b = self.entries[(i as i64 + 1).rem_euclid(n as i64) as usize];
        [
            a[0] + (b[0] - a[0]) * f,
            a[1] + (b[1] - a[1]) * f,
            a[2] + (b[2] - a[2]) * f,
            a[3] + (b[3] - a[3]) * f,
        ]
    }
}

#[cfg(test)]
mod segment_tests;
