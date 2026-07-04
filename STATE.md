# Fractadyne — Current State (resume log)

_Last updated: 2026-07-04. Version **v0.1.18** (auto-incrementing build counter in
`.build_seq` at repo root, exposed as `FRACT_BUILD`). [CHANGELOG.md](CHANGELOG.md) is the
authoritative running log; this file is a higher-level snapshot. Some deep-dive investigation
sections below are historical (kept for reference) — the CHANGELOG is the source of truth for
what shipped._

Companion docs: [TODO.md](TODO.md) (backlog, what's done), [CHANGELOG.md](CHANGELOG.md)
(per-version changes), [DESIGN.md](DESIGN.md) / [UI-DESIGN.md](UI-DESIGN.md) (specs).

## What this is
Native Windows fractal explorer (Rust + wgpu/egui, eframe 0.31). Priorities: ultra-deep
zoom + performance. 9-crate workspace under `crates/`; the app is `fractadyne-app`
(binary `fractadyne`).

## Build / run (this machine's constraints)
```
# debug
$env:Path="$env:USERPROFILE\.cargo\bin;$env:Path"; $env:CARGO_INCREMENTAL="0"
Stop-Process -Name fractadyne -Force   # release the exe lock first
cargo build -p fractadyne-app -j 1
Start-Process .\target\debug\fractadyne.exe

# release (optimized numerics — bignum ~8× faster; use for deep-zoom perf)
cargo build --release -p fractadyne-app -j 1
```
- Build at **-j 1** (low commit limit). `[profile.dev]`/`[profile.release]` set
  `debug=false` (full debuginfo OOMs the linker here — OS error 1455). Do NOT re-enable
  debuginfo without enlarging the page file. Cargo pipelining disabled in `.cargo/config.toml`.
- Do NOT add AV/Defender exclusions (previously denied). `Stop-Process -Name fractadyne`
  to unlock the exe before rebuild is authorized.

## Headless CLI (automation / debugging)
`fractadyne --help` prints the full, always-current reference (generated from the shared
`CLI_REFERENCE` table in `help.rs`, the same list the in-app Help window shows). Common:
```
fractadyne --help                                 # full command-line reference, quits
fractadyne --benchmark [--out report.txt]         # fixed deep-zoom tour, samples FPS/CPU/GPU/RAM, quits
fractadyne --render --out img.png [--fractal Mandelbrot --center X Y --zoom M \
           --size WxH --ss N --iter K --julia --julia-c RE IM --palette I --show-location]
fractadyne --render-tour tour.toml --out frames [--fps N --size WxH --ss N --mp4 [f.mp4] --show-location]
```
All skip session autosave. `--render` reuses the tiled export pipeline; PNG/EXR by
extension; full-precision center via decimal strings. `--render-tour` renders a keyframe-tour
TOML to a PNG frame sequence (+ optional ffmpeg mp4), pipelining reference/render/encode across
frames. Great for golden-image checks and movie export.

## Deep-zoom engine (the headline feature) — current status
- Arbitrary-precision center (`astro_float::BigFloat`, precision scales with zoom).
- Reference orbit computed in bignum on CPU, handed to GPU as df32 samples.
- **GPU perturbation is now a 3-tier hybrid by depth** (see `mandelbrot.wgsl` `fs_iterate`
  and `build_params` in `main.rs`):
  - `mode 1` direct df32 — `< 1e4×` or non-perturbation formulas.
  - `mode 0` df32 perturbation — `1e4 … 1e28×` (fast common path).
  - `mode 2` **floatexp** perturbation — `≥ 1e28×` (`PERT_FE_THRESHOLD`). df32 mantissa +
    i32 exponent (`Fe` type in the shader) → no f32 exponent underflow → **extreme
    depth** (bounded by center-coord precision + iteration budget, not f32).
- Shared base-2 `delta_exp` keeps the input δ mantissas (step / ref_offset) O(1) at any
  depth; the perf panel `mode` line shows which path is active.
- Verified clean via `--render` at 1e15 / 1e25 / 1e27 (df32) / 1e29 / 1e32 (floatexp);
  shallow unchanged; df32↔floatexp crossover seamless. Release benchmark score ~3220.
- Earlier fix: Julia rebasing subtracts `reference[0]` (no-op for Mandelbrot, required
  for Julia where Z₀ ≠ 0).

## Interactive orbit overlay — ✅ DONE (shipped; notes historical)
View → "Show orbits" (+ "Normalize (fit to view)", + "Animate (racing dot)" with speed).
Draws the iteration path of the point under the cursor (`draw_orbit` in `main.rs`).
- Shallow (≤1e12×): f64 orbit from the exact cursor point (`orbit_points`).
- Deep (>1e12×, perturbation families): **bignum orbit from the cursor's high-precision
  coordinate** (`pixel_to_complex` → `reference_orbit`, cap `ORBIT_MAX_DEEP=8192`,
  runs toward escape), recomputed on cursor/view change and **cached** (`orbit_cache`,
  `RefCell`). Trims the final blow-up at `|z|>4` so the normalized fit isn't dominated
  by one escaping iterate.
- **Open question (pending user confirm on build 16):** earlier builds looked "static"
  at deep zoom because (a) the overlay was the fixed *reference* orbit, then (b) capped
  at 512 < the orbit's divergence/escape length so only the shared early trajectory
  showed, and (c) the escaping iterate dominated the normalized bounding box. Build 16
  addresses all three (cursor-following + cap 8192 + |z|>4 trim). **Need to confirm it
  now reshapes as the cursor moves.** If still static at extreme depth, the divergence
  length exceeds 8192 → raise `ORBIT_MAX_DEEP` (cost grows; cache mitigates) or cap to
  the view's eff_iter. Debug build may feel laggy sweeping the cursor (bignum at
  opt-level 0); release is smooth.

