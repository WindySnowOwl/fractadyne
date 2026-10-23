//! How the last session ended, and whether this one gives back the iteration setting it ended
//! with (W7, design/live-render-robustness.md §5.7).
//!
//! ⭐**A relaunch must not walk straight back into the load that ended the last session.** The
//! view already does not come back after a device loss (the crash-loop guard opens at Home), but
//! the ITERATION SETTING did, and it is half the load: an explicit 10,000,000 — or auto-iterations
//! from a base that large, which adds the depth bonus on top — iterates every interior pixel of
//! Home's cardioid to that count, and the relaunch ground again (MEM-38, U18). So after a session
//! that did not end cleanly, a setting that asks more of a pixel than the default would at the
//! depth where it ended is set aside: the session starts on the default and SAYS so, offering the
//! old setting back in one click (P17 — a feature that declines must say it declined).
//!
//! The outcome is assembled at launch from what already survives every death, rather than written
//! by a crash path a kill never reaches: the `session.running` marker (clean or not, for every
//! kind of ending), the relaunch's own `FRACTADYNE_RESTARTED_AFTER_GPU_LOSS` (a device loss), the
//! saved session (the setting and the view, autosaved within a second of any change), and the dead
//! session's frame record (what its controller had learned at its last frame).

use fractadyne_core::recommended_iterations;

/// How the previous session ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionEnd {
    Clean,
    /// No clean shutdown: a crash, an abort, a kill — the marker survived it.
    Unclean,
    /// It lost its graphics device and relaunched this process.
    DeviceLost,
}

/// An iteration setting: the count, and whether auto-iterations scale it with depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ask {
    pub(crate) max_iter: u32,
    pub(crate) auto_iter: bool,
}

impl Ask {
    /// The setting a fresh install starts with.
    pub(crate) fn default_ask() -> Ask {
        let s = fractadyne_state::SessionState::default();
        Ask { max_iter: s.max_iter, auto_iter: s.auto_iter }
    }

    /// What the setting asks of one pixel at `log2mag`: the count itself, or under auto-iterations
    /// the count as a base plus the depth bonus (`Viewport::recommended_max_iter`).
    pub(crate) fn at(self, log2mag: f64) -> u32 {
        if self.auto_iter {
            recommended_iterations(self.max_iter, log2mag)
        } else {
            self.max_iter
        }
    }

    /// The setting as the Controls panel would name it, in the language in use.
    pub(crate) fn describe(self) -> String {
        if self.auto_iter {
            tr!(quarantine_setting_auto, base = self.max_iter)
        } else {
            tr!(quarantine_setting_fixed, count = self.max_iter)
        }
    }

    /// The same in English, for the log.
    pub(crate) fn log_describe(self) -> String {
        let n = crate::grouped_count(self.max_iter as f64);
        if self.auto_iter {
            format!("automatic iterations from a base of {n}")
        } else {
            format!("{n} iterations")
        }
    }
}

/// What a launch knows about the session before it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SessionOutcome {
    pub(crate) end: SessionEnd,
    /// The depth of the saved view, log2 of the magnification.
    pub(crate) at_log2mag: f64,
    /// The iteration setting it was saved with.
    pub(crate) ask: Ask,
    /// From its own frame record, when that survived: the steps the admission let one dispatch
    /// carry at its last planned frame, and the measured rate (steps a millisecond) in its mode.
    pub(crate) admitted_budget: Option<u64>,
    pub(crate) mode_rate: Option<f64>,
}

/// A setting set aside: the session's, and the one this session starts with instead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Quarantine {
    pub(crate) outcome: SessionOutcome,
    pub(crate) restored: Ask,
}

/// Should the last session's setting be set aside? Only after a session that did not end cleanly,
/// and only when the setting asks more of a pixel, at the depth where that session ended, than
/// `default` would there: anything at or under the default cannot be the load that ended it, and
/// setting it aside would only surprise. `#[must_use]`: a caller that drops the verdict restores
/// the setting silently.
#[must_use]
pub(crate) fn quarantine(o: &SessionOutcome, default: Ask) -> Option<Quarantine> {
    if o.end == SessionEnd::Clean {
        return None;
    }
    // A non-finite depth (a poisoned session the loader already refuses) asks as at Home.
    let l2 = if o.at_log2mag.is_finite() { o.at_log2mag.max(0.0) } else { 0.0 };
    (o.ask.at(l2) > default.at(l2)).then_some(Quarantine { outcome: *o, restored: default })
}

impl Quarantine {
    /// How the session ended, in English, for the log.
    fn ended(&self) -> &'static str {
        match self.outcome.end {
            SessionEnd::DeviceLost => "lost its graphics device",
            _ => "did not end cleanly",
        }
    }

    /// The always-on log line (`[fd-start]`).
    pub(crate) fn log_line(&self) -> String {
        let o = &self.outcome;
        format!(
            "the previous session {} at {}× with {} ({} a pixel there) — set aside: this session starts with {} \
             ({} a pixel there), and offers it back{}",
            self.ended(),
            crate::fmt_log2_count(o.at_log2mag),
            o.ask.log_describe(),
            crate::grouped_count(o.ask.at(o.at_log2mag.max(0.0)) as f64),
            self.restored.log_describe(),
            crate::grouped_count(self.restored.at(o.at_log2mag.max(0.0)) as f64),
            match (o.admitted_budget, o.mode_rate) {
                (Some(b), Some(r)) if r > 0.0 =>
                    format!("; its last frame was admitted {b:.3e} steps a dispatch at {r:.3e} steps/ms"),
                (Some(b), _) => format!("; its last frame was admitted {b:.3e} steps a dispatch"),
                _ => "; no frame record of it survived".to_string(),
            }
        )
    }

    /// What the notice says.
    pub(crate) fn notice(&self) -> String {
        let end = match self.outcome.end {
            SessionEnd::DeviceLost => "device",
            _ => "unclean",
        };
        tr!(
            quarantine_notice,
            end = end,
            depth = crate::fmt_log2_count(self.outcome.at_log2mag),
            setting = self.outcome.ask.describe(),
            restored = self.restored.describe()
        )
    }
}
