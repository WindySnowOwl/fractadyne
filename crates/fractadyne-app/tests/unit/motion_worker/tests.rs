use super::*;

#[test]
fn a_worker_frame_goes_where_the_display_reads() {
    // A frame that renders the current view itself is newer: the worker's is dropped.
    assert_eq!(adopt_target(true, false, false), Adopt::Drop);
    // A reprojection with no gate shows the live texture: into it.
    assert_eq!(adopt_target(false, false, false), Adopt::Live);
    // A pin in flight (or its residue) serves the hold: into the hold.
    assert_eq!(adopt_target(false, true, false), Adopt::Hold);
    // A pin starting this frame snapshots the live texture after the adoption lands: into it.
    assert_eq!(adopt_target(false, true, true), Adopt::Live);
}

#[test]
fn a_frame_is_ranked_by_how_it_shows_the_view_now() {
    // Sharp and current: 0. Magnified one octave: -1. Half the texels: one more octave down.
    assert_eq!(frame_quality(0.0, 10.0, 10.0), 0.0);
    assert_eq!(frame_quality(0.0, 9.0, 10.0), -1.0);
    assert_eq!(frame_quality(-1.0, 9.0, 10.0), -2.0);
    // Extra texels are not counted past one a pixel; a deeper frame covers a shrinking patch.
    assert_eq!(frame_quality(1.0, 10.0, 10.0), 0.0);
    assert_eq!(frame_quality(0.0, 11.0, 10.0), -1.0);
    // The ranking of two frames does not change as a dive goes on: a quarter-resolution frame
    // 1.5 octaves newer loses to a full-resolution one at every later view.
    for now in [10.0, 11.0, 14.0] {
        assert!(frame_quality(0.0, 9.0, now) > frame_quality(-2.0, 10.5, now));
    }
}

#[test]
fn a_live_view_is_inside_a_pinned_one_only_when_all_of_it_is() {
    // Zoomed in at the same centre: inside; zoomed out: not.
    assert!(view_inside(0.0, 0.0, 0.5, 0.4, 1.0, 0.8));
    assert!(view_inside(0.0, 0.0, 1.0, 0.8, 1.0, 0.8));
    assert!(!view_inside(0.0, 0.0, 1.01, 0.8, 1.0, 0.8));
    // Panned: inside while its far edge is; a quarter span right at half the size just fits.
    assert!(view_inside(0.25, 0.0, 0.5, 0.4, 1.0, 0.8));
    assert!(!view_inside(0.26, 0.0, 0.5, 0.4, 1.0, 0.8));
    assert!(!view_inside(0.0, -0.21, 0.5, 0.4, 1.0, 0.8));
}

#[test]
fn the_worker_resolution_follows_its_time() {
    // On target: unchanged. Four times too slow: a quarter of the area, capped at -20% a side.
    assert_eq!(live_res_next(0.5, LIVE_TARGET_MS), 0.5);
    assert!((live_res_next(0.5, 4.0 * LIVE_TARGET_MS) - 0.4).abs() < 1e-12);
    // Fast: grows at most a quarter, never past the panel.
    assert!((live_res_next(0.5, 1.0) - 0.625).abs() < 1e-12);
    assert_eq!(live_res_next(0.95, 1.0), 1.0);
    // Never below the floor; a garbage time keeps it.
    assert_eq!(live_res_next(LIVE_RES_MIN, 1.0e9), LIVE_RES_MIN);
    assert_eq!(live_res_next(0.5, f64::NAN), 0.5);
}

#[test]
fn a_long_worker_pass_says_what_it_ran_and_what_sized_it() {
    use fractadyne_gpu::LongestPass;
    // Walk 137 of the 2026-10-22 Home-glide trace: a window grown on a cheap stretch.
    let window = LongestPass { range: [131_072, 262_144], cap: 0, samples: 314_835, ms: 129.7, prev_ms: 0.93, prev_work: 65_536, index: 10 };
    assert_eq!(
        long_pass_line(&window, 28),
        "⚠long worker pass: 130 ms, iterations [131072, 262144) over 314835 samples (pass 11 of 28); \
         the pass before it took 0.9 ms over 65536 iterations"
    );
    let stepped = LongestPass { range: [1, 316_593], cap: 4_096, samples: 12_000, ms: 101.0, prev_ms: 30.0, prev_work: 28_672_000, index: 7 };
    assert_eq!(
        long_pass_line(&stepped, 30),
        "⚠long worker pass: 101 ms, up to 4096 steps for each of 12000 running samples (pass 8 of 30); \
         the pass before it took 30.0 ms for 28672000 sample-steps"
    );
    let first = LongestPass { index: 0, ..stepped };
    assert!(long_pass_line(&first, 1).ends_with("(pass 1 of 1); the first of its walk"));
}
