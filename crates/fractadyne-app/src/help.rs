//! In-app Help content (the `Help → Help & reference…` window, F1).
//!
//! Pure presentation: each `help_*` section renders into an `egui::Ui`. The window chrome and
//! table-of-contents live in `FractadyneApp::help_window`; this module is just the text.
//!
//! The text is in the language in use (`tr!`, the `help-*` messages of `i18n/en/fractadyne.ftl`):
//! one message per paragraph, list item or heading. The command-line reference ([`CLI_REFERENCE`])
//! stays English, as the command line does; names, formulas and expressions stay as written.

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
    help_h(ui, &tr!(help_section_overview));
    help_p(ui, &tr!(help_overview_intro));
    help_sub(ui, &tr!(help_overview_what_is));
    help_p(ui, &tr!(help_overview_what_is_text));
    help_sub(ui, &tr!(help_overview_can_do));
    help_bullet(ui, &tr!(help_overview_can_pan));
    help_bullet(ui, &tr!(help_overview_can_families));
    help_bullet(ui, &tr!(help_overview_can_recolor));
    help_bullet(ui, &tr!(help_overview_can_lighting));
    help_bullet(ui, &tr!(help_overview_can_minibrots));
    help_bullet(ui, &tr!(help_overview_can_autopilot));
    help_bullet(ui, &tr!(help_overview_can_share));
    help_bullet(ui, &tr!(help_overview_can_tours));
    help_sub(ui, &tr!(help_overview_first_steps));
    help_p(ui, &tr!(help_overview_first_steps_text));
}

/// The reference for the coordinate grammar. The function and constant tables are rendered from
/// `fractadyne_core::EXPR_FUNCTIONS` / `EXPR_CONSTANTS` — the same tables the evaluator's error
/// messages quote and a core test holds equal to what it accepts — so this page cannot list a
/// function the parser refuses or omit one it takes. Only the prose and examples live here.
pub(crate) fn help_expressions(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_expressions));
    help_p(ui, &tr!(help_expr_intro));

    help_sub(ui, &tr!(help_expr_examples));
    help_kv(ui, "1/3", &tr!(help_expr_ex_third));
    help_kv(ui, "-3/4", &tr!(help_expr_ex_neck));
    help_kv(ui, "-0.5 + 0.25*cos(pi/4)", &tr!(help_expr_ex_circle));
    help_kv(ui, "cos(2*pi/3)/2 - cos(4*pi/3)/4", &tr!(help_expr_ex_bulb));
    help_kv(ui, "(37+16i)/100", &tr!(help_expr_ex_complex));
    help_kv(ui, "sqrt(2)/2", &tr!(help_expr_ex_combine));
    help_kv(ui, "2^-20", &tr!(help_expr_ex_powers));
    help_kv(ui, "root(-8, 3)", &tr!(help_expr_ex_root));
    help_kv(ui, "1e-30", &tr!(help_expr_ex_scientific));

    help_sub(ui, &tr!(help_expr_numbers));
    help_bullet(ui, &tr!(help_expr_numbers_decimals));
    help_bullet(ui, &tr!(help_expr_numbers_imaginary));
    help_bullet(ui, &tr!(help_expr_numbers_operators));
    help_bullet(ui, &tr!(help_expr_numbers_case));

    help_sub(ui, &tr!(help_expr_constants));
    for (name, what) in fractadyne_core::EXPR_CONSTANTS {
        help_kv(ui, name, what);
    }

    help_sub(ui, &tr!(help_expr_functions));
    help_p(ui, &tr!(help_expr_functions_angles));
    for (_, sig, what) in fractadyne_core::EXPR_FUNCTIONS {
        help_kv(ui, sig, what);
    }

    help_sub(ui, &tr!(help_expr_refused));
    help_bullet(ui, &tr!(help_expr_refused_fractional));
    help_bullet(ui, &tr!(help_expr_refused_complex));
    help_bullet(ui, &tr!(help_expr_refused_domain));
    help_bullet(ui, &tr!(help_expr_refused_bounds));
    help_bullet(ui, &tr!(help_expr_refused_grammar));

    help_sub(ui, &tr!(help_expr_no_parse));
    help_p(ui, &tr!(help_expr_no_parse_text));
}

pub(crate) fn help_navigation(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_navigation));
    help_sub(ui, &tr!(help_mouse));
    help_kv(ui, &tr!(help_key_left_drag), &tr!(help_nav_left_drag));
    help_kv(ui, &tr!(help_key_mouse_wheel), &tr!(help_nav_wheel));
    help_kv(ui, &tr!(help_key_shift_drag), &tr!(help_nav_shift_drag));
    help_kv(ui, &tr!(help_key_click_zoom, icon = "🎯"), &tr!(help_nav_click_zoom));
    help_sub(ui, &tr!(help_nav_continuous));
    help_kv(ui, &tr!(help_key_hold_space), &tr!(help_nav_hold_space));
    help_kv(ui, &tr!(help_key_hold_shift_space), &tr!(help_nav_hold_shift_space));
    help_p(ui, &tr!(help_nav_continuous_text));
    help_sub(ui, &tr!(help_nav_autopilot));
    help_p(ui, &tr!(help_nav_autopilot_text));
    help_sub(ui, &tr!(help_nav_history));
    help_kv(ui, "Ctrl+Z / Backspace", &tr!(help_nav_undo));
    help_kv(ui, "Ctrl+Y / Shift+Backspace", &tr!(help_nav_redo));
    help_p(ui, &tr!(help_nav_goto));
    help_p(ui, &tr!(help_nav_exact_points));
    help_p(ui, &tr!(help_nav_share));
    help_p(ui, &tr!(help_nav_open));
    help_sub(ui, &tr!(help_nav_finding));
    help_p(ui, &tr!(help_nav_minibrots));
    help_p(ui, &tr!(help_nav_minimap));
}

pub(crate) fn help_options(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_options));
    help_p(ui, &tr!(help_options_intro));
    help_sub(ui, &tr!(help_options_palette));
    help_p(ui, &tr!(help_options_palette_text));
    help_p(ui, &tr!(help_options_paste));
    help_p(ui, &tr!(help_options_import_map));
    help_p(ui, &tr!(help_options_duotone));
    help_p(ui, &tr!(help_options_cycle));
    help_p(ui, &tr!(help_options_normalize));
    help_p(ui, &tr!(help_options_log_scale));
    help_p(ui, &tr!(help_options_animate));
    help_sub(ui, &tr!(help_options_method));
    help_kv(ui, &tr!(help_method_smooth), &tr!(help_method_smooth_text));
    help_kv(ui, &tr!(help_method_stripe), &tr!(help_method_stripe_text));
    help_kv(ui, &tr!(help_method_triangle), &tr!(help_method_triangle_text));
    help_kv(ui, &tr!(help_method_trap), &tr!(help_method_trap_text));
    help_kv(ui, &tr!(help_method_distance), &tr!(help_method_distance_text));
    help_kv(ui, &tr!(help_method_decomposition), &tr!(help_method_decomposition_text));
    help_sub(ui, &tr!(help_options_relief));
    help_p(ui, &tr!(help_options_relief_text));
    help_sub(ui, &tr!(help_options_glow));
    help_p(ui, &tr!(help_options_glow_text));
    help_sub(ui, &tr!(help_options_quality));
    help_p(ui, &tr!(help_options_quality_text));
    help_sub(ui, &tr!(help_options_other));
    help_p(ui, &tr!(help_options_other_text));
}

pub(crate) fn help_fractals(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_fractals));
    help_p(ui, &tr!(help_fractals_intro));
    help_sub(ui, "Mandelbrot");
    help_p(ui, &tr!(help_fractals_mandelbrot));
    help_sub(ui, "Multibrot 3 / 4 / 5");
    help_p(ui, &tr!(help_fractals_multibrot));
    help_sub(ui, "Tricorn (Mandelbar)");
    help_p(ui, &tr!(help_fractals_tricorn));
    help_sub(ui, "Burning Ship");
    help_p(ui, &tr!(help_fractals_burning_ship));
    help_sub(ui, "Celtic");
    help_p(ui, &tr!(help_fractals_celtic));
    help_sub(ui, "Buffalo");
    help_p(ui, &tr!(help_fractals_buffalo));
    help_sub(ui, &tr!(help_fractals_powers));
    help_p(ui, &tr!(help_fractals_powers_text));
    help_sub(ui, "Phoenix");
    help_p(ui, &tr!(help_fractals_phoenix));
    help_sub(ui, "Newton");
    help_p(ui, &tr!(help_fractals_newton));
    help_sub(ui, &tr!(help_fractals_julia));
    help_p(ui, &tr!(help_fractals_julia_text));
    help_sub(ui, &tr!(help_fractals_deep));
    help_p(ui, &tr!(help_fractals_deep_text));
    help_sub(ui, &tr!(help_fractals_custom));
    help_p(ui, &tr!(help_fractals_custom_text));
    help_p(ui, &tr!(help_fractals_custom_sections));
    help_p(ui, &tr!(help_fractals_custom_editor));
    help_p(ui, &tr!(help_fractals_custom_textbook));
    help_p(ui, &tr!(help_fractals_custom_numbers));
    help_p(ui, &tr!(help_fractals_custom_library));
    help_p(ui, &tr!(help_fractals_custom_collection));
}

