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
    Export,
    ScriptExport,
    TourRender,
    ResetConfirm,
    Notice,
    PaletteEditor,
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
    // once this stops growing (the progressive reference build has finished), so a deep band lands
    // on the SAME resolved frame regardless of how fast the machine builds it. (On a 3070 the ref
    // reached 2.0M and showed structure; on a 3080 a capped-fraction gate fired the hard cap at a
    // partial 30k ref and captured black — this makes it machine-independent.)
    ref_len_seen: u32,
    ref_changed_at: Instant,
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
        }
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
        kind: StepKind::Live(LiveView { decades, expect: RenderMode::select(true, 10f64.powf(decades)) }),
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
            kind: StepKind::Window(WindowSize { w: 1500.0, h: 850.0, expect_single_line: true }),
        },
        Step {
            name: "window-medium".into(),
            // Not asserted single-line: the wrap threshold is content- and DPI-dependent, so this
            // width just reports its line count. The load-bearing check is height STABILITY.
            kind: StepKind::Window(WindowSize { w: 1100.0, h: 800.0, expect_single_line: false }),
        },
        Step {
            name: "window-narrow".into(),
            kind: StepKind::Window(WindowSize { w: 680.0, h: 780.0, expect_single_line: false }),
        },
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
                let ref_settled = ol > 0
                    && now.duration_since(ut.ref_changed_at) >= std::time::Duration::from_millis(700);
                let ready = now >= ut.settle_until
                    && match &ut.steps[ut.idx].kind {
                        StepKind::Live(v) => v.expect == RenderMode::Direct || ref_settled,
                        _ => true,
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
            StepKind::Screen(s) => self.uitest_open_screen(*s),
            StepKind::Live(v) => self.uitest_set_live(v.decades),
            StepKind::Window(win) => {
                // Home view (so the status bar shows a normal centre readout), then resize. winit
                // applies the new inner size over the next frame or two — hence the longer settle.
                self.uitest_open_screen(Screen::Home);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(win.w, win.h)));
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

    fn uitest_open_screen(&mut self, s: Screen) {
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
            Screen::PaletteEditor => self.coloring.palette_editor_open = true,
        }
    }

    /// Jump the live view to a magnification band on the canonical deep point and let the on-screen
    /// live path render it. Precision is sized for the depth; the mode is auto-picked downstream.
    fn uitest_set_live(&mut self, decades: f64) {
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
        self.uitest_open_screen(Screen::Home);
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

        // Window-sizing steps: report the status-bar height, whether it wrapped, and — the point
        // of the user's report — whether it WAVERED (height changed across a fixed-width settle).
        if let StepKind::Window(win) = &step.kind {
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
        let plat = format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH);
        let gpu = ut.gpu_name.clone().unwrap_or_else(|| "(unknown)".into());
        let version = env!("CARGO_PKG_VERSION");

        let _ = std::fs::create_dir_all(&ut.out_dir);
        let mut log = String::new();
        log.push_str(&format!(
            "Fractadyne UI validation — v{version}\nplatform: {plat}\nGPU: {gpu}\n\
             steps: {} ({pass_n} pass / {warn_n} warn / {fail_n} fail)\n\n",
            ut.results.len()
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
        eprintln!("bundle: {}", ut.out_dir.display());
        if fail_n > 0 { 1 } else { 0 }
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
