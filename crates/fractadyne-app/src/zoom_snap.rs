//! SNAP ZOOM: while zooming in with the wheel or Space, pull the zoom toward a feature near the
//! cursor — a minibrot's nucleus, or the real axis — so the feature slides under the cursor and
//! the zoom lands on it instead of beside it. The Navigate panel's "Snap zoom to features"
//! (`RenderConfig::snap_zoom`, on by default). Mandelbrot only, single view.
//!
//! The behaviour follows Imagina's (studied, not copied — Imagina is AGPL): a feature within 1/24
//! of the view's height of the cursor attracts the zoom, more strongly the nearer it is, and each
//! octave zoomed in shrinks its distance from the cursor by up to half. The finder is our own:
//! `fractadyne_core::feature::find_nucleus_near`, run off the UI thread against the view's
//! reference orbit whenever the cursor or the view has moved since the last search.

use crate::*;
use fractadyne_core::feature::{find_nucleus_near, NucleusNear};
use fractadyne_core::{BigFloat, CFloatExp, FloatExp};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

/// The attraction (and search) radius, as a fraction of the view's height.
pub(crate) const SNAP_RADIUS_FRAC: f64 = 1.0 / 24.0;
/// The most an octave of zooming in shrinks a feature's distance from the cursor: by half, when
/// the feature is right under it, tapering to nothing at the radius.
pub(crate) const SNAP_PULL: f64 = 0.5;
/// A new search once the cursor has moved this many pixels, or the view this many octaves, since
/// the last one.
const RESEARCH_PX: f64 = 3.0;
const RESEARCH_OCT: f64 = 0.25;
/// Periods searched at most, whatever the iteration count (a bound on a search's cost).
const MAX_PERIOD_CAP: u32 = 1 << 22;

/// Where to anchor a zoom by `factor` (< 1 zooms in) so that a feature at `feature`, within
/// `radius_px` of the cursor at `cursor`, comes nearer it: its distance shrinks by
/// `(1 − SNAP_PULL·√a)` per octave, `a = 1 − d/radius` its attraction. Zooming about an anchor
/// `A = cursor + k(feature − cursor)` multiplies that distance by `m − k(m − 1)` for a
/// magnification `m = 1/factor`, so `k` is solved for. A zoom out, or a feature outside the
/// radius, keeps the cursor.
pub(crate) fn snap_anchor(cursor: [f64; 2], feature: [f64; 2], radius_px: f64, factor: f64) -> [f64; 2] {
    let (dx, dy) = (feature[0] - cursor[0], feature[1] - cursor[1]);
    let d = dx.hypot(dy);
    if !(factor > 0.0 && factor < 1.0) || !(radius_px > 0.0) || !(d < radius_px) || !d.is_finite() {
        return cursor;
    }
    let a = 1.0 - d / radius_px;
    let m = 1.0 / factor;
    let g = (1.0 - SNAP_PULL * a.sqrt()).powf(m.log2());
    let k = (m - g) / (m - 1.0);
    [cursor[0] + k * dx, cursor[1] + k * dy]
}

/// What a search was for: the cursor (view pixels), the depth, the reference.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SnapKey {
    px: [f64; 2],
    l2: f64,
    orbit_id: u64,
}

impl SnapKey {
    fn near(&self, o: &SnapKey) -> bool {
        self.orbit_id == o.orbit_id
            && (self.px[0] - o.px[0]).hypot(self.px[1] - o.px[1]) < RESEARCH_PX
            && (self.l2 - o.l2).abs() < RESEARCH_OCT
    }
}

/// The feature a zoom in is pulled toward.
#[derive(Clone, Debug)]
pub(crate) enum SnapTarget {
    /// A minibrot nucleus, at full precision, and its period.
    Nucleus { cx: BigFloat, cy: BigFloat, period: u32 },
    /// The real axis (the point on it below or above the cursor).
    Axis,
}

struct SnapJob {
    rx: mpsc::Receiver<Option<NucleusNear>>,
    cancel: Arc<AtomicBool>,
    key: SnapKey,
    /// The reference the search's `dc` is relative to (`None`: `dc` is `c` itself).
    ref_pt: Option<[BigFloat; 2]>,
    precision: usize,
    /// The cursor was within the radius of the real axis.
    near_axis: bool,
}

/// The snap's state for the single view (`FractadyneApp::zoom_snap`).
#[derive(Default)]
pub(crate) struct ZoomSnap {
    job: Option<SnapJob>,
    last: Option<SnapKey>,
    pub(crate) target: Option<SnapTarget>,
}

impl FractadyneApp {
    /// Whether zooms snap at all: the option, a Mandelbrot single view, and a person at the
    /// controls (a harness's driven zoom keeps its scripted path).
    pub(crate) fn snap_active(&self) -> bool {
        let person = !launched_as_task() || self.harness.zoomtest.as_ref().is_some_and(|z| z.snap);
        self.render_cfg.snap_zoom && person && self.fractal == FractalKind::Mandelbrot && !self.dual
    }

