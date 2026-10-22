use super::{series_collapse, series_share, shader_series, short_escaper_ask, SERIES_COLLAPSE_SLACK};

// crash-1792627197 (2026-10-21), from its frame record: an escaped reference of 298,416 samples,
// built for asks of 636,597–702,557, carrying a 298,404-iteration series skip.
const LEN: u32 = 298_416;
const SKIP: u32 = 298_404;

#[test]
fn the_crash_frames_skip_was_the_references_to_keep() {
    // Frame 27938 (ask 592,562 + 8,192) kept the skip; frame 27939 (490,875 + 8,192) dropped it —
    // the live rule read the FRAME's ask. Every reference of that glide was built for an ask well
    // past twice its length, so the reference's own rule keeps the series on both frames.
    assert!(short_escaper_ask(LEN, 592_562 + 8_192), "frame 27938's ask");
    assert!(!short_escaper_ask(LEN, 490_875 + 8_192), "frame 27939's ask: the flip");
    for built_for in [702_557, 636_597] {
        assert!(short_escaper_ask(LEN, built_for), "the reference built for {built_for}");
    }
    // The threshold is twice the orbit, exclusive.
    assert!(!short_escaper_ask(LEN, 2 * LEN));
    assert!(short_escaper_ask(LEN, 2 * LEN + 1));
}

#[test]
fn the_crash_frames_skip_collapse_is_caught() {
    // Frame 27938: the skip applied, every pixel escaping within ~70 iterations of it.
    let (_, it, skip) = shader_series(false, true, false, 592_562, LEN, SKIP);
    assert_eq!((it, skip), (592_562, SKIP));
    let before = series_share(it, LEN, skip);
    assert!(before < 1.0e-4, "a dozen of 592,562 iterations: {before}");
    // Frame 27939 without it: the pixels' share of the nominal iterations is now 0.61.
    let (_, it, skip) = shader_series(false, true, false, 490_875, LEN, 0);
    let after = series_share(it, LEN, skip);
    assert!((after - 298_416.0 / 490_875.0).abs() < 1.0e-6);
    assert!(series_collapse(Some(before), after), "{before} → {after}");
    // With the skip kept, the same frame is the same regime.
    let (_, it, skip) = shader_series(false, true, false, 490_875, LEN, SKIP);
    assert!(!series_collapse(Some(before), series_share(it, LEN, skip)));
}

#[test]
fn an_ask_below_the_skip_keeps_it_through_the_rescue() {
    // A zoom out far enough that the ask falls under the skip: `sa_skip_rescue` raises the count
    // to the orbit's last sample so the skip still applies — no collapse.
    let (base, it, skip) = shader_series(false, true, false, 250_000, LEN, SKIP);
    assert_eq!(base, 250_000);
    assert_eq!((it, skip), (LEN - 1, SKIP), "raised past the skip, the skip kept");
    let prev = series_share(592_562, LEN, SKIP);
    assert!(!series_collapse(Some(prev), series_share(it, LEN, skip)));
}

#[test]
fn ordinary_changes_are_not_a_collapse() {
    // No frame to compare with.
    assert!(!series_collapse(None, 1.0));
    // No series either side: a partial reference growing longer under a rising ask (a dive's
    // lookahead installs, ×2.3 length jumps) stays at a share of 1.
    let (_, a, s) = shader_series(false, true, true, 600_000, 256_001, 0);
    let (_, b, t) = shader_series(false, true, true, 900_000, 508_193, 0);
    assert!(!series_collapse(Some(series_share(a, 256_001, s)), series_share(b, 508_193, t)));
    // A skip shrinking a little (a reference for a slightly wider view).
    let prev = series_share(700_000, 650_000, 200_000);
    assert!(!series_collapse(Some(prev), series_share(700_000, 650_000, 180_000)));
    // The skip GROWING is cheaper, never a collapse.
    assert!(!series_collapse(Some(series_share(600_000, LEN, 0)), series_share(600_000, LEN, SKIP)));
    // Within the slack of a tiny share.
    assert!(!series_collapse(Some(0.0), SERIES_COLLAPSE_SLACK));
    assert!(series_collapse(Some(0.0), SERIES_COLLAPSE_SLACK * 1.5));
}

#[test]
fn the_shader_series_matches_the_dispatch_rules() {
    // Direct: the full count, no skip.
    assert_eq!(shader_series(true, false, false, 5_000, 0, 0), (5_000, 5_000, 0));
    // No reference yet: the TDR-safe placeholder.
    assert_eq!(shader_series(false, false, false, 500_000, 0, 0), (256, 256, 0));
    // A partial reference caps the count at its last sample.
    assert_eq!(shader_series(false, true, true, 900_000, 256_001, 0), (256_000, 256_000, 0));
    // A skip at or past an ordinary count is refused when the rescue would not be cheaper.
    assert_eq!(shader_series(false, true, false, 1_000, 2_000_000, 1_500), (1_000, 1_000, 0));
}
