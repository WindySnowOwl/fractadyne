//! The coloring matrix (`--coloring-matrix`, design/coloring-plan.md B1): every coloring method,
//! with its variants, across the fractal families and depth bands of `validation/coloring/views.toml`.
//!
//! Each cell renders as an export renders it (`current_export_request_for`, `render_export`) and is
//! measured:
//! - the picture: tonal spread (flat is red), NaN pixels (red), the share of pure black, and how far
//!   it is from the same view's Smooth cell (a method that draws Smooth's picture is red);
//! - the accelerators the request uses, against the same request without them: BLA on and off, the
//!   series on and off (`aux-bla`'s tolerance);
//! - determinism: the same request twice must give the same channels;
//! - Distance at supersampling 1 and 2 (its width is in output pixels since A9);
//! - GPU iterate and color times (timestamp queries), against this card's baseline: a warning,
//!   never a failure, as the bench matrix's.
//!
//! A family that does not offer a method ([`ColorMethod::offered_for`]) records the cell as n/a. A red
//! cell the view set lists under `[[known]]` is reported with its reason and not counted.
//!
//! Writes, under `--out` (default `logs/coloring-matrix-<stamp>/`): one contact sheet per view (its
//! cells at 1:1) and one per variant (every view, half size), `coloring-matrix.json` with every number
//! and `summary.md` with the red cells. ⭐The sheets are the point: a number that looks right on a
//! broken picture is a bug report, so LOOK at them.

use crate::{ColorMethod, FractalKind, TrapType};
use fractadyne_core::Viewport;
use std::path::{Path, PathBuf};

/// The view set unless `--views` names another (repo-relative, resolved by `anchored`).
pub(crate) const VIEWS_FILE: &str = "validation/coloring/views.toml";
/// One cell, output pixels (16:10). Small enough that the full set runs in minutes, large enough to
/// judge a coloring at 1:1.
const CELL_W: u32 = 320;
const CELL_H: u32 = 200;
/// A cell slower than its baseline × (1 + this), and by more than [`TIMING_FLOOR_MS`], is a
/// warning (the bench matrix's rule). Never a failure: one build's timings drift between runs.
const TIMING_REGRESSION: f64 = 0.35;
const TIMING_FLOOR_MS: f64 = 1.0;
/// How far apart two channels may be before they differ (linear color, 0..1): the self-test's
/// `aux-bla` threshold.
const AB_CHANNEL: f32 = 0.02;
/// An accelerator A/B (the render as built against the plain render: no BLA, no series) passes when
/// fewer than this share of the pixels differ where the view is not steep ([`Ab::of`]).
const AB_UNEXPLAINED: f64 = 0.001;
/// Two counts this far apart in neighboring pixels make both steep ([`steep_pixels`]).
const STEEP_ITER: f32 = 2.0;
/// The nudge, in pixels, that finds a cell's sensitive pixels ([`Ab::of`]): `--glitch-audit`'s.
const NUDGE_PX: f32 = 1.0e-3;
/// Distance at ss 1 and ss 2: the median smooth pixel's mean |Δ| over r, g, b (`de-pixels`), and
/// the least share of smooth pixels a view needs for a verdict.
const SS_P50: f64 = 0.01;
const SS_SMOOTH_MIN: f64 = 0.02;
/// A pixel is smooth when no channel differs from a 4-neighbor's by this much.
const SMOOTH_STEP: f32 = 0.03;
/// Mean |Δ| per 8-bit channel below which a method's picture is Smooth's: it fell back.
const SAME_AS_SMOOTH: f64 = 1.0;

/// The checks a cell can fail, by the name `[[known]]` uses.
pub(crate) const CHECKS: &[&str] =
    &["render", "nan", "flat", "same-as-smooth", "bla", "series", "bla+series", "determinism", "supersampling"];

/// `--coloring-matrix` and its options.
pub(crate) struct Cli {
    /// `--quick`: only the views marked `quick`.
    pub(crate) quick: bool,
    /// `--only TEXT`: only the cells whose id (`view/variant`) contains TEXT. The only way to run an
    /// `opt_in` view, which TEXT must then name (part of its name, or `name/variant`).
    pub(crate) only: Option<String>,
    /// `--views FILE`, else [`VIEWS_FILE`].
    pub(crate) views: PathBuf,
    /// `--out DIR`.
    pub(crate) out: Option<PathBuf>,
    /// `--bless`: write this run's timings into this card's baseline.
    pub(crate) bless: bool,
    pub(crate) done: bool,
}

impl Cli {
    /// The options when `--coloring-matrix` is on the command line. A value flag without its value
    /// is fatal, as every other one is.
    pub(crate) fn from_args(args: &[String]) -> Option<Cli> {
        if !args.iter().any(|a| a == "--coloring-matrix") {
            return None;
        }
        let val = |name: &str| -> Option<String> {
            let i = args.iter().position(|a| a == name)?;
            match args.get(i + 1).filter(|s| !s.starts_with("--")) {
                Some(v) => Some(v.clone()),
                None => {
                    eprintln!("fractadyne: {name} needs a value.");
                    crate::exit(2)
                }
            }
        };
        Some(Cli {
            quick: args.iter().any(|a| a == "--quick"),
            only: val("--only").map(|s| s.to_ascii_lowercase()),
            views: val("--views").map(PathBuf::from).unwrap_or_else(|| super::selftest::anchored(VIEWS_FILE)),
            out: val("--out").or_else(|| val("-o")).map(PathBuf::from),
            bless: args.iter().any(|a| a == "--bless"),
            done: false,
        })
    }
}

/// One cell's coloring: a method of the table, and the parameters that make a variant of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Variant {
    pub(crate) method: ColorMethod,
    pub(crate) trap: TrapType,
    /// The stripe average over the tail only.
    pub(crate) tail: bool,
}

impl Variant {
    /// The cell's name: the method's key, the trap's after it (`trap-cross`), `-tail` for the
    /// stripe tail. Built from the table's keys, so it is stable as long as they are.
    pub(crate) fn key(self) -> String {
        match (self.method, self.tail) {
            (ColorMethod::OrbitTrap, _) => format!("{}-{}", self.method.key(), self.trap.key()),
            (m, true) => format!("{}-tail", m.key()),
            (m, false) => m.key().to_string(),
        }
    }
}

/// Every cell a view gets, in the table's order: each method once, the orbit trap once per shape,
/// and the stripe average once more over the tail only. Walks the tables, so a new method or trap
/// shape gets its cells without a change here.
pub(crate) fn variants() -> Vec<Variant> {
    let mut v = Vec::new();
    for m in ColorMethod::ALL {
        let one = Variant { method: m, trap: TrapType::default(), tail: false };
        match m {
            ColorMethod::OrbitTrap => v.extend(TrapType::ALL.map(|t| Variant { trap: t, ..one })),
            ColorMethod::Stripe => v.extend([one, Variant { tail: true, ..one }]),
            _ => v.push(one),
        }
    }
    v
}

/// One view of the set.
#[derive(Clone, Debug)]
pub(crate) struct View {
    pub(crate) name: String,
    pub(crate) fractal: FractalKind,
    /// A custom formula's step (the family is then Custom).
    pub(crate) formula: Option<String>,
    pub(crate) julia_c: Option<(f64, f64)>,
    /// Full-precision text.
    pub(crate) center: (String, String),
    /// The magnification as written, and its log2.
    pub(crate) zoom: String,
    pub(crate) zoom_log2: f64,
    pub(crate) iter: u32,
    pub(crate) quick: bool,
    pub(crate) opt_in: bool,
    pub(crate) note: String,
}

/// A red cell that is understood: reported with its reason, not counted.
#[derive(Clone, Debug)]
pub(crate) struct Known {
    /// `view/variant`, where `*` stands for any run of characters (`*-deep-*/trap-*`).
    pub(crate) cell: String,
    pub(crate) check: String,
    pub(crate) reason: String,
}

/// Whether `text` matches `pattern`, where `*` stands for any run of characters (none included).
pub(crate) fn glob(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !text.starts_with(first) || !text[first.len()..].ends_with(last) || text.len() < first.len() + last.len() {
        return false;
    }
    let mut rest = &text[first.len()..text.len() - last.len()];
    for p in &parts[1..parts.len() - 1] {
        match rest.find(p) {
            Some(i) => rest = &rest[i + p.len()..],
            None => return false,
        }
    }
    true
}

impl Known {
    fn covers(&self, view: &str, variant: &str, check: &str) -> bool {
        self.check == check && glob(&self.cell, &format!("{view}/{variant}"))
    }
}

#[derive(Debug)]
pub(crate) struct ViewSet {
    pub(crate) views: Vec<View>,
    pub(crate) known: Vec<Known>,
}

