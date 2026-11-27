//! The coloring methods, the orbit statistics and the orbit-trap shapes: one table each, read by
//! the app, the GPU crate and the shader (design/coloring-plan.md A10, B2).
//!
//! Outside the GPU a method or a trap is known by its **key**: session files, view files, the
//! farm's settings, the command line and every harness output store the key. Its **GPU number**
//! is its position in the table, and holds for one build only: the shader takes the numbers as
//! named constants generated from this table ([`wgsl_ids`]), and every Rust check asks the table
//! (`reads`, `bla_aggregate`, ...), so no number is written down twice and the table can be
//! reordered or extended without breaking a saved file.
//!
//! **What a frame accumulates is not what it shows** (B2). The iteration accumulates a set of
//! orbit statistics ([`StatSet`]) into the aux lane, and a method reads at most one of them
//! ([`ColorMethod::reads`]). Every rule about the iteration (series approximation, BLA, the
//! chunked walk, the corrector, the normalized export, the reference's aggregates) asks the set;
//! every rule about the picture (the color pass, strips, the fit range) asks the method.

use std::fmt::Write as _;

/// One coloring method's row of the table.
#[derive(Debug)]
pub struct MethodSpec {
    /// The persisted key (session, view files, farm, `--method`, harness outputs).
    pub key: &'static str,
    /// The name of the shader constant that carries its GPU number (`METHOD_STRIPE`).
    pub wgsl: &'static str,
    /// The orbit statistic it colors by, read from the aux lane: stripe, triangle inequality,
    /// orbit trap, decomposition (the final angle). `None` for the methods colored from the main
    /// lane (the count, the distance estimate).
    pub reads: Option<Stat>,
    /// It reads the distance estimate, which only a family with a derivative has
    /// (`FormulaCaps::distance_estimate`); elsewhere it colors the whole view one color.
    pub needs_distance_estimate: bool,
    /// Its colored value lies in 0..1 (stripe, triangle inequality, decomposition), so the palette
    /// takes a few cycles across that range instead of the per-unit rate of the unbounded values
    /// (iteration count, orbit trap, distance).
    pub bounded_value: bool,
    /// The heading it is listed under in the method box and the Color menu (coloring plan B6).
    pub group: MethodGroup,
}

/// The headings the method box and the Color menu list the methods under (coloring plan B6,
/// design §9.1), by what a method colors by. The order is the list's.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum MethodGroup {
    /// How fast a point escapes: the iteration count.
    EscapeCount,
    /// The distance estimate to the set.
    Distance,
    /// A statistic accumulated along the orbit (stripe, triangle inequality, the traps).
    OrbitStatistics,
    /// The orbit's final angle.
    Angle,
}

impl MethodGroup {
    /// Every heading, in the list's order.
    pub const ALL: [MethodGroup; 4] =
        [MethodGroup::EscapeCount, MethodGroup::Distance, MethodGroup::OrbitStatistics, MethodGroup::Angle];
    /// Its methods, in the table's order.
    pub fn methods(self) -> impl Iterator<Item = ColorMethod> {
        ColorMethod::ALL.into_iter().filter(move |m| m.group() == self)
    }
}

/// A coloring method. The variants' order is the table's order and gives the GPU numbers; the
/// order carries no meaning outside one build.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum ColorMethod {
    #[default]
    Smooth,
    Stripe,
    TriangleIneq,
    OrbitTrap,
    Distance,
    /// The fringe (coloring plan C9): the palette at the slope's direction, at the distance's
    /// lightness.
    Fringe,
    Decomposition,
}

/// The table, in [`ColorMethod`]'s order.
const METHODS: [MethodSpec; 7] = [
    MethodSpec {
        key: "smooth",
        wgsl: "METHOD_SMOOTH",
        reads: None,
        needs_distance_estimate: false,
        bounded_value: false,
        group: MethodGroup::EscapeCount,
    },
    MethodSpec {
        key: "stripe",
        wgsl: "METHOD_STRIPE",
        reads: Some(Stat::Stripe),
        needs_distance_estimate: false,
        bounded_value: true,
        group: MethodGroup::OrbitStatistics,
    },
    MethodSpec {
        key: "triangle",
        wgsl: "METHOD_TRIANGLE",
        reads: Some(Stat::Triangle),
        needs_distance_estimate: false,
        bounded_value: true,
        group: MethodGroup::OrbitStatistics,
    },
    MethodSpec {
        key: "trap",
        wgsl: "METHOD_TRAP",
        reads: Some(Stat::Trap),
        needs_distance_estimate: false,
        bounded_value: false,
        group: MethodGroup::OrbitStatistics,
    },
    MethodSpec {
        key: "distance",
        wgsl: "METHOD_DISTANCE",
        reads: None,
        needs_distance_estimate: true,
        bounded_value: false,
        group: MethodGroup::Distance,
    },
    MethodSpec {
        key: "fringe",
        wgsl: "METHOD_FRINGE",
        // The main lane: the slope normal and the distance estimate, as relief lighting reads them.
        reads: None,
        needs_distance_estimate: true,
        // The slope's direction in turns, 0..1.
        bounded_value: true,
        group: MethodGroup::Distance,
    },
    MethodSpec {
        key: "decomposition",
        wgsl: "METHOD_DECOMPOSITION",
        reads: Some(Stat::Angle),
        needs_distance_estimate: false,
        bounded_value: true,
        group: MethodGroup::Angle,
    },
];

impl ColorMethod {
    /// Every method, in the table's order.
    pub const ALL: [ColorMethod; 7] = [
        ColorMethod::Smooth,
        ColorMethod::Stripe,
        ColorMethod::TriangleIneq,
        ColorMethod::OrbitTrap,
        ColorMethod::Distance,
        ColorMethod::Fringe,
        ColorMethod::Decomposition,
    ];
    /// This method's row of the table.
    pub const fn spec(self) -> &'static MethodSpec {
        &METHODS[self as usize]
    }
    /// Its GPU number in this build (`IterUniforms`/`ColorUniforms::color_method`). Never store it.
    pub const fn gpu_id(self) -> u32 {
        self as u32
    }
    /// The method a GPU number stands for in this build.
    pub fn from_gpu_id(id: u32) -> Option<ColorMethod> {
        ColorMethod::ALL.get(id as usize).copied()
    }
    /// The persisted key.
    pub const fn key(self) -> &'static str {
        self.spec().key
    }
    /// The method a key names, if any.
    pub fn parse_key(s: &str) -> Option<ColorMethod> {
        ColorMethod::ALL.into_iter().find(|m| m.key() == s)
    }
    /// The method a key names, or Smooth for one it does not know (an older or newer file's).
    /// Where an unknown key must be an error, use [`Self::parse_key`].
    pub fn from_key(s: &str) -> ColorMethod {
        ColorMethod::parse_key(s).unwrap_or_default()
    }
    /// See [`MethodSpec::reads`].
    pub const fn reads(self) -> Option<Stat> {
        self.spec().reads
    }
    /// It colors by an orbit statistic, read from the aux lane: a question about the PICTURE (what
    /// the color pass, strips and a window must keep). What the iteration carries is [`StatSet`]'s.
    pub const fn reads_aux(self) -> bool {
        self.spec().reads.is_some()
    }
    /// What a frame colored with this method accumulates unless asked for more: the statistic it
    /// reads, or nothing.
    pub const fn orbit_stats(self) -> StatSet {
        match self.spec().reads {
            Some(s) => StatSet::of(s),
            None => StatSet::NONE,
        }
    }
    /// What a frame colored with this method accumulates (coloring plan B3): what it reads, and in
    /// floatexp, when that is a statistic of the G-buffer, the whole G-buffer, so a switch among
    /// stripe, triangle and the traps recolors instead of iterating again: there an iteration can
    /// take minutes. The price, iterate time per frame (coloring matrix, median over 25 views,
    /// 2026-11-11): +2-4% on the RTX 3080 and 3070, +2-7% on the RX 6800 XT; the user chose to pay
    /// it in every deep frame (the app's `GBUFFER_DEEP` switch turns it off). In df32 and direct,
    /// where an iteration is cheap anyway, it would cost the NVIDIA cards 10-35% per frame, so there
    /// a frame carries only what its method reads.
    /// Decomposition is left as it is: its final angle alone iterates as Smooth does, with the
    /// series, BLA and the walk, which the G-buffer would take away ([`StatSet::GBUFFER`]).
    pub const fn orbit_stats_at(self, floatexp: bool) -> StatSet {
        let own = self.orbit_stats();
        match self.spec().reads {
            Some(s) if floatexp && StatSet::GBUFFER.has(s) => own.union(StatSet::GBUFFER),
            _ => own,
        }
    }
    /// See [`MethodSpec::needs_distance_estimate`].
    pub const fn needs_distance_estimate(self) -> bool {
        self.spec().needs_distance_estimate
    }
    /// See [`MethodSpec::bounded_value`].
    pub const fn bounded_value(self) -> bool {
        self.spec().bounded_value
    }
    /// See [`MethodSpec::group`].
    pub const fn group(self) -> MethodGroup {
        self.spec().group
    }
    /// Whether a frame colored with this method can be rendered through `c` ([`Capability`]): read
    /// off the rules the renderer follows, the statistics the method accumulates for the
    /// iteration's and [`Self::reads_aux`] for the picture's.
    pub const fn can(self, c: Capability) -> bool {
        let stats = self.orbit_stats();
        match c {
            Capability::GlitchCorrection | Capability::NormalizedExport => stats.is_empty(),
            Capability::LiveWalk => !stats.accumulates(),
            Capability::Strips => !self.reads_aux(),
            Capability::Series => !stats.blocks_iter_skip(),
            Capability::Bla => stats.rides_bla(),
        }
    }

    /// What a family with these capabilities lacks for this method, if anything: Distance needs the
    /// distance estimate, the methods that read the aux lane need the orbit's statistics.
    pub const fn lacks(self, caps: crate::formula::FormulaCaps) -> Option<Lacks> {
        if self.needs_distance_estimate() && !caps.distance_estimate {
            Some(Lacks::DistanceEstimate)
        } else if self.reads_aux() && !caps.orbit_statistics {
            Some(Lacks::OrbitStatistics)
        } else {
            None
        }
    }
    /// Whether a family with these capabilities offers this method: it lacks nothing the method
    /// needs. The one rule the method box, the Color menu and the coloring matrix
    /// (`--coloring-matrix`) share.
    pub const fn offered_for(self, caps: crate::formula::FormulaCaps) -> bool {
        self.lacks(caps).is_none()
    }
}

/// What a family can lack that a coloring method needs ([`ColorMethod::lacks`]): each has its own
/// reason in the method box and the Color menu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lacks {
    /// The derivative the distance estimate comes from (`FormulaCaps::distance_estimate`).
    DistanceEstimate,
    /// The orbit's statistics (`FormulaCaps::orbit_statistics`): Newton's pixels converge to a root.
    OrbitStatistics,
}

/// A way of rendering that not every method can be drawn through (coloring plan B4): the capability
/// table, which the menus and dialogs read to gray a method out or say why ([`ColorMethod::can`]).
/// What a FAMILY lacks for a method is [`ColorMethod::lacks`]'s.
///
/// ⭐Each answer is derived from the rules the renderer follows, not written down beside them: the
/// iteration's from the method's statistics ([`StatSet`]), the picture's from [`ColorMethod::reads_aux`].
/// So the table cannot say a method can do something the renderer then refuses. (The deep G-buffer,
/// [`ColorMethod::orbit_stats_at`], changes none of these: it joins only a frame that already
/// accumulates, and changes none of its rules.)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Capability {
    /// A glitch-corrected export (a second reference for the pixels the first one failed): the
    /// corrector carries no orbit statistics.
    GlitchCorrection,
    /// The normalized export (`--normalize`, a tour's `normalize`): its range and color passes read
    /// the main lane only.
    NormalizedExport,
    /// A picture assembled from strips (a zoom video, `--assemble`): strips keep the main lane only.
    Strips,
    /// The live view's walk in passes (the chunked walk, band BLA): its state carries no
    /// statistics, so it serves a frame that accumulates none; its resolve writes the final angle.
    LiveWalk,
    /// Series approximation: skipping iterations would drop the running statistics' terms.
    Series,
    /// BLA: its nodes fold every running statistic, and the final angle needs no fold.
    Bla,
}

