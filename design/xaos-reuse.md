# XaoS-style continuous-zoom pixel reuse — implementation plan

Concrete, code-grounded roadmap for the remaining part of TODO.md's "XaoS-style continuous-zoom
pixel reuse (reuse-first zoom)" item. Written after a full read of the live pipeline (2026-07-11).
The point of this doc: the live render loop has a **documented freeze/hang history**, so the work
must be staged and each stage verified against a from-scratch golden before the next — not hacked
in. Anchors are `file:line` at the time of writing; re-grep before editing.

## What already exists (do NOT rebuild)

The system is already reuse-first — a warm view never blanks during motion:

- **Persistent iteration texture.** `ViewResources.tex_view` (fractadyne-gpu/src/lib.rs:245) is
  per-view, allocated once (lib.rs:547), survives across frames. Reuse = *skip the iterate dispatch
  and re-color the existing texture*.
- **Whole-texture affine reprojection** (color pass). `fs_color` samples the frozen texture through
  aspect-fit → scale-about-centre → translate (mandelbrot.wgsl:1308-1322), off-frame → `view_average()`
  (not black). Driven by `reproject`/`uv_scale`/`uv_off` in `ColorU` (mandelbrot.wgsl:1197-1199).
- **`reuse_hold`** (render.rs:1319): deep interactive zoom holds+magnifies the frozen frame until it
  has drifted `REFRESH_OCTAVES = 0.5` octaves (render.rs:1317), then takes ONE real re-iterate at
  reduced motion resolution and resets `frozen_l2` (render.rs:1923-1925). `will_reproject`
  (render.rs:1328) is the master "no iterate this frame" flag.
- **Tiled settle** (v0.2.4): once settled (`fe_budget_ok`, render.rs:1448), a center-out
  `next_settle_tile` (render.rs:1110) re-iterates one scissored tile per frame with `LoadOp::Load`
  (lib.rs:1004) onto the persistent texture, sharpening to native. `seeded_resize` (lib.rs:594)
  upscales-in-place so a growing grid never shows black.

**The gap:** today's motion reuse is a *whole-texture affine resample* — magnifying a fixed-resolution
texture, so a dive gets progressively blockier until it stops and the tiled settle runs. The only
pixel-preserving *re-iterate* is the axis-aligned tile scissor, and it runs only when settled. XaoS
instead keeps already-computed pixels at sub-pixel accuracy and streams new detail into the
newly-revealed regions *during* the dive.

## Fragility constraints any new reuse path MUST respect

From the freeze/hang history (comments at render.rs:1413-1420, 56-59, 1159-1174; lib.rs:942-945):

1. **`IterKey` / `settings_hash` lockstep.** The GPU re-iterate trigger is
   `view.last_iter_key != key || view.last_tile != tile` (lib.rs:945). Any reuse that re-iterates a
   sub-region must extend these keys so a stale tile can't splice two views into one image.
2. **One budget-sized dispatch per view per frame.** `next_settle_tile`'s turn token (render.rs:1159)
   prevents two TDR-budget dispatches pairing up past the ~2 s watchdog. New re-iterates must honor
   `tdr_steps` / `tdr_allowed` (render.rs:1468) and the token.
3. **`view_gen` coherence.** `view_gen` bumps on every interacting/reproject frame (render.rs:1401),
   which is what invalidates a settle grid when the view moves. A reuse scheme that re-iterates
   *during* motion must define how it keys the texture while `view_gen` churns every frame — the
   single hardest design question here.
4. **Never a native-res full floatexp iterate during motion.** That was hang cause (a); motion frames
   are budgeted `wb/6` + `res_scale` (render.rs:1250, 1287). New motion re-iterates stay within that.
5. **No placeholder iterate without a reference** (`PLACEHOLDER_ITER_CAP`, render.rs:2011).

## Verification strategy (build FIRST — Stage 0)

The item explicitly needs "a reuse-vs-full-render golden check (a reused frame must converge to the
same image as a from-scratch render)." Without it, no reuse change can be trusted. Two headless
harnesses, both CLI-driven (no live-loop change, deterministic):

- **`--reusetest` staleness curve.** For each of a few deep views: render from-scratch (`render_iter`)
  → reference R; for Δ ∈ {0.1, 0.25, 0.5, 0.75, 1.0} octaves deeper, CPU-resample R by the exact
  color-pass affine (2^Δ magnify about centre) and compare to a from-scratch render of the Δ-zoomed
  view. Report max/mean smooth-iter error vs Δ. **Immediate payoff:** it turns `REFRESH_OCTAVES = 0.5`
  from a guess into a measured choice (refresh when reprojection error crosses a threshold), a safe
  data-driven tuning that needs no new reuse machinery.
- **Reuse-convergence golden.** Once Stage 2 exists: assert a settled reused frame equals a
  from-scratch full-res render within the golden tolerance (reuse the selftest golden diff:
  `img_diff`, selftest.rs). A reused frame that doesn't converge is a bug, caught headlessly.

## Staged implementation

Each stage ships behind a flag (default off) until its golden passes, then flips on.

- **Stage 0 — verification harness** (above). Small, safe, no pipeline change. Also yields the
  data-driven `REFRESH_OCTAVES` tuning as a standalone win.
- **Stage 1 — coordinate-keyed tile store.** Give the tiled-settle tiles an identity: key each
  composited tile by (complex-rect, depth-octave, ss, settings_hash) and keep them in a bounded
  per-view LRU keyed off `orbit_id`. No behavior change yet (tiles are still produced by the settle);
  this just makes them *addressable* for reuse. Verify: byte-identical output to today (goldens).
- **Stage 2 — reuse valid tiles on zoom + refine the rest.** On a zoom step, a tile whose complex-rect
  is still on screen at a compatible octave is reprojected from the store (sharp, not upscaled); the
  newly-revealed annulus + the interior tiles that crossed an octave boundary are re-iterated
  progressively (center-out, one budget tile/frame, honoring constraint 2). This is the XaoS core and
  the riskiest stage — it re-iterates during slow motion, so it must resolve constraint 3 (likely:
  key tiles by complex-rect+octave, independent of `view_gen`, and only reuse when the frozen
  reference is still valid at the tile's depth). Verify: the Stage-0 golden must converge.
- **Stage 3 — shallow (mode-1 direct) exact reuse.** The direct path re-iterates the whole frame every
  frame (render.rs:1644 gates reproject to `!is_direct`). At <1e4× the per-pixel dwell at a complex
  coordinate is fixed, so pan/zoom-out reuse is *lossless* and zoom-in needs only the new in-between
  pixels. Lower value (the direct path is already ~10 ms) but the cleanest exact-reuse golden.

## Risk / recommendation

Stage 0 is safe and independently useful (do it regardless). Stage 1 is low-risk (addressing only).
**Stage 2 is the high-value, high-risk core** — it re-iterates during motion on a loop with a
freeze/hang history, so it should be a dedicated effort gated by the Stage-0 golden, not folded into
other work. Recommend: land Stage 0 (+ the REFRESH_OCTAVES tuning it justifies) first as a concrete
win, then scope Stage 2 as its own tracked task once the golden exists to catch regressions.
