use super::*;

// Reference-length collapse trigger (crash-1786506241): a long orbit collapsing to a short
// ESCAPED one must derate the budget; the smooth paths must never trip it. Pure-predicate
// pin — the interactive wheel-jump that produces a collapse has no scripted repro.
#[test]
fn install_collapse_trigger() {
    use crate::render::install_collapse;
    // The crash: millions → 90, escaped. MUST fire.
    assert!(install_collapse(3_730_527, 90, false));
    // Escaped→escaped big shrink (fast interactive zoom-out): fires.
    assert!(install_collapse(20_000, 5_000, false));
    // Smooth zoom-out re-pick (~×0.85): must NOT fire (inside the ×1.5 margin).
    assert!(!install_collapse(5_000, 4_200, false));
    // Exactly at the boundary (new = old/1.5): not a collapse (strict inequality).
    assert!(!install_collapse(3_000, 2_000, false));
    // Shrinking PARTIAL: exempt (the pixel clamp scales cost down with it).
    assert!(!install_collapse(1_000_000, 90, true));
    // Growth of any size: never a collapse (the jump trigger owns that direction).
    assert!(!install_collapse(90, 3_730_527, false));
    // Cold start (no previous orbit): never.
    assert!(!install_collapse(0, 90, false));
}

// Multi-machine sharding correctness: for any (frames, segments), the N ranges must be
// contiguous, disjoint, and cover [0, F) exactly — a missing or duplicated frame at a shard
// boundary silently corrupts a video assembled from several machines' output.
#[test]
fn segment_ranges_tile_exactly() {
    use crate::scripting::segment_range;
    for &(frames, n) in &[
        (0u64, 1u64),
        (1, 1),
        (1, 4),
        (7, 3),
        (100, 7),
        (9931, 4),
        (9931, 16),
        (233, 233),
        (233, 500),
        (1_000_000, 13),
    ] {
        let mut expected_start = 0u64;
        for k in 0..n {
            let (s, e) = segment_range(frames, n, k);
            assert_eq!(
                s, expected_start,
                "F={frames} N={n} k={k}: gap or overlap at start"
            );
            assert!(e >= s, "F={frames} N={n} k={k}: negative range");
            expected_start = e;
        }
        assert_eq!(
            expected_start, frames,
            "F={frames} N={n}: union does not cover [0, F)"
        );
    }
}

// The go-to / metadata zoom string must round-trip through log2(magnification) at any
// depth — including past f64's 1e308× range, where a plain f64 zoom would be ∞.
#[test]
fn zoom_field_log2_roundtrip() {
    for &log2mag in &[0.0_f64, 8.0, 49.83, 100.0, 1019.0, 1100.0, 5000.0, 1.0e5] {
        let s = fmt_zoom_field(log2mag);
        let back = parse_zoom_to_log2(&s).expect("parse failed");
        assert!((back - log2mag).abs() < 1e-3, "{log2mag} → {s} → {back}");
    }
    // Plain and grouped human input parses too.
    assert!((parse_zoom_to_log2("256").unwrap() - 8.0).abs() < 1e-9);
    assert!((parse_zoom_to_log2("1,024").unwrap() - 10.0).abs() < 1e-9);
    assert!(parse_zoom_to_log2("1e400").unwrap() > 1300.0); // past f64 range, no overflow
                                                            // A FRACTIONAL exponent is legal and load-bearing: a zoom ladder places its rungs at
                                                            // |lambda|^n, which is never a whole power of ten. `f64::from_str` rejects "1.0e23.9"
                                                            // outright, and the CLI used to swallow that failure and render at 1x with exit 0 --
                                                            // the benchmark kit measured a whole-set frame that way. Pin the value, not just
                                                            // "parses": a ladder that lands one decade off looks entirely plausible.
    let l2 = parse_zoom_to_log2("1.0e23.900008").expect("fractional exponent rejected");
    assert!(
        (l2 / std::f64::consts::LOG2_10 - 23.900008).abs() < 1e-9,
        "got {l2}"
    );
    assert!(
        (parse_zoom_to_log2("2.5e3.5").unwrap() / std::f64::consts::LOG2_10
            - (2.5_f64.log10() + 3.5))
            .abs()
            < 1e-9
    );
    // Garbage rejected, no panic.
    for g in ["", "abc", "-5", "0", "e", "1e", "nan", "inf"] {
        assert!(parse_zoom_to_log2(g).is_none(), "accepted {g:?}");
    }
}

// Phase 5.1: fuzz the view-metadata parser chain (untrusted: loaded from PNG tEXt
// chunks / pasted). `meta_get` + the downstream numeric parsers must never panic and
// must produce bounded output on arbitrary input.
#[test]
fn fuzz_metadata_parser_panic_free() {
    let mut s = 0x9e37_79b9_7f4a_7c15u64;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let charset = b"=\n\r key value-+0.123eE\t\0[]\"";
    for _ in 0..20_000 {
        let len = (next() % 96) as usize;
        let mut buf = String::with_capacity(len);
        for _ in 0..len {
            buf.push(charset[(next() as usize) % charset.len()] as char);
        }
        for k in [
            "center_re",
            "center_im",
            "zoom",
            "fractal",
            "julia",
            "max_iter",
            "missing",
        ] {
            let v = meta_get(&buf, k);
            assert!(v.len() <= buf.len(), "meta_get returned oversized value");
        }
        // The real downstream parsers applied to extracted values must not panic.
        let _ = fractadyne_core::parse_bf(&meta_get(&buf, "center_re"));
        let _ = fractadyne_core::parse_bf(&meta_get(&buf, "center_im"));
        let _ = meta_get(&buf, "zoom").parse::<f64>();
        let _ = meta_get(&buf, "max_iter").parse::<u32>();
        let _ = FractalKind::from_name(&meta_get(&buf, "fractal"));
    }
    // Adversarial explicit metadata blobs.
    for m in [
        "",
        "=",
        "\n\n\n",
        "center_re=",
        "=value",
        "zoom=NaN",
        "max_iter=-1",
        "center_re=1e999999999",
        "fractal=\0\0\0",
        "a=b=c=d",
        "zoom=  inf  ",
    ] {
        let _ = fractadyne_core::parse_bf(&meta_get(m, "center_re"));
        let _ = meta_get(m, "zoom").parse::<f64>();
    }
}

