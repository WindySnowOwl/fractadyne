//! `--render-expmap`'s pure parts: the strip range, the caps over a dive, the view a strip renders
//! as, and the conversion of the renderer's targets into the strip's channels.

use super::*;

#[test]
fn a_strip_range_reads_as_given_or_not_at_all() {
    assert_eq!(parse_strip_range("3-7"), Some((3, 7)));
    assert_eq!(parse_strip_range("5"), Some((5, 5)));
    assert_eq!(parse_strip_range(" 0 - 12 "), Some((0, 12)));
    assert_eq!(parse_strip_range("7-3"), None, "backwards");
    assert_eq!(parse_strip_range("a-3"), None);
    assert_eq!(parse_strip_range("3-"), None);
}

#[test]
fn rising_caps_never_fall_and_fixed_caps_never_change() {
    let (w, h) = (1920.0, 1080.0);
    let map = ExpMap::for_frames(w, h, 1.0, 0.0);
    let n = map.strips_to(300.0, h);
    let rising = strip_caps(&map, n, w, h, 1000, CapMode::Rising, false);
    assert_eq!(rising.len(), n as usize);
    assert!(rising.windows(2).all(|p| p[0] <= p[1]), "a rising cap fell");
    assert!(rising[0] < *rising.last().unwrap(), "300 octaves and the cap never rose");
    // The depth rule at strip k's inner edge: the base plus ~220 an octave.
    let k = 100u64;
    assert_eq!(rising[k as usize], recommended_iterations(1000, map.frame_log2mag(k, w, h) + 1.0));

    let fixed = strip_caps(&map, n, w, h, 1000, CapMode::Fixed, false);
    assert!(fixed.iter().all(|&c| c == *rising.last().unwrap()), "fixed = the deepest strip's cap");
    let given = strip_caps(&map, n, w, h, 250_000, CapMode::Fixed, true);
    assert!(given.iter().all(|&c| c == 250_000), "fixed with --iter N = N");
    let grow = strip_caps(&map, n, w, h, 250_000, CapMode::Rising, true);
    assert_eq!(grow[0], recommended_iterations(250_000, map.frame_log2mag(0, w, h) + 1.0), "rising grows from --iter's N");
}

#[test]
fn a_strips_view_has_the_strips_disk_as_its_half_diagonal() {
    let map = ExpMap::for_frames(1920.0, 1080.0, 1.0, 0.0);
    let (cx, cy) = (fractadyne_core::BigFloat::from_f64(-0.75, 64), fractadyne_core::BigFloat::from_f64(0.1, 64));
    for k in [0u64, 7, 90, 2_000] {
        let vp = strip_view(&map, k, &cx, &cy);
        assert_eq!((vp.width_px, vp.height_px), (map.width as f64, map.rows as f64));
        let (sx, sy) = vp.complex_span_fe();
        let half_diag_log2 = (sx.mul_f64(0.5) * sx.mul_f64(0.5) + sy.mul_f64(0.5) * sy.mul_f64(0.5)).sqrt().log2();
        assert!((half_diag_log2 - map.outer_log2(k)).abs() < 1e-9, "strip {k}: half diagonal 2^{half_diag_log2}, radius 2^{}", map.outer_log2(k));
        // The shader's radius over 2^delta_exp stays O(1), as its f32 offset needs.
        let scale = vp.gpu_scale();
        let (m, e) = map.outer_scale(k);
        let r = fractadyne_gpu::StripMap::strip(map.row_step(), 0.0, m, e, scale.delta_exp).r;
        assert!(r > 0.4 && r <= 1.0, "strip {k}: map r {r}");
    }
}

#[test]
fn a_strips_channels_keep_the_exact_count_and_mark_capped_and_glitched_texels() {
    // Three texels: escaped at 123,456,789 iterations, capped (-1), glitched (-2).
    let iter: u32 = 123_456_789;
    let main = [
        iter as f32 + 0.3, 0.6, 0.8, 2.5, //
        -1.0, 0.0, 0.0, 1.0e30, //
        -2.0, 0.0, 0.0, 1.0e30,
    ];
    let aux = [
        (iter >> 12) as f32, (iter & 4095) as f32, 0.3, 0.25, //
        0.0, 0.0, 1.0e30, 0.0, //
        0.0, 0.0, 1.0e30, 0.0,
    ];
    let ch = strip_channels(&main, &aux);
    let get = |name: &str| ch.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("no channel {name}"));
    match &get("N").data {
        fractadyne_export::ExrChannelData::U32(n) => assert_eq!(n, &vec![iter, u32::MAX, 0]),
        _ => panic!("N is not uint32"),
    }
    match &get("NF").data {
        fractadyne_export::ExrChannelData::F32(v) => assert_eq!(v[0], 0.3),
        _ => panic!("NF is not f32"),
    }
    match &get("T").data {
        fractadyne_export::ExrChannelData::F32(v) => assert_eq!(v[0], 0.25),
        _ => panic!("T is not f32"),
    }
    match &get("DE").data {
        fractadyne_export::ExrChannelData::F32(v) => assert_eq!(v[0], 2.5),
        _ => panic!("DE is not f32"),
    }
    let names: Vec<_> = ch.iter().map(|c| c.name).collect();
    assert_eq!(names, ["N", "NF", "T", "DE", "NX", "NY"]);
}
