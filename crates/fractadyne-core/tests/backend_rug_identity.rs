//! Does the MPFR backend reproduce astro-float's reference orbit **byte for byte**?
//!
//! This is the test the whole second-backend idea stands on. If it holds, the F3 corpus goldens
//! gate both backends and a cost comparison has no correctness confound; if it does not, every
//! deep render needs its own blessed set per backend and the feature is a much larger proposition.
//!
//! Both arithmetics run in ONE process via [`reference_orbit_t_in`], so the comparison cannot be
//! confounded by toolchain, build flags, machine or run — the same reason the slice-2 refactor was
//! validated against a HEAD copy linked into a single binary rather than across two builds.
//!
//! Only compiled with `--features rug`, which needs the GNU toolchain (see `backend_rug.rs`).
#![cfg(feature = "rug")]

use fractadyne_core as fc;
use fc::BackendChoice;

fn bits(s: &[[f32; 4]]) -> Vec<[u32; 4]> {
    s.iter().map(|q| [q[0].to_bits(), q[1].to_bits(), q[2].to_bits(), q[3].to_bits()]).collect()
}

/// Tails compared by VALUE, because the two libraries disagree on the SIGN OF ZERO and nothing
/// else: astro-float's `0 - 0` yields `-0` where MPFR yields `+0` (IEEE makes `x - x` positive zero
/// in every rounding mode except toward negative infinity). It shows up on Tricorn's `cy - Im(z^2)`
/// and Phoenix's second term at the origin.
///
/// This is deliberately NOT papered over. It is admissible only because it is unobservable, and
/// that is established rather than assumed:
///   * no emitted sample differs (the matrix below compares ~950k of them) -- `crate::to_f64`
///     returns a literal `0.0` for a zero mantissa, discarding the sign before packing;
///   * the tail is never serialized, displayed or compared in the app -- it is read only for
///     `escaped` and handed straight back to `extend_reference_orbit`;
///   * and `a_signed_zero_in_the_tail_does_not_change_the_continuation` below extends from both
///     tails and shows the resulting orbits are identical.
/// Chasing bit-equality here would mean reproducing astro-float's zero-sign rules, which are an
/// implementation detail rather than a documented contract, and would be fragile for no gain.
fn tail_eq(a: &fc::BigFloat, b: &fc::BigFloat) -> bool {
    if a.is_zero() && b.is_zero() {
        return true;
    }
    fc::to_decimal_string(a) == fc::to_decimal_string(b)
}

/// (label, cx, cy, z0x, z0y) — a boundary point, an interior one, an escaper, the origin, a
/// Julia-style non-zero start, and a Misiurewicz point.
const POINTS: &[(&str, &str, &str, &str, &str)] = &[
    ("boundary", "-0.743643887037158704752191506114774", "0.131825904205311970493132056385139", "0", "0"),
    ("interior", "-0.5", "0.1", "0", "0"),
    ("escaper", "0.4", "0.4", "0", "0"),
    ("origin", "0", "0", "0", "0"),
    ("julia-start", "-0.8", "0.156", "0.3", "-0.2"),
    ("misiurewicz", "-0.77568377", "0.13646737", "0", "0"),
];

#[test]
fn the_mpfr_backend_is_byte_identical_to_astro_float() {
    let precisions = [64usize, 128, 256, 576, 1088, 2112];
    let iters = [1u32, 2, 17, 500, 5000];
    let (mut cases, mut samples) = (0usize, 0usize);
    let mut bad: Vec<String> = Vec::new();

    // All ten formula ids, so Phoenix's two-term recurrence and the `abs` families (Burning Ship,
    // Celtic, Buffalo, Tricorn) are covered — not just Mandelbrot's `z²+c`.
    for formula in 0..10u32 {
        for (label, sx, sy, zx, zy) in POINTS {
            for &p in &precisions {
                let cx = fc::parse_bf_prec(sx, p).unwrap();
                let cy = fc::parse_bf_prec(sy, p).unwrap();
                let z0x = fc::parse_bf_prec(zx, p).unwrap();
                let z0y = fc::parse_bf_prec(zy, p).unwrap();

                for &mi in &iters {
                    let (a, al, at) = fc::reference_orbit_t_in(
                        BackendChoice::Astro, &z0x, &z0y, &cx, &cy, formula, mi, p,
                    );
                    let (r, rl, rt) = fc::reference_orbit_t_in(
                        BackendChoice::Rug, &z0x, &z0y, &cx, &cy, formula, mi, p,
                    );
                    cases += 1;
                    samples += a.len();

                    if al != rl {
                        bad.push(format!("len f={formula} {label} p={p} it={mi}: {al} vs {rl}"));
                        continue;
                    }
                    if bits(&a) != bits(&r) {
                        let k = bits(&a).iter().zip(bits(&r)).position(|(x, y)| *x != y).unwrap();
                        bad.push(format!("samples f={formula} {label} p={p} it={mi}: first diff Z_{k}"));
                    }
                    if at.escaped != rt.escaped {
                        bad.push(format!("escaped f={formula} {label} p={p} it={mi}"));
                    }
                    // The tail is what `extend` resumes from, so it must match too -- by value.
                    if !tail_eq(&at.zx, &rt.zx)
                        || !tail_eq(&at.zy, &rt.zy)
                        || !tail_eq(&at.zpx, &rt.zpx)
                        || !tail_eq(&at.zpy, &rt.zpy)
                    {
                        bad.push(format!(
                            "tail f={formula} {label} p={p} it={mi}: {:?}/{:?} vs {:?}/{:?}",
                            fc::to_decimal_string(&at.zx),
                            fc::to_decimal_string(&at.zy),
                            fc::to_decimal_string(&rt.zx),
                            fc::to_decimal_string(&rt.zy),
                        ));
                    }
                    // And each tail must be tagged with the backend that actually produced it,
                    // since `extend` dispatches on that tag rather than on the current selection.
                    assert_eq!(at.backend, BackendChoice::Astro.bit(), "astro tail mis-tagged");
                    assert_eq!(rt.backend, BackendChoice::Rug.bit(), "rug tail mis-tagged");
                }
            }
        }
    }

    assert!(cases > 1000, "matrix collapsed to {cases} cases — it is not covering what it claims");
    assert!(
        bad.is_empty(),
        "{} of {cases} cases ({samples} samples) diverged:\n  {}",
        bad.len(),
        bad.iter().take(20).cloned().collect::<Vec<_>>().join("\n  ")
    );
}

