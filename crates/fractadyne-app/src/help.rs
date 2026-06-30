//! In-app Help content (the `Help → Help & reference…` window, F1).
//!
//! Pure presentation: each `help_*` section renders into an `egui::Ui`. The window chrome and
//! table-of-contents live in `FractadyneApp::help_window`; this module is just the text.

use crate::{version_string, BRAND_ACCENT, BRAND_TEXT};
use eframe::egui;

fn help_h(ui: &mut egui::Ui, t: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(t).size(18.0).strong().color(BRAND_ACCENT));
    ui.add_space(4.0);
}
fn help_sub(ui: &mut egui::Ui, t: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(t).strong().color(BRAND_TEXT));
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
/// A monospace key + wrapped description row (shortcuts / CLI flags).
fn help_kv(ui: &mut egui::Ui, k: &str, v: &str) {
    ui.horizontal_top(|ui| {
        ui.add_sized(
            [180.0, 0.0],
            egui::Label::new(egui::RichText::new(k).monospace()).wrap(),
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
    help_bullet(ui, "Switch between ten fractal families, and view any as a Julia set.");
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
    help_kv(ui, "Right-drag", "Box zoom — drag a rectangle to zoom into it.");
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
        "z → zᵈ + c for power d = 3, 4, 5. Higher powers add lobes: the set has (d−1)-fold \
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
        "z → z² + c + p·z₋₁, where z₋₁ is the previous iterate and p is a constant (here p = −0.5). \
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
        "Mandelbrot, Multibrot 3/4/5 and Tricorn support unlimited (floatexp) perturbation deep \
         zoom. Burning Ship, Celtic and Buffalo are non-analytic (they take absolute values), so \
         they use a sign-aware perturbation; this now runs at floatexp range too, deep-zooming far \
         past the old ~10²⁸× df32 limit (rare speckle near the abs folds awaits multi-reference \
         glitch correction). Phoenix and Newton currently use the direct path, sharp to ~10⁶×.",
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
        "Ordinary 64-bit numbers run out of digits near 10¹⁵× zoom. Fractadyne keeps the view center \
         in arbitrary precision, with the number of digits growing as you zoom, so the location never \
         degrades. The pixel scale is likewise held with an extended exponent, so it doesn't stall at \
         64-bit's ~10³⁰⁸× limit either — depth is bounded only by patience, not by any fixed number \
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
    help_sub(ui, "Unlimited depth (floatexp)");
    help_p(
        ui,
        "Past about 10²⁸× even that tiny difference underflows 32-bit range, so it is stored as a \
         mantissa plus a separate integer exponent (\"floatexp\"), removing the depth wall. The \
         engine switches automatically: direct math when shallow, perturbation when deep, and \
         floatexp when deepest.",
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
         Mandelbrot, deepest range, non-stripe coloring for now.",
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

pub(crate) fn help_command_line(ui: &mut egui::Ui) {
    help_h(ui, "Command line");
    help_p(
        ui,
        "Fractadyne can run headless for automation, golden-image checks, and benchmarking. Flags:",
    );
    help_sub(ui, "Modes");
    help_kv(ui, "--render", "Render one image and exit.");
    help_kv(ui, "--out PATH, -o PATH", "Output file (PNG/EXR), or output dir for --render-tour.");
    help_kv(
        ui,
        "--render-tour FILE",
        "Render a keyframe tour (TOML) to a PNG frame sequence, then exit. Options: \
         --fps N (default 30), --size W, --height H, --ss N, --out DIR (default \"frames\"). \
         Assemble with ffmpeg. See scripts/tour.example.toml.",
    );
    help_kv(ui, "--benchmark, --bench", "Run the benchmark tour and exit (use --out to save).");
    help_kv(ui, "--find-minibrot", "Print the nearby minibrot's period + center and exit.");
    help_sub(ui, "View");
    help_kv(ui, "--fractal NAME", "Family, e.g. \"Mandelbrot\" or \"Burning Ship\".");
    help_kv(ui, "--center X Y", "View center (full-precision decimals).");
    help_kv(ui, "--zoom M", "Magnification (f64, ≤ ~1e308×).");
    help_kv(ui, "--zoom-log2 L", "Magnification = 2^L — for depths past f64 range (≥ ~1e308×).");
    help_kv(ui, "--julia", "Julia mode.");
    help_kv(ui, "--julia-c RE IM", "Julia parameter c.");
    help_sub(ui, "Image & color");
    help_kv(ui, "--size W", "Image width in pixels (height from aspect).");
    help_kv(ui, "--ss N", "Supersampling 1–8.");
    help_kv(ui, "--iter N", "Maximum iterations.");
    help_kv(ui, "--palette N", "Preset palette index.");
    help_kv(ui, "--method NAME", "smooth | stripe | triangle | trap | distance | decomposition.");
    help_kv(ui, "--stripe-freq N", "Stripe density (stripe method).");
    help_kv(ui, "--trap SHAPE", "point | cross | circle (orbit-trap method).");
    help_kv(ui, "--light [--light-angle R]", "Enable 3D relief lighting.");
    help_kv(ui, "--de", "Enable distance glow.");
    help_kv(ui, "--no-perf / --perf", "Hide / show the performance panel.");
    help_sub(ui, "Validation");
    help_kv(ui, "--selftest [--bless]", "Run the validation suite; exit 0 = all passed (--bless records goldens).");
    help_kv(ui, "--render-iter -o F.exr", "Export raw iteration data (EXR) instead of a colored image.");
    help_kv(ui, "--compare A B", "Diff two renders/EXRs: max/mean Δ + difference heatmap.");
    help_kv(ui, "--import-kfr F.kfr", "Load a Kalles Fraktaler location.");
    help_kv(
        ui,
        "--validate-deep",
        "Extreme-depth precision self-consistency battery (1e1000…1e1000000×).",
    );
    help_kv(
        ui,
        "--profile [--reps N] [--regions F] [--out P]",
        "Dev profiling: time render stages per benchmark region → JSON log in logs/.",
    );
    help_kv(
        ui,
        "--crosscheck-f3 raw.exr",
        "Compare a Fraktaler-3 raw EXR (channel \"N\") against our CPU bignum oracle \
         (--center X Y --zoom-f3 Z [--iter K] [--er R]).",
    );
    help_sub(ui, "Example");
    ui.label(
        egui::RichText::new(
            "fractadyne --render -o out.png --fractal Mandelbrot \\\n  \
             --center -0.743644 0.131826 --zoom 2e7 --iter 6000 --method stripe --ss 3",
        )
        .monospace()
        .small(),
    );
}

pub(crate) fn help_shortcuts(ui: &mut egui::Ui) {
    help_h(ui, "Shortcuts");
    help_sub(ui, "Mouse");
    help_kv(ui, "Left-drag", "Pan");
    help_kv(ui, "Wheel", "Zoom at cursor");
    help_kv(ui, "Right-drag", "Box zoom");
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

pub(crate) fn help_about(ui: &mut egui::Ui) {
    help_h(ui, "About");
    help_p(ui, &format!("Fractadyne v{}", version_string()));
    help_p(ui, "A native fractal explorer built in Rust with wgpu.");
    help_sub(ui, "License");
    help_p(ui, "MIT OR Apache-2.0 — use under either license, at your option.");
    help_p(ui, "© 2026 Rithea Hong.");
    ui.hyperlink_to("Source on GitHub \u{2197}", "https://github.com/WindySnowOwl/fractadyne");
}
