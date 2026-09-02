//! The Misiurewicz auto-detector at EXTREME depth — the case the 20,000-iteration walk cap used
//! to miss entirely (user report, 2026-09-02: "does not find a misi point at e40010").
//!
//! At a 2.37e4001× view the organizing Misiurewicz point sits at preperiod ~438,732 — its
//! identifying near-return is ~22× past the old cap, so `detect_misiurewicz_at_scale` returned
//! `None` and "Find near view" reported "No Misiurewicz point found near this view". With the walk
//! cost-bounded instead of hard-capped, the detector reaches it. The critical orbit is ~440k
//! bignum steps at ~13,366-bit precision, so this is `#[ignore]` (≈70 s) — run explicitly:
//!   cargo test -p fractadyne-core --release --test misiurewicz_deep -- --ignored --nocapture
//!
//! The coordinate lives in `tests/fixtures/e4001-misiurewicz.txt` (the user's exact centre) rather
//! than inline, because it is 4,031 digits.

use fractadyne_core as fc;

fn fixture() -> (String, String) {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/e4001-misiurewicz.txt");
    let text = std::fs::read_to_string(path).expect("e4001 fixture present");
    let mut lines = text.lines();
    (lines.next().unwrap().trim().to_string(), lines.next().unwrap().trim().to_string())
}

/// The bug, fixed: detection now finds a pair at the e4001 view. Its point is a deep Misiurewicz
/// point — a LARGE preperiod and a small period — and it must actually solve back onto the view
/// centre (a pair whose point is elsewhere would be a false positive, the trap the scale-ranking
/// exists to avoid). Not pinned to one exact pair: several deep pairs near this centre all solve
/// to within a fraction of a pixel of it (measured 2^-15…2^-24 view-widths), so a better ranking
/// may legitimately pick another — but it must have the SHAPE of a deep point and land here.
#[test]
#[ignore = "≈70 s: ~440k bignum steps at 13,366-bit precision"]
fn the_e4001_view_finds_its_deep_misiurewicz_point() {
    let (cxs, cys) = fixture();
    let cur_l2 = 13_291.5f64; // the e4001 view depth (upp_log2 −13302, 1465 px wide)
    let p = fc::precision_for_octaves(cur_l2.ceil() as u64);
    let cx = fc::parse_bf_prec(&cxs, p).unwrap();
    let cy = fc::parse_bf_prec(&cys, p).unwrap();
    let span_log2 = -cur_l2 + (1465f64).log2();

    let (k, per) = fc::detect_misiurewicz_at_scale(&cx, &cy, 0, 2_000_000, 1_024, p, Some(span_log2))
        .expect("a Misiurewicz pair is found at the e4001 view (was None under the 20k cap)");
    assert!(k > 100_000, "preperiod {k} is too shallow to organize a 1e4001 view");
    assert!(per > 0 && per < 4_096, "period {per} is not a plausible cycle length");

    // It must solve back onto the view — a real point at the centre, not a distant misfit.
    let m = fc::find_misiurewicz(
        &[cx.clone(), cy.clone()],
        k,
        per,
        fc::SolveScale { log2_seed: cur_l2, log2_target: cur_l2 },
        0,
    )
    .expect("the detected pair solves to a real point near the seed view");
    assert_eq!((m.preperiod, m.period), (k, per), "the solve confirms the detected pair");
    println!("e4001 detect -> (k={k}, p={per}); solved onto the view centre");
}
