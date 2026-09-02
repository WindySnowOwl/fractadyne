# Fractadyne — Architecture (as-built)

**Version:** 0.2.41 (unreleased) · **Updated:** 2026-09-02

This document describes the system **as it is actually implemented**. It is the counterpart to
[`DESIGN.md`](DESIGN.md), which is the *original design intent* (2026-06-25) and has diverged from
the code in several places. Where the two disagree, this document is authoritative for the current
state; [`CHANGELOG.md`](CHANGELOG.md) is the running per-version log.

> **Divergence summary (design → as-built):** the `Fractal`/`RenderStrategy` trait abstraction was
> not built (formulas are a `FractalKind` enum switched on by hand-written per-path `match` arms);
> the `-render`, `-ui` and `-fractals` stub crates were
> retired (render orchestration lives in `fractadyne-app`; UI is split under
> `fractadyne-app/src/ui/`); the live view
> uses **full-frame render + reprojection freeze**, not a per-tile
> RAM cache; `rayon` is not used (off-thread work is `std::thread`); and the programmable formula
> DSL, L-systems, cellular automata, and histogram coloring are **not implemented** (roadmap).

---

## 1. What it is

A native **Windows** desktop fractal explorer (Rust + `wgpu`/`egui`/`eframe` 0.31), focused on
**extreme deep zoom** of escape-time fractals and on **correctness you can verify**. Binary:
`fractadyne` (crate `fractadyne-app`).

