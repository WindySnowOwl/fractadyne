//! The per-frame record: one structured row per frame per view, always on
//! (design/live-render-robustness.md §6.3–§6.6, workstream W1).
//!
//! ⭐⭐**Three jobs, one record.** Diagnose a field failure from the artefacts alone; VALIDATE a
//! fix by comparing records before and after rather than by argument; and be the input a replay
//! test is built from. Before this, the richest always-on artefact was a 24-entry ring of budget
//! decisions — about 1.4 s of a 33 s slow episode, with no frame index to join it to anything —
//! and everything else useful was behind `FRACTADYNE_TRACE`, which a field session never has on.
//!
//! **Where a record's values come from.** Each field is written IN PLACE, into
//! `Perf::rec[view]`, by the code that decided it — the plan half at the end of `build_params`,
//! the reading at the budget controller, the counters where the GPU reading is drained — and the
//! frame loop fills the rest and emits it once, at the end of `update`. Never recomputed at the
//! emit: a record that re-derives what a guard decided agrees with itself by construction, and
//! then measures nothing (the design's P10).
//!
//! **Sinks.** (1) An in-memory ring of [`RING_LEN`] records, always on. (2) `<logs>/frames.bin`,
//! a fixed-size circular file of [`SLOT_BYTES`]-byte slots written in place with no fsync, for the
//! deaths the panic hook never sees (`0xc0000409`, `0xc0000005`): the page stays in the OS cache,
//! so a process abort leaves the tail intact, and the next launch folds it into its unclean-exit
//! report. A hard power-off still loses it. (3) The crash report's `frames:` section and a
//! companion `crash-*-frames.jsonl` holding the whole ring. (4) `<logs>/frames.jsonl`, the
//! long-horizon log — written by its OWN thread (see [`JsonlMsg`]); only `frames.bin` is written
//! on the recording thread, because only it has to survive an abort.
//!
//! ⚠**A slot is keyed on the record's sequence number, not on its frame index.** The design text
//! keys it `(frame × 2 + view) % RING_LEN`, which hands a single view only the even slots — half
//! the coverage its own figures promise. `seq % RING_LEN` gives one view all of them and two views
//! half each, which is what was intended, and a view can still never overwrite the other's slot
//! for the same frame.
//!
//! **The encoding is this file.** Fields are declared once, in `frame_record!` below; the struct,
//! the field list a reader checks against ([`FrameRecord::FIELDS`]), the binary slot codec and the
//! JSON writer are all generated from that one list, so a field cannot be added to one and not the
//! others. Scalars are little-endian in declared order, `bool` is one byte, floats are their bit
//! patterns (so the binary round trip is exact), and every enumeration is a `u8` whose 0 means
//! "unset" — so "a required field was never filled in" is something a check can see.
//!
//! ⚠**The JSON is exact; a reader must be too.** Floats are written with Rust's shortest
//! round-trip `Display`, so the text holds the value to the last bit. But `serde_json`'s DEFAULT
//! float parser is best-effort: measured on 200 values of the kind recorded here it mis-rounded 22
//! by one ULP. Python's `float()` is correctly rounded; a Rust reader (the replay of W5) must parse
//! with `str::parse` or enable `serde_json/float_roundtrip`, or a replayed controller input can
//! land a ULP to the other side of a threshold and the replay will "diverge" for no reason.

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

/// Bumped whenever a field is added, removed, retyped or reordered. Every record and every file
/// header carries it, and a reader must refuse a schema it does not know rather than guess.
pub(crate) const SCHEMA: u16 = 3;
/// Records the in-memory ring (and `frames.bin`) holds. At the 1–5 fps of the failing cadence
/// that is 14–68 minutes of one view — the ring lengthens in TIME exactly as the app slows down,
/// the opposite of the 24-entry decision ring it supersedes.
pub(crate) const RING_LEN: usize = 4096;
/// One `frames.bin` slot. Deliberately larger than a record, so adding a field does not re-key an
/// existing file. 640 from schema 2: its moving-frame sizing fields left 8 bytes of 512, under the
/// 32 the headroom test keeps (the file grows from 2.1 to 2.6 MB).
pub(crate) const SLOT_BYTES: usize = 640;
/// `frames.bin` starts with this many bytes of header (magic, schema, session, then the session's
/// header JSON), followed by `RING_LEN` slots.
pub(crate) const HEADER_BYTES: usize = 4096;
/// magic(4) schema(2) payload_len(2) session(8) seq(8) checksum(4) reserved(4)
const SLOT_HEADER: usize = 32;
const SLOT_MAGIC: [u8; 4] = *b"FDFR";
const FILE_MAGIC: [u8; 4] = *b"FDFH";
/// The records the crash report prints inline; the full ring goes to the companion file.
const CRASH_INLINE: usize = 40;

/// What a row IS. `0` = unset.
pub(crate) mod kind {
    /// One frame of one view, written by the UI thread at the end of `update`.
    pub(crate) const FRAME: u8 = 1;
    /// Written by the WATCHDOG thread when the UI thread has recorded nothing for
    /// `stall_ms`. The record's only other writer is the thread a wedge stops, so without this
    /// a wedge is a silence a reader has to notice; with it, a positive row with a length (the
    /// design's P19: a judge must not run on the thread it judges).
    pub(crate) const STALL: u8 = 2;
}
/// What the frame put on screen for this view. `0` = unset (the view was not built this frame).
pub(crate) mod present {
    pub(crate) const LIVE: u8 = 1;
    pub(crate) const REPROJECT: u8 = 2;
    pub(crate) const HOLD: u8 = 3;
}
/// Where a judged reading came from. `0` = no reading judged this frame.
pub(crate) mod read_src {
    pub(crate) const GPU: u8 = 1;
    pub(crate) const WALL: u8 = 2;
}
/// What the budget controller did with that reading. `0` = no reading.
pub(crate) mod verdict {
    /// Under 0.7× the budget and not slow: carries no signal for the budget (`budget_step` → None).
    pub(crate) const DISCARDED: u8 = 1;
    pub(crate) const MOVED: u8 = 2;
    pub(crate) const UNCHANGED: u8 = 3;
    /// Longer than the CPU-side window its pass ran inside (`timing_witness`): it did not happen,
    /// and prices nothing (beta.129).
    pub(crate) const IMPOSSIBLE: u8 = 4;
}
/// Why a reading that asked for growth did not get it. `0` = no refusal. A REASON, recorded
/// rather than inferred from an "(unchanged)" line: the refusal used to be visible only under
/// `FRACTADYNE_TRACE=gpu`, i.e. never in the field.
pub(crate) mod refusal {
    pub(crate) const BUILDING: u8 = 1;
    /// The timing witness judged the reading possibly too SHORT (an empty queue's window left far
    /// more unexplained than any tested card): it may shrink the budget, never grow it (beta.129).
    pub(crate) const SHORT: u8 = 2;
    /// The wall-clock dead-man is latched (the budget-blind tripwire fired and no frame since has
    /// been quick): growth waits for the latch to clear (beta.131, `DEAD_MAN`).
    pub(crate) const BLIND: u8 = 3;
}

