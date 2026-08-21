//! The render-finished tone: FRACTINT's "normal completion" tune, synthesized band-limited.
//!
//! The tune itself is read out of the DOS source rather than guessed (user request 2026-08-16;
//! sourcing 2026-08-21). From `general.asm` (mirror: LegalizeAdulthood/fractint), verbatim:
//!
//! ```text
//! buzzer0         dw      1047,100        ; "normal" completion
//!                 dw      1109,100
//!                 dw      1175,100
//!                 dw      0,0
//! ```
//!
//! Three rising 100 ms notes — C6, C#6, D6 — on the PC speaker. ("Interrupted" was the
//! descending mirror 2093/1976/1857 and "error" a 40 Hz razzberry; neither is used here.)
//!
//! Through beta.122 this played via `kernel32 Beep`, whose synthesized shim is a plain naive
//! square. Since beta.123 this module renders the tune itself (user request 2026-08-21):
//!
//! - **Square via phase accumulator + polyBLEP.** The PIT channel-2 speaker gate was a 50%-duty
//!   square; a naive square sampled at 48 kHz aliases. Each edge gets a 2-sample polynomial
//!   band-limited step correction with sub-sample timing (Välimäki's polyBLEP): O(1) per sample
//!   at any pitch, aliasing around −40 dB — below audibility under the tune's own harmonics.
//!   ONE phase accumulator runs through all three notes (the PIT was reprogrammed mid-run, not
//!   reset), so note joins are phase-continuous and click-free.
//! - **A high-pass models the speaker cone.** The little cone had no low end; a 2nd-order
//!   high-pass at 450 Hz reproduces that thinness on a full-range output device. The 1047 Hz
//!   fundamental passes essentially unchanged — the filter shapes the step plateaus and
//!   transients, which is exactly what the real cone did.
//! - Millisecond edge fades declick the DAC start/stop; playback is `winmm PlaySound` from an
//!   in-memory WAV cached for the process lifetime (`SND_MEMORY` requires the buffer to outlive
//!   an async play — a `OnceLock` guarantees it).

/// FRACTINT `buzzer0`, verbatim: (frequency Hz, duration seconds).
const TUNE: [(f32, f32); 3] = [(1047.0, 0.100), (1109.0, 0.100), (1175.0, 0.100)];

/// Synthesis rate. 48 kHz keeps the shared-mode mixer from resampling on most systems.
const SAMPLE_RATE: u32 = 48_000;

/// Peak amplitude of the raw square (the high-pass only removes energy at these ratios).
/// A notification should be clearly audible without startling — `Beep`'s shim sat near this.
const AMPLITUDE: f32 = 0.30;

/// Speaker-cone model: 2nd-order Butterworth high-pass corner. A 5-cent cone driver rolls off
/// steeply below a few hundred Hz; 450 Hz keeps the 1047 Hz fundamental intact (−0.1 dB) while
/// thinning the plateaus the way the original hardware did.
const SPEAKER_HPF_HZ: f64 = 450.0;

/// Edge declick fades (attack/release). The DOS speaker's engage/release clicks were artifacts
/// of a relay-driven cone, not the tune; 1–2 ms is inaudible and keeps the DAC step-free.
const FADE_IN_S: f32 = 0.001;
const FADE_OUT_S: f32 = 0.002;

/// 2-sample polyBLEP residual for a unit step at phase 0 (phase and `dt` in cycles). Zero away
/// from the edge; the two quadratic pieces meet the naive step continuously at `t = dt` and
/// `t = 1 − dt`, replacing the instantaneous jump with a band-limited one.
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let x = t / dt;
        2.0 * x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + 2.0 * x + 1.0
    } else {
        0.0
    }
}

/// The raw band-limited square tune at `sr`, one continuous phase accumulator across all three
/// notes, ±[`AMPLITUDE`]. No filtering or fades — see [`finish_tone_samples`] for the full chain.
fn synth_tune(sr: u32) -> Vec<f32> {
    let mut out = Vec::new();
    let mut phase = 0.0f32;
    for (freq, secs) in TUNE {
        let dt = freq / sr as f32;
        let n = (secs * sr as f32).round() as usize;
        for _ in 0..n {
            let naive = if phase < 0.5 { 1.0 } else { -1.0 };
            // Rising edge lives at phase 0, falling at 0.5 — each gets its own correction.
            let s = naive + poly_blep(phase, dt) - poly_blep((phase + 0.5).fract(), dt);
            out.push(s * AMPLITUDE);
            phase += dt;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }
    }
    out
}