/// Read a view set. `read_fdn` reads a view file a `fdn` key names (repo-relative).
pub(crate) fn parse_views(text: &str, read_fdn: &dyn Fn(&str) -> Result<String, String>) -> Result<ViewSet, String> {
    // ⚠An unknown key is an error: serde ignores one by default, and a misspelled `zom` would
    // render the view at 1× under its deep name (the profile regions' trap).
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct File {
        #[serde(default)]
        view: Vec<Spec>,
        #[serde(default)]
        known: Vec<KnownSpec>,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Spec {
        name: String,
        fractal: Option<String>,
        formula: Option<String>,
        julia_c: Option<[String; 2]>,
        center: Option<[String; 2]>,
        zoom: Option<String>,
        fdn: Option<String>,
        iter: u32,
        #[serde(default)]
        quick: bool,
        #[serde(default)]
        opt_in: bool,
        #[serde(default)]
        note: String,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct KnownSpec {
        cell: String,
        check: String,
        reason: String,
    }
    let f: File = toml::from_str(text).map_err(|e| e.to_string())?;
    if f.view.is_empty() {
        return Err("no [[view]] entries".into());
    }
    let mut views: Vec<View> = Vec::new();
    for s in f.view {
        let name = s.name.clone();
        let err = |m: String| format!("view \"{name}\": {m}");
        let name_ok = |n: &str| {
            !n.is_empty() && n.len() <= 64 && n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
        };
        if !name_ok(&name) {
            return Err(err("a name is 1 to 64 of a-z, 0-9, '-' and '.'".into()));
        }
        if views.iter().any(|v| v.name == name) {
            return Err(err("named twice".into()));
        }
        // The family, center and zoom: from a view file, or written out.
        let (mut fractal, mut center, mut zoom) = (s.fractal.clone(), s.center.clone().map(|[a, b]| (a, b)), s.zoom.clone());
        if let Some(path) = &s.fdn {
            if center.is_some() || zoom.is_some() {
                return Err(err("give either fdn or center and zoom, not both".into()));
            }
            let text = read_fdn(path).map_err(|e| err(format!("fdn {path}: {e}")))?;
            let pairs = crate::export::view_field_pairs(&text);
            let get = |k: &str| pairs.iter().find(|(key, _)| *key == k).map(|(_, v)| v.to_string());
            let need = |k: &str| get(k).ok_or_else(|| err(format!("{path} has no {k}")));
            center = Some((need("center_re")?, need("center_im")?));
            zoom = Some(need("zoom")?);
            if fractal.is_none() {
                fractal = get("fractal");
            }
        }
        let fractal = match (&s.formula, fractal.as_deref()) {
            (Some(_), None | Some("Custom")) => FractalKind::Custom,
            (Some(_), Some(f)) => return Err(err(format!("a formula is the Custom family, not {f}"))),
            (None, None) => FractalKind::Mandelbrot,
            (None, Some(f)) => match FractalKind::from_name(f) {
                Some(FractalKind::Custom) => return Err(err("Custom needs a formula".into())),
                Some(k) if k.is_escape_time() => k,
                _ => return Err(err(format!("unknown fractal \"{f}\""))),
            },
        };
        let julia_c = match &s.julia_c {
            None => None,
            Some([re, im]) => {
                if !fractal.supports_julia() {
                    return Err(err(format!("{} has no Julia form", fractal.name())));
                }
                let p = |t: &str| t.trim().parse::<f64>().ok().filter(|v| v.is_finite());
                match (p(re), p(im)) {
                    (Some(a), Some(b)) => Some((a, b)),
                    _ => return Err(err("julia_c is two numbers".into())),
                }
            }
        };
        let zoom = zoom.unwrap_or_else(|| "1".into());
        let zoom_log2 = crate::parse_zoom_to_log2(&zoom).ok_or_else(|| err(format!("unreadable zoom \"{zoom}\"")))?;
        let center = center.unwrap_or_else(|| {
            let (x, y) = fractal.default_center();
            (format!("{x:?}"), format!("{y:?}"))
        });
        for c in [&center.0, &center.1] {
            if c.len() > 200_000 || fractadyne_core::parse_bf(c).is_none() {
                return Err(err(format!("unreadable center coordinate \"{}\"", c.chars().take(40).collect::<String>())));
            }
        }
        if !(16..=crate::MAX_ITER_LIMIT).contains(&s.iter) {
            return Err(err(format!("iter {} outside 16..={}", s.iter, crate::MAX_ITER_LIMIT)));
        }
        views.push(View {
            name: s.name,
            fractal,
            formula: s.formula,
            julia_c,
            center,
            zoom,
            zoom_log2,
            iter: s.iter,
            quick: s.quick,
            opt_in: s.opt_in,
            note: s.note,
        });
    }
    let keys: Vec<String> = variants().into_iter().map(Variant::key).collect();
    let mut known = Vec::new();
    for k in f.known {
        let err = |m: String| format!("known \"{}\": {m}", k.cell);
        let (view, variant) = k.cell.split_once('/').ok_or_else(|| err("cell is \"view/variant\"".into()))?;
        // A pattern that names nothing is a typo, and would excuse nothing while looking as if it did.
        if !views.iter().any(|v| glob(view, &v.name)) {
            return Err(err(format!("no view matches \"{view}\"")));
        }
        if !keys.iter().any(|v| glob(variant, v)) {
            return Err(err(format!("no variant matches \"{variant}\" (variants: {})", keys.join(", "))));
        }
        if !CHECKS.contains(&k.check.as_str()) {
            return Err(err(format!("no check \"{}\" (one of {})", k.check, CHECKS.join(", "))));
        }
        if k.reason.trim().is_empty() {
            return Err(err("a known red cell needs its reason".into()));
        }
        known.push(Known { cell: k.cell, check: k.check, reason: k.reason });
    }
    Ok(ViewSet { views, known })
}

/// The cells a run renders, as `(view index, variant)`. A view's Smooth cell always runs when any
/// of its cells does: every other cell is compared with it.
pub(crate) fn select(set: &ViewSet, quick: bool, only: Option<&str>) -> Vec<(usize, Variant)> {
    let all = variants();
    let mut out = Vec::new();
    for (vi, v) in set.views.iter().enumerate() {
        if quick && !v.quick {
            continue;
        }
        // An opt-in view runs only when --only names it: part of its name, or one of its cells.
        if v.opt_in && only.is_none_or(|t| !(v.name.contains(t) || t.starts_with(&format!("{}/", v.name)))) {
            continue;
        }
        let chosen: Vec<Variant> =
            all.iter().copied().filter(|var| only.is_none_or(|t| format!("{}/{}", v.name, var.key()).contains(t))).collect();
        if chosen.is_empty() {
            continue;
        }
        if !chosen.iter().any(|c| c.method == ColorMethod::Smooth) {
            out.push((vi, all[0]));
        }
        out.extend(chosen.into_iter().map(|c| (vi, c)));
    }
    out
}

/// Which pixels of a view are ill-conditioned (steep), from its iteration texture (`render_iter`:
/// the smooth count every fourth float, −1 interior, −2 a glitch): a pixel is steep when a
/// 4-neighbor lies on the other side of the interior's edge or more than [`STEEP_ITER`] iterations
/// away, or when it or a neighbor is a glitch or not a number. The self-test's oracle's rule
/// (`oracle` in `selftest.rs`): there a sub-ulp change of arithmetic legitimately changes the
/// count, so two correct renders may disagree.
pub(crate) fn steep_pixels(iter: &[f32], w: usize) -> Vec<bool> {
    let h = iter.len() / 4 / w.max(1);
    let at = |x: usize, y: usize| iter[(y * w + x) * 4];
    let mut out = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let v = at(x, y);
            let n = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)];
            out[y * w + x] = !v.is_finite()
                || v == -2.0
                || n.iter().filter(|(nx, ny)| *nx < w && *ny < h).any(|&(nx, ny)| {
                    let u = at(nx, ny);
                    !u.is_finite() || u == -2.0 || (u < 0.0) != (v < 0.0) || (v >= 0.0 && (u - v).abs() > STEEP_ITER)
                });
        }
    }
    out
}

/// Whether a method's color is a function of the escape count (Smooth, Distance through the
/// derivative, Decomposition through the final z): where the count is steep, so is its color, and a
/// steep pixel excuses a difference. A statistic of the whole orbit (stripe, triangle, the traps) is
/// not: it can be smooth where the count is steep. Near a minibrot at 9.3e78x, at 200,000
/// iterations, every pixel's count is steep while the traps draw one flat color (2026-11-11), so
/// the count would have excused every pixel of a trap A/B.
pub(crate) fn reads_the_count(m: ColorMethod) -> bool {
    matches!(m, ColorMethod::Smooth | ColorMethod::Distance | ColorMethod::Decomposition)
}

/// The first few non-finite pixels of `px` (`CELL_W` wide) as "(x, y)", with the view's iteration
/// texture there (`iter`, may be empty): count and log2 distance, what a NaN comes from.
fn nan_where(px: &[f32], iter: &[f32]) -> (u64, String) {
    let n = px.iter().filter(|v| !v.is_finite()).count() as u64;
    let at: Vec<String> = px
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, p)| p.iter().any(|v| !v.is_finite()))
        .take(4)
        .map(|(i, _)| {
            let t = iter.get(i * 4..i * 4 + 4).map(|t| format!(" count {} DE {}", t[0], t[3])).unwrap_or_default();
            format!("({}, {}{t})", i as u32 % CELL_W, i as u32 / CELL_W)
        })
        .collect();
    (n, at.join(", "))
}

