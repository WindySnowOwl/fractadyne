use super::*;

/// The lethal threshold the shipped tunables use, so the cases below are the real numbers rather
/// than round ones invented for a test.
const LETHAL: f64 = 900.0;

/// The case the whole mechanism exists for: a pass that has ALREADY accumulated lethal-band wall
/// time, with no drain in sight, sheds the ledger.
///
/// ⭐The field loss it was written for ran **1024 iterations in 1216 ms** while the queue was on
/// its way down, and a queue in that state never produces the quick present the drain gate waits
/// for — so before this predicate existed, the one pass whose price would have shed the licence
/// was precisely the pass that could not be priced.
#[test]
fn an_in_flight_pass_past_the_lethal_band_sheds() {
    assert!(wall_shed_now(false, false, 1216.0, LETHAL), "the 2026-08-22 field frame");
    assert!(wall_shed_now(false, false, LETHAL, LETHAL), "exactly at the band: >=, not >");
    assert!(wall_shed_now(false, false, 60_000.0, LETHAL), "far past it");
}

/// Below the band there is no evidence yet, and the accumulator is only a LOWER bound — acting on
/// it early would floor licences on passes that were about to drain normally.
#[test]
fn a_pass_short_of_the_band_does_not_shed() {
    for acc in [0.0, 1.0, 400.0, LETHAL - 1.0] {
        assert!(!wall_shed_now(false, false, acc, LETHAL), "acc {acc} should not shed");
    }
}

/// ⚠**The latch.** A pass sitting in the lethal band for many frames sheds ONCE. The ledger is
/// already at the floor after the first, so re-shedding every frame is pure noise — and noise that
/// buries the one line a reader needs to find in a field log.
#[test]
fn the_shed_latches_so_a_long_pass_sheds_once() {
    assert!(wall_shed_now(false, false, 1000.0, LETHAL), "first frame in the band");
    // Same pass, still in the band, several frames later: already shed.
    for acc in [1000.0, 5_000.0, 30_000.0] {
        assert!(!wall_shed_now(true, false, acc, LETHAL), "acc {acc} re-shed after latching");
    }
}

/// ⚠⚠**Present throttling vetoes the shed, and this is the guard most likely to be "simplified"
/// away by someone who has not read the field log.**
///
/// Under present throttling the wall accumulator measures the COMPOSITOR, not the GPU: the frame
/// is waiting to be shown, not computing. Shedding on that would floor every band licence off an
/// **idle** card — which is what the 2026-09-04 field log recorded. It is the same
/// present-throttle poisoning of a wall-clock controller that the dual-view Julia investigation
/// turned on, arriving in a second mechanism.
#[test]
fn present_throttling_vetoes_the_shed_however_large_the_accumulator() {
    for acc in [LETHAL, 5_000.0, 60_000.0] {
        assert!(
            !wall_shed_now(false, true, acc, LETHAL),
            "acc {acc} sheds while present-throttled - that measures the compositor, not the GPU",
        );
    }
    // And the veto is the THROTTLE, not the size: the identical accumulator sheds without it.
    assert!(wall_shed_now(false, false, 5_000.0, LETHAL));
}

/// A garbage accumulator never sheds. NaN reaches this from a clock that went backwards or a
/// subtraction of two stale stamps; a negative one from the same. ⚠Zero is NOT garbage — it is a
/// pass too cheap to measure — but it is also nowhere near the band, so it declines on the
/// threshold rather than on the guard.
#[test]
fn a_garbage_accumulator_never_sheds() {
    for acc in [f64::NAN, f64::NEG_INFINITY, -1.0] {
        assert!(!wall_shed_now(false, false, acc, LETHAL), "acc {acc:?} should not shed");
    }
    // Infinity is not garbage in the same sense - it is "unboundedly expensive" - and a pass that
    // has genuinely run forever is exactly what this exists to catch.
    assert!(wall_shed_now(false, false, f64::INFINITY, LETHAL));
    assert!(!wall_shed_now(false, false, 0.0, LETHAL), "zero declines on the threshold");
}

/// The threshold is a PARAMETER, not a constant baked into the rule — `--set TDR_LETHAL_MS=1` is
/// how the mechanism is forced to fire in a livetest (46 observed sheds), and a rule that ignored
/// the argument would make that harness silently vacuous.
#[test]
fn the_threshold_is_honoured_not_hardcoded() {
    // The forced-lethal harness setting: almost everything sheds.
    assert!(wall_shed_now(false, false, 2.0, 1.0), "--set TDR_LETHAL_MS=1 must fire");
    // A raised threshold must stop a frame that would have shed at the stock one.
    assert!(wall_shed_now(false, false, 1000.0, LETHAL));
    assert!(!wall_shed_now(false, false, 1000.0, 5_000.0));
}
