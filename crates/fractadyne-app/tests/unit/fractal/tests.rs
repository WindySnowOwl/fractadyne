use super::*;

/// The `spec()` O(1) index and `formula_id`-as-index invariant both depend on `SPECS` being in
/// exact `FractalKind` declaration order. Guard it so a mis-ordered/forgotten row can't ship.
#[test]
fn specs_cover_all_kinds_in_order() {
    assert_eq!(
        FractalKind::SPECS.len(),
        FractalKind::ALL.len() + 1 + FractalKind::AUTOMATA.len() + FractalKind::LSYSTEMS.len(),
        "every FractalKind needs exactly one SPECS row (the built-ins, Custom, the automata, the L-systems)"
    );
    let custom = &FractalKind::SPECS[FractalKind::ALL.len()];
    assert_eq!(custom.kind, FractalKind::Custom);
    assert_eq!(FractalKind::Custom as usize, FractalKind::ALL.len(), "spec() indexes by variant");
    assert_eq!(custom.formula_id, fractadyne_core::formula::CUSTOM);
    assert!(!custom.supports_perturbation, "a custom formula renders on the direct path");
    assert_eq!(custom.class, FractalClass::EscapeTime);
    for (k, kind) in FractalKind::AUTOMATA.iter().chain(&FractalKind::LSYSTEMS).enumerate() {
        let spec = &FractalKind::SPECS[FractalKind::ALL.len() + 1 + k];
        assert_eq!(spec.kind, *kind, "the automata's then the L-systems' rows follow Custom's, in list order");
        assert_eq!(*kind as usize, FractalKind::ALL.len() + 1 + k);
        assert_ne!(spec.class, FractalClass::EscapeTime);
        assert!(!spec.supports_julia && !spec.supports_perturbation);
        assert!(!fractadyne_core::is_valid_formula(spec.formula_id), "outside the escape-time ids");
    }
    assert_eq!(FractalKind::Life.formula_id(), fractadyne_core::formula::LIFE);
    assert_eq!(FractalKind::LSystem.formula_id(), fractadyne_core::formula::LSYSTEM);
    assert_eq!(FractalKind::LSystem.class(), FractalClass::LSystem);
    for (i, kind) in FractalKind::ALL.iter().enumerate() {
        let spec = &FractalKind::SPECS[i];
        assert_eq!(spec.kind, *kind, "SPECS row {i} is out of declaration order");
        assert_eq!(spec.class, FractalClass::EscapeTime, "{}", spec.name);
        assert_eq!(
            spec.formula_id as usize, i,
            "{}: formula_id must equal its index",
            spec.name
        );
    }
}

/// A blank `formula` or `about` would ship as an empty tooltip, which reads as a broken
/// menu rather than a missing sentence. Guard the whole table at once.
#[test]
fn every_family_has_a_hint_worth_showing() {
    let others = [FractalKind::Custom].into_iter().chain(FractalKind::AUTOMATA).chain(FractalKind::LSYSTEMS);
    for kind in FractalKind::ALL.into_iter().chain(others) {
        let (formula, about) = (kind.formula(), kind.about());
        assert!(!formula.trim().is_empty(), "{}: no formula", kind.name());
        assert!(!about.trim().is_empty(), "{}: no description", kind.name());
        let hint = kind.menu_hint();
        assert!(hint.contains(&formula), "{}: hint drops the formula", kind.name());
        assert!(hint.contains(&about), "{}: hint drops the description", kind.name());
    }
}

/// Newton is the one family with no perturbation path, and this hint is the only place
/// the interface says so. Pinned BY NAME on purpose: asserting the hint against
/// `supports_perturbation` looks stronger but is a tautology, since the hint is derived
/// from that same flag - flip the flag and both sides move together and the test still
/// passes. (Tried it; it did.)
#[test]
fn only_the_family_without_deep_zoom_says_so() {
    assert!(FractalKind::Newton.menu_hint().contains("no deep zoom"));
    assert!(!FractalKind::Mandelbrot.menu_hint().contains("no deep zoom"));
}

/// The pickers list every power family through `POWER_GROUPS` and every other one by itself:
/// together, each built-in exactly once.
#[test]
fn the_power_groups_list_each_power_family_once() {
    let grouped: Vec<FractalKind> = FractalKind::POWER_GROUPS.iter().flat_map(|(_, ks)| *ks).collect();
    let singles = FractalKind::ALL.iter().filter(|k| k.power_family().is_none()).count();
    assert_eq!(grouped.len() + singles, FractalKind::ALL.len());
    for (label, kinds) in FractalKind::POWER_GROUPS {
        for k in kinds {
            let d = k.power_family().expect("a power family");
            assert_eq!(k.name(), format!("{label} {d}"), "the row's label and power name the family");
            assert_eq!(fractadyne_core::formula::power(k.formula_id()), d);
        }
    }
}

/// Names are used as stable tokens in view files; they must round-trip and be unique.
#[test]
fn names_round_trip_and_are_unique() {
    for k in FractalKind::ALL.into_iter().chain([FractalKind::Custom]).chain(FractalKind::AUTOMATA).chain(FractalKind::LSYSTEMS) {
        assert_eq!(FractalKind::from_name(k.name()), Some(k));
    }
    let mut names: Vec<&str> = FractalKind::SPECS.iter().map(|s| s.name).collect();
    names.sort_unstable();
    let n = names.len();
    names.dedup();
    assert_eq!(names.len(), n, "duplicate fractal name");
}

/// In English the name a family shows is its token, word for word: translating the interface must
/// not change what an English user reads, and the token is what files and scripts hold.
#[test]
fn in_english_each_family_shows_its_token() {
    let all = FractalKind::ALL.into_iter().chain([FractalKind::Custom]).chain(FractalKind::AUTOMATA).chain(FractalKind::LSYSTEMS);
    for kind in all {
        assert_eq!(kind.display_name(), kind.name());
    }
    for (label, kinds) in FractalKind::POWER_GROUPS {
        for k in kinds {
            assert_eq!(k.family_name(), label, "{}", k.name());
        }
    }
}
