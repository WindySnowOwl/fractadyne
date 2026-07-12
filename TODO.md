# Fractadyne — Development Tracking

Living backlog. Specs: [DESIGN.md](DESIGN.md), [UI-DESIGN.md](UI-DESIGN.md).
Mockups: [design/mockups/](design/mockups/).

## Open bugs

- [ ] **App freezes on load at extreme zoom (~1e2100×, center −2.0, the Mandelbrot filament tip) —
  DIAGNOSED 2026-07-12.** A saved session at this arbitrary-depth view (`units_per_pixel_e = −6986`,
  ~7000-bit precision, `max_iter = 500000`) leaves the window unresponsive ("Not Responding") on
  boot. **Confirmed cause (headless repro at `--center -2.0 0.0 --zoom-log2 6980 --iter 500000`,
  `FRACTADYNE_TRACE=ref`): the SYNCHRONOUS cold-start reference build — only the very first cold
  reference is built on the UI thread (render.rs) — takes ~12 s at this depth: `best_reference`
  candidate scoring 8.6 s + bignum orbit build 1.45 s + BLA 0.2 s at 7172-bit.** The GPU render is
  NOT the problem: a full 1920×1080 frame iterates in 22 ms (maxiter=0, BLA skips 36M steps), so no
  pixel spin — it's purely the UI-thread freeze during the cold bignum build (dominated by the same
  `best_reference` lever the throughput work isolated, now scaled to 7000-bit). **Fix directions:**
  (1) make the cold-start reference build ASYNC like every subsequent one (off-thread + the existing
  placeholder/spinner) so the window stays responsive — the direct fix; (2) cap/shortcut
  `best_reference` scoring depth at extreme zoom (the throughput lever) to cut the 8.6 s; (3) a
  coarse→fine progressive cold reference. Higher severity than the XaoS enhancement (a stated
  arbitrary-zoom test case is unusable). Note it is NOT an infinite hang — it completes in ~12 s
  headlessly; deeper zoom / larger window / slower CPU stretches the freeze.