impl Capability {
    /// Every capability, in the table's order.
    pub const ALL: [Capability; 6] = [
        Capability::GlitchCorrection,
        Capability::NormalizedExport,
        Capability::Strips,
        Capability::LiveWalk,
        Capability::Series,
        Capability::Bla,
    ];

    /// A name for logs, tests and harness output.
    pub const fn key(self) -> &'static str {
        match self {
            Capability::GlitchCorrection => "glitch-correction",
            Capability::NormalizedExport => "normalized-export",
            Capability::Strips => "strips",
            Capability::LiveWalk => "live-walk",
            Capability::Series => "series",
            Capability::Bla => "bla",
        }
    }
}

/// One orbit statistic's row of the table: something a frame can carry in the aux lane, either
/// accumulated as the orbit runs (`aux_step` in the shader) or read off the final z at escape.
#[derive(Debug)]
pub struct StatSpec {
    /// A name for logs and tests (never stored: a frame's set is derived, not saved).
    pub key: &'static str,
    /// The name of the shader constant that carries its bit (`STAT_STRIPE`).
    pub wgsl: &'static str,
    /// Skipping iterations would change it: a running statistic (stripe, triangle inequality, trap
    /// minimum) would drop the skipped terms. Series approximation stays off while it is
    /// accumulated. Not the final angle (coloring plan B8, measured 2026-11-23): the series moves the
    /// final z within its tolerance, and in the coloring matrix the angle with the series on changed
    /// fewer pixels than a nudge of a thousandth of a pixel does (14% against 40% at the seahorse at
    /// 1e6x: decomposition's cell edges are that dense), with none unexplained, in every df32 view
    /// of Mandelbrot and Multibrot 3-8. (The ~15% the series once "moved" was those same pixels.)
    pub blocks_iter_skip: bool,
    /// Each BLA node carries its aggregate (the node's fourth vec4, `[span, trap min, sum TIA,
    /// sum stripe]`), so BLA may skip with it accumulated past 1e28x. The final angle has none and
    /// needs none ([`Self::at_escape`]).
    pub bla_aggregate: bool,
    /// Read off the final z where the pixel escapes, never accumulated: the frame iterates as one
    /// without statistics (no per-iteration work, the series and BLA as for Smooth, the walk in
    /// passes, whose resolve writes it from the state's z), and pays one `atan2` at escape. The
    /// final angle. BLA keeps it as the series does (B8, 2026-11-23: with BLA, and BLA and the
    /// series together, the angle changed fewer pixels than the nudge in every floatexp view of the
    /// matrix, Mandelbrot at 1e30x, 9.3e78x and 7.4e143x, Multibrot 3-8 at 1e30x and the sixteen
    /// folds at 2e28x, none unexplained).
    pub at_escape: bool,
}

/// An orbit statistic. The variants' order is the table's order and gives the bits; the order
/// carries no meaning outside one build.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Stat {
    /// The stripe average (aux.x).
    Stripe,
    /// The triangle-inequality average (aux.y).
    Triangle,
    /// The orbit trap's nearest approach, for the session's trap shape (aux.z).
    Trap,
    /// The final angle (aux.w), decomposition's.
    Angle,
}

const STATS: [StatSpec; 4] = [
    StatSpec { key: "stripe", wgsl: "STAT_STRIPE", blocks_iter_skip: true, bla_aggregate: true, at_escape: false },
    StatSpec { key: "triangle", wgsl: "STAT_TRIANGLE", blocks_iter_skip: true, bla_aggregate: true, at_escape: false },
    StatSpec { key: "trap", wgsl: "STAT_TRAP", blocks_iter_skip: true, bla_aggregate: true, at_escape: false },
    StatSpec { key: "angle", wgsl: "STAT_ANGLE", blocks_iter_skip: false, bla_aggregate: false, at_escape: true },
];

impl Stat {
    /// Every statistic, in the table's order.
    pub const ALL: [Stat; 4] = [Stat::Stripe, Stat::Triangle, Stat::Trap, Stat::Angle];
    /// This statistic's row of the table.
    pub const fn spec(self) -> &'static StatSpec {
        &STATS[self as usize]
    }
    /// Its bit in the GPU word ([`StatSet::bits`]) in this build. Never store it.
    pub const fn bit(self) -> u32 {
        1 << self as u32
    }
}

/// A set of orbit statistics: what a frame's iteration accumulates into the aux lane (the
/// accumulate mask, coloring plan B2). See the module's note: the iteration's rules ask this, the
/// picture's ask [`ColorMethod::reads`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct StatSet(u32);

impl StatSet {
    /// Nothing: the iteration writes `AUX_NONE`.
    pub const NONE: StatSet = StatSet(0);
    /// The full G-buffer (coloring plan B3): every statistic BLA folds (stripe, triangle
    /// inequality, trap). Every one of them also blocks the series, so a frame that accumulates any
    /// of them runs under the same rules with these added: it pays only their operations, and a
    /// switch between the methods that read them can recolor. (Every such frame holds the final
    /// angle too: `aux_pack` writes it.) The final angle is not in it although BLA keeps it (B8): a
    /// Decomposition frame carrying the G-buffer would pay these three statistics' work on every
    /// iteration and lose the walk in passes, which the angle alone keeps.
    pub const GBUFFER: StatSet = StatSet::folded_by_bla();
    const fn folded_by_bla() -> StatSet {
        let mut set = StatSet::NONE;
        let mut i = 0;
        while i < Stat::ALL.len() {
            if Stat::ALL[i].spec().bla_aggregate {
                set = set.with(Stat::ALL[i]);
            }
            i += 1;
        }
        set
    }
    /// One statistic.
    pub const fn of(s: Stat) -> StatSet {
        StatSet(s.bit())
    }
    /// This set and `s`.
    pub const fn with(self, s: Stat) -> StatSet {
        StatSet(self.0 | s.bit())
    }
    /// Both sets.
    pub const fn union(self, other: StatSet) -> StatSet {
        StatSet(self.0 | other.0)
    }
    /// Whether `s` is accumulated.
    pub const fn has(self, s: Stat) -> bool {
        self.0 & s.bit() != 0
    }
    /// Whether nothing is: the frame needs no aux lane, and nothing about the iteration is
    /// restricted for coloring's sake.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// The GPU word (`IterUniforms::orbit_stats`): the statistics' bits for this build. Never
    /// store it.
    pub const fn bits(self) -> u32 {
        self.0
    }
    /// Skipping iterations would change a statistic in the set ([`StatSpec::blocks_iter_skip`]).
    pub const fn blocks_iter_skip(self) -> bool {
        let mut i = 0;
        while i < Stat::ALL.len() {
            if self.has(Stat::ALL[i]) && Stat::ALL[i].spec().blocks_iter_skip {
                return true;
            }
            i += 1;
        }
        false
    }
    /// BLA may skip with the set accumulated: it folds each statistic in it with the node's
    /// aggregate ([`StatSpec::bla_aggregate`]), or the statistic is read off the final z
    /// ([`StatSpec::at_escape`]). True for none.
    pub const fn rides_bla(self) -> bool {
        let mut i = 0;
        while i < Stat::ALL.len() {
            let s = Stat::ALL[i].spec();
            if self.has(Stat::ALL[i]) && !s.bla_aggregate && !s.at_escape {
                return false;
            }
            i += 1;
        }
        true
    }
    /// Some statistic in the set is accumulated as the orbit runs (not [`StatSpec::at_escape`]):
    /// the iteration carries the aux machinery (`aux_on` bit 0), and the walk in passes cannot
    /// serve the frame (its state carries no statistics). False for the final angle alone.
    pub const fn accumulates(self) -> bool {
        let mut i = 0;
        while i < Stat::ALL.len() {
            if self.has(Stat::ALL[i]) && !Stat::ALL[i].spec().at_escape {
                return true;
            }
            i += 1;
        }
        false
    }
    /// The statistics in the set, in the table's order.
    pub fn iter(self) -> impl Iterator<Item = Stat> {
        Stat::ALL.into_iter().filter(move |s| self.has(*s))
    }
}

impl std::fmt::Display for StatSet {
    /// `none`, or the keys joined by `+` (`stripe+trap`), for logs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("none");
        }
        for (i, s) in self.iter().enumerate() {
            if i > 0 {
                f.write_str("+")?;
            }
            f.write_str(s.spec().key)?;
        }
        Ok(())
    }
}

/// One orbit-trap shape's row of the table.
#[derive(Debug)]
pub struct TrapSpec {
    /// The persisted key (session, view files, farm, `--trap`).
    pub key: &'static str,
    /// The name of the shader constant that carries its GPU number (`TRAP_CROSS`).
    pub wgsl: &'static str,
    /// A turn changes it ([`TrapPlace::rotation`]): not the point or the circle, which look the
    /// same turned.
    pub turns: bool,
    /// Its size changes it ([`TrapPlace::size`]): not the point, the cross or the line, which look
    /// the same at any size.
    pub sized: bool,
    /// A closed outline around its center (the circle, the polygons, the star): placed on a cycle
    /// point it keeps its center and is scaled to pass through it ([`TrapPlace::on_cycle`]); every
    /// other shape passes through its own center and is moved there.
    pub closed: bool,
    /// Which pipelines compile its shader code in ([`TrapTier`]): `Base` is in every one.
    pub tier: TrapTier,
    /// A phase shifts it along itself ([`TrapPlace::phase`]): the sine wave and the square wave, a full
    /// wave every 360°.
    pub phased: bool,
    /// It is summed from a number of harmonics ([`TrapPlace::harmonics`]): the square wave.
    pub harmonic: bool,
    /// Its first harmonics can be drawn over it ([`TrapPlace::sine_harmonics`]): the sine wave.
    pub overlaid: bool,
}

/// Which of the shader's trap-shape switches compiles a shape's code in (`TRAP_EXTRA`, `TRAP_WAVES`,
/// `TRAP_DRAWING` in mandelbrot.wgsl). A frame's iterate pipeline is the one for the highest tier among its traps, and
/// each tier's pipelines carry the lower tiers' shapes too, so one pipeline draws any pair. A frame
/// whose traps need no more than `Base` runs the code it ran before either switch existed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub enum TrapTier {
    /// In every pipeline: the point, the cross, the circle.
    #[default]
    Base,
    /// `TRAP_EXTRA`: the shapes of F4 part 3, whose code present in every trap frame cost the point,
    /// cross and circle frames 2-3%.
    Extra,
    /// `TRAP_WAVES`, with `TRAP_EXTRA`: the sinc and the square wave (user 2026-11-25), kept out of
    /// the Extra pipelines so the six shapes there keep their speed (code added there for the F4.4
    /// outline cost them 7-10% on the RX 6800 XT, 2026-11-25).
    Waves,
    /// `TRAP_DRAWING`, with the tiers below: a drawing's outline (coloring plan F4, part 4), whose
    /// cell lookup and segment loop live only in its own pipelines (in the Extra ones they slowed the
    /// six shapes there 7-10% on the RX 6800 XT, and the RTX 3070's trap matrix timed out).
    Outline,
}

impl TrapTier {
    /// Every tier, lowest first.
    pub const ALL: [TrapTier; 4] = [TrapTier::Base, TrapTier::Extra, TrapTier::Waves, TrapTier::Outline];
}

