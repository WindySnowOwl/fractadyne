use crate::FractadyneApp;

/// The direct-path precision wall fires where blocks were MEASURED and stays quiet where the
/// image was sharp. The panel is 1,340 px tall (the field report's), home span 4.
#[test]
fn direct_precision_status_matches_the_measured_views() {
    let h = 1340.0;
    let w = 2160.0;
    let at = |mag: f64, c: (f64, f64), direct: bool| {
        let upp = 4.0 / mag / h;
        FractadyneApp::direct_precision_status(direct, c, (upp * w * 0.5, upp * h * 0.5), upp, mag)
    };
    let field = (-1.637_805_920_305_324, 0.358_468_406_943_270);
    // 549,309×: bricked, 21×5 px measured in the screenshot.
    let s = at(549_309.0, field, true).expect("the bricked view must warn");
    assert!(s.0.contains("depth limit") && s.2, "{s:?}");
    assert!(s.1.contains("22 pixels wide"), "one f32 step at |x| = 1.64 is ~22 px here: {}", s.1);
    // 21,077×: sharp in the screenshot (0.84 px per step).
    assert!(at(21_077.0, field, true).is_none());
    // The same bricked view on a perturbation path has no such wall.
    assert!(at(549_309.0, field, false).is_none());
    // Near the origin the step is the coordinate's own, not |z| ≈ 1's: 0.286+0.012i at a
    // 5e-8 pixel (1.00-px runs measured) is quiet; at 1.5e-8 (1.98-px runs) it warns.
    let upp_mag = |upp: f64| 4.0 / (upp * h);
    let near = (0.2860, 0.0118);
    let quiet = FractadyneApp::direct_precision_status(true, near, (5e-8 * w / 2.0, 5e-8 * h / 2.0), 5e-8, upp_mag(5e-8));
    assert!(quiet.is_none(), "{quiet:?}");
    let warns = FractadyneApp::direct_precision_status(true, near, (1.5e-8 * w / 2.0, 1.5e-8 * h / 2.0), 1.5e-8, upp_mag(1.5e-8));
    assert!(warns.is_some());
}

/// The measured limit regimes classify correctly, and ordinary states stay quiet.
/// Every case here is a real view measured this week, not an invented one.
#[test]
fn limit_status_matches_measured_regimes() {
    const DEV: u32 = 7_452_444;
    const MAX: u32 = crate::MAX_ITER_LIMIT;
    // 2e95× spar: the reference is still partial AT the device cap → the red depth-limit wall.
    let s = FractadyneApp::limit_status(true, DEV, DEV, MAX, None, 0, false, None, false, false)
        .expect("partial-at-device-cap must warn");
    assert!(s.0.contains("depth limit") && s.2, "want severe depth limit, got {s:?}");
    // A partial BELOW the device cap is an ordinary motion/clamp state, not the wall — quiet.
    assert!(FractadyneApp::limit_status(true, 256_001, DEV, 500_000, None, 0, false, None, false, false).is_none());
    // The 6.5e94× black screen: probe climbed to the full appetite, frame stayed all-capped.
    let s = FractadyneApp::limit_status(false, 3_631_055, DEV, MAX, Some(1.0), MAX, true, None, true, true)
        .expect("exhausted must warn");
    assert!(s.0.contains("iter exhausted"), "got {s:?}");
    // Starved at the app maximum: most pixels capped AND escapes pressing the ceiling.
    let s = FractadyneApp::limit_status(
        false, 900_000, DEV, MAX, Some(0.60), MAX, true, Some(MAX as f64 * 0.97), false, false,
    )
    .expect("starved-at-ceiling must warn");
    assert!(s.0.contains("iter capped") && s.1.contains("already at the maximum"), "got {s:?}");

    // Quiet: an ordinary dive's motion partial (nothing refused)…
    assert!(FractadyneApp::limit_status(true, 256_001, DEV, 500_000, None, 0, false, None, false, false).is_none());
    // …the user's 6.9e94× view — a minibrot in frame caps its in-set core (measured 3 px of
    // 2304 = 0.13%) while escapes finish ~1.1M under a 10M budget: resolved, must stay quiet…
    assert!(FractadyneApp::limit_status(
        false, 2_848_721, DEV, MAX, Some(0.0013), MAX, true, Some(1_120_000.0), false, false
    )
    .is_none());
    // …a BIG minibrot (30% of frame in-set) with escapes far below the ceiling: still quiet —
    // in-set pixels always exhaust the budget and raising cannot help them…
    assert!(FractadyneApp::limit_status(
        false, 2_848_721, DEV, MAX, Some(0.30), MAX, true, Some(1_500_000.0), false, false
    )
    .is_none());
    // …mid-climb capping (the app can still raise the budget itself — transient)…
    assert!(FractadyneApp::limit_status(
        false, 900_000, DEV, 1_000_000, Some(0.5), 82_640 * 6, false, Some(490_000.0), false, false
    )
    .is_none());
    // …and a latched plateau that did NOT exhaust the appetite (partial-frame interior).
    assert!(FractadyneApp::limit_status(
        false, 900_000, DEV, MAX, Some(0.9), MAX, true, Some(MAX as f64), true, false
    )
    .is_none());
}
