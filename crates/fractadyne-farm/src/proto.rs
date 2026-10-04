//! The protocol: every message either end may send, and nothing else (design §4).
//!
//! The list IS the authorization model — a client can be asked to do exactly what a message here
//! describes. Every message is size-capped and every field checked on arrival ([`decode`]); one
//! that fails is a protocol error and the connection closes. Both ends run the same commit (the
//! version gate), so unknown fields are refused rather than ignored.
//!
//! On the wire, inside a Noise transport message, a payload is one kind byte and then either a
//! control message (`KIND_CONTROL`, JSON) or a chunk of a blob (`KIND_CHUNK`: id, offset, bytes).
//! Blobs — the job bundle, frames, the link-speed sample — are announced by a control message with
//! their length and SHA-256, and are complete only when the digest matches.
//!
//! Orbit sharing (design §8, protocol 6): a client offers the controller each reference orbit its
//! renders built (`OrbitOffer`), and the controller pushes its verified pool to every machine on the
//! job (`OrbitPush`). There is no query: a render finds an admissible orbit in its own cache.

use serde::{Deserialize, Serialize};

/// Largest control message, encoded.
pub const MAX_CONTROL_BYTES: usize = 64 * 1024;
/// Largest frame a client may send.
pub const MAX_FRAME_BYTES: u64 = 256 << 20;
/// Largest job bundle (script, settings, anchors).
pub const MAX_BUNDLE_BYTES: u64 = 16 << 20;
/// Largest link-speed sample.
pub const MAX_LINK_SAMPLE_BYTES: u64 = 4 << 20;
/// Largest probe image (the self-check render, a small PNG).
pub const MAX_PROBE_BYTES: u64 = 4 << 20;
/// Largest reference orbit sent between machines (design §8: 16 bytes an iteration, so 16 M
/// iterations; larger ones are rebuilt where they are needed).
pub const MAX_ORBIT_BYTES: u64 = 256 << 20;
/// What a frame's `reference` may say: picked and built here, from the orbit cache (which holds
/// the farm's shared orbits), extended from one in memory, or none new (the previous one served).
pub const REFERENCE_SOURCES: [&str; 4] = ["fresh", "cache", "reused", "none"];
/// Data bytes per chunk: a Noise message holds 65,535 bytes including its 16-byte tag, and a chunk
/// carries 17 bytes of header.
pub const CHUNK_BYTES: usize = 60 * 1024;
/// Most frames one run may cover.
pub const MAX_RUN_FRAMES: u64 = 10_000;
/// Most frames a job may have.
pub const MAX_JOB_FRAMES: u64 = 10_000_000;

pub const KIND_CONTROL: u8 = 1;
pub const KIND_CHUNK: u8 = 2;

/// Every message.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "kind", content = "body", rename_all = "snake_case", deny_unknown_fields)]
pub enum Msg {
    /// Client → controller, first message of a connection.
    Hello(Hello),
    /// Controller → client: admitted, waiting for approval, or refused (and why).
    HelloAck(HelloAck),
    /// Client → controller: the handshake self-check's results (design §9.1).
    SelfCheck(SelfCheck),
    /// Controller → client: a job; its bundle follows as a blob.
    JobOpen(JobOpen),
    /// Controller → client: render frames `[start, end)` of a job.
    Assign(Assign),
    /// Controller → client: stop a run (or every run) of a job.
    Cancel(Cancel),
    /// Controller → client: the job is over; delete its files.
    JobClose(JobClose),
    /// Client → controller, every 2 s.
    Heartbeat(Heartbeat),
    /// Client → controller: a frame is finished; its file follows as a blob.
    FrameDone(FrameDone),
    /// Client → controller: a frame could not be produced.
    FrameFailed(FrameFailed),
    /// Client → controller: a run stopped early, and how far it got.
    RunAborted(RunAborted),
    /// Controller → client: send these diagnostics.
    DiagRequest(DiagRequest),
    /// Client → controller: the diagnostics asked for, each capped.
    DiagReport(DiagReport),
    /// Either way: the sender is closing the connection on purpose, and why.
    Bye(Bye),
    /// Client → controller: a reference orbit its renders built and cached; the blob follows.
    OrbitOffer(OrbitBlob),
    /// Controller → client: a reference orbit from the farm's pool, for its renders' cache; the
    /// blob follows.
    OrbitPush(OrbitBlob),
    /// Controller → client, every few seconds, so a client can tell a quiet controller from a gone
    /// one (its read times out when these stop).
    Keepalive,
}