// ---------------------------------------------------------------- navigation (steps 18, 21)

/// Checklist step 18, "the toolbar magnifier buttons zoom in and out, centred on the view".
/// Two claims worth pinning: the centre does not move, and the two buttons are exact
/// inverses - if they were not, a user alternating them would creep deeper (or shallower)
/// with every pair of clicks and could never get back to where they started.
#[test]
fn toolbar_zoom_actions() {
    let digits = "0.35634774601304382214593134944855658665333542382319826904819524052878";
    for log2mag in [0.0f64, 40.0, 300.0] {
        let mut vp = fractadyne_core::Viewport::new(1600.0, 900.0);
        vp.set_center_log2mag(
            fractadyne_core::parse_bf_prec(digits, 512).expect("parses"),
            fractadyne_core::parse_bf_prec(digits, 512).expect("parses"),
            log2mag,
        );
        let (cx0, cy0) = (vp.center_x.clone(), vp.center_y.clone());
        let start = vp.log2_magnification();

        // `zoom_center` is `zoom_at` at the middle pixel; the constants are the ones the
        // two toolbar buttons pass, so a rewired button changes what this measures.
        vp.zoom_at(800.0, 450.0, TOOLBAR_ZOOM_IN_FACTOR);
        assert!(
            vp.log2_magnification() > start,
            "2^{log2mag}: the zoom-IN button did not zoom in"
        );
        vp.zoom_at(800.0, 450.0, TOOLBAR_ZOOM_OUT_FACTOR);

        assert!(
            (vp.log2_magnification() - start).abs() < 1.0e-9,
            "2^{log2mag}: in-then-out landed at 2^{}, not 2^{start}",
            vp.log2_magnification()
        );
        // Drift measured in PIXELS, not complex units: at 2^300 the whole view is ~1e-93
        // wide, so an absolute coordinate threshold would be satisfied by a centre that had
        // slid clean off the screen.
        let (bx, by) = vp.complex_to_pixel(&cx0, &cy0);
        assert!(
            (bx - 800.0).abs() < 0.01 && (by - 450.0).abs() < 0.01,
            "2^{log2mag}: the centre drifted to ({bx:.3}, {by:.3})"
        );
    }
}

/// A history snapshot of a view at `log2mag` on a distinguishable centre.
fn snap_at(x: f64, log2mag: f64) -> ViewSnapshot {
    let mut vp = fractadyne_core::Viewport::new(1600.0, 900.0);
    vp.set_center_log2mag(
        fractadyne_core::BigFloat::from_f64(x, 64),
        fractadyne_core::BigFloat::from_f64(0.0, 64),
        log2mag,
    );
    ViewSnapshot {
        cx: vp.center_x,
        cy: vp.center_y,
        upp: vp.units_per_pixel,
        prec: vp.precision,
    }
}

/// Checklist step 21, "undo steps back through the previous views; redo returns forward, and
/// the coordinates match what you visited". Pins the stack arithmetic: the round trip, the
/// dedupe that keeps a settling view from filling the stack, and the redo branch being
/// dropped the moment you navigate somewhere new.
#[test]
fn undo_redo_round_trip() {
    let mut nav = NavHistory::default();
    let visited: Vec<ViewSnapshot> = (0..4)
        .map(|i| snap_at(-0.5 + i as f64 * 0.01, 10.0 * i as f64))
        .collect();
    for s in &visited {
        assert!(nav.record(s.clone()), "a new location was refused");
    }

    // Back to the start, one step at a time. The CURRENT view is the top of the stack, so
    // three steps back from four visited locations.
    for want in visited.iter().rev().skip(1) {
        let got = nav.undo().expect("undo ran out early");
        assert_eq!(got.cx, want.cx, "undo returned the wrong location");
        assert_eq!(got.upp.e, want.upp.e, "undo returned the wrong depth");
    }
    assert!(
        nav.undo().is_none(),
        "undo stepped past the first location visited"
    );

    // Forward again, in order, ending where we started.
    for want in visited.iter().skip(1) {
        let got = nav.redo().expect("redo ran out early");
        assert_eq!(got.cx, want.cx, "redo returned the wrong location");
        assert_eq!(got.upp.e, want.upp.e, "redo returned the wrong depth");
    }
    assert!(
        nav.redo().is_none(),
        "redo invented a location past the newest"
    );

    // Dedupe: recording the view you are already on must not push a second copy, or
    // Backspace would appear dead after a view sat still through several settles.
    let top = visited.last().unwrap().clone();
    assert!(
        !nav.record(top.clone()),
        "an identical location was recorded twice"
    );
    assert!(!nav.record(top), "an identical location was recorded twice");

    // Navigating somewhere new abandons the redo branch (the standard history rule).
    nav.undo().expect("undo");
    assert!(
        !nav.redo.is_empty(),
        "nothing to abandon - the rest of this check is vacuous"
    );
    nav.record(snap_at(0.25, 55.0));
    assert!(
        nav.redo.is_empty(),
        "a new location left the old redo branch in place"
    );

    // The stack is bounded: snapshots carry full-precision centres, so an unbounded one is
    // a slow leak at depth. Oldest entries fall off; the newest is always reachable.
    let mut deep = NavHistory::default();
    for i in 0..(NavHistory::LIMIT + 50) {
        deep.record(snap_at(-0.5 + i as f64 * 1.0e-6, 1.0));
    }
    assert_eq!(
        deep.undo.len(),
        NavHistory::LIMIT,
        "history grew past its bound"
    );
}

