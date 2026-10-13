//! What a live frame IS, and the identity of the iteration walk it belongs to
//! (design/live-render-robustness.md §5.3, W4).
//!
//! `build_params` produces nine kinds of frame through staged mutation, and nothing used to name
//! which one a frame was: the record carried a dozen flags (`interacting`, `pin_frame`, `tiled`,
//! `chunked`, `present`, …) whose combinations a reader had to decode, and the decisions that
//! depend on the kind — the dispatch stamp, the held-frame floor — each re-inferred it from a
//! proxy. [`FrameKind`] is decided once, from the values the frame actually handed the GPU, and
//! recorded (frame record `frame_kind`, schema 10).
//!
//! [`WalkSig`] is the walk's identity: everything that shapes a resumable walk, so that any change
//! to it restarts the walk rather than resuming per-pixel state computed for another view. It is
//! built in ONE place ([`WalkSig::of`]); its fields are private, so a walk identity cannot be
//! assembled by hand somewhere that forgets an input.

/// What a live frame did, decided from what it handed the GPU (not from what was requested).
///
/// The codes are the frame record's `frame_kind` and are part of the schema: append, never
/// renumber. 0 is "not built this frame".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum FrameKind {
    /// A hold with nothing behind it: no frame has been rendered for this view yet (a cold start
    /// waiting for its reference). The display serves an empty texture.
    Placeholder = 1,
    /// Re-samples the held texture under a transform — a pan drag, the reuse hold between moving
    /// refreshes, a freeze while a reference catches up. Iterates nothing.
    Reproject = 2,
    /// One pass of a pinned refresh (a chunked walk or a split), composing under the held
    /// snapshot while the view moves. Its first pass is the frame that started the pin.
    PinWalk = 3,
    /// A moving frame that iterates and is shown as it is: a one-pass motion preview or a live
    /// refresh.
    Motion = 4,
    /// A settled frame that dispatches nothing new: the texture already holds it (an unchanged
    /// key, a walk's tail, a grid waiting its turn).
    Hold = 5,
    /// One pass of a settled chunked walk.
    SettledWalk = 6,
    /// One tile of a settled grid.
    Tile = 7,
    /// A settled frame rendering a progressive-supersampling sample (a sub-pixel-shifted view).
    Accum = 8,
    /// A settled frame rendered whole, in one dispatch.
    Settled = 9,
}

impl FrameKind {
    /// The record's code.
    pub(crate) fn code(self) -> u8 {
        self as u8
    }

    /// The record's codes, by name. `scripts/framelog.py`'s `KIND` table must say the same, and a
    /// test holds it to that.
    #[cfg(test)]
    pub(crate) const NAMES: [(u8, &'static str); 9] = [
        (1, "placeholder"),
        (2, "reproject"),
        (3, "pin-walk"),
        (4, "motion"),
        (5, "hold"),
        (6, "settled-walk"),
        (7, "tile"),
        (8, "accum"),
        (9, "settled"),
    ];
}

/// The facts [`classify`] decides a frame's kind from: what the frame handed the GPU.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FrameFacts {
    /// The colour pass re-samples the held texture (`MandelbrotParams::reproject`).
    pub(crate) reproject: bool,
    /// A frame has been rendered for this view (`frozen_center`), so a reprojection has
    /// something to show.
    pub(crate) has_frozen: bool,
    pub(crate) interacting: bool,
    /// A pass of a pinned refresh: a pin frame, or the frame that started the pin.
    pub(crate) pin_pass: bool,
    /// A real pass goes out this frame (not deduped, not a zero-area hold rect).
    pub(crate) dispatches: bool,
    pub(crate) tile: bool,
    pub(crate) chunked: bool,
    /// The dispatch renders a supersampling sample (a non-zero jitter).
    pub(crate) accum: bool,
}

/// A frame's kind. Total and mutually exclusive: the first rule that applies decides.
pub(crate) fn classify(f: FrameFacts) -> FrameKind {
    if f.reproject {
        if f.has_frozen { FrameKind::Reproject } else { FrameKind::Placeholder }
    } else if f.pin_pass {
        FrameKind::PinWalk
    } else if f.interacting {
        FrameKind::Motion
    } else if !f.dispatches {
        FrameKind::Hold
    } else if f.accum {
        FrameKind::Accum
    } else if f.tile {
        FrameKind::Tile
    } else if f.chunked {
        FrameKind::SettledWalk
    } else {
        FrameKind::Settled
    }
}

