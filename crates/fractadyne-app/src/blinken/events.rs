//! The event ring (design/blinkenlights.md, "What feeds it"): every live-render event the panel
//! shows, pushed whether or not the panel is open, so opening it shows the last seconds at once.
//!
//! Bounded ([`RING_CAP`] events; the oldest is overwritten), cheap (a short lock and a copy per
//! push, a few hundred a second at most), and stamped with the UI thread's frame and the app clock.
//! Builders on other threads push through the same lock. Readers ask for everything after the last
//! sequence number they saw, so each event is read once per reader and none is consumed.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// Events kept: ~10 s of a busy dive.
pub(crate) const RING_CAP: usize = 4096;

/// What happened. The meaning of an event's `a`, `b` and `f` depends on its kind, as noted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A pinned refresh began walking. `a` = the iteration ask, `b` = its resolution packed as
    /// `w << 16 | h`.
    PinStart,
    /// A pass of the walk went to the GPU. `a` = band start, `b` = band end (iterations).
    PinPass,
    /// The walk's last pass is in; it waits for that pass's reading.
    PinVerify,
    /// The walk's picture went on screen. `f` = the picture's age when adopted, octaves.
    PinAdopt,
    /// The walk was thrown away. `a` = [`Abandon`] as u32.
    PinAbandon,
    /// A pass of a split refresh. `a` = this pass (0-based), `b` = passes in the refresh.
    SplitPass,
    /// A pass's GPU time came back. `a` = [`Pass`] as u32, or [`PASS_OF_FRAME`] for the frame's
    /// own pass (whatever kind it was); `b` = the frame it ran in (low 32 bits), `f` = ms. `b`'s
    /// frame lets the scope put it in its column.
    PassTimed,
    /// A settle grid froze its geometry. `a` = its resolution `w << 16 | h`, `b` = the tile side
    /// (px); its columns and rows follow, as `next_settle_tile` derives them.
    GridStart,
    /// A settle tile, or a strip of one, went to the GPU. `a` = tile index (in grid order, not
    /// visit order), `b` = the tile's rows sent so far << 16 | its rows, `f` = this piece's rows.
    /// A whole tile is one piece of all its rows; strips need not be equal (the last is short).
    TileSent,
    /// The settle grid finished.
    GridDone,
    /// A supersampling sample was added. `a` = samples so far.
    SsaaSample,
    /// A reference build began. `a` = [`RefRole`] as u32, `f` = its depth, log2 magnification.
    RefLaunch,
    /// A reference build finished. `a` = role, `f` = depth.
    RefBuilt,
    /// A built reference was installed for the view. `a` = role, `f` = depth.
    RefInstalled,
    /// A build was dropped: the view overtook it, or it was superseded. `a` = role, `f` = depth.
    RefDropped,
    /// The series-approximation walk for a reference. `a` = role, `f` = ms (0 at its start).
    Series,
    /// The worker GPU delivered a frame. `a` = 1 when it went into the hold, 0 live.
    WorkerFrame,
    /// The palette's normalization took in a new escape range. `a`, `b` = its min and max,
    /// rounded to whole iterations.
    PaletteFeed,
    /// The arithmetic changed. `a` = from, `b` = to ([`Arith`] as u32).
    ModeSwitch,
}

/// Why a walk was thrown away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum Abandon {
    /// The view moved too far from the pin's.
    Drift = 0,
    /// It grew too old to be worth finishing.
    Age = 1,
    /// A newer picture (another walk, a live refresh, a worker frame) replaced it.
    Superseded = 2,
    /// Something it was keyed on changed (settings, reference, size).
    Rekeyed = 3,
    /// Any other reason.
    Other = 4,
}

/// [`Kind::PassTimed`]'s `a` for a reading of the frame's own pass, whatever its kind: the
/// iterate timer times one pass a frame and does not say which.
pub(crate) const PASS_OF_FRAME: u32 = u32::MAX;

/// The kind of a GPU pass, for the scope's stacked bars and the tile plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum Pass {
    Moving = 0,
    Pin = 1,
    Split = 2,
    Tile = 3,
    Ssaa = 4,
    Probe = 5,
    Worker = 6,
}

impl Pass {
    pub(crate) fn from_u32(v: u32) -> Pass {
        match v {
            1 => Pass::Pin,
            2 => Pass::Split,
            3 => Pass::Tile,
            4 => Pass::Ssaa,
            5 => Pass::Probe,
            6 => Pass::Worker,
            _ => Pass::Moving,
        }
    }
}

/// Which reference builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum RefRole {
    /// For the view as it is.
    Live = 0,
    /// Ahead of the view, where the zoom is heading.
    Ahead = 1,
    /// A tour's playback hold, prefetched so playback never waits.
    Hold = 2,
}

