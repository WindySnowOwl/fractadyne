//! Core numerics for Fractadyne.
//!
//! The viewport center is **arbitrary precision** (`astro_float::BigFloat`) at a
//! mantissa size that scales with zoom, so position stays sub-pixel at *any* depth
//! (no coordinate jump, ever). `units_per_pixel` is a plain f64 scale. The reference
//! orbit is iterated in bignum and stored as `f32` hi/lo pairs (df64) for the GPU.
//!
//! Bignum is slow, so the reference orbit should be recomputed only when the
//! reference point changes (the app caches it), not every frame.
//!
//! ## Naming conventions (glossary)
//!
//! Short identifiers recur throughout this crate; they mean:
//! - `p` — working **precision** in mantissa bits for a `BigFloat` (scales with zoom depth).
//! - `bf` — a `BigFloat`, or a shorthand constructor from an f64.
//! - `(cx, cy)` — the complex parameter *c* (real, imaginary). `(zx, zy)` — the iterate *z*.
//! - `(dzx, dzy)` / `dc` — perturbation **deltas** (δz, δc) relative to the reference orbit.
//! - `FloatExp` (`m`,`e`) — an extended-range float `m·2^e` (see the `FloatExp` docs); `df64`/df32
//!   — an f32 hi+lo pair (double-single) carrying ~46 bits for the GPU.
//! - `RM` — `RoundingMode`; `SA` — series approximation; `BLA` — bivariate linear approximation.

pub use astro_float::BigFloat;

mod floatexp;
pub use floatexp::*;

mod bignum;

/// Exact byte serialization of a `BigFloat` — for the reference-orbit cache, where a coordinate
/// that comes back differing in its last bits is the orbit of a DIFFERENT point.
pub mod bfbytes;
pub use bignum::*;

mod viewport;
pub use viewport::*;

mod reference;
pub use reference::*;

mod fractal;

/// The formula intermediate representation and its interpreters (design/custom-formulas.md).
pub mod ir;

mod backend;
#[cfg(feature = "rug")]
mod backend_rug;
pub use backend::{
    available_backends, built_in_backends, mpfr_found_message, mpfr_missing_message,
    mpfr_restored_message, mpfr_runtime_available, observed_backends,
    parse_choice as parse_backend_choice, resolve_startup_backend,
    select as select_backend, selected as selected_backend, status_line as backend_status_line,
    BackendChoice, BACKEND_NAMES,
};

/// Canonical numeric ids for the escape-time families — the `u32 formula` argument threaded through
/// this crate's dispatch and uploaded to the shader. These are the single source of truth for the
/// numbering; the app's `FractalKind::formula_id` and the WGSL `fs_iterate` branches MUST agree.
///
/// # Adding a formula (core + shader side)
///
/// After adding the app-side row (see `fractadyne-app/src/fractal.rs`), give it an id here and
/// implement its iteration in every path it should support, all keyed on this id:
/// - [`step_bf`] — the bignum reference-orbit step (required for deep zoom).
/// - [`orbit_points`] — the f64 orbit overlay (required).
/// - [`series_skip`] — only for polynomial `z^d + c` families (see [`formula_power`]).
/// - [`formula_power`] — the escape power, if the family is a Multibrot-style `z^d + c`.
/// - `fractadyne-gpu/src/mandelbrot.wgsl` `fs_iterate` — one branch per active render mode.
/// - [`formula::caps`] — what else it supports (series approximation, BLA, resumable passes, the
///   finders, the export glitch-correction policy). Callers ask the capability, never the id.
///
/// An unknown id falls back to Mandelbrot in [`step_bf`]/[`orbit_points`] (a safe default, not an
/// error) — validate with [`is_valid_formula`] at UI/CLI boundaries if a hard reject is wanted.
pub mod formula {
    pub const MANDELBROT: u32 = 0;
    pub const MULTIBROT3: u32 = 1;
    pub const MULTIBROT4: u32 = 2;
    pub const MULTIBROT5: u32 = 3;
    pub const TRICORN: u32 = 4;
    pub const BURNING_SHIP: u32 = 5;
    pub const CELTIC: u32 = 6;
    pub const BUFFALO: u32 = 7;
    pub const PHOENIX: u32 = 8;
    pub const NEWTON: u32 = 9;
    /// The power and fold families (design/power-families.md): ids `FAMILY_FIRST..COUNT`, five
    /// shapes in [`Shape`] order, three powers each — see [`family`].
    pub const MULTIBROT6: u32 = 10;
    pub const MULTIBROT7: u32 = 11;
    pub const MULTIBROT8: u32 = 12;
    pub const BURNING_SHIP3: u32 = 13;
    pub const BURNING_SHIP4: u32 = 14;
    pub const BURNING_SHIP5: u32 = 15;
    pub const TRICORN3: u32 = 16;
    pub const TRICORN4: u32 = 17;
    pub const TRICORN5: u32 = 18;
    pub const CELTIC3: u32 = 19;
    pub const CELTIC4: u32 = 20;
    pub const CELTIC5: u32 = 21;
    pub const BUFFALO3: u32 = 22;
    pub const BUFFALO4: u32 = 23;
    pub const BUFFALO5: u32 = 24;
    /// The first power-family id.
    pub const FAMILY_FIRST: u32 = MULTIBROT6;
    /// Number of defined formula ids (ids are `0..COUNT`).
    pub const COUNT: u32 = 25;

