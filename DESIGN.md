# Fractadyne — Design Document

**Status:** Draft v0.2
**Date:** 2026-06-25
**Author:** rhong (with Claude Code)
**Name:** *Fractadyne* — *fract* (fractal) + *-dyne* (Gk *dýnamis*, "power/force"); i.e. a high-performance fractal engine. The on-disk working directory is still `FractEx/` from the project's original title.

> This is an early draft. The prior version of this document was lost before it
> was written, so nothing here is "recovered" — it is reconstructed from the
> stated goals plus four design decisions made at kickoff (see
> [§1.3](#13-key-decisions)). Five follow-up scope questions have since been
> answered — see [§17](#17-resolved-scope-decisions). Anything not explicitly
> decided is marked as an **Assumption** and is open to change.

---

## 1. Overview

### 1.1 Summary

**Fractadyne** is a high-performance, native desktop application for exploring and
rendering fractals. Its two defining priorities are:

1. **Ultra-deep zoom** — extreme magnification (validated well past `1e300×`),
   using arbitrary-precision arithmetic with perturbation theory and series
   approximation. Depth is bounded by coordinate precision + compute, not a fixed wall.
2. **Extreme performance** — saturate all available hardware: multiple CPU cores,
   the GPU, and large amounts of RAM.

Beyond the Mandelbrot/Julia core, Fractadyne aims to be a broad fractal *workbench*
supporting structurally different families (escape-time fractals, L-systems,
cellular automata) behind a unified UI, with a programmable engine for custom
formulas and coloring.

### 1.2 Goals

- **Deep, fast, interactive zoom** into escape-time fractals with no practical
  depth limit and smooth real-time navigation.
- **Multiple fractal families:** Mandelbrot, Julia, and other escape-time
  variants (Burning Ship, etc.); L-systems; cellular / finite automata.
- **Dual linked views:** explore a parameter plane (Mandelbrot-like) and its
  dynamical plane (Julia-like) side by side, with live preview as the mouse moves.
- **Mouse-driven navigation:** pan, wheel-zoom, and box-zoom.
- **Automatic state persistence:** the full session is saved continuously and
  restored on relaunch; named bookmarks/presets for locations.
- **Image export,** including **single-scene rendering at very high resolution**
  for printing/archival.
- **Per-fractal information:** description, formula, history, parameters, and
  references surfaced in-app.
- **Coloring system:** preset *and* custom palettes, plus preset *and* custom
  coloring algorithms.
- **Programmable engine:** users can define custom fractal formulas and custom
  coloring algorithms (phased — see [§15](#15-roadmap)).

### 1.3 Key decisions

These were chosen at kickoff and anchor the rest of the design:

| Axis                     | Decision                                                              |
| ------------------------ | --------------------------------------------------------------------- |
| **Platform**             | Native desktop application                                            |
| **Language + GPU stack** | Rust + `wgpu` (compute via WGSL)                                      |
| **Zoom depth**           | Extreme — arbitrary precision + perturbation + series approximation (validated past 1e300×) |
| **Extensibility**        | Programmable engine — custom fractal formulas and coloring            |

### 1.4 Non-goals (initial release)

- **3D / distance-estimated 3D fractals** (Mandelbulb, Mandelbox). The
  architecture should not *preclude* them, but they are out of scope for v1.
- **Animation / zoom-movie rendering.** Design should allow it later
  (keyframable state already exists), but it is not a v1 feature.
- **Networking, cloud rendering, multi-machine distribution, accounts.**
- **Mobile / touch-first UI.**

### 1.5 Assumptions (open, correctable)

> Target-OS and GPU-tier questions are now **decided** — see
> [§17](#17-resolved-scope-decisions).

- **Bignum library:** a pure-Rust arbitrary-precision float (`astro-float` or
  `dashu`) to avoid GMP/MPFR build pain on Windows; `rug` (MPFR) kept as a
  performance fallback to benchmark against. See [§5.3](#53-numeric-strategy).
- **No f64 in shaders:** WGSL compute has no native `f64`. Deltas run in `f32`
  (with optional emulated double-single) on the GPU; high precision lives on the
  CPU. See [§5](#5-deep-zoom-rendering-engine-escape-time).

---

## 2. Design Principles

1. **Decouple *computation* from *coloring*.** Iteration results (smooth escape
   values, distance estimates, etc.) are computed once and cached per pixel.
   Changing palette or coloring algorithm re-shades instantly from the cache
   without re-iterating. This is both a performance win and a core UX feature.
2. **The UI thread never blocks.** All heavy work (reference orbits, GPU
   dispatch, export, L-system expansion, CA stepping) runs off the UI thread.
   The view stays interactive even mid-render.
3. **Progressive everywhere.** Show a coarse result immediately, then refine.
   Reproject the previous frame on pan/zoom so motion feels instant.
4. **One abstraction, many fractal kinds.** Escape-time, L-system, and CA
   fractals differ fundamentally in how they're computed and drawn. A
   `Fractal` / render-strategy abstraction hides that from the rest of the app.
5. **Precision is explicit.** Coordinates that exceed `f64` range live as
   arbitrary-precision values end-to-end; `f64`/`f32` are derived views for the
   hot path, never the source of truth for location.
6. **Everything serializable.** View, parameters, palettes, and custom
   formulas all round-trip through `serde` for auto-save, presets, export
   metadata, and sharing.

---

## 3. High-Level Architecture

```
Layer 5 · binary
  fractadyne-app          native window + event loop; wires everything together

Layer 4 · UI & I/O
  fractadyne-ui           egui panels, dual views, input handling
  fractadyne-state        app state, serde, auto-save, presets/bookmarks
  fractadyne-export       tiled high-res PNG/EXR export jobs

Layer 3 · orchestration
  fractadyne-render       tile scheduler, tiling, tile cache, orchestration
  fractadyne-color        palettes, coloring algorithms, coloring compiler

Layer 2 · engine
  fractadyne-gpu          wgpu device, compute pipelines, WGSL codegen
  fractadyne-core         numerics, Fractal trait, perturbation engine, formula DSL
  fractadyne-fractals     built-in fractal defs + info metadata
                          (Mandelbrot, Julia, Burning Ship, L-system, CA, …)

Dependencies point downward (5 → 2); fractadyne-core and fractadyne-gpu
have no knowledge of the UI.
```

Implemented as a **Cargo workspace** of focused crates (see
[§10](#10-project-structure)). The dependency arrows point downward; `core` and
`gpu` have no knowledge of the UI.

### 3.1 Threading model

- **UI thread:** runs the `egui`/window event loop and submits per-frame GPU
  work for *display*. Owns no long computation.
- **Render orchestrator (async tasks / thread pool):** schedules tiles, drives
  CPU reference-orbit computation (via `rayon`), dispatches GPU compute, and
  fills the tile cache. Communicates with the UI via channels and shared,
  lock-light buffers.
- **`rayon` pool:** data-parallel CPU work — reference orbits, series
  coefficients, L-system expansion, CA stepping, CPU fallback rendering.
- **Export worker:** long-running, cancelable high-res export jobs, fully
  isolated from interactive rendering.

---

## 4. Fractal Abstraction

Different families need different pipelines. We model each as a **render
strategy**:

| Family                                               | Compute model                                                                   | Render model                                                  | Deep-zoom behavior                                                           |
| ---------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| **Escape-time** (Mandelbrot, Julia, Burning Ship, …) | Per-pixel iteration; perturbation on GPU + arbitrary-precision reference on CPU | Iteration buffer → coloring shader                            | Extreme depth (perturbation)                                                 |
| **L-system**                                         | CPU grammar expansion → turtle graphics → line/vertex buffer                    | GPU vector/line rasterization with `f64`→`f32` view transform | Limited by float transform precision; arbitrary-precision transform optional |
| **Cellular / finite automata**                       | GPU (or CPU) grid simulation (1D elementary CA spacetime, 2D life-like)         | Grid → texture → sampled to screen                            | Pan/scale over computed grid; on-demand generation                           |

```rust
/// A fractal definition: what to compute and how to describe it.
trait Fractal {
    fn id(&self) -> &str;
    fn info(&self) -> &FractalInfo;            // description, formula, history, refs
    fn parameters(&self) -> &ParamSchema;       // typed, UI-drivable params
    fn family(&self) -> Family;                 // EscapeTime | LSystem | Automaton
    fn render_strategy(&self) -> Box<dyn RenderStrategy>;
    /// For parameter/dynamical-plane linking (dual views). None if not applicable.
    fn linkage(&self) -> Option<PlaneLinkage>;
}

trait RenderStrategy {
    /// Produce/refine a tile's raw result buffer (not yet colored).
    fn render_tile(&self, ctx: &RenderCtx, tile: TileRequest) -> TileResult;
}
```

**Parameter / dynamical plane linkage.** The Mandelbrot set is the *parameter
plane* of the Julia family: each point `c` in it names a Julia set in the
*dynamical plane*. We generalize this with a `PlaneLinkage` so the dual-window
feature (§7) is not Mandelbrot-specific — any family that defines such a
relationship gets live linked views for free.

### 4.1 Cellular automata ("finite automata") modes

"Finite automata" here means **cellular automata**, and **both** sub-modes ship:

- **1-D elementary / totalistic CA** — a single row evolved over time and drawn
  as a *space-time diagram* (vertical axis = generations). Includes the Wolfram
  elementary rules (e.g. Rule 30, 90, 110) and configurable totalistic rules;
  these produce genuinely self-similar structure (Rule 90 → Sierpiński triangle).
- **2-D life-like CA** — a grid evolved in place (Conway's Life and the broader
  B/S "life-like" family), shown as an animatable grid.

Both run as GPU compute over a grid texture (CPU fallback available). "Zoom" here
means pan/scale over the computed lattice; the grid is generated on demand and
extended as the view moves (1-D: compute more generations; 2-D: simulate more
steps / a larger board), bounded by the same RAM tile budget as everything else.
Unlike escape-time fractals there is no precision wall — extent is bounded by
lattice resolution and simulation cost, not floating-point range.

---

## 5. Deep-Zoom Rendering Engine (escape-time)

This is the core of the application and the hardest part. The target is
*extreme-depth* zoom with real-time interactivity, which standard `f64` cannot
provide (it pixelates around `~1e-15`).

### 5.1 The problem

For `z_{n+1} = z_n² + c`, at deep zoom every pixel's `c` differs only in
extremely low-order digits. `f64`/`f32` cannot represent those differences, so a
naive per-pixel high-precision iteration would be correct but far too slow for
interaction.

### 5.2 Perturbation theory

Pick a single high-precision **reference point** `c₀` near the view center and
compute its orbit `Zₙ` in arbitrary precision **on the CPU** (once per frame).
For any other pixel `c = c₀ + δc` where `δc` is small (and representable in
`f64`/`f32`), write `zₙ = Zₙ + δzₙ`. The perturbed recurrence is:

```
δz_{n+1} = 2·Zₙ·δzₙ + δzₙ² + δc
```

Every term here is small and runs in low precision **on the GPU**, while the
expensive high-precision work is amortized across the whole image in the single
reference orbit. Escape test uses the full value `|Zₙ + δzₙ|`.

### 5.3 Numeric strategy

- **Reference orbit:** arbitrary-precision float (CPU). Precision (bits) scales
  with zoom depth. Library: pure-Rust `astro-float`/`dashu` (assumption — easy
  Windows builds), benchmarked against `rug`/MPFR.
- **Stored reference:** `Zₙ` downcast to `f32` (or two `f32` = *double-single*
  for an extra ~7 digits) and uploaded to the GPU as a storage buffer.
- **Per-pixel deltas:** `f32`, with **rescaling** (carry an exponent / scale
  `δ` by `2^k`) to avoid underflow at extreme depth. Optional double-single
  delta path for the moderate-depth band.

### 5.4 Series approximation (SA)

The early iterations of `δzₙ` are well-approximated by a polynomial in `δc`:

```
δzₙ ≈ Aₙ·δc + Bₙ·δc² + Cₙ·δc³ + …
```

The coefficients depend only on the reference orbit, so we compute them once
(CPU) and let every pixel **skip the first K iterations** by evaluating the
polynomial. A validity threshold picks the largest safe `K`. This is a major
speedup at depth.

### 5.5 Glitch detection & correction

Perturbation loses precision when a pixel's true orbit diverges from the
reference (catastrophic cancellation). Detect with **Pauldelbrot's criterion**
(glitch when `|zₙ|² < tol · |Zₙ|²`). Correct by:

1. Collecting glitched pixels.
2. Choosing a **new reference** from within a glitched region (high precision).
3. Recomputing just those pixels against the new reference.
4. Repeating until the glitch set is empty.

Multiple reference orbits are computed in parallel (`rayon`). **Rebasing**
(switching reference mid-orbit) is the alternative/companion technique and is
evaluated during implementation.

### 5.6 GPU pipeline (WGSL compute)

1. Upload reference orbit + SA coefficients + view/delta params.
2. Compute shader: one invocation per pixel → run perturbed iteration from `K`
   to escape/`maxIter`, write **raw result** (smooth iteration value, final
   `|z|`, optional distance-estimate derivative) to a storage buffer — *not* a
   color.
3. Glitch mask written alongside; orchestrator decides on correction passes.
4. Coloring is a **separate** shader pass over the raw buffer (§6), so palette
   changes never re-trigger iteration.

> **WGSL note:** no native `f64`. The reference orbit's precision comes from the
> CPU; the GPU only ever handles small `f32`/double-single deltas. This is the
> standard approach used by real-time deep zoomers (e.g. Kalles Fraktaler /
> Imagina-style renderers).

### 5.7 Adaptive iteration & quality

- `maxIter` scales with zoom depth (deeper → more iterations).
- Optional supersampling / MSAA-style AA for stills; cheaper for interaction.
- Distance estimation available for crisp boundary coloring.

---

## 6. Coloring System

### 6.1 Pipeline

Raw per-pixel results (cached, §2.1) → **coloring algorithm** → **palette** →
framebuffer. Both the algorithm and palette can change without recomputation.

### 6.2 Built-in coloring algorithms

- Escape-time (discrete)
- **Smooth / normalized iteration count** (continuous, `n - log₂(log|z|)`)
- Histogram equalization
- **Distance estimation** (boundary emphasis)
- Interior coloring (e.g., orbit traps, period detection)
- Orbit-trap based

### 6.3 Palettes

- Preset gradient library (perceptually pleasant defaults, including
  perceptually-uniform options).
- Custom palettes: multi-stop gradients, cyclic with adjustable period/offset,
  with import/export.
- Palette applied in-shader via a 1D LUT texture for speed.

### 6.4 Custom (programmable) coloring

Custom coloring algorithms are authored as small expressions/snippets over the
available per-pixel fields (iteration value, `|z|`, distance estimate, orbit-trap
data) and **compiled to WGSL** at runtime (`wgpu` supports runtime shader
creation). See [§8](#8-programmable-engine).

---

## 7. Dual Linked Views (Mandelbrot ↔ Julia)

- Two render surfaces side by side: a **parameter plane** and a **dynamical
  plane**, bound by the active fractal's `PlaneLinkage`.
- **Live preview:** moving the mouse over the parameter plane updates the
  dynamical-plane `c` in real time (cheap, low-res preview while moving; full
  quality on settle). Clicking pins it.
- Either pane can be the primary explorable view; both support full
  pan/zoom/deep-zoom independently.
- Layout: side-by-side or picture-in-picture, user-toggleable.

---

## 8. Programmable Engine

The most advanced capability, delivered in phases (§12). Two things are
programmable: **fractal formulas** and **coloring algorithms**.

### 8.1 Formula DSL

A small expression language over complex numbers (`z`, `c`, parameters,
arithmetic, common functions). A formula is parsed to an AST, which is compiled
to **two** backends:

- **CPU / arbitrary-precision** evaluator — drives the high-precision *reference
  orbit*.
- **GPU / WGSL** code — drives the *perturbed per-pixel* iteration.

### 8.2 The hard part: perturbation for arbitrary formulas

Perturbation needs the *perturbed recurrence* (the `δz` update), which for
custom formulas must be derived from the formula itself. Plan:

- **Built-in formulas:** hand-written, verified perturbed recurrences (fast,
  exact).
- **Custom formulas:** auto-derive the perturbed form via **symbolic
  differentiation** of the AST. Where derivation is unstable or unsupported,
  **fall back** to per-pixel high-precision iteration (correct, slower) so the
  formula always renders.

### 8.3 Safety & sharing

- The DSL is sandboxed (no arbitrary code execution; only the math surface).
- Custom formulas, palettes, and coloring algorithms are serializable and
  shareable as small definition files, with import/export and a local library.

### 8.4 Authoring surface (two tiers)

Both fractal formulas and coloring algorithms are authored through **two
complementary surfaces** that share one underlying compiler:

- **Guided expression editor** (default) — a high-level, validated expression
  language with autocomplete, inline error feedback, live preview, and typed
  parameter widgets. Safe and approachable; covers the large majority of cases.
- **Raw shader mode** (power users) — direct editing of the WGSL-like
  per-iteration / per-pixel snippet the editor would otherwise generate, for
  effects the high-level surface can't express. Still sandboxed to the math
  surface and the exposed input fields.

The guided editor compiles to the same intermediate form as raw mode, so a user
can start in the editor and "drop down" to the shader to refine.

**Every built-in fractal and coloring algorithm ships as an editable sample
definition** in this system — the built-ins *are* the worked examples. A user can
open Mandelbrot, Julia, Burning Ship, the smooth-coloring algorithm, etc., read
exactly how each is defined, and fork it as the starting point for their own.
(Built-in escape-time formulas additionally carry a hand-written, verified
perturbation path per [§8.2](#82-the-hard-part-perturbation-for-arbitrary-formulas);
forks fall back to auto-derivation.)

---

## 9. UI / UX

> **Full UI/UX brief:** [`UI-DESIGN.md`](UI-DESIGN.md) — design principles,
> layout/wireframes, theme tokens, stock-widget component map, and per-screen
> specs. This section is just the summary.

- **Toolkit:** **`egui`** (immediate-mode), rendering through the same `wgpu`
  device as the fractal surfaces — fluid live controls and trivial integration
  with a custom render target. **Decided** (over a Tauri web-UI shell) for
  familiarity, maintainability, and stock-widget reuse — see `UI-DESIGN.md` §1.1.
- **Navigation:** drag to pan, wheel to zoom (cursor-centered), drag-rectangle
  for box-zoom, keyboard nudges; zoom level / coordinates shown numerically and
  directly editable.
- **Panels:** fractal picker, parameter inspector (schema-driven), coloring &
  palette editor, info panel, bookmarks/presets, export dialog.
- **Info panel:** renders each fractal's `FractalInfo` (description, formula,
  history, parameter docs, references) from bundled metadata.
- **Responsiveness:** progressive/coarse-to-fine rendering and a live status
  (current depth, iterations, render progress, glitch passes).

---

## 10. Project Structure

Cargo workspace:

```
Fractadyne/
├─ Cargo.toml                     # workspace
├─ DESIGN.md
├─ crates/
│  ├─ fractadyne-core/            # numerics, Fractal trait, perturbation, formula DSL
│  ├─ fractadyne-gpu/             # wgpu device, compute pipelines, WGSL codegen
│  ├─ fractadyne-render/          # scheduler, tiling, tile cache, orchestration
│  ├─ fractadyne-color/           # palettes, coloring algorithms, coloring compiler
│  ├─ fractadyne-fractals/        # built-in fractal defs + info metadata
│  ├─ fractadyne-state/           # app state, serde, auto-save, presets/bookmarks
│  ├─ fractadyne-export/          # tiled high-res export, PNG/EXR encoders
│  ├─ fractadyne-ui/              # egui panels, dual views, input handling
│  └─ fractadyne-app/             # binary: window + event loop wiring
├─ assets/                        # default palettes, fractal info, shaders
└─ tests/                         # integration + golden-image tests
```

---

## 11. Performance & Memory Strategy

- **Compute/coloring split** (§2.1): re-coloring is free; iteration is cached.
- **Tile cache in RAM:** raw iteration buffers cached per tile/zoom; pan reuses
  cached tiles and only computes newly exposed regions. Cache sized to a
  configurable RAM budget (leverages large RAM as requested).
- **Progressive refinement:** coarse pass first; refine to full resolution and
  higher `maxIter` as the view settles.
- **Reprojection:** on pan/zoom, warp the previous frame as an instant
  placeholder while the real tiles compute.
- **CPU parallelism:** `rayon` for reference orbits, SA coefficients, glitch
  correction (parallel references), L-system expansion, CA stepping.
- **GPU saturation:** large compute dispatches; double-buffering so display
  never waits on compute.
- **Primary GPU target — NVIDIA RTX 3000-series (Ampere) and newer.** Workgroup
  sizes, occupancy, memory-access patterns, and per-invocation iteration counts
  are tuned for that class. Tuning stays within portable `wgpu`/WGSL — **no CUDA
  or vendor-specific APIs** — so the same code still runs on other vendors and on
  macOS/Linux (per [§17](#17-resolved-scope-decisions)).
- **Integrated-GPU fallback (functional, not optimized).** On integrated GPUs the
  app must still run correctly: feature-detect device limits, shrink tiles, keep
  the conservative precision band, and disable optional quality passes (heavy
  supersampling, deep SA). Correctness over speed; no separate optimization
  effort is spent on this tier.
- **Adaptive work:** iteration count and precision scale with depth; SA skips
  iterations; glitch passes only touch glitched pixels.

---

## 12. High-Resolution Export

- Render a single scene at arbitrary dimensions (e.g. 16384×16384 and beyond),
  independent of the screen.
- **Tiled rendering:** the target image is split into GPU-sized tiles; each is
  computed (with higher `maxIter` and supersampling for quality) and streamed
  into a full-resolution buffer in RAM or to disk to bound GPU memory.
- **Background & cancelable:** runs on the export worker with progress; the
  interactive view stays live.
- **Formats:** **PNG** (8- and 16-bit, sRGB) and **OpenEXR** (32-bit float,
  linear/HDR — preserves the raw coloring data for later re-grading). Export
  metadata (full view state) is embedded so any exported image can be reopened at
  its exact location.
- **No hard resolution cap.** Because rendering is tiled and streamed, output
  size is bounded by disk, not GPU/RAM; the streamer is validated to very large
  targets (≥ 64k × 64k). The UI offers presets (e.g. print sizes) plus free
  numeric entry, and estimates render time and output-file size before starting.

---

## 13. Persistence

- **Auto-save:** full session state (active fractal, per-fractal params,
  high-precision location, zoom, coloring, palette, layout) serialized
  continuously (debounced) to the OS app-data dir. **Atomic writes** (temp file
  + rename) prevent corruption. Restored on launch.
- **Bookmarks / presets:** named locations and full-state presets in a local
  library; import/export as files.
- **Format:** human-readable (TOML/JSON via `serde`); coordinates stored as
  arbitrary-precision decimal strings so deep locations survive round-trips.

---

## 14. Testing & Quality

- **Unit tests:** numerics (perturbation vs. naive high-precision at shallow
  depth must agree), DSL parser/codegen, coordinate transforms (property tests).
- **Golden-image / visual regression:** render known locations and compare to
  stored references within tolerance (catches shader/coloring regressions).
- **Benchmarks (`criterion`):** reference-orbit throughput, GPU iteration rate,
  re-color latency, export throughput.
- **Glitch correctness:** verify deep-zoom frames are glitch-free after
  correction at a set of known-hard coordinates.

---

## 15. Roadmap

> Milestones are ordered to make the **core deep-zoom engine** real early, since
> it is the primary risk and priority.

- **M0 — Foundations:** workspace, `wgpu` device + window + `egui`, basic
  Mandelbrot in `f64` on GPU, pan/wheel/box zoom.
- **M1 — Coloring & state:** compute/coloring split + tile cache, smooth
  coloring, preset palettes, auto-save/restore, basic info panel.
- **M2 — Deep zoom (core):** arbitrary-precision center, CPU reference orbit,
  GPU perturbation, glitch detection + correction, series approximation,
  rescaling. *Delivers "ultra-deep zoom."*
- **M3 — Dual views:** parameter/dynamical-plane linkage, side-by-side
  Mandelbrot↔Julia with live hover preview.
- **M4 — Fractal variety:** additional escape-time fractals (Burning Ship,
  etc.); L-system vector pipeline; cellular/finite-automata pipeline; solidify
  the `Fractal`/`RenderStrategy` abstraction.
- **M5 — High-res export:** tiled, supersampled, background, cancelable; PNG/EXR;
  embedded metadata.
- **M6 — Programmability:** formula DSL → WGSL + CPU codegen, auto-derived
  perturbation with high-precision fallback, custom coloring algorithms,
  custom-palette/algorithm library with import/export.
- **M7 — Polish & perf:** profiling, tile-scheduler tuning, parallel
  multi-reference glitch correction, UX refinement.

---

## 16. Risks & Mitigations

| Risk                                                                           | Mitigation                                                                                                                                                                   |
| ------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Auto-perturbation for arbitrary custom formulas** (symbolic diff, stability) | Hand-written perturbation for built-ins; auto-derivation for custom with high-precision **fallback** so it always renders.                                                   |
| **No `f64` in WGSL**                                                           | High precision stays on CPU (reference orbit); GPU uses `f32`/double-single deltas with rescaling.                                                                           |
| **GMP/MPFR build friction on Windows**                                         | Default to pure-Rust bignum (`astro-float`/`dashu`); benchmark `rug` as optional.                                                                                            |
| **GPU memory limits on huge exports**                                          | Tiled rendering + stream to RAM/disk; never hold the full image on the GPU.                                                                                                  |
| **L-system string explosion at high depth**                                    | Cap expansion depth, stream geometry, warn the user; consider on-the-fly expansion.                                                                                          |
| **Deep-zoom glitch correctness**                                               | Pauldelbrot criterion + iterative multi-reference correction; golden-image tests at known-hard coordinates.                                                                  |
| **Integrated-GPU fallback**                                                    | Feature-detect device limits; shrink tiles, hold the conservative precision band, disable optional passes; prioritize correctness over speed (no perf tuning for this tier). |
| **Cross-platform drift while Windows-first**                                   | Portable `wgpu`/WGSL only, no OS-specific APIs, `directories` crate for paths; periodic macOS/Linux build checks so portability doesn't rot.                                 |

---

## 17. Resolved Scope Decisions

*Resolved 2026-06-25. These answer the open questions raised at kickoff and are
reflected in the sections noted.*

1. **GPU target & floor** — **Optimize for NVIDIA RTX 3000-series (Ampere) and
   newer.** Integrated GPUs are supported as a **functional fallback only** (run
   correctly with reduced settings; no perf tuning). Optimization stays within
   portable `wgpu` — no CUDA. → [§11](#11-performance--memory-strategy)
2. **Export formats** — **PNG and OpenEXR.** No hard resolution cap (tiled +
   streamed to disk). → [§12](#12-high-resolution-export)
3. **Cellular / "finite automata" scope** — **Both** 1-D elementary/totalistic
   CA (space-time diagrams, e.g. Rule 30/90/110) **and** 2-D life-like CA (e.g.
   Conway's Life). → [§4.1](#41-cellular-automata-finite-automata-modes)
4. **Custom-authoring surface** — **Both** a guided expression editor **and** a
   raw shader mode, and **every built-in fractal/coloring algorithm ships as an
   editable sample** users can fork. → [§8.4](#84-authoring-surface-two-tiers)
5. **Cross-platform stance** — **Windows-first**, but make **no choice that
   forecloses macOS/Linux**: portable `wgpu`/WGSL only, no OS-specific APIs,
   cross-platform config/app-data paths (`directories` crate), cross-platform
   dependencies. macOS/Linux validated later, not at v1. → [§1.5](#15-assumptions-open-correctable)

---

## 18. Glossary

- **Escape-time fractal** — colored by how fast an iterated point's value
  escapes a bound (e.g. Mandelbrot).
- **Perturbation** — computing one high-precision *reference* orbit and deriving
  all other pixels as small low-precision deltas from it.
- **Reference orbit** — the high-precision iterated sequence `Zₙ` for the
  reference point.
- **Series approximation (SA)** — a polynomial in `δc` that lets pixels skip the
  first K iterations.
- **Glitch** — pixel where perturbation loses precision and must be recomputed
  against a new reference.
- **Distance estimation** — estimates distance to the fractal boundary for crisp
  edge coloring.
- **Smooth / normalized iteration count** — fractional escape value for banding-free
  continuous coloring.
- **Parameter plane vs. dynamical plane** — the space of a family's parameter
  `c` (Mandelbrot) vs. the orbit space for a fixed `c` (Julia).
- **L-system** — a grammar whose rewritten string drives turtle graphics.
- **Cellular automaton** — a grid of cells updated by local rules; some produce
  fractal-like structure.
- **double-single (df64)** — an extra-precision number emulated as a pair of
  `f32`/`f64` values, used on the GPU where native `f64` is unavailable.
```
