//! L-systems in the app (design/lsystems.md, phase 2): the session's system, the walk for the
//! view, the side-panel section, the files, and the frame that hands the segments to the GPU.
//!
//! **The walk runs off the UI thread.** A view's walk is bounded by its pixels, but a big window
//! at a dense order is still millions of segments — tens of milliseconds. So a frame asks for the
//! walk its view needs and draws the last one finished, moved and scaled to where its view now
//! sits (`LSystemFrame::scale`/`offset`); at most one walk runs at a time, and when it finishes
//! the next starts for wherever the view has got to. A zoom therefore shows the picture following
//! at the walk's rate, never a blank.

use crate::{FractadyneApp, FractalKind};
use fractadyne_core::lsystem::{
    self, library, BigTables, Colouring, DeepView, Drawn, Expansion, LEntry, LSystem, Tables, View, WalkOptions, WalkStats,
};
use fractadyne_core::BigFloat;
use fractadyne_gpu::lsystem::{LSystemFrame, SegmentInstance, TriangleInstance};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// The most segments a walk hands over (the GPU pass's cap is above it).
const BUDGET: u64 = 4_000_000;
/// The order that follows the zoom puts a step of the curve at most this many pixels — or
/// [`STEP_WIDTHS`] line widths, if more: a space-filling curve drawn at a step no wider than its
/// lines is a solid block (the Hilbert curve's first screenshot), so the step leaves room
/// between them…
const STEP_PX: f64 = 3.0;
const STEP_WIDTHS: f64 = 2.5;
/// …and a subtree reaching less than this many pixels is drawn as its chord.
const LOD_PX: f64 = 1.5;
/// The order a system that does not grow by a factor is drawn at, unless it names one.
const DEFAULT_ORDER: u32 = 6;
/// The system a new session opens with.
const DEFAULT_SYSTEM: &str = "Heighway dragon";
/// The largest system text a view file carries (one escaped line).
pub(crate) const MAX_VIEW_SYSTEM: usize = 64 * 1024;

/// What a walk was for: when the view's key differs from the shown walk's, a new walk is due.
#[derive(Clone, Debug, PartialEq)]
struct WalkKey {
    /// The system shown (its tables change with its angle; it does not).
    system: u64,
    tables: u64,
    order: u32,
    /// The view's centre, exact.
    centre: [BigFloat; 2],
    /// `log₂` of world units a pixel.
    upp_log2: f64,
    size: [u32; 2],
    colouring: Colouring,
    margin: f32,
    /// Walked in `BigFloat` ([`lsystem::deep_walk`]): the picture is past what `f64` places.
    deep: bool,
}

/// A finished walk.
struct Walked {
    key: WalkKey,
    segments: Arc<Vec<SegmentInstance>>,
    triangles: Arc<Vec<TriangleInstance>>,
    id: u64,
    stats: WalkStats,
    ms: f64,
    /// The deep tables it used, to reuse for the next deep walk.
    big: Option<Arc<BigTables>>,
    /// A parametric or context-sensitive system's word, to reuse at the same order.
    ex: Option<Arc<Expansion>>,
}

/// The last walk's numbers, for the readouts.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LastWalk {
    pub(crate) order: u32,
    pub(crate) segments: u64,
    /// Filled shapes.
    pub(crate) polygons: u64,
    /// It stopped at the budget.
    pub(crate) stopped: bool,
    pub(crate) ms: f64,
    /// A word built short of its order (the next would pass the budget): the order built, and the
    /// one asked for.
    pub(crate) built: Option<(u32, u32)>,
}

/// A walk in progress on its own thread.
struct Job {
    result: Arc<Mutex<Option<Walked>>>,
    done: Arc<AtomicBool>,
}

pub(crate) struct LSystemState {
    /// The system as opened or edited (its own angle, not the slider's).
    pub(crate) system: LSystem,
    /// Its tables at the angle drawn.
    pub(crate) tables: Arc<Tables>,
    tables_id: u64,
    /// A fixed order (`None`: the order follows the zoom, or is the system's own).
    pub(crate) fixed_order: Option<u32>,
    /// The angle slider's override, degrees (`None`: the system's).
    pub(crate) angle: Option<f64>,
    /// The line width, pixels.
    pub(crate) width: f32,
    /// The colouring (`None`: the system's default).
    pub(crate) colour: Option<Colouring>,
    /// The editor window, its text, and why the last Apply was refused.
    pub(crate) editor_open: bool,
    pub(crate) editor_text: String,
    pub(crate) editor_error: Option<String>,
    /// A `.l` file's entries, while the user picks one.
    pub(crate) entries: Option<(String, Vec<LEntry>)>,
    /// Shown in the panel: why the last open or save failed.
    pub(crate) message: Option<String>,
    shown: Option<Walked>,
    job: Option<Job>,
    next_id: u64,
    /// The deep tables of the last deep walk, and the tables they belong to.
    big: Option<(u64, Arc<BigTables>)>,
    /// What the home view framed, while the view is still exactly that.
    framed: Option<Framed>,
    /// A parametric or context-sensitive system's last word: the tables it is for, its order.
    expansion: Option<(u64, u32, Arc<Expansion>)>,
    /// Which system this is: a new one for each system opened, the same through angle and seed
    /// changes — so a walk of the angle a moment ago is drawn while the next one runs.
    system_id: u64,
    /// How much of the curve is drawn (1: all of it), and whether it is drawing itself on: the
    /// time of the last step.
    pub(crate) progress: f32,
    draw_anim: Option<std::time::Instant>,
    /// The angle sweeping: the last step's time and the direction (+1 or −1).
    angle_anim: Option<(std::time::Instant, f64)>,
}

/// How long the curve takes to draw itself on, whole.
const DRAW_SECONDS: f64 = 8.0;
/// How fast the angle sweeps, degrees a second (between its bounds, there and back).
const ANGLE_SPEED: f64 = 4.0;
const ANGLE_RANGE: [f64; 2] = [1.0, 179.0];

/// A home view's framing: the picture's box (`None`: it draws nothing), and the canvas size and
/// view it set. A canvas that changes size before the user moves — the first layout after a file
/// opens the app, before the side panel took its width — is framed again for its new size.
#[derive(Clone, Copy, Debug)]
struct Framed {
    bounds: Option<[f64; 4]>,
    size: [f64; 2],
    centre: [f64; 2],
    upp: f64,
}

/// The library system called `name`, or the default.
fn library_system(name: &str) -> LSystem {
    library::find(name)
        .or_else(|| library::find(DEFAULT_SYSTEM))
        .and_then(|e| e.system().ok())
        .expect("the library parses")
}