/// A resumable walk's identity. A walk RESUMES its per-pixel state only while this is unchanged,
/// so it must change whenever anything that shapes the walk does — and a sig that misses an input
/// resumes a walk against a view it was not computed for (the translucent copy of another location,
/// 2026-09-16; a formula switch re-showing the previous formula; a series refined mid-walk).
///
/// ⭐[`WalkSig::of`] is the ONLY constructor, and the fields are private to this module: a walk
/// identity assembled anywhere else would be the place an input gets forgotten.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) struct WalkSig {
    /// The view, the reference length, the exact position, the settings and the series seed,
    /// folded together (`bp_chunk_tiling`'s `view_bits`), with any supersampling jitter.
    view: u64,
    /// The iteration ask the walk targets.
    ask: u32,
    /// The texture it iterates, and its supersampling factor.
    res: [u32; 2],
    ss: u32,
}

impl WalkSig {
    /// No walk: fresh state, or a harness forcing a restart. A zero sig says nothing about the view.
    pub(crate) const NONE: WalkSig = WalkSig { view: 0, ask: 0, res: [0, 0], ss: 0 };

    /// The walk identity of a view. `jitter_bits` is the supersampling sample's sub-pixel shift
    /// (0 for the jitter-free walk VIEW, which keys the band ledger): each sample is a shifted view,
    /// so a new one must restart the walk.
    pub(crate) const fn of(view_bits: u64, jitter_bits: u64, ask: u32, res: [u32; 2], ss: u32) -> WalkSig {
        WalkSig { view: view_bits ^ jitter_bits.rotate_left(7), ask, res, ss }
    }

    /// The iteration ask the walk targets.
    pub(crate) fn ask(self) -> u32 {
        self.ask
    }

    /// The texture it iterates.
    pub(crate) fn res(self) -> [u32; 2] {
        self.res
    }

    pub(crate) fn ss(self) -> u32 {
        self.ss
    }

    /// One word for the frame record: equal sigs give equal words; a change in any part changes
    /// it (up to a hash collision).
    pub(crate) fn word(self) -> u64 {
        if self == WalkSig::NONE {
            return 0;
        }
        let mut h = self.view;
        for x in [self.ask as u64, ((self.res[0] as u64) << 32) | self.res[1] as u64, self.ss as u64] {
            h = (h ^ x).wrapping_mul(0x0000_0100_0000_01B3).rotate_left(29);
        }
        h | 1
    }
}

/// Did starting a new walk throw progress away? The walk being replaced had advanced past its
/// first pass (`cursor > 0`) and not reached its end. A moving frame's one-pass preview never
/// commits its cursor, so restarting it discards nothing; a completed walk has nothing left to
/// lose. What is left is the W4 regression shape: a settled walk or a pinned refresh discarded
/// part-way — by a resolution rung, a jitter, an abandon — and the frame shows its blank start.
pub(crate) fn walk_restart_discards(prev_cursor: u32, prev_walk_end: u32) -> bool {
    prev_cursor > 0 && prev_cursor < prev_walk_end
}

/// A JUMP IN is a frame that zooms in by more than this many octaves... ([`is_jump_in`]).
pub(crate) const JUMP_OCT_PER_FRAME: f64 = 1.5;
/// ...and by more than this many times the previous frame's step: a step from rest, not a rate.
pub(crate) const JUMP_SUDDEN: f64 = 4.0;

/// Did this frame's view JUMP in? `d_l2` is the frame's change in log2 magnification (positive =
/// in), `prev_step` the previous frame's |change| (NaN on a view's first frame, which is no jump).
///
/// ⚠Not a speed threshold. A continuous zoom can move any distance in a frame — the Home glide is a
/// smoothstep over at most 9 s, so from 1e31 it peaks near 0.3 octaves a frame and from 1e30000 at
/// hundreds, and a tour plays at whatever rate its keyframes ask. What a jump has that a glide does
/// not is SUDDENNESS: click-to-zoom, a box zoom, a bookmark, Go to and undo move the view from rest
/// in one frame, while every glide ramps (the Space glide and the auto-zoom ease their speed, the
/// Home glide and tours are eased curves). And only a jump IN matters: the floor bounds
/// magnification, and a zoom-out shows the held frame shrunk (scale > 1), which no floor touches.
pub(crate) fn is_jump_in(d_l2: f64, prev_step: f64) -> bool {
    let prev = if prev_step.is_finite() { prev_step } else { 0.0 };
    d_l2 > JUMP_OCT_PER_FRAME && d_l2 > JUMP_SUDDEN * prev
}