/// The radius of the polygon and star shapes' outer corners at size 1, and the star's inner ones
/// (the trap-shape study's 5-point star: 0.9 and 0.38).
pub const STAR_INNER: f64 = 0.38 / 0.9;
/// The sine wave's amplitude and angular frequency at size 1 (the study's: 0.3 and 4).
pub const SINE_AMP: f64 = 0.3;
pub const SINE_FREQ: f64 = 4.0;
/// The S curve's two bowls' radius at size 1: the curve spans −1..1 vertically.
pub const S_RADIUS: f64 = 0.5;
/// The sinc's angular frequency at size 1: y = sin(π·x)/(π·x), 1 at its center and 0 at every other
/// whole x.
pub const SINC_FREQ: f64 = std::f64::consts::PI;
/// How many odd harmonics the square wave sums ([`TrapPlace::harmonics`]): 1 is a sine wave, more make
/// its corners sharper and its sides steeper, each one more work in every step of every orbit. The
/// same range holds the sine wave's overlaid harmonics ([`TrapPlace::sine_harmonics`]), 1 the wave
/// alone.
pub const HARMONICS_MIN: u32 = 1;
pub const HARMONICS_MAX: u32 = 32;
pub const HARMONICS_DEFAULT: u32 = 5;

/// The trap's tail (coloring plan F4, part 5): how many iterations it takes a hit to count as
/// twice as far ([`Trap::tail`]). At depth every pixel's orbit follows the reference for all but its
/// last iterations, so the nearest approach of the whole orbit is reached in that shared stretch and
/// the picture is one value; aged by its distance from the orbit's end, a hit there loses to the
/// pixel's own late ones, and the trap draws again (the study, 2026-11-26: the seahorse at 1e20x, the
/// line trap's spread 0.000 whole and 8.1 at a tail of 256; a hard window of the last iterations
/// alone was noise there, the late orbits already chaotic).
pub const TRAP_TAIL_MIN: u32 = 4;
pub const TRAP_TAIL_MAX: u32 = 1 << 20;
pub const TRAP_TAIL_DEFAULT: u32 = 256;

/// How much a trap hit's distance grows each iteration under a tail of `tail` iterations (0: off,
/// 1, nothing ages): 2^(1/tail).
pub fn trap_tail_growth(tail: u32) -> f64 {
    if tail == 0 { 1.0 } else { (1.0 / f64::from(tail)).exp2() }
}

/// How much an aged hit grows over `span` iterations under a tail of `tail` (0: off, 1): the factor
/// a BLA skip of that span applies, 2^(span/tail), held at 2^126 so it stays finite in f32 (a hit
/// that old loses to any other), and what the shader's `aux_fold_run` applies.
pub fn trap_tail_factor(tail: u32, span: u32) -> f64 {
    if tail == 0 { 1.0 } else { (f64::from(span) / f64::from(tail)).min(126.0).exp2() }
}

/// Two consecutive runs' aged nearest approaches merged, `x` then `y` of `y_span` iterations: `x`'s
/// hits are `y_span` iterations older by the end of `y` ([`trap_tail_factor`]). Off, the plain
/// minimum.
pub fn merge_trap_aged(x: f64, y: f64, y_span: u32, tail: u32) -> f64 {
    if tail == 0 { x.min(y) } else { (x * trap_tail_factor(tail, y_span)).min(y) }
}

/// `deg` wrapped into [-180, 180).
pub fn wrap_degrees(deg: f32) -> f32 {
    let w = (deg + 180.0).rem_euclid(360.0) - 180.0;
    if w.is_finite() { w } else { 0.0 }
}

/// An orbit-trap shape (the orbit-trap method's). Like [`ColorMethod`], the order gives the GPU
/// numbers for one build.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum TrapType {
    /// Distance to the origin, |z|.
    #[default]
    Point,
    /// Distance to the axes, min(|x|, |y|).
    Cross,
    /// Distance to the unit circle, ||z| - 1|.
    Circle,
    /// Distance to the real axis, |y| (coloring plan F4, part 3: the shapes from the trap-shape
    /// study, each its own picture in the seahorse valley).
    Line,
    /// A square inscribed in the unit circle, its sides parallel to the axes.
    Square,
    /// A regular hexagon inscribed in the unit circle, a corner on the real axis.
    Hexagon,
    /// A 5-point star, outer corners on the unit circle ([`STAR_INNER`] inside), a point up.
    Star,
    /// The sine wave y = [`SINE_AMP`]·sin([`SINE_FREQ`]·x) (its first-order distance), and over it,
    /// when [`TrapPlace::sine_harmonics`] asks, its next harmonics at the same height (the nearest).
    Sine,
    /// An S of two half-open bowls of radius [`S_RADIUS`], from y = −1 to 1 through the origin.
    SCurve,
    /// The sinc y = sin(π·x)/(π·x) (its first-order distance): a peak of 1 at x = 0, ripples that
    /// fade along the real axis (user 2026-11-25).
    Sinc,
    /// The square wave of height [`SINE_AMP`] and the sine wave's frequency, summed from its first
    /// [`TrapPlace::harmonics`] odd harmonics (its Fourier series; first-order distance; user
    /// 2026-11-25).
    SquareWave,
    /// A drawing's outline (coloring plan F4, part 4; [`crate::outline`]): the view's own, from an
    /// SVG file or a text, which this type does not carry. Measured with it by
    /// [`unit_dist_with`](Self::unit_dist_with); without it, as the point.
    Outline,
}

const TRAPS: [TrapSpec; 12] = [
    TrapSpec { key: "point", wgsl: "TRAP_POINT", turns: false, sized: false, closed: false, tier: TrapTier::Base, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "cross", wgsl: "TRAP_CROSS", turns: true, sized: false, closed: false, tier: TrapTier::Base, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "circle", wgsl: "TRAP_CIRCLE", turns: false, sized: true, closed: true, tier: TrapTier::Base, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "line", wgsl: "TRAP_LINE", turns: true, sized: false, closed: false, tier: TrapTier::Extra, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "square", wgsl: "TRAP_SQUARE", turns: true, sized: true, closed: true, tier: TrapTier::Extra, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "hexagon", wgsl: "TRAP_HEXAGON", turns: true, sized: true, closed: true, tier: TrapTier::Extra, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "star", wgsl: "TRAP_STAR", turns: true, sized: true, closed: true, tier: TrapTier::Extra, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "sine", wgsl: "TRAP_SINE", turns: true, sized: true, closed: false, tier: TrapTier::Extra, phased: true, harmonic: false, overlaid: true },
    TrapSpec { key: "s-curve", wgsl: "TRAP_S_CURVE", turns: true, sized: true, closed: false, tier: TrapTier::Extra, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "sinc", wgsl: "TRAP_SINC", turns: true, sized: true, closed: false, tier: TrapTier::Waves, phased: false, harmonic: false, overlaid: false },
    TrapSpec { key: "square-wave", wgsl: "TRAP_SQUARE_WAVE", turns: true, sized: true, closed: false, tier: TrapTier::Waves, phased: true, harmonic: true, overlaid: false },
    TrapSpec { key: "outline", wgsl: "TRAP_OUTLINE", turns: true, sized: true, closed: false, tier: TrapTier::Outline, phased: false, harmonic: false, overlaid: false },
];

/// The polygon and star shapes in their own frame: `(n, outer radius, inner radius, the first outer
/// corner's angle)`. A regular polygon's inner radius is its edge midpoint's, `cos(π/n)`.
fn star_params(t: TrapType) -> Option<(f64, f64, f64, f64)> {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
    match t {
        TrapType::Square => Some((4.0, 1.0, (PI / 4.0).cos(), FRAC_PI_4)),
        TrapType::Hexagon => Some((6.0, 1.0, (PI / 6.0).cos(), 0.0)),
        TrapType::Star => Some((5.0, 1.0, STAR_INNER, FRAC_PI_2)),
        _ => None,
    }
}