impl Default for LSystemState {
    fn default() -> Self {
        let system = library_system(DEFAULT_SYSTEM);
        let tables = Arc::new(Tables::new(&system));
        LSystemState {
            system,
            tables,
            tables_id: 1,
            fixed_order: None,
            angle: None,
            width: 1.5,
            colour: None,
            editor_open: false,
            editor_text: String::new(),
            editor_error: None,
            entries: None,
            message: None,
            shown: None,
            job: None,
            next_id: 1,
            big: None,
            framed: None,
            expansion: None,
            system_id: 1,
            progress: 1.0,
            draw_anim: None,
            angle_anim: None,
        }
    }
}

/// The deepest bracket nesting in a word.
fn nesting(word: &[lsystem::Tok]) -> u32 {
    let (mut d, mut max) = (0u32, 0u32);
    for t in word {
        match t {
            lsystem::Tok::Push => {
                d += 1;
                max = max.max(d);
            }
            lsystem::Tok::Pop => d = d.saturating_sub(1),
            _ => {}
        }
    }
    max
}

impl LSystemState {
    /// The system as drawn: with the slider's angle, if it moved.
    pub(crate) fn drawn_system(&self) -> LSystem {
        let mut s = self.system.clone();
        if let Some(a) = self.angle {
            s.angle = lsystem::Angle::Degrees(a);
        }
        s
    }

    /// Rebuild the tables after the system or the angle changed.
    fn rebuild(&mut self) {
        self.tables = Arc::new(Tables::new(&self.drawn_system()));
        self.tables_id += 1;
    }

    /// Make `system` the one shown, with the slider's angle and a fixed order cleared.
    pub(crate) fn set_system(&mut self, system: LSystem) {
        self.system = system;
        self.angle = None;
        self.fixed_order = None;
        self.system_id += 1;
        self.progress = 1.0;
        self.draw_anim = None;
        self.angle_anim = None;
        self.rebuild();
    }

    /// Whether the curve is drawing itself on, or its angle sweeping.
    pub(crate) fn animating(&self) -> bool {
        self.draw_anim.is_some() || self.angle_anim.is_some()
    }

    /// Start (from the beginning, if it was all drawn) or stop the curve drawing itself on.
    pub(crate) fn toggle_draw_on(&mut self) {
        if self.draw_anim.take().is_none() {
            if self.progress >= 1.0 {
                self.progress = 0.0;
            }
            self.draw_anim = Some(std::time::Instant::now());
        }
    }

    /// Start or stop the angle sweeping (from wherever it is).
    pub(crate) fn toggle_angle_sweep(&mut self) {
        if self.angle_anim.take().is_none() {
            self.angle_anim = Some((std::time::Instant::now(), 1.0));
        }
    }

    /// Advance the animations to now.
    fn animate(&mut self) {
        let now = std::time::Instant::now();
        if let Some(last) = self.draw_anim {
            let dt = now.duration_since(last).as_secs_f64();
            self.progress = (f64::from(self.progress) + dt / DRAW_SECONDS).min(1.0) as f32;
            self.draw_anim = (self.progress < 1.0).then_some(now);
        }
        if let Some((last, dir)) = self.angle_anim {
            let dt = now.duration_since(last).as_secs_f64().min(0.25);
            let mut a = self.angle.unwrap_or_else(|| self.system.angle.degrees()) + dir * ANGLE_SPEED * dt;
            let mut dir = dir;
            // There and back between the bounds.
            if a > ANGLE_RANGE[1] {
                a = 2.0 * ANGLE_RANGE[1] - a;
                dir = -1.0;
            } else if a < ANGLE_RANGE[0] {
                a = 2.0 * ANGLE_RANGE[0] - a;
                dir = 1.0;
            }
            self.angle_anim = Some((now, dir));
            self.set_angle(Some(a));
        }
    }

    /// Set a stochastic system's seed (the view stays where it is: the plant is about the same
    /// size whatever its seed).
    pub(crate) fn set_seed(&mut self, seed: u64) {
        if self.system.seed != seed {
            self.system.seed = seed;
            self.rebuild();
        }
    }

    /// Set the angle override (`None`: the system's own).
    pub(crate) fn set_angle(&mut self, angle: Option<f64>) {
        if self.angle != angle {
            self.angle = angle;
            self.rebuild();
        }
    }

    pub(crate) fn colouring(&self) -> Colouring {
        self.colour.unwrap_or_else(|| self.system.colouring())
    }

    /// The order to draw at, `2^upp_log2` world units a pixel — at any zoom: past the `f64`
    /// tables' depth, the deep walk draws it.
    pub(crate) fn order_for(&self, upp_log2: f64) -> u32 {
        let t = &self.tables;
        // A parametric or context-sensitive system is built at a fixed order (its tables are empty).
        if self.system.expanded.is_some() {
            return self.fixed_order.unwrap_or(self.system.order.unwrap_or(DEFAULT_ORDER)).min(lsystem::MAX_ORDER);
        }
        let step_px = STEP_PX.max(STEP_WIDTHS * f64::from(self.width));
        let n = match self.fixed_order {
            Some(n) => n.min(t.max_depth.max(lsystem::MAX_ORDER.min(n))),
            None => t.auto_order_log2(-upp_log2, step_px).unwrap_or_else(|| self.system.order.unwrap_or(DEFAULT_ORDER).min(t.max_depth)),
        };
        n.min(lsystem::MAX_ORDER)
    }

    /// The order the viewport draws at.
    pub(crate) fn order_at(&self, vp: &fractadyne_core::Viewport) -> u32 {
        self.order_for(vp.units_per_pixel.log2())
    }

    /// Whether a view at `upp_log2` and `order` needs the deep walk: the picture is larger than the
    /// `f64` walk places to a fraction of a pixel ([`Tables::f64_reach_log2`]), or the order is
    /// past the `f64` tables.
    fn needs_deep(&self, upp_log2: f64, order: u32) -> bool {
        if self.system.expanded.is_some() {
            return false; // built as a word, in f64
        }
        let t = &self.tables;
        let picture_px = if t.box_size > 0.0 { t.box_size.log2() - upp_log2 } else { -upp_log2 };
        order > t.max_depth || picture_px > t.f64_reach_log2()
    }

    /// The bracket depth that colours as the palette's end at `order`: the most a branch can nest.
    fn depth_scale(&self, order: u32) -> f64 {
        let deepest = self.system.rules.iter().flatten().map(|p| nesting(&p.word)).max().unwrap_or(0);
        f64::from(nesting(&self.system.axiom) + order * deepest).max(1.0)
    }

