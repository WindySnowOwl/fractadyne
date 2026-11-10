use super::*;

fn card() -> Card {
    Card {
        app: "0.3.0-beta.27 (build 6500, g1234567)".into(),
        name: "Intel(R) HD Graphics 4400".into(),
        api: "Gl".into(),
        driver: "4.3.0 - Build 20.19.15.5171".into(),
    }
}

fn plugged_in() -> PowerState {
    PowerState { ac: Some(true), saver: false, plan: "381b4222-f694-41f0-9685-ff5bb260df2e".into(), mode: String::new(), cpu_limit_pct: 100 }
}

/// Eris's dev.9 readings (2026-11-10).
fn eris(power: PowerState) -> Reading {
    let m = |iters, ms, ms_per_step| SavedFinding::of(Finding::Measured { iters, ms, wall_ms: ms + 1.0, ms_per_step });
    Reading {
        card: card(),
        power,
        taken: "2026-11-10 14:03".into(),
        direct: m(32, 19.6, 2.34e-6),
        df32: m(16, 23.5, 5.60e-6),
        floatexp: m(16, 49.5, 1.18e-5),
    }
}

#[test]
fn a_reading_is_found_for_its_card_and_power_state_only() {
    let saved = vec![eris(plugged_in())];
    assert_eq!(find(&saved, &card(), &plugged_in()), Some(&saved[0]));
    assert!(find(&saved, &card(), &PowerState { ac: Some(false), ..plugged_in() }).is_none(), "on battery is another state");
    for other in [
        Card { app: "0.3.0-beta.27 (build 6501, g7654321)".into(), ..card() },
        Card { driver: "4.3.0 - Build 20.19.15.5200".into(), ..card() },
        Card { api: "Vulkan".into(), ..card() },
        Card { name: "Intel(R) HD Graphics 5500".into(), ..card() },
    ] {
        assert!(find(&saved, &other, &plugged_in()).is_none(), "a new build, driver, API or card measures again: {other:?}");
    }
}

#[test]
fn a_laptop_keeps_one_reading_per_state_the_newest_first() {
    let battery = PowerState { ac: Some(false), ..plugged_in() };
    let v = with(&[], eris(plugged_in()));
    let v = with(&v, eris(battery.clone()));
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].power, battery, "newest first");
    // Measuring a state again replaces its reading, also through a small clock wobble.
    let again = Reading { taken: "later".into(), ..eris(PowerState { cpu_limit_pct: 95, ..plugged_in() }) };
    let v = with(&v, again.clone());
    assert_eq!(v.len(), 2);
    assert_eq!(v[0], again);
    assert_eq!(without(&v, &card(), &plugged_in()).len(), 1);
    // At most KEEP.
    let many = (0..KEEP as u32 + 3).fold(Vec::new(), |v, k| with(&v, eris(PowerState { plan: format!("plan-{k}"), ..plugged_in() })));
    assert_eq!(many.len(), KEEP);
    assert_eq!(many[0].power.plan, format!("plan-{}", KEEP + 2));
}

#[test]
fn the_file_round_trips_and_a_damaged_value_is_not_used() {
    let v = vec![eris(plugged_in()), eris(PowerState { ac: None, ..plugged_in() })];
    let text = render(&v).expect("renders");
    assert!(text.starts_with("# Fractadyne's start-up speed checks"), "{text}");
    assert_eq!(parse(&text), Some(v.clone()));
    assert!(parse("not toml [").is_none());
    assert_eq!(parse(""), Some(Vec::new()));
    // A value no probe produces leaves the reading unusable: the check measures instead.
    for bad in [
        SavedFinding { ms_per_step: f64::NAN, ..v[0].direct },
        SavedFinding { ms_per_step: 0.0, ..v[0].direct },
        SavedFinding { ms_per_step: 5.0, ..v[0].direct },
        SavedFinding { iters: 0, ..v[0].direct },
        SavedFinding { iters: ITERS_MAX + 1, ..v[0].direct },
        SavedFinding { ms: -1.0, ..v[0].direct },
    ] {
        let r = Reading { direct: bad, ..v[0].clone() };
        assert!(r.findings().is_none(), "{bad:?}");
        assert!(find(&[r], &card(), &plugged_in()).is_none());
    }
}

#[test]
fn the_file_is_written_whole_and_read_back() {
    let dir = std::env::temp_dir().join(format!("fd-speed-checks-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(load(&dir).is_empty(), "no file: nothing saved");
    let v = vec![eris(plugged_in())];
    store(&dir, &v).expect("stores");
    assert_eq!(load(&dir), v);
    assert!(!dir.join(format!("{FILE}.part")).exists(), "the temporary file is renamed away");
    // A file too large to be ours is not read.
    std::fs::write(dir.join(FILE), "#".repeat(70 * 1024)).unwrap();
    assert!(load(&dir).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_re_check_confirms_a_reading_within_its_slack_either_way() {
    let saved = Finding::Measured { iters: 32, ms: 19.6, wall_ms: 20.6, ms_per_step: 2.34e-6 };
    let fast_below = 1e-7;
    assert!(still_holds(saved, 2.34e-6, fast_below));
    assert!(still_holds(saved, 2.34e-6 * 1.4, fast_below), "jitter");
    assert!(still_holds(saved, 2.34e-6 / 1.4, fast_below));
    assert!(!still_holds(saved, 2.34e-6 * 2.0, fast_below), "slower now (on battery, held at a low clock): measure again");
    assert!(!still_holds(saved, 2.34e-6 / 2.0, fast_below), "faster now (it was measured while busy): measure again");
    assert!(!still_holds(saved, f64::NAN, fast_below));
    assert!(!still_holds(saved, 0.0, fast_below));
}

#[test]
fn a_fast_card_s_bound_stands_while_the_re_check_stays_fast() {
    // RTX 3070 (PLUTO, dev.9): direct reads fast, far under the thresholds.
    let saved = Finding::Fast { iters: 256, ms: 0.9, ms_per_step: 1.3e-9 };
    let fast_below = 1.92e-8;
    assert!(still_holds(saved, 1.3e-9, fast_below));
    assert!(still_holds(saved, 1.5e-8, fast_below), "still under what would change anything");
    assert!(still_holds(saved, 1e-10, fast_below), "faster is still fast");
    assert!(!still_holds(saved, 3e-8, fast_below), "slower than the threshold: measure again");
}
