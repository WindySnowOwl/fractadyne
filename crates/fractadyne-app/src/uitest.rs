//! `--uitest` — a scripted walk through the UI and the live-render path that captures a screenshot
//! per screen, checks each frame, and writes a review bundle (screenshots + `log.txt` +
//! `report.md`/`report.json`).
//!
//! It runs INSIDE the real eframe loop (so it validates the actual composited UI and the on-screen
//! live GPU render, not an offscreen approximation), driven one step per frame by a small state
//! machine in [`FractadyneApp::uitest_frame`], called from `update()`. Capture uses egui's
//! `ViewportCommand::Screenshot` round-trip — the reply arrives a frame later as
//! `Event::Screenshot`. Headless boxes run it under `xvfb-run` (a window/surface is still needed).
//!
//! The bundle is for HUMAN review — the screenshots are the real validation. The per-step verdicts
//! (screenshot captured, frame not blank, live RenderMode matches the depth, a real iterate ran)
//! are supporting evidence, flagged PASS/WARN/FAIL so a reviewer knows where to look first.

use std::path::PathBuf;
use std::time::Instant;

use crate::{FractadyneApp, RenderMode};

/// Which screen a step forces on. Each is opened in isolation (all other dialogs closed first) so
/// the screenshot shows exactly one thing.
#[derive(Clone, Copy, Debug)]
enum Screen {
    Home,          // the plain view — menu bar, status bar, central fractal, no dialog
    RightPanel,    // the controls panel open over the view
    Minimap,       // the minimap overview on
    Help,
    Welcome,
    Bookmarks,
    BenchConfig,
    BenchResults,  // seeded with a synthetic report so the populated layout renders
    Gallery,
    Goto,
    Share,
    Report,
    Diagnostics,
    Export,
    ScriptExport,
    TourRender,
    ResetConfirm,
    Notice,
    PaletteEditor,
    /// Dual view on one formula, then the SAME dual view on another — checklist steps 45-46, and
    /// the field report behind them: switching formula while dual left the parameter pane showing
    /// the previous formula. The pair is the check; neither screen means anything alone.
    DualFormulaA,
    DualFormulaB,
    /// Field report 2026-08-30: panning FROM THE MINIMAP leaves rectangular blocks of stale
    /// image. The minimap moves the viewport with `pan_complex` and never marks the view as
    /// interacting, so `view_gen` — the only part of the settle key that stands for the centre —
    /// does not move, and a completed tile grid keeps holding tiles drawn for the old view.
    ///
    /// Three steps, because the assertion is a comparison: settle a view, pan it the way the
    /// minimap does, then force an honest re-render of the SAME view. The last two must agree.
    MinimapBase,
    MinimapPan,
    MinimapPanVerify,
}

/// A live-render step: jump to a view and let the on-screen live path render it, then screenshot.
#[derive(Clone, Debug)]
struct LiveView {
    decades: f64, // log10 magnification
    expect: RenderMode,
}

/// A window-sizing step: resize the window, then capture the home view — validates layout and,
/// specifically, the bottom status bar's wrap behaviour (and stability) at that width.
#[derive(Clone, Copy, Debug)]
struct WindowSize {
    w: f32,
    h: f32,
    /// Whether the status bar is expected to fit on ONE line at this width. Narrow windows are
    /// allowed to wrap to two (by design); a fixed reasonable width should not.
    expect_single_line: bool,
    /// What else this step does before it settles.
    action: WindowAction,
}

/// What a window step does beyond setting a size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowAction {
    /// Just the resize.
    Resize,
    /// Maximize, then restore — checklist step 13. The frame after each must still be complete,
    /// which is what the per-step blank/layout checks already assert; what this adds is that the
    /// round trip happens at all and the layout survives it.
    MaximizeRestore,
    /// ~50 size changes in quick succession — checklist step 14. The failure it hunts is a crash
    /// or a wedged frame under resize churn, not a wrong pixel.
    RapidResize,
    /// Hide the control panel and show it again — checklist step 10. The canvas must reflow to
    /// use the space and the settings must be unchanged.
    TogglePanel,
}

#[derive(Clone, Debug)]
enum StepKind {
    Screen(Screen),
    Live(LiveView),
    Window(WindowSize),
}

#[derive(Clone, Debug)]
struct Step {
    name: String,
    kind: StepKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    Pass,
    Warn,
    Fail,
}

impl Verdict {
    fn tag(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Warn => "WARN",
            Verdict::Fail => "FAIL",
        }
    }
}

struct Check {
    name: String,
    verdict: Verdict,
    detail: String,
}

struct StepResult {
    name: String,
    kind: &'static str, // "screen" | "live"
    file: String,       // screenshot filename, or "" if none
    width: u32,
    height: u32,
    checks: Vec<Check>,
    // Live-render evidence (None for pure screens).
    mode: Option<String>,
    eff_iter: Option<u32>,
    orbit_len: Option<u32>,
    precision: Option<usize>,
    mag_log10: Option<f64>,
    frame_ms: Option<f64>,
    // Content-region image stats (centre crop): mean luma, its stddev, non-empty luma buckets/32.
    mean_luma: f64,
    luma_stddev: f64,
    buckets: usize,
    /// A 16x16 luma thumbnail of the LEFT pane's area, so one step can be compared against the
    /// one before it. Whole-frame statistics are too coarse for that: two different fractals can
    /// share a mean and a spread, and the panel chrome dominates either way.
    left_fp: Vec<u8>,
    /// Whether view 0 held a tiled settle grid when this step was captured.
    ///
    /// ⭐Recorded so a check whose PRECONDITION is "a grid completed" can assert it rather than
    /// assume it. Written because the minimap-pan check first passed at meanD 0.0 on a view that
    /// never tiled at all — a clean, confident, meaningless zero.
    tiled: bool,
}

/// A 16x16 luma thumbnail of the left ~45% of the frame, vertically inset past the toolbar and
/// status bar. That region is the dual view's PARAMETER pane — the half the field report was
/// about — and cropping to it keeps the Julia pane and the chrome out of the comparison.
fn left_pane_fingerprint(px: &[egui::Color32], w: usize, h: usize) -> Vec<u8> {
    let (x0, x1) = (w / 40, (w * 45) / 100);
    let (y0, y1) = (h / 8, (h * 7) / 8);
    if x1 <= x0 + 16 || y1 <= y0 + 16 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(256);
    for ty in 0..16 {
        for tx in 0..16 {
            let sx = x0 + (x1 - x0) * tx / 16;
            let sy = y0 + (y1 - y0) * ty / 16;
            let c = px[sy * w + sx];
            out.push(
                ((0.2126 * c.r() as f32 + 0.7152 * c.g() as f32 + 0.0722 * c.b() as f32) as u32)
                    .min(255) as u8,
            );
        }
    }
    out
}

/// Mean absolute difference between two fingerprints (255 when either is missing).
fn fp_distance(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 255.0;
    }
    a.iter().zip(b).map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs() as u64).sum::<u64>() as f64
        / a.len() as f64
}

impl StepResult {
    /// Worst verdict across this step's checks.
    fn worst(&self) -> Verdict {
        self.checks.iter().fold(Verdict::Pass, |acc, c| match (acc, c.verdict) {
            (Verdict::Fail, _) | (_, Verdict::Fail) => Verdict::Fail,
            (Verdict::Warn, _) | (_, Verdict::Warn) => Verdict::Warn,
            _ => Verdict::Pass,
        })
    }
}

/// Per-step phase of the driver.
enum Phase {
    Setup,   // apply the step's mutation, then start settling
    Settle,  // let the frame(s) render/stabilize
    Shot,    // screenshot requested; poll input for the reply
    Done,    // walk finished; write the bundle and exit
}

pub(crate) struct UiTest {
    out_dir: PathBuf,
    steps: Vec<Step>,
    idx: usize,
    phase: Phase,
    results: Vec<StepResult>,
    // Timing gates for the current step (wall-clock, so deep live builds get enough time).
    step_start: Instant,
    settle_until: Instant,
    hard_until: Instant,
    gpu_name: Option<String>,
    // Status-bar height range observed across the current step's settle frames — a spread means the
    // bar wavered between one and two lines at a fixed width (a repaint storm).
    sb_min: f32,
    sb_max: f32,
    // Reference-orbit length last seen, and when it last changed — a live view screenshots only
    // once this stops growing AND no build is in flight (see the gate below; the length alone is
    // not enough, because the progressive build parks at the coarse cap while the full orbit is
    // still being computed). This is what makes a deep band land on the same resolved frame
    // regardless of how fast the machine builds it.
    ref_len_seen: u32,
    ref_changed_at: Instant,
    /// Crash reports already on disk when the walk started. Anything not in this list at the end
    /// appeared DURING the run — which is the only version of "no crash reports" that means
    /// anything on a config dir that is not wiped.
    crashes_at_start: Vec<String>,
    /// Control-panel width measured during the toggle step's OPEN phase (`None` until then).
    panel_w: Option<f32>,
    /// Whether the session BEFORE this one left its unclean-exit marker armed — i.e. it did not
    /// shut down through `crate::exit`. Read at construction, because reporting clears the marker.
    prev_unclean: bool,
    /// What the PREVIOUS walk in this profile left behind: `Some(true)` = it finished and wrote
    /// its own completion marker, `Some(false)` = it started and never finished, `None` = there
    /// was no previous walk.
    ///
    /// ⚠The app's general unclean-exit marker cannot answer this on its own: its absence means
    /// either "exited cleanly" or "never ran", and `--uitest` suppresses the session autosave, so
    /// there is no session file to tell the two apart either. A marker owned by the harness is
    /// unambiguous — and it is the only way a harness can testify about its OWN exit.
    prev_walk_clean: Option<bool>,
}

