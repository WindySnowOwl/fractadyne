//! Autopilot steering: the goal chooser on synthetic probes, and the bookkeeping that carries a
//! goal through the camera's motion.

use super::*;

const NONE: f32 = 1.0e30;

/// A probe where every cell is flat exterior (distance estimate `far` cells) until painted.
fn flat(w: usize, h: usize) -> Probe {
    Probe { w, h, iter: vec![100.0; w * h], de_log2: vec![4.0; w * h] }
}

/// Mark cell (i, j) as on the boundary (distance estimate a quarter cell).
fn edge(p: &mut Probe, i: usize, j: usize) {
    p.de_log2[j * p.w + i] = -2.0;
}

/// A compact knot of boundary cells (a spiral centre / branch point, as the probe sees one).
fn knot(p: &mut Probe, ci: usize, cj: usize, r: usize) {
    for j in cj - r..=cj + r {
        for i in ci - r..=ci + r {
            edge(p, i, j);
        }
    }
}

fn cell(p: &Probe, f: (f64, f64)) -> (usize, usize) {
    ((f.0 * p.w as f64) as usize, (f.1 * p.h as f64) as usize)
}

#[test]
fn a_view_with_no_boundary_is_a_dead_end() {
    let p = flat(40, 25);
    assert_eq!(choose_goal(&p, 1.6, None, true), None);
    let mut all_inside = flat(40, 25);
    all_inside.iter.iter_mut().for_each(|s| *s = -1.0);
    assert_eq!(choose_goal(&all_inside, 1.6, None, true), None);
}

#[test]
fn the_goal_is_always_on_the_boundary_never_in_the_flat_between() {
    // Two equal knots either side of the centre: the old argmax-then-average steering converged
    // on the flat gap between them. Whatever is chosen must itself be a boundary cell.
    let mut p = flat(40, 25);
    knot(&mut p, 12, 12, 2);
    knot(&mut p, 27, 12, 2);
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    let (i, j) = cell(&p, s.goal);
    assert!(p.de_log2[j * p.w + i] < 0.0, "goal {:?} is not on a boundary cell", s.goal);
}

#[test]
fn the_first_pick_prefers_detail_near_the_centre() {
    let mut p = flat(40, 25);
    knot(&mut p, 5, 5, 2); // corner
    knot(&mut p, 21, 13, 2); // centre
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    let (i, j) = cell(&p, s.goal);
    assert!((19..=23).contains(&i) && (11..=15).contains(&j), "picked cell ({i},{j})");
}

#[test]
fn richer_detail_off_centre_still_wins_over_a_bare_line_at_the_centre() {
    let mut p = flat(40, 25);
    for i in 0..40 {
        edge(&mut p, i, 12); // a single filament through the middle
    }
    knot(&mut p, 28, 8, 3); // a dense knot a little off centre
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    let (i, j) = cell(&p, s.goal);
    assert!((25..=31).contains(&i) && (5..=11).contains(&j), "picked cell ({i},{j})");
}

#[test]
fn a_tracked_goal_on_the_boundary_is_kept_exactly() {
    // Held to its sub-cell position: re-quantising to a cell centre every evaluation is the wobble
    // this is meant to remove.
    let mut p = flat(40, 25);
    knot(&mut p, 20, 12, 3);
    let g = (20.3 / 40.0, 12.7 / 25.0);
    let s = choose_goal(&p, 1.6, Some(g), true).unwrap();
    assert_eq!(s, Steer { goal: g, retarget: false });
}

#[test]
fn a_goal_that_drifted_off_the_boundary_snaps_back_nearby_not_across_the_screen() {
    let mut p = flat(40, 25);
    for j in 0..25 {
        edge(&mut p, 20, j); // a filament
    }
    knot(&mut p, 34, 6, 2); // other detail far away
    let g = (21.5 / 40.0, 12.5 / 25.0); // one cell off the filament
    let s = choose_goal(&p, 1.6, Some(g), true).unwrap();
    assert!(!s.retarget);
    let (i, j) = cell(&p, s.goal);
    assert_eq!(i, 20);
    assert!((10..=14).contains(&j));
}

#[test]
fn a_somewhat_better_region_elsewhere_does_not_steal_the_goal() {
    let mut p = flat(40, 25);
    knot(&mut p, 20, 12, 1); // the current goal's neighbourhood
    knot(&mut p, 26, 12, 2); // a richer one nearby, but not RETARGET_GAIN richer after centring
    let g = (20.5 / 40.0, 12.5 / 25.0);
    let s = choose_goal(&p, 1.6, Some(g), true).unwrap();
    assert!(!s.retarget, "{s:?}");
}

#[test]
fn a_goal_that_went_flat_retargets_even_inside_the_cooldown() {
    let mut p = flat(40, 25);
    knot(&mut p, 30, 8, 2);
    let g = (10.5 / 40.0, 12.5 / 25.0); // nothing within reach
    let s = choose_goal(&p, 1.6, Some(g), false).unwrap();
    assert!(s.retarget);
    let (i, j) = cell(&p, s.goal);
    assert!((28..=32).contains(&i) && (6..=10).contains(&j));
}

#[test]
fn the_cooldown_holds_a_viable_goal_against_a_much_richer_one() {
    let mut p = flat(40, 25);
    for i in 0..40 {
        edge(&mut p, i, 12);
    }
    knot(&mut p, 24, 10, 3);
    let g = (5.5 / 40.0, 12.5 / 25.0); // on the line, far from the knot
    assert!(choose_goal(&p, 1.6, Some(g), true).unwrap().retarget);
    assert!(!choose_goal(&p, 1.6, Some(g), false).unwrap().retarget);
}

