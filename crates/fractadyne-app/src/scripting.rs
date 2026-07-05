//! Scripting & benchmark: TOML keyframe camera tours (`Tools -> Play script`) and the
//! built-in benchmark tour, plus the playback engine that glides center+zoom along the
//! timeline and samples FPS/CPU/RAM. `Playback`/`Bench` are pub(crate) (held as app state).

use crate::{now_utc_string, process_memory, version_string, FractadyneApp, FractalKind};
use serde::Deserialize;

/// Script schema version. Bump on a breaking change to an existing key's meaning; purely
/// additive new keys don't need it (unknown keys are ignored, missing ones default — so old
/// and new builds interoperate). A script whose `format_version` exceeds this is from a newer
/// build: we still play the parts we understand but warn that newer features may not apply.
pub(crate) const SCRIPT_FORMAT_VERSION: u32 = 1;

/// On-disk script format (TOML). A keyframe with no `center_x`/`center_y` inherits
/// the previous keyframe's center (handy for pure zoom-in tours). Captions are timed
/// independently of keyframes (so narration can span or overlap camera moves).
#[derive(Deserialize, Default)]
struct ScriptFile {
    #[serde(default)]
    name: String,
    /// Schema version the script was authored for (see `SCRIPT_FORMAT_VERSION`).
    #[serde(default)]
    format_version: Option<u32>,
    /// Coloring palette for the tour: a preset name (e.g. "Ember") or index. Applied on load so a
    /// tour looks the same regardless of the session palette (e.g. a binary palette would render
    /// deep exterior-only views as one flat color). Omit to keep the current palette.
    #[serde(default)]
    palette: Option<String>,
    #[serde(default, rename = "loop")]
    loop_: bool,
    /// Burn a small zoom/coordinate HUD into the top-left of every frame. Overridden on by the
    /// `--show-location` CLI flag. Omit / false to leave frames clean.
    #[serde(default)]
    show_location: Option<bool>,
    #[serde(default)]
    keyframe: Vec<KeyframeFile>,
    #[serde(default)]
    caption: Vec<CaptionFile>,
    #[serde(default)]
    callout: Vec<CalloutFile>,
    #[serde(default)]
    spotlight: Vec<SpotlightFile>,
}

/// A spotlight vignette: dim everything outside a soft circle centred on a fractal coordinate.
#[derive(Deserialize, Clone)]
struct SpotlightFile {
    center_x: String,
    center_y: String,
    /// Circle radius as a fraction of the frame height (default 0.25).
    #[serde(default)]
    radius: Option<f64>,
    #[serde(default)]
    at: f64,
    #[serde(default)]
    secs: f64,
    #[serde(default)]
    fade: Option<f64>,
    /// How dark outside the circle, 0..1 (default 0.7).
    #[serde(default)]
    dim: Option<f64>,
    /// Soft-edge width as a fraction of the frame height (default 0.08).
    #[serde(default)]
    softness: Option<f64>,
}

/// A labeled marker anchored to a complex coordinate (tracks the point as the view moves).
#[derive(Deserialize, Clone)]
struct CalloutFile {
    text: String,
    center_x: String,
    center_y: String,
    #[serde(default)]
    at: f64,
    #[serde(default)]
    secs: f64,
    #[serde(default)]
    fade: Option<f64>,
    #[serde(default)]
    size: Option<f64>,
}