/// ⭐U39: does the HELD frame now stand across a jump? This replaces "no zoom velocity means a
/// jump", which only a Space glide could ever answer correctly — `zoom_vel` is written by Space
/// alone, so every auto-zoom dive, wheel zoom, Home glide and tour took the jump floor, and a held
/// frame that fell more than 3 octaves behind such a zoom was shown at the clamped scale: the wrong
/// magnification, translated too far, sliding against the moving view (`HELD_JUMP_MAX_OCT`'s own
/// doc says why a moving frame must keep its true scale). Measured on `--motiontest`'s Home glide
/// before the change: every moving frame took the jump floor, and it bound on 7 of them.
///
/// Sticky: set by a frame that jumped in ([`is_jump_in`]), kept while that held frame stays on
/// screen — after a jump the view is static, and a clamped transform has nothing to slide against —
/// and cleared when a new frame is latched as the held one (`latched`), since that frame is at the
/// new view.
pub(crate) fn hold_jump_next(prev: bool, jumped_in: bool, latched: bool) -> bool {
    if latched {
        false
    } else {
        prev || jumped_in
    }
}

/// Which bound a held frame's magnification took this frame (frame record `hold_floor`; U39):
/// 0 none (no hold transform), 1 the dive floor, 2 the jump floor without binding, 3 the jump
/// floor BINDING — the held frame is shown at less magnification than the view's, so it is the
/// wrong scale on screen — and 4 the dive floor with the held frame magnified PAST the jump floor
/// (more than `HELD_JUMP_MAX_OCT` behind: true scale, but a big flat patch; after a jump it would
/// be the 100× patch the jump floor exists to prevent). Pending per frame; the highest code wins.
pub(crate) mod hold_floor {
    // 0 (none) is the record's default: no hold transform was taken this frame.
    pub(crate) const DIVE: u8 = 1;
    pub(crate) const JUMP: u8 = 2;
    pub(crate) const JUMP_BOUND: u8 = 3;
    pub(crate) const DIVE_DEEP: u8 = 4;
}