- [ ] **Glitch-correction pass goes pathological (>1 hour) at extreme depth — ROOT CAUSE
  DIAGNOSED 2026-07-11 (v0.2.11 tracing); a robust fix is a substantial change, deferred.**
  Original report: offscreen `--render` of corpus 09 (6.1e500×) hangs >1 h with
  `glitch_correct=true`, ~12 s with it off. **Diagnosis (measured at corpus 14 / 1e148×, which
  reproduces it):** the multi-reference loop (`render_corrected_iter`, render.rs) re-renders the
  WHOLE frame per correction pass with `bla_on=0`. The per-pass reference build is cheap
  (compute_reference ~0.4 s + build_bla), so it is NOT the cost — the **render** is: a single
  correction pass hung >150 s for just 47 glitched px (killed on pass 1). The killer is that
  glitch-corrected deep-INTERIOR pixels (the dark dendrite cores) must iterate the FULL 800k
  floatexp steps that no acceleration skips — and at these cores the per-iteration cost is ~50×
  normal (measured: a 33×33 window of cores took >100 s = seconds *per pixel*, vs a 129×129
  window elsewhere in ~1 s). One such render is a single **uninterruptible GPU dispatch**, so a
  wall-clock time-box checked between passes cannot bound it. **Fixes ATTEMPTED and shown
  insufficient (do not just retry):** (a) build a BLA tree for each correction reference —
  helped exterior/boundary passes (45–90 ms) but not the interior cores (BLA can't skip them);
  (b) render only a bounded window around the seed via a new `render_iter_region` — bounded the
  *pixel count* but a small window OF cores is still catastrophically slow; (c) a `span==0` BLA
  guard (defensive, harmless, didn't fix it); (d) a wall-clock time-box — can't interrupt a
  running dispatch. All reverted (kept the tree clean at v0.2.11). **The real fix must bound the
  per-DISPATCH work for expensive interior pixels** — e.g. tile the correction render with a
  work-budget *calibrated for these ~50×-cost pixels* (render_export's `TILE_WORK_BUDGET`
  assumes normal cost, so its tiles would still be too big here) and check the time-box between
  tiles; or cap per-pixel shader work with a hardware-measured bound; or (simplest safe stopgap)
  detect the deep-interior-core regime and SKIP correction there (return the base render), so it
  never hangs — same visible result as today's corpus workaround but automatic. Aux methods skip
  correction entirely (masked this for stripe sessions). Corpus staging pins `glitch_correct=false`.

- [x] **Uniform-exterior misrender past ~1e142× — FIXED in v0.2.6 (sub-f32 orbit dips).**
  Root cause: the 11–15 dive path's reference orbit passes within ~1e-71 of zero every 4383
  iterations; orbit samples are stored as plain df32, so those dips flushed to zero in the GPU
  buffer, dropping the `2Z·δz` recurrence term at exactly those iterations — past
  ~(dip ÷ per-period growth) zoom (~1e142×) that re-glued every pixel to the reference each
  period and frames rendered all-interior (the "uniform" color was the interior color). Fix:
  sub-1e-36 samples are stored extended-range as `[0.0, exponent, m_re+4, m_im]`
  (`pack_sample`/`sample_xy` in fractadyne-core) and decoded by the shader (`orbit_fe`/
  `orbit_cdf`). The marker is FINITE and provably unambiguous (a legit df32 pair with
  hi == 0.0 always has lo == 0.0, while lane 2 here is ≥ 2) — a first attempt used NaN
  and silently failed on the GPU: WGSL gives no NaN guarantees and compilers may fold
  `x != x` to false; the mode-2 rebase and Pauldelbrot-glitch comparisons also moved to scalar
  floatexp (`fe_abs_sf`/`sf_lt`) — both underflowed to `0 < 0` below dz.e ≈ −75 and were
  silently disabled. Diagnosed with two kept probe tests (`probe_orbit.rs`, `probe_escape.rs`:
  bignum dip profiles + CPU-perturbation escape times). Measured escape bands: loc 14
  304k–582k (cap now 800k), loc 15 894k–1.46M (cap now 1.6M) — the .exr's 6M was exploratory.

- [x] **Deep export throughput vs Fraktaler-3 — MEASURED 2026-07-11 (v0.2.10); the "~50× slower"
  claim is REFUTED: we are on par with F3.** A controlled render of corpus location 14 (= F3's
  me148) at F3's **2560×1440**, SA off / glitch off (corpus staging), on the 3080: **~15 s total**
  (render fn 14.8 s; 18 s incl. app boot) vs **F3's ~14 s** — roughly equal, not 50× (and not even
  2×). The `[fd-perf]` split: **GPU iterate 2.71 s** (nominal ~1.09e12 steps/s), GPU color 1 ms,
  and the remaining **~12 s is one-time CPU bignum setup** (reference orbit + BLA build, before the
  first tile — progress sits at 0% for ~8 s of that). Consistent with the `--profile`
  deep-interior-1e148 breakdown (~81% CPU setup). Counters confirmed the deep paths fire live:
  ext=281.9M (extended-range dip samples — the v0.2.6 fix executing), bla_skip=3.48e9 (would have
  wrapped u32 — validated the v0.2.10 per-tile-u64 fix), rebase=19.4M, maxiter=18,481. **Conclusion:
  the GPU iterate is fast and competitive; there is no 50× gap.** The old ~1e9-steps/s figure was
  stale (predated v0.2.x reference-reuse/BLA/extended-range, or was measured with glitch correction
  on = the separate >1 h pathology). **The cold CPU bignum setup (~12 s) breaks down (v0.2.11
  `[fd-ref]` timing split, me148 cold): `best_reference` candidate scoring ~9.3 s (79%!) + orbit
  build ~1.2 s + BLA build ~0.8 s.** So the ONE meaningful throughput lever is **`best_reference`
  scoring** (fractadyne-core), NOT the shader/orbit/BLA — the `--profile` path hides it because its
  timed reps reuse the reference and skip the scoring. Candidate ideas (all delicate — the scorer
  was tuned to avoid poor-reference glitches at depth, so any change needs a reuse-vs-quality golden
  check): score candidates to a much shallower iteration cap (a long-lived orbit is identifiable
  early; no need to run 800 k in bignum to rank), score fewer candidates, or reuse the boot-frame's
  reference for the export instead of re-picking cold. Not GPU-bound; mode-2 Fe per-iter cost is not
  the bottleneck.

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
- [~] **Tile cache + pan reprojection** — **pan reprojection DONE**: while dragging, the last
      settled iteration texture is frozen and translated in the color pass by the accumulated
      pixel offset (no bignum recompute, no re-iterate), so detail slides under the cursor; the
      revealed edge fills with the frame's average color; on settle it re-renders at full detail.
      Single + dual (left) at deep zoom. *(Scale reprojection now also exists — see the
      XaoS-style item below — but only as a deep-zoom stall fallback, not the primary zoom path.)*
- [~] **XaoS-style continuous-zoom pixel reuse (reuse-first zoom)** — the headline UX gap vs.
      XaoS. **PLAN + STAGE 0 landed 2026-07-11 (v0.2.12):** a full read of the live pipeline is
      written up as a concrete staged roadmap in **design/xaos-reuse.md** (Stage 0 verification →
      Stage 1 coordinate-keyed tile store → Stage 2 during-motion tile refine [the high-value,
      high-risk core] → Stage 3 shallow exact reuse), with the freeze/hang fragility constraints any
      new reuse path must respect. **Stage 0 shipped: `--reusetest`** (reusetest.rs) — a headless
      staleness harness measuring the color-pass reprojection vs a from-scratch render across
      Δ-octave zoom-ins. Data (RTX 3080): in fine detail the NEAREST reprojection loses real
      per-pixel iter fidelity fast (seahorse-1e6 ~61% of escaped px differ >2 iter by Δ=0.1 oct,
      rising slowly to ~68% by 1.0), a conservative raw-iter proxy (colored/perceptual staleness is
      lower). Quantifies WHY the reproject is only a placeholder and motivates the Stage-2 refine.
      **Stage 0 findings (v0.2.13–0.2.14):** a perceptual sRGB metric shows staleness is far below
      the raw-iter proxy (sRGBmean ~12–33/255) so REFRESH_OCTAVES=0.5 is validated; and a
      nearest-vs-bilinear comparison found **bilinear reprojection is WORSE by 4–16%** (it smears
      across escape-time bands) — nearest is correctly chosen, the filter is not the lever, and only
      the Stage-2 real-detail refine can reduce staleness. **Next: Stage 2 is its own dedicated task**
      (re-iterates during motion on a fragile loop — gate it with the `--reusetest` colored golden;
      needs live visual verification, so best done in a focused session). *Earlier shipped (v0.1.53–57,
      0.1.66): reuse-first refresh for mode-0 zoom, deep-dive reference **reuse** (extend the cached
      orbit, ~20× faster rebuilds), frozen-frame reprojection/hold, and adaptive motion resolution
      (AIMD) — deep zoom is smooth in motion; the full coordinate-keyed tile/mip reuse in (2) below
      remains open.* Today every zoom frame
      still re-renders from scratch on settle (GPU iterate → color), so a deep dive
      visibly pixelates/blanks until the frame settles; XaoS instead *remaps already-computed
      pixels* from the previous frame each step and only computes what's newly needed, so zooming
      feels continuous. **Foundation already present:** the color shader does an affine
      scale+translate of the frozen iteration texture (`uv_scale`/`uv_off` — `mandelbrot.wgsl`
      ~L1148), and `render.rs` computes `reproject_scale = 2^(l2_frozen − l2_now)` +
      `frozen_center`/`frozen_l2` (~L881–909). But it fires **only** as a stall fallback when the
      deep reference goes `too_stale` — not on shallow/normal zoom, and it just holds a *scaled*
      (upsampled, blurry) copy until a fresh reference snaps in. To make it XaoS-like:
      1. **Promote reuse to the primary zoom path** (all depths, every zoom frame): start each
         frame from the reprojected prior texture instead of black, so there's never a
         blank/pixellated intermediate.
      2. **Refine, don't just upscale** — a scaled frozen texture is upsampling, not real detail.
         Re-iterate only the newly-revealed annulus at the edges + progressively re-iterate the
         interior at correct resolution (center-out or priority tiles) so reused regions stay
         sharp while new detail streams in. Needs the long-planned **coordinate-keyed tile/mip
         cache** (the "persistent tile cache" noted above) so tiles survive across frames.
      3. **Shallow regime (mode 1, <1e4×) can reuse *exactly*** — direct per-pixel dwell means the
         iteration counts can be remapped by coordinate (true XaoS reuse: recompute only the
         rows/columns that moved past tolerance), cheap and lossless; the deep regime is the
         tile-refine path in (2).
      Effort: **large.** The live pipeline is fragile (per the freeze/hang history), so this needs
      a careful progressive-refinement scheduler + a reuse-vs-full-render golden check (a reused
      frame must converge to the same image as a from-scratch render). Biggest single win for
      perceived smoothness; orthogonal to the perturbation math already in place.
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
      now that deep zoom reaches extreme depths (re-zooming to 1e30× by hand is painful).
- [ ] **Left Parameters panel** (type, power, location, zoom).

### Planned settings (Preferences UI — mockup 10)

- [x] **Continuous-zoom rate** — "Zoom speed" slider (0.25×–4×, log scale) in the
      side panel's NAVIGATION section; multiplies `ZOOM_RATE`, persisted with session
      state (`SessionState::zoom_rate`, `serde(default)` so old files still load).
      *(Requested 2026-06-26.)*

## Milestone M2 — Deep zoom (in progress)

Goal (DESIGN.md §5): extreme-depth zoom via arbitrary-precision reference orbit + GPU
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
- [x] **Arbitrary-precision center + reference (extreme-depth zoom)** — center is now
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
- [x] Re-add `zoom_to_rect` unit test (dropped in the dd rewrite) — two tests in
      `fractadyne-core` cover centered uniform 4× scaling, the max()-fit invariant for
      off-center/non-aspect boxes, and drag-direction independence.
- [x] **UI digit separators** — commas on zoom/iter, spaces grouping coordinate digits.
- [x] **Floatexp perturbation δ (extreme depth)** — the df32 δ has f32's *exponent*
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
      1e308×), which was the real live-zoom wall (the bignum center already had no fixed precision cap).
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
- [~] **Full glitch correction** (Pauldelbrot criterion + multi-reference recompute). Multi-ref
      correction is implemented + validated, **on by default**, and covers **single + dual exports**
      (side-by-side + separate; ≤ ~32 MP / texture limit, non-aux; VRAM-capped; `--glitch`/
      `--no-glitch`). *Remaining:* live-view correction (settle-time / async — the live pipeline is
      fragile, so deferred).
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
- [~] **Newton / Phoenix deep zoom** — **Phoenix DONE (v0.1.2):** perturbation deep zoom in df32
      (mode 0) + floatexp (mode 2), with the two-term `δz_{n-1}` register and previous-term rebasing
      (rebase-to-0 valid since the reference's `z_{-1}=0`); bignum reference + `orbit_length_bf` made
      Phoenix-aware. Validated in `--selftest` (mode 0 vs direct mean Δ 0.007 iter @1e5×; mode 2 vs
      mode 0 exact) + a core unit test. **Newton stays direct-only** — convergence-based with a
      nonlinear, coloring-incompatible perturbation (revisit only via a separate higher-precision
      *direct* path if there's demand).
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
- [x] **Dual-view polish** — draggable splitter (persisted `dual_split` fraction; drag the
      separator between panels, clamped 15–85%).
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
- [x] **Guided-tour scripting (narrated, annotated tours)** — DONE (all five sub-items below):
      grew the keyframe format into an authored, self-documenting tour — captions, coordinate-
      anchored callouts, spotlight vignettes, per-segment easing + holds, and schema version
      tracking — rendered live and burned into `--render-tour` movie frames. Optional extras noted
      per sub-item (pause-until-dismissed captions, off-screen callout arrows, rect spotlights):
      - [x] **On-screen commentary / text** — DONE: `[[caption]]` entries (timed independently of
        keyframes) with `text` (multi-line), `at`/`secs`, `pos` (top/center/bottom), `fade`, and
        `size`. Eased fade in/out; wrapped + centred on a soft dark backing. Renders live
        (`draw_captions`, egui painter) **and** burned into exported tour frames (`stamp_caption`,
        rasterized from the font atlas). *(Remaining: optional pause-until-dismissed.)*
      - [x] **Callouts** — DONE: `[[callout]]` entries with a target `center_x`/`center_y` (fractal
        coordinate), `text`, `at`/`secs`, `fade`, `size`. Drawn as an amber marker ring + leader
        line + label, **anchored in fractal space** (new `Viewport::complex_to_pixel`, exact at any
        depth) so they track the point as the view pans/zooms; off-screen anchors are skipped. Live
        (`draw_callouts`) + exported frames (`stamp_callout`). *(Remaining: off-screen edge arrows.)*
      - [x] **Vignettes / spotlights** — DONE: `[[spotlight]]` entries dim everything outside a soft
        circle centred on a fractal coordinate (`center_x`/`center_y`), with `radius`/`softness`
        (frame-height fractions), `dim`, and `at`/`secs`/`fade`. Applied in the color shader
        (aspect-corrected round circle) so live + export are identical; anchored via
        `complex_to_pixel` so it tracks the point; the dim eases with the fade window.
        *(Remaining: rectangular regions.)*
      - [x] **Eased transitions** — DONE: per-keyframe `ease` (`smooth` default, `linear`,
        `smoother`, `in`, `out`) for the glide arriving at it, plus `hold` seconds to pause at a
        keyframe before the next glide. `Playback::sample` now splits each segment into a hold phase
        + an eased move phase (log2-mag + BigFloat-lerp as before). Verified: hold-window frames are
        identical, the hold extends the timeline.
      - [x] **Targeted version tracking** — DONE: scripts declare `format_version`; loading (live +
        `--render-tour`) warns when it exceeds this build's `SCRIPT_FORMAT_VERSION` (like the `.fdn`
        / export `NewerFormat` path). Schema is additive (unknown keys ignored, missing default), so
        old scripts still play.
      Pairs with the existing live playback + `--render-tour` movie export (annotations should
      render in exported frames too). *(Design the schema additively — new keys, old scripts still
      play — and reuse the hardened `meta_get`/version-check machinery for untrusted script files.)*
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
      a `profile` module. **GPU timestamp queries DONE (v0.1.34):** `--profile` now reports pure-GPU
      `gpu-it` / `gpu-col` per-pass time (wgpu `TIMESTAMP_QUERY` + a `fractadyne_gpu::timing`
      thread-local capture bracketing `render_export`'s iterate/color passes), independent of the
      CPU submit/poll/readback overhead. *(Follow-ups: opt-in per-frame logging of live interactive
      sessions incl. a live perf-overlay GPU-timestamp row; fold the series coefficients into the
      reference-orbit pass to cut the ~100 ms series-skip setup at depth.)*
- [x] **Record-to-video / frame export** from a script (offline, deterministic) — done via
      `--render-tour` (see the Zoom-movie entry below).
- [x] **Ship compiled binaries on GitHub (Releases)** — `.github/workflows/release.yml`
      builds the Windows `x86_64` binary on `windows-latest` and, on a `v*` tag push,
      packages `fractadyne.exe` + README + both licenses into a versioned zip with a SHA-256
      sidecar and publishes a GitHub Release (auto-generated notes) via the `gh` CLI. A
      manual `workflow_dispatch` run instead uploads the zip as a downloadable artifact (no
      publish) for testing. Uses the standard `--release` profile (the local `-j1`/no-LTO
      constraints are this machine's page-file workaround; runners don't need them). Verified
      locally: the build command, output path, and the packaging/zip/checksum steps.
      README gained a **Download** section. Possible later: Linux/macOS jobs (need GTK/X11
      runner deps for `rfd`), code signing, and a more-optimized `dist` profile (LTO).
- [x] **Continuous integration** — `.github/workflows/ci.yml` gates every push to `main` and
      every PR: a **core-tests** job (`cargo test -p fractadyne-core --release` on Linux — the
      exact-math suite is pure Rust, no GPU/GUI/system deps) plus a **build** job
      (`cargo build --workspace` on Windows) confirming the GPU/egui crates still compile on
      the target. `concurrency` cancels superseded runs. The GPU `--selftest` needs a real
      GPU (runners have none → flaky), so it stays a local/manual gate. Verified both commands
      locally (29 core tests pass; workspace compiles). Possible later: a software-adapter
      `--selftest` job, `clippy`/`fmt` checks.
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
- [ ] **In-app editor for the authorable files** — a text/TOML editor (probably a Tools →
      "Edit file…" panel) for the file types the app reads: tour scripts (`.toml`),
      profiling region files (`.toml`), `.fdn` locations, and response/`@args` files.
      Ultimately it should offer **schema validation** (flag unknown keys, out-of-range
      values, malformed sections before the file is used), **autocomplete** (key names,
      enum values, section templates), and **pasteable sample snippets** (a palette of
      ready-made keyframes / sections / whole example files to insert). The existing
      `TOUR_SCHEMA` in scripting.rs (which already generates TOURS.md) is the natural
      source of truth for the tour-script validation/autocomplete/samples; the untrusted
      parsers elsewhere give the range clamps to surface as validation errors. Could start
      as a validate-on-load "problems" list and grow toward live editing.

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
      the continuous-zoom rate). Stops on a dead end (no boundary detail → flat interior/exterior)
      or the user-set **dive limit** (Navigation-panel slider, persisted, 1e30×–1e5000×). Up to
      ~1e271× (the smooth regime) it glides; past that it switches to a **stepped dive** (jump ×4 →
      render → hold the last full frame while the next computes) so it reaches extreme depth without
      staring at a blank; re-evaluation is adaptive (spaces out as frames slow). Started/stopped by
      **A**, the **🛸 toolbar button** (highlighted while running), or the View menu; **Esc** stops
      it. *(Follow-ups: minibrot-seeking / boundary-tracking steering modes.)*
- [x] **Zoom-movie / frame→video export** — `--render-tour FILE [--fps N] [--size W]
      [--height H] [--ss N] [--out DIR]` renders a keyframe-tour TOML to a numbered PNG
      frame sequence (`frame_00000.png …`) for assembly into a video (prints an ffmpeg
      one-liner). Reuses the scripting keyframe interpolation — factored into
      `Playback::sample(t)` shared with live playback — and the offscreen export path; steps
      the timeline at fixed `fps`, recomputing a fresh deep reference per frame. Deep-correct
      (`set_center_log2mag`, octave-based precision) so dives past 1e308× sample exactly; this
      also upgraded **live** playback to the log2 path (was `set_center_mag`, which saturated
      at 1e308×). Example: `scripts/tour.example.toml`. Verified: a 9-frame 1→1e3× test dive
      renders correctly. Now prints live progress (frames done / elapsed / ETA / fps) and, with
      `--mp4 [PATH]`, assembles the frames into an H.264 mp4 via ffmpeg (frames kept; falls back to
      printing the assemble command if ffmpeg is absent). *Follow-up: in-app "Render tour…" UI.*
- [ ] **Layers + blend modes** (Ultra Fractal-style compositing).
- [ ] **Formula DSL / custom formulas** (M6).
- [~] **Series approximation** — order-3 polynomial (`δz ≈ A·δc + B·δc² + C·δc³`) seeds the
      perturbation and skips the early iterations. **Done for the holomorphic polynomial
      families — Mandelbrot + Multibrot 3/4/5 — on both perturbation paths: mode 2 (floatexp,
      ≥1e28×) and mode 0 (df32, 1e4–1e28×, the common range)**, non-Julia, non-aux coloring.
      Coefficients iterated in bignum alongside the reference (mode-independent), generalized
      to `z^d+c`: `A'=d·Z^{d-1}·A+1`, `B'=d·Z^{d-1}·B+C(d,2)·Z^{d-2}·A²`,
      `C'=d·Z^{d-1}·C+2C(d,2)·Z^{d-2}·AB+C(d,3)·Z^{d-3}·A³`. Skip chosen from the worst-case
      corner `|δc|` (cubic ≤ 2⁻¹⁶ of linear ⇒ no premature escape), cached per reference. The
      mode-0 seed is evaluated in floatexp (coeffs overflow f32) then collapsed to absolute
      df32 via `fe_to_cdf`; the GPU seed is formula-agnostic. Validated: core tests of the
      series vs exact perturbation for d=2 and d=3 (rel err <1e-3); seed vs full iteration
      `maxΔ 0` at 1e30× (mode 2) and 1e20× (mode 0); Multibrot 3/4/5 SA engages + matches
      SA-off. (Tricorn/abs families have no such δc expansion — anti-holomorphic / non-analytic.)
- [x] **BLA (bilinear approximation)** — skips iterations *throughout* the orbit (SA only skips
      the start). **On by default; ~5× faster GPU render at 1e30×.** A binary tree of merged linear maps `δz' ≈ A·δz + B·δc` (A=2Z, B=1 per
      Mandelbrot step) with validity radii; a pixel skips 2^l steps when `|δz| ≤` the merged
      radius (Zhuoran's BLA; KF2+/Fraktaler-3). **Phase 1 DONE (core, fractadyne-core):**
      `CFloatExp` (complex extended-range), `BlaNode`, `bla_merge`, `build_bla_mandel`
      (level tree, odd-tail carry); merged radius `min(r₁,(r₂−|B₁|·δc_max)/|A₁|)`. Validated:
      `bla_reproduces_exact_perturbation` — a BLA traversal matches full-step perturbation
      (rel err <1e-3) while skipping >¾ of iterations on a main-cardioid reference.
      **Phase 2a DONE (core reference algorithm):** `bla_iterate` — the exact per-pixel render
      the shader will mirror: skip with the highest valid level, **revert to a lower level /
      full step on escape overshoot**, full step when `|δz|` exceeds even the level-0 radius.
      Validated by `bla_matches_naive_including_escapes` (BLA == naive perturbation on the
      escape iteration for both BLA-engaged tiny-δc pixels and large-δc fast escapers).
      **Phase 2b DONE (GPU port, off by default):** the tree is appended after the reference
      in the SAME storage buffer (no new binding) — 4 `vec4` per node (`[A],[B],[a_exp,b_exp,
      r_exp,r_mant],[span]`); the shader reconstructs per-level offsets from `orbit_len` and
      ports `bla_iterate` into the mode-2 loop (skip highest valid level → revert on escape
      overshoot → full step), updating the derivative `D=A·D+B` on skips. Core packers
      `CFloatExp::to_mantissa_exp`/`FloatExp::to_f32_exp`/`bla_to_gpu`; one uniform flag
      `bla_on`; app gate `self.use_bla` (mode 2, Mandelbrot, non-Julia, non-aux). **Phase 2c DONE
      (user-facing + escape-path validated):** `use_bla` is now a persisted **View-menu toggle**
      (`SessionState::use_bla`), and the GPU escape-overshoot revert is validated — a new selftest
      "BLA escape path == non-BLA @1e30× (boundary)" renders a deep boundary view (**48400
      escapers**, 0 mismatch) alongside the all-interior nucleus test (48400 interior, 0 mismatch),
      so both BLA code paths are covered. **Measured (2026-07-01, RTX 3080 / Ryzen 3950X, via the
      new `scripts/profile-bla.ps1` + `--bla` flag):** at 1e30× (mode 2, SA on) BLA cuts the GPU
      render **73.4 → 12.7 ms (5.8×)**; the tree build costs **~20 ms** (CPU, currently per frame).
      Net: **2.2× faster even uncached** (build-every-frame + render, 73.4 → 33.1 ms) and **5.8×
      with a per-reference cache** (build amortized). Zero cost where it doesn't apply (mode 0/1,
      aux, Julia — `build_bla` returns early). **Conclusion: enabling by default is justified.**
      **Phase 2d DONE (per-reference caching):** `build_bla` split into `bla_eligible` + a
      **conservative, offset-independent `bla_dc_max`** (2.5× the view diagonal — covers the whole
      region a reference stays valid over, up to the ~1.5-span "gone" recompute threshold, with
      margin) + the tree build; the live path (`build_params`) now caches the tree in
      `RefCache.{bla, bla_id}` and rebuilds only when `orbit_id` changes. Validated: selftest still
      0 mismatch (interior + boundary) with the conservative bound, and the profile shows it barely
      costs skips (render 12.7 → 13.6 ms, still **5.4×** vs off). Effect: a settled deep view drops
      from build-every-frame (~35 ms, 2.2×) to render-only (~13.6 ms, **5.4×**), and the ~20 ms tree
      build becomes a one-time per-reference cost (like the reference orbit) instead of per-frame —
      removing the weak-CPU risk. **Phase 2e DONE (on by default):** `SessionState::use_bla` now
      defaults **on**; a `--no-bla` flag forces it off for profiling (`--bla` still forces on). The
      cache was hardened against the zoom-out edge case — it rebuilds when the view needs a larger
      `dc_max` than the cached tree was built for (compared in log2 space to avoid underflow), with
      2× headroom so continuous zoom-out doesn't thrash. Verified: `--profile` (no flag) engages
      BLA (render 70→13 ms), `--no-bla` disables it, selftest 53/53. The View-menu toggle disables
      it if an artifact ever shows. **Phase 3:** mode-0 (df32, 1e4–1e28×) + Multibrot.
- [x] **Multi-reference glitch correction** (Pauldelbrot criterion + per-glitch recompute) —
      beyond the current single-reference Zhuoran rebasing. **Shipping for single-view exports**
      via a "Glitch correction (export)" preference (View menu, persisted): detects perturbation
      glitches and re-renders those pixels against extra references until clean. **Phase 2c DONE
      (color + wire):** `fractadyne_gpu::color_iter_buffer` colors the merged glitch-free iteration
      buffer (non-aux methods); `FractadyneApp::render_export_corrected` = correction → color →
      `ExportResult`, wired into both the headless (`render_to_file`) and interactive
      (`start_export_to`, run synchronously) export paths, gated by `glitch_correct`
      (`SessionState`). Selftest "corrected buffer colors to a valid image" (52/52, goldens 4/4).
      *(Follow-ups: tiling so it applies past the GPU max texture dim; aux coloring methods
      (stripe/TIA/trap/decomp) — need per-orbit stats merged too; apply to the live settled view;
      dual-view layouts.)* **Phase 1 DONE (core algorithm,
      fractadyne-core, Mandelbrot):** `Perturb` outcome, `reference_orbit_f64`,
      `perturb_pixel_mandel` (Zhuoran rebasing + Pauldelbrot detection, δz carried in **f32** to
      mirror the GPU's df64-reference/df32-δz precision gap — the gap that makes glitches real and
      fixable), and `render_multiref_mandel` (detect glitched pixels → place a new reference at the
      glitch region's centroid → re-render + merge → repeat to convergence). Validated: a real
      period-3 minibrot with an off-nucleus reference induces glitches, correction converges (≥2
      references, 0 unresolved), and the result matches a bignum per-pixel oracle exactly
      (`multi_reference_resolves_glitches`); plus a perturbation-vs-direct accuracy test. As with
      BLA, the core algorithm is validated first. **Phase 2a DONE (GPU detection):** shader gains
      a `glitch_on` uniform + Pauldelbrot check (`|z|² < GLITCH_TOL2·|Z|²`) in both perturbation
      loops (mode 0 df32 + mode 2 floatexp), flagging glitched pixels with a `-2` sentinel in the
      iteration texture's `r` channel (the color pass already treats `r<0` as interior, so it's
      harmless when uncorrected). `ExportRequest.glitch_on` plumbs it through `render_iter`/
      `render_export`; live rendering leaves it 0. Selftest "glitch detection responds to
      reference quality" confirms detection fires and a far-offset reference flags ≥ the auto
      reference (50/50, goldens 4/4). **Phase 2b DONE (correction orchestration):**
      `FractadyneApp::render_corrected_iter` renders the iteration buffer with `glitch_on`, then
      repeatedly drops a fresh reference (bignum, via `compute_reference`) at the glitched region's
      centroid, re-renders, and adopts the newly-resolved pixels — until nothing is glitched or
      `max_refs`. Seeding at the exact pixel *center* (the +0.5 texel offset) makes δc = 0 there,
      so each pass resolves at least its seed ⇒ guaranteed convergence. Selftest "multi-reference
      correction resolves glitches" (seahorse 1e8×): 9 flagged → **0 residual** with 7 references
      (51/51, goldens 4/4). **Phase 2c (next):** color the corrected buffer (GPU color-only pass
      over an uploaded iteration texture) + wire into the export path behind a "Glitch correction"
      preference + tiling for exports larger than the GPU max texture dim.

### Tier 3 — big bets (separate engines)
- [ ] **3D fractals** (Mandelbulb / Mandelbox, ray-marched).
- [ ] **Flame / IFS fractals; L-systems; cellular automata.**

## Performance & throughput (M7)

- [ ] **Deep floatexp *settled* frames are slow in filament fields — a shader-speed fix, NOT multi-reference.**
  *Update (v0.1.57–0.1.68): interactive MOTION is now smooth — reference-orbit reuse (~20× faster
  rebuilds), frozen-frame reprojection/hold, and adaptive motion resolution (AIMD) replaced the old
  "blank during deep dives." What remains is a full-detail SETTLED frame in filament/Misiurewicz
  fields.* Full profiling in the archived `multiref-live` design note (git history). Deep mode-2
  frames cost seconds; this forced the v0.1.10 "reproject during mode-2 motion" hang fix (responsive
  but **blank** during deep dives). **Multi-reference was validated and abandoned (2026-07-03):** a
  `--refdiag` prototype showed the deep-spiral/Misiurewicz views have **zero long/interior references**
  (every point escapes at ~2400–6490 iters; at 1e75× all collapse to 6490), so there's nothing to
  rebase onto — multi-ref can't help. A finer sweep showed the cost is flat iter 4000→10000, so it's
  **BLA failing to skip in the filament structure**, not rebasing. Confirmed non-fixes (don't retry):
  resolution/WORK_BUDGET reduction, BLA rebuild on zoom-in, longer/predictive reference selection
  (`REF_SCORE_SCAN` 4096→65536 was *slower*), multi-reference. **Real levers:** (1) cheaper floatexp
  ops in `mandelbrot.wgsl` — proportional speedup to every deep frame, best next candidate; (2) GPU
  occupancy (register pressure); (3) iteration cap during motion (trades detail); (4) accept it —
  export (`--render-tour`) already renders full detail per frame. `--refdiag` CLI added as a dev tool.
  - **Confirmed (2026-07-06): op-level shader micro-opts are noise vs the floatexp iteration.** A perf
    recon proposed gating the per-iteration aux stats on the selected color method and a 3-mul `c_sqr`.
    Both landed bit-exact (pixel-verified; goldens 17/17) but measured **perf-neutral** on the RTX 3080:
    removing *all* of decomposition's per-iteration aux work (atan2+sin+pow) moved 1e30× iterate
    298.2→297.0 ms, and `c_sqr` is CSE'd by the compiler. So the aux-at-depth cost is **not** the aux
    transcendentals — it's that stripe/TIA/decomp disable BLA/SA → 25k full floatexp steps (=the 297 ms;
    trap keeps BLA → 11 ms).
  - **Reranked by GPU timestamps (v0.1.34, RTX 3080, 1e30× mode-2 seahorse).** The new per-pass
    `gpu-it`/`gpu-col` split shows pure-GPU iterate is **1.4 ms WITH BLA vs ~316 ms without** — BLA is
    a **~220× lever**, and aux is slow at depth *only* because it disables BLA. So **(b) aux⇄BLA
    coexistence is #1** (a cheaper aux accumulation that lets SA/BLA skip, or per-orbit stats that
    survive skips); **(a) cheaper floatexp arithmetic is #2** — it only matters for the filament views
    where BLA genuinely can't skip. The color/downsample pass is **~0.01 ms** at 512² (negligible), and
    the CPU-timed `iter/render ms` columns carry ~9 ms of fixed submit+poll+readback overhead (smooth's
    true GPU iterate is 1.4 ms, not the 10.7 ms the CPU clock showed) — which is why op-counting on the
    CPU columns mis-ranked the earlier pass. Measure GPU levers with `gpu-it`, not the CPU columns.