/// Where a walk records that it started, and then that it finished.
fn walk_marker_path() -> Option<std::path::PathBuf> {
    crate::diag::logs_dir().map(|d| d.join("uitest-last-walk.txt"))
}

// The canonical Seahorse-Valley deep-zoom point, to ~33 digits — self-similar structure from the
// surface down past ~1e30×, so every live band below lands on real detail, not empty space.
const LIVE_CX: &str = "-0.743643887037158704752191506114774";
const LIVE_CY: &str = "0.131825904205311970493132056385139";

impl UiTest {
    /// Build the harness. `out_base` is an optional base directory (`--uitest DIR`); otherwise the
    /// mounted \\vger\share is preferred, else `logs/`. A timestamped run folder is created under it.
    pub(crate) fn new(out_base: Option<PathBuf>) -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let stamp = crate::FractadyneApp::file_stamp(secs);
        let base = out_base.unwrap_or_else(default_out_base);
        let out_dir = base.join(format!("uitest-{stamp}"));
        // Create it up front — screenshots are written during the walk, long before `uitest_finish`.
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("--uitest: could not create {}: {e}", out_dir.display());
        }

        Self {
            out_dir,
            steps: build_steps(),
            idx: 0,
            phase: Phase::Setup,
            results: Vec::new(),
            step_start: Instant::now(),
            settle_until: Instant::now(),
            hard_until: Instant::now(),
            gpu_name: None,
            sb_min: f32::INFINITY,
            sb_max: 0.0,
            ref_len_seen: 0,
            ref_changed_at: Instant::now(),
            crashes_at_start: crate::diag::crash_report_names(),
            panel_w: None,
            prev_unclean: crate::diag::previous_session_unclean(),
            prev_walk_clean: {
                // Read the previous walk's verdict, then claim the marker for this one.
                let prev = walk_marker_path()
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|t| t.trim() == "finished");
                if let Some(p) = walk_marker_path() {
                    if let Some(parent) = p.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&p, "started");
                }
                prev
            },
        }
    }
}

/// `--juliadive [DIR]` — dev harness for the DUAL-VIEW JULIA motion path (reported 2026-08-13:
/// blockiness + a center artifact while zooming the Julia panel). Boots into dual view with a
/// pinned spiral `c` and the reporter's settings (explicit 2000 iterations, prefer-detail on),
/// then zooms the Julia viewport in-app at ~2 octaves/s to ~2^11 (≈2000×), screenshotting every
/// octave IN MOTION plus a stopped and a settled frame. In-app because synthetic OS input
/// (wheel/focus routing) proved unreliable; this drives the same viewport + interaction stamps
/// the real wheel path does.
pub(crate) struct JuliaDive {
    out_dir: PathBuf,
    frame: u64,
    /// Next log2-magnification at which to take an in-motion screenshot.
    next_shot_l2: f64,
    /// A screenshot request is in flight (the reply arrives as `Event::Screenshot` next frame).
    pending: Option<String>,
    /// Set when the target depth is reached: the settle clock for the final two shots.
    stopped_at: Option<Instant>,
    settled_shot_done: bool,
}

impl JuliaDive {
    pub(crate) fn new(out_base: Option<PathBuf>) -> Self {
        let base = out_base.unwrap_or_else(|| PathBuf::from("logs"));
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let out_dir = base.join(format!("juliadive-{}", crate::FractadyneApp::file_stamp(secs)));
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("--juliadive: could not create {}: {e}", out_dir.display());
        }
        Self {
            out_dir,
            frame: 0,
            next_shot_l2: 3.0,
            pending: None,
            stopped_at: None,
            settled_shot_done: false,
        }
    }
}

impl FractadyneApp {
    pub(crate) fn juliadive_frame(&mut self, ctx: &egui::Context) {
        const TARGET_L2: f64 = 15.0; // ≈ 32,000× — crosses PERT_JULIA_THRESHOLD (1e2) and 1e4
        const OCTAVES_PER_S: f64 = 2.0;
        let Some(jd) = self.juliadive.as_mut() else { return };
        jd.frame += 1;
        if jd.frame == 1 {
            // The reporter's setup: dual view, a BOUNDARY Julia c (dense structure at every
            // depth — a bulb-interior c goes smooth by ~1000× and hides resolution artifacts),
            // PINNED so the cursor is irrelevant, explicit 2000 iterations, prefer-detail on.
            self.dual = true;
            self.julia_c = (-0.743643887037158, 0.131825904205311);
            self.julia_pin = Some(self.julia_c);
            self.render_cfg.max_iter = 2000;
            self.render_cfg.auto_iter = false;
            self.render_cfg.prefer_detail = true;
            self.julia_viewport.reset();
            self.julia_viewport.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
            self.julia_viewport.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
            self.invalidate_refs();
            ctx.request_repaint();
            return;
        }
        // Harvest a pending screenshot reply.
        if let Some(name) = self.juliadive.as_ref().and_then(|j| j.pending.clone()) {
            let shot = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(image) = shot {
                let (w, h) = (image.size[0] as u32, image.size[1] as u32);
                let mut bytes = Vec::with_capacity(image.pixels.len() * 4);
                for px in &image.pixels {
                    bytes.extend_from_slice(&[px.r(), px.g(), px.b(), px.a()]);
                }
                let jd = self.juliadive.as_mut().unwrap();
                let path = jd.out_dir.join(&name);
                if let Err(e) = fractadyne_export::write_png_rgba8(&path, w, h, &bytes, None) {
                    eprintln!("--juliadive: write {name}: {e}");
                } else {
                    println!("--juliadive: {name}");
                }
                let was_settled = name.starts_with("settled");
                jd.pending = None;
                if was_settled {
                    println!("--juliadive: done → {}", jd.out_dir.display());
                    crate::exit(0);
                }
            } else {
                ctx.request_repaint();
                return; // wait for the reply before advancing the zoom (shot = one moment)
            }
        }
        let l2 = self.julia_viewport.log2_magnification();
        let jd = self.juliadive.as_mut().unwrap();
        if jd.stopped_at.is_none() {
            if l2 < TARGET_L2 {
                // Zoom about the Julia panel centre, with the same interaction stamp the real
                // wheel/Space path applies (settle_t[1] ⇒ view 1 is "interacting").
                let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);
                let factor = (-(OCTAVES_PER_S * std::f64::consts::LN_2) * dt).exp();
                let (w, h) = (self.julia_viewport.width_px, self.julia_viewport.height_px);
                self.julia_viewport.zoom_at(w * 0.5, h * 0.5, factor);
                self.pointer.settle_t[1] = ctx.input(|i| i.time);
                let jd = self.juliadive.as_mut().unwrap();
                if l2 >= jd.next_shot_l2 && jd.pending.is_none() {
                    let name = format!("mid-l2-{:04.1}.png", l2);
                    jd.pending = Some(name);
                    jd.next_shot_l2 += 1.0;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                        egui::UserData::default(),
                    ));
                }
            } else {
                jd.stopped_at = Some(Instant::now());
                jd.pending = Some("stopped.png".into());
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
        } else if !jd.settled_shot_done
            && jd.pending.is_none()
            && jd.stopped_at.unwrap().elapsed().as_secs_f64() > 3.0
        {
            jd.settled_shot_done = true;
            jd.pending = Some("settled.png".into());
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        ctx.request_repaint();
    }
}

/// Default staging base: the mounted Windows share when present (the dev box reads it directly),
/// else the repo/cwd `logs/` dir.
fn default_out_base() -> PathBuf {
    let share = PathBuf::from("/mnt/vger/Fractadyne/uitest");
    if PathBuf::from("/mnt/vger/Fractadyne").is_dir() {
        share
    } else {
        PathBuf::from("logs")
    }
}

