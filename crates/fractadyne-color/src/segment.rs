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

/// How a segment interpolates between its endpoints — GIMP's five blend functions, plus ours.
///
/// ⭐**Kinds 0–4 are GIMP's and are FROZEN**; kind 5 is a cubic-Bézier ease, the parametric one
/// (`design/gradient-curves.md` §8). 6–31 are reserved and deliberately empty: flexibility comes
/// from a continuous PARAMETER, not from a longer list of names nobody can tell apart.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Blend {
    #[default]
    Linear,
    Curved,
    Sine,
    SphereIncreasing,
    SphereDecreasing,
    /// A CSS-style cubic-Bézier ease: the two interior control points `[x1, y1, x2, y2]` of a
    /// curve from `(0,0)` to `(1,1)`.
    ///
    /// ⭐**The payload rides in the enum rather than in [`Segment`]** so that `from_u8`/`as_u8`
    /// keep carrying the KIND alone — the file-format number stays exactly what it was, and the
    /// four floats travel beside it in `PaletteSegment::blend_params`.
    Bezier([f32; 4]),
}

/// A cubic-Bézier ease that is exactly the identity, `y = x`, with both handles visible and
/// grabbable at the thirds — what "convert this to an editable curve" starts from.
///
/// ⚠With `x1 = 1/3` and `x2 = 2/3` the parameterisation is exact: `x(u) = u`. That is what makes
/// the fit in `Blend::fit_to` a closed form rather than an optimisation.
pub const BEZIER_IDENTITY: [f32; 4] = [1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0];

impl Blend {
    /// GIMP's `.ggr` blend-type number, which is also what a session file stores.
    ///
    /// ⚠These numbers are a FILE FORMAT, not an internal detail — they appear in `.ggr` files
    /// written by other applications and in our own saved sessions. Reordering the enum without
    /// changing this mapping would silently re-interpret every stored gradient.
    /// ⚠**Append only.** A `.ggr` never contains 5, so import is unaffected; our own sessions may.
    pub fn from_u8(v: u8) -> Self {
        Self::from_u8_params(v, BEZIER_IDENTITY)
    }

    /// The kind number plus the parameters that travel beside it.
    ///
    /// ⚠An all-zero `params` — what `#[serde(default)]` produces for a session written before
    /// kind 5 existed — is *also* the identity (`x(u) = y(u) = u³`, so `y = x`), so a stored
    /// kind 5 with no parameters degrades to a straight line rather than to something arbitrary.
    pub fn from_u8_params(v: u8, params: [f32; 4]) -> Self {
        match v {
            1 => Blend::Curved,
            2 => Blend::Sine,
            3 => Blend::SphereIncreasing,
            4 => Blend::SphereDecreasing,
            5 => Blend::Bezier(params),
            _ => Blend::Linear,
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Blend::Linear => 0,
            Blend::Curved => 1,
            Blend::Sine => 2,
            Blend::SphereIncreasing => 3,
            Blend::SphereDecreasing => 4,
            Blend::Bezier(_) => 5,
        }
    }

    /// The four control-point coordinates to store beside the kind. Kinds 0–4 have none, and
    /// report the identity so a round trip through storage never invents a curve.
    pub fn params(self) -> [f32; 4] {
        match self {
            Blend::Bezier(p) => p,
            _ => BEZIER_IDENTITY,
        }
    }