// ------------------------------------------------------------------------------------------------
// The field codec.

/// A fixed-width scalar the record can hold.
pub(crate) trait Field: Copy {
    const TYPE: &'static str;
    const SIZE: usize;
    /// The type's zero, for the `const` [`FrameRecord::ZERO`] a `static` can be built from.
    const ZERO: Self;
    fn put(self, b: &mut [u8], at: &mut usize);
    fn get(b: &[u8], at: &mut usize) -> Option<Self>;
    fn json(self, s: &mut String);
}

macro_rules! int_field {
    ($t:ty, $name:literal) => {
        impl Field for $t {
            const TYPE: &'static str = $name;
            const SIZE: usize = std::mem::size_of::<$t>();
            const ZERO: Self = 0;
            fn put(self, b: &mut [u8], at: &mut usize) {
                b[*at..*at + Self::SIZE].copy_from_slice(&self.to_le_bytes());
                *at += Self::SIZE;
            }
            fn get(b: &[u8], at: &mut usize) -> Option<Self> {
                let v = <$t>::from_le_bytes(b.get(*at..*at + Self::SIZE)?.try_into().ok()?);
                *at += Self::SIZE;
                Some(v)
            }
            fn json(self, s: &mut String) {
                use std::fmt::Write as _;
                let _ = write!(s, "{self}");
            }
        }
    };
}
int_field!(u8, "u8");
int_field!(u16, "u16");
int_field!(u32, "u32");
int_field!(u64, "u64");

macro_rules! float_field {
    ($t:ty, $bits:ty, $name:literal) => {
        impl Field for $t {
            const TYPE: &'static str = $name;
            const SIZE: usize = std::mem::size_of::<$t>();
            const ZERO: Self = 0.0;
            fn put(self, b: &mut [u8], at: &mut usize) {
                self.to_bits().put(b, at);
            }
            fn get(b: &[u8], at: &mut usize) -> Option<Self> {
                <$bits>::get(b, at).map(<$t>::from_bits)
            }
            fn json(self, s: &mut String) {
                use std::fmt::Write as _;
                // Rust's float Display is the shortest string that round-trips, so the JSON is as
                // exact as the binary. JSON has no NaN or infinity: those are written as null.
                if self.is_finite() {
                    let _ = write!(s, "{self}");
                } else {
                    s.push_str("null");
                }
            }
        }
    };
}
float_field!(f32, u32, "f32");
float_field!(f64, u64, "f64");

impl Field for bool {
    const TYPE: &'static str = "bool";
    const SIZE: usize = 1;
    const ZERO: Self = false;
    fn put(self, b: &mut [u8], at: &mut usize) {
        (self as u8).put(b, at);
    }
    fn get(b: &[u8], at: &mut usize) -> Option<Self> {
        match u8::get(b, at)? {
            0 => Some(false),
            1 => Some(true),
            _ => None, // a torn or foreign slot, not a boolean
        }
    }
    fn json(self, s: &mut String) {
        s.push_str(if self { "true" } else { "false" });
    }
}

