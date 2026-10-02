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
    s.change(Change::Cells(vec![(1000, 1000, 1)]));
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
    s.change(Change::Cells(vec![(-5, -5, 1)]));
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