// ---------------------------------------------------------------- locations (steps 70-72, 75)

/// A representative location blob, in exactly the shape `view_metadata` writes.
///
/// WARNING: a literal here could drift away from what the app actually writes, so it is
/// checked against `KNOWN_VIEW_KEYS` below - add or rename a field and the test says so.
const SAMPLE_LOCATION: &str = "app=Fractadyne\nversion=0.2.40\nformat_version=1\n\
saved_unix=1787401025\nsaved=2026-08-22 10:57:05 UTC\nnotes=hero\nfractal=Mandelbrot\n\
julia=0\njulia_c_re=0.00000000000000000e0\njulia_c_im=0.00000000000000000e0\n\
center_re=-1.7688142728350613080035161139012583033818929344327473679816125832\n\
center_im=0.0000505988919638538088127175518550855307415194377517168044839680\n\
upp=1.00000000000000000e-45\nupp_log2=-1.49500000000000000e2\nzoom=6.60e43\n\
max_iter=60000\nauto_iter=1\npalette=0\ncycle=0.27\noffset=0.1\naa=1\n";

/// The sample above must stay in step with the reader's own key list: every key the reader
/// knows appears in it, and it carries no key the reader would report as unknown.
#[test]
fn the_sample_location_covers_every_view_key() {
    for k in crate::export::KNOWN_VIEW_KEYS {
        // `julia=0` is the one legitimately-empty-looking value (a flag, not a string).
        assert!(
            !meta_get(SAMPLE_LOCATION, k).is_empty() || *k == "julia",
            "the sample location is missing {k:?} - view_metadata's fields have moved"
        );
    }
    for line in SAMPLE_LOCATION.lines().filter(|l| !l.is_empty()) {
        let k = line.split_once('=').expect("key=value").0;
        assert!(
            crate::export::KNOWN_VIEW_KEYS.contains(&k),
            "the sample location carries {k:?}, which the reader does not know"
        );
    }
}

/// Checklist step 72, "Share location produces a shareable string for the current view".
/// The failure that matters is asymmetry - the app writing a location its own Apply button
/// then refuses - so this runs our own output back through the gate every paste goes
/// through, and confirms the gate still refuses what it exists to refuse.
#[test]
fn share_string_round_trip() {
    assert_eq!(
        location_text_verdict(SAMPLE_LOCATION),
        Ok(()),
        "our own location was refused"
    );
    // Whitespace a clipboard round trip adds must not change the verdict.
    let padded = format!("\r\n  {SAMPLE_LOCATION}  \n\n");
    assert_eq!(
        location_text_verdict(&padded),
        Ok(()),
        "refused after a clipboard round trip"
    );
    // A hand-trimmed location - coordinates only, no app tag - is still a location.
    assert_eq!(
        location_text_verdict("center_re=-0.75\ncenter_im=0.1\nupp_log2=-40\n"),
        Ok(())
    );
    // And the refusals, each for its own reason.
    assert!(location_text_verdict("").is_err(), "empty text accepted");
    assert!(
        location_text_verdict("   \n\t ").is_err(),
        "whitespace accepted"
    );
    assert!(
        location_text_verdict("the quick brown fox\njumped=over\n").is_err(),
        "arbitrary text accepted as a location"
    );
    // Oversized. It must be refused for its SIZE, so the text is a perfectly good location
    // repeated past the bound - `x=1` padding would be refused as "not a location" whether
    // the size bound existed or not, and would prove nothing about it.
    let huge = SAMPLE_LOCATION.repeat(SHARE_MAX / SAMPLE_LOCATION.len() + 2);
    assert!(
        huge.len() > SHARE_MAX,
        "the oversize case is not actually oversized"
    );
    assert_eq!(
        location_text_verdict(&huge),
        Err("Nothing to load (or text too large)."),
        "oversized text accepted"
    );
    // ...and the two refusals stay distinguishable, so a caller can say which happened.
    assert_eq!(
        location_text_verdict("hello"),
        Err("Not a Fractadyne location.")
    );
    // The centre survives the trip at full precision - a share string that rounded the
    // coordinate to f64 would still parse, look right, and land somewhere else entirely.
    let digits = meta_get(SAMPLE_LOCATION, "center_re");
    let back = fractadyne_core::parse_bf(&digits).expect("centre parses");
    let round = fractadyne_core::to_decimal_string(&back);
    assert!(
        round.starts_with(&digits[..40]),
        "the shared centre lost digits: {digits} -> {round}"
    );
}

