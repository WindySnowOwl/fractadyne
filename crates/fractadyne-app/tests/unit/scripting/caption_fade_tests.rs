use super::fade_alpha;

/// A fade-0 caption is a cut: full on its first frame, and back-to-back fade-0 captions hand over on
/// the same frame with exactly one of them showing. Before 2026-11-16 the 1e-3 floor made such a
/// caption invisible on its first frame (and its last), so labels refreshed every 3 frames at 30 fps
/// were blank on every third frame — a 10 Hz flicker in the slices video.
#[test]
fn a_zero_fade_caption_is_a_cut_with_no_blank_frame() {
    let fps = 30.0;
    // Labels of 3 frames each, back to back from t = 9.0, as the video's value labels were.
    let labels: Vec<(f64, f64)> = (0..10).map(|k| (9.0 + 3.0 * k as f64 / fps, 9.0 + 3.0 * (k + 1) as f64 / fps)).collect();
    for frame in 270..300 {
        let t = frame as f64 / fps;
        let shown: Vec<f32> = labels.iter().map(|&(s, e)| fade_alpha(t, s, e, 0.0)).filter(|a| *a > 0.0).collect();
        assert_eq!(shown, [1.0], "frame {frame}: expected exactly one label at full opacity, got {shown:?}");
    }
    // Outside it: nothing.
    assert_eq!(fade_alpha(8.99, 9.0, 9.1, 0.0), 0.0);
    assert_eq!(fade_alpha(9.1, 9.0, 9.1, 0.0), 0.0, "half-open: gone at its end");
}

/// A caption with a fade still rises from nothing and sinks back, as before.
#[test]
fn a_fading_caption_still_fades() {
    assert_eq!(fade_alpha(1.0, 1.0, 5.0, 0.4), 0.0, "starts at 0");
    assert!((fade_alpha(1.2, 1.0, 5.0, 0.4) - 0.5).abs() < 1e-6, "half-way up its fade");
    assert_eq!(fade_alpha(3.0, 1.0, 5.0, 0.4), 1.0);
    assert!((fade_alpha(4.8, 1.0, 5.0, 0.4) - 0.5).abs() < 1e-6, "half-way down");
}
