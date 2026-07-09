# Fractadyne — Architecture (as-built)

**Version:** 0.2.0 · **Updated:** 2026-07-09

This document describes the system **as it is actually implemented**. It is the counterpart to
[`DESIGN.md`](DESIGN.md), which is the *original design intent* (2026-06-25) and has diverged from
the code in several places. Where the two disagree, this document is authoritative for the current
state; [`CHANGELOG.md`](CHANGELOG.md) is the running per-version log.

> **Divergence summary (design → as-built):** the `Fractal`/`RenderStrategy` trait abstraction was
> not built (formulas are a `FractalKind` enum switched on by hand-written per-path `match` arms);
> one crate (`-render`) is an empty stub whose intended logic lives in `fractadyne-app` (the `-ui`
> and `-fractals` stubs were retired, and UI is split under `fractadyne-app/src/ui/`); the live view
> uses **full-frame render + reprojection freeze**, not a per-tile
> RAM cache; `rayon` is not used (off-thread work is `std::thread`); and the programmable formula
> DSL, L-systems, cellular automata, and histogram coloring are **not implemented** (roadmap).

---

## 1. What it is

A native **Windows** desktop fractal explorer (Rust + `wgpu`/`egui`/`eframe` 0.31), focused on
**extreme deep zoom** of escape-time fractals and on **correctness you can verify**. Binary:
`fractadyne` (crate `fractadyne-app`).

Deep zoom is bounded by coordinate precision + iteration/compute budget, not a fixed wall:
cross-checked against Fraktaler-3 to ~1e300×, renders far deeper offline (a ~1e1108× sample), and
the arbitrary-precision core is self-consistency-validated to 1e1000000×.

---

## 2. Crate layout

A 7-crate Cargo workspace under `crates/`: **six are functional** and one (`fractadyne-render`) is a
reserved stub whose intended responsibility currently lives in `fractadyne-app`.

| Crate | Status | Responsibility |
|-------|--------|----------------|
| `fractadyne-core` | ✅ | Numerics: `Viewport` (BigFloat center, `FloatExp` scale), reference-orbit iteration (`step_bf`, `reference_orbit`), `best_reference`, series approximation (`series_skip`), BLA (`build_bla_mandel`), precision helpers, the CPU dwell oracle, minibrot/nucleus finder. No GPU/UI. **Decomposed into submodules** — `floatexp`, `bignum`, `viewport`, `reference` — behind a thin `lib.rs` facade that re-exports the public API. |
| `fractadyne-gpu` | ✅ | `wgpu` device, render pipelines, WGSL shaders (`mandelbrot.wgsl` — iterate + color), `render_export` (tiled), per-view resources. |
| `fractadyne-color` | ✅ | Preset gradient palettes + interpolation. |
| `fractadyne-state` | ✅ | Session persistence (`session.toml`), versioned state, `config_dir()` + `FRACTADYNE_CONFIG_DIR`, reset. |
| `fractadyne-export` | ✅ | PNG/OpenEXR encode/decode + embedded view metadata. |
| `fractadyne-app` | ✅ | **Everything else** — app struct, UI, input, scripting, CLI, autopilot, coloring/mode logic, `FractalKind`. Split into modules (below). |
| `fractadyne-render` | ⛔ stub | *Planned:* tile scheduler / cache. (Placeholder; the two earlier `-ui` / `-fractals` stubs were retired in the refactor.) |

**`fractadyne-app` modules:** `main.rs` (app struct + `update()` UI loop), `render.rs` (mode
select + reference recompute / reuse / freeze-reproject + export requests), `autopilot.rs`
(auto-zoom), `scripting.rs` (tours + `render_tour_to_dir`), `help.rs` (`CLI_REFERENCE` + Help
window), `cli.rs` (headless modes), `export.rs` (view-metadata / `.fdn`), `fractal.rs`
(`FractalKind`), `refcache_persist.rs` (persist/restore the deep-zoom reference), `error.rs`
(`AppError`), `selftest.rs` (GPU validation), `theme.rs`, `profile.rs`, `sysinfo.rs`, and a `ui/`
submodule tree (`central.rs`, `menus.rs`, `panels.rs`, `dialogs.rs`) from the intra-crate UI split.

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
- **Reference selection:** `best_reference` picks a long/interior reference (scored in bignum).
- **Precision** auto-scales with zoom octaves (`precision_for_octaves`), +64 guard bits.

