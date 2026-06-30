# Fractadyne — Development Tracking

Living backlog. Specs: [DESIGN.md](DESIGN.md), [UI-DESIGN.md](UI-DESIGN.md).
Mockups: [design/mockups/](design/mockups/).

## Design follow-ups (from mockup review, 2026-06-25)

- [ ] **CA 2-D Birth/Survive rows only go 0–5; must be 0–8.** A 2-D life-like cell
  has up to 8 neighbours, so rules like HighLife (B36/S23) can't be expressed.
  Fix in mockup `14` and in the real CA 2-D rule editor. (DESIGN.md §4.1)
- [ ] **Mockup `12` footer says "1 rule" but shows 2 rows** (the 2nd, `G →`, is an
  empty placeholder). Cosmetic — fix the count or drop the empty row.

## Milestone M0 — Foundations ✅ complete

Goal (DESIGN.md §15): workspace + wgpu/window/egui + a basic Mandelbrot
(f32 in-shader, f64 CPU viewport) + pan / wheel / box zoom.

- [x] Cargo workspace + 9-crate skeleton (DESIGN.md §10)
- [x] `fractadyne-core`: `Viewport` (f64) + pixel↔complex + pan/zoom + unit tests
- [x] `fractadyne-gpu`: Mandelbrot render pipeline (WGSL, smooth coloring) via an
      `egui_wgpu` paint callback
- [x] `fractadyne-app`: eframe shell (menu bar + canvas + status bar)
- [x] Pan (left-drag) + cursor-centered wheel zoom
- [x] **Box-zoom (right-drag rectangle)** — `Viewport::zoom_to_rect` + an amber
      selection overlay drawn over the canvas; verified by a unit test (6/6 pass).
- [x] **Continuous zoom** — hold **Space** (in) / **Shift+Space** (out),
      cursor-anchored; exponential rate (~2× per 1.5 s) with ease in/out for a
      relaxing glide. Tunable via `ZOOM_RATE` / `EASE_TAU` in `fractadyne-app`.
- [x] **Build verified** — `cargo build --workspace` succeeds; `cargo test -p
      fractadyne-core` passes **5/5**. rustc 1.96 + egui/eframe/wgpu 0.31; no code
      changes were needed (the API usage compiled clean) and the `fractadyne`
      binary links.
- [x] **Launch the GUI window** — verified live: Mandelbrot renders, drag-pan and
      wheel-zoom work (confirmed to 307× with crisp smooth-colored detail).

### Environment workarounds (this machine)

- **Small Windows page file → OS error 1455** when mmapping the large debuginfo
  rlibs (naga ~55 MB). Fixed via `[profile.dev] debug = false` in `Cargo.toml`
  (rlibs shrink ~2–10×). Re-enable `debug = "line-tables-only"` after enlarging
  the page file.
- **Cargo pipelining** produced unusable rmeta stubs here → disabled in
  `.cargo/config.toml`.
- Build at `-j 1`; it **self-resumes on retry** (transient `LNK1105` temp-file
  locks were a symptom of the same memory pressure).

### Build

```
# Requires the Rust toolchain (rustup). First build pulls wgpu/egui (~minutes).
cargo run -p fractadyne-app        # launch the app
cargo test  -p fractadyne-core     # viewport math tests
```

Pinned: egui / egui-wgpu / eframe **0.31** (eframe with the `wgpu` backend; wgpu is
used via `egui_wgpu`'s re-export so versions can't drift). If a different egui
release resolves, a few wgpu descriptor fields (`entry_point: Option<&str>`,
`compilation_options`, `cache`) may need tweaking — these match the wgpu that
eframe 0.31 ships.

## Milestone M1 — Coloring & state (in progress)

Goal (DESIGN.md §15): real coloring (palettes + the compute↔coloring split), tile
cache, adaptive iterations, auto-save/restore, and the first side panels.

- [x] **Preset palettes** — Ember / Ice / Nebula / Grayscale as gradient stops,
      interpolated in-shader, cyclic (`fractadyne-color`).
- [x] **Coloring side panel** — palette picker + Cycle / Offset / Max-iter sliders,
      live updates (first real panel from the mockups).
- [x] **Adaptive iteration count** — `Viewport::recommended_max_iter` scales iters
      with zoom octaves; Auto toggle + base slider in the Coloring panel.
- [x] **Compute↔coloring split** — iterate to an offscreen `R32Float` texture
      (recompute only on view/iter/size change); recolor every frame from it.
- [x] **Auto-save / restore** — session (location/zoom/coloring) persisted to TOML
      in the OS config dir, debounced + atomic on change/close (`fractadyne-state`).
- [ ] **Tile cache + pan reprojection** — reuse computed tiles on pan/zoom (builds
      on the split; cheap navigation at depth + export groundwork).
- [x] **Palette animation** — Coloring panel "Animate" (Off / Forward / Reverse /
      Ping-pong / **Random gradients**) + logarithmic Speed slider; modes shift the
      color offset, **Random** synthesizes & continuously morphs gradients (seamless
      endpoints) with a "Shuffle gradient" button. Mode + speed persist.
- [x] **Harmonious random palettes** — `gen_stops` now uses one base hue + a gentle
      analogous excursion + a smooth dark→bright→dark `sin(πt)` arc (seamless), moderate
      constant saturation, dim ends — flowing/tasteful instead of a clashing rainbow.
      (Later polish: random complementary-pair / monochrome flavors.)
