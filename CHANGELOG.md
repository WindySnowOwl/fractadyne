# Changelog

All notable changes to Fractadyne. Versioning is `MAJOR.MINOR.PATCH` (Cargo) plus an
auto-incrementing **build** number (bumped by `build.rs` on every recompile) shown as
`v0.1.0 (build N)` in the title bar, Help menu, and exported image metadata.

The project enters tracked versioning at **0.1.0**; entries below summarize the state
at that point and changes after it.

## 0.1.0

Baseline for tracked versioning. Notable capabilities already present:

- **Deep zoom** — arbitrary-precision center, df64 reference orbit, df32 GPU
  perturbation with Zhuoran rebasing, hybrid direct/perturbation crossover; clean to
  ~10²²×. Generalized to Multibrot 3/4/5 and Tricorn in both Mandelbrot and Julia modes.
- **Fractal variety** — 10 escape-time families with per-fractal info panels.
- **Dual linked view** (Mandelbrot ↔ Julia) with per-view reference caches, Julia pin.
- **High-res export** — tiled PNG / OpenEXR with reloadable metadata, gallery browser,
  background rendering, progress + cancel.
- **UI** — combined menu/toolbar with icons, docked performance panel, animated
  zoom-home, fullscreen (Esc to exit), interactive orbit overlay (tapered gradient,
  racing-dot animation, normalized full-view mode), palette cycling animation.

### Added (post-baseline, this session)

- **Validation & self-test suite** — a layered correctness harness with no external data
  (exact mathematics + internal cross-checks), designed to be independently verifiable.
  `cargo test -p fractadyne-core` adds exact-ground-truth tests: hyperbolic-component
  nuclei & periods (recovered to 1e-9), Misiurewicz pre-periodicity, closed-form interior
  membership, and dwell symmetry. `fractadyne --selftest` runs a GPU validation suite
  (exit code 0/1) checking the perturbation path against an independent **CPU f64 dwell**
  and a **naive arbitrary-precision (bignum) dwell oracle** (no perturbation, no reference)
  comparing the **integer escape count** across a **depth battery (1e6× … 1e30×)** that
  exercises whichever render mode the depth selector actually uses (df32 perturbation and
  floatexp), excluding only ill-conditioned boundary samples — independent deep-zoom
  correctness, not just internal consistency. Plus a **reference-independence** check
  (renders one view with three distinct in-view references and a reference-override path;
  the auto reference must agree with the per-pixel majority across the smooth region — an
  oracle-free glitch detector that also seeds multi-reference correction), floatexp-vs-df32
  agreement, real-axis symmetry, interior/exterior presence, and finiteness (via a new
  `render_iter` that reads back the raw iteration texture). **Family symmetries** are
  verified exactly in `fractadyne-core` (Multibrot (d−1)-fold rotation, Tricorn 3-fold,
  Julia z→−z, Celtic real-axis; confirmed Burning Ship / Buffalo have *no* axis symmetry)
  and the **render pipeline** is checked for the non-Mandelbrot family shaders in
  `--selftest` (Multibrot-3 180°, Tricorn / Celtic real-axis). The exact-landmark catalog
  is extended (cardioid cusp c=¼, period-1↔2 neck c=−¾, period-2 disk, cardioid boundary
  parametrization). **Golden-image regression**: `--selftest --bless` records
  reference PNGs under `validation/golden/`; subsequent runs diff against them with a pixel
  tolerance. Every run writes a **readable, verifiable Markdown report**
  (`validation/report.md`) with full provenance (version, GPU, CPU, OS), each check's
  parameters/result/threshold/verdict, golden checksums, and the exact `--render` command
  to reproduce each golden — so a third party can independently re-run and confirm.

- **In-app Help & reference** — Help → "Help & reference…" (or F1 / ?) opens a multi-section
  window with a table of contents: Overview, Navigation, Coloring & options, Fractals
  (mathematically accurate per-family formulas + descriptions, Julia mode, deep-zoom
  support), How it works (escape-time, arbitrary precision, perturbation, floatexp,
  distance estimation), Command line, Shortcuts, and About. Written for newcomers.

- **Famous-locations tour, random location & help overlay** — a **Locations** menu with
  curated named Mandelbrot spots (Seahorse / Elephant Valley, spirals, a mini-Mandelbrot,
  a deep seahorse) that jump at full precision, plus **"🎲 Random location"** which finds
  a detail-rich boundary point (bisecting between an interior anchor and a random exterior
  direction) and zooms in a random amount. A **Keyboard & controls** overlay (Help menu /
  **F1** / **?**) documents every shortcut and the new coloring/minimap/minibrot features.

- **Custom gradient / palette editor** — Coloring → "Edit gradient…" (or the "Custom"
  palette entry) opens an editor with a live gradient preview, a color picker and
  position slider per stop, add/remove (up to 8 stops), and "Copy preset…" to seed from a
  built-in. The custom gradient is persisted and used everywhere the palette is (live
  view, export, minimap thumbnail).

