//! Every delayed repaint goes through `diag::repaint_after`, so the budget tripwire knows how long
//! the app chose to sleep before the next frame (2026-11-18: the settled beat's ~1 s waits read as
//! lethal-band frames and reset the budgets, 36 of 54 BLIND events in the maintainer's log).

use std::path::Path;

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).expect("src is readable").flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn every_delayed_repaint_goes_through_diag_repaint_after() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let (mut wrapped, mut bare) = (0, Vec::new());
    for f in &files {
        let text = std::fs::read_to_string(f).expect("a source file reads");
        let own = f.ends_with("diag.rs") && f.parent().is_some_and(|d| d == src);
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            wrapped += code.matches("diag::repaint_after(").count();
            // The wrapper's own call is the one bare call there may be.
            if code.contains(".request_repaint_after") && !own {
                bare.push(format!("{}:{}", f.strip_prefix(&src).unwrap_or(f).display(), n + 1));
            }
        }
    }
    // Not vacuous: the app has 20 of them (2026-11-18).
    assert!(wrapped >= 15, "found only {wrapped} wrapped repaints: did the scan miss the sources?");
    assert!(bare.is_empty(), "delayed repaints outside diag::repaint_after: {bare:?}");
}

#[test]
fn the_planned_sleep_is_the_last_frames_shortest_ask() {
    // One sequential test: the frame statics are process-wide.
    let ctx = eframe::egui::Context::default();
    let ms = std::time::Duration::from_millis;
    // A frame that asked two delays: the sleep is the shorter, read in the NEXT frame.
    let f = super::frame_begin(&ctx);
    super::repaint_after(&ctx, ms(984));
    super::repaint_after(&ctx, ms(250));
    drop(f);
    assert!((super::planned_sleep_ms(false) - 250.0).abs() < 1e-9);
    // …unless that frame's final ask was immediate: then nothing was planned.
    assert_eq!(super::planned_sleep_ms(true), 0.0);
    // A frame that asked no delay through the wrapper plans none (egui's own asks count as owed).
    let f = super::frame_begin(&ctx);
    drop(f);
    assert_eq!(super::planned_sleep_ms(false), 0.0);
    // The record is per frame: the next frame starts clean.
    let f = super::frame_begin(&ctx);
    super::repaint_after(&ctx, ms(1_000));
    drop(f);
    let f = super::frame_begin(&ctx);
    drop(f);
    assert_eq!(super::planned_sleep_ms(false), 0.0);
}
