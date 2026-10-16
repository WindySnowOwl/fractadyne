use super::*;

/// The status bar's Life readouts keep one width whatever they show (the reflow rule).
#[test]
fn the_status_readouts_never_change_width() {
    let width = |t: (String, String, String)| (t.0.chars().count(), t.1.chars().count(), t.2.chars().count());
    let w0 = width(status_readouts(1.0, 0, 0));
    for upp in [1e-300, 1e-9, 0.001, 0.25, 0.5, 1.0, 3.7, 64.0, 4096.0, 1e6, 1e18, 9.99e300] {
        for n in [0u64, 9, 1_000, 123_456_789, u64::MAX] {
            assert_eq!(width(status_readouts(upp, n, n)), w0, "upp {upp}, n {n}");
        }
    }
    assert_eq!(status_readouts(4.0, 1234, 5).0.trim_start_matches("scale").trim(), "1 px = 4 cells");
    assert_eq!(status_readouts(0.125, 0, 0).0.trim_start_matches("scale").trim(), "8 px / cell");
    assert!(status_readouts(1.0, 1234, 0).1.ends_with("1,234"));
}

#[test]
fn a_stroke_draws_every_cell_between_its_points() {
    assert_eq!(line((0, 0), (0, 0)), [(0, 0)]);
    assert_eq!(line((0, 0), (3, 0)), [(0, 0), (1, 0), (2, 0), (3, 0)]);
    assert_eq!(line((2, 2), (-1, -1)), [(2, 2), (1, 1), (0, 0), (-1, -1)]);
    let l = line((0, 0), (5, 2));
    assert_eq!((l.first(), l.last(), l.len()), (Some(&(0, 0)), Some(&(5, 2)), 6));
    // Each step moves one cell, so a fast stroke leaves no gaps.
    assert!(l.windows(2).all(|w| (w[0].0 - w[1].0).abs() <= 1 && (w[0].1 - w[1].1).abs() <= 1));
}

#[test]
fn a_new_rule_keeps_the_cells_it_can_hold() {
    let mut u = Universe::new(Rule::parse("B2/S345/C4").unwrap(), Topology::Plane).unwrap();
    u.set(0, 0, 1);
    u.set(1, 0, 3);
    u.set_generation(7);
    let v = rebuild(&u, Rule::life());
    assert_eq!(v.cells(), [(0, 0, 1)], "state 3 does not exist in a binary rule");
    assert_eq!(v.generation(), 7);
    // A live background survives into another binary rule.
    let mut a = Universe::new(Rule::parse("B0123478/S01234678").unwrap(), Topology::Plane).unwrap();
    a.set_background(1);
    a.set(4, 4, 0);
    let b = rebuild(&a, Rule::parse("B0123478/S012345678").unwrap());
    assert_eq!((b.background(), b.cells()), (1, vec![(4, 4, 0)]));
}

/// A change lands at once while the GPU holds what was loaded; once it has stepped on, the change
/// waits for a download and is made to that — never to the stale copy.
#[test]
fn a_change_waits_for_a_current_copy() {
    let mut s = LifeState::default();
    let start_id = s.load_id;
    // The GPU has applied the load and not stepped: in sync.
    {
        let mut st = s.status.lock().unwrap();
        st.load_id = s.load_id;
        st.generation = s.loaded.generation();
    }
    s.change(Change::Stroke { cells: vec![(1000, 1000)], start: true });
    assert_eq!(s.load_id, start_id + 1);
    assert_eq!(s.loaded.get(1000, 1000), 1);
    assert!(!s.want_download);
    assert_eq!(s.start.get(1000, 1000), 1, "an edit at generation 0 is part of the start");

    // The GPU applies that load and runs 50 generations: out of sync.
    {
        let mut st = s.status.lock().unwrap();
        st.load_id = s.load_id;
        st.generation = 50;
    }
    s.playing = true;
    s.change(Change::Stroke { cells: vec![(-5, -5)], start: true });
    assert!(s.want_download && !s.playing, "pause and ask for the universe");
    assert_eq!(s.loaded.get(-5, -5), 0, "not made to the stale copy");
    // The download arrives (generation 50, the gun's own cells): the change is made to it.
    let mut ran = (*s.loaded).clone();
    ran.step_n(50);
    s.status.lock().unwrap().downloaded = Some(ran.clone());
    let got = s.take_download().expect("the download");
    s.apply_pending(Some(got));
    assert_eq!(s.loaded.generation(), 50);
    assert_eq!(s.loaded.get(-5, -5), 1);
    assert_eq!(s.loaded.population(), ran.population() + 1);
    assert_eq!(s.start.get(-5, -5), 0, "an edit after generation 0 leaves the start alone");
    assert!(!s.want_download);
}

