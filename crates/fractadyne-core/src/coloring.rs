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
    /// Decomposition is left as it is: its angle keeps BLA off, so no switch to or from it could
    /// recolor.
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
            Capability::GlitchCorrection | Capability::NormalizedExport | Capability::LiveWalk => stats.is_empty(),
            Capability::Strips => !self.reads_aux(),
            Capability::Series => !stats.blocks_iter_skip(),
            Capability::Bla => stats.bla_aggregate(),
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
    /// The live view's walk in passes (the chunked walk, band BLA): it carries no statistics.
    LiveWalk,
    /// Series approximation: skipping iterations would drop the statistics' terms.
    Series,
    /// BLA: its nodes fold every statistic but the final angle.
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

/// One orbit statistic's row of the table: something the iteration can accumulate into the aux
/// lane as it runs (`aux_step` in the shader).
#[derive(Debug)]
pub struct StatSpec {
    /// A name for logs and tests (never stored: a frame's set is derived, not saved).
    pub key: &'static str,
    /// The name of the shader constant that carries its bit (`STAT_STRIPE`).
    pub wgsl: &'static str,
    /// Skipping iterations would change it: a running statistic (stripe, triangle inequality, trap
    /// minimum) would drop the skipped terms, and the final angle is a discontinuous function of the
    /// exact final z (series approximation moved ~15% of decomposition's cell edges). Series
    /// approximation stays off while it is accumulated.
    pub blocks_iter_skip: bool,
    /// Each BLA node carries its aggregate (the node's fourth vec4, `[span, trap min, sum TIA,
    /// sum stripe]`), so BLA may skip with it accumulated past 1e28x. The final angle has none.
    pub bla_aggregate: bool,
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
    StatSpec { key: "stripe", wgsl: "STAT_STRIPE", blocks_iter_skip: true, bla_aggregate: true },
    StatSpec { key: "triangle", wgsl: "STAT_TRIANGLE", blocks_iter_skip: true, bla_aggregate: true },
    StatSpec { key: "trap", wgsl: "STAT_TRAP", blocks_iter_skip: true, bla_aggregate: true },
    StatSpec { key: "angle", wgsl: "STAT_ANGLE", blocks_iter_skip: true, bla_aggregate: false },
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
    /// statistic at all runs under the same rules with these added: it pays only their operations,
    /// and a switch between the methods that read them can recolor. The final angle is not in it:
    /// it has no BLA aggregate, so it would turn BLA off for the others (B8 measures whether it can
    /// join).
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
    /// BLA folds every statistic in the set ([`StatSpec::bla_aggregate`]); true for none.
    pub const fn bla_aggregate(self) -> bool {
        let mut i = 0;
        while i < Stat::ALL.len() {
            if self.has(Stat::ALL[i]) && !Stat::ALL[i].spec().bla_aggregate {
                return false;
            }
            i += 1;
        }
        true
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
}

const TRAPS: [TrapSpec; 3] = [
    TrapSpec { key: "point", wgsl: "TRAP_POINT" },
    TrapSpec { key: "cross", wgsl: "TRAP_CROSS" },
    TrapSpec { key: "circle", wgsl: "TRAP_CIRCLE" },
];

impl TrapType {
    /// Every shape, in the table's order.
    pub const ALL: [TrapType; 3] = [TrapType::Point, TrapType::Cross, TrapType::Circle];
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
