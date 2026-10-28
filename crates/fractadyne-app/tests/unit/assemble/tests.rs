//! `--assemble`'s CPU side: reading a dive's manifest and strips, placing a frame among the
//! strips, and the frames of a straight dive.

use super::*;
use fractadyne_export::{ExrChannel, ExrChannelData};

/// A throwaway folder per test.
fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fractadyne_assemble_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("temp dir");
    d
}

fn manifest(map: &ExpMap, format: &str, convention: &str, rows: u32) -> String {
    serde_json::json!({
        "format": "fractadyne-expmap",
        "version": 1,
        "convention": convention,
        "dive": {
            "fractal": "Mandelbrot",
            "format": format,
            "frame": [640.0, 360.0],
            "oversample": 1.0,
            "start_log2mag": 0.0,
            "end_log2mag": 20.0,
            "width": map.width,
            "rows": rows,
            "r0_log2": map.r0_log2,
            "strips": 30,
        },
    })
    .to_string()
}

#[test]
fn a_dive_is_read_only_in_the_layout_assembly_reads() {
    let map = ExpMap::for_frames(640.0, 360.0, 1.0, 0.0);
    let ours = manifest_convention(StripFormat::Fractadyne).0;
    let d = tmp("dive");
    std::fs::write(d.join(MANIFEST), manifest(&map, "fractadyne", ours, map.rows)).unwrap();
    let dive = read_dive(&d).expect("our own strips");
    assert_eq!(dive.map, map);
    assert_eq!((dive.strips, dive.frame, dive.end_log2mag), (30, (640.0, 360.0), 20.0));

    std::fs::write(d.join(MANIFEST), manifest(&map, "zoomasm", ours, map.rows)).unwrap();
    assert!(read_dive(&d).unwrap_err().contains("zoomasm"), "zoomasm's strips are for zoomasm");
    std::fs::write(d.join(MANIFEST), manifest(&map, "fractadyne", "texels somewhere else", map.rows)).unwrap();
    assert!(read_dive(&d).is_err(), "another layout");
    std::fs::write(d.join(MANIFEST), manifest(&map, "fractadyne", ours, map.rows + 1)).unwrap();
    assert!(read_dive(&d).is_err(), "rows that are not this width's");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_strip_file_becomes_the_main_target_an_iterate_pass_writes() {
    let map = ExpMap { width: 4, rows: 2, r0_log2: 1.0, layout: StripRows::Conformal };
    let n = vec![7u32, u32::MAX, 0, 123_456_789, 1, 2, 3, 4];
    let nf: Vec<f32> = vec![0.25, 0.0, 0.0, 0.5, 0.0, 0.75, 0.125, 0.0625];
    let de: Vec<f32> = (0..8).map(|i| i as f32 - 3.0).collect();
    let nx: Vec<f32> = (0..8).map(|i| 0.1 * i as f32).collect();
    let ny: Vec<f32> = (0..8).map(|i| -0.1 * i as f32).collect();
    let d = tmp("strip");
    let p = d.join("strip_000000.exr");
    fractadyne_export::write_exr_channels(
        &p,
        4,
        2,
        vec![
            ExrChannel { name: "N", data: ExrChannelData::U32(n.clone()) },
            ExrChannel { name: "NF", data: ExrChannelData::F32(nf.clone()) },
            ExrChannel { name: "T", data: ExrChannelData::F32(vec![0.0; 8]) },
            ExrChannel { name: "DE", data: ExrChannelData::F32(de.clone()) },
            ExrChannel { name: "NX", data: ExrChannelData::F32(nx.clone()) },
            ExrChannel { name: "NY", data: ExrChannelData::F32(ny.clone()) },
        ],
        None,
    )
    .unwrap();
    let main = strip_main(&p, &map).expect("read");
    let smooth: Vec<f32> = main.chunks_exact(4).map(|t| t[0]).collect();
    assert_eq!(smooth, vec![7.25, -1.0, -2.0, (123_456_789.5f64) as f32, 1.0, 2.75, 3.125, 4.0625]);
    for i in 0..8 {
        assert_eq!((main[i * 4 + 1], main[i * 4 + 2], main[i * 4 + 3]), (nx[i], ny[i], de[i]));
    }
    // A strip of another size is not this dive's.
    let other = ExpMap { width: 5, ..map };
    assert!(strip_main(&p, &other).is_err());
    let _ = std::fs::remove_dir_all(&d);
}

/// The frame where strip `k` first appears (its corners on the strip's outer radius,
/// `ExpMap::frame_log2mag`) places its corner sample `k` octaves in, at any depth and supersampling.
#[test]
fn a_frames_corner_lies_on_the_outer_radius_of_the_strip_it_first_shows() {
    let (fw, fh) = (1920.0, 1080.0);
    let map = ExpMap::for_frames(fw, fh, 1.0, 0.0);
    for k in [0u64, 1, 17, 3_000] {
        let l = map.frame_log2mag(k, fw, fh);
        for ss in [1u32, 2, 3] {
            let p = placement(&map, l, fh, ss, 0.0);
            // The frame's corner is half its diagonal from the centre: diag·ss/2 samples.
            let corner = ((fw * ss as f64).hypot(fh * ss as f64) / 2.0).log2();
            let depth_at_corner = (p.depth_whole as f64 - k as f64) + p.depth_frac - corner;
            assert!(depth_at_corner.abs() < 1e-9, "k {k}, ss {ss}: the corner is {depth_at_corner} octaves off strip {k}'s edge");
        }
    }
}

#[test]
fn a_straight_dive_runs_from_its_first_frame_to_its_last_at_a_constant_speed() {
    let f = dive_frames(0.0, 20.0, 1.0, 30.0);
    assert_eq!(f.len(), 601);
    assert_eq!((f[0], f[600]), (0.0, 20.0));
    let step = f[1] - f[0];
    assert!(f.windows(2).all(|w| ((w[1] - w[0]) - step).abs() < 1e-12));
    // Too short for a frame of motion still has its two ends.
    assert_eq!(dive_frames(5.0, 5.001, 1.0, 30.0), vec![5.0, 5.001]);
}

#[test]
fn a_strips_blocks_keep_the_least_and_greatest_escape_value_of_their_texels() {
    let (w, h) = (20usize, 11usize);
    let mut main = vec![0f32; w * h * 4];
    for t in 0..w * h {
        // Interior in the first block row's left block, escaped elsewhere.
        main[t * 4] = if t % w < 8 && t / w < 8 { -1.0 } else { (t % w) as f32 + 100.0 * (t / w) as f32 };
    }
    let r = strip_range(&main, w, h);
    assert_eq!((r.bw, r.bh), (3, 2));
    assert!(r.hi[0] < r.lo[0], "an all-interior block has no range");
    assert_eq!((r.lo[1], r.hi[1]), (8.0, 15.0 + 700.0));
    assert_eq!((r.lo[2], r.hi[2]), (16.0, 19.0 + 700.0));
    assert_eq!((r.lo[5], r.hi[5]), (16.0 + 800.0, 19.0 + 1000.0), "a partial block at the corner");
    assert_eq!((r.n[0], r.n[1], r.n[5]), (0, 64, 12), "escaped texels a block");
    // Block 5: columns 16..20, rows 8..11: mean of i + 100 j.
    assert_eq!(r.mean[5], 17.5 + 900.0);
}

/// A frame's palette window from block summaries against every texel whose point lies inside the
/// frame, worked out from `texel_polar` directly: its least value, and the value by which 98% of
/// the frame's AREA (each texel weighted by the square of its radius) had escaped — the same to
/// within a block's spread of values.
#[test]
fn a_frames_window_is_the_least_value_and_the_area_quantile_of_the_texels_inside_it() {
    let (fw, fh) = (96.0, 54.0);
    let map = ExpMap::for_frames(fw, fh, 1.0, 0.0);
    let strips = 20u64;
    let (w, rows) = (map.width as usize, map.rows as usize);
    // A value that grows with depth and turns with angle: smallest at the outside, largest inside.
    let value = |k: u64, i: usize, j: usize| (k as usize * 1000 + j * 3 + i % 2) as f32;
    let mut ranges = HashMap::new();
    for k in 0..strips {
        let mut main = vec![0f32; w * rows * 4];
        for t in 0..w * rows {
            main[t * 4] = value(k, t % w, t / w);
        }
        ranges.insert(k, Arc::new(strip_range(&main, w, rows)));
    }
    for l in [0.0, 3.3, 9.75] {
        let got = frame_range(&map, &ranges, strips, l, fw, fh, 0.98).unwrap().expect("escaped");
        let upp_log2 = (Viewport::REFERENCE_HEIGHT / fh).log2() - l;
        // The strips the frame's samples read: deeper ones lie within half a pixel of its centre.
        let (first, last) = fractadyne_gpu::strips_for_frame(&placement(&map, l, fh, 1, 0.0), fw as u32, fh as u32, strips).unwrap();
        let mut lo = f32::INFINITY;
        let mut inside: Vec<(f32, f64)> = Vec::new();
        for k in first..=last {
            for j in 0..rows {
                for i in 0..w {
                    let (r_log2, th) = map.texel_polar(k, i as f64, j as f64);
                    let r = (r_log2 - upp_log2).exp2();
                    if (r * th.cos()).abs() <= fw / 2.0 && (r * th.sin()).abs() <= fh / 2.0 {
                        lo = lo.min(value(k, i, j));
                        inside.push((value(k, i, j), r * r));
                    }
                }
            }
        }
        let top = weighted_quantile(&mut inside, 0.98).unwrap();
        // A block spans 8 rows (3 apart) and 8 columns: 25 of value either side at the most.
        assert!((got.0 - lo).abs() <= 25.0, "2^{l}: least {} against the texels' {lo}", got.0);
        assert!((got.1 - top).abs() <= 25.0, "2^{l}: top {} against the texels' area quantile {top}", got.1);
        // The top is the bulk's, far below the deepest texels' values.
        let deepest = inside.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!(got.1 < deepest - 100.0, "2^{l}: the top {} is the slowest texel's {deepest}", got.1);
    }
    // A strip whose range was never read is an error, not a narrower range.
    let mut missing = ranges.clone();
    missing.remove(&3);
    assert!(frame_range(&map, &missing, strips, 3.3, fw, fh, 0.98).is_err());
}

#[test]
fn a_weighted_quantile_reads_the_weight_not_the_count() {
    let mut v = vec![(5.0f32, 1.0f64), (1.0, 1.0), (3.0, 8.0)];
    assert_eq!(weighted_quantile(&mut v, 0.05), Some(1.0));
    assert_eq!(weighted_quantile(&mut v, 0.5), Some(3.0), "the heavy middle value");
    assert_eq!(weighted_quantile(&mut v, 0.95), Some(5.0));
    assert_eq!(weighted_quantile(&mut [], 0.5), None);
    assert_eq!(weighted_quantile(&mut [(1.0, 0.0)], 0.5), None, "no weight");
}

/// The linear-to-log switch the live view makes at a window ratio of 1000 is a dissolve in a video:
/// none of the log form a factor of two below, all of it a factor of two above, half at the ratio,
/// rising smoothly between — and all of it when the user asked for log.
#[test]
fn a_videos_palette_dissolves_between_linear_and_log_around_the_live_views_switch() {
    let r = crate::render::NORM_LOG_AUTO_RATIO;
    assert_eq!(log_weight(10.0, 10.0 * r / 2.0, false), 0.0);
    assert_eq!(log_weight(10.0, 10.0 * r * 2.0, false), 1.0);
    assert!((log_weight(10.0, 10.0 * r, false) - 0.5).abs() < 1e-6);
    let mut prev = 0.0;
    for i in 0..=40 {
        let w = log_weight(1.0, r * (2f32).powf(i as f32 / 10.0 - 2.0), false);
        assert!(w >= prev && (0.0..=1.0).contains(&w));
        prev = w;
    }
    assert_eq!(log_weight(45_075.0, 63_736.0, false), 0.0, "a deep narrow window stays linear");
    assert_eq!(log_weight(45_075.0, 63_736.0, true), 1.0, "asked for log");
    // Below 1 the low end counts as 1, as the live view's test does.
    assert_eq!(log_weight(0.01, 3.0, false), 0.0);
}

#[test]
fn a_videos_palette_ranges_are_measured_once_an_octave_and_interpolated_between() {
    let mut a = NormAnchors::new(2.0, 5.5);
    let mut measured = Vec::new();
    let mut measure = |d: f64| -> Result<Option<(f32, f32)>, String> {
        measured.push(d);
        Ok(Some((d as f32 * 10.0, d as f32 * 100.0)))
    };
    let near = |got: Option<(f32, f32)>, want: (f64, f64)| {
        let g = got.unwrap();
        ((g.0 as f64 - want.0).abs() < 1e-3 * want.0 && (g.1 as f64 - want.1).abs() < 1e-3 * want.1, g)
    };
    // Halfway between the anchors at 3 and 4: the geometric mean (a window grows by factors).
    let (ok, g) = near(a.range(3.5, &mut measure).unwrap(), ((30.0f64 * 40.0).sqrt(), (300.0f64 * 400.0).sqrt()));
    assert!(ok, "{g:?}");
    let (ok, g) = near(a.range(3.75, &mut measure).unwrap(), (30.0f64.powf(0.25) * 40.0f64.powf(0.75), 300.0f64.powf(0.25) * 400.0f64.powf(0.75)));
    assert!(ok, "{g:?}");
    // The last octave is short: anchors at 5 and the end, 5.5.
    let (ok, g) = near(a.range(5.25, &mut measure).unwrap(), ((50.0f64 * 55.0).sqrt(), (500.0f64 * 550.0).sqrt()));
    assert!(ok, "{g:?}");
    assert_eq!(a.range(5.5, &mut measure).unwrap(), Some((55.0, 550.0)));
    assert_eq!(measured, vec![3.0, 4.0, 5.0, 5.5], "each anchor measured once, in order");
    // Nothing escaped at an anchor: its neighbour's range; at both: the last one given.
    let mut b = NormAnchors::new(0.0, 4.0);
    let mut empty_at_2 = |d: f64| -> Result<Option<(f32, f32)>, String> { Ok((d != 2.0 && d != 3.0).then_some((d as f32, d as f32 + 1.0))) };
    assert_eq!(b.range(1.5, &mut empty_at_2).unwrap(), Some((1.0, 2.0)));
    assert_eq!(b.range(2.5, &mut empty_at_2).unwrap(), Some((1.0, 2.0)), "carried over");
}
