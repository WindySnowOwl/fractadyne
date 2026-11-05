//! Background standing for reference work done on the live view's behalf.
//!
//! A reference pick fans out across every core, and the live view can have several in flight at
//! once (the build for the frame, its lookahead queue, the hold). On a machine with few cores
//! that is enough to starve the UI thread and the graphics driver's present work: measured on an
//! 8-thread i7-9700K (2026-11-04), two lookahead picks of 3.1-3.8 s each at 1e999x left the frames
//! at the end of a zoom waiting 20-57 ms to present, while each frame's own work took 1 ms.
//!
//! So that work runs below normal priority. A thread marks itself with [`enter_background`]; every
//! thread it spawns adopts the same standing first thing ([`current`], then [`Standing::adopt`]),
//! however deep the spawning goes. Lowering a thread's priority is platform code, which this crate
//! does not carry: the application installs it once with [`install`]. Without it the standing is
//! still tracked and nothing else changes. On a machine with cores to spare, nothing waits on the
//! difference.

use std::cell::Cell;
use std::sync::OnceLock;

thread_local! {
    static BACKGROUND: Cell<bool> = const { Cell::new(false) };
}

/// What lowers the calling thread's priority, installed by the application.
static LOWER: OnceLock<fn()> = OnceLock::new();

/// Install the function that lowers the calling thread's priority. The first call wins; later
/// calls are ignored.
pub fn install(lower: fn()) {
    let _ = LOWER.set(lower);
}

/// Mark the calling thread as background work and lower its priority. Once per thread: a second
/// call changes nothing.
pub fn enter_background() {
    BACKGROUND.with(|b| {
        if !b.replace(true) {
            if let Some(lower) = LOWER.get() {
                lower();
            }
        }
    });
}

/// Whether the calling thread is background work.
pub fn is_background() -> bool {
    BACKGROUND.with(Cell::get)
}

/// A thread's standing, read by the thread that spawns workers and handed to them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Standing(bool);

/// The calling thread's standing, for the threads it is about to spawn.
pub fn current() -> Standing {
    Standing(is_background())
}

impl Standing {
    /// Take on this standing: called first thing in a spawned thread with its spawner's.
    pub fn adopt(self) {
        if self.0 {
            enter_background();
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/worker_priority/tests.rs"]
mod tests;
