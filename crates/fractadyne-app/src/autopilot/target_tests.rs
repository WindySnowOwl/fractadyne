//! The Misiurewicz target: the goal is the point's EXACT screen position every frame, so a dive
//! that zooms about an eased aim cannot drift off it — the property `goal_of_point` exists for.

use super::*;
use fractadyne_core::{BigFloat, Viewport};

const DT: f64 = 1.0 / 60.0;

#[test]
fn the_target_round_trips_through_its_session_string() {
    for t in [AutopilotTarget::Detail, AutopilotTarget::Misiurewicz] {
        assert_eq!(AutopilotTarget::parse(t.as_str()), Some(t));
    }
    assert_eq!(AutopilotTarget::parse(" Misi "), Some(AutopilotTarget::Misiurewicz));
    assert_eq!(AutopilotTarget::parse("nucleus"), None);
    assert_eq!(AutopilotTarget::default(), AutopilotTarget::Detail);
}

#[test]
fn the_goal_of_a_point_is_where_the_viewport_puts_it() {
    let mut vp = Viewport::new(1456.0, 1102.0);
    vp.reset_to(-0.5, 0.0);
    // The centre maps to the middle of the screen…
    let (cx, cy) = (vp.center_x.clone(), vp.center_y.clone());
    let g = goal_of_point(&vp, &cx, &cy);
    assert!((g.0 - 0.5).abs() < 1e-12 && (g.1 - 0.5).abs() < 1e-12, "{g:?}");
    // …and a point one quarter of the width to the right, a quarter down (screen y is down).
    let (px, py) = vp.pixel_to_complex(1456.0 * 0.75, 1102.0 * 0.75);
    let g = goal_of_point(&vp, &px, &py);
    assert!((g.0 - 0.75).abs() < 1e-9 && (g.1 - 0.75).abs() < 1e-9, "{g:?}");
}

/// One simulated frame of a Misiurewicz dive, exactly as `autopilot_step` + `autopilot_glide`
/// apply it: the goal is re-projected from the fixed coordinate, the camera eases and zooms about
/// the aim, then pans; the aim and lead are carried through both moves.
struct Cam {
    vp: Viewport,
    tx: BigFloat,
    ty: BigFloat,
    aim: (f64, f64),
    lead: (f64, f64),
    speed: f64,
}

impl Cam {
    fn frame(&mut self, rate: f64) -> (f64, f64) {
        let goal = goal_of_point(&self.vp, &self.tx, &self.ty);
        let aspect = self.vp.width_px / self.vp.height_px;
        let g = glide_step(self.aim, self.lead, goal, self.speed, rate, f64::INFINITY, aspect, DT);
        self.speed = g.speed;
        let (w, h) = (self.vp.width_px, self.vp.height_px);
        self.vp.zoom_at(g.aim.0 * w, g.aim.1 * h, g.factor);
        self.aim = after_zoom(g.aim, g.aim, g.factor);
        self.lead = after_zoom(g.lead, g.aim, g.factor);
        self.vp.pan_pixels(g.pan.0 * w, g.pan.1 * h);
        self.aim = (self.aim.0 + g.pan.0, self.aim.1 + g.pan.1);
        self.lead = (self.lead.0 + g.pan.0, self.lead.1 + g.pan.1);
        goal
    }
}

#[test]
fn a_misiurewicz_dive_stays_on_its_point_through_a_long_dive() {
    // The antenna tip c = −2 is the Misiurewicz point (2,1); start a screen-quarter away from it
    // at a shallow view and dive at the slider's fastest setting for 20 s (~45 octaves after the
    // turn's slow-down — well past where an f64 screen fraction carried through the zoom would
    // have lost the point).
    let mut vp = Viewport::new(1456.0, 1102.0);
    vp.reset_to(-2.0, 0.0);
    vp.zoom_at(1456.0 * 0.5, 1102.0 * 0.5, 1.0 / 8.0);
    let (tx, ty) = (vp.center_x.clone(), vp.center_y.clone());
    vp.pan_pixels(-1456.0 * 0.25, 1102.0 * 0.15);
    let start_l2 = vp.log2_magnification();
    let mut cam = Cam { vp, tx, ty, aim: (0.5, 0.5), lead: (0.5, 0.5), speed: 0.0 };
    let rate = crate::ZOOM_RATE * 4.0;
    let mut last_goal = (0.0, 0.0);
    for _ in 0..1200 {
        last_goal = cam.frame(rate);
        assert!(
            (0.0..=1.0).contains(&last_goal.0) && (0.0..=1.0).contains(&last_goal.1),
            "the point left the screen: {last_goal:?} at 2^{:.2}",
            cam.vp.log2_magnification()
        );
    }
    let descended = cam.vp.log2_magnification() - start_l2;
    assert!(descended > 40.0, "descended {descended:.1} octaves");
    // The aim converged onto the point: within a pixel…
    let dpx = ((last_goal.0 - cam.aim.0) * 1456.0).hypot((last_goal.1 - cam.aim.1) * 1102.0);
    assert!(dpx < 1.0, "aim is {dpx:.2} px off the point");
    // …and the point sits inside the centring dead zone, not sliding to the edge.
    let off = ((last_goal.0 - 0.5) * 1456.0).hypot((last_goal.1 - 0.5) * 1102.0) / 1102.0;
    assert!(off <= CENTER_DEAD + 0.01, "point is {off:.3} of the short side off centre");
    // And the precision followed the depth (the viewport refreshes it on every zoom).
    assert!(cam.vp.precision >= fractadyne_core::precision_for_octaves(descended as u64));
}
