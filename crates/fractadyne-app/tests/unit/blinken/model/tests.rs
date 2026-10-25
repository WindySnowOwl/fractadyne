use super::*;

fn ev(frame: u64, kind: Kind, a: u32, b: u32, f: f32) -> Event {
    Event { seq: 0, frame, t_us: 0, view: 0, kind, a, b, f }
}

fn snap(frame: u64) -> Snapshot {
    Snapshot { frame, dt_s: 1.0 / 60.0, frame_ms: 16.7, res: (436, 330), iterations: 30_000, ..Snapshot::default() }
}

#[test]
fn a_walk_collects_its_passes_pairs_each_timing_by_frame_and_ends_on_adoption() {
    let mut m = Model::default();
    m.update(&[ev(10, Kind::PinStart, 30_000, 436 << 16 | 330, 0.0), ev(10, Kind::PinPass, 256, 512, 0.0)], snap(10));
    m.update(&[ev(11, Kind::PinPass, 512, 1024, 0.0)], snap(11));
    // The first pass's timing lands two frames later, naming the frame it ran in.
    m.update(&[ev(12, Kind::PassTimed, Pass::Pin as u32, 10, 3.5)], snap(12));
    let w = m.walk.as_ref().expect("walking");
    assert_eq!(w.res, (436, 330));
    assert_eq!(w.cursor, 1024);
    assert_eq!(w.passes.len(), 2);
    assert_eq!(w.passes[0].ms, Some(3.5));
    assert_eq!(w.passes[1].ms, None);
    assert!(m.lamp(Lamp::PinPass) > 0.5);
    // The scope puts the timing in the frame it ran in, not the one it arrived in.
    let c10 = m.columns.iter().find(|c| c.frame == 10).expect("column 10");
    assert_eq!(c10.ms[Pass::Pin as usize], 3.5);

    m.update(&[ev(13, Kind::PinVerify, 0, 0, 0.0)], snap(13));
    assert!(m.walk.as_ref().unwrap().verifying);
    m.update(&[ev(16, Kind::PinAdopt, 0, 0, 0.4)], snap(16));
    assert!(m.flash > 0.9);
    assert!(m.lamp(Lamp::Adopt) > 0.9);
    let w = m.walk.as_ref().expect("fading");
    assert!(w.ended && !w.verifying);
    // It fades off the radar within its afterglow.
    for f in 17..17 + 60 {
        m.update(&[], snap(f));
    }
    assert!(m.walk.is_none());
}

#[test]
fn a_pass_whose_timing_never_comes_is_marked_untimed() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::PinStart, 4096, 436 << 16 | 330, 0.0), ev(1, Kind::PinPass, 256, 512, 0.0)], snap(1));
    for f in 2..=1 + UNTIMED_AFTER_FRAMES {
        m.update(&[], snap(f));
        assert!(!m.walk.as_ref().unwrap().passes[0].untimed, "frame {f}");
    }
    m.update(&[], snap(2 + UNTIMED_AFTER_FRAMES));
    assert!(m.walk.as_ref().unwrap().passes[0].untimed);
}

#[test]
fn a_split_refresh_shows_its_set_on_the_plane_and_counts_its_passes() {
    let mut m = Model::default();
    let mut s = snap(5);
    s.res = (1455, 1102);
    m.update(&[ev(5, Kind::PinStart, 20_000, 1455 << 16 | 1102, 0.0), ev(5, Kind::SplitPass, 0, 3, 0.0)], s);
    match &m.plane {
        Plane::Split { res, k, j, .. } => assert_eq!((*res, *k, *j), ((1455, 1102), 3, 0)),
        p => panic!("plane {p:?}"),
    }
    m.update(&[ev(6, Kind::SplitPass, 1, 3, 0.0)], snap(6));
    let w = m.walk.as_ref().unwrap();
    assert_eq!(w.split, Some((2, 3)));
    assert_eq!(w.passes.len(), 2);
    assert!(m.lamp(Lamp::Split) > 0.5);
}

