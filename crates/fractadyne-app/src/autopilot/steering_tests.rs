//! Autopilot steering: the goal chooser on synthetic probes, and the bookkeeping that carries a
//! goal through the camera's motion.

use super::*;

const NONE: f32 = 1.0e30;

/// Positions are compared in probe cells. The probe stores the distance estimate as an f32 log₂,
/// so an edge recovered from it is good to about 1e-8 of a cell; this is far below anything seen.
const CELL_TOL: f64 = 1e-5;

/// A probe where every cell is flat exterior (distance estimate 16 cells, no normal) until painted.
fn flat(w: usize, h: usize) -> Probe {
    Probe {
        w,
        h,
        iter: vec![100.0; w * h],
        de_log2: vec![4.0; w * h],
        normal: vec![(0.0, 0.0); w * h],
    }
}

/// A probe whose only structure is a straight edge, with the distance estimate and normal a real
/// probe reports on both sides of it: `vertical` = the line x = `at` (cells), else y = `at`.
/// Normals point away from the edge, in COMPLEX orientation (screen y flipped).
fn straight_edge(w: usize, h: usize, at: f64, vertical: bool) -> Probe {
    let mut p = flat(w, h);
    for j in 0..h {
        for i in 0..w {
            let c = if vertical { i as f64 + 0.5 } else { j as f64 + 0.5 };
            let d = (c - at).abs().max(1.0e-3);
            let s = (c - at).signum() as f32;
            p.de_log2[j * w + i] = d.log2() as f32;
            p.normal[j * w + i] = if vertical { (s, 0.0) } else { (0.0, -s) };
        }
    }
    p
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
    // The goal sits on a short bare segment (neighbourhood richness 3/49 ≈ 0.061); a small knot off
    // to the side scores ≈ 0.107 after centring — better, but short of RETARGET_GAIN (2×).
    let mut p = flat(40, 25);
    for i in 19..=21 {
        edge(&mut p, i, 12);
    }
    knot(&mut p, 27, 12, 1);
    let g = (20.5 / 40.0, 12.5 / 25.0);
    let s = choose_goal(&p, 1.6, Some(g), true).unwrap();
    assert_eq!(s, Steer { goal: g, retarget: false });
}

#[test]
fn a_goal_on_the_boundary_is_not_pulled_toward_richer_cells_beside_it() {
    // The neighbouring knot cell is 1.4× richer: the old refinement hill-climbed to it every look,
    // which at 4× zoom speed re-aimed the camera almost every evaluation.
    let mut p = flat(40, 25);
    for i in 0..40 {
        edge(&mut p, i, 12);
    }
    knot(&mut p, 23, 9, 2);
    let g = (20.4 / 40.0, 12.6 / 25.0);
    let s = choose_goal(&p, 1.6, Some(g), true).unwrap();
    assert_eq!(s, Steer { goal: g, retarget: false });
}

#[test]
fn a_goal_off_the_edge_moves_back_across_it_and_not_along_it() {
    // Vertical edge at x = 20.3 cells; the goal has slid 0.8 cells right of it.
    let p = straight_edge(40, 25, 20.3, true);
    let s = choose_goal(&p, 1.6, Some((21.1 / 40.0, 12.37 / 25.0)), true).unwrap();
    assert!(!s.retarget);
    assert!((s.goal.0 * 40.0 - 20.3).abs() < CELL_TOL, "x {}", s.goal.0 * 40.0);
    assert!((s.goal.1 * 25.0 - 12.37).abs() < CELL_TOL, "the place along the edge must not change");
    // A horizontal edge exercises the screen/complex y flip: the goal is 0.8 cells below y = 12.3.
    let p = straight_edge(40, 25, 12.3, false);
    let s = choose_goal(&p, 1.6, Some((20.37 / 40.0, 13.1 / 25.0)), true).unwrap();
    assert!((s.goal.1 * 25.0 - 12.3).abs() < CELL_TOL, "y {}", s.goal.1 * 25.0);
    assert!((s.goal.0 * 40.0 - 20.37).abs() < CELL_TOL, "x {}", s.goal.0 * 40.0);
}

