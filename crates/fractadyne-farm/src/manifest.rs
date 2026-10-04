//! The job's state, kept in the output folder so the folder alone resumes a render (design §6).
//!
//! ```text
//! <out>/<prefix>_NNNNN.png      the frames, named as --render-tour names them
//! <out>/farm/job.toml           what the job IS: id, script and settings digests, size, fps, frames
//! <out>/farm/done.jsonl         one line per verified frame: index, bytes, SHA-256, machine, ms
//! <out>/farm/events.jsonl       what happened: assignments, timeouts, strikes, removals
//! <out>/farm/status.toml        rewritten every few seconds: counts, ETA, machines, free space
//! <out>/farm/metrics.jsonl      one sample every 10 s
//! <out>/farm/bad/               frames that failed verification, named by sender
//! <out>/farm/diag/              diagnostics bundles written when a machine is removed
//! ```
//!
//! A resume trusts `done.jsonl` only for files that are still there at the recorded size, adopts
//! frames on disk that it lacks a record for (a plain `--render-tour --resume`, or a frame whose
//! record was lost) when the caller's structural check passes, and removes unfinished `.part` files.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

const FORMAT: &str = "fractadyne-farm-job";
const VERSION: u32 = 1;

/// What a job is. Two jobs with the same id would render the same frames identically.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct JobIdentity {
    pub format: String,
    pub version: u32,
    pub job_id: String,
    pub name: String,
    /// For the person reading the file; never used as a path.
    pub script_file: String,
    pub script_sha256: String,
    pub settings_sha256: String,
    pub anchors_sha256: Option<String>,
    pub app_version: String,
    pub git: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub ss: u32,
    pub prefix: String,
    pub frames: u64,
    pub created_unix: u64,
}

impl JobIdentity {
    /// The identity, with its id derived from everything that decides the pictures.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        script_file: &str,
        script_text: &str,
        settings_json: &str,
        anchors_text: Option<&str>,
        app_version: &str,
        git: &str,
        (width, height): (u32, u32),
        fps: f64,
        ss: u32,
        prefix: &str,
        frames: u64,
        created_unix: u64,
    ) -> Self {
        let script_sha256 = crate::sha256_hex(script_text.as_bytes());
        let settings_sha256 = crate::sha256_hex(settings_json.as_bytes());
        let anchors_sha256 = anchors_text.map(|a| crate::sha256_hex(a.as_bytes()));
        let key = format!(
            "{script_sha256}|{settings_sha256}|{}|{app_version}|{git}|{width}x{height}|{fps:?}|{ss}|{prefix}|{frames}",
            anchors_sha256.as_deref().unwrap_or("-")
        );
        let job_id = crate::sha256_hex(key.as_bytes())[..16].to_string();
        Self {
            format: FORMAT.into(),
            version: VERSION,
            job_id,
            name: name.into(),
            script_file: script_file.into(),
            script_sha256,
            settings_sha256,
            anchors_sha256,
            app_version: app_version.into(),
            git: git.into(),
            width,
            height,
            fps,
            ss,
            prefix: prefix.into(),
            frames,
            created_unix,
        }
    }
}

/// One verified frame, as `done.jsonl` records it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DoneRecord {
    pub index: u64,
    pub bytes: u64,
    pub sha256: String,
    pub machine: String,
    pub ms: u64,
    pub at_unix: u64,
}

/// What a resume found.
#[derive(Debug, Default, PartialEq)]
pub struct Resume {
    /// Frames done: recorded and present at the recorded size, or adopted.
    pub done: Vec<u64>,
    /// Frames on disk with no record that passed the structural check.
    pub adopted: Vec<u64>,
    /// Records whose file is missing or has changed size: rendered again.
    pub lost: Vec<u64>,
    /// Unfinished `.part` files removed.
    pub partials_removed: usize,
    /// `done.jsonl` lines that did not parse (a torn last line after a crash).
    pub bad_lines: usize,
}

pub struct Manifest {
    out: PathBuf,
    dir: PathBuf,
    pub job: JobIdentity,
}

