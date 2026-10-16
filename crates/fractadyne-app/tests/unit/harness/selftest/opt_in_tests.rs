use super::*;

/// The names this file passes to `opt_in("…")`, read from its own source.
fn opt_in_calls() -> Vec<String> {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/harness/selftest.rs"));
    let needle = ["opt_in", "(\""].concat(); // split so this line is not itself a match
    src.match_indices(needle.as_str())
        .filter_map(|(i, _)| {
            let rest = &src[i + needle.len()..];
            let name: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
            (!name.is_empty() && rest[name.len()..].starts_with('"')).then_some(name)
        })
        .collect()
}

#[test]
fn every_opt_in_group_is_registered() {
    let calls = opt_in_calls();
    let registered: Vec<&str> = OPT_IN_GROUPS.iter().map(|(g, _)| *g).collect();
    assert!(calls.len() >= 2, "the source scan found {calls:?}: it is not reading the opt_in calls");
    for c in &calls {
        assert!(registered.contains(&c.as_str()), "opt_in(\"{c}\") is not in OPT_IN_GROUPS: a full run would not say it skipped it");
    }
    for g in &registered {
        assert!(calls.iter().any(|c| c == g), "OPT_IN_GROUPS names {g}, which no opt_in call runs");
    }
}

#[test]
fn each_card_is_judged_by_its_own_golden_set_and_blesses_its_own() {
    let g = std::env::temp_dir().join(format!("fd-golden-sets-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&g);
    std::fs::create_dir_all(&g).unwrap();
    let (a, b) = ("NVIDIA GeForce RTX 3080", "AMD Radeon RX 6800 XT");
    let own_b = g.join("amd-radeon-rx-6800-xt");
    // No marker anywhere: the reference set, STRICT (a missing file never loosens the gate),
    // and the first bless writes the reference set.
    assert_eq!(golden_set_in(&g, b), GoldenSet { dir: g.clone(), strict: true, own: false });
    assert_eq!(bless_dir_in(&g, b, false), g);
    // The reference blessed on A: strict for A, cross-GPU for B — until B has a set of its own.
    std::fs::write(g.join("BLESSED-GPU.txt"), format!("{a}\n")).unwrap();
    assert_eq!(golden_set_in(&g, a), GoldenSet { dir: g.clone(), strict: true, own: false });
    assert_eq!(golden_set_in(&g, b), GoldenSet { dir: g.clone(), strict: false, own: false });
    // A bless on B writes B's own set; on A, the reference; only the flag moves the reference.
    assert_eq!(bless_dir_in(&g, b, false), own_b);
    assert_eq!(bless_dir_in(&g, a, false), g);
    assert_eq!(bless_dir_in(&g, b, true), g);
    // B's set counts only when its marker names B (a folder alone, or one naming another card,
    // is not B's set).
    std::fs::create_dir_all(&own_b).unwrap();
    assert!(!golden_set_in(&g, b).own);
    std::fs::write(own_b.join("BLESSED-GPU.txt"), a).unwrap();
    assert!(!golden_set_in(&g, b).own);
    std::fs::write(own_b.join("BLESSED-GPU.txt"), b).unwrap();
    assert_eq!(golden_set_in(&g, b), GoldenSet { dir: own_b.clone(), strict: true, own: true });
    assert_eq!(golden_set_in(&g, a), GoldenSet { dir: g.clone(), strict: true, own: false });
    let _ = std::fs::remove_dir_all(&g);
}

#[test]
fn a_run_names_the_opt_in_groups_it_did_not_run() {
    assert_eq!(opt_in_not_run(None), vec!["deep-location", "fe-df32-probe"]);
    assert_eq!(opt_in_not_run(Some("deep-location")), vec!["fe-df32-probe"]);
    assert_eq!(opt_in_not_run(Some("probe")), vec!["deep-location"]);
    // A filter naming an ordinary group runs no opt-in group either.
    assert_eq!(opt_in_not_run(Some("goldens")), vec!["deep-location", "fe-df32-probe"]);
}