    /// Approximate any blend as a cubic-Bézier ease — what the editor's "make this editable" does.
    ///
    /// ⚠**Approximate, and the editor must say so.** Sine and the two spheres are not cubics, so
    /// no choice of control points reproduces them. ⭐⭐**The spheres are the bad case and it is
    /// worth knowing why**: `sqrt(1 - (p-1)²)` has a VERTICAL TANGENT at one end, and no cubic
    /// with finite control points has infinite slope — so the error there is structural, not a
    /// matter of fitting harder. Measured worst-case error is pinned by
    /// `fitting_a_bezier_is_exact_for_linear_and_close_for_the_rest`.
    ///
    /// Pinning `x1 = 1/3`, `x2 = 2/3` makes `x(u) = u` exactly, so the curve is LINEAR in
    /// `(y1, y2)` and the best fit is a 2×2 normal-equation solve — a closed form, not a search.
    /// ⚠**Least squares over the whole span, not interpolation at two points.** Interpolating
    /// `t = 1/3` and `t = 2/3` is a one-liner and was the first version; it nails those two
    /// samples and lets the error between them run to 0.15 on a sphere blend. Fitting all of it
    /// halves that, for the same closed form.
    /// ⭐For a blend that IS a cubic in this family — `Linear`, and `Curved` at a centred midpoint —
    /// the model contains the answer, so the fit is exact and converting moves no pixels.
    pub fn fit_to(sample: impl Fn(f32) -> f32) -> [f32; 4] {
        const N: usize = 64;
        let (mut saa, mut sab, mut sbb, mut sa, mut sb) = (0.0_f64, 0.0, 0.0, 0.0, 0.0);
        for k in 1..N {
            let t = k as f32 / N as f32;
            let m = 1.0 - t;
            // The two free Bernstein weights, and the fixed cubic term the endpoints contribute.
            let (a, b, c) = (3.0 * m * m * t, 3.0 * m * t * t, t * t * t);
            let r = f64::from(sample(t) - c);
            saa += f64::from(a) * f64::from(a);
            sab += f64::from(a) * f64::from(b);
            sbb += f64::from(b) * f64::from(b);
            sa += f64::from(a) * r;
            sb += f64::from(b) * r;
        }
        let det = saa * sbb - sab * sab;
        if det.abs() < 1.0e-12 {
            return BEZIER_IDENTITY;
        }
        let y1 = ((sa * sbb - sb * sab) / det) as f32;
        let y2 = ((sb * saa - sa * sab) / det) as f32;
        [1.0 / 3.0, y1, 2.0 / 3.0, y2]
    }

    /// [`Self::fit_to`] plus the worst-case error of the result, so the editor can say **how**
    /// approximate a conversion is before the user commits to it.
    ///
    /// ⭐⭐**The number is not decoration — it varies by an order of magnitude across the five
    /// kinds.** Measured: `Linear` 0, `Curved` and `Sine` under 0.01, and the two SPHERE blends
    /// **~0.14** — because `sqrt(1 - (p-1)²)` has a vertical tangent that no cubic can have. A
    /// single "this is approximate" warning would put those two in the same sentence as a
    /// conversion that is exact, which is the kind of hedge that trains people to ignore warnings.
    pub fn fit_with_error(sample: impl Fn(f32) -> f32) -> ([f32; 4], f32) {
        let p = Self::fit_to(&sample);
        let worst = (0..=64)
            .map(|k| {
                let t = k as f32 / 64.0;
                (bezier_ease(p, t) - sample(t)).abs()
            })
            .fold(0.0_f32, f32::max);
        (p, worst)
    }
}

/// One coordinate of a cubic Bézier from 0 to 1 with interior control values `a`, `b`, at `u`.
fn bez(a: f32, b: f32, u: f32) -> f32 {
    let m = 1.0 - u;
    3.0 * m * m * u * a + 3.0 * m * u * u * b + u * u * u
}

/// Its derivative with respect to `u` — Newton's step needs it.
fn bez_slope(a: f32, b: f32, u: f32) -> f32 {
    let m = 1.0 - u;
    3.0 * m * m * a + 6.0 * m * u * (b - a) + 3.0 * u * u * (1.0 - b)
}

