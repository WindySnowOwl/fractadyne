//! The ffmpeg pipe: its command line, its partial file, and (where ffmpeg is installed) frames in,
//! the same frames out.

use super::*;

#[test]
fn the_command_line_takes_raw_rgba_on_stdin_and_names_the_container() {
    let a: Vec<String> = ffmpeg_args(641, 361, 29.97, 18, "mp4", Path::new("out.mp4.partial"))
        .into_iter()
        .map(|s| s.into_string().unwrap())
        .collect();
    let pair = |k: &str| a.iter().position(|x| x == k).map(|i| a[i + 1].as_str());
    assert_eq!(pair("-s"), Some("641x361"));
    assert_eq!(pair("-framerate"), Some("29.97"));
    assert_eq!(pair("-i"), Some("-"), "frames come on stdin");
    assert_eq!(pair("-crf"), Some("18"));
    assert!(a.windows(2).any(|w| w[0] == "-pix_fmt" && w[1] == "rgba"), "the input is RGBA");
    assert!(a.windows(2).any(|w| w[0] == "-pix_fmt" && w[1] == "yuv420p"), "the output plays everywhere");
    assert!(a.iter().any(|x| x.starts_with("pad=")), "an odd size is padded to even");
    // The container is named, so the partial file's name cannot mislead ffmpeg; the file is last.
    assert_eq!(&a[a.len() - 3..], ["-f", "mp4", "out.mp4.partial"]);
    assert!(a.iter().any(|x| x == "+faststart"));
    let mkv: Vec<OsString> = ffmpeg_args(64, 64, 30.0, 18, "matroska", Path::new("x"));
    assert!(!mkv.iter().any(|x| x == "+faststart"), "faststart is an mp4/mov flag");
}

#[test]
fn a_video_is_written_beside_its_name_in_the_container_its_extension_says() {
    assert_eq!(container_for(Path::new("a/dive.mp4")), "mp4");
    assert_eq!(container_for(Path::new("a/dive.MKV")), "matroska");
    assert_eq!(container_for(Path::new("dive.mov")), "mov");
    assert_eq!(container_for(Path::new("dive")), "mp4");
    assert_eq!(partial_video_path(Path::new("a/dive.mp4")), Path::new("a/dive.mp4.partial"));
}

#[test]
fn ffmpegs_last_words_are_kept_for_the_error() {
    assert_eq!(tail_lines("", 3), "");
    assert_eq!(tail_lines("\n \n", 3), "");
    assert_eq!(tail_lines("a\nb\n\nc\nd\n", 2), "; ffmpeg said:\n  c\n  d");
}

/// End to end, where ffmpeg is installed: five flat frames in, five frames of those colours out
/// (decoded again by ffmpeg; H.264's 4:2:0 and rounding allow a few levels). Where it is not, the
/// pipe must say so rather than pretend.
#[test]
fn frames_piped_in_come_back_out_of_the_video() {
    let Some(ffmpeg) = crate::exec_resolve::external("ffmpeg") else {
        let d = std::env::temp_dir().join(format!("fractadyne_vp_none_{}", std::process::id()));
        let err = VideoPipe::start(&d.join("x.mp4"), 64, 48, 30.0, 18).err().expect("no ffmpeg, no pipe");
        assert!(err.contains("ffmpeg"));
        return;
    };
    let d = std::env::temp_dir().join(format!("fractadyne_vp_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let out = d.join("five.mp4");
    let (w, h) = (64u32, 48u32);
    let colours = [[0.9f32, 0.1, 0.1], [0.1, 0.9, 0.1], [0.1, 0.1, 0.9], [0.5, 0.5, 0.5], [1.0, 1.0, 1.0]];
    let pipe = VideoPipe::start(&out, w, h, 30.0, 18).expect("start");
    for c in colours {
        let frame: Vec<f32> = (0..w * h).flat_map(|_| [c[0], c[1], c[2], 1.0]).collect();
        pipe.push(frame).expect("push");
    }
    assert_eq!(pipe.finish().expect("finish"), 5);
    assert!(out.exists() && !partial_video_path(&out).exists(), "the video is in place and its partial file gone");
    let dec = std::process::Command::new(&ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(&out)
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .expect("decode");
    let frame_len = (w * h * 3) as usize;
    assert_eq!(dec.stdout.len(), 5 * frame_len, "five frames came back");
    for (f, c) in colours.iter().enumerate() {
        let px = &dec.stdout[f * frame_len..(f + 1) * frame_len];
        for ch in 0..3 {
            let mean = px.iter().skip(ch).step_by(3).map(|&v| v as f64).sum::<f64>() / (w * h) as f64;
            let want = (c[ch] * 255.0) as f64;
            assert!((mean - want).abs() < 6.0, "frame {f} channel {ch}: {mean:.1}, wanted {want:.1}");
        }
    }
    // A frame of the wrong size fails the video, and nothing is left under the name.
    let bad = d.join("bad.mp4");
    let pipe = VideoPipe::start(&bad, w, h, 30.0, 18).expect("start");
    pipe.push(vec![0.0; 10]).expect("queued");
    let err = pipe.finish().expect_err("a short frame");
    assert!(err.contains("frame 0"), "{err}");
    assert!(!bad.exists() && !partial_video_path(&bad).exists());
    let _ = std::fs::remove_dir_all(&d);
}