- **Minimap overview ("you are here")** — View → "Minimap overview" shows a small static
  home-view thumbnail (rendered via the export pipeline, cached per fractal/palette/
  method) in the bottom-left, with a marker for the current location (the view rectangle
  when shallow, a crosshair when the view is sub-pixel deep) and the live zoom-depth
  label. Click it to jump to a region at home zoom. Persisted; shown in single
  Mandelbrot-mode (hidden in dual / Julia).

- **Period / minibrot finder ("zoom to center")** — View → "Find minibrot center" (or
  press **M**) snaps the view center to the nearby minibrot's exact nucleus and reports
  its period in a transient toast. Detects the atom-domain period (global argmin of |Zₙ|
  on the critical orbit), then Newton-refines `c` in arbitrary precision until the orbit
  closes (`Z_period(c) = 0`), recovering the true smallest period and rejecting runaway
  Newton / non-nuclei. Holomorphic families (Mandelbrot / Multibrot). Verified deep
  (period-998 at 2e7×). Headless `--find-minibrot --center X Y [--zoom M] [--fractal N]`.

- **More coloring methods** — a Coloring → "Method" picker beyond smooth iteration:
  **stripe average** (flowing sinusoidal orbit bands, with a density slider),
  **triangle-inequality average**, **orbit trap** (point / cross / circle shapes, colors
  the interior too), **distance estimate** (shades by proximity to the boundary), and
  **decomposition** (binary external-angle cells). Orbit statistics are accumulated on
  the GPU into a second render target (added only when a method needs it, so smooth/
  distance keep full speed) and work at any zoom depth (direct + both perturbation
  paths). Persisted; CLI `--method NAME [--stripe-freq N] [--trap point|cross|circle]`.

- **Go-to location dialog + navigation undo/redo** — View → "Go to location…" to
  view/edit/paste/copy the exact center (full precision) + zoom; navigation history
  records each settled location and discrete jumps, with Backspace = undo view,
  Shift+Backspace / Ctrl+Y = redo (and View-menu items). Keys are ignored while typing.