Deep zoom is bounded by coordinate precision + iteration/compute budget, not a fixed wall:
renders match **Fraktaler-3** across a 38-location reference corpus up to **~6.1e1105×**
(pixel-exact against F3's raw iteration counts where directly comparable), a bundled tour dives
live to ~1e838× (and generated dives beyond 1e1200×), and the arbitrary-precision core is
self-consistency-validated to 1e1000000×.

---

## 2. Crate layout

A 6-crate Cargo workspace under `crates/`. (The earlier `-ui` / `-fractals` stubs, and later the
`-render` stub, were retired; render orchestration can return as its own crate if that refactor
lands — its intended responsibility currently lives in `fractadyne-app`.)

| Crate | Status | Responsibility |
|-------|--------|----------------|
| `fractadyne-core` | ✅ | Numerics: `Viewport` (BigFloat center, `FloatExp` scale), reference-orbit iteration (`step_bf`, `reference_orbit`), `best_reference`, series approximation (`series_skip`), BLA (`build_bla_mandel`), precision helpers, the CPU dwell oracle, minibrot/nucleus finder. No GPU/UI. **Decomposed into submodules** — `floatexp`, `bignum`, `viewport`, `reference`, `backend` — behind a thin `lib.rs` facade that re-exports the public API. The orbit loop is generic over a `RefBackend` (`backend.rs`), so the arbitrary-precision library under it can be swapped: `astro-float` always, and MPFR via `rug` behind the off-by-default `rug` feature (`backend_rug.rs`). Dispatch happens ONCE per orbit build, never per operation, and the two backends are byte-identical — which is what lets one set of goldens gate both. |
| `fractadyne-gpu` | ✅ | `wgpu` device, render pipelines, WGSL shaders (`mandelbrot.wgsl` — iterate + color), `render_export` (tiled), per-view resources. |
| `fractadyne-color` | ✅ | Preset gradient palettes + interpolation. |
| `fractadyne-state` | ✅ | Session persistence (`session.toml`), versioned state, `config_dir()` + `FRACTADYNE_CONFIG_DIR`, reset. |
| `fractadyne-export` | ✅ | PNG/OpenEXR encode/decode + embedded view metadata. |
| `fractadyne-app` | ✅ | **Everything else** — app struct, UI, input, scripting, CLI, autopilot, coloring/mode logic, `FractalKind`. Split into modules (below). |

**`fractadyne-app` modules:** `main.rs` (entry + app struct + `update()` frame loop, in
banner-sectioned reading order), `render.rs` (mode select + reference recompute / reuse /
freeze-reproject + export requests + the staged `build_params` frame builder), `autopilot.rs`
(auto-zoom + `--autodive`), `scripting.rs` (tours + `render_tour_to_dir`), `help.rs`
(`CLI_REFERENCE` + Help window), `cli.rs` (headless commands, the CLI-launched mode-state
structs, and `update()`'s harness hooks + mode ladder), `export.rs` (view-metadata / `.fdn`),
`tunables.rs` (every frame-cost constant; `--set` overrides), `fractal.rs`
(`FractalKind`), `refcache_persist.rs` (persist/restore the deep-zoom reference), `error.rs`
(`AppError`), `selftest.rs` (GPU validation), `theme.rs`, `profile.rs` (profiling + the
`--frametest`/`--divetest` harnesses), `livetest.rs` (the live-vs-offline output harness),
`sysinfo.rs`, `diag.rs` (log/crash/watchdog/trace), `alloc.rs` (allocation-failure hook),
`update.rs` (GitHub-Releases update check, Stable/Beta tracks), `bench_matrix.rs` (the
`--bench-matrix` path-coverage perf/regression suite), `motiontest.rs` (`--motiontest`:
the motion-presentation gate), `chunksweep.rs` (`--chunk-sweep`), `torture.rs` (`--torture`
escalation suite), `soak.rs` (`--soak` liveness), `shot.rs` (`--shot` screenshot regen),
`tone.rs` (finish sound), `icons.rs` (generated Lucide subset), `gputest.rs` (`--gputest`: the WGSL
df32/floatexp primitives against CPU oracles, swept over every backend — the harness that found
NVIDIA's shader compiler folding the error-free transforms), `uitest.rs` (`--uitest`: the scripted
UI + live-render walk), `reusetest.rs`, and a `ui/` submodule tree (`central.rs`, `menus.rs`,
`panels.rs`, `dialogs.rs`, `tour_render.rs`, `diagnostics.rs`) from the intra-crate UI split.
Unit tests live in sibling files (`#[cfg(test)] mod name;` beside the code under test — see
`CONTRIBUTING.md` for the layout and item-order conventions).

---

## 3. The fractal system (no trait, but a single metadata table)

There is **no `Fractal`/`RenderStrategy` trait**. Instead, [`fractal.rs`](crates/fractadyne-app/src/fractal.rs)
defines a `FractalKind` enum (10 families) with an integer **`formula_id()` (0–9)** that every layer
switches on. Families: Mandelbrot, Multibrot 3/4/5, Tricorn, Burning Ship, Celtic, Buffalo, Phoenix,
Newton. Julia mode is an orthogonal flag on any family that supports it.

All the app-side per-family metadata — `name`, `formula_id`, `default_center`, `supports_julia`,
`supports_perturbation`, `info` — lives in **one `FractalKind::SPECS` table** (one row per family);
the accessors read from it, so the app side of adding a formula is a single row (guard tests enforce
row order and `formula_id == index`). The numeric id numbering is a single source of truth in
**`core::formula::{MANDELBROT…NEWTON, COUNT}}`**, adopted at every core dispatch site (arms read
`formula::PHOENIX => …`); the WGSL carries a matching id legend.

The per-iteration step for each family is still written **six times**, once per numeric
representation, all keyed on the formula id — this is the irreducible cost of no trait / no DSL:

1. `core::reference::step_bf` — bignum reference orbit (exact).
2. `core::reference::orbit_points` — f64 cursor-overlay orbit.
3. `mandelbrot.wgsl` mode 1 — direct df32 (shallow).
4. `mandelbrot.wgsl` mode 0 — df32 perturbation δz.
5. `mandelbrot.wgsl` mode 2 — floatexp perturbation δz.
6. `core::reference::series_skip` — the SA coefficient recurrence (polynomial families only; generic in degree).

Adding a formula means editing all of these (plus the SPECS row). An authoritative **"Adding a new
formula" checklist** — mapping every edit site across app → core numerics → shader — lives in the
`fractal.rs` and `core` module docs. (A formula DSL that would generate all paths from one definition
is designed in `DESIGN.md` §8 but **not built**; a `Fractal` trait unifying the two CPU step paths
remains a possible future step.)

---

## 4. Deep-zoom engine

Perturbation: a bignum **reference orbit** `Zₙ` is computed on the CPU near the view center; every
pixel `c = c₀ + δc` iterates the perturbed recurrence `δz' = 2Z·δz + δz² + δc` (per-formula variant)
on the GPU in low precision. The engine switches representation by **depth** (`build_params` /
`current_export_request_for` in `render.rs`; `PERT_FE_THRESHOLD = 1e28`):

- **mode 1 — direct df32:** `< 1e4×`, or any non-perturbation formula (Newton). Glitch-free.
- **mode 0 — df32 perturbation:** `1e4× … 1e28×` (the common deep range).
- **mode 2 — floatexp perturbation:** `≥ 1e28×`. δ carried as a df32 mantissa + `i32` exponent
  (`Fe` type in the shader) so it never underflows f32's exponent range. ~1.7× costlier, used only
  when needed.

Layered on top:

- **Extended-range viewport scale.** `units_per_pixel` is a `FloatExp` (`m·2^e`), which lifted the
  old ~1e308× f64 scale wall. A shared base-2 `delta_exp` keeps the GPU's input δ mantissas O(1) at
  any depth (the WGSL is exponent-aware).
- **Series approximation (SA):** order-3 `δz ≈ Aδc + Bδc² + Cδc³` seeds the perturbation and skips
  the early iterations. Polynomial families (Mandelbrot + Multibrot 3/4/5), both perturbation modes,
  non-Julia. Coefficients iterated in bignum alongside the reference.
- **BLA (bilinear approximation):** a binary tree of merged linear maps that skips iterations
  *throughout* the orbit (Zhuoran/KF style). **Mandelbrot mode-2 only, on by default** (~5× faster
  GPU render at 1e30×). Appended into the same orbit storage buffer. Per-node **aux aggregates** let
  orbit-trap / TIA / stripe coloring ride the BLA (folded O(1) on a skip, ~146–150× faster than
  dropping the skip); where the BLA is active it subsumes SA's early skip. **Deep-exterior
  exception:** for a *short escaped* reference the BLA is kept but SA is forced back on — otherwise
  "BLA subsumes SA" leaves an early-iteration perturbation glitch exposed and the view tiles.
- **Zhuoran rebasing:** single-reference glitch handling (rebase to `δz = z_full − reference[0]`;
  the `−reference[0]` term is required for Julia, a no-op for Mandelbrot).
- **Reference selection:** `best_reference` picks a long/interior reference (scored in bignum);
  the candidate scoring — the dominant cold-recompute cost at depth — fans out across **all CPU
  cores** via scoped threads (result-identical to the sequential scan; ~12–14× at 1e400–1e1216×).
- **Precision** auto-scales with zoom octaves (`precision_for_octaves`), +64 guard bits.

**Reference recompute is off the render thread** — including the cold start (progressive
coarse-then-full). The bignum reference + SA + BLA bundle (`recompute_worker`) is the deep-zoom
stall, so it runs on a spawned `std::thread`; the render keeps drawing with the cached reference
and installs the fresh one when it lands. A deeper rebuild **reuses** the cached orbit — it
*extends* the stored bignum prefix from a saved tail (byte-identical) instead of recomputing every
step, since the orbit build is ~90% of a deep frame (~20× faster dive-rebuilds). The last deep
view's reference also persists across sessions (`refcache_persist.rs`) so it resumes instantly.

**Script-playback reference lookahead + pacing.** A tour knows its future camera path, so during
playback a small queue of workers pre-builds the references the dive is about to need
(`playback_ref_prefetch` in `render.rs`: targets bisected onto the script's future zoom curve,
0.5-octave spacing so the active reference's lag never clips the pacer/freeze thresholds), and each
installs seamlessly as the dive reaches its validity window. When the pipeline still lags
(`last_depth_lag`), the tour clock **dilates** and interactive zoom-in velocity damps
(`PACE_LAG_LO..HI`) — the dive slows rather than blurring into stale reprojection.

**Freeze / reproject on motion.** While the reference is stale for the current depth (`depth_lag`)
or a fast dive would spin the mode-2 shader, `render.rs` holds the last good iteration texture and
**reprojects** it (scale + translate in the color pass, `uv_scale`/`uv_off`) so the view keeps
moving smoothly until the fresh reference snaps in. This replaced the "Not Responding" hang; it is
frame-level, not tile-level. The reuse-hold expires on **0.5 octaves of zoom or ~150 ms, whichever
first** — so slow deep dives still stream real detail. Moving-frame resolution is **adaptive**
(`perf.motion_res`): it adapts only on the raw interval following a **real re-iterate** frame
(reprojection frames carry no iterate-cost signal), cutting proportionally toward a ~vsync budget
and growing gently when real frames run cheap, floored at the user's `min_motion_res`.

---

## 5. Rendering & interaction

- **Full-frame, not tiled, for the live view.** Each settled frame iterates to an offscreen
  `RGBA32F` iteration texture; **coloring is a separate shader pass** over that texture, so palette /
  method / lighting changes re-shade instantly without re-iterating (the core compute↔coloring
  split). There is **no per-tile RAM cache**; pan/zoom during motion uses the frame-level
  reprojection freeze above. (Export *is* tiled — §8.)
- **Quality on settle:** anti-aliasing (SSAA 1–8×) runs only when the view settles; motion stays
  smooth. A `WORK_BUDGET` (texels × iterations) auto-reduces supersampling on heavy frames to stay
  under the GPU watchdog (TDR) — important for deep dual-view renders.
- **Frame-cost control (TDR safety).** Every dispatch is priced in nominal steps (px·ss²·iter)
  against a per-view budget learned from **measured GPU timings** (`TIMESTAMP_QUERY`, with a
  wall-clock fallback when timestamps starve). Its actuators: motion frames shrink the
  iteration-texture resolution; a settled frame that exceeds one dispatch budget runs either a
  **tiled settle** (a grid of bounded dispatches, revealed whole) or — where a spatial split
  cannot bound the cost — a resumable **iteration-range chunked walk**
  (`fs_iterate_chunk`/`fs_resolve`), price-serialized so at most one unpriced pass is in
  flight. A **present gate** serves the last complete frame while compose work runs
  underneath. The constants live in `tunables.rs` (`--set NAME=VALUE` per run for field
  experiments; `--selftest` refuses to pass under overrides).
- **Auto-zoom autopilot** ([`autopilot.rs`](crates/fractadyne-app/src/autopilot.rs)): renders a small
  56×56 iteration field, steers toward the boundary/gradient-richest cell, and dives. Smooth glide up
  to a "smooth regime" (log₂ 900 ≈ 1e271×); past that it switches to a **stepped dive** (jump ×4 →
  render a real frame → hold it while the next computes) with adaptive re-evaluation, so it reaches
  extreme depth without blanking. A user **dive limit** (Navigation slider, persisted, 1e30×–1e5000×)
  sets where it stops; A / 🛸 toolbar button / Esc control it.

---

## 6. Coloring

Compute↔coloring split (§5). Implemented methods (`color_method`, 6): **smooth**, **stripe
average**, **triangle-inequality average (TIA)**, **orbit trap** (point/cross/circle), **distance
estimate**, **decomposition**. Plus **duotone** and **binary** two-color modes. Palettes: preset
gradients (`fractadyne-color`), a custom multi-stop gradient editor (≤8 stops), cyclic with
cycle/offset, and palette animation (Off / Forward / Reverse / Ping-pong / **Random gradients**).
**Relief lighting** (distance-estimate slope normal) and **distance glow** re-shade in the color pass
so they don't re-iterate; both hold at any depth via the floatexp derivative.

*Not built:* histogram/equalized coloring (needs new GPU compute infra) and programmable/custom
coloring algorithms (`DESIGN.md` §6.4/§8).

---

## 7. Dual linked views

`View → Dual view` renders Mandelbrot (left) ↔ its Julia (right) with per-view resources
(texture/uniforms/orbit/reference cache keyed by `view_id`). Hovering the map drives the Julia `c`
live; click pins it. Each panel pans / wheel-zooms / deep-zooms independently; a draggable splitter
sets the split. This is hard-wired to Mandelbrot↔Julia (no generic `PlaneLinkage` abstraction).

---

## 8. High-resolution export

`render_export` tiles the target into ≤2048-px tiles (sized to texture/buffer limits) with a per-tile
offset, assembled on the CPU — so large sizes don't hit GPU limits (verified seamless in the low
thousands of pixels per side; there is no hard cap, but the "≥64k×64k validated" claim in DESIGN.md
is aspirational). Runs on a background worker with progress + cancel. **PNG** (8-bit sRGB) and
**OpenEXR** (32-bit linear) by extension; the full view state is embedded as reloadable metadata
(incl. `upp_log2` so depths past 1e308× round-trip). **Multi-reference glitch correction**
(Pauldelbrot criterion + iterative re-reference) runs on the export path (single + dual). A gallery
browser reads the metadata back.

---

## 9. Scripting, tours & movie export

Keyframe **camera tours** (TOML): eased center/log-magnification interpolation (`Playback::sample`),
timed captions, coordinate-anchored callouts, and spotlight vignettes — rendered live and burned into
exported frames. `--render-tour` renders a tour to a numbered PNG sequence (reference/render/encode
pipelined across frames), optionally assembled to an **mp4** via ffmpeg (`--mp4`), with an optional
zoom/coordinate HUD (`--show-location`). Interrupted renders resume with **`--resume`** (keep frames
on disk, render only the missing ones). Helper scripts: `scripts/{render-spiral-dive,render-deepest,
setup}.ps1`. **Tools → "Script to current view…"** generates a dive tour to the current view (deep
targets get a pan-shallow-then-dive structure so every deep frame stays centered on the target).
Live playback is **pipeline-paced** (clock dilation on reference lag) and feeds the reference
**lookahead queue** — see §4.

---

## 10. Persistence

Stored in the OS per-user config dir (`FRACTADYNE_CONFIG_DIR` overrides it):

- **`session.toml`** — full working state, auto-saved (debounced, atomic temp+rename). **Versioned**
  (`state_version`): a file from a newer build loads best-effort and warns. Center stored as
  full-precision decimal strings + a `FloatExp` scale exponent so deep locations survive restart.
- **`bookmarks.toml`** + **`bookmark_thumbs/`** — saved locations + thumbnails.
- **`.fdn` share files** — self-contained key=value locations (copy/paste/save/load); parsed through
  the hardened, fuzzed metadata reader (allow-list, every field validated/clamped).

**Reset:** `File → Reset application state` (confirmation dialog) or `--reset-state` (terminal
confirm) removes the whole config dir.

---

## 11. Command-line interface

`fractadyne --help` prints the always-current reference (generated from `CLI_REFERENCE` in
`help.rs`). Headless modes include: `--render` (one image), `--render-tour` (movie frames),
`--benchmark` / `--benchmark-std`, `--selftest`, `--validate-deep`, `--crosscheck-f3`, `--compare`,
`--render-iter`, `--find-minibrot`, `--import-kfr`, `--check-updates`, `--reset-state`,
`--gputest` (WGSL primitives vs CPU oracles, every backend — headless, no display needed),
`@args-file`; dev harnesses: `--profile`, `--bench-matrix` (path-coverage perf/regression suite vs
a blessed baseline), `--frametest`, `--divetest` (headless live-dive windows per depth band),
`--livetest` (live-output validation against offline renders of the same views), `--uitest`
(scripted UI + live-render walk with screenshots), `--juliadive`, `--reusetest`, `--refdiag`, `--autodive` (unpaced frame-cost controller
hammer), `--motiontest` (motion-presentation gate), `--chunk-sweep`, `--soak`, `--shot`
(screenshot regeneration), `--torture` (the escalation suite), `--pickcheck`; `--set
NAME=VALUE` overrides one frame-cost tunable for a single run.

`--selftest`, `--uitest` and `--gputest` are also reachable without a command line: **Help →
Diagnostics…** (`ui/diagnostics.rs`) runs the first two as child processes — so a lost device kills
the test rather than the session — and can attach the verdict to an issue report. The developer
harnesses stay CLI-only on purpose. `scripts/gpu-validate.ps1` / `.sh` run the whole battery on a
machine and leave one comparable bundle.

---

## 12. Validation

Layered, mostly external-data-free:

- **Core exact-math tests** (`cargo test -p fractadyne-core`) — perturbation/SA/BLA reproduce the
  exact bignum recurrence; nuclei Newton-solve to known constants; coordinate round-trips.
- **`--selftest`** (**~170 checks + 18 goldens**; the run prints its own totals) — GPU pipeline compared pixel-for-pixel against an
  independent arbitrary-precision **CPU dwell oracle** (shares nothing with the GPU path), plus
  **golden images** (18: a direct-mode overview per family + a deep df32-perturbation, 1e6×, golden
  per polynomial family — so per-formula dispatch and the deep reference-orbit path are both
  guarded; read from the canonical `validation/golden/`; all render-affecting state pinned so
  they're deterministic; `--bless` records, and also stamps `BLESSED-GPU.txt` with the card that
  produced them), plus the **bench-matrix** group: the matrix's deterministic rendering-path signatures
  (mode / SA-skip / orbit length / GPU event counters; 28-segment suite) checked against
  `benchmarks/bench-matrix-baseline.json` — an algorithmic-regression tripwire for any change
  touching the rendering pipeline (see
  [design/bench-matrix.md](design/bench-matrix.md)).
- **`--validate-deep`** — precision self-consistency of the bignum core from 1e1000× to 1e1000000×.
- **`--crosscheck-f3`** — F3's exact integer escape counts vs. the same oracle (transitively:
  GPU≈oracle and F3≈oracle ⇒ GPU≈F3).
- **`validation/catalog.toml`** — externally-verifiable challenge coordinates (period/nucleus,
  membership).
- **Cross-GPU tolerance.** Both image and signature comparisons are strict on the card that
  blessed them and deliberately looser on any other, because neither is machine-independent:
  cross-vendor floating point differs, and on hardware whose shader compiler preserves the df32
  error-free transforms (AMD's Vulkan/GL do; NVIDIA's fold them — see `--gputest`) escape
  decisions move by a pixel here and there, taking the rebase/skip counts with them. A gate that
  reddens on every non-reference GPU teaches testers to ignore it, which costs more than it
  catches. An **absent** `BLESSED-GPU.txt` means strict: the gate only ever loosens when the
  hardware is positively known to differ.

CI (`.github/workflows/ci.yml`) runs the core tests on Linux + a workspace build on Windows; the GPU
`--selftest` is a local/manual gate (runners have no GPU).

---

## 13. Threading

- **UI thread:** the `egui`/`eframe` `update()` loop; owns display and input.
- **Off-thread reference recompute:** a spawned `std::thread` per view (gated to one in flight via a
  channel `recompute_rx`), so the deep-zoom bignum stall never blocks the UI.
- **Parallel reference-candidate scoring:** inside any recompute, `best_reference` fans its
  candidate orbits across all cores with `std::thread::scope` (deterministic reduction).
- **Playback lookahead workers:** during a tour, up to 6 additional worker threads pre-build
  future references (`ref_prefetch` queue).
- **Background export worker:** a thread for cancelable high-res export.
- **Update check:** a short-lived background thread for the GitHub Releases HTTP call.

There is **no `rayon` pool** and no async task scheduler (DESIGN.md §3.1 describes both
aspirationally) — the scoped-thread fan-outs above are plain `std::thread`.

---

## 14. Not built / roadmap

Present in `DESIGN.md` as intent, **not implemented**: the `Fractal`/`RenderStrategy` trait
abstraction; the programmable **formula DSL** + auto-derived perturbation + guided/raw authoring
(M6); **L-systems** and **cellular automata**; **histogram/equalized** coloring; layers/compositing;
3D (Mandelbulb/box); a live **tile cache** (a future render-orchestration crate). Also open (`TODO.md`): XaoS-style
prior-frame **pixel-reuse** during zoom (the frame-reprojection foundation exists as a stall fallback
only), live-view multi-reference glitch correction, and autopilot steering modes. Newton stays direct
(~1e6×) — its convergence dynamics don't fit the perturbation coloring.

---

## 15. Key files

- `crates/fractadyne-core/src/` — the numeric core, split into
  [`lib.rs`](crates/fractadyne-core/src/lib.rs) (facade + `formula` ids + tests),
  [`floatexp.rs`](crates/fractadyne-core/src/floatexp.rs) (`FloatExp`/`CFloatExp`),
  [`bignum.rs`](crates/fractadyne-core/src/bignum.rs) (BigFloat helpers + precision),
  [`viewport.rs`](crates/fractadyne-core/src/viewport.rs) (`Viewport`), and
  [`reference.rs`](crates/fractadyne-core/src/reference.rs) (`reference_orbit`/`step_bf`,
  `best_reference`, `series_skip`, BLA, nucleus finder, multi-ref).
- [`crates/fractadyne-gpu/src/mandelbrot.wgsl`](crates/fractadyne-gpu/src/mandelbrot.wgsl) —
  iterate + color shaders; `Cdf` (df32) + `Fe` (floatexp) helpers; the 3-mode formula branches.
- [`crates/fractadyne-gpu/src/lib.rs`](crates/fractadyne-gpu/src/lib.rs) — renderer, per-view
  resources, `ExportRequest`, `render_export` (tiled).
- [`crates/fractadyne-app/src/render.rs`](crates/fractadyne-app/src/render.rs) — mode select,
  reference recompute + freeze/reproject, export-request builders.
- [`crates/fractadyne-app/src/fractal.rs`](crates/fractadyne-app/src/fractal.rs) — `FractalKind`.
- [`crates/fractadyne-app/src/main.rs`](crates/fractadyne-app/src/main.rs) — app struct + `update()`.
