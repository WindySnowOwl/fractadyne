use super::*;

fn hello(ver: &str, git: &str, dev: bool, tunables: &str) -> Hello {
    Hello {
        protocol: fractadyne_farm::PROTOCOL_VERSION,
        app_version: ver.into(),
        git: git.into(),
        allow_dirty: dev,
        name: "PLUTO".into(),
        tunables: tunables.into(),
        policy: Policy { max_width: 3840, max_height: 2160, max_ss: 4, max_iter: 10_000_000 },
        clock_unix_ms: 0,
    }
}

/// ⭐The gate the user chose: exact version AND commit; a dirty build never in a production farm.
#[test]
fn only_the_exact_build_joins() {
    let ok = hello("0.3.0-beta.18", "gabc1234", false, "stock");
    assert_eq!(admission_refusal(&ok, "0.3.0-beta.18", "gabc1234", false), None);
    // Same version label, different commit — the case that motivated the exact-commit gate.
    let r = admission_refusal(&hello("0.3.0-beta.18", "gdef5678", false, "stock"), "0.3.0-beta.18", "gabc1234", false).expect("refused");
    assert!(r.contains("version mismatch") && r.contains("gdef5678") && r.contains("gabc1234"), "{r}");
    assert!(admission_refusal(&hello("0.3.0-beta.17", "gabc1234", false, "stock"), "0.3.0-beta.18", "gabc1234", false).is_some());
    let mut old_protocol = ok.clone();
    old_protocol.protocol += 1;
    assert!(admission_refusal(&old_protocol, "0.3.0-beta.18", "gabc1234", false).unwrap().contains("protocol"));
}

#[test]
fn a_dirty_build_needs_the_development_flag_on_both_ends() {
    let dirty = hello("v", "gabc-dirty", false, "stock");
    assert!(admission_refusal(&dirty, "v", "gabc-dirty", false).unwrap().contains("-dirty"));
    assert!(admission_refusal(&dirty, "v", "gabc-dirty", true).is_some(), "the client did not ask for it");
    let both = hello("v", "gabc-dirty", true, "stock");
    assert!(admission_refusal(&both, "v", "gabc-dirty", false).is_some(), "the controller did not ask for it");
    assert_eq!(admission_refusal(&both, "v", "gabc-dirty", true), None);
}

/// Tunable overrides change pictures: never admitted. An instrument (the farmtest fault) only in
/// development mode.
#[test]
fn changed_tunables_never_join_and_instruments_only_in_development() {
    let overridden = hello("v", "g", true, "1 OVERRIDE(S) — BLA_EPS 0.000001 → 0.001");
    assert!(admission_refusal(&overridden, "v", "g", true).unwrap().contains("tunables"));
    let instrumented = hello("v", "g", true, "INSTRUMENT FRACTADYNE_FARM_CORRUPT_FRAMES=2");
    assert_eq!(admission_refusal(&instrumented, "v", "g", true), None);
    let prod = hello("v", "g", false, "INSTRUMENT FRACTADYNE_FARM_CORRUPT_FRAMES=2");
    assert!(admission_refusal(&prod, "v", "g", false).is_some());
}