/// An accelerator A/B's reading: a render against the plain render of the same view (linear RGBA).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ab {
    /// The largest channel difference anywhere, and the share of channels more than
    /// [`AB_CHANNEL`] apart: `aux-bla`'s numbers, reported beside the verdict.
    pub(crate) max: f32,
    pub(crate) share: f64,
    /// The share of pixels that differ where the caller's mask does not excuse them (the verdict),
    /// and the largest channel difference among them.
    pub(crate) unexplained: f64,
    pub(crate) unexplained_max: f32,
    /// The share of pixels not excused: how much of the view the verdict could judge.
    pub(crate) judged: f64,
    /// A difference that is not a number (a NaN in one render and not the other): always a failure.
    pub(crate) nonfinite: bool,
    /// The noise floor ([`Ab::with_floor`]): the share of pixels a nudge of [`NUDGE_PX`] changes.
    pub(crate) floor: f64,
}

impl Ab {
    /// Compare two renders of one view; `steep` marks the pixels not judged (empty: every one).
    ///
    /// ⭐**Two correct renders of a chaotic picture differ, so a difference is judged against the
    /// picture's own sensitivity**, in two ways.
    /// - **Pixel by pixel, where the picture is stable.** A deep view's dense escape field colors as
    ///   speckle, and there a change of rounding (the series' seed, a BLA step) changes a pixel's
    ///   count: at the seahorse at 1e6x, Smooth with the series on vs off differed in 4.77% of
    ///   channels by up to 0.998 (2026-11-11), every one a speckle pixel. The caller excuses the
    ///   pixels where the count is steep (the self-test oracle's rule, for a method colored by the
    ///   count), where the plain render is not smooth, and where a nudge of a thousandth of a pixel
    ///   changes it. (A rule by color, a pixel explained when its color lies within its neighbors'
    ///   3×3 range, still left 3.2% unexplained at that view: a palette is a curve through RGB, and
    ///   nine of its colors need not box a tenth.)
    /// - **By rate, against a nudge.** A chaotic statistic leaves single dots on a smooth ground (an
    ///   orbit trap's nearest approach jumps between two close passes), and a dot that one render
    ///   has and the other lacks sits on a smooth, stable pixel of the other. A nudge of a thousandth
    ///   of a pixel moves the sample far more than an accelerator's rounding does, so when the
    ///   accelerated render changes no more pixels than the nudge does ([`Ab::with_floor`]), its
    ///   error is within the picture's own sensitivity. At the folds and Multibrot 4-8 at depth the
    ///   differences that survived the masks (0.1-8% of the pixels, 2026-11-11) were such dots,
    ///   mixed in with the nudge's own, in the grainy part of the statistic's picture and nowhere
    ///   in its uniform part.
    ///
    /// A systematic error still fails both: it moves a smooth region the nudge leaves alone (near a
    /// minibrot at 9.3e78x the point and cross traps through BLA differed from the plain render on
    /// every pixel of a flat picture). A NaN on one side only always fails.
    pub(crate) fn of(a: &[f32], b: &[f32], steep: &[bool]) -> Ab {
        let bad = Ab {
            max: f32::INFINITY,
            share: 1.0,
            unexplained: 1.0,
            unexplained_max: f32::INFINITY,
            judged: 0.0,
            nonfinite: true,
            floor: 0.0,
        };
        if a.len() != b.len() || a.is_empty() || a.len() % 4 != 0 || !(steep.is_empty() || steep.len() * 4 == a.len()) {
            return bad;
        }
        let (mut max, mut n) = (0.0f32, 0u64);
        for (x, y) in a.iter().zip(b) {
            let d = (x - y).abs();
            if !d.is_finite() || d > max {
                max = if d.is_finite() { d } else { f32::INFINITY };
            }
            if !(d <= AB_CHANNEL) {
                n += 1;
            }
        }
        let (mut un, mut un_max, mut judged, mut nonfinite) = (0u64, 0.0f32, 0u64, false);
        for (i, (p, q)) in a.chunks_exact(4).zip(b.chunks_exact(4)).enumerate() {
            let d = (0..3).map(|c| (p[c] - q[c]).abs()).fold(0.0f32, |m, d| if d.is_finite() { m.max(d) } else { f32::INFINITY });
            nonfinite |= !d.is_finite();
            if steep.get(i).copied().unwrap_or(false) && d.is_finite() {
                continue;
            }
            judged += 1;
            if !(d <= AB_CHANNEL) {
                un += 1;
                un_max = un_max.max(d);
            }
        }
        let pixels = (a.len() / 4) as f64;
        Ab {
            max,
            share: n as f64 / a.len() as f64,
            unexplained: un as f64 / pixels,
            unexplained_max: un_max,
            judged: judged as f64 / pixels,
            nonfinite,
            floor: 0.0,
        }
    }
    /// The noise floor: the share of the pixels a nudge of [`NUDGE_PX`] changes in the plain render.
    pub(crate) fn with_floor(self, floor: f64) -> Ab {
        Ab { floor, ..self }
    }
    pub(crate) fn ok(self) -> bool {
        !self.nonfinite && (self.unexplained < AB_UNEXPLAINED || self.unexplained <= self.floor)
    }
    fn describe(self) -> String {
        format!(
            "{:.3}% of the pixels differ where the picture is smooth and stable (max {:.3}; {:.0}% judged), against {:.3}% a nudge of {NUDGE_PX} px changes{}; {:.2}% of channels > {AB_CHANNEL}, max {:.3}",
            self.unexplained * 100.0,
            self.unexplained_max,
            self.judged * 100.0,
            self.floor * 100.0,
            if self.nonfinite { "; a NaN on one side" } else { "" },
            self.share * 100.0,
            self.max
        )
    }
}

/// The pixels of `a` (`w` wide) no channel of which differs from a 4-neighbor's by
/// [`SMOOTH_STEP`]: where a picture is a picture and not speckle.
fn smooth_pixels(a: &[f32], w: usize) -> Vec<bool> {
    let h = a.len() / 4 / w.max(1);
    let mut out = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let p = &a[(y * w + x) * 4..(y * w + x) * 4 + 3];
            let n = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)];
            out[y * w + x] = p.iter().all(|v| v.is_finite())
                && n.iter().filter(|(nx, ny)| *nx < w && *ny < h).all(|&(nx, ny)| {
                    let q = &a[(ny * w + nx) * 4..(ny * w + nx) * 4 + 3];
                    (0..3).all(|c| (p[c] - q[c]).abs() < SMOOTH_STEP)
                });
        }
    }
    out
}

