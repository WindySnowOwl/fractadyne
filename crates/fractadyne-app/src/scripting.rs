//! Scripting & benchmark: TOML keyframe camera tours (`Tools -> Play script`) and the
//! built-in benchmark tour, plus the playback engine that glides center+zoom along the
//! timeline and samples FPS/CPU/RAM. `Playback`/`Bench` are pub(crate) (held as app state).

use crate::{now_utc_string, process_memory, version_string, FractadyneApp, FractalKind};
use serde::Deserialize;

/// Script schema version. **v2 is a breaking restructure of v1** — absolute keyframe times, one
/// `[[annotation]]` array, `[render]`, `[[location]]`, `zoom` strings, per-keyframe budgets — and
/// there is deliberately no v1 reader: a v1 script is rejected with a migration message rather
/// than silently mis-played (its `secs`/`mag` keys simply don't exist in v2, so every keyframe
/// would land at t=0, zoom 1). Purely additive new keys don't need a bump (unknown keys are
/// ignored, missing ones default). A script whose `format_version` exceeds this is from a newer
/// build: we still play the parts we understand but warn that newer features may not apply.
pub(crate) const SCRIPT_FORMAT_VERSION: u32 = 2;

/// On-disk script format v2 (TOML). Keyframe `t` is the ABSOLUTE second the camera arrives (so
/// inserting a keyframe can't desync downstream narration), every element takes a stable `id`,
/// annotations share one array tagged by `kind`, and output settings live in `[render]`.
#[derive(Deserialize, Default)]
struct ScriptFile {
    #[serde(default)]
    name: String,
    /// Schema version the script was authored for. Gated by `check_format_version` *before*
    /// deserializing (a v1 file must not reach serde at all), so it's declared here only to
    /// document the key and keep the struct a faithful picture of the format.
    #[serde(default)]
    #[allow(dead_code)]
    format_version: Option<u32>,
    #[serde(default, rename = "loop")]
    loop_: bool,
    /// Output settings, so `--render-tour x.toml` with no flags reproduces the intended render
    /// and CLI flags merely override.
    #[serde(default)]
    render: RenderFile,
    /// Live-playback behaviour (pacing).
    #[serde(default)]
    playback: PlaybackFile,
    /// Named coordinates, referenced by `location = "id"` from keyframes and annotations.
    #[serde(default)]
    location: Vec<LocationFile>,
    /// Named palettes, referenced by `palette = "id"` from keyframes.
    #[serde(default)]
    palette: Vec<PaletteFile>,
    /// Chapters, so one can be rendered or scrubbed in isolation (`--segment`).
    #[serde(default)]
    segment: Vec<SegmentFile>,
    #[serde(default)]
    keyframe: Vec<KeyframeFile>,
    #[serde(default)]
    annotation: Vec<AnnotationFile>,
    /// Reserved for editor-only state (selection, timeline zoom …) so an editor needs no sidecar
    /// file. Parsed and ignored by the player.
    #[serde(default)]
    #[allow(dead_code)]
    editor: Option<toml::Value>,
}

/// A number that may be written as a TOML number or a string — `zoom = 8` and `zoom = "6.5e94"`
/// both work, and only the string form survives past f64's ~1e308 ceiling.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum NumOrStr {
    Num(f64),
    Str(String),
}

impl NumOrStr {
    fn as_string(&self) -> String {
        match self {
            NumOrStr::Num(n) => format!("{n}"),
            NumOrStr::Str(s) => s.clone(),
        }
    }
}

/// `mp4 = true` (default path next to the frames) or `mp4 = "movie.mp4"`.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum Mp4Spec {
    Flag(bool),
    Path(String),
}

/// The `[playback]` table: how the tour behaves in the LIVE view (`Tools -> Play script`).
/// Nothing here affects an offline render, which always renders every frame to completion.
#[derive(Deserialize, Default, Clone)]
struct PlaybackFile {
    #[serde(default)]
    pace: Option<String>,
    /// Seconds a `settled` hold may wait for the view to resolve before giving up and moving on.
    #[serde(default)]
    settle_timeout: Option<f64>,
}

/// How the live playback clock treats a renderer that can't keep up.
#[derive(Clone, Copy, PartialEq, Default)]
pub(crate) enum Pace {
    /// Never dilate: the tour runs on the wall clock and shows whatever is ready. What a
    /// benchmark wants — the measurement IS how much the machine got done in real time.
    Realtime,
    /// Dilate the clock while the reference pipeline lags, so a dive slows instead of blurring
    /// into a stale reprojection. The default.
    #[default]
    Adaptive,
    /// Adaptive, plus: at a keyframe HOLD, stop the clock until the view has actually resolved
    /// (or `settle_timeout` elapses). The hold is where the viewer is looking at a still frame,
    /// and at depth the adaptive iteration budget needs several settled frames to climb to what
    /// the field needs — more than a 3-second hold of wall clock. Correctness over duration:
    /// what a demo reel or a regression pass wants.
    Settled,
}

impl Pace {
    fn parse(s: Option<&str>) -> Result<Pace, String> {
        match s.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
            None | Some("adaptive") => Ok(Pace::Adaptive),
            Some("realtime" | "real-time") => Ok(Pace::Realtime),
            Some("settled" | "quality") => Ok(Pace::Settled),
            Some(other) => Err(format!(
                "[playback] pace = \"{other}\": expected realtime, adaptive, or settled"
            )),
        }
    }
}

/// The `[render]` table: how this tour is meant to be rendered.
#[derive(Deserialize, Default, Clone)]
struct RenderFile {
    /// `"1920x1080"` or a bare width (`"1920"`).
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    fps: Option<f64>,
    #[serde(default)]
    ss: Option<u32>,
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    out: Option<String>,
    #[serde(default)]
    mp4: Option<Mp4Spec>,
    /// Iteration budget when a keyframe doesn't state its own. Without this a deep tour is at the
    /// mercy of whatever the viewer happens to have set, and auto-iter's depth formula
    /// (~220/octave) under-budgets hard fields badly: the Misiurewicz three-spar gets ~46k at
    /// 1e61× and ~71k at 1e95× where it measurably needs 222k and millions — every deep frame
    /// renders FLAT.
    #[serde(default)]
    max_iter: Option<u32>,
    /// Whether `max_iter` is a BASE that still scales with depth (`true`) or an exact count
    /// (`false`). Per-keyframe `max_iter` is always exact.
    #[serde(default)]
    auto_iter: Option<bool>,
    /// Burn a small zoom/coordinate HUD into the top-left of every frame (also `--show-location`).
    #[serde(default)]
    show_location: Option<bool>,
    /// Auto-normalize the palette cycle to each frame's escape-value range, temporally smoothed
    /// across frames so a video doesn't shimmer (the `--normalize` idea, made tour-safe). Off by
    /// default; deep tours want it on — a fixed cycle aliases the ~1e5–1e6 smooth-iter field into
    /// per-pixel confetti past ~1e50×.
    #[serde(default)]
    normalize: Option<bool>,
}

/// A named coordinate — kills the 120-digit duplication when several keyframes and annotations
/// share a dive center, and doubles as an editor's location picker.
#[derive(Deserialize, Clone)]
struct LocationFile {
    id: String,
    re: String,
    im: String,
    /// Default magnification for keyframes that name this location without their own `zoom`.
    #[serde(default)]
    zoom: Option<NumOrStr>,
    /// Preview image path, relative to the script (reserved: generated by the app's thumbnail
    /// cache; not read by the player).
    #[serde(default)]
    #[allow(dead_code)]
    thumb: Option<String>,
    /// Free-text note for the author (ignored).
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
}

/// A named palette: a built-in preset by name/index, or explicit gradient stops.
#[derive(Deserialize, Clone)]
struct PaletteFile {
    id: String,
    /// Built-in preset name (e.g. "Ember") or index.
    #[serde(default)]
    preset: Option<String>,
    /// Explicit gradient stops, ascending by `at` (0..1).
    #[serde(default)]
    stops: Vec<StopFile>,
}

/// One gradient stop: `{ at = 0.4, color = "#e04c0a" }`.
#[derive(Deserialize, Clone)]
struct StopFile {
    at: f64,
    color: String,
}

/// A chapter of the timeline.
#[derive(Deserialize, Clone)]
struct SegmentFile {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    /// Absolute start time (seconds); the chapter runs to the next segment's `t` (or the end).
    #[serde(default)]
    t: f64,
}

/// A timed overlay. One array for all kinds so an editor sees a single track list and new kinds
/// are additive rather than a new top-level array each time.
#[derive(Deserialize, Clone)]
struct AnnotationFile {
    /// `caption`, `callout`, or `spotlight`.
    kind: String,
    #[serde(default)]
    #[allow(dead_code)]
    id: Option<String>,
    /// When it appears (absolute seconds).
    #[serde(default)]
    t: f64,
    /// How long it stays; 0 or omitted ⇒ until the tour ends.
    #[serde(default)]
    secs: f64,
    /// Fade in/out time (seconds) at each end. Default 0.4.
    #[serde(default)]
    fade: Option<f64>,
    // --- caption / callout ---
    /// The text to show (supports `\n` for multiple lines). Required for captions and callouts.
    #[serde(default)]
    text: Option<String>,
    /// Caption screen anchor: `top`, `center`, or `bottom` (default).
    #[serde(default)]
    pos: Option<String>,
    /// Font size in points.
    #[serde(default)]
    size: Option<f64>,
    // --- callout / spotlight anchor ---
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    re: Option<String>,
    #[serde(default)]
    im: Option<String>,
    // --- spotlight ---
    /// Circle radius as a fraction of the frame height (default 0.25).
    #[serde(default)]
    radius: Option<f64>,
    /// Soft-edge width as a fraction of the frame height (default 0.08).
    #[serde(default)]
    softness: Option<f64>,
    /// How dark outside the circle, 0..1 (default 0.7).
    #[serde(default)]
    dim: Option<f64>,
}

/// Where a caption sits on screen.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CaptionPos {
    Top,
    Center,
    Bottom,
}

/// A resolved caption ready to draw.
pub(crate) struct Caption {
    pub(crate) text: String,
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) fade: f64,
    pub(crate) pos: CaptionPos,
    pub(crate) size: f32,
}

/// Eased fade opacity (0..1) for a timed annotation window `[start, end]` at tour time `t`.
/// `(width, height)` of a COMPLETE PNG, or `None` if the file is missing, malformed, or truncated.
///
/// Reads only the head and tail: the 8-byte signature, the `IHDR` dimensions that follow it, and
/// the 12-byte `IEND` trailer that a writer emits last. That trailer is the point — a file cut off
/// by a full disk or a killed process has everything except its ending, so its absence is the
/// signal, and checking it costs two seeks rather than decoding a 4K image.
fn png_frame_size(path: &std::path::Path) -> Option<(u32, u32)> {
    use std::io::{Read, Seek, SeekFrom};
    const MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    // 8 magic + 8 (IHDR length+type) + 8 (w,h) = 24, and a 12-byte IEND cannot overlap them.
    const IEND: [u8; 8] = [0, 0, 0, 0, b'I', b'E', b'N', b'D'];
    let mut f = std::fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    if len < 24 + 12 {
        return None;
    }
    let mut head = [0u8; 24];
    f.read_exact(&mut head).ok()?;
    if head[..8] != MAGIC || &head[12..16] != b"IHDR" {
        return None;
    }
    f.seek(SeekFrom::End(-12)).ok()?;
    let mut tail = [0u8; 8];
    f.read_exact(&mut tail).ok()?;
    if tail != IEND {
        return None;
    }
    let w = u32::from_be_bytes([head[16], head[17], head[18], head[19]]);
    let h = u32::from_be_bytes([head[20], head[21], head[22], head[23]]);
    (w > 0 && h > 0).then_some((w, h))
}

/// A finished tour frame handed to the background encoder pool for PNG compression.
struct EncodeJob {
    path: std::path::PathBuf,
    w: u32,
    h: u32,
    px: Vec<f32>,
    fi: u64,
}

/// The user's answer to an overwrite prompt.
enum OverwriteChoice {
    /// Overwrite this file.
    Yes,
    /// Overwrite this file and all later collisions without asking again.
    YesAll,
    /// Keep the existing file (skip this frame).
    No,
    /// Abort the render.
    Quit,
}

/// Ask on the terminal whether to overwrite `path`: `[y]es / [a]ll / [n]o / [q]uit` (loops on
/// invalid input). If stdin isn't a terminal (piped / no console), returns an error pointing at
/// `--overwrite` instead of blocking — so an automated run never hangs waiting for a keypress.
fn prompt_overwrite(path: &std::path::Path) -> Result<OverwriteChoice, String> {
    use std::io::{BufRead, IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Err(format!(
            "{} already exists; pass --overwrite (or -y) to replace, or use an empty --out directory",
            path.display()
        ));
    }
    loop {
        print!("Overwrite {}? [y]es / [a]ll / [n]o / [q]uit: ", path.display());
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if std::io::stdin().lock().read_line(&mut line).unwrap_or(0) == 0 {
            return Ok(OverwriteChoice::Quit); // EOF → treat as quit
        }
        match line.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(OverwriteChoice::Yes),
            "a" | "all" | "ya" => return Ok(OverwriteChoice::YesAll),
            "n" | "no" => return Ok(OverwriteChoice::No),
            "q" | "quit" => return Ok(OverwriteChoice::Quit),
            _ => println!("  please enter y (yes), a (yes to all), n (no), or q (quit)"),
        }
    }
}

/// Pipe-safe progress line for the tour RENDER path. `println!` PANICS when stdout closes
/// ("failed printing to stdout: The pipe is being closed") — and the render usually runs as a
/// child process of the GUI with stdout piped, so the parent exiting mid-render used to KILL a
/// headless render that needed nothing from it (observed in a beta.45 crash report; the prime
/// suspect in the silent 4K death at frame 5682/9931). Progress is best-effort: write, ignore a
/// dead pipe, and mirror to the render log so a parentless child still leaves a trail on disk.
fn say(msg: &str) {
    use std::io::Write;
    let _ = writeln!(std::io::stdout(), "{msg}");
    crate::diag::log_line("render", msg);
}

/// On-disk render status marker: `<out_dir>/render-status.txt`. The 2026-08-08 4K render died
/// silently at frame 5682/9931 — no panic, no report, detected only by the session-global
/// unclean-exit marker — so the OUTPUT DIRECTORY itself now records the render's state:
/// `running` (with pid, so a live render is distinguishable from a corpse), then `complete` /
/// `canceled` / `failed: <why>`. A stale `running` whose pid is gone IS the silent-death
/// signature, diagnosable from the frames folder alone — and the Render Script dialog's planned
/// progress bar can read the same file. Best-effort by design: a full disk must not take down a
/// render that is otherwise succeeding.
fn write_render_status(out_dir: &std::path::Path, state: &str) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("{state}\npid {}\n{}\n", std::process::id(), crate::sysinfo::utc_string(now));
    let _ = std::fs::write(out_dir.join("render-status.txt"), line);
}

/// Format a duration in seconds as a compact `1h02m03s` / `2m03s` / `4.2s` string for progress logs.
fn fmt_hms(secs: f64) -> String {
    let s = secs.max(0.0);
    if s < 60.0 {
        return format!("{s:.1}s");
    }
    let total = s.round() as u64;
    let (h, m, sec) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}h{m:02}m{sec:02}s")
    } else {
        format!("{m}m{sec:02}s")
    }
}

fn fade_alpha(t: f64, start: f64, end: f64, fade: f64) -> f32 {
    if t < start || t > end {
        return 0.0;
    }
    let f = fade.max(1.0e-3);
    let a = ((t - start) / f).min(1.0);
    let b = ((end - t) / f).min(1.0);
    (a.min(b).clamp(0.0, 1.0)) as f32
}

impl Caption {
    /// Opacity (0..1) of this caption at tour time `t`, with eased fade in/out; 0 = not shown.
    pub(crate) fn alpha_at(&self, t: f64) -> f32 {
        fade_alpha(t, self.start, self.end, self.fade)
    }
}

/// A labeled marker anchored to a fractal coordinate — tracks the point as the view pans/zooms.
pub(crate) struct Callout {
    pub(crate) text: String,
    pub(crate) cx: fractadyne_core::BigFloat,
    pub(crate) cy: fractadyne_core::BigFloat,
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) fade: f64,
    pub(crate) size: f32,
}

impl Callout {
    pub(crate) fn alpha_at(&self, t: f64) -> f32 {
        fade_alpha(t, self.start, self.end, self.fade)
    }
}

/// A spotlight vignette anchored to a fractal coordinate (dims everything outside a soft circle).
pub(crate) struct Spotlight {
    pub(crate) cx: fractadyne_core::BigFloat,
    pub(crate) cy: fractadyne_core::BigFloat,
    pub(crate) radius: f32,
    pub(crate) soft: f32,
    pub(crate) dim: f32,
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) fade: f64,
}

impl Spotlight {
    pub(crate) fn alpha_at(&self, t: f64) -> f32 {
        fade_alpha(t, self.start, self.end, self.fade)
    }
}

/// The GPU vignette for the first spotlight active at tour time `t`, anchored via `vp` (so it
/// tracks its point + stays a constant on-screen size). Off (`on == 0`) when none is active.
pub(crate) fn vignette_for(spots: &[Spotlight], vp: &fractadyne_core::Viewport, t: f64) -> fractadyne_gpu::Vignette {
    for sp in spots {
        let a = sp.alpha_at(t);
        if a <= 0.0 {
            continue;
        }
        let (vpx, vpy) = vp.complex_to_pixel(&sp.cx, &sp.cy);
        return fractadyne_gpu::Vignette {
            on: 1,
            dim: sp.dim * a, // fade the dimming in/out with the window
            soft: sp.soft,
            center: [(vpx / vp.width_px) as f32, (vpy / vp.height_px) as f32],
            radius: sp.radius,
        };
    }
    fractadyne_gpu::Vignette::default()
}

#[derive(Deserialize, Clone, Default)]
struct KeyframeFile {
    /// Stable identity — reorder, undo, selection and cross-references all need one that isn't
    /// positional. Also what error messages name.
    #[serde(default)]
    id: Option<String>,
    /// **Absolute** time the camera arrives here (seconds from the start). The first keyframe
    /// defaults to 0; later ones must not arrive before the previous one's hold ends.
    #[serde(default)]
    t: Option<f64>,
    /// Named coordinate to sit on (see `[[location]]`), instead of inline `re`/`im`.
    #[serde(default)]
    location: Option<String>,
    /// Center, full-precision decimal or an exact rational (`-3/4`). Omit to inherit.
    #[serde(default)]
    re: Option<String>,
    #[serde(default)]
    im: Option<String>,
    /// Magnification, e.g. `2667` or `"6.5e94"`. Omit to inherit the previous keyframe's.
    #[serde(default)]
    zoom: Option<NumOrStr>,
    /// Exact iteration budget for frames at this keyframe (interpolated geometrically along the
    /// glide from the previous one). One script-wide number cannot serve both a 1.33× home view
    /// and a 1e94× dive; this is how a deep chapter asks for what it needs without making the
    /// shallow frames cost minutes each. Inherited forward until changed.
    #[serde(default)]
    max_iter: Option<u32>,
    /// Palette id (see `[[palette]]`), a preset name, or a preset index. Inherited forward.
    #[serde(default)]
    palette: Option<String>,
    #[serde(default)]
    fractal: Option<String>,
    #[serde(default)]
    julia: Option<bool>,
    /// Easing for the glide arriving at this keyframe: `smooth` (default), `linear`, `smoother`,
    /// `in` (accelerate), or `out` (decelerate).
    #[serde(default)]
    ease: Option<String>,
    /// Seconds to hold (pause) at this keyframe before gliding to the next.
    #[serde(default)]
    hold: f64,
    // --- discrete state (inherited forward until changed, like the center) ---
    /// Show the linked dual view (Mandelbrot + its Julia set side by side).
    #[serde(default)]
    dual: Option<bool>,
    /// Pin the Julia set's parameter `c` (the Mandelbrot point whose Julia set to show). Both
    /// components must be given to take effect.
    #[serde(default)]
    julia_re: Option<f64>,
    #[serde(default)]
    julia_im: Option<f64>,
    /// Overlay the escape-time orbit (the path of z under iteration).
    #[serde(default)]
    orbits: Option<bool>,
    /// Show the minimap overview overlay (live playback; inherited forward until changed).
    /// Offline renders ignore it — the minimap is a navigation aid, not frame content. The
    /// viewer's own toggle is restored when the tour ends.
    #[serde(default)]
    minimap: Option<bool>,
    /// The point whose orbit to draw (when `orbits` is on). Both components required.
    #[serde(default)]
    orbit_re: Option<f64>,
    #[serde(default)]
    orbit_im: Option<f64>,
}

// ---------------------------------------------------------------------------------------------
// Tour-script schema reference (single source of truth for TOURS.md).
//
// This table sits next to the serde structs above and mirrors their fields. `--dump-tour-schema`
// renders it to Markdown (→ TOURS.md), and the `tour_schema_doc_current` test fails if the checked-
// in TOURS.md drifts from it — so the docs regenerate from here and can't silently rot.
// ---------------------------------------------------------------------------------------------

/// One documented field of a tour-script table.
struct SchemaField {
    name: &'static str,
    ty: &'static str,
    default: &'static str,
    doc: &'static str,
}

/// One documented TOML table in the tour schema (top level or a `[[repeatable]]`).
struct SchemaTable {
    toml: &'static str,
    repeatable: bool,
    summary: &'static str,
    fields: &'static [SchemaField],
}

