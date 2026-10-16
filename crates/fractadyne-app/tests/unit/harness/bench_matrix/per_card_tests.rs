use super::*;

fn write(dir: &std::path::Path, file: &str, gpu: &str) {
    let b = Baseline { version: "t".into(), gpu: gpu.into(), utc: "t".into(), segments: Default::default() };
    std::fs::write(dir.join(file), serde_json::to_string(&b).unwrap()).unwrap();
}

#[test]
fn each_card_is_judged_against_its_own_baseline_or_none() {
    let tmp = std::env::temp_dir().join(format!("fd-bm-per-card-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let (a, b) = ("NVIDIA GeForce RTX 3080", "AMD Radeon RX 6800 XT");
    let own_b = tmp.join("bench-matrix-baseline-amd-radeon-rx-6800-xt.json");
    // Nothing blessed anywhere: no card has a baseline, and the first bless writes the reference.
    assert_eq!(baseline_in(&tmp, a), Ok(None));
    assert_eq!(bless_in(&tmp, b), reference_in(&tmp));
    // The reference blessed on A: it is A's, NOT B's (B used to be "compared" against it and pass).
    write(&tmp, REFERENCE_FILE, a);
    assert_eq!(baseline_in(&tmp, a), Ok(Some(reference_in(&tmp))));
    assert_eq!(baseline_in(&tmp, b), Ok(None));
    assert_eq!(bless_in(&tmp, a), reference_in(&tmp));
    assert_eq!(bless_in(&tmp, b), own_b);
    // B's own file, once it exists, is B's — and A's stays the reference.
    write(&tmp, "bench-matrix-baseline-amd-radeon-rx-6800-xt.json", b);
    assert_eq!(baseline_in(&tmp, b), Ok(Some(own_b.clone())));
    assert_eq!(bless_in(&tmp, b), own_b);
    assert_eq!(baseline_in(&tmp, a), Ok(Some(reference_in(&tmp))));
    // An unreadable reference is an error, not "no baseline".
    std::fs::write(reference_in(&tmp), "{not json").unwrap();
    assert!(baseline_in(&tmp, a).is_err());
    let _ = std::fs::remove_dir_all(&tmp);
}
