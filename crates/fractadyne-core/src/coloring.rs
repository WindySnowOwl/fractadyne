//! The coloring methods and the orbit-trap shapes: one table each, read by the app, the GPU crate
//! and the shader (design/coloring-plan.md A10).
//!
//! Outside the GPU a method or a trap is known by its **key**: session files, view files, the
//! farm's settings, the command line and every harness output store the key. Its **GPU number**
//! is its position in the table, and holds for one build only: the shader takes the numbers as
//! named constants generated from this table ([`wgsl_ids`]), and every Rust check asks the table
//! (`needs_aux`, `bla_aggregate`, ...), so no number is written down twice and the table can be
//! reordered or extended without breaking a saved file.

use std::fmt::Write as _;

/// One coloring method's row of the table.
#[derive(Debug)]
pub struct MethodSpec {
    /// The persisted key (session, view files, farm, `--method`, harness outputs).
    pub key: &'static str,
    /// The name of the shader constant that carries its GPU number (`METHOD_STRIPE`).
    pub wgsl: &'static str,
    /// It accumulates the aux statistics texture: stripe, triangle inequality, orbit trap,
    /// decomposition (the final angle). The shader computes the aux lane only for these, and every
    /// path that cannot carry the statistics (the chunked walk, the band's BLA, the corrector, the
    /// normalized export, zoom-video strips) is gated on it.
    pub needs_aux: bool,
    /// It reads the distance estimate, which only a family with a derivative has
    /// (`FormulaCaps::distance_estimate`); elsewhere it colors the whole view one color.
    pub needs_distance_estimate: bool,
    /// Its colored value lies in 0..1 (stripe, triangle inequality, decomposition), so the palette
    /// takes a few cycles across that range instead of the per-unit rate of the unbounded values
    /// (iteration count, orbit trap, distance).
    pub bounded_value: bool,
    /// Skipping iterations would change it: a running per-iteration statistic (stripe, triangle
    /// inequality, trap minimum) would drop the skipped terms, and decomposition is a discontinuous
    /// function of the exact final z (series approximation moved ~15% of its cell edges). Series
    /// approximation stays off for these.
    pub blocks_iter_skip: bool,
    /// Its statistic has an aggregate in each BLA node (`[span, trap min, sum TIA, sum stripe]`), so
    /// BLA may skip for it past 1e28x. Decomposition has none.
    pub bla_aggregate: bool,
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
        needs_aux: false,
        needs_distance_estimate: false,
        bounded_value: false,
        blocks_iter_skip: false,
        bla_aggregate: false,
    },
    MethodSpec {
        key: "stripe",
        wgsl: "METHOD_STRIPE",
        needs_aux: true,
        needs_distance_estimate: false,
        bounded_value: true,
        blocks_iter_skip: true,
        bla_aggregate: true,
    },
    MethodSpec {
        key: "triangle",
        wgsl: "METHOD_TRIANGLE",
        needs_aux: true,
        needs_distance_estimate: false,
        bounded_value: true,
        blocks_iter_skip: true,
        bla_aggregate: true,
    },
    MethodSpec {
        key: "trap",
        wgsl: "METHOD_TRAP",
        needs_aux: true,
        needs_distance_estimate: false,
        bounded_value: false,
        blocks_iter_skip: true,
        bla_aggregate: true,
    },
    MethodSpec {
        key: "distance",
        wgsl: "METHOD_DISTANCE",
        needs_aux: false,
        needs_distance_estimate: true,
        bounded_value: false,
        blocks_iter_skip: false,
        bla_aggregate: false,
    },
    MethodSpec {
        key: "decomposition",
        wgsl: "METHOD_DECOMPOSITION",
        needs_aux: true,
        needs_distance_estimate: false,
        bounded_value: true,
        blocks_iter_skip: true,
        bla_aggregate: false,
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
    /// See [`MethodSpec::needs_aux`].
    pub const fn needs_aux(self) -> bool {
        self.spec().needs_aux
    }
    /// See [`MethodSpec::needs_distance_estimate`].
    pub const fn needs_distance_estimate(self) -> bool {
        self.spec().needs_distance_estimate
    }
    /// See [`MethodSpec::bounded_value`].
    pub const fn bounded_value(self) -> bool {
        self.spec().bounded_value
    }
    /// See [`MethodSpec::blocks_iter_skip`].
    pub const fn blocks_iter_skip(self) -> bool {
        self.spec().blocks_iter_skip
    }
    /// See [`MethodSpec::bla_aggregate`].
    pub const fn bla_aggregate(self) -> bool {
        self.spec().bla_aggregate
    }
    /// Whether a family with these capabilities offers this method: the one rule the method box,
    /// the Color menu and the coloring matrix (`--coloring-matrix`) share. Only Distance asks for
    /// something a family can lack, the distance estimate; every other method is offered on every
    /// escape-time family.
    pub const fn offered_for(self, caps: crate::formula::FormulaCaps) -> bool {
        caps.distance_estimate || !self.needs_distance_estimate()
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
/// method and `const TRAP_CROSS: u32 = 1u;` for every trap shape. The GPU crate appends this to the
/// renderer's shader source; the shader compares `color_method` and `trap_type` only with these
/// names, never with a number.
pub fn wgsl_ids() -> String {
    let mut s = String::from(
        "\n// ---- Generated by fractadyne_core::coloring::wgsl_ids (design/coloring-plan.md A10) ----\n\
         // The coloring methods' and trap shapes' GPU numbers for this build. Compare\n\
         // `color_method` and `trap_type` with these names only, never with a number.\n",
    );
    for m in ColorMethod::ALL {
        let _ = writeln!(s, "const {}: u32 = {}u;", m.spec().wgsl, m.gpu_id());
    }
    for t in TrapType::ALL {
        let _ = writeln!(s, "const {}: u32 = {}u;", t.spec().wgsl, t.gpu_id());
    }
    s
}

#[cfg(test)]
#[path = "../tests/unit/coloring/tests.rs"]
mod tests;
