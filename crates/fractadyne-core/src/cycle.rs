//! The attracting cycle an orbit trap can be placed on (coloring plan F4, part 2).
//!
//! Inside a hyperbolic component of `z^d + c` every orbit settles onto the component's attracting
//! cycle, so the trap measures the inside only by how near that cycle it passes. The trap-shape study
//! (2026-11-13) found the inside's spread of trap values 0.4 to 0.9 with the trap far from the cycle
//! and 9 to 15 with its outline on or through it, whatever the shape; at the edge of the period-2
//! bulb the point trap at the origin draws the inside one color and on the cycle a field of rays.
//!
//! Two ways to the cycle: [`attracting_cycle`] iterates the critical point in f64 from `c` (a view
//! whose center f64 holds, and every Julia view, whose `c` is f64); [`cycle_of_orbit`] reads it off
//! a reference orbit's samples (deeper views, where f64 cannot hold `c`, but the reference was
//! iterated at the view's precision).

/// An attracting cycle of `z^d + c`: its points in orbit order (each the image of the one before),
/// starting from the one nearest the critical point 0.
#[derive(Clone, Debug, PartialEq)]
pub struct Cycle {
    pub points: Vec<[f64; 2]>,
    /// log2 of the multiplier's modulus, `|∏ f'(z_k)|`: below 0 for an attracting cycle, −∞ for a
    /// superattracting one (through 0).
    pub log2_multiplier: f64,
}

impl Cycle {
    pub fn period(&self) -> usize {
        self.points.len()
    }
}

/// The longest period either finder looks for.
pub const MAX_PERIOD: usize = 1 << 16;

/// Two points of an orbit repeat when they are this close, relative to their size (or 1).
const REPEAT_TOL: f64 = 1.0e-10;

/// An orbit that reaches this `|z|²` has escaped (any degree's bailout is far below).
const ESCAPED2: f64 = 1.0e6;

