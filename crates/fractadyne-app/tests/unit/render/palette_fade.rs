use super::*;

const CLASSIC: NormMap = NormMap { cycle: 0.0352, offset: 0.8, mode: 0, lo: 0.0 };
const LOG: NormMap = NormMap { cycle: 0.48, offset: 0.8, mode: 1, lo: 302.5 };
const LINEAR: NormMap = NormMap { cycle: 0.0022, offset: 0.13, mode: 0, lo: 0.0 };
const FRAME: f32 = 1.0 / 60.0;

#[test]
fn the_same_kind_of_map_never_dissolves() {
    // A normalised map gliding (its window moving) is one kind throughout: no fade.
    let (s, f) = palette_fade_step(PaletteFadeState::default(), LINEAR, true, 7, FRAME);
    assert_eq!(f.weight, 0.0);
    let moved = NormMap { cycle: 0.0021, offset: 0.12, ..LINEAR };
    let (_, f) = palette_fade_step(s, moved, true, 7, FRAME);
    assert_eq!(f.weight, 0.0);
}

#[test]
fn the_normalization_switching_off_dissolves_from_its_last_map() {
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LOG, true, 7, FRAME);
    // The classic map takes over: the frame fades from the log map at full weight…
    let (mut s, f) = palette_fade_step(s, CLASSIC, false, 7, FRAME);
    assert_eq!((f.weight, f.mode, f.cycle, f.lo), (1.0, 1, LOG.cycle, LOG.lo));
    // …which falls to nothing over PALETTE_FADE_S, a little each frame, and then stops.
    let mut last = f.weight;
    let mut frames = 0;
    loop {
        let (n, f) = palette_fade_step(s, CLASSIC, false, 7, FRAME);
        s = n;
        frames += 1;
        assert!(f.weight < last && last - f.weight < 0.05, "frame {frames}: {last} -> {}", f.weight);
        last = f.weight;
        if f.weight == 0.0 {
            break;
        }
        assert!(frames < 100, "never ended");
    }
    assert!((22..=26).contains(&frames), "{frames} frames at 60 fps for {PALETTE_FADE_S} s");
    assert_eq!(s.weight(), 0.0);
}

#[test]
fn switching_on_and_linear_to_log_dissolve_too_and_a_slow_frame_does_not_skip_it() {
    let (s, _) = palette_fade_step(PaletteFadeState::default(), CLASSIC, false, 7, FRAME);
    let (s, f) = palette_fade_step(s, LINEAR, true, 7, FRAME);
    assert_eq!((f.weight, f.cycle), (1.0, CLASSIC.cycle));
    // Linear to log within the normalization: another kind.
    let (s, f) = palette_fade_step(s, LOG, true, 7, FRAME);
    assert_eq!((f.weight, f.mode), (1.0, 0));
    // A 400 ms settled pass advances the dissolve by a tenth of a second, not all of it.
    let (_, f) = palette_fade_step(s, LOG, true, 7, 0.4);
    assert!((f.weight - 0.75).abs() < 1e-6, "{}", f.weight);
}

#[test]
fn the_users_own_colouring_changes_at_once() {
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LOG, true, 7, FRAME);
    // "Normalize deep colors" unticked: another colouring signature, so no dissolve…
    let (s, f) = palette_fade_step(s, CLASSIC, false, 8, FRAME);
    assert_eq!(f.weight, 0.0);
    // …and one under way stops when the user changes something.
    let (s, f) = palette_fade_step(s, LOG, true, 8, FRAME);
    assert_eq!(f.weight, 1.0);
    let (_, f) = palette_fade_step(s, LOG, true, 9, FRAME);
    assert_eq!(f.weight, 0.0);
}
