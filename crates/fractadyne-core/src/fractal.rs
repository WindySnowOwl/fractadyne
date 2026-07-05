//! Formula-step abstraction (REFACTOR-PLAN Phase 5 — narrow `Field`-generic-step PoC).
//!
//! A single `Field`-generic per-family `step<F>` serves BOTH the `f64` orbit overlay
//! ([`crate::orbit_points`]) and the `BigFloat` reference orbit ([`step_bf`](crate::step_bf)),
//! so a family's escape recurrence is authored **once** instead of hand-copied across the two
//! number types (where the copies could silently drift between the overlay and the deep render).
//!
//! **Scope (PoC):** the holomorphic `z^d + c` families (Mandelbrot, Multibrot 3/4/5) plus one abs
//! family (Burning Ship) — together they exercise `mul`/`add`/`sub` **and** `abs`. Tricorn /
//! Celtic / Buffalo / Phoenix / Newton stay on the enum path; [`trait_step`] returns `None` for
//! them so no numeric path is deleted before its trait replacement is proven bit-identical.
//!
//! **Bit-identity** (the whole point — guarded by the exact SA cross-check unit tests in
//! `lib.rs`, which iterate `step_bf` for Mandelbrot and Multibrot 3): the generic [`cmul`]
//! reproduces [`crate::reference::cmul_bf`]'s exact op order for `BigFloat`; and a `z²` imaginary
//! part computed as `x·y + y·x` equals both the old `double_bf(x·y)` (BigFloat) and `2·x·y` (f64)
//! to the bit — `a + a` doubles exactly at any precision (the sum's mantissa is unchanged, only
//! the exponent bumps), so no rounding is introduced.

use crate::bignum::RM;
use crate::formula;
use astro_float::BigFloat;

/// The number field a per-family [`Fractal::step`] is generic over: `f64` (the orbit overlay) and
/// arbitrary-precision `BigFloat` (the reference orbit). Method names are prefixed (`fmul`…) so
/// they don't collide with the inherent `BigFloat::mul` / `f64::abs`. `Ctx` carries the `BigFloat`
/// precision (the rounding mode is the fixed [`RM`]); it is `()` for `f64`.
pub(crate) trait Field: Clone {
    type Ctx: Copy;
    fn fmul(&self, o: &Self, ctx: Self::Ctx) -> Self;
    fn fadd(&self, o: &Self, ctx: Self::Ctx) -> Self;
    fn fsub(&self, o: &Self, ctx: Self::Ctx) -> Self;
    fn fabs(&self) -> Self;
}

impl Field for f64 {
    type Ctx = ();
    #[inline]
    fn fmul(&self, o: &f64, _: ()) -> f64 {
        self * o
    }
    #[inline]
    fn fadd(&self, o: &f64, _: ()) -> f64 {
        self + o
    }
    #[inline]
    fn fsub(&self, o: &f64, _: ()) -> f64 {
        self - o
    }
    #[inline]
    fn fabs(&self) -> f64 {
        (*self).abs()
    }
}

impl Field for BigFloat {
    type Ctx = usize; // precision `p`; the rounding mode is the fixed `RM`
    #[inline]
    fn fmul(&self, o: &BigFloat, p: usize) -> BigFloat {
        self.mul(o, p, RM)
    }
    #[inline]
    fn fadd(&self, o: &BigFloat, p: usize) -> BigFloat {
        self.add(o, p, RM)
    }
    #[inline]
    fn fsub(&self, o: &BigFloat, p: usize) -> BigFloat {
        self.sub(o, p, RM)
    }
    #[inline]
    fn fabs(&self) -> BigFloat {
        self.abs()
    }
}

/// Complex multiply `(ax + i·ay)·(bx + i·by)`, generic over the field. For `BigFloat` this is
/// exactly [`crate::reference::cmul_bf`]'s op order: `ax·bx − ay·by`, `ax·by + ay·bx` (each
/// product rounded, then combined).
#[inline]
fn cmul<F: Field>(ax: &F, ay: &F, bx: &F, by: &F, ctx: F::Ctx) -> (F, F) {
    let rx = ax.fmul(bx, ctx).fsub(&ay.fmul(by, ctx), ctx);
    let ry = ax.fmul(by, ctx).fadd(&ay.fmul(bx, ctx), ctx);
    (rx, ry)
}

/// One escape-time step `z → f(z) + c`, authored once per family and generic over the number
/// field. `z₀ = 0, c = point` for the Mandelbrot set; `z₀ = point, c = const` for Julia.
pub(crate) trait Fractal {
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F);
}

struct Mandelbrot;
struct Multibrot3;
struct Multibrot4;
struct Multibrot5;
struct BurningShip;
struct Tricorn;
struct Celtic;
struct Buffalo;

