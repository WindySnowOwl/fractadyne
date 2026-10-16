use super::*;

impl FrameKind {
    /// The record's codes, by name. `scripts/framelog.py`'s `KIND` table must say the same, and a
    /// test holds it to that.
    const NAMES: [(u8, &'static str); 9] = [
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

/// §7.4's exit criterion, as a test rather than a grep someone has to remember to run: outside
/// this module and the pure `decide` it calls (W5: the scale is DECIDED in `decide`, applied
/// here), nothing WRITES a frame's scale — no `let res_scale` / `let budget_res_scale` computed in
/// place, no assignment to `perf.visible_res[..]` or `perf.motion_res[..]` — and a frame's
/// `Resolution` is made in exactly one place. A binding of a `frame_plan` function's result is
/// not a writer. Scans every source file of the crate at test time.
#[test]
fn nothing_outside_frame_plan_writes_a_frames_scale_or_makes_its_resolution() {
    fn files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).expect("src dir").flatten() {
            let p = e.path();
            if p.is_dir() {
                files(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut all = Vec::new();
    files(&src, &mut all);
    assert!(all.len() > 20, "the scan must see the crate (found {} files)", all.len());
    // `src/` holds no test file (the tests, which make their own, live under `tests/unit/`).
    let texts: Vec<(std::path::PathBuf, String)> =
        all.iter().map(|p| (p.clone(), std::fs::read_to_string(p).expect("readable source"))).collect();
    let mut writers = Vec::new();
    let mut panel_makers = 0;
    for (p, text) in &texts {
        if p.file_name().is_some_and(|n| n == "frame_plan.rs" || n == "decide.rs") {
            continue;
        }
        for (i, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            let binds = ["let res_scale =", "let budget_res_scale ="].iter().any(|k| code.contains(k));
            let assigns = ["perf.visible_res[", "perf.motion_res["].iter().any(|k| {
                code.find(k).is_some_and(|at| {
                    let rest = &code[at..];
                    rest.find(']').is_some_and(|e| {
                        let after = rest[e + 1..].trim_start();
                        after.starts_with('=') && !after.starts_with("==")
                    })
                })
            });
            if (binds || assigns) && !code.contains("frame_plan::") {
                writers.push(format!("{}:{}: {}", p.display(), i + 1, line.trim()));
            }
            if code.contains("Resolution::panel(") {
                panel_makers += 1;
            }
        }
    }
    assert!(writers.is_empty(), "scale writers outside frame_plan:\n{}", writers.join("\n"));
    // `render.rs` makes the frame's resolution once (top of `build_params`); tests make their own.
    assert_eq!(panel_makers, 1, "a frame's Resolution must be made in exactly one place outside tests");
}

#[test]
fn framelog_names_the_kinds_as_the_code_does() {
    // The reader decodes the record by these names; a kind added here and not there would
    // print as "?" in every summary.
    let script = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/framelog.py"));
    for (code, name) in FrameKind::NAMES {
        assert!(
            script.contains(&format!("{code}: \"{name}\"")),
            "scripts/framelog.py KIND lacks {code}: {name:?}"
        );
    }
}

#[test]
fn the_jitter_free_sig_is_the_view_and_a_sample_differs_only_there() {
    let view = WalkSig::of(0xABCD, 0, 10_000, Resolution::panel([551, 870]), 1);
    let sample = WalkSig::of(0xABCD, 0x55, 10_000, Resolution::panel([551, 870]), 1);
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
            let a = WalkSig::of(0x1234, 0, 50_000, Resolution::panel([w, h]), 1);
            for (dw, dh) in [(1, 0), (0, 1), (w / 2, h / 2)] {
                if dw == 0 && dh == 0 {
                    continue;
                }
                let b = WalkSig::of(0x1234, 0, 50_000, Resolution::panel([w + dw, h + dh]), 1);
                assert_ne!(a, b);
                assert_ne!(a.word(), b.word(), "{w}x{h} vs {}x{}", w + dw, h + dh);
            }
            let ss2 = WalkSig::of(0x1234, 0, 50_000, Resolution::panel([w, h]), 2);
            assert_ne!(a.word(), ss2.word());
        }
    }
}

#[test]
fn the_record_word_is_zero_only_for_no_walk() {
    assert_eq!(WalkSig::NONE.word(), 0);
    assert_eq!(WalkSig::default(), WalkSig::NONE);
    assert_ne!(WalkSig::of(0, 0, 1, Resolution::panel([16, 16]), 1).word(), 0);
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