/// The role of a build the panel does not show (an export's).
pub(crate) const NO_PANEL_ROLE: u32 = u32::MAX;

/// A fresh id for a reference build, so the panel can pair its launch with its end.
pub(crate) fn next_build_id() -> u32 {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl RefRole {
    pub(crate) fn from_u32(v: u32) -> RefRole {
        match v {
            1 => RefRole::Ahead,
            2 => RefRole::Hold,
            _ => RefRole::Live,
        }
    }
}

/// The GPU arithmetic, for the mode lamp and the dial's bands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum Arith {
    Direct = 0,
    Df32 = 1,
    Floatexp = 2,
}

/// One event: what, when (frame and microseconds), for which view, and up to three values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Event {
    /// Its place in the ring's history, from 0; a reader asks for events after the last it saw.
    pub(crate) seq: u64,
    /// The UI thread's frame when it was pushed (a builder thread pushes the frame it observed).
    pub(crate) frame: u64,
    /// `crate::app_micros()` when it was pushed.
    pub(crate) t_us: u64,
    /// 0 = the main view, 1 = the second (dual view).
    pub(crate) view: u8,
    pub(crate) kind: Kind,
    pub(crate) a: u32,
    pub(crate) b: u32,
    pub(crate) f: f32,
}

/// A bounded history of events, oldest overwritten first.
pub(crate) struct Ring {
    buf: Vec<Event>,
    cap: usize,
    /// The sequence number the next event gets: also how many were ever pushed.
    next: u64,
}

impl Ring {
    pub(crate) fn new(cap: usize) -> Ring {
        Ring { buf: Vec::with_capacity(cap.max(1)), cap: cap.max(1), next: 0 }
    }

    /// Adds `e` (its `seq` is assigned here), overwriting the oldest event when full.
    pub(crate) fn push(&mut self, mut e: Event) {
        e.seq = self.next;
        let slot = (self.next % self.cap as u64) as usize;
        if self.buf.len() < self.cap {
            self.buf.push(e);
        } else {
            self.buf[slot] = e;
        }
        self.next += 1;
    }

    /// Appends to `out`, oldest first, every event still held whose `seq` is at least `from`, and
    /// returns the sequence number to ask from next time. A reader that fell more than the ring's
    /// capacity behind gets what is left (the oldest it missed are gone).
    pub(crate) fn since(&self, from: u64, out: &mut Vec<Event>) -> u64 {
        let oldest = self.next.saturating_sub(self.buf.len() as u64);
        for seq in from.max(oldest)..self.next {
            out.push(self.buf[(seq % self.cap as u64) as usize]);
        }
        self.next
    }

    /// How many events were ever pushed.
    pub(crate) fn pushed(&self) -> u64 {
        self.next
    }
}

fn ring() -> &'static Mutex<Ring> {
    static RING: OnceLock<Mutex<Ring>> = OnceLock::new();
    RING.get_or_init(|| Mutex::new(Ring::new(RING_CAP)))
}

/// The UI thread's current frame, for stamping events (set once a frame by [`set_frame`]).
static FRAME: AtomicU64 = AtomicU64::new(0);

/// Records the UI thread's frame index; every event pushed after this carries it.
pub(crate) fn set_frame(frame: u64) {
    FRAME.store(frame, Ordering::Relaxed);
}

/// The frame events are being stamped with.
pub(crate) fn frame() -> u64 {
    FRAME.load(Ordering::Relaxed)
}

/// Pushes an event for `view` (0 main, 1 second). Never blocks for long and never panics: a
/// poisoned lock (a panic while pushing, which cannot happen in `push`) is recovered.
pub(crate) fn emit(view: usize, kind: Kind, a: u32, b: u32, f: f32) {
    let e = Event { seq: 0, frame: frame(), t_us: crate::app_micros(), view: view.min(1) as u8, kind, a, b, f };
    let mut r = ring().lock().unwrap_or_else(|p| p.into_inner());
    r.push(e);
}

/// A resolution packed into one value, `w << 16 | h` (each clamped to 16 bits).
pub(crate) fn pack_res(w: u32, h: u32) -> u32 {
    (w.min(0xffff) << 16) | h.min(0xffff)
}

/// Appends every event after `from` to `out` and returns where to ask from next.
pub(crate) fn read_since(from: u64, out: &mut Vec<Event>) -> u64 {
    let r = ring().lock().unwrap_or_else(|p| p.into_inner());
    r.since(from, out)
}

/// How many events were ever pushed (for a reader that opens late and wants only the recent past).
pub(crate) fn pushed() -> u64 {
    ring().lock().unwrap_or_else(|p| p.into_inner()).pushed()
}

#[cfg(test)]
#[path = "../../tests/unit/blinken/events/tests.rs"]
mod tests;