/// The S curve's top bowl (the shader's `trap_s_bowl`): the arc of radius [`S_RADIUS`] about
/// `(0, S_RADIUS)` from angle 0 counterclockwise to 270 degrees, in its own frame.
fn s_bowl(x: f64, y: f64) -> f64 {
    use std::f64::consts::FRAC_1_SQRT_2;
    let r = S_RADIUS;
    let (px, py) = (x, y - r);
    let (qx, qy) = ((FRAC_1_SQRT_2 * (px + py)).abs(), FRAC_1_SQRT_2 * (py - px));
    let (sx, sy) = (FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
    if sy * qx > sx * qy {
        return (qx - sx * r).hypot(qy - sy * r);
    }
    // The arc's distance from the unturned p, as the shader must take it (see `trap_s_bowl`).
    (px.hypot(py) - r).abs()
}

/// The sinc's height and slope at `x` (the shader's `trap_sinc`): sin(v)/v for v = [`SINC_FREQ`]·x,
/// and near the center its series, where the quotient's slope loses its digits to cancellation.
fn sinc(x: f64) -> (f64, f64) {
    let v = SINC_FREQ * x;
    let v2 = v * v;
    if v.abs() < 0.5 {
        let y = 1.0 - v2 / 6.0 * (1.0 - v2 / 20.0 * (1.0 - v2 / 42.0 * (1.0 - v2 / 72.0)));
        let dy = -v / 3.0 * (1.0 - v2 / 10.0 * (1.0 - v2 / 28.0 * (1.0 - v2 / 54.0 * (1.0 - v2 / 88.0))));
        return (y, SINC_FREQ * dy);
    }
    let (s, c) = v.sin_cos();
    (s / v, SINC_FREQ * (v * c - s) / v2)
}

/// The square wave's height and slope at `x` (the shader's `trap_square_wave`): its Fourier series
/// [`SINE_AMP`]·(4/π)·Σ sin((2k−1)·u)/(2k−1) over the first `n` odd harmonics, u = [`SINE_FREQ`]·x +
/// `phase`, each harmonic's angle the last one's turned by 2u, so one sine and cosine serve them all.
fn square_wave(x: f64, phase: f64, n: u32) -> (f64, f64) {
    let u = SINE_FREQ * x + phase;
    let (s1, c1) = u.sin_cos();
    let (s2, c2) = (2.0 * s1 * c1, c1 * c1 - s1 * s1);
    let (mut s, mut c) = (s1, c1);
    let (mut y, mut dy) = (0.0, 0.0);
    for k in 0..n {
        y += s / f64::from(2 * k + 1);
        dy += c;
        (s, c) = (s * c2 + c * s2, c * c2 - s * s2);
    }
    let h = SINE_AMP * 4.0 / std::f64::consts::PI;
    (h * y, h * SINE_FREQ * dy)
}

/// The sine wave with its first `n` harmonics overlaid (user 2026-11-25): how far `(x, y)` is from
/// the nearest of the curves y = [`SINE_AMP`]·sin(k·u), k = 1 to `n`, u = [`SINE_FREQ`]·x + `phase`,
/// each to first order (the shader's `trap_sine_overlaid`). Each harmonic's angle is the last one's
/// turned by u, so one sine and cosine serve them all; the first curve is the sine wave itself.
fn sine_overlaid(x: f64, y: f64, phase: f64, n: u32) -> f64 {
    let u = SINE_FREQ * x + phase;
    let (s1, c1) = u.sin_cos();
    let (mut s, mut c) = (s1, c1);
    let mut best = f64::INFINITY;
    for k in 1..=n {
        best = best.min(curve_dist(y, (SINE_AMP * s, SINE_AMP * SINE_FREQ * f64::from(k) * c)));
        (s, c) = (s * c1 + c * s1, c * c1 - s * s1);
    }
    best
}

/// How far `(x, y)` is from the curve whose height and slope at `x` are `(f, df)`, to first order.
fn curve_dist(y: f64, (f, df): (f64, f64)) -> f64 {
    (y - f).abs() / (1.0 + df * df).sqrt()
}

impl TrapType {
    /// Every shape, in the table's order.
    pub const ALL: [TrapType; 12] = [
        TrapType::Point,
        TrapType::Cross,
        TrapType::Circle,
        TrapType::Line,
        TrapType::Square,
        TrapType::Hexagon,
        TrapType::Star,
        TrapType::Sine,
        TrapType::SCurve,
        TrapType::Sinc,
        TrapType::SquareWave,
        TrapType::Outline,
    ];
    /// This shape's row of the table.
    pub const fn spec(self) -> &'static TrapSpec {
        &TRAPS[self as usize]
    }
    /// Its GPU number in this build. Never store it.
    pub const fn gpu_id(self) -> u32 {
        self as u32
    }
    /// The shape a GPU number stands for in this build.
    pub fn from_gpu_id(id: u32) -> Option<TrapType> {
        TrapType::ALL.get(id as usize).copied()
    }
    /// The persisted key.
    pub const fn key(self) -> &'static str {
        self.spec().key
    }
    /// The shape a key names, if any.
    pub fn parse_key(s: &str) -> Option<TrapType> {
        TrapType::ALL.into_iter().find(|t| t.key() == s)
    }
    /// The shape a key names, or the point trap for one it does not know.
    pub fn from_key(s: &str) -> TrapType {
        TrapType::parse_key(s).unwrap_or_default()
    }
    /// [`unit_dist`](Self::unit_dist) with a wave shifted by `phase` radians along itself (the sine,
    /// `y = A·sin(F·x + phase)`, and the square wave), and `count` its count ([`TrapPlace::count`]):
    /// the square wave summed from that many odd harmonics, the sine wave with that many of its own
    /// overlaid (1, the wave alone, exactly as before); every other shape has neither.
    pub fn unit_dist_at(self, zx: f64, zy: f64, phase: f64, count: u32) -> f64 {
        let harmonics = count;
        match self {
            TrapType::Sine if count > 1 => sine_overlaid(zx, zy, phase, count),
            TrapType::Sine if phase != 0.0 => {
                let (s, c) = (SINE_FREQ * zx + phase).sin_cos();
                (zy - SINE_AMP * s).abs() / (1.0 + (SINE_AMP * SINE_FREQ * c).powi(2)).sqrt()
            }
            TrapType::SquareWave => curve_dist(zy, square_wave(zx, phase, harmonics)),
            _ => self.unit_dist(zx, zy),
        }
    }
    /// How far `z` is from the unit shape at the origin, the outline's drawing for [`TrapType::Outline`]
    /// (without one, the point): the shader's distance in `aux_step`.
    pub fn unit_dist_with(self, outline: Option<&crate::outline::Outline>, zx: f64, zy: f64) -> f64 {
        match (self, outline) {
            (TrapType::Outline, Some(o)) => o.dist(zx, zy),
            _ => self.unit_dist(zx, zy),
        }
    }
    /// How far `z` is from the unit shape at the origin: the shader's distance in `aux_step`.
    pub fn unit_dist(self, zx: f64, zy: f64) -> f64 {
        use std::f64::consts::{FRAC_1_SQRT_2, PI};
        match self {
            TrapType::Cross => zx.abs().min(zy.abs()),
            TrapType::Circle => ((zx * zx + zy * zy).sqrt() - 1.0).abs(),
            TrapType::Point => (zx * zx + zy * zy).sqrt(),
            TrapType::Line => zy.abs(),
            // The shader's forms, with no angle taken (see `trap_extra_dist` in mandelbrot.wgsl).
            TrapType::Square => {
                let (qx, qy) = (zx.abs() - FRAC_1_SQRT_2, zy.abs() - FRAC_1_SQRT_2);
                (qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0)).abs()
            }
            TrapType::Hexagon => {
                let (kx, ky) = (-(3.0f64.sqrt()) / 2.0, 0.5);
                let (mut px, mut py) = (zx.abs(), zy.abs());
                let t = 2.0 * (kx * px + ky * py).min(0.0);
                (px, py) = (px - t * kx, py - t * ky);
                (px - px.clamp(-0.5, 0.5)).hypot(py - 3.0f64.sqrt() / 2.0)
            }
            TrapType::Star => {
                let (c, s) = ((PI / 5.0).cos(), (PI / 5.0).sin());
                let (k1, k2) = ((c, -s), (-c, -s));
                let (mut px, mut py) = (zx.abs(), zy);
                let t = 2.0 * (k1.0 * px + k1.1 * py).max(0.0);
                (px, py) = (px - t * k1.0, py - t * k1.1);
                let t = 2.0 * (k2.0 * px + k2.1 * py).max(0.0);
                (px, py) = (px - t * k2.0, py - t * k2.1);
                (px, py) = (px.abs(), py - 1.0);
                let (bx, by) = (STAR_INNER * s, STAR_INNER * c - 1.0);
                let h = ((px * bx + py * by) / (bx * bx + by * by)).clamp(0.0, 1.0);
                (px - bx * h).hypot(py - by * h)
            }
            TrapType::Sine => {
                let (s, c) = (SINE_FREQ * zx).sin_cos();
                (zy - SINE_AMP * s).abs() / (1.0 + (SINE_AMP * SINE_FREQ * c).powi(2)).sqrt()
            }
            TrapType::SCurve => s_bowl(zx, zy).min(s_bowl(-zx, -zy)),
            TrapType::Sinc => curve_dist(zy, sinc(zx)),
            TrapType::SquareWave => curve_dist(zy, square_wave(zx, 0.0, HARMONICS_DEFAULT)),
            // Its drawing is the view's, not this type's: `unit_dist_with` measures it.
            TrapType::Outline => (zx * zx + zy * zy).sqrt(),
        }
    }

    /// Which pipelines must carry this shape placed at `place` ([`TrapTier`]): its row's tier, but the
    /// sine wave with harmonics overlaid is the waves' (its loop kept out of the Extra pipelines, as
    /// the square wave's is).
    pub fn tier_for(self, place: TrapPlace) -> TrapTier {
        if self == TrapType::Sine && place.sine_harmonics > 1 {
            TrapTier::Waves
        } else {
            self.spec().tier
        }
    }

    /// The point of the shape, in its own frame at size 1, that Place on the cycle puts on the cycle
    /// ([`TrapPlace::on_cycle`]) for an open shape: its center, which every one but the sinc passes
    /// through; the sinc's peak.
    pub fn anchor(self) -> (f64, f64) {
        match self {
            TrapType::Sinc => (0.0, 1.0),
            _ => (0.0, 0.0),
        }
    }

    /// A closed shape's outline radius in direction `angle` at size 1 ([`TrapSpec::closed`]): how
    /// far from its center the ray at that angle meets it. `None` for an open shape.
    pub fn unit_radius(self, angle: f64) -> Option<f64> {
        if self == TrapType::Circle {
            return Some(1.0);
        }
        let (n, ro, ri, a0) = star_params(self)?;
        let sector = std::f64::consts::TAU / n;
        let mut a = (angle - a0).rem_euclid(sector);
        if a > 0.5 * sector {
            a = sector - a;
        }
        // The ray (cos a, sin a) against the edge from (ro, 0) to ri·(cos, sin)(sector/2).
        let (sn, cs) = (0.5 * sector).sin_cos();
        let (bx, by) = (ri * cs, ri * sn);
        let (uy, ux) = a.sin_cos();
        let (vx, vy) = (bx - ro, by);
        // cross(u, A + t·V) = 0 for A = (ro, 0): t = uy·ro / (ux·vy − uy·vx).
        let t = uy * ro / (ux * vy - uy * vx);
        Some(ux * (ro + t * vx) + uy * (t * vy))
    }
}

/// Where the orbit trap sits, how it is turned and how large it is (coloring plan F4, part 1).
///
/// The shape's own coordinates are `w = R(−rotation)·(z − center) / size`; the distance is measured
/// there and scaled back, `size · shape(w)`, so it stays a distance in the orbit's plane: the point
/// trap is `|z − center|`, the cross is the two lines through `center` turned by `rotation`, and the
/// circle's radius is `size`.
///
/// A shape ignores what it does not use ([`TrapPlace::for_shape`]). The default is each shape as it
/// always was, and a placement at the default computes exactly as before, with no transform (on the
/// GPU too), so a view that never moved its trap keeps every bit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrapPlace {
    /// The trap's center in the orbit's plane, `[re, im]`.
    pub center: [f32; 2],
    /// Its turn, in degrees counterclockwise.
    pub rotation: f32,
    /// Its size: the circle's radius. 1: the unit shape.
    pub size: f32,
    /// Its phase, in degrees: the wave shifted along itself, a full wave every 360°
    /// ([`TrapSpec::phased`]; user 2026-11-25). Wrapped into [-180, 180).
    pub phase: f32,
    /// How many odd harmonics the square wave sums ([`TrapSpec::harmonic`]; user 2026-11-25),
    /// [`HARMONICS_MIN`] to [`HARMONICS_MAX`].
    pub harmonics: u32,
    /// How many of the sine wave's harmonics are drawn, overlaid, at its height ([`TrapSpec::overlaid`];
    /// user 2026-11-25): 1, the wave alone, to [`HARMONICS_MAX`]. Its own count, not the square
    /// wave's, whose default of 5 would otherwise draw a sine wave picked after it five times over.
    pub sine_harmonics: u32,
}

impl Default for TrapPlace {
    fn default() -> Self {
        TrapPlace::DEFAULT
    }
}

impl TrapPlace {
    /// Each shape as it always was: at the origin, unturned, of size 1.
    pub const DEFAULT: TrapPlace =
        TrapPlace { center: [0.0, 0.0], rotation: 0.0, size: 1.0, phase: 0.0, harmonics: HARMONICS_DEFAULT, sine_harmonics: 1 };
    /// The sizes a placement computes with; [`for_shape`](Self::for_shape) clamps into them.
    pub const SIZE_MIN: f32 = 1.0e-4;
    pub const SIZE_MAX: f32 = 1.0e4;

    /// This placement as `shape` uses it: the turn and the size at their defaults where the shape
    /// ignores them, every number finite (an unreadable one at its default), the size clamped and
    /// no negative zero. What reaches the iteration, its keys and the BLA aggregates, so a setting the
    /// shape ignores never iterates a frame again or moves a bit.
    pub fn for_shape(self, shape: TrapType) -> TrapPlace {
        let spec = shape.spec();
        // `+ 0.0` turns −0.0 into 0.0, so a key from the bits sees one zero.
        let fin = |v: f32, d: f32| if v.is_finite() { v + 0.0 } else { d };
        TrapPlace {
            center: [fin(self.center[0], 0.0), fin(self.center[1], 0.0)],
            rotation: if spec.turns { fin(self.rotation, 0.0) } else { 0.0 },
            size: if spec.sized { fin(self.size, 1.0).clamp(Self::SIZE_MIN, Self::SIZE_MAX) } else { 1.0 },
            phase: if spec.phased { wrap_degrees(fin(self.phase, 0.0)) + 0.0 } else { 0.0 },
            harmonics: if spec.harmonic { self.harmonics.clamp(HARMONICS_MIN, HARMONICS_MAX) } else { HARMONICS_DEFAULT },
            sine_harmonics: if spec.overlaid { self.sine_harmonics.clamp(HARMONICS_MIN, HARMONICS_MAX) } else { 1 },
        }
    }
    /// The count `shape` draws with: the sine wave's overlaid harmonics, else the square wave's
    /// (what the shader's `trap_harm` words carry).
    pub fn count(self, shape: TrapType) -> u32 {
        if shape.spec().overlaid {
            self.sine_harmonics
        } else {
            self.harmonics
        }
    }
    /// Each shape as it always was.
    pub fn is_default(self) -> bool {
        self == Self::DEFAULT
    }
    /// The seven numbers' bits, for keys: `[center re, center im, rotation, size, phase, harmonics,
    /// sine harmonics]`.
    pub fn bits(self) -> [u32; 7] {
        [
            self.center[0].to_bits(),
            self.center[1].to_bits(),
            self.rotation.to_bits(),
            self.size.to_bits(),
            self.phase.to_bits(),
            self.harmonics,
            self.sine_harmonics,
        ]
    }
    /// The phase in radians.
    pub fn phase_radians(self) -> f64 {
        f64::from(self.phase).to_radians()
    }
    /// The turn's cosine and sine.
    pub fn turn(self) -> (f64, f64) {
        let r = f64::from(self.rotation).to_radians();
        (r.cos(), r.sin())
    }
    /// How far `z` is from `shape` placed here: what the shader's `trap_dist` computes in f32, and
    /// exactly [`TrapType::unit_dist`] at the default.
    pub fn dist(self, shape: TrapType, zx: f64, zy: f64) -> f64 {
        self.dist_with(shape, None, zx, zy)
    }
    /// [`dist`](Self::dist) with the view's outline, which [`TrapType::Outline`] measures from.
    pub fn dist_with(self, shape: TrapType, outline: Option<&crate::outline::Outline>, zx: f64, zy: f64) -> f64 {
        if self.is_default() {
            return shape.unit_dist_with(outline, zx, zy);
        }
        let (c, s) = self.turn();
        let (dx, dy) = (zx - f64::from(self.center[0]), zy - f64::from(self.center[1]));
        let k = f64::from(self.size);
        let (ux, uy) = ((c * dx + s * dy) / k, (c * dy - s * dx) / k);
        k * match (shape, outline) {
            (TrapType::Outline, Some(o)) => o.dist(ux, uy),
            _ => shape.unit_dist_at(ux, uy, self.phase_radians(), self.count(shape)),
        }
    }

