//! What the device can do to make a frame cheaper — its ACTUATORS — probed from the device once it
//! exists (W7, design/live-render-robustness.md §5.6).
//!
//! ⭐**Unprobed is not "unavailable".** These used to be two booleans (`chunk_ok`, `chunk_fe_ok`)
//! defaulting to false, set by a probe that ran on frame 0 only: had the GPU not been ready on that
//! frame, the session would have run without its resumable walk — the actuator every deep frame's
//! budget leans on — with nothing in the log to say so (U1's capability latch). The probe now runs
//! on the first frame that HAS a device, `Perf::actuators` is `None` until it has, and the verdict
//! is one always-on line naming every actuator a retreat can reach.

use crate::RenderMode;

/// The resumable iteration-range walk (`MandelbrotParams::chunk_range`): a frame's iterations split
/// across passes, so no single dispatch carries the whole count. It needs three `Rgba32Float` state
/// targets (48 colour-attachment bytes a sample); `mode2` says whether the device also granted the
/// fourth that floatexp's 13 floats of state need (64 bytes) — a device can have one and not the
/// other, and then loses only the deep path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ChunkCap {
    pub(crate) mode2: bool,
}

/// The actuators of the device drawing the window. The resolution shrink, the settle tiles and the
/// supersampling level are software paths every device has; the walk and the GPU's own timing are
/// the device's to grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Actuators {
    /// The walk, or `None` when the device lacks the attachment bytes.
    pub(crate) chunk: Option<ChunkCap>,
    /// GPU timestamps (`TIMESTAMP_QUERY`): without them every dispatch is priced by the wall clock.
    pub(crate) timestamps: bool,
    /// The frame the probe ran on — 0 unless the device was not there yet (the latch case).
    pub(crate) probed_at: u64,
}

impl Actuators {
    pub(crate) fn new(chunk: bool, chunk_mode2: bool, timestamps: bool, probed_at: u64) -> Self {
        Actuators { chunk: chunk.then_some(ChunkCap { mode2: chunk_mode2 }), timestamps, probed_at }
    }

    /// Can a frame in `mode` be spread over a walk on this device? Direct and df32 need three state
    /// targets; floatexp needs four.
    pub(crate) fn can_chunk(&self, mode: RenderMode) -> bool {
        self.chunk.is_some_and(|c| mode != RenderMode::Floatexp || c.mode2)
    }

    /// The always-on verdict line (`[fd-wgpu]`), beside the adapter line.
    pub(crate) fn verdict_line(&self) -> String {
        let walk = match self.chunk {
            Some(ChunkCap { mode2: true }) => "yes (direct, df32 and floatexp)",
            Some(ChunkCap { mode2: false }) => "df32 and direct only — NOT floatexp (no fourth state target)",
            None => "NO (fewer than 48 attachment bytes a sample): every refresh is one dispatch",
        };
        format!(
            "actuators: resumable walk {walk} · resolution shrink yes · settle tiles yes · supersampling yes · \
             GPU timing {}{}",
            if self.timestamps { "yes" } else { "NO (wall clock prices every dispatch)" },
            if self.probed_at > 0 {
                format!(" — probed at frame {}: the device was not ready before it", self.probed_at)
            } else {
                String::new()
            }
        )
    }
}

/// Can a frame in `mode` walk, given what is known so far? `None` (not probed yet) cannot: nothing
/// may be dispatched on the strength of a capability nobody has read.
pub(crate) fn can_chunk(actuators: Option<&Actuators>, mode: RenderMode) -> bool {
    actuators.is_some_and(|a| a.can_chunk(mode))
}

#[cfg(test)]
#[path = "../../tests/unit/render/actuators/tests.rs"]
mod tests;
