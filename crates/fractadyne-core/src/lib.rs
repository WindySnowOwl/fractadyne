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
pub use bignum::*;

mod viewport;
pub use viewport::*;

mod reference;
pub use reference::*;

mod fractal;

mod backend;
#[cfg(feature = "rug")]
mod backend_rug;
pub use backend::{
    available_backends, built_in_backends, observed_backends, parse_choice as parse_backend_choice,
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
    /// Number of defined formula ids (ids are `0..COUNT`).
    pub const COUNT: u32 = 10;
}

/// Whether `formula` is a defined id (`0..formula::COUNT`). Dispatch tolerates unknown ids by
/// falling back to Mandelbrot; callers wanting a hard reject (untrusted view files, CLI) use this.
pub fn is_valid_formula(formula: u32) -> bool {
    formula < formula::COUNT
}

#[cfg(test)]
mod tests;