#[test]
fn repeated_looks_keep_the_goal_on_its_piece_of_edge() {
    // What a fast zoom does between looks: the goal's sub-cell error grows and pushes it off the
    // edge, alternately to either side. Snapping to cell centres turned that into a walk; the
    // projection must bring it back to the same point every time.
    let p = straight_edge(40, 25, 20.3, true);
    let mut g = (20.3 / 40.0, 12.37 / 25.0);
    for n in 0..30 {
        let push = if n % 2 == 0 { 0.9 } else { -1.3 };
        g.0 += push / 40.0;
        g = choose_goal(&p, 1.6, Some(g), true).unwrap().goal;
    }
    assert!((g.0 * 40.0 - 20.3).abs() < CELL_TOL && (g.1 * 25.0 - 12.37).abs() < CELL_TOL, "{g:?}");
}

#[test]
fn a_new_goal_is_placed_on_the_edge_not_at_its_cell_centre() {
    let p = straight_edge(40, 25, 20.3, true);
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    assert!((s.goal.0 * 40.0 - 20.3).abs() < CELL_TOL, "x {}", s.goal.0 * 40.0);
}

/// Set one cell's distance estimate (log₂ cells) directly — how deep the dive could go about it.
fn at_depth(p: &mut Probe, i: usize, j: usize, de_log2: f32) {
    p.de_log2[j * p.w + i] = de_log2;
}

/// Boundary density over the neighbourhood of a cell — the quantity the FIRST rewrite maximised.
fn density_at(p: &Probe, i: usize, j: usize) -> f64 {
    box_mean(&p.boundary_weights(), p.w, p.h, DENSITY_R)[j * p.w + i]
}

#[test]
fn the_goal_takes_the_deep_edge_over_the_denser_eye_of_the_spiral() {
    // ⭐The reported failure, as a fixture. A ring of deep edge around a smooth middle. The middle
    // saturates the boundary WEIGHT at 1.0 — its estimate is half a cell, which on a deep view is
    // about eleven screen pixels — so it is a perfectly legal goal, it is dead centre, and it has
    // the higher boundary density, because the density of a ring peaks at the ring's centre.
    // Maximising density therefore aims at the eye, which is the flat part. But the eye is only two
    // octaves deep and the ring is nine: the dive must take the ring, where the octaves are.
    let mut p = flat(40, 25);
    for j in 0..25 {
        for i in 0..40 {
            let r = ((i as f64 - 20.0).hypot(j as f64 - 12.0)) as f32;
            if (5.0..=6.5).contains(&r) {
                at_depth(&mut p, i, j, -8.0); // the structure: 9 octaves of descent
            } else if r <= 3.0 {
                at_depth(&mut p, i, j, -1.0); // the eye: weight 1.0, but 2 octaves and it is spent
            }
        }
    }
    assert_eq!(p.boundary_weights()[12 * 40 + 20], 1.0, "fixture: the eye must saturate the weight");
    // The fixture discriminates only if the eye really is the denser, more central candidate.
    assert!(
        density_at(&p, 20, 12) > density_at(&p, 25, 12),
        "fixture: the eye must out-score the ring on density, or this test proves nothing"
    );
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    let (i, j) = cell(&p, s.goal);
    let r = (i as f64 - 20.0).hypot(j as f64 - 12.0);
    assert!((4.0..=7.5).contains(&r), "picked cell ({i},{j}), {r:.1} cells from the eye");
}

#[test]
fn an_incredible_distance_estimate_cannot_outbid_a_credible_one() {
    // Both cells are single specks with identical surroundings, so only depth and centring decide.
    // The near one reports the deepest estimate the shader is trusted for; the far one reports a
    // depth no estimate is trusted at (`DE_TRUST_LOG2`), of the kind a pixel that lost its
    // derivative produces. Floored, they tie on depth and the near one wins on centring; taken at
    // face value the far one would win by 3:1 and the dive would be steered by a broken pixel.
    let mut p = flat(40, 25);
    at_depth(&mut p, 21, 12, DE_TRUST_LOG2); // credible, near the centre
    at_depth(&mut p, 26, 12, -40.0); // 28 octaves past anything the shader can resolve
    assert!(off_centre(((26.5) / 40.0, 12.5 / 25.0), 1.6) <= TARGET_MAX_OFF, "fixture: both in bound");
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    let (i, _) = cell(&p, s.goal);
    assert_eq!(i, 21, "steered by the untrustworthy reading (picked column {i})");
}