    /// This trap moved onto a point of `cycle` (coloring plan F4, part 2; `crate::cycle`), its
    /// outline through it: an open shape is moved so that its [`anchor`](TrapType::anchor) is there
    /// (a point, a cross, a line, the waves and the S curve centered on it, the sinc's peak on it),
    /// keeping its turn and size; a closed one (a
    /// circle, a polygon, the star) keeps its center and turn and is scaled to pass through it. A trap
    /// already through one of the points moves on to the next in orbit order, so placing it again
    /// walks the whole cycle; one through none takes the nearest (to its center, or a closed one's to
    /// its outline). Returns the placement and
    /// the index taken. `cycle` must not be empty; the size is clamped as [`for_shape`](Self::for_shape)
    /// clamps it.
    pub fn on_cycle(self, shape: TrapType, cycle: &[[f64; 2]]) -> (TrapPlace, usize) {
        assert!(!cycle.is_empty(), "a cycle has a point");
        let place = self.for_shape(shape);
        let center = [f64::from(place.center[0]), f64::from(place.center[1])];
        let from = |z: [f64; 2]| (z[0] - center[0]).hypot(z[1] - center[1]);
        // An open shape's anchor in the orbit's plane, and the offset from it to the center.
        let (ax, ay) = shape.anchor();
        let (tc, ts) = place.turn();
        let ks = f64::from(place.size);
        let to_center = [-ks * (tc * ax - ts * ay), -ks * (ts * ax + tc * ay)];
        let from_anchor = |z: [f64; 2]| (z[0] + to_center[0] - center[0]).hypot(z[1] + to_center[1] - center[1]);
        // How far a cycle point is from the trap's outline, and the tolerance of f32 there.
        let closed = shape.spec().closed;
        let off = |z: [f64; 2]| if closed { place.dist(shape, z[0], z[1]) } else { from_anchor(z) };
        let tol = |z: [f64; 2]| 1.0e-6 * z[0].hypot(z[1]).max(1.0);
        let k = match (0..cycle.len()).find(|&k| off(cycle[k]) <= tol(cycle[k])) {
            Some(on) => (on + 1) % cycle.len(),
            None => (0..cycle.len()).min_by(|&i, &j| off(cycle[i]).total_cmp(&off(cycle[j]))).unwrap_or(0),
        };
        let z = cycle[k];
        let placed = if closed {
            // The outline's radius toward the point, in the shape's own frame.
            let angle = (z[1] - center[1]).atan2(z[0] - center[0]) - f64::from(place.rotation).to_radians();
            let unit = shape.unit_radius(angle).unwrap_or(1.0).max(1.0e-9);
            TrapPlace { size: (from(z) / unit) as f32, ..place }
        } else {
            TrapPlace { center: [(z[0] + to_center[0]) as f32, (z[1] + to_center[1]) as f32], ..place }
        };
        (placed.for_shape(shape), k)
    }
}

/// The orbit trap as the iteration measures it: its shape and its placement as the shape uses it
/// ([`TrapPlace::for_shape`], applied by [`Trap::new`]). What the iteration's keys, the BLA
/// aggregates and their staleness checks compare, so two settings that measure alike are one trap.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Trap {
    pub shape: TrapType,
    pub place: TrapPlace,
    /// The inside's own trap, shape and placement as the shape uses it, when it has one: the trap
    /// method's inside colored by a trap of its own. The iteration then measures both; an interior
    /// pixel's lane holds this one, an escaped pixel's the trap above. `None`: the inside measures
    /// the trap above, as before.
    pub inside: Option<(TrapType, TrapPlace)>,
    /// The view's outline's key ([`crate::outline::Outline::key`]) when this trap or the inside's is
    /// [`TrapType::Outline`], else 0: so a new drawing changes the trap as a new shape does, for
    /// every key and cache that compares traps. The drawing itself travels beside the trap.
    pub outline: u64,
    /// The tail (F4 part 5, [`TRAP_TAIL_DEFAULT`]): every hit's distance doubles each `tail`
    /// iterations it lies before the orbit's end, the trap's and the inside's alike; 0, off, the
    /// nearest approach of the whole orbit, as before.
    pub tail: u32,
}

impl Trap {
    /// `shape` placed at `place`, as the shape uses it.
    pub fn new(shape: TrapType, place: TrapPlace) -> Trap {
        Trap { shape, place: place.for_shape(shape), inside: None, outline: 0, tail: 0 }
    }
    /// `shape` at the default placement: the trap as it was before F4.
    pub fn of(shape: TrapType) -> Trap {
        Trap { shape, place: TrapPlace::DEFAULT, inside: None, outline: 0, tail: 0 }
    }
    /// This trap with the inside's own, `shape` at `place` as the shape uses it; none when that is
    /// this trap itself, which the inside then measures anyway (no second statistic, every bit as
    /// without one).
    pub fn with_inside(self, shape: TrapType, place: TrapPlace) -> Trap {
        let place = place.for_shape(shape);
        let inside = (shape != self.shape || place != self.place).then_some((shape, place));
        Trap { inside, ..self }
    }
    /// Whether this trap or the inside's measures from the view's outline.
    pub fn uses_outline(self) -> bool {
        self.shape == TrapType::Outline || self.inside.is_some_and(|(s, _)| s == TrapType::Outline)
    }
    /// This trap with the view's outline's key (0: none), kept only where a trap measures from it.
    pub fn with_outline(self, key: u64) -> Trap {
        Trap { outline: if self.uses_outline() { key } else { 0 }, ..self }
    }
    /// This trap with a tail of `tail` iterations (0: off), clamped to its range.
    pub fn with_tail(self, tail: u32) -> Trap {
        Trap { tail: if tail == 0 { 0 } else { tail.clamp(TRAP_TAIL_MIN, TRAP_TAIL_MAX) }, ..self }
    }
    /// What keys hash: the shape's GPU number for this build and the placement's bits, the inside's
    /// own the same way, the outline's key and the tail.
    pub fn key_bits(self) -> (u32, [u32; 7], Option<(u32, [u32; 7])>, u64, u32) {
        (self.shape.gpu_id(), self.place.bits(), self.inside.map(|(s, p)| (s.gpu_id(), p.bits())), self.outline, self.tail)
    }
}

/// What colors the inside of the set (coloring plan B5): a persisted choice, known by its key.
/// A view that makes none takes its method's own ([`InteriorMode::of_method`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum InteriorMode {
    /// One color.
    Solid,
    /// How near each orbit came to the orbit trap (its shape is the trap method's), through the
    /// interior palette.
    Trap,
}

const INTERIOR_KEYS: [&str; 2] = ["solid", "trap"];

impl InteriorMode {
    /// Every mode, in the table's order.
    pub const ALL: [InteriorMode; 2] = [InteriorMode::Solid, InteriorMode::Trap];
    /// The persisted key.
    pub const fn key(self) -> &'static str {
        INTERIOR_KEYS[self as usize]
    }
    /// The mode a key names, if any.
    pub fn parse_key(s: &str) -> Option<InteriorMode> {
        InteriorMode::ALL.into_iter().find(|m| m.key() == s)
    }
    /// What `method` colors the inside with when the view chose nothing: the orbit trap its trap
    /// (it always has), every other method one color.
    pub const fn of_method(method: ColorMethod) -> InteriorMode {
        match method {
            ColorMethod::OrbitTrap => InteriorMode::Trap,
            _ => InteriorMode::Solid,
        }
    }
    /// The statistic the inside needs the frame to hold, if any.
    pub const fn reads(self) -> Option<Stat> {
        match self {
            InteriorMode::Trap => Some(Stat::Trap),
            InteriorMode::Solid => None,
        }
    }
}

/// How the color pass shades an interior pixel: the GPU's side of [`InteriorMode`]. Known to the
/// shader by the constants [`wgsl_ids`] generates (`INTERIOR_METHOD`, ...); its number holds for
/// one build only.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum InteriorShading {
    /// As the method does: the orbit trap colors it by its own coordinate, palette and range
    /// mapping (exactly as before B5); every other method, one color. A request built without a
    /// choice gets this, so it renders as it always has.
    #[default]
    Method,
    /// One color, whatever the method.
    Solid,
    /// The trap's value under the interior's own rate, through the interior palette (the second
    /// LUT), or the outside one when the request carries none.
    Trap,
}

const SHADING_WGSL: [&str; 3] = ["INTERIOR_METHOD", "INTERIOR_SOLID", "INTERIOR_TRAP"];

impl InteriorShading {
    pub const ALL: [InteriorShading; 3] = [InteriorShading::Method, InteriorShading::Solid, InteriorShading::Trap];
    /// Its GPU number in this build. Never store it.
    pub const fn gpu_id(self) -> u32 {
        self as u32
    }
    /// How a frame colored by `method` shades its inside for the `mode` it shows, with or without
    /// colors of its own (a palette of its own, or its own fitted range, C4). The trap method's own
    /// interior, in the method's colors, keeps its exact path ([`InteriorShading::Method`]: its
    /// range mapping included).
    pub const fn for_view(mode: InteriorMode, method: ColorMethod, own_colors: bool) -> InteriorShading {
        match (mode, method) {
            (InteriorMode::Solid, _) => InteriorShading::Solid,
            (InteriorMode::Trap, ColorMethod::OrbitTrap) if !own_colors => InteriorShading::Method,
            (InteriorMode::Trap, _) => InteriorShading::Trap,
        }
    }
}

/// The shader constants for this build's GPU numbers: `const METHOD_STRIPE: u32 = 1u;` for every
/// method, `const TRAP_CROSS: u32 = 1u;` for every trap shape and `const STAT_TRAP: u32 = 4u;`
/// for every statistic's bit. The GPU crate appends this to the renderer's shader source; the
/// shader compares `color_method` and `trap_type` and masks `orbit_stats` only with these names,
/// never with a number.
pub fn wgsl_ids() -> String {
    let mut s = String::from(
        "\n// ---- Generated by fractadyne_core::coloring::wgsl_ids (design/coloring-plan.md A10, B2) ----\n\
         // The coloring methods', trap shapes' and statistics' GPU numbers for this build. Compare\n\
         // `color_method` and `trap_type`, and mask `orbit_stats`, with these names only.\n",
    );
    for m in ColorMethod::ALL {
        let _ = writeln!(s, "const {}: u32 = {}u;", m.spec().wgsl, m.gpu_id());
    }
    for t in TrapType::ALL {
        let _ = writeln!(s, "const {}: u32 = {}u;", t.spec().wgsl, t.gpu_id());
    }
    // The shapes' dimensions at size 1 (F4 part 3), so the shader's `trap_dist_of` and the BLA
    // aggregates (`TrapType::unit_dist`) measure the same outline.
    let _ = writeln!(s, "const TRAP_STAR_INNER: f32 = {:?};", STAR_INNER as f32);
    let _ = writeln!(s, "const TRAP_SINE_AMP: f32 = {:?};", SINE_AMP as f32);
    let _ = writeln!(s, "const TRAP_SINE_FREQ: f32 = {:?};", SINE_FREQ as f32);
    let _ = writeln!(s, "const TRAP_S_RADIUS: f32 = {:?};", S_RADIUS as f32);
    let _ = writeln!(s, "const TRAP_SINC_FREQ: f32 = {:?};", SINC_FREQ as f32);
    // The square wave sums at most this many harmonics, whatever reaches the shader.
    let _ = writeln!(s, "const TRAP_HARMONICS_MAX: u32 = {HARMONICS_MAX}u;");
    // The outline's grid (`crate::outline`): cells per side and the half-width of the square it covers.
    let _ = writeln!(s, "const OUTLINE_GRID: u32 = {}u;", crate::outline::GRID);
    let _ = writeln!(s, "const OUTLINE_DOMAIN: f32 = {:?};", crate::outline::DOMAIN as f32);
    for st in Stat::ALL {
        let _ = writeln!(s, "const {}: u32 = {}u;", st.spec().wgsl, st.bit());
    }
    for i in InteriorShading::ALL {
        let _ = writeln!(s, "const {}: u32 = {}u;", SHADING_WGSL[i as usize], i.gpu_id());
    }
    s
}