const TOUR_SCHEMA: &[SchemaTable] = &[
    SchemaTable {
        toml: "(top level)",
        repeatable: false,
        summary: "Script-wide settings.",
        fields: &[
            SchemaField { name: "format_version", ty: "int", default: "(required)", doc: "Schema version the script targets. Must be 2 — v1 scripts (cumulative `secs`, `mag`, separate caption/callout/spotlight arrays) are rejected with a migration message rather than mis-played. A version newer than this build warns that some annotations may not render." },
            SchemaField { name: "name", ty: "string", default: "\"\"", doc: "Display name (shown in render progress and the end-of-script toast)." },
            SchemaField { name: "loop", ty: "bool", default: "false", doc: "Loop the tour during live playback (Tools -> Play script)." },
            SchemaField { name: "render", ty: "[render]", default: "{}", doc: "How the tour is meant to be rendered — see below." },
            SchemaField { name: "playback", ty: "[playback]", default: "{}", doc: "How the tour behaves in the LIVE view — see below." },
            SchemaField { name: "location", ty: "[[location]]", default: "[]", doc: "Named coordinates, referenced by `location = \"id\"`." },
            SchemaField { name: "palette", ty: "[[palette]]", default: "[]", doc: "Named palettes, referenced by a keyframe's `palette = \"id\"`." },
            SchemaField { name: "segment", ty: "[[segment]]", default: "[]", doc: "Chapters, so one can be rendered in isolation (`--segment`)." },
            SchemaField { name: "keyframe", ty: "[[keyframe]]", default: "[]", doc: "The camera path (at least one required) — see below." },
            SchemaField { name: "annotation", ty: "[[annotation]]", default: "[]", doc: "Timed overlays of every kind (caption / callout / spotlight) — see below." },
            SchemaField { name: "editor", ty: "table", default: "(unset)", doc: "Reserved for editor-only state (selection, timeline zoom). Parsed and ignored by the player." },
        ],
    },
    SchemaTable {
        toml: "[render]",
        repeatable: false,
        summary: "Output settings, so `--render-tour x.toml` with no flags reproduces the intended render. Every field is overridden by the matching CLI flag when one is given.",
        fields: &[
            SchemaField { name: "size", ty: "string", default: "(CLI, else 1280x720)", doc: "Frame size, \"WIDTHxHEIGHT\" or a bare width (16:9 height)." },
            SchemaField { name: "fps", ty: "float", default: "(CLI, else 30)", doc: "Frames per second of the rendered sequence." },
            SchemaField { name: "ss", ty: "int", default: "(CLI, else 1)", doc: "Supersampling factor (2 = 2x2 samples per pixel)." },
            SchemaField { name: "prefix", ty: "string", default: "(script file stem)", doc: "Frame-name prefix: frames are written <prefix>_00000.png." },
            SchemaField { name: "out", ty: "string", default: "(CLI, else \"frames\")", doc: "Output directory for the frame sequence, relative to the working directory." },
            SchemaField { name: "mp4", ty: "bool | string", default: "false", doc: "Assemble the frames into an H.264 mp4 with ffmpeg afterwards; a string names the file." },
            SchemaField { name: "max_iter", ty: "int", default: "(session, min 500000)", doc: "Iteration budget for frames whose keyframes don't state their own. Deep tours SHOULD set a per-keyframe budget instead: the depth formula under-budgets hard fields badly (a Misiurewicz spar gets ~46k at 1e61x where it needs 222k), and every frame there renders flat." },
            SchemaField { name: "auto_iter", ty: "bool", default: "true", doc: "Whether this `max_iter` is a base that still scales with depth (true) or an exact count used as-is (false). Per-keyframe budgets are always exact." },
            SchemaField { name: "show_location", ty: "bool", default: "false", doc: "Burn a zoom-level + coordinate HUD into every frame (same as the --show-location CLI flag)." },
            SchemaField { name: "normalize", ty: "bool", default: "false", doc: "Auto-normalize the palette cycle to each frame's escape-value range, temporally smoothed so a video doesn't shimmer. Deep tours need it — past ~1e50x a fixed cycle aliases the escape field into confetti. (Colors up to ~40 Mpx/frame; above that the frame falls back to un-normalized with a logged warning, pending a tiled normalized color pass.)" },
        ],
    },
    SchemaTable {
        toml: "[playback]",
        repeatable: false,
        summary: "How the tour behaves during LIVE playback (Tools -> Play script). None of this affects an offline render, which always computes every frame to completion.",
        fields: &[
            SchemaField { name: "pace", ty: "string", default: "adaptive", doc: "realtime = run on the wall clock and show whatever is ready (what a benchmark wants — the measurement IS what the machine got done in real time). adaptive = slow the tour while the reference pipeline lags, so a deep dive degrades in duration rather than into a stale blur. settled = adaptive, plus stop the clock at every keyframe HOLD until the view has actually resolved. Deep tours want settled: the live iteration budget only climbs on settled frames and needs more of them than a few seconds of hold provides, so without it the tour walks past its own destination while the screen is still starved." },
            SchemaField { name: "settle_timeout", ty: "float", default: "20", doc: "With pace = \"settled\": seconds a hold may wait for the view to resolve before giving up and moving on, so an unresolvable view cannot stall the tour forever." },
        ],
    },
    SchemaTable {
        toml: "[[location]]",
        repeatable: true,
        summary: "A named coordinate. Referenced by keyframes and annotations via `location = \"id\"`, so a 120-digit dive center is written once.",
        fields: &[
            SchemaField { name: "id", ty: "string", default: "(required)", doc: "Unique name used to reference this location." },
            SchemaField { name: "re", ty: "string", default: "(required)", doc: "Real part — full-precision decimal or an exact rational expression like (37+16i)/100." },
            SchemaField { name: "im", ty: "string", default: "(required)", doc: "Imaginary part." },
            SchemaField { name: "zoom", ty: "float | string", default: "(unset)", doc: "Default magnification for keyframes that name this location without their own `zoom`." },
            SchemaField { name: "thumb", ty: "string", default: "(unset)", doc: "Preview image path relative to the script (reserved for editors; not read by the player)." },
            SchemaField { name: "note", ty: "string", default: "(unset)", doc: "Free-text note for the author (ignored)." },
        ],
    },
    SchemaTable {
        toml: "[[palette]]",
        repeatable: true,
        summary: "A named palette. A keyframe's `palette = \"id\"` selects it, and the coloring interpolates between keyframes — one mechanism covering static palettes, morphs, and cycling.",
        fields: &[
            SchemaField { name: "id", ty: "string", default: "(required)", doc: "Unique name used to reference this palette." },
            SchemaField { name: "preset", ty: "string", default: "(unset)", doc: "Built-in preset name (e.g. \"Ember\") or index. Either this or `stops` is required." },
            SchemaField { name: "stops", ty: "[{at, color}]", default: "[]", doc: "Explicit gradient stops ascending by `at` (0..1), colors as #rrggbb; up to 8." },
        ],
    },
    SchemaTable {
        toml: "[[segment]]",
        repeatable: true,
        summary: "A chapter of the timeline. `--segment NAME` renders (or plays) only that range, keeping the global frame numbering — so ten seconds of narration can be re-rendered without redoing the whole tour.",
        fields: &[
            SchemaField { name: "id", ty: "string", default: "(title)", doc: "Short name matched by --segment (case-insensitive; the 1-based index also works)." },
            SchemaField { name: "title", ty: "string", default: "(id)", doc: "Human-readable chapter title." },
            SchemaField { name: "t", ty: "float", default: "0", doc: "Absolute start time (seconds). The chapter runs to the next segment's `t`, or the end of the tour." },
        ],
    },
    SchemaTable {
        toml: "[[keyframe]]",
        repeatable: true,
        summary: "A camera waypoint. The view eases from the previous keyframe to this one, ARRIVING at absolute time `t`, then holds for `hold`. Everything except `t`/`hold`/`ease` inherits forward until changed, so a pure zoom-in needs only `t` and `zoom`.",
        fields: &[
            SchemaField { name: "id", ty: "string", default: "(index)", doc: "Stable identity for reordering, cross-references, and error messages." },
            SchemaField { name: "t", ty: "float", default: "0 (first) / (required)", doc: "ABSOLUTE second the camera arrives here. Must be >= the previous keyframe's t + hold. Absolute times mean inserting a keyframe can't desync downstream narration." },
            SchemaField { name: "hold", ty: "float", default: "0", doc: "Seconds to pause here before the next glide begins." },
            SchemaField { name: "ease", ty: "string", default: "smooth", doc: "Easing for the glide arriving here: smooth, linear, smoother, in (accelerate), or out (decelerate)." },
            SchemaField { name: "location", ty: "string", default: "(inherit)", doc: "Named coordinate to sit on (see [[location]]), instead of inline re/im." },
            SchemaField { name: "re", ty: "string", default: "(inherit)", doc: "Center, real part: full-precision decimal or an exact rational. Omit to inherit (pure zoom)." },
            SchemaField { name: "im", ty: "string", default: "(inherit)", doc: "Center, imaginary part." },
            SchemaField { name: "zoom", ty: "float | string", default: "(inherit, else 1)", doc: "Magnification here, e.g. 2667 or \"6.5e94\". Strings carry depths past f64's ~1e308 ceiling." },
            SchemaField { name: "max_iter", ty: "int", default: "(inherit, else [render])", doc: "Exact iteration budget at this keyframe, interpolated geometrically along the glide. One script-wide number cannot serve both a 1.33x home view and a 1e94x dive." },
            SchemaField { name: "palette", ty: "string", default: "(inherit)", doc: "Palette id, preset name, or preset index; interpolated between keyframes." },
            SchemaField { name: "fractal", ty: "string", default: "(inherit)", doc: "Fractal family name (e.g. \"Mandelbrot\", \"Burning Ship\")." },
            SchemaField { name: "julia", ty: "bool", default: "(inherit)", doc: "Julia mode for the family." },
            SchemaField { name: "dual", ty: "bool", default: "(inherit)", doc: "Show the linked dual view (Mandelbrot + its Julia set side by side)." },
            SchemaField { name: "julia_re", ty: "float", default: "(inherit)", doc: "Pin the Julia parameter c (real part). Both julia_re and julia_im are required; interpolated between keyframes." },
            SchemaField { name: "julia_im", ty: "float", default: "(inherit)", doc: "Julia parameter c (imaginary part)." },
            SchemaField { name: "orbits", ty: "bool", default: "(inherit)", doc: "Overlay the escape-time orbit (the path of z under iteration)." },
            SchemaField { name: "minimap", ty: "bool", default: "(inherit)", doc: "Show the minimap overview overlay (live playback only; offline renders ignore it). The viewer's own setting is restored when the tour ends." },
            SchemaField { name: "orbit_re", ty: "float", default: "(inherit)", doc: "The point whose orbit to draw, real part (both components required; interpolated)." },
            SchemaField { name: "orbit_im", ty: "float", default: "(inherit)", doc: "Orbit point imaginary part." },
        ],
    },
    SchemaTable {
        toml: "[[annotation]]",
        repeatable: true,
        summary: "A timed overlay, independent of the camera path. One array for every kind: an editor sees a single track list, and new kinds are additive.",
        fields: &[
            SchemaField { name: "kind", ty: "string", default: "(required)", doc: "caption (narration text), callout (label anchored to a coordinate, tracking it as the view moves), or spotlight (dim everything outside a soft circle)." },
            SchemaField { name: "id", ty: "string", default: "(index)", doc: "Stable identity for editors and cross-references." },
            SchemaField { name: "t", ty: "float", default: "0", doc: "When it appears (absolute seconds)." },
            SchemaField { name: "secs", ty: "float", default: "0 = until end", doc: "How long it stays (seconds); 0/omitted keeps it until the tour ends." },
            SchemaField { name: "fade", ty: "float", default: "0.4", doc: "Fade in/out time (seconds) at each end." },
            SchemaField { name: "text", ty: "string", default: "(caption/callout)", doc: "The text to show (supports \\n for multiple lines). Required for caption and callout." },
            SchemaField { name: "pos", ty: "string", default: "bottom", doc: "caption: screen anchor — top, center, or bottom." },
            SchemaField { name: "size", ty: "float", default: "22 / 18", doc: "Font size in points, scaled to the frame height (caption 22, callout 18)." },
            SchemaField { name: "location", ty: "string", default: "(unset)", doc: "callout/spotlight: named coordinate to anchor to, instead of re/im." },
            SchemaField { name: "re", ty: "string", default: "(unset)", doc: "callout/spotlight: anchor coordinate, real part." },
            SchemaField { name: "im", ty: "string", default: "(unset)", doc: "callout/spotlight: anchor coordinate, imaginary part." },
            SchemaField { name: "radius", ty: "float", default: "0.25", doc: "spotlight: circle radius as a fraction of the frame height." },
            SchemaField { name: "softness", ty: "float", default: "0.08", doc: "spotlight: soft-edge width as a fraction of the frame height." },
            SchemaField { name: "dim", ty: "float", default: "0.7", doc: "spotlight: how dark outside the circle (0..1)." },
        ],
    },
];

/// Render [`TOUR_SCHEMA`] (plus stable prose) to the Markdown of `TOURS.md`. Emitted by
/// `fractadyne --dump-tour-schema`; kept byte-identical to the checked-in file by a test.
pub(crate) fn tour_schema_markdown() -> String {
    let mut s = String::new();
    s.push_str(
        "<!-- Generated by `fractadyne --dump-tour-schema` from TOUR_SCHEMA in\n     \
         crates/fractadyne-app/src/scripting.rs. Do not edit by hand — edit the schema table\n     \
         there and regenerate (a test enforces this file matches). -->\n\n",
    );
    s.push_str("# Fractadyne tour scripts\n\n");
    s.push_str(
        "A **tour** is a TOML script describing an eased camera path — plus optional captions, \
         callouts, and spotlights — through a fractal. Play one live via **Tools -> Play script...**, \
         or render it headless to a PNG frame sequence (and, with ffmpeg, straight to an mp4):\n\n",
    );
    s.push_str(
        "```sh\nfractadyne --render-tour my-tour.toml --size 1920x1080 --fps 30 --ss 2 --out frames --mp4\n```\n\n",
    );
    s.push_str(
        "Frames are written `<prefix>_00000.png` (prefix defaults to the script's file name; override \
         with `--prefix`). A script's `[render]` block supplies all of those settings, so a tour that \
         declares them renders correctly from `--render-tour x.toml` alone and CLI flags merely \
         override. `--segment NAME` renders just one `[[segment]]` chapter, keeping the global frame \
         numbering so the frames drop back into the full sequence. Ready-made examples live in \
         [`tours/`](tours/); run `fractadyne --help` for the full CLI.\n\n",
    );

    s.push_str("## Schema\n\n");
    for t in TOUR_SCHEMA {
        let heading = if t.repeatable {
            format!("### `{}` — repeatable\n\n", t.toml)
        } else {
            format!("### `{}`\n\n", t.toml)
        };
        s.push_str(&heading);
        s.push_str(t.summary);
        s.push_str("\n\n| Field | Type | Default | Description |\n|---|---|---|---|\n");
        for f in t.fields {
            s.push_str(&format!("| `{}` | {} | {} | {} |\n", f.name, f.ty, f.default, f.doc));
        }
        s.push('\n');
    }

    s.push_str(
        "## Timeline\n\n\
         **Every time in a script is absolute seconds from the start.** A `[[keyframe]]`'s `t` is \
         when the camera *arrives* there; it then sits still for `hold` before easing toward the \
         next one, which must arrive no earlier than `t + hold`. Annotation `t` is when the overlay \
         appears, and `secs = 0` means \"until the tour ends\". At `--fps N` the tour renders \
         `round(total * N) + 1` frames.\n\n\
         Absolute times are what make a script editable: inserting or lengthening a keyframe leaves \
         every other element exactly where it was, instead of silently sliding all downstream \
         narration out of sync.\n\n\
         For deep dives, give the descent generous time, use `ease = \"in\"` / `\"out\"` at the ends \
         with `linear` for the cruise, and write magnifications past f64 range as strings \
         (`zoom = \"6.5e94\"`). Pan at low zoom *before* diving so the camera doesn't zoom through \
         the set's black interior. Give the deep keyframes their own `max_iter`: one script-wide \
         budget cannot serve both a 1.33x home view and a 1e94x dive.\n\n\
         **Live playback is not the same as a render.** An offline render computes every frame to \
         completion, however long that takes. The live view has a wall clock to keep up with, so it \
         bounds per-frame cost and raises its iteration budget adaptively over successive *settled* \
         frames — which at depth needs more of them than a few seconds of hold provides. A deep tour \
         meant to be WATCHED should therefore set `[playback] pace = \"settled\"`, which stops the \
         clock at each hold until the picture has actually resolved. Validate either path with \
         `--livetest` (live) or by rendering frames (offline).\n\n",
    );

    s.push_str(
        "## Example\n\n\
         ```toml\n\
         format_version = 2\n\
         name = \"Mini tour\"\n\n\
         [render]\n\
         size = \"1920x1080\"\n\
         fps = 30\n\
         ss = 2\n\n\
         [[location]]\n\
         id = \"seahorse\"\n\
         re = \"-0.743643887037158704752191506114774\"\n\
         im = \"0.131825904205311970493132056385139\"\n\n\
         [[keyframe]]            # overview — arrives at t=0, sits for 2s\n\
         id = \"home\"\n\
         t = 0\n\
         re = \"-0.5\"\n\
         im = \"0.0\"\n\
         zoom = 1\n\
         palette = \"Ember\"\n\
         max_iter = 2000\n\
         hold = 2\n\n\
         [[keyframe]]            # dive into Seahorse Valley, arriving at t=8\n\
         id = \"dive\"\n\
         t = 8\n\
         location = \"seahorse\"\n\
         zoom = \"1e6\"\n\
         max_iter = 20000\n\
         ease = \"in\"\n\
         hold = 3\n\n\
         [[annotation]]\n\
         kind = \"caption\"\n\
         text = \"Seahorse Valley\"\n\
         t = 8\n\
         secs = 4\n\
         pos = \"bottom\"\n\n\
         [[annotation]]\n\
         kind = \"spotlight\"\n\
         location = \"seahorse\"\n\
         radius = 0.28\n\
         t = 8\n\
         secs = 4\n\
         ```\n",
    );
    s
}

/// Easing curve for a keyframe glide segment.
#[derive(Clone, Copy)]
enum EaseKind {
    Linear,
    Smooth,
    Smoother,
    In,
    Out,
}

impl EaseKind {
    fn parse(s: Option<&str>) -> EaseKind {
        match s.map(|s| s.to_ascii_lowercase()).as_deref() {
            Some("linear" | "line") => EaseKind::Linear,
            Some("smoother" | "smootherstep") => EaseKind::Smoother,
            Some("in" | "ease-in" | "accelerate") => EaseKind::In,
            Some("out" | "ease-out" | "decelerate") => EaseKind::Out,
            _ => EaseKind::Smooth, // smoothstep — the previous global default
        }
    }
    fn apply(self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        match self {
            EaseKind::Linear => u,
            EaseKind::Smooth => u * u * (3.0 - 2.0 * u),
            EaseKind::Smoother => u * u * u * (u * (u * 6.0 - 15.0) + 10.0),
            EaseKind::In => u * u,
            EaseKind::Out => 1.0 - (1.0 - u) * (1.0 - u),
        }
    }
}

/// Parse a magnification string into **log10 of the magnification**. Accepts a plain decimal
/// (`"1.33"`, `"2667"`) or scientific notation (`"6.5e94"`, `"1e1216"`), with an optional trailing
/// `x`/`×`. The exponent is handled symbolically rather than parsed into an f64, so depths past
/// f64's ~1e308 ceiling — the whole point of this app — survive.
fn parse_zoom_log10(s: &str) -> Result<f64, String> {
    let t = s.trim().trim_end_matches(['x', 'X', '\u{d7}']).trim();
    let (mant, exp) = match t.find(['e', 'E']) {
        Some(i) => (
            &t[..i],
            t[i + 1..]
                .trim()
                .parse::<f64>()
                .map_err(|_| format!("zoom \"{s}\": exponent is not a number"))?,
        ),
        None => (t, 0.0),
    };
    let m: f64 = mant
        .trim()
        .parse()
        .map_err(|_| format!("zoom \"{s}\": not a number"))?;
    if !(m.is_finite() && m > 0.0) {
        // A literal with 300+ integer digits overflows f64 — say so instead of rendering at 1x.
        return Err(format!(
            "zoom \"{s}\": magnification must be a positive finite number; write deep zooms in \
             scientific notation (e.g. \"6.5e94\")"
        ));
    }
    Ok(m.log10() + exp)
}

/// The resolved `[render]` block: what the script asks for. Every field is `None` when the script
/// is silent, so the CLI (or a built-in default) fills it in — see `TourRenderConfig::resolve`.
#[derive(Clone, Default)]
pub(crate) struct TourRender {
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
    pub(crate) fps: Option<f64>,
    pub(crate) ss: Option<u32>,
    pub(crate) prefix: Option<String>,
    pub(crate) out: Option<std::path::PathBuf>,
    /// `Some(None)` = encode an mp4 at the default path; `Some(Some(p))` = at `p`.
    pub(crate) mp4: Option<Option<std::path::PathBuf>>,
    pub(crate) max_iter: Option<u32>,
    pub(crate) auto_iter: Option<bool>,
    pub(crate) show_location: bool,
    pub(crate) normalize: bool,
}