#[test]
fn depth_is_unsaturated_through_the_range_the_dive_lives_in() {
    // The weight saturates at a half cell and cannot rank anything below it; the depth must.
    let mut p = flat(6, 1);
    for (i, de) in [2.0f32, 0.0, -1.0, -4.0, -12.0, -30.0].into_iter().enumerate() {
        at_depth(&mut p, i, 0, de);
    }
    let wt = p.boundary_weights();
    assert_eq!((wt[2], wt[3], wt[4]), (1.0, 1.0, 1.0), "the weight ties everything below a half cell");
    let d: Vec<f64> = (0..6).map(|k| p.depth(k, &wt)).collect();
    assert_eq!(d[0], 0.0, "two cells clear of the edge is no depth at all");
    assert!(d[1] < d[2] && d[2] < d[3] && d[3] < d[4], "depth must rank where the weight ties: {d:?}");
    assert_eq!(d[4], d[5], "past the trusted depth every reading counts the same");
    let mut interior = flat(1, 1);
    interior.iter[0] = -1.0;
    assert_eq!(interior.depth(0, &interior.boundary_weights()), 0.0, "interior is never a goal");
}

/// A knot of boundary cells all reporting the same distance estimate — same weight and density as
/// any other knot, so only DEPTH and position tell two of them apart.
fn knot_at(p: &mut Probe, ci: usize, cj: usize, r: usize, de_log2: f32) {
    for j in cj - r..=cj + r {
        for i in ci - r..=ci + r {
            at_depth(p, i, j, de_log2);
        }
    }
}

#[test]
fn an_unmeasurably_deep_reading_elsewhere_does_not_steal_a_healthy_goal() {
    // ⭐⭐The regression this rule exists for. Depth SATURATES at the cap wherever the estimate stops
    // being trustworthy, and `best` is the maximum over every cell in the frame — so a depth-weighted
    // `best > 2 x kept` sits pinned at the cap and fires on any dip in the goal's own single reading.
    // Two equally appealing knots, differing only in depth: the far one reports a depth no estimate
    // can resolve, the tracked one has ample runway. Nothing about the dive has got worse, so the
    // goal must be kept exactly.
    let mut p = flat(40, 25);
    knot_at(&mut p, 17, 12, 1, -4.0); // the tracked goal: 5 octaves, plenty
    knot_at(&mut p, 23, 12, 1, -40.0); // 28 octaves past anything the shader can resolve
    let wt = p.boundary_weights();
    assert_eq!(wt[12 * 40 + 17], wt[12 * 40 + 23], "fixture: equal weight, so equal density");
    assert!(
        p.depth(12 * 40 + 23, &wt) > 2.0 * p.depth(12 * 40 + 17, &wt),
        "fixture: the far reading must out-DEPTH the goal 2:1, or this test proves nothing"
    );
    let g = (17.5 / 40.0, 12.5 / 25.0);
    let s = choose_goal(&p, 1.6, Some(g), true).unwrap();
    assert!(!s.retarget, "a saturated reading elsewhere stole the goal: {s:?}");
    assert_eq!(s.goal, g);
}

#[test]
fn a_goal_with_no_runway_left_is_abandoned_even_if_nothing_is_more_appealing() {
    // The other half of the rule: depth decides a retarget as a THRESHOLD on the goal's own runway.
    // Here the goal is down to 2.5 octaves — the dive magnifies past that within a couple of looks —
    // while the alternative is deeper but LESS appealing, so the appeal ratio would never fire.
    let mut p = flat(40, 25);
    knot_at(&mut p, 20, 12, 1, -1.5); // the tracked goal: 2.5 octaves, nearly spent
    knot_at(&mut p, 26, 9, 1, -8.0); // 9 octaves, but further out
    let wt = p.boundary_weights();
    assert!(p.depth(12 * 40 + 20, &wt) < RETARGET_MIN_DEPTH, "fixture: the goal must be out of runway");
    let g = (20.5 / 40.0, 12.5 / 25.0);
    let s = choose_goal(&p, 1.6, Some(g), true).unwrap();
    assert!(s.retarget, "a spent goal was kept: {s:?}");
    let (i, j) = cell(&p, s.goal);
    assert!((25..=27).contains(&i) && (8..=10).contains(&j), "went to ({i},{j}), not the deep knot");
}

/// Distance from the screen centre in short-side units — the measure the target bound uses.
fn off_centre(g: (f64, f64), aspect: f64) -> f64 {
    ((g.0 - 0.5) * aspect).hypot(g.1 - 0.5) / aspect.min(1.0)
}