- **Fractadyne branding & theme** — a charcoal dark UI theme with amber (#E0A030)
  accents (selection, links, hovered/active widgets), the two-color "Fracta·dyne"
  logotype in the top bar, and a procedural amber-ring window/taskbar icon — matching
  the `design/Fractadyne.dc.html` mockup.
- **Animated 3D relief lighting** — "Rotate light" spins the relief light direction over
  time (shares the Speed slider), alongside the animated distance glow and palette cycling.

- **Auto-incrementing build versioning** + this changelog; version shown in the title
  bar, Help menu, and export metadata.
- **Randomized palette mode** — palette animation can synthesize and continuously morph
  random gradients instead of cycling a fixed preset. Gradients are **harmonious** (one
  base hue + gentle analogous excursion + smooth dark→bright→dark arc), not clashing.
- **Scripting** — keyframe camera tours (center + zoom over time, eased), loadable from
  TOML, with a built-in demo/benchmark tour.
- **Benchmark** — runs a fixed scripted tour while sampling FPS, CPU ms, GPU ms, and
  peak RAM, then reports aggregates for comparing builds and machines. Report includes
  host **system info** (CPU brand/cores/cache, GPU, VRAM). Runnable headless via
  `fractadyne --benchmark [--out PATH]` for automated evaluation.
- **Headless render** — `fractadyne --render --out IMG [view options]` renders a single
  fractal image (fractal/center/zoom/size/ss/iter/julia/palette) and exits, for
  debugging and automated golden-image checks.
- **Release build** — `[profile.release]` tuned to build under this machine's memory
  limit (no debuginfo, no LTO). Optimized numerics: bignum reference recompute ~8×
  faster (374 ms → 45 ms), avg CPU ~7.6× faster. Build counter is now shared across
  debug/release profiles (`.build_seq` at the workspace root) so it stays monotonic.

- **Unlimited deep zoom (floatexp δ)** — the GPU perturbation delta now uses a floatexp
  representation (df32 mantissa + i32 exponent) past ~1e28×, removing the f32 exponent
  wall that broke rendering around 1e31–1e32×. Hybrid by depth (direct → df32
  perturbation → floatexp perturbation) so the common range keeps full speed; floatexp
  (~1.7× per-iteration cost) engages only when needed. Depth is now limited by the
  center-coordinate precision (which grows as you zoom) and the iteration budget.

- **Bookmarks / presets** — save the current view (full precision) and jump back to it
  instantly. Bookmarks menu + ★ toolbar button + a Manage… window (add/name/list/delete);
  persisted to `bookmarks.toml` in the config dir.

- **Distance-estimate relief lighting** — optional 3D/embossed shading from the
  fractal's derivative (`dz/dc`), tracked in floatexp so it works at any zoom depth
  (direct + perturbation paths; holomorphic families). Coloring-panel toggle + light
  angle/relief sliders; `--light` CLI flag; persisted. Iteration texture is now
  RGBA32F (the slope normal rides alongside the iteration value).
- **Distance-estimate glow (+ animation)** — bright distance-contour bands that densify
  into glowing filaments near the boundary, from the derivative magnitude (distance
  estimate). "Distance glow" toggle + Glow/Band-width sliders + "Animate glow" (flows
  the bands). `--de` CLI flag; persisted. Works at any depth (verified at 1e8×).

### Fixed (post-baseline, this session)

- **Speckle/noise across the exterior at deep zoom on a large window** — a very high
  iteration count (e.g. a base of 50,000) over-resolved the boundary's sub-pixel "dust"
  into per-pixel noise *and* consumed the entire GPU-watchdog budget (forcing low
  resolution and no anti-aliasing). Rendering now caps the iteration count at a
  **zoom-scaled** value (`~2000 + 256·octaves`) — generous enough that normal
  auto-iteration is never limited, but an inflated manual base is — applied to both the
  live view and exports so they match. Result: coherent structure instead of dust. When
  the budget still can't afford true supersampling, a color-pass box filter anti-aliases
  the settled view; at extreme depth it falls back to reducing resolution (box-filtered).
  The perf panel's "eff iter" now reports the count actually rendered.
- **Quick export froze the app / could crash on a deep view** — the export's reference
  orbit was built on the main thread (briefly freezing the UI), and `render_export` tiled
  by texture/buffer size only, so a single tile at a huge iteration count was an enormous
  GPU submission that monopolized the shared device (freezing the live UI) and could trip
  the OS GPU watchdog (TDR → device-lost). Exports now use the same zoom-appropriate
  iteration cap, and export tiles are additionally bounded by **iteration work** so each
  GPU submission stays short — the UI stays responsive and the watchdog never fires. (A
  3840×2160, 2× export of a deep view now finishes in a few seconds.)

- **Deep zoom lost on quit/restart (uniform screen after relaunch)** — the auto-saved
  session stored the center as `f64`, so restoring a deep view truncated the coordinate
  to ~16 digits → a wrong location → uniform. The session now persists the center as a
  full-precision decimal string (like bookmarks/exports) and restores via `parse_bf`,
  falling back to `f64` for old session files. Also fixed the autosave debounce, which
  reset its timer every frame so an animating palette offset blocked the idle save
  (it now saves ~every second). Plus **multi-scale reference search**: the perturbation
  reference picker sampled a single coarse grid over the full view, which on a wide
  window at deep zoom landed between the thin filaments → a useless reference → uniform;
  it now samples several scales concentrated toward the center.
- **Uniform render at extreme depth on a large window** — the GPU-watchdog budget
  (texels × iterations) was kept by *clamping the iteration count*; on a big window
  (>~1.2 Mpx, i.e. maximized or with Windows display scaling) at very deep zoom that
  capped iterations well below what the detail needs, so the whole view escaped
  "late"/never and rendered as flat interior. Now the budget is met by reducing the
  iteration-texture *resolution* (the color pass upscales it) while keeping the full
  iteration count — graceful softness instead of a blank. (`--render` was unaffected,
  which is why a bookmark looked detailed when exported but blank live.)
- **Uniform/blank render after a cold jump to deep zoom (bookmark reload, Open view,
  `--render`)** — `best_reference` ranked candidate reference points using `f64`
  coordinates, which all collapse to the same value at deep zoom, so the search
  defaulted to the view center; if that sat in a fast-escape gap it was a poor
  reference → glitchy/uniform. Gradual zoom hid this by carrying a good reference
  chosen at shallower depth. Reference candidates are now scored in **arbitrary
  precision** (`orbit_length_bf`, scan-capped), so cold jumps find a good reference at
  any depth. (Confirmed the bookmark *coordinate* itself round-trips to ~1e-79 — far
  sub-pixel — so it was never an imprecision; added round-trip tests as guards.)
- **Soft "impressionist" frames while zooming deep** — the high-precision reference
  orbit was only refreshed once the view *settled*, so during motion a stale /
  out-of-view / under-precise reference made the perturbation blotchy until you paused.
  It now also refreshes *during* motion when out of view or under-precise, throttled
  adaptively (~2.5× the last recompute's duration) so it stays sharp without stalling
  the frame-rate (smooth on the release build; throttle widens automatically on debug).
- **Julia deep-zoom rebasing** — rebasing reset `δz = z_full`, valid only when
  `reference[0] = 0` (Mandelbrot). Julia orbits start at `Z₀ = ref_point ≠ 0`, so every
  rebase offset the perturbation and corrupted deep Julia renders. Now rebases to
  `δz = z_full − reference[0]` (a no-op for Mandelbrot).
