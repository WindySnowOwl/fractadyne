//! What the panel shows, as state: the events since last frame and this frame's snapshot, turned
//! into lamp glows, the walk on the radar, the tile plane, the scope's columns, the builders on the
//! dial, and the needles (design/blinkenlights.md, part 2).
//!
//! Pure: nothing here reads a clock, a GPU or a global, so every rule is a unit test. The display
//! physics is the design's, the same for every signal: lamps fade with τ = [`LAMP_TAU_S`], needles
//! follow with τ = [`NEEDLE_TAU_S`], an adoption flash with τ = [`FLASH_TAU_S`]. A frozen renderer
//! gives a still panel: every movement starts from an event or a reading.

use super::events::{Abandon, Arith, Event, Kind, PASS_OF_FRAME, PASSES, Pass, RefRole};
use std::collections::VecDeque;

/// An incandescent lamp's fade, seconds.
pub(crate) const LAMP_TAU_S: f32 = 0.15;
/// A needle's lag behind its reading, seconds (critically damped in spirit: it never overshoots).
pub(crate) const NEEDLE_TAU_S: f32 = 0.06;
/// The radar's adoption flash, seconds.
pub(crate) const FLASH_TAU_S: f32 = 0.18;
/// How long a finished walk stays on the radar, fading, seconds.
pub(crate) const WALK_AFTERGLOW_S: f32 = 0.6;
/// How long an install ring or a dropped build's cross shows on the dial, seconds.
pub(crate) const PUCK_AFTERGLOW_S: f32 = 0.6;
/// Frames kept for the scope and the age strip.
pub(crate) const HISTORY_FRAMES: usize = 400;
/// The scope legend's names, in its order, and the kinds of pass each stands for: moving frames
/// (and the probe), pin passes (a split refresh is one), the worker GPU's frames, a still view's
/// settle (tiles and walk passes), supersampling samples.
pub(crate) const LEGEND: [&[Pass]; 5] =
    [&[Pass::Moving, Pass::Probe], &[Pass::Pin, Pass::Split], &[Pass::Worker], &[Pass::Tile, Pass::Walk], &[Pass::Ssaa]];

/// The legend name that stands for `kind`.
pub(crate) fn legend_of(kind: Pass) -> usize {
    LEGEND.iter().position(|ks| ks.contains(&kind)).unwrap_or(0)
}
/// A pass whose timing has not come back this many frames after it ran is drawn as never timed
/// (and, for a pass sized for longer than a few frames, not before twice its target has passed).
pub(crate) const UNTIMED_AFTER_FRAMES: u64 = 8;
/// The GPU time a pinned walk's pass is sized for, ms (`tunables::MOTION_PIN_TARGET_MS`).
pub(crate) const PIN_TARGET_MS: f32 = 20.0;
/// A still view's walk that has had no pass and no timing this long is over: its end was never seen
/// (the panel opened partway, or the walk was cut off), seconds.
pub(crate) const WALK_STALE_S: f32 = 5.0;

/// The light bank's lamps, left to right (design/blinkenlights.md, "The light bank").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Lamp {
    Plan = 0,
    Dispatch,
    Live,
    Held,
    Reproj,
    Reading,
    PinPass,
    Split,
    Adopt,
    Abandon,
    RefLive,
    RefAhead,
    RefHold,
    Series,
    Worker,
    Tile,
    Ssaa,
    Palette,
    Mode,
}

pub(crate) const LAMPS: usize = 19;

/// What the view put on screen this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Present {
    /// Nothing new this frame (the view was not built).
    #[default]
    None,
    /// This frame's own render.
    Live,
    /// The last picture stretched to the moving view.
    Reprojected,
    /// A held picture while a refresh composes underneath.
    Held,
}