    /// Whether a walk is running or an animation playing (the view should keep repainting).
    pub(crate) fn busy(&self) -> bool {
        self.job.is_some() || self.animating()
    }

    /// The last walk's segments, those the draw-on progress has reached and those it has not.
    pub(crate) fn drawn_split(&self) -> Option<(usize, usize)> {
        let w = self.shown.as_ref()?;
        let drawn = w.segments.iter().filter(|s| s.t[0] < self.progress).count();
        Some((drawn, w.segments.len() - drawn))
    }

    /// The last walk's numbers.
    pub(crate) fn last_walk(&self) -> Option<LastWalk> {
        self.shown.as_ref().map(|w| LastWalk {
            order: w.key.order,
            segments: w.stats.segments,
            polygons: w.stats.polygons,
            stopped: w.stats.stopped,
            ms: w.ms,
            built: w.ex.as_ref().filter(|x| x.short()).map(|x| (x.order, x.wanted)),
        })
    }

    /// Take a finished walk; start the next one when the view needs it.
    fn drive(&mut self, want: WalkKey) {
        if let Some(job) = &self.job {
            if job.done.load(Ordering::Acquire) {
                let job = self.job.take().expect("a job");
                if let Some(w) = job.result.lock().ok().and_then(|mut r| r.take()) {
                    // Another system's walk is not this one's picture; a walk at the angle a
                    // moment ago is (while the angle sweeps, every walk lands a step behind it).
                    if w.key.system == self.system_id {
                        if w.key.tables == self.tables_id {
                            if let Some(b) = &w.big {
                                self.big = Some((w.key.tables, b.clone()));
                            }
                            if let Some(x) = &w.ex {
                                self.expansion = Some((w.key.tables, w.key.order, x.clone()));
                            }
                        }
                        self.shown = Some(w);
                    }
                }
            }
        }
        if self.job.is_none() && self.shown.as_ref().is_none_or(|w| w.key != want) {
            self.start(want);
        }
        // A system changed under the shown walk: drop it rather than draw another system's picture.
        if self.shown.as_ref().is_some_and(|w| w.key.system != self.system_id) {
            self.shown = None;
        }
    }

    fn start(&mut self, key: WalkKey) {
        let tables = self.tables.clone();
        let system = self.drawn_system();
        // The last deep tables, if they are this system's.
        let big = self.big.as_ref().filter(|(id, _)| *id == key.tables).map(|(_, b)| b.clone());
        // The last word, if it is this system's at this order.
        let ex = self.expansion.as_ref().filter(|(id, o, _)| *id == key.tables && *o == key.order).map(|(_, _, x)| x.clone());
        let result = Arc::new(Mutex::new(None));
        let done = Arc::new(AtomicBool::new(false));
        let id = self.next_id;
        self.next_id += 1;
        let depth_scale = self.depth_scale(key.order);
        let (r, d) = (result.clone(), done.clone());
        std::thread::Builder::new()
            .name("lsystem-walk".into())
            .spawn(move || {
                let t0 = std::time::Instant::now();
                let WalkOut { segments, triangles, stats, big, ex, .. } = walk_segments(&system, &tables, big, ex, &key, depth_scale);
                let ms = t0.elapsed().as_secs_f64() * 1e3;
                if let Ok(mut slot) = r.lock() {
                    *slot = Some(Walked { key, segments: Arc::new(segments), triangles: Arc::new(triangles), id, stats, ms, big, ex });
                }
                d.store(true, Ordering::Release);
            })
            .expect("a thread for the walk");
        self.job = Some(Job { result, done });
    }
}

/// Walks `key`'s view — in `f64`, or deep through `BigTables` (reusing `big` when it is precise
/// and deep enough) — and colours each segment. Returns the deep tables it used.
/// What a walk hands the GPU.
struct WalkOut {
    segments: Vec<SegmentInstance>,
    triangles: Vec<TriangleInstance>,
    /// The filled shapes' outlines (clipped to the view) and values, for SVG.
    outlines: Vec<(Vec<[f32; 2]>, f32)>,
    stats: WalkStats,
    big: Option<Arc<BigTables>>,
    /// A parametric or context-sensitive system's word (built, or `ex` handed back).
    ex: Option<Arc<Expansion>>,
}