/// A state whose GPU holds what was loaded, unstepped (so changes land at once).
fn synced() -> LifeState {
    let s = LifeState::default();
    {
        let mut st = s.status.lock().unwrap();
        st.load_id = s.load_id;
        st.generation = s.loaded.generation();
    }
    s
}

fn sync(s: &LifeState) {
    let mut st = s.status.lock().unwrap();
    st.load_id = s.load_id;
    st.generation = s.loaded.generation();
}

/// One tool draws and erases: a stroke starting on a dead cell paints its cells alive, one starting
/// on a live cell clears them — whatever those other cells were.
#[test]
fn a_stroke_paints_the_opposite_of_its_first_cell() {
    let mut s = synced();
    s.stroke((-10, -10), true, false, 1.0);
    s.stroke((-7, -10), false, false, 1.1);
    s.stroke_end();
    sync(&s);
    assert_eq!((-10..=-7).map(|x| s.loaded.get(x, -10)).collect::<Vec<_>>(), [1, 1, 1, 1], "a dead start draws a line");
    // Start on a live cell (-8, -10), drag over a dead one: both end dead.
    s.stroke((-8, -10), true, false, 2.0);
    s.stroke((-8, -12), false, false, 2.1);
    s.stroke_end();
    sync(&s);
    assert_eq!([s.loaded.get(-8, -10), s.loaded.get(-8, -11), s.loaded.get(-8, -12)], [0, 0, 0]);
    assert_eq!([s.loaded.get(-10, -10), s.loaded.get(-9, -10), s.loaded.get(-7, -10)], [1, 1, 1], "the rest of the line stays");
}

/// A click flips its cell ONCE: not when egui lays the press frame out twice, and not again when
/// the release arrives as a click (egui has forgotten the press time by then). The bug the
/// uitest's life-draw step found: the click on a live cell left it alive.
#[test]
fn a_click_flips_its_cell_once() {
    let mut s = synced();
    s.stroke((3, 3), true, false, 5.0); // the press
    sync(&s);
    s.stroke((3, 3), true, false, 5.0); // the same frame, laid out again
    sync(&s);
    s.stroke((3, 3), false, true, 5.2); // the release, as a click
    s.stroke_end();
    sync(&s);
    assert_eq!(s.loaded.get(3, 3), 1);
    // A new click on the same cell flips it back; a press and release in ONE frame does too.
    s.stroke((3, 3), true, false, 6.0);
    s.stroke((3, 3), false, true, 6.1);
    s.stroke_end();
    sync(&s);
    assert_eq!(s.loaded.get(3, 3), 0);
    s.stroke((3, 3), true, true, 7.0);
    s.stroke_end();
    assert_eq!(s.loaded.get(3, 3), 1);
}

/// A stroke pauses a running universe and lets it run on when it ends; one that started paused
/// leaves it paused.
#[test]
fn drawing_pauses_and_resumes_a_running_universe() {
    let mut s = synced();
    s.playing = true;
    s.stroke((0, 50), true, false, 1.0);
    assert!(!s.playing);
    s.stroke_end();
    assert!(s.playing);
    s.playing = false;
    s.stroke((1, 50), true, false, 2.0);
    s.stroke_end();
    assert!(!s.playing);
}

/// Undo returns to the universe before each edit, newest first — a stroke, a clear.
#[test]
fn undo_returns_to_before_each_edit() {
    let mut s = synced();
    let original = s.loaded.cells();
    s.stroke((100, 100), true, false, 1.0);
    s.stroke_end();
    sync(&s);
    let drawn = s.loaded.cells();
    s.change(Change::Clear);
    sync(&s);
    assert_eq!(s.loaded.population(), 0);
    assert!(s.undo_edit());
    assert_eq!(s.loaded.cells(), drawn, "undo the clear");
    sync(&s);
    assert!(s.undo_edit());
    assert_eq!(s.loaded.cells(), original, "undo the stroke");
    assert!(!s.undo_edit() && !s.can_undo());
}