/// The per-frame readings the panel needs that are not events: read by the UI thread from the
/// renderer's state once a frame (`blinken::snapshot`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Snapshot {
    /// The UI thread's frame index.
    pub(crate) frame: u64,
    /// Seconds since the last frame.
    pub(crate) dt_s: f32,
    /// The interval between displayed frames, ms.
    pub(crate) frame_ms: f32,
    /// The view's depth: log2 of its magnification.
    pub(crate) depth: f64,
    /// The iteration ask.
    pub(crate) iterations: u32,
    pub(crate) arith: Option<Arith>,
    /// How far the picture on screen lags the view, octaves (0 when live).
    pub(crate) age_oct: f32,
    pub(crate) present: Present,
    /// The view was planned (built) this frame.
    pub(crate) planned: bool,
    /// A pass went to the GPU this frame, and its kind.
    pub(crate) dispatched: Option<Pass>,
    /// The resolution the pass (or the frame) rendered at.
    pub(crate) res: (u32, u32),
    /// The GPU timer ring's slots: 0 idle, else a timing in flight (recorded, mapping, ready).
    pub(crate) timers: [u8; 3],
    /// The walk under way: the share of its texels still iterating after its last counted pass
    /// (`Perf::chunk_running`); a pixel that escaped is done, so later passes iterate only these.
    pub(crate) running: Option<f32>,
}

/// The engine room's readings, sampled by the UI thread (some, such as CPU use, a few times a
/// second rather than every frame).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Engine {
    /// Iteration steps per second the GPU was asked for (pixels × iterations), and executed.
    pub(crate) steps_asked: f64,
    pub(crate) steps_done: f64,
    /// Steps asked for since the app started.
    pub(crate) total_steps: f64,
    /// How busy each GPU is, 0..1: the window's, and the worker's.
    pub(crate) gpu_busy: [f32; 2],
    /// A worker GPU is enabled.
    pub(crate) worker: bool,
    /// The process's working set and its peak, MB; the machine's RAM, MB.
    pub(crate) ram_mb: f32,
    pub(crate) ram_peak_mb: f32,
    pub(crate) ram_cap_mb: f32,
    /// GPU memory the app holds, MB, by kind: G-buffers, hold and SSAA textures, orbit and BLA
    /// buffers, the worker's reference copy.
    pub(crate) vram_mb: [f32; 4],
    /// The card's memory, MB (0 = unknown).
    pub(crate) vram_cap_mb: f32,
    /// CPU use per logical core, 0..1.
    pub(crate) cores: Vec<f32>,
    /// Threads busy by role (`registry::Role`).
    pub(crate) threads: [u32; super::registry::ROLES],
    /// Pixels finished per second.
    pub(crate) px_per_s: f64,
}

/// A pixel falling in the pixel stream: column, row (fractional), speed in rows a second, and
/// whether it finished (bright) or is still walking (dim).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Particle {
    pub(crate) col: u8,
    pub(crate) row: f32,
    pub(crate) speed: f32,
    pub(crate) done: bool,
}

/// The pixel stream's matrix, columns by rows.
pub(crate) const STREAM_COLS: u8 = 40;
pub(crate) const STREAM_ROWS: f32 = 14.0;

/// One pass of a walk on the radar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WalkPass {
    pub(crate) lo: u32,
    pub(crate) hi: u32,
    /// The frame it ran in, to pair it with its timing.
    pub(crate) frame: u64,
    /// GPU ms once its timing came back.
    pub(crate) ms: Option<f32>,
    /// No timing came back in time: drawn hollow.
    pub(crate) untimed: bool,
    /// The GPU time it was sized for, ms: its colour is its time against this.
    pub(crate) target_ms: f32,
    /// Seconds since it ran.
    pub(crate) age_s: f32,
}

/// Which walk the radar shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WalkKind {
    /// A pinned refresh while the view moves (or a split refresh).
    Pin,
    /// A still view walked to its iteration ask.
    Settle,
    /// A supersampling sample's walk over the still view.
    Ssaa,
}

/// The refresh being walked (or the last one, fading).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Walk {
    pub(crate) ask: u32,
    pub(crate) res: (u32, u32),
    pub(crate) cursor: u32,
    pub(crate) passes: Vec<WalkPass>,
    /// Waiting for the last pass's reading.
    pub(crate) verifying: bool,
    /// A split refresh: (passes done, passes in all).
    pub(crate) split: Option<(u32, u32)>,
    /// 1 while live; fades after it ends.
    pub(crate) fade: f32,
    pub(crate) ended: bool,
    pub(crate) kind: WalkKind,
    /// Seconds since its last pass or timing (a still view's walk ends at [`WALK_STALE_S`]).
    pub(crate) idle_s: f32,
}