impl Fractal for Mandelbrot {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        (sx.fadd(cx, ctx), sy.fadd(cy, ctx))
    }
}
impl Fractal for Multibrot3 {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        let (rx, ry) = cmul(&sx, &sy, zx, zy, ctx); // z³ = z²·z
        (rx.fadd(cx, ctx), ry.fadd(cy, ctx))
    }
}
impl Fractal for Multibrot4 {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        let (rx, ry) = cmul(&sx, &sy, &sx, &sy, ctx); // z⁴ = (z²)²
        (rx.fadd(cx, ctx), ry.fadd(cy, ctx))
    }
}
impl Fractal for Multibrot5 {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        let (qx, qy) = cmul(&sx, &sy, &sx, &sy, ctx); // z⁴
        let (rx, ry) = cmul(&qx, &qy, zx, zy, ctx); // z⁵ = z⁴·z
        (rx.fadd(cx, ctx), ry.fadd(cy, ctx))
    }
}
impl Fractal for BurningShip {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        // (|Re z| + i|Im z|)² + c == (Re(z²) + cx, |Im(z²)| + cy): squaring drops the input signs,
        // so only the imaginary part needs the abs (Im(z²) = 2·x·y).
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        (sx.fadd(cx, ctx), sy.fabs().fadd(cy, ctx))
    }
}
impl Fractal for Tricorn {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        // conj(z)² + c = (Re(z²) + cx, −Im(z²) + cy). `cy − sy` (not `neg(sy) + cy`) so BigFloat
        // matches the old `cy.sub(&txy)` exactly; in f64 `cy − sy == −sy + cy` (add commutes).
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        (sx.fadd(cx, ctx), cy.fsub(&sy, ctx))
    }
}
impl Fractal for Celtic {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        // |Re(z²)| + cx, Im(z²) + cy — abs on the real part only.
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        (sx.fabs().fadd(cx, ctx), sy.fadd(cy, ctx))
    }
}
impl Fractal for Buffalo {
    #[inline]
    fn step<F: Field>(&self, zx: &F, zy: &F, cx: &F, cy: &F, ctx: F::Ctx) -> (F, F) {
        // |Re(z²)| + cx, |Im(z²)| + cy — abs on both parts.
        let (sx, sy) = cmul(zx, zy, zx, zy, ctx); // z²
        (sx.fabs().fadd(cx, ctx), sy.fabs().fadd(cy, ctx))
    }
}

/// Dispatch to a migrated family's generic step, or `None` for families still on the enum path.
/// PoC scope: `{Mandelbrot, Multibrot3/4/5, BurningShip}`. `None` keeps every other family's
/// numeric path exactly as it was (Tricorn / Celtic / Buffalo in the `match`, Phoenix / Newton in
/// their own branches).
#[inline]
pub(crate) fn trait_step<F: Field>(
    formula: u32,
    zx: &F,
    zy: &F,
    cx: &F,
    cy: &F,
    ctx: F::Ctx,
) -> Option<(F, F)> {
    Some(match formula {
        formula::MANDELBROT => Mandelbrot.step(zx, zy, cx, cy, ctx),
        formula::MULTIBROT3 => Multibrot3.step(zx, zy, cx, cy, ctx),
        formula::MULTIBROT4 => Multibrot4.step(zx, zy, cx, cy, ctx),
        formula::MULTIBROT5 => Multibrot5.step(zx, zy, cx, cy, ctx),
        formula::BURNING_SHIP => BurningShip.step(zx, zy, cx, cy, ctx),
        formula::TRICORN => Tricorn.step(zx, zy, cx, cy, ctx),
        formula::CELTIC => Celtic.step(zx, zy, cx, cy, ctx),
        formula::BUFFALO => Buffalo.step(zx, zy, cx, cy, ctx),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pre-refactor inline `f64` arms, kept here ONLY to cross-check the generic step against
    /// what `orbit_points` used to compute — a direct bit-identity tripwire for the migration
    /// (esp. Burning Ship + Multibrot 4/5, whose deep reference orbit has no golden of its own).
    fn old_f64(formula: u32, zx: f64, zy: f64, cx: f64, cy: f64) -> (f64, f64) {
        let (sx, sy) = (zx * zx - zy * zy, 2.0 * zx * zy);
        match formula {
            formula::MULTIBROT3 => (sx * zx - sy * zy + cx, sx * zy + sy * zx + cy),
            formula::MULTIBROT4 => (sx * sx - sy * sy + cx, 2.0 * sx * sy + cy),
            formula::MULTIBROT5 => {
                let (qx, qy) = (sx * sx - sy * sy, 2.0 * sx * sy);
                (qx * zx - qy * zy + cx, qx * zy + qy * zx + cy)
            }
            formula::BURNING_SHIP => (sx + cx, sy.abs() + cy),
            formula::TRICORN => (sx + cx, -sy + cy),
            formula::CELTIC => (sx.abs() + cx, sy + cy),
            formula::BUFFALO => (sx.abs() + cx, sy.abs() + cy),
            _ => (sx + cx, sy + cy), // Mandelbrot
        }
    }

    #[test]
    fn generic_step_matches_old_f64_bitwise() {
        let cases = [
            (0.30_f64, -0.70, -0.50, 0.10),
            (1.10, 0.20, 0.00, 0.00),
            (-0.90, 0.44, 0.20, -0.30),
            (0.001, -1.3, 0.7, 0.7),
        ];
        let families = [
            formula::MANDELBROT,
            formula::MULTIBROT3,
            formula::MULTIBROT4,
            formula::MULTIBROT5,
            formula::BURNING_SHIP,
            formula::TRICORN,
            formula::CELTIC,
            formula::BUFFALO,
        ];
        for &(zx, zy, cx, cy) in &cases {
            for f in families {
                let (gx, gy) = trait_step(f, &zx, &zy, &cx, &cy, ()).unwrap();
                let (ox, oy) = old_f64(f, zx, zy, cx, cy);
                assert_eq!(gx.to_bits(), ox.to_bits(), "formula {f} re @ ({zx},{zy})");
                assert_eq!(gy.to_bits(), oy.to_bits(), "formula {f} im @ ({zx},{zy})");
            }
        }
    }
}
