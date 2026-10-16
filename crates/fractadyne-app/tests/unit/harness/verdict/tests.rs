use super::*;

#[test]
fn a_real_failure_is_never_reported_as_vacuous() {
    assert_eq!(code(false, false), PASS);
    assert_eq!(code(false, true), VACUOUS);
    assert_eq!(code(true, false), RED);
    assert_eq!(code(true, true), RED);
    // Every code a gate can return is distinct, and none of the failures is a pass.
    let all = [PASS, RED, VACUOUS, WATCHDOG];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a, b);
        }
    }
}
