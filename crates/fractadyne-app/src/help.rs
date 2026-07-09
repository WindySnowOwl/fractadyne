//! In-app Help content (the `Help → Help & reference…` window, F1).
//!
//! Pure presentation: each `help_*` section renders into an `egui::Ui`. The window chrome and
//! table-of-contents live in `FractadyneApp::help_window`; this module is just the text.

use crate::version_string;
use eframe::egui;

fn help_h(ui: &mut egui::Ui, t: &str) {
    ui.add_space(2.0);
    let accent = crate::theme::ui_accent(ui.ctx());
    ui.label(egui::RichText::new(t).size(18.0).strong().color(accent));
    ui.add_space(4.0);
}
fn help_sub(ui: &mut egui::Ui, t: &str) {
    ui.add_space(8.0);
    let ink = ui.visuals().strong_text_color();
    ui.label(egui::RichText::new(t).strong().color(ink));
    ui.add_space(2.0);
}
fn help_p(ui: &mut egui::Ui, t: &str) {
    ui.label(t);
    ui.add_space(3.0);
}
fn help_bullet(ui: &mut egui::Ui, t: &str) {
    ui.horizontal_top(|ui| {
        ui.add_space(4.0);
        ui.label("•");
        ui.add(egui::Label::new(t).wrap());
    });
}
/// A monospace key + wrapped description row (shortcuts / CLI flags). The key is a
/// fixed-width, LEFT-aligned column (a plain sized label centers it).
fn help_kv(ui: &mut egui::Ui, k: &str, v: &str) {
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(180.0, 0.0),
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                // Reserve the full column width so descriptions line up (a plain sized label
                // advances only by its content, leaving the descriptions ragged).
                ui.set_min_width(180.0);
                ui.add(egui::Label::new(egui::RichText::new(k).monospace()).wrap());
            },
        );
        ui.add(egui::Label::new(v).wrap());
    });
}

pub(crate) fn help_overview(ui: &mut egui::Ui) {
    help_h(ui, "Overview");
    help_p(
        ui,
        "Fractadyne is a native fractal explorer built for ultra-deep zooming and speed. \
         It draws \"escape-time\" fractals — images created by repeating one simple formula \
         at every pixel.",
    );
    help_sub(ui, "What is an escape-time fractal?");
    help_p(
        ui,
        "For each pixel the program runs a formula such as z → z² + c over and over, starting \
         from zero. If the running value stays small forever, the pixel belongs to the set and \
         is drawn dark. If it eventually grows without bound (\"escapes\"), the pixel is outside \
         the set, and its color records how many steps that took. The infinitely detailed \
         border between \"stays\" and \"escapes\" is the fractal.",
    );
    help_sub(ui, "What you can do");
    help_bullet(ui, "Pan and zoom essentially without limit (position is exact at any depth).");
    help_bullet(ui, "Switch between ten fractal families, and view any (except Newton) as a Julia set.");
    help_bullet(ui, "Recolor with preset or custom gradients and several coloring methods.");
    help_bullet(ui, "Add 3D relief lighting and glowing boundary contours.");
    help_bullet(ui, "Snap to minibrots, bookmark spots, and export high-resolution images.");
    help_bullet(ui, "Let the auto-zoom autopilot dive toward detail on its own.");
    help_bullet(ui, "Share any spot as a small \".fdn\" text snippet or file.");
    help_bullet(ui, "Run scripted tours and a hardware benchmark.");
    help_sub(ui, "First steps");
    help_p(
        ui,
        "Open the Locations menu and pick \"Seahorse Valley\", then roll the mouse wheel to \
         zoom in. Drag to pan. Or just press A and let the autopilot dive for you. Press F1 \
         at any time to return to this help.",
    );
}

