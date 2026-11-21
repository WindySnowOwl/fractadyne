//! The session's coloring hook (design/coloring-plan.md B10): a WGSL `custom_color` that colors the
//! view in place of the method ([`fractadyne_gpu::color_hook`]), its parameters, and the file it
//! came from, which is watched so a save recolors the view.
//!
//! The TEXT is the hook's identity, as a custom formula's is: the session, a farm job and a view
//! file carry the text (`ColoringSpec::coloring_hook`), never the file's path, which only the
//! session keeps (it names the user's folders, and a view is shared). A text that no longer builds
//! (a save with a typo) is reported and the last one that built stays in use, so a mistake while
//! editing never blanks the view.

use fractadyne_gpu::color_hook::{self, ColorHook, MAX_PARAMS};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// How often the watcher looks at the file.
const WATCH_EVERY: Duration = Duration::from_millis(300);

/// The hook in use, if any, and where it came from.
#[derive(Default)]
pub(crate) struct HookState {
    /// The hook the color pass runs; `None`: the method's own colors.
    pub(crate) active: Option<Arc<ColorHook>>,
    /// `p1`..`p4` (the hook's defaults when it was loaded, then what the sliders say).
    pub(crate) params: [f32; MAX_PARAMS],
    /// The file it was loaded from and is watched; `None` for a hook a view or a session carried.
    pub(crate) path: Option<PathBuf>,
    /// Why the file's text does not build, or could not be read: shown in the dialog while the
    /// last hook that built stays in use.
    pub(crate) error: Option<String>,
    /// Bumped on every change of the hook (not of its parameters): what the live renderer's
    /// pipeline is prepared for.
    pub(crate) rev: u64,
    watcher: Option<Watcher>,
}

/// A thread that reads the file whenever its modification time or length changes and hands the
/// text over; the UI thread builds it ([`crate::FractadyneApp::poll_coloring_hook`]).
struct Watcher {
    stop: Arc<AtomicBool>,
    rx: Receiver<Result<String, String>>,
    /// The UI's context, to wake it when the file changes: set by the first [`HookState::poll`]
    /// (the session's hook is loaded before there is a window).
    ctx: Arc<std::sync::OnceLock<egui::Context>>,
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// A file's text, refused past [`color_hook::MAX_TEXT`] (a hook is a short file; a wrong pick
/// should not be read whole).
pub(crate) fn read_hook_file(path: &Path) -> Result<String, String> {
    let len = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if len > color_hook::MAX_TEXT as u64 {
        return Err(format!("{} KB is too long for a coloring hook (at most {} KB)", len / 1024, color_hook::MAX_TEXT / 1024));
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let text = String::from_utf8(bytes).map_err(|_| "the file is not UTF-8 text".to_string())?;
    Ok(text.strip_prefix('\u{feff}').unwrap_or(&text).to_string())
}

fn stamp(path: &Path) -> Option<(SystemTime, u64)> {
    let m = std::fs::metadata(path).ok()?;
    Some((m.modified().ok()?, m.len()))
}

impl Watcher {
    fn start(path: PathBuf) -> Option<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::channel();
        let flag = stop.clone();
        let ctx = Arc::new(std::sync::OnceLock::<egui::Context>::new());
        let wake = ctx.clone();
        std::thread::Builder::new()
            .name("fd-hook-watch".into())
            .spawn(move || {
                let mut seen = stamp(&path);
                while !flag.load(Ordering::Relaxed) {
                    std::thread::sleep(WATCH_EVERY);
                    let now = stamp(&path);
                    if now.is_none() || now == seen {
                        continue;
                    }
                    seen = now;
                    // An editor may still be writing it: read once it has held still a moment.
                    std::thread::sleep(Duration::from_millis(60));
                    if tx.send(read_hook_file(&path)).is_err() {
                        break;
                    }
                    if let Some(c) = wake.get() {
                        c.request_repaint();
                    }
                }
            })
            .ok()?;
        Some(Watcher { stop, rx, ctx })
    }
}

impl HookState {
    /// The statistics the hook in use reads, which the iteration accumulates whatever the method.
    pub(crate) fn stats(&self) -> crate::StatSet {
        self.active.as_ref().map_or(crate::StatSet::NONE, |h| h.stats)
    }