impl Walk {
    fn new(kind: WalkKind, ask: u32, res: (u32, u32)) -> Walk {
        Walk { ask, res, cursor: 0, passes: Vec::new(), verifying: false, split: None, fade: 1.0, ended: false, kind, idle_s: 0.0 }
    }

    /// A still view's walk, not yet over.
    fn settling(&self) -> bool {
        self.kind != WalkKind::Pin && !self.ended
    }
}

/// What this frame's pass covered, for the tile plane.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Plane {
    /// No pass this frame; the label says why.
    Idle(&'static str),
    /// A whole-frame pass: a moving frame or a walk's band. `band` = (lo, hi) for a walk pass;
    /// `settled`: a still view's walk, not a pinned one.
    Whole { res: (u32, u32), band: Option<(u32, u32)>, settled: bool, glow: f32 },
    /// Pass `j` of a split refresh in `k` passes of interleaved 16-px tiles.
    Split { res: (u32, u32), k: u32, j: u32, glow: f32 },
    /// The settle grid: done tiles lit, the one in progress outlined.
    Grid(Grid),
    /// Supersampling samples over the settled picture: `samples` folded so far, and `band` = (lo,
    /// hi) while the next sample walks.
    Ssaa { res: (u32, u32), samples: u32, band: Option<(u32, u32)>, glow: f32 },
}

/// A settle grid's progress.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Grid {
    pub(crate) res: (u32, u32),
    pub(crate) cols: u32,
    pub(crate) rows: u32,
    pub(crate) side: u32,
    pub(crate) done: Vec<bool>,
    pub(crate) flash: Vec<f32>,
    /// The tile last sent: (index, its rows sent so far, its rows, this piece's rows).
    pub(crate) last: Option<(u32, u32, u32, u32)>,
    pub(crate) complete: bool,
}

/// One frame's column on the scope: GPU ms by pass kind, and which passes never got a timing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Column {
    pub(crate) frame: u64,
    /// ms by `Pass as usize`.
    pub(crate) ms: [f32; PASSES],
    /// A pass of this kind ran in this frame.
    pub(crate) ran: [bool; PASSES],
}

/// A reference build on the dial.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Puck {
    pub(crate) role: RefRole,
    /// The build's id, to pair its launch with its end.
    pub(crate) id: u32,
    pub(crate) depth: f32,
    pub(crate) state: PuckState,
    /// Seconds left of an install ring or a cross.
    pub(crate) left_s: f32,
    /// Seconds since launch: a build whose end the panel never saw expires (`PUCK_LOST_S`).
    pub(crate) age_s: f32,
}

/// A build still building or built-not-installed this long is dropped from the dial: its end was
/// never seen (superseded in a queue, or the panel opened partway through).
pub(crate) const PUCK_LOST_S: f32 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PuckState {
    Building,
    Built,
    Installed,
    Dropped,
}

/// The panel's state.
pub(crate) struct Model {
    pub(crate) lamps: [f32; LAMPS],
    pub(crate) walk: Option<Walk>,
    pub(crate) plane: Plane,
    pub(crate) columns: VecDeque<Column>,
    /// Each scope legend name's glow ([`LEGEND`]): lit when a pass of its kind is first heard of,
    /// fading as a lamp does.
    pub(crate) legend: [f32; LEGEND.len()],
    pub(crate) timers: [f32; 3],
    pub(crate) pucks: Vec<Puck>,
    pub(crate) series_running: u32,
    /// The age needle (smoothed) and its history strip.
    pub(crate) age: f32,
    pub(crate) age_hist: VecDeque<f32>,
    /// The radar's adoption flash.
    pub(crate) flash: f32,
    /// The last snapshot.
    pub(crate) snap: Snapshot,
    /// Adoptions in the last two seconds, by source (0 this GPU, 1 the worker), for the engines.
    pub(crate) shown: VecDeque<(u64, u8)>,
    /// Seconds of display time, for orbits and pulses (advances only with frames).
    pub(crate) clock_s: f32,
    /// The pixel stream.
    pub(crate) particles: Vec<Particle>,
    /// Particles owed to the stream (fractional), and the stream's random state.
    stream_debt: f32,
    rng: u32,
}