/// The cubic-Bézier ease `y` at `t`, the CSS `cubic-bezier(x1, y1, x2, y2)` function.
///
/// ⭐⭐**The `x(u) = t` solve costs nothing at render time.** It runs once per LUT entry — 1024
/// times per bake — and zero times per pixel, which is the same argument that made every other
/// feature in this model free. ⛔A solver in the shader would be the wrong shape entirely.
///
/// ⚠`x1` and `x2` are clamped into `0..1`, which is what guarantees `x(u)` is monotone and the
/// solve therefore has exactly one answer. `y1`/`y2` are deliberately NOT clamped: a curve that
/// overshoots is a legitimate effect, and it is the COLOUR that gets clamped, not the factor —
/// clamping here would silently flatten the handle the user is dragging.
pub fn bezier_ease(p: [f32; 4], t: f32) -> f32 {
    let (x1, x2) = (p[0].clamp(0.0, 1.0), p[2].clamp(0.0, 1.0));
    let (y1, y2) = (p[1], p[3]);
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    // Newton from u = t, which is the exact answer whenever x is the identity — the common case.
    let mut u = t;
    for _ in 0..8 {
        let dx = bez(x1, x2, u) - t;
        if dx.abs() < 1.0e-6 {
            return bez(y1, y2, u);
        }
        let slope = bez_slope(x1, x2, u);
        if slope.abs() < 1.0e-6 {
            break;
        }
        let next = u - dx / slope;
        if !(0.0..=1.0).contains(&next) {
            break;
        }
        u = next;
    }
    // Bisection is the fallback rather than the primary because Newton converges in two or three
    // steps here; it is what makes a flat or near-flat region terminate at all.
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    let mut u = t;
    for _ in 0..40 {
        let x = bez(x1, x2, u);
        if (x - t).abs() < 1.0e-6 {
            break;
        }
        if x < t {
            lo = u;
        } else {
            hi = u;
        }
        u = 0.5 * (lo + hi);
    }
    bez(y1, y2, u)
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

impl Space {
    /// GIMP's `.ggr` colouring-type number — a file format, like [`Blend::from_u8`].
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Space::HsvCcw,
            2 => Space::HsvCw,
            _ => Space::Rgb,
        }
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Space::Rgb => 0,
            Space::HsvCcw => 1,
            Space::HsvCw => 2,
        }
    }
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

    /// How far along the blend `t` sits: `0` at the left endpoint, `1` at the right.
    ///
    /// ⭐**This is the curve itself, and the editor's preview must draw THIS** rather than an
    /// idealised shape. Two of GIMP's five are deliberately not midpoint-symmetric — a
    /// sphere-increasing segment reads **0.866** at its halfway point, not 0.5 — so a prettified
    /// preview would misrepresent them. Exposing the real function makes the preview correct by
    /// construction instead of by a second implementation that can drift.
    pub fn factor(&self, t: f32) -> f32 {
        // GIMP normalises position and midpoint into the segment's own 0..1 before applying the
        // blend function, so the same `mid` means the same thing in a wide and a narrow segment.
        let len = self.right - self.left;
        let (mid, pos) = if len < f32::EPSILON {
            (0.5, 0.5)
        } else {
            (((self.mid - self.left) / len).clamp(0.0, 1.0), ((t - self.left) / len).clamp(0.0, 1.0))
        };
        match self.blend {
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
            // ⭐⭐**Kind 5 IGNORES the midpoint, deliberately** — note `pos`, not
            // `linear_factor(mid, pos)`. `mid` pre-warps the input of every GIMP blend, and a
            // Bézier's own handles already say where the curve reaches halfway; composing the two
            // would give two knobs for one shape and make the handles lie about where the curve
            // goes. The stored `mid` is preserved untouched, so switching back to kinds 0–4
            // restores it. `bezier_ignores_the_midpoint` pins this.
            Blend::Bezier(p) => bezier_ease(p, pos),
        }
    }

    /// Does this segment blend through hue with an endpoint that has no hue to blend from?
    ///
    /// ⭐⭐**The trap a user cannot see coming.** `rgb_to_hsv` reports hue 0 for anything
    /// unsaturated, and an HSV segment whose endpoint hues are equal takes a FULL turn of the
    /// wheel — so a **black → red** segment passes through green and blue on the way, measured
    /// green-dominant across a third of its span. The two swatches give no hint of it. The editor
    /// uses this to warn rather than to forbid: the effect is legitimate and sometimes wanted.
    pub fn hue_undefined_endpoint(&self) -> bool {
        if self.space == Space::Rgb {
            return false;
        }
        let sat = |c: [f32; 4]| {
            let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
            if mx <= 0.0 { 0.0 } else { (mx - mn) / mx }
        };
        sat(self.left_color) < 1e-3 || sat(self.right_color) < 1e-3
    }

    /// Colour at `t`, which the caller has already established lies in `left..=right`.
    fn eval(&self, t: f32) -> [f32; 4] {
        let f = self.factor(t);
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
        // ⭐**The clamp that lets a Bézier overshoot safely.** `factor` may return outside `0..1`
        // for kind 5 — that is the point of allowing the handles out of the box, and it
        // extrapolates past an endpoint colour. Clamping HERE rather than in `factor` keeps the
        // curve the user drew intact while guaranteeing an in-gamut colour.
        // ⚠**A no-op for kinds 0–4**, whose factor is always in `0..1` between in-range endpoints —
        // which is what keeps this change at zero drift for every existing gradient.
        let c = |v: f32| v.clamp(0.0, 1.0);
        [c(rgb[0]), c(rgb[1]), c(rgb[2]), c(alpha)]
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

    /// Does a plain stop list describe this gradient exactly — i.e. would [`Self::to_stops`] be
    /// lossless here?
    ///
    /// ⭐⭐**The predicate the editor's "Convert to editable stops" notice needs.** That notice
    /// used to fire on "the gradient has segments", which meant "it came from a `.ggr`" only until
    /// P1 made the editor segment-native — after which EVERY custom gradient has segments, so it
    /// fired for one the user had just copied from a preset and offered to convert it into what it
    /// already was. What actually matters is whether any segment carries something a stop list
    /// cannot hold: a non-linear blend, a hue sweep, or an off-centre midpoint.
    ///
    /// ⚠The midpoint test is a TOLERANCE, not equality. A centred midpoint is stored as
    /// `left + 0.5 * (right - left)` and survives f32 arithmetic, a session round trip and a
    /// segment being re-spanned by a neighbouring drag; an `==` here would report a plain dragged
    /// gradient as rich for the sake of one ulp.
    pub fn is_stop_expressible(&self) -> bool {
        self.segments.iter().all(|s| {
            let span = s.right - s.left;
            let centred =
                span.abs() < f32::EPSILON || ((s.mid - s.left) / span - 0.5).abs() <= 1.0e-3;
            s.blend == Blend::Linear && s.space == Space::Rgb && centred
        })
    }

    /// Does the colour at `1.0` already equal the colour at `0.0`?
    ///
    /// ⭐**The property that decides whether a CYCLED palette has a visible seam.** The renderer
    /// takes `fract()` of the palette coordinate, so `t = 1` and `t = 0` are adjacent pixels on
    /// screen; if the two ends differ, every sweep shows a hard edge there. On a bar the two ends
    /// are as far apart as they can be, which is why this is easy to miss and why the ring view
    /// exists.
    pub fn is_seamless(&self) -> bool {
        match (self.segments.first(), self.segments.last()) {
            (Some(a), Some(b)) => a.left_color == b.right_color,
            _ => true,
        }
    }

    /// Force the end to match the start, so a cycled palette has no seam.
    ///
    /// ⚠**The END moves, not the start.** Position 0 is where the eye lands first and where a
    /// preset's defining colour usually sits, so pulling the start toward the end would change the
    /// gradient's identity to fix its join. Moving the end is the smaller edit and the reversible
    /// one — the user can always recolour it afterwards.
    /// ⚠Alpha travels with it: a seam in opacity is a seam.
    pub fn make_seamless(&mut self) {
        let Some(&first) = self.segments.first() else {
            return;
        };
        if let Some(last) = self.segments.last_mut() {
            last.right_color = first.left_color;
        }
    }

    // ── Editing ─────────────────────────────────────────────────────────────────────────────
    //
    // ⭐**The gradient editor edits SEGMENTS, and a "stop" is a segment boundary.** Before P1 the
    // editor owned a flat `[pos, r, g, b]` list and every edit DESTROYED the segment gradient
    // beside it, so a curve, a colour space or a midpoint could only ever arrive by importing
    // somebody else's `.ggr`. These operations are the replacement: each one preserves the blend
    // function, the colour space and the midpoint FRACTION of every segment it does not remove.
    //
    // ⭐⭐**Zero drift is the contract.** On a gradient of `Linear`/`Rgb` segments with centred
    // midpoints — which is exactly what `from_stops` produces, i.e. every preset, every pasted
    // palette and every session written before this — each operation gives the same gradient
    // `from_stops` would give for the edited stop list. `edit_tests.rs` asserts that directly.

    /// Editable stops = segment boundaries. `N` segments have `N + 1`.
    pub fn stop_count(&self) -> usize {
        if self.segments.is_empty() { 0 } else { self.segments.len() + 1 }
    }

    /// Stop `i` as `(position, RGB)`.
    ///
    /// ⚠For a gradient with a hard edge the two sides of a boundary hold different colours; this
    /// reports the LEFT-hand one (the colour arriving at the boundary), which is what a strip marker
    /// sits on. [`Self::to_stops`] is the lossless view and emits both.
    pub fn stop(&self, i: usize) -> Option<(f32, [f32; 3])> {
        let rgb = |c: [f32; 4]| [c[0], c[1], c[2]];
        match (i, self.segments.len()) {
            (_, 0) => None,
            (0, _) => Some((self.segments[0].left, rgb(self.segments[0].left_color))),
            (i, n) if i == n => {
                let s = &self.segments[n - 1];
                Some((s.right, rgb(s.right_color)))
            }
            (i, n) if i < n => Some((self.segments[i].left, rgb(self.segments[i].left_color))),
            _ => None,
        }
    }

    /// Recolour stop `i`, updating both segments that meet there.
    ///
    /// ⚠This deliberately CLOSES a hard edge at that boundary: setting one colour on a stop the user
    /// sees as one marker must not leave the other side untouched, or the swatch would disagree with
    /// the picture. Splitting a boundary back into two colours is a separate, explicit action.
    pub fn set_stop_color(&mut self, i: usize, rgb: [f32; 3]) {
        let n = self.segments.len();
        if n == 0 || i > n {
            return;
        }
        let c = |a: f32| [rgb[0], rgb[1], rgb[2], a];
        if i > 0 {
            let alpha = self.segments[i - 1].right_color[3];
            self.segments[i - 1].right_color = c(alpha);
        }
        if i < n {
            let alpha = self.segments[i].left_color[3];
            self.segments[i].left_color = c(alpha);
        }
    }

    /// Move stop `i` to `pos`, clamped strictly inside its neighbours.
    ///
    /// ⭐**Midpoint FRACTIONS are preserved, not midpoint positions.** A segment whose midpoint sits
    /// halfway must still sit halfway after its span moves — otherwise dragging a neighbouring stop
    /// would silently reshape a segment the user did not touch, and a centred (default) segment
    /// would stop matching what `from_stops` produces.
    /// ⚠The two end stops (0 and `N`) do not move: the gradient covers `0..1` by contract, and a
    /// gradient that stopped short would render a flat clamp at the end rather than the colour the
    /// user placed there.
    pub fn set_stop_position(&mut self, i: usize, pos: f32) {
        let n = self.segments.len();
        if n == 0 || i == 0 || i >= n || !pos.is_finite() {
            return;
        }
        const EPS: f32 = 1e-4;
        let lo = self.segments[i - 1].left + EPS;
        let hi = self.segments[i].right - EPS;
        if hi <= lo {
            return;
        }
        let pos = pos.clamp(lo, hi);
        let (l, r) = (self.segments[i - 1].left, self.segments[i].right);
        set_span(&mut self.segments[i - 1], l, pos);
        set_span(&mut self.segments[i], pos, r);
    }

    /// Insert a stop at `pos`, splitting the segment that contains it. Returns the new stop index.
    ///
    /// Both halves inherit the parent's blend and colour space, and the split colour is EVALUATED so
    /// the boundary joins exactly.
    /// ⚠**Exact for a `Linear`/`Rgb` segment; approximate for a curved or HSV one** — half of a
    /// curve is not the same curve, so re-fitting the parent's blend to each half changes the shape
    /// between the new stops. That is inherent to splitting a parametric segment, and the editor
    /// should say so rather than pretend; a linear segment (the default everywhere) is unaffected.
    pub fn insert_stop(&mut self, pos: f32) -> Option<usize> {
        const EPS: f32 = 1e-4;
        if !pos.is_finite() {
            return None;
        }
        let idx = self
            .segments
            .iter()
            .position(|s| pos > s.left + EPS && pos < s.right - EPS)?;
        let parent = self.segments[idx];
        let split = parent.eval(pos);
        let (mut a, mut b) = (parent, parent);
        a.right_color = split;
        b.left_color = split;
        set_span(&mut a, parent.left, pos);
        set_span(&mut b, pos, parent.right);
        self.segments[idx] = a;
        self.segments.insert(idx + 1, b);
        Some(idx + 1)
    }

    /// Remove interior stop `i`, merging the two segments that meet there into one.
    ///
    /// ⚠**The LEFT segment's blend and colour space win**, and the merged midpoint keeps the left
    /// segment's fraction. Something has to, and choosing the left matches reading order; the
    /// alternative (whichever segment is wider) is defensible and would be a surprise, since the
    /// answer would change as the user dragged a neighbour.
    /// ⚠End stops cannot be removed — see [`Self::set_stop_position`].
    pub fn remove_stop(&mut self, i: usize) {
        let n = self.segments.len();
        if n < 2 || i == 0 || i >= n {
            return;
        }
        let right = self.segments[i];
        let left = &mut self.segments[i - 1];
        let frac = span_fraction(left);
        left.right_color = right.right_color;
        let (l, r) = (left.left, right.right);
        left.left = l;
        left.right = r;
        left.mid = l + frac * (r - l);
        self.segments.remove(i);
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

/// Where the midpoint sits inside a segment, as a fraction of its span. `0.5` is centred, which is
/// what every default segment is. A zero-width span reports centred rather than dividing by zero.
fn span_fraction(seg: &Segment) -> f32 {
    let len = seg.right - seg.left;
    if len.abs() < f32::EPSILON {
        0.5
    } else {
        ((seg.mid - seg.left) / len).clamp(0.0, 1.0)
    }
}

/// Move a segment to a new span, keeping its midpoint at the same FRACTION of it. See
/// [`Gradient::set_stop_position`] for why the fraction rather than the position is the invariant.
fn set_span(seg: &mut Segment, left: f32, right: f32) {
    let frac = span_fraction(seg);
    seg.left = left;
    seg.right = right;
    seg.mid = left + frac * (right - left);
}

/// An RGB triple as RGBA with opaque alpha — the internal colour shape is RGBA because `.ggr`
/// carries per-endpoint alpha and dropping it at import would be unrecoverable.
fn rgba(c: [f32; 3]) -> [f32; 4] {
    [c[0], c[1], c[2], 1.0]
}


/// Write a gradient as a GIMP `.ggr` file — the interchange format for "save and share".
///
/// ⭐**The internal model IS GIMP's**, so this is a formatter rather than a conversion: every
/// midpoint, blend curve and colour space survives, which is what makes a saved gradient openable
/// in GIMP, Krita, Inkscape and back in here without loss.
///
/// ⚠⚠**Blend kind 5 (our Bézier) has no `.ggr` number, and this is where that costs something.**
/// It is written as GIMP's nearest expressible curve, and [`ggr_lossy_segments`] counts which
/// segments were approximated so the UI can say so.
/// ⭐**An IDENTITY Bézier is exempt and is written as plain linear**, because it is linear to
/// within **~1e-4** — a quarter of the 1/255 the output can even express. ⚠Not *bit*-identical:
/// the ease solves `x(u) = t` numerically, so it lands within a rounding error of the straight
/// line rather than on it. Visually exact, which is the property that matters here; the tests
/// compare with a tolerance for exactly this reason.
/// Without the exemption a gradient nobody had bent would still be reported as lossy, and a
/// warning that fires when nothing was lost is a warning people learn to skip.
pub fn write_ggr(g: &Gradient) -> String {
    let mut out = String::from("GIMP Gradient\n");
    let name = if g.name.trim().is_empty() { "Fractadyne" } else { g.name.trim() };
    // ⚠A newline in a name would forge a segment count line. GIMP reads `Name:` to end of line.
    let name: String = name.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    out.push_str(&format!("Name: {name}\n"));
    out.push_str(&format!("{}\n", g.segments.len()));
    for s in &g.segments {
        let c = |v: [f32; 4]| {
            format!("{:.6} {:.6} {:.6} {:.6}", v[0], v[1], v[2], v[3])
        };
        out.push_str(&format!(
            "{:.6} {:.6} {:.6} {} {} {} {}\n",
            s.left,
            s.mid,
            s.right,
            c(s.left_color),
            c(s.right_color),
            ggr_blend_number(s.blend),
            s.space.as_u8(),
        ));
    }
    out
}

/// The `.ggr` blend number to write for a blend, approximating anything GIMP has no number for.
///
/// ⚠Kinds 0–4 are GIMP's own and pass through. Kind 5 becomes **linear** — not because linear is a
/// good fit for an arbitrary cubic, but because it is the only choice that is exactly right for the
/// identity case and honestly neutral for the rest; guessing "curved" would claim a shape the file
/// does not carry.
fn ggr_blend_number(b: Blend) -> u8 {
    match b {
        Blend::Bezier(_) => 0,
        other => other.as_u8(),
    }
}

/// How many segments `write_ggr` would have to approximate, for the UI to warn with.
///
/// ⭐An identity Bézier does not count: it round-trips exactly. See [`write_ggr`].
pub fn ggr_lossy_segments(g: &Gradient) -> usize {
    g.segments
        .iter()
        .filter(|s| match s.blend {
            Blend::Bezier(p) => !bezier_is_identity(p),
            _ => false,
        })
        .count()
}

/// Is this Bézier the straight line, to within what the format's 6 decimal places could record?
pub fn bezier_is_identity(p: [f32; 4]) -> bool {
    (0..=16).all(|k| {
        let t = k as f32 / 16.0;
        (bezier_ease(p, t) - t).abs() < 1.0e-4
    })
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
mod edit_tests;
#[cfg(test)]
mod segment_tests;
