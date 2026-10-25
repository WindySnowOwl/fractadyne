use super::*;
use fractadyne_gpu::{norm_hist_bucket, NormHist, NORM_HIST_BUCKETS};

/// A fine histogram of `values` from `base`, bucketed as the shader does.
fn hist(values: impl IntoIterator<Item = f32>, base: f32) -> NormHist {
    let mut counts = vec![0u32; NORM_HIST_BUCKETS];
    for v in values {
        counts[norm_hist_bucket(v, base)] += 1;
    }
    NormHist { base, counts }
}

/// The 2026-10-25 picture: escapes between 3,900 and 6,400, and 1% of it — beside a minibrot —
/// escaping up to the ask at 80,000.
fn minibrot_frame() -> NormHist {
    let bulk = (0..9900).map(|i| 3900.0 + 2500.0 * i as f32 / 9899.0);
    let fringe = (0..100).map(|i| 6400.0 + 73_600.0 * i as f32 / 99.0);
    hist(bulk.chain(fringe), 3900.0)
}

#[test]
fn the_palette_top_is_where_most_of_the_picture_escapes() {
    let h = minibrot_frame();
    let top = norm_palette_top(3900.0, 80_000.0, Some(&h), 0.98);
    assert!((6000.0..6500.0).contains(&top), "top {top}");
    // The window it makes spans the picture, not the fringe: the bulk fills most of the palette.
    let bulk_share = (6400.0 - 3900.0) / (top - 3900.0);
    assert!(bulk_share > 0.9, "{bulk_share}");
}

#[test]
fn the_palette_top_falls_back_to_the_range() {
    let h = minibrot_frame();
    // q = 1 is the shipped behaviour, exactly: the top of the range.
    assert_eq!(norm_palette_top(3900.0, 80_000.0, Some(&h), 1.0), 80_000.0);
    // No histogram (a reading from a path that has none), or too few samples to be a statistic.
    assert_eq!(norm_palette_top(3900.0, 80_000.0, None, 0.98), 80_000.0);
    let thin = hist((0..10).map(|i| 4000.0 + i as f32), 3900.0);
    assert_eq!(norm_palette_top(3900.0, 80_000.0, Some(&thin), 0.98), 80_000.0);
    // Never outside the reading, whatever the histogram says (it is sampled on a grid; the range
    // is every pixel).
    let wide = hist((0..1000).map(|i| 100.0 + i as f32 * 100.0), 0.0);
    assert_eq!(norm_palette_top(500.0, 2000.0, Some(&wide), 0.98), 2000.0);
    assert_eq!(norm_palette_top(500.0, 2000.0, Some(&hist((0..1000).map(|_| 10.0), 0.0)), 0.98), 500.0);
}

/// The feed's rules, written out as they stood before the palette window shared them: the escape
/// range must be fed exactly as before.
fn legacy(win: Option<(f32, f32)>, r: (f32, f32), nfeed: NormFeed, complete: bool, same_view: bool) -> Option<(f32, f32)> {
    match nfeed {
        NormFeed::Hold => match win {
            Some(held) if norm_hold_break(held, r) => Some(r),
            w => w,
        },
        NormFeed::Adopt if complete => Some(r),
        NormFeed::Adopt => {
            let fed = win.filter(|&f| same_view || !norm_glide_is_reacquire(f, r));
            Some(norm_reading_bound(r, false, fed))
        }
        NormFeed::Chase => {
            let reading = norm_reading_bound(r, complete, win);
            match norm_window_feed(None, reading, false, win).1 {
                Some(e) => Some(e),
                None => win,
            }
        }
    }
}

#[test]
fn the_escape_range_is_fed_exactly_as_before() {
    let wins = [None, Some((100.0, 900.0)), Some((40_000.0, 60_000.0)), Some((6.0, 191.0))];
    let readings = [(120.0, 1000.0), (90.0, 700.0), (181_573.0, 182_297.0), (45_000.0, 64_000.0), (5.0, 9.0)];
    for win in wins {
        for r in readings {
            for nfeed in [NormFeed::Hold, NormFeed::Adopt, NormFeed::Chase] {
                for complete in [false, true] {
                    for same_view in [false, true] {
                        let broken = nfeed == NormFeed::Hold && win.is_some_and(|h| norm_hold_break(h, r));
                        assert_eq!(
                            norm_feed_window(win, r, nfeed, complete, same_view, broken),
                            legacy(win, r, nfeed, complete, same_view),
                            "{win:?} {r:?} {nfeed:?} complete={complete} same_view={same_view}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_palette_window_breaks_its_hold_with_the_range() {
    // The hold is judged on the escape range; the palette window re-adopts when it does, even if
    // its own top would not have looked disjoint.
    let pal = Some((1000.0, 1200.0));
    assert_eq!(norm_feed_window(pal, (50_000.0, 52_000.0), NormFeed::Hold, true, true, true), Some((50_000.0, 52_000.0)));
    assert_eq!(norm_feed_window(pal, (50_000.0, 52_000.0), NormFeed::Hold, true, true, false), pal);
}