impl Default for Model {
    fn default() -> Self {
        Model {
            lamps: [0.0; LAMPS],
            walk: None,
            plane: Plane::Idle("STARTING"),
            columns: VecDeque::with_capacity(HISTORY_FRAMES),
            legend: [0.0; LEGEND.len()],
            timers: [0.0; 3],
            pucks: Vec::new(),
            series_running: 0,
            age: 0.0,
            age_hist: VecDeque::with_capacity(HISTORY_FRAMES),
            flash: 0.0,
            snap: Snapshot::default(),
            shown: VecDeque::new(),
            clock_s: 0.0,
            particles: Vec::new(),
            stream_debt: 0.0,
            rng: 0x2545_f491,
        }
    }
}

fn decay(v: f32, dt: f32, tau: f32) -> f32 {
    v * (-dt / tau).exp()
}

impl Model {
    fn light(&mut self, l: Lamp, v: f32) {
        let i = l as usize;
        self.lamps[i] = self.lamps[i].max(v);
    }

    /// Records that a pass of `kind` ran in `frame` (in its column, while the scope has one), and
    /// lights its legend name the first time it is heard of: when it is dispatched, or for a kind
    /// first known from its reading (the worker's frames), when that arrives. A reading of a pass
    /// already recorded does not light it again.
    fn mark(&mut self, frame: u64, kind: Pass) {
        let Some(c) = self.column(frame) else { return };
        let new = !c.ran[kind as usize];
        c.ran[kind as usize] = true;
        if new {
            self.legend[legend_of(kind)] = 1.0;
        }
    }

    /// The scope's column for `frame`: opened when it is newer than every column, else the one
    /// already there; `None` for a frame the scope never had or has scrolled off (a reading for a
    /// frame before the panel opened), which is dropped.
    fn column(&mut self, frame: u64) -> Option<&mut Column> {
        if let Some(i) = self.columns.iter().rposition(|c| c.frame == frame) {
            return Some(&mut self.columns[i]);
        }
        if self.columns.back().map_or(true, |c| frame > c.frame) {
            self.columns.push_back(Column { frame, ..Column::default() });
            while self.columns.len() > HISTORY_FRAMES {
                self.columns.pop_front();
            }
            return self.columns.back_mut();
        }
        None
    }

    /// Advances the panel by one frame: decays, then applies `events` (oldest first) and `snap`.
    pub(crate) fn update(&mut self, events: &[Event], snap: Snapshot) {
        let dt = snap.dt_s.clamp(0.0, 0.25);
        for v in self.lamps.iter_mut() {
            *v = decay(*v, dt, LAMP_TAU_S);
        }
        for v in self.timers.iter_mut().chain(self.legend.iter_mut()) {
            *v = decay(*v, dt, LAMP_TAU_S);
        }
        self.flash = decay(self.flash, dt, FLASH_TAU_S);
        match &mut self.plane {
            Plane::Whole { glow, .. } | Plane::Split { glow, .. } | Plane::Ssaa { glow, .. } => *glow = decay(*glow, dt, LAMP_TAU_S),
            Plane::Grid(g) => {
                for f in g.flash.iter_mut() {
                    *f = decay(*f, dt, LAMP_TAU_S);
                }
            }
            Plane::Idle(_) => {}
        }
        if let Some(w) = &mut self.walk {
            if w.ended {
                w.fade -= dt / WALK_AFTERGLOW_S;
            }
            w.idle_s += dt;
            for p in w.passes.iter_mut() {
                p.age_s += dt;
            }
            if w.settling() && w.idle_s > WALK_STALE_S {
                w.ended = true;
                w.verifying = false;
            }
        }
        if self.walk.as_ref().is_some_and(|w| w.ended && w.fade <= 0.0) {
            self.walk = None;
        }
        for p in self.pucks.iter_mut() {
            p.age_s += dt;
            if matches!(p.state, PuckState::Installed | PuckState::Dropped) {
                p.left_s -= dt;
            }
        }
        self.pucks.retain(|p| match p.state {
            PuckState::Installed | PuckState::Dropped => p.left_s > 0.0,
            PuckState::Building | PuckState::Built => p.age_s < PUCK_LOST_S,
        });

        // This frame's column (opened whether or not a pass ran) and its own pass.
        let _ = self.column(snap.frame);
        if let Some(k) = snap.dispatched {
            self.mark(snap.frame, k);
        }
        self.apply_snapshot(&snap);
        for e in events {
            self.apply(e, &snap);
        }
        // Builders in progress hold their lamps lit.
        for (role, lamp) in [(RefRole::Live, Lamp::RefLive), (RefRole::Ahead, Lamp::RefAhead), (RefRole::Hold, Lamp::RefHold)] {
            if self.pucks.iter().any(|p| p.role == role && p.state == PuckState::Building) {
                self.light(lamp, 0.9);
            }
        }
        if self.series_running > 0 {
            self.light(Lamp::Series, 0.8);
        }
        // Passes whose timing never came: hollow. A still view's walk whose last pass is one of them
        // is over (nothing else will end it).
        if let Some(w) = &mut self.walk {
            for p in w.passes.iter_mut() {
                if p.ms.is_none() && snap.frame.saturating_sub(p.frame) > UNTIMED_AFTER_FRAMES && p.age_s > 2.0 * p.target_ms / 1000.0 {
                    p.untimed = true;
                }
            }
            if w.settling() && w.verifying && w.passes.last().is_some_and(|p| p.untimed) {
                w.ended = true;
                w.verifying = false;
            }
        }
        let horizon = snap.frame.saturating_sub(120);
        while self.shown.front().is_some_and(|&(f, _)| f < horizon) {
            self.shown.pop_front();
        }
        self.snap = snap;
    }

