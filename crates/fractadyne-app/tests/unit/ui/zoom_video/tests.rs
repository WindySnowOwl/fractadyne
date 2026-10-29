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
    for absent in ["--expmap-start", "--assemble-rotate", "--expmap-oversample", "--assemble-ss", "--normalize", "--keep-strips", "--no-bla"] {
        assert!(!a.iter().any(|x| x == absent), "{absent} was not asked for");
    }
    // BLA is said either way (on by default), so the session's own setting never decides it.
    assert!(a.iter().any(|x| x == "--bla"), "BLA on by default, on the command line");
    let z = ZoomVideoUi { start_log2: 3.5, rotate: -0.25, ss: 2, bla: false, normalize: true, keep: true, keep_dir: "D:/strips".into(), ..dialog() };
    let a = zoom_video_args(&z);
    let pair = |k: &str| a.iter().position(|x| x == k).map(|i| a[i + 1].clone());
    assert_eq!(pair("--expmap-start").as_deref(), Some("3.5"));
    assert_eq!(pair("--assemble-rotate").as_deref(), Some("-0.25"));
    assert_eq!(pair("--expmap-oversample").as_deref(), Some("2"), "strips oversampled as the frames are sampled");
    assert_eq!(pair("--assemble-ss").as_deref(), Some("2"));
    assert_eq!(pair("--keep-strips").as_deref(), Some("D:/strips"));
    assert!(a.iter().any(|x| x == "--normalize"));
    assert!(a.iter().any(|x| x == "--no-bla") && !a.iter().any(|x| x == "--bla"), "BLA off is said as --no-bla, alone");
    // Every flag the dialog writes is one the command line knows.
    let known = crate::cli::known_long_flags();
    for f in a.iter().filter(|x| x.starts_with("--")) {
        assert!(known.contains(f.as_str()), "{f} is not in the command-line reference");
    }
}

/// "Use all graphics cards" adds `--gpus all` only when this machine has two or more; a farm job
/// is the same command without it (every machine's cards are the farm's), its summary and file
/// carried with it (`design/zoom-video.md` §8.5).
#[test]
fn several_cards_and_a_farm_take_the_same_command() {
    let one_card = ZoomVideoUi { all_gpus: true, cards: Some(1), ..dialog() };
    assert!(!zoom_video_args(&one_card).iter().any(|x| x == "--gpus"), "one card renders as usual");
    let two = ZoomVideoUi { all_gpus: true, cards: Some(2), ..dialog() };
    let a = zoom_video_args(&two);
    let i = a.iter().position(|x| x == "--gpus").expect("--gpus with two cards");
    assert_eq!(a[i + 1], "all");
    assert!(!zoom_video_args(&ZoomVideoUi { cards: Some(2), ..dialog() }).iter().any(|x| x == "--gpus"), "only when ticked");
    let job = zoom_video_farm_job(&two, "1,201 frames".into());
    assert!(!job.args.iter().any(|x| x == "--gpus" || x == "all"), "a farm job without --gpus: {:?}", job.args);
    assert_eq!(job.args, zoom_video_args(&ZoomVideoUi { cards: Some(2), ..dialog() }), "otherwise the very command this window runs");
    assert_eq!((job.summary.as_str(), job.video.as_str()), ("1,201 frames", "C:/videos/my dive.mp4"));
}

#[test]
fn the_summary_is_the_dive_the_command_line_will_render() {
    use fractadyne_gpu::WindowTexels as T;
    let z = ZoomVideoUi { texels: T::All, ..dialog() };
    let p = zoom_video_plan(&z, 32768);
    let map = fractadyne_core::ExpMap::for_frames(640.0, 360.0, 1.0, 0.0);
    assert_eq!((p.width, p.rows), (map.width, map.rows));
    assert_eq!(p.strips, map.strips_to(20.0, 360.0));
    assert_eq!(p.seconds, 10.0, "20 octaves at 2 a second");
    assert_eq!(p.frames, crate::assemble::dive_frames(0.0, 20.0, 2.0, 30.0).len() as u64);
    assert_eq!(p.layers, fractadyne_gpu::layers_for_frame(640, 360));
    let window = p.layers as u64 * p.width as u64 * p.rows as u64 * 16;
    assert_eq!(p.gpu, window + fractadyne_gpu::frame_bytes([640, 360], 1), "the whole window and the frame");
    // Smooth colouring's window holds the count alone: a quarter.
    let smooth = zoom_video_plan(&ZoomVideoUi { texels: T::Count, ..dialog() }, 32768);
    assert_eq!(smooth.gpu, window / 4 + fractadyne_gpu::frame_bytes([640, 360], 1));
    // Anti-aliasing widens the strips and the window.
    let p2 = zoom_video_plan(&ZoomVideoUi { ss: 2, texels: T::All, ..dialog() }, 32768);
    assert!(p2.width > p.width * 2 - 2 && p2.gpu > p.gpu * 4);
}

/// The user's three tries on a 10 GB RTX 3080 (2026-10-28), at 2× anti-aliasing: 5120x2880 was
/// refused by the render after 13 s (a strip wider than the card's 32768-texel textures),
/// 3840x2160 crashed allocating a ~17.6 GB window, 1920x1080 ran with a 4.4 GB one. Since the window
/// holds a wide strip in pieces and only what the colouring reads (`design/zoom-video.md` §8.6), a
/// strip's width is no reason to refuse, and 4K at 2x fits that card with smooth colouring; relief
/// lighting, which reads everything, still does not.
#[test]
fn the_dialog_refuses_what_the_card_cannot_hold_before_render() {
    use fractadyne_gpu::WindowTexels as T;
    const DIM: u32 = 32768;
    const VRAM: u64 = 10 << 30;
    let at = |width, height, texels| ZoomVideoUi { width, height, ss: 2, end_log2: 120.7, texels, ..dialog() };
    let five_k = at(5120, 2880, T::Count);
    let p = zoom_video_plan(&five_k, DIM);
    assert!(p.width > DIM, "the 5K strip is {} texels around", p.width);
    assert_eq!(zoom_video_fit(&five_k, DIM, 24 << 30), CardFit::Fits, "5K at 2x: wider than the textures, held in pieces");
    assert!(matches!(zoom_video_fit(&five_k, DIM, VRAM), CardFit::Refused(_)), "5K at 2x needs more than a 10 GB card's memory");
    assert_eq!(zoom_video_fit(&at(3840, 2160, T::Count), DIM, VRAM), CardFit::Fits, "4K at 2x, smooth colouring, on the user's card");
    let relief = at(3840, 2160, T::All);
    assert!(zoom_video_plan(&relief, DIM).gpu > VRAM, "relief at 4K 2x keeps everything");
    assert!(matches!(zoom_video_fit(&relief, DIM, VRAM), CardFit::Refused(_)), "4K at 2x with relief: the window exceeds the card");
    for t in [T::Count, T::CountDe, T::All] {
        assert_eq!(zoom_video_fit(&at(1920, 1080, t), DIM, VRAM), CardFit::Fits, "1080p at 2x ran on this card ({t:?})");
    }
    // A smaller card warns at the same video; an unknown one is not judged; a grid larger than the
    // textures is still refused.
    assert!(matches!(zoom_video_fit(&at(1920, 1080, T::All), DIM, 6 << 30), CardFit::Tight(_)));
    assert_eq!(zoom_video_fit(&relief, 0, 0), CardFit::Fits, "nothing known, nothing refused");
    assert!(matches!(zoom_video_fit(&at(7680, 4320, T::Count), 8192, 0), CardFit::Refused(_)), "a 15360x8640 grid on 8192-texel textures");
}