pub(crate) fn help_navigation(ui: &mut egui::Ui) {
    help_h(ui, "Navigation");
    help_sub(ui, "Mouse");
    help_kv(ui, "Left-drag", "Pan the view.");
    help_kv(ui, "Mouse wheel", "Zoom in/out toward the cursor.");
    help_kv(ui, "Shift+drag", "Box zoom — drag a rectangle and it zooms to fill the view.");
    help_sub(ui, "Continuous zoom & home");
    help_kv(ui, "Hold Space", "Smoothly zoom in, anchored at the cursor.");
    help_kv(ui, "Hold Shift+Space", "Smoothly zoom out.");
    help_p(
        ui,
        "The Zoom-home button animates a gentle fly-back to the full view. \"Zoom speed\" in the \
         right panel sets the continuous-zoom rate.",
    );
    help_sub(ui, "Autopilot");
    help_p(
        ui,
        "Press A (or View → \"Auto-zoom (autopilot)\") for a hands-free dive: every fraction of a \
         second it finds the most detailed region in view and zooms smoothly toward it, re-steering \
         as new structure appears. Any navigation input — or Esc — stops it.",
    );
    help_sub(ui, "History & precise moves");
    help_kv(ui, "Backspace", "Undo the previous view.");
    help_kv(ui, "Shift+Backspace / Ctrl+Y", "Redo.");
    help_p(
        ui,
        "View → \"Go to location…\" lets you read, type, paste, or copy the exact center and zoom \
         (full precision, any depth) — handy for revisiting a spot. The Bookmarks menu saves and \
         recalls locations.",
    );
    help_p(
        ui,
        "File → \"Share location…\" captures the whole view — fractal, full-precision center, zoom, \
         and coloring — as a compact \".fdn\" snippet you can Copy, paste back and Apply, or \
         Save/Load as a file, to reproduce a spot exactly (or send it to someone).",
    );
    help_sub(ui, "Finding detail");
    help_p(
        ui,
        "Minibrots are tiny copies of the whole set hidden along the boundary. Center one roughly \
         and press M (or View → \"Find minibrot center\") to Newton-snap exactly onto its center \
         and read its period.",
    );
    help_p(
        ui,
        "View → \"Minimap overview\" shows a small map of the whole set with a \"you are here\" \
         marker and the current zoom depth; click the map to jump to a region.",
    );
}

pub(crate) fn help_options(ui: &mut egui::Ui) {
    help_h(ui, "Coloring & options");
    help_p(ui, "All of these live in the right-hand panel (and persist between sessions).");
    help_sub(ui, "Palette");
    help_p(
        ui,
        "Palette chooses a preset gradient, your Custom one, or a two-color mode. \"Edit \
         gradient…\" opens an editor where each color stop has a color and a position (0–1); add \
         up to eight stops or copy a preset to start from.",
    );
    help_p(
        ui,
        "Duotone maps the value to a smooth two-color ramp (Shadow → Highlight). Binary (set) \
         is a flat two-color view — one solid color for points inside the set and another for \
         outside, with no gradient — the clearest way to see the set's shape.",
    );
    help_p(
        ui,
        "Cycle sets how many times the gradient repeats across the iteration range (tighter or \
         looser bands). Offset rotates the whole gradient.",
    );
    help_p(
        ui,
        "Animate cycles the colors over time — Forward, Reverse, or Ping-pong shift the offset; \
         Random continuously synthesizes smoothly morphing, harmonious gradients. Speed controls \
         cycles (or morphs) per second; \"Shuffle gradient\" rolls a new one in Random mode.",
    );
    help_sub(ui, "Coloring method — how the data becomes color");
    help_kv(ui, "Smooth iteration", "Classic continuous bands by escape time.");
    help_kv(ui, "Stripe average", "Flowing bands from the orbit's angle (Stripe density slider).");
    help_kv(ui, "Triangle inequality", "Fine texture from where each step lands between bounds.");
    help_kv(ui, "Orbit trap", "Distance of the orbit to a shape (point/cross/circle); colors interior too.");
    help_kv(ui, "Distance estimate", "Shades by nearness to the boundary.");
    help_kv(ui, "Decomposition", "Cells from the final escape angle.");
    help_sub(ui, "3D relief lighting");
    help_p(
        ui,
        "Shades the surface from the boundary's slope (the derivative) for an embossed, lit look. \
         Light angle sets the direction; Relief sets strength (lower = sharper); \"Rotate light\" \
         animates it. Holomorphic families only (Mandelbrot / Multibrot).",
    );
    help_sub(ui, "Distance glow");
    help_p(
        ui,
        "Bright contour bands that densify into glowing filaments near the boundary. Glow is the \
         blend amount, Band width the spacing, and \"Animate glow\" flows them.",
    );
    help_sub(ui, "Quality & iterations");
    help_p(
        ui,
        "Iterations is the maximum number of steps before a pixel is treated as inside the set; \
         \"Auto-scale\" raises it automatically as you zoom (deeper detail needs more). Anti-alias \
         supersamples still images (2×–8×) once the view settles, taming the fine exterior \"dust\".",
    );
    help_sub(ui, "Other");
    help_p(
        ui,
        "Zoom speed sets the continuous-zoom rate; the FPS cap limits frame rate; Dual view shows a \
         Mandelbrot set and its Julia set side by side (the cursor sets the Julia parameter live).",
    );
}