/// A blob about to be sent: its id on this connection, length and SHA-256 (lower-case hex).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BlobAnnounce {
    pub id: u64,
    pub len: u64,
    pub sha256: String,
}

/// What a client will accept, set by its user. A request outside it is refused, not clamped.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub max_width: u32,
    pub max_height: u32,
    pub max_ss: u32,
    pub max_iter: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    pub protocol: u32,
    pub app_version: String,
    pub git: String,
    /// The client was started with `--farm-allow-dirty` (development only).
    pub allow_dirty: bool,
    pub name: String,
    /// The tunables status line (`stock`, or the overrides in force).
    pub tunables: String,
    pub policy: Policy,
    /// The client's clock, for log alignment only.
    pub clock_unix_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "verdict", content = "reason", rename_all = "snake_case", deny_unknown_fields)]
pub enum Verdict {
    Admitted,
    WaitingForApproval,
    Refused(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HelloAck {
    pub protocol: u32,
    pub app_version: String,
    pub git: String,
    pub name: String,
    pub verdict: Verdict,
    /// Bytes of link-speed sample to send with the self-check (0 = none).
    pub link_sample_bytes: u64,
    /// The controller's own GPU, when it knows it: the client says when its own differs.
    pub gpu: Option<GpuInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CheckItem {
    pub name: String,
    pub ok: bool,
    /// A failed hard item refuses admission.
    pub hard: bool,
    pub detail: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GpuInfo {
    /// "NVIDIA GeForce RTX 3080 · Vulkan": the adapter and the graphics API it is driven through.
    pub adapter: String,
    /// "NVIDIA 581.42": the driver and its version, as the graphics API reports them ("" unknown).
    pub driver: String,
    /// The reference-orbit length cap this GPU allows (samples).
    pub orbit_len_cap: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SelfCheck {
    pub items: Vec<CheckItem>,
    pub gpu: Option<GpuInfo>,
    pub free_bytes: Option<u64>,
    pub link_sample: Option<BlobAnnounce>,
    /// The self-check render as a PNG: the controller compares it with its own render of the same
    /// view, pixel by pixel, to tell GPUs whose pictures differ (design §9).
    pub probe: Option<BlobAnnounce>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct JobOpen {
    pub job_id: String,
    pub name: String,
    pub bundle: BlobAnnounce,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Assign {
    pub job_id: String,
    pub run_id: u64,
    pub start: u64,
    pub end: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    /// Its frames went to another client (stealing, or the run was cut short).
    Reassigned,
    /// No frame finished within the stall timeout.
    Stalled,
    /// A frame passed the hard deadline.
    Deadline,
    /// The job was paused.
    Paused,
    /// The job was stopped.
    Stopped,
    /// The client was removed from the farm.
    Removed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Cancel {
    pub job_id: String,
    /// `None` = every run of the job.
    pub run_id: Option<u64>,
    pub reason: CancelReason,
    /// Cut the run here rather than end it: the frames from `from` on went to another client.
    /// The client finishes the frames before it, then ends the run.
    pub from: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JobClose {
    pub job_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientActivity {
    Idle,
    Rendering,
    Paused,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Heartbeat {
    pub activity: ClientActivity,
    pub job_id: Option<String>,
    pub run_id: Option<u64>,
    /// The frame being rendered, and how long it has been.
    pub frame: Option<u64>,
    pub frame_ms: Option<u64>,
    pub frames_done: u64,
    pub free_bytes: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FrameDone {
    pub job_id: String,
    pub run_id: u64,
    pub index: u64,
    pub render_ms: u64,
    pub blob: BlobAnnounce,
    /// The frame is in the client's folder on the shared drive (`names::share_dir`), not streamed:
    /// no chunks follow; the controller reads and checks it there.
    pub on_share: bool,
    /// Where its reference came from (one of [`REFERENCE_SOURCES`]), when the render said.
    pub reference: Option<String>,
}

/// A reference orbit on its way (design §8). Its file name is not sent: the receiver names the
/// file from the orbit's own verified header.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OrbitBlob {
    pub job_id: String,
    pub blob: BlobAnnounce,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailClass {
    Storage,
    ReadBack,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FrameFailed {
    pub job_id: String,
    pub run_id: u64,
    pub index: u64,
    pub class: FailClass,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "why", content = "detail", rename_all = "snake_case", deny_unknown_fields)]
pub enum AbortReason {
    /// The client's user cancelled the frame in progress.
    UserCancel,
    /// The client's user paused: the current frame finished, no more work is taken.
    Paused,
    /// The controller cancelled it.
    Canceled,
    /// The render process lost its GPU.
    DeviceLost,
    /// The render process exited with this code.
    ChildCrash(i32),
    /// The job asks for more than the client's policy allows.
    Policy(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RunAborted {
    pub job_id: String,
    pub run_id: u64,
    /// The last frame of the run whose `FrameDone` was sent (`None` = none).
    pub done_up_to: Option<u64>,
    pub reason: AbortReason,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DiagItem {
    /// The tail of the current (or last) render process's log.
    ChildLog,
    /// The handshake and self-check record.
    Handshake,
    /// The client's recent heartbeats, as it sent them.
    Heartbeats,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DiagRequest {
    pub items: Vec<DiagItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DiagReport {
    pub items: Vec<(DiagItem, String)>,
}

/// Most bytes of text one diagnostics item may carry.
pub const MAX_DIAG_ITEM: usize = 16 * 1024;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bye {
    pub reason: String,
}

/// What arrives in one transport message.
#[derive(Debug, PartialEq)]
pub enum Incoming {
    Control(Msg),
    Chunk { id: u64, offset: u64, data: Vec<u8> },
}

/// Encode a control message, kind byte first. Fails (rather than sending) above the cap — the
/// sender must have truncated what it controls, and the receiver would refuse it anyway.
pub fn encode_control(m: &Msg) -> Result<Vec<u8>, String> {
    let mut out = vec![KIND_CONTROL];
    serde_json::to_writer(&mut out, m).map_err(|e| format!("encode: {e}"))?;
    if out.len() > MAX_CONTROL_BYTES + 1 {
        return Err(format!("a {} message is {} bytes, over the {MAX_CONTROL_BYTES}-byte cap", kind_name(m), out.len()));
    }
    Ok(out)
}

/// Encode one chunk of a blob.
pub fn encode_chunk(id: u64, offset: u64, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(17 + data.len());
    out.push(KIND_CHUNK);
    out.extend_from_slice(&id.to_be_bytes());
    out.extend_from_slice(&offset.to_be_bytes());
    out.extend_from_slice(data);
    out
}

/// Decode and CHECK one payload. Any failure is a protocol error.
pub fn decode(payload: &[u8]) -> Result<Incoming, String> {
    match payload.first() {
        Some(&KIND_CONTROL) => {
            let body = &payload[1..];
            if body.len() > MAX_CONTROL_BYTES {
                return Err(format!("control message of {} bytes is over the cap", body.len()));
            }
            let m: Msg = serde_json::from_slice(body).map_err(|e| format!("malformed message: {e}"))?;
            validate(&m)?;
            Ok(Incoming::Control(m))
        }
        Some(&KIND_CHUNK) => {
            if payload.len() < 17 || payload.len() > 17 + CHUNK_BYTES {
                return Err(format!("chunk of {} bytes has the wrong size", payload.len()));
            }
            let id = u64::from_be_bytes(payload[1..9].try_into().unwrap_or_default());
            let offset = u64::from_be_bytes(payload[9..17].try_into().unwrap_or_default());
            Ok(Incoming::Chunk { id, offset, data: payload[17..].to_vec() })
        }
        Some(k) => Err(format!("unknown payload kind {k}")),
        None => Err("empty payload".into()),
    }
}

pub fn kind_name(m: &Msg) -> &'static str {
    match m {
        Msg::Hello(_) => "Hello",
        Msg::HelloAck(_) => "HelloAck",
        Msg::SelfCheck(_) => "SelfCheck",
        Msg::JobOpen(_) => "JobOpen",
        Msg::Assign(_) => "Assign",
        Msg::Cancel(_) => "Cancel",
        Msg::JobClose(_) => "JobClose",
        Msg::Heartbeat(_) => "Heartbeat",
        Msg::FrameDone(_) => "FrameDone",
        Msg::FrameFailed(_) => "FrameFailed",
        Msg::RunAborted(_) => "RunAborted",
        Msg::DiagRequest(_) => "DiagRequest",
        Msg::DiagReport(_) => "DiagReport",
        Msg::Bye(_) => "Bye",
        Msg::OrbitOffer(_) => "OrbitOffer",
        Msg::OrbitPush(_) => "OrbitPush",
        Msg::Keepalive => "Keepalive",
    }
}

fn text(what: &str, s: &str, max: usize) -> Result<(), String> {
    if s.len() > max {
        return Err(format!("{what} is {} bytes, over {max}", s.len()));
    }
    if s.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err(format!("{what} contains control characters"));
    }
    Ok(())
}

fn digest(what: &str, s: &str) -> Result<(), String> {
    if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(format!("{what} is not a SHA-256 in lower-case hex"));
    }
    Ok(())
}

fn blob(what: &str, b: &BlobAnnounce, max: u64) -> Result<(), String> {
    if b.len > max {
        return Err(format!("{what} of {} bytes is over the {max}-byte cap", b.len));
    }
    digest(what, &b.sha256)
}

fn job(id: &str) -> Result<(), String> {
    crate::names::check_file_part("job id", id)
}

/// Every field of every message, checked. Pure, and the only gate between the network and the app.
pub fn validate(m: &Msg) -> Result<(), String> {
    match m {
        Msg::Hello(h) => {
            text("app version", &h.app_version, 64)?;
            text("git", &h.git, 64)?;
            crate::names::check_display_name(&h.name)?;
            text("tunables", &h.tunables, 1024)?;
            let p = &h.policy;
            if p.max_width == 0 || p.max_height == 0 || p.max_width > 16384 || p.max_height > 16384 {
                return Err("policy size out of range".into());
            }
            if !(1..=8).contains(&p.max_ss) || p.max_iter == 0 {
                return Err("policy sampling or iterations out of range".into());
            }
        }
        Msg::HelloAck(a) => {
            text("app version", &a.app_version, 64)?;
            text("git", &a.git, 64)?;
            crate::names::check_display_name(&a.name)?;
            if let Verdict::Refused(r) = &a.verdict {
                text("refusal", r, 1024)?;
            }
            if a.link_sample_bytes > MAX_LINK_SAMPLE_BYTES {
                return Err("link sample too large".into());
            }
            if let Some(g) = &a.gpu {
                text("adapter", &g.adapter, 256)?;
                text("driver", &g.driver, 256)?;
            }
        }
        Msg::SelfCheck(s) => {
            if s.items.len() > 32 {
                return Err("too many self-check items".into());
            }
            for i in &s.items {
                text("check name", &i.name, 64)?;
                text("check detail", &i.detail, 1024)?;
            }
            if let Some(g) = &s.gpu {
                text("adapter", &g.adapter, 256)?;
                text("driver", &g.driver, 256)?;
            }
            if let Some(b) = &s.link_sample {
                blob("link sample", b, MAX_LINK_SAMPLE_BYTES)?;
            }
            if let Some(b) = &s.probe {
                blob("probe", b, MAX_PROBE_BYTES)?;
                if s.link_sample.as_ref().is_some_and(|l| l.id == b.id) {
                    return Err("the probe and the link sample share a blob id".into());
                }
            }
        }
        Msg::JobOpen(j) => {
            job(&j.job_id)?;
            text("job name", &j.name, 256)?;
            blob("bundle", &j.bundle, MAX_BUNDLE_BYTES)?;
        }
        Msg::Assign(a) => {
            job(&a.job_id)?;
            if a.end <= a.start || a.end - a.start > MAX_RUN_FRAMES || a.end > MAX_JOB_FRAMES {
                return Err(format!("run [{}, {}) is out of range", a.start, a.end));
            }
        }
        Msg::OrbitOffer(o) | Msg::OrbitPush(o) => {
            job(&o.job_id)?;
            blob("orbit", &o.blob, MAX_ORBIT_BYTES)?;
        }
        Msg::Cancel(c) => {
            job(&c.job_id)?;
            if c.from.is_some() && c.run_id.is_none() {
                return Err("a cut names no run".into());
            }
        }
        Msg::JobClose(c) => job(&c.job_id)?,
        Msg::Heartbeat(h) => {
            if let Some(j) = &h.job_id {
                job(j)?;
            }
        }
        Msg::FrameDone(f) => {
            job(&f.job_id)?;
            if f.index >= MAX_JOB_FRAMES {
                return Err("frame index out of range".into());
            }
            blob("frame", &f.blob, MAX_FRAME_BYTES)?;
            if f.reference.as_deref().is_some_and(|r| !REFERENCE_SOURCES.contains(&r)) {
                return Err("an unknown reference source".into());
            }
        }
        Msg::FrameFailed(f) => {
            job(&f.job_id)?;
            text("failure message", &f.message, 1024)?;
        }
        Msg::RunAborted(r) => {
            job(&r.job_id)?;
            if let AbortReason::Policy(w) = &r.reason {
                text("policy", w, 512)?;
            }
        }
        Msg::DiagRequest(d) => {
            if d.items.is_empty() || d.items.len() > 3 {
                return Err("a diagnostics request names 1 to 3 items".into());
            }
        }
        Msg::DiagReport(d) => {
            if d.items.len() > 3 {
                return Err("too many diagnostics items".into());
            }
            for (_, t) in &d.items {
                if t.len() > MAX_DIAG_ITEM {
                    return Err("a diagnostics item is over its cap".into());
                }
            }
        }
        Msg::Bye(b) => text("reason", &b.reason, 1024)?,
        Msg::Keepalive => {}
    }
    Ok(())
}

/// Truncate `s` to at most `max` bytes on a character boundary, keeping the END (a log's tail is
/// what explains a failure).
pub fn tail(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut start = s.len() - max;
    while !s.is_char_boundary(start) {
        start += 1;
    }
    s[start..].to_string()
}

/// Receives one announced blob chunk by chunk, hashing as it goes. Chunks must arrive in order and
/// exactly fill the announced length; anything else is a protocol error, and a digest mismatch is
/// reported as such (for a frame: a bad frame, which is a strike).
pub struct BlobSink<W: std::io::Write> {
    pub announce: BlobAnnounce,
    received: u64,
    hash: crate::Sha256,
    out: W,
}

/// How a chunk left a blob.
#[derive(Debug, PartialEq, Eq)]
pub enum BlobProgress {
    More,
    /// Complete, and the digest matched.
    Complete,
}

/// Why a blob was not accepted. The two are handled differently: a protocol violation closes the
/// connection; a digest mismatch on a frame is a BAD FRAME — re-queued elsewhere and a strike.
#[derive(Debug, PartialEq, Eq)]
pub enum BlobError {
    /// Out of order, past its announced length, empty, or the sink failed.
    Protocol(String),
    /// Every byte arrived, and they are not the bytes announced.
    Digest(String),
}

impl std::fmt::Display for BlobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlobError::Protocol(e) | BlobError::Digest(e) => f.write_str(e),
        }
    }
}

impl<W: std::io::Write> BlobSink<W> {
    pub fn new(announce: BlobAnnounce, out: W) -> Self {
        Self { announce, received: 0, hash: crate::Sha256::default(), out }
    }

    pub fn received(&self) -> u64 {
        self.received
    }

    /// Feed one chunk.
    pub fn push(&mut self, offset: u64, data: &[u8]) -> Result<BlobProgress, BlobError> {
        let id = self.announce.id;
        if offset != self.received {
            return Err(BlobError::Protocol(format!("blob {id}: chunk at {offset}, expected {}", self.received)));
        }
        let end = self.received + data.len() as u64;
        if end > self.announce.len {
            return Err(BlobError::Protocol(format!("blob {id}: {end} bytes is past its announced {}", self.announce.len)));
        }
        if data.is_empty() && self.announce.len != 0 {
            return Err(BlobError::Protocol(format!("blob {id}: empty chunk")));
        }
        self.out.write_all(data).map_err(|e| BlobError::Protocol(format!("blob {id}: {e}")))?;
        self.hash.update(data);
        self.received = end;
        if self.received < self.announce.len {
            return Ok(BlobProgress::More);
        }
        self.out.flush().map_err(|e| BlobError::Protocol(format!("blob {id}: {e}")))?;
        let got = std::mem::take(&mut self.hash).finish_hex();
        if got != self.announce.sha256 {
            return Err(BlobError::Digest(format!("SHA-256 {got} does not match the announced {}", self.announce.sha256)));
        }
        Ok(BlobProgress::Complete)
    }

    pub fn into_inner(self) -> W {
        self.out
    }
}

/// Split `bytes` into the chunks that carry it.
pub fn chunks(bytes: &[u8]) -> impl Iterator<Item = (u64, &[u8])> {
    bytes.chunks(CHUNK_BYTES).enumerate().map(|(i, c)| ((i * CHUNK_BYTES) as u64, c))
}

#[cfg(test)]
mod tests;