**Reference recompute is off the render thread.** The bignum reference + SA + BLA bundle
(`recompute_worker`) is the deep-zoom stall, so it runs on a spawned `std::thread`; the render keeps
drawing with the cached reference and installs the fresh one when it lands. Only the very first
(cold) reference is synchronous. A deeper rebuild **reuses** the cached orbit — it *extends* the
stored bignum prefix from a saved tail (byte-identical) instead of recomputing every step, since the
orbit build is ~90% of a deep frame (~20× faster dive-rebuilds). The last deep view's reference also
persists across sessions (`refcache_persist.rs`) so it resumes instantly.

**Freeze / reproject on motion.** While the reference is stale for the current depth (`depth_lag`)
or a fast dive would spin the mode-2 shader, `render.rs` holds the last good iteration texture and
**reprojects** it (scale + translate in the color pass, `uv_scale`/`uv_off`) so the view keeps
moving smoothly until the fresh reference snaps in. This replaced the "Not Responding" hang; it is
frame-level, not tile-level. Moving-frame resolution is **adaptive** (AIMD, `perf.motion_res`): it
follows the measured frame time — raised while frames stay near vsync (the BLA is skipping), backed
off when they run long — so deep motion sharpens without stalling.

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
setup}.ps1`.

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
`--render-iter`, `--find-minibrot`, `--import-kfr`, `--reset-state`, `@args-file`.

---

## 12. Validation

Layered, mostly external-data-free:

- **Core exact-math tests** (`cargo test -p fractadyne-core`) — perturbation/SA/BLA reproduce the
  exact bignum recurrence; nuclei Newton-solve to known constants; coordinate round-trips.
- **`--selftest`** — GPU pipeline compared pixel-for-pixel against an independent arbitrary-precision
  **CPU dwell oracle** (shares nothing with the GPU path), plus **golden images** (17: a direct-mode
  overview per family + a deep df32-perturbation, 1e6×, golden per polynomial family — so per-formula
  dispatch and the deep reference-orbit path are both guarded; bit-identical, read from the canonical
  `validation/golden/`; all render-affecting state pinned so they're deterministic). `--bless` records.
- **`--validate-deep`** — precision self-consistency of the bignum core from 1e1000× to 1e1000000×.
- **`--crosscheck-f3`** — F3's exact integer escape counts vs. the same oracle (transitively:
  GPU≈oracle and F3≈oracle ⇒ GPU≈F3).
- **`validation/catalog.toml`** — externally-verifiable challenge coordinates (period/nucleus,
  membership).

CI (`.github/workflows/ci.yml`) runs the core tests on Linux + a workspace build on Windows; the GPU
`--selftest` is a local/manual gate (runners have no GPU).

---

## 13. Threading

- **UI thread:** the `egui`/`eframe` `update()` loop; owns display and input.
- **Off-thread reference recompute:** a spawned `std::thread` per view (gated to one in flight via a
  channel `recompute_rx`), so the deep-zoom bignum stall never blocks the UI.
- **Background export worker:** a thread for cancelable high-res export.

There is **no `rayon` pool** and no async task scheduler (DESIGN.md §3.1 describes both aspirationally).

---

## 14. Not built / roadmap

Present in `DESIGN.md` as intent, **not implemented**: the `Fractal`/`RenderStrategy` trait
abstraction; the programmable **formula DSL** + auto-derived perturbation + guided/raw authoring
(M6); **L-systems** and **cellular automata**; **histogram/equalized** coloring; layers/compositing;
3D (Mandelbulb/box); a live **tile cache** (`fractadyne-render`). Also open (`TODO.md`): XaoS-style
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
