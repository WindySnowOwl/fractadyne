use super::*;

fn laptop_on_ac() -> PowerState {
    PowerState {
        ac: Some(true),
        saver: false,
        plan: "381b4222-f694-41f0-9685-ff5bb260df2e".into(),
        mode: "00000000-0000-0000-0000-000000000000".into(),
        cpu_limit_pct: 100,
    }
}

#[test]
fn a_reading_holds_in_its_own_state_and_through_a_small_clock_wobble() {
    let s = laptop_on_ac();
    assert!(s.matches(&s));
    assert!(PowerState { cpu_limit_pct: 92, ..s.clone() }.matches(&s), "a few points of thermal wobble is the same state");
    assert!(PowerState { cpu_limit_pct: 110, ..s.clone() }.matches(&s));
}

#[test]
fn each_part_of_the_power_state_is_a_state_of_its_own() {
    let s = laptop_on_ac();
    for other in [
        PowerState { ac: Some(false), ..s.clone() },
        PowerState { ac: None, ..s.clone() },
        PowerState { saver: true, ..s.clone() },
        PowerState { plan: "a1841308-3541-4fab-bc81-f71556f20b4a".into(), ..s.clone() },
        PowerState { mode: "961cc777-2547-4f9d-8174-7d86181b8a7a".into(), ..s.clone() },
        PowerState { cpu_limit_pct: 60, ..s.clone() },
    ] {
        assert!(!other.matches(&s) && !s.matches(&other), "{other:?}");
    }
}

#[test]
fn the_log_names_the_state_in_words() {
    let s = laptop_on_ac();
    assert_eq!(s.line(), "plugged in, battery saver off, plan Balanced, power mode Balanced, CPU clock limit 100%");
    let battery = PowerState { ac: Some(false), saver: true, mode: String::new(), cpu_limit_pct: 0, plan: "1234".into() };
    assert_eq!(battery.line(), "on battery, battery saver on, plan 1234, CPU clock limit unread");
    assert_eq!(PowerState::default().line(), "power source unknown, battery saver off, plan unread, CPU clock limit unread");
}

#[test]
fn the_clock_limit_is_a_share_of_the_rated_clock() {
    assert_eq!(limit_pct(8 * 2600, 8 * 2600), 100);
    assert_eq!(limit_pct(8 * 1300, 8 * 2600), 50);
    assert_eq!(limit_pct(0, 8 * 2600), 0, "no limit read");
    assert_eq!(limit_pct(2600, 0), 0, "no rated clock read");
}

#[test]
fn this_machine_s_state_reads() {
    // Whatever this machine is, a read must not fail, and what it reads must match itself.
    let s = PowerState::read();
    assert!(s.matches(&s), "{s:?}");
    assert!(s.cpu_limit_pct <= 1000, "{s:?}");
}
