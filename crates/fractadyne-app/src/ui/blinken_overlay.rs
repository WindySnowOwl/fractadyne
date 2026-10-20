//! The live-render front panel over the view (design/blinkenlights.md; View ▸ Live render panel,
//! key `B`): this frame's readings gathered from the renderer's state for `crate::blinken`.

use crate::FractadyneApp;
use crate::blinken::events::{Arith, Pass};
use crate::blinken::model::{Present, Snapshot};
use crate::blinken::{EngineFrame, Size};

impl FractadyneApp {
    /// Reads this frame for the panel and draws it over `view`. Called after the central view is
    /// built, so the frame record holds this frame's plan.
    pub(crate) fn draw_blinken(&mut self, ctx: &egui::Context, view: egui::Rect) {
        if self.blinken.size == Size::Off {
            self.blinken.update(Snapshot::default(), &EngineFrame::default());
            return;
        }
        let t0 = std::time::Instant::now();
        let v = 0usize;
        let rec = &self.perf.rec[v];
        let planned = rec.plan_calls > 0;
        let l2 = self.viewport.log2_magnification();
        // The picture's age, as the zoom harness and the P-G gate measure it: how far the view has
        // zoomed past the picture frozen on screen.
        let rc = &self.ref_cache[v];
        let age = if rc.frozen_center.is_some() { (l2 - rc.frozen_l2).max(0.0) } else { 0.0 };
        let dispatched = (planned && rec.dispatched).then(|| match rec.frame_kind {
            3 | 6 => Pass::Pin,
            7 => Pass::Tile,
            8 => Pass::Ssaa,
            _ => Pass::Moving,
        });
        let snap = Snapshot {
            frame: self.perf.frame_idx,
            dt_s: (self.perf.last_dt_ms / 1000.0) as f32,
            frame_ms: self.perf.frame_period_ms as f32,
            depth: l2,
            iterations: if planned { rec.gpu_iter } else { self.blinken_last_iter },
            arith: planned.then(|| match rec.mode {
                1 => Arith::Direct,
                2 => Arith::Floatexp,
                _ => Arith::Df32,
            }),
            age_oct: age as f32,
            present: match rec.present {
                1 => Present::Live,
                2 => Present::Reprojected,
                3 => Present::Held,
                _ => Present::None,
            },
            planned,
            dispatched,
            res: (rec.res_w, rec.res_h),
            timers: {
                let w = self.perf.timer_states[v].load(std::sync::atomic::Ordering::Relaxed);
                [(w & 3) as u8, ((w >> 2) & 3) as u8, ((w >> 4) & 3) as u8]
            },
        };
        if planned {
            self.blinken_last_iter = rec.gpu_iter;
        }
        let ss = rec.ss.max(1) as u64;
        let px = match dispatched {
            Some(Pass::Tile) if rec.tiled => rec.tile_w as u64 * rec.tile_h as u64 * ss * ss,
            Some(_) => rec.res_w as u64 * rec.res_h as u64 * ss * ss,
            None => 0,
        };
        // GPU memory, from the sizes the renderer allocates (16-byte texels; design/blinkenlights.md
        // "The engine room"): the iteration and aux G-buffers, the hold snapshot and the SSAA
        // accumulator, the orbit and BLA buffer, and the worker's copy of the reference.
        let texels = rec.res_w as u64 * rec.res_h as u64 * ss * ss;
        let gbuf = 2 * texels * 16;
        let display_px = (view.width() * view.height() * ctx.pixels_per_point().powi(2)) as u64;
        let hold = if self.perf.hold_active[v] { 2 * texels * 16 } else { 0 } + if rec.accum_count > 0 { display_px * 36 } else { 0 };
        let orbit = 16 * (rc.orbit.len() + rc.bla.len()) as u64;
        let worker = self.gpu_worker.is_some();
        // A counter reading's step accounting, when one landed: loop trips executed against
        // iterations advanced (their ratio is BLA's multiplier).
        let steps = self.perf.step_stats[v].lock().ok().and_then(|mut g| g.take());
        let ef = EngineFrame {
            nominal_steps: if dispatched.is_some() { rec.nominal_steps } else { 0 },
            executed_steps: steps.map_or(0, |s| s.executed),
            executed_of: steps.map_or(0, |s| s.iterations),
            px,
            worker,
            ram: self.perf.mem_rss,
            ram_peak: self.perf.mem_peak,
            ram_total: self.perf.mem_total.unwrap_or(0),
            vram: [gbuf, hold, orbit, if worker { orbit } else { 0 }],
            vram_cap_mb: self.sysinfo.vram_mb,
        };
        self.blinken.update(snap, &ef);
        let df32_from = 1.0e4f64.log2() as f32;
        let fe_from = crate::PERT_FE_THRESHOLD.log2() as f32;
        self.blinken.show(ctx, view, df32_from, fe_from);
        self.blinken.note_cost(t0.elapsed().as_secs_f64() * 1000.0);
    }
}
