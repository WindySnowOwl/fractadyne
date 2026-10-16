use super::*;
use fractadyne_core::Viewport;

const LN_2: f64 = std::f64::consts::LN_2;

/// Integrate the app's OWN glide (`ui/central.rs`): per frame, ease the velocity toward
/// `target_v` with time constant `EASE_TAU`, then `zoom_at(anchor, e^(−vel·dt))`.
fn forward_glide(vp0: &Viewport, ax: f64, ay: f64, v0: f64, target_v: f64, tau: f64) -> Viewport {
    let n = (tau / 0.016).ceil().max(1.0) as usize;
    let dt = tau / n as f64;
    let mut vp = vp0.clone();
    let mut v = v0;
    for _ in 0..n {
        let ease = 1.0 - (-dt / crate::EASE_TAU).exp();
        v += (target_v - v) * ease;
        vp.zoom_at(ax, ay, (-v * dt).exp());
    }
    vp
}

fn glide_for(vp: &Viewport, ax: f64, ay: f64, v0: f64, target_v: f64) -> Glide {
    Glide {
        l2: vp.log2_magnification(),
        center: (vp.center_x.clone(), vp.center_y.clone()),
        anchor: vp.pixel_to_complex(ax, ay),
        v0,
        target_v,
        horizon_l2: f64::INFINITY,
        precision: vp.precision,
        fractal: FractalKind::Mandelbrot,
    }
}

/// The closed form must land where the frame-by-frame glide lands: depth AND centre, for every
/// slider setting, from rest and in steady state. Steady state is exact (the per-frame factors
/// multiply to `e^(−v·τ)`); the 0.15 s easing transient is integrated right-Riemann by the app
/// (each frame applies the velocity it just eased TO), which the closed form undershoots by
/// ≈ 0.0075·|v₀ − T| nepers at 16 ms frames — 0.02 oct at the 4× slider, measured. Against
/// `PREFETCH_OCT` = 0.5 spacing and `PREFETCH_REACH_SLACK` = 0.14 that is noise; pinned at 0.05.
#[test]
fn glide_closed_form_matches_the_frame_by_frame_glide() {
    let (w, h) = (1280.0, 720.0);
    let mut vp = Viewport::new(w, h);
    vp.set_center_mag(
        fractadyne_core::BigFloat::from_f64(-0.743_643_887, 64),
        fractadyne_core::BigFloat::from_f64(0.131_825_904, 64),
        5.0e3,
    );
    // Cursor-anchored (off-centre) so the centre actually moves, plus the centred case.
    for &(ax, ay) in &[(w * 0.5, h * 0.5), (w * 0.8, h * 0.3)] {
        for &zr in &[0.25, 1.0, 4.0] {
            let rate = crate::ZOOM_RATE * zr;
            for &(v0, target_v, tol_oct, tol_px) in &[
                (rate, rate, 1e-9, 1e-3),  // key held, easing complete: exact
                (0.0, rate, 0.05, 0.03 * w), // key just pressed: transient
                (rate, 0.0, 0.05, 0.03 * w), // key released: glide-out
            ] {
                for &tau in &[0.25, 1.0, 3.0] {
                    let truth = forward_glide(&vp, ax, ay, v0, target_v, tau);
                    let traj = Trajectory::Glide(glide_for(&vp, ax, ay, v0, target_v));
                    let pred = traj.at(tau);
                    assert!(
                        (pred.l2 - truth.log2_magnification()).abs() < tol_oct,
                        "depth: zr {zr} v0 {v0:.3} T {target_v:.3} τ {tau}: predicted {:.6} vs {:.6}",
                        pred.l2,
                        truth.log2_magnification()
                    );
                    assert!((traj.l2_at(tau) - pred.l2).abs() < 1e-12);
                    // The predicted centre, seen through the true future view, sits at its middle.
                    let (px, py) = truth.complex_to_pixel(&pred.cx, &pred.cy);
                    let err = ((px - w * 0.5).powi(2) + (py - h * 0.5).powi(2)).sqrt();
                    assert!(
                        err < tol_px,
                        "centre: zr {zr} v0 {v0:.3} T {target_v:.3} τ {tau} anchor ({ax},{ay}): \
                         {err:.4} px off"
                    );
                }
            }
        }
    }
}