#[test]
fn a_new_target_stays_inside_the_bounded_region_around_the_centre() {
    // A dense knot near the edge and a thinner one near the middle: the edge knot would score
    // higher but is a long pan at depth, so the bounded one is taken.
    let mut p = flat(40, 25);
    knot(&mut p, 36, 3, 3); // corner, rich
    knot(&mut p, 22, 12, 1); // near the middle, thin
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    assert!(off_centre(s.goal, 1.6) <= TARGET_MAX_OFF, "picked {:?}", s.goal);
    let (i, j) = cell(&p, s.goal);
    assert!((21..=23).contains(&i) && (11..=13).contains(&j), "picked cell ({i},{j})");
}

#[test]
fn with_detail_only_outside_the_bound_the_closest_of_it_is_taken() {
    // The dive must not stop just because everything is off to one side; it heads for the NEAREST
    // structure (the shortest journey), not the richest corner.
    let mut p = flat(40, 25);
    knot(&mut p, 34, 4, 2); // far corner, richer
    knot(&mut p, 30, 12, 1); // still outside the bound, but closer to the centre
    let s = choose_goal(&p, 1.6, None, true).unwrap();
    let (i, j) = cell(&p, s.goal);
    assert!((29..=31).contains(&i) && (11..=13).contains(&j), "picked cell ({i},{j})");
}