#[test]
fn a_settle_grid_lights_tiles_as_they_finish_strips_included() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::GridStart, 1455 << 16 | 1102, 256, 0.0)], snap(1));
    match &m.plane {
        Plane::Grid(g) => {
            assert_eq!((g.cols, g.rows, g.side, g.res), (6, 5, 256, (1455, 1102)));
            assert_eq!(g.done.len(), 30);
        }
        p => panic!("plane {p:?}"),
    }
    // Tile 14 (256 rows) goes out in two unequal strips: done only after the second.
    m.update(&[ev(2, Kind::TileSent, 14, 160 << 16 | 256, 160.0)], snap(2));
    let Plane::Grid(g) = &m.plane else { panic!() };
    assert!(!g.done[14]);
    assert_eq!(g.last, Some((14, 160, 256, 160)));
    m.update(&[ev(3, Kind::TileSent, 14, 256 << 16 | 256, 96.0)], snap(3));
    let Plane::Grid(g) = &m.plane else { panic!() };
    assert!(g.done[14]);
    assert_eq!(g.last, Some((14, 256, 256, 96)));
    // A whole tile is one piece of all its rows.
    m.update(&[ev(4, Kind::TileSent, 15, 256 << 16 | 256, 256.0)], snap(4));
    let Plane::Grid(g) = &m.plane else { panic!() };
    assert!(g.done[15]);
    // A frame with no pass keeps the grid on the plane (it is between tiles, not idle).
    m.update(&[], snap(5));
    assert!(matches!(m.plane, Plane::Grid(_)));
    m.update(&[ev(6, Kind::GridDone, 0, 0, 0.0)], snap(6));
    let Plane::Grid(g) = &m.plane else { panic!() };
    assert!(g.complete && g.done.iter().all(|d| *d));
    m.update(&[ev(7, Kind::SsaaSample, 1, 0, 0.0)], snap(7));
    assert!(matches!(m.plane, Plane::Ssaa { res: (1455, 1102), samples: 1, .. }));
}

#[test]
fn a_moving_frame_covers_the_whole_picture_and_a_frame_with_no_pass_says_why() {
    let mut m = Model::default();
    let mut s = snap(1);
    s.dispatched = Some(Pass::Moving);
    s.res = (728, 551);
    s.present = Present::Reprojected;
    m.update(&[], s);
    assert!(matches!(m.plane, Plane::Whole { res: (728, 551), band: None, .. }));
    let mut s = snap(2);
    s.present = Present::Held;
    m.update(&[], s);
    assert_eq!(m.plane, Plane::Idle("HELD"));
    assert!(m.lamp(Lamp::Held) > 0.5);
}

#[test]
fn builders_hold_their_lamps_lit_and_leave_a_ring_or_a_cross() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::RefLaunch, RefRole::Ahead as u32, 7, 101.5), ev(1, Kind::RefLaunch, RefRole::Hold as u32, 8, 100.2)], snap(1));
    assert_eq!(m.pucks.len(), 2);
    for f in 2..30 {
        m.update(&[], snap(f));
        assert!(m.lamp(Lamp::RefAhead) > 0.5, "lit while building, frame {f}");
    }
    m.update(&[ev(30, Kind::RefBuilt, RefRole::Ahead as u32, 7, 101.5), ev(30, Kind::RefInstalled, RefRole::Ahead as u32, 7, 101.5), ev(30, Kind::RefDropped, RefRole::Hold as u32, 8, 100.2)], snap(30));
    assert!(m.pucks.iter().any(|p| p.id == 7 && p.state == PuckState::Installed));
    assert!(m.pucks.iter().any(|p| p.id == 8 && p.state == PuckState::Dropped));
    for f in 31..31 + 60 {
        m.update(&[], snap(f));
    }
    assert!(m.pucks.is_empty(), "rings and crosses expire");
    assert!(m.lamp(Lamp::RefAhead) < 0.05, "and the lamp goes dark");
}

#[test]
fn lamps_fade_with_the_incandescent_time_constant() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::ModeSwitch, Arith::Df32 as u32, Arith::Floatexp as u32, 0.0)], snap(1));
    assert!((m.lamp(Lamp::Mode) - 1.0).abs() < 1e-6);
    // 150 ms later it is at 1/e.
    let mut s = snap(2);
    s.dt_s = LAMP_TAU_S;
    m.update(&[], s);
    assert!((m.lamp(Lamp::Mode) - (-1.0f32).exp()).abs() < 1e-4);
}

