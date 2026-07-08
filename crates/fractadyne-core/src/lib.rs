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
mod tests {
    use super::*;

    // Utility (run: `cargo test -p fractadyne-core dump_deep_boundary_coords -- --ignored
    // --nocapture`): bisect a deep boundary point for each perturbation family — the deep-golden
    // coords hard-coded in selftest.rs. A point accurate to ~1e-39 sits on the boundary at every
    // scale, so one coord serves both the 1e6× and 1e30× goldens. When adding a formula, add a seed
    // row here and rerun to get its deep coordinate (see the checklist in fractal.rs).
    #[test]
    #[ignore]
    fn dump_deep_boundary_coords() {
        // (formula, name, exterior-direction from (0,0); scaled out until it escapes)
        let seeds: &[(u32, &str, f64, f64)] = &[
            (formula::MANDELBROT, "mandelbrot", -0.75, 0.25),
            (formula::MULTIBROT3, "multibrot3", 0.3, 1.0),
            (formula::MULTIBROT4, "multibrot4", 0.3, 1.0),
            (formula::MULTIBROT5, "multibrot5", 0.3, 1.0),
            (formula::TRICORN, "tricorn", 0.3, 1.0),
            (formula::BURNING_SHIP, "burning-ship", -1.0, -0.6),
            (formula::CELTIC, "celtic", -1.0, 0.3),
            (formula::BUFFALO, "buffalo", -0.8, -0.6),
            (formula::PHOENIX, "phoenix", 0.5, 0.8),
        ];
        let p = 160usize;
        let max_iter = 15000u32;
        let bounded = |f: u32, cx: &BigFloat, cy: &BigFloat| -> bool {
            let z0 = bf(0.0, p);
            let (_, len) = reference_orbit(&z0, &z0, cx, cy, f, max_iter, p);
            len > max_iter
        };
        for &(f, name, sx, sy) in seeds {
            let (mut ax, mut ay) = (bf(0.0, p), bf(0.0, p)); // interior: (0,0) is bounded for all
            let (mut bx, mut by) = (bf(sx, p), bf(sy, p));
            let onehalf = bf(1.5, p);
            let mut guard = 0;
            while bounded(f, &bx, &by) && guard < 100 {
                bx = bx.mul(&onehalf, p, RM);
                by = by.mul(&onehalf, p, RM);
                guard += 1;
            }
            for _ in 0..130 {
                let mx = lerp_bf(&ax, &bx, 0.5, p);
                let my = lerp_bf(&ay, &by, 0.5, p);
                if bounded(f, &mx, &my) {
                    ax = mx;
                    ay = my;
                } else {
                    bx = mx;
                    by = my;
                }
            }
            println!("DEEPCOORD {name} {} {}", to_decimal_string(&ax), to_decimal_string(&ay));
        }
    }

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} !≈ {b}");
    }

    fn octaves_for(exp_decimal: f64) -> u64 {
        (exp_decimal * std::f64::consts::LN_10 / std::f64::consts::LN_2).ceil() as u64
    }

    // The arbitrary-precision Phoenix reference orbit (z' = z² + c − 0.5·z_prev, two-term recurrence)
    // must reproduce the f64 direct orbit while both are bounded — validates the bignum step +
    // z_prev threading that the deep-zoom perturbation path relies on.
    #[test]
    fn phoenix_reference_matches_direct() {
        let p = 80;
        let c = (0.1_f64, 0.05_f64);
        let (cx, cy) = (BigFloat::from_f64(c.0, p), BigFloat::from_f64(c.1, p));
        let z0 = BigFloat::from_f64(0.0, p);
        let (orbit, _len) = reference_orbit(&z0, &z0, &cx, &cy, 8, 60, p);
        let direct = orbit_points((0.0, 0.0), c, 8, 60, 1.0e12);
        let mut compared = 0;
        for i in 0..orbit.len().min(direct.len()) {
            let (dx, dy) = direct[i];
            if dx * dx + dy * dy > 16.0 {
                break; // stop once it's escaping (large values make absolute compare meaningless)
            }
            let zx = orbit[i][0] as f64 + orbit[i][2] as f64; // df64 hi + lo
            let zy = orbit[i][1] as f64 + orbit[i][3] as f64;
            approx(zx, dx);
            approx(zy, dy);
            compared += 1;
        }
        assert!(compared >= 12, "only {compared} bounded Phoenix iterates compared");
    }

    // Extreme-depth arithmetic validation (feasible single-point form): at a magnification of
    // 1e1000× — far beyond f64 range — the arbitrary-precision z²+c iteration must be stable
    // under a precision increase (agree to ≈ p bits) and the coordinate must survive a decimal
    // round-trip. Fast (~3.4k-bit precision); the deeper 1e100000×/1e1000000× cases run via
    // `--validate-deep` (seconds–minutes) and the #[ignore]d test below.
    #[test]
    fn deep_precision_self_consistent_1e1000() {
        let p = precision_for_octaves(octaves_for(1_000.0));
        let agree = deep_consistency_bits(p, 256, 2_000);
        assert!(agree >= p as i64 - 128, "self-consistency {agree} of {p} bits");
        let rt = deep_roundtrip_bits(p);
        assert!(rt >= p as i64 - 256, "round-trip {rt} of {p} bits");
    }

    // Same check at 1e100000× (~332k-bit precision). Opt-in (slow, ~seconds):
    // `cargo test -p fractadyne-core -- --ignored deep_precision_self_consistent_1e100000`.
    #[test]
    #[ignore = "slow: ~332k-bit precision, runs in seconds"]
    fn deep_precision_self_consistent_1e100000() {
        let p = precision_for_octaves(octaves_for(100_000.0));
        let agree = deep_consistency_bits(p, 256, 400);
        assert!(agree >= p as i64 - 128, "self-consistency {agree} of {p} bits");
        let rt = deep_roundtrip_bits(p);
        assert!(rt >= p as i64 - 256, "round-trip {rt} of {p} bits");
    }

    // Two centers differing only at the ~34th significant digit must remain distinct
    // after parse_bf — i.e. parsing preserves deep digits (not truncated to f64).
    #[test]
    fn parse_bf_preserves_deep_digits() {
        let s1 = "-0.7436438870371587047521915061147707";
        let s2 = "-0.7436438870371587047521915061147000"; // differ ~digit 31
        let a = parse_bf(s1).unwrap();
        let b = parse_bf(s2).unwrap();
        let diff = to_f64(&a.sub(&b, 256, RM)).abs();
        assert!(
            diff > 1e-40,
            "parse_bf collapsed deep digits (diff={diff:e}) — bookmarks/exports would \
             lose the location at deep zoom"
        );
    }

    // Full round-trip: a deep center → decimal string → back must reproduce it to far
    // below any pixel size reachable at practical zoom. (`to_decimal_string` caps the
    // digit count, so it's not bit-exact, but the residual is ~1e-79 here — sub-pixel
    // even past ~1e60×, so bookmarks/exports restore the location, not a nearby zone.)
    #[test]
    fn center_string_roundtrip_subpixel() {
        let s = "-0.74364388703715870475219150611477078529733293886840544098878";
        let a = parse_bf(s).unwrap();
        let b = parse_bf(&to_decimal_string(&a)).unwrap();
        let diff = to_f64(&a.sub(&b, 512, RM)).abs();
        // 1e-50 ≈ one pixel at ~1e47× (1000-px wide); we expect far better (~1e-79).
        assert!(diff < 1e-50, "center string round-trip too lossy (diff={diff:e})");
    }

    #[test]
    fn to_f64_roundtrips_simple_values() {
        for &v in &[-0.5, 0.0, 1.0, -2.25, 0.131825, 1234.5] {
            approx(to_f64(&bf(v, 64)), v);
        }
    }

    #[test]
    fn center_maps_to_view_center() {
        let vp = Viewport::new(800.0, 600.0);
        let (cx, cy) = vp.pixel_to_complex(400.0, 300.0);
        approx(to_f64(&cx), to_f64(&vp.center_x));
        approx(to_f64(&cy), to_f64(&vp.center_y));
    }

    #[test]
    fn zoom_keeps_cursor_fixed() {
        let mut vp = Viewport::new(800.0, 600.0);
        let (px, py) = (123.0, 456.0);
        let before = vp.pixel_to_complex(px, py);
        let base_upp = vp.units_per_pixel;
        vp.zoom_at(px, py, 0.5);
        let after = vp.pixel_to_complex(px, py);
        approx(to_f64(&before.0), to_f64(&after.0));
        approx(to_f64(&before.1), to_f64(&after.1));
        assert!(vp.units_per_pixel.log2() < base_upp.log2());
    }

    // Box-zoom: a centered rectangle that is ¼ of the view in BOTH axes zooms uniformly to 4×,
    // and its center stays put (the constraining ratio is the same on each axis).
    #[test]
    fn zoom_to_rect_centers_and_scales() {
        let want = Viewport::new(800.0, 600.0).pixel_to_complex(400.0, 300.0);
        let base_upp = Viewport::new(800.0, 600.0).units_per_pixel.to_f64();
        let mut vp = Viewport::new(800.0, 600.0);
        vp.zoom_to_rect(300.0, 225.0, 500.0, 375.0); // ¼ w AND ¼ h, centered
        approx(to_f64(&vp.center_x), to_f64(&want.0));
        approx(to_f64(&vp.center_y), to_f64(&want.1));
        approx(vp.units_per_pixel.to_f64(), base_upp * 0.25);
        approx(vp.magnification(), 4.0);
    }

    // Box-zoom, general case: the rect's center becomes the view center, its larger *relative*
    // dimension exactly fills the view (the other fits within), and the drag direction is
    // irrelevant (opposite corners give the identical view).
    #[test]
    fn zoom_to_rect_fits_and_ignores_drag_direction() {
        let base_upp = Viewport::new(800.0, 600.0).units_per_pixel.to_f64();
        let (px0, py0, px1, py1) = (100.0, 120.0, 700.0, 270.0); // off-center, wider than tall
        let want = Viewport::new(800.0, 600.0).pixel_to_complex((px0 + px1) * 0.5, (py0 + py1) * 0.5);

        let mut vp = Viewport::new(800.0, 600.0);
        vp.zoom_to_rect(px0, py0, px1, py1);
        approx(to_f64(&vp.center_x), to_f64(&want.0));
        approx(to_f64(&vp.center_y), to_f64(&want.1));

        // Box extent (in the OLD units) relative to the NEW view extent: the constraining axis
        // fills it exactly (ratio 1), the other fits within (ratio ≤ 1).
        let new_upp = vp.units_per_pixel.to_f64();
        let ratio_w = (px1 - px0).abs() * base_upp / (vp.width_px * new_upp);
        let ratio_h = (py1 - py0).abs() * base_upp / (vp.height_px * new_upp);
        approx(ratio_w.max(ratio_h), 1.0);
        assert!(ratio_w.min(ratio_h) <= 1.0 + 1e-9, "box overflows the view: {ratio_w}, {ratio_h}");
        assert!(ratio_w > ratio_h, "width should be the constraining axis for this box");

        // Dragging the opposite corners must yield the identical view.
        let mut rev = Viewport::new(800.0, 600.0);
        rev.zoom_to_rect(px1, py1, px0, py0);
        approx(rev.units_per_pixel.to_f64(), new_upp);
        approx(to_f64(&rev.center_x), to_f64(&vp.center_x));
        approx(to_f64(&rev.center_y), to_f64(&vp.center_y));
    }

    #[test]
    fn pan_moves_center() {
        let mut vp = Viewport::new(800.0, 600.0);
        let upp = vp.units_per_pixel.to_f64();
        let (cx, cy) = vp.center_f64();
        vp.pan_pixels(10.0, -4.0);
        approx(to_f64(&vp.center_x), cx - 10.0 * upp);
        approx(to_f64(&vp.center_y), cy - 4.0 * upp);
    }

    #[test]
    fn default_magnification_is_one() {
        let vp = Viewport::new(1280.0, 720.0);
        approx(vp.magnification(), 1.0);
    }

    // The viewport scale must survive past f64's ~1e308× ceiling: precision scales up, the
    // magnitude stays exact (adjacent pixels map to distinct bignum coordinates rather than
    // collapsing to the center as an f64 `units_per_pixel` would), and the GPU scale params
    // stay well-formed (O(1) span mantissa + a deep shared exponent). This is what lets the
    // renderer reach the depths `--validate-deep` validates arithmetically.
    #[test]
    fn viewport_scale_survives_past_f64_range() {
        let mut vp = Viewport::new(1000.0, 1000.0);
        vp.set_center_log2mag(bf(-0.5, 2048), bf(0.0, 2048), 1100.0); // ≈ 1e331× (past f64)
        assert!(vp.precision > 1000, "precision did not scale: {}", vp.precision);
        assert!((vp.log2_magnification() - 1100.0).abs() < 2.0, "log2mag off: {}", vp.log2_magnification());
        // Adjacent pixels must differ (a plain-f64 upp would underflow to 0 → identical).
        let p = vp.precision;
        let (ax, _) = vp.pixel_to_complex(500.0, 500.0);
        let (bx, _) = vp.pixel_to_complex(501.0, 500.0);
        let diff = ax.sub(&bx, p, RM);
        assert!(diff.exponent().is_some(), "adjacent pixels collapsed past 1e308×");
        // GPU scale: O(1) span mantissa, deep shared exponent.
        let gs = vp.gpu_scale();
        assert!(
            gs.span_mantissa.x.abs() >= 1.0 && gs.span_mantissa.x.abs() < 4.0,
            "span mantissa not O(1): {}",
            gs.span_mantissa.x
        );
        assert!(gs.delta_exp < -1000, "delta_exp not deep: {}", gs.delta_exp);
        // The per-pixel offset, rescaled by 2^-delta_exp, is the O(1) mantissa the GPU needs.
        let off = ref_offset_mantissa(&ax, &bx, gs.delta_exp, p);
        assert!(off.is_finite() && off != 0.0, "ref-offset mantissa degenerate: {off}");
    }

    #[test]
    fn span_matches_extent() {
        let vp = Viewport::new(800.0, 600.0);
        let (sx, sy) = vp.complex_span();
        approx(sx, vp.width_px * vp.units_per_pixel.to_f64());
        approx(sy, vp.height_px * vp.units_per_pixel.to_f64());
    }

    #[test]
    fn recommended_iter_grows_with_zoom() {
        let mut vp = Viewport::new(800.0, 600.0);
        assert_eq!(vp.recommended_max_iter(256), 256);
        vp.zoom_at(400.0, 300.0, 1.0 / 1024.0);
        assert!(vp.recommended_max_iter(256) > 256);
    }

    #[test]
    fn precision_grows_with_zoom() {
        assert_eq!(precision_for_magnification(1.0), 64);
        assert!(precision_for_magnification(1e30) > precision_for_magnification(1e3));
    }

    // Series approximation: the order-3 polynomial A·δc + B·δc² + C·δc³ at the chosen skip
    // must reproduce the exact perturbation δz after that many iterations, for a worst-case
    // corner δc — i.e. skipping those iterations introduces negligible error.
    #[test]
    fn series_skip_matches_exact_perturbation() {
        let p = 160;
        let (cx, cy) = (bf(-0.745, p), bf(0.113, p)); // seahorse boundary (slow orbit)
        let max_dc = 1.0e-9_f64; // deep-ish δc, where SA actually applies
        let log2_max_dc = max_dc.log2();
        let max_iter = 5000;
        let s = series_skip(&cx, &cy, log2_max_dc, max_iter, max_iter, 0, p);
        assert!(s.skip >= 8, "no usable skip found (skip={})", s.skip);

        // Reconstruct the (f64) coefficients and evaluate the series at a worst-case δc.
        let cof = |m: [f32; 4], e: i32| -> (f64, f64) {
            let f = 2f64.powi(e);
            ((m[0] as f64 + m[1] as f64) * f, (m[2] as f64 + m[3] as f64) * f)
        };
        let (ar, ai) = cof(s.a, s.a_exp);
        let (br, bi) = cof(s.b, s.b_exp);
        let (cr, ci) = cof(s.c, s.c_exp);
        let cmul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
        let dc = (max_dc, 0.0); // worst-case real corner
        let dc2 = cmul(dc, dc);
        let dc3 = cmul(dc2, dc);
        let t1 = cmul((ar, ai), dc);
        let t2 = cmul((br, bi), dc2);
        let t3 = cmul((cr, ci), dc3);
        let series = (t1.0 + t2.0 + t3.0, t1.1 + t2.1 + t3.1);

        // Exact perturbation in bignum (same reference as series_skip): δz'=2Zδz+δz²+δc.
        let (dcx, dcy) = (bf(max_dc, p), bf(0.0, p));
        let two = bf(2.0, p);
        let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
        let (mut dzx, mut dzy) = (bf(0.0, p), bf(0.0, p));
        for _ in 0..s.skip {
            let (tzx, tzy) = cmul_bf(&zx, &zy, &dzx, &dzy, p); // Z·δz
            let (d2x, d2y) = cmul_bf(&dzx, &dzy, &dzx, &dzy, p);
            let ndzx = tzx.mul(&two, p, RM).add(&d2x, p, RM).add(&dcx, p, RM);
            let ndzy = tzy.mul(&two, p, RM).add(&d2y, p, RM).add(&dcy, p, RM);
            let (nzx, nzy) = step_bf(&zx, &zy, &cx, &cy, 0, p);
            zx = nzx;
            zy = nzy;
            dzx = ndzx;
            dzy = ndzy;
        }
        let (ex, ey) = (to_f64(&dzx), to_f64(&dzy));
        let err = ((series.0 - ex).powi(2) + (series.1 - ey).powi(2)).sqrt();
        let mag = (ex * ex + ey * ey).sqrt().max(1e-300);
        assert!(err / mag < 1.0e-3, "series vs exact δz rel err {:.2e} at skip {}", err / mag, s.skip);
    }

    // Same validation for the Multibrot-3 (z³+c) coefficient recurrence — confirms the
    // generalized order-3 series (A'=3Z²A+1, B'=3Z²B+3ZA², C'=3Z²C+6ZAB+A³) reproduces the
    // exact perturbation δz' = 3Z²δz + 3Zδz² + δz³ + δc.
    #[test]
    fn series_skip_matches_exact_multibrot3() {
        let p = 160;
        let (cx, cy) = (bf(0.2, p), bf(0.2, p)); // interior z³ point → long bounded orbit
        let max_dc = 1.0e-9_f64;
        let s = series_skip(&cx, &cy, max_dc.log2(), 5000, 5000, 1, p); // formula 1 = z³
        assert!(s.skip >= 8, "no usable skip found (skip={})", s.skip);

        let cof = |m: [f32; 4], e: i32| -> (f64, f64) {
            let f = 2f64.powi(e);
            ((m[0] as f64 + m[1] as f64) * f, (m[2] as f64 + m[3] as f64) * f)
        };
        let (ar, ai) = cof(s.a, s.a_exp);
        let (br, bi) = cof(s.b, s.b_exp);
        let (cr, ci) = cof(s.c, s.c_exp);
        let cmul = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
        let dc = (max_dc, 0.0);
        let dc2 = cmul(dc, dc);
        let dc3 = cmul(dc2, dc);
        let t1 = cmul((ar, ai), dc);
        let t2 = cmul((br, bi), dc2);
        let t3 = cmul((cr, ci), dc3);
        let series = (t1.0 + t2.0 + t3.0, t1.1 + t2.1 + t3.1);

        // Exact z³ perturbation in bignum, same reference: δz' = 3Z²δz + 3Zδz² + δz³ + δc.
        let (dcx, dcy) = (bf(max_dc, p), bf(0.0, p));
        let three = bf(3.0, p);
        let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
        let (mut dzx, mut dzy) = (bf(0.0, p), bf(0.0, p));
        for _ in 0..s.skip {
            let (z2x, z2y) = cmul_bf(&zx, &zy, &zx, &zy, p); // Z²
            let (zdx, zdy) = cmul_bf(&z2x, &z2y, &dzx, &dzy, p); // Z²δz
            let (d2x, d2y) = cmul_bf(&dzx, &dzy, &dzx, &dzy, p); // δz²
            let (zd2x, zd2y) = cmul_bf(&zx, &zy, &d2x, &d2y, p); // Zδz²
            let (d3x, d3y) = cmul_bf(&d2x, &d2y, &dzx, &dzy, p); // δz³
            let ndzx = zdx
                .mul(&three, p, RM)
                .add(&zd2x.mul(&three, p, RM), p, RM)
                .add(&d3x, p, RM)
                .add(&dcx, p, RM);
            let ndzy = zdy
                .mul(&three, p, RM)
                .add(&zd2y.mul(&three, p, RM), p, RM)
                .add(&d3y, p, RM)
                .add(&dcy, p, RM);
            dzx = ndzx;
            dzy = ndzy;
            let (nzx, nzy) = step_bf(&zx, &zy, &cx, &cy, 1, p); // Z³ + c
            zx = nzx;
            zy = nzy;
        }
        let (ex, ey) = (to_f64(&dzx), to_f64(&dzy));
        let err = ((series.0 - ex).powi(2) + (series.1 - ey).powi(2)).sqrt();
        let mag = (ex * ex + ey * ey).sqrt().max(1e-300);
        assert!(err / mag < 1.0e-3, "z³ series vs exact rel err {:.2e} at skip {}", err / mag, s.skip);
    }

    // BLA: a tree traversal must reproduce the exact (full-step) perturbation while skipping
    // most iterations. Interior reference (main cardioid) → bounded orbit, |δz| stays tiny.
    #[test]
    fn bla_reproduces_exact_perturbation() {
        let p = 96;
        let (cx, cy) = (bf(-0.5, p), bf(0.0, p)); // main-cardioid interior, never escapes
        let target: u32 = 2000;
        let (orbit, len) = reference_orbit(&bf(0.0, p), &bf(0.0, p), &cx, &cy, 0, target, p);
        assert!(len >= target, "reference escaped early (len={len})");
        let dc = (1.0e-9_f64, 0.0_f64); // worst-case corner δc
        let dc_max = FloatExp::from_f64((dc.0 * dc.0 + dc.1 * dc.1).sqrt());
        let levels = build_bla_mandel(&orbit, dc_max, 1.0e-6, AuxAggParams::default());
        assert!(!levels.is_empty());

        // BLA traversal: skip with the highest valid level, else a full perturbation step.
        let dc_c = CFloatExp { re: FloatExp::from_f64(dc.0), im: FloatExp::from_f64(dc.1) };
        let mut dz = CFloatExp { re: FloatExp::ZERO, im: FloatExp::ZERO };
        let (mut m, mut ops) = (0u32, 0u32);
        while m < target {
            ops += 1;
            let dzmag = dz.abs();
            let mut used = false;
            for l in (0..levels.len()).rev() {
                let step = 1u32 << l;
                if (m & (step - 1)) != 0 {
                    continue; // not aligned to 2^l
                }
                let j = (m >> l) as usize;
                let Some(&node) = levels[l].get(j) else { continue };
                if m + node.span > target || !dzmag.lt(node.r) {
                    continue;
                }
                dz = node.a * dz + node.b * dc_c; // δz = A·δz + B·δc
                m += node.span;
                used = true;
                break;
            }
            if !used {
                let zr = orbit[m as usize][0] as f64 + orbit[m as usize][2] as f64;
                let zi = orbit[m as usize][1] as f64 + orbit[m as usize][3] as f64;
                let z = CFloatExp { re: FloatExp::from_f64(2.0 * zr), im: FloatExp::from_f64(2.0 * zi) };
                dz = z * dz + dz * dz + dc_c; // δz' = 2Zδz + δz² + δc
                m += 1;
            }
        }
        let (bx, by) = (dz.re.to_f64(), dz.im.to_f64());

        // Exact perturbation (full steps, f64 — δz stays ~1e-9 for this interior reference).
        let (mut ex, mut ey) = (0.0f64, 0.0f64);
        for z in orbit.iter().take(target as usize) {
            let (zr, zi) = (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64);
            let (nzx, nzy) = (
                2.0 * zr * ex - 2.0 * zi * ey + (ex * ex - ey * ey) + dc.0,
                2.0 * zr * ey + 2.0 * zi * ex + 2.0 * ex * ey + dc.1,
            );
            ex = nzx;
            ey = nzy;
        }

        let err = ((bx - ex).powi(2) + (by - ey).powi(2)).sqrt();
        let mag = (ex * ex + ey * ey).sqrt().max(1e-300);
        assert!(err / mag < 1.0e-3, "BLA vs exact rel err {:.2e} (ops={ops})", err / mag);
        assert!(ops < target / 4, "BLA didn't skip enough (ops={ops} of {target})");
    }

    // BLA end-to-end: the reference BLA render (with escape-overshoot handling) must agree
    // with a naive full-step perturbation on the escape iteration — for BLA-engaged pixels
    // (tiny δc, escape late after big skips) AND fast escapers (large δc). This validates the
    // revert-on-overshoot logic the GPU shader will mirror.
    #[test]
    fn bla_matches_naive_including_escapes() {
        let p = 96;
        // Seahorse-ish boundary reference: a long orbit with a dwell gradient nearby.
        let (cx, cy) = (
            parse_bf("-0.743643887037158704752191506114774").unwrap(),
            parse_bf("0.131825904205311970493132056385139").unwrap(),
        );
        let max_iter: u32 = 5000;
        let (orbit, _len) = reference_orbit(&bf(0.0, p), &bf(0.0, p), &cx, &cy, 0, max_iter, p);
        let bail2 = 65536.0_f64; // 256², matching the app's smooth bailout

        // Naive full-step perturbation escape count (same smooth formula as bla_iterate).
        let naive = |dc: (f64, f64)| -> Option<f64> {
            let nstep = orbit.len() - 1;
            let (mut ex, mut ey) = (0.0f64, 0.0f64);
            for m in 0..(max_iter as usize).min(nstep) {
                let z = orbit[m];
                let (zr, zi) = (z[0] as f64 + z[2] as f64, z[1] as f64 + z[3] as f64);
                let (nex, ney) = (
                    2.0 * zr * ex - 2.0 * zi * ey + (ex * ex - ey * ey) + dc.0,
                    2.0 * zr * ey + 2.0 * zi * ex + 2.0 * ex * ey + dc.1,
                );
                ex = nex;
                ey = ney;
                let zn = orbit[m + 1];
                let (zx, zy) = (zn[0] as f64 + zn[2] as f64 + ex, zn[1] as f64 + zn[3] as f64 + ey);
                let mag2 = zx * zx + zy * zy;
                if mag2 > bail2 {
                    let nu = (mag2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                    return Some((m + 1) as f64 + 1.0 - nu);
                }
            }
            None
        };

        let check = |dc_max: f64, dcs: &[(f64, f64)]| {
            let levels = build_bla_mandel(&orbit, FloatExp::from_f64(dc_max), 1.0e-6, AuxAggParams::default());
            for &dc in dcs {
                let b = bla_iterate(&orbit, &levels, dc, bail2, max_iter);
                let n = naive(dc);
                match (b, n) {
                    (None, None) => {}
                    (Some(bv), Some(nv)) => assert!(
                        (bv - nv).abs() < 0.5,
                        "BLA {bv} vs naive {nv} at δc={dc:?} (dc_max={dc_max})"
                    ),
                    _ => panic!("BLA/naive disagree on escape at δc={dc:?}: bla={b:?} naive={n:?}"),
                }
            }
        };

        // BLA-engaged: tiny δc near the boundary (mix of late-escaping and bounded).
        check(1.5e-3, &[(0.0, 0.0), (1.0e-3, 0.0), (-1.0e-3, 5.0e-4), (8.0e-4, -9.0e-4), (1.5e-3, 1.5e-3)]);
        // Fast escapers: large δc leave the set quickly (BLA can't engage — full steps).
        check(0.8, &[(0.2, 0.1), (0.4, -0.2), (0.5, 0.3), (-0.6, 0.0)]);
    }

    #[test]
    fn reference_orbit_starts_at_z0() {
        // Mandelbrot mode: Z0 = 0, c = reference point.
        let (orbit, len) =
            reference_orbit(&bf(0.0, 64), &bf(0.0, 64), &bf(-0.5, 64), &bf(0.0, 64), 0, 64, 64);
        assert_eq!(orbit[0], [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(len as usize, orbit.len());
        assert!(len >= 2);
    }

    #[test]
    fn best_reference_prefers_interior_center() {
        let r = best_reference(
            &[bf(-0.5, 64), bf(0.0, 64)],
            [FloatExp::from_f64(0.1), FloatExp::from_f64(0.1)],
            0,
            false,
            [0.0, 0.0],
            500,
            64,
        );
        assert_eq!(
            reference_orbit(&bf(0.0, 64), &bf(0.0, 64), &r[0], &r[1], 0, 500, 64).1,
            501
        );
    }

    // ---- multi-reference glitch correction ----

    /// Exact f64 direct escape smooth-iter (ground truth at shallow zoom, where f64 is exact).
    fn oracle_smooth_f64(cx: f64, cy: f64, max_iter: u32) -> f64 {
        let (mut zx, mut zy) = (0.0f64, 0.0f64);
        for n in 0..max_iter {
            let (nx, ny) = (zx * zx - zy * zy + cx, 2.0 * zx * zy + cy);
            zx = nx;
            zy = ny;
            let m2 = zx * zx + zy * zy;
            if m2 > 256.0 * 256.0 {
                let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                return (n + 1) as f64 + 1.0 - nu;
            }
        }
        f64::NAN
    }

    /// Exact bignum direct escape smooth-iter — ground truth at any depth.
    fn oracle_smooth_bf(cx: &BigFloat, cy: &BigFloat, max_iter: u32, p: usize) -> f64 {
        let (mut zx, mut zy) = (bf(0.0, p), bf(0.0, p));
        for n in 0..max_iter {
            let (nx, ny) = step_bf(&zx, &zy, cx, cy, 0, p);
            zx = nx;
            zy = ny;
            let (xv, yv) = (to_f64(&zx), to_f64(&zy));
            let m2 = xv * xv + yv * yv;
            if m2 > 256.0 * 256.0 {
                let nu = (m2.ln() * 0.5 / std::f64::consts::LN_2).ln() / std::f64::consts::LN_2;
                return (n + 1) as f64 + 1.0 - nu;
            }
        }
        f64::NAN
    }

    /// Two smooth values agree, tolerating the interior-vs-late-escape ambiguity right at the
    /// boundary (a pixel that escapes at ~max_iter and one flagged interior are indistinguishable).
    fn smooth_agrees(a: f64, b: f64, max_iter: u32) -> bool {
        let near_max = |v: f64| v.is_nan() || v > max_iter as f64 - 5.0;
        if a.is_nan() && b.is_nan() {
            return true;
        }
        if near_max(a) && near_max(b) {
            return true;
        }
        (a - b).abs() < 1.0
    }

    // A genuine glitch: a small view around a real minibrot. The nucleus's critical orbit dips
    // to ~0 every `period` iterations; an off-nucleus reference does NOT dip as low, so near-
    // nucleus pixels satisfy the Pauldelbrot criterion (|z| ≪ |Z| while δz stays small — a glitch
    // rebasing cannot pre-empt). Multi-reference must detect them, add a reference inside the
    // glitched region, converge, and HEAL the image back to what an accurate reference produces.
    // Compared against a nucleus-seeded render (not a bignum oracle) so it's exact per pixel: both
    // use identical f64 math, and the small δc keeps every pixel a genuine, accurate perturbation.
    #[test]
    fn multi_reference_resolves_glitches() {
        // The large period-3 island on the real axis: low period ⇒ few iterations, so the f32-δz
        // *cancellation* glitch (fixable by a closer reference) dominates over mere accumulation.
        let nuc = find_nucleus(&[bf(-1.7548, 96), bf(0.0, 96)], 1.0e3, 0, 100)
            .expect("expected the period-3 minibrot");
        let p = 96;
        let (cx, cy) = (nuc.cx.clone(), nuc.cy.clone());
        let (w, h) = (24usize, 24usize);
        let upp = 0.010 / w as f64; // a view spanning the island + its glitch halo
        let max_iter = (nuc.period * 120).clamp(360, 4000);
        let tol = 1.0e-3;

        // Ground truth: per-pixel bignum direct iteration.
        let (cx0, cy0) = (to_f64(&cx), to_f64(&cy));
        let oracle: Vec<f64> = (0..w * h)
            .map(|i| {
                let (px, py) = (i % w, i / w);
                let pcx = bf(cx0 + (px as f64 - w as f64 * 0.5) * upp, p);
                let pcy = bf(cy0 - (py as f64 - h as f64 * 0.5) * upp, p);
                oracle_smooth_bf(&pcx, &pcy, max_iter, p)
            })
            .collect();

        // Off-nucleus corner reference, WITHOUT correction (single reference only, max_refs=1).
        let single = render_multiref_mandel(&cx, &cy, upp, w, h, max_iter, tol, (0, 0), 1, p);
        // Same corner seed, WITH multi-reference correction.
        let multi = render_multiref_mandel(&cx, &cy, upp, w, h, max_iter, tol, (0, 0), 60, p);

        let count_bad = |v: &[f64]| -> usize {
            (0..w * h).filter(|&i| !smooth_agrees(v[i], oracle[i], max_iter)).count()
        };
        let (s_bad, m_bad) = (count_bad(&single.smooth), count_bad(&multi.smooth));
        eprintln!(
            "period {} max_iter {max_iter} | single-ref wrong {s_bad}, multi-ref wrong {m_bad}, refs {}, glitch0 {}",
            nuc.period, multi.refs_used, multi.glitched_pass0
        );
        // The off-nucleus reference flags glitches; the correction adds references and converges.
        assert!(multi.glitched_pass0 > 0, "off-nucleus seed should induce glitches");
        assert!(multi.refs_used >= 2, "expected ≥2 references, used {}", multi.refs_used);
        assert_eq!(multi.unresolved, 0, "{} glitches unresolved", multi.unresolved);
        // The corrected image matches the bignum ground truth for every pixel, and never worse
        // than single-reference. (The precision gap that makes correction *necessary* — df64
        // reference vs df32 δz — lives on the GPU; this validates the algorithm is correct.)
        assert_eq!(m_bad, 0, "corrected result must match the oracle (wrong: {m_bad})");
        assert!(m_bad <= s_bad, "correction must not increase error (single {s_bad}, multi {m_bad})");
    }

    // Sanity on the perturbation math itself: a good central reference reproduces ground truth
    // (and multi-reference cleans up any far pixels it can't serve directly).
    #[test]
    fn perturbation_matches_direct() {
        let p = 64;
        let (cx, cy) = (bf(-0.5, p), bf(0.0, p));
        let (w, h) = (16usize, 16usize);
        let upp = 2.0 / w as f64;
        let max_iter = 800u32;
        let (cx0, cy0) = (to_f64(&cx), to_f64(&cy));
        let res = render_multiref_mandel(&cx, &cy, upp, w, h, max_iter, 1.0e-3, (w / 2, h / 2), 24, p);
        assert_eq!(res.unresolved, 0);
        let mut bad = 0;
        let mut worst = 0.0f64;
        for py in 0..h {
            for px in 0..w {
                let o = oracle_smooth_f64(
                    cx0 + (px as f64 - w as f64 * 0.5) * upp,
                    cy0 - (py as f64 - h as f64 * 0.5) * upp,
                    max_iter,
                );
                let g = res.smooth[py * w + px];
                if !smooth_agrees(g, o, max_iter) {
                    bad += 1;
                    if g.is_finite() && o.is_finite() {
                        worst = worst.max((g - o).abs());
                    }
                    eprintln!("mismatch ({px},{py}): got {g} oracle {o}");
                }
            }
        }
        assert_eq!(bad, 0, "{bad} pixels disagree with ground truth (worst Δ {worst})");
    }

    // The period-2 disk's nucleus is exactly c = -1. Starting near it, the finder must
    // report period 2 and Newton-snap to (-1, 0).
    #[test]
    fn find_nucleus_period2_disk() {
        let n = find_nucleus(&[bf(-1.01, 80), bf(0.012, 80)], 50.0, 0, 2000).unwrap();
        assert_eq!(n.period, 2);
        approx(to_f64(&n.cx), -1.0);
        approx(to_f64(&n.cy), 0.0);
    }

    // The period-3 bulb's nucleus is c ≈ -0.122561 + 0.744862 i.
    #[test]
    fn find_nucleus_period3_bulb() {
        let n = find_nucleus(&[bf(-0.12, 80), bf(0.74, 80)], 80.0, 0, 4000).unwrap();
        assert_eq!(n.period, 3);
        assert!((to_f64(&n.cx) - (-0.122561)).abs() < 1e-5, "cx={}", to_f64(&n.cx));
        assert!((to_f64(&n.cy) - 0.744862).abs() < 1e-5, "cy={}", to_f64(&n.cy));
    }

    // ---- analytic ground-truth validation (no external data; exact mathematics) ----

    /// Plain-f64 Mandelbrot escape-time dwell (test ground truth). `None` = interior
    /// (did not escape within `max`).
    fn dwell(cx: f64, cy: f64, max: u32) -> Option<u32> {
        let (mut zx, mut zy) = (0.0_f64, 0.0_f64);
        for i in 0..max {
            let (x2, y2) = (zx * zx, zy * zy);
            if x2 + y2 > 4.0 {
                return Some(i);
            }
            zy = 2.0 * zx * zy + cy;
            zx = x2 - y2 + cx;
        }
        None
    }

    /// Exact membership of the main cardioid: a point is inside iff
    /// `q·(q + (x − ¼)) < ¼·y²` with `q = (x − ¼)² + y²`.
    fn in_main_cardioid(x: f64, y: f64) -> bool {
        let xm = x - 0.25;
        let q = xm * xm + y * y;
        q * (q + xm) < 0.25 * y * y
    }

    /// Exact membership of the period-2 bulb: the disc of radius ¼ around c = −1.
    fn in_period2_bulb(x: f64, y: f64) -> bool {
        (x + 1.0) * (x + 1.0) + y * y < 0.0625
    }

    // Points proven interior by the closed-form cardioid/bulb tests must never escape;
    // points well outside the set must escape quickly. Validates the escape iteration.
    #[test]
    fn interior_closed_form_never_escapes() {
        let interior = [(-0.5, 0.0), (0.0, 0.0), (-0.1, 0.1), (-1.0, 0.0), (-0.9, 0.1)];
        for (x, y) in interior {
            assert!(in_main_cardioid(x, y) || in_period2_bulb(x, y), "({x},{y}) not classified interior");
            assert_eq!(dwell(x, y, 20_000), None, "interior point ({x},{y}) escaped");
        }
        let exterior = [(2.0, 0.0), (0.4, 0.4), (-0.8, 0.4), (0.3, 0.6)];
        for (x, y) in exterior {
            assert!(dwell(x, y, 20_000).is_some(), "exterior point ({x},{y}) did not escape");
        }
    }

    // The set is symmetric about the real axis: dwell(x, y) == dwell(x, −y).
    #[test]
    fn dwell_symmetric_about_real_axis() {
        for &(x, y) in &[(0.3, 0.5), (-0.75, 0.13), (-0.1, 0.9), (0.28, 0.012)] {
            assert_eq!(dwell(x, y, 5_000), dwell(x, -y, 5_000), "asymmetry at ({x},{y})");
        }
    }

    // A table of exact hyperbolic-component nuclei: the finder must recover both the
    // period and the coordinates. These are mathematical constants (Munafo mu-ency / KF).
    #[test]
    fn known_nuclei_table() {
        // (period, cx, cy, search-start offset)
        let table: &[(u32, f64, f64)] = &[
            (2, -1.0, 0.0),
            (4, -1.3107026413368328, 0.0),
            (3, -1.7548776662466927, 0.0),
            (3, -0.1225611668766536, 0.7448617666197446),
        ];
        for &(period, cx, cy) in table {
            // Start slightly off the exact nucleus so Newton has to converge to it.
            let start = [bf(cx + 1.0e-3, 96), bf(cy + 1.0e-3, 96)];
            let n = find_nucleus(&start, 200.0, 0, 8000)
                .unwrap_or_else(|| panic!("no nucleus found near period-{period} ({cx},{cy})"));
            assert_eq!(n.period, period, "wrong period for ({cx},{cy})");
            assert!((to_f64(&n.cx) - cx).abs() < 1e-9, "cx off: {} vs {cx}", to_f64(&n.cx));
            assert!((to_f64(&n.cy) - cy).abs() < 1e-9, "cy off: {} vs {cy}", to_f64(&n.cy));
        }
    }

    // Misiurewicz points are pre-periodic: the critical orbit reaches a repelling cycle
    // after a transient. c = −2 lands on the fixed point 2; c = i reaches a 2-cycle.
    #[test]
    fn misiurewicz_preperiodic() {
        // c = −2: 0 → −2 → 2 → 2 → … (fixed at 2 from iterate 2 on).
        let o = orbit_points((0.0, 0.0), (-2.0, 0.0), 0, 30, 1.0e6);
        assert!((o[2].0 - 2.0).abs() < 1e-9 && o[2].1.abs() < 1e-9);
        assert!((o[10].0 - 2.0).abs() < 1e-6, "c=-2 not fixed at 2: {:?}", o[10]);
        // c = i: 0 → i → (−1+i) → −i → (−1+i) → −i → … (pre-period 1, period 2).
        let o = orbit_points((0.0, 0.0), (0.0, 1.0), 0, 40, 1.0e6);
        let a = o[20];
        let b = o[22];
        assert!((a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6, "c=i not 2-periodic: {a:?} {b:?}");
        let mid = o[21];
        assert!((a.0 - mid.0).abs() + (a.1 - mid.1).abs() > 0.5, "c=i collapsed to a fixed point");
    }

    // ---- family symmetries (Phase 2.1; §9 claims verified here, then encoded) ----

    /// Escape-time dwell for any escape-time family (z₀ = 0, c = point), mirroring the
    /// shader's direct iteration. Bailout `|z| > 2` (classification is bailout-independent).
    fn dwell_family(formula: u32, cx: f64, cy: f64, max: u32) -> Option<u32> {
        let (mut x, mut y) = (0.0_f64, 0.0_f64);
        for i in 1..=max {
            let (nx, ny) = match formula {
                1 => (x * x * x - 3.0 * x * y * y + cx, 3.0 * x * x * y - y * y * y + cy), // z³
                2 => {
                    let (ax, ay) = (x * x - y * y, 2.0 * x * y); // z²
                    (ax * ax - ay * ay + cx, 2.0 * ax * ay + cy) // (z²)²
                }
                3 => {
                    let (ax, ay) = (x * x - y * y, 2.0 * x * y);
                    let (bx, by) = (ax * ax - ay * ay, 2.0 * ax * ay); // z⁴
                    (bx * x - by * y + cx, bx * y + by * x + cy) // z⁴·z
                }
                4 => (x * x - y * y + cx, -2.0 * x * y + cy),               // Tricorn conj(z)²
                5 => (x * x - y * y + cx, 2.0 * (x * y).abs() + cy),         // Burning Ship
                6 => ((x * x - y * y).abs() + cx, 2.0 * x * y + cy),         // Celtic
                7 => ((x * x - y * y).abs() + cx, (2.0 * x * y).abs() + cy), // Buffalo
                _ => (x * x - y * y + cx, 2.0 * x * y + cy),                 // Mandelbrot
            };
            x = nx;
            y = ny;
            if x * x + y * y > 4.0 {
                return Some(i);
            }
        }
        None
    }

    fn rot(cx: f64, cy: f64, deg: f64) -> (f64, f64) {
        let t = deg * std::f64::consts::PI / 180.0;
        (cx * t.cos() - cy * t.sin(), cx * t.sin() + cy * t.cos())
    }

    /// Count dwell mismatches over a grid under a coordinate transform.
    fn mismatches(formula: u32, xform: &dyn Fn(f64, f64) -> (f64, f64)) -> (u32, u32) {
        let (mut total, mut bad) = (0u32, 0u32);
        let mut iy = -30i32;
        while iy <= 30 {
            let mut ix = -30i32;
            while ix <= 30 {
                let (cx, cy) = (ix as f64 * 0.06, iy as f64 * 0.06);
                let (tx, ty) = xform(cx, cy);
                total += 1;
                if dwell_family(formula, cx, cy, 600) != dwell_family(formula, tx, ty, 600) {
                    bad += 1;
                }
                ix += 1;
            }
            iy += 1;
        }
        (total, bad)
    }

    // Multibrot z^d+c has (d−1)-fold rotational symmetry in c; the exact rotations (180°,
    // 90°) negate/swap coordinates in float, so dwell must match *exactly*. (§9: confirmed.)
    #[test]
    fn multibrot_rotational_symmetry_exact() {
        // Multibrot-3 (d=3): 2-fold, 180° → (−cx, −cy).
        let (_, b3) = mismatches(1, &|x, y| (-x, -y));
        assert_eq!(b3, 0, "Multibrot-3 not 180°-symmetric ({b3} mismatches)");
        // Multibrot-5 (d=5): 4-fold, 90° → (−cy, cx).
        let (_, b5) = mismatches(3, &|x, y| (-y, x));
        assert_eq!(b5, 0, "Multibrot-5 not 90°-symmetric ({b5} mismatches)");
    }

    // 120° rotations (Multibrot-4, Tricorn) involve an irrational sine, so allow a handful
    // of boundary flips from float error; the symmetry must otherwise hold. (§9: verified.)
    #[test]
    fn threefold_rotational_symmetry_120deg() {
        let (t4, b4) = mismatches(2, &|x, y| rot(x, y, 120.0)); // Multibrot-4: 3-fold
        assert!(b4 * 100 < t4, "Multibrot-4 not ~120°-symmetric ({b4}/{t4})");
        let (tt, bt) = mismatches(4, &|x, y| rot(x, y, 120.0)); // Tricorn: 3-fold
        assert!(bt * 100 < tt, "Tricorn not ~120°-symmetric ({bt}/{tt})");
    }

    // Reflection axes (§9: verify, do not assume). Celtic is symmetric about the real axis;
    // Burning Ship and Buffalo have NO axis reflection (their parts are even in x and y).
    #[test]
    fn abs_variation_reflection_axes() {
        // Celtic (6): real-axis reflection cy → −cy is exact.
        let (_, bc) = mismatches(6, &|x, y| (x, -y));
        assert_eq!(bc, 0, "Celtic not real-axis-symmetric ({bc} mismatches)");
        // Mandelbrot (0): real-axis reflection, exact.
        let (_, bm) = mismatches(0, &|x, y| (x, -y));
        assert_eq!(bm, 0, "Mandelbrot not real-axis-symmetric ({bm} mismatches)");
        // Burning Ship (5) and Buffalo (7): neither axis is a symmetry — confirm both
        // candidate reflections produce many mismatches (documents the asymmetry).
        for formula in [5u32, 7u32] {
            let (t, bx) = mismatches(formula, &|x, y| (-x, y));
            let (_, by) = mismatches(formula, &|x, y| (x, -y));
            assert!(bx * 20 > t && by * 20 > t, "formula {formula} unexpectedly symmetric (x:{bx} y:{by} / {t})");
        }
    }

    // Julia symmetry (dynamical plane): for z^d+c, f_c(ωz)=f_c(z) when ωᵈ=1, so the dwell is
    // invariant under z → ωz for every c. Quadratic case: z → −z (point symmetry).
    #[test]
    fn julia_quadratic_point_symmetry() {
        // Julia: z₀ = pixel, c fixed. dwell(−z₀) must equal dwell(z₀).
        let jc = (-0.512_511_498_387_07_f64, 0.521_295_242_424_99);
        let julia_dwell = |zx: f64, zy: f64| -> Option<u32> {
            let (mut x, mut y) = (zx, zy);
            for i in 1..=2000u32 {
                let (nx, ny) = (x * x - y * y + jc.0, 2.0 * x * y + jc.1);
                x = nx;
                y = ny;
                if x * x + y * y > 4.0 {
                    return Some(i);
                }
            }
            None
        };
        let mut iy = -20i32;
        while iy <= 20 {
            let mut ix = -20i32;
            while ix <= 20 {
                let (zx, zy) = (ix as f64 * 0.08, iy as f64 * 0.08);
                assert_eq!(julia_dwell(zx, zy), julia_dwell(-zx, -zy), "Julia not (−z)-symmetric at ({zx},{zy})");
                ix += 1;
            }
            iy += 1;
        }
    }

    // Extended landmark catalog (§2.2): exact interior/exterior across hyperbolic-component
    // boundaries — cardioid cusp c=¼, period-1↔2 neck c=−¾, period-2 disk tip c=−5/4.
    #[test]
    fn landmark_boundary_classification() {
        // Cusp at c = 1/4: just inside is interior, just outside escapes.
        assert_eq!(dwell_family(0, 0.24, 0.0, 5000), None, "inside cusp escaped");
        assert!(dwell_family(0, 0.26, 0.0, 5000).is_some(), "outside cusp didn't escape");
        // Neck c = −3/4: interior on both the cardioid and period-2 sides.
        assert_eq!(dwell_family(0, -0.74, 0.0, 5000), None, "cardioid side escaped");
        assert_eq!(dwell_family(0, -0.76, 0.0, 5000), None, "period-2 side escaped");
        // Period-2 disk |c+1| < 1/4 (centered at −1): interior. (Just past the real-axis
        // tip −5/4 is the period-4 window — also interior — so leave the disk vertically to
        // reach the exterior.)
        assert_eq!(dwell_family(0, -1.0, 0.0, 5000), None, "period-2 center escaped");
        assert_eq!(dwell_family(0, -1.24, 0.0, 5000), None, "inside period-2 disk escaped");
        assert!(dwell_family(0, -1.0, 0.5, 5000).is_some(), "exterior (−1+0.5i) didn't escape");
        // Cardioid boundary parametrization c(θ)=e^{iθ}/2 − e^{2iθ}/4: a point pulled
        // slightly inward from the boundary is interior.
        for k in 0..8 {
            let th = k as f64 * std::f64::consts::TAU / 8.0;
            let (bx, by) = (0.5 * th.cos() - 0.25 * (2.0 * th).cos(), 0.5 * th.sin() - 0.25 * (2.0 * th).sin());
            let (inx, iny) = (bx * 0.92 + 0.0 * 0.08, by * 0.92); // pull 8% toward interior point ~0
            assert_eq!(dwell_family(0, inx, iny, 5000), None, "inside cardioid boundary escaped at θ={th}");
        }
    }

    // ---- Phase 5.1: fuzz the untrusted coordinate parser (panic-free + round-trip) ----
    #[test]
    fn fuzz_parse_bf_panic_free_and_roundtrips() {
        // Deterministic pseudo-random fuzzing — `parse_bf` ingests pasted/loaded text and
        // must never panic, only return None on garbage.
        let mut s = 0x2545_f491_4f6c_dd1du64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let charset = b"0123456789.-+eE  \t";
        for _ in 0..20_000 {
            let len = (next() % 48) as usize;
            let mut buf = String::with_capacity(len);
            for _ in 0..len {
                buf.push(charset[(next() as usize) % charset.len()] as char);
            }
            let _ = parse_bf(&buf); // must not panic
        }
        // Adversarial explicit inputs.
        for a in [
            "", " ", "\t", "-", "+", ".", "-.", "e", "E", "1e", "1e+", "-1.5e-3", "1e308",
            "1e-308", "1e1000", "-0", "0.0000", "1_000", "0x1f", "NaN", "nan", "inf", "-inf",
            "infinity", "１２３", "1.2.3", "--1", "++1", "1e1e1", ".5", "5.", "  -3.0  ",
        ] {
            let _ = parse_bf(a);
        }
        // Long inputs: bounded work, no hang/panic.
        let _ = parse_bf(&"9".repeat(5000));
        let _ = parse_bf(&format!("-0.{}", "1234567890".repeat(400)));
        // Sanity: well-formed values parse to Some; clear garbage to None. (Exact value
        // round-trips are covered by `center_string_roundtrip_subpixel`.)
        for v in ["0", "-0.5", "-1.7548776662466927", "1e-40", "3.14159"] {
            assert!(parse_bf(v).is_some(), "rejected valid input {v}");
        }
        for v in ["", "abc", "hello", "..", "+-", "inf", "-inf", "NaN"] {
            assert!(parse_bf(v).is_none(), "accepted garbage {v:?}");
        }
    }

    // ---- Phase 6.3: Kalles Fraktaler .kfr import (hardened, fuzzed) ----
    #[test]
    fn parse_kfr_valid_and_robust() {
        // A well-formed .kfr (unknown keys must be ignored, key case ignored).
        let kfr = "Re: -0.743643887037151\nIm: 0.131825904205330\nZoom: 800\n\
                   Iterations: 2000\nColors: 1 2 3\nLocation: ignored\n";
        let v = parse_kfr(kfr).expect("valid .kfr rejected");
        assert!((to_f64(&v.cx) - (-0.743643887037151)).abs() < 1e-12);
        assert!((to_f64(&v.cy) - 0.131825904205330).abs() < 1e-12);
        assert_eq!(v.zoom, 800.0);
        assert_eq!(v.iterations, Some(2000));
        // Over-range zoom clamps; iterations clamp; case-insensitive keys.
        let v = parse_kfr("re: -1\nIM: 0\nZOOM: 1E1000\niterations: 99999999999\n").unwrap();
        assert_eq!(v.zoom, 1.0e300);
        assert_eq!(v.iterations, Some(1_000_000));
        // Missing required fields → None.
        assert!(parse_kfr("Re: 0\nIm: 0\n").is_none(), "missing Zoom accepted");
        assert!(parse_kfr("Im: 0\nZoom: 2\n").is_none(), "missing Re accepted");
        // Invalid center → None (parse_bf rejects).
        assert!(parse_kfr("Re: abc\nIm: 0\nZoom: 2\n").is_none());
    }

    #[test]
    fn fuzz_parse_kfr_panic_free() {
        let mut s = 0x1234_5678_9abc_def1u64;
        let mut next = || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        let charset = b"ReImZoomIterations:-+0.9eE \n\t\0xyz";
        for _ in 0..20_000 {
            let len = (next() % 200) as usize;
            let mut buf = String::with_capacity(len);
            for _ in 0..len {
                buf.push(charset[(next() as usize) % charset.len()] as char);
            }
            let _ = parse_kfr(&buf); // must not panic
        }
        // Oversized / adversarial inputs.
        let _ = parse_kfr(&"Re: 1\n".repeat(100_000));
        let _ = parse_kfr(&format!("Re: -0.{}\nIm: 0\nZoom: 2\n", "1".repeat(200_000)));
        let _ = parse_kfr(&"X".repeat(5_000_000));
        for m in ["", ":", "Re:", "Zoom:::", "Re: \0\0\0\nIm:\nZoom:e"] {
            let _ = parse_kfr(m);
        }
    }
}