fn mul(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn abs(a: [f64; 2]) -> f64 {
    a[0].hypot(a[1])
}

fn near(a: [f64; 2], b: [f64; 2]) -> bool {
    abs(sub(a, b)) <= REPEAT_TOL * abs(a).max(1.0)
}

/// `z^d`.
fn pow(z: [f64; 2], d: u32) -> [f64; 2] {
    let mut w = z;
    for _ in 1..d {
        w = mul(w, z);
    }
    w
}

/// `z^d + c`.
fn step(z: [f64; 2], c: [f64; 2], d: u32) -> [f64; 2] {
    let w = pow(z, d);
    [w[0] + c[0], w[1] + c[1]]
}

/// `log2 |f'(z)|` = `log2 d + (d − 1) log2 |z|`; −∞ at 0.
fn log2_deriv(z: [f64; 2], d: u32) -> f64 {
    f64::from(d).log2() + f64::from(d - 1) * abs(z).log2()
}

/// `points` from the one nearest 0, with the multiplier; `None` unless it attracts.
fn finish(mut points: Vec<[f64; 2]>, d: u32) -> Option<Cycle> {
    if points.is_empty() {
        return None;
    }
    let log2_multiplier: f64 = points.iter().map(|&z| log2_deriv(z, d)).sum();
    if !(log2_multiplier < 0.0) {
        return None;
    }
    let first = (0..points.len()).min_by(|&i, &j| abs(points[i]).total_cmp(&abs(points[j]))).unwrap_or(0);
    points.rotate_left(first);
    Some(Cycle { points, log2_multiplier })
}

/// The smallest period `p` (up to `max_p`) with which `at(0)`, `at(p)` and `at(2p)` repeat, `at(k)`
/// being the orbit `k` steps back from its newest point.
fn settled_period(at: impl Fn(usize) -> [f64; 2], max_p: usize) -> Option<usize> {
    let z = at(0);
    (1..=max_p).find(|&p| near(z, at(p)) && near(at(p), at(2 * p)))
}

/// The attracting cycle of `z^d + c` that the critical orbit from 0 falls into, found in f64: up to
/// `max_iter` steps and periods up to `max_period`, the cycle point Newton-refined. `None` when the
/// orbit escapes, or has not repeated within the budget (a `c` outside the set whose orbit stays
/// bounded a long time, or a period beyond `max_period`), or the cycle does not attract. Near a
/// component's edge, where the orbit settles slowly, successive periods differ by about (1 − the
/// multiplier) times the distance left, so they repeat before the orbit arrives and Newton finishes.
pub fn attracting_cycle(c: [f64; 2], d: u32, max_iter: u32, max_period: usize) -> Option<Cycle> {
    let d = d.max(2);
    let max_period = max_period.clamp(1, MAX_PERIOD);
    let ring = 2 * max_period + 1;
    let mut hist = vec![[0.0f64; 2]; ring];
    let mut z = [0.0f64, 0.0];
    let mut check = 1024u32;
    for n in 1..=max_iter {
        z = step(z, c, d);
        if !(z[0] * z[0] + z[1] * z[1] <= ESCAPED2) {
            return None;
        }
        hist[n as usize % ring] = z;
        if n == check || n == max_iter {
            check = check.saturating_mul(4);
            let back = (n as usize).min(ring - 1);
            let at = |k: usize| hist[(n as usize + ring - k) % ring];
            if let Some(p) = settled_period(at, (back / 2).min(max_period)) {
                // Newton may land on another root of f^p(z) = z (near a component's edge the cycle
                // sits close to the repelling fixed point it budded from): only a true, closed,
                // attracting cycle of period p is taken; otherwise the orbit iterates on.
                if let Some(cyc) = true_cycle(refine(z, c, d, p), c, d, p) {
                    return Some(cyc);
                }
            }
        }
    }
    None
}

/// Newton on `f^p(z) − z`, from a point the orbit has settled near; the point unchanged where the
/// derivative over a long period leaves f64's range.
fn refine(mut z: [f64; 2], c: [f64; 2], d: u32, p: usize) -> [f64; 2] {
    for _ in 0..8 {
        let (mut w, mut dw) = (z, [1.0f64, 0.0]);
        for _ in 0..p {
            let dp = pow(w, d - 1);
            dw = mul([f64::from(d) * dp[0], f64::from(d) * dp[1]], dw);
            w = step(w, c, d);
        }
        let g = sub(w, z);
        let gp = [dw[0] - 1.0, dw[1]];
        let den = gp[0] * gp[0] + gp[1] * gp[1];
        if !(den.is_finite() && den > 0.0 && g[0].is_finite() && g[1].is_finite()) {
            return z;
        }
        // z ← z − g / g'
        let q = [(g[0] * gp[0] + g[1] * gp[1]) / den, (g[1] * gp[0] - g[0] * gp[1]) / den];
        let next = sub(z, q);
        let done = abs(q) <= 1.0e-15 * abs(z).max(1.0);
        z = next;
        if done {
            break;
        }
    }
    z
}

/// The cycle through `z` if it is one of period exactly `p` that closes and attracts.
fn true_cycle(z: [f64; 2], c: [f64; 2], d: u32, p: usize) -> Option<Cycle> {
    let points = cycle_from(z, c, d, p);
    let closes = near(step(points[p - 1], c, d), points[0]);
    let shorter = (1..p).filter(|q| p % q == 0).any(|q| near(points[q], points[0]));
    if closes && !shorter { finish(points, d) } else { None }
}

/// The `p` points of the cycle through `z`, in orbit order.
fn cycle_from(z: [f64; 2], c: [f64; 2], d: u32, p: usize) -> Vec<[f64; 2]> {
    let mut points = Vec::with_capacity(p);
    let mut w = z;
    for _ in 0..p {
        points.push(w);
        w = step(w, c, d);
    }
    points
}

/// The attracting cycle a reference orbit `z` (`Z_0 = 0`, `Z_1`, ...) settled on, for `z^d + c`.
/// `closes`: the period where the orbit closed on itself (a reference built to its nucleus, whose
/// samples end at `Z_period ≈ 0`): the cycle is then `Z_0..Z_period`, superattracting. Otherwise
/// the orbit's last points must repeat with some period up to `max_period`, twice over. `None` for
/// an orbit that did neither (it escaped, or had not settled).
pub fn cycle_of_orbit(z: &[[f64; 2]], closes: Option<usize>, d: u32, max_period: usize) -> Option<Cycle> {
    let d = d.max(2);
    if let Some(p) = closes.filter(|&p| p >= 1 && p < z.len()) {
        return finish(z[..p].to_vec(), d);
    }
    let n = z.len().checked_sub(1)?;
    if !(abs(z[n]).powi(2) <= ESCAPED2) {
        return None;
    }
    let p = settled_period(|k| z[n - k], (n / 2).min(max_period.clamp(1, MAX_PERIOD)))?;
    finish(z[n + 1 - p..=n].to_vec(), d)
}

/// [`cycle_of_orbit`] for a reference orbit as it is stored (`orbit`, the first `orbit_len` packed
/// samples, decoded by [`crate::sample_xy`], extended-range dips included) with the build's tail: an
/// escaped orbit has no cycle; one that closed at its period ([`crate::OrbitPeriod`]) is the cycle
/// through its nucleus; else its last points must repeat.
pub fn cycle_of_reference(orbit: &[[f32; 4]], orbit_len: u32, tail: Option<&crate::OrbitTail>, d: u32) -> Option<Cycle> {
    if tail.is_some_and(|t| t.escaped) {
        return None;
    }
    let len = (orbit_len as usize).min(orbit.len());
    let z: Vec<[f64; 2]> = orbit[..len]
        .iter()
        .map(|s| {
            let (x, y) = crate::sample_xy(s);
            [x, y]
        })
        .collect();
    let closes = tail.and_then(|t| t.period.as_ref()).map(|p| p.period as usize);
    cycle_of_orbit(&z, closes, d, MAX_PERIOD)
}

#[cfg(test)]
#[path = "../tests/unit/cycle/tests.rs"]
mod tests;