#[test]
fn the_age_needle_follows_its_reading_and_the_strip_keeps_the_raw_values() {
    let mut m = Model::default();
    let mut s = snap(1);
    s.age_oct = 1.0;
    m.update(&[], s);
    assert!(m.age > 0.0 && m.age < 1.0, "the needle lags: {}", m.age);
    assert_eq!(m.age_hist.back(), Some(&1.0));
    for f in 2..60 {
        let mut s = snap(f);
        s.age_oct = 1.0;
        m.update(&[], s);
    }
    assert!((m.age - 1.0).abs() < 1e-3);
}

#[test]
fn a_timing_names_its_frame_by_its_low_bits_across_a_wrap() {
    let m = Model::default();
    let near = (1u64 << 32) + 3;
    // A pass from frame 2^32 - 2, timed at 2^32 + 3.
    assert_eq!(m.full_frame(u32::MAX - 1, near), (1u64 << 32) - 2);
    assert_eq!(m.full_frame(1, near), (1u64 << 32) + 1);
}

#[test]
fn the_engines_share_counts_adoptions_by_source() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::PinAdopt, 0, 0, 0.0), ev(1, Kind::WorkerFrame, 1, 0, 4.0), ev(1, Kind::WorkerFrame, 1, 0, 4.0), ev(1, Kind::WorkerFrame, 0, 0, 4.0)], snap(1));
    assert_eq!(m.shown_share(), (0.25, 0.75));
}

#[test]
fn a_legend_name_lights_when_its_kind_runs_and_fades_like_a_lamp() {
    let name = |k: Pass| legend_of(k);
    let mut m = Model::default();
    // A pin pass dispatched lights PIN at once, and nothing else.
    m.update(&[ev(1, Kind::PinStart, 30_000, 436 << 16 | 330, 0.0), ev(1, Kind::PinPass, 256, 512, 0.0)], snap(1));
    assert_eq!(m.legend[name(Pass::Pin)], 1.0);
    for k in [Pass::Moving, Pass::Worker, Pass::Walk, Pass::Ssaa] {
        assert_eq!(m.legend[name(k)], 0.0, "{k:?}");
    }
    assert!(m.active(), "a lit name keeps the panel drawing while it fades");
    // Its reading two frames later is not news: the name keeps fading rather than lighting again.
    let mut s = snap(3);
    s.dt_s = LAMP_TAU_S;
    m.update(&[ev(3, Kind::PassTimed, Pass::Pin as u32, 1, 4.0)], s);
    assert!((m.legend[name(Pass::Pin)] - (-1.0f32).exp()).abs() < 1e-4, "{}", m.legend[name(Pass::Pin)]);
    // A kind first heard of from its reading (the worker's frames) lights when it arrives (for a
    // frame the scope still has; every frame opens one).
    m.update(&[ev(4, Kind::PassTimed, Pass::Worker as u32, 3, 6.0)], snap(4));
    assert_eq!(m.legend[name(Pass::Worker)], 1.0);
    // The frame's own pass: a moving frame and a settle tile light MOVING and SETTLE.
    let mut s = snap(5);
    s.dispatched = Some(Pass::Moving);
    m.update(&[], s);
    assert_eq!(m.legend[name(Pass::Moving)], 1.0);
    let mut s = snap(6);
    s.dispatched = Some(Pass::Tile);
    m.update(&[], s);
    assert_eq!(m.legend[name(Pass::Tile)], 1.0);
    assert_eq!(name(Pass::Tile), name(Pass::Walk));
    assert_eq!(name(Pass::Split), name(Pass::Pin));
}

#[test]
fn a_reading_of_the_frames_own_pass_is_filed_under_the_kind_that_frame_ran() {
    let mut m = Model::default();
    let mut s = snap(20);
    s.dispatched = Some(Pass::Tile);
    m.update(&[], s);
    m.update(&[ev(22, Kind::PassTimed, PASS_OF_FRAME, 20, 12.5)], snap(22));
    let c = m.columns.iter().find(|c| c.frame == 20).expect("column 20");
    assert_eq!(c.ms[Pass::Tile as usize], 12.5);
    assert_eq!(c.ms[Pass::Moving as usize], 0.0);
}

