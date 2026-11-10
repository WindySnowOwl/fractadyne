//! ⭐The speed check's SAVED READINGS (`speed-checks.toml` in the config folder, 2026-11-10).
//!
//! On Intel HD Graphics 4400 (Eris) the check takes ~7 s, mostly the driver compiling each
//! precision mode's shader the first time it draws, at every launch. A reading is a property of the
//! card, its driver, this build's shaders, and the machine's power state (`power_state`), so it is
//! filed under all of them and reused while they all hold.
//!
//! **A saved reading is checked, not trusted.** A launch that finds one uses it at once, and ONE
//! mode is measured again in the background, at the iterations it was saved at: the mode the first
//! frame renders in (`super::recheck`). A reading that moved more than [`RECHECK_SLACK`] either way
//! is dropped and every mode measured again: something the key cannot see changed (the card runs
//! hot, a power setting not read, another program on the GPU). A slower card must not inherit a
//! faster reading, and a reading taken while the machine was busy must not cap it for good.
//!
//! One reading per card and power state, the newest first, at most [`KEEP`]: a laptop moving
//! between the charger and its battery measures each state once. A new build measures again (its
//! shaders differ). The file is the app's own but read as untrusted: bounded, and every value
//! checked before it is used ([`SavedFinding::finding`]).

use super::{Finding, ITERS_MAX};
use crate::power_state::PowerState;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) const FILE: &str = "speed-checks.toml";
/// The most readings kept, the newest first.
pub(crate) const KEEP: usize = 8;
/// A file larger than this is not read (it holds [`KEEP`] readings of well under 1 KB).
const MAX_BYTES: u64 = 64 * 1024;
/// How far the re-check may read from the saved reading, either way, for the reading to stand.
/// Pass-to-pass jitter on the probe's own passes is a few percent; a laptop on battery or a card
/// held at a low clock moves it 2× or more.
pub(crate) const RECHECK_SLACK: f64 = 1.5;

/// What a reading is OF, beside the power state: all must be equal for it to be used.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Card {
    /// This build (`sysinfo::version_string`): its shaders are what was timed.
    pub app: String,
    pub name: String,
    /// The graphics API (`Vulkan`, `Gl`, ...).
    pub api: String,
    pub driver: String,
}

/// One mode's [`Finding`], as stored.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedFinding {
    /// [`Finding::Fast`] (a bound) rather than [`Finding::Measured`].
    #[serde(default)]
    pub fast: bool,
    pub iters: u32,
    pub ms: f64,
    #[serde(default)]
    pub wall_ms: f64,
    pub ms_per_step: f64,
}

impl SavedFinding {
    pub(crate) fn of(f: Finding) -> SavedFinding {
        match f {
            Finding::Measured { iters, ms, wall_ms, ms_per_step } => SavedFinding { fast: false, iters, ms, wall_ms, ms_per_step },
            Finding::Fast { iters, ms, ms_per_step } => SavedFinding { fast: true, iters, ms, wall_ms: 0.0, ms_per_step },
        }
    }

    /// The finding, where every value is one a probe could have produced; `None` otherwise.
    pub(crate) fn finding(&self) -> Option<Finding> {
        let ok = (1..=ITERS_MAX).contains(&self.iters)
            && self.ms.is_finite()
            && self.ms > 0.0
            && self.ms < 1e5
            && self.ms_per_step.is_finite()
            && self.ms_per_step > 0.0
            && self.ms_per_step < 1.0
            && self.wall_ms.is_finite()
            && self.wall_ms >= 0.0;
        ok.then_some(if self.fast {
            Finding::Fast { iters: self.iters, ms: self.ms, ms_per_step: self.ms_per_step }
        } else {
            Finding::Measured { iters: self.iters, ms: self.ms, wall_ms: self.wall_ms, ms_per_step: self.ms_per_step }
        })
    }
}

