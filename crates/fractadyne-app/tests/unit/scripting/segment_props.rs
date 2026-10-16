use super::order_tests::Rng;
use super::segment_range;

#[test]
fn segments_tile_the_frame_range_exactly() {
    // `--segments N --segment-index K` shards one offline render across machines, so this is
    // the property the whole feature rests on: half-open `[start, end)` ranges that abut with
    // no overlap and no gap. Overlap means two boxes burn hours on the same frames; a gap
    // means the assembled mp4 is missing frames, and nothing reports it — the omission only
    // shows up as a jump on playback, long after the machines are released.
    //
    // This function had NO tests before this. It is four lines, which is exactly why.
    let mut r = Rng(0xA5A5_5A5A_1234_9876);
    for _ in 0..4000 {
        let frames = r.below(20_000);
        let n = 1 + r.below(64);
        let mut prev_end = 0u64;
        for k in 0..n {
            let (s, e) = segment_range(frames, n, k);
            assert!(s <= e, "inverted shard k={k} n={n} frames={frames}: [{s},{e})");
            assert_eq!(s, prev_end, "gap or overlap at k={k} n={n} frames={frames}");
            prev_end = e;
        }
        assert_eq!(prev_end, frames, "shards miss frames: n={n} frames={frames}");
    }
}

#[test]
fn an_out_of_range_index_clamps_to_the_last_shard() {
    // `--segment-index` is user input. Clamping (rather than panicking or returning an empty
    // range) means a typo re-renders the tail instead of silently producing nothing.
    let mut r = Rng(0x1357_9BDF_0246_8ACE);
    for _ in 0..1000 {
        let frames = r.below(5000);
        let n = 1 + r.below(32);
        let last = segment_range(frames, n, n - 1);
        for over in [n, n + 1, n + 997, u64::MAX] {
            assert_eq!(
                segment_range(frames, n, over),
                last,
                "index {over} past n={n} did not clamp to the last shard"
            );
        }
    }
}

#[test]
fn zero_segments_behaves_as_one_whole_render() {
    // `n.max(1)` — a zero segment count must mean "render everything", not divide by zero.
    for frames in [0u64, 1, 7, 1000, 20_000] {
        assert_eq!(segment_range(frames, 0, 0), (0, frames));
        assert_eq!(segment_range(frames, 1, 0), (0, frames));
    }
}

#[test]
fn more_shards_than_frames_still_tiles_without_panicking() {
    // The degenerate shape a distributed run hits at the end of a short tour: some shards are
    // necessarily EMPTY (start == end). That is fine and must stay fine — the renderer skips
    // them — but they must not overlap their neighbours or drop a frame.
    for frames in 0..40u64 {
        for n in 1..64u64 {
            let mut prev_end = 0u64;
            for k in 0..n {
                let (s, e) = segment_range(frames, n, k);
                assert_eq!(s, prev_end, "gap at k={k} n={n} frames={frames}");
                assert!(e >= s);
                prev_end = e;
            }
            assert_eq!(prev_end, frames, "n={n} frames={frames}");
        }
    }
}

#[test]
fn the_tiling_formula_is_safe_for_any_reachable_frame_count() {
    // ⚠`k * frames` is a u64 multiply with no checked arithmetic, so it overflows once
    // `frames > u64::MAX / (n - 1)`. Tours are UNTRUSTED input (the artifact people share on
    // a forum), so this is worth stating rather than assuming: with the 64-shard maximum used
    // above, the bound is ~2.9e17 frames — about 1.5e11 years of 60 fps video, and far beyond
    // what a frame count parsed from fps x duration can reach. Pinned at a generous ceiling so
    // that if a future caller ever does compute frame counts near that bound, this fails here
    // rather than wrapping into a silently mistiled render.
    let huge = 1_000_000_000_000u64; // 1e12 frames, ~528 years at 60 fps
    for n in [1u64, 2, 7, 64] {
        let mut prev_end = 0u64;
        for k in 0..n {
            let (s, e) = segment_range(huge, n, k);
            assert_eq!(s, prev_end);
            prev_end = e;
        }
        assert_eq!(prev_end, huge, "mistiled at n={n}");
    }
    assert!(huge < u64::MAX / 64, "the tiling multiply would overflow at this frame count");
}