#[test]
fn a_target_carried_past_the_bound_is_replaced_even_inside_the_cooldown() {
    let mut p = flat(40, 25);
    for j in 0..25 {
        edge(&mut p, 3, j); // structure where the old goal has drifted to
    }
    knot(&mut p, 21, 12, 2); // and structure near the middle
    let far = (3.5 / 40.0, 12.5 / 25.0);
    assert!(off_centre(far, 1.6) > TARGET_KEEP_OFF, "test setup: goal must be past the bound");
    let s = choose_goal(&p, 1.6, Some(far), false).unwrap();
    assert!(s.retarget);
    assert!(off_centre(s.goal, 1.6) <= TARGET_MAX_OFF, "replacement {:?} still out of bounds", s.goal);
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
    let g = (12.5 / 40.0, 12.5 / 25.0); // on the line, away from the knot but inside the bound
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
        let g = glide_step(self.aim, self.lead, self.goal, self.speed, rate, f64::INFINITY, 1.6, DT);
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
fn the_camera_closes_on_the_goal_and_brings_it_in_from_the_edge() {
    // The old pivot held its screen position forever: detail picked near the edge stayed there.
    // It is brought in to the dead zone, not to the exact centre — see `glide_step`.
    let mut c = Cam::new((0.8, 0.3));
    for _ in 0..600 {
        c.frame(crate::ZOOM_RATE);
        assert!((0.0..1.0).contains(&c.goal.0) && (0.0..1.0).contains(&c.goal.1), "goal left the screen");
    }
    assert!(dist(c.aim, c.goal) < 1e-3, "aim {:?} goal {:?}", c.aim, c.goal);
    assert!(dist(c.goal, (0.5, 0.5)) < CENTER_DEAD + 0.02, "goal {:?} not brought in", c.goal);
}

#[test]
fn a_target_near_the_middle_is_left_where_it_is() {
    // Inside the dead zone the dive is a pure zoom: no pan, so nothing slides sideways. This is
    // the "it started sliding down and to the right" report — centring faster made it worse.
    let g = glide_step((0.54, 0.47), (0.54, 0.47), (0.54, 0.47), crate::ZOOM_RATE, crate::ZOOM_RATE, f64::INFINITY, 1.354, DT);
    assert_eq!(g.pan, (0.0, 0.0), "a target 4% off centre must not be panned");
    // Far out, the excess is eased away.
    let g = glide_step((0.85, 0.5), (0.85, 0.5), (0.85, 0.5), crate::ZOOM_RATE, crate::ZOOM_RATE, f64::INFINITY, 1.354, DT);
    assert!(g.pan.0 < 0.0, "a target near the edge must be brought in, got {:?}", g.pan);
}

#[test]
fn the_picture_does_not_slide_before_the_zoom_has_started() {
    // ⭐The reported "it starts to pan quickly on zoom". The zoom speed ramps in over `SPEED_TAU`,
    // so on the first frames of a dive there is barely any zoom — and a pan with no zoom under it
    // is a pure sideways slide, which is the most visible thing a camera can do. Centring is
    // scaled by the speed that is actually happening, so it arrives WITH the zoom.
    let rate = 4.0 * crate::ZOOM_RATE;
    let far = (0.85, 0.5); // well outside the dead zone, so centring is active
    let start = glide_step(far, far, far, 0.0, rate, f64::INFINITY, 1.6, DT); // frame one of the dive
    let running = glide_step(far, far, far, rate, rate, f64::INFINITY, 1.6, DT); // the same camera, up to speed
    let pan = |g: &GlideStep| g.pan.0.hypot(g.pan.1);
    assert!(pan(&running) > 0.0, "test premise: a target this far out is centred once running");
    assert!(
        pan(&start) < 0.1 * pan(&running),
        "the dive slides before it zooms: {:.5} of the screen on frame one against {:.5} running",
        pan(&start),
        pan(&running)
    );
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
fn the_centring_keeps_up_with_the_zoom_speed() {
    // Centring is measured in zoom, not in seconds, so a 4× dive brings a far target in about four
    // times as fast: three seconds of 4× zoom is six centring constants and lands in the dead zone.
    let mut fast = Cam::new((0.8, 0.3));
    for _ in 0..180 {
        fast.frame(4.0 * crate::ZOOM_RATE);
    }
    assert!(dist(fast.goal, (0.5, 0.5)) < CENTER_DEAD + 0.02, "4× goal {:?}", fast.goal);
    // The same three seconds at 1× is only 1.5 constants — still on its way in. (If this ever
    // arrives as fast as the 4× case, the scaling with zoom rate has been lost.)
    let mut slow = Cam::new((0.8, 0.3));
    for _ in 0..180 {
        slow.frame(crate::ZOOM_RATE);
    }
    assert!(dist(slow.goal, (0.5, 0.5)) > CENTER_DEAD + 0.05, "1× goal {:?}", slow.goal);
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

#[test]
fn the_first_half_second_of_a_dive_barely_slides() {
    // The user-visible quantity behind "it starts to pan quickly on zoom": how far the picture
    // travels SIDEWAYS while the zoom speed is still ramping in. The frame-one check above is the
    // mechanism; this is the amount a person actually sees. Scaling centring by the commanded rate
    // instead of the real speed put 2.84% of the screen of pure slide into that window.
    let rate = 4.0 * crate::ZOOM_RATE;
    let mut c = Cam::new((0.80, 0.30)); // a target well outside the dead zone, so centring is on
    let mut slide = 0.0f64;
    for _ in 0..30 {
        // 0.5 s at 60 fps
        slide += glide_step(c.aim, c.lead, c.goal, c.speed, rate, f64::INFINITY, 1.6, DT).pan.0.abs();
        c.frame(rate);
    }
    assert!(slide < 0.015, "the dive slid {:.2}% of the screen before it got going", slide * 100.0);
}

#[test]
fn a_dead_end_caused_by_the_iteration_cap_says_so() {
    // The 2026-09-20 report: a dive at 2^1908 with iterations fixed at 10,000 "lost detail" and
    // stopped. The escape range read [9998, 9998] — every pixel that escaped did so at the cap —
    // and the probe, seeing nothing but capped pixels, reported a dead end. "No detail ahead" is
    // the wrong conclusion for a user to be handed there; the fractal had plenty, the count did not.
    let l2 = 1908.0;
    let starved = dead_end_message(l2, 10_000, false, Some(9998.0));
    assert!(starved.contains("10,000") || starved.contains("10000"), "{starved}");
    assert!(starved.to_lowercase().contains("iteration"), "{starved}");
    assert!(starved.to_lowercase().contains("auto"), "must point at the fix: {starved}");
    // The same range reading with auto-iterations ON is not this failure.
    let auto = dead_end_message(l2, 10_000, true, Some(9998.0));
    assert!(!auto.to_lowercase().contains("fixed"), "{auto}");
    // A shallow dead end with a comfortable range is the plain message.
    let plain = dead_end_message(20.0, 10_000, false, Some(600.0));
    assert!(!plain.to_lowercase().contains("iteration"), "{plain}");
    assert!(plain.contains("stopped"), "{plain}");
    // The range alone is enough evidence, even where the depth heuristic would not have warned.
    let capped = dead_end_message(20.0, 10_000, false, Some(9995.0));
    assert!(capped.to_lowercase().contains("iteration"), "{capped}");
}
