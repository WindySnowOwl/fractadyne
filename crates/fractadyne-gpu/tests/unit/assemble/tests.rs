//! Assembly's CPU side: which strips a frame reads and how many layers a window needs, against
//! every sample of real grids worked out the way the shader works them out.

use super::*;

/// The strip the shader reads for the sample at `(x, y)` of a `w × h` grid (pixel centres at
/// `+½`), in f64: `depth_whole + floor(depth_frac − ½·log2(max(d², ¼)))`.
fn strip_of(place: &FramePlacement, w: u32, h: u32, x: u32, y: u32) -> i64 {
    let dx = x as f64 + 0.5 - w as f64 / 2.0;
    let dy = h as f64 / 2.0 - (y as f64 + 0.5);
    let d2 = (dx * dx + dy * dy).max(0.25);
    place.depth_whole + (place.depth_frac - 0.5 * d2.log2()).floor() as i64
}

#[test]
fn a_frame_reads_exactly_the_strips_its_samples_fall_in_and_the_window_holds_them() {
    // Even and odd grids, wide and tall, a 1080p frame at 2×2 samples.
    for (w, h) in [(64u32, 36u32), (65, 37), (36, 64), (3840, 2160)] {
        let (lo, hi) = sample_log2_span(w, h);
        let mut seen_lo = f64::MAX;
        let mut seen_hi = f64::MIN;
        // The span is the nearest and farthest sample, exactly.
        let step = if w > 1000 { 7 } else { 1 };
        for y in (0..h).step_by(step) {
            for x in (0..w).step_by(step) {
                let dx = x as f64 + 0.5 - w as f64 / 2.0;
                let dy = h as f64 / 2.0 - (y as f64 + 0.5);
                let l = 0.5 * (dx * dx + dy * dy).max(0.25).log2();
                seen_lo = seen_lo.min(l);
                seen_hi = seen_hi.max(l);
            }
        }
        // The corners and the centre are on the stride for these sizes or bounded by it.
        assert!(seen_hi <= hi + 1e-12 && seen_lo >= lo - 1e-12, "{w}x{h}: samples span {seen_lo}..{seen_hi}, the bound {lo}..{hi}");
        let layers = layers_for_frame(w, h);
        for n in 0..200 {
            let depth = 5.0 + n as f64 * 0.0731;
            let place = FramePlacement::new(depth, 0.0);
            let (first, last) = strips_for_frame(&place, w, h, 1_000).expect("inside the dive");
            assert!(last - first < layers as u64, "{w}x{h} at {depth}: strips {first}-{last} do not fit {layers} layers");
            for y in (0..h).step_by(step) {
                for x in (0..w).step_by(step) {
                    let k = strip_of(&place, w, h, x, y);
                    // A sample outside the dive (k < 0, a corner of an early frame) reads nothing.
                    let outside = k < 0 && first == 0;
                    assert!(
                        (k >= first as i64 || outside) && k <= last as i64,
                        "{w}x{h} at {depth}: sample ({x}, {y}) is in strip {k}, outside {first}-{last}"
                    );
                }
            }
        }
    }
}

#[test]
fn a_frame_at_the_edges_of_the_dive_is_clipped_to_it() {
    let (w, h) = (640u32, 360u32);
    // Partly before strip 0 (the corners lie outside the dive): starts at strip 0.
    let place = FramePlacement::new(3.0, 0.0);
    let (first, _) = strips_for_frame(&place, w, h, 40).unwrap();
    assert_eq!(first, 0);
    // Wholly shallower than the dive: nothing to read.
    assert_eq!(strips_for_frame(&FramePlacement::new(-30.0, 0.0), w, h, 40), None);
    // Past the dive's last strip: clipped to it.
    let (first, last) = strips_for_frame(&FramePlacement::new(60.0, 0.0), w, h, 40).unwrap();
    assert_eq!((first, last), (39, 39));
    // An empty dive.
    assert_eq!(strips_for_frame(&FramePlacement::new(10.0, 0.0), w, h, 0), None);
}

#[test]
fn a_placement_splits_its_depth_exactly() {
    let p = FramePlacement::new(1234.625, 0.25);
    assert_eq!((p.depth_whole, p.depth_frac, p.turns), (1234, 0.625, 0.25));
    let p = FramePlacement::new(-2.25, 0.0);
    assert_eq!((p.depth_whole, p.depth_frac), (-3, 0.75));
}
