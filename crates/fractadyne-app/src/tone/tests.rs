use super::*;

/// The mute is the whole contract of `--no-sound`; assert the real decision function, since
/// whether a sound actually reached the speakers is not observable from a test.
///
/// ⚠Flag and env var are deliberately ONE test, not two. Both drive PROCESS-GLOBAL state and
/// cargo runs tests in parallel by default, so as separate tests they would race — the env one
/// setting `FRACTADYNE_NO_SOUND` while the flag one asserts `!muted()` is a genuine intermittent
/// failure. Within a single test the order is guaranteed.
#[test]
fn the_flag_and_the_env_var_each_silence_the_tone() {
    clear_override();
    std::env::remove_var("FRACTADYNE_NO_SOUND");
    assert!(!muted(), "default is audible");

    set_muted(true);
    assert!(muted(), "--no-sound must silence it");
    set_muted(false);
    assert!(!muted(), "--sound must restore it");

    // The env var is what reaches child processes (see `muted`).
    clear_override();
    std::env::set_var("FRACTADYNE_NO_SOUND", "1");
    assert!(muted(), "FRACTADYNE_NO_SOUND=1 must silence it");
    std::env::set_var("FRACTADYNE_NO_SOUND", "0");
    assert!(!muted(), "an explicit 0 must NOT silence it");
    std::env::set_var("FRACTADYNE_NO_SOUND", "");
    assert!(!muted(), "an empty value must NOT silence it");

    // ⭐The precedence rule: an explicit --sound outranks an inherited FRACTADYNE_NO_SOUND,
    // and does so WITHOUT clearing the variable, so a child process still inherits it.
    std::env::set_var("FRACTADYNE_NO_SOUND", "1");
    set_muted(false);
    assert!(!muted(), "--sound must beat the environment");
    assert_eq!(
        std::env::var("FRACTADYNE_NO_SOUND").as_deref(),
        Ok("1"),
        "--sound must not strip the variable from the environment children inherit",
    );

    clear_override();
    std::env::remove_var("FRACTADYNE_NO_SOUND");
    assert!(!muted(), "removing both restores the default");
}

#[test]
fn the_tune_is_three_notes_of_the_documented_length() {
    let s = finish_tone_samples(SAMPLE_RATE);
    assert_eq!(s.len(), (0.3 * SAMPLE_RATE as f32) as usize);
}

#[test]
fn zero_crossings_match_each_notes_frequency() {
    // Counted on the raw synth (the filter phase-shifts crossings slightly): a square at
    // f Hz crosses zero 2f times a second. This is the "did it actually synthesize the
    // documented pitches" check — a broken accumulator cannot pass it.
    let s = synth_tune(SAMPLE_RATE);
    let mut start = 0usize;
    for (freq, secs) in TUNE {
        let n = (secs * SAMPLE_RATE as f32).round() as usize;
        let seg = &s[start..start + n];
        let crossings = seg.windows(2).filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0)).count();
        let expected = (2.0 * freq * secs).round() as isize;
        let got = crossings as isize;
        assert!(
            (got - expected).abs() <= 2,
            "note {freq} Hz: {got} crossings, expected ~{expected}"
        );
        start += n;
    }
}

#[test]
fn poly_blep_is_zero_away_from_edges_and_continuous_at_them() {
    let dt = 1047.0 / SAMPLE_RATE as f32;
    assert_eq!(poly_blep(0.25, dt), 0.0);
    assert_eq!(poly_blep(0.75, dt), 0.0);
    // The quadratic pieces must meet the untouched region without a jump of their own.
    assert!(poly_blep(dt * 0.999, dt).abs() < 1e-3);
    assert!(poly_blep(1.0 - dt * 0.999, dt).abs() < 1e-3);
}

#[test]
fn the_speaker_filter_removes_dc_and_the_fades_end_at_silence() {
    let s = finish_tone_samples(SAMPLE_RATE);
    assert!(s.iter().all(|v| v.is_finite()));
    let mean = s.iter().sum::<f32>() / s.len() as f32;
    assert!(mean.abs() < 1.0e-3, "high-passed tone should carry no DC, mean {mean}");
    // Normalized after the filter: the loudest sample IS the amplitude constant.
    let peak = s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    assert!(peak <= AMPLITUDE * 1.001 && peak >= AMPLITUDE * 0.95, "peak {peak}");
    assert_eq!(s[0], 0.0, "attack fade starts from silence");
    assert!(s[s.len() - 1].abs() < 1.0e-4, "release fade ends at silence");
}

#[test]
fn the_wav_container_is_well_formed() {
    let samples = finish_tone_samples(SAMPLE_RATE);
    let w = wav_pcm16(&samples, SAMPLE_RATE);
    assert_eq!(&w[0..4], b"RIFF");
    assert_eq!(&w[8..16], b"WAVEfmt ");
    assert_eq!(&w[36..40], b"data");
    let riff_len = u32::from_le_bytes(w[4..8].try_into().unwrap()) as usize;
    let data_len = u32::from_le_bytes(w[40..44].try_into().unwrap()) as usize;
    assert_eq!(riff_len, w.len() - 8);
    assert_eq!(data_len, samples.len() * 2);
    assert_eq!(w.len(), 44 + data_len);
    let sr = u32::from_le_bytes(w[24..28].try_into().unwrap());
    assert_eq!(sr, SAMPLE_RATE);
}