    fn apply_snapshot(&mut self, s: &Snapshot) {
        let dt = s.dt_s.clamp(0.0, 0.25);
        if s.planned {
            self.light(Lamp::Plan, 0.9);
        }
        if s.dispatched.is_some() {
            self.light(Lamp::Dispatch, 1.0);
        }
        match s.present {
            Present::Live => self.light(Lamp::Live, 0.9),
            Present::Held => self.light(Lamp::Held, 0.85),
            Present::Reprojected => self.light(Lamp::Reproj, 0.75),
            Present::None => {}
        }
        // A timer slot with a timing in flight holds its lamp lit; idle, it fades.
        for (lamp, &st) in self.timers.iter_mut().zip(s.timers.iter()) {
            if st != 0 {
                *lamp = 1.0;
            }
        }
        // The age needle follows its reading; the strip keeps the raw readings.
        let k = 1.0 - (-dt / NEEDLE_TAU_S).exp();
        self.age += (s.age_oct - self.age) * if dt > 0.0 { k } else { 1.0 };
        self.age_hist.push_back(s.age_oct);
        while self.age_hist.len() > HISTORY_FRAMES {
            self.age_hist.pop_front();
        }
        // A moving frame or a settle grid ends a still view's walk: the view moved, or settles
        // another way.
        if matches!(s.dispatched, Some(Pass::Moving | Pass::Tile)) {
            if let Some(w) = self.walk.as_mut().filter(|w| w.settling()) {
                w.ended = true;
                w.verifying = false;
            }
        }
        // A moving frame's pass covers the whole picture; a walk's pass is set by its own event,
        // and a still view's walk keeps its pass on the plane between passes (one runs for many
        // frames).
        let settling = self.walk.as_ref().is_some_and(Walk::settling);
        if s.dispatched == Some(Pass::Moving) {
            self.plane = Plane::Whole { res: s.res, band: None, settled: false, glow: 1.0 };
        } else if s.dispatched == Some(Pass::Walk) && !settling {
            // A still view rendered whole (a walk's pass names its band in its own event, after).
            self.plane = Plane::Whole { res: s.res, band: None, settled: true, glow: 1.0 };
        } else if s.dispatched.is_none() && !settling && !matches!(self.plane, Plane::Grid(_) | Plane::Ssaa { .. }) {
            self.plane = Plane::Idle(match s.present {
                Present::Held => "HELD",
                Present::Reprojected => "REPROJECTED",
                Present::Live => "LIVE",
                Present::None => "IDLE",
            });
        }
    }