/// The tour-render flags exactly as the CLI gave them. `None` means "not specified": the script's
/// `[render]` block fills it in, and a built-in default fills what neither states — so
/// `--render-tour x.toml` alone reproduces the render the script intends, while any flag wins.
#[derive(Clone, Default)]
pub(crate) struct TourRenderConfig {
    pub(crate) fps: Option<f64>,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
    pub(crate) ss: Option<u32>,
    pub(crate) out: Option<std::path::PathBuf>,
    pub(crate) prefix: Option<String>,
    /// `--mp4 [PATH]`: `Some(None)` = default path, `Some(Some(p))` = at `p`.
    pub(crate) mp4: Option<Option<std::path::PathBuf>>,
    /// `--segment NAME`: render only that `[[segment]]` chapter (global frame numbering kept).
    pub(crate) segment: Option<String>,
    /// `--segments N --segment-index K`: shard the WHOLE timeline into `N` contiguous, gap-free
    /// frame ranges and render only range `K` (0-based) — so `N` machines each take one range and
    /// the frames union to exactly the full video. Distinct from `segment` (a named chapter);
    /// both may combine (the shard intersects the chapter). Global frame numbering kept.
    pub(crate) segments: Option<u32>,
    pub(crate) segment_index: Option<u32>,
    /// `--dry-run`: print the resolved plan — total frames and THIS invocation's `[start, end)`
    /// range — and exit without rendering, so a farm script can verify shards tile before
    /// committing hours of GPU.
    pub(crate) dry_run: bool,
    pub(crate) overwrite: bool,
    pub(crate) resume: bool,
}

/// The half-open frame range `[start, end)` of shard `k` of `n` over `frames` total frames:
/// `[⌊k·F/N⌋, ⌊(k+1)·F/N⌋)`. This exact formula is what guarantees the shards TILE: consecutive
/// shards share a boundary (this shard's `end` is the next one's `start`), their union is
/// `[0, F)` with no overlap, and the `F mod N` remainder frames spread one-each across the low
/// shards — no off-by-one, no double-render, no gap (the multi-machine correctness the feature
/// exists for). Pure, so the unit test below can pin the tiling property outright.
pub(crate) fn segment_range(frames: u64, n: u64, k: u64) -> (u64, u64) {
    let n = n.max(1);
    let k = k.min(n - 1);
    (k * frames / n, (k + 1) * frames / n)
}

/// Everything the tour renderer needs, after merging the CLI over the script's `[render]` block.
struct ResolvedTourRender {
    fps: f64,
    width: u32,
    height: u32,
    ss: u32,
    out: std::path::PathBuf,
    prefix: String,
    mp4: Option<std::path::PathBuf>,
}

impl TourRenderConfig {
    /// Merge: CLI flag > script `[render]` > built-in default.
    fn resolve(&self, script: &TourRender, script_path: &std::path::Path) -> ResolvedTourRender {
        let width = self.width.or(script.width).unwrap_or(1280).clamp(16, 16384);
        let height = self
            .height
            .or(script.height)
            .unwrap_or((width * 9 / 16).max(16))
            .clamp(16, 16384);
        let out = self
            .out
            .clone()
            .or_else(|| script.out.clone())
            .unwrap_or_else(|| std::path::PathBuf::from("frames"));
        let prefix = self
            .prefix
            .clone()
            .or_else(|| script.prefix.clone())
            .unwrap_or_else(|| {
                script_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "frame".to_string())
            });
        ResolvedTourRender {
            // Sub-1 fps is legitimate: sampling a long dive at --fps 0.25 is how the deep
            // regression holds get inspected without rendering thousands of frames.
            fps: self.fps.or(script.fps).unwrap_or(30.0).max(1.0e-3),
            width,
            height,
            ss: self.ss.or(script.ss).unwrap_or(1).clamp(1, 8),
            mp4: self
                .mp4
                .clone()
                .or_else(|| script.mp4.clone())
                .map(|p| p.unwrap_or_else(|| out.join(format!("{prefix}.mp4")))),
            out,
            prefix,
        }
    }
}

/// The viewer's own settings, saved when a tour starts so the script's iteration budget and
/// coloring don't outlive it.
pub(crate) struct PlaybackRestore {
    pub(crate) max_iter: u32,
    pub(crate) auto_iter: bool,
    pub(crate) palette_idx: usize,
    pub(crate) use_custom_palette: bool,
    pub(crate) use_binary: bool,
    pub(crate) use_duotone: bool,
    /// Overlay toggles a script may drive (`minimap`, `orbits` keyframe fields) — the viewer's
    /// own settings, handed back when the tour ends like the budget and palette above.
    pub(crate) minimap: bool,
    pub(crate) show_orbits: bool,
}

/// A resolved chapter: `[start, end)` in tour seconds.
pub(crate) struct Segment {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) start: f64,
    pub(crate) end: f64,
}

/// A resolved keyframe: parsed center, the time the glide *reaches* it, how long it holds there,
/// and the easing of the glide arriving at it.
struct Kf {
    /// The script's `id` (or `#index`) — what harnesses and error messages call this keyframe.
    id: String,
    at: f64,
    hold: f64,
    ease: EaseKind,
    cx: fractadyne_core::BigFloat,
    cy: fractadyne_core::BigFloat,
    logmag: f64,
    fractal: FractalKind,
    julia: bool,
    // Discrete (non-interpolated) state, inherited forward.
    dual: bool,
    julia_c: Option<(f64, f64)>,
    orbits: bool,
    minimap: bool,
    orbit: Option<(f64, f64)>,
    /// Exact iteration budget at this keyframe (interpolated geometrically along a glide), or
    /// `None` to leave the budget to `[render]` / the session.
    max_iter: Option<u32>,
    /// Index into `Playback::palettes` for this keyframe's coloring (interpolated along a glide).
    palette: Option<usize>,
}

/// The fully-resolved tour state at a moment in time: interpolated camera + the active keyframe's
/// discrete overlays (dual view, Julia pin, orbits).
/// One tick of the playback clock (see [`FractadyneApp::advance_playback_core`]).
pub(crate) enum PlaybackTick {
    /// No playback active.
    Idle,
    /// Still playing (the viewport was advanced).
    Playing,
    /// The tour just ended: `Some(name)` = surface a "finished" toast; `None` = a benchmark run
    /// whose report dialog was already queued.
    Finished(Option<String>),
    /// A finished tour, still loaded so its player stays on screen. The camera was re-sampled (a
    /// scrub while stopped must move the view) but the clock did not advance, so this must NOT
    /// drive a repaint — otherwise a finished tour spins the render loop forever.
    Ended,
}

/// Read + parse a tour script, rejecting a retired v1 file with a migration message and warning
/// (on stderr) about a version from a newer build. Shared by every entry point so the diagnosis
/// is identical whether the script came from the CLI, the file dialog, or a test harness.
fn read_script(path: &std::path::Path) -> Result<ScriptFile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    parse_script_text(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parse script TOML from memory — the path-free half of [`read_script`], so the selftest can
/// resolve the shipped tours (compiled in with `include_str!`) without touching the filesystem.
fn parse_script_text(text: &str) -> Result<ScriptFile, String> {
    check_format_version(text)?;
    toml::from_str(text).map_err(|e| format!("parse: {e}"))
}

/// Resolve script TOML from memory into a playable tour. Used by the selftest suite.
pub(crate) fn parse_tour_text(text: &str) -> Result<Playback, String> {
    resolve_script(parse_script_text(text)?, None)
}

/// Gate a script on its `format_version` **before** deserializing, so a v1 file gets a migration
/// message instead of playing as garbage. v1 and v2 share no timing keys: v1's `secs`/`mag` simply
/// don't exist in v2, so serde would default every keyframe to t=0 at 1× and the tour would render
/// as one still frame of the whole set. There is deliberately no v1 reader — the format changed
/// while the app had no users, and carrying a dead branch costs more than the migration did.
fn check_format_version(text: &str) -> Result<(), String> {
    let doc: toml::Value = toml::from_str(text).map_err(|e| format!("parse script: {e}"))?;
    let ver = doc.get("format_version").and_then(|v| v.as_integer()).unwrap_or(0);
    if ver < SCRIPT_FORMAT_VERSION as i64 {
        return Err(format!(
            "this script is format v{ver}, which this build no longer reads (current: v{}).\n\
             v2 changed the timeline to ABSOLUTE keyframe times and reorganized the file:\n  \
             [[keyframe]] secs = N        -> t = <absolute arrival second>\n  \
             mag = N / mag_log10 = N      -> zoom = N  or  zoom = \"6.5e94\"\n  \
             center_re / center_im        -> re / im   (or location = \"id\")\n  \
             [[caption]]/[[callout]]/[[spotlight]] -> [[annotation]] with kind = \"...\", at -> t\n  \
             top-level render settings    -> a [render] block\n\
             See TOURS.md for the full schema and tours/ for migrated examples.",
            SCRIPT_FORMAT_VERSION
        ));
    }
    if ver > SCRIPT_FORMAT_VERSION as i64 {
        eprintln!(
            "Warning: script format v{ver} is newer than this build (v{}); newer features may \
             not apply.",
            SCRIPT_FORMAT_VERSION
        );
    }
    Ok(())
}

/// Load + resolve a tour script file into a ready [`Playback`] (no palette side effects, no
/// dialogs) — shared by the `--divetest` harness; `load_script` remains the interactive path.
pub(crate) fn parse_tour_file(path: &std::path::Path) -> Result<Playback, String> {
    let mut pb = resolve_script(read_script(path)?, None)?;
    pb.source = Some(path.to_path_buf());
    Ok(pb)
}

/// A tour palette: either a built-in preset (applied verbatim, so a static tour colors exactly as
/// selecting that preset would) or explicit gradient stops.
#[derive(Clone)]
pub(crate) enum TourPalette {
    Preset(usize),
    Stops(Vec<[f32; 4]>),
}

/// What the app should do with the coloring for the current frame (see `Sampled::palette`).
pub(crate) enum PaletteApply {
    /// Select a built-in preset.
    Preset(usize),
    /// Install these gradient stops (`[pos, r, g, b]`, linear RGB) as the custom palette —
    /// a keyframe-to-keyframe palette morph resolves to this.
    Stops(Vec<[f32; 4]>),
}

impl TourPalette {
    /// Gradient stops as `[pos, r, g, b]`, whatever the source.
    fn stops(&self) -> Vec<[f32; 4]> {
        match self {
            TourPalette::Stops(s) => s.clone(),
            TourPalette::Preset(i) => fractadyne_color::PRESETS[*i]
                .stops
                .iter()
                .map(|(p, c)| [*p, c[0], c[1], c[2]])
                .collect(),
        }
    }

    /// Linear-RGB color at gradient position `u` (0..1), interpolating between stops.
    fn color_at(stops: &[[f32; 4]], u: f32) -> [f32; 3] {
        if stops.is_empty() {
            return [0.0; 3];
        }
        let first = stops[0];
        let last = stops[stops.len() - 1];
        if u <= first[0] {
            return [first[1], first[2], first[3]];
        }
        for w in stops.windows(2) {
            let (a, b) = (w[0], w[1]);
            if u <= b[0] {
                let span = (b[0] - a[0]).max(1.0e-6);
                let f = ((u - a[0]) / span).clamp(0.0, 1.0);
                return [
                    a[1] + (b[1] - a[1]) * f,
                    a[2] + (b[2] - a[2]) * f,
                    a[3] + (b[3] - a[3]) * f,
                ];
            }
        }
        [last[1], last[2], last[3]]
    }

    /// Cross-fade two palettes at `t` (0..1). Their stop *positions* generally differ, so both are
    /// resampled onto the same evenly spaced grid and the colors blended there — the only way to
    /// morph gradients whose shapes don't line up.
    fn blend(a: &TourPalette, b: &TourPalette, t: f32) -> Vec<[f32; 4]> {
        let (sa, sb) = (a.stops(), b.stops());
        let n = fractadyne_color::MAX_STOPS;
        (0..n)
            .map(|i| {
                let pos = i as f32 / (n - 1) as f32;
                let ca = Self::color_at(&sa, pos);
                let cb = Self::color_at(&sb, pos);
                [
                    pos,
                    ca[0] + (cb[0] - ca[0]) * t,
                    ca[1] + (cb[1] - ca[1]) * t,
                    ca[2] + (cb[2] - ca[2]) * t,
                ]
            })
            .collect()
    }
}

pub(crate) struct Sampled {
    pub(crate) cx: fractadyne_core::BigFloat,
    pub(crate) cy: fractadyne_core::BigFloat,
    pub(crate) logmag: f64,
    pub(crate) fractal: FractalKind,
    pub(crate) julia: bool,
    pub(crate) dual: bool,
    pub(crate) julia_c: Option<(f64, f64)>,
    pub(crate) orbits: bool,
    pub(crate) minimap: bool,
    pub(crate) orbit: Option<(f64, f64)>,
    /// Exact iteration budget this frame wants, when the script states one.
    pub(crate) max_iter: Option<u32>,
    /// Coloring for this frame, when the script states one.
    pub(crate) palette: Option<PaletteApply>,
}

/// Aggregates sampled while a benchmark tour plays.
pub(crate) struct Bench {
    frames: u64,
    sum_frame_ms: f64,
    sum_cpu_ms: f64,
    min_fps: f64,
    max_fps: f64,
    peak_ram: u64,
    sum_ram: u64,
    warmup_left: u32,
}

impl Bench {
    fn new() -> Self {
        Bench {
            frames: 0,
            sum_frame_ms: 0.0,
            sum_cpu_ms: 0.0,
            min_fps: f64::INFINITY,
            max_fps: 0.0,
            peak_ram: 0,
            sum_ram: 0,
            warmup_left: 12,
        }
    }
}

/// An active camera tour (and optional benchmark sampling).
pub(crate) struct Playback {
    pub(crate) name: String,
    /// The file this tour was loaded from, when there was one. The render dialog needs it (it
    /// renders the SCRIPT, not the live state), and the built-in benchmark has none — which is
    /// exactly why it is an Option rather than a path everyone assumes exists.
    pub(crate) source: Option<std::path::PathBuf>,
    kfs: Vec<Kf>,
    pub(crate) total: f64,
    /// Repeat when the end is reached (`loop` in the script; toggled by the status-bar transport).
    pub(crate) loop_: bool,
    /// Has the tour been ticked yet? (Was `t0`, the wall-clock origin. The clock is now an
    /// ACCUMULATOR — `cur_t += dt * speed` — because a derived `now - t0` cannot express pause,
    /// seek or speed: every transport control would have to fake them by moving the origin, and
    /// the pacer already moves the origin for its own reasons.)
    pub(crate) started: bool,
    pub(crate) bench: Option<Bench>,
    /// Timed narration overlays (drawn by the app over the fractal + into exported frames).
    pub(crate) captions: Vec<Caption>,
    /// Coordinate-anchored labeled markers (drawn over the fractal + into exported frames).
    pub(crate) callouts: Vec<Callout>,
    /// Spotlight vignettes (dim outside a soft circle; applied in the color shader).
    pub(crate) spotlights: Vec<Spotlight>,
    /// Palettes the keyframes reference (by index).
    palettes: Vec<TourPalette>,
    /// Chapters, in time order (may be empty).
    pub(crate) segments: Vec<Segment>,
    /// What the script's `[render]` block asked for (CLI flags override — see `TourRenderConfig`).
    pub(crate) render: TourRender,
    /// How the live clock treats a renderer that can't keep up (`[playback] pace`).
    pub(crate) pace: Pace,
    /// Seconds a `Settled` hold may wait for the view to resolve before moving on.
    settle_timeout: f64,
    /// Wall-clock seconds the current hold has already spent waiting, and which keyframe it is
    /// waiting at (so the budget resets at the next hold rather than accumulating across the tour).
    settle_waited: f64,
    settle_kf: usize,
    /// Wall-clock seconds the PACER has held the clock effectively stopped, for any reason. The
    /// settled-hold branch below has always been bounded by `settle_timeout`; the lag-based
    /// dilation was not bounded by anything, and a tour that reaches a view whose reference can
    /// never install (the freeze guard refuses it) sits at `hold = 1.0` forever — reported from
    /// the field as a hang at 3:35 of the grand tour, with the renderer still working and the
    /// clock simply never advancing again. A tour must always finish.
    pace_waited: f64,
    /// Latched once `pace_waited` exceeds the timeout: the pacer has given up waiting and lets the
    /// clock run until the pipeline actually catches up (`hold` falls on its own).
    pace_released: bool,
    /// Current tour time (seconds) — the authoritative clock, advanced by `advance_playback_core`.
    pub(crate) cur_t: f64,
    /// Transport: playback rate multiplier, and whether the clock is user-paused. Distinct from
    /// `paced_hold`, which is the RENDERER asking for time; these two are the USER asking.
    pub(crate) speed: f64,
    pub(crate) paused: bool,
    /// The tour has reached its end and stopped there. The player STAYS on screen in this state —
    /// a finished tour is the one moment you most want to scrub back into it, and the old
    /// behaviour (tear the transport down the instant the clock ran out) took that away. The
    /// script's own iteration budget and coloring are still applied; the viewer's are handed back
    /// only when the player is closed (`stop_playback`). Scrubbing or pressing play clears it.
    pub(crate) finished: bool,
    /// How much the pacer is holding the tour clock back this frame: 0 = playing in real time,
    /// 1 = fully stopped (waiting for the renderer). The status bar reads this so a stalled
    /// progress percentage says WHY instead of looking like a hang — which is exactly how it was
    /// first reported ("the script stopped").
    pub(crate) paced_hold: f64,
    /// Wall-clock of the previous `advance_playback` frame — gives the per-frame `dt` the
    /// pipeline pacer needs to dilate the tour clock (see `advance_playback`). `None` = first frame.
    pub(crate) last_now: Option<f64>,
    /// Where a WALL-CLOCK-locked playback would be — advanced by `dt * speed` with NO pacer
    /// dilation, so `wall_t - cur_t` is exactly the time the pacer/settle-holds have stolen. Drives
    /// the transport's "ghost" scrub tick (the tour clock has drifted behind real time). Reset to
    /// `cur_t` on any seek/restart, so drift is always measured from the last thing the USER did.
    pub(crate) wall_t: f64,
}

impl Playback {
    /// Sample the eased camera state at time `e` (seconds, expected in `[0, total]`):
    /// the segment-interpolated center, `ln(magnification)`, fractal, and Julia flag.
    /// Shared by live playback and the headless tour renderer.
    pub(crate) fn sample(&self, e: f64) -> Sampled {
        let n = self.kfs.len();
        // Current segment start = last keyframe whose reach-time is ≤ e.
        let mut i = 0;
        for j in 0..n {
            if self.kfs[j].at <= e {
                i = j;
            } else {
                break;
            }
        }
        let a = &self.kfs[i];
        let mk = |cx, cy, lm, julia_c, orbit, max_iter, palette| Sampled {
            cx,
            cy,
            logmag: lm,
            fractal: a.fractal,
            julia: a.julia,
            dual: a.dual,
            julia_c,
            orbits: a.orbits,
            minimap: a.minimap,
            orbit,
            max_iter,
            palette,
        };
        // Holding at `a` (or past the final keyframe): return its state unchanged.
        if e <= a.at + a.hold || i + 1 >= n {
            let pal = a.palette.map(|p| match &self.palettes[p] {
                TourPalette::Preset(i) => PaletteApply::Preset(*i),
                TourPalette::Stops(s) => PaletteApply::Stops(s.clone()),
            });
            return mk(a.cx.clone(), a.cy.clone(), a.logmag, a.julia_c, a.orbit, a.max_iter, pal);
        }
        // Gliding a → b over its move window, with b's easing.
        let b = &self.kfs[i + 1];
        let move_start = a.at + a.hold;
        let seg = (b.at - move_start).max(1.0e-9);
        let ease = b.ease.apply((e - move_start) / seg);
        let lm = a.logmag + (b.logmag - a.logmag) * ease;
        // Precision from octaves (log2 mag) so it stays valid past f64's 1e308× ceiling.
        let octaves = (lm / std::f64::consts::LN_2).max(0.0).ceil() as u64;
        let p = fractadyne_core::precision_for_octaves(octaves);
        // Interpolate the Julia parameter c and the orbit point too, so the Julia set morphs (and
        // the orbit glides) smoothly between keyframes rather than jumping.
        let lerp2 = |x: (f64, f64), y: (f64, f64)| (x.0 + (y.0 - x.0) * ease, x.1 + (y.1 - x.1) * ease);
        let julia_c = match (a.julia_c, b.julia_c) {
            (Some(ja), Some(jb)) => Some(lerp2(ja, jb)),
            (x, _) => x,
        };
        let orbit = match (a.orbit, b.orbit) {
            (Some(oa), Some(ob)) => Some(lerp2(oa, ob)),
            (x, _) => x,
        };
        // Iteration budget: interpolate GEOMETRICALLY (in log space) — iteration cost grows with
        // depth like the zoom does, so a linear ramp would over-budget the whole first half of a
        // dive, which is exactly the "shallow frames cost minutes" failure per-keyframe budgets
        // exist to fix.
        let max_iter = match (a.max_iter, b.max_iter) {
            (Some(ia), Some(ib)) if ia != ib => {
                let l = (ia as f64).ln() + ((ib as f64).ln() - (ia as f64).ln()) * ease;
                Some(l.exp().round().clamp(1.0, u32::MAX as f64) as u32)
            }
            (Some(ia), _) => Some(ia),
            (None, x) => x,
        };
        // Palette: same index at both ends ⇒ pass the palette through untouched (a preset stays a
        // preset). Different ⇒ cross-fade the two gradients.
        let palette = match (a.palette, b.palette) {
            (Some(pa), Some(pb)) if pa != pb => Some(PaletteApply::Stops(TourPalette::blend(
                &self.palettes[pa],
                &self.palettes[pb],
                ease as f32,
            ))),
            (Some(pa), _) => Some(match &self.palettes[pa] {
                TourPalette::Preset(i) => PaletteApply::Preset(*i),
                TourPalette::Stops(s) => PaletteApply::Stops(s.clone()),
            }),
            (None, _) => None,
        };
        // ⭐A PINNED-CENTRE GLIDE MUST RETURN THE CENTRE EXACTLY. `lerp_bf(x, x, ease, p)` is not
        // the identity — it ROUNDS x to `p` bits, and `p` here is the precision of the CURRENT
        // interpolated depth, so a zoom dive's shallow entry rounds the destination centre it will
        // spend the whole descent converging on. A rounded centre is a genuinely DIFFERENT point,
        // and at a Misiurewicz centre its true orbit escapes at a length set by the rounding
        // (measured, core test `escape_length_of_rounded_centre`: 78 bits → escape 625, 157 →
        // 94,126, 206 → 602,515, 300+ → survives). The grand tour's 626-sample reference — pinned
        // by reuse through the df32→floatexp crossover into `bla_skip=0`, ~90× frame cost and a
        // GPU device loss at 2:58 — was exactly `escape(round(centre, 78)) + 1`, and hold-e72's
        // mysterious 602,516-sample orbit was `escape(round(centre, 206)) + 1`. However good the
        // reference picker, it cannot beat a wrong input point.
        //
        // A genuine pan (a.c ≠ b.c) still interpolates at the depth precision: mid-pan centres are
        // transient (references re-anchor continuously) and every arrival point is a keyframe,
        // which the hold branch above returns EXACTLY.
        let (cx, cy) = if a.cx == b.cx && a.cy == b.cy {
            (a.cx.clone(), a.cy.clone())
        } else {
            (
                fractadyne_core::lerp_bf(&a.cx, &b.cx, ease, p),
                fractadyne_core::lerp_bf(&a.cy, &b.cy, ease, p),
            )
        };
        mk(cx, cy, lm, julia_c, orbit, max_iter, palette)
    }

    /// Is the camera stationary at time `e` — inside a keyframe's hold, or past the final one?
    /// A hold is the only part of a tour where the view is meant to be LOOKED at rather than
    /// travelled through, which is why it gets settled rendering and (under `Settled` pacing) as
    /// much time as it needs.
    pub(crate) fn holding_at(&self, e: f64) -> (bool, usize) {
        let n = self.kfs.len();
        let mut i = 0;
        for j in 0..n {
            if self.kfs[j].at <= e {
                i = j;
            } else {
                break;
            }
        }
        match self.kfs.get(i) {
            Some(a) => (e <= a.at + a.hold || i + 1 >= n, i),
            None => (true, 0),
        }
    }

    /// Tour times worth validating: every keyframe that HOLDS, sampled near the end of its hold.
    /// A hold is where the camera stops for the viewer to look, so it is both the most demanding
    /// moment for the renderer (the view has had the whole hold to resolve) and the only one where
    /// a stale or starved frame is unambiguously a defect rather than motion.
    pub(crate) fn hold_checkpoints(&self) -> Vec<(f64, String)> {
        self.kfs
            .iter()
            .filter(|k| k.hold > 0.0)
            .map(|k| (k.at + k.hold * 0.9, k.id.clone()))
            .collect()
    }

    /// Find a chapter by `--segment` token: id or title (case-insensitive, also a unique prefix),
    /// or a 1-based index. Returns the matching segment, or a message listing what's available.
    pub(crate) fn find_segment(&self, token: &str) -> Result<&Segment, String> {
        let t = token.trim();
        let avail = || {
            self.segments
                .iter()
                .enumerate()
                .map(|(i, s)| format!("  {}. {} ({:.1}–{:.1}s)", i + 1, s.id, s.start, s.end))
                .collect::<Vec<_>>()
                .join("\n")
        };
        if self.segments.is_empty() {
            return Err(format!("--segment {t}: this script defines no [[segment]] chapters"));
        }
        if let Ok(n) = t.parse::<usize>() {
            return self
                .segments
                .get(n.wrapping_sub(1))
                .ok_or_else(|| format!("--segment {n}: out of range. Chapters:\n{}", avail()));
        }
        let eq = |a: &str| a.eq_ignore_ascii_case(t);
        self.segments
            .iter()
            .find(|s| eq(&s.id) || eq(&s.title))
            .or_else(|| {
                let lower = t.to_ascii_lowercase();
                let mut it = self.segments.iter().filter(|s| {
                    s.id.to_ascii_lowercase().starts_with(&lower)
                        || s.title.to_ascii_lowercase().starts_with(&lower)
                });
                match (it.next(), it.next()) {
                    (Some(s), None) => Some(s), // unique prefix
                    _ => None,
                }
            })
            .ok_or_else(|| format!("--segment \"{t}\": no such chapter. Chapters:\n{}", avail()))
    }
}

/// Blit a laid-out galley's glyph coverage as `color` (× `alpha`, straight over) onto a
/// **linear-RGBA** buffer, top-left at `(bx, by)` in frame pixels. `ppp` maps galley points to
/// atlas texels. Caller passes the atlas (one clone) so repeated calls don't re-clone it.
#[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2c: moves into the overlay module
fn blit_galley(
    atlas: &egui::epaint::FontImage, px: &mut [f32], w: u32, h: u32, galley: &egui::Galley,
    bx: f32, by: f32, ppp: f32, color: [f32; 3], alpha: f32,
) {
    let aw = atlas.size[0];
    for row in &galley.rows {
        for g in &row.glyphs {
            let uv = g.uv_rect;
            if uv.max[0] <= uv.min[0] || uv.max[1] <= uv.min[1] {
                continue;
            }
            let ox = (bx + (g.pos.x + uv.offset.x) * ppp).round() as i32;
            let oy = (by + (g.pos.y + uv.offset.y) * ppp).round() as i32;
            for ty in uv.min[1]..uv.max[1] {
                for tx in uv.min[0]..uv.max[0] {
                    let cov = atlas.pixels[ty as usize * aw + tx as usize] * alpha;
                    if cov <= 0.0 {
                        continue;
                    }
                    let dx = ox + (tx - uv.min[0]) as i32;
                    let dy = oy + (ty - uv.min[1]) as i32;
                    if dx < 0 || dy < 0 || dx >= w as i32 || dy >= h as i32 {
                        continue;
                    }
                    let i = (dy as usize * w as usize + dx as usize) * 4;
                    px[i] = color[0] * cov + px[i] * (1.0 - cov);
                    px[i + 1] = color[1] * cov + px[i + 1] * (1.0 - cov);
                    px[i + 2] = color[2] * cov + px[i + 2] * (1.0 - cov);
                }
            }
        }
    }
}

/// Multiply a rectangular region toward black (the soft backing behind annotation text).
#[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2c: moves into the overlay module
fn fill_dark(px: &mut [f32], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, amount: f32) {
    let (rx0, ry0) = (x0.max(0.0) as u32, y0.max(0.0) as u32);
    let (rx1, ry1) = ((x1.min(w as f32)) as u32, (y1.min(h as f32)) as u32);
    for y in ry0..ry1 {
        for x in rx0..rx1 {
            let i = (y as usize * w as usize + x as usize) * 4;
            px[i] *= 1.0 - amount;
            px[i + 1] *= 1.0 - amount;
            px[i + 2] *= 1.0 - amount;
        }
    }
}

fn blend_px(px: &mut [f32], w: u32, h: u32, x: i32, y: i32, color: [f32; 3], a: f32) {
    if a <= 0.0 || x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return;
    }
    let i = (y as usize * w as usize + x as usize) * 4;
    px[i] = color[0] * a + px[i] * (1.0 - a);
    px[i + 1] = color[1] * a + px[i + 1] * (1.0 - a);
    px[i + 2] = color[2] * a + px[i + 2] * (1.0 - a);
}