pub(crate) fn help_fractals(ui: &mut egui::Ui) {
    help_h(ui, "Fractals");
    help_p(
        ui,
        "Every family iterates a formula with z starting at 0 and c set by the pixel (escape-time), \
         unless noted. z = x + iy is a complex number.",
    );
    help_sub(ui, "Mandelbrot");
    help_p(
        ui,
        "z → z² + c. The original. The set is every c whose orbit stays bounded; it is connected, \
         and its boundary is so crinkled it has Hausdorff dimension 2.",
    );
    help_sub(ui, "Multibrot 3 / 4 / 5");
    help_p(
        ui,
        "z → z^d + c for power d = 3, 4, 5. Higher powers add lobes: the set has (d−1)-fold \
         rotational symmetry (Multibrot 3 is 2-fold, 4 is 3-fold, 5 is 4-fold).",
    );
    help_sub(ui, "Tricorn (Mandelbar)");
    help_p(
        ui,
        "z → conj(z)² + c, where conj(x + iy) = x − iy. The conjugation makes it anti-holomorphic, \
         giving 3-fold symmetry and characteristic curved \"claws\".",
    );
    help_sub(ui, "Burning Ship");
    help_p(
        ui,
        "z → (|x| + i|y|)² + c — take absolute values before squaring (real = x²−y²+cx, \
         imag = 2|xy|+cy). Non-analytic; deep zooms reveal ship-like structures (traditionally \
         viewed upside-down).",
    );
    help_sub(ui, "Celtic");
    help_p(
        ui,
        "Like Mandelbrot but with the absolute value of the real part: real = |x²−y²| + cx, \
         imag = 2xy + cy. Produces heart- and shield-shaped motifs.",
    );
    help_sub(ui, "Buffalo");
    help_p(
        ui,
        "Absolute value of both parts of z²: real = |x²−y²| + cx, imag = |2xy| + cy — a cross \
         between Celtic and Burning Ship.",
    );
    help_sub(ui, "Phoenix");
    help_p(
        ui,
        "z → z² + c + p·z(n-1), where z(n-1) is the previous iterate and p is a constant (p = -0.5). \
         The memory term produces flame-like filaments.",
    );
    help_sub(ui, "Newton");
    help_p(
        ui,
        "Newton's method for the roots of z³ − 1 = 0: z → z − (z³−1)/(3z²). Rather than escape time, \
         pixels are colored by which of the three cube roots of unity the iteration converges to \
         (the basins of attraction) and how quickly; the tangled basin boundaries are the fractal.",
    );
    help_sub(ui, "Julia mode");
    help_p(
        ui,
        "For every family except Newton you can switch to a Julia set: instead of starting z at 0 \
         with c = pixel, you fix c (a parameter) and let z start at the pixel. The Julia set is the \
         boundary between starting points that stay bounded and those that escape, for that fixed c. \
         In Dual view, moving the cursor over the Mandelbrot panel sets c live.",
    );
    help_sub(ui, "Deep-zoom support");
    help_p(
        ui,
        "Mandelbrot, Multibrot 3/4/5 and Tricorn support extreme-depth (floatexp) perturbation deep \
         zoom. Burning Ship, Celtic and Buffalo are non-analytic (they take absolute values), so \
         they use a sign-aware perturbation; this now runs at floatexp range too, deep-zooming far \
         past the old ~1e28× df32 limit (rare speckle near the abs folds awaits multi-reference \
         glitch correction). Phoenix uses a two-term perturbation and also deep-zooms at floatexp \
         range (without SA/BLA, so heavier than Mandelbrot). Only Newton uses the direct path, \
         sharp to ~1e6×.",
    );
}