    fn apply(&mut self, e: &Event, s: &Snapshot) {
        match e.kind {
            Kind::PinStart => {
                let res = (e.b >> 16, e.b & 0xffff);
                self.walk = Some(Walk::new(WalkKind::Pin, e.a, res));
            }
            Kind::PinPass => {
                self.light(Lamp::PinPass, 1.0);
                let w = self.walk.get_or_insert_with(|| Walk::new(WalkKind::Pin, e.b, s.res));
                w.passes.push(WalkPass { lo: e.a, hi: e.b, frame: e.frame, ms: None, untimed: false, target_ms: PIN_TARGET_MS, age_s: 0.0 });
                w.cursor = w.cursor.max(e.b);
                w.ask = w.ask.max(e.b);
                w.idle_s = 0.0;
                let res = w.res;
                self.plane = Plane::Whole { res, band: Some((e.a, e.b)), settled: false, glow: 1.0 };
                self.mark(e.frame, Pass::Pin);
            }
            Kind::SplitPass => {
                self.light(Lamp::Split, 1.0);
                let w = self.walk.get_or_insert_with(|| Walk::new(WalkKind::Pin, s.iterations, s.res));
                w.split = Some((e.a + 1, e.b.max(1)));
                w.cursor = w.ask;
                w.idle_s = 0.0;
                w.passes.push(WalkPass { lo: 0, hi: w.ask, frame: e.frame, ms: None, untimed: false, target_ms: PIN_TARGET_MS, age_s: 0.0 });
                let res = w.res;
                self.plane = Plane::Split { res, k: e.b.max(1), j: e.a, glow: 1.0 };
                self.mark(e.frame, Pass::Split);
            }
            Kind::WalkStart => {
                let res = (e.b >> 16, e.b & 0xffff);
                self.walk = Some(Walk::new(if e.f > 0.0 { WalkKind::Ssaa } else { WalkKind::Settle }, e.a, res));
            }
            Kind::WalkPass => {
                // A pass with no start seen (the panel opened partway): a walk of the kind this frame ran.
                if !self.walk.as_ref().is_some_and(Walk::settling) {
                    let kind = if s.dispatched == Some(Pass::Ssaa) { WalkKind::Ssaa } else { WalkKind::Settle };
                    self.walk = Some(Walk::new(kind, s.iterations.max(e.b), s.res));
                }
                let Some(w) = self.walk.as_mut() else { return };
                w.passes.push(WalkPass { lo: e.a, hi: e.b, frame: e.frame, ms: None, untimed: false, target_ms: e.f.max(1.0), age_s: 0.0 });
                w.cursor = w.cursor.max(e.b);
                w.ask = w.ask.max(e.b);
                w.idle_s = 0.0;
                // The last pass is out: the walk waits for its timing.
                w.verifying = e.b >= w.ask;
                let (res, band, kind) = (w.res, Some((e.a, e.b)), w.kind);
                self.plane = match (kind, &self.plane) {
                    (WalkKind::Ssaa, Plane::Ssaa { samples, .. }) => Plane::Ssaa { res, samples: *samples, band, glow: 1.0 },
                    (WalkKind::Ssaa, _) => Plane::Ssaa { res, samples: 0, band, glow: 1.0 },
                    _ => Plane::Whole { res, band, settled: true, glow: 1.0 },
                };
                let pass = if kind == WalkKind::Ssaa { Pass::Ssaa } else { Pass::Walk };
                self.mark(e.frame, pass);
            }
            Kind::PinVerify => {
                if let Some(w) = &mut self.walk {
                    w.verifying = true;
                }
            }
            Kind::PinAdopt => {
                self.light(Lamp::Adopt, 1.0);
                self.flash = 1.0;
                self.shown.push_back((e.frame, 0));
                if let Some(w) = &mut self.walk {
                    w.ended = true;
                    w.verifying = false;
                }
            }
            Kind::PinAbandon => {
                self.light(Lamp::Abandon, if e.a == Abandon::Superseded as u32 { 0.6 } else { 1.0 });
                if let Some(w) = &mut self.walk {
                    w.ended = true;
                    w.verifying = false;
                }
            }
            Kind::PassTimed => {
                self.light(Lamp::Reading, 0.9);
                let frame = self.full_frame(e.b, e.frame);
                // The frame's own pass: whichever kind that frame ran.
                let kind = if e.a == PASS_OF_FRAME {
                    let ran = self.columns.iter().rev().find(|c| c.frame == frame).map(|c| c.ran).unwrap_or_default();
                    [Pass::Pin, Pass::Split, Pass::Walk, Pass::Tile, Pass::Ssaa, Pass::Probe, Pass::Moving]
                        .into_iter()
                        .find(|k| ran[*k as usize])
                        .unwrap_or(Pass::Moving)
                } else {
                    Pass::from_u32(e.a)
                };
                if let Some(c) = self.column(frame) {
                    c.ms[kind as usize] += e.f.max(0.0);
                }
                self.mark(frame, kind);
                if matches!(kind, Pass::Pin | Pass::Split | Pass::Walk | Pass::Ssaa) {
                    if let Some(w) = &mut self.walk {
                        if let Some(p) = w.passes.iter_mut().rev().find(|p| p.frame == frame) {
                            p.ms = Some(e.f);
                            p.untimed = false;
                            w.idle_s = 0.0;
                        }
                        // A still view's walk is over when its last pass is timed.
                        if w.settling() && w.verifying && w.passes.last().is_some_and(|p| p.ms.is_some()) {
                            w.ended = true;
                            w.verifying = false;
                        }
                    }
                }
            }
            Kind::GridStart => {
                let res = (e.a >> 16, e.a & 0xffff);
                let side = e.b.max(1);
                let (cols, rows) = (res.0.div_ceil(side).max(1), res.1.div_ceil(side).max(1));
                let n = (cols * rows) as usize;
                self.plane = Plane::Grid(Grid { res, cols, rows, side, done: vec![false; n], flash: vec![0.0; n], last: None, complete: false });
            }
            Kind::TileSent => {
                self.light(Lamp::Tile, 1.0);
                if let Plane::Grid(g) = &mut self.plane {
                    let i = e.a as usize;
                    let (sent, rows) = ((e.b >> 16), (e.b & 0xffff).max(1));
                    if i < g.done.len() {
                        g.flash[i] = 1.0;
                        if sent >= rows {
                            g.done[i] = true;
                        }
                        g.last = Some((e.a, sent.min(rows), rows, (e.f.max(0.0) as u32).clamp(1, rows)));
                    }
                }
                if let Some(c) = self.column(e.frame) {
                    c.ran[Pass::Tile as usize] = true;
                }
            }
            Kind::GridDone => {
                if let Plane::Grid(g) = &mut self.plane {
                    g.complete = true;
                    g.done.iter_mut().for_each(|d| *d = true);
                    g.last = None;
                }
            }
            Kind::SsaaSample => {
                self.light(Lamp::Ssaa, 1.0);
                let res = match &self.plane {
                    Plane::Grid(g) => g.res,
                    Plane::Ssaa { res, .. } => *res,
                    _ => s.res,
                };
                self.plane = Plane::Ssaa { res, samples: e.a, band: None, glow: 1.0 };
                if let Some(c) = self.column(e.frame) {
                    c.ran[Pass::Ssaa as usize] = true;
                }
            }
            Kind::RefLaunch => {
                self.pucks.push(Puck { role: RefRole::from_u32(e.a), id: e.b, depth: e.f, state: PuckState::Building, left_s: 0.0, age_s: 0.0 });
            }
            Kind::RefBuilt => {
                if let Some(p) = self.puck(e) {
                    p.state = PuckState::Built;
                    // A build thread does not know the depth; the launch's stands.
                    if e.f > 0.0 {
                        p.depth = e.f;
                    }
                }
            }
            Kind::RefInstalled => {
                let role = RefRole::from_u32(e.a);
                match self.puck(e) {
                    Some(p) => {
                        p.state = PuckState::Installed;
                        p.left_s = PUCK_AFTERGLOW_S;
                    }
                    // Installed with no launch seen (built before the panel opened): ring it anyway.
                    None => self.pucks.push(Puck { role, id: e.b, depth: e.f, state: PuckState::Installed, left_s: PUCK_AFTERGLOW_S, age_s: 0.0 }),
                }
            }
            Kind::RefDropped => {
                self.light(Lamp::Abandon, 0.5);
                if let Some(p) = self.puck(e) {
                    p.state = PuckState::Dropped;
                    p.left_s = PUCK_AFTERGLOW_S;
                }
            }
            Kind::Series => {
                if e.f <= 0.0 {
                    self.series_running += 1;
                } else {
                    self.series_running = self.series_running.saturating_sub(1);
                }
                self.light(Lamp::Series, 0.8);
            }
            Kind::WorkerFrame => {
                self.light(Lamp::Worker, 1.0);
                self.shown.push_back((e.frame, 1));
                if let Some(c) = self.column(e.frame) {
                    c.ran[Pass::Worker as usize] = true;
                    c.ms[Pass::Worker as usize] += e.f.max(0.0);
                }
            }
            Kind::PaletteFeed => self.light(Lamp::Palette, 0.9),
            Kind::ModeSwitch => self.light(Lamp::Mode, 1.0),
        }
    }