/// The median smooth pixel's mean |Δ| over r, g, b, and the share of pixels that are smooth in
/// `a`; `None` for the median when too few are ([`SS_SMOOTH_MIN`]). Supersampling legitimately
/// changes a pixel that is speckle or an edge (its four samples differ), and nothing else: a width
/// read per texel instead of per pixel (A9) moves every smooth pixel.
pub(crate) fn smooth_p50_diff(a: &[f32], b: &[f32], w: usize) -> (Option<f64>, f64) {
    if a.len() != b.len() || a.is_empty() || w == 0 {
        return (Some(f64::INFINITY), 0.0);
    }
    let smooth = smooth_pixels(a, w);
    let mut d: Vec<f64> = a
        .chunks_exact(4)
        .zip(b.chunks_exact(4))
        .zip(&smooth)
        .filter(|(_, s)| **s)
        .map(|((p, q), _)| ((p[0] - q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs()) as f64 / 3.0)
        .map(|v| if v.is_finite() { v } else { f64::INFINITY })
        .collect();
    let share = d.len() as f64 / smooth.len().max(1) as f64;
    if share < SS_SMOOTH_MIN {
        return (None, share);
    }
    d.sort_by(|x, y| x.total_cmp(y));
    (Some(d[d.len() / 2]), share)
}

/// A failed comparison's evidence: A | B | where they differ (gray |Δ| × 4, dark blue a steep
/// pixel that differs, red a difference that counts).
fn dump_ab(path: &Path, a: &[f32], b: &[f32], w: u32, steep: &[bool]) {
    let h = (a.len() / 4) as u32 / w.max(1);
    if a.len() != b.len() || h == 0 {
        return;
    }
    let (wa, wb) = (fractadyne_export::to_srgb8_dithered(a, w), fractadyne_export::to_srgb8_dithered(b, w));
    let mut sh = Sheet::new(3 * w + 8, h);
    sh.blit(0, 0, w, h, &wa);
    sh.blit(w + 4, 0, w, h, &wb);
    for i in 0..(w * h) as usize {
        let (p, q) = (&a[i * 4..i * 4 + 3], &b[i * 4..i * 4 + 3]);
        let d = (0..3).map(|c| (p[c] - q[c]).abs()).fold(0.0f32, |m, d| if d.is_finite() { m.max(d) } else { 1.0 });
        let g = (d * 4.0 * 255.0).clamp(0.0, 255.0) as u8;
        let rgb = match (d > AB_CHANNEL, steep.get(i).copied().unwrap_or(false)) {
            (false, _) => [g, g, g],
            (true, true) => [40, 60, 150],
            (true, false) => [255, 40, 40],
        };
        let (x, y) = (2 * (w + 4) + i as u32 % w, i as u32 / w);
        sh.fill(x, y, 1, 1, rgb);
    }
    if let Err(e) = sh.write(path) {
        eprintln!("coloring-matrix: {}: {e}", path.display());
    }
}

/// What a run found: the exit code (0 no new red cell, 1 a new red cell, 2 the view set or the
/// output folder could not be read or made, 3 no cell matched) and what the self-test reports.
pub(crate) struct Verdict {
    pub(crate) code: i32,
    pub(crate) cells: usize,
    /// The new red cells, `view/variant: check: detail` (or why the run could not start).
    pub(crate) red: Vec<String>,
    pub(crate) known: usize,
    pub(crate) not_offered: usize,
}

/// The session state a run changes ([`crate::FractadyneApp::coloring_matrix_saved`]).
pub(crate) struct Saved {
    fractal: FractalKind,
    custom: Option<std::sync::Arc<crate::custom_formula::CustomFormula>>,
    julia_mode: bool,
    julia_c: (f64, f64),
    dual: bool,
    viewport: Viewport,
    max_iter: u32,
    auto_iter: bool,
    series_approx: bool,
    use_bla: bool,
    glitch_correct: bool,
    aa: u32,
    palette_idx: usize,
    cycle: f32,
    offset: f32,
    iter_div: u32,
    use_custom_palette: bool,
    normalize: bool,
    palette_anim: crate::PaletteAnim,
    view_coloring: crate::view_coloring::ViewColoring,
    export_width: u32,
    export_ss: u32,
}

/// What one cell measured.
pub(crate) struct Measured {
    pub(crate) mode: u32,
    pub(crate) bla_on: bool,
    pub(crate) sa_skip: u32,
    pub(crate) spread: f32,
    pub(crate) buckets: usize,
    pub(crate) nan: u64,
    pub(crate) black: f64,
    pub(crate) vs_smooth: Option<f64>,
    /// Channels that differed between two renders of the same request.
    pub(crate) det_diff: Option<u64>,
    /// The render as built against the plain render (no BLA, no series), when it uses either; the
    /// check's name says which ([`CHECKS`]: bla, series, bla+series).
    pub(crate) accel: Option<(&'static str, Ab)>,
    /// When the bla+series check fails: BLA alone against the plain render.
    pub(crate) attribution: Vec<(&'static str, Ab)>,
    /// The share of pixels excused besides the steep ones: a nudge of [`NUDGE_PX`] changes them, or
    /// the plain render is not smooth there.
    pub(crate) sensitive: Option<f64>,
    pub(crate) ss_p50: Option<f64>,
    /// The share of the Distance cell's pixels smooth enough to judge supersampling by.
    pub(crate) ss_smooth: Option<f64>,
    pub(crate) iterate_ms: Option<f64>,
    pub(crate) color_ms: Option<f64>,
    pub(crate) wall_ms: f64,
    pub(crate) base_iterate_ms: Option<f64>,
    pub(crate) timing_warn: bool,
    /// Failed checks: (name in [`CHECKS`], detail).
    pub(crate) fails: Vec<(&'static str, String)>,
}

pub(crate) enum Outcome {
    /// The family does not offer the method ([`ColorMethod::offered_for`]).
    NotOffered,
    Ran(Box<Measured>),
}

pub(crate) struct Cell {
    pub(crate) view: usize,
    pub(crate) variant: Variant,
    pub(crate) outcome: Outcome,
    /// Half-size sRGB pixels for the per-variant sheet.
    pub(crate) thumb: Vec<u8>,
}

/// What a view's Smooth iteration texture says, once per view.
#[derive(Default, Clone, Copy)]
pub(crate) struct ViewStats {
    pub(crate) mode: u32,
    pub(crate) interior: f64,
    pub(crate) glitched: u64,
    pub(crate) nonfinite: u64,
    /// The share of steep pixels ([`steep_pixels`]), which the accelerator A/Bs do not judge.
    pub(crate) steep: f64,
}

fn band_name(mode: u32) -> &'static str {
    match mode {
        m if m == crate::RenderMode::Direct as u32 => "direct",
        m if m == crate::RenderMode::Df32Pert as u32 => "df32",
        m if m == crate::RenderMode::Floatexp as u32 => "floatexp",
        _ => "?",
    }
}

struct Shot {
    px: Vec<f32>,
    w: u32,
    iterate_ms: f64,
    color_ms: f64,
}

fn shoot(
    dev: &eframe::wgpu::Device,
    q: &eframe::wgpu::Queue,
    req: &fractadyne_gpu::ExportRequest,
) -> Result<Shot, String> {
    let progress = std::sync::atomic::AtomicU32::new(0);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    fractadyne_gpu::render_export(dev, q, req, &progress, &cancel)
        .map(|r| Shot { px: r.pixels, w: r.width, iterate_ms: r.iterate_ms, color_ms: r.color_ms })
        .map_err(|e| e.to_string())
}

/// A 2× box downscale of sRGB8 pixels (the per-variant sheet's cells).
fn half(px: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w2, h2) = ((w / 2) as usize, (h / 2) as usize);
    let w = w as usize;
    let mut out = vec![0u8; w2 * h2 * 4];
    for y in 0..h2 {
        for x in 0..w2 {
            for c in 0..4 {
                let s: u32 = [(0, 0), (1, 0), (0, 1), (1, 1)]
                    .iter()
                    .map(|(dx, dy)| px[((2 * y + dy) * w + 2 * x + dx) * 4 + c] as u32)
                    .sum();
                out[(y * w2 + x) * 4 + c] = ((s + 2) / 4) as u8;
            }
        }
    }
    out
}

/// `text`, shortened to `max` characters with a trailing ellipsis, so a label does not run into the
/// next cell's (a monospace character is about 0.6 of its height wide).
fn fit(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut t: String = text.chars().take(max.saturating_sub(1)).collect();
    t.push('\u{2026}');
    t
}

/// A contact sheet: sRGB8 on a dark ground, with labels in the app's font.
struct Sheet {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

const INK: [u8; 3] = [226, 228, 232];
const DIM: [u8; 3] = [150, 154, 160];
const GOOD: [u8; 3] = [120, 205, 130];
const RED: [u8; 3] = [255, 96, 88];
const AMBER: [u8; 3] = [240, 188, 80];

impl Sheet {
    fn new(w: u32, h: u32) -> Sheet {
        let mut px = vec![0u8; (w * h * 4) as usize];
        for p in px.chunks_exact_mut(4) {
            p.copy_from_slice(&[22, 23, 27, 255]);
        }
        Sheet { w, h, px }
    }
    fn blit(&mut self, x: u32, y: u32, w: u32, h: u32, src: &[u8]) {
        for row in 0..h.min(self.h.saturating_sub(y)) {
            let n = w.min(self.w.saturating_sub(x)) as usize * 4;
            let d = ((y + row) * self.w + x) as usize * 4;
            let s = (row * w) as usize * 4;
            self.px[d..d + n].copy_from_slice(&src[s..s + n]);
        }
    }
    fn fill(&mut self, x: u32, y: u32, w: u32, h: u32, rgb: [u8; 3]) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                let i = (yy * self.w + xx) as usize * 4;
                self.px[i..i + 3].copy_from_slice(&rgb);
            }
        }
    }
    /// Draw `text` with its top left at (x, y), `size` pixels tall, from egui's font atlas.
    fn text(&mut self, ctx: Option<&eframe::egui::Context>, x: f32, y: f32, text: &str, size: f32, rgb: [u8; 3]) {
        let Some(ctx) = ctx else { return };
        let ppp = ctx.pixels_per_point();
        let galley = ctx.fonts(|f| {
            f.layout_no_wrap(text.to_string(), eframe::egui::FontId::monospace(size / ppp), eframe::egui::Color32::WHITE)
        });
        // The atlas AFTER layout, so it holds this text's glyphs (egui fills it lazily).
        ctx.fonts(|f| {
            let atlas = f.image();
            let aw = atlas.size[0];
            for row in &galley.rows {
                for g in &row.glyphs {
                    let uv = g.uv_rect;
                    if uv.max[0] <= uv.min[0] || uv.max[1] <= uv.min[1] {
                        continue;
                    }
                    let ox = (x + (g.pos.x + uv.offset.x) * ppp).round() as i32;
                    let oy = (y + (g.pos.y + uv.offset.y) * ppp).round() as i32;
                    for ty in uv.min[1]..uv.max[1] {
                        for tx in uv.min[0]..uv.max[0] {
                            let cov = atlas.pixels[ty as usize * aw + tx as usize];
                            let (dx, dy) = (ox + (tx - uv.min[0]) as i32, oy + (ty - uv.min[1]) as i32);
                            if cov <= 0.0 || dx < 0 || dy < 0 || dx >= self.w as i32 || dy >= self.h as i32 {
                                continue;
                            }
                            let i = (dy as u32 * self.w + dx as u32) as usize * 4;
                            for c in 0..3 {
                                let v = rgb[c] as f32 * cov + self.px[i + c] as f32 * (1.0 - cov);
                                self.px[i + c] = v.round().clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                }
            }
        });
    }
    fn write(&self, path: &Path) -> Result<(), String> {
        fractadyne_export::write_png_rgba8(path, self.w, self.h, &self.px, None).map_err(|e| e.to_string())
    }
}

/// A cell's one-line verdict and its color: red (a check failed that no `[[known]]` covers), amber
/// (known, or a timing warning), green, or dim (n/a).
fn verdict(cell: &Cell, view: &View, known: &[Known]) -> (String, [u8; 3]) {
    match &cell.outcome {
        Outcome::NotOffered => ("n/a: no distance estimate".into(), DIM),
        Outcome::Ran(m) => {
            let key = cell.variant.key();
            let new: Vec<&str> =
                m.fails.iter().filter(|(c, _)| !known.iter().any(|k| k.covers(&view.name, &key, c))).map(|(c, _)| *c).collect();
            if !new.is_empty() {
                (format!("RED {}", new.join(",")), RED)
            } else if !m.fails.is_empty() {
                (format!("known {}", m.fails.iter().map(|(c, _)| *c).collect::<Vec<_>>().join(",")), AMBER)
            } else if m.timing_warn {
                ("ok, slower".into(), AMBER)
            } else {
                ("ok".into(), GOOD)
            }
        }
    }
}

/// This card's baseline file (it need not exist).
fn baseline_path(gpu: &str) -> PathBuf {
    super::selftest::anchored("benchmarks").join(format!("coloring-matrix-baseline-{}.json", crate::calibration::slug(gpu)))
}

/// The baseline's cells: `view/variant` -> (iterate ms, color ms).
fn load_baseline(path: &Path) -> Result<std::collections::BTreeMap<String, (f64, f64)>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Default::default()),
        Err(e) => return Err(e.to_string()),
    };
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut out = std::collections::BTreeMap::new();
    if let Some(cells) = v.get("cells").and_then(|c| c.as_object()) {
        for (k, c) in cells {
            let it = c.get("iterate_ms").and_then(|x| x.as_f64());
            let col = c.get("color_ms").and_then(|x| x.as_f64());
            if let (Some(it), Some(col)) = (it, col) {
                out.insert(k.clone(), (it, col));
            }
        }
    }
    Ok(out)
}