- [x] **FIXED (v0.1.10): fast live dive hung ("Not Responding") crossing into floatexp (~1e28×+).**
  Reproduced 2026-07-03 by auto-playing `tours/deep-spiral-dive.toml` with per-frame stderr timing.
  **Root cause:** the mode-2 (floatexp) iterate shader spins **~5 s/frame** when its reference/BLA are
  even ~0.5–2 octaves depth-stale (a fresh reference renders the same view in ~18 ms; stale, the
  perturbation rebases/does full steps per pixel). Since GPU pixels run in parallel, the frame time is
  the *slowest pixel's* shader duration, so it's **independent of resolution** (proven: 24×18 iterate
  texture still spun) — the ~1 s frame present blocks the UI thread, and because `update()` can't run
  during the block the off-thread reference recompute can't install → feedback loop pinning the dive at
  ~1 fps. On a *centered* dive the existing positional `too_stale`/reproject freeze never fired
  (`drift ≈ 0`), so it always painted the stale reference. **Fix (`render.rs`):** in mode 2, (a) freeze
  = reproject (which skips the iterate pass) for **all interacting frames** — on a dive faster than the
  recompute latency every reference is stale on arrival and the spin onset is data-dependent, so no
  threshold safely lets a real frame through; and (b) also freeze while a `bla_dc_max`-based
  `depth_lag > 1.2` so a *settle* holds until the freshly-recomputed reference lands, then snaps to
  full detail. Result: max mode-2 frame **5167 ms → 32 ms**, tour dives smoothly to 1e193× live;
  selftest 55/55, goldens 4/4. Tradeoff: live mode-2 *motion* is soft (reprojected) and sharpens on
  pause — the offline `--render-tour` export path is unaffected (fresh sync reference per frame = full
  detail). **Dead ends (don't retry):** shrinking the mode-2 `WORK_BUDGET` (even /4000) does nothing
  (cost is per-pixel spin, not total work); a `depth_lag` threshold that still allows real motion
  frames is fragile (spin onset overlaps the "fresh" range). *Possible follow-up:* bound the mode-2
  shader's worst-pixel step count so real motion frames become safe (would restore live detail while
  diving), verified against goldens.

