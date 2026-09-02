use super::coarse_preview_has_content;

fn frame(n: usize, escaped: usize) -> Vec<f32> {
    // render_iter layout: 4 floats/px, r-channel first; r < 0 = did not escape.
    let mut px = vec![-1.0f32; n * 4];
    for i in 0..escaped {
        px[i * 4] = 100.0;
    }
    px
}

/// ⭐The reported frame: at a deep Misiurewicz view NOTHING escapes within the preview's
/// 16,384-iteration cap — solid black — and that frame must not replace the reprojected image.
#[test]
fn an_all_capped_preview_is_suppressed() {
    let (frac, keep) = coarse_preview_has_content(&frame(3136, 0), 56, 56);
    assert_eq!(frac, 0.0);
    assert!(!keep, "a 100%-capped preview is a solid black frame and must not install");
    // One stray pixel is still a flat frame for every practical purpose.
    let (_, keep) = coarse_preview_has_content(&frame(3136, 1), 56, 56);
    assert!(!keep);
}

/// The views the preview exists for (corpus 06/08 measured: the 16,384-iteration preview is
/// visually the finished render) escape most of the frame — those must keep installing.
#[test]
fn a_preview_with_content_still_installs() {
    let (frac, keep) = coarse_preview_has_content(&frame(3136, 2800), 56, 56);
    assert!(frac > 0.85);
    assert!(keep);
    // ...and the threshold itself is inclusive: exactly the tunable's share of escapes keeps.
    let at = (3136.0 * crate::COARSE_PREVIEW_MIN_ESCAPED).ceil() as usize;
    let (_, keep) = coarse_preview_has_content(&frame(3136, at), 56, 56);
    assert!(keep, "the boundary case must keep — the gate removes only flat frames");
}

/// A probe the gate cannot read must FAIL OPEN: the gate exists to remove a black frame and
/// must never be able to remove a good one on a hiccup of its own.
#[test]
fn an_unreadable_probe_fails_open() {
    assert!(coarse_preview_has_content(&[], 56, 56).1);
    assert!(coarse_preview_has_content(&frame(10, 0), 56, 56).1); // short buffer
}
