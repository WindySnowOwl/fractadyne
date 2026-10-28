//! The Render zoom video dialog's command line and its cost summary.

use super::*;

fn dialog() -> ZoomVideoUi {
    ZoomVideoUi {
        centre: ("-0.75".into(), "0.1".into()),
        end_log2: 20.0,
        out: "C:/videos/my dive.mp4".into(),
        width: 640,
        height: 360,
        fps: 30.0,
        speed: 2.0,
        ..Default::default()
    }
}

#[test]
fn the_command_line_carries_the_view_exactly_and_only_the_options_set() {
    let z = dialog();
    let a = zoom_video_args(&z);
    let pair = |k: &str| a.iter().position(|x| x == k).map(|i| a[i + 1].clone());
    assert_eq!(a[0], "--render-zoom-video");
    assert_eq!(a[1], "C:/videos/my dive.mp4", "the path is one argument, spaces and all");
    let i = a.iter().position(|x| x == "--center").unwrap();
    assert_eq!((a[i + 1].as_str(), a[i + 2].as_str()), ("-0.75", "0.1"), "the centre as the viewport's text");
    assert_eq!(pair("--zoom-log2").as_deref(), Some("20"));
    assert_eq!(pair("--size").as_deref(), Some("640x360"));
    assert_eq!(pair("--assemble-fps").as_deref(), Some("30"));
    assert_eq!(pair("--assemble-speed").as_deref(), Some("2"));
    for absent in ["--expmap-start", "--assemble-rotate", "--expmap-oversample", "--assemble-ss", "--normalize", "--keep-strips"] {
        assert!(!a.iter().any(|x| x == absent), "{absent} was not asked for");
    }
    let z = ZoomVideoUi { start_log2: 3.5, rotate: -0.25, ss: 2, normalize: true, keep: true, keep_dir: "D:/strips".into(), ..dialog() };
    let a = zoom_video_args(&z);
    let pair = |k: &str| a.iter().position(|x| x == k).map(|i| a[i + 1].clone());
    assert_eq!(pair("--expmap-start").as_deref(), Some("3.5"));
    assert_eq!(pair("--assemble-rotate").as_deref(), Some("-0.25"));
    assert_eq!(pair("--expmap-oversample").as_deref(), Some("2"), "strips oversampled as the frames are sampled");
    assert_eq!(pair("--assemble-ss").as_deref(), Some("2"));
    assert_eq!(pair("--keep-strips").as_deref(), Some("D:/strips"));
    assert!(a.iter().any(|x| x == "--normalize"));
    // Every flag the dialog writes is one the command line knows.
    let known = crate::cli::known_long_flags();
    for f in a.iter().filter(|x| x.starts_with("--")) {
        assert!(known.contains(f.as_str()), "{f} is not in the command-line reference");
    }
}

#[test]
fn the_summary_is_the_dive_the_command_line_will_render() {
    let z = dialog();
    let (frames, seconds, strips, w, rows, gpu) = zoom_video_plan(&z);
    let map = fractadyne_core::ExpMap::for_frames(640.0, 360.0, 1.0, 0.0);
    assert_eq!((w, rows), (map.width, map.rows));
    assert_eq!(strips, map.strips_to(20.0, 360.0));
    assert_eq!(seconds, 10.0, "20 octaves at 2 a second");
    assert_eq!(frames, crate::assemble::dive_frames(0.0, 20.0, 2.0, 30.0).len() as u64);
    assert_eq!(gpu, fractadyne_gpu::layers_for_frame(640, 360) as u64 * w as u64 * rows as u64 * 16);
    // Anti-aliasing widens the strips and the window.
    let (_, _, _, w2, _, gpu2) = zoom_video_plan(&ZoomVideoUi { ss: 2, ..dialog() });
    assert!(w2 > w * 2 - 2 && gpu2 > gpu * 4);
}

/// The user's three tries on a 10 GB RTX 3080 (2026-10-28), at 2× anti-aliasing: 5120x2880 was
/// refused by the render after 13 s (a strip wider than the card's 32768-texel textures),
/// 3840x2160 crashed allocating a ~17.6 GB window, 1920x1080 ran with a 4.4 GB one. The dialog
/// must say the first two before Render, and let the third go.
#[test]
fn the_dialog_refuses_what_the_card_cannot_hold_before_render() {
    const DIM: u32 = 32768;
    const VRAM: u64 = 10 << 30;
    let at = |width, height| ZoomVideoUi { width, height, ss: 2, end_log2: 120.7, ..dialog() };
    let five_k = at(5120, 2880);
    let (_, _, _, sw, _, _) = zoom_video_plan(&five_k);
    assert!(sw > DIM, "the 5K strip is {sw} texels around");
    assert!(matches!(zoom_video_fit(&five_k, DIM, VRAM), CardFit::Refused(_)), "5K at 2x: a strip wider than the textures");
    let four_k = at(3840, 2160);
    let (_, _, _, sw, _, gpu) = zoom_video_plan(&four_k);
    assert!(sw <= DIM && gpu > VRAM, "4K at 2x fits the textures but not the memory ({gpu} bytes)");
    assert!(matches!(zoom_video_fit(&four_k, DIM, VRAM), CardFit::Refused(_)), "4K at 2x: the window exceeds the card");
    assert_eq!(zoom_video_fit(&at(1920, 1080), DIM, VRAM), CardFit::Fits, "1080p at 2x ran on this card");
    // A smaller card warns at the same video; an unknown one is not judged.
    assert!(matches!(zoom_video_fit(&at(1920, 1080), DIM, 6 << 30), CardFit::Tight(_)));
    assert_eq!(zoom_video_fit(&four_k, 0, 0), CardFit::Fits, "nothing known, nothing refused");
}