pub(crate) fn help_methodology(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_how_it_works));
    help_sub(ui, &tr!(help_how_escape));
    help_p(ui, &tr!(help_how_escape_text));
    help_sub(ui, &tr!(help_how_precision));
    help_p(ui, &tr!(help_how_precision_text));
    help_sub(ui, &tr!(help_how_perturbation));
    help_p(ui, &tr!(help_how_perturbation_text));
    help_sub(ui, &tr!(help_how_floatexp));
    help_p(ui, &tr!(help_how_floatexp_text));
    help_sub(ui, &tr!(help_how_reference));
    help_p(ui, &tr!(help_how_reference_text));
    help_sub(ui, &tr!(help_how_series));
    help_p(ui, &tr!(help_how_series_text));
    help_sub(ui, &tr!(help_how_distance));
    help_p(ui, &tr!(help_how_distance_text));
    help_sub(ui, &tr!(help_how_safety));
    help_p(ui, &tr!(help_how_safety_text));
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
        Flag("--render-expmap DIR", "Render a straight dive into the view (--center, --zoom, --size) as exponential-map strips, one octave of depth each, into DIR with a manifest (expmap.json), then exit: the raw material of a zoom video (see Zoom-video strips). Strips already in DIR are kept."),
        Flag("--assemble DIR --out F.mp4", "Assemble a zoom video from the strips --render-expmap rendered into DIR, coloured with the current colouring, and encode it with ffmpeg (FRACTADYNE_FFMPEG or PATH) into F (.mp4, .mkv or .mov), then exit: the whole dive at a constant zoom speed, at the strips' frame size (see Zoom-video assembly). Smooth, distance, relief and glow colourings; not stripe, TIA, trap or decomposition, whose statistics a strip does not keep."),
        Flag("--render-zoom-video F", "Render the dive into the view (--center, --zoom, --size; the Zoom-video strips options) and assemble it into the video F in one run, then exit: each strip is rendered just before the first frame that shows it, so no strips are written to disk (unless --keep-strips DIR). Coloured with the current colouring; the Zoom-video assembly options set the speed, frame rate and samples a pixel."),
        Flag("--benchmark, --bench", "Run the benchmark with the current settings and exit (use --out to save the report)."),
        Flag("--benchmark-std [--res R] [--depth D] [--burnin N]", "Standardized benchmark: pinned resolution + settings, comparable across machines. --res = 720p | 1080p | 4k | 5k (default 1080p). --depth = standard (1e32x) | ultra (1e48x); --ultra is shorthand. BOTH cross into the floatexp/BLA path, and the report breaks the dive down per arithmetic regime (frames, avg ms, share). --burnin N repeats it N times (default 10) to check stability / throttling."),
        Flag("--refdiag --center X Y --zoom-log2 L", "Dev: sample reference orbit lengths across a view."),
        Flag("--pickcheck [FILE] [--size WxH] [--iter N]", "Dev: reference-pick acceptance harness - run BOTH phase-2 scoring engines (bignum walk vs floatexp perturbation) on the built-in depth ladder (or one .fdn) and assert the same winner. Exit 0 all matched, 1 mismatch, 2 skipped/none."),
        Flag("--shot LOC [--out P] [--size WxH] [--shot-timeout S] [--svg P] [--image P]", "Regenerate the published screenshot: load a location (.fdn or an exported view), turn on dual view, the minimap and the control panel, wait for the reference orbit to finish building, capture the window to a PNG and exit. Deterministic - palette and light animation are pinned off. Default 1920x1200, 180s budget. With --svg, an L-system view is also written there as an SVG drawing (File > Export SVG); with --image, the view's image export is written there at the Export dialog's settings (File > Export image)."),
        Flag("--find-minibrot [--period-search ball] [--max-period N]", "Print the nearby minibrot's period + center and exit (--center X Y --zoom M). --period-search ball asks for the lowest-period minibrot at or near the view (ball arithmetic: it can answer one a few view widths outside), for a view on a deep zoom path whose centre still sits in a shallower minibrot's atom domain (the default search names that one, and it has left the view); --max-period raises the period cap (default 100,000)."),
        Flag("--version, -V", "Print the version and build number, then exit."),
        Flag("--check-updates [stable|beta]", "Check GitHub for a newer release on the given track (default stable), print the result, and exit (needs network)."),
        Flag("--selftest [--bless [--bless-new-adapter]]", "Run the GPU validation suite; exit 0 = all passed. --bless records the goldens: on the reference card (the one validation/golden/BLESSED-GPU.txt names) into validation/golden/, on any other card into that card's own set, validation/golden/<card>/, which the suite then compares it against strictly; --bless-new-adapter instead moves the reference set to this card. A full run ends by naming the opt-in groups it did not run (--selftest-list shows them)."),
        Flag("--selftest-filter SUBSTR", "With --selftest: run only the check groups / goldens whose tag or name contains SUBSTR (see --selftest-list). For fast iteration on one failure; groups share state, so use a full run for the real verdict."),
        Flag("--selftest-list", "Print the selftest group tags (for --selftest-filter) and exit."),
        Flag("--torture [SELECTOR ...]", "Run the escalating torture ladder: each rung is launched as a separate process under a deadline, so a rung that loses the device or hangs is RECORDED rather than taking the run down. SELECTOR targets a lane, family or exact rung id (e.g. `live`, `live/tour`, `live/tour/grand`); omit for everything. Failures do not stop the run — dependents of a failed rung report as blocked while unrelated families keep going — so one pass can surface several distinct defects."),
        Flag("--stop-at-first", "With --torture: stop at the first failing rung instead of continuing. For bisecting a known failure, where everything after it is noise."),
        Flag("--torture-out DIR", "With --torture: where failure reports, per-rung logs and throwaway config dirs are written (default validation/torture)."),
        Flag("--reset-state [--yes]", "Delete ALL saved state (session, bookmarks, thumbnails) and exit. Prompts for confirmation on the terminal (type 'reset'); --yes (or -y) skips the prompt."),
        Flag("--help, -h", "Print this command-line reference and exit."),
        Flag("@FILE, --args-file FILE (or --args)", "Read arguments from a text file (whitespace-separated; # comments; \"quotes\" for spaces), spliced in place. Nestable — keep a whole command line in a file."),
        Section("Output"),
        Flag("--out PATH, -o PATH", "Output file (PNG/EXR), or the output directory for --render-tour."),
        Flag("--mp4 [PATH]", "With --render-tour: assemble the frames into an H.264 mp4 via ffmpeg (must be on PATH). Defaults to <out-dir>/tour.mp4; the PNG frames are kept."),
        Flag("--show-location, --hud", "Burn a zoom-level + center-coordinate HUD into the top-left of each rendered frame (--render / --render-tour)."),
        Section("Zoom-video strips (with --render-expmap)"),
        Flag("--expmap-start L", "Where the dive starts, as log2 of the magnification (default 0 = 1x). It ends at the view's zoom, and the strips go on about log2(diagonal) octaves further, to the last frame's centre."),
        Flag("--expmap-oversample Q", "Strip texels per frame pixel at the frame's corners (default 1; more everywhere else). The strip's width is ceil(Q x pi x the frame's diagonal)."),
        Flag("--expmap-cap rising|fixed", "The iteration cap over the dive: rising (the default) gives each strip what its depth wants, never less than the strip before; fixed gives every strip the deepest strip's cap. With --iter N, the cap is N for every strip (fixed) or the base the rising caps grow from (rising)."),
        Flag("--expmap-strips A-B", "Render only strips A to B (0 is the outermost). Every run uses the dive's one reference, built for its deepest strip, so strips rendered in pieces are the strips one run renders."),
        Flag("--expmap-plan", "Write the dive's manifest (DIR/expmap.json: its strips, their size and iteration caps) and stop, rendering nothing — or check the manifest already in DIR. How a farm's zoom video is planned once for every machine."),
        Flag("--expmap-format fractadyne|zoomasm", "fractadyne (the default) writes Fractadyne's own strips. zoomasm writes Kalles Fraktaler's EXR strips (N, NF, T, DEX, DEY; one octave each), which zoomasm assembles into a video: open DIR as zoomasm's input, with Reverse and Flip unticked (strip 0 is the outermost)."),
        Section("Zoom-video assembly (with --assemble or --render-zoom-video)"),
        Flag("--assemble-at L[,L...]", "Instead of a video, assemble single frames at these log2 magnifications, as PNGs: into --out (a file for one frame, a folder for several)."),
        Flag("--assemble-fps N", "Frames a second (default 30)."),
        Flag("--assemble-speed X", "Octaves of zoom a second (default 1): the video lasts (end - start) / X seconds."),
        Flag("--assemble-tour T.toml", "Follow tour T's timeline instead of a constant speed: its keyframes' zooms, times, holds and easing, at its frame rate (unless --assemble-fps). Every frame must be on the dive's centre and inside its depths: a strip video cannot pan, so start the tour where its last pan ends."),
        Flag("--assemble-rotate R", "Turn the picture R turns an octave of zoom as the video dives (the plane turns clockwise on screen for positive R; relief lighting stays fixed to the screen)."),
        Flag("--assemble-crf N", "H.264 quality, 0-51 (default 18, visually near-lossless; 0 is lossless, and large). Lower is better and bigger; 6 more halves the size, roughly."),
        Flag("--keep-strips DIR", "With --render-zoom-video: write the strips to DIR as well (Fractadyne's format), so --assemble DIR can recolour the video later without rendering again."),
        Flag("--farm", "With --render-zoom-video F (and --center RE IM, --zoom-log2 L, --size WxH): render the dive's strips on a render farm, then assemble the video on this machine. This process is the farm's controller, with --farm-render's options (--listen, --min-clients, --local, --share-root, --sharing, --strikes, --stall, …); machines join with --render-client or File > Render client…. The dive is planned once here (--expmap-plan) and every machine checks its own plan against it; each strip a machine returns is checked whole (its SHA-256, every channel, its own strip number and iteration cap) before it is kept. The strips go to --keep-strips DIR, else to a folder named for the video beside it, emptied once the video is made; the job's state is in its farm/ folder, so running the same command again resumes it. Exit 0 complete, 1 the strips are done but the video could not be assembled (--assemble DIR makes it), 3 strips given up, 4 stopped."),
        Flag("--assemble-ss N", "Samples a pixel along each axis, 1-8 (default: the strips' --expmap-oversample). More than the strips were oversampled repeats texels near the frame's corners."),
        Section("Tour options (with --render-tour)"),
        Flag("(defaults)", "Every option below falls back to the script's [render] block, so a tour renders as authored with no flags at all; a flag given here overrides it."),
        Flag("--fps N", "Frames per second (default 30)."),
        Flag("--size W | WxH", "Frame size: bare width (height from aspect) or WIDTHxHEIGHT, e.g. 5120x2160."),
        Flag("--height H", "Explicit frame height (overrides the height implied by --size)."),
        Flag("--ss N", "Supersampling 1–8."),
        Flag("--prefix NAME", "Frame-name prefix -> NAME_00000.png (default: the tour script's file name)."),
        Flag("--segment NAME", "Render only one [[segment]] chapter (by id, unique prefix, or 1-based index), keeping the global frame numbering so its frames drop back into the full sequence."),
        Flag("--overwrite, -y", "Replace existing frames without the [y]es/[a]ll/[n]o/[q]uit prompt."),
        Flag("--gpus all | N,M,…", "With --render: split the still by tile across the window's graphics card and the cards named (all = every Vulkan card; numbers from --list-adapters; the window's own number once stands for the window, twice adds a second device on it), each with its own copy of the reference; a card that fails hands its tile back; the log says who rendered what. With the step-bounded deep path (mode 2) the tiles are a single card's, so identical cards give that card's image exactly; different models draw a few pixels differently, tile by tile. A render the split cannot take (aux colouring, a formula without resumable passes) uses the window's card. With --render-tour: render on several graphics cards of this machine — all = every card (Vulkan), or numbers from --list-adapters. It runs a render farm on this machine (a --farm-render controller on loopback and a --render-client with one session per card), so the frames, their names, the progress lines, render-status.txt and --mp4 are those of a single-card render; a held shot and a dissolve stay on one card. With one card it renders as usual. The render settings are this machine's saved ones, so --watermark, --show-location, --bla and --set are refused with it, as are --segment(s), --frames, --order and --dry-run. Different card models draw a few pixels slightly differently. With --render-zoom-video: render the strips on several cards the same way (--farm on loopback), then assemble the video here; --bla / --no-bla are kept, being part of the dive."),
        Flag("--resume", "Restart an interrupted render: keep frames already on disk and render only the missing ones. The newest frame is verified first and discarded if incomplete (a render dies on the frame it is writing), stepping back until a good one is found; a folder holding frames at a different size is refused rather than mixed into the sequence."),
        Section("Render farm (design/remote-rendering.md)"),
        Flag("--farm-render TOUR --out DIR [--listen ADDR:PORT] [--min-clients N] [--local]", "Render a tour on several machines. Listens (default 0.0.0.0:46733) for render clients, admits each only after the farm-key handshake, an exact version-and-commit match and the client's self-check, waits for --min-clients (default 1), measures a normalized tour's palette anchors once for all of them, and hands out runs of frames until every frame is verified (SHA-256, PNG structure, size) or given up. Frames, and the job's state in DIR/farm/ (done log, events, status, metrics, quarantined bad frames, diagnostics), resume from the folder alone: run the same command again. --size/--fps/--ss/--prefix override the script's [render] block as for --render-tour. --local also renders on this machine through a local client. --share-root R (share mode): R is this machine's path to a shared drive; clients that have it too write their frames there instead of streaming them, and every one is checked here before it is kept. --sharing neighbourhood|off (default neighbourhood): renders keep reference orbits in a cache and the farm passes them between machines, so a deep reference is built once — a reused one is admissible by the same rule the app uses, and a handful of pixels can differ from a fresh one at extreme depth; off builds every reference where it is used. --prebuild / --no-prebuild: render the deepest keyframe here first so its reference is shared early (default for tours deeper than 1e300×). Exit 0 complete, 3 finished with frames given up, 4 stopped by its user (the folder resumes)."),
        Flag("--strikes N / --stall S / --deadline S / --first-run N", "With --farm-render: frames a client may send that fail verification before it is removed (default 2); seconds without a finished frame before a run is reassigned (default 300, more once frames show how long they take); an optional hard per-frame limit in seconds; frames in each client's first runs (default 8)."),
        Flag("--render-client HOST:PORT [--name NAME] [--max-size WxH] [--max-ss N] [--max-iter N] [--one-job]", "Render frames for a farm controller. Dials HOST:PORT (it opens no port), proves it holds the farm key, pins the controller's identity, runs a self-check render, then renders the runs it is given. A job asking more than --max-size / --max-ss / --max-iter allows is refused, not clamped. Create <config>/farm/PAUSE to stop taking work after the current frame (delete it to resume), or CANCEL to stop at once. Redials if the connection drops; ends if the controller says goodbye (for instance on removal). --one-job: exit once the controller closes its job (how a test machine's field agent runs a client). --share-root R: R is this machine's path to the controller's shared drive; in share mode frames are written there instead of sent."),
        Flag("--adapters all | N,M,…", "With --render-client: one session per graphics card — each joins the farm as \"NAME · GPU k\" and renders only on its card (N, M: numbers from --list-adapters, or parts of names). all = every graphics card, once each. Pause, cancel and disconnect act on all of them. --adapter N picks one card for the single session."),
        Flag("--when-idle MIN", "With --render-client (Windows): take work only once nobody has touched this machine's keyboard or mouse for MIN minutes, or while its screen is locked. When someone starts using it, the frame in progress stops at once and goes back to the farm, with no strike against this machine."),
        Flag("--discover", "List the render-farm controllers answering on this network (and on this machine): address, name, build, identity. A listening controller answers a UDP broadcast on port 46733; the client opens no port to ask."),
        Flag("--no-discovery", "With --farm-render: do not answer --discover / the Render client window's Find (clients then type this machine's address)."),
        Flag("--farm-key-file FILE", "The farm key (fdn1-…): the controller writes one to <config>/farm/farm-key.txt on first use and prints it; give each client a copy. Never put it on a command line."),
        Flag("--farm-allow-dirty", "Development only: let a build with uncommitted changes (-dirty), or a client running an instrument, join a farm. Both ends must pass it; it is logged."),
        Flag("--ui-status", "With --farm-render or --render-client: what the app's farm windows run them with. Prints a machine-readable `farm-status {json}` line a second beside the usual output, and takes commands on stdin (a controller: pause, resume, stop, remove ID, readmit NAME; a client: pause, resume, cancel-frame, leave). When stdin ends — the window closed — a controller stops its job and a client leaves."),
        Section("View (with --render / --find-minibrot)"),
        Flag("--fractal NAME", "Family, e.g. \"Mandelbrot\" or \"Burning Ship\"."),
        Flag("--formula STEP", "A custom formula's step in Fractint-style expressions, e.g. \"z = z^3 - p1*z + c\" (implies --fractal Custom; deep zoom unless a power's exponent varies with z or c, direct rendering then)."),
        Flag("--formula-params LIST", "The custom formula's parameters p1…p5 as re,im;re,im;…"),
        Flag("--center X Y", "View center (full-precision decimals or expressions: -3/4, -0.5 + 0.25*cos(pi/4))."),
        Flag("--zoom M", "Magnification, e.g. 2e7 or 1.0e23.9 (any depth; a bad value is fatal)."),
        Flag("--zoom-log2 L", "Magnification = 2^L — for depths past f64 range (>= ~1e308x)."),
        Flag("--julia", "Julia mode."),
        Flag("--julia-c RE IM", "Julia parameter c."),
        Section("Image & color (with --render)"),
        Flag("--size W | WxH", "Image size: bare width (height from aspect) or WIDTHxHEIGHT."),
        Flag("--ss N", "Supersampling 1–8."),
        Flag("--iter N", "Maximum iterations."),
        Flag("--palette N", "Preset palette index."),
        Flag("--palette-map FILE", "Colour through a Fractint / Kalles Fraktaler .map palette (R G B lines, one per entry). Imported as hard bands, which is what the format means -- it is a lookup table indexed by iteration count with no blending between entries, and those hard steps are the classic Fractint look. Values are taken exactly as written: Fractint's are 6-bit VGA x4, so its white is 252 rather than 255, and rescaling them would fail a comparison against Fractint's own images."),
        Flag("--palette-map-smooth", "With --palette-map: blend between the file's entries instead of banding them."),
        Flag("--palette-ugr FILE", "Colour through an Ultra Fractal .ugr gradient file. A .ugr usually holds MANY named gradients; without --palette-ugr-name the first is used and the run says which. Colours are interpolated (unlike .map), indices run 0-399, and the file's rotation is applied."),
        Flag("--palette-ugr-name NAME", "With --palette-ugr: which gradient to use, by block name or title. An unknown name lists what the file actually contains."),
        Flag("--palette-ggr FILE", "Colour through a GIMP .ggr gradient. This is the richest of the palette formats and the only one imported without loss: each segment keeps its own midpoint, blend curve (linear / curved / sine / spherical) and colour space (RGB, or either way round the hue wheel)."),
        Flag("--palette-ase FILE", "Colour through an Adobe swatch list (.ase). Swatches carry no positions, so they are spread evenly and blended linearly. RGB, Gray and (naively converted) CMYK swatches are read; LAB is refused rather than converted with a guessed colour profile."),
        Flag("--method NAME", "smooth | stripe | triangle | trap | distance | decomposition."),
        Flag("--stripe-freq N", "Stripe density (stripe method)."),
        Flag("--stripe-tail N", "Stripe average over the tail only (stripe method): an exponential window over the last N iterations of each orbit instead of the whole orbit, so extreme-depth views keep their stripe contrast (4..=1048576, 16 is the classic look; the Tail only option in Coloring)."),
        Flag("--trap SHAPE", "point | cross | circle (orbit-trap method)."),
        Flag("--binary", "Binary-decomposition coloring."),
        Flag("--light [--light-angle R]", "Enable 3D relief lighting."),
        Flag("--de", "Enable distance glow."),
        Flag("--watermark / --no-watermark", "Force the \"Fd\" watermark on / off (overrides the saved preference)."),
        Flag("--show-zoom-target / --no-show-zoom-target", "Force the auto-zoom target overlay on / off (View > Show auto-zoom target; overrides the saved preference). While the auto-zoom runs, outlines the region of the view it is zooming into with lines from its corners to the corners of the screen, and rings the point it is steering toward."),
        Flag("--autopilot-priority speed|quality", "What the auto-zoom puts first (the \"Auto-zoom priority\" switch in the right panel; overrides the saved preference). speed: zoom at the set rate and refresh the screen as soon as a frame with detail is ready. quality: only fully resolved full-resolution frames reach the screen, and the zoom slows as needed so the picture never magnifies far before the next one lands."),
        Flag("--autopilot-target detail|misiurewicz", "What the auto-zoom aims at (the \"Auto-zoom target\" switch; overrides the saved preference). detail: the richest edge in view, re-chosen as the dive goes. misiurewicz: the nearest Misiurewicz point, solved to the dive limit's depth first (the dive holds still meanwhile), then dived into exactly — Mandelbrot set only."),
        Flag("--check-language FILE", "Load a language file (i18n/<tag>/fractadyne.ftl, or fractadyne-<tag>.ftl) exactly as the program loads one, print its name, script, status and how much it translates, and list every message it would set aside and why (those show in English). Exit 0 when nothing is set aside, 1 otherwise. For translators: see i18n/README.md."),
        Flag("--language TAG", "The interface's language for this run only (en, es, fr, de, ja, zh-Hans, km, uk, or en-XA, the pseudo-language that shows hard-coded text and overflowing labels); the saved choice is unchanged. FRACTADYNE_LANG=TAG does the same. Harness and offline runs use English unless told otherwise."),
        Flag("--live-panel off|strip|panel|full", "Open the live-render front panel at this size (View > Live render panel, key B; overrides the saved preference): a 1970s-style instrument panel over the view showing what each frame dispatched, how old the picture on screen is, the walk through the iteration bands, the reference builders, and the machine underneath."),
        Flag("--show-timestamp / --no-show-timestamp", "Force the elapsed-time overlay on / off (View > Show timestamp; overrides the saved preference). Draws a large clock over the view reading the same \"+12.345s\" the log stamps every line with, plus the frame number, so a screen recording of a problem that only appears in motion can be lined up against the log frame by frame."),
        Flag("--bla / --no-bla", "Force bilinear approximation (BLA) on / off for deep floatexp Mandelbrot, Multibrot and the fold families (Tricorn, Burning Ship, Celtic, Buffalo), and for zoom-video strips deeper than 1e4x (--render-expmap, --render-zoom-video; the Render zoom video dialog says which)."),
        Flag("--glitch / --no-glitch", "Force multi-reference glitch correction on / off for the export (default on). It never runs for Mandelbrot or Multibrot views outside Julia mode, where --glitch-audit found it repairs nothing."),
        Flag("--orbit-cache / --no-orbit-cache", "Force the on-disk reference-orbit cache on / off. By default it is ON for an ordinary launch and OFF for every task invocation (--render, --selftest, --bench-matrix, tours, …), because a cached orbit makes a timed run look faster than the code is and a gate must build what it measures. A CLI render of a location you have visited in the app can opt in with --orbit-cache. File > Settings > Reference cache… shows where it is, how big, and clears it."),
        Flag("--sound / --no-sound", "Play or silence the tone that marks a finished render (default on). The FRACTADYNE_NO_SOUND environment variable does the same and is what batch runs want, because it is inherited by child processes -- --torture launches each rung as its own exe and the corpus harness shells out per location, neither of which sees a flag passed to the parent. An explicit --sound wins over the variable."),
        Flag("--normalize", "Auto-normalize the palette cycle to the frame's escape-value range on export — prevents smooth-coloring aliasing (speckle) at extreme depth; the cycle slider then means palette sweeps. Falls back to normal coloring for aux methods / huge supersampled sizes. With --assemble, the video's palette follows the dive's escape range smoothly, as the live view's does."),
        Flag("--log-palette", "Spread the palette by the LOGARITHM of the escape value rather than linearly, wherever normalization is active. Escape counts crowd towards the high end at depth, so a linear map spends most of the palette on a thin shell near the boundary and flattens the rest; log keeps color readable and stable as a zoom video's range grows."),
        Flag("--no-perf / --perf", "Hide / show the performance panel."),
        Section("Validation & diagnostics"),
        Flag("--render-iter -o F.exr", "Export raw iteration data (EXR) instead of a colored image."),
        Flag("--compare A B", "Diff two renders/EXRs: max/mean delta + difference heatmap."),
        Flag("--import-kfr F.kfr", "Load a Kalles Fraktaler (.kfr) location."),
        Flag("--import-imagina FILE", "Load an Imagina TEXT location (Formula / Location Size, Re, Im, Iterations). A second independent implementation to cross-check the validation corpus against. Binary .im files are refused by their magic — re-save from Imagina as a text location. NB Imagina's Size is a HALF-HEIGHT, so magnification = 2/Size."),
        Flag("--bench-bignum", "Reference-orbit cost per precision (CPU only, no GPU). Names the backend that actually ran and marks any row whose orbit escaped as INVALID. --iters N scales the counts."),
        Flag("--bignum", "Arbitrary-precision backend for the reference orbit: auto|astro|rug (env FRACTADYNE_BIGNUM; use the env var for batch/gate runs, which launch child processes). `rug` is an off-by-default cargo feature that does not build on windows-msvc and is LGPL-3.0+; asking for a backend this build lacks is a fatal error, never a silent fall back."),
        Flag("--validate-deep", "Extreme-depth precision self-consistency battery (1e1000 … 1e1000000x)."),
        Flag("--profile [--reps N] [--regions F] [--out P]", "Dev: time render stages per benchmark region -> JSON log in logs/."),
        Flag("--bench-matrix [--bless] [--reps N]", "Dev: path-coverage perf + regression suite (zoom bands, fractals, coloring). Compares deterministic path signatures + timings against THIS card's baseline (benchmarks/bench-matrix-baseline-<card>.json, or the reference file when it was blessed on this card); --bless records it there. Every run also writes itself as a would-be baseline (bench-matrix-run.json beside the log). Exit 2 on algorithmic drift, 3 when this card has no baseline (nothing judged)."),
        Flag("--reusetest", "Dev: measure reuse-first-zoom reprojection staleness vs Δ-octaves (design/xaos-reuse.md). REPORT-ONLY: no pass/fail criterion; exit 0 means it ran."),
        Flag("--resizetest", "Dev: headless window-resize regression harness — scripted drag-resize through the real frame logic; asserts every painted frame is aspect-correct (exit 0 = pass)."),
        Flag("--crosscheck-f3 raw.exr", "Compare a Fraktaler-3 raw EXR (channel \"N\") against our CPU bignum oracle (--center X Y --zoom-f3 Z [--iter K] [--er R])."),
        Flag("--glitch-audit [N]", "Dev: at the view and size a --render would draw, render the raw iteration buffer with and without glitch correction, and check N of the pixels correction CHANGES (default 24) against the CPU arbitrary-precision oracle — plus N control pixels where the two agree, which must match the oracle or no verdict is given. Each pixel is judged corrected-right, uncorrected-right, neither, or sensitive (its true answer changes within ±0.001 px, so either render may be right). Answers whether correction repairs wrong pixels or rewrites right ones. Writes no image."),
        Flag("--renorm-audit [N]", "Dev: as --glitch-audit, but compares the perturbation loop alone (RENORM=0) with the renormalized step near a minibrot, judging the pixels the step changes against the oracle; also prints how far apart the two are over every differing pixel. Writes no image."),
        Flag("--tail-audit [N]", "Dev: as --glitch-audit, but compares the all-floatexp deep loop (TAIL_DF32=0) with the df32 tail phase, so a change the phase makes to a pixel is judged against the oracle. Writes no image."),
        Flag("--frametest [--steps N] [--hold N] [--dive R] [--center X Y]", "Dev: deep-dive frame-timing harness (default center: seahorse valley). REPORT-ONLY: no pass/fail criterion; exit 0 means it ran."),
        Flag("--autodive [LOG10] [--autodive-timeout SECS] [--autodive-iter N] [--autodive-home N]", "Dev: UNPACED frame-cost controller hammer. Drives the auto-zoom autopilot from the CLI (auto-iter on) so frames are submitted as fast as they complete, with no tour clock to dilate away the pressure, and reports whether the LETHAL regime was actually reached: deepest depth, controller readings, peak measured iterate, lethal count. Because the autopilot re-targets the detail-richest region it dives into structure, which is where per-step cost explodes. Defaults: LOG10 320, timeout 900s, and an EXPLICIT iteration count of 250000 (the field crash manifest's value) with auto-iter OFF - the live path caps auto-iter lower for responsiveness, so auto-iter frames are too cheap to reach the lethal band (measured: 19.5ms peak over 900s). `--autodive-iter 0` selects auto-iter for comparison. After reaching the limit it performs `--autodive-home N` ZOOM-HOME glides (default 1, `0` to dive only): a monotonic dive cannot stress the controller (measured: 36ms peak against a 400ms target, frames CPU-bound), whereas Home sweeps from extreme depth to 1x across every mode boundary while the budget is still sized for the deep regime - the shape of both field device losses, and literally the button that lost one. Judged from the frame record over the whole run: exit 0 if it reached the regime and every emergency retreat held, 2 if it reached it and a retreat reached nothing, none applied, or lethal-band frames kept coming after one, 3 if it never reached the regime or the frame record lost frames before they were read (VACUOUS: nothing, or not everything, was tested — do NOT read that as a pass), 4 watchdog. Opens a window."),
        Flag("--chunk-sweep [ITERS]", "Dev: measure the UNCENSORED per-window wall cost of a chunked iterate at the CURRENT view, then exit. Settles the live view, borrows its resident reference (no fresh bignum build, so the regime measured is the one on screen), and walks iterations [0,ITERS) at each of several fixed window sizes - timing every submission alone in the queue against its own poll(Wait). That is the only cost signal here that is neither right-censored at wgpu-core's FRAME_TIMEOUT_MS=1000 nor deferred two frames, which is why the live path's own numbers cannot answer this. Reports whether a window's cost is proportional to its iteration count (so sizing the window is a real actuator), whether cost cliffs WITHIN one band (so it is not), and the fit wall~area^a (the go/no-go for any spatial sub-budget). Also asserts the path's bit-identity contract across window sizes. Stops escalating at the first window past 1200 ms rather than reproducing a device loss. Default ITERS 2048, which brackets the 2026-08-22 field crash's [0,1792) ladder. Opens a window."),
        Flag("--deviceloss-repro [ZOOM_LOG2] [REF_ITER] [--center X Y] [--m-jump] [--throttled-iter N]", "Dev: SAFE, deterministic reproduction of the deep-interior device loss (crash-1789060790, a period-3992 interior minibrot at iter-base 10M) and of the SA-SKIP INVERSION (crash-1789092955, a parabolic bulb-root exact point): --throttled-iter N (default 2,843,648, that crash's gpu_iter) is the per-dispatch budget the live frame-cost throttle would hand the view; when it sits below the reference's SA skip, usable_sa_skip refuses the skip and the shader grinds from iteration zero, so the harness prices that from-zero window from its window fit (never run) and MEASURES the dispatch sa_skip_rescue issues instead. Pins the view to the Seahorse period-998 nucleus (a KNOWN interior point, so its reference never escapes and builds to the device cap), builds that reference synchronously, then times the honest floor-window (256-iter) submission cost at increasing pixel areas against poll(Wait) — stopping under a 1200ms safety bound — and EXTRAPOLATES to the field's full resolution. It PREDICTS whether the full-res floor dispatch exceeds the driver watchdog (the field's 'shed-to-floor still lost the device') without ever submitting a lethal one, and fits wall~area^a to say whether spatial tiling below the iteration floor is the actuator. Runs headless with no visible window. Defaults: ZOOM_LOG2 54 (~1.8e16x), REF_ITER 1,000,000 (the per-window cost is reference-length-independent, so a smaller reference builds faster and measures the same thing)."),
        Flag("--motiontest", "Dev: motion-PRESENTATION gate for chunked deep views (design/mode2-chunking.md #11). Self-contained: jumps to a deep mode-2 corpus view with an explicit 1M ask, waits for the reference, then drives a wheel-style dive and a full Home glide while asserting that a partial chunk progression is never adopted as the frozen texture (the 'interior looks like noise' regression settled checkpoints cannot see), that complete refreshes keep streaming during motion, and that no frame displays a texture diverged from the frozen bookkeeping. Asserts no pinned walk completes blank during the driven phases. Exit 0 pass, 2 a criterion failed, 3 VACUOUS (the run never produced interacting chunk-eligible frames, never rendered a moving frame in mode 1 — CL-165's regime, W9 — or never booted: nothing was tested — never a pass), 4 watchdog. Opens a window; run with a wiped FRACTADYNE_CONFIG_DIR."),
        Flag("--zoomtest [OCTAVES] [--zoomtest-rate R] [--zoomtest-location FILE | session] [--zoomtest-start-log2 L] [--zoomtest-taps N,OCT,PAUSE] [--zoomtest-hold S] [--zoomtest-then-log2 L2] [--zoomtest-gate [smooth|appetite]] [--zoomtest-home [REST_S]] [--zoomtest-quiet S] [--zoomtest-snap] [--zoomtest-click FACTOR[,X,Y]] [--out P]", "Dev: on-screen update-latency harness for LIVE zooms. Runs the real window, holds a virtual Space key through the production glide (pacer and interactive reference lookahead included) for OCTAVES octaves (default 40; 1e100 from 1x is 332.2) at zoom rate R (the Zoom speed slider, default 1.0) from FILE (.fdn; default a deep corpus centre at ~1e31x) — with L given, from 2^L on FILE's centre instead of FILE's own depth (L=0: the whole descent from 1x to that point) — and records every presented frame's wall interval, whether it was a real re-iterate or a held reprojection, the zoom step, depth lag, installs and pacer throttle -> JSON (default logs/zoomtest-<stamp>.json) plus a summary: mean/p50/p95/p99/max interval, >33/50/100 ms hitch counts, the longest stall and where it happened, real-refresh cadence, held-frame magnification. The harness for 'the zoom feels jerky'. FRACTADYNE_NO_PREFETCH=1 is the A/B without the lookahead. `--zoomtest-location session` starts from the view the session opened at (a staged crash view) — with --zoomtest-start-log2 and no OCTAVES, the zoom ends on arriving at that view's own depth; `--zoomtest-taps N,OCT,PAUSE` zooms in taps of OCT octaves with PAUSE seconds between them instead of one glide, stopping at N taps or OCTAVES, whichever comes first; `--zoomtest-hold S` rests S seconds where the zoom ended before reporting. `--zoomtest-then-log2 L2`: when the glide arrives, jump (still moving) to 2^L2 on the same centre and glide again from there; only that second leg is reported — the zoom-out-then-zoom sequence a fresh single glide cannot reach. `--zoomtest-home [REST_S]`: after the zoom (none without OCTAVES), rest REST_S seconds (default 2) and click the toolbar's Home button; the zoom ends when its animated glide out to 1× does — the zoom-in-then-Home sequence of the 2026-10-21 device loss, where the zoom-in raises the iteration ask with the depth and the Home glide lowers it again. `--zoomtest-quiet S` rests S seconds at the start view before the zoom (default 2). `--zoomtest-snap`: the glide snaps to features about the view's centre, as a person's zoom does with Navigate ▸ Snap zoom to features (a harness's zoom otherwise never snaps). `--zoomtest-click FACTOR[,X,Y]`: instead of a glide, one click-to-zoom by FACTOR (the Click to zoom tool's own path) on the point X,Y of the settled start view, or its centre, then 8 s of recorded frames.Every run prints `zoomtest-regime:` (the escaped-reference predicate, as --soak-depth session) and `zoomtest-stall:` (the longest run of slow frames across which the frame budget never came down). Exit 3 = VACUOUS (too few glide frames to judge, or FRACTADYNE_REF_ESCAPE_AT armed and the regime never entered), 4 = watchdog. `--zoomtest-gate [smooth|appetite]` holds the glide to a row of design/live-zoom-smoothing.md's P-G matrix and exits 2 if it fails one: smooth = no interval over 50 ms, none over 100 ms, the held frame never more than 0.5·√rate octaves behind (0.5 at 1×, 1 at 4×); appetite = no interval over 33 ms."),
        Flag("--divetest FILE [--out P]","Dev: headless live-dive perf harness — plays real-time windows of a tour script at increasing depths through the actual playback machinery (pacer, lookahead, reuse-hold) and reports fps/hitches/refresh cadence/held-frame stalls per depth band -> JSON. FRACTADYNE_DIVETEST_GLIDE=<zoom rate> drives the windows as an interactive hold-Space glide instead (the interactive reference lookahead's harness). REPORT-ONLY: no pass/fail criterion; exit 0 means it ran."),
        Flag("--livetest FILE [--segment NAME] [--size WxH] [--out DIR] [--quick]", "Dev: headless live-OUTPUT harness — plays a tour through the live pipeline and, at every keyframe hold, compares what the live view SHOWS against an offline render of the same view at the same iteration budget, and, where the live view is black enough for it to matter, against a second offline render at 4× that budget: black that the bigger budget resolves is starvation, graded STARVED unless the status bar names the limit (then 'capped'). Reports excess blackness / sRGB difference per checkpoint, dumps the images of failing checkpoints, and compares the grades against the blessed baseline; exit 0 no drift, 2 drift, 3 nothing judged (--quick, or no checkpoint captured)."),
        Flag("--segments N --segment-index K [--dry-run]", "With --render-tour: shard the whole timeline into N contiguous, gap-free frame ranges and render only range K (0-based) — run one shard per machine and the frames union to exactly the full video (shard k = [floor(k*F/N), floor((k+1)*F/N)); global frame numbering kept; combines with --segment NAME by intersection). --dry-run prints this invocation's frame range and exits without rendering, so a farm script can verify the shards tile first."),
        Flag("--order sequential|progressive", "With --render-tour: the ORDER frames render in (default sequential). `progressive` renders the keyframes first, then repeatedly bisects the largest temporal gap — a coarse flip-book of the whole tour appears within minutes and refines to full rate, so a mis-framed deep keyframe is visible before hours of GPU are spent. Frames land at their correct indices either way (--resume and mp4 assembly unchanged); note that a normalized render's palette smoothing follows render order, so prefer sequential for final normalized delivery."),
        Flag("--no-reference-reuse", "With --render-tour: build every frame's reference orbit itself. By default the tour's deepest frame's reference is built first and every frame it can serve (the same fractal and Julia setting, its point still in the frame's view at enough precision) reuses it, extended where a frame wants more iterations — one bignum build instead of one per frame. The frames keep their order, and a reused reference can draw a handful of pixels differently from one built for the frame itself, as the farm's shared references do."),
        Flag("--frames A..B", "With --render-tour: render only the global frames A through B-1 (A..=B includes B), intersected with --segment/--segments when given — one run of a render farm. Frame numbering stays global, so the frames drop into the full sequence. A range starting past the end of the tour is an error; an empty or reversed one is refused."),
        Flag("--dump-norm-anchors FILE", "With --render-tour on a normalized tour ([render] normalize = true): measure the whole tour's time-keyed palette ranges once, write them to FILE, and exit without rendering a frame. Hand FILE to every machine rendering the tour (--norm-anchors) so they all colour it identically — escape values differ between GPUs, so ranges each machine measured for itself flicker at the seams between their frames."),
        Flag("--norm-anchors FILE", "With --render-tour: use the normalize ranges in FILE (from --dump-norm-anchors) instead of measuring them on this machine. Refused, naming the difference, when FILE was measured for a different script, frame rate, frame count or iteration base — set [render] max_iter in the script so every machine agrees on the last."),
        Flag("--farm-child", "With --render-tour: this render is one run of a render farm, started by a render client. Runs with no visible window, pins what would make the same frame differ between machines (a fixed display scale for the burned-in watermark, palette and light animation off), plays no sound, exits as soon as its last frame is written, and prints one line per frame once it is on disk: `frame-done index=N bytes=B sha256=HEX ms=M`. With --render-expmap: one run of a farm's zoom video — one such line per strip (N = the strip), a strip already in DIR included, and `frame-failed index=N reason=…` for one that could not be rendered."),
        Flag("--dualsettle [DIR]", "Dev: dual-view Julia SETTLE harness — boots dual view unpinned, drives the live-hover c along an arc for ~3 s exactly as the cursor path does, then idles and screenshots the settled state at 1/4/10 s -> DIR (default logs/). Reproduces the 2026-09-04 blurry-Julia-after-motion field report deterministically. REPORT-ONLY: no pass/fail criterion; exit 0 means it ran."),
        Flag("--juliadive [DIR]", "Dev: dual-view Julia motion harness — boots dual view with a pinned spiral c (explicit 2000 iterations, prefer-detail on) and zooms the Julia panel in-app to ~2000x, screenshotting every octave in motion plus stopped/settled frames -> DIR (default logs/). Reproduces the dual-view Julia zoom rendering path deterministically. REPORT-ONLY: no pass/fail criterion; exit 0 means it ran."),
        Flag("--farmtest [DIR]", "Dev: a whole render farm on this machine, with faults, against an independent reference. Renders a small normalized tour once as the reference, then runs a controller and three loopback clients: A sends its first two frames corrupted (instrument FRACTADYNE_FARM_CORRUPT_FRAMES) and must be removed with a diagnostics bundle; B is killed mid-run and its frames must go back to the queue; C behaves. Passes when the job completes with exactly the tour's frames, each pixel-identical to the reference, recorded once, with A's bad frames quarantined and both stories in the event log. Work goes to DIR (default a temp folder). Exit 0 pass, 1 fail, 2 VACUOUS (a fault never happened, so it proved nothing)."),
        Flag("--farmtest-strips [DIR]", "Dev: a zoom video's strips on a render farm on this machine (--render-zoom-video … --farm), with a fault, against an independent reference. Renders a small dive's strips once with --render-expmap as the reference, then a controller and three loopback clients: A sends its first two strips corrupted and must be removed; B streams its strips; C delivers through a shared folder. Passes when the video is made and every strip is identical in every channel to the reference, recorded once, with A's strips quarantined and the shared folder cleaned up. Exit 0 pass, 1 fail, 2 VACUOUS."),
        Flag("--gputest", "Dev: verify the shader's df32/floatexp arithmetic primitives (error-free transforms, double-float ops, floatexp ops, iteration-form accumulation) against CPU oracles, on EVERY graphics backend this machine offers (DX12 / Vulkan / OpenGL), and print a per-op verdict table for each. Runs headless — no window, works over SSH. Catches a shader compiler that folds the error-free transforms (which silently degrades every extended-precision path to plain f32), fast-math fma contraction, flushed denormals, or wrong rounding — include the report in bug reports about wrong deep rendering. Exit 1 if any backend fails."),
        Flag("--uitest [DIR]", "Dev: scripted walk through every UI screen + the live-render bands (Direct/df32/floatexp), screenshotting each and writing a review bundle (screenshots + report.md/json + log.txt) to DIR (default: the mounted share, else logs/). Runs under the real eframe loop — on a headless box wrap it with xvfb-run. Exit 1 if any step FAILs."),
        Flag("--soak SECONDS", "Dev: sit at a deep view for SECONDS and assert the app is still ALIVE — the frame counter must advance in EVERY 20 s window, the resident set may not grow past 256 MB, and no crash report may appear. A soak that only greps for crashes passes a hung app, which is exactly the failure this looks for. Depth with --soak-depth DECADES (default 30). Exit 2 on a stall, a leak, a crash report, or 8+ slow frames the frame budget never came down across (the 2026-09-21 field device-loss signature); 3 (VACUOUS) when no 20 s window completed."),
        Flag("--soak-depth DECADES | session", "log10 magnification for --soak (default 30 — deep enough to be on the perturbation path with a real reference orbit), or `session` to soak the view the session opened at (a staged crash view). With FRACTADYNE_REF_ESCAPE_AT armed this is W9's escaped-reference rung: a `soak-regime:` line says whether the regime was entered, and armed-but-not-entered exits 3 (VACUOUS)."),
        Flag("--soak-location FILE", "Soak the view a .fdn file holds instead of a depth (W9: the field crash view, validation/field/radeon-crash-2026-09-21.fdn, on any machine). Not with --soak-depth."),
        Flag("--recordtest [FRAMES]", "Dev: the per-frame record's own gate. Drives FRAMES live frames (default 240) at a 1e12 view with a jump halfway and a deliberate 13 s wedge of the UI thread three quarters through, then checks every frame was recorded exactly once with every required field set, that frames.bin read back is exactly the in-memory ring, that the watchdog recorded the wedge, that frames.jsonl is well formed and adds up, that the record's own cost is under 0.5% of a 60 Hz frame, and — through a child process that records 60 frames and aborts (launched internally with --recordtest-abort-after N) — that frames.bin survives a death the panic hook never sees. Exit 0 pass, 2 RED, 3 VACUOUS (no dispatch, reading or counter reading was exercised, or --set overrides were in force)."),
        Flag("--play FILE", "Start with a tour already playing in the live view (Tools -> Play tour, from the command line). Useful for reproducing live-playback behaviour that headless harnesses cannot reach."),
        Flag("--oomtest", "Dev: request an unsatisfiable allocation to prove the out-of-memory path writes a crash report. Exits 0xC0000409 (abort) by design — an OOM skips the panic hook, so this is the only way to exercise that reporting."),
        Flag("--dump-tour-schema", "Print the tour-script (.toml) schema reference as Markdown and exit (generates TOURS.md)."),
        Flag("--dump-frame-schema", "Print the per-frame record's encoding (field names, types, slot layout) as JSON and exit (generates validation/frame-schema.json, which scripts/framelog.py reads to decode frames.bin and check a frames .jsonl)."),
        Flag("--log-dir DIR", "Write the log file and crash reports to DIR instead of the config directory's logs folder (created if missing; a network share works). Only the logs move — the session and settings stay put, unlike FRACTADYNE_CONFIG_DIR which relocates everything. The FRACTADYNE_LOG_DIR environment variable does the same; the flag wins when both are set. If DIR cannot be created, logging falls back to the default location and says so in the log."),
        Flag("--console / --no-console", "Force the [fd-…] diagnostic lines on or off for this run. They go to stderr and are ON whenever fractadyne is given any argument (so every headless render, harness and validation script keeps its output) and OFF for a bare launch of the GUI, which nobody asked to narrate. The FRACTADYNE_CONSOLE environment variable does the same (0 off, anything else on) and the flag wins; setting FRACTADYNE_TRACE turns the console on by itself, since a trace nobody can see is not a trace. The log file is written either way, so nothing is lost — see Help ▸ Show recent log, or toggle it live in Tools ▸ Diagnostics. A panic always reaches the console."),
        Flag("--test-banner / --no-test-banner", "Force the LIVE TEST banner on or off. When a harness drives a real window (--livetest, --motiontest, --zoomtest, --uitest, --autodive, --juliadive, --dualsettle, --divetest, --frametest, --resizetest, --reusetest, --chunk-sweep, --play-tour) the app paints a red 'LIVE TEST' banner across the top so the window can't be mistaken for an interactive session you left open; it is a foreground overlay outside the fractal image, so it changes no pixel the self-test, goldens or --livetest compare. On by default for those runs. --no-test-banner hides it (for a clean --uitest screenshot bundle); --test-banner forces it on even for a normal launch."),
        Flag("--logcheck [LOG] [--logcheck-rules FILE] [--logcheck-all]", "Dev: hold a log to validation/logcheck-rules.toml (compiled in; --logcheck-rules reads another file) and exit with the verdict: 0 PASS, 1 FAIL (a rule broken, a harness's verdict line missing, a crash), 2 VACUOUS (nothing to judge, or rules that do not load), 3 NO VERDICT (a harness run that never reached its end: killed, wedged or crashed). LOG defaults to this app's own log; the last session is judged, every session with --logcheck-all. Every harness run also checks its own log as it exits, and a broken rule turns its exit code 0 into 1. The rules bound the warnings the live path logs — a blind frame budget, a pass in the lethal band, GPU timing that stopped arriving — with named, bounded exceptions for known pre-existing conditions, printed as KNOWN on every run."),
        Flag("--window WxH", "The window's starting inner size in logical points (default 1280x800, at least 640x400). Physical pixels are that times the display scale — the log's `[fd-dpi]` line shows both. For reproducing a reported window: what a live frame costs depends on the size it renders at."),
        Flag("--worker-gpu same | N | NAME", "Dev (design/multi-gpu-live.md L2, L3): a SECOND graphics device renders frames of a moving deep view (adopted when they show it better than the frame on screen) and some of the progressive-supersampling samples of a settled one. same = a second device on the window's own card, else a number from --list-adapters or part of a name. Overrides Advanced ▸ Second graphics card. The loss of another card never ends the app; a reset of the window's own card does, whichever device caused it."),
        Flag("--adapter N | NAME", "Render on this graphics adapter: its number in --list-adapters, or part of its name (--adapter \"RX 6800\"; where several match, the Vulkan one). Applies to the window and to every render it runs (--render, --render-tour, a render client's frames). Without it, the high-performance adapter, as always. One that names no adapter is an error that lists them."),
        Flag("--list-adapters", "Print this machine's graphics adapters — number, name, graphics API, kind, driver — and exit."),
        Flag("--set NAME=VALUE", "Dev: override one frame-cost tunable for THIS RUN (repeatable). The shipped defaults are the only tested path — the self-test, goldens, --bench-matrix and --livetest all assume them — so this exists for field diagnosis (\"does this reproduce at a 400 ms target?\"), not configuration. Every override is logged at startup and stamped into any crash report, so a report from an overridden run can never look stock. Names: TDR_BUDGET_MS, TDR_EXPLICIT_BUDGET_MS, TDR_LATENCY_ACCEPT_MS, TDR_GROW_MAX, TDR_SHRINK_MAX, TDR_BOOTSTRAP_STEPS, TDR_MIN_STEPS, TDR_STEPS_CEIL, EXPLICIT_STEPS_CEIL, EXPLICIT_DISPATCH_CAP, TDR_MAX_TILES, TDR_TILES_CEIL, PASS_FIXED_MS, MOTION_NEED_QUANTILE (all documented in src/tunables.rs), and ORBIT_LEN_CAP — the reference-orbit length cap in samples, normally derived from the GPU; a render farm passes its smallest machine's to every machine so a deep reference truncates alike everywhere, and a value above what this GPU holds is refused. An unknown name or a non-positive value is a fatal error, never a silent no-op."),
        Section("Examples"),
        Example("fractadyne --render -o out.png --fractal Mandelbrot --center -0.743644 0.131826 --zoom 2e7 --iter 6000 --method stripe --ss 3"),
        Example("fractadyne --render-tour tours/deep-spiral-dive.toml --size 1920x1080 --fps 30 --ss 2 --out frames --mp4 --show-location"),
        Example("fractadyne @render.args        # read the whole command line from render.args"),
    ]
};

