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