/// How a palette map shapes a value before the palette (coloring plan C2): the value's height above
/// the map's floor `lo`, d = max(v − lo, 0), becomes T(d), which the map's rate and Offset then place
/// on the palette. Every T is 0 at 0, so the floor keeps its color whatever the curve. A normalized
/// map (Normalize deep colors, Always fit range) takes its rate from T over the window's span, so
/// the window holds the same number of palette sweeps under every curve and only where the bands
/// fall changes: the roots and the logs widen the bands far from the set and narrow them near it,
/// where the counts crowd; Square does the opposite. Linear keeps v itself (`v·rate + offset`), as
/// the map always has.
///
/// ⚠The GPU numbers are fixed, not the table's order: 0 and 1 (linear, log) are what every frame
/// and video map was drawn with before the others existed (`ColorU.norm_mode`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Transfer {
    #[default]
    Linear,
    Sqrt,
    Cbrt,
    Root4,
    Log,
    LogLog,
    Square,
}

impl Transfer {
    /// Every curve, in the order a list shows them.
    pub const ALL: [Transfer; 7] =
        [Transfer::Linear, Transfer::Sqrt, Transfer::Cbrt, Transfer::Root4, Transfer::Log, Transfer::LogLog, Transfer::Square];
    /// The persisted key.
    pub const fn key(self) -> &'static str {
        match self {
            Transfer::Linear => "linear",
            Transfer::Sqrt => "sqrt",
            Transfer::Cbrt => "cbrt",
            Transfer::Root4 => "root4",
            Transfer::Log => "log",
            Transfer::LogLog => "loglog",
            Transfer::Square => "square",
        }
    }
    /// The curve a key names, if any.
    pub fn parse_key(s: &str) -> Option<Transfer> {
        Transfer::ALL.into_iter().find(|t| t.key() == s)
    }
    /// Its GPU number (`norm_mode`): fixed, see the type.
    pub const fn gpu_id(self) -> u32 {
        match self {
            Transfer::Linear => 0,
            Transfer::Log => 1,
            Transfer::Sqrt => 2,
            Transfer::Cbrt => 3,
            Transfer::Root4 => 4,
            Transfer::LogLog => 5,
            Transfer::Square => 6,
        }
    }
    /// The curve a GPU number stands for.
    pub fn from_gpu_id(id: u32) -> Option<Transfer> {
        Transfer::ALL.into_iter().find(|t| t.gpu_id() == id)
    }
    /// T(d) for a height d ≥ 0 above the floor, as the color pass's `transfer_of` computes it
    /// (Linear: d itself; a map that is linear does not subtract its floor, see the type).
    pub fn apply(self, d: f64) -> f64 {
        let d = d.max(0.0);
        match self {
            Transfer::Linear => d,
            Transfer::Sqrt => d.sqrt(),
            Transfer::Cbrt => d.cbrt(),
            Transfer::Root4 => d.sqrt().sqrt(),
            Transfer::Log => (d + 1.0).ln(),
            Transfer::LogLog => ((d + 1.0).ln() + 1.0).ln(),
            Transfer::Square => d * d,
        }
    }
    /// dT/dd at d ≥ 0, as the color pass's `transfer_slope` (the palette anti-aliasing's footprint
    /// is the map's rate times it). The roots' slopes are taken at d ≥ 1e-6: unbounded at 0.
    pub fn slope(self, d: f64) -> f64 {
        let d = d.max(0.0);
        let r = d.max(1.0e-6);
        match self {
            Transfer::Linear => 1.0,
            Transfer::Sqrt => 0.5 / r.sqrt(),
            Transfer::Cbrt => r.powf(-2.0 / 3.0) / 3.0,
            Transfer::Root4 => 0.25 * r.powf(-0.75),
            Transfer::Log => 1.0 / (d + 1.0),
            Transfer::LogLog => 1.0 / ((d + 1.0) * ((d + 1.0).ln() + 1.0)),
            Transfer::Square => 2.0 * d,
        }
    }
    /// The height d ≥ 0 whose T(d) is `y` ≥ 0: [`Self::apply`]'s inverse, for a fitted window's
    /// coordinate back to its escape value (the equalizing table, C6).
    pub fn inverse(self, y: f64) -> f64 {
        let y = y.max(0.0);
        match self {
            Transfer::Linear => y,
            Transfer::Sqrt => y * y,
            Transfer::Cbrt => y * y * y,
            Transfer::Root4 => (y * y) * (y * y),
            Transfer::Log => y.exp_m1(),
            Transfer::LogLog => y.exp_m1().exp_m1(),
            Transfer::Square => y.sqrt(),
        }
    }
}

/// Intervals of the equalizing remap's table (coloring plan C6): `R(u)` is held at
/// u = j / `EQ_KNOTS`, j = 0..=`EQ_KNOTS`. 256 put the remap within 0.07 of a palette sweep of the
/// exact rank-order map on six views from 1x to 9.3e78x (the prototype, 2026-11-23); 128 left 0.25
/// at a shallow view whose fitted window is linear and most of whose pixels sit near its floor.
pub const EQ_KNOTS: usize = 256;

/// Intervals of the quantile function a view's escape values are held as ([`Quantiles`]).
pub const EQ_QUANTILES: usize = 256;

/// A view's escaped values held as their QUANTILE FUNCTION (coloring plan C6): `q[i]` is the value
/// by which share i / [`EQ_QUANTILES`] of the escaped pixels had escaped, `q[0]` the lowest and the
/// last the highest; never decreasing. Unlike a histogram it needs no base, so two can be blended
/// knot by knot, and a blend moves each share of the picture rather than each value: the live view
/// eases one reading into the next this way.
pub type Quantiles = [f32; EQ_QUANTILES + 1];

/// The [`Quantiles`] of `sorted` (ascending, finite), exactly: knot i at rank i / [`EQ_QUANTILES`]
/// of the way from the first value to the last, interpolated between the two values it falls
/// between. `None` for no values.
pub fn quantiles_of_sorted(sorted: &[f32]) -> Option<Quantiles> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    let mut q = [0.0f32; EQ_QUANTILES + 1];
    for (i, k) in q.iter_mut().enumerate() {
        let r = (n - 1) as f64 * i as f64 / EQ_QUANTILES as f64;
        let j = (r.floor() as usize).min(n - 1);
        let f = r - j as f64;
        let hi = sorted[(j + 1).min(n - 1)] as f64;
        *k = (sorted[j] as f64 + (hi - sorted[j] as f64) * f) as f32;
    }
    Some(q)
}

/// The [`Quantiles`] of weighted values (coloring plan C6b): `atoms` are `(value, weight)` pairs
/// (reordered by value here), each standing for `weight` of the picture at its value. Knot i is
/// the value by which share i / [`EQ_QUANTILES`] of the weight lies, each atom's weight centered on
/// its value (the midpoint rule) and linear between atoms; the first knot is `lo` and the last
/// `hi`, the lowest and highest values the atoms summarize. A zoom video reads a frame's
/// distribution this way from its strips' block summaries, a block's values weighted by the area
/// its texels cover in the frame. `None` when the atoms weigh nothing.
pub fn quantiles_of_weighted(atoms: &mut [(f32, f64)], lo: f32, hi: f32) -> Option<Quantiles> {
    atoms.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total: f64 = atoms.iter().map(|a| a.1).sum();
    if !(total > 0.0) || !(hi >= lo) {
        return None;
    }
    // Each atom's weight centered on it: the share below its middle.
    let mut mid = Vec::with_capacity(atoms.len());
    let mut acc = 0.0;
    for a in atoms.iter() {
        mid.push((acc + a.1 / 2.0) / total);
        acc += a.1;
    }
    let mut q = [0.0f32; EQ_QUANTILES + 1];
    q[0] = lo;
    q[EQ_QUANTILES] = hi;
    let (first, last) = (0, atoms.len() - 1);
    let mut j = 0usize;
    for (i, k) in q.iter_mut().enumerate().take(EQ_QUANTILES).skip(1) {
        let s = i as f64 / EQ_QUANTILES as f64;
        while j < mid.len() && mid[j] < s {
            j += 1;
        }
        let between = |a: f64, b: f64, sa: f64, sb: f64| a + (b - a) * ((s - sa) / (sb - sa).max(1.0e-300)).clamp(0.0, 1.0);
        *k = if j == first {
            between(lo as f64, atoms[first].0 as f64, 0.0, mid[first])
        } else if j > last {
            between(atoms[last].0 as f64, hi as f64, mid[last], 1.0)
        } else {
            between(atoms[j - 1].0 as f64, atoms[j].0 as f64, mid[j - 1], mid[j])
        } as f32;
    }
    // Never decreasing and inside [lo, hi], whatever rounding did.
    let mut floor = lo;
    for k in q.iter_mut() {
        *k = k.clamp(floor, hi);
        floor = *k;
    }
    Some(q)
}

/// Two held distributions combined knot by knot (coloring plan C6b): knot i of the result is
/// `f(a[i], b[i])`, kept never decreasing. With `f` a blend, each share of the picture moves from
/// its value in one to its value in the other, as the live view eases one reading into the next: a
/// tour blends its keyframes' distributions so, and a zoom video its octaves'.
pub fn quantiles_zip(a: &Quantiles, b: &Quantiles, f: impl Fn(f32, f32) -> f32) -> Quantiles {
    let mut q = [0.0f32; EQ_QUANTILES + 1];
    let mut floor = f32::NEG_INFINITY;
    for (i, k) in q.iter_mut().enumerate() {
        *k = f(a[i], b[i]).max(floor);
        floor = *k;
    }
    q
}

/// The share of the escaped pixels at or below `v` under the held distribution `q`: the quantile
/// function inverted, linear between its knots; 0 below the lowest value, 1 from the highest. A run
/// of equal knots (many pixels at one value) counts whole once `v` reaches it.
pub fn cdf_at(q: &Quantiles, v: f64) -> f64 {
    let hi = q.partition_point(|&x| (x as f64) <= v);
    if hi == 0 {
        return 0.0;
    }
    if hi > EQ_QUANTILES {
        return 1.0;
    }
    let (a, b) = (q[hi - 1] as f64, q[hi] as f64);
    ((hi - 1) as f64 + (v - a) / (b - a)) / EQ_QUANTILES as f64
}