**Update (2026-07-02): the bottleneck moved.** The off-thread reference recompute (below) took the
bignum orbit off the render thread — `--benchmark` now shows **avg CPU 0.38 ms, avg GPU 20.3 ms**,
so the live cost is now the **GPU iterate pass**, not the reference. `--profile` breakdown (render ms
= GPU): home/1e4 ≈ 10 ms · 1e6 17 · 1e12 16 · 1e20 19 · 1e30 (mode 2, BLA) 12 · **1e30 stripe
(aux, no BLA) 214**. Findings:
- **BLA is the GPU lever, but only past ~1e28×.** Measured: forcing the floatexp+BLA path down to
  1e12/1e20 (lowering `PERT_FE_THRESHOLD`) made it **2.5–4× slower** (iter 40/73 ms vs df32 15/18 ms)
  — floatexp's per-op cost dwarfs the BLA skip until the skip becomes huge (~1e28×). The 1e28
  crossover is well-chosen; don't lower it. Getting BLA into the df32 range would need BLA applied in
  the df32 loop (coefficients overflow f32 → need an fe hybrid) — uncertain payoff, high risk.
- **Aux coloring (stripe / distance / orbit-trap) can't use BLA** (it needs every iteration's z), so
  it's ~10–17× slower at depth (214 ms at 1e30). Inherent; the fix is a cheaper aux accumulation, not
  BLA.