fn walk_segments(
    sys: &LSystem,
    t: &Tables,
    big: Option<Arc<BigTables>>,
    ex: Option<Arc<Expansion>>,
    key: &WalkKey,
    depth_scale: f64,
) -> WalkOut {
    let size = [f64::from(key.size[0]), f64::from(key.size[1])];
    let margin = f64::from(key.margin);
    // A parametric or context-sensitive system: its word at this order (the last one, if it was
    // built for this order), its brackets the depth colouring's scale.
    let ex = sys.expanded.as_ref().map(|e| ex.unwrap_or_else(|| Arc::new(lsystem::expand::expand(sys, e, key.order, lsystem::EXPAND_BUDGET))));
    let depth_scale = ex.as_ref().map_or(depth_scale, |x| f64::from(x.max_brackets).max(1.0));
    let opts = WalkOptions { order: key.order, lod_px: LOD_PX, budget: BUDGET };
    let (mut segments, mut triangles, mut outlines) = (Vec::new(), Vec::new(), Vec::new());
    let value_of = |index: f64, span: f64, depth: u16, heading: f64, colour: i32, total: f64| {
        let value = match key.colouring {
            Colouring::Position => (index + 0.5 * span) / total,
            Colouring::Depth => (f64::from(depth) / depth_scale).min(1.0),
            Colouring::Heading => heading,
            Colouring::Index => (f64::from(colour.rem_euclid(16)) + 0.5) / 16.0,
            Colouring::Plain => 0.5,
        };
        // Past f64's integers an index is approximate, and past its range not a number: the
        // value must stay a palette position (the shader reads < 0 as "no line").
        (if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.5 }) as f32
    };
    // A polygon is clipped to a little beyond the view (it may reach 1e30 pixels past it), then cut
    // into triangles.
    let clip = [0.5 * size[0] + margin + 4.0, 0.5 * size[1] + margin + 4.0];
    // Where along the curve, 0 to 1 (what the draw-on animation reveals by).
    let along = |index: f64, total: f64| {
        let t = index / total;
        (if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 }) as f32
    };
    let mut take = |d: Drawn, total: f64| match d {
        Drawn::Segment(s) => segments.push(SegmentInstance {
            a: [s.a[0] as f32, s.a[1] as f32],
            b: [s.b[0] as f32, s.b[1] as f32],
            value: value_of(s.index, s.span, s.depth, s.heading, s.colour, total),
            t: [along(s.index, total), along(s.index + s.span, total)],
        }),
        Drawn::Polygon(p) => {
            let value = value_of(p.index, 0.0, p.depth, p.heading, p.colour, total);
            let pts = lsystem::polygon::clip_to_rect(&p.pts, clip);
            let f = |q: [f64; 2]| [q[0] as f32, q[1] as f32];
            let t = along(p.index, total);
            for [i, j, k] in lsystem::polygon::triangulate(&pts) {
                triangles.push(TriangleInstance { a: f(pts[i]), b: f(pts[j]), c: f(pts[k]), value, t });
            }
            // The outline too, for SVG (a filled polygon, where triangles would show seams).
            outlines.push((pts.iter().map(|&q| f(q)).collect(), value));
        }
    };
    if key.deep {
        let extent = if t.box_size > 0.0 { t.box_size.log2() } else { 0.0 };
        // Rounded up to 64 bits, so a slowly deepening zoom reuses its tables.
        let p = lsystem::deep_precision(t, key.order, key.upp_log2, extent).div_ceil(64) * 64;
        let bt = match big.filter(|b| b.prec >= p && b.depth >= key.order) {
            Some(b) => Some(b),
            // A few orders' room, for the zoom that comes next.
            None => BigTables::new(sys, t, p, key.order + 8).map(Arc::new),
        };
        if let Some(bt) = bt {
            let view = DeepView { centre: key.centre.clone(), upp_log2: key.upp_log2, size, margin };
            let total = bt.axiom_count(key.order).max(1.0);
            let stats = lsystem::deep_walk_all(t, &bt, &view, &opts, lsystem::switch_px(t), &mut |d| take(d, total));
            return WalkOut { segments, triangles, outlines, stats, big: Some(bt), ex: None };
        }
        // Angles with no common unit (a decimal of more than 18 places): f64, as far as it goes.
    }
    let view = View {
        centre: [fractadyne_core::to_f64(&key.centre[0]), fractadyne_core::to_f64(&key.centre[1])],
        upp: key.upp_log2.exp2(),
        size,
        margin,
    };
    if let Some(x) = ex {
        let total = (x.segments as f64).max(1.0);
        let stats = lsystem::expand::draw(sys, &x, &view, BUDGET, &mut |d| take(d, total));
        return WalkOut { segments, triangles, outlines, stats, big: None, ex: Some(x) };
    }
    let total = t.axiom_entry(key.order.min(t.max_depth)).n.max(1.0);
    let stats = lsystem::walk_all(t, &view, &opts, &mut |d| take(d, total));
    WalkOut { segments, triangles, outlines, stats, big: None, ex: None }
}

/// `d / 2^log2_den` as an `f64`, for any magnitudes (a centre difference at 1e400×).
fn ratio_f64(d: &BigFloat, log2_den: f64) -> f64 {
    if d.is_zero() {
        return 0.0;
    }
    let mag = (fractadyne_core::log2_abs(d) - log2_den).exp2();
    if d.is_negative() {
        -mag
    } else {
        mag
    }
}

impl FractadyneApp {
    /// The L-system frame: the walk this view needs (started off-thread if it is not the one
    /// shown), and the last walk, placed under the view.
    /// The walk the view needs, at `size` pixels.
    fn lsystem_walk_key(&self, size: [u32; 2]) -> WalkKey {
        let upp_log2 = self.viewport.units_per_pixel.log2();
        let order = self.lsystem.order_for(upp_log2);
        WalkKey {
            system: self.lsystem.system_id,
            tables: self.lsystem.tables_id,
            order,
            centre: [self.viewport.center_x.clone(), self.viewport.center_y.clone()],
            upp_log2,
            size,
            colouring: self.lsystem.colouring(),
            margin: 0.5 * self.lsystem.width + 1.0,
            deep: self.lsystem.needs_deep(upp_log2, order),
        }
    }

    pub(crate) fn build_lsystem_params(&mut self, resolution: [u32; 2], ss: u32) -> fractadyne_gpu::MandelbrotParams {
        self.lsystem.animate();
        let want = self.lsystem_walk_key(resolution);
        let (centre, upp_log2) = (want.centre.clone(), want.upp_log2);
        self.lsystem.drive(want);
        let p = self.viewport.precision.max(64);
        let frame = match &self.lsystem.shown {
            Some(w) => LSystemFrame {
                segments: w.segments.clone(),
                triangles: w.triangles.clone(),
                segments_id: w.id,
                // A walked pixel is at world `p·upp_w + c_w`; in this view, `(world − c)/upp`.
                scale: (w.key.upp_log2 - upp_log2).exp2() as f32,
                offset: [
                    ratio_f64(&fractadyne_core::bf_sub(&w.key.centre[0], &centre[0], p), upp_log2) as f32,
                    ratio_f64(&fractadyne_core::bf_sub(&w.key.centre[1], &centre[1], p), upp_log2) as f32,
                ],
                width: self.lsystem.width,
                progress: self.lsystem.progress,
            },
            None => LSystemFrame {
                segments: Arc::new(Vec::new()),
                triangles: Arc::new(Vec::new()),
                segments_id: 0,
                scale: 1.0,
                offset: [0.0, 0.0],
                width: self.lsystem.width,
                progress: 1.0,
            },
        };
        let (lut, lut_smooth) = self.active_lut();
        fractadyne_gpu::MandelbrotParams {
            lsystem: Some(Arc::new(frame)),
            formula: fractadyne_core::formula::LSYSTEM,
            lut,
            lut_smooth,
            // Values run over [0, 1): one palette cycle along the curve (or over the depths, the
            // headings), rotated by the offset.
            cycle: 1.0,
            offset: self.coloring.offset,
            interior_col: self.interior_color(),
            aa_palette: crate::palette_aa_enabled(),
            resolution,
            ss,
            view_id: 0,
            ..Default::default()
        }
    }