pub(crate) fn help_methodology(ui: &mut egui::Ui) {
    help_h(ui, "How it works");
    help_sub(ui, "Escape-time & smooth color");
    help_p(
        ui,
        "Each pixel iterates the formula until its magnitude exceeds a bailout radius or it hits the \
         iteration cap. The raw step count alone makes hard bands; adding a fractional part derived \
         from the final magnitude gives continuous, bandless color.",
    );
    help_sub(ui, "Arbitrary-precision position");
    help_p(
        ui,
        "Ordinary 64-bit numbers run out of digits near 1e15× zoom. Fractadyne keeps the view center \
         in arbitrary precision, with the number of digits growing as you zoom, so the location never \
         degrades. The pixel scale is likewise held with an extended exponent, so it doesn't stall at \
         64-bit's ~1e308× limit either — depth is bounded only by patience, not by any fixed number \
         range.",
    );
    help_sub(ui, "Perturbation");
    help_p(
        ui,
        "Iterating every pixel in high precision would be far too slow. Instead one reference pixel \
         is iterated in high precision on the CPU (the \"reference orbit\"), and every other pixel is \
         computed on the GPU as a tiny difference δ from it in fast low precision: \
         δz → 2·Z·δz + δz² + δc.",
    );
    help_sub(ui, "Extreme depth (floatexp)");
    help_p(
        ui,
        "Past about 1e28× even that tiny difference underflows 32-bit range, so it is stored as a \
         mantissa plus a separate integer exponent (\"floatexp\"), lifting the 32-bit depth wall. \
         The engine switches automatically: direct math when shallow, perturbation when deep, and \
         floatexp when deepest. Depth is then bounded by coordinate precision and the iteration \
         budget rather than a fixed limit (validated past 1e300×; the bundled tour reaches ~1e420×).",
    );
    help_sub(ui, "Reference choice & rebasing");
    help_p(
        ui,
        "The reference is chosen (scored in high precision) so its orbit stays within the view as \
         long as possible. When the difference grows too large it is \"rebased\" back onto the \
         reference to stay accurate.",
    );
    help_sub(ui, "Series approximation");
    help_p(
        ui,
        "At deep zoom the early iterations barely move δ from the reference, so they can be \
         computed all at once: δz is approximated by a short polynomial in δc (order 3), whose \
         coefficients are iterated along the reference. The renderer skips ahead to the last \
         iteration where that polynomial is still accurate for the whole view, then iterates \
         normally — saving the skipped steps with no change to the image (toggle in View). \
         Mandelbrot and Multibrot 3/4/5, deepest range; not with stripe / triangle-inequality / \
         orbit-trap / decomposition coloring (those need every iteration). Where the BLA is active \
         it already subsumes this early skip.",
    );
    help_sub(ui, "Distance estimation & lighting");
    help_p(
        ui,
        "Tracking the derivative dz/dc yields each pixel's distance to the boundary. That powers the \
         3D relief lighting, the distance glow, and the \"distance\" coloring method — all valid at \
         any zoom depth.",
    );
    help_sub(ui, "Anti-aliasing & safety");
    help_p(
        ui,
        "Still images are supersampled (2–8× per axis) once the view settles. A work budget keeps a \
         single GPU draw within the driver's watchdog limit by reducing resolution (never the \
         iteration count) at extreme settings, so deep views stay detailed instead of going blank.",
    );
}