- [x] **Deep floatexp blank render investigated** — forced floatexp shallow renders
      clean → floatexp is healthy; the reported blank was a featureless fast-escape
      region (uniform escape ~iter 276, fast frame), not a render bug or build regression.
      A localized single-reference *glitch* remains possible (needs multi-ref correction).
- [x] **Orbit overlay polish** — tapered gradient polyline (thick/warm at z₀ →
      thin/magenta at the tail) instead of a flat line, with green z₀ / red last dots.
- [~] **Real (high-precision) orbit at depth, cursor-following** — past ~1e12× the
      overlay iterates in **bignum from the cursor's arbitrary-precision coordinate**
      (`pixel_to_complex` → `reference_orbit`), recomputed on cursor/view change and
      **cached** (`orbit_cache`). Runs toward escape (cap `ORBIT_MAX_DEEP=8192`) so the
      divergent (cursor-sensitive) tail shows, and **trims the `|z|>4` blow-up** so the
      normalized fit isn't dominated by one escaping iterate. Below ~1e12× uses the f64
      cursor orbit. **PENDING: confirm it reshapes as the cursor moves at deep zoom
      (build 16).** If still static at extreme depth, raise the cap / cap to eff_iter.
- [x] **Zoom display formatting** — magnification shows scientific notation with a
      12-digit mantissa above 1e12×; large integer magnifications drop the cluttering
      `.00`; small zooms trim trailing zeros.
- [x] **Dual-view toolbar icon** — custom-painted "two side-by-side rectangles"
      (`dual_toggle_button`), so it reads as the split view regardless of font glyphs.
- [ ] **Gradient stop editor** (custom palettes) — the one custom widget (UI-DESIGN §8).
- [x] **Bookmarks / presets library** — Bookmarks menu (+ ★ toolbar button) saves the
      current view (full-precision center via the export view-metadata blob) to
      `bookmarks.toml` in the config dir; click any bookmark to jump back instantly.
      Manage… window adds (with optional name), lists with zoom, and deletes. Invaluable
      now that deep zoom is unlimited (re-zooming to 1e30× by hand is painful).
- [ ] **Left Parameters panel** (type, power, location, zoom).

### Planned settings (Preferences UI — mockup 10)

- [x] **Continuous-zoom rate** — "Zoom speed" slider (0.25×–4×, log scale) in the
      side panel's NAVIGATION section; multiplies `ZOOM_RATE`, persisted with session
      state (`SessionState::zoom_rate`, `serde(default)` so old files still load).
      *(Requested 2026-06-26.)*

## Milestone M2 — Deep zoom (in progress)

Goal (DESIGN.md §5): unlimited zoom via arbitrary-precision reference orbit + GPU
perturbation + series approximation + glitch correction. The headline feature.

- [x] **Perturbation pipeline** — CPU reference orbit (`f64`) → per-pixel `δz` on
      the GPU. Verified live; pushes usable zoom well past the `f32`-direct limit.
- [x] **Reference picker** — choose a long-orbit/interior reference within the view
      (`best_reference`); fixed the short-orbit interior artifacts. **Now scores
      candidates in bignum** (`orbit_length_bf`, capped) — f64 scoring collapsed at deep
      zoom and made cold jumps (bookmark reload / Open view / `--render`) pick a poor
      reference → uniform/glitch. Added BigFloat string round-trip tests (the bookmark
      coordinate round-trips to ~1e-79, so it was never a precision problem).
- [x] **Rebasing** (Zhuoran) — single-reference glitch handling; killed the on-zoom
      speckle and self-heals short references.
- [x] **Supersampling (SSAA)** — Anti-alias control (Off / 2× / 3×); averages an
      ss×ss block of samples to remove boundary aliasing at depth.
- [x] **Double-single (df64) reference** — reference stored hi/lo; perturbation uses
      the `Z_lo` correction. Removes the reference-precision noise (dominant ~1e6×).
- Note: df64 reference + rebasing render **cleanly to ~5×10¹⁴×** (verified) — far
  past estimate; reference/delta are no longer the limit. The **f64 center
  coordinate** is now the wall.
- [x] **Double-double (df64) center** — the deep-zoom coordinate **jump is gone**.
      Verified clean *and* smooth to **~4×10³⁰×** (df64 reference + rebasing held far
      past estimate — no noise; the earlier ~10¹⁵× GPU-noise prediction was wrong).
- [x] **Arbitrary-precision center + reference (unlimited zoom)** — center is now
      `astro_float::BigFloat` at a mantissa size that scales with zoom
      (`precision_for_magnification` = octaves + 64 guard bits), so the coordinate
      never runs out of digits → no jump at *any* depth. The reference orbit is
      iterated in bignum on the CPU and handed to the GPU as df64 samples
      (`Arc<Vec<[f32;4]>>`). The GPU does no bignum (pure f32/df64 perturbation).
      Fast `BigFloat`→`f64` via direct mantissa/exponent bit-reconstruction
      (`core::to_f64`, validated by roundtrip test) — no slow string formatting.