- The original premise below (reference is the bottleneck) is now **historical** — kept for context.

Prioritized after a multi-GPU assessment (2026-07-01). The deep-zoom bottleneck during motion is
the **serial arbitrary-precision reference orbit** (bignum CPU, ~45–77 ms recompute) — *not* GPU
work — so it can't be parallelized across GPUs (or even threads: a single orbit `z_{n+1}=z_n²+c`
is a sequential dependency chain). The live GPU render is already frame-capped (~100 FPS via
`WORK_BUDGET`). So these attack the real bottlenecks first; **multi-GPU is deferred** (see below).

- [~] **Off-thread reference recompute** — **DONE for the live view:** the deep-zoom recompute
  (reference orbit + series approximation + BLA tree — all bignum) now runs on a worker thread
  (`recompute_worker`); `build_params` keeps drawing with the cached reference and installs the
  result when it lands (only the very first, cold-start reference is synchronous). Validated with
  the new `--frametest` harness (dive → 1e30×): **recompute stalls 27 → 1**, build-time p95
  **91.8 → 0.1 ms**, max **196 → 30 ms** (the lone remaining stall is the cold start). Selftest
  53/53, goldens 4/4 (sync export path unchanged). **Remaining:** compute the **multiple glitch-
  correction references concurrently** (rayon) and **speculative** next-frame references; thread
  the glitch-corrected export so it doesn't block the UI.