    /// Frame the picture: its bounding box (at an order cheap to walk whole) with a margin.
    pub(crate) fn lsystem_home(&mut self) {
        // A parametric or context-sensitive system: its word at its order (kept for the walk).
        if let Some(e) = self.lsystem.system.expanded.clone() {
            let order = self.lsystem.order_for(0.0);
            let sys = self.lsystem.drawn_system();
            let id = self.lsystem.tables_id;
            let x = match self.lsystem.expansion.as_ref().filter(|(i, o, _)| *i == id && *o == order) {
                Some((_, _, x)) => x.clone(),
                None => Arc::new(lsystem::expand::expand(&sys, &e, order, lsystem::EXPAND_BUDGET)),
            };
            let b = lsystem::expand::bounds(&sys, &x);
            self.lsystem.expansion = Some((id, order, x));
            self.lsystem_frame(b);
            return;
        }
        let t = self.lsystem.tables.clone();
        // A growing picture frames the same at any order; one drawn at a fixed order (its own, or
        // the user's) frames at that order — Bourke's mango leaf at order 18 is a corner of
        // itself at order 300.
        let frame = lsystem::framing_order(&t, 200_000.0);
        let order = if self.lsystem.fixed_order.is_some() || !t.grows() {
            t.in_phase(self.lsystem.order_for(0.0).min(frame))
        } else {
            frame
        };
        self.lsystem_frame(lsystem::bounds(&t, order, 1 << 21));
    }