impl Manifest {
    /// Open `<out>/farm/` for `job`: create it, or resume the same job there. A folder holding a
    /// DIFFERENT job is refused, naming both — mixing two renders into one sequence is the failure
    /// worth preventing. `adopt(path)` is the structural check for frames without a record.
    pub fn open(out: &Path, job: JobIdentity, adopt: &dyn Fn(&Path) -> bool) -> Result<(Self, Resume), String> {
        let dir = out.join("farm");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let jpath = dir.join("job.toml");
        let mut resume = Resume::default();
        match std::fs::read_to_string(&jpath) {
            Ok(text) => {
                let existing: JobIdentity = toml::from_str(&text).map_err(|e| format!("{}: {e}", jpath.display()))?;
                if existing.job_id != job.job_id {
                    return Err(format!(
                        "{} already holds a different farm job (\"{}\", id {}; this job is \"{}\", id {}) — choose another output folder",
                        out.display(),
                        existing.name,
                        existing.job_id,
                        job.name,
                        job.job_id
                    ));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let text = toml::to_string(&job).map_err(|e| e.to_string())?;
                crate::key::write_atomic(&jpath, format!("# Fractadyne render-farm job. The folder resumes from this file.\n{text}").as_bytes())?;
            }
            Err(e) => return Err(format!("{}: {e}", jpath.display())),
        }
        let m = Self { out: out.to_path_buf(), dir, job };
        m.scan(&mut resume, adopt);
        Ok((m, resume))
    }

    fn scan(&self, r: &mut Resume, adopt: &dyn Fn(&Path) -> bool) {
        let mut recorded = std::collections::HashMap::new();
        if let Ok(text) = std::fs::read_to_string(self.dir.join("done.jsonl")) {
            for line in text.lines().filter(|l| !l.trim().is_empty()) {
                match serde_json::from_str::<DoneRecord>(line) {
                    Ok(d) if d.index < self.job.frames => {
                        recorded.insert(d.index, d.bytes);
                    }
                    _ => r.bad_lines += 1,
                }
            }
        }
        let prefix = &self.job.prefix;
        if let Ok(rd) = std::fs::read_dir(&self.out) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                let Some(rest) = name.strip_prefix(&format!("{prefix}_")) else { continue };
                if let Some(num) = rest.strip_suffix(".png.part") {
                    if num.parse::<u64>().is_ok() && std::fs::remove_file(e.path()).is_ok() {
                        r.partials_removed += 1;
                    }
                    continue;
                }
                let Some(num) = rest.strip_suffix(".png") else { continue };
                let Ok(i) = num.parse::<u64>() else { continue };
                if i >= self.job.frames {
                    continue;
                }
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                match recorded.remove(&i) {
                    Some(bytes) if bytes == size => r.done.push(i),
                    Some(_) => r.lost.push(i),
                    None if adopt(&e.path()) => {
                        r.done.push(i);
                        r.adopted.push(i);
                    }
                    None => {}
                }
            }
        }
        r.lost.extend(recorded.into_keys());
        r.done.sort_unstable();
        r.adopted.sort_unstable();
        r.lost.sort_unstable();
    }

    pub fn out_dir(&self) -> &Path {
        &self.out
    }

    pub fn farm_dir(&self) -> &Path {
        &self.dir
    }

    /// Where frame `index` lives.
    pub fn frame_path(&self, index: u64) -> PathBuf {
        self.out.join(crate::names::frame_file_name(&self.job.prefix, index))
    }

    /// Where a verified frame waits until the scheduler accepts it.
    pub fn frame_part_path(&self, index: u64) -> PathBuf {
        let mut p = self.frame_path(index).into_os_string();
        p.push(".part");
        PathBuf::from(p)
    }

    /// Where a frame that failed verification is kept for inspection.
    pub fn quarantine_path(&self, index: u64, machine: &str) -> PathBuf {
        self.dir.join("bad").join(format!("{}_{index:05}.{}.png", self.job.prefix, safe_name(machine)))
    }

    /// A diagnostics bundle's folder for `machine`.
    pub fn diag_dir(&self, machine: &str, unix: u64) -> PathBuf {
        self.dir.join("diag").join(format!("{}-{unix}", safe_name(machine)))
    }

    fn append(&self, file: &str, line: &str) -> Result<(), String> {
        let p = self.dir.join(file);
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        f.write_all(format!("{line}\n").as_bytes()).map_err(|e| format!("{}: {e}", p.display()))
    }

    /// Record a verified frame. Synced, because a resume trusts it.
    pub fn record_done(&self, d: &DoneRecord) -> Result<(), String> {
        let line = serde_json::to_string(d).map_err(|e| e.to_string())?;
        let p = self.dir.join("done.jsonl");
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        f.write_all(format!("{line}\n").as_bytes()).and_then(|()| f.sync_data()).map_err(|e| format!("{}: {e}", p.display()))
    }

    pub fn record_event(&self, unix_ms: u64, what: &str) -> Result<(), String> {
        let line = serde_json::json!({ "at_ms": unix_ms, "event": what }).to_string();
        self.append("events.jsonl", &line)
    }

    pub fn record_metrics(&self, sample: &serde_json::Value) -> Result<(), String> {
        self.append("metrics.jsonl", &sample.to_string())
    }

    /// Rewrite `status.toml` (atomically: a reader never sees half of it).
    pub fn write_status(&self, toml_text: &str) -> Result<(), String> {
        crate::key::write_atomic(&self.dir.join("status.toml"), toml_text.as_bytes())
    }
}

/// A machine name as a file-name part: anything outside `A-Za-z0-9_-` becomes `_`, at most 40.
pub fn safe_name(name: &str) -> String {
    let s: String = name.chars().take(40).map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    if s.is_empty() { "machine".into() } else { s }
}

#[cfg(test)]
mod tests;
