use super::*;

const CLASSIC: NormMap = NormMap { cycle: 0.0352, offset: 0.8, mode: 0, lo: 0.0 };
const LOG: NormMap = NormMap { cycle: 0.48, offset: 0.8, mode: 1, lo: 302.5 };
const LINEAR: NormMap = NormMap { cycle: 0.0022, offset: 0.13, mode: 0, lo: 0.0 };
const FRAME: f32 = 1.0 / 60.0;
/// The window the normalised maps above are drawn over.
const WIN: Option<(f32, f32)> = Some((300.0, 1900.0));

#[test]
fn a_glide_step_never_dissolves() {
    // A normalised map gliding (its window moving a little each frame) is one kind throughout,
    // and each step is far under a jump: no fade.
    let (s, f) = palette_fade_step(PaletteFadeState::default(), LINEAR, true, 7, FRAME, WIN);
    assert_eq!(f.weight, 0.0);
    let moved = NormMap { cycle: 0.00221, offset: 0.1295, ..LINEAR };
    assert!(map_change(&LINEAR, &moved, WIN.unwrap()) < PALETTE_JUMP);
    let (_, f) = palette_fade_step(s, moved, true, 7, FRAME, WIN);
    assert_eq!(f.weight, 0.0);
}

#[test]
fn a_whole_cycle_is_no_change_at_all() {
    // The carried phase wraps by exactly one cycle (`phase_carry_step`): the palette repeats, so
    // nothing on screen moves, and nothing may dissolve.
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LINEAR, true, 7, FRAME, WIN);
    let wrapped = NormMap { offset: LINEAR.offset + 1.0, ..LINEAR };
    assert!(map_change(&LINEAR, &wrapped, WIN.unwrap()) < 1e-4);
    let (_, f) = palette_fade_step(s, wrapped, true, 7, FRAME, WIN);
    assert_eq!(f.weight, 0.0);
    // …but a rotation by 0.6 is one by 0.4 the other way: a jump.
    let turned = NormMap { offset: LINEAR.offset + 0.6, ..LINEAR };
    assert!((map_change(&LINEAR, &turned, WIN.unwrap()) - 0.4).abs() < 1e-3);
}

#[test]
fn a_jump_within_one_kind_of_map_dissolves() {
    // The 2026-10-25 strobe and the settled view's snap: a normalised map whose window jumps
    // recolours the picture by a palette cycle or more in one frame. It dissolves instead.
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LINEAR, true, 7, FRAME, WIN);
    let jumped = NormMap { cycle: 0.0011, offset: 0.4, ..LINEAR };
    assert!(map_change(&LINEAR, &jumped, WIN.unwrap()) > PALETTE_JUMP);
    let (s, f) = palette_fade_step(s, jumped, true, 7, FRAME, WIN);
    assert_eq!((f.weight, f.cycle, f.offset), (1.0, LINEAR.cycle, LINEAR.offset));
    // …and it runs its course: the next frames do not restart it.
    let (_, f) = palette_fade_step(s, jumped, true, 7, FRAME, WIN);
    assert!(f.weight < 1.0 && f.weight > 0.9, "{}", f.weight);
}

#[test]
fn a_second_jump_restarts_a_dissolve_only_once_it_is_half_done() {
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LINEAR, true, 7, FRAME, WIN);
    let a = NormMap { cycle: 0.0011, offset: 0.4, ..LINEAR };
    let (mut s, _) = palette_fade_step(s, a, true, 7, FRAME, WIN);
    // Early in the dissolve, a further jump does not restart it (that would drop most of the old
    // map at once): the dissolve keeps fading from the first map.
    let b = NormMap { cycle: 0.0006, offset: 0.7, ..LINEAR };
    let (s2, f) = palette_fade_step(s, b, true, 7, FRAME, WIN);
    assert_eq!(f.cycle, LINEAR.cycle, "still fading from the first map");
    assert!(f.weight < 1.0);
    let _ = s2;
    // Past half way, a jump starts a new dissolve from the last frame's map.
    for _ in 0..14 {
        s = palette_fade_step(s, a, true, 7, FRAME, WIN).0;
    }
    assert!(s.weight() < 0.5, "{}", s.weight());
    let (_, f) = palette_fade_step(s, b, true, 7, FRAME, WIN);
    assert_eq!((f.weight, f.cycle), (1.0, a.cycle));
}

#[test]
fn the_normalization_switching_off_dissolves_from_its_last_map() {
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LOG, true, 7, FRAME, WIN);
    // The classic map takes over: the frame fades from the log map at full weight…
    let (mut s, f) = palette_fade_step(s, CLASSIC, false, 7, FRAME, None);
    assert_eq!((f.weight, f.mode, f.cycle, f.lo), (1.0, 1, LOG.cycle, LOG.lo));
    // …which falls to nothing over PALETTE_FADE_S, a little each frame, and then stops.
    let mut last = f.weight;
    let mut frames = 0;
    loop {
        let (n, f) = palette_fade_step(s, CLASSIC, false, 7, FRAME, None);
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
    let (s, _) = palette_fade_step(PaletteFadeState::default(), CLASSIC, false, 7, FRAME, None);
    let (s, f) = palette_fade_step(s, LINEAR, true, 7, FRAME, WIN);
    assert_eq!((f.weight, f.cycle), (1.0, CLASSIC.cycle));
    // Linear to log within the normalization: another kind.
    let (s, f) = palette_fade_step(s, LOG, true, 7, FRAME, WIN);
    assert_eq!((f.weight, f.mode), (1.0, 0));
    // A 400 ms settled pass advances the dissolve by a tenth of a second, not all of it.
    let (_, f) = palette_fade_step(s, LOG, true, 7, 0.4, WIN);
    assert!((f.weight - 0.75).abs() < 1e-6, "{}", f.weight);
}

#[test]
fn the_users_own_colouring_changes_at_once() {
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LOG, true, 7, FRAME, WIN);
    // "Normalize deep colors" unticked: another colouring signature, so no dissolve…
    let (s, f) = palette_fade_step(s, CLASSIC, false, 8, FRAME, None);
    assert_eq!(f.weight, 0.0);
    // …and one under way stops when the user changes something.
    let (s, f) = palette_fade_step(s, LOG, true, 8, FRAME, WIN);
    assert_eq!(f.weight, 1.0);
    let (_, f) = palette_fade_step(s, LOG, true, 9, FRAME, WIN);
    assert_eq!(f.weight, 0.0);
    // A jump in the user's own colouring (the cycle slider) snaps, as every colour control does.
    let (s, _) = palette_fade_step(PaletteFadeState::default(), LINEAR, true, 7, FRAME, WIN);
    let (_, f) = palette_fade_step(s, NormMap { cycle: 0.0044, ..LINEAR }, true, 10, FRAME, WIN);
    assert_eq!(f.weight, 0.0);
}