- [x] **Reference-orbit caching** — bignum is slow, so the orbit is recomputed only
      when the reference leaves the view (>0.5 span) or, once the view settles, when
      precision/iterations grow (>0.4 span drift, or higher precision/iter). During
      motion the cached orbit is reused (smooth); refinement happens on settle.
      Caveat to watch: at very deep *continuous* zoom a recompute can micro-stutter;
      optimize later (precision headroom / async recompute) if it shows.
- [x] **Double-single (df32) perturbation delta** — `δz` (and per-pixel `δc`) carried
      as hi/lo f32 pairs with compensated add/mul (`two_sum`/`two_prod` via fused
      `fma`) in the shader. `δc` is built from the *exact integer* texel coordinate ×
      a df32 per-texel step (uniform `step` + `res`), so it isn't pre-truncated by
      f32. Removes the interior speckle that appeared past ~10¹⁵× (was f32 delta
      precision, **not** iterations); should hold clean to ~10²²–10²³×. Assumes a
      fused `fma` (true on NVIDIA/AMD/Intel targets). df64 delta later if needed.
- [x] **Full-precision persisted center** — session now stores `center_x_str`/
      `center_y_str` (decimal, full precision) and restores via `parse_bf` (fallback to
      the old f64 fields). Deep-zoom locations now survive quit/restart instead of
      truncating to f64 → a wrong spot → uniform screen. Also fixed the autosave
      debounce so an animating palette offset no longer blocks the idle save.
- [ ] Re-add `zoom_to_rect` unit test (dropped in the dd rewrite).
- [x] **UI digit separators** — commas on zoom/iter, spaces grouping coordinate digits.
- [x] **Floatexp perturbation δ (unlimited depth)** — the df32 δ has f32's *exponent*
      floor, so its low word denormalizes/underflows ~1e31–1e32× → speckle breakdown.
      Added a floatexp δ (df32 mantissa + i32 exponent) that never underflows. **Hybrid
      by depth**: direct df32 (<1e4×) → df32 perturbation (1e4–1e28×, fast) → floatexp
      perturbation (≥1e28×, ~1.7× costlier, only when needed). Shared base-2 `delta_exp`
      keeps the input δ mantissas (step / ref_offset) O(1) at any depth. Verified clean
      via `--render` at 1e15/1e25/1e27(df32)/1e29/1e32(floatexp); shallow unchanged;
      crossover seamless. Benchmark score held (3220). Depth now bounded by the center
      coordinate precision (auto-scales while zooming) + iteration budget, not f32.
- [x] **Lifted the ~1e308× render ceiling (extended-range `FloatExp` scale)** — the viewport
      scale was `f64` (`units_per_pixel` underflowed, `magnification()` overflowed near
      1e308×), which was the real live-zoom wall (the bignum center was already unlimited).
      Replaced it with a `FloatExp` (`m·2^e`, i32 exponent): `Viewport::units_per_pixel` is
      now `FloatExp`, with `log2_magnification` + `precision_for_octaves` driving precision,
      `complex_span_fe`/`gpu_scale` (O(1) span mantissa + shared `delta_exp`) and
      `ref_offset_mantissa` feeding the GPU (the shader was already exponent-aware — no WGSL
      change), `set_center_log2mag` + `--zoom-log2` for deep jumps, session persistence via a
      stored exponent, and `fmt_zoom_log2` for the readout. Verified: bit-identical to 1e30×
      (selftest goldens), GPU renders correctly at **1e331×**, no regression. *(Follow-ups:
      goto-dialog + exported-image metadata still take f64 zoom — fine to ~1e308×.)*
- [x] **Deep goto / exported-metadata zoom past 1e308×** — the "Go to location" dialog and
      the reloadable PNG/EXR view-metadata encoded zoom as `f64`, so a view deeper than
      ~1e308× lost its scale on reload (the center was fine). Goto now parses/formats via
      `log2(magnification)` (`parse_zoom_to_log2` / `fmt_zoom_field` — accepts `1.5e400`,
      clamped to a sane octave bound); the metadata blob carries an extended-range
      `upp_log2` (reconstructed on load, with the f64 `upp` kept for back-compat), so
      exported images and bookmarks restore deep views exactly. Round-trip unit-tested.
- [ ] **Full glitch correction** (Pauldelbrot criterion + multi-reference recompute).
- [x] **AA auto-drop during motion** — full AA only when the view settles (smooth
      deep zoom; sharp still image).
- [x] **Reference refresh during motion (anti-"impressionist")** — the reference orbit
      now refreshes while zooming (not just on settle) when out of view / under-precise,
      adaptively throttled (~2.5× last recompute duration) so deep zoom stays sharp in
      motion without stalling FPS. Supersedes the earlier "defer entirely during motion"
      that left stale references → blotchy frames. AA still applies only on settle.
- [x] **Hybrid direct/perturbation** — below **1e4×** iterate `z²+c` directly in df32
      (glitch-free); perturbation at/above 1e4×. `mode` uniform + df32 `center`; direct
      path shares the coloring/AA pipeline. Crossover is conservative: direct iteration
      accumulates rounding error and breaks down ~1e6× (random noise — that's *why*
      perturbation exists), so hand off long before. Verified clean at ~2e6×.