/// In-place RBJ biquad high-pass (Butterworth Q), Direct Form 1 — the speaker-cone model.
fn speaker_highpass(samples: &mut [f32], sr: u32) {
    let w0 = std::f64::consts::TAU * SPEAKER_HPF_HZ / sr as f64;
    let (sin_w0, cos_w0) = w0.sin_cos();
    let alpha = sin_w0 / (2.0 * std::f64::consts::FRAC_1_SQRT_2); // Q = 1/sqrt(2), Butterworth
    let a0 = 1.0 + alpha;
    let (b0, b1, b2) = ((1.0 + cos_w0) / 2.0 / a0, -(1.0 + cos_w0) / a0, (1.0 + cos_w0) / 2.0 / a0);
    let (a1, a2) = (-2.0 * cos_w0 / a0, (1.0 - alpha) / a0);
    let (mut x1, mut x2, mut y1, mut y2) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for s in samples {
        let x0 = *s as f64;
        let y0 = b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
        *s = y0 as f32;
        (x2, x1, y2, y1) = (x1, x0, y1, y0);
    }
}

/// The complete finish tone at `sr`: band-limited square → speaker high-pass → declick fades.
fn finish_tone_samples(sr: u32) -> Vec<f32> {
    let mut s = synth_tune(sr);
    speaker_highpass(&mut s, sr);
    let n = s.len();
    let fade_in = ((FADE_IN_S * sr as f32) as usize).min(n);
    let fade_out = ((FADE_OUT_S * sr as f32) as usize).min(n);
    for i in 0..fade_in {
        s[i] *= i as f32 / fade_in as f32;
    }
    for i in 0..fade_out {
        s[n - 1 - i] *= i as f32 / fade_out as f32;
    }
    // The high-pass's step response swings each edge from its decayed plateau by the full step
    // height, so the filtered peak lands well above the raw square's. Normalize (after the
    // fades, so nothing shifts it again): the loudest sample IS the amplitude constant, and
    // nothing can clip.
    let peak = s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if peak > 0.0 {
        let g = AMPLITUDE / peak;
        for v in &mut s {
            *v *= g;
        }
    }
    s
}

/// Mono 16-bit PCM WAV container around `samples` (clamped to ±1).
fn wav_pcm16(samples: &[f32], sr: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + data_len as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data_len).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&sr.to_le_bytes());
    w.extend_from_slice(&(sr * 2).to_le_bytes()); // byte rate
    w.extend_from_slice(&2u16.to_le_bytes()); // block align
    w.extend_from_slice(&16u16.to_le_bytes()); // bits/sample
    w.extend_from_slice(b"data");
    w.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        w.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    w
}

/// Play the finish tone. `blocking`: the GUI passes false (the tune must not stall `update`);
/// the CLI `--render` path passes true — the process exits right after the completion message,
/// which would cut an async tune mid-note.
#[cfg(windows)]
pub(crate) fn play_finish_sound(blocking: bool) {
    static WAV: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    let wav = WAV.get_or_init(|| wav_pcm16(&finish_tone_samples(SAMPLE_RATE), SAMPLE_RATE));
    #[link(name = "winmm")]
    extern "system" {
        fn PlaySoundW(sound: *const u8, hmod: *mut core::ffi::c_void, flags: u32) -> i32;
    }
    const SND_ASYNC: u32 = 0x0001;
    const SND_NODEFAULT: u32 = 0x0002; // no fallback to the system default sound on failure
    const SND_MEMORY: u32 = 0x0004;
    let flags = SND_MEMORY | SND_NODEFAULT | if blocking { 0 } else { SND_ASYNC };
    // SAFETY: with SND_MEMORY the pointer must stay valid for the whole (possibly async)
    // playback; the OnceLock buffer lives for the process lifetime, so it does.
    unsafe {
        PlaySoundW(wav.as_ptr(), std::ptr::null_mut(), flags);
    }
}
#[cfg(not(windows))]
pub(crate) fn play_finish_sound(_blocking: bool) {}

#[cfg(test)]
mod tests {
    use super::*;

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
}
