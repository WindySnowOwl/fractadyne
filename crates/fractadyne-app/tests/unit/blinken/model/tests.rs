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