## ✅ INVESTIGATED — blank fractal at deep floatexp zoom (likely NOT a bug)
Update (2026-06-28): forced floatexp at shallow depth (`PERT_FE_THRESHOLD=1e3`) and
`--render`ed the seahorse at 1e6× → **renders perfectly clean**. So floatexp is healthy;
the blank is **not** a general `Fe` bug. Also: build 16 only changed `draw_orbit` + added
`orbit_cache` — the fractal render path is byte-identical to build 15, so it's **not a
build-16 regression**. Evidence (fast ~11 ms frame + short reference, escape ~276)
indicates the view was in a **featureless fast-escape region** — the whole view escapes
near iteration 276, so the color is near-uniform. Navigating back toward the boundary
shows detail. *Residual possibility:* a localized single-reference perturbation glitch
(would need the planned multi-reference glitch correction), but the whole-view-uniform
symptom fits a featureless region better than a localized glitch. Left as-is; revisit
only if a clearly-detailed area shows a uniform blob.

(Original notes retained below for reference.)

## ⚠️ Original blank-render notes (superseded by the investigation above)
Screenshot `diag/2026-06-27_23-17-30.png` (build 16): at ~3.78e32× (mode `perturb
floatexp`) the **fractal renders blank/uniform** (solid color) while the orbit overlay
works fine. Perf: `orbit len 276` (short reference), `eff iter 50000`, frame ~11 ms.

Key clue: frame is **only ~11 ms** — far too fast for 50000 floatexp iterations over a
full screen. So the iteration loop is exiting almost immediately for ~all pixels →
near-uniform escape value → flat color.

Important: **build 16 only changed `draw_orbit` + added `orbit_cache`** — it did NOT
touch the shader or the render path. So this is most likely a **pre-existing floatexp
issue exposed at this location**, not a build-16 regression. (Build 15 shot `23-11-12`
rendered fine at ~3.78e32 — but likely a *different* spot with a longer reference.)

Hypotheses (ranked):
1. **floatexp + short reference**: `orbit len 276` means best_reference here is short, so
   Zhuoran rebasing fires every ~276 iters. The df32 path tolerated short refs; the
   floatexp rebasing may mishandle them. (Rebasing math was hand-checked and looks
   correct: `δz = z_full − reference[0]`, and the algebra reduces to `z²+c` — so suspect
   the `Fe` ops themselves under heavy rebasing, e.g. `fe_norm`/`fe_add` edge cases, or
   `fe_mag2`/`fe_lo_f32` exp clamps causing a spurious early bail.)
2. **Spurious early escape**: something makes `zf` huge on iteration ~0–1 (NaN/garbage
   from `Fe`, or `exp2` overflow in `fe_lo_f32`/`fe_mag2`), tripping `z2 > bail2`
   immediately → uniform + fast. Check `fe_lo_f32` (clamps e to ±127) and `fe_mag2`
   (clamps 2e to ±250) for cases where the clamp distorts the value near escape.
3. **Genuinely smooth region**: the view sits in a smooth exterior where escape time
   barely varies across the microscopic span → near-uniform. Less likely (fully blank,
   not a gradient), but rule out.

Debug plan:
- Repro headless: find a center that yields a short reference at ~1e30–1e32× and
  `--render` it; compare to a long-reference spot. (Interactive: watch whether "blank"
  correlates with small `orbit len`.)