    /// The build an event is about: same role and id.
    fn puck(&mut self, e: &Event) -> Option<&mut Puck> {
        let role = RefRole::from_u32(e.a);
        self.pucks.iter_mut().rev().find(|p| p.role == role && p.id == e.b)
    }

    /// A frame index from its low 32 bits, the one nearest `near` (a timing names the frame its
    /// pass ran in, a few frames back).
    fn full_frame(&self, low: u32, near: u64) -> u64 {
        let base = near & !0xffff_ffff;
        let cand = base | low as u64;
        if cand > near { cand.saturating_sub(1 << 32) } else { cand }
    }

    /// Advances the pixel stream by `dt` at `px_per_s`: a particle for every share of the rate,
    /// a third of them still walking (dim), all falling through the matrix.
    pub(crate) fn stream(&mut self, dt: f32, px_per_s: f64) {
        let dt = dt.clamp(0.0, 0.25);
        self.clock_s += dt;
        for p in self.particles.iter_mut() {
            p.row += p.speed * dt;
        }
        self.particles.retain(|p| p.row < STREAM_ROWS);
        // About 12 particles a frame at 100 Mpx/s, at least a trickle while anything renders.
        let per_s = if px_per_s > 0.0 { (px_per_s / 1.0e6 * 6.0).clamp(10.0, 300.0) } else { 0.0 } as f32;
        self.stream_debt += per_s * dt;
        while self.stream_debt >= 1.0 && self.particles.len() < 600 {
            self.stream_debt -= 1.0;
            let r = self.next_rand();
            self.particles.push(Particle {
                col: (r % STREAM_COLS as u32) as u8,
                row: -(((r >> 8) % 3) as f32),
                speed: 6.0 + ((r >> 12) % 9) as f32,
                done: (r >> 20) % 3 != 0,
            });
        }
        if self.particles.len() >= 600 {
            self.stream_debt = 0.0;
        }
    }

