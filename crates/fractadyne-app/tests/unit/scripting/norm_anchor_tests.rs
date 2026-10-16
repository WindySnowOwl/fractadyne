use super::*;

/// No anchors = no mapping: the caller falls back to the legacy EMA, never to a made-up range.
#[test]
fn no_anchors_yields_none() {
    assert_eq!(norm_anchor_range(&[], 1.0), None);
}

/// One anchor holds for the whole tour — before, at, and after its time.
#[test]
fn a_single_anchor_is_constant() {
    let a = [(2.0, (10.0, 50.0))];
    for t in [0.0, 2.0, 9.0] {
        assert_eq!(norm_anchor_range(&a, t), Some((10.0, 50.0)));
    }
}

/// Between two anchors the range is the straight line through them; the endpoints are hit
/// exactly (a frame landing on a keyframe uses that keyframe's canonical range verbatim).
#[test]
fn two_anchors_interpolate_linearly_and_hit_the_endpoints() {
    let a = [(0.0, (0.0, 100.0)), (4.0, (40.0, 300.0))];
    assert_eq!(norm_anchor_range(&a, 0.0), Some((0.0, 100.0)));
    assert_eq!(norm_anchor_range(&a, 4.0), Some((40.0, 300.0)));
    assert_eq!(norm_anchor_range(&a, 2.0), Some((20.0, 200.0)));
    // Clamped outside the anchored span — the tour cannot ask for a time it doesn't have,
    // but a rounding edge must not extrapolate.
    assert_eq!(norm_anchor_range(&a, -1.0), Some((0.0, 100.0)));
    assert_eq!(norm_anchor_range(&a, 9.0), Some((40.0, 300.0)));
}

/// Two anchors at the same instant (a zero-length segment) must not divide by zero — the
/// earlier anchor wins up to the shared time.
#[test]
fn coincident_anchor_times_do_not_nan() {
    let a = [(1.0, (0.0, 10.0)), (1.0, (100.0, 200.0))];
    let (lo, hi) = norm_anchor_range(&a, 1.0).unwrap();
    assert!(lo.is_finite() && hi.is_finite());
}

/// Keyframe anchors clamp to the TOUR's last frame — the regression that let a shard ending
/// between keyframes measure its last anchor at its own last frame.
#[test]
fn anchor_keyframes_clamp_to_the_tour_and_dedup() {
    // 6 s at 3 fps = 19 frames (0..=18); a keyframe arriving at 6.2 s rounds to 19 and clamps.
    let kf = anchor_keyframe_frames([0.0, 3.0, 3.0, 6.0, 6.2].into_iter(), 3.0, 19);
    assert_eq!(kf, vec![0, 9, 18]);
}

/// ⭐A range measures only what it needs and still gets EXACTLY the full tour's mapping on every
/// frame in it — including when keyframes measure nothing (all interior) and at the clamped end.
/// Exhaustive over small tours, with a deterministic generator (no dev-dependency).
#[test]
fn a_range_gets_the_full_tours_mapping_on_every_frame() {
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = move |m: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % m
    };
    let mut cases = 0;
    for _ in 0..400 {
        let fps = [1.0, 3.0, 7.5, 30.0][next(4) as usize];
        let frames = 2 + next(60);
        let total = (frames - 1) as f64 / fps;
        // Keyframe frames: sorted, unique, always including 0 (a tour's first keyframe is at t = 0).
        let mut kf: Vec<u64> = (0..1 + next(6)).map(|_| next(frames)).collect();
        kf.push(0);
        kf.sort_unstable();
        kf.dedup();
        // Which keyframes measure nothing (all-interior views), and a range per keyframe.
        let blank: Vec<bool> = kf.iter().map(|_| next(3) == 0).collect();
        let anchor_of = |k: u64| -> Option<(f64, (f32, f32))> {
            let i = kf.iter().position(|&x| x == k).unwrap();
            (!blank[i]).then(|| {
                let t = anchor_file::frame_time(k, fps, total);
                (t, (k as f32 * 3.0 + 1.0, k as f32 * 7.0 + 50.0))
            })
        };
        let full = anchors_for_range(&kf, 0, frames - 1, &mut |k| anchor_of(k));
        let first = next(frames);
        let last = first + next(frames - first);
        let mut measured = 0usize;
        let part = anchors_for_range(&kf, first, last, &mut |k| {
            measured += 1;
            anchor_of(k)
        });
        assert!(measured <= kf.len());
        for f in first..=last {
            let t = anchor_file::frame_time(f, fps, total);
            assert_eq!(
                norm_anchor_range(&part, t),
                norm_anchor_range(&full, t),
                "frame {f} of {frames} @ {fps} fps, range {first}..={last}, keyframes {kf:?}, blank {blank:?}"
            );
        }
        cases += 1;
    }
    assert_eq!(cases, 400);
}

/// The whole tour measures every keyframe, in order — what the anchor pass always did.
#[test]
fn the_whole_tour_measures_every_keyframe_in_order() {
    let kf = [0u64, 9, 18];
    let mut order = Vec::new();
    let got = anchors_for_range(&kf, 0, 18, &mut |k| {
        order.push(k);
        Some(k)
    });
    assert_eq!(order, vec![0, 9, 18]);
    assert_eq!(got, vec![0, 9, 18]);
}

/// A shard between keyframes measures the keyframe on each side and nothing further.
#[test]
fn a_shard_measures_its_bracketing_keyframes_only() {
    let kf = [0u64, 9, 18, 27, 36];
    let mut order = Vec::new();
    let got = anchors_for_range(&kf, 11, 16, &mut |k| {
        order.push(k);
        Some(k)
    });
    assert_eq!(got, vec![9, 18], "the anchors bracketing frames 11..=16");
    assert_eq!(order, vec![9, 18], "no keyframe beyond the brackets was measured");
}