/// Anti-aliased ring outline (marker) of radius `r`, line width `thick`.
#[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2c: moves into the overlay module
fn draw_ring(px: &mut [f32], w: u32, h: u32, cx: f32, cy: f32, r: f32, thick: f32, color: [f32; 3], alpha: f32) {
    let lo = (cx - r - thick).floor() as i32;
    let hi = (cx + r + thick).ceil() as i32;
    let lo_y = (cy - r - thick).floor() as i32;
    let hi_y = (cy + r + thick).ceil() as i32;
    for y in lo_y..=hi_y {
        for x in lo..=hi {
            let d = (((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() - r).abs();
            let a = (1.0 - (d - thick * 0.5).max(0.0)).clamp(0.0, 1.0);
            blend_px(px, w, h, x, y, color, a * alpha);
        }
    }
}

/// A short 2-px-ish leader line from `(x0,y0)` to `(x1,y1)`.
#[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2c: moves into the overlay module
fn draw_line(px: &mut [f32], w: u32, h: u32, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 3], alpha: f32) {
    let steps = ((x1 - x0).abs().max((y1 - y0).abs())).ceil().max(1.0) as i32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = (x0 + (x1 - x0) * t).round() as i32;
        let y = (y0 + (y1 - y0) * t).round() as i32;
        for ( dx, dy) in [(0, 0), (1, 0), (0, 1)] {
            blend_px(px, w, h, x + dx, y + dy, color, alpha);
        }
    }
}

/// Burn a caption into a **linear-RGBA** export frame — the offscreen equivalent of
/// `draw_captions`: a soft dark backing rect + white text (× `alpha`), wrapped and centred on the
/// caption's screen anchor. `atlas` is the (pre-cloned) egui font atlas.
/// Returns the caption's backing rect (device pixels) so callouts can avoid it, or `None` if not drawn.
fn stamp_caption(ctx: &egui::Context, px: &mut [f32], w: u32, h: u32, cap: &Caption, alpha: f32) -> Option<egui::Rect> {
    if alpha <= 0.0 || w == 0 || h == 0 {
        return None;
    }
    let ppp = ctx.pixels_per_point();
    let pts = (cap.size * (h as f32 / 1080.0) / ppp).max(1.0);
    let galley = ctx.fonts(|f| {
        f.layout(cap.text.clone(), egui::FontId::proportional(pts), egui::Color32::WHITE, w as f32 * 0.8 / ppp)
    });
    let (gw, gh) = (galley.size().x * ppp, galley.size().y * ppp);
    let bx = w as f32 * 0.5 - gw * 0.5;
    let by = match cap.pos {
        CaptionPos::Top => h as f32 * 0.07,
        CaptionPos::Center => h as f32 * 0.5 - gh * 0.5,
        CaptionPos::Bottom => h as f32 * 0.91 - gh,
    };
    let pad = 12.0 * (h as f32 / 1080.0);
    fill_dark(px, w, h, bx - pad, by - pad, bx + gw + pad, by + gh + pad, (alpha * 0.5).min(1.0));
    // Clone the atlas AFTER layout so it contains this text's glyphs (egui fills it lazily).
    ctx.fonts(|f| blit_galley(&f.image(), px, w, h, &galley, bx, by, ppp, [1.0, 1.0, 1.0], alpha));
    Some(egui::Rect::from_min_max(egui::pos2(bx - pad, by - pad), egui::pos2(bx + gw + pad, by + gh + pad)))
}

/// Burn a small location HUD — zoom level + center coordinates — into the top-left of an export
/// frame, over a soft dark backing. The zoom line is amber (brand accent); the coordinate lines are
/// light grey monospace. `vp` supplies the arbitrary-precision center + zoom for the current frame.
/// Pre-rasterized location HUD (dark panel + zoom/coordinate text): premultiplied **linear** RGBA at
/// its export scale, plus the top-left pixel position. Built on the main thread (needs the egui font
/// atlas) so it can be blitted by the ctx-less export worker — like the brand watermark. The live
/// view / tour frames build and blit it in one step via [`stamp_location`].
pub(crate) struct HudOverlay {
    pub x: i32,
    pub y: i32,
    pub w: usize,
    pub h: usize,
    pub px: Vec<f32>, // interleaved RGBA, premultiplied, linear
}

/// Rasterize the location HUD for an image of height `img_h` into a standalone overlay.
pub(crate) fn build_location_overlay(
    ctx: &egui::Context,
    vp: &fractadyne_core::Viewport,
    img_h: u32,
) -> Option<HudOverlay> {
    if img_h == 0 {
        return None;
    }
    let log2mag = vp.log2_magnification();
    let lines = [
        format!("zoom {}\u{00d7}", crate::fmt_zoom_log2(log2mag)),
        format!("re {}", crate::fmt_coord_deep(&vp.center_x, log2mag)),
        format!("im {}", crate::fmt_coord_deep(&vp.center_y, log2mag)),
    ];
    let scale = img_h as f32 / 1080.0;
    let ppp = ctx.pixels_per_point();
    let pts = (15.0 * scale / ppp).max(1.0);
    let font = egui::FontId::monospace(pts);
    let galleys: Vec<_> = lines
        .iter()
        .map(|t| ctx.fonts(|f| f.layout_no_wrap(t.clone(), font.clone(), egui::Color32::WHITE)))
        .collect();
    let line_h = galleys.iter().map(|g| g.size().y).fold(0.0f32, f32::max) * ppp;
    let maxw = galleys.iter().map(|g| g.size().x).fold(0.0f32, f32::max) * ppp;
    let pad = 8.0 * scale;
    let margin = 16.0 * scale;
    let bw = (maxw + 2.0 * pad).ceil().max(1.0) as usize;
    let bh = (line_h * galleys.len() as f32 + 2.0 * pad).ceil().max(1.0) as usize;
    // Local premultiplied-linear buffer seeded with the translucent dark panel (black, α = 0.5).
    let mut px = vec![0.0f32; bw * bh * 4];
    for p in px.chunks_exact_mut(4) {
        p[3] = 0.5; // premult black (rgb already 0)
    }
    let amber = {
        let c = egui::Rgba::from(crate::theme::BRAND_ACCENT);
        [c.r(), c.g(), c.b()]
    };
    ctx.fonts(|f| {
        let atlas = f.image();
        let aw = atlas.size[0];
        for (li, g) in galleys.iter().enumerate() {
            let color = if li == 0 { amber } else { [0.85, 0.86, 0.88] };
            let by = pad + li as f32 * line_h;
            for row in &g.rows {
                for gl in &row.glyphs {
                    let uv = gl.uv_rect;
                    if uv.max[0] <= uv.min[0] || uv.max[1] <= uv.min[1] {
                        continue;
                    }
                    let ox = (pad + (gl.pos.x + uv.offset.x) * ppp).round() as i32;
                    let oy = (by + (gl.pos.y + uv.offset.y) * ppp).round() as i32;
                    for ty in uv.min[1]..uv.max[1] {
                        for tx in uv.min[0]..uv.max[0] {
                            let cov = atlas.pixels[ty as usize * aw + tx as usize];
                            if cov <= 0.0 {
                                continue;
                            }
                            let dx = ox + (tx - uv.min[0]) as i32;
                            let dy = oy + (ty - uv.min[1]) as i32;
                            if dx < 0 || dy < 0 || dx >= bw as i32 || dy >= bh as i32 {
                                continue;
                            }
                            let i = (dy as usize * bw + dx as usize) * 4;
                            // premult text (color·cov, cov) OVER the current premult pixel.
                            let inv = 1.0 - cov;
                            px[i] = color[0] * cov + px[i] * inv;
                            px[i + 1] = color[1] * cov + px[i + 1] * inv;
                            px[i + 2] = color[2] * cov + px[i + 2] * inv;
                            px[i + 3] = cov + px[i + 3] * inv;
                        }
                    }
                }
            }
        }
    });
    Some(HudOverlay { x: (margin - pad).round() as i32, y: (margin - pad).round() as i32, w: bw, h: bh, px })
}

/// Composite a pre-built HUD overlay onto a linear RGBA frame buffer (no egui context needed).
pub(crate) fn blit_location_overlay(frame: &mut [f32], w: u32, h: u32, ov: &HudOverlay) {
    for dy in 0..ov.h as i32 {
        let iy = ov.y + dy;
        if iy < 0 || iy >= h as i32 {
            continue;
        }
        for dx in 0..ov.w as i32 {
            let ix = ov.x + dx;
            if ix < 0 || ix >= w as i32 {
                continue;
            }
            let s = (dy as usize * ov.w + dx as usize) * 4;
            let a = ov.px[s + 3];
            if a <= 0.0 {
                continue;
            }
            let d = (iy as usize * w as usize + ix as usize) * 4;
            let inv = 1.0 - a;
            frame[d] = ov.px[s] + frame[d] * inv;
            frame[d + 1] = ov.px[s + 1] + frame[d + 1] * inv;
            frame[d + 2] = ov.px[s + 2] + frame[d + 2] * inv;
        }
    }
}

/// Build + blit the location HUD in one step (live view / tour frames, which have the egui context).
pub(crate) fn stamp_location(ctx: &egui::Context, px: &mut [f32], w: u32, h: u32, vp: &fractadyne_core::Viewport) {
    if let Some(ov) = build_location_overlay(ctx, vp, h) {
        blit_location_overlay(px, w, h, &ov);
    }
}

/// Burn a callout (marker ring + leader line + label) at the target's frame pixel `(vpx, vpy)`.
/// Pick a top-left for a callout label near `anchor` that stays inside `bounds` and doesn't
/// overlap any label already placed this frame (`placed`). Tries the four diagonal positions
/// around the marker (up-right preferred), then nudges vertically as a fallback so several
/// callouts firing at once don't stack on top of each other. The chosen rect is pushed onto
/// `placed`. All coordinates share one space (egui points for the live view, device pixels for
/// export); the caller supplies the matching `off`/`pad`.
fn place_callout_label(
    anchor: egui::Pos2,
    sz: egui::Vec2,
    bounds: egui::Rect,
    off: f32,
    pad: egui::Vec2,
    placed: &mut Vec<egui::Rect>,
) -> egui::Pos2 {
    let candidates = [
        egui::pos2(anchor.x + off, anchor.y - off - sz.y), // up-right (preferred)
        egui::pos2(anchor.x - off - sz.x, anchor.y - off - sz.y), // up-left
        egui::pos2(anchor.x + off, anchor.y + off),        // down-right
        egui::pos2(anchor.x - off - sz.x, anchor.y + off), // down-left
    ];
    let rect_at = |lp: egui::Pos2| egui::Rect::from_min_size(lp - pad, sz + pad * 2.0);
    let inside = |r: egui::Rect| {
        r.min.x >= bounds.min.x && r.min.y >= bounds.min.y && r.max.x <= bounds.max.x && r.max.y <= bounds.max.y
    };
    for &lp in &candidates {
        let r = rect_at(lp);
        if inside(r) && !placed.iter().any(|p| p.intersects(r)) {
            placed.push(r);
            return lp;
        }
    }
    // Fallback: from the preferred spot (clamped into bounds), step down until clear — bounded so
    // it always terminates; wraps back to the top if it runs off the bottom.
    let mut lp = candidates[0];
    // Clamp x into [left margin, right margin]; the upper bound must not fall below the lower one
    // (a label wider than the frame — e.g. a tiny preview render — would otherwise invert the
    // clamp bounds and panic). `.max(lo)` pins it to the left margin in that degenerate case.
    let lo_x = bounds.min.x + pad.x + 2.0;
    lp.x = lp.x.clamp(lo_x, (bounds.max.x - sz.x - pad.x - 2.0).max(lo_x));
    lp.y = lp.y.max(bounds.min.y + pad.y + 2.0);
    let step = sz.y + pad.y * 2.0 + 4.0;
    for _ in 0..24 {
        if !placed.iter().any(|p| p.intersects(rect_at(lp))) {
            break;
        }
        lp.y += step;
        if rect_at(lp).max.y > bounds.max.y {
            lp.y = bounds.min.y + pad.y + 2.0;
        }
    }
    placed.push(rect_at(lp));
    lp
}

#[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2c: moves into the overlay module
fn stamp_callout(
    ctx: &egui::Context,
    px: &mut [f32],
    w: u32,
    h: u32,
    co: &Callout,
    vpx: f32,
    vpy: f32,
    alpha: f32,
    placed: &mut Vec<egui::Rect>,
) {
    if alpha <= 0.0 {
        return;
    }
    let ppp = ctx.pixels_per_point();
    let s = (h as f32 / 1080.0).max(0.4); // scale annotation geometry to the frame
    let accent = {
        let c = egui::Rgba::from(crate::theme::BRAND_ACCENT);
        [c.r(), c.g(), c.b()]
    };
    draw_ring(px, w, h, vpx, vpy, 7.0 * s, 2.0 * s, accent, alpha);
    let pts = (co.size * s / ppp).max(1.0);
    let galley = ctx.fonts(|f| f.layout_no_wrap(co.text.clone(), egui::FontId::proportional(pts), egui::Color32::WHITE));
    let (gw, gh) = (galley.size().x * ppp, galley.size().y * ppp);
    let off = 16.0 * s;
    let pad = 6.0 * s;
    // Place the label so concurrent callouts don't overlap (up-right preferred; nudged otherwise).
    let lp = place_callout_label(
        egui::pos2(vpx, vpy),
        egui::vec2(gw, gh),
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(w as f32, h as f32)),
        off,
        egui::vec2(pad, pad),
        placed,
    );
    let (bx, by) = (lp.x, lp.y);
    draw_line(px, w, h, vpx, vpy, bx + gw * 0.5, by + gh * 0.5, accent, alpha * 0.9);
    fill_dark(px, w, h, bx - pad, by - pad, bx + gw + pad, by + gh + pad, (alpha * 0.55).min(1.0));
    ctx.fonts(|f| blit_galley(&f.image(), px, w, h, &galley, bx, by, ppp, [1.0, 1.0, 1.0], alpha));
}

