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

- **BLA per-reference caching** — the acceleration tree is now cached per reference (rebuilt only
  when the reference orbit changes) instead of every frame, using a conservative view-diagonal
  `dc_max` that stays valid across pans. A settled deep view drops from ~35 ms/frame (build + render)
  to ~13.6 ms (render only) — the full ~5.4× — and the one-time tree build is now amortized like the
  reference orbit, removing the weak-CPU concern. Still opt-in (View menu) pending on-target
  verification before it's enabled by default.

- **BLA profiling tooling + measured verdict** — a `--bla` CLI flag forces BLA on for headless
  runs, the profiler now times the BLA tree build (`bla_build`) and labels runs with `use_bla`, and
  `scripts/profile-bla.ps1` runs BLA off-vs-on and breaks down the tradeoff (export / live /
  cached). Measured on an RTX 3080 / Ryzen 3950X: at 1e30× BLA cuts the GPU render **5.8×** (73→13
  ms) for a **~20 ms** tree build — **2.2× net even rebuilt every frame, 5.8× with caching**, and
  no cost where it doesn't apply. Verdict: enabling BLA by default is justified (per-reference
  caching is the remaining step).

- **BLA acceleration: user toggle + escape-path validation** — bilinear approximation (skip
  iterations throughout the orbit at extreme depth) is now a persisted **View-menu toggle**
  ("BLA acceleration (deep zoom)") instead of a hidden dev flag. Its GPU escape-overshoot revert
  is now validated: a new self-test renders a deep *boundary* view (48400 escaping pixels, 0
  mismatch vs BLA off) to complement the existing all-interior test — both code paths covered.
  Still off by default while the per-frame cost/benefit is measured (the acceleration tree is
  rebuilt each frame; per-reference caching is the next step before enabling by default).

- **Multi-reference glitch correction — now shipping for exports (phase 2c)** — a new
  **"Glitch correction (export)"** preference (View menu, persisted) makes single-view exports
  glitch-free: perturbation glitches are detected and those pixels re-rendered against extra
  references until clean. `color_iter_buffer` colors the merged buffer; `render_export_corrected`
  wires it into both the headless (`--render`) and interactive export paths. Applies to single-view
  exports up to the GPU texture limit with non-aux coloring; the live view is unaffected.
  (Follow-ups: tiling for larger exports, aux coloring methods, dual layouts.)

- **Multi-reference glitch correction — phase 2b (correction orchestration)** — `render_corrected_iter`
  renders the iteration buffer with detection on, then repeatedly places a fresh reference (bignum)
  at the largest glitched region and re-renders, adopting the newly-resolved pixels until nothing is
  glitched. Seeding at the exact pixel center guarantees convergence. A selftest resolves a
  seahorse-1e8× view's flagged glitches to **0 residual** with a handful of references. Next: color
  the corrected buffer and wire it into exports behind a preference.

- **Multi-reference glitch correction — phase 2a (GPU detection)** — the shader now detects
  Pauldelbrot glitches (`|z|² < tol·|Z|²`) in both perturbation paths (df32 + floatexp), flagging
  glitched pixels with a `-2` sentinel in the iteration texture (harmless when uncorrected — the
  color pass treats it as interior). Gated by a `glitch_on` uniform (off for live/normal render),
  plumbed via `ExportRequest`. Validated by a selftest that confirms detection fires and responds
  to reference quality. Next: the app-side multi-pass correction that consumes it.