- A/B the paths at the SAME spot: temporarily lower `PERT_FE_THRESHOLD` to ~1e6 to force
  floatexp shallow (where df32 is known-good) and `--render` a 1e8× view — if floatexp
  is blank there too, it's a pure `Fe` bug independent of depth (fastest way to isolate).
- Add a temporary debug colormap of raw iteration count to see if it's all-escaped
  (iter≈0) vs all-interior (iter=max).
- Inspect the floatexp branch in `mandelbrot.wgsl`: `fe_lo_f32`, `fe_mag2`, the rebase
  block, and whether `dz`'s exponent ever overflows i32 / produces NaN via `log2(0)`
  in `fe_norm` (guarded by `mag==0`, but check denormal mantissas).

## ✅ Random palette redesigned (harmonious) — DONE (build 18)
`RandomPalette::gen_stops` now builds a single base hue with a gentle analogous
excursion + a smooth dark→bright→dark arc (`sin(πt)` so endpoints coincide → seamless),
moderate constant saturation, dim (not black) ends. No more clashing rainbow stripes.
Possible later polish: random *flavors* (complementary-pair / monochrome). Original plan
below.

## 🎨 Random palette redesign — original plan (implemented)
Current (`RandomPalette::gen_stops`): each of 6 stops gets a *fully independent* random
HSV (random hue/sat/val), endpoints equal. Independent random hues → clashing adjacent
colors → garish stripes. Fix: constrain to a **coherent scheme with gentle hue motion**:
- Pick a random **base hue** `h0` and a **small hue span** (e.g. 0.06–0.28 of the wheel),
  with a random direction. Stop hues walk the band smoothly: `h_i = h0 + span·(i/(n-1))`
  (or a small per-stop random-walk Δh ≈ ±0.03) → analogous, flowing hues.
- **Brightness arc** instead of random value: ends darker, peak mid (like Ember), so the
  gradient has depth and the seamless endpoints stay dark. Saturation in a moderate band
  (~0.5–0.85), gently varied.
- Optionally a few scheme *flavors* chosen at random: analogous (narrow band),
  complementary-pair (two anchors), or monochrome (fixed hue, vary value/sat) — all of
  which read as tasteful rather than random.
- Keep first==last for seamless cycling; keep the morph/blend + Shuffle as-is.
This is low-risk (only `gen_stops`); implement when we resume.

## Other notable features (done)
- Dual linked view (Mandelbrot ↔ Julia), per-view ref caches, Julia pin, painted
  two-rectangle Dual toolbar icon. Status bar shows both panels' zoom in dual.
- Combined menu+toolbar — icons grouped **File I/O · Navigation/location · Appearance/display**;
  docked perf panel (julia c readout + `c/panel`), animated zoom-home (🏠).
- **Lifted the ~1e308× live-zoom ceiling**: viewport scale is an extended-range `FloatExp`
  (`m·2^e`); `log2_magnification` + `precision_for_octaves`; GPU fed O(1) span mantissa +
  shared `delta_exp` (shader unchanged). `--zoom-log2`, deep goto/`.fdn` persistence.
- **Auto-zoom autopilot** (A key / 🛸 toolbar button / View menu): renders a small iteration
  field, steers toward the boundary+gradient-richest region, eased smooth dive with an
  **adjustable dive limit** (Navigation-panel slider, persisted, 1e30×–1e5000×). Past ~1e271× it
  switches to a **stepped dive** (jump ×4 → render → hold the frame while the next computes) to
  reach extreme depth. Re-evaluation is adaptive (spaces out as frames slow). Esc / any input
  stops; the toolbar button highlights while running.
- **Coloring**: preset/custom gradient editor (≤8 stops), Duotone + Binary two-color modes,
  methods (smooth/stripe/TIA/orbit-trap/distance/decomposition), DE relief lighting + glow.
- Minibrot finder (M, Newton nucleus + period), minimap overview, famous Locations tour +
  random location, bookmarks, navigation undo/redo, Go-to-location dialog.
- **Sharing**: `.fdn` location copy/paste/save/load (File → Share location…), hardened parser.
- Export: tiled PNG/EXR + reloadable metadata (carries `upp_log2`), gallery browser,
  background render + progress + cancel, dual layouts, quick-save (Ctrl+S).