impl crate::FractadyneApp {
    /// Put the session into the matrix's fixed state: what is not the variable under test is the
    /// default, so a cell renders the same whatever session the run started from.
    fn coloring_matrix_baseline_state(&mut self) {
        let s = fractadyne_state::SessionState::default();
        self.dual = false;
        self.render_cfg.series_approx = true;
        self.render_cfg.use_bla = true;
        self.render_cfg.glitch_correct = true;
        self.render_cfg.auto_iter = false;
        self.render_cfg.aa = 1;
        self.coloring.palette_idx = s.palette_idx;
        self.coloring.cycle = s.cycle;
        self.coloring.offset = s.offset;
        self.coloring.iter_div = s.iter_div;
        self.coloring.use_custom_palette = false;
        self.coloring.normalize = false;
        self.anim.palette_anim = crate::PaletteAnim::Off;
        self.set_view_coloring(&crate::view_coloring::ViewColoring::default());
        self.export.width = CELL_W;
        self.export.ss = 1;
    }

    /// Set the family, Julia mode and iteration count for `v`.
    fn coloring_matrix_set_view(&mut self, v: &View) -> Result<(), String> {
        if let Some(src) = &v.formula {
            let c = crate::custom_formula::CustomFormula::compile(src, &[]).map_err(|e| format!("formula: {e}"))?;
            crate::calibration::set_custom_factor(c.shader.cost_factor);
            self.custom = Some(std::sync::Arc::new(c));
            self.fractal = FractalKind::Custom;
        } else {
            self.set_fractal(v.fractal);
        }
        self.julia_mode = v.julia_c.is_some();
        if let Some(c) = v.julia_c {
            self.julia_c = c;
        }
        self.dual = false;
        self.render_cfg.max_iter = v.iter;
        self.invalidate_refs();
        Ok(())
    }

    /// The view's viewport at the cell size.
    fn coloring_matrix_viewport(&self, v: &View) -> Viewport {
        let mut vp = Viewport::new(CELL_W as f64, CELL_H as f64);
        let (cx, cy) = crate::arg_center(&format!("views \"{}\"", v.name), &v.center.0, &v.center.1, v.zoom_log2);
        vp.set_center_log2mag(cx, cy, v.zoom_log2);
        vp
    }

    /// Render and measure one cell.
    #[allow(clippy::too_many_arguments)]
    fn coloring_matrix_cell(
        &mut self,
        dev: &eframe::wgpu::Device,
        q: &eframe::wgpu::Queue,
        view: &View,
        vp: &Viewport,
        var: Variant,
        smooth: Option<&[u8]>,
        iter: &[f32],
        steep: &[bool],
        evidence: &Path,
    ) -> (Measured, Vec<u8>) {
        // A steep count excuses only a method colored by the count.
        let steep: &[bool] = if reads_the_count(var.method) { steep } else { &[] };
        let dump = |check: &str, a: &[f32], b: &[f32]| {
            dump_ab(&evidence.join(format!("{}__{}__{check}.png", view.name, var.key())), a, b, CELL_W, steep);
        };
        let mut vc = crate::view_coloring::ViewColoring::default();
        vc.method = var.method;
        vc.trap = var.trap;
        vc.stripe_tail = var.tail;
        self.set_view_coloring(&vc);
        let t0 = std::time::Instant::now();
        let req = self.current_export_request_for(vp, view.julia_c.is_some());
        let mut m = Measured {
            mode: req.mode,
            bla_on: req.bla_on == 1,
            sa_skip: req.sa_skip,
            spread: 0.0,
            buckets: 0,
            nan: 0,
            black: 0.0,
            vs_smooth: None,
            det_diff: None,
            accel: None,
            attribution: Vec::new(),
            sensitive: None,
            ss_p50: None,
            ss_smooth: None,
            iterate_ms: None,
            color_ms: None,
            wall_ms: 0.0,
            base_iterate_ms: None,
            timing_warn: false,
            fails: Vec::new(),
        };
        let a = match shoot(dev, q, &req) {
            Ok(a) => a,
            Err(e) => {
                m.fails.push(("render", e));
                return (m, Vec::new());
            }
        };
        m.wall_ms = t0.elapsed().as_secs_f64() * 1000.0;
        // The export clamps an image the device cannot hold; a cell must be the size the sheets draw.
        if a.w != CELL_W || a.px.len() != (CELL_W * CELL_H * 4) as usize {
            m.fails.push(("render", format!("{} x {} pixels, not {CELL_W} x {CELL_H}", a.w, a.px.len() / 4 / a.w.max(1) as usize)));
            return (m, Vec::new());
        }
        let px8 = fractadyne_export::to_srgb8_dithered(&a.px, a.w);
        let (nan, at) = nan_where(&a.px, iter);
        m.nan = nan;
        if nan > 0 {
            m.fails.push(("nan", format!("{nan} non-finite channels, at {at}")));
        }
        let (sd, b) = super::selftest::frame::coherence(&px8);
        m.spread = sd;
        m.buckets = b;
        if !super::selftest::frame::coherent(&px8) {
            m.fails.push(("flat", format!("spread {sd:.2}")));
        }
        let n = (px8.len() / 4).max(1);
        m.black = px8.chunks_exact(4).filter(|p| p[0] == 0 && p[1] == 0 && p[2] == 0).count() as f64 / n as f64;
        if let Some(s) = smooth.filter(|_| var.method != ColorMethod::Smooth) {
            let d = super::selftest::frame::distance(s, &px8);
            m.vs_smooth = Some(d);
            if d < SAME_AS_SMOOTH {
                m.fails.push(("same-as-smooth", format!("mean |Δ| {d:.2}")));
            }
        }
        crate::diag::breadcrumb(format!("coloring-matrix: {}/{} rendered", view.name, var.key()));
        // The same request again: the channels must not move.
        let (mut it, mut col) = (vec![a.iterate_ms], vec![a.color_ms]);
        match shoot(dev, q, &req) {
            Ok(b) => {
                let d = a.px.iter().zip(&b.px).filter(|(x, y)| x.to_bits() != y.to_bits()).count() as u64
                    + a.px.len().abs_diff(b.px.len()) as u64;
                m.det_diff = Some(d);
                if d > 0 {
                    m.fails.push(("determinism", format!("{d} channels differ between two renders")));
                    dump("determinism", &a.px, &b.px);
                }
                it.push(b.iterate_ms);
                col.push(b.color_ms);
            }
            Err(e) => m.fails.push(("render", format!("second render: {e}"))),
        }
        let least = |v: &[f64]| v.iter().copied().filter(|x| *x > 0.0).reduce(f64::min);
        m.iterate_ms = least(&it);
        m.color_ms = least(&col);
        // The accelerators this request uses, against the plain render of the same view: no BLA, no
        // series, every iteration stepped. That render is the truth both must agree with; toggling
        // one at a time compares against a render that still has the other, and cannot say which
        // is wrong (at corpus 07 at 1e30x, stripe with BLA on and off differed on 96% of the pixels
        // with the series on in both, 2026-11-11).
        let (bla, sa) = (req.bla_on == 1, req.sa_skip > 0);
        if bla || sa {
            let check = match (bla, sa) {
                (true, false) => "bla",
                (false, true) => "series",
                _ => "bla+series",
            };
            let mut plain = req.clone();
            plain.bla_on = 0;
            plain.sa_skip = 0;
            // Where the statistic itself is chaotic (an orbit trap's minimum can jump between two close
            // approaches where the count is smooth), two correct renders may disagree as they may
            // where the count is steep. Two signs of it, both read off the plain render: a pixel a
            // nudge of a thousandth of a pixel changes (SENSITIVE), and a pixel not SMOOTH there (a
            // neighbor more than SMOOTH_STEP away: speckle). One nudge is a sample, not a proof: at
            // the folds and Multibrot 4-8 at depth, the differences a nudge left (0.3-16% of the
            // pixels, 2026-11-11) lay among the sensitive ones in the grainy regions of the statistic's
            // own picture and nowhere in its smooth ones, where a wrong aggregate would show.
            let mut nudged = plain.clone();
            nudged.jitter = [NUDGE_PX, NUDGE_PX];
            match shoot(dev, q, &plain).and_then(|p| shoot(dev, q, &nudged).map(|n| (p, n))) {
                Ok((p, n)) => {
                    // The plain render is what a person gets with BLA turned off: a NaN there is a
                    // defect of its own.
                    for (which, r) in [("plain", &p), ("nudged plain", &n)] {
                        let (nan, at) = nan_where(&r.px, iter);
                        if nan > 0 {
                            m.fails.push(("nan", format!("the {which} render (no BLA, no series): {nan} non-finite channels, at {at}")));
                        }
                    }
                    let smooth = smooth_pixels(&p.px, CELL_W as usize);
                    let nudge = p
                        .px
                        .chunks_exact(4)
                        .zip(n.px.chunks_exact(4))
                        .filter(|(x, y)| (0..3).any(|c| !((x[c] - y[c]).abs() <= AB_CHANNEL)))
                        .count() as f64
                        / (p.px.len() / 4).max(1) as f64;
                    let excused: Vec<bool> = p
                        .px
                        .chunks_exact(4)
                        .zip(n.px.chunks_exact(4))
                        .enumerate()
                        .map(|(i, (x, y))| {
                            steep.get(i).copied().unwrap_or(false)
                                || !smooth[i]
                                || (0..3).any(|c| !((x[c] - y[c]).abs() <= AB_CHANNEL))
                        })
                        .collect();
                    m.sensitive = Some(
                        excused.iter().enumerate().filter(|(i, e)| **e && !steep.get(*i).copied().unwrap_or(false)).count() as f64
                            / excused.len().max(1) as f64,
                    );
                    let steep = excused.as_slice();
                    let ab = Ab::of(&a.px, &p.px, steep).with_floor(nudge);
                    m.accel = Some((check, ab));
                    if !ab.ok() {
                        let mut detail = format!("{check} vs plain: {}", ab.describe());
                        if bla && sa {
                            // Which one: BLA alone against the plain render. (Not the series alone: the
                            // app never runs it without BLA, and without the tree the series' prefix
                            // folds no statistics, so that render describes nothing a person sees.)
                            for (name, b_on, skip) in [("BLA alone", 1u32, 0u32)] {
                                let mut one = req.clone();
                                one.bla_on = b_on;
                                one.sa_skip = skip;
                                match shoot(dev, q, &one) {
                                    Ok(o) => {
                                        let ab1 = Ab::of(&o.px, &p.px, steep).with_floor(nudge);
                                        detail.push_str(&format!("; {name}: {:.3}%", ab1.unexplained * 100.0));
                                        m.attribution.push((name, ab1));
                                    }
                                    Err(e) => detail.push_str(&format!("; {name}: render failed ({e})")),
                                }
                            }
                        }
                        m.fails.push((check, detail));
                        dump_ab(&evidence.join(format!("{}__{}__{check}.png", view.name, var.key())), &a.px, &p.px, CELL_W, steep);
                    }
                }
                Err(e) => m.fails.push(("render", format!("plain or nudged render: {e}"))),
            }
        }
        // A width in output pixels must not move with the supersampling (A9).
        if var.method == ColorMethod::Distance {
            let mut ss2 = req.clone();
            ss2.ss = 2;
            match shoot(dev, q, &ss2) {
                Ok(b) => {
                    let (p, smooth) = smooth_p50_diff(&a.px, &b.px, CELL_W as usize);
                    m.ss_p50 = p;
                    m.ss_smooth = Some(smooth);
                    if let Some(p) = p.filter(|p| !(*p < SS_P50)) {
                        m.fails.push((
                            "supersampling",
                            format!("ss 1 vs 2: median smooth pixel |Δ| {p:.4} ({:.0}% of pixels smooth)", smooth * 100.0),
                        ));
                        dump("supersampling", &a.px, &b.px);
                    }
                }
                Err(e) => m.fails.push(("render", format!("ss 2: {e}"))),
            }
        }
        (m, px8)
    }