/// The scripted walk: every UI screen, then the live-render bands (one per RenderMode).
fn build_steps() -> Vec<Step> {
    let screen = |name: &str, s: Screen| Step { name: name.to_string(), kind: StepKind::Screen(s) };
    let live = |name: &str, decades: f64| Step {
        name: name.to_string(),
        // Mode is whatever the app itself would pick at this depth — asserted, not forced.
        kind: StepKind::Live(LiveView { decades, expect: RenderMode::select(true, false, 10f64.powf(decades)) }),
    };
    vec![
        // --- UI screens ---
        screen("home", Screen::Home),
        screen("right-panel", Screen::RightPanel),
        screen("minimap", Screen::Minimap),
        screen("help", Screen::Help),
        screen("welcome", Screen::Welcome),
        screen("bookmarks", Screen::Bookmarks),
        screen("benchmark-config", Screen::BenchConfig),
        screen("benchmark-results", Screen::BenchResults),
        screen("gallery", Screen::Gallery),
        screen("goto", Screen::Goto),
        screen("share", Screen::Share),
        screen("report-issue", Screen::Report),
        screen("diagnostics", Screen::Diagnostics),
        screen("export", Screen::Export),
        screen("script-to-view", Screen::ScriptExport),
        screen("tour-render", Screen::TourRender),
        screen("reset-confirm", Screen::ResetConfirm),
        screen("notice", Screen::Notice),
        screen("palette-editor", Screen::PaletteEditor),
        // --- live render, one per mode (Direct <1e4, Df32Pert <1e28, Floatexp ≥1e28) ---
        live("live-direct-1e2", 2.0),
        live("live-df32-1e6", 6.0),
        live("live-df32-1e12", 12.0),
        live("live-floatexp-1e30", 30.0),
        // --- window sizing: the status bar wraps to a 2nd line when narrow (by design) but must
        //     stay stable at a fixed width (a waver between one and two lines = a repaint storm) ---
        Step {
            name: "window-wide".into(),
            kind: StepKind::Window(WindowSize {
                w: 1500.0, h: 850.0, expect_single_line: true, action: WindowAction::Resize,
            }),
        },
        Step {
            name: "window-medium".into(),
            // Not asserted single-line: the wrap threshold is content- and DPI-dependent, so this
            // width just reports its line count. The load-bearing check is height STABILITY.
            kind: StepKind::Window(WindowSize {
                w: 1100.0, h: 800.0, expect_single_line: false, action: WindowAction::Resize,
            }),
        },
        Step {
            name: "window-narrow".into(),
            kind: StepKind::Window(WindowSize {
                w: 680.0, h: 780.0, expect_single_line: false, action: WindowAction::Resize,
            }),
        },
        Step {
            name: "window-maximize-restore".into(),
            kind: StepKind::Window(WindowSize {
                w: 1280.0, h: 800.0, expect_single_line: false, action: WindowAction::MaximizeRestore,
            }),
        },
        Step {
            name: "window-rapid-resize".into(),
            kind: StepKind::Window(WindowSize {
                w: 1280.0, h: 800.0, expect_single_line: false, action: WindowAction::RapidResize,
            }),
        },
        // LAST, and deliberately so: this pair adopts the REPORTER'S settings (an explicit
        // iteration count, 2x AA) rather than the harness defaults, and those leaked forward the
        // first time it was written — the deep live step became heavy enough to miss its
        // screenshot timeout. Nothing follows them now, so nothing can inherit them.
        // Adjacent and in this order: the second step's check compares against the first's image.
        Step {
            name: "panel-toggle".into(),
            kind: StepKind::Window(WindowSize {
                w: 1280.0, h: 800.0, expect_single_line: false, action: WindowAction::TogglePanel,
            }),
        },
        screen("dual-formula-a", Screen::DualFormulaA),
        screen("dual-formula-b", Screen::DualFormulaB),
        screen("minimap-base", Screen::MinimapBase),
        screen("minimap-pan", Screen::MinimapPan),
        screen("minimap-pan-verify", Screen::MinimapPanVerify),
    ]
}

impl FractadyneApp {
    /// Whether a `--uitest` walk is active (used to suppress autosave etc.).
    pub(crate) fn uitest_active(&self) -> bool {
        self.uitest.is_some()
    }

    /// One frame of the `--uitest` state machine. Called early in `update()` once the GPU is up.
    /// Drives the walk to completion, then writes the bundle and exits the process.
    pub(crate) fn uitest_frame(&mut self, ctx: &egui::Context, gpu_name: Option<&str>) {
        // Keep the loop spinning without user input (headless has no events to wake it).
        ctx.request_repaint();

        let Some(mut ut) = self.uitest.take() else { return };
        if ut.gpu_name.is_none() {
            ut.gpu_name = gpu_name.map(|s| s.to_string());
        }

        match ut.phase {
            Phase::Setup => {
                let step = ut.steps[ut.idx].clone();
                self.uitest_apply(ctx, &step);
                // Screens settle fast; a live view must build its reference + settle its tiles; a
                // window resize needs a few frames for winit to apply the new size and egui to
                // re-lay-out. Hard caps keep a wedged build from ever hanging the harness.
                let (settle_ms, hard_ms) = match step.kind {
                    // The minimap trio renders an expensive view on purpose; give its grid time
                    // to finish, or the capture races the settle (see the `settling` gate below).
                    StepKind::Screen(
                        Screen::MinimapBase | Screen::MinimapPan | Screen::MinimapPanVerify,
                    ) => (500u64, 30_000u64),
                    StepKind::Screen(_) => (250u64, 3_000u64),
                    // Live: a generous hard cap so even a slow box's progressive reference build can
                    // finish (ref-settled gate) before the cap forces a capture.
                    StepKind::Live(_) => (2_500, 30_000),
                    StepKind::Window(_) => (1_800, 5_000),
                };
                ut.step_start = Instant::now();
                ut.settle_until = ut.step_start + std::time::Duration::from_millis(settle_ms);
                ut.hard_until = ut.step_start + std::time::Duration::from_millis(hard_ms);
                ut.sb_min = f32::INFINITY;
                ut.sb_max = 0.0;
                ut.ref_len_seen = 0;
                ut.ref_changed_at = ut.step_start;
                ut.panel_w = None;
                ut.phase = Phase::Settle;
            }
            Phase::Settle => {
                let now = Instant::now();
                // Track the status-bar height range, but ONLY after a transition guard: a window
                // resize legitimately changes the height for a frame or two while winit applies the
                // new size, and that must not count as a waver. Past the guard, the width is fixed,
                // so any remaining spread is a genuine one-line/two-line oscillation at a constant
                // width (the repaint-storm the user reported).
                let sb = self.perf.status_bar_h;
                if sb > 0.0 && now >= ut.step_start + std::time::Duration::from_millis(700) {
                    ut.sb_min = ut.sb_min.min(sb);
                    ut.sb_max = ut.sb_max.max(sb);
                }
                // Track the progressive reference build: note when its length last changed, so we
                // can tell when it has stopped growing (build finished).
                let ol = self.perf.last_orbit_len;
                if ol != ut.ref_len_seen {
                    ut.ref_len_seen = ol;
                    ut.ref_changed_at = now;
                }
                // A step is ready to screenshot once its minimum settle has elapsed AND, for a
                // perturbation live view, the reference orbit has FINISHED building — its length
                // has held steady for ~700ms. Waiting on build completeness (not on capped
                // fraction) makes the deep bands machine-independent: a slower box just takes longer
                // to reach the same resolved frame instead of screenshotting a half-built black one.
                // Direct mode has no reference and escapes immediately; screens/windows just wait
                // out the settle. The hard cap forces progress so a wedge can never hang the harness.
                // "Length stopped changing" is NOT sufficient on its own, and believing it was
                // produced the long-running "the deep view is hardware-dependent" mystery. The
                // reference build is PROGRESSIVE: it installs a coarse preview capped at
                // `COARSE_ITER` (16,384) and then keeps building the full orbit in the worker. At
                // 1e30x the full build takes many seconds, so the length sits at exactly 16,385
                // for far longer than this quiet period — the gate fires, and the harness
                // screenshots the ITERATION-CAPPED PREVIEW, which at a deep interior field is
                // solid black. Which stage you capture is then a race between one machine's build
                // speed and a fixed 700 ms window, which is exactly what "same view, different
                // result per machine" looked like. (Measured: 3070/Linux captured len 16,385 and
                // scored the band black; the 3080 captured 32,117 and scored it rich.)
                // So also require that NO reference build is in flight for this view — the worker
                // channel is the authoritative "still working" signal, the same lesson as the
                // beta.88 pacer fix (an in-flight build is progress the quiet-period can't see).
                // The control-panel toggle: once the OPEN layout has been drawn, record the two
                // widths and hide the panel. The rest of the settle then measures the reflow.
                if let StepKind::Window(w) = &ut.steps[ut.idx].kind {
                    if w.action == WindowAction::TogglePanel && ut.panel_w.is_none() {
                        if let (Some(p), Some(c)) =
                            (self.perf.layout.right_panel, self.perf.layout.central)
                        {
                            ut.panel_w = Some(p.width());
                            self.uitest_panel_w = Some(p.width());
                            self.uitest_central_w = Some(c.width());
                            self.dialogs.right_panel_open = false;
                            // Give the hidden layout its own settle rather than screenshotting
                            // the frame the toggle happened on.
                            ut.settle_until = now + std::time::Duration::from_millis(800);
                        }
                    }
                }
                let ref_building = self.recompute_rx[0].is_some();
                let ref_settled = ol > 0
                    && !ref_building
                    && now.duration_since(ut.ref_changed_at) >= std::time::Duration::from_millis(700);
                // ⚠A SCREEN STEP MUST NOT SCREENSHOT MID-SETTLE. The minimap-pan pair renders a
                // deliberately expensive view (an explicit 400,000 iterations at 2x AA), whose
                // tiled settle takes far longer than the 250 ms minimum — so the capture landed
                // on whatever fraction of the grid had completed, and a check comparing two such
                // frames read 2.8, 3.1 and 17.8 on successive runs of the SAME build. A flaky
                // gate is worse than no gate. Wait for the grid to drain (bounded, as ever, by
                // the hard cap, so a wedge still cannot hang the harness).
                let settling = self.perf.tile_pending[0]
                    || self.perf.chunk_pending[0]
                    || self.perf.tile_pending[1]
                    || self.perf.chunk_pending[1];
                let ready = now >= ut.settle_until
                    && match &ut.steps[ut.idx].kind {
                        StepKind::Live(v) => v.expect == RenderMode::Direct || ref_settled,
                        _ => !settling,
                    };
                if ready || now >= ut.hard_until {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                    ut.hard_until = now + std::time::Duration::from_millis(2_000); // screenshot-reply timeout
                    ut.phase = Phase::Shot;
                }
            }
            Phase::Shot => {
                let shot = ctx.input(|i| {
                    i.events.iter().find_map(|e| match e {
                        egui::Event::Screenshot { image, .. } => Some(image.clone()),
                        _ => None,
                    })
                });
                if let Some(image) = shot {
                    let step = ut.steps[ut.idx].clone();
                    let res = self.uitest_record(&ut, &step, &image);
                    ut.results.push(res);
                    ut.advance();
                } else if Instant::now() >= ut.hard_until {
                    // Reply never came — record a timeout (no image) and move on rather than hang.
                    let step = ut.steps[ut.idx].clone();
                    ut.results.push(timeout_result(&step));
                    ut.advance();
                }
            }
            Phase::Done => {
                let code = self.uitest_finish(&ut);
                crate::exit(code);
            }
        }

        self.uitest = Some(ut);
    }

