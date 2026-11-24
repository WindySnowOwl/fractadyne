//! Placing the orbit trap on the attracting cycle at the view's center (coloring plan F4, part 2):
//! which `c`, which finder (`fractadyne_core::cycle`), and what the panel says.
//!
//! Inside a component every orbit settles onto its attracting cycle, so the trap draws the inside
//! by how near that cycle it passes: through it the inside shows structure (the trap-shape study's
//! spread 9 to 15), away from it one color (0.4 to 0.9). Placing walks the cycle: each press moves
//! the trap's outline onto the next point (`TrapPlace::on_cycle`).

use crate::slider_range;
use fractadyne_core::cycle::{attracting_cycle, cycle_of_reference, Cycle, MAX_PERIOD};

/// Why the trap could not be placed on a cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NoCycle {
    /// Not a `z^d + c` family: its cycles are not found here.
    Family,
    /// The view's center (a Julia view's `c`) has no attracting cycle the search settles on: outside
    /// the set, or at the very edge of its component.
    Outside,
    /// A view past [`F64_LOG2_MAG`] whose reference orbit has not arrived yet.
    NotReady,
}

/// The f64 search's budget: some 10 ms at most, on the press.
const BUDGET: u32 = 1 << 21;

/// Up to this magnification (2^40, about 1.1e12×) the view's center is read in f64; deeper, f64
/// cannot hold it, and the cycle comes from view 0's reference, iterated at the view's precision.
pub(crate) const F64_LOG2_MAG: f64 = 40.0;

impl crate::FractadyneApp {
    /// The attracting cycle the trap can be placed on: a Julia view's `c`'s (an f64 already); the
    /// Mandelbrot view's center's (view 0's in the dual view, the pane the trap is set beside), in f64
    /// up to [`F64_LOG2_MAG`] and from view 0's reference deeper.
    pub(crate) fn trap_cycle(&self) -> Result<Cycle, NoCycle> {
        let f = self.fractal.formula_id();
        // The z^d + c families, the ones the series approximation serves.
        if !self.fractal.is_escape_time() || !fractadyne_core::formula::caps(f).series_approximation {
            return Err(NoCycle::Family);
        }
        let d = fractadyne_core::formula::power(f);
        if self.julia_mode && !self.dual {
            return attracting_cycle([self.julia_c.0, self.julia_c.1], d, BUDGET, MAX_PERIOD).ok_or(NoCycle::Outside);
        }
        if self.viewport.log2_magnification() < F64_LOG2_MAG {
            let c = [fractadyne_core::to_f64(&self.viewport.center_x), fractadyne_core::to_f64(&self.viewport.center_y)];
            return attracting_cycle(c, d, BUDGET, MAX_PERIOD).ok_or(NoCycle::Outside);
        }
        let vc = &self.ref_cache[0];
        if vc.orbit_len == 0 || vc.orbit.is_empty() {
            return Err(NoCycle::NotReady);
        }
        cycle_of_reference(&vc.orbit, vc.orbit_len, vc.orbit_tail.as_ref(), d).ok_or(NoCycle::Outside)
    }

    /// Move the trap onto the next point of that cycle: the period and the point taken, from 1. Only
    /// what the placing moves is written (the center of a point or a cross, a circle's size), clamped
    /// into the panel's ranges, so a turn or a size the shape ignores stays as it was set.
    pub(crate) fn place_trap_on_cycle(&mut self) -> Result<(usize, usize), NoCycle> {
        self.place_trap_on_cycle_for(false)
    }

    /// [`Self::place_trap_on_cycle`] for the method's trap, or (`inside`) the inside's own; the
    /// inside's without a shape of its own is left alone.
    pub(crate) fn place_trap_on_cycle_for(&mut self, inside: bool) -> Result<(usize, usize), NoCycle> {
        let cycle = self.trap_cycle()?;
        let (shape, at) = if inside {
            match self.coloring.interior_trap {
                Some(s) => (s, self.coloring.interior_trap_place),
                None => return Ok((cycle.period(), 0)),
            }
        } else {
            (self.coloring.trap_type, self.coloring.trap_place)
        };
        let (placed, k) = at.on_cycle(shape, &cycle.points);
        let clamp = |v: f32, r: std::ops::RangeInclusive<f32>| v.clamp(*r.start(), *r.end());
        let mut place = at;
        if shape.spec().closed {
            place.size = clamp(placed.size, slider_range::TRAP_SIZE);
        } else {
            place.center = placed.center.map(|v| clamp(v, slider_range::TRAP_CENTER));
        }
        if inside {
            self.coloring.interior_trap_place = place;
        } else {
            self.coloring.trap_place = place;
        }
        Ok((cycle.period(), k + 1))
    }

    /// What the panel says after a press.
    pub(crate) fn trap_cycle_message(&self, r: Result<(usize, usize), NoCycle>) -> String {
        match r {
            Ok((period, point)) => tr!(trap_on_cycle_done, period = period, point = point),
            Err(NoCycle::Family) => tr!(trap_on_cycle_family, family = self.fractal.display_name()),
            Err(NoCycle::Outside) => tr!(trap_on_cycle_outside),
            Err(NoCycle::NotReady) => tr!(trap_on_cycle_not_ready),
        }
    }
}
