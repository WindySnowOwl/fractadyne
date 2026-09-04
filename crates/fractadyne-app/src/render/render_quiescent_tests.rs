use super::{render_quiescent, ViewActivity, IDLE_DISPATCH_FRAMES};

/// A view that has been quiet long enough to count as settled.
fn quiet() -> ViewActivity {
    ViewActivity { frames_since_dispatch: IDLE_DISPATCH_FRAMES + 1, ..Default::default() }
}

#[test]
fn both_views_quiet_and_nothing_animating_is_quiescent() {
    assert!(render_quiescent([quiet(), quiet()], false, true));
}

/// The dispatch tail: a view that dispatched this frame — or within the tail — is NOT settled.
/// The timestamp that prices a pass comes back up to two frames later, so the tail must outlast
/// it; standing down at the dispatch itself would stop the frames that carry the reading.
#[test]
fn the_dispatch_tail_must_elapse_first() {
    for n in 0..=IDLE_DISPATCH_FRAMES {
        let v = ViewActivity { frames_since_dispatch: n, ..Default::default() };
        assert!(!render_quiescent([v, quiet()], false, true), "stood down {n} frames after a dispatch");
    }
}

/// ⭐The dual-view case the beta.16 saga was about: view 0 finishes and view 1 is still walking
/// its chunk progression. A view-0-only test would call that idle — this must not.
#[test]
fn a_busy_second_view_keeps_the_beat() {
    let walking = ViewActivity { chunk_pending: true, ..quiet() };
    assert!(!render_quiescent([quiet(), walking], false, true));
    assert!(!render_quiescent([walking, quiet()], false, true));
}

/// Each work signal alone is enough to stay busy, in EITHER view.
#[test]
fn any_single_work_signal_keeps_the_beat() {
    let cases = [
        ViewActivity { tile_pending: true, ..quiet() },
        ViewActivity { chunk_pending: true, ..quiet() },
        ViewActivity { building: true, ..quiet() },
    ];
    for (i, c) in cases.into_iter().enumerate() {
        assert!(!render_quiescent([c, quiet()], false, true), "case {i} in view 0");
        assert!(!render_quiescent([quiet(), c], false, true), "case {i} in view 1");
    }
}

/// Animation runs off its own repaint request, but the readouts still move while it does — and
/// a false "idle" is the one mistake this predicate must never make, so animation holds the beat.
#[test]
fn animation_keeps_the_beat() {
    assert!(!render_quiescent([quiet(), quiet()], true, true));
}

/// Standing down freezes whatever is on screen, so it must not happen while a per-second rate
/// is still decaying — that would leave a stale non-zero "builds/s" frozen in the panel.
#[test]
fn a_rate_still_decaying_keeps_the_beat() {
    assert!(!render_quiescent([quiet(), quiet()], false, false));
}
