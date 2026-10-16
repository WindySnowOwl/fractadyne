use super::parse_frame_progress;

#[test]
fn frame_progress_lines_parse() {
    // The exact shape `render_tour_to_dir` emits.
    assert_eq!(
        parse_frame_progress("  frame 5682/9931  (1h10m02s elapsed, 52m18s left, 1.35 fps)"),
        Some((5682, 9931))
    );
    assert_eq!(parse_frame_progress("frame 1/1"), Some((1, 1)));
    // Non-progress lines leave the bar alone.
    assert_eq!(parse_frame_progress("tour render: 233 frames → frames"), None);
    assert_eq!(parse_frame_progress("Encoding → out.mp4 (ffmpeg)…"), None);
    assert_eq!(parse_frame_progress("frame 12: There is not enough space"), None);
    assert_eq!(parse_frame_progress("frame 0/0"), None);
    assert_eq!(parse_frame_progress("frame 5/3"), None);
}
