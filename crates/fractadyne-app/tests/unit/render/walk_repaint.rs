use super::{walk_repaint_ms, WALK_REPAINT_MS};
use crate::tunables::CHUNK_DRAIN_DT_MS;

#[test]
fn the_beat_during_an_in_flight_pass_can_satisfy_the_drain_criterion() {
    // 2026-09-11: a settled walk at a parabolic exact point sat at `size=2` for ~700 s. A pass in
    // flight is released only by a QUICK PRESENT (`last_dt_ms < CHUNK_DRAIN_DT_MS`), but the settled
    // repaint beat idled ≥250 ms between frames, so no present could ever be quick: every idle
    // interval was accumulated as the pass's cost, the ledger shed at the lethal band on the second
    // frame, and the wall-aware floor collapsed the window to 2. The invariant this pins: while a
    // pass is in flight the beat must stay BELOW the drain threshold, or the criterion is
    // unsatisfiable — a check that goes red if someone raises the beat or lowers the threshold.
    assert!(
        WALK_REPAINT_MS < CHUNK_DRAIN_DT_MS,
        "the in-flight beat ({WALK_REPAINT_MS} ms) must undercut the drain threshold ({CHUNK_DRAIN_DT_MS} ms)"
    );
    for frame_ms in [5.0, 60.0, 120.0, 400.0, 2000.0] {
        assert!(
            walk_repaint_ms(true, frame_ms) < CHUNK_DRAIN_DT_MS,
            "in flight at frame_ms {frame_ms}: the beat must allow a quick present"
        );
    }
    // With nothing in flight the ordinary settled beat is unchanged: 4× the frame EMA, 250–1000 ms.
    assert_eq!(walk_repaint_ms(false, 10.0), 250.0);
    assert_eq!(walk_repaint_ms(false, 100.0), 400.0);
    assert_eq!(walk_repaint_ms(false, 1000.0), 1000.0);
}