/// One saved reading: the three modes' findings for a card in a power state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Reading {
    pub card: Card,
    pub power: PowerState,
    /// When it was taken, local time, for a reader of the file.
    #[serde(default)]
    pub taken: String,
    pub direct: SavedFinding,
    pub df32: SavedFinding,
    pub floatexp: SavedFinding,
}

impl Reading {
    /// The three findings in the probe's order (direct, df32, floatexp), where all are usable.
    pub(crate) fn findings(&self) -> Option<[Finding; 3]> {
        Some([self.direct.finding()?, self.df32.finding()?, self.floatexp.finding()?])
    }
}

#[derive(Default, Serialize, Deserialize)]
struct FileBody {
    #[serde(default, rename = "reading")]
    readings: Vec<Reading>,
}

/// The reading for `card` in `power`, if one is saved and usable.
pub(crate) fn find<'a>(readings: &'a [Reading], card: &Card, power: &PowerState) -> Option<&'a Reading> {
    readings.iter().find(|r| r.card == *card && power.matches(&r.power) && r.findings().is_some())
}

/// `readings` without the one(s) for `card` in `power`.
pub(crate) fn without(readings: &[Reading], card: &Card, power: &PowerState) -> Vec<Reading> {
    readings.iter().filter(|r| !(r.card == *card && power.matches(&r.power))).cloned().collect()
}

/// `readings` with `new` first, in place of any it replaces, and at most [`KEEP`].
pub(crate) fn with(readings: &[Reading], new: Reading) -> Vec<Reading> {
    let mut out = vec![new.clone()];
    out.extend(without(readings, &new.card, &new.power));
    out.truncate(KEEP);
    out
}

/// Whether a saved `finding` stands, given the re-check's cost per step at its iterations, `fresh`.
/// A measured cost stands within [`RECHECK_SLACK`] either way; a [`Finding::Fast`] bound stands while
/// the re-check stays under what would change anything (`fast_below`) or near the bound itself.
pub(crate) fn still_holds(saved: Finding, fresh: f64, fast_below: f64) -> bool {
    if !(fresh.is_finite() && fresh > 0.0) {
        return false;
    }
    match saved {
        Finding::Measured { ms_per_step, .. } => fresh <= ms_per_step * RECHECK_SLACK && fresh >= ms_per_step / RECHECK_SLACK,
        Finding::Fast { ms_per_step, .. } => fresh < fast_below.max(ms_per_step * RECHECK_SLACK),
    }
}

/// The readings saved in `dir` (none where the file is missing, too large, or unreadable: the check
/// then measures, as it would on a first launch).
pub(crate) fn load(dir: &Path) -> Vec<Reading> {
    let path = dir.join(FILE);
    let small = std::fs::metadata(&path).is_ok_and(|m| m.len() <= MAX_BYTES);
    if !small {
        return Vec::new();
    }
    std::fs::read_to_string(&path).ok().and_then(|t| parse(&t)).unwrap_or_default()
}

pub(crate) fn parse(text: &str) -> Option<Vec<Reading>> {
    let mut v = toml::from_str::<FileBody>(text).ok()?.readings;
    v.truncate(KEEP);
    Some(v)
}

pub(crate) fn render(readings: &[Reading]) -> Result<String, String> {
    let body = toml::to_string(&FileBody { readings: readings.to_vec() }).map_err(|e| e.to_string())?;
    Ok(format!(
        "# Fractadyne's start-up speed checks: one per graphics card and power state, the newest first.\n\
         # Written by the app; delete this file to measure again at the next launch.\n\n{body}"
    ))
}

/// Write `readings` to `dir`, whole or not at all (a temporary file renamed over the old one).
pub(crate) fn store(dir: &Path, readings: &[Reading]) -> Result<(), String> {
    let text = render(readings)?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(FILE);
    let part = dir.join(format!("{FILE}.part"));
    std::fs::write(&part, text).and_then(|()| std::fs::rename(&part, &path)).map_err(|e| {
        let _ = std::fs::remove_file(&part);
        format!("{}: {e}", path.display())
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/render/speed_probe/saved/tests.rs"]
mod tests;