pub(crate) fn help_command_line(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_command_line));
    // The intro is prose, so the window shows it in the language in use (`help-cli-intro`, the
    // same English as CLI_INTRO); the flags below are the command line itself, which is English
    // in every language (design/multilingual.md §2), as is `--help`.
    help_p(ui, &tr!(help_cli_intro));
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
    help_h(ui, &tr!(help_section_shortcuts));
    help_sub(ui, &tr!(help_mouse));
    help_kv(ui, &tr!(help_key_left_drag), &tr!(help_shortcuts_pan));
    help_kv(ui, &tr!(help_key_wheel), &tr!(help_shortcuts_wheel));
    help_kv(ui, &tr!(help_key_shift_drag), &tr!(help_shortcuts_box_zoom));
    help_kv(ui, &tr!(help_key_click_tool, icon = "🎯"), &tr!(help_shortcuts_click));
    help_sub(ui, &tr!(help_shortcuts_keyboard));
    help_kv(ui, "Space / Shift+Space", &tr!(help_shortcuts_space));
    help_kv(ui, "Backspace", &tr!(help_shortcuts_undo));
    help_kv(ui, "Shift+Backspace / Ctrl+Y", &tr!(help_shortcuts_redo));
    help_kv(ui, "M", &tr!(help_shortcuts_minibrot));
    help_kv(ui, "A", &tr!(help_shortcuts_autopilot));
    help_kv(ui, "B", &tr!(help_shortcuts_live_panel));
    help_kv(ui, "Esc", &tr!(help_shortcuts_esc));
    help_kv(ui, "Ctrl+S", &tr!(help_shortcuts_snapshot));
    help_kv(ui, "F1 / ?", &tr!(help_shortcuts_help));
    help_sub(ui, &tr!(help_shortcuts_life));
    help_kv(ui, "Enter", &tr!(help_shortcuts_life_run));
    help_kv(ui, "N / Shift+N", &tr!(help_shortcuts_life_step));
    help_kv(ui, "D", &tr!(help_shortcuts_life_draw));
    help_kv(ui, &tr!(help_key_click_drag_draw), &tr!(help_shortcuts_life_paint));
    help_kv(ui, &tr!(help_key_when_drawing, key = "Ctrl+Z"), &tr!(help_shortcuts_life_undo));
}