    /// Make `text` the hook in use, with `params` (its defaults where `None` or of the wrong length)
    /// and `path` the file to watch (`None`: watch nothing). On an error nothing changes but the
    /// error, which is also returned.
    pub(crate) fn set_text(&mut self, text: &str, params: Option<&[f32]>, path: Option<PathBuf>) -> Result<(), String> {
        let hook = match color_hook::build(text) {
            Ok(h) => h,
            Err(e) => {
                let e = e.to_string();
                self.error = Some(e.clone());
                return Err(e);
            }
        };
        self.params = match params {
            Some(p) if p.len() == MAX_PARAMS && p.iter().all(|v| v.is_finite()) => std::array::from_fn(|i| p[i]),
            _ => hook.defaults(),
        };
        self.active = Some(Arc::new(hook));
        self.error = None;
        self.rev += 1;
        if path != self.path || (path.is_some() && self.watcher.is_none()) {
            self.watcher = path.clone().and_then(Watcher::start);
        }
        self.path = path;
        Ok(())
    }

    /// Load and watch the file at `path`. A file that cannot be read or does not build is an error
    /// and changes nothing (the dialog shows why).
    pub(crate) fn load_file(&mut self, path: &Path) -> Result<(), String> {
        let text = read_hook_file(path).inspect_err(|e| self.error = Some(e.clone()))?;
        self.set_text(&text, None, Some(path.to_path_buf()))
    }

    /// Back to the method's own colors; the file is no longer watched.
    pub(crate) fn turn_off(&mut self) {
        if self.active.is_some() {
            self.rev += 1;
        }
        self.active = None;
        self.error = None;
        self.path = None;
        self.watcher = None;
    }

    /// Take what the watcher read since the last call and build it: `true` when the hook changed.
    /// A text that does not build keeps the hook in use and sets the error; the parameters a
    /// hook still has keep their values (only a range that moved clamps them).
    pub(crate) fn poll(&mut self, ctx: Option<&egui::Context>) -> bool {
        let Some(w) = &self.watcher else { return false };
        if let Some(c) = ctx {
            w.ctx.get_or_init(|| c.clone());
        }
        let mut latest = None;
        loop {
            match w.rx.try_recv() {
                Ok(r) => latest = Some(r),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.watcher = None;
                    break;
                }
            }
        }
        match latest {
            None => false,
            Some(Err(e)) => {
                self.error = Some(e);
                false
            }
            Some(Ok(text)) if self.active.as_ref().is_some_and(|h| h.text == text) => {
                self.error = None;
                false
            }
            Some(Ok(text)) => {
                let kept = self.params;
                // The same path: `set_text` keeps the watcher running.
                let path = self.path.clone();
                let ok = self.set_text(&text, Some(&kept), path).is_ok();
                if ok {
                    if let Some(h) = &self.active {
                        for (v, p) in self.params.iter_mut().zip(&h.params) {
                            *v = v.clamp(p.min, p.max);
                        }
                    }
                }
                ok
            }
        }
    }

    /// The parameters for the GPU (`ColorU.hook_p`): zeros without a hook.
    pub(crate) fn gpu_params(&self) -> [f32; MAX_PARAMS] {
        if self.active.is_some() {
            self.params
        } else {
            [0.0; MAX_PARAMS]
        }
    }
}

impl crate::FractadyneApp {
    /// Once a frame: take a save of the hook's file (rebuilding it), and have the live renderer's
    /// pipeline for the hook in use ready, built here rather than in the paint callback (whose own
    /// build the stall guards would read as a slow GPU pass). A color-only module: milliseconds.
    pub(crate) fn poll_coloring_hook(&mut self, ctx: &egui::Context) {
        if self.coloring.hook.poll(Some(ctx)) {
            crate::diag::log_line("hook", "the coloring hook's file was saved: rebuilt");
            ctx.request_repaint();
        }
        let rev = self.coloring.hook.rev;
        if rev == self.hook_prepared {
            return;
        }
        self.hook_prepared = rev;
        if let (Some(h), Some(rs)) = (self.coloring.hook.active.clone(), self.render_state.as_ref()) {
            let t = std::time::Instant::now();
            if fractadyne_gpu::prepare_color_hook_now(rs, &h) {
                crate::diag::log_line("hook", &format!("coloring hook pipeline built in {:.0} ms", t.elapsed().as_secs_f64() * 1000.0));
            }
        }
    }
}

/// `a,b,c,d` → four finite numbers, else `None`.
pub(crate) fn parse_params(s: &str) -> Option<Vec<f32>> {
    let v: Vec<f32> = s.split(',').map(|t| t.trim().parse::<f32>().ok().filter(|x| x.is_finite())).collect::<Option<_>>()?;
    (v.len() == MAX_PARAMS).then_some(v)
}

/// The `a,b,c,d` form of four parameters (each the shortest decimal that reads back to its bits).
pub(crate) fn params_line(p: &[f32]) -> String {
    p.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join(",")
}

#[cfg(test)]
#[path = "../tests/unit/coloring_hook/tests.rs"]
mod tests;