/// The equalizing remap of a fitted palette window (coloring plan C6), as its [`EQ_KNOTS`] + 1
/// knots: the fitted map puts value v at u in [0, 1] across the window `[lo, top]` (u = (v − lo) /
/// (top − lo) for `Transfer::Linear`, T(v − lo) / T(top − lo) for a curve); the knot at u is the
/// share of the picture at or below that v, as a share of what lies at or below `top`. So R(0) is 0
/// when the window starts at the lowest value, R(1) = 1, and R climbs fastest where the pixels
/// crowd: a palette laid on R gives each color about the same area of the picture.
/// An empty share at `top` (nothing measured below it) gives the identity, R(u) = u.
pub fn eq_knots(q: &Quantiles, lo: f64, top: f64, t: Transfer) -> [f32; EQ_KNOTS + 1] {
    let mut k = [0.0f32; EQ_KNOTS + 1];
    let span = top - lo;
    let c_top = cdf_at(q, top);
    if !(span > 0.0) || !(c_top > 0.0) {
        for (j, x) in k.iter_mut().enumerate() {
            *x = j as f32 / EQ_KNOTS as f32;
        }
        return k;
    }
    let t_span = t.apply(span);
    let mut last = 0.0f32;
    for (j, x) in k.iter_mut().enumerate() {
        let u = j as f64 / EQ_KNOTS as f64;
        let v = match t {
            Transfer::Linear => lo + u * span,
            t => lo + t.inverse(u * t_span),
        };
        // Never decreasing and inside [0, 1], whatever rounding did.
        last = ((cdf_at(q, v) / c_top) as f32).clamp(last, 1.0);
        *x = last;
    }
    k[EQ_KNOTS] = 1.0;
    k
}

/// R(u) from its knots ([`eq_knots`]): linear between them; outside [0, 1] it goes on at slope 1
/// from its ends, so values past the window keep cycling at the fitted rate, as they always have.
pub fn eq_remap(k: &[f32; EQ_KNOTS + 1], u: f64) -> f64 {
    if u <= 0.0 {
        return k[0] as f64 + u;
    }
    if u >= 1.0 {
        return k[EQ_KNOTS] as f64 + (u - 1.0);
    }
    let x = u * EQ_KNOTS as f64;
    let i = (x.floor() as usize).min(EQ_KNOTS - 1);
    let f = x - i as f64;
    k[i] as f64 + (k[i + 1] as f64 - k[i] as f64) * f
}

/// dR/du ([`eq_remap`]'s slope): the interval's, 1 outside [0, 1].
pub fn eq_slope(k: &[f32; EQ_KNOTS + 1], u: f64) -> f64 {
    if !(0.0..1.0).contains(&u) {
        return 1.0;
    }
    let i = ((u * EQ_KNOTS as f64).floor() as usize).min(EQ_KNOTS - 1);
    (k[i + 1] as f64 - k[i] as f64) * EQ_KNOTS as f64
}

/// The equalized coordinate (coloring plan C6, the color pass's `equalized`): `x` is the fitted
/// map's coordinate before its offset, `sweeps · u`; the result mixes it with the remap,
/// (1 − `share`) · x + `share` · sweeps · R(u). `share` 0 is the fitted map itself, 1 histogram
/// coloring.
pub fn equalized(x: f64, share: f64, sweeps: f64, k: &[f32; EQ_KNOTS + 1]) -> f64 {
    if !(sweeps > 0.0) {
        return x;
    }
    (1.0 - share) * x + share * sweeps * eq_remap(k, x / sweeps)
}

/// How the palette continues past its end (coloring plan C2). Repeat starts it again; Mirror runs
/// it back the other way, so a palette whose two ends differ has no seam; Clamp holds the last
/// color (and the first below 0), so every color is used once across the map's range.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Wrap {
    #[default]
    Repeat,
    Mirror,
    Clamp,
}

impl Wrap {
    pub const ALL: [Wrap; 3] = [Wrap::Repeat, Wrap::Mirror, Wrap::Clamp];
    pub const fn key(self) -> &'static str {
        match self {
            Wrap::Repeat => "repeat",
            Wrap::Mirror => "mirror",
            Wrap::Clamp => "clamp",
        }
    }
    pub fn parse_key(s: &str) -> Option<Wrap> {
        Wrap::ALL.into_iter().find(|w| w.key() == s)
    }
    /// Its GPU number (`ColorU.wrap`).
    pub const fn gpu_id(self) -> u32 {
        match self {
            Wrap::Repeat => 0,
            Wrap::Mirror => 1,
            Wrap::Clamp => 2,
        }
    }
    pub fn from_gpu_id(id: u32) -> Option<Wrap> {
        Wrap::ALL.into_iter().find(|w| w.gpu_id() == id)
    }
}

/// What textures the Smooth method's bands (coloring plan E1, "statistic-warped bands"): a
/// statistic of each orbit shifts its palette position by `amount · (statistic − center)` palette
/// cycles, so the bands keep their colors and order while every band edge feathers along the field
/// lines: into fur or flame with the stripes, into fewer, longer strands with the triangle
/// inequality (the study, 2026-11-14: at the seahorse valley, amount 2, no new aliasing: 23.0% and
/// 22.7% of pixels with a palette footprint over half a cycle, against 22.8% for Smooth). The
/// statistic is accumulated as its own method accumulates it, the stripe at its density.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum BandTexture {
    #[default]
    None,
    Stripe,
    Triangle,
}

/// The band texture's strength in palette cycles per unit of the statistic: the panel's and a view's
/// range, and the study's 2.
pub const BAND_TEXTURE_AMOUNT_MAX: f32 = 8.0;
pub const BAND_TEXTURE_AMOUNT_DEFAULT: f32 = 2.0;

impl BandTexture {
    pub const ALL: [BandTexture; 3] = [BandTexture::None, BandTexture::Stripe, BandTexture::Triangle];
    pub const fn key(self) -> &'static str {
        match self {
            BandTexture::None => "none",
            BandTexture::Stripe => "stripe",
            BandTexture::Triangle => "triangle",
        }
    }
    pub fn parse_key(s: &str) -> Option<BandTexture> {
        BandTexture::ALL.into_iter().find(|t| t.key() == s)
    }
    /// What the iteration must accumulate for it (none for `None`).
    pub fn stat(self) -> Option<Stat> {
        match self {
            BandTexture::None => None,
            BandTexture::Stripe => Some(Stat::Stripe),
            BandTexture::Triangle => Some(Stat::Triangle),
        }
    }
    /// Its GPU number (`ColorU.band_tex`): 0 none, 1 the stripe average (`aux.r`), 2 the triangle
    /// inequality's (`aux.g`).
    pub const fn gpu_id(self) -> u32 {
        match self {
            BandTexture::None => 0,
            BandTexture::Stripe => 1,
            BandTexture::Triangle => 2,
        }
    }
    /// The statistic's value that leaves a band where it was: the stripe term's mean, 0.5 by its
    /// symmetry, and the triangle inequality's median at the seahorse valley, 0.34 (the study). Fixed,
    /// since a constant only moves every band alike, as the palette's offset does; held so that a
    /// change of strength does not.
    pub const fn center(self) -> f32 {
        match self {
            BandTexture::None => 0.0,
            BandTexture::Stripe => 0.5,
            BandTexture::Triangle => 0.34,
        }
    }
}

/// How the statistic layer merges over the Smooth method's colors (coloring plan E3, "two-layer
/// blend"): the merge modes of Ultra Fractal's layers and the W3C compositing spec, on the
/// display-encoded colors as those blend, the backdrop B the palette's color and the source S the
/// layer's gray. Hard light (Very Plotter's stripe over smooth) is the default.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum BlendMode {
    /// B·S: always darker, the gray's middle halves the color.
    Multiply,
    /// B + S − B·S: always lighter.
    Screen,
    /// Hard light with the roles swapped: the backdrop decides between multiply and screen.
    Overlay,
    /// Multiply by 2S where S ≤ ½, else screen by 2S − 1: the gray's middle leaves the color as it
    /// is, its darks and lights shade and light it.
    #[default]
    HardLight,
    /// A milder hard light (the W3C formula).
    SoftLight,
}

impl BlendMode {
    pub const ALL: [BlendMode; 5] =
        [BlendMode::HardLight, BlendMode::SoftLight, BlendMode::Overlay, BlendMode::Multiply, BlendMode::Screen];
    pub const fn key(self) -> &'static str {
        match self {
            BlendMode::Multiply => "multiply",
            BlendMode::Screen => "screen",
            BlendMode::Overlay => "overlay",
            BlendMode::HardLight => "hard-light",
            BlendMode::SoftLight => "soft-light",
        }
    }
    pub fn parse_key(s: &str) -> Option<BlendMode> {
        BlendMode::ALL.into_iter().find(|m| m.key() == s)
    }
    /// Its GPU number (`ColorU.layer_blend`), for one build only.
    pub const fn gpu_id(self) -> u32 {
        match self {
            BlendMode::Multiply => 0,
            BlendMode::Screen => 1,
            BlendMode::Overlay => 2,
            BlendMode::HardLight => 3,
            BlendMode::SoftLight => 4,
        }
    }
    /// One channel merged, `b` the backdrop and `s` the source, both 0..1: the shader's `blended`.
    pub fn apply(self, b: f32, s: f32) -> f32 {
        let multiply = |b: f32, s: f32| b * s;
        let screen = |b: f32, s: f32| b + s - b * s;
        let hard = |b: f32, s: f32| if s <= 0.5 { multiply(b, 2.0 * s) } else { screen(b, 2.0 * s - 1.0) };
        match self {
            BlendMode::Multiply => multiply(b, s),
            BlendMode::Screen => screen(b, s),
            BlendMode::Overlay => hard(s, b),
            BlendMode::HardLight => hard(b, s),
            BlendMode::SoftLight => {
                if s <= 0.5 {
                    b - (1.0 - 2.0 * s) * b * (1.0 - b)
                } else {
                    let d = if b <= 0.25 { ((16.0 * b - 12.0) * b + 4.0) * b } else { b.sqrt() };
                    b + (2.0 * s - 1.0) * (d - b)
                }
            }
        }
    }
}

/// The statistic layer's contrast: its gray is `0.5 + contrast · (statistic − center)`, clamped,
/// the statistic's center ([`BandTexture::center`]) the gray's middle. 4 maps the stripe's 1st to
/// 99th percentile at the seahorse valley (0.37 to 0.62) about onto 0 to 1, where the hard light
/// looked as the prototype's stretch to those percentiles did (2026-11-26); the GPU has no
/// histogram of the statistic, so the stretch is fixed.
pub const LAYER_CONTRAST_DEFAULT: f32 = 4.0;
pub const LAYER_CONTRAST_MAX: f32 = 16.0;

/// The statistic bump's height (coloring plan E4): output pixels of height per unit of the layer's
/// gray ([`layer_gray`] at the default contrast), the study's 6 (2026-11-26: at the seahorse valley
/// the bump's shade from each neighbor pixel's mean held 0.89 of its structure against a render
/// shifted a third of a pixel; from one sample a neighbor, 0.56).
pub const BUMP_HEIGHT_DEFAULT: f32 = 6.0;
pub const BUMP_HEIGHT_MAX: f32 = 32.0;

/// The gray of the statistic layer for a statistic value `s` (coloring plan E3): the shader's.
pub fn layer_gray(s: f32, center: f32, contrast: f32) -> f32 {
    (0.5 + contrast * (s - center)).clamp(0.0, 1.0)
}

/// The most steps a palette cycle may be cut into (`steps`; 0 is none).
pub const MAX_STEPS: u32 = 64;

/// A palette position `t` as the color pass looks it up under `steps` and `wrap` (its
/// `wrap_coord`): `steps` > 0 cuts each palette cycle into that many flat bands, each the color at
/// its start; then the position is folded into the palette, 0 to 1: Repeat by its fraction, Mirror
/// back and forth (period 2), Clamp held at the ends. The palette is then read at the result
/// (`fractadyne_color`'s LUT sample: wrapping for Repeat, clamped to the end texels otherwise).
pub fn wrap_coord(t: f64, wrap: Wrap, steps: u32) -> f64 {
    let t = if steps > 0 { (t * steps as f64).floor() / steps as f64 } else { t };
    match wrap {
        Wrap::Repeat => t - t.floor(),
        Wrap::Mirror => {
            let p = t - 2.0 * (t / 2.0).floor();
            if p > 1.0 {
                2.0 - p
            } else {
                p
            }
        }
        Wrap::Clamp => t.clamp(0.0, 1.0),
    }
}

