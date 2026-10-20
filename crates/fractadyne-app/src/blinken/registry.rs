//! The thread registry (design/blinkenlights.md, "The engine room"): how many threads are busy in
//! each role right now. A thread marks the span of its work with a [`Busy`] guard; a fan-out across
//! a pool marks all its threads at once. The panel reads a snapshot each frame; no one waits on it.

use std::sync::atomic::{AtomicU32, Ordering};

/// What a busy thread is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    /// Building a reference orbit (live, lookahead, hold or reactive).
    Reference = 0,
    /// One of a scoped pool's threads: the reference pick's scoring, the MPFR split products, the
    /// series walks.
    Pool = 1,
    /// Driving the worker GPU.
    Worker = 2,
    /// The pre-switch probe and other short side renders.
    Probe = 3,
    /// An image or tour export.
    Export = 4,
}

pub(crate) const ROLES: usize = 5;

static BUSY: [AtomicU32; ROLES] = [const { AtomicU32::new(0) }; ROLES];

/// Holds `n` threads of a role busy until dropped.
#[must_use = "the threads count as busy only while the guard lives"]
pub(crate) struct Busy {
    role: Role,
    n: u32,
}

impl Drop for Busy {
    fn drop(&mut self) {
        // Saturating: a count can never go below zero, whatever order guards drop in.
        let _ = BUSY[self.role as usize].fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(v.saturating_sub(self.n)));
    }
}

/// Marks one thread busy in `role` until the guard drops.
pub(crate) fn busy(role: Role) -> Busy {
    busy_n(role, 1)
}

/// Marks `n` threads busy in `role` (a pool's fan-out) until the guard drops.
pub(crate) fn busy_n(role: Role, n: u32) -> Busy {
    BUSY[role as usize].fetch_add(n, Ordering::Relaxed);
    Busy { role, n }
}

/// Threads busy in each role, indexed by `Role as usize`.
pub(crate) fn snapshot() -> [u32; ROLES] {
    std::array::from_fn(|i| BUSY[i].load(Ordering::Relaxed))
}

#[cfg(test)]
#[path = "../../tests/unit/blinken/registry/tests.rs"]
mod tests;
