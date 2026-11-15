//! Keeping a split walk's threads on separate physical cores.
//!
//! A deep reference orbit's products are split across threads that meet at a barrier every step
//! (`backend_rug`'s split walk: 9 threads, the calling one included), so the step runs at the pace
//! of its slowest thread, and the threads spin between stages rather than sleep. Two of them on
//! the two logical CPUs of one core share that core: measured on a Ryzen 9 3950X (16 cores, 32
//! logical CPUs; 2026-11-15, the benchmark kit's 1e30000 path, alternated runs), the reference took
//! 43.0 and 42.4 s with no restriction, 44.5 and 43.3 s held to the 16 logical CPUs of 8 cores, and
//! 37.8 and 36.3 s held to one logical CPU of each of the 16 cores. Unrestricted, the scheduler
//! packed the threads onto sibling pairs.
//!
//! So a split walk's threads hold themselves to one logical CPU per physical core while they run
//! ([`spread`]). Which CPUs those are is platform code, which this crate does not carry: the
//! application installs it once with [`install`]. Without it, nothing changes.

use std::sync::OnceLock;

/// What holds the calling thread to one logical CPU per core (returning what to restore), and
/// what restores it; installed by the application.
#[derive(Clone, Copy)]
pub struct Hooks {
    /// Hold the calling thread to the spread set; `Some(previous mask)` when it changed anything.
    pub spread: fn() -> Option<usize>,
    /// Give the calling thread back the mask [`Hooks::spread`] returned.
    pub restore: fn(usize),
}

static HOOKS: OnceLock<Hooks> = OnceLock::new();

/// Install the platform hooks. The first call wins; later calls are ignored.
pub fn install(hooks: Hooks) {
    let _ = HOOKS.set(hooks);
}

/// Holds the calling thread to one logical CPU per core until dropped, then restores its mask.
#[must_use = "the thread is restored when this guard drops"]
pub struct Spread(Option<usize>);

/// Hold the calling thread to one logical CPU per core for as long as the guard lives. A no-op
/// without installed hooks, or where the platform declines (no SMT, a hybrid CPU, several
/// processor groups).
pub fn spread() -> Spread {
    Spread(HOOKS.get().and_then(|h| (h.spread)()))
}

impl Drop for Spread {
    fn drop(&mut self) {
        if let (Some(prev), Some(h)) = (self.0, HOOKS.get()) {
            (h.restore)(prev);
        }
    }
}

impl Spread {
    /// Whether this guard changed the thread's CPUs.
    pub fn active(&self) -> bool {
        self.0.is_some()
    }
}