- [x] **Higher AA (4×/8×) + persisted AA** — exterior "speckle" was diagnosed (via
      the glitch-free direct path) as **undersampling of real sub-pixel exterior
      dust**, NOT precision or glitches: it persisted without perturbation and
      cleaned up at 8×. Added 4×/8× options (8× auto-reduced to fit the GPU texture
      limit) and persisted the AA choice (`SessionState::aa`). AA only runs on settle,
      so motion stays smooth.
- [ ] **Smarter exterior sampling** — adaptive/jittered supersampling or higher
      export-time AA so the dense dust sea is clean without brute 8× every frame.
- [ ] **Coloring tuning** — optionally scale color cycling with zoom so steep
      escape-time gradients don't read as grain at default Cycle.

## Milestone M3/M4 — Fractal variety & dual view (in progress)

- [x] **Fractal type system** — `Fractal` menu lists 10 escape-time families:
      Mandelbrot, Multibrot 3/4/5, Tricorn (Mandelbar), Burning Ship, Celtic, Buffalo,
      Phoenix, Newton (z³−1). Shader carries a `formula` id + `julia` flag (decoupled
      from the formula), with complex-df32 helpers (mul/sqr/div, `Cdf` struct) for
      powers and Newton. **Julia mode** is a toggle (Fractal menu) for any family; the
      **dual view** shows each family's map ↔ its Julia. Mandelbrot/Multibrot/Tricorn
      and the abs families (Burning Ship/Celtic/Buffalo) all deep-zoom at floatexp
      range; Phoenix/Newton are direct df32 (clean to ~1e6×).
- [x] **Per-fractal info panel** — collapsible section atop the side panel with the
      formula, a short background, and a reference hyperlink (Wikipedia / Paul Bourke),
      sourced from `FractalKind::info()`.
- [x] **Dual linked view** — View menu → "Dual view (Mandelbrot ↔ Julia)". GPU
      renderer refactored to per-view resources keyed by `view_id` (each panel has
      its own texture/uniforms/orbit/caching). Left = Mandelbrot, right = Julia;
      hovering the Mandelbrot sets the Julia `c` live. Each panel pans (drag) and
      wheel-zooms independently.
- [x] **Dual-view interaction** — per-panel drag-pan + wheel-zoom + continuous (Space)
      zoom toward the cursor; hovering the Mandelbrot drives the Julia `c` live (uses
      the global pointer position — per-widget hover was unreliable since both panels
      allocate from the same source line). Reset resets both panels in dual mode.
- [x] **Performance overlay + diagnostics** — draggable overlay (FPS, cpu vs gpu/idle,
      reference-recompute ms/rate, mode/iter/precision/orbit/zoom) + `[perf]` stderr
      log. On by default; toggle in View, or `--no-perf`. Caught a per-frame reference
      recompute loop (`best_reference` sits ~0.4 span off-center, which the stale check
      mis-flagged) that pinned the app at ~2 FPS; fixed → ~60+ FPS.
- [x] **Frame-rate cap** — View → Frame-rate cap (Uncapped/30/60/120, default 60),
      persisted; enforced by pacing the main loop (request_repaint_after is a deadline,
      not a throttle).
- [x] **Deep zoom for the analytic families** — perturbation generalized to
      Multibrot 3/4/5 and Tricorn (exact polynomial / anti-holomorphic δz series),
      in **both Mandelbrot and Julia modes**, sharing the bignum-reference + rebasing
      pipeline. Core `reference_orbit`/`best_reference`/`orbit_length` are now
      formula+mode aware (`step_bf`/`step_f64`, `cmul_bf`); the shader's perturbation
      branch carries δz in `Cdf` with the per-formula series. Verified vs the direct
      path at boundary regions (mean diff 0.001–0.4%).
- [x] **Per-view reference caches** — `ref_cache[2]` (main/left + dual Julia), each
      with its own orbit / ref_pt / orbit_id, so **both dual-view panels deep-zoom with
      perturbation** independently (previously dual was direct-only → the deep panel
      pixelated). `invalidate_refs()` drops both on formula/mode/center change.