    /// Once a frame, before the zoom is applied: take a finished search's answer, and start a new
    /// search when the cursor (view pixels, `None` off the canvas) or the view has moved since the
    /// last one.
    pub(crate) fn snap_update(&mut self, cursor: Option<[f64; 2]>) {
        if let Some(job) = self.zoom_snap.job.as_ref() {
            match job.rx.try_recv() {
                Ok(found) => {
                    let job = self.zoom_snap.job.take().expect("a job was polled");
                    self.zoom_snap.last = Some(job.key);
                    self.zoom_snap.target = match found {
                        Some(n) => {
                            let p = job.precision;
                            let (rx, ry) = (n.dc.re.to_bf(p), n.dc.im.to_bf(p));
                            let (cx, cy) = match &job.ref_pt {
                                Some(r) => (fractadyne_core::bf_add(&r[0], &rx, p), fractadyne_core::bf_add(&r[1], &ry, p)),
                                None => (rx, ry),
                            };
                            Some(SnapTarget::Nucleus { cx, cy, period: n.period })
                        }
                        None if job.near_axis => Some(SnapTarget::Axis),
                        None => None,
                    };
                    if crate::diag::trace_on("snap") {
                        let what = match &self.zoom_snap.target {
                            Some(SnapTarget::Nucleus { cx, cy, period }) => {
                                let (x, y) = self.viewport.complex_to_pixel(cx, cy);
                                format!("a period-{period} nucleus at ({x:.1}, {y:.1}) px")
                            }
                            Some(SnapTarget::Axis) => "the real axis".to_string(),
                            None => "nothing".to_string(),
                        };
                        crate::diag::trace(
                            "snap",
                            format!("search at ({:.1}, {:.1}) px, 2^{:.2}: {what}", job.key.px[0], job.key.px[1], job.key.l2),
                        );
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => self.zoom_snap.job = None,
            }
        }
        let Some(px) = cursor.filter(|_| self.snap_active()) else {
            return;
        };
        let key = SnapKey { px, l2: self.viewport.log2_magnification(), orbit_id: self.ref_cache[0].orbit_id };
        if let Some(job) = self.zoom_snap.job.as_ref() {
            if job.key.near(&key) {
                return; // the search under way is still the right one
            }
            job.cancel.store(true, Ordering::Relaxed);
            self.zoom_snap.job = None;
        }
        if self.zoom_snap.last.is_some_and(|l| l.near(&key)) {
            return;
        }
        self.snap_search(key);
    }

    /// Start a search about the cursor in `key`, on its own thread.
    fn snap_search(&mut self, key: SnapKey) {
        let vp = &self.viewport;
        let p = vp.precision;
        let upp = vp.units_per_pixel;
        let radius = upp.mul_f64(vp.height_px * SNAP_RADIUS_FRAC);
        let (cx, cy) = vp.pixel_to_complex(key.px[0], key.px[1]);
        // `a − b` as an extended-range number, through a mantissa scaled at the view's own
        // exponent (exact at any depth; see `ref_offset_mantissa`).
        let diff = |a: &BigFloat, b: &BigFloat| FloatExp::new(fractadyne_core::ref_offset_mantissa(a, b, upp.e, p), upp.e);
        let zero = BigFloat::from_f64(0.0, p);
        let near_axis = diff(&cy, &zero).abs().lt(radius);
        let direct = self.render_mode(self.fractal, false, vp.magnification()).is_direct();
        let rc = &self.ref_cache[0];
        let (orbit, len, ref_pt, dc0) = match (&rc.ref_pt, direct) {
            (_, true) => (Arc::new(Vec::new()), 0, None, CFloatExp { re: diff(&cx, &zero), im: diff(&cy, &zero) }),
            (Some(r), false) => (rc.orbit.clone(), rc.orbit_len, Some(r.clone()), CFloatExp { re: diff(&cx, &r[0]), im: diff(&cy, &r[1]) }),
            (None, false) => return, // no reference yet: search once it lands
        };
        let max_period = self.perf.last_eff_iter.clamp(1000, MAX_PERIOD_CAP);
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let flag = Arc::clone(&cancel);
        render::spawn_named("fd-snap", move || {
            let stop = || flag.load(Ordering::Relaxed);
            let _ = tx.send(find_nucleus_near(&orbit, len, dc0, radius, max_period, &stop));
        });
        self.zoom_snap.job = Some(SnapJob { rx, cancel, key, ref_pt, precision: p, near_axis });
    }

    /// The anchor a zoom by `factor` about the cursor at `cursor` (view pixels) should use: pulled
    /// toward the snap's target when it is within the radius (`snap_anchor`), else the cursor.
    pub(crate) fn snap_zoom_anchor(&self, cursor: [f64; 2], factor: f64) -> [f64; 2] {
        if !self.snap_active() || factor >= 1.0 {
            return cursor;
        }
        let vp = &self.viewport;
        let feature = match &self.zoom_snap.target {
            Some(SnapTarget::Nucleus { cx, cy, .. }) => {
                let (x, y) = vp.complex_to_pixel(cx, cy);
                [x, y]
            }
            Some(SnapTarget::Axis) => {
                let zero = BigFloat::from_f64(0.0, vp.precision);
                [cursor[0], vp.complex_to_pixel(&vp.center_x, &zero).1]
            }
            None => return cursor,
        };
        snap_anchor(cursor, feature, vp.height_px * SNAP_RADIUS_FRAC, factor)
    }
}

#[cfg(test)]
#[path = "../tests/unit/zoom_snap/tests.rs"]
mod tests;