#[test]
fn cells_next_to_the_interior_are_boundary_even_without_an_estimate() {
    let mut p = flat(20, 20);
    p.de_log2.iter_mut().for_each(|d| *d = NONE);
    for j in 0..20 {
        for i in 12..20 {
            p.iter[j * 20 + i] = -1.0;
        }
    }
    let wt = p.boundary_weights();
    assert_eq!(wt[10 * 20 + 11], 1.0);
    assert_eq!(wt[10 * 20 + 5], 0.0);
    assert_eq!(wt[10 * 20 + 12], 0.0, "interior cells are never a goal");
}

#[test]
fn after_zoom_tracks_the_content_under_a_point() {
    use fractadyne_core::Viewport;
    let mut vp = Viewport::new(1600.0, 1000.0);
    let (w, h) = (vp.width_px, vp.height_px);
    let goal = (0.71, 0.33);
    let (gx, gy) = vp.pixel_to_complex(goal.0 * w, goal.1 * h);
    let pivot = (0.42, 0.58);
    for _ in 0..40 {
        vp.zoom_at(pivot.0 * w, pivot.1 * h, 0.97);
    }
    let tracked = (0..40).fold(goal, |g, _| after_zoom(g, pivot, 0.97));
    let (px, py) = vp.complex_to_pixel(&gx, &gy);
    assert!((px / w - tracked.0).abs() < 1e-9 && (py / h - tracked.1).abs() < 1e-9);
}

/// One simulated frame of the camera, exactly as `autopilot_glide` applies `glide_step`: zoom about
/// the new aim, then pan, carrying the lead and the goal (a point fixed in the fractal) along.
struct Cam {
    aim: (f64, f64),
    lead: (f64, f64),
    goal: (f64, f64),
    speed: f64,
}

const DT: f64 = 1.0 / 60.0;

impl Cam {
    fn new(goal: (f64, f64)) -> Self {
        Cam { aim: (0.5, 0.5), lead: (0.5, 0.5), goal, speed: 0.0 }
    }

    fn frame(&mut self, rate: f64) {
        let g = glide_step(self.aim, self.lead, self.goal, self.speed, rate, 1.6, DT);
        self.speed = g.speed;
        self.aim = g.aim;
        self.lead = after_zoom(g.lead, g.aim, g.factor);
        self.goal = after_zoom(self.goal, g.aim, g.factor);
        let shift = |p: (f64, f64)| (p.0 + g.pan.0, p.1 + g.pan.1);
        self.aim = shift(self.aim);
        self.lead = shift(self.lead);
        self.goal = shift(self.goal);
    }
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

#[test]
fn the_camera_closes_on_the_goal_and_brings_it_to_the_centre() {
    // The old pivot held its screen position forever: detail picked near the edge stayed there.
    let mut c = Cam::new((0.8, 0.3));
    for _ in 0..600 {
        c.frame(crate::ZOOM_RATE);
        assert!((0.0..1.0).contains(&c.goal.0) && (0.0..1.0).contains(&c.goal.1), "goal left the screen");
    }
    assert!(dist(c.aim, c.goal) < 1e-3, "aim {:?} goal {:?}", c.aim, c.goal);
    assert!(dist(c.goal, (0.5, 0.5)) < 0.01, "goal {:?} not centred", c.goal);
}

#[test]
fn the_approach_holds_at_the_fastest_zoom_setting() {
    // The zoom-rate slider goes to 4×; the approach is net of the zoom, so it still converges.
    let mut c = Cam::new((0.85, 0.2));
    for _ in 0..240 {
        c.frame(4.0 * crate::ZOOM_RATE);
        assert!((0.0..1.0).contains(&c.goal.0) && (0.0..1.0).contains(&c.goal.1), "goal left the screen");
    }
    assert!(dist(c.aim, c.goal) < 0.01, "aim {:?} goal {:?}", c.aim, c.goal);
}

#[test]
fn a_retarget_does_not_jolt_the_camera() {
    let mut c = Cam::new((0.55, 0.45));
    for _ in 0..300 {
        c.frame(crate::ZOOM_RATE);
    }
    let before = c.aim;
    c.goal = (0.2, 0.75); // the goal jumps across the screen
    // The aim's own frame-to-frame movement, net of the pan that moves everything together.
    let mut first = None;
    let mut worst: f64 = 0.0;
    let mut prev = before;
    for _ in 0..180 {
        c.frame(crate::ZOOM_RATE);
        let pan = 1.0 - (-DT / CENTER_TAU).exp();
        let step = dist(c.aim, prev) - pan * dist(prev, (0.5, 0.5));
        first.get_or_insert(step);
        worst = worst.max(step);
        prev = c.aim;
    }
    // A single ease would have moved the aim ~0.02 of the screen in the very first frame.
    assert!(first.unwrap() < 0.003, "first frame moved {:.4}", first.unwrap());
    assert!(worst < 0.015, "fastest frame moved {worst:.4}");
}

#[test]
fn the_zoom_speed_ramps_in_rather_than_starting_at_full_rate() {
    let mut c = Cam::new((0.5, 0.5));
    let rate = crate::ZOOM_RATE;
    let mut prev = 0.0;
    for n in 0..240 {
        c.frame(rate);
        assert!(c.speed - prev <= rate * DT / SPEED_TAU + 1e-12, "speed jumped at frame {n}");
        prev = c.speed;
    }
    assert!((c.speed - rate).abs() < 0.01 * rate);
}