- [x] **GPU watchdog (TDR) guard** — a heavy live render (high AA × deep iterations,
      esp. both dual panels) could exceed the OS GPU watchdog (~2 s) and crash with a
      device-lost error during `Queue::submit`. Added a per-render `WORK_BUDGET`
      (texels × iterations): supersampling auto-reduces on heavy frames (and, only on
      a very large window at extreme depth, the GPU iteration count is clamped) so a
      single submission stays well under the watchdog. Verified the previously-crashing
      deep dual Multibrot 5 8× case now survives. (Export already tiles, so it's safe.)
- [x] **Julia deep-zoom rebasing fix** — Zhuoran rebasing reset `δz = z_full`, which
      assumes `reference[0] = Z₀ = 0`. True for Mandelbrot, but a Julia reference orbit
      starts at `Z₀ = ref_point ≠ 0`, so every rebase offset the perturbation by `Z₀`
      and corrupted deep Julia renders (worse the deeper you go, as rebasing fires more
      often). Fixed by rebasing to `δz = z_full − reference[0]` (no-op for Mandelbrot).
      Applies to all analytic families in Julia mode + exports (shared shader).
- [x] **Burning Ship / Celtic / Buffalo perturbation** — sign-aware (abs) deep zoom
      for the non-analytic families. The abs fold on a z² component becomes a `diffabs`
      step `|c+d|−|c|` (KF/Zhuoran), evaluated branch-wise to avoid catastrophic
      cancellation: exactly `±d` when the reference and perturbed component share a sign,
      `±(2c+d)` across a sign flip (a wrong branch at a near-fold is the inherent glitch).
      Core `step_bf` gained the bignum reference iterations (5/6/7). The shader folds
      each abs component with diffabs in BOTH render paths: `df_diffabs` in the df32
      loop (mode 0, ~1e4×…~1e28×) and a scalar-floatexp `sf_diffabs` in the floatexp
      loop (mode 2, past ~1e28× — the complex `Fe` shares one exponent across re/im, so
      the per-component fold drops to a scalar `Sf` then recombines via `fe_from_sf`).
      So they now deep-zoom at floatexp range like the analytic families (vs ~1e6×
      direct before). Validated in `--selftest`: perturbation == direct at 1e5× (exact
      Burning Ship/Buffalo, mean Δ 0.18 iter Celtic, 0 px >2 iter), floatexp == df32 at
      1e10× (exact, all three), finite + structured at 1e35×. Lighting/DE stay off
      (non-holomorphic). Remaining: multi-reference glitch correction for the residual
      speckle at the abs folds (where a tiny df32 reference z² component flips the
      diffabs branch — same root cause as Mandelbrot perturbation glitches).
- [ ] **Newton / Phoenix deep zoom** — Newton is convergence-based; Phoenix needs the
      previous-iterate δz term + rebasing care. Both still direct.
- [x] **Click-to-pin Julia `c`** — in dual view, click the Mandelbrot to freeze the
      Julia at that point (a marker is drawn there); click the marker to release and
      resume live cursor-follow. Pinning also stops the per-move Julia reference
      recompute, so it's smoother at depth.
- [x] **Export hotkey (Ctrl+S)** — quick-saves the current view to the last-used folder
      with an auto timestamped name (no dialog), using current export settings.
- [x] **Dual export layouts** — Export dialog "Dual layout": Side-by-side (one
      stitched file, default), Separate files (`…_map` / `…_julia`), or Map only.
      Persisted. Verified the side-by-side stitch.
- [x] **Action toolbar** — fractal **dropdown** (the name is a picker), Julia + Dual
      toggles, Export / Gallery / Open… / Reset, Perf toggle. **Merged with the menu
      bar** on one `horizontal_wrapped` row: shares the menu line when the window is
      wide, wraps below when narrow. Action buttons use **emoji icons** (💾 🖼 📂 🔄
      📷 🔍± 🎨 📊 🖥) with tooltips; File-menu items are icon-prefixed.
- [x] **Docked performance panel** — the perf diagnostics render as a "PERFORMANCE"
      section at the **bottom of the right-hand control panel** (toggle via the Perf
      button) instead of a floating window. The whole right panel is **hidden in
      fullscreen** for an edge-to-edge view.
- [x] **More toolbar buttons** — Snap (quick export), Zoom +/− (about center),
      Palette (cycle preset), Fullscreen toggle, **Home 🏠 (animated zoom-out)**.
      (AA cycle / pin-release still candidates if wanted.)
- [x] **Animated "zoom home"** — 🏠 smoothly glides back to the default view (vs the
      instant 🔄 Reset). `Viewport::home_lerp` sets magnification from a log-mag track
      and lerps the center with `frac = 1 − 1/mag` so the focal point stays on-screen
      during the zoom-out (a linear lerp flings it off at depth). Eased (smoothstep),
      duration scales with depth (1.5–9 s), animates both panels in dual, treated as
      interaction (AA off / references deferred), and any pan/zoom/Space cancels it.
- [x] **Esc exits fullscreen**, in addition to the 🖥 toolbar toggle.
- [x] **Orbit overlay** — View → "Show orbits" draws the iteration path of the point
      under the cursor (`core::orbit_points`, f64, matches the shader's per-formula
      direct step incl. Burning Ship/Celtic/Buffalo/Phoenix/Newton). z₀ green, last
      iterate red, connecting polyline; works in single and dual (hovered panel).
- [x] **Higher max iterations** — base slider raised 4000 → **50,000** (logarithmic)
      to match the auto-scale cap; useful for deep minibrots / thin filaments.
- [ ] **Dual-view polish** — draggable splitter.
- [x] **Release build** — `[profile.release]` (debug=false, lto=false, codegen-units=16
      to bound compile memory) builds clean here. Measured via `--benchmark`: bignum
      **reference recompute 374 ms → 45 ms (~8×)**, avg CPU 2.5 ms → 0.33 ms (~7.6×),
      score 2750 → 3031. Deep-zoom recompute stutter cut ~8×; steady-state FPS is
      GPU-bound so only +10%. Build with `cargo build --release -p fractadyne-app -j 1`.
- [ ] L-systems, cellular automata (1-D & 2-D).

## Milestone M5 — High-res export (in progress)

- [x] **PNG / OpenEXR export** — `File → Export image…`: pick width (1280–7680),
      supersampling (1–4×), and format. Renders the current view offscreen at the
      chosen resolution (`fractadyne_gpu::render_export`: iterate → color → readback),
      then encodes via `fractadyne-export` (8-bit sRGB PNG with the linear→sRGB OETF;
      32-bit float linear EXR). Saves to the user's Pictures dir, timestamped; the
      dialog shows the resulting path. Reuses the live precision/AA/coloring pipeline
      so deep Mandelbrot exports use perturbation. Verified end-to-end (PNG + EXR).
- [x] **Tiled export** — renders in ≤2048-px tiles (sized to the texture + buffer
      limits) via a per-tile `px_offset` uniform, assembled on the CPU. Removes the
      ~8192 single-texture cap and fixes the large-size crash (was exceeding
      `max_buffer_size`). Verified seamless at 3840×2903.
- [x] **Native save/open dialogs** (`rfd`) — Export uses a Save dialog; `File ▸ Open
      view…` uses an Open dialog. Export width / supersampling / format and the **last
      save directory** persist in the session and default the dialogs.
- [x] **Reloadable PNG metadata** — exported PNGs embed a `tEXt` chunk with the full
      view state (fractal, Julia mode + c, **full-precision** center via
      `core::to_decimal_string`/`parse_bf`, units-per-pixel, iterations, palette,
      cycle/offset, AA). `File ▸ Open view…` restores it to continue exploring.
- [x] **EXR metadata** — same view state embedded as a custom `Fractadyne` OpenEXR
      attribute (write + read); `File ▸ Open view…` now accepts PNG *and* EXR.
- [x] **Background export** — render + encode run on a worker thread; the UI stays
      responsive (status polled via a channel; Export button disabled while busy).
      *(Cancelation is still TODO.)*
- [x] **Richer metadata + Notes** — exports now embed `app=Fractadyne`, `version`,
      `saved` (UTC date) + `saved_unix`, and a user **Notes** field (≤120 chars, in the
      Export dialog) alongside the view state.
- [x] **Export progress bar + cancel** — the Export dialog shows a live `ProgressBar`
      (% of tiles done) while rendering, with a **Cancel** button. `render_export`
      reports per-tile progress (permille via `AtomicU32`) and checks a cooperative
      `AtomicBool` cancel flag each tile (returns "canceled"). Verified: normal render
      reaches 100%, pre-set cancel aborts. Encode/write shows a distinct "Saving…"
      phase (progress sentinel ≥2000); default filename is now `..._YYYYMMDD_HHMMSS`.
- [x] **Gallery / metadata browser** — `File ▸ Gallery…` scans a folder (default
      Pictures, switchable) for exported PNG/EXR with Fractadyne metadata, newest
      first, showing a **thumbnail** + parsed metadata (fractal, zoom, saved date,
      notes, app/version) and a **one-click "Open this view"** to jump back in.
      Thumbnails decode lazily (one/frame, box-downsampled, cached as egui textures).
- [x] **EXR thumbnails** — gallery now decodes EXR too (`read_first_rgba_layer`,
      box-downsampled, linear→sRGB), so EXR entries get real thumbnails like PNG.

## Tooling, scripting & versioning (M7)

- [x] **Versioning + changelog** — workspace at **0.1.0**; `build.rs` auto-increments a
      per-build counter (`FRACT_BUILD`) shown as `v0.1.0 (build N)` in the title bar,
      Help menu, and export metadata. [CHANGELOG.md](CHANGELOG.md) tracks changes.
- [x] **Scripting (camera tours)** — Tools → "Play script…" loads a TOML of keyframes
      (`secs`, `center_x/y`, `mag`, `fractal`, `julia`; centers inherit if omitted) and
      glides center (BigFloat lerp) + log-magnification (eased) along the timeline.
      `core::set_center_mag` / `lerp_bf` drive it; Esc or Tools → Stop ends it.
- [x] **Benchmark** — Tools → "Run benchmark" plays a fixed deep-zoom tour and samples
      FPS (avg/min/max), CPU ms, GPU ms (frame−cpu), and RAM (working set + peak via
      `K32GetProcessMemoryInfo`), reporting aggregates + score in a copy/save-able window.
- [x] **Benchmark system info** — report includes CPU brand (CPUID), physical/logical
      cores + L2/L3 cache (GetLogicalProcessorInformation), GPU name (wgpu adapter), and
      VRAM (display-adapter registry). Verified: Ryzen 9 3950X / RTX 3080 / 10 GB.
- [x] **CLI benchmark** — `fractadyne --benchmark [--out PATH]` runs the tour on
      startup, prints + saves the report, and quits (skips session autosave). Enables
      automated build/machine evaluation. Default out `fractadyne_benchmark.txt`.
- [ ] **Benchmark presets** — multiple scenes (Julia deep, Multibrot, dual) + CSV/JSON
      output and a results-history compare view.
- [x] **Headless render** — `fractadyne --render --out IMG [--fractal N --center X Y
      --zoom M --size W --ss N --iter K --julia --julia-c RE IM --palette I]` renders one
      image (reusing the tiled export + perturbation pipeline) and quits. PNG/EXR by
      extension; full-precision center. For debugging / automated golden-image checks.
- [x] **Development profiling harness** — `--profile` times the render stages (bignum
      reference orbit, series-approximation setup, GPU iterate / full render) per benchmark
      region and writes a JSON log to `logs/` with run context; `scripts/profile.ps1` runs it
      and `scripts/profile-compare.ps1` diffs before/after to validate optimizations. Logic in
      a `profile` module. *(Follow-ups: GPU timestamp queries to split iterate vs color
      precisely; opt-in per-frame logging of live interactive sessions; fold the series
      coefficients into the reference-orbit pass to cut the ~100 ms series-skip setup at depth.)*
- [ ] **Record-to-video / frame export** from a script (offline, deterministic).
- [ ] **Ship compiled binaries on GitHub (Releases)** — publish prebuilt executables so
      users don't need the Rust toolchain. Tag a release (`vMAJOR.MINOR.PATCH`) and attach a
      Windows `x86_64` build (the primary target; add Linux/macOS later if cross-building is
      set up). Automate with a GitHub Actions workflow (`release.yml`) triggered on tag push:
      build `--release`, then upload the artifact + a SHA-256 checksum to the release. Note
      the local build constraints (`-j 1`, `debug=false`, no pipelining) are this machine's
      page-file workaround — CI runners likely don't need them, but the release profile must
      still link cleanly. Include the README build note + a short "Download" section.
- [x] **File format versioning + minimum-version validation** — the reloadable view metadata
      (exports / `.fdn` / bookmarks) now has a single source-of-truth `VIEW_FORMAT_VERSION`
      (export.rs); the writer emits it and `load_view_metadata` returns a `ViewLoad`
      (`Ok` / `NewerFormat(v)`). A file whose `format_version` exceeds this build's loads
      best-effort (the format is additive key=value, so core fields still parse) but the
      untrusted callers — Open-view and Apply-location — surface a clear "saved by a newer
      Fractadyne; some settings may not apply, consider updating" message instead of
      silently mis-loading. Same pass **hardens the untrusted parser**: `max_iter` clamped
      to ≤1e7, `aa` to 1..16, zoom depth to ≤3.4e7 octaves (prevents a hostile `upp_log2`
      from ballooning bignum precision into a memory DoS), and `cycle`/`offset` rejected if
      non-finite. `--selftest` covers round-trip, newer-version detection, and clamping.
      (A file is missing `format_version` ⇒ treated as v1; legacy files still load.)
      Possible later: an explicit `min_app_version` for forward signaling of hard breaks.

## Branding & UI (M7)

- [x] **Fractadyne theme + branding** — dark deep-space theme with cyan/magenta accents
      (`apply_brand_theme`), painted brand mark + wordmark in the top bar
      (`brand_wordmark`), and a procedural window icon (`brand_icon`).
- [x] **Animated relief lighting** — "Rotate light" spins the light direction over time
      (shares the Speed slider), complementing the animated distance glow + palette cycle.
- [ ] **Theme polish** — optional light/preset themes, custom font, accent picker.

## Survey-driven roadmap (2026-06-28)

Gaps vs. Ultra Fractal / Kalles Fraktaler / XaoS / Mandelbulber / Apophysis, prioritized
for fun, informative value, and ease of use.

### Tier 1 — best value, good fit for the escape-time engine
- [x] **Distance-estimate slope/relief lighting** — tracks the derivative `dz/dc`
      (`dz/dz0` in Julia mode) and shades by the slope normal → embossed, lit 3D look.
      Works on the **direct path** (Cdf derivative) and the **perturbation paths**
      (floatexp derivative, so it holds at any depth — verified at 1e8×). Holomorphic
      families (Mandelbrot / Multibrot 3/4/5). Iter texture now RGBA32F (r=iter,
      g/b=normal, a=reserved for DE); light angle/relief live in the color pass so they
      re-light without re-iterating. Coloring panel toggle + angle/relief sliders;
      `--light [--light-angle R]` CLI; persisted.
- [x] **Distance-estimate glow + animation** — the derivative magnitude → distance
      estimate (stored as log2(pixels) in the iter texture's alpha); the color pass draws
      bright distance-contour bands that densify into glowing filaments near the boundary.
      Coloring panel: "Distance glow" toggle + Glow strength + Band width + "Animate glow"
      (flows the bands, shares the Speed slider). `--de` CLI; persisted. Works direct +
      perturbation (verified at 1e8×).
- [x] **More coloring methods** — Coloring → "Method": stripe average (+ density),
      triangle-inequality average (TIA), orbit trap (point/cross/circle, colors interior),
      distance estimate, and decomposition. Orbit stats accumulate into a second GPU
      render target (only when a method needs it); works at any depth (direct + both
      perturbation paths). Persisted; `--method/--stripe-freq/--trap` CLI. *(Follow-up:
      histogram/equalized auto-coloring still open.)*
- [x] **Goto-location dialog + navigation undo/redo** — View → "Go to location…":
      view/edit/paste/copy the exact center (full precision) + zoom, with validation.
      Navigation history records each settled location (+ discrete jumps); **Backspace**
      = undo view, **Shift+Backspace / Ctrl+Y** = redo (also in the View menu), gated so
      it doesn't fire while typing. (Single view; dual skipped.) *(Follow-up: this is
      the basis for the `.fdn` share format — same key=value, hardened parse.)*
- [x] **Period / minibrot finder ("zoom to center")** — View → "Find minibrot center"
      (or **M**) snaps the view center to the nearby minibrot's exact nucleus and reports
      its period via a transient toast. Detects the atom-domain period (global argmin of
      |Zₙ|), Newton-refines `c` so the critical orbit closes (`Z_period(c)=0`) in
      arbitrary precision, then recovers the true smallest period; rejects runaway Newton
      / non-nuclei. Holomorphic families (Mandelbrot / Multibrot). Unit-tested
      (period-2 → c=−1, period-3 bulb); verified deep (period-998 at 2e7×). Headless
      `--find-minibrot --center X Y [--zoom M] [--fractal NAME]`.
- [x] **Minimap / "you are here" overview + zoom-depth context** — View → "Minimap
      overview" shows a small static home-view thumbnail (rendered once per fractal/
      palette/method via the export pipeline) in the bottom-left, with a "you are here"
      marker (view rectangle when shallow, crosshair when sub-pixel deep) and the live
      zoom-depth label. Click to jump to a region at home zoom. Persisted; single
      Mandelbrot-mode only (hidden in dual / Julia).
