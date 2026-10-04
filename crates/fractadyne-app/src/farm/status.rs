//! The farm modes' machine-readable side (`--ui-status`): what the app's farm windows read and how
//! they steer the process they started.
//!
//! With `--ui-status`, a controller or client prints one `farm-status {json}` line a second (and on
//! every change of phase) beside its ordinary human lines, and reads commands, one per line, from
//! its stdin. The window starts the process with stdin piped, so when the window goes — closed, or
//! the app crashed — stdin ends and the process stops (a controller) or leaves (a client) rather
//! than running on with no one watching. Without the flag nothing here happens: a farm started from
//! a terminal or by the field agent never reads its stdin.
//!
//! One process per window, the tested headless path, rather than threads in the app: a lost GPU or
//! a crash in the farm takes down that process, never the app (as `ui/tour_render.rs` reasons).

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::mpsc;

/// What a status line starts with.
pub(crate) const PREFIX: &str = "farm-status ";

/// The flag that turns this on.
pub(crate) const FLAG: &str = "--ui-status";

pub(crate) fn line<T: Serialize>(s: &T) -> String {
    format!("{PREFIX}{}", serde_json::to_string(s).unwrap_or_else(|_| "{}".into()))
}

/// A status line's payload, or `None` for any other line.
pub(crate) fn parse<T: DeserializeOwned>(l: &str) -> Option<T> {
    serde_json::from_str(l.trim_end().strip_prefix(PREFIX)?).ok()
}

/// Read commands from stdin on a thread: each trimmed, non-empty line as sent, then `None` once
/// stdin ends.
pub(crate) fn commands() -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        use std::io::BufRead;
        for l in std::io::stdin().lock().lines() {
            let Ok(l) = l else { break };
            let l = l.trim().to_string();
            if !l.is_empty() && tx.send(Some(l)).is_err() {
                return;
            }
        }
        let _ = tx.send(None);
    });
    rx
}

// --- the client --------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ClientPhase {
    #[default]
    Connecting,
    /// Connected; the self-check (a probe render) is running.
    Checking,
    /// Connected and admitted, no work.
    Idle,
    Rendering,
    /// Paused by this machine's user: finishing a frame, or holding.
    Paused,
    /// The connection is down; redialling.
    Retrying,
    /// Over: refused, removed, the controller said goodbye, or the user left.
    Ended,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub(crate) struct ClientStatus {
    pub(crate) phase: ClientPhase,
    /// The sentence the window shows, in the client's words ("Rendering frames 120–135 …").
    pub(crate) detail: String,
    pub(crate) controller: String,
    pub(crate) controller_name: String,
    pub(crate) controller_fingerprint: String,
    pub(crate) identity: String,
    pub(crate) name: String,
    pub(crate) job: Option<String>,
    /// "640×360 ss1 at 3 fps, 19 frames".
    pub(crate) job_detail: Option<String>,
    pub(crate) run: Option<(u64, u64)>,
    pub(crate) frame: Option<u64>,
    pub(crate) frame_ms: Option<u64>,
    pub(crate) frames_done: u64,
    pub(crate) mean_ms: Option<f64>,
    /// A copy of the last frame sent, for the window's thumbnail; `last_frame_seq` changes with it.
    pub(crate) last_frame: Option<String>,
    pub(crate) last_frame_seq: u64,
    /// The self-check's lines ("ok   render: test frame in 707 ms on …").
    pub(crate) self_check: Vec<String>,
    pub(crate) paused: bool,
    pub(crate) retry_in_s: Option<u64>,
    /// Set when `phase` is `Ended`: the process is about to exit with this code.
    pub(crate) exit_code: Option<i32>,
}

/// Commands a client takes on stdin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClientCommand {
    /// Finish the frame in progress, then take no work.
    Pause,
    Resume,
    /// Stop the frame in progress now, and pause.
    CancelFrame,
    /// Disconnect and exit.
    Leave,
}