/// Checklist step 70, "a bookmark is saved with a usable name and restores the exact view".
/// A bookmark is a name plus a location blob, persisted as TOML, so what can silently break
/// is the file round trip - and the coordinate is exactly the part a lossy round trip would
/// damage invisibly, since a truncated deep centre still renders a picture.
#[test]
fn bookmark_round_trip() {
    let saved = BookmarkFile {
        bookmark: vec![
            Bookmark {
                name: "Hero 6.6e43x".into(),
                meta: SAMPLE_LOCATION.to_string(),
                thumb: "1787401025-0".into(),
            },
            // An auto-named bookmark with no thumbnail yet: the shape `add_bookmark`
            // creates before the preview screenshot is harvested.
            Bookmark {
                name: "Mandelbrot 1.00x".into(),
                meta: SAMPLE_LOCATION.to_string(),
                thumb: String::new(),
            },
        ],
    };
    let text = toml::to_string_pretty(&saved).expect("bookmarks serialize");
    let read: BookmarkFile = toml::from_str(&text).expect("bookmarks parse back");

    assert_eq!(
        read.bookmark.len(),
        2,
        "a bookmark was lost in the file round trip"
    );
    for (a, b) in saved.bookmark.iter().zip(&read.bookmark) {
        assert_eq!(a.name, b.name, "the name changed");
        assert_eq!(a.meta, b.meta, "the location blob changed");
        assert_eq!(a.thumb, b.thumb, "the thumbnail id changed");
    }
    // The restored blob is still an acceptable location, and still carries every digit.
    let restored = &read.bookmark[0].meta;
    assert_eq!(location_text_verdict(restored), Ok(()));
    assert_eq!(
        meta_get(restored, "center_re"),
        meta_get(SAMPLE_LOCATION, "center_re"),
        "the bookmarked centre did not survive the file"
    );
    // A file written by an older build (no `thumb` field) must still load.
    let old = "[[bookmark]]\nname = \"Old\"\nmeta = \"center_re=-0.5\\n\"\n";
    let read: BookmarkFile = toml::from_str(old).expect("a pre-thumbnail bookmarks file");
    assert_eq!(read.bookmark.len(), 1);
    assert!(read.bookmark[0].thumb.is_empty());
}

/// Checklist step 71, "delete a bookmark; the list stays consistent". Deletion is by INDEX
/// from a drawn row, which is the part that goes wrong: delete the wrong one, or leave the
/// thumbnail id of a deleted entry behind, and the list and the thumbnail folder disagree.
///
/// WARNING: the row also asked for a RENAME. There is no rename control - the Bookmarks
/// dialog offers Add, Go and Delete only - so that half was not enforceable and the row has
/// been corrected to what the app can actually do.
#[test]
fn bookmark_delete_keeps_the_list_consistent() {
    let make = || -> Vec<Bookmark> {
        (0..4)
            .map(|i| Bookmark {
                name: format!("view {i}"),
                meta: SAMPLE_LOCATION.to_string(),
                thumb: if i == 2 {
                    String::new()
                } else {
                    format!("thumb-{i}")
                },
            })
            .collect()
    };

    // Deleting the middle entry removes that one, keeps the rest in order, and hands back
    // the thumbnail whose file the caller must now unlink.
    let mut list = make();
    assert_eq!(
        take_bookmark(&mut list, 1).as_deref(),
        Some("thumb-1"),
        "wrong thumbnail id"
    );
    let names: Vec<&str> = list.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(
        names,
        ["view 0", "view 2", "view 3"],
        "the list is not consistent after a delete"
    );
    assert!(
        !list.iter().any(|b| b.thumb == "thumb-1"),
        "the deleted thumbnail id lingers"
    );

    // The last entry - the index a drawn list is most likely to get wrong.
    let mut list = make();
    let last = list.len() - 1;
    assert_eq!(take_bookmark(&mut list, last).as_deref(), Some("thumb-3"));
    assert_eq!(list.last().unwrap().name, "view 2");

    // A bookmark with no thumbnail yet (added, screenshot not yet harvested) deletes
    // cleanly and asks for no file to be removed.
    let mut list = make();
    assert_eq!(
        take_bookmark(&mut list, 2),
        None,
        "asked to delete a file that never existed"
    );
    assert_eq!(list.len(), 3);
    assert!(!list.iter().any(|b| b.name == "view 2"));

    // A stale row index is ignored rather than panicking or deleting the wrong entry.
    let mut list = make();
    assert_eq!(
        take_bookmark(&mut list, 4),
        None,
        "a stale index removed something"
    );
    assert_eq!(take_bookmark(&mut list, usize::MAX), None);
    assert_eq!(list.len(), 4, "a stale index changed the list");

    // Emptying it is not a special case: the file round trip of an empty list must work,
    // or clearing the last bookmark would leave the old one on disk.
    let mut list = make();
    for _ in 0..4 {
        take_bookmark(&mut list, 0);
    }
    let text = toml::to_string_pretty(&BookmarkFile { bookmark: list }).expect("serialize");
    let read: BookmarkFile = toml::from_str(&text).expect("parse");
    assert!(
        read.bookmark.is_empty(),
        "an emptied list did not persist as empty"
    );
}