    /// Run the matrix: render, measure, write the sheets (with `ctx`), the JSON and the summary.
    /// Leaves the session in the matrix's state: a caller that goes on (the self-test) saves and
    /// restores it ([`Self::coloring_matrix_saved`]).
    pub(crate) fn run_coloring_matrix(
        &mut self,
        ctx: Option<&eframe::egui::Context>,
        dev: &eframe::wgpu::Device,
        q: &eframe::wgpu::Queue,
        cli: &Cli,
    ) -> Verdict {
        let fail = |code: i32, why: String| Verdict { code, cells: 0, red: vec![why], known: 0, not_offered: 0 };
        let set = match std::fs::read_to_string(&cli.views)
            .map_err(|e| e.to_string())
            .and_then(|t| parse_views(&t, &|p| std::fs::read_to_string(super::selftest::anchored(p)).map_err(|e| e.to_string())))
        {
            Ok(s) => s,
            Err(e) => {
                eprintln!("fractadyne: --coloring-matrix: {}: {e}", cli.views.display());
                return fail(2, format!("{}: {e}", cli.views.display()));
            }
        };
        let picked = select(&set, cli.quick, cli.only.as_deref());
        if picked.is_empty() {
            eprintln!("fractadyne: --coloring-matrix: no cell matches (--only {:?}, --quick {})", cli.only, cli.quick);
            return fail(3, "no cell matches".into());
        }
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let out = cli.out.clone().unwrap_or_else(|| PathBuf::from(format!("logs/coloring-matrix-{}", Self::file_stamp(secs))));
        for d in [out.clone(), out.join("views"), out.join("variants"), out.join("ab")] {
            if let Err(e) = std::fs::create_dir_all(&d) {
                eprintln!("fractadyne: --coloring-matrix: {}: {e}", d.display());
                return fail(2, format!("{}: {e}", d.display()));
            }
        }
        let base_path = baseline_path(&self.gpu_name);
        let baseline = load_baseline(&base_path).unwrap_or_else(|e| {
            eprintln!("coloring-matrix: baseline {} unreadable ({e}); timings not compared", base_path.display());
            Default::default()
        });
        let views_run: Vec<usize> = {
            let mut v: Vec<usize> = picked.iter().map(|(vi, _)| *vi).collect();
            v.dedup();
            v
        };
        println!(
            "Fractadyne coloring matrix: {} views, {} cells, {}x{} ({}), on {}",
            views_run.len(),
            picked.len(),
            CELL_W,
            CELL_H,
            cli.views.display(),
            self.gpu_name
        );
        if baseline.is_empty() {
            println!("  no baseline for this card ({}): timings are recorded, not compared", base_path.display());
        }
        self.coloring_matrix_baseline_state();
        let mut cells: Vec<Cell> = Vec::new();
        let mut stats: Vec<(usize, ViewStats)> = Vec::new();
        let started = std::time::Instant::now();
        for &vi in &views_run {
            let view = set.views[vi].clone();
            if let Err(e) = self.coloring_matrix_set_view(&view) {
                eprintln!("fractadyne: --coloring-matrix: view \"{}\": {e}", view.name);
                return fail(2, format!("view \"{}\": {e}", view.name));
            }
            let vp = self.coloring_matrix_viewport(&view);
            let caps = self.fractal.caps();
            // The view's Smooth iteration texture: interior, glitch sentinels, non-finite counts.
            let mut vs = ViewStats::default();
            let mut steep: Vec<bool> = Vec::new();
            let mut iter: Vec<f32> = Vec::new();
            {
                let mut vc = crate::view_coloring::ViewColoring::default();
                vc.method = ColorMethod::Smooth;
                self.set_view_coloring(&vc);
                let req = self.current_export_request_for(&vp, view.julia_c.is_some());
                vs.mode = req.mode;
                if let Ok(r) = fractadyne_gpu::render_iter(dev, q, &req) {
                    let n = (r.pixels.len() / 4).max(1);
                    let s = r.pixels.iter().step_by(4);
                    vs.interior = s.clone().filter(|v| **v == -1.0).count() as f64 / n as f64;
                    vs.glitched = s.clone().filter(|v| **v == -2.0).count() as u64;
                    vs.nonfinite = s.filter(|v| !v.is_finite()).count() as u64;
                    if r.pixels.len() == (CELL_W * CELL_H * 4) as usize {
                        steep = steep_pixels(&r.pixels, CELL_W as usize);
                        iter = r.pixels.clone();
                        vs.steep = steep.iter().filter(|s| **s).count() as f64 / steep.len().max(1) as f64;
                    }
                }
            }
            stats.push((vi, vs));
            println!(
                "\n  {} ({}{}, {}x, {}, iter {}): interior {:.0}%, steep {:.0}%{}",
                view.name,
                view.fractal.name(),
                view.julia_c.map(|(a, b)| format!(" Julia c {a}{b:+}i")).unwrap_or_default(),
                view.zoom,
                band_name(vs.mode),
                view.iter,
                vs.interior * 100.0,
                vs.steep * 100.0,
                if vs.glitched + vs.nonfinite > 0 {
                    format!(", {} glitch sentinels, {} non-finite", vs.glitched, vs.nonfinite)
                } else {
                    String::new()
                }
            );
            let mut smooth_px: Option<Vec<u8>> = None;
            let mut view_cells: Vec<(Cell, Vec<u8>)> = Vec::new();
            for &(_, var) in picked.iter().filter(|(v, _)| *v == vi) {
                if !var.method.offered_for(caps) {
                    view_cells.push((Cell { view: vi, variant: var, outcome: Outcome::NotOffered, thumb: Vec::new() }, Vec::new()));
                    continue;
                }
                let (mut m, px8) = self.coloring_matrix_cell(dev, q, &view, &vp, var, smooth_px.as_deref(), &iter, &steep, &out.join("ab"));
                let id = format!("{}/{}", view.name, var.key());
                if let (Some(now), Some(&(base, _))) = (m.iterate_ms, baseline.get(&id)) {
                    m.base_iterate_ms = Some(base);
                    m.timing_warn = now > base * (1.0 + TIMING_REGRESSION) && now - base > TIMING_FLOOR_MS;
                }
                if var.method == ColorMethod::Smooth && !px8.is_empty() {
                    smooth_px = Some(px8.clone());
                }
                let cell = Cell {
                    view: vi,
                    variant: var,
                    thumb: if px8.is_empty() { Vec::new() } else { half(&px8, CELL_W, CELL_H) },
                    outcome: Outcome::Ran(Box::new(m)),
                };
                let (word, _) = verdict(&cell, &view, &set.known);
                if let Outcome::Ran(m) = &cell.outcome {
                    let ms = |v: Option<f64>, p: usize| v.map_or_else(|| "   n/a".to_string(), |x| format!("{x:>7.p$}"));
                    println!(
                        "    {:<14} {:<8} iterate {} ms  color {} ms{}{}  {}",
                        var.key(),
                        band_name(m.mode),
                        ms(m.iterate_ms, 2),
                        ms(m.color_ms, 3),
                        if m.bla_on { "  BLA" } else { "" },
                        if m.sa_skip > 0 { format!("  SA {}", m.sa_skip) } else { String::new() },
                        word
                    );
                    for (c, d) in &m.fails {
                        println!("        {c}: {d}");
                    }
                }
                view_cells.push((cell, px8));
            }
            // The view's sheet: its cells at 1:1, three to a row.
            if ctx.is_some() {
                let cols = 3u32;
                let (lab, gap, head) = (18u32, 6u32, 44u32);
                let rows = (view_cells.len() as u32).div_ceil(cols);
                let mut sh = Sheet::new(cols * (CELL_W + gap) + gap, head + rows * (CELL_H + lab + gap) + gap);
                let title = format!(
                    "{}  {}{}  {}x  {}  iter {}",
                    view.name,
                    view.fractal.name(),
                    view.julia_c.map(|(a, b)| format!(" Julia c {a}{b:+}i")).unwrap_or_default(),
                    view.zoom,
                    band_name(vs.mode),
                    view.iter
                );
                sh.text(ctx, gap as f32, 6.0, &title, 14.0, INK);
                let sub = format!(
                    "interior {:.0}%  steep {:.0}%   {}   {}",
                    vs.interior * 100.0,
                    vs.steep * 100.0,
                    view.note,
                    crate::version_string()
                );
                sh.text(ctx, gap as f32, 25.0, &sub, 12.0, DIM);
                for (k, (cell, px8)) in view_cells.iter().enumerate() {
                    let (cx, cy) = (gap + (k as u32 % cols) * (CELL_W + gap), head + (k as u32 / cols) * (CELL_H + lab + gap));
                    let (word, color) = verdict(cell, &view, &set.known);
                    sh.text(ctx, cx as f32, cy as f32 + 1.0, &cell.variant.key(), 13.0, INK);
                    sh.text(ctx, (cx + 130) as f32, cy as f32 + 1.0, &word, 13.0, color);
                    if px8.is_empty() {
                        sh.fill(cx, cy + lab, CELL_W, CELL_H, [40, 41, 46]);
                    } else {
                        sh.blit(cx, cy + lab, CELL_W, CELL_H, px8);
                    }
                }
                if let Err(e) = sh.write(&out.join("views").join(format!("{}.png", view.name))) {
                    eprintln!("coloring-matrix: sheet for {}: {e}", view.name);
                }
            }
            cells.extend(view_cells.into_iter().map(|(c, _)| c));
        }
        // One sheet per variant: every view at half size.
        if ctx.is_some() {
            let (tw, th) = (CELL_W / 2, CELL_H / 2);
            for var in variants() {
                let mine: Vec<&Cell> = cells.iter().filter(|c| c.variant == var).collect();
                if mine.is_empty() {
                    continue;
                }
                let cols = 8u32.min(mine.len() as u32);
                let (lab, gap, head) = (28u32, 4u32, 30u32);
                let rows = (mine.len() as u32).div_ceil(cols);
                // At least as wide as its title, for a run of a few views.
                let mut sh = Sheet::new((cols * (tw + gap) + gap).max(560), head + rows * (th + lab + gap) + gap);
                sh.text(ctx, gap as f32, 6.0, &format!("{}   {}", var.key(), crate::version_string()), 14.0, INK);
                for (k, cell) in mine.iter().enumerate() {
                    let view = &set.views[cell.view];
                    let (cx, cy) = (gap + (k as u32 % cols) * (tw + gap), head + (k as u32 / cols) * (th + lab + gap));
                    let (word, color) = verdict(cell, view, &set.known);
                    sh.text(ctx, cx as f32, cy as f32, &fit(&view.name, (tw / 6) as usize), 10.0, INK);
                    sh.text(ctx, cx as f32, cy as f32 + 13.0, &word, 10.0, color);
                    if cell.thumb.is_empty() {
                        sh.fill(cx, cy + lab, tw, th, [40, 41, 46]);
                    } else {
                        sh.blit(cx, cy + lab, tw, th, &cell.thumb);
                    }
                }
                if let Err(e) = sh.write(&out.join("variants").join(format!("{}.png", var.key()))) {
                    eprintln!("coloring-matrix: sheet for {}: {e}", var.key());
                }
            }
        }
        // The numbers, and the verdict.
        let mut red = Vec::new();
        let mut known_hit = Vec::new();
        let mut slower = Vec::new();
        let mut cells_json = Vec::new();
        let mut not_offered = 0usize;
        for c in &cells {
            let view = &set.views[c.view];
            let id = format!("{}/{}", view.name, c.variant.key());
            let mut j = serde_json::json!({
                "view": view.name,
                "variant": c.variant.key(),
                "method": c.variant.method.key(),
                "trap": c.variant.trap.key(),
                "stripe_tail": c.variant.tail,
            });
            match &c.outcome {
                Outcome::NotOffered => {
                    not_offered += 1;
                    j["status"] = "n/a".into();
                }
                Outcome::Ran(m) => {
                    for (check, detail) in &m.fails {
                        match set.known.iter().find(|k| k.covers(&view.name, &c.variant.key(), check)) {
                            Some(k) => known_hit.push((id.clone(), *check, detail.clone(), k.reason.clone())),
                            None => red.push((id.clone(), *check, detail.clone())),
                        }
                    }
                    if m.timing_warn {
                        slower.push((id.clone(), m.iterate_ms.unwrap_or(0.0), m.base_iterate_ms.unwrap_or(0.0)));
                    }
                    let ab = |a: &Ab| {
                        serde_json::json!({"unexplained": a.unexplained, "unexplained_max": a.unexplained_max,
                            "judged": a.judged, "nudge_floor": a.floor, "nonfinite": a.nonfinite,
                            "max": a.max, "share": a.share, "ok": a.ok()})
                    };
                    j["status"] = if m.fails.is_empty() { "ok" } else { "red" }.into();
                    j["band"] = band_name(m.mode).into();
                    j["bla_on"] = m.bla_on.into();
                    j["sa_skip"] = m.sa_skip.into();
                    j["spread"] = (m.spread as f64).into();
                    j["buckets"] = m.buckets.into();
                    j["nan_channels"] = m.nan.into();
                    j["black_share"] = m.black.into();
                    j["vs_smooth"] = m.vs_smooth.into();
                    j["determinism_diff"] = m.det_diff.into();
                    j["sensitive_share"] = m.sensitive.into();
                    j["accelerators"] = m
                        .accel
                        .as_ref()
                        .map(|(check, a)| {
                            let mut v = ab(a);
                            v["check"] = (*check).into();
                            v["attribution"] =
                                m.attribution.iter().map(|(n, a)| serde_json::json!({"alone": n, "ab": ab(a)})).collect::<Vec<_>>().into();
                            v
                        })
                        .into();
                    j["ss_p50"] = m.ss_p50.into();
                    j["ss_smooth_share"] = m.ss_smooth.into();
                    j["iterate_ms"] = m.iterate_ms.into();
                    j["color_ms"] = m.color_ms.into();
                    j["wall_ms"] = m.wall_ms.into();
                    j["baseline_iterate_ms"] = m.base_iterate_ms.into();
                    j["timing_warn"] = m.timing_warn.into();
                    j["fails"] = m.fails.iter().map(|(c, d)| serde_json::json!({"check": c, "detail": d})).collect::<Vec<_>>().into();
                }
            }
            cells_json.push(j);
        }
        // The Smooth cell's iterate time, per view: each cell's ratio to it.
        let smooth_ms: std::collections::HashMap<usize, f64> = cells
            .iter()
            .filter(|c| c.variant.method == ColorMethod::Smooth)
            .filter_map(|c| match &c.outcome {
                Outcome::Ran(m) => m.iterate_ms.map(|t| (c.view, t)),
                Outcome::NotOffered => None,
            })
            .collect();
        for (j, c) in cells_json.iter_mut().zip(&cells) {
            if let (Outcome::Ran(m), Some(s)) = (&c.outcome, smooth_ms.get(&c.view)) {
                j["iterate_vs_smooth"] = m.iterate_ms.map(|t| t / s).into();
            }
        }
        let views_json: Vec<serde_json::Value> = stats
            .iter()
            .map(|(vi, s)| {
                let v = &set.views[*vi];
                serde_json::json!({
                    "name": v.name, "fractal": v.fractal.name(), "formula": v.formula,
                    "julia_c": v.julia_c.map(|(a, b)| [a, b]), "center": [v.center.0, v.center.1],
                    "zoom": v.zoom, "zoom_log2": v.zoom_log2, "iter": v.iter, "band": band_name(s.mode),
                    "interior_share": s.interior, "steep_share": s.steep, "glitch_sentinels": s.glitched,
                    "nonfinite": s.nonfinite, "note": v.note,
                })
            })
            .collect();
        let report = serde_json::json!({
            "tool": "fractadyne --coloring-matrix",
            "version": crate::version_string(),
            "utc": crate::utc_string(secs),
            "gpu": self.gpu_name,
            "views_file": cli.views.display().to_string(),
            "quick": cli.quick,
            "only": cli.only,
            "cell_size": [CELL_W, CELL_H],
            "baseline": base_path.display().to_string(),
            "seconds": started.elapsed().as_secs_f64(),
            "views": views_json,
            "cells": cells_json,
        });
        let json_path = out.join("coloring-matrix.json");
        if let Err(e) = std::fs::write(&json_path, serde_json::to_string_pretty(&report).unwrap_or_default()) {
            eprintln!("coloring-matrix: {}: {e}", json_path.display());
        }
        // The summary a person reads first.
        let mut md = String::new();
        md.push_str("# Coloring matrix\n\n");
        md.push_str(&format!(
            "{} on {}, {}. {} views, {} cells ({} n/a), {:.0} s. Sheets: `views/` (one per view, 1:1) and `variants/` (one per variant, half size); a failed comparison's evidence in `ab/` (A, B, and where they differ: gray |Δ| x 4, dark blue a steep pixel, red a difference that counts).\n\n",
            crate::version_string(),
            self.gpu_name,
            crate::utc_string(secs),
            views_run.len(),
            cells.len(),
            not_offered,
            started.elapsed().as_secs_f64()
        ));
        md.push_str(&format!("**{} red** (new), {} known, {} slower than the baseline.\n\n", red.len(), known_hit.len(), slower.len()));
        if !red.is_empty() {
            md.push_str("## Red cells\n\n| cell | check | detail |\n|---|---|---|\n");
            for (id, c, d) in &red {
                md.push_str(&format!("| {id} | {c} | {d} |\n"));
            }
            md.push('\n');
        }
        if !known_hit.is_empty() {
            md.push_str("## Known red cells\n\n| cell | check | detail | reason |\n|---|---|---|---|\n");
            for (id, c, d, r) in &known_hit {
                md.push_str(&format!("| {id} | {c} | {d} | {r} |\n"));
            }
            md.push('\n');
        }
        if !slower.is_empty() {
            md.push_str(&format!(
                "## Slower than the baseline (warning: > +{:.0}% and > {TIMING_FLOOR_MS} ms)\n\n| cell | iterate ms | baseline ms |\n|---|---|---|\n",
                TIMING_REGRESSION * 100.0
            ));
            for (id, now, base) in &slower {
                md.push_str(&format!("| {id} | {now:.2} | {base:.2} |\n"));
            }
            md.push('\n');
        }
        md.push_str("## Views\n\n| view | family | zoom | band | iter | interior |\n|---|---|---|---|---|---|\n");
        for (vi, s) in &stats {
            let v = &set.views[*vi];
            md.push_str(&format!(
                "| {} | {}{} | {} | {} | {} | {:.0}% |\n",
                v.name,
                v.fractal.name(),
                v.julia_c.map(|(a, b)| format!(" Julia {a}{b:+}i")).unwrap_or_default(),
                v.zoom,
                band_name(s.mode),
                v.iter,
                s.interior * 100.0
            ));
        }
        if let Err(e) = std::fs::write(out.join("summary.md"), md) {
            eprintln!("coloring-matrix: summary: {e}");
        }
        // --bless: this run's timings into this card's baseline, over what it already holds.
        if cli.bless {
            let mut merged = baseline.clone();
            for c in &cells {
                if let Outcome::Ran(m) = &c.outcome {
                    if let (Some(it), Some(col)) = (m.iterate_ms, m.color_ms) {
                        merged.insert(format!("{}/{}", set.views[c.view].name, c.variant.key()), (it, col));
                    }
                }
            }
            let cells: serde_json::Map<String, serde_json::Value> = merged
                .into_iter()
                .map(|(k, (it, col))| (k, serde_json::json!({"iterate_ms": it, "color_ms": col})))
                .collect();
            let doc = serde_json::json!({
                "tool": "fractadyne --coloring-matrix --bless",
                "gpu": self.gpu_name,
                "version": crate::version_string(),
                "utc": crate::utc_string(secs),
                "cells": cells,
            });
            match std::fs::write(&base_path, serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n") {
                Ok(()) => println!("\nbaseline written: {}", base_path.display()),
                Err(e) => eprintln!("\ncoloring-matrix: baseline {}: {e}", base_path.display()),
            }
        }
        println!(
            "\ncoloring matrix: {} cells, {} red, {} known, {} slower, {} n/a ({:.0} s) -> {}",
            cells.len(),
            red.len(),
            known_hit.len(),
            slower.len(),
            not_offered,
            started.elapsed().as_secs_f64(),
            out.display()
        );
        for (id, c, d) in &red {
            println!("  RED {id}: {c}: {d}");
        }
        Verdict {
            code: i32::from(!red.is_empty()),
            cells: cells.len(),
            red: red.iter().map(|(id, c, d)| format!("{id}: {c}: {d}")).collect(),
            known: known_hit.len(),
            not_offered,
        }
    }