    /// The home view's centre and `log₁₀` magnification, without moving to it — what a tour that
    /// starts framed starts from (Tools ▸ Tour from current view).
    pub(crate) fn lsystem_home_view(&self) -> ([f64; 2], f64) {
        let order = self.lsystem.order_for(0.0);
        let sys = self.lsystem.drawn_system();
        let bounds = match &sys.expanded {
            Some(e) => lsystem::expand::bounds(&sys, &lsystem::expand::expand(&sys, e, order, lsystem::EXPAND_BUDGET)),
            None => {
                let t = &self.lsystem.tables;
                let frame = lsystem::framing_order(t, 200_000.0);
                let o = if self.lsystem.fixed_order.is_some() || !t.grows() { t.in_phase(order.min(frame)) } else { frame };
                lsystem::bounds(t, o, 1 << 21)
            }
        };
        let (w, h) = (self.viewport.width_px.max(1.0), self.viewport.height_px.max(1.0));
        let (c, upp) = match bounds {
            Some(b) => ([(b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5], ((b[2] - b[0]) / w).max((b[3] - b[1]) / h).max(1e-9) * 1.15),
            None => ([0.0, 0.0], 4.0 / h),
        };
        (c, (fractadyne_core::Viewport::REFERENCE_HEIGHT / (upp * h)).log10())
    }

    /// A tour frame's L-system state (scripting): its system, order, angle and drawing. The panel's
    /// animations stop — the tour drives them now.
    pub(crate) fn apply_tour_lsystem(&mut self, t: &crate::scripting::LsTour) {
        let st = &mut self.lsystem;
        if let Some(s) = &t.system {
            if st.system != **s {
                st.set_system((**s).clone());
            }
        }
        st.fixed_order = t.order;
        st.set_angle(t.angle);
        st.progress = t.draw.unwrap_or(1.0);
        st.draw_anim = None;
        st.angle_anim = None;
    }

    /// Fit `bounds` (world units) to the canvas, with a margin.
    fn lsystem_frame(&mut self, bounds: Option<[f64; 4]>) {
        let size = [self.viewport.width_px, self.viewport.height_px];
        let (w, h) = (size[0].max(1.0), size[1].max(1.0));
        let (cx, cy, upp) = match bounds {
            Some(b) => {
                let span = ((b[2] - b[0]) / w).max((b[3] - b[1]) / h).max(1e-9) * 1.15;
                ((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5, span)
            }
            None => (0.0, 0.0, 4.0 / h),
        };
        self.viewport.reset_to(cx, cy);
        self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(upp);
        self.pointer.zoom_vel = 0.0;
        self.lsystem.framed = Some(Framed { bounds, size, centre: [cx, cy], upp });
    }

    /// After the canvas is sized: a view still exactly as the home view framed it, on a canvas of
    /// another size, is framed again for this one (see [`Framed`]); a view the user moved stays.
    pub(crate) fn lsystem_keep_framed(&mut self) {
        let Some(f) = self.lsystem.framed else { return };
        if [self.viewport.width_px, self.viewport.height_px] == f.size {
            return;
        }
        let untouched = self.viewport.units_per_pixel.to_f64() == f.upp
            && fractadyne_core::to_f64(&self.viewport.center_x) == f.centre[0]
            && fractadyne_core::to_f64(&self.viewport.center_y) == f.centre[1];
        if untouched {
            self.lsystem_frame(f.bounds);
        } else {
            self.lsystem.framed = None;
        }
    }

    /// Show `system`, framed.
    pub(crate) fn lsystem_open(&mut self, system: LSystem) {
        self.lsystem.set_system(system);
        self.lsystem.message = None;
        if self.fractal != FractalKind::LSystem {
            self.set_fractal(FractalKind::LSystem);
        }
        self.lsystem_home();
    }

    /// Open a system from the library.
    pub(crate) fn lsystem_open_library(&mut self, name: &str) {
        if let Some(Ok(s)) = library::find(name).map(|e| e.system()) {
            self.lsystem_open(s);
        }
    }

    /// Read `text` as a system: a Fractint `.l` file (its entries to pick from, or the one) or the
    /// native format. `name` names a native system that does not name itself.
    pub(crate) fn lsystem_open_text(&mut self, text: &str, name: &str) -> Result<(), String> {
        if text.contains('{') {
            let entries = lsystem::parse_l_file(text);
            let good: Vec<&LEntry> = entries.iter().filter(|e| e.system.is_ok()).collect();
            match (entries.len(), good.len()) {
                (0, _) => return Err("no entries (a .l file holds `Name { … }` entries)".into()),
                (1, 1) => {
                    let s = good[0].system.clone().expect("checked");
                    self.lsystem_open(s);
                }
                (1, 0) => {
                    let e = entries[0].system.as_ref().expect_err("checked");
                    return Err(format!("{}: {e}", entries[0].name));
                }
                _ => self.lsystem.entries = Some((name.to_string(), entries)),
            }
            return Ok(());
        }
        let mut s = LSystem::parse(text).map_err(|e| e.to_string())?;
        if s.name.is_empty() {
            s.name = name.to_string();
        }
        self.lsystem_open(s);
        Ok(())
    }

    /// File > Open L-system…: a Fractint `.l` file or the native format.
    pub(crate) fn lsystem_open_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("L-systems", &["l", "lsys", "txt"])
            .set_directory(self.dialog_dir_default())
            .pick_file()
        else {
            return;
        };
        self.remember_dir(&path);
        let name = path.file_stem().map_or_else(|| "L-system".into(), |s| s.to_string_lossy().into_owned());
        let read = std::fs::metadata(&path)
            .map_err(|e| e.to_string())
            .and_then(|m| if m.len() > 4 << 20 { Err("larger than 4 MiB".into()) } else { Ok(()) })
            .and_then(|()| std::fs::read(&path).map_err(|e| e.to_string()))
            .map(|b| String::from_utf8_lossy(&b).into_owned());
        let result = read.and_then(|t| self.lsystem_open_text(&t, &name));
        if let Err(e) = result {
            self.lsystem.message = Some(format!("Could not open {}: {e}", path.display()));
        }
    }

    /// File > Save L-system…: the system in the native format.
    pub(crate) fn lsystem_save_file(&mut self) {
        let s = self.lsystem.system.clone();
        let stem: String = s.name.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
        let Some(path) = rfd::FileDialog::new()
            .add_filter("L-system", &["lsys"])
            .set_directory(self.dialog_dir_default())
            .set_file_name(format!("{}.lsys", if stem.trim().is_empty() { "L-system" } else { stem.trim() }))
            .save_file()
        else {
            return;
        };
        self.remember_dir(&path);
        self.lsystem.message = Some(match std::fs::write(&path, s.to_text()) {
            Ok(()) => format!("Saved {}", path.display()),
            Err(e) => format!("Save failed: {e}"),
        });
    }

    /// File > Export SVG…: the view as vectors (design/lsystems.md §7) — what it shows, in its
    /// colours, at the canvas's size.
    pub(crate) fn lsystem_export_svg(&mut self) {
        let svg = self.lsystem_svg_of_view();
        let name = self.lsystem.system.name.clone();
        let stem: String = name.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' }).collect();
        let Some(path) = rfd::FileDialog::new()
            .add_filter("SVG", &["svg"])
            .set_directory(self.dialog_dir_default())
            .set_file_name(format!("{}.svg", if stem.trim().is_empty() { "L-system" } else { stem.trim() }))
            .save_file()
        else {
            return;
        };
        self.remember_dir(&path);
        self.lsystem.message = Some(match std::fs::write(&path, svg) {
            Ok(()) => format!("Exported {}", path.display()),
            Err(e) => format!("Export failed: {e}"),
        });
    }

    /// The view as an SVG document, at the canvas's size.
    pub(crate) fn lsystem_svg_of_view(&self) -> String {
        let size = [self.viewport.width_px.round().max(1.0) as u32, self.viewport.height_px.round().max(1.0) as u32];
        self.lsystem_svg(&self.lsystem_walk_key(size))
    }

    /// The view walked at `key` as an SVG document, coloured as the screen colours it.
    fn lsystem_svg(&self, key: &WalkKey) -> String {
        let st = &self.lsystem;
        let big = st.big.as_ref().filter(|(id, _)| *id == key.tables).map(|(_, b)| b.clone());
        let ex = st.expansion.as_ref().filter(|(id, o, _)| *id == key.tables && *o == key.order).map(|(_, _, x)| x.clone());
        let out = walk_segments(&st.drawn_system(), &st.tables, big, ex, key, st.depth_scale(key.order));
        // The screen's palette lookup (`palette` in mandelbrot.wgsl): value + offset, wrapped; its
        // channels are the bytes the monitor shows.
        let (entries, smooth) = self.active_lut();
        let lut = fractadyne_color::segment::Lut { entries: entries.to_vec(), smooth };
        let offset = self.coloring.offset;
        let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        let colour = |v: f32| {
            let c = lut.sample(v + offset);
            [byte(c[0]), byte(c[1]), byte(c[2])]
        };
        let bg = self.interior_color();
        svg::document(&svg::Picture {
            size: key.size,
            width: st.width,
            segments: &out.segments,
            polygons: &out.outlines,
            colour: &colour,
            background: [byte(bg[0]), byte(bg[1]), byte(bg[2])],
            title: &st.system.name,
        })
    }

    /// The side panel's L-system section.
    pub(crate) fn lsystem_panel(&mut self, ui: &mut egui::Ui) {
        crate::ui::labelled(ui, "System", |ui| {
            let mut pick = None;
            let r = egui::ComboBox::from_id_salt("lsystem_pick")
                .selected_text(self.lsystem.system.name.clone())
                .width(170.0)
                .height(420.0)
                .show_ui(ui, |ui| {
                    for c in library::Category::ALL {
                        ui.label(egui::RichText::new(c.label()).small().weak());
                        for e in library::SYSTEMS.iter().filter(|e| e.category == c) {
                            if ui.selectable_label(self.lsystem.system.name == e.name, e.name).on_hover_text(e.about).clicked() {
                                pick = Some(e.name);
                            }
                        }
                    }
                })
                .response;
            if let Some(name) = pick {
                self.lsystem_open_library(name);
            }
            r
        });
        ui.horizontal(|ui| {
            if ui.button("Open…").on_hover_text("Open a Fractint .l file or a .lsys system.").clicked() {
                self.lsystem_open_file();
            }
            if ui.button("Edit…").on_hover_text("The system's text: axiom, productions, angle — edit and apply.").clicked() {
                self.lsystem.editor_text = self.lsystem.system.to_text();
                self.lsystem.editor_error = None;
                self.lsystem.editor_open = true;
            }
            if ui.button("Save…").on_hover_text("Save the system as a .lsys text file.").clicked() {
                self.lsystem_save_file();
            }
            if ui.button("SVG…").on_hover_text("Export the view as an SVG drawing: its lines and shapes as vectors, in its colours.").clicked() {
                self.lsystem_export_svg();
            }
        });
        ui.separator();
        // Order: follow the zoom, or fixed.
        let now = self.lsystem.order_at(&self.viewport);
        let grows = self.lsystem.tables.grows();
        ui.horizontal(|ui| {
            let mut follow = self.lsystem.fixed_order.is_none();
            let hint = if grows {
                "The order rises as you zoom in, so the detail never runs out."
            } else {
                "This system does not grow by a factor each order, so it is drawn at a fixed order."
            };
            if ui.checkbox(&mut follow, "Order follows zoom").on_hover_text(hint).changed() {
                self.lsystem.fixed_order = if follow { None } else { Some(now) };
            }
        });
        crate::ui::labelled(ui, "Order", |ui| {
            // (A parametric system has no tables; its word's budget bounds its order.)
            let max = if self.lsystem.system.expanded.is_some() { 64 } else { self.lsystem.tables.max_depth };
            match self.lsystem.fixed_order.as_mut() {
                Some(n) => ui.add(egui::Slider::new(n, 0..=max.min(64))),
                None => ui.label(format!("{now}")),
            }
        });
        crate::ui::labelled(ui, "Angle", |ui| {
            let own = self.lsystem.system.angle.degrees();
            let mut a = self.lsystem.angle.unwrap_or(own);
            let r = ui.add(egui::Slider::new(&mut a, 0.0..=180.0).suffix("°").max_decimals(2));
            if r.changed() {
                self.lsystem.set_angle(Some(a));
            }
            let sweeping = self.lsystem.angle_anim.is_some();
            let icon = if sweeping { crate::icons::PAUSE } else { crate::icons::PLAY };
            if ui.button(icon).on_hover_text("Sweep the angle, there and back: the curve morphing as it turns").clicked() {
                self.lsystem.toggle_angle_sweep();
            }
            r
        });
        if self.lsystem.angle.is_some() && ui.small_button("System's angle").clicked() {
            self.lsystem.set_angle(None);
        }
        if self.lsystem.system.is_stochastic() {
            crate::ui::labelled(ui, "Seed", |ui| {
                let mut seed = self.lsystem.system.seed;
                if ui.add(egui::DragValue::new(&mut seed)).on_hover_text("What the random choices follow: the same seed, the same plant.").changed() {
                    self.lsystem.set_seed(seed);
                }
                if ui.button("New").on_hover_text("Another plant: a new seed.").clicked() {
                    self.lsystem.set_seed(fresh_seed());
                }
            });
        }
        crate::ui::labelled(ui, "Line width", |ui| ui.add(egui::Slider::new(&mut self.lsystem.width, 0.5..=8.0).suffix(" px")));
        crate::ui::labelled(ui, "Draw on", |ui| {
            let mut pct = f64::from(self.lsystem.progress) * 100.0;
            if ui.add(egui::Slider::new(&mut pct, 0.0..=100.0).suffix("%").max_decimals(0)).on_hover_text("How much of the curve is drawn, in the order the turtle draws it").changed() {
                self.lsystem.progress = (pct / 100.0) as f32;
                self.lsystem.draw_anim = None;
            }
            let drawing = self.lsystem.draw_anim.is_some();
            let icon = if drawing { crate::icons::PAUSE } else { crate::icons::PLAY };
            if ui.button(icon).on_hover_text("The curve drawing itself on").clicked() {
                self.lsystem.toggle_draw_on();
            }
        });
        crate::ui::labelled(ui, "Colour by", |ui| {
            let label = |c: Colouring| match c {
                Colouring::Position => "position along the curve",
                Colouring::Depth => "branch depth",
                Colouring::Heading => "heading",
                Colouring::Index => "colour index (C, <, >)",
                Colouring::Plain => "one colour",
            };
            let current = self.lsystem.colouring();
            let mut pick = None;
            let r = egui::ComboBox::from_id_salt("lsystem_colour")
                .selected_text(label(current))
                .width(170.0)
                .show_ui(ui, |ui| {
                    for c in Colouring::ALL {
                        if ui.selectable_label(current == c, label(c)).clicked() {
                            pick = Some(c);
                        }
                    }
                })
                .response;
            if let Some(c) = pick {
                self.lsystem.colour = Some(c);
            }
            r
        });
        egui::Grid::new("lsystem_status").num_columns(2).show(ui, |ui| {
            let t = &self.lsystem.tables;
            let built = self.lsystem.system.expanded.is_some();
            ui.label("Growth");
            if built {
                ui.label("built as a word, at its order").on_hover_text(
                    "Parametric or context-sensitive: a module's rewrite depends on its numbers or its neighbours, \
                     so the word is built a generation at a time (up to two million modules) and drawn in double \
                     precision — the order does not follow the zoom.",
                );
            } else {
                ui.label(if t.grows() {
                    format!("×{:.4} an order{}", t.growth, if t.period == 2 { " (orders step by 2)" } else { "" })
                } else {
                    "none (fixed order)".into()
                });
            }
            ui.end_row();
            if let Some(w) = self.lsystem.last_walk() {
                ui.label("Drawn");
                let order = match w.built {
                    Some((got, _)) => got,
                    None => w.order,
                };
                ui.label(format!("{} at order {order}{}", drawn_text(w.segments, w.polygons), if w.stopped { " (budget)" } else { "" }))
                    .on_hover_text(format!("walked in {:.1} ms; subtrees off the view are skipped, those under a pixel drawn as one segment", w.ms));
                ui.end_row();
                if let Some((got, want)) = w.built {
                    ui.label("");
                    ui.label(egui::RichText::new(format!("Order {want}'s word is over two million modules: drawn at {got}.")).small());
                    ui.end_row();
                }
            }
        });
        if now >= lsystem::MAX_ORDER && grows {
            ui.label(egui::RichText::new(format!("At the deepest order drawn ({}): zooming further adds no detail.", lsystem::MAX_ORDER)).small());
        }
        if let Some(m) = &self.lsystem.message {
            ui.label(egui::RichText::new(m).small());
        }
    }

    /// The toolbar's L-system group, in the slot Julia and the dual view take for an escape-time
    /// family: lower / raise the order (fixing it), follow the zoom again, edit.
    pub(crate) fn lsystem_toolbar(&mut self, ui: &mut egui::Ui) {
        let now = self.lsystem.order_at(&self.viewport);
        let max = if self.lsystem.system.expanded.is_some() { lsystem::MAX_ORDER } else { self.lsystem.tables.max_depth.max(now) };
        if ui.button("−").on_hover_text("Lower the order (fixes it)").clicked() {
            self.lsystem.fixed_order = Some(now.saturating_sub(1));
        }
        ui.label(egui::RichText::new(format!("order {now:>3}")).monospace());
        if ui.button("+").on_hover_text("Raise the order (fixes it)").clicked() {
            self.lsystem.fixed_order = Some((now + 1).min(max));
        }
        let follow = self.lsystem.fixed_order.is_none();
        if ui
            .add(egui::SelectableLabel::new(follow, "auto"))
            .on_hover_text("The order follows the zoom")
            .clicked()
        {
            self.lsystem.fixed_order = if follow { Some(now) } else { None };
        }
        if ui.button(crate::icons::EDIT).on_hover_text("Edit the system's text").clicked() {
            self.lsystem.editor_text = self.lsystem.system.to_text();
            self.lsystem.editor_error = None;
            self.lsystem.editor_open = true;
        }
    }

    /// The editor window and a `.l` file's entry picker.
    pub(crate) fn lsystem_windows(&mut self, ctx: &egui::Context) {
        if self.lsystem.editor_open {
            let mut open = true;
            egui::Window::new("L-system").open(&mut open).default_width(460.0).show(ctx, |ui| {
                ui.label(
                    "One key a line: angle (degrees, or /n for a division of the circle), axiom, \
                     productions 'X = word', and optionally heading, draw / move / variables, colour, \
                     order. A symbol with alternatives chosen at random has one line for each, with \
                     its weight: 'X (0.3) = word'; 'seed n' picks the plant. In a word, { } fills the \
                     turtle's path and . adds a vertex. Parametric and context-sensitive productions \
                     (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', \
                     with 'define R 1.456' and 'ignore +-F'; F(l) steps l, +(a) turns a degrees. A Fractint \
                     .l entry can be pasted as it is.",
                );
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.lsystem.editor_text).code_editor().desired_rows(12).desired_width(f32::INFINITY));
                });
                if let Some(e) = &self.lsystem.editor_error {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0x60, 0x60), e);
                }
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        let text = self.lsystem.editor_text.clone();
                        let name = self.lsystem.system.name.clone();
                        match self.lsystem_open_text(&text, &name) {
                            Ok(()) => self.lsystem.editor_error = None,
                            Err(e) => self.lsystem.editor_error = Some(e),
                        }
                    }
                    if ui.button("Revert").on_hover_text("The text of the system shown.").clicked() {
                        self.lsystem.editor_text = self.lsystem.system.to_text();
                        self.lsystem.editor_error = None;
                    }
                });
            });
            if !open {
                self.lsystem.editor_open = false;
            }
        }
        if let Some((file, entries)) = &self.lsystem.entries {
            let mut open = true;
            let mut pick = None;
            egui::Window::new(format!("{file} — entries")).open(&mut open).default_width(320.0).show(ctx, |ui| {
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for (k, e) in entries.iter().enumerate() {
                        match &e.system {
                            Ok(_) => {
                                if ui.selectable_label(false, &e.name).clicked() {
                                    pick = Some(k);
                                }
                            }
                            Err(err) => {
                                ui.label(egui::RichText::new(format!("{} — {err}", e.name)).weak()).on_hover_text("This entry does not read.");
                            }
                        }
                    }
                });
            });
            if let Some(k) = pick {
                let s = entries[k].system.clone().expect("a good entry");
                self.lsystem_open(s);
            }
            if !open {
                self.lsystem.entries = None;
            }
        }
    }

    /// The lines an L-system view carries: the system's text (one escaped line), the order, the
    /// angle override, the line width, the colouring. Also the session's `lsystem`.
    pub(crate) fn lsystem_lines(&self) -> String {
        let l = &self.lsystem;
        let mut s = format!("lsystem={}\n", crate::custom_formula::escape_line(&l.system.to_text()));
        s.push_str(&format!("lsystem_order={}\n", l.fixed_order.map_or_else(|| "auto".to_string(), |n| n.to_string())));
        if let Some(a) = l.angle {
            s.push_str(&format!("lsystem_angle={a}\n"));
        }
        s.push_str(&format!("lsystem_width={}\n", l.width));
        if let Some(c) = l.colour {
            s.push_str(&format!("lsystem_colour={}\n", c.key()));
        }
        s
    }

    /// An L-system view's fields: the system, then the order, angle, width and colouring. With
    /// `show`, switched to (the view's own centre and zoom are applied by the caller afterwards).
    pub(crate) fn apply_lsystem_fields(&mut self, get: impl Fn(&str) -> Option<String>, show: bool) -> Result<(), String> {
        let text = get("lsystem").ok_or("the L-system view carries no system")?;
        if text.len() > MAX_VIEW_SYSTEM {
            return Err(format!("the system is over {} KiB", MAX_VIEW_SYSTEM / 1024));
        }
        let system = LSystem::parse(&crate::custom_formula::unescape_line(&text)).map_err(|e| format!("the L-system does not read: {e}"))?;
        self.lsystem.set_system(system);
        self.lsystem.fixed_order = get("lsystem_order").and_then(|s| s.trim().parse::<u32>().ok()).map(|n| n.min(lsystem::MAX_ORDER));
        let angle = get("lsystem_angle").and_then(|s| s.trim().parse::<f64>().ok()).filter(|a| a.is_finite() && a.abs() <= 360.0);
        self.lsystem.set_angle(angle);
        if let Some(w) = get("lsystem_width").and_then(|s| s.trim().parse::<f32>().ok()).filter(|w| w.is_finite()) {
            self.lsystem.width = w.clamp(0.5, 8.0);
        }
        self.lsystem.colour = get("lsystem_colour").and_then(|s| Colouring::from_key(s.trim()));
        if show {
            if self.fractal != FractalKind::LSystem {
                self.set_fractal(FractalKind::LSystem);
            }
            // Framed, whatever was on screen (an L-system already showing included): a view with
            // its own centre and zoom has them applied after this.
            self.lsystem_home();
        }
        Ok(())
    }

    /// Restore from [`Self::lsystem_lines`] text — the session's `lsystem`.
    pub(crate) fn apply_lsystem_lines(&mut self, text: &str, show: bool) -> Result<(), String> {
        let get = |k: &str| {
            let v = crate::meta_get(text, k);
            (!v.is_empty()).then_some(v)
        };
        self.apply_lsystem_fields(get, show)
    }
}