/// A cited entry: bold title, a wrapped description, and a source link.
fn help_cite(ui: &mut egui::Ui, title: &str, body: &str, link_label: &str, url: &str) {
    help_cite_unlinked(ui, title, body);
    ui.hyperlink_to(format!("{link_label} \u{2197}"), url);
}

/// A citation with no link. For credits whose only web source has gone: the credit still stands,
/// and a dead link in an acknowledgement reads as neglect. (2026-09-19: two cited sites went
/// offline and their links were removed.)
fn help_cite_unlinked(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.add_space(6.0);
    let ink = ui.visuals().strong_text_color();
    ui.label(egui::RichText::new(title).strong().color(ink));
    ui.add(egui::Label::new(body).wrap());
}

pub(crate) fn help_hardware(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_hardware));
    help_p(ui, &tr!(help_hardware_intro));

    help_sub(ui, &tr!(help_hardware_gpu));
    help_p(ui, &tr!(help_hardware_gpu_text));
    help_kv(ui, &tr!(help_hardware_minimum), &tr!(help_hardware_gpu_minimum));
    help_kv(ui, &tr!(help_hardware_recommended), &tr!(help_hardware_gpu_recommended));
    help_kv(ui, &tr!(help_hardware_big_exports), &tr!(help_hardware_gpu_big_exports));

    help_sub(ui, &tr!(help_hardware_cpu));
    help_p(ui, &tr!(help_hardware_cpu_text));
    help_kv(ui, &tr!(help_hardware_recommended), &tr!(help_hardware_cpu_recommended));

    help_sub(ui, &tr!(help_hardware_memory));
    help_p(ui, &tr!(help_hardware_memory_text));

    help_sub(ui, &tr!(help_hardware_checking));
    help_p(ui, &tr!(help_hardware_diagnostics));
    help_p(ui, &tr!(help_hardware_gpu_check));
    help_p(ui, &tr!(help_hardware_report));

    help_sub(ui, &tr!(help_hardware_notes));
    help_bullet(ui, &tr!(help_hardware_note_watchdog));
    help_bullet(ui, &tr!(help_hardware_note_binary));
    help_bullet(ui, &tr!(help_hardware_note_benchmark));
}