    /// Put the app into the state a step wants to capture.
    fn uitest_apply(&mut self, ctx: &egui::Context, step: &Step) {
        self.uitest_close_all();
        match &step.kind {
            StepKind::Screen(s) => self.uitest_open_screen(ctx, *s),
            StepKind::Live(v) => self.uitest_set_live(ctx, v.decades),
            StepKind::Window(win) => {
                // Home view (so the status bar shows a normal centre readout), then resize. winit
                // applies the new inner size over the next frame or two — hence the longer settle.
                self.uitest_open_screen(ctx, Screen::Home);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(win.w, win.h)));
                match win.action {
                    WindowAction::Resize => {}
                    WindowAction::MaximizeRestore => {
                        // Maximize and come back. The settle that follows re-lays out the window,
                        // and every per-step check (layout, status bar, not-blank) then applies to
                        // the RESTORED frame — which is what checklist step 13 asks about.
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(win.w, win.h)));
                    }
                    WindowAction::RapidResize => {
                        // ~50 size changes with no settle between them. The failure this hunts is
                        // a crash or a wedged frame under churn, so the sizes only have to be
                        // varied and legal; the frame that gets checked is the one after it all.
                        for i in 0..50u32 {
                            let w = win.w - (i % 17) as f32 * 20.0;
                            let h = win.h - (i % 11) as f32 * 15.0;
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(win.w, win.h)));
                    }
                    WindowAction::TogglePanel => {
                        // OPEN it here; the settle loop measures the layout it produces and only
                        // then hides it. Measuring at this instant would read the PREVIOUS step's
                        // layout — and every window step opens the home screen, which closes the
                        // panel, so the widths were simply absent (measured: "no before/after
                        // canvas width recorded").
                        self.uitest_panel_w = None;
                        self.uitest_central_w = None;
                        self.dialogs.right_panel_open = true;
                    }
                }
            }
        }
    }

    /// Close every dialog/overlay so each screenshot shows one screen only.
    fn uitest_close_all(&mut self) {
        self.dialogs.help_open = false;
        self.dialogs.welcome_open = false;
        self.dialogs.bookmarks_open = false;
        self.dialogs.bench_open = false;
        self.dialogs.bench_dialog_open = false;
        self.dialogs.reset_confirm_open = false;
        self.dialogs.script_export_open = false;
        self.dialogs.notice = None;
        self.gallery.open = false;
        self.goto.open = false;
        self.share.open = false;
        self.report.open = false;
        self.export.open = false;
        self.tour_render.open = false;
        self.update_prompt_open = false;
        self.coloring.palette_editor_open = false;
    }

    fn uitest_open_screen(&mut self, ctx: &egui::Context, s: Screen) {
        match s {
            Screen::Home => {
                // Reset to the full set — a fresh, recognizable home. Without this, a step after a
                // deep live dive would inherit that (often black) camera instead of the fractal.
                self.reset_view();
                self.render_cfg.auto_iter = true;
                self.dialogs.right_panel_open = false;
                self.dialogs.minimap = false;
            }
            Screen::RightPanel => self.dialogs.right_panel_open = true,
            Screen::Minimap => self.dialogs.minimap = true,
            Screen::Help => self.dialogs.help_open = true,
            Screen::Welcome => self.dialogs.welcome_open = true,
            Screen::Bookmarks => self.dialogs.bookmarks_open = true,
            Screen::BenchConfig => self.dialogs.bench_dialog_open = true,
            Screen::BenchResults => {
                // Synthetic fixture so the populated results layout (Copy/Save/Run-again) renders.
                self.bench_report = Some(
                    "Fractadyne benchmark (synthetic UI-test fixture)\n\
                     iterate  12.3 ms   color  1.1 ms   1920x1080 ss1\n\
                     (values are placeholder — this screen validates layout, not perf)"
                        .to_string(),
                );
                self.dialogs.bench_open = true;
            }
            Screen::Gallery => self.gallery.open = true,
            Screen::Goto => self.goto.open = true,
            Screen::Share => self.share.open = true,
            Screen::Report => self.report.open = true,
            // Opened only — the walk must NOT start a test. A self-test child inside the UI walk
            // would contend for the same GPU the walk is rendering with, and turn a UI check into
            // a flaky performance test.
            Screen::Diagnostics => self.diagnostics.open = true,
            Screen::Export => self.export.open = true,
            Screen::ScriptExport => self.dialogs.script_export_open = true,
            Screen::TourRender => self.tour_render.open = true,
            Screen::ResetConfirm => self.dialogs.reset_confirm_open = true,
            Screen::Notice => {
                self.dialogs.notice = Some((
                    "UI-test notice".to_string(),
                    "This is the generic titled-message dialog, shown by the UI walk.".to_string(),
                ));
            }
            // Dual view, first formula. `set_fractal` resets the view to that formula's own
            // default centre, which is the state a user is in after picking from the dropdown.
            Screen::DualFormulaA => {
                // The REPORTER'S SETTINGS, not the defaults. This matters: the settle grid arms
                // far more eagerly under an EXPLICIT iteration count ("an explicit count arms
                // whenever the view is settled, converged or not"), and a completed grid is the
                // precondition for a pane that holds instead of redrawing. Auto-iter — the
                // harness default — waits for budget convergence and so may never be sitting in
                // the state the report describes.
                self.render_cfg.auto_iter = false;
                self.render_cfg.max_iter = 30_000;
                self.render_cfg.aa = 2;
                self.coloring.normalize_live = true;
                self.coloring.log_palette = true;
                self.set_fractal(crate::FractalKind::BurningShip);
                self.set_show_mode(crate::ShowMode::Both);
            }
            // ...and now switch the formula WITHOUT touching the canvas. Nothing else changes:
            // still dual, still settled. This is the reported gesture exactly.
            Screen::DualFormulaB => self.set_fractal(crate::FractalKind::Celtic),
            // A detailed single view, reached the ordinary way, and left to settle completely —
            // a COMPLETED tile grid is the precondition for the bug.
            Screen::MinimapBase => {
                self.set_show_mode(crate::ShowMode::Set);
                self.set_fractal(crate::FractalKind::Buffalo);
                // ⚠⚠A TILED SETTLE IS THE PRECONDITION, and forcing it took two attempts.
                // The default budget rendered the frame in one cheap dispatch — `tiling=false,
                // geo=None` under FRACTADYNE_TRACE=tile — so no grid existed to go stale and the
                // check passed at meanD 0.0 while proving nothing. Raising the budget under
                // AUTO-iter did not help either: the live cap held the dispatch at gpu_iter=3173,
                // and an auto-budgeted view only arms a grid once its budget has CONVERGED.
                // An EXPLICIT count arms whenever the view is settled, which is the state the
                // report was in. `tiled` below asserts it actually happened.
                self.render_cfg.auto_iter = false;
                self.render_cfg.max_iter = 400_000;
                self.render_cfg.aa = 2;
                self.viewport.set_center_mag(
                    fractadyne_core::BigFloat::from_f64(-0.139_634_789_365_238, 64),
                    fractadyne_core::BigFloat::from_f64(0.450_279_899_250_188, 64),
                    24.0,
                );
                self.pointer.settle_t = [-1.0e9, -1.0e9]; // settled, not interacting
            }
            // EXACTLY what the minimap's drag does, and nothing else: move the centre, cancel the
            // glide. No settle_t, no invalidate — that omission IS the bug under test.
            Screen::MinimapPan => {
                // THE REAL ACTION, not a copy of it: `minimap_pan` is what a drag on the overview
                // map calls. Re-implementing the gesture here would have tested the harness.
                let (cw, _) = self.viewport.complex_span();
                // ⚠The SAME clock `settle_t` is compared against — `ctx.input().time`, seconds
                // since start. A frame counter would read as an interaction that never ends.
                let now = ctx.input(|i| i.time);
                self.minimap_pan(cw * 0.35, 0.0, now);
            }
            // Same viewport, but now say the view was touched, which forces an honest full
            // re-render. Whatever this draws is what the step before it SHOULD have drawn.
            Screen::MinimapPanVerify => {
                self.perf.view_gen = [self.perf.frame_idx + 1, self.perf.frame_idx + 1];
            }
            Screen::PaletteEditor => {
                self.coloring.palette_editor_open = true;
                // Expand the paste-import section and seed it, so the walk covers that UI too
                // rather than only the stop rows it shares with every other run.
                self.coloring.paste_open = true;
                self.coloring.paste_text = "#000000, #8b1a1a, #ff8800, #ffe6b3".to_string();
            }
        }
    }

    /// Jump the live view to a magnification band on the canonical deep point and let the on-screen
    /// live path render it. Precision is sized for the depth; the mode is auto-picked downstream.
    pub(crate) fn uitest_set_live(&mut self, ctx: &egui::Context, decades: f64) {
        use std::f64::consts::LOG2_10;
        let log2mag = decades * LOG2_10;
        let prec = fractadyne_core::precision_for_octaves(log2mag.ceil() as u64) as usize;
        let cx = fractadyne_core::parse_bf_prec(LIVE_CX, prec)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(-0.75, prec));
        let cy = fractadyne_core::parse_bf_prec(LIVE_CY, prec)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.1, prec));
        // Fresh, dialog-free single view on the ADAPTIVE budget — the real, TDR-safe live path.
        // (Forcing a large explicit iteration count here risks a single heavy dispatch tripping the
        // GPU watchdog — the very device-loss class this app guards against; the harness must never
        // crash the GPU it is validating.) The settle loop instead waits for the adaptive budget to
        // climb until the view actually shows escaped structure (low capped fraction), bounded by a
        // hard cap, so the screenshot lands on detail rather than a mid-ramp black frame.
        self.uitest_open_screen(ctx, Screen::Home);
        self.playback = None;
        self.render_cfg.auto_iter = true;
        self.viewport.set_center_log2mag(cx, cy, log2mag);
        // Reset the live perf evidence so this step reads THIS view's numbers, not the last one's.
        self.perf.last_orbit_len = 0;
    }

    /// Capture a step: save the screenshot PNG and evaluate the checks.
    fn uitest_record(&self, ut: &UiTest, step: &Step, image: &egui::ColorImage) -> StepResult {
        let [w, h] = image.size;
        let (w, h) = (w as u32, h as u32);
        let file = format!("{:02}-{}.png", ut.idx, step.name);

        // sRGB8 bytes straight from the composited framebuffer (already display-space).
        let mut bytes = Vec::with_capacity(w as usize * h as usize * 4);
        for px in &image.pixels {
            bytes.extend_from_slice(&[px.r(), px.g(), px.b(), px.a()]);
        }
        let meta = format!("Fractadyne --uitest step {} ({})", step.name, env!("CARGO_PKG_VERSION"));
        let mut checks = Vec::new();
        match fractadyne_export::write_png_rgba8(&ut.out_dir.join(&file), w, h, &bytes, Some(&meta)) {
            Ok(()) => checks.push(pass("screenshot captured", format!("{w}x{h}"))),
            Err(e) => checks.push(Check {
                name: "screenshot captured".into(),
                verdict: Verdict::Fail,
                detail: format!("write failed: {e}"),
            }),
        }

        // Content-region stats: the central 60%, so menu/status chrome doesn't mask a blank view.
        let (mean_luma, luma_stddev, buckets) = centre_stats(&image.pixels, w as usize, h as usize);
        let left_fp = left_pane_fingerprint(&image.pixels, w as usize, h as usize);
        let tiled = self.tile_state_present(0);

        let is_live = matches!(step.kind, StepKind::Live(_));
        // "Not blank": a rendered UI or fractal has tonal spread. A flat frame is suspicious.
        // Shallow live views (Direct/Df32) MUST show structure at these coords — a black frame is a
        // real bug (FAIL). A deep Floatexp view can legitimately be dark near an interior minibrot,
        // and the adaptive budget may not fully ramp within the settle — ambiguous, so WARN. Pure
        // UI screens always have chrome, so a flat one is only a WARN.
        let deep_ambiguous =
            matches!(&step.kind, StepKind::Live(v) if v.expect == RenderMode::Floatexp);
        let blank = luma_stddev < 3.0 && buckets <= 2;
        if blank {
            checks.push(Check {
                name: "frame not blank".into(),
                verdict: if is_live && !deep_ambiguous { Verdict::Fail } else { Verdict::Warn },
                detail: format!("stddev {luma_stddev:.1}, {buckets} luma buckets — looks flat"),
            });
        } else {
            checks.push(pass("frame not blank", format!("stddev {luma_stddev:.1}, {buckets} buckets")));
        }

        let mut mode = None;
        let mut eff_iter = None;
        let mut orbit_len = None;
        let mut precision = None;
        let mut mag_log10 = None;
        let mut frame_ms = None;
        if let StepKind::Live(v) = &step.kind {
            let got = RenderMode::from_u32(self.perf.last_mode);
            mode = Some(format!("{got:?}"));
            eff_iter = Some(self.perf.last_eff_iter);
            orbit_len = Some(self.perf.last_orbit_len);
            precision = Some(self.perf.last_precision);
            mag_log10 = Some(self.viewport.log2_magnification() / std::f64::consts::LOG2_10);
            frame_ms = Some(self.perf.frame_ms);

            // The mode the app chose must match the mode this depth band exists to exercise.
            if got == v.expect {
                checks.push(pass("render mode matches depth", format!("{got:?}")));
            } else {
                checks.push(Check {
                    name: "render mode matches depth".into(),
                    verdict: Verdict::Fail,
                    detail: format!("expected {:?}, got {got:?}", v.expect),
                });
            }
            // A real iterate must have run. Perturbation modes build a reference orbit
            // (orbit_len > 0); Direct has none by design, so its absence is expected there.
            if v.expect == RenderMode::Direct {
                checks.push(pass("live iterate ran", "direct path — no reference orbit".into()));
            } else if self.perf.last_orbit_len > 0 {
                checks.push(pass("live iterate ran", format!("orbit_len {}", self.perf.last_orbit_len)));
            } else {
                checks.push(Check {
                    name: "live iterate ran".into(),
                    verdict: Verdict::Warn,
                    detail: "no reference orbit recorded within the settle window".into(),
                });
            }
        }

        // ---- layout invariants (checklist step 5) ----
        // The regions must exist, occupy real area, sit inside the window, and not overlap. These
        // hold on EVERY step, including the ones that open a dialog over the view, so a dialog
        // that pushed a panel off the window would be caught wherever it happened.
        let win = self.perf.layout;
        let regions = win.present();
        let screen = win.window;
        let mut layout_problems: Vec<String> = Vec::new();
        for (n, r) in &regions {
            if r.width() <= 1.0 || r.height() <= 1.0 {
                layout_problems.push(format!("{n} has no area ({:.0}x{:.0})", r.width(), r.height()));
            }
            if let Some(sc) = screen {
                // A half-pixel of slack: panel rects are laid out in points and rounded.
                if r.min.x < sc.min.x - 0.5
                    || r.min.y < sc.min.y - 0.5
                    || r.max.x > sc.max.x + 0.5
                    || r.max.y > sc.max.y + 0.5
                {
                    layout_problems.push(format!("{n} spills outside the window"));
                }
            }
        }
        for i in 0..regions.len() {
            for j in (i + 1)..regions.len() {
                let (a, b) = (regions[i].1, regions[j].1);
                let overlap = a.intersect(b);
                if overlap.width() > 0.5 && overlap.height() > 0.5 {
                    layout_problems.push(format!("{} overlaps {}", regions[i].0, regions[j].0));
                }
            }
        }
        // The two that must ALWAYS be there. The control panel is toggleable and the top bar can
        // be suppressed in fullscreen, so their absence is not automatically a fault — but a
        // window with no canvas or no status bar is.
        for must in ["central", "status bar"] {
            if !regions.iter().any(|(n, _)| *n == must) {
                layout_problems.push(format!("{must} did not draw"));
            }
        }
        if layout_problems.is_empty() {
            checks.push(pass(
                "layout-regions",
                format!(
                    "{} regions, none overlapping: {}",
                    regions.len(),
                    regions
                        .iter()
                        .map(|(n, r)| format!("{n} {:.0}x{:.0}", r.width(), r.height()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        } else {
            checks.push(Check {
                name: "layout-regions".into(),
                verdict: Verdict::Fail,
                detail: layout_problems.join("; "),
            });
        }

        // ---- status-bar readouts (checklist step 8) ----
        // Read from what the bar actually DREW this frame, not rebuilt here: a check that
        // re-derives the strings it is validating agrees with itself whatever the bar shows.
        let texts = &self.perf.status_bar_texts;
        let find = |prefix: &str| texts.iter().find(|t| t.trim_start().starts_with(prefix));
        let mut sb_problems: Vec<String> = Vec::new();
        for want in ["center ", "cursor ", "zoom", "iter "] {
            if find(want).is_none() {
                sb_problems.push(format!("no {want:?} readout"));
            }
        }
        // The centre is two coordinates. It is never absent — unlike the cursor, which reads `—`
        // when the pointer is off the canvas, and legitimately does here (the harness has no
        // pointer): what matters for that one is that its RESERVED FIELD is still drawn, since
        // the field appearing and disappearing is what wrapped the bar in the field report.
        if let Some(c) = find("center ") {
            if !c.contains(',') || c.trim_end().ends_with(',') {
                sb_problems.push(format!("centre readout is incomplete: {c:?}"));
            }
        }
        if let Some(c) = find("cursor ") {
            if c.len() < 40 {
                sb_problems.push(format!("cursor field is not reserved at its full width: {c:?}"));
            }
        }
        // Zoom and iteration must be numbers, and the iteration count must be a real budget.
        let num = |t: &str| -> Option<f64> {
            let digits: String = t
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == ' ')
                .filter(|c| *c != ',' && *c != ' ')
                .collect();
            digits.parse().ok()
        };
        match find("zoom").and_then(|t| num(t)) {
            Some(z) if z > 0.0 => {}
            other => sb_problems.push(format!("zoom does not read as a number: {other:?}")),
        }
        match find("iter ").and_then(|t| num(t)) {
            Some(i) if i >= 1.0 => {}
            other => sb_problems.push(format!("iteration count does not read as a number: {other:?}")),
        }
        if sb_problems.is_empty() {
            checks.push(pass("status-bar-populated", format!("{} readouts, all parse", texts.len())));
        } else {
            checks.push(Check {
                name: "status-bar-populated".into(),
                verdict: Verdict::Fail,
                detail: sb_problems.join("; "),
            });
        }

        // ---- the Performance readout (checklist step 67) ----
        // Only on a live step: these are measurements of a render, and on a dialog screen an
        // empty value is the truth rather than a fault.
        if is_live {
            let p = &self.perf;
            let mut perf_problems: Vec<String> = Vec::new();
            if !(p.frame_ms.is_finite() && p.frame_ms > 0.0) {
                perf_problems.push(format!("frame time {:.3}ms", p.frame_ms));
            }
            if !(p.cpu_ms.is_finite() && p.cpu_ms >= 0.0) {
                perf_problems.push(format!("cpu time {:.3}ms", p.cpu_ms));
            }
            if p.last_eff_iter == 0 {
                perf_problems.push("effective iterations 0".into());
            }
            if p.last_precision < 64 {
                perf_problems.push(format!("precision {} bits", p.last_precision));
            }
            if RenderMode::from_u32(p.last_mode) != RenderMode::Direct && p.last_orbit_len == 0 {
                perf_problems.push("orbit length 0 in a perturbation mode".into());
            }
            if perf_problems.is_empty() {
                checks.push(pass(
                    "perf-fields-populated",
                    format!(
                        "frame {:.1}ms, cpu {:.1}ms, eff_iter {}, precision {}b, orbit {}",
                        p.frame_ms, p.cpu_ms, p.last_eff_iter, p.last_precision, p.last_orbit_len
                    ),
                ));
            } else {
                checks.push(Check {
                    name: "perf-fields-populated".into(),
                    verdict: Verdict::Fail,
                    detail: perf_problems.join("; "),
                });
            }
        }

        // ---- switching formula in the dual view actually redraws the pane (steps 45-46) ----
        //
        // FIELD REPORT (2026-08-30): in dual view, picking a different formula left the LEFT pane
        // drawing the previous one — the dropdown, the panel heading and the minimap all updated,
        // the pane did not. A differential is the whole check: the two formulas are rendered at
        // their own default centres, so a working app cannot produce the same picture twice.
        if matches!(step.kind, StepKind::Screen(Screen::DualFormulaB)) {
            let before = ut.results.last().filter(|r| r.name == "dual-formula-a");
            match before {
                Some(a) => {
                    let d = fp_distance(&a.left_fp, &left_fp);
                    // Both frames must be non-flat as well, or "they differ" would be satisfied by
                    // one of them being blank — which is a different bug wearing this one's hat.
                    let both_real = luma_stddev >= 3.0 && a.luma_stddev >= 3.0;
                    if d >= 8.0 && both_real {
                        checks.push(pass(
                            "dual-formula-switch-redraws",
                            format!("left pane changed: meanD {d:.1} between the two formulas"),
                        ));
                    } else {
                        checks.push(Check {
                            name: "dual-formula-switch-redraws".into(),
                            verdict: Verdict::Fail,
                            detail: format!(
                                "left pane meanD {d:.1} after switching formula in dual view \
                                 (stddev {:.1} then {luma_stddev:.1}) — the pane is showing the \
                                 previous formula",
                                a.luma_stddev
                            ),
                        });
                    }
                }
                None => checks.push(Check {
                    name: "dual-formula-switch-redraws".into(),
                    verdict: Verdict::Warn,
                    detail: "no preceding dual-formula-a step to compare against".into(),
                }),
            }
        }

        // ---- panning from the minimap must actually redraw (field report 2026-08-30) ----
        if matches!(step.kind, StepKind::Screen(Screen::MinimapPanVerify)) {
            let panned = ut.results.last().filter(|r| r.name == "minimap-pan");
            match panned {
                Some(a) => {
                    let d = fp_distance(&a.left_fp, &left_fp);
                    // ⭐ANTI-VACUITY FIRST. The bug needs a tiled settle to have been in force;
                    // without one there is no grid to hold a stale tile, and "the frames match"
                    // is true for a reason that has nothing to do with the defect.
                    if !a.tiled {
                        checks.push(Check {
                            name: "minimap-pan-redraws".into(),
                            verdict: Verdict::Warn,
                            detail: format!(
                                "no settle grid was in force at the pan (meanD {d:.1}) — this run \
                                 did not exercise the defect, so its pass means nothing"
                            ),
                        });
                    // ⚠A DIFFERENTIAL WITH REAL NOISE, so the threshold is calibrated on samples
                    // rather than picked. Two independent re-renders of the same view never agree
                    // exactly (AA and where each settle happened to end), and the spread is wide:
                    //   broken: 12.9, 17.8, 19.6      fixed: 0.0, 2.8, 3.1, 4.2, 6.6
                    // 9.5 sits between the two populations. Read it as "large means stale", never
                    // as a precise measure — and if the fixed side ever creeps up, the answer is
                    // to quiet the noise (the settle gate above did most of that), not to raise
                    // this number until the check cannot fail.
                    } else if d <= 9.5 && luma_stddev >= 3.0 {
                        checks.push(pass(
                            "minimap-pan-redraws",
                            format!("panned frame matches an honest re-render (meanD {d:.1}, tiled)"),
                        ));
                    } else {
                        checks.push(Check {
                            name: "minimap-pan-redraws".into(),
                            verdict: Verdict::Fail,
                            detail: format!(
                                "panning from the minimap left a stale frame: meanD {d:.1} against \
                                 an honest re-render of the SAME view (stddev {luma_stddev:.1})"
                            ),
                        });
                    }
                }
                None => checks.push(Check {
                    name: "minimap-pan-redraws".into(),
                    verdict: Verdict::Warn,
                    detail: "no preceding minimap-pan step to compare against".into(),
                }),
            }
        }

        // ---- the Diagnostics dialog's system facts (checklist step 88) ----
        if matches!(step.kind, StepKind::Screen(Screen::Diagnostics)) {
            let si = &self.sysinfo;
            let mut d: Vec<String> = Vec::new();
            if si.cpu.trim().is_empty() {
                d.push("CPU brand is blank".into());
            }
            if si.physical == 0 || si.logical == 0 {
                d.push(format!("core count {}/{}", si.physical, si.logical));
            }
            // The arithmetic line must name a real backend list, never a placeholder.
            let arith = fractadyne_core::built_in_backends();
            if !arith.contains("astro-float") {
                d.push(format!("arithmetic line names no known backend: {arith:?}"));
            }
            if d.is_empty() {
                checks.push(pass(
                    "diagnostics-populated",
                    format!(
                        "CPU {:?}, {}/{} cores, arithmetic {arith}",
                        si.cpu, si.physical, si.logical
                    ),
                ));
            } else {
                checks.push(Check {
                    name: "diagnostics-populated".into(),
                    verdict: Verdict::Fail,
                    detail: d.join("; "),
                });
            }
        }

        // Window-sizing steps: report the status-bar height, whether it wrapped, and — the point
        // of the user's report — whether it WAVERED (height changed across a fixed-width settle).
        if let StepKind::Window(win) = &step.kind {
            // ⚠NO ASPECT CHECK HERE, deliberately. One was written and removed the same
            // hour: `complex_span` is `(units_per_pixel * width_px, units_per_pixel *
            // height_px)` — ONE scalar for both axes — so comparing the complex aspect
            // against the pixel aspect is identically 1 by construction and can never
            // fail. It ran green on all three window sizes and proved nothing. A real
            // check for checklist steps 11-12 has to measure the RENDERED IMAGE, because
            // a stretch could only enter in the shader or the present path, neither of
            // which the viewport can see.
            let one_line = if ut.sb_min.is_finite() { ut.sb_min } else { self.perf.status_bar_h };
            let waver = ut.sb_max - if ut.sb_min.is_finite() { ut.sb_min } else { ut.sb_max };
            if waver <= 1.5 {
                checks.push(pass("status bar height stable", format!("{:.0}px steady", ut.sb_max)));
            } else {
                checks.push(Check {
                    name: "status bar height stable".into(),
                    verdict: Verdict::Fail,
                    detail: format!(
                        "wavered {:.0}→{:.0}px at a fixed {}px width (repaint storm)",
                        ut.sb_min, ut.sb_max, win.w as u32
                    ),
                });
            }
            // Checklist step 10: hiding the control panel must hand its width to the canvas.
            // Measured against the rects the two layouts actually produced, so a panel that
            // "hid" by drawing itself transparently (or a canvas that did not reflow) fails.
            if win.action == WindowAction::TogglePanel {
                let now_w = self.perf.layout.central.map(|r| r.width());
                match (self.uitest_central_w, self.uitest_panel_w, now_w) {
                    (Some(before), Some(panel), Some(after)) => {
                        let gained = after - before;
                        // Within a couple of points of the panel's width: the divider and the
                        // panel's own frame are part of the space that comes back.
                        let ok = (gained - panel).abs() <= 4.0;
                        checks.push(if ok {
                            pass(
                                "panel-toggle-reflows",
                                format!("canvas {before:.0} -> {after:.0}px, panel was {panel:.0}px"),
                            )
                        } else {
                            Check {
                                name: "panel-toggle-reflows".into(),
                                verdict: Verdict::Fail,
                                detail: format!(
                                    "canvas gained {gained:.0}px when a {panel:.0}px panel was hidden"
                                ),
                            }
                        });
                        // ...and the panel must be genuinely gone, not merely narrow.
                        if self.perf.layout.right_panel.is_some() {
                            checks.push(Check {
                                name: "panel-toggle-reflows".into(),
                                verdict: Verdict::Fail,
                                detail: "the control panel still laid out after being hidden".into(),
                            });
                        }
                    }
                    _ => checks.push(Check {
                        name: "panel-toggle-reflows".into(),
                        verdict: Verdict::Warn,
                        detail: "no before/after canvas width recorded".into(),
                    }),
                }
            }

            // A "single line" is ~one text row + panel padding (~28px here). Two lines roughly
            // doubles it; use 40px as the split. Narrow windows are allowed to wrap.
            let two_lines = one_line > 40.0;
            if win.expect_single_line && two_lines {
                checks.push(Check {
                    name: "status bar fits one line".into(),
                    verdict: Verdict::Warn,
                    detail: format!("{one_line:.0}px — wrapped at {}px width", win.w as u32),
                });
            } else {
                checks.push(pass(
                    "status bar line count",
                    format!("{one_line:.0}px ({} line)", if two_lines { "2nd" } else { "single" }),
                ));
            }
        }

        StepResult {
            name: step.name.clone(),
            kind: match step.kind {
                StepKind::Live(_) => "live",
                StepKind::Window(_) => "window",
                StepKind::Screen(_) => "screen",
            },
            file,
            width: w,
            height: h,
            checks,
            mode,
            eff_iter,
            orbit_len,
            precision,
            mag_log10,
            frame_ms,
            mean_luma,
            luma_stddev,
            buckets,
            left_fp,
            tiled,
        }
    }

    /// Write the bundle (log.txt + report.md + report.json) and return the process exit code
    /// (0 = no FAIL, 1 = at least one FAIL).
    fn uitest_finish(&self, ut: &UiTest) -> i32 {
        let (mut pass_n, mut warn_n, mut fail_n) = (0u32, 0u32, 0u32);
        for r in &ut.results {
            match r.worst() {
                Verdict::Pass => pass_n += 1,
                Verdict::Warn => warn_n += 1,
                Verdict::Fail => fail_n += 1,
            }
        }
        // Checklist steps 1 and 102: no crash report may APPEAR during the walk. Taken as a
        // DIFFERENCE against the census at start, so a report an earlier session left in a
        // non-wiped config dir is not blamed on this run — and one written by this run cannot
        // hide behind it either.
        let new_crashes: Vec<String> = crate::diag::crash_report_names()
            .into_iter()
            .filter(|n| !ut.crashes_at_start.contains(n))
            .collect();
        // Checklist step 100, "the app exits cleanly — no hang, no crash dialog, no lingering
        // process". A harness cannot watch its own exit, but the NEXT launch can: every
        // deliberate shutdown goes through `crate::exit`, which disarms the unclean-exit marker,
        // so a marker still armed at startup means the previous session died. ⚠It only binds on a
        // profile that HAS a previous session — a wiped one has nothing to say, which is why this
        // reports three states rather than two.
        // This walk is about to exit deliberately, so record that before saying anything about
        // the previous one.
        if let Some(p) = walk_marker_path() {
            let _ = std::fs::write(p, "finished");
        }
        let exit_line = match (ut.prev_unclean, ut.prev_walk_clean) {
            (true, _) => "clean-exit-marker: FAIL — the previous session did not shut down cleanly",
            (_, Some(false)) => "clean-exit-marker: FAIL — the previous walk never reached its exit",
            (_, Some(true)) => "clean-exit-marker: PASS (the previous walk in this profile exited cleanly)",
            (_, None) => "clean-exit-marker: (no previous walk in this profile — nothing to check)",
        };
        let plat = format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH);
        let gpu = ut.gpu_name.clone().unwrap_or_else(|| "(unknown)".into());
        let version = env!("CARGO_PKG_VERSION");

        let _ = std::fs::create_dir_all(&ut.out_dir);
        let mut log = String::new();
        log.push_str(&format!(
            "Fractadyne UI validation — v{version}\nplatform: {plat}\nGPU: {gpu}\n\
             steps: {} ({pass_n} pass / {warn_n} warn / {fail_n} fail)\n\
             no-crash-files: {}\n{exit_line}\n\n",
            ut.results.len(),
            if new_crashes.is_empty() {
                "PASS".to_string()
            } else {
                format!("FAIL — {}", new_crashes.join(", "))
            },
        ));
        for r in &ut.results {
            log.push_str(&format!("[{}] {} ({})  {}\n", r.worst().tag(), r.name, r.kind, r.file));
            if let Some(m) = &r.mode {
                log.push_str(&format!(
                    "    mode={m} eff_iter={} orbit_len={} precision={} mag=1e{:.1} frame={:.1}ms\n",
                    r.eff_iter.unwrap_or(0),
                    r.orbit_len.unwrap_or(0),
                    r.precision.unwrap_or(0),
                    r.mag_log10.unwrap_or(0.0),
                    r.frame_ms.unwrap_or(0.0),
                ));
            }
            log.push_str(&format!(
                "    luma mean={:.1} stddev={:.1} buckets={}/32\n",
                r.mean_luma, r.luma_stddev, r.buckets
            ));
            for c in &r.checks {
                log.push_str(&format!("    - {:<26} [{}] {}\n", c.name, c.verdict.tag(), c.detail));
            }
            log.push('\n');
        }
        let _ = std::fs::write(ut.out_dir.join("log.txt"), &log);
        let _ = std::fs::write(ut.out_dir.join("report.md"), render_md(ut, version, &plat, &gpu, pass_n, warn_n, fail_n));
        let _ = std::fs::write(ut.out_dir.join("report.json"), render_json(ut, version, &plat, &gpu));

        eprintln!(
            "\n=== --uitest complete: {} steps, {pass_n} pass / {warn_n} warn / {fail_n} fail ===",
            ut.results.len()
        );
        eprintln!("{exit_line}");
        if new_crashes.is_empty() {
            eprintln!("no-crash-files: PASS (no crash report appeared during the walk)");
        } else {
            eprintln!("no-crash-files: FAIL — {}", new_crashes.join(", "));
        }
        eprintln!("bundle: {}", ut.out_dir.display());
        // A crash report that appeared during the walk fails the RUN, not just a step: the
        // harness's own exit code is what a gate script reads, and steps can all pass while the
        // app panicked on a worker thread.
        if fail_n > 0 || !new_crashes.is_empty() || ut.prev_unclean || ut.prev_walk_clean == Some(false) {
            1
        } else {
            0
        }
    }
}

impl UiTest {
    fn advance(&mut self) {
        self.idx += 1;
        self.phase = if self.idx < self.steps.len() { Phase::Setup } else { Phase::Done };
    }
}

fn pass(name: &str, detail: String) -> Check {
    Check { name: name.to_string(), verdict: Verdict::Pass, detail }
}

fn timeout_result(step: &Step) -> StepResult {
    StepResult {
        name: step.name.clone(),
        kind: match step.kind {
            StepKind::Live(_) => "live",
            StepKind::Window(_) => "window",
            StepKind::Screen(_) => "screen",
        },
        file: String::new(),
        width: 0,
        height: 0,
        checks: vec![Check {
            name: "screenshot captured".into(),
            verdict: Verdict::Fail,
            detail: "no Event::Screenshot reply within the timeout".into(),
        }],
        mode: None,
        eff_iter: None,
        orbit_len: None,
        precision: None,
        mag_log10: None,
        frame_ms: None,
        mean_luma: 0.0,
        luma_stddev: 0.0,
        buckets: 0,
        left_fp: Vec::new(),
        tiled: false,
    }
}

/// Luma statistics over the central 60% of the image (crops out menu/status-bar chrome), so a
/// blank fractal isn't masked by a populated frame border. Returns (mean, stddev, non-empty
/// buckets out of 32).
fn centre_stats(pixels: &[egui::Color32], w: usize, h: usize) -> (f64, f64, usize) {
    if w == 0 || h == 0 || pixels.len() < w * h {
        return (0.0, 0.0, 0);
    }
    let (x0, x1) = ((w as f64 * 0.2) as usize, (w as f64 * 0.8) as usize);
    let (y0, y1) = ((h as f64 * 0.2) as usize, (h as f64 * 0.8) as usize);
    let mut sum = 0.0f64;
    let mut sum2 = 0.0f64;
    let mut n = 0u64;
    let mut buckets = [false; 32];
    for y in y0..y1 {
        for x in x0..x1 {
            let p = pixels[y * w + x];
            let l = 0.299 * p.r() as f64 + 0.587 * p.g() as f64 + 0.114 * p.b() as f64;
            sum += l;
            sum2 += l * l;
            n += 1;
            buckets[((l / 256.0 * 32.0) as usize).min(31)] = true;
        }
    }
    if n == 0 {
        return (0.0, 0.0, 0);
    }
    let mean = sum / n as f64;
    let var = (sum2 / n as f64 - mean * mean).max(0.0);
    (mean, var.sqrt(), buckets.iter().filter(|b| **b).count())
}

fn render_md(
    ut: &UiTest,
    version: &str,
    plat: &str,
    gpu: &str,
    pass_n: u32,
    warn_n: u32,
    fail_n: u32,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("# Fractadyne UI validation — v{version}\n\n"));
    s.push_str(&format!(
        "- **platform**: {plat}\n- **GPU**: {gpu}\n- **result**: {pass_n} pass / {warn_n} warn / {fail_n} fail across {} steps\n\n",
        ut.results.len()
    ));
    s.push_str("| # | step | kind | verdict | mode | size | screenshot |\n");
    s.push_str("|---|------|------|---------|------|------|------------|\n");
    for (i, r) in ut.results.iter().enumerate() {
        let img = if r.file.is_empty() { "—".to_string() } else { format!("![{0}]({0})", r.file) };
        s.push_str(&format!(
            "| {i} | {} | {} | {} | {} | {}x{} | {img} |\n",
            r.name,
            r.kind,
            r.worst().tag(),
            r.mode.clone().unwrap_or_else(|| "—".into()),
            r.width,
            r.height,
        ));
    }
    s.push_str("\n## Checks\n\n");
    for r in &ut.results {
        s.push_str(&format!("### {} — {}\n\n", r.name, r.worst().tag()));
        for c in &r.checks {
            s.push_str(&format!("- **{}** [{}] {}\n", c.name, c.verdict.tag(), c.detail));
        }
        s.push('\n');
    }
    s
}

fn render_json(ut: &UiTest, version: &str, plat: &str, gpu: &str) -> String {
    // Hand-rolled JSON (no serde derive needed for this flat shape).
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let mut s = String::from("{\n");
    s.push_str(&format!("  \"version\": \"{}\",\n", esc(version)));
    s.push_str(&format!("  \"platform\": \"{}\",\n", esc(plat)));
    s.push_str(&format!("  \"gpu\": \"{}\",\n", esc(gpu)));
    s.push_str("  \"steps\": [\n");
    for (i, r) in ut.results.iter().enumerate() {
        s.push_str("    {\n");
        s.push_str(&format!("      \"name\": \"{}\",\n", esc(&r.name)));
        s.push_str(&format!("      \"kind\": \"{}\",\n", r.kind));
        s.push_str(&format!("      \"verdict\": \"{}\",\n", r.worst().tag()));
        s.push_str(&format!("      \"file\": \"{}\",\n", esc(&r.file)));
        s.push_str(&format!("      \"width\": {}, \"height\": {},\n", r.width, r.height));
        if let Some(m) = &r.mode {
            s.push_str(&format!("      \"mode\": \"{}\",\n", esc(m)));
            s.push_str(&format!("      \"eff_iter\": {},\n", r.eff_iter.unwrap_or(0)));
            s.push_str(&format!("      \"orbit_len\": {},\n", r.orbit_len.unwrap_or(0)));
            s.push_str(&format!("      \"precision\": {},\n", r.precision.unwrap_or(0)));
            s.push_str(&format!("      \"mag_log10\": {:.3},\n", r.mag_log10.unwrap_or(0.0)));
            s.push_str(&format!("      \"frame_ms\": {:.3},\n", r.frame_ms.unwrap_or(0.0)));
        }
        s.push_str(&format!(
            "      \"luma_mean\": {:.2}, \"luma_stddev\": {:.2}, \"buckets\": {},\n",
            r.mean_luma, r.luma_stddev, r.buckets
        ));
        s.push_str("      \"checks\": [\n");
        for (j, c) in r.checks.iter().enumerate() {
            let comma = if j + 1 < r.checks.len() { "," } else { "" };
            s.push_str(&format!(
                "        {{ \"name\": \"{}\", \"verdict\": \"{}\", \"detail\": \"{}\" }}{comma}\n",
                esc(&c.name),
                c.verdict.tag(),
                esc(&c.detail)
            ));
        }
        s.push_str("      ]\n");
        let comma = if i + 1 < ut.results.len() { "," } else { "" };
        s.push_str(&format!("    }}{comma}\n"));
    }
    s.push_str("  ]\n}\n");
    s
}