/// A new seed for a stochastic system: from the clock, kept to six digits so it is easy to note.
fn fresh_seed() -> u64 {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
    (nanos ^ (nanos >> 20)) % 1_000_000
}

/// The panel's count of what a walk drew: "12 segments", "1 filled shape", "8 segments, 3 filled
/// shapes".
fn drawn_text(segments: u64, polygons: u64) -> String {
    let n = |k: u64, one: &str, many: &str| format!("{} {}", crate::life_view::grouped(k), if k == 1 { one } else { many });
    match (segments, polygons) {
        (_, 0) => n(segments, "segment", "segments"),
        (0, _) => n(polygons, "filled shape", "filled shapes"),
        _ => format!("{}, {}", n(segments, "segment", "segments"), n(polygons, "filled shape", "filled shapes")),
    }
}

/// The status bar's L-system readouts after the zoom — order, and what was drawn (segments and
/// filled shapes) — each padded to a fixed width (a growing count never wraps the bar).
pub(crate) fn status_readouts(order: u32, drawn: Option<u64>) -> (String, String) {
    // Up to 999,999,999 grouped (11 characters); the walk's budget keeps it there.
    const COUNT_W: usize = 11;
    let count = drawn.map_or_else(|| "…".to_string(), |n| if n < 1_000_000_000 { crate::life_view::grouped(n) } else { format!("{:.3e}", n as f64) });
    (format!("order {order:>4}"), format!("drawn {count:>COUNT_W$}"))
}

pub(crate) mod export;
mod svg;

#[cfg(test)]
mod tests;
