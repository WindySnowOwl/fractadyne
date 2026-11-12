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
    Decomposition,
}

/// The table, in [`ColorMethod`]'s order.
const METHODS: [MethodSpec; 6] = [
    MethodSpec {
        key: "smooth",
        wgsl: "METHOD_SMOOTH",
        reads: None,
        needs_distance_estimate: false,
        bounded_value: false,
    },
    MethodSpec {
        key: "stripe",
        wgsl: "METHOD_STRIPE",
        reads: Some(Stat::Stripe),
        needs_distance_estimate: false,
        bounded_value: true,
    },
    MethodSpec {
        key: "triangle",
        wgsl: "METHOD_TRIANGLE",
        reads: Some(Stat::Triangle),
        needs_distance_estimate: false,
        bounded_value: true,
    },
    MethodSpec {
        key: "trap",
        wgsl: "METHOD_TRAP",
        reads: Some(Stat::Trap),
        needs_distance_estimate: false,
        bounded_value: false,
    },
    MethodSpec {
        key: "distance",
        wgsl: "METHOD_DISTANCE",
        reads: None,
        needs_distance_estimate: true,
        bounded_value: false,
    },
    MethodSpec {
        key: "decomposition",
        wgsl: "METHOD_DECOMPOSITION",
        reads: Some(Stat::Angle),
        needs_distance_estimate: false,
        bounded_value: true,
    },
];

impl ColorMethod {
    /// Every method, in the table's order.
    pub const ALL: [ColorMethod; 6] = [
        ColorMethod::Smooth,
        ColorMethod::Stripe,
        ColorMethod::TriangleIneq,
        ColorMethod::OrbitTrap,
        ColorMethod::Distance,
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
    /// See [`MethodSpec::needs_distance_estimate`].
    pub const fn needs_distance_estimate(self) -> bool {
        self.spec().needs_distance_estimate
    }
    /// See [`MethodSpec::bounded_value`].
    pub const fn bounded_value(self) -> bool {
        self.spec().bounded_value
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
    s
}

#[cfg(test)]
#[path = "../tests/unit/coloring/tests.rs"]
mod tests;