/// Rasterize the escape-time orbit of `point` (z → z²+c from 0 for Mandelbrot, or from `point`
/// with c = `julia_c` for Julia) onto an export frame — the offscreen twin of `draw_orbit`.
fn stamp_orbit(px: &mut [f32], w: u32, h: u32, vp: &fractadyne_core::Viewport, point: (f64, f64), julia: bool, julia_c: (f64, f64)) {
    let (mut zx, mut zy, cx, cy) = if julia {
        (point.0, point.1, julia_c.0, julia_c.1)
    } else {
        (0.0, 0.0, point.0, point.1)
    };
    let mut zs = vec![(zx, zy)];
    for _ in 0..96 {
        let (nx, ny) = (zx * zx - zy * zy + cx, 2.0 * zx * zy + cy);
        zx = nx;
        zy = ny;
        zs.push((zx, zy));
        if zx * zx + zy * zy > 16.0 {
            break; // escaped
        }
    }
    let p = vp.precision;
    let pts: Vec<(f32, f32)> = zs
        .iter()
        .map(|&(x, y)| {
            let (a, b) = vp.complex_to_pixel(
                &fractadyne_core::BigFloat::from_f64(x, p),
                &fractadyne_core::BigFloat::from_f64(y, p),
            );
            (a as f32, b as f32)
        })
        .collect();
    let line = [0.95, 0.75, 0.30]; // amber path
    for seg in pts.windows(2) {
        draw_line(px, w, h, seg[0].0, seg[0].1, seg[1].0, seg[1].1, line, 0.85);
    }
    for &(mx, my) in &pts {
        // small white dot per orbit point
        for dy in -1..=1 {
            for dx in -1..=1 {
                blend_px(px, w, h, mx.round() as i32 + dx, my.round() as i32 + dy, [1.0, 1.0, 1.0], 0.9);
            }
        }
    }
}

/// Resolve one `[[palette]]` definition into gradient stops (or a preset reference).
fn resolve_palette_def(p: &PaletteFile) -> Result<TourPalette, String> {
    if let Some(spec) = &p.preset {
        let idx = preset_index(spec)
            .ok_or_else(|| format!("palette \"{}\": unknown preset \"{spec}\"", p.id))?;
        return Ok(TourPalette::Preset(idx));
    }
    if p.stops.is_empty() {
        return Err(format!("palette \"{}\": needs either `preset` or `stops`", p.id));
    }
    if p.stops.len() > fractadyne_color::MAX_STOPS {
        return Err(format!(
            "palette \"{}\": {} stops, but at most {} are supported",
            p.id,
            p.stops.len(),
            fractadyne_color::MAX_STOPS
        ));
    }
    let mut stops = Vec::with_capacity(p.stops.len());
    for s in &p.stops {
        let c = parse_hex_color(&s.color)
            .ok_or_else(|| format!("palette \"{}\": color \"{}\" is not #rrggbb", p.id, s.color))?;
        stops.push([s.at.clamp(0.0, 1.0) as f32, c[0], c[1], c[2]]);
    }
    stops.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));
    Ok(TourPalette::Stops(stops))
}

/// Built-in preset by name (case-insensitive) or index.
fn preset_index(spec: &str) -> Option<usize> {
    let s = spec.trim();
    s.parse::<usize>()
        .ok()
        .filter(|i| *i < fractadyne_color::PRESETS.len())
        .or_else(|| fractadyne_color::PRESETS.iter().position(|p| p.name.eq_ignore_ascii_case(s)))
}

/// `#rrggbb` (or bare `rrggbb`) → linear RGB, matching how the presets store their colors.
fn parse_hex_color(s: &str) -> Option<[f32; 3]> {
    let h = s.trim().trim_start_matches('#');
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let srgb_to_linear = |c: f32| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let v = u32::from_str_radix(h, 16).ok()?;
    Some([
        srgb_to_linear(((v >> 16) & 0xff) as f32 / 255.0),
        srgb_to_linear(((v >> 8) & 0xff) as f32 / 255.0),
        srgb_to_linear((v & 0xff) as f32 / 255.0),
    ])
}

/// Resolve a parsed v2 script into a playable tour: keyframe times validated against each other,
/// centers and zooms parsed at enough precision for the depth they're viewed at, `[[location]]` /
/// `[[palette]]` references dereferenced, annotations split by kind, chapters closed off. Errors
/// name the offending element (its `id`, or `#index` when it has none) so a broken script says
/// exactly what to fix.
fn resolve_script(sf: ScriptFile, bench: Option<Bench>) -> Result<Playback, String> {
    use std::collections::HashMap;
    if sf.keyframe.is_empty() {
        return Err("script has no [[keyframe]] — a tour needs at least one".to_string());
    }
    // --- named locations ---
    let mut locs: HashMap<&str, &LocationFile> = HashMap::new();
    for l in &sf.location {
        if locs.insert(l.id.as_str(), l).is_some() {
            return Err(format!("duplicate [[location]] id \"{}\"", l.id));
        }
    }
    // --- named palettes (a keyframe may also name a preset directly, appended on demand) ---
    let mut palettes: Vec<TourPalette> = Vec::new();
    let mut pal_ids: HashMap<String, usize> = HashMap::new();
    for p in &sf.palette {
        let resolved = resolve_palette_def(p)?;
        if pal_ids.contains_key(&p.id) {
            return Err(format!("duplicate [[palette]] id \"{}\"", p.id));
        }
        pal_ids.insert(p.id.clone(), palettes.len());
        palettes.push(resolved);
    }

    // --- keyframes ---
    // PRE-PASS: the deepest zoom anywhere in the tour. Every centre is then parsed at THAT
    // precision, not at its own keyframe's.
    //
    // Parsing each centre at its own depth looks right and is badly wrong: the camera interpolates
    // BETWEEN keyframes, so a glide from 1e30× to 1e55× is limited by the 1e30× endpoint's
    // precision. The spar coordinate carries 120 digits; parsed at 1e30×'s ~140 bits it loses the
    // rest, and by 1e55× the resulting centre error is larger than the whole view span — the camera
    // is pointed somewhere else entirely. Measured live: every reference along that glide escaped
    // after 626 samples (a point nowhere near the set boundary), the prefetch queue rebuilt one
    // 13,529 times in six minutes, and the run ended in a lost GPU device. The lookahead makes it
    // worse still, since it targets depths ahead of the current view.
    let deepest_octaves = {
        let (mut zoom, mut deepest) = (None::<String>, 0.0f64);
        for k in &sf.keyframe {
            if let Some(z) = &k.zoom {
                zoom = Some(z.as_string());
            } else if let Some(z) = k
                .location
                .as_ref()
                .and_then(|n| locs.get(n.as_str()))
                .and_then(|l| l.zoom.as_ref())
            {
                zoom = Some(z.as_string());
            }
            if let Some(z) = &zoom {
                // A malformed zoom is reported by the main pass, with the keyframe's id attached.
                if let Ok(l10) = parse_zoom_log10(z) {
                    deepest = deepest.max(l10.max(0.0) * std::f64::consts::LN_10);
                }
            }
        }
        (deepest / std::f64::consts::LN_2).max(0.0).ceil() as u64
    };
    let center_prec = fractadyne_core::precision_for_octaves(deepest_octaves);
    let mut kfs: Vec<Kf> = Vec::with_capacity(sf.keyframe.len());
    let mut prev_end = 0.0f64; // when the previous keyframe finishes holding
    let mut center: Option<(String, String)> = None;
    let mut zoom: Option<String> = None;
    // State inherited forward until a keyframe changes it.
    let mut fractal = FractalKind::Mandelbrot;
    let (mut julia, mut dual, mut julia_c, mut orbits, mut orbit) = (false, false, None, false, None);
    let mut minimap = false;
    let (mut max_iter, mut palette): (Option<u32>, Option<usize>) = (None, None);
    for (i, k) in sf.keyframe.iter().enumerate() {
        let id = k.id.clone().unwrap_or_else(|| format!("#{}", i + 1));
        let at = match k.t {
            Some(t) if t.is_finite() && t >= 0.0 => t,
            Some(t) => return Err(format!("keyframe {id}: t = {t} is not a valid time")),
            None if i == 0 => 0.0,
            None => {
                return Err(format!(
                    "keyframe {id}: `t` is required — the absolute second the camera arrives here"
                ))
            }
        };
        if at < prev_end - 1.0e-9 {
            return Err(format!(
                "keyframe {id}: t = {at} is before the previous keyframe finishes holding \
                 ({prev_end}). Keyframe times are absolute seconds from the start of the tour."
            ));
        }
        let hold = k.hold.max(0.0);
        prev_end = at + hold;
        // Center: a named location, an inline pair, or inherited from the previous keyframe.
        let loc = match &k.location {
            Some(name) => Some(
                *locs
                    .get(name.as_str())
                    .ok_or_else(|| format!("keyframe {id}: unknown location \"{name}\""))?,
            ),
            None => None,
        };
        if let Some(l) = loc {
            center = Some((l.re.clone(), l.im.clone()));
        }
        match (&k.re, &k.im) {
            (Some(re), Some(im)) => center = Some((re.clone(), im.clone())),
            (None, None) => {}
            _ => return Err(format!("keyframe {id}: `re` and `im` must be given together")),
        }
        // Zoom: explicit wins over the named location's default, else inherited, else 1×.
        if let Some(z) = &k.zoom {
            zoom = Some(z.as_string());
        } else if let Some(z) = loc.and_then(|l| l.zoom.as_ref()) {
            zoom = Some(z.as_string());
        }
        let logmag = match &zoom {
            Some(z) => {
                parse_zoom_log10(z).map_err(|e| format!("keyframe {id}: {e}"))?.max(0.0)
                    * std::f64::consts::LN_10
            }
            None => 0.0,
        };
        // Parse the centre at the DEEPEST depth the tour reaches (see `center_prec` above), not at
        // this keyframe's — the camera interpolates between keyframes and the lookahead builds
        // references for depths ahead of the current one, so a shallow keyframe's centre still has
        // to carry the digits a deep neighbour needs. An exact rational like (37+16i)/100 is only
        // as good as the bits it is evaluated to, which is the same argument.
        let prec = center_prec;
        let (re, im) = center
            .clone()
            .unwrap_or_else(|| ("-0.5".to_string(), "0.0".to_string()));
        let cx = fractadyne_core::parse_bf_prec(&re, prec)
            .ok_or_else(|| format!("keyframe {id}: invalid coordinate re = \"{re}\""))?;
        let cy = fractadyne_core::parse_bf_prec(&im, prec)
            .ok_or_else(|| format!("keyframe {id}: invalid coordinate im = \"{im}\""))?;
        if let Some(name) = &k.fractal {
            fractal = FractalKind::from_name(name)
                .ok_or_else(|| format!("keyframe {id}: unknown fractal \"{name}\""))?;
        }
        if let Some(j) = k.julia {
            julia = j;
        }
        if let Some(d) = k.dual {
            dual = d;
        }
        if let (Some(r), Some(i)) = (k.julia_re, k.julia_im) {
            julia_c = Some((r, i));
        }
        if let Some(o) = k.orbits {
            orbits = o;
        }
        if let Some(m) = k.minimap {
            minimap = m;
        }
        if let (Some(r), Some(i)) = (k.orbit_re, k.orbit_im) {
            orbit = Some((r, i));
        }
        if let Some(m) = k.max_iter {
            max_iter = Some(m.max(1));
        }
        if let Some(spec) = &k.palette {
            palette = Some(match pal_ids.get(spec) {
                Some(i) => *i,
                None => {
                    // Not a [[palette]] id — a built-in preset named (or indexed) inline.
                    let idx = preset_index(spec)
                        .ok_or_else(|| format!("keyframe {id}: unknown palette \"{spec}\""))?;
                    *pal_ids.entry(spec.clone()).or_insert_with(|| {
                        palettes.push(TourPalette::Preset(idx));
                        palettes.len() - 1
                    })
                }
            });
        }
        kfs.push(Kf {
            id: id.clone(),
            at,
            hold,
            ease: EaseKind::parse(k.ease.as_deref()),
            cx,
            cy,
            logmag,
            fractal,
            julia,
            dual,
            julia_c,
            orbits,
            minimap,
            orbit,
            max_iter,
            palette,
        });
    }
    let total = kfs.last().map(|k| k.at + k.hold).unwrap_or(0.0);
    // Annotation anchors are viewed at whatever depth the camera reaches, so parse them at the
    // deepest keyframe's precision — a callout on a 1e94× dendrite is useless at 64 bits.
    let anchor_prec = center_prec;

    // --- annotations (one array, tagged by kind) ---
    let (mut captions, mut callouts, mut spotlights) = (Vec::new(), Vec::new(), Vec::new());
    for (i, a) in sf.annotation.iter().enumerate() {
        let id = a.id.clone().unwrap_or_else(|| format!("#{}", i + 1));
        let start = a.t.max(0.0);
        let end = if a.secs > 0.0 { start + a.secs } else { total.max(start) };
        let fade = a.fade.unwrap_or(0.4).max(0.0);
        let kind = a.kind.trim().to_ascii_lowercase();
        // Anchor coordinate, for the kinds that have one.
        let anchor = || -> Result<(fractadyne_core::BigFloat, fractadyne_core::BigFloat), String> {
            let (re, im) = match (&a.location, &a.re, &a.im) {
                (Some(name), _, _) => {
                    let l = locs
                        .get(name.as_str())
                        .ok_or_else(|| format!("{kind} {id}: unknown location \"{name}\""))?;
                    (l.re.clone(), l.im.clone())
                }
                (None, Some(re), Some(im)) => (re.clone(), im.clone()),
                _ => {
                    return Err(format!(
                        "{kind} {id}: needs an anchor — either location = \"id\" or both re and im"
                    ))
                }
            };
            let cx = fractadyne_core::parse_bf_prec(&re, anchor_prec)
                .ok_or_else(|| format!("{kind} {id}: invalid coordinate re = \"{re}\""))?;
            let cy = fractadyne_core::parse_bf_prec(&im, anchor_prec)
                .ok_or_else(|| format!("{kind} {id}: invalid coordinate im = \"{im}\""))?;
            Ok((cx, cy))
        };
        let text = || -> Result<String, String> {
            a.text
                .clone()
                .filter(|t| !t.is_empty())
                .ok_or_else(|| format!("{kind} {id}: `text` is required"))
        };
        match kind.as_str() {
            "caption" => captions.push(Caption {
                text: text()?,
                start,
                end,
                fade,
                pos: match a.pos.as_deref().map(|s| s.to_ascii_lowercase()).as_deref() {
                    Some("top") => CaptionPos::Top,
                    Some("center" | "centre" | "middle") => CaptionPos::Center,
                    _ => CaptionPos::Bottom,
                },
                size: a.size.unwrap_or(22.0).clamp(8.0, 96.0) as f32,
            }),
            "callout" => {
                let (cx, cy) = anchor()?;
                callouts.push(Callout {
                    text: text()?,
                    cx,
                    cy,
                    start,
                    end,
                    fade,
                    size: a.size.unwrap_or(18.0).clamp(8.0, 96.0) as f32,
                });
            }
            "spotlight" => {
                let (cx, cy) = anchor()?;
                spotlights.push(Spotlight {
                    cx,
                    cy,
                    radius: a.radius.unwrap_or(0.25).clamp(0.02, 2.0) as f32,
                    soft: a.softness.unwrap_or(0.08).clamp(0.0, 1.0) as f32,
                    dim: a.dim.unwrap_or(0.7).clamp(0.0, 1.0) as f32,
                    start,
                    end,
                    fade,
                });
            }
            other => {
                return Err(format!(
                    "annotation {id}: unknown kind \"{other}\" (expected caption, callout, or spotlight)"
                ))
            }
        }
    }

    // --- chapters: each runs to the start of the next ---
    let mut segments: Vec<Segment> = sf
        .segment
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let id = s
                .id
                .clone()
                .or_else(|| s.title.clone())
                .unwrap_or_else(|| format!("{}", i + 1));
            Segment {
                title: s.title.clone().unwrap_or_else(|| id.clone()),
                id,
                start: s.t.max(0.0),
                end: total,
            }
        })
        .collect();
    segments.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
    for i in 0..segments.len().saturating_sub(1) {
        segments[i].end = segments[i + 1].start;
    }

    Ok(Playback {
        name: if sf.name.is_empty() { "Script".to_string() } else { sf.name },
        source: None, // set by the loaders below; a synthesized tour has no file
        kfs,
        total,
        started: false,
        loop_: sf.loop_,
        bench,
        captions,
        callouts,
        spotlights,
        palettes,
        segments,
        speed: 1.0,
        paused: false,
        finished: false,
        render: resolve_render(&sf.render)?,
        paced_hold: 0.0,
        pace: Pace::parse(sf.playback.pace.as_deref())?,
        settle_timeout: sf.playback.settle_timeout.unwrap_or(20.0).max(0.0),
        settle_waited: 0.0,
        settle_kf: usize::MAX,
        pace_waited: 0.0,
        pace_released: false,
        cur_t: 0.0,
        last_now: None,
        wall_t: 0.0,
    })
}

/// Resolve the `[render]` table (size string, mp4 spec) into the form the renderer merges with
/// the CLI flags.
fn resolve_render(r: &RenderFile) -> Result<TourRender, String> {
    let (width, height) = match &r.size {
        Some(s) => {
            let (w, h) = crate::parse_size(s);
            if w.is_none() {
                return Err(format!("[render] size = \"{s}\": expected WIDTHxHEIGHT or a width"));
            }
            (w, h)
        }
        None => (None, None),
    };
    Ok(TourRender {
        width,
        height,
        fps: r.fps.filter(|f| *f > 0.0),
        ss: r.ss.map(|s| s.clamp(1, 8)),
        prefix: r.prefix.clone(),
        out: r.out.as_ref().map(std::path::PathBuf::from),
        mp4: match &r.mp4 {
            None | Some(Mp4Spec::Flag(false)) => None,
            Some(Mp4Spec::Flag(true)) => Some(None),
            Some(Mp4Spec::Path(p)) => Some(Some(std::path::PathBuf::from(p))),
        },
        max_iter: r.max_iter,
        auto_iter: r.auto_iter,
        show_location: r.show_location.unwrap_or(false),
        normalize: r.normalize.unwrap_or(false),
    })
}

/// Built-in benchmark tour: a steady zoom into a Seahorse-Valley spiral over a fixed
/// timeline, so successive builds / machines render the same work.
fn benchmark_playback() -> Playback {
    let cx = "-0.743643887037158704752191506114774";
    let cy = "0.131825904205311970493132056385139";
    let zooms = ["1", "1e3", "1e6", "1e9", "1e12"];
    let keyframe = zooms
        .iter()
        .enumerate()
        .map(|(i, z)| KeyframeFile {
            t: Some(i as f64 * 4.0), // one 4-second glide per decade-triple
            re: Some(cx.to_string()),
            im: Some(cy.to_string()),
            zoom: Some(NumOrStr::Str(z.to_string())),
            fractal: Some("Mandelbrot".to_string()),
            julia: Some(false),
            ..Default::default()
        })
        .collect();
    let sf = ScriptFile {
        name: "Built-in benchmark".to_string(),
        loop_: false,
        keyframe,
        ..Default::default()
    };
    resolve_script(sf, Some(Bench::new())).expect("valid benchmark script")
}

impl FractadyneApp {
    /// Start the built-in benchmark tour.
    pub(crate) fn start_benchmark(&mut self) {
        self.dual = false; // benchmark measures the single-view pipeline
        self.playback = Some(benchmark_playback());
    }

    /// Apply a frame's scripted coloring. A preset is selected outright (so a static tour colors
    /// exactly as picking that preset would); a keyframe-to-keyframe morph installs blended stops
    /// as the custom palette. Either way binary/duotone are cleared, so a deep exterior-only view
    /// renders as a gradient rather than one flat color.
    pub(crate) fn apply_script_palette(&mut self, pal: &PaletteApply) {
        match pal {
            PaletteApply::Preset(i) => {
                self.coloring.palette_idx = (*i).min(fractadyne_color::PRESETS.len() - 1);
                self.coloring.use_custom_palette = false;
            }
            PaletteApply::Stops(stops) => {
                self.coloring.custom_palette = stops.clone();
                self.coloring.use_custom_palette = true;
            }
        }
        self.coloring.use_binary = false;
        self.coloring.use_duotone = false;
    }

    /// Is a tour actually RUNNING — as opposed to merely loaded? A finished tour keeps its player
    /// on screen parked at the final keyframe, and for everything that asks "is the camera being
    /// driven?" that state is an idle view, not playback: it may settle its AA, arm frame-cost
    /// measurements, and use the progressive cold start, exactly as if a user had stopped there.
    /// Callers that mean "is a tour loaded at all?" (its annotations, Esc, the Stop menu item) keep
    /// testing `playback.is_some()`.
    pub(crate) fn tour_playing(&self) -> bool {
        self.playback.as_ref().is_some_and(|p| !p.finished)
    }

    /// Stop the active tour and hand the viewer their own iteration budget and coloring back.
    /// Every path that ends playback goes through here — Esc, Tools → Stop playback, autopilot,
    /// and the tour running out — or a script's settings would silently become the session's.
    pub(crate) fn stop_playback(&mut self) {
        self.playback = None;
        if let Some(r) = self.playback_restore.take() {
            self.render_cfg.max_iter = r.max_iter;
            self.render_cfg.auto_iter = r.auto_iter;
            self.coloring.palette_idx = r.palette_idx;
            self.coloring.use_custom_palette = r.use_custom_palette;
            self.coloring.use_binary = r.use_binary;
            self.coloring.use_duotone = r.use_duotone;
            self.dialogs.minimap = r.minimap;
            self.anim.show_orbits = r.show_orbits;
        }
    }