- **Multi-reference glitch correction — phase 1 (core algorithm)** — the last real deep-zoom
  correctness gap. `fractadyne-core` gains the validated CPU algorithm: single-reference
  perturbation with Zhuoran rebasing **and Pauldelbrot glitch detection** (`perturb_pixel_mandel`,
  δz in f32 to mirror the GPU's high-precision-reference / low-precision-δz gap), plus
  `render_multiref_mandel`, which detects glitched pixels, places a new reference inside each
  glitched region, re-renders and merges, and repeats to convergence. Validated against a bignum
  per-pixel oracle at a real period-3 minibrot (induces glitches, converges with multiple
  references, matches ground truth). Follows the BLA playbook (correct core first); the GPU/export
  port is the next phase.

- **Pan reprojection (retain detail while dragging)** — dragging to pan no longer drops to the
  coarse moving preview (which shows no detail at deep zoom, so you couldn't see where you were
  going). Instead the last detailed frame is frozen and translated with the cursor in the color
  pass — no bignum recompute, no re-iterate — so the real image slides under the pointer. Only
  the newly-exposed edge is blank until you stop, at which point the view settles and re-renders
  at full detail. Applies to single and dual (left panel) views at deep zoom; the shallow direct
  path is already detailed so it renders normally.

- **Progressive iteration refinement (sharpen on settle)** — deep views no longer look
  permanently smooth. The Iterations slider now goes to 500,000 (was 50,000) and auto-scale's
  appetite climbs past 50k with depth. While you're moving, the preview caps iterations at
  50,000 with a tight work budget so motion stays responsive; the moment the view settles it
  re-renders at the full zoom-appropriate count (up to ~200k+ deep) with a ~6× larger budget —
  still well under the GPU watchdog — so the finest boundary filaments resolve on screen,
  matching an export. The live reference orbit is built only to the count actually rendered, so
  navigation speed is unchanged. A note appears only in the rare case a settled view is still
  resolution-limited (huge window at extreme depth), pointing to export for full resolution.

- **Recommended-hardware Help section** — GPU/CPU/memory guidance (what matters and why: the GPU
  drives per-pixel iteration + frame rate; the CPU's single-core speed drives the deep-zoom
  reference orbit) with minimum/recommended tiers.

- **Acknowledgments & citations (Help)** — a new Help section crediting the prior art Fractadyne
  builds on, each verified against its source: perturbation & series approximation (K. I.
  Martin), BLA + rebasing (Zhuoran), glitch detection (Pauldelbrot), non-analytic/Burning-Ship
  perturbation (laser blaster), reference implementations & cross-checks (Fraktaler-3 / Kalles
  Fraktaler 2+ by Claude Heiland-Allen, orig. Karl Runmo), smooth + stripe coloring (Jussi
  Härkönen), triangle-inequality average (Kerry Mitchell), the Mandelbrot set (B. Mandelbrot),
  and the libraries used. Includes a **dedication to the Stone Soup Group of Fractint**.

- **Bookmark thumbnails** — each saved bookmark now shows a small preview image in the
  Bookmarks (Manage) dialog. The thumbnail is rendered from the exact view at save time
  (small offscreen render) and stored as a PNG under `bookmark_thumbs/`; it's lazily loaded
  for display and cleaned up when the bookmark is deleted. The dialog now lists each bookmark
  as thumbnail + name + zoom + Go/Delete.

- **Minimap shown in dual view** — the "you are here" overview was hidden in dual view; it's
  now shown (it maps the left/Mandelbrot panel). Only a single Julia view still hides it,
  where a Mandelbrot overview wouldn't correspond to the shown set.

- **Zoom box (Shift+drag)** — hold Shift and drag a rectangle to zoom so it fills the view.
  The box is constrained to the panel's aspect ratio (fills exactly, no distortion), drawn as
  a live amber rubber-band overlay, and applied deep-zoom-correctly (recenter + scale via the
  arbitrary-precision viewport, so it's exact at any depth). Works in single and dual views; a
  tiny drag is ignored (treated as a click). (Replaces a "Right-drag box zoom" that Help
  documented but was never implemented.)

- **Duotone & binary palette modes** — Coloring → Palette gains two two-color modes sharing
  a pair of color pickers: **Duotone** maps the coloring value to a smooth Shadow→Highlight
  ramp; **Binary (set)** is a flat two-color view (one solid color inside the set, another
  outside, no gradient) — the clearest way to see the set's shape. The in-set (interior)
  color is now selectable (a new shader uniform; defaults to the previous near-black, so
  existing renders are unchanged). Persisted.

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
  parametrization). **Invariance/consistency** checks target the tier crossovers:
  resolution independence (N vs 3N — validates δc construction), max-iter monotonic
  stability, zoom-sequence consistency across the direct→df32 seam, pan consistency, and
  render determinism. **Derivative checks** validate the `dz/dc`-derived distance estimate
  independently of dwell: DE self-consistency (a boundary-adjacent pixel can't claim a far
  boundary) and the Koebe-¼ lower bound (a disk of radius DE/4 is boundary-free, verified
  against an independent CPU dwell). **External checkability:** a committed, human-readable
  **location catalog** (`validation/catalog.toml`) of full-precision coordinates with
  independently-known answers (period + nucleus, set membership) that `--selftest` verifies
  — doubling as published *challenge coordinates*; and a **Coverage & scope** section in the
  report stating exactly what each oracle checks and, importantly, where the deep regime is
  *not* independently oracle-checked. **Fuzz tests** (dependency-free, deterministic) hammer
  the untrusted-input parsers — the arbitrary-precision coordinate parser and the
  view-metadata parser chain — asserting they never panic on random/adversarial/oversized
  input. This hardened `parse_bf` to also reject non-finite (±∞/NaN) coordinates.
  **Comparison tooling:** `--render-iter` exports the raw iteration texture as an EXR
  (R=smooth iteration, G/B=slope normal, A=log₂ DE-in-pixels) with a documented layout, so
  a reviewer can diff iteration data directly (no coloring confound); and `--compare A B`
  reports max/mean per-pixel difference (channel-0 iteration data + finite all-channels) and
  writes a difference heatmap — for A/B against another build or imported renderer data.
  **Cross-renderer import:** Locations → "Import .kfr…" (and `--import-kfr FILE`) loads a
  Kalles Fraktaler location via a **hardened, fuzzed** parser (size/length-bounded, strict
  key allow-list, every field validated/clamped, no paths/code) — so the identical
  coordinate can be opened in a trusted third-party renderer for the strongest external
  cross-check. Verified bit-identical to a direct render of the same coordinates. **Golden-image regression**: `--selftest --bless` records
  reference PNGs under `validation/golden/`; subsequent runs diff against them with a pixel
  tolerance. Every run writes a **readable, verifiable Markdown report**
  (`validation/report.md`) with full provenance (version, GPU, CPU, OS), each check's
  parameters/result/threshold/verdict, golden checksums, and the exact `--render` command
  to reproduce each golden — so a third party can independently re-run and confirm.

- **Cross-renderer cross-check (Fraktaler-3)** — `--crosscheck-f3 raw.exr --center X Y
  --zoom-f3 Z [--iter K] [--er R]` validates against **Fraktaler-3** (Claude
  Heiland-Allen's independent GPU-perturbation renderer) at the *iteration* level. F3's raw
  EXR carries the integer escape count in a `UINT` channel `N` (exterior `n + 1024`,
  interior `0xFFFFFFFF`); we recover each pixel's exact `c` from F3's documented pixel
  mapping — including replicating its deterministic triangular sub-pixel **jitter**
  (`burtle_hash`/`triangle`, applied even at `subframes = 1`) and the vertical EXR flip —
  and compare F3's count to our independent arbitrary-precision **CPU bignum dwell oracle**
  (the same oracle `--selftest` checks our GPU pipeline against, so the results compose
  transitively into `our GPU ≈ Fraktaler-3`). Boundary/max-iteration-cliff pixels are
  excluded as genuinely ULP-ambiguous. Measured: **100%** interior/exterior membership and
  **100%** of exterior counts agree to within one iteration (≈79% exact; the residual ±1 is
  the `≥`-vs-`>` escape-test convention at band edges), holding undiminished at **10⁶×**
  zoom. New `fractadyne-export::read_exr_channel_f32` reads an arbitrary named EXR channel
  (UINT/F16/F32 → f32). Reproduction recipe + results table:
  [validation/crosscheck-fraktaler3.md](validation/crosscheck-fraktaler3.md). (Uses an
  external F3 EXR by design; kept entirely separate from `--selftest`, which uses no
  external data.)

- **Extreme-depth precision validation** — `--validate-deep [--out report.md]` validates the
  arbitrary-precision arithmetic core at magnifications far beyond `f64` range — **1e1000×,
  1e10000×, 1e100000×, and 1e1000000×** (≈3.3-million-bit precision). With no external
  corpus at this depth it uses the standard precision-doubling technique: iterate `z²+c`
  from a full-mantissa interior point (seeded by `√½` so the multiply exercises real carries
  across every limb) at precision `p` and again at `p+256`, and require the results agree to
  ≈`p` bits, plus a decimal `to_string → parse` coordinate round-trip. Feasible because
  `astro-float` switches to **FFT multiplication** above ~5400 limbs (measured ~32 ms per
  iteration at 3.3 M bits — near-linear, not quadratic) and the check is **single-point**
  (a per-pixel dwell oracle would take years that deep). New core API:
  `precision_for_octaves` (bypasses the `f64` magnification overflow), `deep_consistency_bits`,
  `deep_roundtrip_bits`; new `fractadyne-core` tests (`deep_precision_self_consistent_1e1000`,
  plus an `#[ignore]`d 1e100000× case). This surfaced that the renderer's **live** zoom is
  capped near **1e308×** by the viewport's `f64` `units_per_pixel`/`magnification` (the
  bignum *center* is unlimited; the *scale* underflows) — tracked in TODO as a floatexp /
  log-magnitude scale rework. Recipe + measured cost-scaling table:
  [validation/extreme-depth.md](validation/extreme-depth.md).

- **Lifted the ~1e308× live-zoom ceiling (extended-range scale)** — the viewport scale was
  an `f64` (`units_per_pixel`), so it underflowed (and `magnification()` overflowed) near
  **1e308×** — the real live-zoom wall, even though the arbitrary-precision *center* never
  ran out of digits. Introduced `FloatExp` (`m · 2^e`, `i32` exponent) and made
  `Viewport::units_per_pixel` use it, with `log2_magnification` + `precision_for_octaves`
  driving precision past f64 range, `complex_span_fe` / `gpu_scale` (an O(1) span mantissa +
  shared `delta_exp`) and `ref_offset_mantissa` feeding the GPU, `set_center_log2mag` +
  `--render --zoom-log2 L` for deep jumps, an extra session field so deep locations persist,
  and `fmt_zoom_log2` for the readout. **The WGSL shader was already exponent-aware (it
  consumes mantissas + `delta_exp`), so it needed no change** — the fix was entirely
  CPU-side. Verified: **bit-identical** to the previous build through 1e30× (selftest
  goldens, maxΔ 0), the GPU renders correctly at **1e331×** (interior/exterior classified
  exactly), 28 `fractadyne-core` tests pass (incl. a new past-1e308 scale test), and
  `--selftest` stays 29/29 + 4/4. (Follow-ups: the goto dialog and exported-image metadata
  still take `f64` zoom — fine to ~1e308×.)

- **Deep zoom save/restore/goto past 1e308×** — completes the ceiling lift so the deepest
  views are fully round-trippable. The "Go to location" dialog now parses and displays zoom
  via `log2(magnification)` (`parse_zoom_to_log2` / `fmt_zoom_field`: accepts plain or
  scientific input like `1.5e400`, grouping-tolerant, clamped to a sane octave bound — no
  more `inf` readout or f64 truncation). The reloadable image metadata carries an
  extended-range `upp_log2` (reconstructed on load; the f64 `upp` stays for back-compat and
  readability), so **exported PNG/EXR images and bookmarks restore views deeper than 1e308×
  exactly**. Round-trip unit-tested (shallow through 1e30000×).

- **Auto-zoom autopilot** — hands-free continuous deep zoom that re-steers toward detail
  (XaoS-style), via **View → "Auto-zoom (autopilot)"** or the **A** key; **Esc** or any
  navigation input stops it. Every ~0.35 s it renders a small (56×56) iteration field of the
  current view through the live perturbation pipeline (so it works at any depth) and scores
  each cell by **boundary adjacency + escape-time gradient**, center-biased for a stable
  dive. The zoom pivot **eases toward the evaluated goal every frame** (time-constant
  smoothing) rather than snapping at each re-evaluation, so the pan direction changes
  smoothly instead of jerking; it zooms toward that gliding pivot each frame (reusing
  `zoom_at` + the continuous-zoom rate), treating the dive as interaction (AA off, throttled
  reference refresh). Stops on a dead end (no boundary detail in view) or at a depth cap
  (~1e271×).

- **Shareable `.fdn` locations** — **File → "Share location…"** opens a dialog showing the
  current view as a self-contained text blob (fractal, full-precision center, the
  extended-range `upp_log2` so depths past 1e308× round-trip, zoom, coloring): **Copy** it to
  the clipboard, **Apply** a pasted/edited one, or **Save .fdn… / Load .fdn…**. So an exact
  location/look is shareable as a short text snippet or a tiny file. Untrusted input is
  handled safely — size-bounded (a 256 KB cap plus a file-size check) and parsed through the
  existing **hardened, fuzzed** `load_view_metadata`/`meta_get` chain (key=value allow-list,
  every field validated/clamped, unknown keys ignored, no paths or code execution).

- **Series approximation (iteration-skipping)** — deep Mandelbrot renders (mode 2, ≥1e28×)
  now skip the early perturbation iterations by seeding `δz` from an order-3 polynomial
  `δz ≈ A·δc + B·δc² + C·δc³`. The coefficients are iterated in arbitrary precision alongside
  the reference orbit (`A'=2ZA+1, B'=2ZB+A², C'=2ZC+2AB`); the skip is the largest count where
  the cubic term stays `≤2⁻¹⁶` of the linear term for the worst-case corner `|δc|` (which also
  guarantees no pixel escapes before the skip), cached per reference. The GPU evaluates the
  polynomial in floatexp to seed `δz` and the derivative `D`, then iterates from `skip`.
  Disabled for Julia, non-Mandelbrot formulas, and aux-accumulating coloring methods
  (stripe/TIA/orbit-trap/decomposition need every iterate). **Validated:** the seeded render
  matches full iteration (`maxΔ 0`) and the independent bignum oracle (0 mismatches) at 1e30×
  — a new `--selftest` check confirms both engagement and equivalence. Default on (toggle in
  View → "Series approximation"); the perf panel shows the skip count. Mode-0 (1e4–1e28×) and
  other formulas are follow-ups (see TODO).

- **Development profiling harness** — `--profile [--reps N] [--regions FILE] [--out PATH]`
  renders a set of benchmark **regions** (built-in defaults spanning the regimes: direct,
  df32 perturbation, floatexp + series approximation, plus a stripe variant that disables SA)
  and times the costly stages separately — bignum **reference orbit**, **series-approximation**
  setup, and the **GPU iterate / full render** passes — then writes a structured **JSON log**
  to `logs/` with full run context (version, GPU, CPU, OS, settings) plus per-region
  min/median/mean/max. Surfaces bottlenecks at a glance (e.g. at 1e30× the smooth render is
  ~3× faster than the SA-disabled stripe one, while the series-skip setup is itself a
  measurable cost). `scripts/profile.ps1` runs it; `scripts/profile-compare.ps1` diffs a
  before/after pair (per-stage % change, flags regressions) to validate optimizations;
  `scripts/regions.example.toml` is an editable region set. Logic lives in a new
  `profile` module (keeping `main.rs` glue lean); opt-in, so zero overhead in normal use.

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

- **Deep zoom for the abs families (Burning Ship / Celtic / Buffalo)** — the non-analytic
  families now perturbation-deep-zoom like the analytic ones (previously direct only,
  ~10⁶×). Because they take absolute values, the abs fold on a z² component is handled with
  the Kalles-Fraktaler **`diffabs`** identity `|c+d|−|c|`, evaluated branch-wise against the
  reference z² so it never suffers catastrophic cancellation: exactly `±d` when the
  reference component and its perturbation share a sign, `±(2c+d)` across a sign flip. Both
  render paths fold: `df_diffabs` in the df32 loop (mode 0, ~10⁴×…~10²⁸×) and a new
  **scalar-floatexp** `Sf` type with `sf_diffabs`/`fe_from_sf` in the floatexp loop (mode 2,
  past 10²⁸×) — needed because the complex `Fe` shares one exponent across re/im while the
  fold is per-component. Core `step_bf` gained the bignum reference iterations. `--selftest`
  verifies perturbation == direct at 1e5×, floatexp == df32 at 1e10×, and finiteness at
  1e35×. Lighting/DE stay off (these maps are non-holomorphic). Residual fold speckle awaits
  multi-reference glitch correction.

- **View-file format versioning + hardened loader** — the reloadable view metadata (exports
  / `.fdn` / bookmarks) gained a single source-of-truth `VIEW_FORMAT_VERSION`; loading now
  returns a report and **surfaces anything noteworthy** instead of loading silently. Opening
  a file from a *newer* Fractadyne warns "some settings may not apply — consider updating"
  (best-effort load; the format is additive key=value so core fields still parse); the
  loader also reports **clamped** out-of-range fields and **ignored unknown** keys. The
  untrusted parser is hardened: `max_iter` ≤ 10⁷, anti-aliasing 1..16, zoom depth ≤ 3.4e7
  octaves (a hostile `upp_log2` can't balloon bignum precision into a memory DoS), and
  `cycle`/`offset` rejected when non-finite. A file with no `format_version` is treated as
  v1 (legacy files still load). `--selftest` covers round-trip, newer-version detection, and
  clamp/report.

- **Depth-aware status-bar readouts** — the center coordinate now shows full arbitrary
  precision with the changing **frontier** digits visible: at deep zoom the middle is elided
  (`-0.74364 38870 … 06114 7740`) so the deepest digits no longer freeze at `f64`'s ~15
  (they used to look static while panning deep). The magnification's scientific-notation
  mantissa is space-grouped in 5s (`3.38050 02722 7e15`) to match the coordinate readout.

- **Series approximation on the df32 path (mode 0)** — the iteration-skipping seed, previously
  floatexp-only (mode 2, ≥1e28×), now also accelerates the common df32 perturbation range
  (1e4–1e28×). The order-3 polynomial seed is evaluated in floatexp (the coefficients overflow
  f32) then collapsed to the absolute df32 δ that path carries (`fe_to_cdf`); coefficients are
  mode-independent (computed once in bignum). Validated to reproduce full iteration exactly
  (max Δ 0) at 1e20× — skipping 19007 of 19008 iterations at a deep minibrot.

- **Series approximation for the Multibrot families** — SA now also accelerates Multibrot
  3/4/5, not just Mandelbrot. The order-3 coefficient recurrence is generalized to `z^d+c`
  (`A'=d·Z^{d-1}·A+1`, etc., with binomial weights); the GPU seed is already formula-agnostic.
  Validated by a core test (series vs exact perturbation for z³, rel err <1e-3) and a GPU
  check that SA engages and matches an SA-off render for all three families. (Tricorn and the
  abs families have no such δc expansion.)

- **Zoom-movie / frame-sequence export** — `--render-tour FILE [--fps N] [--size W]
  [--height H] [--ss N] [--out DIR]` renders a keyframe-tour TOML to a numbered PNG frame
  sequence (`frame_00000.png …`) for assembly into a deep-zoom dive video (prints an ffmpeg
  one-liner; example in `scripts/tour.example.toml`, also loadable via Tools → Play script).
  Reuses the scripting keyframe interpolation (factored into `Playback::sample`, shared with
  live playback) and the offscreen export path; samples the timeline at a fixed fps and
  recomputes a fresh deep reference per frame. Deep-correct (`set_center_log2mag`,
  octave-based precision) so dives past 1e308× sample exactly — which also fixed live
  playback (it used `set_center_mag`, saturating at 1e308×).

- **Prebuilt binaries via GitHub Releases** — `.github/workflows/release.yml` builds the
  Windows x64 binary and, on a `v*` tag push, packages `fractadyne.exe` + README + licenses
  into a versioned zip with a SHA-256 sidecar and publishes a GitHub Release (auto-generated
  notes) via the `gh` CLI. A manual `workflow_dispatch` run uploads the zip as a test
  artifact instead of publishing. Users can now download and run without the Rust toolchain
  (README gained a **Download** section).

- **Continuous integration** — `.github/workflows/ci.yml` gates every push/PR with the
  exact-math core test suite (`cargo test -p fractadyne-core`, Linux) and a full
  `cargo build --workspace` (Windows) confirming the GPU/egui crates still compile. The GPU
  `--selftest` stays a local/manual gate (runners have no GPU).

### Fixed (post-baseline, this session)

- **More Help polish** — the content now scrolls when it overflows (the window was growing to
  fit and pushing content off-screen; its height is now capped so the scroll area engages);
  the key column in shortcut/flag tables is left-aligned (was centered); and math glyphs that
  the default font lacked (→, ≪, super/subscripts) now render via a bundled fallback font
  instead of showing as tofu boxes.

- **Help window layout was broken** — the table-of-contents + content were hand-split in a
  horizontal layout with manual width math that egui didn't honor, so the content ran off
  sideways and paragraphs wrapped to one character per line. Rebuilt with the standard
  `SidePanel` + `CentralPanel` idiom so the content is width-bounded and wraps normally, with
  a proper vertical scroll.

- **Minimap "you are here" marker was invisible / missing** — the amber marker had almost no
  contrast against a warm-palette thumbnail (e.g. Ember), and it was only drawn when the view
  center fell inside the minimap's fixed region. Now it always shows (clamped to the thumbnail
  edge if the view is outside the region) and is drawn with a dark halo behind the bright
  marker so it reads on any palette — a view rectangle when shallow, a crosshair + centre dot
  when deep.

- **Frame-rate cap "Uncapped" wasn't persisted** — the cap was stored as an `Option<f64>`,
  and TOML omits `None`, so the *uncapped* choice was dropped and reloaded as the default 60
  every restart. Now stored as a plain `f64` (`0` = uncapped) that round-trips. While auditing
  persistence, also added the missing view/preference state to the saved session so restart
  fully restores where you were: **fractal family, Julia mode + parameter `c`, dual view, and
  the series-approximation toggle** (center/zoom/coloring/lighting/export settings already
  persisted). Round-trip + legacy-file tests added in `fractadyne-state`.

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
