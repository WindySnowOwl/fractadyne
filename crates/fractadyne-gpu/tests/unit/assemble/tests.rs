//! Assembly's CPU side: which strips a frame reads and how many layers a window needs, against
//! every sample of real grids worked out the way the shader works them out.

use super::*;

/// The strip the shader reads for the sample at `(x, y)` of a `w × h` grid (pixel centers at
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
        // The corners and the center are on the stride for these sizes or bounded by it.
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
#[test]
fn a_strip_is_held_in_pieces_no_wider_than_asked_covering_it_once() {
    // (width, widest piece) -> pieces, as equal as whole texels allow.
    for (w, max) in [(36_911u32, 32_768u32), (27_683, 32_768), (55_366, 16_384), (10, 3), (7, 3), (9, 2), (1, 5), (100, 0), (13_842, 4_096)] {
        let (n, piece) = strip_pieces(w, max);
        let limit = if max == 0 { w } else { max };
        assert!(piece <= limit && piece >= 1, "{w} by {max}: a piece of {piece}");
        assert_eq!(n, w.div_ceil(limit.min(w)), "{w} by {max}: {n} pieces, not the fewest");
        // Every column in exactly one piece, the last one not empty.
        assert!((n - 1) * piece < w && n * piece >= w, "{w} by {max}: {n} pieces of {piece}");
    }
    assert_eq!(strip_pieces(36_911, 32_768), (2, 18_456));
    assert_eq!(strip_pieces(27_683, 32_768), (1, 27_683));
}

#[test]
fn a_window_holds_what_the_colouring_reads() {
    use fractadyne_core::coloring::ColorMethod;
    use crate::PaletteAa::{Distance, Neighbors, Off};
    let (smooth, distance) = (ColorMethod::Smooth.gpu_id(), ColorMethod::Distance.gpu_id());
    // Smooth: the count alone; distance coloring, glow or the palette anti-aliased by the distance
    // (C8): and the DE; relief: everything.
    for aa in [Off, Neighbors] {
        assert_eq!(WindowTexels::for_colour(smooth, false, false, aa), WindowTexels::Count);
    }
    assert_eq!(WindowTexels::for_colour(smooth, false, false, Distance), WindowTexels::CountDe);
    assert_eq!(WindowTexels::for_colour(smooth, false, true, Off), WindowTexels::CountDe);
    assert_eq!(WindowTexels::for_colour(distance, false, false, Off), WindowTexels::CountDe);
    assert_eq!(WindowTexels::for_colour(distance, false, true, Distance), WindowTexels::CountDe);
    for (m, g) in [(smooth, false), (smooth, true), (distance, false), (distance, true)] {
        assert_eq!(WindowTexels::for_colour(m, true, g, Distance), WindowTexels::All);
    }
    // The aux methods, and a number no method has, keep everything.
    for m in ColorMethod::ALL.into_iter().filter(|m| m.reads_aux()).map(ColorMethod::gpu_id).chain([ColorMethod::ALL.len() as u32, 99]) {
        assert_eq!(WindowTexels::for_colour(m, false, false, Off), WindowTexels::All);
    }
    let shape = StripShape { width: 27_683, rows: 3_059, rows_per_octave: 3_054.0, sample_offset: 0.5, row_step: 2.27e-4, strips: 130 };
    let whole = window_bytes(shape, 14, WindowLayout::default(), 32_768);
    assert_eq!(whole, 14 * 27_683 * 3_059 * 16);
    let count = WindowLayout { texels: WindowTexels::Count, piece_max: 0 };
    assert_eq!(window_bytes(shape, 14, count, 32_768) * 4, whole);
    // In pieces: whole pieces, the last one's spare columns too.
    let wide = StripShape { width: 36_911, ..shape };
    assert_eq!(window_bytes(wide, 15, count, 32_768), 15 * 2 * 18_456 * 3_059 * 4);
    assert_eq!(window_bytes(wide, 15, WindowLayout { piece_max: 10_000, ..count }, 32_768), 15 * 4 * 9_228 * 3_059 * 4);
}