/// One entry in the shared command-line reference. Rendered both in the in-app Help window
/// ([`help_command_line`]) and by the headless `--help` text ([`cli_help_text`]), so the two can
/// never drift out of sync.
pub(crate) enum CliRef {
    /// A section heading.
    Section(&'static str),
    /// A flag (with argument placeholders) and its one-line description.
    Flag(&'static str, &'static str),
    /// A free-standing example command line.
    Example(&'static str),
}

/// Intro shown above the flag list (both in-app and in `--help`).
pub(crate) const CLI_INTRO: &str =
    "Fractadyne runs headless for rendering, movie/tour export, automation, golden-image checks, \
     and benchmarking. With no flags it launches the interactive GUI.";

/// The single source of truth for the command line. Keep it complete — both the Help window and
/// `fractadyne --help` render from this list.
pub(crate) const CLI_REFERENCE: &[CliRef] = {
    use CliRef::{Example, Flag, Section};
    &[
        Section("Modes (pick one; otherwise the GUI launches)"),
        Flag("--render", "Render one image and exit."),
        Flag("--render-tour FILE", "Render a keyframe-tour TOML to a PNG frame sequence, then exit (see Tour options)."),
        Flag("--benchmark, --bench", "Run the benchmark with the current settings and exit (use --out to save the report)."),
        Flag("--benchmark-std [--res R] [--depth D] [--burnin N]", "Standardized benchmark: pinned resolution + settings, comparable across machines. --res = 720p | 1080p | 4k | 5k (default 1080p). --depth = standard (1e12x) | ultra (1e28x, hammers the deep-zoom path); --ultra is shorthand. --burnin N repeats it N times (default 10) to check stability / throttling."),
        Flag("--refdiag --center X Y --zoom-log2 L", "Dev: sample reference orbit lengths across a view."),
        Flag("--find-minibrot", "Print the nearby minibrot's period + center and exit (--center X Y --zoom M)."),
        Flag("--selftest [--bless]", "Run the GPU validation suite; exit 0 = all passed. --bless records the goldens."),
        Flag("--reset-state [--yes]", "Delete ALL saved state (session, bookmarks, thumbnails) and exit. Prompts for confirmation on the terminal (type 'reset'); --yes (or -y) skips the prompt."),
        Flag("--help, -h", "Print this command-line reference and exit."),
        Flag("@FILE, --args-file FILE", "Read arguments from a text file (whitespace-separated; # comments; \"quotes\" for spaces), spliced in place. Nestable — keep a whole command line in a file."),
        Section("Output"),
        Flag("--out PATH, -o PATH", "Output file (PNG/EXR), or the output directory for --render-tour."),
        Flag("--mp4 [PATH]", "With --render-tour: assemble the frames into an H.264 mp4 via ffmpeg (must be on PATH). Defaults to <out-dir>/tour.mp4; the PNG frames are kept."),
        Flag("--show-location, --hud", "Burn a zoom-level + center-coordinate HUD into the top-left of each rendered frame (--render / --render-tour)."),
        Section("Tour options (with --render-tour)"),
        Flag("--fps N", "Frames per second (default 30)."),
        Flag("--size W | WxH", "Frame size: bare width (height from aspect) or WIDTHxHEIGHT, e.g. 5120x2160."),
        Flag("--height H", "Explicit frame height (overrides the height implied by --size)."),
        Flag("--ss N", "Supersampling 1–8."),
        Flag("--prefix NAME", "Frame-name prefix -> NAME_00000.png (default: the tour script's file name)."),
        Flag("--overwrite, -y", "Replace existing frames without the [y]es/[a]ll/[n]o/[q]uit prompt."),
        Flag("--resume", "Restart an interrupted render: silently keep frames already on disk and render only the missing ones."),
        Section("View (with --render / --find-minibrot)"),
        Flag("--fractal NAME", "Family, e.g. \"Mandelbrot\" or \"Burning Ship\"."),
        Flag("--center X Y", "View center (full-precision decimals)."),
        Flag("--zoom M", "Magnification (f64, up to ~1e308x)."),
        Flag("--zoom-log2 L", "Magnification = 2^L — for depths past f64 range (>= ~1e308x)."),
        Flag("--julia", "Julia mode."),
        Flag("--julia-c RE IM", "Julia parameter c."),
        Section("Image & color (with --render)"),
        Flag("--size W | WxH", "Image size: bare width (height from aspect) or WIDTHxHEIGHT."),
        Flag("--ss N", "Supersampling 1–8."),
        Flag("--iter N", "Maximum iterations."),
        Flag("--palette N", "Preset palette index."),
        Flag("--method NAME", "smooth | stripe | triangle | trap | distance | decomposition."),
        Flag("--stripe-freq N", "Stripe density (stripe method)."),
        Flag("--trap SHAPE", "point | cross | circle (orbit-trap method)."),
        Flag("--binary", "Binary-decomposition coloring."),
        Flag("--light [--light-angle R]", "Enable 3D relief lighting."),
        Flag("--de", "Enable distance glow."),
        Flag("--watermark / --no-watermark", "Force the \"Fd\" watermark on / off (overrides the saved preference)."),
        Flag("--bla / --no-bla", "Force bilinear approximation (BLA) on / off for deep floatexp Mandelbrot."),
        Flag("--glitch / --no-glitch", "Force multi-reference glitch correction on / off for the export (default on)."),
        Flag("--no-perf / --perf", "Hide / show the performance panel."),
        Section("Validation & diagnostics"),
        Flag("--render-iter -o F.exr", "Export raw iteration data (EXR) instead of a colored image."),
        Flag("--compare A B", "Diff two renders/EXRs: max/mean delta + difference heatmap."),
        Flag("--import-kfr F.kfr", "Load a Kalles Fraktaler (.kfr) location."),
        Flag("--validate-deep", "Extreme-depth precision self-consistency battery (1e1000 … 1e1000000x)."),
        Flag("--profile [--reps N] [--regions F] [--out P]", "Dev: time render stages per benchmark region -> JSON log in logs/."),
        Flag("--crosscheck-f3 raw.exr", "Compare a Fraktaler-3 raw EXR (channel \"N\") against our CPU bignum oracle (--center X Y --zoom-f3 Z [--iter K] [--er R])."),
        Flag("--frametest [--steps N] [--hold N] [--dive R]", "Dev: deep-dive frame-timing harness."),
        Flag("--dump-tour-schema", "Print the tour-script (.toml) schema reference as Markdown and exit (generates TOURS.md)."),
        Section("Examples"),
        Example("fractadyne --render -o out.png --fractal Mandelbrot --center -0.743644 0.131826 --zoom 2e7 --iter 6000 --method stripe --ss 3"),
        Example("fractadyne --render-tour tours/deep-spiral-dive.toml --size 1920x1080 --fps 30 --ss 2 --out frames --mp4 --show-location"),
        Example("fractadyne @render.args        # read the whole command line from render.args"),
    ]
};

pub(crate) fn help_command_line(ui: &mut egui::Ui) {
    help_h(ui, "Command line");
    help_p(ui, CLI_INTRO);
    for entry in CLI_REFERENCE {
        match entry {
            CliRef::Section(s) => help_sub(ui, s),
            CliRef::Flag(f, d) => help_kv(ui, f, d),
            CliRef::Example(x) => {
                ui.label(egui::RichText::new(*x).monospace().small());
            }
        }
    }
}

/// The command-line reference as plain text, for the headless `fractadyne --help`. Built from the
/// same [`CLI_REFERENCE`] as the in-app Help window.
pub(crate) fn cli_help_text() -> String {
    let mut s = String::new();
    s.push_str(&format!("Fractadyne v{} — deep-zoom fractal explorer\n\n", version_string()));
    s.push_str("Usage: fractadyne [FLAGS]\n\n");
    s.push_str(CLI_INTRO);
    s.push('\n');
    for entry in CLI_REFERENCE {
        match entry {
            CliRef::Section(sec) => s.push_str(&format!("\n{sec}:\n")),
            CliRef::Flag(f, d) => s.push_str(&format!("  {f}\n      {d}\n")),
            CliRef::Example(x) => s.push_str(&format!("  {x}\n")),
        }
    }
    s
}

pub(crate) fn help_shortcuts(ui: &mut egui::Ui) {
    help_h(ui, "Shortcuts");
    help_sub(ui, "Mouse");
    help_kv(ui, "Left-drag", "Pan");
    help_kv(ui, "Wheel", "Zoom at cursor");
    help_kv(ui, "Shift+drag", "Box zoom (drag a rectangle to zoom to it)");
    help_sub(ui, "Keyboard");
    help_kv(ui, "Space / Shift+Space", "Continuous zoom in / out");
    help_kv(ui, "Backspace", "Undo view");
    help_kv(ui, "Shift+Backspace / Ctrl+Y", "Redo view");
    help_kv(ui, "M", "Find minibrot center");
    help_kv(ui, "A", "Auto-zoom autopilot (dive toward detail; any input stops)");
    help_kv(ui, "Esc", "Stop autopilot / a playing tour, or exit fullscreen");
    help_kv(ui, "Ctrl+S", "Quick export to the last folder");
    help_kv(ui, "F1 / ?", "Open this help");
}

/// A cited entry: bold title, a wrapped description, and a source link.
fn help_cite(ui: &mut egui::Ui, title: &str, body: &str, link_label: &str, url: &str) {
    ui.add_space(6.0);
    let ink = ui.visuals().strong_text_color();
    ui.label(egui::RichText::new(title).strong().color(ink));
    ui.add(egui::Label::new(body).wrap());
    ui.hyperlink_to(format!("{link_label} \u{2197}"), url);
}

pub(crate) fn help_hardware(ui: &mut egui::Ui) {
    help_h(ui, "Recommended hardware");
    help_p(
        ui,
        "Fractadyne splits its work between the GPU (per-pixel iteration + coloring) and the CPU \
         (the arbitrary-precision reference orbit that deep zoom is built on). Both matter, but \
         for different things.",
    );

    help_sub(ui, "GPU (most important)");
    help_p(
        ui,
        "Every pixel is iterated and colored on the GPU, so GPU speed sets the frame rate and \
         export time. A discrete GPU with strong 32-bit (single-precision) throughput gives the \
         smoothest deep zooming. Integrated GPUs work fine — just slower, and they reach the \
         per-frame work budget sooner (at which point the renderer trims resolution, not \
         detail). Requires a modern graphics API: Vulkan, Direct3D 12, Metal, or OpenGL via \
         wgpu.",
    );
    help_kv(ui, "Minimum", "Any GPU with Vulkan / DX12 / Metal / GL 3.3.");
    help_kv(ui, "Recommended", "A discrete GPU (last ~5 years); >= 2 GB VRAM.");
    help_kv(ui, "For big exports", ">= 4 GB VRAM (tiled, so it degrades gracefully).");

    help_sub(ui, "CPU");
    help_p(
        ui,
        "The high-precision reference orbit (and BLA tables) are computed on the CPU and are \
         largely single-threaded, so fast per-core performance matters most — it drives how \
         quickly a deep view resolves after you move. Extra cores mainly help background export \
         encoding. A 64-bit CPU is required.",
    );
    help_kv(ui, "Recommended", "A modern 64-bit CPU with good single-core speed.");

    help_sub(ui, "Memory");
    help_p(
        ui,
        "Deep references grow with depth (many iterations at high precision); huge exports hold \
         tiles in RAM. 8 GB is comfortable for everyday exploring; 16 GB+ suits extreme depth \
         and very large exports.",
    );

    help_sub(ui, "Notes");
    help_bullet(ui, "A capable GPU avoids the watchdog work-budget kicking in at extreme settings.");
    help_bullet(ui, "The build is single-binary and self-contained — no runtime install needed.");
    help_bullet(ui, "Run the built-in benchmark (Tools -> Benchmark) to gauge your machine.");
}

pub(crate) fn help_acknowledgments(ui: &mut egui::Ui) {
    help_h(ui, "Acknowledgments & citations");
    // Dedication.
    let accent = crate::theme::ui_accent(ui.ctx());
    ui.label(
        egui::RichText::new(
            "Dedicated to the Stone Soup Group — the volunteers behind Fractint (from 1988).",
        )
        .italics()
        .color(accent),
    );
    help_p(
        ui,
        "Their freely shared code and spirit of open, collaborative fractal exploration are the \
         reason so many of us fell in love with this art. Fractadyne is built in that tradition.",
    );
    help_p(
        ui,
        "Fractadyne stands on decades of work by the fractal community. Techniques and prior art \
         it builds on — verified against the sources below:",
    );

    help_sub(ui, "Deep-zoom algorithms");
    help_cite(
        ui,
        "Perturbation & series approximation — K. I. Martin",
        "The reference-orbit + low-precision-delta method (δz → 2Z·δz + δz² + δc) and the order-n \
         series that skips early iterations, introduced in SuperFractalThing and its note \
         “SuperFractalThing Maths” (sft_maths.pdf, 2013).",
        "Perturbation theory — Fractal Wiki",
        "https://fractalwiki.org/wiki/Perturbation_theory",
    );
    help_cite(
        ui,
        "Bivariate Linear Approximation (BLA) & rebasing — Zhuoran",
        "Skipping runs of iterations via merged linear maps, and rebasing to a single reference to \
         avoid glitches; developed on fractalforums.org (2021).",
        "BLA explained — Phil Thompson",
        "https://philthompson.me/2023/Faster-Mandelbrot-Set-Rendering-with-BLA-Bivariate-Linear-Approximation.html",
    );
    help_cite(
        ui,
        "Glitch-detection criterion — Pauldelbrot",
        "The |Z + z| << |Z| test for when a pixel needs a different reference — the basis of \
         glitch-free perturbation (fractalforums).",
        "Deep zoom theory & practice — mathr",
        "https://mathr.co.uk/blog/2021-05-14_deep_zoom_theory_and_practice.html",
    );
    help_cite(
        ui,
        "Non-analytic (Burning Ship) perturbation — laser blaster",
        "The absolute-value “diffabs” case analysis that makes perturbation work for the \
         abs-based families (Burning Ship / Celtic / Buffalo), on fractalforums.",
        "Deep zoom theory & practice — mathr",
        "https://mathr.co.uk/blog/2021-05-14_deep_zoom_theory_and_practice.html",
    );
    help_cite(
        ui,
        "Reference implementations & cross-checks — Claude Heiland-Allen (mathr)",
        "Fraktaler-3 and Kalles Fraktaler 2+ (originally by Karl Runmo) — whose write-ups guided \
         this engine, and whose EXR output Fractadyne cross-checks its renderer against.",
        "Fraktaler-3",
        "https://fraktaler.mathr.co.uk/",
    );

    help_sub(ui, "Coloring");
    help_cite(
        ui,
        "Smooth iteration count & Stripe Average — Jussi Härkönen",
        "The continuous (normalized) escape-time coloring and the Stripe Average method, from \
         “On Smooth Fractal Coloring Techniques” (M.Sc. thesis, 2007), which also analyzes the \
         Triangle Inequality / branching-average colorings.",
        "On Smooth Fractal Coloring Techniques (2007)",
        "https://archive.org/details/j-harkonen-on-smooth-fractal-coloring-techniques-masters-thesis-2007-hi-res",
    );
    help_cite(
        ui,
        "Triangle Inequality Average — Kerry Mitchell",
        "Averaging where each z² + c lands within the triangle-inequality bounds, for smooth \
         flame-like exterior texture.",
        "Kerry Mitchell — tutorials",
        "http://www.kerrymitchellart.com/tutorials/formulas2/uf2-2.htm",
    );
    help_p(
        ui,
        "The distance-estimate coloring/lighting uses the classic dz/dc boundary-distance method \
         (see Peitgen & Saupe, “The Science of Fractal Images”, 1988).",
    );

    help_sub(ui, "Foundations & tools");
    help_cite(
        ui,
        "The Mandelbrot set — Benoit B. Mandelbrot",
        "The object at the heart of it all.",
        "Fractint — the Stone Soup Group",
        "https://www.fractint.org/",
    );
    help_p(
        ui,
        "Built with Rust, egui/eframe (emilk) and wgpu (gfx-rs); arbitrary precision via \
         astro-float, double-word floats via twofloat; PNG/OpenEXR, rfd, serde. Thanks to their \
         authors and maintainers.",
    );
    help_p(
        ui,
        "Any errors in attribution are the author's; corrections are welcome on the project's \
         issue tracker.",
    );
}

/// Full third-party dependency license notices — reproduced verbatim (the MIT / BSD / Apache /
/// Zlib / Unicode / font licenses require this in binary distributions). Generated by
/// `cargo-about`; the same text ships as `THIRD-PARTY-NOTICES.md` with the download.
pub(crate) fn help_licenses(ui: &mut egui::Ui) {
    // Notices generated at the repo root (see about.toml / about.hbs).
    const NOTICES: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../THIRD-PARTY-NOTICES.md"));
    help_h(ui, "Third-party licenses");
    help_p(
        ui,
        "Fractadyne is distributed as a statically-linked binary that includes the open-source \
         crates and bundled fonts below. The full license texts (also in THIRD-PARTY-NOTICES.md, \
         included with the download) follow. Fractadyne's own code is dual-licensed MIT OR \
         Apache-2.0. `option-ext` is MPL-2.0 — its unmodified source is at \
         https://crates.io/crates/option-ext.",
    );
    ui.add_space(6.0);
    if ui.button("📋 Copy all notices").clicked() {
        ui.ctx().copy_text(NOTICES.to_string());
    }
    ui.add_space(6.0);
    // The full notices in a bounded, scrollable monospace pane. No wrap (long license lines scroll
    // horizontally rather than overlap); egui caches the galley, so the one-time layout of this
    // rarely-opened pane is a non-issue.
    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::ScrollArea::both()
            .max_height(380.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.add(egui::Label::new(egui::RichText::new(NOTICES).monospace()));
            });
    });
}