/// The held frame's reprojection scale for `gap_oct` octaves of zoom since it was rendered
/// (`frozen_l2 − l2_now`: negative as the view dives in), bounded by [`super::hold_scale_floor`],
/// and the [`hold_floor`] code saying which bound applied.
pub(crate) fn hold_scale(gap_oct: f64, jumped: bool) -> (f32, u8) {
    let floor = super::hold_scale_floor(jumped);
    let raw = (gap_oct as f32).exp2();
    let scale = raw.clamp(floor, 1.099_512e12);
    let code = if !jumped {
        if raw < super::hold_scale_floor(true) { hold_floor::DIVE_DEEP } else { hold_floor::DIVE }
    } else if raw < floor {
        hold_floor::JUMP_BOUND
    } else {
        hold_floor::JUMP
    };
    (scale, code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settled() -> FrameFacts {
        FrameFacts { has_frozen: true, dispatches: true, ..Default::default() }
    }

    #[test]
    fn a_reprojection_with_nothing_behind_it_is_a_placeholder() {
        let f = FrameFacts { reproject: true, has_frozen: false, ..Default::default() };
        assert_eq!(classify(f), FrameKind::Placeholder);
        assert_eq!(classify(FrameFacts { has_frozen: true, ..f }), FrameKind::Reproject);
    }

    #[test]
    fn a_reprojection_wins_over_everything_it_cancels() {
        // The freeze verdict turns a planned pass into a reprojection: nothing iterated, whatever
        // was planned.
        let f = FrameFacts {
            reproject: true,
            has_frozen: true,
            interacting: true,
            pin_pass: true,
            dispatches: true,
            chunked: true,
            ..Default::default()
        };
        assert_eq!(classify(f), FrameKind::Reproject);
    }

    #[test]
    fn moving_frames_are_pins_or_motion() {
        let moving = FrameFacts { interacting: true, ..settled() };
        assert_eq!(classify(moving), FrameKind::Motion);
        assert_eq!(classify(FrameFacts { chunked: true, ..moving }), FrameKind::Motion);
        assert_eq!(classify(FrameFacts { pin_pass: true, chunked: true, ..moving }), FrameKind::PinWalk);
        // A pin's passes keep composing once the user stops: still the pin's.
        assert_eq!(classify(FrameFacts { pin_pass: true, ..settled() }), FrameKind::PinWalk);
    }

    #[test]
    fn settled_frames_split_by_what_they_dispatch() {
        assert_eq!(classify(settled()), FrameKind::Settled);
        assert_eq!(classify(FrameFacts { dispatches: false, ..settled() }), FrameKind::Hold);
        assert_eq!(classify(FrameFacts { tile: true, ..settled() }), FrameKind::Tile);
        assert_eq!(classify(FrameFacts { chunked: true, ..settled() }), FrameKind::SettledWalk);
        assert_eq!(classify(FrameFacts { accum: true, chunked: true, ..settled() }), FrameKind::Accum);
        // A tile that waits its turn dispatches nothing.
        assert_eq!(classify(FrameFacts { tile: true, dispatches: false, ..settled() }), FrameKind::Hold);
    }

    #[test]
    fn every_kind_has_one_name_and_a_distinct_nonzero_code() {
        let mut codes: Vec<u8> = FrameKind::NAMES.iter().map(|&(c, _)| c).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), FrameKind::NAMES.len());
        assert!(!codes.contains(&0), "0 is 'not built this frame'");
        for k in [
            FrameKind::Placeholder,
            FrameKind::Reproject,
            FrameKind::PinWalk,
            FrameKind::Motion,
            FrameKind::Hold,
            FrameKind::SettledWalk,
            FrameKind::Tile,
            FrameKind::Accum,
            FrameKind::Settled,
        ] {
            assert!(FrameKind::NAMES.iter().any(|&(c, _)| c == k.code()), "{k:?} has no name");
        }
    }

    #[test]
    fn framelog_names_the_kinds_as_the_code_does() {
        // The reader decodes the record by these names; a kind added here and not there would
        // print as "?" in every summary.
        let script = include_str!("../../../../scripts/framelog.py");
        for (code, name) in FrameKind::NAMES {
            assert!(
                script.contains(&format!("{code}: \"{name}\"")),
                "scripts/framelog.py KIND lacks {code}: {name:?}"
            );
        }
    }

    #[test]
    fn the_jitter_free_sig_is_the_view_and_a_sample_differs_only_there() {
        let view = WalkSig::of(0xABCD, 0, 10_000, [551, 870], 1);
        let sample = WalkSig::of(0xABCD, 0x55, 10_000, [551, 870], 1);
        assert_ne!(view, sample);
        assert_eq!((view.ask(), view.res(), view.ss()), (sample.ask(), sample.res(), sample.ss()));
        // The historical fold, so the sig is the same value it was as a tuple.
        assert_eq!(sample.view, 0xABCD ^ 0x55u64.rotate_left(7));
    }

    #[test]
    fn two_walks_differing_only_in_resolution_never_share_a_sig() {
        // §7.4: a resolution change restarts the walk — the walk-restart class is about the cost of
        // that, never about skipping it. Property over a spread of sizes.
        for w in [16_u32, 145, 551, 1452, 3840] {
            for h in [16_u32, 110, 870, 1102, 2160] {
                let a = WalkSig::of(0x1234, 0, 50_000, [w, h], 1);
                for (dw, dh) in [(1, 0), (0, 1), (w / 2, h / 2)] {
                    if dw == 0 && dh == 0 {
                        continue;
                    }
                    let b = WalkSig::of(0x1234, 0, 50_000, [w + dw, h + dh], 1);
                    assert_ne!(a, b);
                    assert_ne!(a.word(), b.word(), "{w}x{h} vs {}x{}", w + dw, h + dh);
                }
                let ss2 = WalkSig::of(0x1234, 0, 50_000, [w, h], 2);
                assert_ne!(a.word(), ss2.word());
            }
        }
    }

    #[test]
    fn the_record_word_is_zero_only_for_no_walk() {
        assert_eq!(WalkSig::NONE.word(), 0);
        assert_eq!(WalkSig::default(), WalkSig::NONE);
        assert_ne!(WalkSig::of(0, 0, 1, [16, 16], 1).word(), 0);
    }

    #[test]
    fn only_a_walk_part_way_through_is_lost_by_a_restart() {
        assert!(!walk_restart_discards(0, 10_000), "a motion preview never commits its cursor");
        assert!(walk_restart_discards(256, 10_000));
        assert!(walk_restart_discards(9_999, 10_000));
        assert!(!walk_restart_discards(10_000, 10_000), "a completed walk has nothing left to lose");
        assert!(!walk_restart_discards(0, 0), "no walk");
    }

    #[test]
    fn continuous_motion_never_reads_as_a_jump() {
        // U39: the auto-zoom, the wheel, the Home glide and tours used to take the jump floor
        // because only Space wrote a zoom velocity. A zoom that ramps is motion at any speed.
        // The Space glide / auto-zoom: eased, at most ~0.27 octaves a frame.
        let mut prev = f64::NAN;
        for step in [0.01, 0.05, 0.12, 0.2, 0.27, 0.27, 0.27] {
            assert!(!is_jump_in(step, prev), "a {step}-octave glide step");
            prev = step;
        }
        // A smoothstep dive from rest at 60 fps, fast enough to move hundreds of octaves a frame
        // at its peak (the Home glide's curve, run in): never a jump.
        let total = 99_658.0_f64; // octaves, 1e30000
        let n = 9 * 60;
        let ease = |u: f64| u * u * (3.0 - 2.0 * u);
        let mut prev = f64::NAN;
        let mut jumps = 0;
        for i in 1..=n {
            let d = total * (ease(i as f64 / n as f64) - ease((i - 1) as f64 / n as f64));
            jumps += is_jump_in(d, prev) as u32;
            prev = d;
        }
        // Its peak is ~277 octaves a frame, and its first frame from rest ~1.02 — under the jump
        // threshold, and every later step under four times the one before.
        assert_eq!(jumps, 0, "an eased curve never reads as a jump, at any depth");
        // From 1e31 the first frame moves ~0.001 octaves.
        let first = 103.0 * ease(1.0 / n as f64);
        assert!(first < 0.01 && !is_jump_in(first, f64::NAN));
        // Zooming OUT is never a jump in, however far.
        assert!(!is_jump_in(-6.64, 0.0));
        assert!(!is_jump_in(-1.0e4, 0.0));
        // A view's first frame has nothing to compare against.
        assert!(!is_jump_in(f64::NAN, f64::NAN));
    }

    #[test]
    fn a_click_from_rest_is_a_jump_and_holds_until_a_new_frame_is_latched() {
        // The 100× click (6.64 octaves) of the black-patch report, from a still view.
        assert!(is_jump_in(6.64, 0.0));
        assert!(is_jump_in(6.64, f64::NAN));
        // 4× (2 octaves) from rest too; 2× is under the floor's reach on its own.
        assert!(is_jump_in(2.0, 0.0));
        assert!(!is_jump_in(1.0, 0.0));
        // A click during a glide is sudden against the glide's step.
        assert!(is_jump_in(6.64, 0.27));
        let mut j = hold_jump_next(false, true, false);
        assert!(j);
        // The view sits still while the reference builds: the held frame still stands across it.
        for _ in 0..8 {
            j = hold_jump_next(j, false, false);
            assert!(j);
        }
        // A real frame at the new view replaces the held one: no jump between them any more.
        j = hold_jump_next(j, false, true);
        assert!(!j);
        // A jump and a latch on the same frame: the latched frame is at the new view.
        assert!(!hold_jump_next(false, true, true));
    }

    #[test]
    fn the_hold_floor_code_says_whether_the_jump_bound_moved_the_picture() {
        // A dive keeps its true scale and says so — and says when that is past the jump floor.
        let (s, c) = hold_scale(-1.0, false);
        assert_eq!((s, c), (0.5, hold_floor::DIVE));
        let (s, c) = hold_scale(-6.64, false);
        assert_eq!(c, hold_floor::DIVE_DEEP);
        assert!((s - (-6.64_f32).exp2()).abs() < 1.0e-6);
        // A jump within the bound is shown at its true scale.
        let (s, c) = hold_scale(-1.0, true);
        assert_eq!((s, c), (0.5, hold_floor::JUMP));
        // A jump past it is clamped, and the code says the screen shows the wrong scale.
        let (s, c) = hold_scale(-6.64, true);
        assert_eq!(c, hold_floor::JUMP_BOUND);
        assert_eq!(s, super::super::hold_scale_floor(true));
        // Zooming out is never floored.
        assert_eq!(hold_scale(2.0, true), (4.0, hold_floor::JUMP));
    }
}