    /// Apply the per-frame render settings a script states (iteration budget, palette). The
    /// budget is EXACT — a keyframe that asks for 8M iterations at 1e94× means 8M, not 8M scaled
    /// again by depth. Settings the script leaves unset are left alone.
    pub(crate) fn apply_sampled_settings(&mut self, s: &Sampled) {
        if let Some(m) = s.max_iter {
            self.render_cfg.max_iter = m;
            self.render_cfg.auto_iter = false;
        }
        if let Some(p) = &s.palette {
            self.apply_script_palette(p);
        }
    }

    /// Load a camera-tour script (TOML) via a file dialog and start playing it.
    /// Menu / picker entry: choose a script file and play it. The picker opens in the last
    /// script's directory when there is one.
    pub(crate) fn load_script(&mut self) {
        let mut dialog = rfd::FileDialog::new().add_filter("Fractadyne script (TOML)", &["toml"]);
        // Prefer the last script's own directory; otherwise the shared last-used directory.
        let seed = self
            .last_script
            .as_ref()
            .and_then(|p| p.parent())
            .filter(|d| d.is_dir())
            .map(|d| d.to_path_buf())
            .unwrap_or_else(|| self.dialog_dir_default());
        dialog = dialog.set_directory(seed);
        let Some(path) = dialog.pick_file() else {
            return;
        };
        self.remember_dir(&path);
        self.start_script(path);
    }

    /// Toolbar ▶ entry: replay the last script with one click, falling back to the picker when
    /// there is none or the remembered file has moved. The common case — re-playing the same tour
    /// (during this session's crash hunting it meant a file dialog per attempt) — is now one click.
    pub(crate) fn play_last_or_pick_script(&mut self) {
        match self.last_script.clone() {
            Some(path) if path.is_file() => self.start_script(path),
            _ => self.load_script(),
        }
    }

    /// Load `path`, remember it as the last script (for the toolbar/menu default), and start
    /// playback. A parse error reports through the same panel the picker used.
    pub(crate) fn start_script(&mut self, path: std::path::PathBuf) {
        match read_script(&path).and_then(|sf| resolve_script(sf, None)) {
            Ok(mut pb) => {
                pb.source = Some(path.clone());
                self.last_script = Some(path);
                // Restore the viewer's own iteration/palette settings when the tour ends — a
                // script's budget is the script's, not a permanent change to the session.
                self.playback_restore = Some(PlaybackRestore {
                    max_iter: self.render_cfg.max_iter,
                    auto_iter: self.render_cfg.auto_iter,
                    palette_idx: self.coloring.palette_idx,
                    use_custom_palette: self.coloring.use_custom_palette,
                    use_binary: self.coloring.use_binary,
                    use_duotone: self.coloring.use_duotone,
                    minimap: self.dialogs.minimap,
                    show_orbits: self.anim.show_orbits,
                });
                self.playback = Some(pb);
            }
            Err(e) => {
                self.dialogs.notice = Some((
                    "Could not load script".to_string(),
                    format!("{}\n\n{e}", path.display()),
                ));
            }
        }
    }

