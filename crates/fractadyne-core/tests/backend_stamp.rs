//! The backend stamp must report what **executed**, not what was configured.
//!
//! This lives in its own integration test — and is the ONLY test in this file — on purpose. The
//! observation mask is process-global, so an assertion about the "nothing has run yet" state is
//! only meaningful in a process where nothing else can build an orbit underneath it. As a unit
//! test alongside the crate's other tests it would pass or fail on thread scheduling.

use fractadyne_core as fc;

#[test]
fn the_stamp_names_the_backend_only_after_an_orbit_actually_runs() {
    // Nothing has iterated yet: the stamp must say so rather than name a backend it merely could
    // have used. A stamp that reads from configuration would already answer "astro-float" here.
    assert_eq!(fc::backend_status_line(), "none (no reference orbit built yet)");
    assert!(fc::observed_backends().is_empty());

    // Work that touches the carrier type but builds no orbit must still not set it.
    let p = 128;
    let zero = fc::BigFloat::from_f64(0.0, p);
    let cx = fc::parse_bf_prec("-0.743643887037158704752191506114774", p).unwrap();
    let cy = fc::parse_bf_prec("0.131825904205311970493132056385139", p).unwrap();
    assert!(
        fc::observed_backends().is_empty(),
        "parsing coordinates must not stamp a backend"
    );

    let (samples, len) =
        fc::reference_orbit(&zero, &zero, &cx, &cy, fc::formula::MANDELBROT, 32, p);
    assert_eq!(len as usize, samples.len());
    assert!(len > 1, "the orbit must actually have run for this test to mean anything");

    assert_eq!(fc::observed_backends(), vec!["astro-float"]);
    assert_eq!(fc::backend_status_line(), "astro-float");
}