/// A timed on-screen caption (narration overlay). Additive to the camera path.
#[derive(Deserialize, Clone)]
struct CaptionFile {
    /// The text to show (supports `\n` for multiple lines).
    text: String,
    /// When it appears on the timeline (seconds from the start).
    #[serde(default)]
    at: f64,
    /// How long it stays (seconds). 0 or omitted ⇒ until the tour ends.
    #[serde(default)]
    secs: f64,
    /// Screen anchor: `top`, `center`, or `bottom` (default).
    #[serde(default)]
    pos: Option<String>,
    /// Fade in/out time (seconds) at each end. Default 0.4.
    #[serde(default)]
    fade: Option<f64>,
    /// Font size in points. Default 22.
    #[serde(default)]
    size: Option<f64>,
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

#[derive(Deserialize, Clone)]
struct KeyframeFile {
    /// Seconds to glide here from the previous keyframe.
    #[serde(default)]
    secs: f64,
    #[serde(default)]
    center_x: Option<String>,
    #[serde(default)]
    center_y: Option<String>,
    #[serde(default = "one_f64")]
    mag: f64,
    /// Magnification as log10 (so deep zooms past f64's ~1e308 ceiling are expressible, e.g.
    /// `mag_log10 = 420` for 1e420×). Takes precedence over `mag` when present.
    #[serde(default)]
    mag_log10: Option<f64>,
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
            SchemaField { name: "name", ty: "string", default: "\"\"", doc: "Display name (shown in render progress and the end-of-script toast)." },
            SchemaField { name: "format_version", ty: "int", default: "0", doc: "Schema version the script targets. A version newer than this build warns that some annotations may not render." },
            SchemaField { name: "palette", ty: "string", default: "(session)", doc: "Coloring palette preset name (e.g. \"Ember\") or index, applied on load so the tour looks the same regardless of the current palette." },
            SchemaField { name: "loop", ty: "bool", default: "false", doc: "Loop the tour during live playback (Tools -> Play script)." },
            SchemaField { name: "show_location", ty: "bool", default: "false", doc: "Burn a zoom-level + coordinate HUD into every frame (same as the --show-location CLI flag)." },
            SchemaField { name: "keyframe", ty: "[[keyframe]]", default: "[]", doc: "The camera path (at least one required) — see below." },
            SchemaField { name: "caption", ty: "[[caption]]", default: "[]", doc: "Timed narration overlays — see below." },
            SchemaField { name: "callout", ty: "[[callout]]", default: "[]", doc: "Labeled markers anchored to a coordinate — see below." },
            SchemaField { name: "spotlight", ty: "[[spotlight]]", default: "[]", doc: "Dim-outside-a-circle vignettes — see below." },
        ],
    },
    SchemaTable {
        toml: "[[keyframe]]",
        repeatable: true,
        summary: "A camera waypoint. The view eases from the previous keyframe to this one over `secs`, then holds for `hold`. Discrete settings (center, fractal, julia, dual, orbits) inherit forward until changed.",
        fields: &[
            SchemaField { name: "secs", ty: "float", default: "0", doc: "Seconds to glide here from the previous keyframe." },
            SchemaField { name: "center_x", ty: "string", default: "(inherit)", doc: "Full-precision decimal real part of the center. Omit to inherit the previous center (pure zoom)." },
            SchemaField { name: "center_y", ty: "string", default: "(inherit)", doc: "Full-precision decimal imaginary part of the center." },
            SchemaField { name: "mag", ty: "float", default: "1", doc: "Magnification at this keyframe (up to ~1e308)." },
            SchemaField { name: "mag_log10", ty: "float", default: "(unset)", doc: "Magnification as log10, for depths past f64's ~1e308 ceiling (e.g. 420 = 1e420x). Takes precedence over `mag`." },
            SchemaField { name: "fractal", ty: "string", default: "(inherit)", doc: "Fractal family name (e.g. \"Mandelbrot\", \"Burning Ship\")." },
            SchemaField { name: "julia", ty: "bool", default: "false", doc: "Julia mode for the family." },
            SchemaField { name: "ease", ty: "string", default: "smooth", doc: "Easing for the glide arriving here: smooth, linear, smoother, in (accelerate), or out (decelerate)." },
            SchemaField { name: "hold", ty: "float", default: "0", doc: "Seconds to pause at this keyframe before the next glide." },
            SchemaField { name: "dual", ty: "bool", default: "false", doc: "Show the linked dual view (Mandelbrot + its Julia set side by side)." },
            SchemaField { name: "julia_re", ty: "float", default: "(unset)", doc: "Pin the Julia parameter c (real part). Both julia_re and julia_im are required; interpolated between keyframes." },
            SchemaField { name: "julia_im", ty: "float", default: "(unset)", doc: "Julia parameter c (imaginary part)." },
            SchemaField { name: "orbits", ty: "bool", default: "false", doc: "Overlay the escape-time orbit (the path of z under iteration)." },
            SchemaField { name: "orbit_re", ty: "float", default: "(unset)", doc: "The point whose orbit to draw, real part (both components required; interpolated)." },
            SchemaField { name: "orbit_im", ty: "float", default: "(unset)", doc: "Orbit point imaginary part." },
        ],
    },
    SchemaTable {
        toml: "[[caption]]",
        repeatable: true,
        summary: "A timed on-screen caption (narration), independent of the camera path.",
        fields: &[
            SchemaField { name: "text", ty: "string", default: "(required)", doc: "The text to show (supports \\n for multiple lines)." },
            SchemaField { name: "at", ty: "float", default: "0", doc: "When it appears on the timeline (seconds from the start)." },
            SchemaField { name: "secs", ty: "float", default: "0 = until end", doc: "How long it stays (seconds); 0/omitted keeps it until the tour ends." },
            SchemaField { name: "pos", ty: "string", default: "bottom", doc: "Screen anchor: top, center, or bottom." },
            SchemaField { name: "fade", ty: "float", default: "0.4", doc: "Fade in/out time (seconds) at each end." },
            SchemaField { name: "size", ty: "float", default: "22", doc: "Font size in points (scaled to the frame height)." },
        ],
    },
    SchemaTable {
        toml: "[[callout]]",
        repeatable: true,
        summary: "A labeled marker anchored to a fractal coordinate — it tracks the point as the view pans/zooms, with a leader line to the label.",
        fields: &[
            SchemaField { name: "text", ty: "string", default: "(required)", doc: "Label text." },
            SchemaField { name: "center_x", ty: "string", default: "(required)", doc: "Anchor coordinate, real part (full-precision decimal)." },
            SchemaField { name: "center_y", ty: "string", default: "(required)", doc: "Anchor coordinate, imaginary part." },
            SchemaField { name: "at", ty: "float", default: "0", doc: "When it appears (seconds)." },
            SchemaField { name: "secs", ty: "float", default: "0 = until end", doc: "How long it stays (seconds)." },
            SchemaField { name: "fade", ty: "float", default: "0.4", doc: "Fade in/out time (seconds)." },
            SchemaField { name: "size", ty: "float", default: "18", doc: "Label font size in points." },
        ],
    },
    SchemaTable {
        toml: "[[spotlight]]",
        repeatable: true,
        summary: "A vignette that dims everything outside a soft circle to draw the eye. Anchored to a coordinate so it tracks the point and stays a constant on-screen size.",
        fields: &[
            SchemaField { name: "center_x", ty: "string", default: "(required)", doc: "Circle center, real part (full-precision decimal)." },
            SchemaField { name: "center_y", ty: "string", default: "(required)", doc: "Circle center, imaginary part." },
            SchemaField { name: "radius", ty: "float", default: "0.25", doc: "Circle radius as a fraction of the frame height." },
            SchemaField { name: "softness", ty: "float", default: "0.08", doc: "Soft-edge width as a fraction of the frame height." },
            SchemaField { name: "dim", ty: "float", default: "0.7", doc: "How dark outside the circle (0..1)." },
            SchemaField { name: "at", ty: "float", default: "0", doc: "When it appears (seconds)." },
            SchemaField { name: "secs", ty: "float", default: "0 = until end", doc: "How long it stays (seconds)." },
            SchemaField { name: "fade", ty: "float", default: "0.4", doc: "Fade in/out time (seconds)." },
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
         with `--prefix`). Ready-made examples live in [`tours/`](tours/); run \
         `fractadyne --help` for the full CLI.\n\n",
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
         Keyframe timing is cumulative: each `[[keyframe]]` adds `secs` of glide from the previous \
         keyframe's hold-end, then `hold` seconds of pause. The first keyframe starts at t=0. \
         Caption / callout / spotlight `at` times are absolute seconds from the start, and `secs = 0` \
         means \"until the tour ends\". At `--fps N` the tour renders `round(total * N) + 1` frames.\n\n\
         For deep dives, keep `secs` generous, use `ease = \"in\"` / `\"out\"` at the ends with `linear` \
         for the cruise, and express magnifications past f64 range with `mag_log10`. Pan at low zoom \
         *before* diving so the camera doesn't zoom through the set's black interior.\n\n",
    );

    s.push_str(
        "## Example\n\n\
         ```toml\n\
         name = \"Mini tour\"\n\
         palette = \"Ember\"\n\
         format_version = 1\n\n\
         [[keyframe]]            # overview\n\
         secs = 0\n\
         center_x = \"-0.5\"\n\
         center_y = \"0.0\"\n\
         mag = 1\n\
         hold = 2\n\n\
         [[keyframe]]            # dive into Seahorse Valley\n\
         secs = 6\n\
         center_x = \"-0.743643887037158704752191506114774\"\n\
         center_y = \"0.131825904205311970493132056385139\"\n\
         mag_log10 = 6\n\
         ease = \"in\"\n\
         hold = 3\n\n\
         [[caption]]\n\
         text = \"Seahorse Valley\"\n\
         at = 8\n\
         secs = 4\n\
         pos = \"bottom\"\n\n\
         [[spotlight]]\n\
         center_x = \"-0.743643887037158704752191506114774\"\n\
         center_y = \"0.131825904205311970493132056385139\"\n\
         radius = 0.28\n\
         at = 8\n\
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

fn one_f64() -> f64 {
    1.0
}

/// A resolved keyframe: parsed center, the time the glide *reaches* it, how long it holds there,
/// and the easing of the glide arriving at it.
struct Kf {
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
    orbit: Option<(f64, f64)>,
}

/// The fully-resolved tour state at a moment in time: interpolated camera + the active keyframe's
/// discrete overlays (dual view, Julia pin, orbits).
pub(crate) struct Sampled {
    pub(crate) cx: fractadyne_core::BigFloat,
    pub(crate) cy: fractadyne_core::BigFloat,
    pub(crate) logmag: f64,
    pub(crate) fractal: FractalKind,
    pub(crate) julia: bool,
    pub(crate) dual: bool,
    pub(crate) julia_c: Option<(f64, f64)>,
    pub(crate) orbits: bool,
    pub(crate) orbit: Option<(f64, f64)>,
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
    kfs: Vec<Kf>,
    pub(crate) total: f64,
    pub(crate) t0: Option<f64>,
    loop_: bool,
    pub(crate) bench: Option<Bench>,
    /// Timed narration overlays (drawn by the app over the fractal + into exported frames).
    pub(crate) captions: Vec<Caption>,
    /// Coordinate-anchored labeled markers (drawn over the fractal + into exported frames).
    pub(crate) callouts: Vec<Callout>,
    /// Spotlight vignettes (dim outside a soft circle; applied in the color shader).
    pub(crate) spotlights: Vec<Spotlight>,
    /// Current tour time (seconds), updated each frame — so the caption overlay knows what to show.
    pub(crate) cur_t: f64,
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
        let mk = |cx, cy, lm, julia_c, orbit| Sampled {
            cx,
            cy,
            logmag: lm,
            fractal: a.fractal,
            julia: a.julia,
            dual: a.dual,
            julia_c,
            orbits: a.orbits,
            orbit,
        };
        // Holding at `a` (or past the final keyframe): return its state unchanged.
        if e <= a.at + a.hold || i + 1 >= n {
            return mk(a.cx.clone(), a.cy.clone(), a.logmag, a.julia_c, a.orbit);
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
        mk(
            fractadyne_core::lerp_bf(&a.cx, &b.cx, ease, p),
            fractadyne_core::lerp_bf(&a.cy, &b.cy, ease, p),
            lm,
            julia_c,
            orbit,
        )
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
    lp.x = lp.x.clamp(bounds.min.x + pad.x + 2.0, (bounds.max.x - sz.x - pad.x - 2.0).max(bounds.min.x));
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

/// Resolve a parsed script file into a playable tour (parses centers, accumulates
/// keyframe times, fills inherited centers). Returns `None` if it has no keyframes.
fn resolve_script(sf: ScriptFile, bench: Option<Bench>) -> Option<Playback> {
    if sf.keyframe.is_empty() {
        return None;
    }
    let mut kfs = Vec::with_capacity(sf.keyframe.len());
    let mut at = 0.0;
    let mut last = ("-0.5".to_string(), "0.0".to_string());
    // Discrete overlay state, inherited forward until a keyframe changes it.
    let (mut dual, mut julia_c, mut orbits, mut orbit) = (false, None, false, None);
    for k in &sf.keyframe {
        at += k.secs.max(0.0); // glide time from the previous keyframe's hold-end to here
        let reach = at;
        let hold = k.hold.max(0.0);
        at += hold; // then hold here before the next glide begins
        if let Some(x) = &k.center_x {
            last.0 = x.clone();
        }
        if let Some(y) = &k.center_y {
            last.1 = y.clone();
        }
        let cx = fractadyne_core::parse_bf(&last.0)?;
        let cy = fractadyne_core::parse_bf(&last.1)?;
        let fractal = k
            .fractal
            .as_deref()
            .and_then(FractalKind::from_name)
            .unwrap_or(FractalKind::Mandelbrot);
        // log-magnification: `mag_log10` (for deep zooms past f64 range) wins over `mag`.
        let logmag = match k.mag_log10 {
            Some(l10) => (l10.max(0.0)) * std::f64::consts::LN_10,
            None => k.mag.max(1.0).ln(),
        };
        // Discrete overlays: update only when the keyframe specifies them; otherwise inherit.
        if let Some(d) = k.dual {
            dual = d;
        }
        if let (Some(r), Some(i)) = (k.julia_re, k.julia_im) {
            julia_c = Some((r, i));
        }
        if let Some(o) = k.orbits {
            orbits = o;
        }
        if let (Some(r), Some(i)) = (k.orbit_re, k.orbit_im) {
            orbit = Some((r, i));
        }
        kfs.push(Kf {
            at: reach,
            hold,
            ease: EaseKind::parse(k.ease.as_deref()),
            cx,
            cy,
            logmag,
            fractal,
            julia: k.julia.unwrap_or(false),
            dual,
            julia_c,
            orbits,
            orbit,
        });
    }
    let total = kfs.last().map(|k| k.at + k.hold).unwrap_or(0.0);
    let captions = sf
        .caption
        .iter()
        .filter(|c| !c.text.is_empty())
        .map(|c| {
            let start = c.at.max(0.0);
            let end = if c.secs > 0.0 { start + c.secs } else { total.max(start) };
            let pos = match c.pos.as_deref().map(|s| s.to_ascii_lowercase()).as_deref() {
                Some("top") => CaptionPos::Top,
                Some("center") | Some("centre") | Some("middle") => CaptionPos::Center,
                _ => CaptionPos::Bottom,
            };
            Caption {
                text: c.text.clone(),
                start,
                end,
                fade: c.fade.unwrap_or(0.4).max(0.0),
                pos,
                size: c.size.unwrap_or(22.0).clamp(8.0, 96.0) as f32,
            }
        })
        .collect();
    let callouts = sf
        .callout
        .iter()
        .filter(|c| !c.text.is_empty())
        .filter_map(|c| {
            let cx = fractadyne_core::parse_bf(&c.center_x)?;
            let cy = fractadyne_core::parse_bf(&c.center_y)?;
            let start = c.at.max(0.0);
            let end = if c.secs > 0.0 { start + c.secs } else { total.max(start) };
            Some(Callout {
                text: c.text.clone(),
                cx,
                cy,
                start,
                end,
                fade: c.fade.unwrap_or(0.4).max(0.0),
                size: c.size.unwrap_or(18.0).clamp(8.0, 96.0) as f32,
            })
        })
        .collect();
    let spotlights = sf
        .spotlight
        .iter()
        .filter_map(|s| {
            let cx = fractadyne_core::parse_bf(&s.center_x)?;
            let cy = fractadyne_core::parse_bf(&s.center_y)?;
            let start = s.at.max(0.0);
            let end = if s.secs > 0.0 { start + s.secs } else { total.max(start) };
            Some(Spotlight {
                cx,
                cy,
                radius: s.radius.unwrap_or(0.25).clamp(0.02, 2.0) as f32,
                soft: s.softness.unwrap_or(0.08).clamp(0.0, 1.0) as f32,
                dim: s.dim.unwrap_or(0.7).clamp(0.0, 1.0) as f32,
                start,
                end,
                fade: s.fade.unwrap_or(0.4).max(0.0),
            })
        })
        .collect();
    Some(Playback {
        name: if sf.name.is_empty() {
            "Script".to_string()
        } else {
            sf.name
        },
        kfs,
        total,
        t0: None,
        loop_: sf.loop_,
        bench,
        captions,
        callouts,
        spotlights,
        cur_t: 0.0,
    })
}

/// Built-in benchmark tour: a steady zoom into a Seahorse-Valley spiral over a fixed
/// timeline, so successive builds / machines render the same work.
fn benchmark_playback() -> Playback {
    let cx = "-0.743643887037158704752191506114774";
    let cy = "0.131825904205311970493132056385139";
    let mags = [1.0, 1.0e3, 1.0e6, 1.0e9, 1.0e12];
    let mut keyframe = Vec::new();
    for (i, &m) in mags.iter().enumerate() {
        keyframe.push(KeyframeFile {
            secs: if i == 0 { 0.0 } else { 4.0 },
            center_x: Some(cx.to_string()),
            center_y: Some(cy.to_string()),
            mag: m,
            mag_log10: None,
            fractal: Some("Mandelbrot".to_string()),
            julia: Some(false),
            ease: None,
            hold: 0.0,
            dual: None,
            julia_re: None,
            julia_im: None,
            orbits: None,
            orbit_re: None,
            orbit_im: None,
        });
    }
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

    /// Apply a script's `palette` (preset name, case-insensitive, or index). A preset clears any
    /// binary/duotone/custom palette so the tour colours consistently (deep exterior-only views
    /// render as a gradient rather than one flat binary color).
    fn apply_script_palette(&mut self, spec: &str) {
        let s = spec.trim();
        let idx = s
            .parse::<usize>()
            .ok()
            .or_else(|| fractadyne_color::PRESETS.iter().position(|p| p.name.eq_ignore_ascii_case(s)));
        if let Some(i) = idx {
            self.palette_idx = i.min(fractadyne_color::PRESETS.len() - 1);
            self.use_binary = false;
            self.use_duotone = false;
            self.use_custom_palette = false;
        }
    }

    /// Load a camera-tour script (TOML) via a file dialog and start playing it.
    pub(crate) fn load_script(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Fractadyne script (TOML)", &["toml"])
            .pick_file()
        else {
            return;
        };
        let parsed = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| toml::from_str::<ScriptFile>(&t).ok());
        let script_ver = parsed.as_ref().and_then(|sf| sf.format_version).unwrap_or(0);
        let palette = parsed.as_ref().and_then(|sf| sf.palette.clone());
        match parsed.and_then(|sf| resolve_script(sf, None)) {
            Some(pb) => {
                if let Some(p) = &palette {
                    self.apply_script_palette(p);
                }
                if script_ver > SCRIPT_FORMAT_VERSION {
                    self.bench_report = Some(format!(
                        "Note: \"{}\" was authored for a newer script format (v{script_ver} > \
                         v{SCRIPT_FORMAT_VERSION}). Playing what this build understands; newer \
                         annotations may not appear.",
                        pb.name
                    ));
                    self.bench_open = true;
                }
                self.playback = Some(pb);
            }
            None => self.bench_report = Some(format!("Could not load script:\n{}", path.display())),
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

    /// Render a keyframe-tour script (TOML) to a numbered PNG frame sequence — the headless
    /// `--render-tour` mode for producing a deep-zoom dive video. Steps the timeline at a
    /// fixed `fps`, rendering each frame at `width×height` (× `ss` supersampling) via the
    /// offscreen export path. Blocking; assemble the frames afterward (e.g. with ffmpeg).
    #[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 3: extract a TourRenderConfig + scripting crate
    pub(crate) fn render_tour_to_dir(
        &mut self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        script_path: &std::path::Path,
        fps: f64,
        width: u32,
        height: u32,
        ss: u32,
        out_dir: &std::path::Path,
        // Frame-name prefix: frames are written `<prefix>_00000.png`.
        prefix: &str,
        // Replace existing frames without prompting (CLI `--overwrite` / `-y`).
        overwrite: bool,
        // Resume an interrupted render (CLI `--resume`): silently keep frames that already exist and
        // render only the missing ones. Takes precedence over `overwrite` for existing frames.
        resume: bool,
        // Some(path) → after the PNG sequence is written, invoke `ffmpeg` to assemble it into an
        // H.264 mp4 at `path` (frames are kept). None → leave just the PNG sequence.
        mp4: Option<&std::path::Path>,
    ) -> Result<String, String> {
        let text = std::fs::read_to_string(script_path)
            .map_err(|e| format!("read {}: {e}", script_path.display()))?;
        let sf: ScriptFile = toml::from_str(&text).map_err(|e| format!("parse script: {e}"))?;
        if sf.format_version.unwrap_or(0) > SCRIPT_FORMAT_VERSION {
            eprintln!(
                "Warning: script format v{} is newer than this build (v{SCRIPT_FORMAT_VERSION}); \
                 newer annotations may not render.",
                sf.format_version.unwrap_or(0)
            );
        }
        let palette = sf.palette.clone();
        let show_location = self.show_location || sf.show_location.unwrap_or(false);
        let pb = resolve_script(sf, None).ok_or("script has no keyframes")?;
        if let Some(p) = &palette {
            self.apply_script_palette(p);
        }
        std::fs::create_dir_all(out_dir)
            .map_err(|e| format!("create {}: {e}", out_dir.display()))?;
        // Single-view offscreen render at the requested frame size.
        self.dual = false;
        // Auto-scale iterations with depth so deep tour frames resolve (a fixed low cap would
        // render the deep structure as under-iterated blobs). Keep a high base for detail.
        self.auto_iter = true;
        self.max_iter = self.max_iter.max(500_000);
        self.viewport = fractadyne_core::Viewport::new(width as f64, height as f64);
        self.export.width = width;
        self.export.ss = ss.max(1);
        let fps = fps.max(1.0);
        let frames: u64 = if pb.total <= 0.0 { 1 } else { (pb.total * fps).round() as u64 + 1 };
        println!(
            "Rendering tour \"{}\": {frames} frames at {width}×{height} ss{}, {fps} fps ({:.1}s)…",
            pb.name, self.export.ss, pb.total
        );
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
        let started = std::time::Instant::now();
        for fi in 0..frames {
            let t = if pb.total <= 0.0 { 0.0 } else { (fi as f64 / fps).min(pb.total) };
            let s = pb.sample(t);
            self.fractal = s.fractal;
            self.julia_mode = s.julia && s.fractal.supports_julia();
            self.dual = s.dual;
            if let Some(c) = s.julia_c {
                self.julia_c = c;
            }
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
            let this_ref = match pending_ref.take() {
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
                    if let Some(rx) = self.spawn_export_reference(&vp2, self.julia_mode) {
                        pending_ref = Some((fi + 1, rx));
                    }
                }
            }
            let progress = std::sync::atomic::AtomicU32::new(0);
            let cancel = std::sync::atomic::AtomicBool::new(false);
            let render = |app: &Self, vp: &fractadyne_core::Viewport, julia: bool, w: u32, vg| -> Result<fractadyne_gpu::ExportResult, String> {
                let mut req = app.current_export_request_for(vp, julia);
                req.width = w;
                req.height = height;
                req.ss = app.export.ss;
                req.max_iter = req.max_iter.max(200);
                req.vignette = vg;
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
                // Single view: use the pipelined precomputed reference (falls back to sync internally).
                let mut req = self.current_export_request_with_ref(&self.viewport, self.julia_mode, this_ref);
                req.width = width;
                req.height = height;
                req.ss = self.export.ss;
                req.max_iter = req.max_iter.max(200);
                req.vignette = vg;
                let r = fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
                    .map_err(|e| format!("frame {fi}: {e}"))?;
                (r.pixels, r.width, r.height)
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
            if fi % 10 == 0 || fi + 1 == frames {
                let done = fi + 1;
                let elapsed = started.elapsed().as_secs_f64();
                let rate = if elapsed > 0.0 { done as f64 / elapsed } else { 0.0 };
                let eta = if rate > 0.0 { (frames - done) as f64 / rate } else { 0.0 };
                println!(
                    "  frame {done}/{frames}  ({} elapsed, {} left, {rate:.2} fps)",
                    fmt_hms(elapsed),
                    fmt_hms(eta)
                );
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
        println!(
            "Rendered {frames} frame(s) in {} → {}",
            fmt_hms(render_secs),
            out_dir.display()
        );

        // Optionally assemble the PNG sequence into an mp4 via ffmpeg (kept separate from the
        // frames so a failed/absent ffmpeg never loses the render).
        let pattern = out_dir.join(format!("{prefix}_%05d.png"));
        if let Some(mp4_path) = mp4 {
            println!("Encoding → {} (ffmpeg)…", mp4_path.display());
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
                .arg(mp4_path)
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
    }

    /// Advance the active camera tour by one frame; drives the view and, for a
    /// benchmark, samples performance. Returns true while still playing.
    pub(crate) fn advance_playback(&mut self, ctx: &egui::Context) -> bool {
        let Some(mut pb) = self.playback.take() else {
            return false;
        };
        let now = ctx.input(|i| i.time);
        let t0 = *pb.t0.get_or_insert(now);
        let mut elapsed = now - t0;
        if pb.loop_ && pb.total > 0.0 && elapsed >= pb.total {
            pb.t0 = Some(now);
            elapsed = 0.0;
        }
        let finished = !pb.loop_ && elapsed >= pb.total;
        let e = elapsed.clamp(0.0, pb.total);
        pb.cur_t = e; // for the caption overlay
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
        self.show_orbits = s.orbits;
        self.tour_orbit = s.orbit;
        // log2 path so playback stays exact past f64's 1e308× ceiling.
        self.viewport.set_center_log2mag(s.cx, s.cy, s.logmag / std::f64::consts::LN_2);
        self.settle_t = [now; 2]; // glide → cheap (interacting) render path

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

        if finished {
            if let Some(b) = pb.bench.take() {
                self.bench_report = Some(self.format_bench(&pb, &b));
                self.bench_open = true;
            } else {
                self.set_toast(format!("Script finished — \"{}\"", pb.name), ctx);
            }
            return false; // pb dropped → playback stops at the final keyframe
        }
        self.playback = Some(pb);
        true
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
            color_method: self.color_method,
            auto_iter: self.auto_iter,
            aa: self.aa,
            series_approx: self.series_approx,
            use_bla: self.use_bla,
            glitch_correct: self.glitch_correct,
            palette_idx: self.palette_idx,
            use_binary: self.use_binary,
            use_duotone: self.use_duotone,
            use_custom_palette: self.use_custom_palette,
        };
        // Pin the canonical configuration.
        self.set_fractal(FractalKind::Mandelbrot);
        self.julia_mode = false;
        self.dual = false;
        self.color_method = 0; // smooth
        self.auto_iter = true; // depth-appropriate, deterministic per depth
        self.aa = STD_AA;
        self.series_approx = true;
        self.use_bla = true;
        self.glitch_correct = false; // data-dependent cost → off for a deterministic timing
        self.palette_idx = 0; // Ember
        self.use_binary = false;
        self.use_duotone = false;
        self.use_custom_palette = false;
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
        self.color_method = s.color_method;
        self.auto_iter = s.auto_iter;
        self.aa = s.aa;
        self.series_approx = s.series_approx;
        self.use_bla = s.use_bla;
        self.glitch_correct = s.glitch_correct;
        self.palette_idx = s.palette_idx;
        self.use_binary = s.use_binary;
        self.use_duotone = s.use_duotone;
        self.use_custom_palette = s.use_custom_palette;
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
        let eff_iter = vp.recommended_max_iter(self.max_iter);
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