    /// Draw the active tour captions over the fractal (live playback). Each caption fades in/out
    /// per its timeline window; text is wrapped and centered on its screen anchor over a soft dark
    /// backing so it stays legible on any fractal. (Exported tour frames get the same via a
    /// rasterized overlay — see `render_tour_to_dir`.)
    /// Returns the backing rects of the captions drawn this frame, so callouts can avoid them.
    pub(crate) fn draw_captions(&self, ctx: &egui::Context, rect: egui::Rect) -> Vec<egui::Rect> {
        let Some(pb) = &self.playback else { return Vec::new() };
        if pb.captions.is_empty() {
            return Vec::new();
        }
        let painter =
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("tour_captions")));
        let mut rects = Vec::new();
        for cap in &pb.captions {
            let a = cap.alpha_at(pb.cur_t);
            if a <= 0.0 {
                continue;
            }
            let color = egui::Color32::from_white_alpha((a * 240.0) as u8);
            let galley = ctx.fonts(|f| {
                f.layout(cap.text.clone(), egui::FontId::proportional(cap.size), color, rect.width() * 0.8)
            });
            let sz = galley.size();
            let x = rect.center().x - sz.x * 0.5;
            let y = match cap.pos {
                CaptionPos::Top => rect.top() + rect.height() * 0.07,
                CaptionPos::Center => rect.center().y - sz.y * 0.5,
                CaptionPos::Bottom => rect.bottom() - rect.height() * 0.09 - sz.y,
            };
            let pos = egui::pos2(x, y);
            let pad = egui::vec2(12.0, 7.0);
            let bg = egui::Rect::from_min_size(pos - pad, sz + pad * 2.0);
            painter.rect_filled(bg, 5.0, egui::Color32::from_black_alpha((a * 130.0) as u8));
            painter.galley(pos, galley, color);
            rects.push(bg);
        }
        rects
    }

    /// Draw the active tour callouts (live playback): a marker ring at each anchored fractal
    /// coordinate — tracking the point as the view moves — plus a labeled leader. Off-screen
    /// anchors are skipped. Exported frames get the same via `stamp_callout`.
    pub(crate) fn draw_callouts(&self, ctx: &egui::Context, rect: egui::Rect, caption_rects: &[egui::Rect]) {
        let Some(pb) = &self.playback else { return };
        if pb.callouts.is_empty() {
            return;
        }
        let ppp = ctx.pixels_per_point() as f64;
        let painter =
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("tour_callouts")));
        let with_a = |c: egui::Color32, a: f32| {
            egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a * 255.0) as u8)
        };
        let pad = egui::vec2(6.0, 4.0);
        // Labels placed so far this frame — new ones avoid overlapping them (several callouts can be
        // on-screen at once, e.g. the intro landmark labels). Seed with the caption rects so labels
        // don't collide with the title/subtitle text either.
        let mut placed: Vec<egui::Rect> = caption_rects.to_vec();
        for co in &pb.callouts {
            let a = co.alpha_at(pb.cur_t);
            if a <= 0.0 {
                continue;
            }
            // The viewport tracks device pixels; convert to egui points, offset by the panel origin.
            let (vpx, vpy) = self.viewport.complex_to_pixel(&co.cx, &co.cy);
            let sp = egui::pos2(rect.min.x + (vpx / ppp) as f32, rect.min.y + (vpy / ppp) as f32);
            if !rect.contains(sp) {
                continue;
            }
            let accent = with_a(crate::theme::BRAND_ACCENT, a);
            painter.circle_stroke(sp, 7.0, egui::Stroke::new(2.0, accent));
            painter.circle_filled(sp, 1.8, accent);
            let galley = ctx.fonts(|f| {
                f.layout_no_wrap(co.text.clone(), egui::FontId::proportional(co.size), with_a(egui::Color32::WHITE, a))
            });
            let gs = galley.size();
            let lp = place_callout_label(sp, gs, rect, 16.0, pad, &mut placed);
            painter.line_segment([sp, lp + gs * 0.5], egui::Stroke::new(1.5, accent));
            let bg = egui::Rect::from_min_size(lp - pad, gs + pad * 2.0);
            painter.rect_filled(bg, 4.0, egui::Color32::from_black_alpha((a * 150.0) as u8));
            painter.galley(lp, galley, with_a(egui::Color32::WHITE, a));
        }
    }

    /// Prepare an interrupted sequence for `--resume`: verify the frames already on disk, discard
    /// any TRAILING ones that are unusable, and confirm the survivors were rendered at this size.
    ///
    /// Necessary because resume's per-frame test is `frame_path.exists()`, and the frame a render
    /// dies on is exactly the one that is present but INCOMPLETE — the disk-full failure at frame
    /// 1091 left a truncated PNG that a naive resume would have kept forever, baking a corrupt
    /// frame into the middle of a 9,931-frame sequence. So the newest frame is checked, and if it
    /// fails, the one before it, and so on until a good one is found: a render can only ever be
    /// killed mid-write on the frame it was writing, so in practice this discards one file.
    ///
    /// Validity here is STRUCTURAL — magic, `IHDR` dimensions, terminating `IEND` — which is
    /// precisely what a truncated or partially-flushed write breaks, and costs a few bytes of IO
    /// per frame instead of a full decode of thousands of 4K images. It would not catch a file
    /// whose pixel data is corrupt but whose framing survived; nothing short of re-rendering
    /// would, and that is not a failure mode an interrupted write produces.
    fn prepare_resume(
        out_dir: &std::path::Path,
        prefix: &str,
        want_w: u32,
        want_h: u32,
    ) -> Result<String, String> {
        let mut idx: Vec<u64> = Vec::new();
        let Ok(rd) = std::fs::read_dir(out_dir) else {
            return Ok(String::new()); // nothing rendered yet
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let Some(rest) = name.strip_prefix(&format!("{prefix}_")) {
                if let Some(num) = rest.strip_suffix(".png") {
                    if let Ok(n) = num.parse::<u64>() {
                        idx.push(n);
                    }
                }
            }
        }
        if idx.is_empty() {
            return Ok(String::new());
        }
        idx.sort_unstable();
        let mut discarded = 0usize;
        while let Some(&top) = idx.last() {
            let path = out_dir.join(format!("{prefix}_{top:05}.png"));
            match png_frame_size(&path) {
                Some((w, h)) if w == want_w && h == want_h => {
                    return Ok(format!(
                        "Resuming: {} frames on disk (through {top}){}",
                        idx.len(),
                        if discarded > 0 {
                            format!(", discarded {discarded} incomplete")
                        } else {
                            String::new()
                        }
                    ));
                }
                // A COMPLETE frame at the wrong size means the folder holds a different render.
                // Resuming would interleave two resolutions into one sequence, so refuse and say
                // so rather than silently producing an unusable mix.
                Some((w, h)) => {
                    return Err(format!(
                        "{} holds {w}×{h} frames but this render is {want_w}×{want_h} — \
                         use a different output folder, or turn Resume off to re-render",
                        out_dir.display()
                    ));
                }
                None => {
                    // Incomplete/unreadable: drop it and check the frame before it.
                    let _ = std::fs::remove_file(&path);
                    discarded += 1;
                    idx.pop();
                }
            }
        }
        Ok(format!("Resuming: no usable frames found, discarded {discarded} incomplete"))
    }

    /// Render a keyframe-tour script (TOML) to a numbered PNG frame sequence — the headless
    /// `--render-tour` mode for producing a deep-zoom dive video. Steps the timeline at a
    /// fixed `fps`, rendering each frame at `width×height` (× `ss` supersampling) via the
    /// offscreen export path. Blocking; assemble the frames afterward (e.g. with ffmpeg).
    pub(crate) fn render_tour_to_dir(
        &mut self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        script_path: &std::path::Path,
        cli: &TourRenderConfig,
    ) -> Result<String, String> {
        let pb = resolve_script(read_script(script_path)?, None)?;
        let cfg = cli.resolve(&pb.render, script_path);
        let (width, height, fps, out_dir, prefix) =
            (cfg.width, cfg.height, cfg.fps, cfg.out.clone(), cfg.prefix.clone());
        let (overwrite, resume, mp4) = (cli.overwrite, cli.resume, cfg.mp4.clone());
        let show_location = self.show_location || pb.render.show_location;
        // --segment: render just one chapter, keeping the GLOBAL frame numbering so its frames drop
        // straight back into the full sequence (re-cutting ten seconds of narration must not mean
        // re-rendering — or renumbering — the whole tour).
        let (first_frame, last_frame) = match &cli.segment {
            Some(name) => {
                let seg = pb.find_segment(name)?;
                say(&format!(
                    "Segment \"{}\" — {:.1}s to {:.1}s of \"{}\"",
                    seg.title, seg.start, seg.end, pb.name
                ));
                ((seg.start * fps).floor() as u64, (seg.end * fps).ceil() as u64)
            }
            None => (0, u64::MAX),
        };
        // Total frame count — the SAME formula on every machine, which is what the sharding below
        // depends on (all hosts must agree on F for the ranges to tile).
        let frames: u64 = if pb.total <= 0.0 { 1 } else { (pb.total * fps).round() as u64 + 1 };
        let last_frame = last_frame.min(frames.saturating_sub(1));
        // --segments N --segment-index K: intersect the (possibly chapter-restricted) range with
        // this shard's half-open `[start, end)` — see `segment_range` for why the formula tiles.
        let (first_frame, last_frame) = if let Some(n) = cli.segments {
            if n == 0 {
                return Err("--segments must be at least 1".into());
            }
            let k = cli.segment_index.unwrap_or(0);
            if k >= n {
                return Err(format!(
                    "--segment-index {k} is out of range for --segments {n} (valid: 0..={})",
                    n - 1
                ));
            }
            let (s, e) = segment_range(frames, n as u64, k as u64);
            say(&format!(
                "Shard {k} of {n}: frames [{s}, {e}) of {frames} total ({} in this shard)",
                e - s
            ));
            if e <= s {
                return Ok(format!(
                    "Shard {k}/{n} is empty ({frames} frames split {n} ways) — nothing to render."
                ));
            }
            (first_frame.max(s), last_frame.min(e - 1))
        } else {
            (first_frame, last_frame)
        };
        // --dry-run: report the plan and stop BEFORE touching the output directory, so a farm
        // script can verify its shards tile across hosts before committing hours of GPU.
        if cli.dry_run {
            let planned = (last_frame + 1).saturating_sub(first_frame);
            return Ok(format!(
                "Dry run: would render frames {first_frame}..={last_frame} ({planned} of {frames} \
                 total) at {width}×{height} ss{} to {}",
                cfg.ss,
                out_dir.display()
            ));
        }
        std::fs::create_dir_all(&out_dir)
            .map_err(|e| format!("create {}: {e}", out_dir.display()))?;
        write_render_status(&out_dir, "running");
        // Immediately-invoked closure so the status marker below brackets EVERY exit path —
        // the `?`s and early `return Ok(…)`s inside become the closure's returns.
        let render_result = (|| -> Result<String, String> {
        // Vet what is already there BEFORE rendering a frame, so a mismatch is reported in seconds
        // rather than after the render has appended to a sequence it can never complete.
        if resume {
            let note = Self::prepare_resume(&out_dir, &prefix, width, height)?;
            if !note.is_empty() {
                say(&note); // say() mirrors to the render log itself
            }
        }
        // Single-view offscreen render at the requested frame size.
        self.dual = false;
        // Iteration budget for frames whose keyframes don't state their own: the script's
        // `[render]` block decides when it says so, otherwise auto-scale from a high base (a fixed
        // low cap renders deep structure as blobs). The "match the on-screen budget" export cap
        // does NOT apply to tours — see `export_auto_iter_cap`; without that exemption every deep
        // frame here rendered flat. Per-keyframe `max_iter` overrides this per frame.
        let (base_iter, base_auto) = (
            pb.render.max_iter.unwrap_or_else(|| self.render_cfg.max_iter.max(500_000)),
            pb.render.auto_iter.unwrap_or(true),
        );
        self.viewport = fractadyne_core::Viewport::new(width as f64, height as f64);
        self.export.width = width;
        self.export.ss = cfg.ss;
        let planned = (last_frame + 1).saturating_sub(first_frame);
        say(&format!(
            "Rendering tour \"{}\": {planned} frames at {width}×{height} ss{}, {fps} fps ({:.1}s)…",
            pb.name, self.export.ss, pb.total
        ));
        let meta = std::sync::Arc::new(self.view_metadata());
        // Encode PNGs on a small worker pool so compression overlaps the next frame's GPU render
        // (rendering a frame is CPU-bignum + GPU; PNG deflate is pure CPU — they run concurrently).
        // A bounded channel caps how many big frame buffers sit in RAM at once (~1 GB budget), so the
        // main thread blocks (backpressure) rather than the process ballooning on a large deep tour.
        let frame_bytes = (width as u64) * (height as u64) * 16;
        let inflight = (1_000_000_000u64 / frame_bytes.max(1)).clamp(2, 8) as usize;
        let workers = inflight.clamp(1, 3);
        let (enc_tx, enc_rx) = std::sync::mpsc::sync_channel::<EncodeJob>(inflight.saturating_sub(workers).max(1));
        let enc_rx = std::sync::Arc::new(std::sync::Mutex::new(enc_rx));
        let enc_err: std::sync::Arc<std::sync::Mutex<Option<String>>> = std::sync::Arc::new(std::sync::Mutex::new(None));
        let encoders: Vec<_> = (0..workers)
            .map(|_| {
                let rx = enc_rx.clone();
                let meta = meta.clone();
                let err = enc_err.clone();
                std::thread::spawn(move || loop {
                    // Hold the lock only across recv() (fast dequeue); encode without it so workers
                    // compress in parallel.
                    // Poison-recover: if another encoder panicked while holding the lock, keep
                    // draining rather than cascading the panic across every worker.
                    let job = { rx.lock().unwrap_or_else(|e| e.into_inner()).recv() };
                    let job = match job {
                        Ok(j) => j,
                        Err(_) => break, // channel closed → all frames handed off
                    };
                    if let Err(e) = fractadyne_export::write_png(&job.path, job.w, job.h, &job.px, Some(&meta)) {
                        let mut slot = err.lock().unwrap_or_else(|e| e.into_inner());
                        if slot.is_none() {
                            *slot = Some(format!("frame {}: {e}", job.fi));
                        }
                    }
                })
            })
            .collect();
        // Reference pipeline: frame N+1's bignum reference (orbit + SA + BLA) is computed on a worker
        // while frame N renders on the GPU, so the deep-zoom reference stall overlaps the render.
        let mut pending_ref: Option<(u64, std::sync::mpsc::Receiver<crate::render::RecomputeResult>)> = None;
        // Overwrite policy: `overwrite_all` skips the per-frame prompt; `canceled` breaks the render.
        let mut overwrite_all = overwrite;
        let mut canceled = false;
        // Tour normalize: the escape-value range smoothed across frames (see
        // `render_export_normalized`), and a one-shot flag so an oversized frame warns once rather
        // than per-frame. `resume` may start mid-tour with no prior range — the EMA simply seeds
        // from the first rendered frame, which is what `None` does.
        let mut norm_range: Option<(f32, f32)> = None;
        let mut norm_oversize_warned = false;
        let want_normalize = pb.render.normalize;
        // Per-tile nominal-work cap for tour frames — conservatively below the interactive export's
        // 2e10 so that even a shallow, all-interior frame (BLA skips nothing there, so nominal work
        // ≈ real GPU steps) keeps each dispatch well under the ~2 s OS watchdog. It over-splits a
        // deep frame (nominal ≫ real), but many short tiles are safe; one long dispatch is not.
        const TOUR_WORK_BUDGET: u64 = 2_000_000_000;
        // Headroom to leave free beyond the references — in-flight encode buffers (~1 GB), GPU
        // staging, general slack. The reference lookahead is skipped when the next one would eat in.
        const TOUR_MEM_MARGIN: u64 = 1_500_000_000;
        // Conservative peak-build footprint of one reference: the CPU bignum orbit (length ≈
        // max_iter, ~prec bits/sample + Vec/enum overhead) plus its BLA table. Deliberately high —
        // over-estimating only costs a little pipelining, under-estimating risks the OOM this fixes.
        fn est_ref_bytes(max_iter: u32, prec: usize) -> u64 {
            let per_sample = (prec as u64 / 8) * 4 + 128;
            (max_iter as u64).saturating_mul(per_sample)
        }
        let mut low_mem_warned = false;
        let started = std::time::Instant::now();
        for fi in first_frame..=last_frame {
            let t = if pb.total <= 0.0 { 0.0 } else { (fi as f64 / fps).min(pb.total) };
            let s = pb.sample(t);
            self.fractal = s.fractal;
            self.julia_mode = s.julia && s.fractal.supports_julia();
            self.dual = s.dual;
            if let Some(c) = s.julia_c {
                self.julia_c = c;
            }
            // Per-keyframe iteration budget + palette; fall back to the script-wide base. This is
            // what lets one script hold a 1.33× home view at a few thousand iterations and a 1e94×
            // spar at millions — a single budget either starves the deep frames or makes the
            // shallow ones cost minutes each.
            self.render_cfg.max_iter = base_iter;
            self.render_cfg.auto_iter = base_auto;
            self.apply_sampled_settings(&s);
            // Output path for this frame; ask before clobbering an existing one (unless overwriting).
            let frame_path = out_dir.join(format!("{prefix}_{fi:05}.png"));
            // Resume: a frame that's already on disk is assumed complete — skip it silently and
            // render only what's missing (no prompt, no re-render). This is what makes an
            // interrupted render restartable.
            if resume && frame_path.exists() {
                continue;
            }
            if !overwrite_all && frame_path.exists() {
                match prompt_overwrite(&frame_path)? {
                    OverwriteChoice::Yes => {}
                    OverwriteChoice::YesAll => overwrite_all = true,
                    OverwriteChoice::No => continue,
                    OverwriteChoice::Quit => {
                        canceled = true;
                        break;
                    }
                }
            }
            // Claim frame `fi`'s precomputed reference if the previous iteration started one for it.
            let mut this_ref = match pending_ref.take() {
                Some((idx, rx)) if idx == fi => rx.recv().ok(),
                _ => None,
            };
            // Kick off frame `fi+1`'s reference now (overlaps this frame's render + encode). Only when
            // both this frame and its successor are single-view with the same fractal/Julia state, so
            // `self`'s current fractal/julia_c (set above) validly describe the successor's reference.
            pending_ref = None;
            if !s.dual && fi + 1 < frames {
                let t2 = if pb.total <= 0.0 { 0.0 } else { ((fi + 1) as f64 / fps).min(pb.total) };
                let s2 = pb.sample(t2);
                if !s2.dual && s2.fractal == s.fractal && s2.julia == s.julia && s2.julia_c == s.julia_c {
                    let mut vp2 = fractadyne_core::Viewport::new(width as f64, height as f64);
                    vp2.set_center_log2mag(s2.cx, s2.cy, s2.logmag / std::f64::consts::LN_2);
                    // Only prefetch the NEXT reference if it will fit alongside the one still
                    // resident for THIS frame — otherwise two big bignum references at once OOM the
                    // process (measured: an 8M-sample pair ~2.3 GB each killed a 32 GB render at
                    // frame 221/233). When memory is tight, skip the lookahead and let frame N+1
                    // build synchronously (one reference resident at a time) — slower, never fatal.
                    let est = est_ref_bytes(self.render_cfg.max_iter, vp2.precision);
                    let room = crate::sysinfo::available_memory()
                        .map_or(true, |avail| est.saturating_add(TOUR_MEM_MARGIN) < avail);
                    if room {
                        if let Some(rx) = self.spawn_export_reference(&vp2, self.julia_mode) {
                            pending_ref = Some((fi + 1, rx));
                        }
                    } else if !low_mem_warned {
                        let avail = crate::sysinfo::available_memory().unwrap_or(0);
                        say(&format!(
                            "⚠ low memory (~{:.1} GB free, next reference ~{:.1} GB) — building \
                             references synchronously to avoid an out-of-memory abort",
                            avail as f64 / 1e9,
                            est as f64 / 1e9,
                        ));
                        low_mem_warned = true;
                    }
                }
            }
            crate::diag::breadcrumb(format!("tour frame {}/{frames}", fi + 1));
            let progress = std::sync::atomic::AtomicU32::new(0);
            let cancel = std::sync::atomic::AtomicBool::new(false);
            let render = |app: &Self, vp: &fractadyne_core::Viewport, julia: bool, w: u32, vg| -> Result<fractadyne_gpu::ExportResult, String> {
                let mut req = app.current_export_request_for(vp, julia);
                req.width = w;
                req.height = height;
                req.ss = app.export.ss;
                req.max_iter = req.max_iter.max(200);
                req.vignette = vg;
                req.work_budget = Some(TOUR_WORK_BUDGET); // bound each tile's dispatch (TDR-safe)
                fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
                    .map_err(|e| format!("frame {fi}: {e}"))
            };
            let (mut px, rw, rh) = if s.dual {
                // Side-by-side: Mandelbrot (left) | its Julia set (right), each a half-width panel.
                let half = (width / 2).max(1);
                self.viewport = fractadyne_core::Viewport::new(half as f64, height as f64);
                self.viewport.set_center_log2mag(s.cx, s.cy, s.logmag / std::f64::consts::LN_2);
                let mut jvp = fractadyne_core::Viewport::new(half as f64, height as f64);
                jvp.center_x = fractadyne_core::BigFloat::from_f64(0.0, 64);
                jvp.center_y = fractadyne_core::BigFloat::from_f64(0.0, 64);
                jvp.units_per_pixel = fractadyne_core::FloatExp::from_f64(3.2 / height as f64);
                jvp.precision = 64;
                let mr = render(self, &self.viewport, false, half, fractadyne_gpu::Vignette::default())?;
                let jr = render(self, &jvp, true, half, fractadyne_gpu::Vignette::default())?;
                let (w, h, p) = crate::stitch_side_by_side(&mr, &jr);
                (p, w, h)
            } else {
                self.viewport = fractadyne_core::Viewport::new(width as f64, height as f64);
                self.viewport.set_center_log2mag(s.cx, s.cy, s.logmag / std::f64::consts::LN_2);
                let vg = vignette_for(&pb.spotlights, &self.viewport, t);
                let ss = self.export.ss;
                // Only attempt the normalized path when the supersampled buffer fits its
                // single-texture color cap — otherwise warn ONCE and render normally, so a big
                // normalized render is never silently un-normalized (nor does it lose the
                // pipelined reference to a `take()` that the fallback then can't use).
                let over_cap = (width * ss) as u64 * (height * ss) as u64 > 40_000_000;
                if want_normalize && over_cap && !norm_oversize_warned {
                    say(&format!(
                        "⚠ normalize: {width}×{height} ss{ss} exceeds the ~40 Mpx normalized color \
                         cap — rendering this tour UN-normalized (deep frames may alias). Lower ss \
                         or size, or wait for the tiled normalized color pass."
                    ));
                    norm_oversize_warned = true;
                }
                // The normalized path returns `None` for an all-interior frame or aux coloring; in
                // that case fall through to the normal render (with the reference it handed back).
                let normalized = if want_normalize && !over_cap {
                    self.render_export_normalized(
                        device, queue, &self.viewport, self.julia_mode, width, height, ss,
                        norm_range, this_ref.take(), TOUR_WORK_BUDGET,
                    )
                } else {
                    None
                };
                match normalized {
                    Some((r, range)) => {
                        norm_range = Some(range);
                        (r.pixels, r.width, r.height)
                    }
                    None => {
                        // Single view: pipelined precomputed reference (falls back to sync internally).
                        let mut req = self
                            .current_export_request_with_ref(&self.viewport, self.julia_mode, this_ref);
                        req.width = width;
                        req.height = height;
                        req.ss = self.export.ss;
                        req.max_iter = req.max_iter.max(200);
                        req.vignette = vg;
                        req.work_budget = Some(TOUR_WORK_BUDGET); // bound each tile's dispatch (TDR-safe)
                        let r = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
                            .map_err(|e| format!("frame {fi}: {e}"))?;
                        (r.pixels, r.width, r.height)
                    }
                }
            };
            self.apply_watermark(&mut px, rw, rh);
            let mut placed_labels: Vec<egui::Rect> = Vec::new();
            for cap in &pb.captions {
                let a = cap.alpha_at(t);
                if a > 0.0 {
                    if let Some(r) = stamp_caption(ctx, &mut px, rw, rh, cap, a) {
                        placed_labels.push(r); // callouts avoid caption text too
                    }
                }
            }
            for co in &pb.callouts {
                let a = co.alpha_at(t);
                if a <= 0.0 {
                    continue;
                }
                let (vpx, vpy) = self.viewport.complex_to_pixel(&co.cx, &co.cy);
                if vpx >= 0.0 && vpy >= 0.0 && vpx < rw as f64 && vpy < rh as f64 {
                    stamp_callout(ctx, &mut px, rw, rh, co, vpx as f32, vpy as f32, a, &mut placed_labels);
                }
            }
            // Orbit overlay (single view only; the dual path already split the frame).
            if s.orbits && !s.dual {
                if let Some(pt) = s.orbit {
                    stamp_orbit(&mut px, rw, rh, &self.viewport, pt, self.julia_mode, self.julia_c);
                }
            }
            // Location HUD (single view only — meaningless on the split dual frame).
            if show_location && !s.dual {
                stamp_location(ctx, &mut px, rw, rh, &self.viewport);
            }
            // Surface any earlier encode failure before enqueuing more work.
            if let Some(e) = enc_err.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                return Err(e.clone());
            }
            // Hand the finished frame to the encoder pool; blocks if the queue is full (backpressure).
            enc_tx
                .send(EncodeJob { path: frame_path, w: rw, h: rh, px, fi })
                .map_err(|_| format!("frame {fi}: encoder thread stopped"))?;
            if fi % 10 == 0 || fi == last_frame {
                let done = fi + 1 - first_frame;
                let elapsed = started.elapsed().as_secs_f64();
                let rate = if elapsed > 0.0 { done as f64 / elapsed } else { 0.0 };
                let eta = if rate > 0.0 { (planned - done) as f64 / rate } else { 0.0 };
                say(&format!(
                    "  frame {done}/{planned}  ({} elapsed, {} left, {rate:.2} fps)",
                    fmt_hms(elapsed),
                    fmt_hms(eta)
                ));
            }
        }
        // Close the queue and wait for the encoder pool to drain the remaining frames.
        drop(enc_tx);
        for h in encoders {
            let _ = h.join();
        }
        if let Some(e) = enc_err.lock().unwrap_or_else(|e| e.into_inner()).take() {
            return Err(e);
        }
        let render_secs = started.elapsed().as_secs_f64();
        if canceled {
            return Ok(format!(
                "Canceled after {} → {} (existing frames left in place).",
                fmt_hms(render_secs),
                out_dir.display()
            ));
        }
        say(&format!(
            "Rendered {planned} frame(s) in {} → {}",
            fmt_hms(render_secs),
            out_dir.display()
        ));

        // Optionally assemble the PNG sequence into an mp4 via ffmpeg (kept separate from the
        // frames so a failed/absent ffmpeg never loses the render).
        let pattern = out_dir.join(format!("{prefix}_%05d.png"));
        if let Some(mp4_path) = mp4 {
            say(&format!("Encoding → {} (ffmpeg)…", mp4_path.display()));
            let enc = std::time::Instant::now();
            // `-vf pad…` rounds the frame up to even dimensions (yuv420p/H.264 requires it) without
            // resampling; `-crf 18` is visually near-lossless.
            let status = std::process::Command::new("ffmpeg")
                .arg("-y")
                .arg("-hide_banner")
                .args(["-framerate", &format!("{fps}")])
                .arg("-i")
                .arg(&pattern)
                .args(["-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2"])
                .args(["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18"])
                .arg(&mp4_path)
                .status();
            match status {
                Ok(s) if s.success() => {
                    return Ok(format!(
                        "Encoded {} in {}.",
                        mp4_path.display(),
                        fmt_hms(enc.elapsed().as_secs_f64())
                    ));
                }
                Ok(s) => {
                    return Ok(format!(
                        "ffmpeg exited with {s}; frames are intact. Assemble manually:\n  \
                         ffmpeg -framerate {fps} -i {} -c:v libx264 -pix_fmt yuv420p {}",
                        pattern.display(),
                        mp4_path.display()
                    ));
                }
                Err(e) => {
                    return Ok(format!(
                        "Could not run ffmpeg ({e}); is it on your PATH? Frames are intact. Assemble:\n  \
                         ffmpeg -framerate {fps} -i {} -c:v libx264 -pix_fmt yuv420p {}",
                        pattern.display(),
                        mp4_path.display()
                    ));
                }
            }
        }
        Ok(format!(
            "Assemble into a video:\n  ffmpeg -framerate {fps} -i {} -c:v libx264 -pix_fmt yuv420p \
             tour.mp4\n(or re-run with --mp4 to do this automatically)",
            pattern.display()
        ))
        })();
        // One writer, every exit path: the marker can never be stale-by-omission.
        match &render_result {
            Ok(m) if m.starts_with("Canceled") => write_render_status(&out_dir, "canceled"),
            Ok(_) => write_render_status(&out_dir, "complete"),
            Err(e) => write_render_status(&out_dir, &format!("failed: {e}")),
        }
        render_result
    }

    /// Advance the active camera tour by one frame; drives the view and, for a
    /// benchmark, samples performance. Returns true while still playing.
    pub(crate) fn advance_playback(&mut self, ctx: &egui::Context) -> bool {
        let now = ctx.input(|i| i.time);
        match self.advance_playback_core(now) {
            PlaybackTick::Idle => false,
            PlaybackTick::Playing => true,
            PlaybackTick::Finished(Some(name)) => {
                self.set_toast(format!("Script finished — \"{name}\""), ctx);
                false
            }
            PlaybackTick::Finished(None) => false, // benchmark → report dialog already queued
            PlaybackTick::Ended => false, // player still up, clock stopped — no repaint
        }
    }

    /// Ctx-free core of [`advance_playback`] — also driven headless (real wall-clock `now`) by the
    /// `--divetest` harness, so the harness exercises the IDENTICAL playback machinery (pipeline
    /// pacer, camera sampling, reference lookahead, perf capture) the GUI runs.
    pub(crate) fn advance_playback_core(&mut self, now: f64) -> PlaybackTick {
        let Some(mut pb) = self.playback.take() else {
            return PlaybackTick::Idle;
        };
        // Fresh tour → no leftover lookahead state from a previous run may install into it.
        if !pb.started {
            pb.started = true;
            self.ref_prefetch.clear();
        }
        // Pipeline-paced clock: at extreme depth the async reference rebuild can fall behind a fast
        // dive (a fresh `best_reference` costs seconds past ~1e400×) — the screen then reprojects an
        // ever-staler frame, which magnifies into a monocolor blur. `last_depth_lag` (octaves the
        // view has zoomed past the cached BLA's validity, from `build_params`) is exactly that
        // "pipeline is behind" signal, so DILATE the tour clock by it: lag ≤ LO plays real-time,
        // ≥ HI fully holds (just under the mode-2 stale-reference spin/freeze threshold ≈ 3), and
        // in between the dive proportionally slows. The image stays sharp; the dive takes longer.
        // Shallow tours (and the built-in benchmark, ≤ 1e12×) sit at lag ≈ 0–1.1 → untouched.
        {
            let dt = pb.last_now.map_or(0.0, |t| (now - t).max(0.0));
            pb.last_now = Some(now);
            let lag = self.ref_cache.iter().map(|c| c.last_depth_lag).fold(0.0, f64::max);
            let mut hold = if pb.pace == Pace::Realtime {
                0.0 // the wall clock IS the measurement — never dilate
            } else {
                ((lag - crate::PACE_LAG_LO) / (crate::PACE_LAG_HI - crate::PACE_LAG_LO)).clamp(0.0, 1.0)
            };
            // `Settled` pacing: at a hold, stop the clock outright until the view has resolved.
            // The lag-based dilation above only reacts to the REFERENCE pipeline falling behind;
            // it says nothing about whether the picture is finished. At depth the adaptive
            // iteration budget needs several settled frames to climb to what the field needs —
            // far more than a 3-second hold of wall clock — so without this the tour walks past
            // its own destination while the screen is still starved (black).
            if pb.pace == Pace::Settled && dt > 0.0 {
                let (holding, kf) = pb.holding_at(pb.cur_t);
                if pb.settle_kf != kf {
                    pb.settle_kf = kf;
                    pb.settle_waited = 0.0;
                }
                // A view that has stopped changing entirely — settled, nothing in flight, no new
                // counter reading — is finished, and "no reading" is how a settled view that needs
                // no re-iterate looks. But that is also how a view looks in the instant after the
                // camera arrives, before the work it needs has even been requested, so only
                // conclude it after a grace period. Without this, an ordinary shallow view (which
                // never re-iterates once settled) waits out the entire timeout: measured, the
                // 1.33× home view sat for the full 90 s while already matching an offline render.
                const SETTLE_GRACE: f64 = 1.5;
                let idle = pb.settle_waited > SETTLE_GRACE && self.view_idle(0);
                if holding
                    && !idle
                    && !self.view_resolved(0)
                    && pb.settle_waited < pb.settle_timeout
                {
                    pb.settle_waited += dt;
                    hold = 1.0;
                }
            }
            // FINAL BACKSTOP, over every reason the clock can be held. `settle_timeout` bounds the
            // settled-hold branch above, but the lag dilation had no bound at all: at a view whose
            // reference the freeze guard refuses, no new reference ever installs, so the BLA never
            // refreshes, `last_depth_lag` never falls, and `hold` stays 1.0 for the rest of time —
            // measured in the field as the grand tour stopping dead at 3:35 with the renderer still
            // busy. Whatever the pipeline is waiting for, a tour that has been fully stopped for
            // `settle_timeout` gives up waiting and moves on: a blurry frame that keeps playing is
            // strictly better than a tour that never reaches its end. Releasing only clears the
            // dilation — the renderer keeps working, so the view still sharpens if it can.
            // The release is STICKY until the pipeline genuinely recovers. Releasing for a single
            // frame and then re-arming the timer would just make the stall a duty cycle — 15 s
            // stopped, one frame of progress, 15 s stopped — which is a hang with extra steps.
            if hold > 0.5 {
                if !pb.pace_released {
                    pb.pace_waited += dt;
                    pb.pace_released = pb.pace_waited > pb.settle_timeout;
                }
            } else {
                pb.pace_waited = 0.0;
                pb.pace_released = false; // caught up — normal pacing resumes
            }
            if pb.pace_released {
                hold = 0.0;
            }
            pb.paced_hold = hold;
            // Advance the clock. `1 - hold` is the pacer's dilation (0 = fully stopped while the
            // renderer catches up), `speed` and `paused` are the user's transport. At speed 1 with
            // no hold this is exactly the old wall-clock behaviour, so tour durations and the
            // `--divetest` timings are unchanged.
            let live = if pb.paused || pb.finished { 0.0 } else { dt * pb.speed };
            let step = live * (1.0 - hold);
            pb.cur_t += step;
            // The wall-clock ghost ignores the pacer's `hold`, so it runs ahead exactly by the
            // dilation. Clamp to the tour end — a ghost past the finish just pins at the end tick.
            pb.wall_t = (pb.wall_t + live).min(pb.total);
            if pb.loop_ && pb.total > 0.0 && pb.cur_t >= pb.total {
                pb.cur_t = 0.0;
                pb.wall_t = 0.0;
            }
        }
        // Only the FIRST tick past the end ends the tour. Afterwards the player stays up with the
        // clock parked, so this must not re-fire the toast (or re-queue a benchmark report) on
        // every frame.
        let just_finished = !pb.loop_ && !pb.finished && pb.cur_t >= pb.total;
        let e = pb.cur_t.clamp(0.0, pb.total);
        pb.cur_t = e;
        let s = pb.sample(e);
        if s.fractal != self.fractal || s.julia != self.julia_mode {
            self.fractal = s.fractal;
            self.julia_mode = s.julia && s.fractal.supports_julia();
            self.invalidate_refs();
        }
        // Discrete overlays (dual view, Julia pin, orbits).
        if self.dual != s.dual {
            self.dual = s.dual;
            self.invalidate_refs();
        }
        if let Some(c) = s.julia_c {
            if self.julia_c != c {
                self.julia_c = c;
                self.ref_cache[1].ref_pt = None; // Julia parameter changed
            }
            self.julia_pin = Some(c); // hold it (don't let cursor hover override)
        }
        self.anim.show_orbits = s.orbits;
        self.anim.tour_orbit = s.orbit;
        // Minimap is script-driven during a tour (a navigation aid while the script pans between
        // locations); the viewer's own toggle is in `PlaybackRestore`, so this never leaks past
        // the tour's end. Offline renders never reach this path.
        self.dialogs.minimap = s.minimap;
        // Per-keyframe iteration budget + palette (restored when the tour ends).
        self.apply_sampled_settings(&s);
        // log2 path so playback stays exact past f64's 1e308× ceiling.
        self.viewport.set_center_log2mag(s.cx, s.cy, s.logmag / std::f64::consts::LN_2);
        // Only a MOVING camera counts as interaction. Stamping every playback tick kept the view
        // permanently "interacting", which is right for a glide (cheap moving path, reprojection)
        // but wrong for a HOLD: the adaptive iteration budget only measures and adapts on SETTLED
        // frames, so a tour could never raise its budget and starved deep views stayed black for
        // the whole hold — while the same view resolves the moment a human stops touching it.
        if !pb.holding_at(e).0 {
            self.pointer.settle_t = [now; 2];
        }
        // Reference LOOKAHEAD: the script knows where the camera is going — build the reference the
        // dive is about to need on idle cores, and install it as the dive arrives (render.rs).
        self.playback_ref_prefetch(&pb, e);
        // FRACTADYNE_PERF=1: one JSONL record per playback frame — tour time, depth, and the
        // PREVIOUS frame's cost (what `Perf` maintains). Offline analysis of the frame-time
        // distribution/periodicity pins live judder mechanisms (e.g. "every ~150 ms a 2-3 vsync
        // spike" = the real-refresh hitch) that aggregate stats hide.
        if crate::diag::perf_on() {
            crate::diag::perf_jsonl(&format!(
                "\"kind\":\"live\",\"e\":{e:.3},\"l2\":{:.1},\"frame_ms\":{:.2},\"cpu_ms\":{:.2},\"lag\":{:.2}",
                s.logmag / std::f64::consts::LN_2,
                self.perf.frame_ms,
                self.perf.cpu_ms,
                self.ref_cache[0].last_depth_lag,
            ));
        }

        // Benchmark sampling (skip warm-up frames).
        if let Some(b) = pb.bench.as_mut() {
            if b.warmup_left > 0 {
                b.warmup_left -= 1;
            } else if self.perf.frame_ms > 0.0 {
                b.frames += 1;
                b.sum_frame_ms += self.perf.frame_ms;
                b.sum_cpu_ms += self.perf.cpu_ms;
                let fps = 1000.0 / self.perf.frame_ms;
                b.min_fps = b.min_fps.min(fps);
                b.max_fps = b.max_fps.max(fps);
                let (ws, peak) = process_memory();
                b.peak_ram = b.peak_ram.max(peak).max(ws);
                b.sum_ram += ws;
            }
        }

        if just_finished {
            if let Some(b) = pb.bench.take() {
                // A benchmark run OWNS the session while it plays and reports through a dialog;
                // there is nothing to scrub back into, so it still tears down. `pb` is already out
                // of `self.playback`, so this only hands the viewer's settings back.
                self.stop_playback();
                self.bench_report = Some(self.format_bench(&pb, &b));
                self.dialogs.bench_open = true;
                return PlaybackTick::Finished(None); // report dialog carries the outcome
            }
            // A script parks at its final keyframe with the player still up (see `Playback::
            // finished`). The viewer's own iteration budget and coloring are handed back when they
            // close it, not here — restoring them now would recolor the frame they are looking at.
            pb.finished = true;
            pb.paused = true;
            let name = pb.name.clone();
            self.playback = Some(pb);
            return PlaybackTick::Finished(Some(name));
        }
        let ended = pb.finished;
        self.playback = Some(pb);
        if ended {
            PlaybackTick::Ended
        } else {
            PlaybackTick::Playing
        }
    }

    /// Build a human-readable benchmark report.
    pub(crate) fn format_bench(&self, pb: &Playback, b: &Bench) -> String {
        let f = b.frames.max(1) as f64;
        let avg_frame = b.sum_frame_ms / f;
        let avg_fps = if avg_frame > 0.0 { 1000.0 / avg_frame } else { 0.0 };
        let avg_cpu = b.sum_cpu_ms / f;
        let avg_gpu = (b.sum_frame_ms - b.sum_cpu_ms).max(0.0) / f;
        let avg_ram = b.sum_ram / b.frames.max(1);
        let mb = |bytes: u64| bytes as f64 / (1024.0 * 1024.0);
        let si = &self.sysinfo;
        let cache = if si.l3_kb > 0 {
            format!("L2 {} KB · L3 {} MB", si.l2_kb, si.l3_kb / 1024)
        } else if si.l2_kb > 0 {
            format!("L2 {} KB", si.l2_kb)
        } else {
            "—".to_string()
        };
        let vram = if si.vram_mb > 0 {
            format!("{} MB", si.vram_mb)
        } else {
            "—".to_string()
        };
        format!(
            "Fractadyne benchmark — {tour}\n\
             version    v{ver}\n\
             date       {date}\n\
             cpu        {cpu}\n\
             cores      {phys} physical / {logi} logical\n\
             cache      {cache}\n\
             gpu        {gpu}\n\
             vram       {vram}\n\
             frames     {frames}  over {dur:.0}s\n\
             ----------------------------------------\n\
             avg FPS    {afps:8.1}\n\
             min FPS    {minf:8.1}\n\
             max FPS    {maxf:8.1}\n\
             avg frame  {aframe:8.2} ms\n\
             avg CPU    {acpu:8.2} ms\n\
             avg GPU    {agpu:8.2} ms   (frame − cpu)\n\
             avg RAM    {aram:8.1} MB\n\
             peak RAM   {pram:8.1} MB\n\
             ----------------------------------------\n\
             score      {score:8.0}   (avg FPS × 100)",
            tour = pb.name,
            ver = version_string(),
            date = now_utc_string(),
            cpu = if si.cpu.is_empty() { "—" } else { &si.cpu },
            phys = si.physical,
            logi = si.logical,
            cache = cache,
            gpu = self.gpu_name,
            vram = vram,
            frames = b.frames,
            dur = pb.total,
            afps = avg_fps,
            minf = if b.min_fps.is_finite() { b.min_fps } else { 0.0 },
            maxf = b.max_fps,
            aframe = avg_frame,
            acpu = avg_cpu,
            agpu = avg_gpu,
            aram = mb(avg_ram),
            pram = mb(b.peak_ram),
            score = avg_fps * 100.0,
        )
    }
}