    /// What the matrix changes in the session, saved so a caller that goes on can put it back.
    pub(crate) fn coloring_matrix_saved(&self) -> Saved {
        Saved {
            fractal: self.fractal,
            custom: self.custom.clone(),
            julia_mode: self.julia_mode,
            julia_c: self.julia_c,
            dual: self.dual,
            viewport: self.viewport.clone(),
            max_iter: self.render_cfg.max_iter,
            auto_iter: self.render_cfg.auto_iter,
            series_approx: self.render_cfg.series_approx,
            use_bla: self.render_cfg.use_bla,
            glitch_correct: self.render_cfg.glitch_correct,
            aa: self.render_cfg.aa,
            palette_idx: self.coloring.palette_idx,
            cycle: self.coloring.cycle,
            offset: self.coloring.offset,
            iter_div: self.coloring.iter_div,
            use_custom_palette: self.coloring.use_custom_palette,
            normalize: self.coloring.normalize,
            palette_anim: self.anim.palette_anim,
            view_coloring: self.view_coloring(),
            export_width: self.export.width,
            export_ss: self.export.ss,
        }
    }

    /// Put back what [`Self::coloring_matrix_saved`] saved.
    pub(crate) fn coloring_matrix_restore(&mut self, s: Saved) {
        self.fractal = s.fractal;
        self.custom = s.custom;
        self.julia_mode = s.julia_mode;
        self.julia_c = s.julia_c;
        self.dual = s.dual;
        self.viewport = s.viewport;
        self.render_cfg.max_iter = s.max_iter;
        self.render_cfg.auto_iter = s.auto_iter;
        self.render_cfg.series_approx = s.series_approx;
        self.render_cfg.use_bla = s.use_bla;
        self.render_cfg.glitch_correct = s.glitch_correct;
        self.render_cfg.aa = s.aa;
        self.coloring.palette_idx = s.palette_idx;
        self.coloring.cycle = s.cycle;
        self.coloring.offset = s.offset;
        self.coloring.iter_div = s.iter_div;
        self.coloring.use_custom_palette = s.use_custom_palette;
        self.coloring.normalize = s.normalize;
        self.anim.palette_anim = s.palette_anim;
        self.set_view_coloring(&s.view_coloring);
        self.export.width = s.export_width;
        self.export.ss = s.export_ss;
        self.invalidate_refs();
    }
}

#[cfg(test)]
#[path = "../../tests/unit/harness/coloring_matrix/tests.rs"]
mod tests;