pub(crate) fn help_acknowledgments(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_ack_title));
    // Dedication.
    let accent = crate::theme::ui_accent(ui.ctx());
    ui.label(egui::RichText::new(tr!(help_ack_dedication)).italics().color(accent));
    help_p(ui, &tr!(help_ack_dedication_text));
    help_p(ui, &tr!(help_ack_intro));

    help_sub(ui, &tr!(help_ack_deep_zoom));
    help_cite_unlinked(
        ui,
        &tr!(help_ack_martin_title, who = "K. I. Martin"),
        &tr!(help_ack_martin),
    );
    help_cite(
        ui,
        &tr!(help_ack_bla_title, who = "Zhuoran"),
        &tr!(help_ack_bla),
        &tr!(help_ack_bla_link, who = "Phil Thompson"),
        "https://philthompson.me/2023/Faster-Mandelbrot-Set-Rendering-with-BLA-Bivariate-Linear-Approximation.html",
    );
    help_cite(
        ui,
        &tr!(help_ack_glitch_title, who = "Pauldelbrot"),
        &tr!(help_ack_glitch),
        "Deep zoom theory & practice — mathr",
        "https://mathr.co.uk/blog/2021-05-14_deep_zoom_theory_and_practice.html",
    );
    help_cite(
        ui,
        &tr!(help_ack_diffabs_title, who = "laser blaster"),
        &tr!(help_ack_diffabs),
        "Deep zoom theory & practice — mathr",
        "https://mathr.co.uk/blog/2021-05-14_deep_zoom_theory_and_practice.html",
    );
    help_cite(
        ui,
        &tr!(help_ack_mathr_title, who = "Claude Heiland-Allen (mathr)"),
        &tr!(help_ack_mathr),
        "Fraktaler-3",
        "https://fraktaler.mathr.co.uk/",
    );

    help_sub(ui, &tr!(help_ack_coloring));
    help_cite(
        ui,
        &tr!(help_ack_harkonen_title, who = "Jussi Härkönen"),
        &tr!(help_ack_harkonen),
        "On Smooth Fractal Coloring Techniques (2007)",
        "https://archive.org/details/j-harkonen-on-smooth-fractal-coloring-techniques-masters-thesis-2007-hi-res",
    );
    help_cite(
        ui,
        &tr!(help_ack_mitchell_title, who = "Kerry Mitchell"),
        &tr!(help_ack_mitchell),
        &tr!(help_ack_mitchell_link, who = "Kerry Mitchell"),
        "http://www.kerrymitchellart.com/tutorials/formulas2/uf2-2.htm",
    );
    help_p(ui, &tr!(help_ack_distance));

    help_sub(ui, &tr!(help_ack_lsystems_life));
    help_cite_unlinked(
        ui,
        &tr!(help_ack_lindenmayer_title, who = "Aristid Lindenmayer"),
        &tr!(help_ack_lindenmayer),
    );
    help_cite(
        ui,
        "The Algorithmic Beauty of Plants — Przemysław Prusinkiewicz & Aristid Lindenmayer",
        &tr!(help_ack_abop),
        &tr!(help_ack_abop_link, site = "Algorithmic Botany"),
        "https://algorithmicbotany.org/papers/#abop",
    );
    help_cite(
        ui,
        &tr!(help_ack_bourke_title, who = "Paul Bourke"),
        &tr!(help_ack_bourke),
        "L-system user notes — Paul Bourke",
        "https://paulbourke.net/fractals/lsys/",
    );
    help_cite_unlinked(
        ui,
        &tr!(help_ack_conway_title, who = "John Horton Conway"),
        &tr!(help_ack_conway),
    );
    help_cite(
        ui,
        &tr!(help_ack_lifewiki_title, site = "LifeWiki"),
        &tr!(help_ack_lifewiki),
        "LifeWiki",
        "https://conwaylife.com/wiki/",
    );

    help_sub(ui, &tr!(help_ack_foundations));
    help_cite_unlinked(
        ui,
        &tr!(help_ack_mandelbrot_title, who = "Benoit B. Mandelbrot"),
        &tr!(help_ack_mandelbrot),
    );
    help_p(ui, &tr!(help_ack_built_with));
    help_p(ui, &tr!(help_ack_errors));
}

