use super::*;

const BOOT: u64 = 400_000_000; // TDR_BOOTSTRAP_STEPS
const FLOOR: u64 = 4_000_000; // TDR_MIN_STEPS

const ALL: [DerateTrigger; 3] =
    [DerateTrigger::ClampLifted, DerateTrigger::LengthJump, DerateTrigger::LengthCollapse];

#[test]
fn an_unmeasured_budget_stays_unmeasured_whatever_installs() {
    // The user's log, 2026-11-18 +51.09 s: a mode switch left v0 at 0, its first df32 reference
    // (262,035 partial → 198 complete) installed one frame later, and the budget became the 4e6
    // floor instead of the 4e8 opening guess. 0 must stay 0, skipping or not.
    for t in ALL {
        for skipped in [false, true] {
            assert_eq!(install_derate(0, t, skipped, BOOT, FLOOR), None, "{t:?} skipped={skipped}");
        }
    }
}

#[test]
fn a_length_trigger_from_a_reference_that_skipped_nothing_leaves_the_budget() {
    // +38.73 s: the Julia panel's 134 → 222 re-pick took 7.39e10 to 9.24e9; +38.99 s 270 → 153
    // took it to the bootstrap; +39.18 s 233 → 155 to 5e7. No series, no BLA, no renorm step.
    assert_eq!(install_derate(73_900_000_000, DerateTrigger::LengthJump, false, BOOT, FLOOR), None);
    assert_eq!(install_derate(9_240_000_000, DerateTrigger::LengthCollapse, false, BOOT, FLOOR), None);
    assert_eq!(install_derate(BOOT, DerateTrigger::LengthCollapse, false, BOOT, FLOOR), None);
    // +51.47 s: 198 → 396 took v0 from 9e6 back to the floor.
    assert_eq!(install_derate(9_000_000, DerateTrigger::LengthJump, false, BOOT, FLOOR), None);
}

#[test]
fn a_length_trigger_from_a_reference_that_skipped_derates_as_before() {
    // crash-1786506241's shape: a measured 3e11 budget and a collapse caps at the bootstrap.
    assert_eq!(
        install_derate(300_000_000_000, DerateTrigger::LengthCollapse, true, BOOT, FLOOR),
        Some(BOOT)
    );
    // ÷8 under the cap.
    assert_eq!(
        install_derate(2_000_000_000, DerateTrigger::LengthCollapse, true, BOOT, FLOOR),
        Some(250_000_000)
    );
    // A jump is ÷8, not capped.
    assert_eq!(
        install_derate(300_000_000_000, DerateTrigger::LengthJump, true, BOOT, FLOOR),
        Some(37_500_000_000)
    );
    // Never below the floor, and never a raise: at the floor there is nothing to derate.
    assert_eq!(install_derate(5_000_000, DerateTrigger::LengthJump, true, BOOT, FLOOR), Some(FLOOR));
    assert_eq!(install_derate(FLOOR, DerateTrigger::LengthJump, true, BOOT, FLOOR), None);
    assert_eq!(install_derate(FLOOR, DerateTrigger::LengthCollapse, true, BOOT, FLOOR), None);
}

#[test]
fn a_clamp_lift_derates_a_measured_budget_with_or_without_skipping() {
    // The 2.6e72× spar crash (beta.29): unchanged by this fix, except that 0 stays 0.
    for skipped in [false, true] {
        assert_eq!(
            install_derate(300_000_000_000, DerateTrigger::ClampLifted, skipped, BOOT, FLOOR),
            Some(37_500_000_000)
        );
        assert_eq!(install_derate(0, DerateTrigger::ClampLifted, skipped, BOOT, FLOOR), None);
    }
}

#[test]
fn every_derate_lowers_the_budget_and_respects_the_floor() {
    for t in ALL {
        for skipped in [false, true] {
            for cur in [1_u64, FLOOR - 1, FLOOR, FLOOR + 1, BOOT, BOOT * 8, 300_000_000_000, u64::MAX] {
                if let Some(d) = install_derate(cur, t, skipped, BOOT, FLOOR) {
                    assert!(d < cur, "{t:?} {cur}: {d} is not a derate");
                    assert!(d >= FLOOR, "{t:?} {cur}: {d} under the floor");
                    if t == DerateTrigger::LengthCollapse {
                        assert!(d <= BOOT, "{t:?} {cur}: a collapse caps at the bootstrap");
                    }
                }
            }
        }
    }
}