/// How edge shading (coloring plan C3) fades out with the distance to the set. A texel `d` output
/// pixels from the set takes the edge color at the weight [`edge_weight`], `strength · (1 − f(x))`
/// with x = min(d / width, 1): all of it at the set, none `width` pixels out. Linear is Chéritat's
/// blend (t = d / (thickness · pixel)); Square root keeps the line thin with a long soft tail;
/// Square widens it; Smooth eases both ends; Hard is Chéritat's thickness test, every texel nearer
/// than `width` fully shaded (a supersampled pixel averages its texels, so even that line is
/// anti-aliased). The GPU numbers are the list's order (`ColorU.edge_curve`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum EdgeCurve {
    #[default]
    Linear,
    Sqrt,
    Square,
    Smooth,
    Hard,
}

impl EdgeCurve {
    /// Every curve, in the order a list shows them.
    pub const ALL: [EdgeCurve; 5] = [EdgeCurve::Linear, EdgeCurve::Sqrt, EdgeCurve::Square, EdgeCurve::Smooth, EdgeCurve::Hard];
    /// The persisted key.
    pub const fn key(self) -> &'static str {
        match self {
            EdgeCurve::Linear => "linear",
            EdgeCurve::Sqrt => "sqrt",
            EdgeCurve::Square => "square",
            EdgeCurve::Smooth => "smooth",
            EdgeCurve::Hard => "hard",
        }
    }
    /// The curve a key names, if any.
    pub fn parse_key(s: &str) -> Option<EdgeCurve> {
        EdgeCurve::ALL.into_iter().find(|c| c.key() == s)
    }
    /// Its GPU number (`ColorU.edge_curve`, the shader's `edge_curve_of`).
    pub const fn gpu_id(self) -> u32 {
        match self {
            EdgeCurve::Linear => 0,
            EdgeCurve::Sqrt => 1,
            EdgeCurve::Square => 2,
            EdgeCurve::Smooth => 3,
            EdgeCurve::Hard => 4,
        }
    }
    pub fn from_gpu_id(id: u32) -> Option<EdgeCurve> {
        EdgeCurve::ALL.into_iter().find(|c| c.gpu_id() == id)
    }
    /// f(x) for x in 0..=1 (clamped into it): 0 at the set, 1 at the shading's width.
    pub fn apply(self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        match self {
            EdgeCurve::Linear => x,
            EdgeCurve::Sqrt => x.sqrt(),
            EdgeCurve::Square => x * x,
            EdgeCurve::Smooth => x * x * (3.0 - 2.0 * x),
            EdgeCurve::Hard => {
                if x >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// The narrowest edge shading, in output pixels (the shader divides by the width).
pub const EDGE_WIDTH_MIN: f32 = 0.05;

/// The share of the edge color in a texel `d_px` output pixels from the set (coloring plan C3), as
/// the color pass's `edge_shaded` mixes it: `strength · (1 − curve(min(d_px / width, 1)))`, 0 to 1.
pub fn edge_weight(d_px: f64, width: f64, strength: f64, curve: EdgeCurve) -> f64 {
    let x = (d_px / width.max(EDGE_WIDTH_MIN as f64)).clamp(0.0, 1.0);
    (strength * (1.0 - curve.apply(x))).clamp(0.0, 1.0)
}

// ---- Equipotential lines (coloring plan C10) ----
// Lines along the levels of the smooth count μ, drawn `width` output pixels wide and kept about
// `spacing` output pixels apart wherever they are: a texel's distance from the set d (output pixels)
// gives μ's gradient, |∇μ| = 1 / (d · ln p) (the stored distance, |z|·ln|z| / |dz/dc|, is G / |∇G| for
// the Green's function G, and μ = −log_p G + const), so a texel is d · ln p pixels from the level one
// count away. Each texel draws the lines at the multiples of 1/L, L the power of two that puts them
// `spacing` to twice that apart there, and fades in the next level's extra lines (2L) as the spacing
// widens toward 2 · spacing: the finer set holds the coarser one, so nothing pops as the view zooms,
// and the lines stay visible at every depth (the coloring research's level-of-detail grid: 1.1 to
// 1.5 px lines at eight depths in its prototype). The color pass's `grid_shaded` is this.

/// The narrowest spacing of the equipotential lines, in output pixels.
pub const GRID_SPACING_MIN: f32 = 1.0;
/// The smallest tap a line's coverage is filtered over, in output pixels (the shader divides by it).
pub const GRID_TAP_MIN: f64 = 1.0e-3;
/// The lines fade in from `GRID_SET_FADE.0` to `GRID_SET_FADE.1` output pixels from the set: nearer,
/// μ's levels crowd into the filaments' dust.
pub const GRID_SET_FADE: (f64, f64) = (0.5, 2.0);
/// The lines fade out as μ's own f32 step, in output pixels, grows from `GRID_PRECISION_FADE.0` to
/// `GRID_PRECISION_FADE.1`: the count is stored as an f32 (`iter + 1 − ν`), whose step is about
/// |μ| · 2^-23, so far from the set at a high count (a million iterations: 1/16 of one) a line's
/// position moves in steps of whole pixels, and it would be drawn as a staircase of blocks.
pub const GRID_PRECISION_FADE: (f64, f64) = (0.25, 1.0);

/// WGSL's `smoothstep`.
fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How much of a tap `tap` output pixels across, centered `dist` output pixels from a line's
/// center, the line `width` pixels wide covers: the exact box-filtered line, so over any one line the
/// coverage adds up to its width whatever the tap (a supersampled pixel's texels, a progressive
/// sample's share), and the line keeps its width at every anti-aliasing level.
pub fn grid_cover(dist: f64, width: f64, tap: f64) -> f64 {
    let r = tap.max(GRID_TAP_MIN);
    let lo = (dist - 0.5 * r).max(-0.5 * width);
    let hi = (dist + 0.5 * r).min(0.5 * width);
    ((hi - lo) / r).clamp(0.0, 1.0)
}

/// The line color's share in a texel (coloring plan C10), as the color pass's `grid_shaded` mixes it:
/// `mu` its smooth count, `d_px` its distance from the set and `tap` its size, in output pixels;
/// `ln_p` the log of the formula's escape degree; the lines `spacing` and `width` output pixels, at
/// `strength`. 0 to 1.
pub fn grid_weight(mu: f64, d_px: f64, ln_p: f64, spacing: f64, width: f64, strength: f64, tap: f64) -> f64 {
    if ln_p <= 0.0 {
        return 0.0;
    }
    // Output pixels per count.
    let s = d_px * ln_p;
    let lev = (s / spacing.max(GRID_SPACING_MIN as f64)).max(1.0e-30).log2();
    let l0 = lev.floor().clamp(-60.0, 60.0);
    let fr = (lev - l0).clamp(0.0, 1.0);
    let lines = l0.exp2();
    let cover = |n: f64| {
        let f = ((n * mu + 0.5).rem_euclid(1.0) - 0.5).abs() / n;
        grid_cover(f * s, width, tap)
    };
    let c = cover(lines).max(smoothstep(0.0, 1.0, fr) * cover(2.0 * lines));
    let near = smoothstep(GRID_SET_FADE.0, GRID_SET_FADE.1, d_px);
    let q = mu.abs() * (-23.0f64).exp2() * s;
    let precise = 1.0 - smoothstep(GRID_PRECISION_FADE.0, GRID_PRECISION_FADE.1, q);
    (strength * c * near * precise).clamp(0.0, 1.0)
}

// ---- Relief lighting: the slope from the distance and the highlight (coloring plan C5) ----
//
// The relief light shades the surface whose horizontal direction of steepest ascent is the stored
// slope normal `n` (a unit 2-vector, the direction of z / (dz/dc)). With a slope `k` (rise over
// run) the surface normal is (k·n, 1), and the light, `light_angle` round and `h` up (the Relief
// setting: h = tan of the light's elevation), is (l, h). The diffuse shade is their dot product,
// scaled so a face turned to the light is 1: (k·(n·l) + h) / (k + h). At k = 1 that is the classic
// relief, (n·l + h) / (1 + h), exactly. From the distance, the slope is `steep_px / d` with d the
// distance from the set in output pixels: 45 degrees `steep_px` pixels from the set, steeper nearer
// and flat far away, so the shading gathers at the set's edges, the background stays evenly lit, and
// it looks the same at every depth. The highlight is Blinn-Phong on that surface, viewed from
// straight above: (N·H)^shininess, H halfway between the light and the eye. It sits where the
// surface tilts halfway toward the light, (90 degrees - the light's elevation) / 2, which a surface
// of one slope at 45 degrees never does at the default light; so the highlight goes with the slope
// from the distance.

/// The steepest slope the relief takes from the distance (a texel a millionth of `steep_px` from the
/// set): enough that the shade is the light's horizontal part alone.
pub const RELIEF_SLOPE_MAX: f64 = 1.0e6;

/// The relief's slope (rise over run) at `d_px` output pixels from the set: `steep_px / d_px`, or 1
/// everywhere (the classic relief) when `steep_px` is 0 or less. 0 where the distance is unknown
/// (`d_px` not finite: a flat, evenly lit surface).
pub fn relief_slope(d_px: f64, steep_px: f64) -> f64 {
    if steep_px <= 0.0 {
        return 1.0;
    }
    if !d_px.is_finite() {
        return 0.0;
    }
    (steep_px / d_px.max(1.0e-30)).clamp(0.0, RELIEF_SLOPE_MAX)
}

/// The diffuse shade, 0 to 1, of a slope `k` whose direction is at cosine `dn` to the light's, the
/// light `h` up (`light_height`): (k·dn + h) / (k + h), 1 on a face turned to the light.
pub fn relief_diffuse(dn: f64, k: f64, h: f64) -> f64 {
    ((k * dn + h) / (k + h)).clamp(0.0, 1.0)
}

/// The highlight, 0 to 1: Blinn-Phong (N·H)^shininess on the surface of slope `k` along the unit
/// direction `n`, the light at `light_angle` radians and `h` up, the eye straight above.
pub fn relief_highlight(n: [f64; 2], k: f64, light_angle: f64, h: f64, shininess: f64) -> f64 {
    let norm = |v: [f64; 3]| {
        let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / m, v[1] / m, v[2] / m]
    };
    let l = norm([light_angle.cos(), light_angle.sin(), h]);
    let half = norm([l[0], l[1], l[2] + 1.0]);
    let nn = norm([n[0] * k, n[1] * k, 1.0]);
    let c = (nn[0] * half[0] + nn[1] * half[1] + nn[2] * half[2]).max(0.0);
    c.powf(shininess)
}

/// Where the highlight peaks, in output pixels from the set, on a slope facing the light: the
/// distance at which the slope from the distance tilts halfway toward the light, `steep_px` /
/// tan((90 degrees - elevation) / 2), the elevation atan(`h`).
pub fn relief_highlight_at(steep_px: f64, h: f64) -> f64 {
    let tilt = 0.5 * (std::f64::consts::FRAC_PI_2 - h.atan());
    steep_px / tilt.tan()
}

#[cfg(test)]
#[path = "../tests/unit/coloring/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "../tests/unit/coloring/equalize_tests.rs"]
mod equalize_tests;
#[cfg(test)]
#[path = "../tests/unit/coloring/transfer_tests.rs"]
mod transfer_tests;
#[cfg(test)]
#[path = "../tests/unit/coloring/edge_tests.rs"]
mod edge_tests;
#[cfg(test)]
#[path = "../tests/unit/coloring/grid_tests.rs"]
mod grid_tests;
#[cfg(test)]
#[path = "../tests/unit/coloring/relief_tests.rs"]
mod relief_tests;
#[cfg(test)]
#[path = "../tests/unit/coloring/layer_tests.rs"]
mod layer_tests;