    /// A power family's shape: where its fold sits relative to the power (design/power-families.md
    /// §1). The WGSL twin numbers them in this order (`fam_shape`).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Shape {
        /// `z^d + c`.
        Multibrot,
        /// `(|x| + i|y|)^d + c`: the fold before the power.
        BurningShip,
        /// `conj(z)^d + c`.
        Tricorn,
        /// `|Re w| + i·Im w + c`, `w = z^d`: the fold after.
        Celtic,
        /// `|Re w| + i·|Im w| + c`.
        Buffalo,
    }

    /// A power family's shape and power, for ids `FAMILY_FIRST..COUNT` (`None` for every other id,
    /// the older families included: their arms are their own).
    pub const fn family(formula: u32) -> Option<(Shape, u32)> {
        if formula < FAMILY_FIRST || formula >= COUNT {
            return None;
        }
        let k = formula - FAMILY_FIRST;
        let shape = match k / 3 {
            0 => Shape::Multibrot,
            1 => Shape::BurningShip,
            2 => Shape::Tricorn,
            3 => Shape::Celtic,
            _ => Shape::Buffalo,
        };
        // Multibrot runs on from 5 (6, 7, 8); the fold families from their power-2 originals.
        let d = if k < 3 { 6 + k } else { 3 + k % 3 };
        Some((shape, d))
    }
    /// The id a custom formula ([`crate::ir`]) renders under. Outside `0..COUNT`, so every
    /// built-in branch of the shader skips it and [`caps`] grants it none of the built-in
    /// capabilities; its step comes from its own generated shader module instead.
    pub const CUSTOM: u32 = 1000;

    /// The escape degree `d` (`|z'| ≈ |z|^d` far out): the smooth count's log base, as the
    /// shader's `power_f`. 2 for every family that is not a higher power (Phoenix and Newton
    /// included, as the shader has them).
    pub const fn power(formula: u32) -> u32 {
        match formula {
            MULTIBROT3 => 3,
            MULTIBROT4 => 4,
            MULTIBROT5 => 5,
            f => match family(f) {
                Some((_, d)) => d,
                None => 2,
            },
        }
    }

    /// What a formula supports beyond plain iteration (design/custom-formulas.md §4.3). These were
    /// id ranges written out at each use (`formula_id() <= 3`, `== 0`, `> 3`); a custom formula
    /// will compute its own from its definition, so every caller asks the capability instead.
    /// (Julia and perturbation support are still the app's `FractalSpec` flags.)
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct FormulaCaps {
        /// Series approximation can seed the perturbation (`series_skip`): the `z^d + c` families.
        pub series_approximation: bool,
        /// A BLA tree can be built for it (`bla_tree_gpu`): the `z^d + c` families (Multibrot 3–8
        /// since the power families' phase 3; Mandelbrot before) and, with a 2×2 tree since phase 4,
        /// the folds (Tricorn, Burning Ship, Celtic, Buffalo at every power).
        pub bla: bool,
        /// The resumable chunk shaders (`fs_iterate_chunk*`) implement it, so a live refresh or an
        /// export tile can be split on the iteration axis.
        pub resumable_passes: bool,
        /// The minibrot (nucleus) finder applies (`find_nucleus`, via `formula_power`).
        pub nucleus_finder: bool,
        /// The Misiurewicz explorer, feature go-to, snap-to-nucleus and the autopilot's
        /// Misiurewicz target: Mandelbrot only.
        pub feature_solvers: bool,
        /// Exports run multi-reference glitch correction (outside Julia mode): every family but the
        /// `z^d + c` ones, where the glitch audit found it repaired nothing.
        pub export_glitch_correction: bool,
        /// Convergent (root finding) rather than escape time: the orbit starts at the point.
        pub convergent: bool,
    }

    /// The built-in families' capabilities. An unknown id gets none of them, and glitch correction
    /// on — what each gate gave an out-of-range id when it was written as an id range. Except
    /// [`CUSTOM`]: glitch correction builds its extra references from the formula ID, which names
    /// no custom step (a custom export ran 52 references of the wrong orbit before this); and a
    /// custom formula's generated module carries its own resumable chunk pass (`custom.rs`), so
    /// it has `resumable_passes` — ⚠which every chunk pipeline must then build from THAT module.
    pub const fn caps(formula: u32) -> FormulaCaps {
        let polynomial = matches!(
            formula,
            MANDELBROT | MULTIBROT3 | MULTIBROT4 | MULTIBROT5 | MULTIBROT6 | MULTIBROT7 | MULTIBROT8
        );
        // The folds at every power: a real 2×2 BLA tree (`build_bla_fold`).
        let fold = matches!(formula, TRICORN | BURNING_SHIP | CELTIC | BUFFALO)
            || matches!(
                family(formula),
                Some((Shape::BurningShip | Shape::Tricorn | Shape::Celtic | Shape::Buffalo, _))
            );
        FormulaCaps {
            series_approximation: polynomial,
            bla: polynomial || fold,
            resumable_passes: polynomial || formula == CUSTOM,
            nucleus_finder: polynomial,
            feature_solvers: formula == MANDELBROT,
            export_glitch_correction: !polynomial && formula != CUSTOM,
            convergent: formula == NEWTON,
        }
    }
}

/// Whether `formula` is a defined id (`0..formula::COUNT`). Dispatch tolerates unknown ids by
/// falling back to Mandelbrot; callers wanting a hard reject (untrusted view files, CLI) use this.
pub fn is_valid_formula(formula: u32) -> bool {
    formula < formula::COUNT
}

#[cfg(test)]
mod tests;