#[test]
fn a_still_panel_says_so_and_an_event_wakes_it() {
    let mut m = Model::default();
    for f in 1..200 {
        m.update(&[], snap(f));
    }
    assert!(!m.active(), "nothing happened: the panel can idle");
    m.update(&[ev(200, Kind::PaletteFeed, 10, 900, 0.0)], snap(200));
    assert!(m.active());
}

/// A frame of a still view's walk: `dispatched` is what the frame ran (None for a hold between passes).
fn still(frame: u64, dispatched: Option<Pass>) -> Snapshot {
    Snapshot { dispatched, res: (2400, 1300), iterations: 396_661, present: Present::Held, ..snap(frame) }
}

#[test]
fn a_still_views_walk_sweeps_the_radar_coloured_against_its_own_target_and_ends_when_its_last_pass_is_timed() {
    let mut m = Model::default();
    let res = 2400 << 16 | 1300;
    m.update(&[ev(10, Kind::WalkStart, 396_661, res, 0.0), ev(10, Kind::WalkPass, 147_000, 200_000, 400.0)], still(10, Some(Pass::Walk)));
    let w = m.walk.as_ref().expect("walking");
    assert_eq!((w.kind, w.ask, w.res, w.cursor), (WalkKind::Settle, 396_661, (2400, 1300), 200_000));
    assert_eq!(w.passes[0].target_ms, 400.0);
    assert!(!w.verifying);
    assert_eq!(m.plane, Plane::Whole { res: (2400, 1300), band: Some((147_000, 200_000)), settled: true, glow: 1.0 });
    // Frames between passes hold: the plane keeps the pass in flight rather than going idle.
    for f in 11..14 {
        m.update(&[], still(f, None));
        assert!(matches!(m.plane, Plane::Whole { band: Some((147_000, 200_000)), settled: true, .. }), "frame {f}: {:?}", m.plane);
    }
    // The pass's reading is the frame's own, filed under the walk.
    m.update(&[ev(14, Kind::PassTimed, PASS_OF_FRAME, 10, 310.0)], still(14, None));
    assert_eq!(m.walk.as_ref().unwrap().passes[0].ms, Some(310.0));
    let c10 = m.columns.iter().find(|c| c.frame == 10).expect("column 10");
    assert_eq!(c10.ms[Pass::Walk as usize], 310.0);
    // The last pass: the walk waits for its reading, then ends and fades.
    m.update(&[ev(15, Kind::WalkPass, 200_000, 396_661, 400.0)], still(15, Some(Pass::Walk)));
    assert!(m.walk.as_ref().unwrap().verifying);
    m.update(&[ev(18, Kind::PassTimed, PASS_OF_FRAME, 15, 280.0)], still(18, None));
    let w = m.walk.as_ref().unwrap();
    assert!(w.ended && !w.verifying);
    for f in 19..19 + 60 {
        m.update(&[], still(f, None));
    }
    assert!(m.walk.is_none());
    assert_eq!(m.plane, Plane::Idle("HELD"), "the walk over, an idle frame says why again");
}

#[test]
fn a_supersampling_samples_walk_shows_its_band_and_the_samples_done_on_the_plane() {
    let mut m = Model::default();
    let res = 2400 << 16 | 1300;
    m.update(&[ev(1, Kind::SsaaSample, 1, 0, 0.0)], still(1, None));
    m.update(&[ev(2, Kind::WalkStart, 396_661, res, 1.0), ev(2, Kind::WalkPass, 0, 65_536, 400.0)], still(2, Some(Pass::Ssaa)));
    assert_eq!(m.walk.as_ref().unwrap().kind, WalkKind::Ssaa);
    assert_eq!(m.plane, Plane::Ssaa { res: (2400, 1300), samples: 1, band: Some((0, 65_536)), glow: 1.0 });
    m.update(&[ev(5, Kind::PassTimed, PASS_OF_FRAME, 2, 120.0)], still(5, None));
    assert_eq!(m.walk.as_ref().unwrap().passes[0].ms, Some(120.0));
    assert_eq!(m.columns.iter().find(|c| c.frame == 2).unwrap().ms[Pass::Ssaa as usize], 120.0);
    // Folded: the plane counts it and drops the band until the next sample walks.
    m.update(&[ev(6, Kind::SsaaSample, 2, 0, 0.0)], still(6, None));
    assert_eq!(m.plane, Plane::Ssaa { res: (2400, 1300), samples: 2, band: None, glow: 1.0 });
    // Each sample's walk starts the radar again.
    m.update(&[ev(7, Kind::WalkStart, 396_661, res, 1.0), ev(7, Kind::WalkPass, 0, 32_768, 400.0)], still(7, Some(Pass::Ssaa)));
    let w = m.walk.as_ref().unwrap();
    assert_eq!((w.passes.len(), w.cursor), (1, 32_768));
}