// ============================================================================
// Standardized benchmark — pins every render setting so a score is comparable
// across machines, regardless of window size or the user's current settings.
// ============================================================================

/// Fixed output resolutions offered for the standardized benchmark. These render
/// **offscreen** (via the export/tiling path), so 4K/5K work even on a smaller monitor.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BenchRes {
    P720,
    P1080,
    P4K,
    W5K2K,
}

impl BenchRes {
    pub(crate) const ALL: [BenchRes; 4] =
        [BenchRes::P720, BenchRes::P1080, BenchRes::P4K, BenchRes::W5K2K];

    pub(crate) fn dims(self) -> (u32, u32) {
        match self {
            BenchRes::P720 => (1280, 720),
            BenchRes::P1080 => (1920, 1080),
            BenchRes::P4K => (3840, 2160),
            BenchRes::W5K2K => (5120, 2160),
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            BenchRes::P720 => "720p (1280×720)",
            BenchRes::P1080 => "1080p (1920×1080)",
            BenchRes::P4K => "4K (3840×2160)",
            BenchRes::W5K2K => "5K×2K (5120×2160)",
        }
    }

    /// Parse a CLI token (`720p` / `1080p` / `4k` / `5k` …, case-insensitive).
    pub(crate) fn from_token(s: &str) -> Option<BenchRes> {
        match s.trim().to_ascii_lowercase().as_str() {
            "720" | "720p" | "hd" => Some(BenchRes::P720),
            "1080" | "1080p" | "fhd" => Some(BenchRes::P1080),
            "4k" | "2160" | "2160p" | "uhd" => Some(BenchRes::P4K),
            "5k" | "5kx2k" | "5k2k" => Some(BenchRes::W5K2K),
            _ => None,
        }
    }
}

/// Canonical settings the standardized benchmark pins (so the score means the same on
/// every machine). Recorded verbatim in the report.
pub(crate) const STD_AA: u32 = 2; // 2×2 supersampling
pub(crate) const STD_FRAMES: u32 = 60; // frames rendered along the fixed dive
pub(crate) const STD_ZOOM_LOG10: f64 = 12.0; // standard dive depth: 1 → 1e12×
/// Ultra-deep dive: 1 → 1e28×. Well past f64's ~1e15 magnification limit, so it hammers the
/// perturbation / series-approx / BLA machinery (iteration counts climb steeply with depth).
/// Kept ≤ the ~33-significant-digit `STD_CX`/`STD_CY` precision (sub-pixel to ~1e30×), so the
/// dive lands on a fixed, reproducible high-detail location rather than precision noise.
pub(crate) const STD_ZOOM_LOG10_ULTRA: f64 = 28.0;
/// Seahorse-Valley point with structure at every scale (same as the built-in tour).
const STD_CX: &str = "-0.743643887037158704752191506114774";
const STD_CY: &str = "0.131825904205311970493132056385139";

/// Dive depth for the standardized benchmark. Deeper endpoints exercise the deep-zoom path
/// (perturbation reference, series skip, BLA) far harder than the shallow default.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BenchDepth {
    Standard,
    Ultra,
}

impl BenchDepth {
    pub(crate) const ALL: [BenchDepth; 2] = [BenchDepth::Standard, BenchDepth::Ultra];

    /// log10 of the final magnification the fixed 60-frame dive reaches.
    pub(crate) fn zoom_log10(self) -> f64 {
        match self {
            BenchDepth::Standard => STD_ZOOM_LOG10,
            BenchDepth::Ultra => STD_ZOOM_LOG10_ULTRA,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            BenchDepth::Standard => "Standard (1e12×)",
            BenchDepth::Ultra => "Ultra deep (1e28×)",
        }
    }

    /// Parse a CLI token (`standard` / `ultra` / `deep` …, case-insensitive).
    pub(crate) fn from_token(s: &str) -> Option<BenchDepth> {
        match s.trim().to_ascii_lowercase().as_str() {
            "standard" | "std" | "shallow" | "12" | "1e12" => Some(BenchDepth::Standard),
            "ultra" | "deep" | "ultradeep" | "ultra-deep" | "28" | "1e28" => Some(BenchDepth::Ultra),
            _ => None,
        }
    }
}

/// The subset of app render settings the standardized benchmark overrides, saved so the
/// live view is restored untouched afterward.
pub(crate) struct BenchSnapshot {
    fractal: FractalKind,
    julia_mode: bool,
    dual: bool,
    color_method: u32,
    auto_iter: bool,
    aa: u32,
    series_approx: bool,
    use_bla: bool,
    glitch_correct: bool,
    palette_idx: usize,
    use_binary: bool,
    use_duotone: bool,
    use_custom_palette: bool,
}

/// A running standardized benchmark (one or more passes; >1 = burn-in). Driven one dive-frame at
/// a time from `update()` so the window stays responsive (spinner animates, cancellable) rather
/// than blocking the whole event loop for a multi-second pass.
pub(crate) struct StdBench {
    pub(crate) res: BenchRes,
    pub(crate) passes_total: u32,
    pub(crate) passes_done: u32,
    /// Average FPS of each completed pass (burn-in stability / thermal trend).
    pub(crate) pass_fps: Vec<f64>,
    /// Per-frame stats of the most recent completed pass (for the detailed report).
    last: Option<Bench>,
    snapshot: BenchSnapshot,
    /// Fixed dive center (parsed once).
    cx: fractadyne_core::BigFloat,
    cy: fractadyne_core::BigFloat,
    /// Frame cursor within the current pass: `-1` = warm-up (uncounted), `0..STD_FRAMES` = dive.
    frame_in_pass: i32,
    /// Accumulator for the in-progress pass (`None` between passes → next step starts a fresh one).
    cur: Option<Bench>,
    /// log10 of the dive's final magnification (depth preset — 1e12× standard, 1e28× ultra).
    zoom_log10: f64,
}

impl StdBench {
    /// Consume the run, yielding the saved settings so the caller can restore the live view.
    pub(crate) fn take_snapshot(self) -> BenchSnapshot {
        self.snapshot
    }

    /// Progress within the current pass as `(dive_frames_done, total)` for the UI (warm-up → 0).
    pub(crate) fn frame_progress(&self) -> (u32, u32) {
        (self.frame_in_pass.max(0) as u32, STD_FRAMES.max(2))
    }
}

impl FractadyneApp {
    /// Begin a standardized benchmark (`passes` ≥ 2 ⇒ burn-in). Snapshots the live settings,
    /// pins the canonical ones, and returns the run state to drive pass-by-pass.
    pub(crate) fn begin_standard_bench(
        &mut self,
        res: BenchRes,
        passes: u32,
        depth: BenchDepth,
    ) -> StdBench {
        let snapshot = BenchSnapshot {
            fractal: self.fractal,
            julia_mode: self.julia_mode,
            dual: self.dual,
            color_method: self.coloring.color_method.to_u32(),
            auto_iter: self.render_cfg.auto_iter,
            aa: self.render_cfg.aa,
            series_approx: self.render_cfg.series_approx,
            use_bla: self.render_cfg.use_bla,
            glitch_correct: self.render_cfg.glitch_correct,
            palette_idx: self.coloring.palette_idx,
            use_binary: self.coloring.use_binary,
            use_duotone: self.coloring.use_duotone,
            use_custom_palette: self.coloring.use_custom_palette,
        };
        // Pin the canonical configuration.
        self.set_fractal(FractalKind::Mandelbrot);
        self.julia_mode = false;
        self.dual = false;
        self.coloring.color_method = crate::ColorMethod::Smooth; // smooth
        self.render_cfg.auto_iter = true; // depth-appropriate, deterministic per depth
        self.render_cfg.aa = STD_AA;
        self.render_cfg.series_approx = true;
        self.render_cfg.use_bla = true;
        self.render_cfg.glitch_correct = false; // data-dependent cost → off for a deterministic timing
        self.coloring.palette_idx = 0; // Ember
        self.coloring.use_binary = false;
        self.coloring.use_duotone = false;
        self.coloring.use_custom_palette = false;
        self.invalidate_refs();
        let cx = fractadyne_core::parse_bf(STD_CX)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(-0.5, 64));
        let cy = fractadyne_core::parse_bf(STD_CY)
            .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.0, 64));
        StdBench {
            res,
            passes_total: passes.clamp(1, 500),
            passes_done: 0,
            pass_fps: Vec::new(),
            last: None,
            snapshot,
            cx,
            cy,
            frame_in_pass: -1,
            cur: None,
            zoom_log10: depth.zoom_log10(),
        }
    }

    /// Restore the live settings a standardized benchmark overrode.
    pub(crate) fn restore_from_bench(&mut self, s: BenchSnapshot) {
        self.set_fractal(s.fractal);
        self.julia_mode = s.julia_mode;
        self.dual = s.dual;
        self.coloring.color_method = crate::ColorMethod::from_u32(s.color_method);
        self.render_cfg.auto_iter = s.auto_iter;
        self.render_cfg.aa = s.aa;
        self.render_cfg.series_approx = s.series_approx;
        self.render_cfg.use_bla = s.use_bla;
        self.render_cfg.glitch_correct = s.glitch_correct;
        self.coloring.palette_idx = s.palette_idx;
        self.coloring.use_binary = s.use_binary;
        self.coloring.use_duotone = s.use_duotone;
        self.coloring.use_custom_palette = s.use_custom_palette;
        self.invalidate_refs();
    }

    /// Render one offscreen frame of the standardized dive at `dims`/`log2mag`, returning the
    /// sampled CPU (reference build) + GPU (full offscreen render) time in ms. Blocks only for the
    /// single frame (like one export), so callers can advance the dive a frame per event loop tick.
    fn render_std_frame(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        dims: (u32, u32),
        cx: &fractadyne_core::BigFloat,
        cy: &fractadyne_core::BigFloat,
        log2mag: f64,
    ) -> (f64, f64) {
        use std::sync::atomic::{AtomicBool, AtomicU32};
        use std::time::Instant;
        let (w, h) = dims;
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let mut vp = fractadyne_core::Viewport::new(w as f64, h as f64);
        vp.set_center_log2mag(cx.clone(), cy.clone(), log2mag);
        let center_bf = [vp.center_x.clone(), vp.center_y.clone()];
        let center = vp.center_f64();
        let span = vp.complex_span_fe();
        let mag = vp.magnification();
        let l2 = vp.log2_magnification();
        let eff_iter = vp.recommended_max_iter(self.render_cfg.max_iter);
        let t = Instant::now();
        let params = self.build_params(
            center_bf, center, span, mag, l2, self.fractal, false, eff_iter, false, STD_AA,
            [w, h], 0, None,
        );
        let build_ms = t.elapsed().as_secs_f64() * 1000.0;
        let req = crate::profile::params_to_request(&params);
        let t = Instant::now();
        let _ = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel);
        let gpu_ms = t.elapsed().as_secs_f64() * 1000.0;
        (build_ms, gpu_ms)
    }

    /// Advance an in-flight standardized benchmark by exactly one dive-frame (or the uncounted
    /// warm-up frame at a pass boundary). Returns `true` only once every pass is complete (so the
    /// caller can build the report and restore state); `false` means more frames remain.
    pub(crate) fn step_std_bench(
        &mut self,
        run: &mut StdBench,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
    ) -> bool {
        const LOG2_10: f64 = std::f64::consts::LOG2_10;
        let dims = run.res.dims();
        let frames = STD_FRAMES.max(2) as i32;

        // Start a fresh pass from a cold reference cache when none is in flight.
        if run.cur.is_none() {
            self.invalidate_refs();
            let mut b = Bench::new();
            b.warmup_left = 0; // explicit warm-up frame below instead
            run.cur = Some(b);
            run.frame_in_pass = -1;
        }

        if run.frame_in_pass < 0 {
            // Warm-up frame (shader/pipeline compile, first upload) — not counted.
            let (cx, cy) = (run.cx.clone(), run.cy.clone());
            let _ = self.render_std_frame(device, queue, dims, &cx, &cy, 0.0);
            run.frame_in_pass = 0;
            return false;
        }

        // One counted dive frame.
        let i = run.frame_in_pass;
        let frac = i as f64 / (frames - 1) as f64;
        let log2mag = frac * run.zoom_log10 * LOG2_10;
        let (cx, cy) = (run.cx.clone(), run.cy.clone());
        let (build_ms, gpu_ms) = self.render_std_frame(device, queue, dims, &cx, &cy, log2mag);
        let (ws, peak) = process_memory();
        {
            let b = run.cur.as_mut().unwrap();
            let frame_ms = build_ms + gpu_ms;
            b.frames += 1;
            b.sum_frame_ms += frame_ms;
            b.sum_cpu_ms += build_ms;
            if frame_ms > 0.0 {
                let fps = 1000.0 / frame_ms;
                b.min_fps = b.min_fps.min(fps);
                b.max_fps = b.max_fps.max(fps);
            }
            b.peak_ram = b.peak_ram.max(peak).max(ws);
            b.sum_ram += ws;
        }
        run.frame_in_pass += 1;
        if run.frame_in_pass < frames {
            return false; // pass still in progress
        }

        // Pass complete — record its average FPS and roll to the next pass (if any).
        let b = run.cur.take().unwrap();
        let f = b.frames.max(1) as f64;
        let avg_fps = if b.sum_frame_ms > 0.0 { 1000.0 / (b.sum_frame_ms / f) } else { 0.0 };
        run.pass_fps.push(avg_fps);
        run.last = Some(b);
        run.passes_done += 1;
        run.passes_done >= run.passes_total
    }

    /// Build the human-readable standardized-benchmark report (settings block + results, plus a
    /// per-pass table when it was a burn-in run).
    pub(crate) fn format_std_bench(&self, run: &StdBench) -> String {
        let (w, h) = run.res.dims();
        let si = &self.sysinfo;
        let cache = if si.l3_kb > 0 {
            format!("L2 {} KB · L3 {} MB", si.l2_kb, si.l3_kb / 1024)
        } else if si.l2_kb > 0 {
            format!("L2 {} KB", si.l2_kb)
        } else {
            "—".to_string()
        };
        let vram = if si.vram_mb > 0 { format!("{} MB", si.vram_mb) } else { "—".to_string() };

        // Aggregate across passes.
        let fps = &run.pass_fps;
        let n = fps.len().max(1) as f64;
        let mean = fps.iter().sum::<f64>() / n;
        let pmin = fps.iter().cloned().fold(f64::INFINITY, f64::min);
        let pmax = fps.iter().cloned().fold(0.0_f64, f64::max);
        let var = fps.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let sd = var.sqrt();

        let b = run.last.as_ref();
        let (aframe, acpu, agpu, aram, pram) = if let Some(b) = b {
            let bf = b.frames.max(1) as f64;
            let af = b.sum_frame_ms / bf;
            let ac = b.sum_cpu_ms / bf;
            (af, ac, (b.sum_frame_ms - b.sum_cpu_ms).max(0.0) / bf, b.sum_ram as f64 / bf, b.peak_ram as f64)
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        };
        let mb = |bytes: f64| bytes / (1024.0 * 1024.0);

        let mut s = String::new();
        s.push_str("Fractadyne standardized benchmark\n");
        s.push_str(&format!("version    v{}\n", version_string()));
        s.push_str(&format!("date       {}\n", now_utc_string()));
        s.push_str(&format!("cpu        {}\n", if si.cpu.is_empty() { "—" } else { &si.cpu }));
        s.push_str(&format!("cores      {} physical / {} logical\n", si.physical, si.logical));
        s.push_str(&format!("cache      {cache}\n"));
        s.push_str(&format!("gpu        {}\n", self.gpu_name));
        s.push_str(&format!("vram       {vram}\n"));
        s.push_str("---- fixed settings (comparable across machines) ----\n");
        s.push_str(&format!("resolution {}  ({w}×{h})\n", run.res.label()));
        s.push_str(&format!("aa (ss)    {STD_AA}x  ({}× samples/px)\n", STD_AA * STD_AA));
        s.push_str("fractal    Mandelbrot   coloring  Smooth (Ember)\n");
        s.push_str("max-iter   auto (depth-adaptive)\n");
        s.push_str("deep zoom  series-approx on · BLA on · glitch off\n");
        s.push_str(&format!(
            "dive       {} frames, 1 → 1e{:.0}× (seahorse valley)\n",
            STD_FRAMES, run.zoom_log10
        ));
        s.push_str("----------------------------------------\n");
        if run.passes_total > 1 {
            s.push_str(&format!("burn-in    {} passes\n", run.passes_done));
            s.push_str(&format!("avg FPS    {mean:8.1}   (mean of passes)\n"));
            s.push_str(&format!("min FPS    {pmin:8.1}   (worst pass)\n"));
            s.push_str(&format!("max FPS    {pmax:8.1}   (best pass)\n"));
            s.push_str(&format!("std dev    {sd:8.2}   ({:.1}% — stability)\n", if mean > 0.0 { 100.0 * sd / mean } else { 0.0 }));
            if run.pass_fps.len() >= 2 {
                let first = run.pass_fps[0];
                let last = *run.pass_fps.last().unwrap();
                let drop = if first > 0.0 { 100.0 * (last - first) / first } else { 0.0 };
                s.push_str(&format!("throttle   {drop:+8.1}%  (last vs first pass)\n"));
            }
            s.push_str("----------------------------------------\n");
            s.push_str("pass   FPS\n");
            for (i, f) in run.pass_fps.iter().enumerate() {
                s.push_str(&format!("{:>4}  {:>6.1}\n", i + 1, f));
            }
            s.push_str("----------------------------------------\n");
        } else {
            s.push_str(&format!("avg FPS    {mean:8.1}\n"));
            if let Some(b) = b {
                let bmin = if b.min_fps.is_finite() { b.min_fps } else { 0.0 };
                s.push_str(&format!("min FPS    {bmin:8.1}   (deepest frames)\n"));
                s.push_str(&format!("max FPS    {:8.1}   (shallow frames)\n", b.max_fps));
            }
            s.push_str("----------------------------------------\n");
        }
        s.push_str(&format!("avg frame  {aframe:8.2} ms\n"));
        s.push_str(&format!("avg CPU    {acpu:8.2} ms   (reference build)\n"));
        s.push_str(&format!("avg GPU    {agpu:8.2} ms   (render)\n"));
        s.push_str(&format!("avg RAM    {:8.1} MB\n", mb(aram)));
        s.push_str(&format!("peak RAM   {:8.1} MB\n", mb(pram)));
        s.push_str("----------------------------------------\n");
        s.push_str(&format!("score      {:8.0}   (avg FPS × 100)", mean * 100.0));
        s
    }
}

#[cfg(test)]
mod schema_tests {
    /// The checked-in TOURS.md must match what the schema generates — regenerate it with
    /// `fractadyne --dump-tour-schema > TOURS.md` after editing `TOUR_SCHEMA`. (Line endings are
    /// normalized so a CRLF working copy still matches the LF the generator emits.)
    #[test]
    fn tour_schema_doc_current() {
        let generated = super::tour_schema_markdown();
        let committed = include_str!("../../../TOURS.md").replace("\r\n", "\n");
        assert_eq!(
            generated, committed,
            "TOURS.md is stale — run `fractadyne --dump-tour-schema > TOURS.md` to regenerate"
        );
    }
}