    fn next_rand(&mut self) -> u32 {
        // xorshift32: a fixed, cheap sequence (the stream's look, not a measurement).
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    /// Something on the panel is still moving: a lamp, legend name or flash decaying, a walk or builder on its
    /// instrument, a glow on the tile plane, pixels in the stream. When nothing is, the panel can
    /// stop asking for frames and an idle app idles.
    pub(crate) fn active(&self) -> bool {
        let lit = |v: f32| v > 0.02;
        self.lamps.iter().copied().any(lit)
            || self.timers.iter().copied().any(lit)
            || self.legend.iter().copied().any(lit)
            || lit(self.flash)
            || self.walk.is_some()
            || !self.pucks.is_empty()
            || !self.particles.is_empty()
            || match &self.plane {
                Plane::Whole { glow, .. } | Plane::Split { glow, .. } | Plane::Ssaa { glow, .. } => lit(*glow),
                Plane::Grid(g) => g.flash.iter().copied().any(lit),
                Plane::Idle(_) => false,
            }
    }

    /// Lamp brightness, 0..1.
    pub(crate) fn lamp(&self, l: Lamp) -> f32 {
        self.lamps[l as usize]
    }

    /// The share of adopted frames each source supplied in the last two seconds: (this GPU, worker).
    pub(crate) fn shown_share(&self) -> (f32, f32) {
        let n1 = self.shown.iter().filter(|s| s.1 == 1).count() as f32;
        let n = self.shown.len() as f32;
        if n == 0.0 { (0.0, 0.0) } else { ((n - n1) / n, n1 / n) }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/blinken/model/tests.rs"]
mod tests;