- [x] **Gradient / palette editor** (custom palettes) — Coloring → "Edit gradient…" (or
      the "Custom" palette entry) opens an editor with a live gradient preview, per-stop
      color picker + position slider, add/remove stops (up to 8), and "Copy preset…" to
      seed from a built-in. Custom gradient persists and renders everywhere (live, export,
      minimap). Verified end-to-end via a custom-palette render.
- [x] **Famous-locations tour + "random interesting location" + help/keyboard overlay** —
      a **Locations** menu with curated named Mandelbrot spots (Seahorse/Elephant Valley,
      spirals, mini-Mandelbrot, a deep seahorse) that jump (full-precision) + a "🎲 Random
      location" that bisects to a random detail-rich boundary point and zooms in. A
      **Keyboard & controls** overlay (Help menu / **F1** / **?**) lists all shortcuts and
      feature tips. Famous coordinates verified to render detail.

### Tier 2 — high value, larger effort
- [x] **Shareable location `.fdn` + paste-text** — **File → "Share location…"** opens a
      dialog with the current view as a self-contained `.fdn` text blob (fractal,
      full-precision center, `upp_log2` so depths past 1e308× round-trip, zoom, coloring):
      **Copy** to clipboard, **Apply** a pasted/edited one, **Save .fdn… / Load .fdn…**.
      Untrusted input is handled safely — size-bounded (`SHARE_MAX`, plus a file-size check)
      and parsed through the existing **hardened, fuzzed** `load_view_metadata`/`meta_get`
      chain (key=value allow-list, every field validated/clamped, unknown keys ignored, no
      paths/code). *(Optional follow-up: QR-code generate/scan for the compact string.)*