- [~] **Faster / adaptive bignum reference** — the reference orbit is the deep-zoom wall.
  **Done so far:** the per-iteration `2xy` was formed with a *full bignum multiply by 2*; replaced
  with `double_bf` (exact base-2 exponent bump) in `step_bf` + `iter_zsq_c`. Measured via
  `--profile`: reference-orbit compute **−13–17% at 1e6–1e20×** (deep-1e20 14.5→12.6 ms), −6% at
  1e30×; goldens bit-identical (exact change). **Remaining:** audit the +64 guard bits (trim where
  safe), find a dedicated bignum square (x²/y² are still general muls), profile `astro_float` hot
  paths, and evaluate a GPU-bignum / fixed-point reference pass to move it off the single CPU core.
- [~] **Pipeline the export** — overlap render → encode so the CPU/GPU never idle waiting on I/O.
  - [x] **Tour frame encode** — `--render-tour` now hands finished frames to a background PNG
    encoder pool (bounded ~1 GB in-flight for backpressure), so deflate overlaps the next frame's
    render. Byte-identical output; win scales with resolution.
  - [ ] **Tile-level export pipeline** — overlap tile N+1 iterate with tile N async readback +
    encode inside `render_export` (still `poll(Wait)`-serial per tile). Smooths the synchronous
    glitch-corrected export too.
  - [x] **Reference precompute overlap (tours)** — DONE: the export path now routes through
    `recompute_worker` (unified with the live path), and `--render-tour` computes frame N+1's
    reference on a worker while frame N renders on the GPU, feeding it to
    `current_export_request_with_ref`. Gated to single-view successors with matching fractal/Julia
    state (falls back to synchronous otherwise — always correct). Verified byte-identical output
    (0/37 frames differ); measured ~1.2× on a 1e12 mode-0 dive at 1000px, more on deep mode-2
    large frames where the bignum reference is a larger share of the per-frame cost.