#[test]
fn a_long_pass_is_not_called_untimed_before_twice_its_target_has_passed() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::WalkStart, 396_661, 2400 << 16 | 1300, 0.0), ev(1, Kind::WalkPass, 0, 396_661, 400.0)], still(1, Some(Pass::Walk)));
    // 60 fps hold frames: 8 frames is 133 ms, far inside a 400 ms pass (to 0.65 s here).
    for f in 2..=40 {
        m.update(&[], still(f, None));
        let w = m.walk.as_ref().unwrap();
        assert!(!w.passes[0].untimed && !w.ended, "frame {f}");
    }
    // Past 0.8 s with no reading: hollow, and the walk is over (nothing else would end it).
    for f in 41..=60 {
        m.update(&[], still(f, None));
    }
    let w = m.walk.as_ref().unwrap();
    assert!(w.passes[0].untimed && w.ended);
}

#[test]
fn a_still_views_walk_ends_when_the_view_moves_or_nothing_is_heard_from_it() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::WalkStart, 396_661, 2400 << 16 | 1300, 0.0), ev(1, Kind::WalkPass, 0, 50_000, 400.0)], still(1, Some(Pass::Walk)));
    m.update(&[], still(2, Some(Pass::Moving)));
    assert!(m.walk.as_ref().unwrap().ended, "a moving frame ends it");
    assert!(matches!(m.plane, Plane::Whole { band: None, settled: false, .. }));
    // A still view rendered whole in one pass is a settled frame, not a moving one.
    m.update(&[], still(3, Some(Pass::Walk)));
    assert!(matches!(m.plane, Plane::Whole { band: None, settled: true, .. }));

    // A walk whose passes stop (its end never seen) is over after WALK_STALE_S, and the panel idles.
    let mut m = Model::default();
    m.update(&[ev(1, Kind::WalkPass, 0, 50_000, 400.0)], still(1, Some(Pass::Walk)));
    assert_eq!(m.walk.as_ref().unwrap().kind, WalkKind::Settle, "a pass with no start seen walks as the frame ran");
    let frames = (WALK_STALE_S * 60.0) as u64;
    let quiet = |f| Snapshot { present: Present::None, ..still(f, None) };
    m.update(&[ev(2, Kind::PassTimed, PASS_OF_FRAME, 1, 300.0)], quiet(2));
    for f in 3..frames {
        m.update(&[], quiet(f));
        assert!(!m.walk.as_ref().unwrap().ended, "frame {f}");
    }
    for f in frames..frames + 120 {
        m.update(&[], quiet(f));
    }
    assert!(m.walk.is_none());
    assert!(!m.active(), "nothing left moving: the panel can idle");
}

#[test]
fn a_pin_walks_passes_keep_the_pin_target_and_its_hold_frames_still_read_held() {
    let mut m = Model::default();
    m.update(&[ev(1, Kind::PinStart, 30_000, 436 << 16 | 330, 0.0), ev(1, Kind::PinPass, 256, 512, 0.0)], snap(1));
    let w = m.walk.as_ref().unwrap();
    assert_eq!((w.kind, w.passes[0].target_ms), (WalkKind::Pin, PIN_TARGET_MS));
    let mut s = snap(2);
    s.present = Present::Held;
    m.update(&[], s);
    assert_eq!(m.plane, Plane::Idle("HELD"));
}