- [x] **Auto-zoom (autopilot) — follow interesting areas downward** — hands-free continuous
      deep zoom that re-steers toward detail (XaoS-style), via **View → "Auto-zoom"** or the
      **A** key (Esc / any navigation input stops it). Every ~0.35 s it renders a small
      (56×56) iteration field of the current view through the live perturbation pipeline and
      scores each cell by **boundary adjacency + escape-time gradient**, center-biased for a
      stable dive; it eases the target and zooms toward it each frame (reusing `zoom_at` +
      the continuous-zoom rate), treating the dive as interaction (AA off, throttled
      reference refresh). Stops on a dead end (no boundary detail → flat interior/exterior)
      or the depth cap (~1e271×, the smooth regime). *(Follow-ups: minibrot-seeking /
      boundary-tracking steering modes; pairs with the planned zoom-movie export.)*
- [ ] **Zoom-movie / frame→video export** — build on scripting + headless render.
- [ ] **Layers + blend modes** (Ultra Fractal-style compositing).
- [ ] **Formula DSL / custom formulas** (M6).
- [~] **Series approximation** — order-3 polynomial (`δz ≈ A·δc + B·δc² + C·δc³`) seeds the
      perturbation and skips the early iterations. **Done for deep Mandelbrot (mode 2,
      ≥1e28×)**, non-Julia, non-aux coloring; coefficients iterated in bignum alongside the
      reference, skip chosen from the worst-case corner `|δc|` (cubic ≤ 2⁻¹⁶ of linear, which
      also guarantees no premature escape), cached per reference. Validated: seed vs full
      iteration `maxΔ 0` and 0 mismatches vs the bignum oracle at 1e30×. **Follow-ups:**
      extend to mode 0 (1e4–1e28×, the common range; needs a floatexp→df32 seed conversion)
      and to Multibrot/other formulas; consider **BLA** (bilinear approximation, Fraktaler-3
      style) which skips iterations throughout the orbit, not just the start.
- [ ] **Multi-reference glitch correction** (Pauldelbrot criterion + per-glitch recompute) —
      beyond the current single-reference Zhuoran rebasing.

### Tier 3 — big bets (separate engines)
- [ ] **3D fractals** (Mandelbulb / Mandelbox, ray-marched).
- [ ] **Flame / IFS fractals; L-systems; cellular automata.**

## Backlog (later milestones — DESIGN.md §15)

- **M4** more fractal variety: L-systems, cellular automata
- **M5** high-res tiled export (PNG / OpenEXR)
- **M6** programmable engine (formula DSL → WGSL + CPU; custom coloring)
- **M7** polish & perf

## Stub crates (created, awaiting their milestone)

`fractadyne-color` (M1) · `fractadyne-render` (M1/M2) · `fractadyne-state` (M1) ·
`fractadyne-fractals` (M4) · `fractadyne-export` (M5) · `fractadyne-ui` (panels, M1+).