- [x] **BLA on by default** — shipped (`SessionState.use_bla` defaults true). Confirmed by
  `--profile` as the key GPU lever at ≥1e28× (1e30 iter 10 ms with BLA vs 174 ms without). Note it
  only helps past the floatexp crossover (see the update note above) and not for aux coloring.
- [ ] **Better single-GPU utilization** — before adding GPUs, check the live dispatch actually
  saturates the one GPU (occupancy, workgroup sizing, async compute for the iterate vs. color
  passes). Often a cheaper 1.5–2× than a second device.
- [ ] **Multi-GPU — offline/export only (deferred)** — a second GPU gives near-linear speedup for
  **embarrassingly-parallel batch work** (high-res export tiles, movie/tour frame sequences), and
  the export path already does CPU readback so there's no shared-texture problem. It does **not**
  help the serial reference orbit or the frame-capped live view, and interactive multi-GPU is very
  invasive (the `egui_wgpu` paint callback is single-device; wgpu has no cross-device sharing).
  Low priority: most users are single-GPU, and the items above are higher-ROI. Revisit only if
  batch-render throughput becomes the pain point — and only for the offline path. Measure the
  GPU-vs-reference split with `--profile` / `--benchmark` before investing.

## Backlog (later milestones — DESIGN.md §15)

- **M4** more fractal variety: L-systems, cellular automata
- **M5** high-res tiled export (PNG / OpenEXR)
- **M6** programmable engine (formula DSL → WGSL + CPU; custom coloring)
- **M7** polish & perf

## Stub crates (created, awaiting their milestone)

`fractadyne-color` (M1) · `fractadyne-render` (M1/M2) · `fractadyne-state` (M1) ·
`fractadyne-fractals` (M4) · `fractadyne-export` (M5) · `fractadyne-ui` (panels, M1+).
