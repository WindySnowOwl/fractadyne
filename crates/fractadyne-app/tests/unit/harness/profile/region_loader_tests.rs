use super::load_regions;

fn load(body: &str) -> Result<Vec<super::ProfRegion>, String> {
    let dir = std::env::temp_dir().join("fd-region-tests");
    std::fs::create_dir_all(&dir).unwrap();
    // Unique file per call, or parallel tests read each other's bodies.
    let path = dir.join(format!(
        "r-{}-{:?}.toml",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::write(&path, body).unwrap();
    let r = load_regions(&path);
    let _ = std::fs::remove_file(&path);
    r
}


fn region(extra: &str) -> String {
    format!(
        "[[region]]\nname = \"t\"\ncx = \"-0.5\"\ncy = \"0.0\"\nzoom_log2 = 10.0\n{extra}"
    )
}

/// ⭐The measured failure: a 4,031-digit e4000 centre was silently `take(2048)`-truncated, the
/// picked reference landed ~1e2000 view-widths off the view, and the profile printed
/// `ref 0.31 ms` for a 400-second build. Deep centres are the regions file's whole purpose —
/// every digit must survive the trip.
#[test]
fn a_deep_centre_round_trips_every_digit() {
    let cx = format!("-0.{}", "123456789".repeat(500)); // 4,502 digits — past the old cap
    let body = format!(
        "[[region]]\nname = \"deep\"\ncx = \"{cx}\"\ncy = \"0.0\"\nzoom_log2 = 13288.0\niter = 2008192\n"
    );
    let r = load(&body).expect("a deep region must load");
    assert_eq!(r[0].cx, cx, "the centre lost digits on the way in");
    // ...and the iteration ask survives too: the old clamp(64, 200_000) silently rewrote
    // 2,008,192 — while the BUILT-IN deep-interior region uses 800,000, which a file could
    // not even express.
    assert_eq!(r[0].iter, 2_008_192);
}

/// Out of range is an error, never a silent adjustment — the caller exits on Err, so the wrong
/// profile is never produced at all.
#[test]
fn out_of_range_input_is_refused_not_reshaped() {
    let e = load(&region("iter = 20000001\n")).expect_err("iter past MAX_ITER_LIMIT");
    assert!(e.contains("iter"), "error should name the field: {e}");
    let giant = format!("cx = \"0.{}\"", "9".repeat(100_001));
    let body = format!("[[region]]\nname = \"g\"\n{giant}\ncy = \"0.0\"\n");
    let e = load(&body).expect_err("an implausible centre length");
    assert!(e.contains("cx"), "error should name the field: {e}");
}

/// The e21000 and e4000 region files in the repo/scratchpad load — guarding the shipped
/// example against schema drift (unknown keys are silently ignored by serde, so a renamed
/// field would otherwise turn a deep region into a shallow default without a sound).
#[test]
fn the_shipped_extreme_region_still_loads_deep() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let r = load_regions(&root.join("validation/extreme-zoom.toml")).expect("shipped file");
    assert_eq!(r.len(), 1);
    assert!(r[0].zoom_log2 > 60_000.0, "the e21000 region lost its depth");
}