pub(crate) fn help_about(ui: &mut egui::Ui) {
    help_h(ui, "About");
    help_p(ui, &format!("Fractadyne v{}", version_string()));
    help_p(ui, "A native fractal explorer built in Rust with wgpu.");

    help_sub(ui, "Files & data");
    help_p(
        ui,
        "Your session (current view, coloring, and preferences), bookmarks, and their \
         thumbnails are stored in this per-user folder, and reloaded on the next launch:",
    );
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(fractadyne_state::state_location_display())
                .monospace()
                .small(),
        );
        if ui.small_button("📋").on_hover_text("Copy path").clicked() {
            ui.ctx().copy_text(fractadyne_state::state_location_display());
        }
    });
    help_p(
        ui,
        "The session file is versioned; if it was written by a newer Fractadyne than can fully \
         read it, the app loads it best-effort and warns you. To start completely fresh, use \
         File ▸ Reset application state… (asks first), or run `fractadyne --reset-state` from a \
         terminal.",
    );

    help_sub(ui, "License");
    help_p(ui, "MIT OR Apache-2.0 — use under either license, at your option.");
    help_p(ui, "© 2026 Rithea Hong.");
    ui.hyperlink_to("Source on GitHub \u{2197}", "https://github.com/WindySnowOwl/fractadyne");
}