- Palette animation: Off/Forward/Reverse/Ping-pong/**Random gradients** (+Shuffle), speed.
- Scripting (Tools → Play script…): TOML keyframe camera tours with eased moves, timed
  captions, coordinate-anchored callouts, and spotlight vignettes; built-in benchmark tour.
  Headless `--render-tour` renders a tour to a PNG sequence + optional ffmpeg **mp4**, with an
  optional zoom/coordinate HUD (`--show-location`); reference/render/encode pipelined per frame.
- Watermark: subtle "Fd" mark (BRAND colors) on live view + exports, on by default, toggleable
  (`--watermark`/`--no-watermark`).
- In-app **Help** (F1): Overview / Navigation / Coloring & options / Fractals / How it works /
  Command line / Shortcuts / Recommended hardware / Acknowledgments / Licenses / About.
- **Reset application state** (File → Reset application state… / `--reset-state`): clears session +
  bookmarks + thumbnails after a confirmation dialog; the session file is versioned (warns if a
  newer build wrote it). `FRACTADYNE_CONFIG_DIR` overrides the storage location (sandbox/portable).
- **Restartable tour renders** (`--render-tour --resume`): keep frames already on disk, render only
  the missing ones. `scripts/render-spiral-dive.ps1` detects prior runs and offers Resume / Over.
- **Third-party license notices**: `THIRD-PARTY-NOTICES.md` (generated with cargo-about, shipped
  with the release) + in-app **Help → Licenses**.
- **Deep sample location + helper scripts**: `scripts/deep-sample.fdn` (~1e1108× Mandelbrot,
  loadable via Open view); `scripts/setup.ps1` (Windows build bootstrap),
  `scripts/render-deepest.ps1`, `scripts/render-spiral-dive.ps1`.
- **Validation**: `--selftest` (GPU vs CPU-f64 + bignum oracle to 1e30×, goldens), core
  exact-math tests, `--validate-deep` (precision self-consistency to 1e1000000×),
  `--crosscheck-f3` (vs Fraktaler-3), `--compare`, `--render-iter`, `.kfr` import.

## Top open items (from TODO.md)
- Newton deep zoom (convergence-based; perturbation impractical — stays direct ~1e6×).
  (Burning Ship/Celtic/Buffalo and Phoenix now perturbation-deep-zoom — done.)
- Full glitch correction (Pauldelbrot/multi-ref) — base multi-ref path exists; broaden + thread.
- Left Parameters panel; dual-view splitter; tile cache + pan reprojection (live reprojection ✓).
- Autopilot steering modes (minibrot-seek/boundary-track) — the adjustable dive limit + stepped
  deep dive shipped (0.1.15–0.1.18); smarter *steering* is still open.
- XaoS-style continuous-zoom pixel reuse (reuse-first zoom) — the biggest remaining smoothness win
  (foundation exists as a deep-zoom stall fallback; not yet the primary path).
- Tile-level export pipeline (async readback); histogram/equalized coloring; benchmark CSV/JSON history.
- Done since last snapshot (0.1.11–0.1.18): third-party license notices + Help→Licenses ✓, reset
  application state + versioned session + `FRACTADYNE_CONFIG_DIR` ✓, restartable tour renders
  (`--resume`) ✓, adjustable auto-zoom dive limit + stepped deep dive + toolbar button + Esc-stops ✓,
  deep-zoom sample location + helper scripts (`setup`/`render-deepest`/`render-spiral-dive`) ✓,
  softened "unlimited" → concrete depth claims ✓, BLA on by default ✓.

## Key files
- `crates/fractadyne-core/src/lib.rs` — Viewport (BigFloat center), `reference_orbit`,
  `orbit_points`, `best_reference`, `lerp_bf`, `set_center_mag`, precision helpers.
- `crates/fractadyne-gpu/src/mandelbrot.wgsl` — iterate+color shaders; `Cdf` (df32) +
  `Fe` (floatexp) helpers; `fs_iterate` 3-tier branch.
- `crates/fractadyne-gpu/src/lib.rs` — Renderer, per-view resources, `IterUniforms`
  (`delta_exp`), `MandelbrotParams`, `render_export` (tiled).
- `crates/fractadyne-app/src/` — app split into modules: `main.rs` (app struct + `update()` UI),
  `render.rs` (mode select + reference recompute / freeze-reproject), `autopilot.rs` (auto-zoom),
  `scripting.rs` (tours + `render_tour_to_dir`), `help.rs` (`CLI_REFERENCE` + Help window),
  `export.rs` (view-metadata / `.fdn`), `fractal.rs` (`FractalKind`), `cli.rs` (headless modes),
  `theme.rs`, `profile.rs`, `sysinfo.rs`, `selftest.rs`.
- `crates/fractadyne-app/build.rs` — build counter → `FRACT_BUILD` (`.build_seq` at repo root).