/// Declares the record once. Generates the struct, [`FrameRecord::FIELDS`], the payload size, the
/// slot codec and the JSON writer from the same list.
macro_rules! frame_record {
    ($( $(#[$meta:meta])* $name:ident : $ty:ty, )*) => {
        /// One frame of one view. See the module doc for where each value comes from.
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub(crate) struct FrameRecord {
            $( $(#[$meta])* pub(crate) $name: $ty, )*
        }

        impl FrameRecord {
            /// `(name, type)` for every field in encoding order — what a reader validates a file
            /// against, and what `scripts/framelog.py` is checked against.
            pub(crate) const FIELDS: &'static [(&'static str, &'static str)] =
                &[ $( (stringify!($name), <$ty as Field>::TYPE), )* ];
            /// Encoded payload size in bytes.
            pub(crate) const PAYLOAD_BYTES: usize = 0 $( + <$ty as Field>::SIZE )*;
            /// Every field zero — `Default`, but usable in a `static`.
            pub(crate) const ZERO: Self = Self { $( $name: <$ty as Field>::ZERO, )* };

            fn put_payload(&self, b: &mut [u8], at: &mut usize) {
                $( self.$name.put(b, at); )*
            }

            fn get_payload(b: &[u8], at: &mut usize) -> Option<Self> {
                Some(Self { $( $name: <$ty as Field>::get(b, at)?, )* })
            }

            /// One JSON object, keys in encoding order.
            pub(crate) fn to_json(&self) -> String {
                let mut s = String::with_capacity(2048);
                s.push('{');
                let mut first = true;
                $(
                    if !first { s.push(','); }
                    first = false;
                    s.push('"');
                    s.push_str(stringify!($name));
                    s.push_str("\":");
                    self.$name.json(&mut s);
                )*
                let _ = first;
                s.push('}');
                s
            }
        }
    };
}

frame_record! {
    // ---- identity: the join key nothing had before
    /// Always [`SCHEMA`]; stamped by [`record`].
    schema: u16,
    /// [`kind`]: a frame, or a watchdog stall row.
    kind: u8,
    /// Record sequence number within the session (both views share it); stamped by [`record`].
    seq: u64,
    /// The app's `frame_idx` — the same number the slow-frame line, `[fd-glide] fN`, the chunk and
    /// pin traces and the timestamp overlay carry.
    frame: u64,
    /// Milliseconds since process start (the log's `[+s]` clock).
    t_ms: u64,
    view: u8,
    dual: bool,
    /// `build_params` calls for this view this frame. 0 = the view was not built this frame, and
    /// every plan field below is then unset.
    plan_calls: u8,
    /// On a [`kind::STALL`] row: how long the UI thread had recorded nothing (its `frame` is the
    /// last frame that WAS recorded). 0 on a frame row.
    stall_ms: u32,

    // ---- the ask
    /// The session's iteration setting (the ceiling auto-iter scales under).
    max_iter: u32,
    auto_iter: bool,
    /// What the shader was handed — after the placeholder cap and the SA-skip rescue.
    iter: u32,
    gpu_iter: u32,
    eff_iter: u32,
    boost: f64,
    /// Last settled capped-pixel fraction; -1 = none (moving, or never measured).
    capped_frac: f64,
    iter_exhausted: bool,
    budget_maxed: bool,
    /// A reference build for this view is in flight.
    building: bool,

    // ---- the reference: the field shape of 2026-09-21 was orbit_len 655 against an ask of 4627
    has_ref: bool,
    orbit_id: u64,
    ref_len: u32,
    ref_partial: bool,
    ref_prec: u32,
    /// The skip the shader was actually handed (`usable_sa_skip`), and the reference's own.
    sa_skip: u32,
    sa_skip_raw: u32,
    bla_on: bool,

    // ---- the price
    fe_budget: u64,
    budget_ok: bool,
    bootstrap: u64,
    /// The step budget this frame's plan was sized against.
    tdr_steps: u64,
    /// The dispatch's cost as the LIVE manifest states it (a chunk's RANGE, past the free skip).
    steps: u64,
    /// The steps that travel with the pass, so its timing comes back already paired.
    nominal_steps: u64,
    /// Worst measured steps/ms in the current mode (0 = none) and the current smoothness rate.
    mode_rate: f64,
    motion_rate: f64,
    wall_fallback: bool,
    full_inflight: u32,
    pin_inflight: u32,
    present_throttle: u32,

    // ---- the state (as dispatched, not as requested)
    mode: u8,
    res_w: u32,
    res_h: u32,
    ss: u8,
    visible_res: f64,
    motion_res: f64,
    /// A moving frame's sizing (schema 2): the motion pass it was sized against, how far its walk
    /// had to reach (`render::motion_need`), the scale `visible_res_target` asked for, and the scale
    /// the other caps allowed before the ladder's rung (`res_scale`). The dispatched scale is the
    /// smaller of that and `visible_res`, so these say which one bound it. 0 when not sized moving.
    vis_pass: u64,
    vis_need: f64,
    vis_target: f64,
    vis_pre_scale: f64,
    chunked: bool,
    chunk_lo: u32,
    chunk_hi: u32,
    chunk_cursor: u32,
    chunk_governed: bool,
    tiled: bool,
    tile_w: u32,
    tile_h: u32,
    tile_pending: bool,
    /// The app's own "this frame submitted a pass" stamp (`fe_dispatch_frame == frame`).
    dispatched: bool,
    key_changed: bool,
    interacting: bool,
    pin_frame: bool,
    pin_active: bool,
    hold_copy: bool,
    display_hold: bool,
    /// [`present`]: what the view put on screen.
    present: u8,
    /// The render this frame's pixels belong to (a pin's passes carry the pin's start frame).
    content_tag: u64,
    live_complete: bool,
    hold_verified: bool,
    blank_walks: u32,
    accum_count: u32,
    accum_present: bool,
    /// Progressive-supersampling runs BEGUN this frame for this view. The log line that marks a
    /// begin is rate-limited (it once made up ~65% of a crashing session's log, restarting ~31
    /// times a second); this count is not, so a flapping run stays visible in the record.
    accum_begins: u8,

    // ---- the signals
    /// The interval that ENDED at this frame's start, and this frame's own `update` body. The
    /// time outside a body is `last_dt_ms` minus the PREVIOUS record's `body_ms`.
    last_dt_ms: f64,
    body_ms: f64,
    cap_sleep_ms: f64,
    repaint_requested: bool,
    ts_supported: bool,
    /// Frames since a GPU timing last arrived for this view, and since it last dispatched.
    frames_since_reading: u64,
    frames_since_dispatch: u64,
    last_iterate_ms: f64,
    blind_slow_frames: u32,
    blind_slow_readings: u32,
    blind_warned: bool,

    // ---- the counters, ONLY on the frame a reading arrives, stamped with the render it describes.
    // A readback lands 2-3 frames late and arms only for full-frame iterates; recorded as a
    // per-frame value it would attribute one frame's rebase count to a dozen others — the very
    // misattribution this record exists to detect, manufactured by the instrument.
    ctr_new: bool,
    ctr_tag: u64,
    ctr_cursor: u32,
    ctr_escaped: u32,
    ctr_px: u32,
    ctr_rebase: u64,
    ctr_bla_skip: u64,

    // ---- colour and normalization (absent from every diagnostic artefact before this)
    norm_set: bool,
    norm_lo: f32,
    norm_hi: f32,
    norm_shown_set: bool,
    norm_shown_lo: f32,
    norm_shown_hi: f32,
    norm_locked: bool,
    norm_complete: bool,

    // ---- the reading the budget controller judged this frame (the last, if several)
    read_n: u8,
    /// [`read_src`]
    read_src: u8,
    read_ms: f64,
    read_steps: u64,
    read_budget_before: u64,
    read_budget_after: u64,
    /// [`verdict`]
    read_verdict: u8,
    read_ok: bool,
    read_lethal: bool,
    /// [`refusal`]
    refusal: u8,
    /// Schema 3 — the TIMING WITNESS for a GPU reading (`timing_witness`): ms from the arming of
    /// the pass's timer to its frame's completion callback, an upper bound on the pass's GPU time
    /// (0 = not witnessed: a wall reading, or the completion not yet seen).
    read_window_ms: f64,
    /// The previous frame's work had completed before the timer was armed, so the window is this
    /// frame's own GPU work plus callback latency.
    read_queue_empty: bool,

    // ---- the user's input this frame (window-wide; the same on both views' records)
    in_wheel: f32,
    in_zoom: f32,
    in_drag_dx: f32,
    in_drag_dy: f32,
    in_primary_down: bool,
    in_primary_pressed: bool,
    in_pointer_x: f32,
    in_pointer_y: f32,
    in_space: bool,
    in_keys: u8,
    autopilot: bool,
    zoom_oct_s: f64,

    // ---- the instrument's own cost
    /// Wall time the PREVIOUS frame's emit took (both views, field fill + ring + `frames.bin`),
    /// in microseconds. The record's overhead is a claim to be measured, and this is the
    /// measurement — in every run, including the field, where a slow disk would show here first.
    rec_us: f32,
}

/// The record must fit its slot with room to grow; a field added past that is a compile error,
/// not a silently truncated file.
const _: () = assert!(SLOT_HEADER + FrameRecord::PAYLOAD_BYTES <= SLOT_BYTES);

/// FNV-1a over the payload: enough to tell a torn or foreign slot from a record.
fn checksum(b: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &x in b {
        h ^= x as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Encode one record into a slot.
pub(crate) fn encode_slot(r: &FrameRecord, session: u64) -> [u8; SLOT_BYTES] {
    let mut b = [0u8; SLOT_BYTES];
    let mut at = SLOT_HEADER;
    r.put_payload(&mut b, &mut at);
    let len = at - SLOT_HEADER;
    let sum = checksum(&b[SLOT_HEADER..at]);
    b[0..4].copy_from_slice(&SLOT_MAGIC);
    b[4..6].copy_from_slice(&SCHEMA.to_le_bytes());
    b[6..8].copy_from_slice(&(len as u16).to_le_bytes());
    b[8..16].copy_from_slice(&session.to_le_bytes());
    b[16..24].copy_from_slice(&r.seq.to_le_bytes());
    b[24..28].copy_from_slice(&sum.to_le_bytes());
    b
}

/// Decode a slot written by this schema for this session. `None` for an empty slot, a torn one
/// (checksum), another session's, or another schema's — a reader must never guess.
pub(crate) fn decode_slot(b: &[u8], session: u64) -> Option<FrameRecord> {
    if b.len() < SLOT_BYTES || b[0..4] != SLOT_MAGIC {
        return None;
    }
    let schema = u16::from_le_bytes(b[4..6].try_into().ok()?);
    let len = u16::from_le_bytes(b[6..8].try_into().ok()?) as usize;
    let sess = u64::from_le_bytes(b[8..16].try_into().ok()?);
    let sum = u32::from_le_bytes(b[24..28].try_into().ok()?);
    if schema != SCHEMA || sess != session || len != FrameRecord::PAYLOAD_BYTES {
        return None;
    }
    let payload = b.get(SLOT_HEADER..SLOT_HEADER + len)?;
    if checksum(payload) != sum {
        return None;
    }
    let mut at = 0;
    FrameRecord::get_payload(payload, &mut at)
}

// ------------------------------------------------------------------------------------------------
// The sinks.

struct Ring {
    /// Allocated once, to `RING_LEN`, on the first record.
    buf: Vec<FrameRecord>,
    /// Sequence number the next record gets.
    next: u64,
}

static RING: Mutex<Ring> = Mutex::new(Ring { buf: Vec::new(), next: 0 });
/// `frames.bin`, once opened for this session. `Err` = opening failed or logging is off, and it
/// is not retried every frame.
static BIN: Mutex<Option<Result<std::fs::File, ()>>> = Mutex::new(None);
/// The session header (JSON object text), set by the app once the adapter is known.
static HEADER: Mutex<String> = Mutex::new(String::new());

/// This process's session id: a random-enough 64-bit value (clock ⊕ pid), fixed for the run. It is
/// what tells this session's slots from a previous one's in a file that was not fully overwritten.
pub(crate) fn session_id() -> u64 {
    static ID: OnceLock<u64> = OnceLock::new();
    *ID.get_or_init(|| {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let x = t ^ ((std::process::id() as u64) << 32) ^ 0x9E37_79B9_7F4A_7C15;
        // splitmix64 finaliser, so neighbouring launches do not share high bits.
        let x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        let x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        (x ^ (x >> 31)).max(1)
    })
}

fn bin_path() -> Option<PathBuf> {
    super::logs_dir().map(|d| d.join("frames.bin"))
}

/// Set (or replace) the session header — the facts a reader needs to know WHICH code produced the
/// records: version and commit, adapter and its capabilities, tunables, bignum backend, window,
/// and the session's controller settings. Rewritten into `frames.bin` if that is already open.
pub(crate) fn set_header(json: String) {
    if let Ok(mut h) = HEADER.lock() {
        *h = json;
    }
    if let Ok(mut g) = BIN.lock() {
        if let Some(Ok(f)) = g.as_mut() {
            let _ = write_file_header(f);
        }
    }
}

fn header_json() -> String {
    let h = HEADER.lock().map(|h| h.clone()).unwrap_or_default();
    if h.is_empty() {
        "{}".to_string()
    } else {
        h
    }
}

fn write_file_header(f: &mut std::fs::File) -> std::io::Result<()> {
    let json = header_json();
    // Never cut JSON mid-token: a reader would get an unparseable header and no way to tell why.
    let json = if json.len() > HEADER_BYTES - 20 {
        format!("{{\"header_truncated\":true,\"bytes\":{}}}", json.len())
    } else {
        json
    };
    let mut b = vec![0u8; HEADER_BYTES];
    b[0..4].copy_from_slice(&FILE_MAGIC);
    b[4..6].copy_from_slice(&SCHEMA.to_le_bytes());
    b[8..16].copy_from_slice(&session_id().to_le_bytes());
    let j = json.as_bytes();
    let n = j.len().min(HEADER_BYTES - 20);
    b[16..20].copy_from_slice(&(n as u32).to_le_bytes());
    b[20..20 + n].copy_from_slice(&j[..n]);
    f.seek(SeekFrom::Start(0))?;
    f.write_all(&b)
}

/// Create (truncating any previous session's) `frames.bin` at its full size and write its header.
/// ⚠The previous session's file must have been read first — [`super::init`] does that, through
/// the unclean-exit report, before any frame can be recorded.
fn open_bin(path: &Path) -> std::io::Result<std::fs::File> {
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    f.set_len((HEADER_BYTES + RING_LEN * SLOT_BYTES) as u64)?;
    write_file_header(&mut f)?;
    Ok(f)
}

fn write_slot(f: &mut std::fs::File, index: usize, slot: &[u8; SLOT_BYTES]) -> std::io::Result<()> {
    let off = (HEADER_BYTES + index * SLOT_BYTES) as u64;
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileExt;
        f.seek_write(slot, off).map(|_| ())
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileExt;
        f.write_all_at(slot, off)
    }
    #[cfg(not(any(windows, unix)))]
    {
        f.seek(SeekFrom::Start(off))?;
        f.write_all(slot)
    }
}

/// Record one frame of one view: stamp its schema and sequence number, keep it in the ring, write
/// its slot to `frames.bin`, and hand it to `frames.jsonl`. Cheap by construction — no allocation
/// on an ordinary frame after the first call, uncontended locks, one positional 512-byte write
/// into the page cache, and no `frames.jsonl` I/O at all (that is [`JsonlMsg`]'s thread) — and
/// MEASURED: every record carries the previous emit's cost (`rec_us`).
pub(crate) fn record(r: FrameRecord) {
    record_inner(r, true);
}

/// The watchdog's STALL row: the UI thread has recorded nothing for `stall_ms`. Never blocks — the
/// thread being judged may be the one holding a lock, and a watchdog that wedges behind the wedge
/// it is reporting is no watchdog. Skipped (and said so, by its absence) if the locks stay busy.
pub(crate) fn record_stall(stall_ms: u64) {
    let r = FrameRecord {
        kind: kind::STALL,
        stall_ms: stall_ms.min(u32::MAX as u64) as u32,
        t_ms: (super::elapsed_s() * 1000.0) as u64,
        ..Default::default()
    };
    record_inner(r, false);
}

/// `lock()` or, for the watchdog, a bounded `try_lock`.
fn acquire<T>(m: &Mutex<T>, blocking: bool) -> Option<std::sync::MutexGuard<'_, T>> {
    if blocking {
        return Some(m.lock().unwrap_or_else(|p| p.into_inner()));
    }
    for _ in 0..50 {
        match m.try_lock() {
            Ok(g) => return Some(g),
            Err(std::sync::TryLockError::Poisoned(p)) => return Some(p.into_inner()),
            Err(std::sync::TryLockError::WouldBlock) => std::thread::sleep(std::time::Duration::from_millis(1)),
        }
    }
    None
}

fn record_inner(mut r: FrameRecord, blocking: bool) {
    let Some(mut ring) = acquire(&RING, blocking) else { return };
    r.schema = SCHEMA;
    if r.kind == kind::STALL {
        // The last frame that WAS recorded — what the stall is measured from.
        let n = ring.next;
        r.frame = if n == 0 || ring.buf.is_empty() {
            0
        } else {
            (0..n.min(RING_LEN as u64))
                .map(|k| ring.buf[((n - 1 - k) % RING_LEN as u64) as usize])
                .find(|x| x.kind == kind::FRAME)
                .map_or(0, |x| x.frame)
        };
    }
    r.seq = ring.next;
    ring.next += 1;
    if ring.buf.is_empty() {
        ring.buf = vec![FrameRecord::default(); RING_LEN];
    }
    let idx = (r.seq % RING_LEN as u64) as usize;
    ring.buf[idx] = r;
    drop(ring);

    if let Some(mut g) = acquire(&BIN, blocking) {
        if g.is_none() {
            *g = Some(bin_path().ok_or(()).and_then(|p| open_bin(&p).map_err(|_| ())));
        }
        if let Some(Ok(f)) = g.as_mut() {
            let _ = write_slot(f, idx, &encode_slot(&r, session_id()));
        }
    }
    if let Some(mut j) = acquire(&JSONL, blocking) {
        j.take(&r);
    }
}

// ------------------------------------------------------------------------------------------------
// frames.jsonl: the long-horizon log.

/// Full rows are written only for EVENTS, and a compact summary row once a second. Whatever the
/// design text lists, a trigger that fires on ordinary motion ("the frame was not presented live")
/// would write a 2 KB row per frame of every dive — ~120 KB/s — against an exit criterion of under
/// 5 MB for a fourteen-minute session. So the SEVERE events (a slow frame, a lethal reading, a
/// watchdog stall) are always written, up to [`SEVERE_PER_S`]; the ROUTINE ones (the budget moved,
/// a growth refusal, a mode switch) at most [`ROUTINE_PER_S`] a second; and the summary row carries
/// the counts of everything, so a rate-limited event is still COUNTED.
const SEVERE_PER_S: u32 = 20;
const ROUTINE_PER_S: u32 = 2;
/// A frame the wall calls slow for the event log (the slow-frame log line's threshold).
const SLOW_MS: f64 = 200.0;
/// Rotation: past this size `frames.jsonl` shifts to `.1` … `.3`, checked while running.
const ROTATE_BYTES: u64 = 32 << 20;
const ROTATE_SLOTS: u32 = 4;

/// The summary row's keys, in order — also published in `validation/frame-schema.json` so the
/// reader checks them as strictly as the frame rows.
pub(crate) const SUMMARY_KEYS: &[&str] = &[
    "kind", "t_ms", "frame", "frames", "dt_mean", "dt_max", "slow", "dispatched", "live",
    "reproject", "hold", "readings", "discarded", "moved", "lethal", "refused", "events",
    "events_dropped", "stalls", "rec_us_max", "budget", "mode", "iter", "ref_len", "blind",
];

#[derive(Default)]
struct Summary {
    frames: u64,
    dt_sum: f64,
    dt_max: f64,
    slow: u64,
    dispatched: u64,
    present: [u64; 4],
    readings: u64,
    discarded: u64,
    moved: u64,
    lethal: u64,
    refused: u64,
    events: u64,
    events_dropped: u64,
    stalls: u64,
    rec_us_max: f32,
    last: FrameRecord,
}

/// Summary rows kept in memory for the issue report: one minute.
const RECENT_SUMMARIES: usize = 60;

/// The recording thread's half of `frames.jsonl`: what goes in each row, and when. The FILE is the
/// writer thread's ([`JsonlWriter`]).
#[derive(Default)]
struct Jsonl {
    /// The last [`RECENT_SUMMARIES`] summary rows, newest last.
    recent: std::collections::VecDeque<String>,
    sec_start_ms: Option<u64>,
    acc: Summary,
    severe_this_s: u32,
    routine_this_s: u32,
    last_mode: [Option<u8>; 2],
}

static JSONL: Mutex<Jsonl> = Mutex::new(Jsonl {
    recent: std::collections::VecDeque::new(),
    sec_start_ms: None,
    acc: Summary {
        frames: 0,
        dt_sum: 0.0,
        dt_max: 0.0,
        slow: 0,
        dispatched: 0,
        present: [0; 4],
        readings: 0,
        discarded: 0,
        moved: 0,
        lethal: 0,
        refused: 0,
        events: 0,
        events_dropped: 0,
        stalls: 0,
        rec_us_max: 0.0,
        last: FrameRecord::ZERO,
    },
    severe_this_s: 0,
    routine_this_s: 0,
    last_mode: [None; 2],
});

/// What the recording thread hands the `frames.jsonl` writer.
///
/// ⭐⭐**Why the file has its own thread.** The RX 6800 XT's beta.113 battery ran with its logs on a
/// network share, and its `--recordtest` failed on cost: p99 706 µs against a limit of 83. The
/// record attributed it to itself — `rec_us` is the previous emit's cost, and every one of the 68
/// emits over 300 µs followed a `frames.jsonl` flush (one a second for the summary, one per severe
/// event), at ~650 µs each: a LAN round trip, paid on the UI thread. `frames.bin` has to be written
/// where the frame is (an abort must find it on disk); this file does not, since `frames.bin` holds
/// every record anyway. So the row is decided here and the I/O is done there — including
/// formatting an event row, which is most of its cost on a local disk.
enum JsonlMsg {
    /// A full row for an event frame (formatted by the writer).
    Event(Box<FrameRecord>),
    /// A summary row, already composed (the issue report keeps a copy).
    Summary(String),
    /// Answer once everything sent before this is written and flushed.
    Flush(SyncSender<()>),
}

/// Rows the writer may fall behind by before the recording thread drops them: about three minutes
/// of the worst case (20 severe + 2 routine events + 1 summary a second). BOUNDED, so a writer stuck
/// on an unreachable share costs a counted loss rather than memory without limit — and never a
/// blocked frame.
const JSONL_QUEUE: usize = 4096;

/// The writer's queue. `None` inside: no logs directory, or the thread could not start — the same
/// "unavailable, not retried every frame" the file itself used to carry.
static JSONL_TX: OnceLock<Option<SyncSender<JsonlMsg>>> = OnceLock::new();
/// Rows dropped because the writer was [`JSONL_QUEUE`] behind.
static JSONL_DROPPED: AtomicU64 = AtomicU64::new(0);

fn jsonl_tx() -> Option<&'static SyncSender<JsonlMsg>> {
    JSONL_TX
        .get_or_init(|| {
            let path = jsonl_path(0)?;
            let (tx, rx) = std::sync::mpsc::sync_channel(JSONL_QUEUE);
            std::thread::Builder::new()
                .name("fd-frames-jsonl".into())
                .spawn(move || run_jsonl_writer(JsonlWriter::new(path), rx))
                .ok()?;
            Some(tx)
        })
        .as_ref()
}

/// Queue a row. Never blocks: a row the writer cannot take is dropped, counted, and said so once
/// in the log.
fn send_jsonl(m: JsonlMsg) -> bool {
    let Some(tx) = jsonl_tx() else { return false };
    match tx.try_send(m) {
        Ok(()) => true,
        Err(e) => {
            if JSONL_DROPPED.fetch_add(1, Ordering::Relaxed) == 0 {
                let why = match e {
                    std::sync::mpsc::TrySendError::Full(_) => "is falling behind (a slow or unreachable disk?)",
                    std::sync::mpsc::TrySendError::Disconnected(_) => "writer thread has stopped",
                };
                super::log_line(
                    "frames",
                    &format!(
                        "frames.jsonl {why} — dropping its rows; frames.bin and the in-memory record are unaffected"
                    ),
                );
            }
            false
        }
    }
}

/// The writer thread: write what arrives, then — once the queue is momentarily empty — hand the
/// batch to the OS. Off the recording thread a flush per batch costs nothing, and every row reaches
/// the OS within one batch (the routine rows used to wait in a buffer until the next second).
fn run_jsonl_writer(mut w: JsonlWriter, rx: Receiver<JsonlMsg>) {
    while let Ok(m) = rx.recv() {
        w.handle(m);
        while let Ok(m) = rx.try_recv() {
            w.handle(m);
        }
        w.flush();
    }
}

/// The file itself, owned by the writer thread.
struct JsonlWriter {
    path: PathBuf,
    /// `None` = not opened yet; `Some(Err)` = unavailable, not retried for every row.
    file: Option<Result<std::io::BufWriter<std::fs::File>, ()>>,
    bytes: u64,
}

impl JsonlWriter {
    fn new(path: PathBuf) -> Self {
        Self { path, file: None, bytes: 0 }
    }

    fn open(&mut self) {
        // A previous session's log is kept, not overwritten: rotate it out of the way.
        if self.path.exists() {
            rotate(&self.path);
        }
        self.file = Some(std::fs::File::create(&self.path).map(std::io::BufWriter::new).map_err(|_| ()));
        self.bytes = 0;
        let header = format!(
            "{{\"kind\":\"header\",\"schema\":{SCHEMA},\"session\":{},\"header\":{}}}",
            session_id(),
            header_json()
        );
        self.line(&header);
    }

    fn line(&mut self, s: &str) {
        if let Some(Ok(f)) = self.file.as_mut() {
            if writeln!(f, "{s}").is_ok() {
                self.bytes += s.len() as u64 + 1;
            }
        }
    }

    fn flush(&mut self) {
        if let Some(Ok(f)) = self.file.as_mut() {
            let _ = f.flush();
        }
    }

    fn handle(&mut self, m: JsonlMsg) {
        if self.file.is_none() {
            self.open();
        }
        match m {
            JsonlMsg::Event(r) => self.line(&r.to_json()),
            JsonlMsg::Summary(row) => {
                self.line(&row);
                if self.bytes > ROTATE_BYTES {
                    // Mid-run, at a second boundary: a long session must not overwrite its own
                    // head (the single-slot, startup-only rotation of fractadyne.log does exactly
                    // that). Closed before the rename.
                    self.flush();
                    self.file = None;
                    self.open();
                }
            }
            JsonlMsg::Flush(ack) => {
                self.flush();
                let _ = ack.send(());
            }
        }
    }
}

/// Wait up to `limit` for every `frames.jsonl` row queued so far to reach the OS. For a reader in
/// the same process (`--recordtest`), a clean shutdown and the panic hook. `false` if there is no
/// writer, its queue is full, or it did not answer in time.
pub(crate) fn flush_jsonl_within(limit: Duration) -> bool {
    let Some(tx) = JSONL_TX.get().and_then(Option::as_ref) else { return false };
    let (ack_tx, ack_rx) = std::sync::mpsc::sync_channel(1);
    if tx.try_send(JsonlMsg::Flush(ack_tx)).is_err() {
        return false;
    }
    ack_rx.recv_timeout(limit).is_ok()
}

/// [`flush_jsonl_within`] with room for a slow disk.
pub(crate) fn flush_jsonl() -> bool {
    flush_jsonl_within(Duration::from_secs(5))
}

/// Rows `frames.jsonl` lost because its writer fell behind (see [`JSONL_QUEUE`]).
pub(crate) fn jsonl_dropped() -> u64 {
    JSONL_DROPPED.load(Ordering::Relaxed)
}

/// `frames.jsonl`'s path, for a reader in the same process.
pub(crate) fn jsonl_file() -> Option<PathBuf> {
    jsonl_path(0)
}

/// Why a row is an event, if it is one. `(severe, reason)`.
pub(crate) fn event_reason(r: &FrameRecord, prev_mode: Option<u8>) -> Option<(bool, &'static str)> {
    if r.kind == kind::STALL {
        return Some((true, "stall"));
    }
    if r.read_n > 0 && r.read_lethal {
        return Some((true, "lethal"));
    }
    if r.repaint_requested && r.last_dt_ms > SLOW_MS {
        return Some((true, "slow"));
    }
    if r.refusal != 0 {
        return Some((false, "refused"));
    }
    if r.read_n > 0 && r.read_verdict == verdict::MOVED {
        return Some((false, "budget-moved"));
    }
    if r.plan_calls > 0 && prev_mode.is_some_and(|m| m != r.mode) {
        return Some((false, "mode-switch"));
    }
    None
}

fn jsonl_path(n: u32) -> Option<PathBuf> {
    super::logs_dir().map(|d| numbered(&d.join("frames.jsonl"), n))
}

/// `base` for 0, else `base.n`.
fn numbered(base: &Path, n: u32) -> PathBuf {
    if n == 0 {
        return base.to_path_buf();
    }
    let mut s = base.as_os_str().to_owned();
    s.push(format!(".{n}"));
    PathBuf::from(s)
}

/// Shift `base` → `.1` → … → `.3`, dropping the oldest.
fn rotate(base: &Path) {
    for n in (1..ROTATE_SLOTS).rev() {
        let _ = std::fs::rename(numbered(base, n - 1), numbered(base, n));
    }
}

impl Jsonl {
    fn take(&mut self, r: &FrameRecord) {
        if jsonl_tx().is_none() {
            return;
        }
        // Second boundary: write the summary for the second that just ended.
        let start = *self.sec_start_ms.get_or_insert(r.t_ms);
        if r.t_ms >= start + 1000 {
            self.flush_summary(r.t_ms);
        }
        // Accumulate.
        let v = (r.view as usize).min(1);
        let prev_mode = self.last_mode[v];
        let a = &mut self.acc;
        if r.kind == kind::STALL {
            a.stalls += 1;
        } else {
            if r.view == 0 {
                a.frames += 1;
                a.dt_sum += r.last_dt_ms;
                a.dt_max = a.dt_max.max(r.last_dt_ms);
                if r.repaint_requested && r.last_dt_ms > SLOW_MS {
                    a.slow += 1;
                }
                a.last = *r;
            }
            a.dispatched += r.dispatched as u64;
            a.present[(r.present as usize).min(3)] += 1;
            if r.read_n > 0 {
                a.readings += 1;
                a.discarded += (r.read_verdict == verdict::DISCARDED) as u64;
                a.moved += (r.read_verdict == verdict::MOVED) as u64;
                a.lethal += r.read_lethal as u64;
            }
            a.refused += (r.refusal != 0) as u64;
            a.rec_us_max = a.rec_us_max.max(r.rec_us);
            if r.plan_calls > 0 {
                self.last_mode[v] = Some(r.mode);
            }
        }
        // The event row, rate-limited by class.
        if let Some((severe, _why)) = event_reason(r, prev_mode) {
            let (count, cap) = if severe {
                (&mut self.severe_this_s, SEVERE_PER_S)
            } else {
                (&mut self.routine_this_s, ROUTINE_PER_S)
            };
            if *count < cap {
                *count += 1;
                // Events are the rows a death is most likely to follow; the writer hands each
                // batch to the OS as soon as it has written it.
                if send_jsonl(JsonlMsg::Event(Box::new(*r))) {
                    self.acc.events += 1;
                } else {
                    self.acc.events_dropped += 1;
                }
            } else {
                self.acc.events_dropped += 1;
            }
        }
    }

    fn flush_summary(&mut self, now_ms: u64) {
        let a = std::mem::take(&mut self.acc);
        let l = &a.last;
        let row = format!(
            "{{\"kind\":\"summary\",\"t_ms\":{},\"frame\":{},\"frames\":{},\"dt_mean\":{},\"dt_max\":{},\
             \"slow\":{},\"dispatched\":{},\"live\":{},\"reproject\":{},\"hold\":{},\"readings\":{},\
             \"discarded\":{},\"moved\":{},\"lethal\":{},\"refused\":{},\"events\":{},\"events_dropped\":{},\
             \"stalls\":{},\"rec_us_max\":{},\"budget\":{},\"mode\":{},\"iter\":{},\"ref_len\":{},\"blind\":{}}}",
            now_ms,
            l.frame,
            a.frames,
            if a.frames > 0 { a.dt_sum / a.frames as f64 } else { 0.0 },
            a.dt_max,
            a.slow,
            a.dispatched,
            a.present[present::LIVE as usize],
            a.present[present::REPROJECT as usize],
            a.present[present::HOLD as usize],
            a.readings,
            a.discarded,
            a.moved,
            a.lethal,
            a.refused,
            a.events,
            a.events_dropped,
            a.stalls,
            a.rec_us_max,
            l.fe_budget,
            l.mode,
            l.iter,
            l.ref_len,
            l.blind_slow_frames,
        );
        send_jsonl(JsonlMsg::Summary(row.clone()));
        if self.recent.len() == RECENT_SUMMARIES {
            self.recent.pop_front();
        }
        self.recent.push_back(row);
        self.sec_start_ms = Some(now_ms);
        self.severe_this_s = 0;
        self.routine_this_s = 0;
    }
}

/// Every record held, oldest first.
///
/// ⚠Never a blocking `lock()`: this is read from the panic hook, which may be running on the very
/// thread that holds the lock (a panic inside [`record`]), where blocking would deadlock the crash
/// report itself. But not a single `try_lock` either — a device-lost callback on another thread can
/// land in the few microseconds the frame loop holds it, and the device-loss report is the one
/// that most needs its frames. So: retry briefly, then give up and say so (an empty section reads
/// "no frame was recorded", which a reader then weighs against the log).
pub(crate) fn snapshot() -> Vec<FrameRecord> {
    for _ in 0..50 {
        match RING.try_lock() {
            Ok(g) => return ordered(&g.buf, g.next),
            Err(std::sync::TryLockError::Poisoned(p)) => {
                let g = p.into_inner();
                return ordered(&g.buf, g.next);
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                std::thread::sleep(std::time::Duration::from_millis(1))
            }
        }
    }
    Vec::new()
}

/// The records with `seq >= from`, oldest first, the `seq` to ask for next time, and how many of
/// the requested records the ring had already overwritten. For a harness that must judge a WHOLE
/// run: a verdict read from [`snapshot`] at the end sees only the last [`RING_LEN`] records, and
/// W9's escaped-reference rung lost its storm that way (frames 12–19 of an 8,220-frame soak on the
/// RX 6800 XT, 2026-09-23, reported "never entered"). Blocking lock: harness-only, UI thread.
pub(crate) fn since(from: u64) -> (Vec<FrameRecord>, u64, u64) {
    let g = RING.lock().unwrap_or_else(|p| p.into_inner());
    let next = g.next;
    if g.buf.is_empty() || next <= from {
        return (Vec::new(), next.max(from), 0);
    }
    let oldest = next.saturating_sub(RING_LEN as u64);
    let start = from.max(oldest);
    let recs = (start..next).map(|s| g.buf[(s % RING_LEN as u64) as usize]).collect();
    (recs, next, start - from)
}

fn ordered(buf: &[FrameRecord], next: u64) -> Vec<FrameRecord> {
    if buf.is_empty() || next == 0 {
        return Vec::new();
    }
    let n = (next as usize).min(RING_LEN);
    (0..n)
        .map(|k| {
            let seq = next - n as u64 + k as u64;
            buf[(seq % RING_LEN as u64) as usize]
        })
        .collect()
}

/// Write records as JSON Lines: the header object first (tagged `"kind":"header"`), then one
/// record per line. Used for the crash companion file; the same shape `frames.jsonl` will use.
pub(crate) fn write_jsonl(path: &Path, header: &str, records: &[FrameRecord]) -> std::io::Result<()> {
    let header = if header.trim().is_empty() { "{}" } else { header };
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(
        f,
        "{{\"kind\":\"header\",\"schema\":{SCHEMA},\"session\":{},\"header\":{header}}}",
        session_id()
    )?;
    for r in records {
        writeln!(f, "{}", r.to_json())?;
    }
    f.flush()
}

/// A `frames.bin` read back: its session header and every valid record, oldest first.
#[derive(Debug, Default)]
pub(crate) struct BinFile {
    pub(crate) session: u64,
    pub(crate) header: String,
    pub(crate) records: Vec<FrameRecord>,
    /// Slots that carried this session's magic but failed the checksum or the decode — a torn
    /// write. Reported, never silently dropped: a count here says the tail was interrupted.
    pub(crate) torn: usize,
}

/// Read a `frames.bin`. `Ok(None)` for a file of another schema (refused, not guessed at).
pub(crate) fn read_bin(path: &Path) -> std::io::Result<Option<BinFile>> {
    let mut f = std::fs::File::open(path)?;
    let mut data = Vec::new();
    f.read_to_end(&mut data)?;
    if data.len() < HEADER_BYTES || data[0..4] != FILE_MAGIC {
        return Ok(None);
    }
    let schema = u16::from_le_bytes([data[4], data[5]]);
    if schema != SCHEMA {
        return Ok(None);
    }
    let session = u64::from_le_bytes(data[8..16].try_into().unwrap_or_default());
    let hlen = u32::from_le_bytes(data[16..20].try_into().unwrap_or_default()) as usize;
    let header = String::from_utf8_lossy(&data[20..20 + hlen.min(HEADER_BYTES - 20)]).into_owned();
    let mut out = BinFile { session, header, ..Default::default() };
    for slot in data[HEADER_BYTES..].chunks_exact(SLOT_BYTES) {
        if slot[0..4] != SLOT_MAGIC {
            continue; // never written this session
        }
        match decode_slot(slot, session) {
            Some(r) => out.records.push(r),
            None if slot[8..16] == session.to_le_bytes() => out.torn += 1,
            None => {} // another session's leftover slot
        }
    }
    out.records.sort_by_key(|r| r.seq);
    Ok(Some(out))
}

/// Read the PREVIOUS session's `frames.bin`, if one is there. Called from the unclean-exit
/// report, which runs before this session records anything (and so before the file is truncated).
pub(crate) fn previous_session_frames() -> Option<BinFile> {
    read_bin(&bin_path()?).ok().flatten().filter(|b| !b.records.is_empty())
}

// ------------------------------------------------------------------------------------------------
// The crash report's view.

/// One record as a compact line for a human reading a crash report.
pub(crate) fn brief(r: &FrameRecord) -> String {
    use std::fmt::Write as _;
    let mut s = format!(
        "f={} v{} +{:.3}s dt={:.0}ms body={:.0}ms",
        r.frame,
        r.view,
        r.t_ms as f64 / 1000.0,
        r.last_dt_ms,
        r.body_ms
    );
    if r.plan_calls == 0 {
        s.push_str(" (view not built)");
    } else {
        let _ = write!(
            s,
            " {} mode={} {}x{} ss={} iter={} ref={}{} steps={:.3e} budget={:.3e}",
            match r.present {
                present::LIVE => "live",
                present::REPROJECT => "reproject",
                present::HOLD => "hold",
                _ => "present=?",
            },
            r.mode,
            r.res_w,
            r.res_h,
            r.ss,
            r.iter,
            r.ref_len,
            if r.ref_partial { "(partial)" } else { "" },
            r.steps as f64,
            r.fe_budget as f64,
        );
        if r.chunked {
            let _ = write!(s, " chunk=[{},{})", r.chunk_lo, r.chunk_hi);
        }
        if !r.dispatched {
            s.push_str(" no-dispatch");
        }
    }
    if r.read_n > 0 {
        let _ = write!(
            s,
            " read={}:{:.1}ms/{:.3e} {}",
            match r.read_src {
                read_src::GPU => "gpu",
                read_src::WALL => "wall",
                _ => "?",
            },
            r.read_ms,
            r.read_steps as f64,
            match r.read_verdict {
                verdict::DISCARDED => "DISCARDED",
                verdict::MOVED => "moved",
                verdict::UNCHANGED => "unchanged",
                _ => "?",
            }
        );
        if r.refusal == refusal::BUILDING {
            s.push_str(" (growth refused: building)");
        }
    }
    if r.ctr_new {
        let _ = write!(
            s,
            " ctr[tag {}]: rebase={} bla_skip={} esc={}/{}",
            r.ctr_tag, r.ctr_rebase, r.ctr_bla_skip, r.ctr_escaped, r.ctr_px
        );
    }
    if r.blind_slow_frames > 0 {
        let _ = write!(s, " blind={}/{}", r.blind_slow_frames, r.blind_slow_readings);
    }
    s
}

/// The crash report's `frames:` section for `records` (oldest first), naming the companion file
/// the whole set was written to, if any.
pub(crate) fn crash_section(records: &[FrameRecord], companion: Option<&str>) -> String {
    let Some((first, last)) = records.first().zip(records.last()) else {
        return "frames  : no frame was recorded before the crash\n".to_string();
    };
    let mut s = format!(
        "frames  : {} records (seq {}..{}), frames {}..{}, {:.1}s of history{}\n\
         \x20         the last {} (oldest first):\n",
        records.len(),
        first.seq,
        last.seq,
        first.frame,
        last.frame,
        (last.t_ms.saturating_sub(first.t_ms)) as f64 / 1000.0,
        companion.map_or(String::new(), |c| format!("; all of them in {c}")),
        records.len().min(CRASH_INLINE),
    );
    for r in &records[records.len().saturating_sub(CRASH_INLINE)..] {
        s.push_str("          ");
        s.push_str(&brief(r));
        s.push('\n');
    }
    s
}

/// The header JSON, for callers composing a companion file.
pub(crate) fn header() -> String {
    header_json()
}

/// The issue report's "Frame record" section: the session header, the last minute of summary rows
/// and the last [`CRASH_INLINE`] frames. `None` when nothing has been recorded. The caller redacts
/// the whole report; nothing here names the user, but a future field might.
pub(crate) fn report_section() -> Option<String> {
    let recs = snapshot();
    if recs.is_empty() {
        return None;
    }
    let mut s = format!("== Frame record (session {:016x}) ==\nheader: {}\n", session_id(), header_json());
    if let Ok(j) = JSONL.lock() {
        if !j.recent.is_empty() {
            s.push_str(&format!("the last {} per-second summaries (oldest first):\n", j.recent.len()));
            for row in &j.recent {
                s.push_str("  ");
                s.push_str(row);
                s.push('\n');
            }
        }
    }
    s.push_str(&crash_section(&recs, None));
    Some(s)
}

/// The encoding as data, for readers in other languages: `--dump-frame-schema` prints it, the
/// committed `validation/frame-schema.json` is it (pinned by test, as `TOURS.md` is by the tour
/// schema), and `scripts/framelog.py` decodes `frames.bin` and checks a `.jsonl` against it —
/// strictly: an unknown or a missing key is an error, never a skipped field.
pub(crate) fn schema_json() -> String {
    let fields: Vec<String> = FrameRecord::FIELDS
        .iter()
        .map(|(n, t)| format!("    [\"{n}\", \"{t}\"]"))
        .collect();
    format!(
        "{{\n  \"schema\": {SCHEMA},\n  \"slot_bytes\": {SLOT_BYTES},\n  \"slot_header_bytes\": {SLOT_HEADER},\n  \
         \"header_bytes\": {HEADER_BYTES},\n  \"ring_len\": {RING_LEN},\n  \"payload_bytes\": {},\n  \
         \"slot_magic\": \"FDFR\",\n  \"file_magic\": \"FDFH\",\n  \"fields\": [\n{}\n  ],\n  \
         \"summary_keys\": [{}]\n}}\n",
        FrameRecord::PAYLOAD_BYTES,
        fields.join(",\n"),
        SUMMARY_KEYS.iter().map(|k| format!("\"{k}\"")).collect::<Vec<_>>().join(", ")
    )
}

#[cfg(test)]
#[path = "frame_record_tests.rs"]
mod tests;