/// Full third-party dependency license notices — reproduced verbatim (the MIT / BSD / Apache /
/// Zlib / Unicode / font licenses require this in binary distributions). Generated by
/// `cargo-about`; the same text ships as `THIRD-PARTY-NOTICES.md` with the download.
pub(crate) fn help_licenses(ui: &mut egui::Ui) {
    // Notices generated at the repo root (see about.toml / about.hbs).
    const NOTICES: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../THIRD-PARTY-NOTICES.md"));
    help_h(ui, &tr!(help_licenses_title));
    help_p(
        ui,
        &tr!(
            help_licenses_intro,
            licenses = "MIT OR Apache-2.0",
            url = "https://crates.io/crates/option-ext"
        ),
    );
    ui.add_space(6.0);
    if ui.button(format!("{} {}", crate::icons::COPY, tr!(help_licenses_copy_all))).clicked() {
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

/// The contents-list name of the coordinate-expression reference, so the Go-to dialog's "?" can
/// open Help AT it by name rather than by an index that moves when a section is inserted.
pub(crate) const EXPRESSIONS_SECTION: &str = "Coordinate expressions";

/// The contents-list name of the credits, which the UI walk (`--uitest`) photographs.
pub(crate) const ACKNOWLEDGMENTS_SECTION: &str = "Acknowledgments";

/// Contents-list index of a section by name (the window stores the selection as an index). A
/// name not in the list yields Overview rather than a panic — the caller passes a constant from
/// this module, and `section_index_finds_every_named_section` pins that it resolves.
pub(crate) fn section_index(name: &str) -> usize {
    SECTION_NAMES.iter().position(|n| *n == name).unwrap_or(0)
}

/// The Help window's contents list and the body that draws each entry — ONE table, so a section
/// cannot exist in the list without something to render.
///
/// It replaced a `match` whose `_` arm fell through to About: a section added to the list and not
/// to the match rendered the About text under its own heading, which is invisible unless someone
/// clicks that exact entry. Paired here, and pinned by `help_sections_all_render`.
///
/// These English names are the sections' KEYS ([`section_index`], the Go-to dialog, the UI walk):
/// they never change with the language. What the window draws is [`section_title`].
pub(crate) const SECTION_NAMES: [&str; 12] = [
    "Overview",
    "Navigation",
    EXPRESSIONS_SECTION,
    "Coloring & options",
    "Fractals",
    "How it works",
    "Command line",
    "Shortcuts",
    "Recommended hardware",
    ACKNOWLEDGMENTS_SECTION,
    "Licenses",
    "About",
];

/// The body for each entry of [`SECTION_NAMES`], in the same order.
pub(crate) const SECTION_BODIES: [fn(&mut egui::Ui); SECTION_NAMES.len()] = [
    help_overview,
    help_navigation,
    help_expressions,
    help_options,
    help_fractals,
    help_methodology,
    help_command_line,
    help_shortcuts,
    help_hardware,
    help_acknowledgments,
    help_licenses,
    help_about,
];

/// The title the Help window's contents list draws for entry `i` of [`SECTION_NAMES`], in the
/// language in use. Sized by `SECTION_NAMES.len()`, so a section added to the list without a title
/// does not compile; an index past the end is clamped to About, as the window clamps it.
pub(crate) fn section_title(i: usize) -> String {
    let titles: [fn() -> String; SECTION_NAMES.len()] = [
        || tr!(help_section_overview),
        || tr!(help_section_navigation),
        || tr!(help_section_expressions),
        || tr!(help_section_options),
        || tr!(help_section_fractals),
        || tr!(help_section_how_it_works),
        || tr!(help_section_command_line),
        || tr!(help_section_shortcuts),
        || tr!(help_section_hardware),
        || tr!(help_section_acknowledgments),
        || tr!(help_section_licenses),
        || tr!(help_section_about),
    ];
    titles[i.min(SECTION_NAMES.len() - 1)]()
}

/// About's "Deep-zoom arithmetic" line: what has actually RUN, and what this binary contains.
///
/// The two halves answer different questions, and both matter. The first cannot name a backend
/// that never iterated - it is stamped by orbits finishing, not by configuration - so before the
/// first deep render it honestly reports that none has. The second is the compile-time fact that
/// tells an accelerated build apart from the standard one. Split out of the panel so checklist
/// steps 89 and 95 can be checked without drawing it.
pub(crate) fn about_arithmetic_line() -> String {
    tr!(
        help_about_arithmetic,
        ran = fractadyne_core::backend_status_line(),
        built = fractadyne_core::built_in_backends()
    )
}

pub(crate) fn help_about(ui: &mut egui::Ui) {
    help_h(ui, &tr!(help_section_about));
    help_p(ui, &format!("Fractadyne v{}", version_string()));
    help_p(ui, &tr!(help_about_tagline));
    help_p(ui, &about_arithmetic_line());

    help_sub(ui, &tr!(help_about_files));
    help_p(ui, &tr!(help_about_files_text));
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(fractadyne_state::state_location_display())
                .monospace()
                .small(),
        );
        if ui.small_button(crate::icons::COPY).on_hover_text(tr!(help_about_copy_path)).clicked() {
            ui.ctx().copy_text(fractadyne_state::state_location_display());
        }
    });
    help_p(ui, &tr!(help_about_session, command = "fractadyne --reset-state"));

    help_sub(ui, &tr!(help_about_license));
    help_p(ui, &tr!(help_about_license_text, licenses = "MIT OR Apache-2.0"));
    help_p(ui, "© 2026 Rithea Hong.");
    ui.hyperlink_to(
        format!("{} \u{2197}", tr!(help_about_source)),
        "https://github.com/WindySnowOwl/fractadyne",
    );
}

#[cfg(test)]
#[path = "../tests/unit/help/help_tests.rs"]
mod help_tests;