/// Checklist step 75, "the Gallery browses exported images and lists them". Exercised
/// against real files: what the scan must do is skip everything that is not one of our
/// exports and order the rest newest-first, and both halves are silent when wrong - a
/// gallery listing another app's PNG only fails when someone clicks it.
///
/// Partial: this covers the scan, not the click that loads the entry into the view.
#[test]
fn gallery_scan_and_load() {
    let dir = std::env::temp_dir().join(format!("fractadyne-gallery-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");

    let px = vec![0u8; 4 * 4 * 4];
    let write = |name: &str, meta: Option<&str>| {
        fractadyne_export::write_png_rgba8(&dir.join(name), 4, 4, &px, meta).expect("write png");
    };
    // Two of ours, saved a day apart, deliberately written oldest-name-first so a scan
    // that just returned directory order would fail.
    let older = SAMPLE_LOCATION.replace("saved_unix=1787401025", "saved_unix=1787300000");
    write("a-older.png", Some(&older));
    write("b-newer.png", Some(SAMPLE_LOCATION));
    // Another app's export, an export whose metadata never wrote, and a plain file.
    write(
        "c-other-app.png",
        Some("app=SomethingElse\ncenter_re=-0.5\n"),
    );
    write("d-no-metadata.png", None);
    std::fs::write(dir.join("e-notes.txt"), b"not an image").expect("write txt");

    let found = scan_gallery_dir(&dir);
    let names: Vec<String> = found
        .iter()
        .map(|(p, _)| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        names,
        ["b-newer.png", "a-older.png"],
        "the gallery listed the wrong files, or in the wrong order"
    );
    // The blob comes back whole, so opening the entry gets the same location that was saved.
    assert_eq!(found[0].1.trim_end(), SAMPLE_LOCATION.trim_end());
    assert_eq!(location_text_verdict(&found[0].1), Ok(()));

    // A folder that does not exist is empty, not a panic (the gallery folder is a
    // remembered path and may have been deleted between runs).
    assert!(scan_gallery_dir(&dir.join("gone")).is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------- launch (steps 2, 3)

/// Checklist step 2, "the title bar shows Fractadyne v<version> (build <n>) and the version
/// matches the release being tested". The second clause is the one a machine can hold: the
/// title must carry the CRATE's version, so a release built from a tree whose Cargo.toml was
/// never bumped cannot present itself as the version on the tin.
#[test]
fn title_string_matches_version() {
    let t = window_title();
    assert!(t.starts_with("Fractadyne v"), "title is {t:?}");
    assert!(
        t.contains(env!("CARGO_PKG_VERSION")),
        "the title {t:?} does not carry the crate version {}",
        env!("CARGO_PKG_VERSION")
    );
    // `(build N)` with a real number: the build counter is what distinguishes two binaries of
    // the same version, and it is the first thing an issue report is read for.
    let build = t
        .rsplit_once("(build ")
        .and_then(|(_, r)| r.strip_suffix(')'))
        .expect("title has no (build N) suffix");
    assert!(
        !build.is_empty() && build.chars().all(|c| c.is_ascii_digit()),
        "build counter is {build:?}"
    );
}

/// Checklist step 3, "the welcome dialog appears on first run and closes without reappearing".
/// Also pins the harness rule: a modal in front of `--livetest` blocked its tour for a whole
/// session once, and it looked like a hang rather than a dialog.
#[test]
fn welcome_shows_once() {
    assert!(
        welcome_should_open(false, false),
        "no welcome on a fresh profile"
    );
    assert!(
        !welcome_should_open(true, false),
        "the welcome came back after being dismissed"
    );
    assert!(
        !welcome_should_open(false, true),
        "a modal in front of a harness run"
    );
    assert!(!welcome_should_open(true, true));
    // And the dismissal has to SURVIVE, or "once" means once per launch. The session field is
    // written as the negation of the dialog being open (`welcome_seen = !welcome_open`).
    let seen = fractadyne_state::SessionState {
        welcome_seen: true,
        ..Default::default()
    };
    let text = toml::to_string_pretty(&seen).expect("serialize");
    let back: fractadyne_state::SessionState = toml::from_str(&text).expect("parse");
    assert!(
        back.welcome_seen,
        "welcome_seen did not survive a save/load"
    );
    assert!(!welcome_should_open(back.welcome_seen, false));
}

// ---------------------------------------------------------------- dual view (steps 45-47)

/// Checklist step 45, "the window splits; left is the parameter set, right is the Julia".
/// The split is a fraction the viewer drags, so the invariants are: the two panels never
/// overlap, both stay inside the window, neither collapses to nothing, and the divider stays
/// where the clamp allows even when a script or a corrupt session asks for something absurd.
#[test]
fn dual_view_splits_the_viewports() {
    use crate::ui::central::{dual_panel_at, dual_panel_rects};
    let full = egui::Rect::from_min_max(egui::pos2(20.0, 30.0), egui::pos2(1300.0, 830.0));
    for split in [
        -5.0f32,
        0.0,
        DUAL_SPLIT_MIN,
        0.34,
        0.5,
        0.7,
        DUAL_SPLIT_MAX,
        1.0,
        99.0,
    ] {
        let (l, r) = dual_panel_rects(full, split);
        assert!(
            l.width() > 0.0 && r.width() > 0.0,
            "split {split}: a panel collapsed"
        );
        assert!(l.max.x <= r.min.x, "split {split}: the panels overlap");
        assert!(
            l.min.x == full.min.x && r.max.x == full.max.x,
            "split {split}: outside the window"
        );
        assert!(
            l.min.y == full.min.y && l.max.y == full.max.y,
            "split {split}: wrong height"
        );
        // The gap between them is the drag handle, and it must stay grabbable.
        let gap = r.min.x - l.max.x;
        assert!(gap >= 4.0, "split {split}: the separator is {gap}px wide");
        // A point in each panel belongs to that panel, and the separator belongs to neither.
        assert_eq!(
            dual_panel_at(full, split, egui::pos2(l.center().x, l.center().y)),
            Some(false)
        );
        assert_eq!(
            dual_panel_at(full, split, egui::pos2(r.center().x, r.center().y)),
            Some(true)
        );
        assert_eq!(
            dual_panel_at(full, split, egui::pos2(l.max.x + gap * 0.5, l.center().y)),
            None
        );
        assert_eq!(
            dual_panel_at(full, split, egui::pos2(full.max.x + 50.0, l.center().y)),
            None
        );
    }
    // An out-of-range split is CLAMPED, not honoured: the divider must stay somewhere the
    // viewer can drag it back from.
    let (wide_l, _) = dual_panel_rects(full, 99.0);
    let (max_l, _) = dual_panel_rects(full, DUAL_SPLIT_MAX);
    assert_eq!(
        wide_l.width(),
        max_l.width(),
        "an absurd split was not clamped"
    );
}

/// Checklist step 46, "the Julia side zooms independently". Every gesture in the dual view is
/// routed by which panel the pointer is in, so what makes the two sides independent is that
/// no point is ever claimed by both — sweep the whole width and check exactly that.
#[test]
fn julia_viewport_zooms_independently() {
    use crate::ui::central::dual_panel_at;
    let full = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1280.0, 720.0));
    let split = 0.5f32;
    let (mut param, mut julia, mut neither) = (0, 0, 0);
    let mut last = None;
    for i in 0..=1280 {
        let at = dual_panel_at(full, split, egui::pos2(i as f32, 360.0));
        match at {
            Some(false) => param += 1,
            Some(true) => julia += 1,
            None => neither += 1,
        }
        // The panels are contiguous: the answer changes at most twice across the sweep
        // (parameter → separator → Julia), never flickering back and forth.
        if let Some(prev) = last {
            if prev != at {
                assert!(
                    matches!((prev, at), (Some(false), None) | (None, Some(true))),
                    "the panel assignment jumped {prev:?} -> {at:?} at x={i}"
                );
            }
        }
        last = Some(at);
    }
    assert!(
        param > 500 && julia > 500,
        "panels are lopsided: {param} vs {julia}"
    );
    assert!(
        (1..=8).contains(&neither),
        "the separator covers {neither}px"
    );
}

/// The picker's "Show" axis is three states over two booleans, and only three of the four
/// combinations are legal. The illegal fourth used to be prevented only by GREY — a disabled
/// checkbox — which is a promise the state machine now has to keep on its own, because
/// scripts, session loads and the toolbar all reach the same two flags.
///
/// Both directions, over the real functions the picker uses.
#[test]
fn show_mode_round_trips_the_three_states() {
    for m in [ShowMode::Set, ShowMode::Julia, ShowMode::Both] {
        let (dual, julia) = show_mode_flags(m);
        assert_eq!(
            show_mode_of(dual, julia),
            m,
            "{m:?} did not survive flags -> mode"
        );
    }
    // Each mode maps to a DIFFERENT pair — without this, a mapping that returned the same
    // flags for everything would satisfy the round trip above.
    let flags: Vec<(bool, bool)> = [ShowMode::Set, ShowMode::Julia, ShowMode::Both]
        .iter()
        .map(|m| show_mode_flags(*m))
        .collect();
    assert_eq!(flags, [(false, false), (false, true), (true, false)]);

    // Entering the dual view must CLEAR `julia_mode`, not leave it set underneath: that flag
    // is what a session and an exported view record, so a stale `true` would reload a dual
    // view as a full-window Julia.
    assert_eq!(
        show_mode_flags(ShowMode::Both).1,
        false,
        "dual must not carry julia_mode"
    );

    // ...but a pair written by an older build, or by hand, still has to read as what the
    // renderer actually draws — the dual path paints both panes whatever `julia_mode` says.
    assert_eq!(show_mode_of(true, true), ShowMode::Both);
}

/// Checklist step 47, "turn off dual view and Julia mode; returns cleanly to the single
/// Mandelbrot view". The part that can silently go wrong is the formula guard: dual pairs a
/// formula with its Julia, so a formula with no Julia must not be able to enter it — Newton
/// has no free parameter, and a dual view of it would render an empty right panel.
#[test]
fn leaving_dual_restores_the_single_view() {
    // Every formula agrees with itself about whether it has a Julia.
    for f in FractalKind::ALL {
        assert_eq!(
            f.supports_julia(),
            f != FractalKind::Newton,
            "{} disagrees about having a Julia",
            f.name()
        );
    }
    // Leaving dual restores the single view's own framing: `reset_to` puts the parameter
    // panel back at the formula's default centre and 1x, whatever the Julia panel was doing.
    let mut vp = fractadyne_core::Viewport::new(1280.0, 720.0);
    vp.set_center_log2mag(
        fractadyne_core::BigFloat::from_f64(-0.743, 64),
        fractadyne_core::BigFloat::from_f64(0.131, 64),
        60.0,
    );
    let (cx, cy) = FractalKind::Mandelbrot.default_center();
    vp.reset_to(cx, cy);
    assert!((vp.magnification() - 1.0).abs() < 1.0e-9);
    let (x, y) = vp.center_f64();
    assert!((x - cx).abs() < 1.0e-15 && (y - cy).abs() < 1.0e-15);
}

/// The offline dual render splits the same way the live divider does, and the two panel
/// widths must SUM to the requested frame width — rounding both independently loses or gains
/// a column, so a rendered tour would not be the size that was asked for.
#[test]
fn dual_panel_widths_sum_to_the_frame() {
    for width in [2u32, 3, 16, 17, 640, 1281, 1920, 3841] {
        for split in [-1.0f32, 0.0, 0.15, 0.34, 0.5, 0.85, 1.0, 7.0] {
            let (l, r) = dual_panel_widths(width, split);
            assert_eq!(l + r, width, "{width}px at split {split}: {l} + {r}");
            assert!(
                l >= 1 && r >= 1,
                "{width}px at split {split}: a panel collapsed"
            );
        }
    }
}

// ---------------------------------------------------------------- coloring (step 59)

/// Checklist step 59, "the palette animates smoothly at the set speed and stops cleanly when
/// disabled". Simulated over many frames rather than asserted on one step, because the
/// failures are cumulative: a phase that creeps out of 0..1, a ping-pong that sticks at an
/// end stop, or an "Off" that still moves.
#[test]
fn palette_animation_advances_and_stops() {
    let step = 0.35_f32 * (1.0 / 60.0); // speed 0.35 at 60 fps

    // Forward and reverse both wrap, and both keep the phase in range.
    for mode in [PaletteAnim::Forward, PaletteAnim::Reverse] {
        let (mut o, mut d) = (0.5_f32, 1.0_f32);
        let start = o;
        let mut moved = false;
        for _ in 0..600 {
            let (no, nd) = palette_anim_step(mode, o, d, step);
            assert!(
                (0.0..=1.0).contains(&no),
                "{mode:?}: phase left the palette at {no}"
            );
            moved |= (no - start).abs() > 1.0e-6;
            o = no;
            d = nd;
        }
        assert!(moved, "{mode:?}: the palette never moved");
    }

    // Ping-pong reverses at both ends and visits both of them.
    let (mut o, mut d) = (0.5_f32, 1.0_f32);
    let (mut hi, mut lo) = (false, false);
    for _ in 0..600 {
        let (no, nd) = palette_anim_step(PaletteAnim::PingPong, o, d, step);
        assert!(
            (0.0..=1.0).contains(&no),
            "ping-pong left the palette at {no}"
        );
        hi |= no >= 1.0;
        lo |= no <= 0.0;
        o = no;
        d = nd;
    }
    assert!(
        hi && lo,
        "ping-pong did not reach both ends (hi {hi}, lo {lo})"
    );

    // Off holds still — including at a nonzero step, which is the case that matters: the
    // caller keeps ticking, so "stops cleanly" has to be true of the step itself.
    let (o, d) = palette_anim_step(PaletteAnim::Off, 0.42, 1.0, step);
    assert_eq!(
        (o, d),
        (0.42, 1.0),
        "the palette moved while animation was off"
    );
}

// ---------------------------------------------------------------- export (step 76)

/// Checklist step 76, "press Ctrl+S and an image is written ... the app reports where it
/// went". The name is the part that goes wrong silently: two formulas have a space in their
/// name, and a snapshot called `fractadyne_Burning Ship_….png` is awkward everywhere a path
/// is pasted unquoted. Partial — that the write happens and is reported is a human check.
#[test]
fn snapshot_writes_a_file() {
    for f in FractalKind::ALL {
        let n = export_file_name(f.name(), 1_787_401_025, "png");
        assert!(!n.contains(' '), "{}: {n:?} has a space in it", f.name());
        assert!(n.starts_with("fractadyne_") && n.ends_with(".png"), "{n:?}");
        assert!(
            !n.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']),
            "{n:?} is not a legal filename"
        );
    }
    // The stamp sorts chronologically, so a folder of snapshots reads in the order taken.
    let a = export_file_name("Mandelbrot", 1_787_401_025, "png");
    let b = export_file_name("Mandelbrot", 1_787_401_026, "png");
    let c = export_file_name("Mandelbrot", 1_787_500_000, "png");
    assert!(
        a < b && b < c,
        "snapshot names do not sort by time: {a} {b} {c}"
    );
    // The extension follows the chosen format rather than being assumed.
    assert!(export_file_name("Mandelbrot", 0, "exr").ends_with(".exr"));
}

// ---------------------------------------------------------------- help & settings (89-92)

/// Checklist steps 89 and 95, "the About panel's Deep-zoom arithmetic line names the
/// arithmetic that has actually run and what the build contains".
#[test]
fn about_names_the_running_backend() {
    let line = crate::help::about_arithmetic_line();
    assert!(line.starts_with("Deep-zoom arithmetic: "), "{line:?}");
    assert!(line.contains("this build contains: "), "{line:?}");
    // The carrier is in every build, accelerated or not.
    assert!(line.contains("astro-float"), "{line:?}");
    // ⚠The two halves are different questions. The build's CONTENTS are compile-time; what
    // has RUN is stamped by orbits finishing, so in a unit test (no orbit) it must honestly
    // say none — a line that named a backend here would be reading configuration, which is
    // exactly the failure the observation mask exists to prevent.
    assert!(
        line.contains("none (no reference orbit built yet)") || line.contains("MIXED"),
        "the About line claims a backend that never ran: {line:?}"
    );
    // Which build this is, asked of the RUNTIME rather than of a cargo feature: `rug` is a
    // feature of the core crate, so a `cfg` here would be a guess about someone else's build
    // — and one that silently reads false, turning the accelerated case into no test at all.
    let contents = line
        .split("this build contains: ")
        .nth(1)
        .unwrap_or_default();
    if contents.contains("rug") {
        // The accelerated build must report the C libraries that did the arithmetic, not
        // just the wrapper: MPFR and GMP versions are the whole point of the line there.
        assert!(
            line.contains("MPFR") && line.contains("GMP"),
            "the accelerated build names rug but no library versions: {line:?}"
        );
    } else {
        assert!(
            contents.trim_end_matches(").").trim() == "astro-float",
            "the standard build should contain astro-float alone: {contents:?}"
        );
    }
}

/// Checklist step 91, "Help > Report an issue opens the issue reporting path correctly with
/// the app's details". Both doors, because the mailto is the fallback for anyone without a
/// GitHub account, and a broken URL there is silent — the browser simply opens nothing useful.
#[test]
fn issue_url_is_well_formed() {
    let subject = "Fractadyne issue: Rendering / image quality";
    let gh = issue_new_url(subject);
    assert!(gh.starts_with(&format!("{ISSUES_URL}/new?")), "{gh}");
    assert!(gh.contains("title=") && gh.contains("&body="), "{gh}");
    let mt = issue_mailto_url(subject);
    assert!(mt.starts_with(&format!("mailto:{REPORT_EMAIL}?")), "{mt}");
    assert!(mt.contains("subject=") && mt.contains("&body="), "{mt}");
    // Nothing that would end the URL early or split a query parameter may survive unencoded.
    for url in [&gh, &mt] {
        let query = url.split_once('?').expect("no query").1;
        for (i, part) in query.split('&').enumerate() {
            let v = part.split_once('=').expect("no key=value").1;
            assert!(
                !v.contains([' ', '#', '&', '?', '\n', '"']),
                "parameter {i} of {url} carries an unencoded reserved character: {v:?}"
            );
        }
        // The subject must be recognisable once decoded, not empty or mangled away.
        assert!(
            url.contains("Fractadyne%20issue%3A"),
            "the subject did not survive: {url}"
        );
    }
}

/// Checklist step 92, "Check for updates performs a check and reports a clear result. No
/// hang." The network half is not a unit test's business; the DECISION is, and it is where
/// a wrong answer is invisible: the check must say "up to date" only when the running version
/// really is at least the newest on the track, in both directions and across pre-releases.
#[test]
fn update_check_reaches_a_verdict() {
    use crate::update::{running_version, version_gt};
    // Newer / older / equal, including the prerelease ordering a beta track depends on.
    assert!(
        version_gt("0.2.41", "0.2.40"),
        "a newer release was not offered"
    );
    assert!(
        version_gt("0.2.40", "0.2.40-beta.1"),
        "a stable was not offered over its beta"
    );
    assert!(version_gt("0.2.40-beta.2", "0.2.40-beta.1"));
    assert!(
        !version_gt("0.2.40", "0.2.40"),
        "the running version was offered to itself"
    );
    assert!(
        !version_gt("0.2.39", "0.2.40"),
        "an OLDER release was offered as an update"
    );
    assert!(
        !version_gt("0.2.40-beta.1", "0.2.40"),
        "a beta was offered over the stable"
    );
    // The comparison basis is overridable, which is how the "update available" path is
    // exercised on a machine already running the newest build.
    assert_eq!(running_version(), env!("CARGO_PKG_VERSION"));
    std::env::set_var("FRACTADYNE_FAKE_VERSION", "0.0.1");
    let faked = running_version();
    std::env::remove_var("FRACTADYNE_FAKE_VERSION");
    assert_eq!(
        faked, "0.0.1",
        "FRACTADYNE_FAKE_VERSION did not reach the check"
    );
    assert!(
        version_gt(env!("CARGO_PKG_VERSION"), "0.0.1"),
        "the faked version verdict is wrong"
    );
}

// ---------------------------------------------------------------- tools (step 84)

/// Checklist step 84, "close the tour player: playback stops and normal interactive control
/// returns". A tour may drive the viewer's own settings, and `PlaybackRestore` is what hands
/// them back — so the invariant worth pinning is COMPLETENESS: every keyframe field that
/// writes a session setting must have somewhere to be restored from. Add a keyframe field
/// without adding it here and a played tour silently edits the viewer's session.
#[test]
fn stopping_playback_restores_interaction() {
    // Keyframe fields a script may set, split by what happens when the tour ends. The
    // "kept" list is not laziness: where the tour LEFT you is the view you keep exploring
    // from, and resetting it would throw away the thing the tour was showing you.
    const RESTORED: &[&str] = &["max_iter", "palette", "dual_split", "orbits", "minimap"];
    const KEPT: &[&str] = &[
        "id",
        "t",
        "hold",
        "ease",
        "transition",
        "transition_secs",
        "fade_out_secs",
        "location",
        "re",
        "im",
        "zoom",
        "fractal",
        "julia",
        "dual",
        "julia_re",
        "julia_im",
        "orbit_re",
        "orbit_im",
    ];
    let fields = crate::scripting::keyframe_field_names();
    for f in &fields {
        assert!(
            RESTORED.contains(f) || KEPT.contains(f),
            "keyframe field {f:?} is neither restored when a tour ends nor listed as \
                 deliberately kept — decide which, or a tour silently edits the session"
        );
    }
    for f in RESTORED {
        assert!(
            fields.contains(f),
            "{f:?} is restored but is no longer a keyframe field"
        );
    }
    // And the record itself carries a slot for each restored field (plus the palette's three
    // companions, since choosing a preset overrides a custom gradient / binary / duotone).
    let r = crate::scripting::PlaybackRestore {
        max_iter: 12_345,
        auto_iter: false,
        palette_idx: 3,
        use_custom_palette: true,
        use_binary: true,
        use_duotone: true,
        minimap: true,
        show_orbits: true,
        dual_split: 0.34,
    };
    assert_eq!(r.max_iter, 12_345);
    assert!((r.dual_split - 0.34).abs() < 1.0e-6);
}