/// A released key coasts `v₀·τₑ` nepers and no further — the prediction never runs away.
#[test]
fn glide_out_saturates() {
    let g = Glide {
        l2: 40.0,
        center: (fractadyne_core::BigFloat::from_f64(0.0, 64), fractadyne_core::BigFloat::from_f64(0.0, 64)),
        anchor: (fractadyne_core::BigFloat::from_f64(0.0, 64), fractadyne_core::BigFloat::from_f64(0.0, 64)),
        v0: crate::ZOOM_RATE,
        target_v: 0.0,
        horizon_l2: f64::INFINITY,
        precision: 64,
        fractal: FractalKind::Mandelbrot,
    };
    let cap = crate::ZOOM_RATE * crate::EASE_TAU;
    assert!(g.nepers(100.0) <= cap + 1e-12);
    assert!(g.nepers(100.0) > 0.999 * cap);
    assert!((g.oct_per_s() - crate::ZOOM_RATE / LN_2).abs() < 1e-12);
}

/// The queue length the slider buys: floor at the tour's `PREFETCH_SLOTS`, cap at
/// `PREFETCH_SLOTS_MAX`, `PREFETCH_RUNWAY_S` of zoom in between.
#[test]
fn queue_length_follows_the_zoom_rate() {
    let oct = |zr: f64| crate::ZOOM_RATE * zr / LN_2;
    assert_eq!(prefetch_slots_for(oct(0.25)), PREFETCH_SLOTS);
    assert_eq!(prefetch_slots_for(oct(1.0)), PREFETCH_SLOTS);
    assert_eq!(prefetch_slots_for(oct(2.0)), 8);
    assert_eq!(prefetch_slots_for(oct(4.0)), PREFETCH_SLOTS_MAX);
    assert_eq!(prefetch_slots_for(0.0), PREFETCH_SLOTS);
    assert_eq!(prefetch_slots_for(-1.0), PREFETCH_SLOTS);
}

/// The tour variant reads the script exactly as `playback_ref_prefetch` always did: `tau == 0`
/// is the unclamped sample at `e`, everything else clamps at `total`.
#[test]
fn playback_trajectory_is_the_script_sample() {
    const TOUR: &str = "format_version = 2\n\
        [[keyframe]]\nt = 0\nre = \"-0.5\"\nim = \"0.0\"\nzoom = \"1e3\"\n\
        [[keyframe]]\nt = 10\nre = \"-0.75\"\nim = \"0.1\"\nzoom = \"1e9\"\n";
    let pb = crate::scripting::parse_tour_text(TOUR).expect("fixture tour parses");
    let e = 2.0;
    let traj = Trajectory::Playback { pb: &pb, e };
    assert_eq!(traj.horizon_l2(), f64::INFINITY);
    for tau in [0.0, 0.25, 1.0, 5.0, 64.0, 128.0] {
        let want = pb.sample(if tau == 0.0 { e } else { (e + tau).min(pb.total) });
        assert_eq!(traj.l2_at(tau), want.logmag / LN_2, "l2 at τ {tau}");
        if tau > 0.0 {
            let s = traj.at(tau);
            assert_eq!(s.l2, want.logmag / LN_2);
            assert_eq!(fractadyne_core::to_f64(&s.cx), fractadyne_core::to_f64(&want.cx));
            assert_eq!(fractadyne_core::to_f64(&s.cy), fractadyne_core::to_f64(&want.cy));
            assert!(!s.julia && !s.dual);
        }
    }
}