impl ClientCommand {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "pause" => Self::Pause,
            "resume" => Self::Resume,
            "cancel-frame" => Self::CancelFrame,
            "leave" => Self::Leave,
            _ => return None,
        })
    }

    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::CancelFrame => "cancel-frame",
            Self::Leave => "leave",
        }
    }
}

// --- the controller ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ControllerPhase {
    /// Listening; fewer clients admitted than the job waits for.
    #[default]
    Waiting,
    /// Measuring what is measured once for every machine (the normalize anchors).
    Preparing,
    Rendering,
    /// No new work handed out (by the user, or the output folder is nearly full).
    Paused,
    /// Over: `exit_code` says how.
    Finished,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub(crate) struct ControllerStatus {
    pub(crate) phase: ControllerPhase,
    pub(crate) detail: String,
    pub(crate) name: String,
    pub(crate) identity: String,
    pub(crate) listen: String,
    pub(crate) port: u16,
    pub(crate) key_file: String,
    pub(crate) tour: String,
    pub(crate) out: String,
    pub(crate) min_clients: usize,
    pub(crate) job_id: Option<String>,
    pub(crate) frames: u64,
    pub(crate) done: u64,
    pub(crate) assigned: u64,
    pub(crate) pending: u64,
    pub(crate) failed: u64,
    pub(crate) storage_low: bool,
    pub(crate) frames_per_s: f64,
    pub(crate) kb_in_per_s: f64,
    pub(crate) kb_out_per_s: f64,
    pub(crate) eta_s: Option<f64>,
    pub(crate) free_bytes: Option<u64>,
    pub(crate) elapsed_s: f64,
    /// `Scheduler::strip`: one character per cell, `d . a x`.
    pub(crate) strip: String,
    pub(crate) clients: Vec<ClientRow>,
    /// Distinct probe images among the admitted machines (design §9): more than one means the job
    /// mixes GPUs whose pictures differ.
    pub(crate) gpu_classes: usize,
    pub(crate) exit_code: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub(crate) struct ClientRow {
    /// The connection's id: what `remove` takes.
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) addr: String,
    /// In words a person can act on ("rendering 123 (4/16)", "removed: 2 bad frames").
    pub(crate) state: String,
    pub(crate) adapter: String,
    pub(crate) link_mbps: Option<f64>,
    pub(crate) frames_done: u64,
    pub(crate) ms_per_frame: Option<f64>,
    pub(crate) strikes: u32,
    pub(crate) heartbeat_age_s: Option<f64>,
    pub(crate) kb_in: u64,
    /// Pixels where this machine's probe differs from the controller's own (`None`: not compared).
    pub(crate) probe_px: Option<u64>,
    /// "A", "B", …: machines whose probes are identical share a letter.
    pub(crate) gpu_class: Option<String>,
    /// Present in the table but no longer connected (removed, or gone); `readmit` takes its name.
    pub(crate) removed: bool,
}

/// Commands a controller takes on stdin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ControllerCommand {
    /// Hand out no new work; runs in flight finish.
    Pause,
    Resume,
    /// End the job with what it has (it resumes from the folder later).
    Stop,
    Remove(u32),
    Readmit(String),
}

impl ControllerCommand {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        let (verb, rest) = s.split_once(' ').map_or((s, ""), |(v, r)| (v, r.trim()));
        Some(match verb {
            "pause" if rest.is_empty() => Self::Pause,
            "resume" if rest.is_empty() => Self::Resume,
            "stop" if rest.is_empty() => Self::Stop,
            "remove" => Self::Remove(rest.parse().ok()?),
            "readmit" if !rest.is_empty() => Self::Readmit(rest.to_string()),
            _ => return None,
        })
    }

    pub(crate) fn text(&self) -> String {
        match self {
            Self::Pause => "pause".into(),
            Self::Resume => "resume".into(),
            Self::Stop => "stop".into(),
            Self::Remove(id) => format!("remove {id}"),
            Self::Readmit(n) => format!("readmit {n}"),
        }
    }
}

#[cfg(test)]
mod tests;