#[test]
fn an_extend_resumes_in_the_backend_that_built_the_prefix() {
    // `extend_reference_orbit` promises byte-identity with a fresh build. It keeps that across a
    // backend switch only by finishing the orbit in the arithmetic that started it, so this holds
    // the two backends' extends against their OWN fresh builds, not against each other.
    let p = 576;
    let cx = fc::parse_bf_prec("-0.743643887037158704752191506114774", p).unwrap();
    let cy = fc::parse_bf_prec("0.131825904205311970493132056385139", p).unwrap();
    let z0 = fc::BigFloat::from_f64(0.0, p);

    for backend in [BackendChoice::Astro, BackendChoice::Rug] {
        let (prefix, _, tail) =
            fc::reference_orbit_t_in(backend, &z0, &z0, &cx, &cy, fc::formula::MANDELBROT, 200, p);
        let (ext, extl, _) =
            fc::extend_reference_orbit(&prefix, &tail, &cx, &cy, fc::formula::MANDELBROT, 900, p);
        let (fresh, freshl, _) =
            fc::reference_orbit_t_in(backend, &z0, &z0, &cx, &cy, fc::formula::MANDELBROT, 900, p);
        assert_eq!(extl, freshl, "{backend:?}: extended length differs from a fresh build");
        assert_eq!(bits(&ext), bits(&fresh), "{backend:?}: extend is not byte-identical to fresh");
        assert!(extl > 200, "the extend did no work, so this proves nothing");
    }
}

#[test]
fn the_stamp_records_both_backends_once_both_have_run() {
    let p = 128;
    let z0 = fc::BigFloat::from_f64(0.0, p);
    let cx = fc::parse_bf_prec("-0.5", p).unwrap();
    for b in [BackendChoice::Astro, BackendChoice::Rug] {
        let _ = fc::reference_orbit_in(b, &z0, &z0, &cx, &z0, fc::formula::MANDELBROT, 16, p);
    }
    let seen = fc::observed_backends();
    assert!(seen.contains(&"astro-float") && seen.contains(&"rug"), "observed: {seen:?}");
    // Two backends in one process is exactly what `--selftest` must refuse to attribute a run to.
    assert!(fc::backend_status_line().starts_with("MIXED"), "{}", fc::backend_status_line());
}

/// The signed-zero difference documented on [`tail_eq`] must not change what happens NEXT.
///
/// Covers exactly the cases the byte-identity matrix flagged -- Tricorn and Phoenix at points that
/// drive a tail component to zero -- by continuing each backend's orbit from its own tail and
/// comparing the resulting sample streams. If `-0` vs `+0` could propagate, it would show here.
#[test]
fn a_signed_zero_in_the_tail_does_not_change_the_continuation() {
    let mut checked = 0usize;
    let mut saw_sign_difference = false;
    for &(formula, sx, sy) in
        &[(4u32, "0", "0"), (8u32, "0", "0"), (4u32, "-0.5", "0.1"), (8u32, "-0.5", "0.1")]
    {
        for &p in &[64usize, 576] {
            for &pre in &[1u32, 2, 17] {
                let z0 = fc::BigFloat::from_f64(0.0, p);
                let cx = fc::parse_bf_prec(sx, p).unwrap();
                let cy = fc::parse_bf_prec(sy, p).unwrap();

                let (ap, _, at) =
                    fc::reference_orbit_t_in(BackendChoice::Astro, &z0, &z0, &cx, &cy, formula, pre, p);
                let (rp, _, rt) =
                    fc::reference_orbit_t_in(BackendChoice::Rug, &z0, &z0, &cx, &cy, formula, pre, p);

                // Only interesting where the representations actually differ.
                let differs = fc::to_decimal_string(&at.zx) != fc::to_decimal_string(&rt.zx)
                    || fc::to_decimal_string(&at.zy) != fc::to_decimal_string(&rt.zy);
                saw_sign_difference |= differs;

                let (ax, axl, _) =
                    fc::extend_reference_orbit(&ap, &at, &cx, &cy, formula, pre + 400, p);
                let (rx, rxl, _) =
                    fc::extend_reference_orbit(&rp, &rt, &cx, &cy, formula, pre + 400, p);
                assert_eq!(axl, rxl, "f={formula} p={p} pre={pre}: continuation lengths differ");
                assert_eq!(
                    bits(&ax),
                    bits(&rx),
                    "f={formula} p={p} pre={pre}: a signed zero in the tail CHANGED the continuation"
                );
                checked += 1;
            }
        }
    }
    assert!(checked >= 12, "only {checked} continuations checked");
    assert!(
        saw_sign_difference,
        "no tail representation difference occurred, so this test never exercised the case it          exists for -- it would pass whether or not signed zero propagates"
    );
}
