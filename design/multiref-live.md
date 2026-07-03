# Multi-reference live rendering — design & plan

**Goal:** make deep floatexp (mode 2) live frames render *real detail* at interactive
rates. Today they cost ~1–5 s/frame on interior/filament-heavy views, which forced the
v0.1.10 "reproject-during-mode-2-motion" hang fix — responsive but **blank**.

## Root cause (profiled 2026-07-03)

The cost is **rebasing against a single, short reference**, not the shader ops:

- A fixed deep view (Elephant-Valley spiral, ~1e40×, 800×450) renders in **~0.4 s at
  iter 2000 but ~15 s at iter ≥ 8000** — a nonlinear jump right at the reference orbit
  length (~6136, where the reference *escapes*), then flat.
- Pixels that iterate past the reference's escape point **rebase** (Zhuoran → `reference[0]`,
  shader `mandelbrot.wgsl` ~line 852). After a rebase δz is large, so **BLA can't skip**, and
  every subsequent step is a full, expensive floatexp iteration until the pixel escapes.
- A longer/better reference does **not** help here: the spiral is a Misiurewicz point
  (pre-periodic → escapes) surrounded by filaments, so no interior/long-orbit reference exists
  nearby. Tested `REF_SCORE_SCAN` 4096→65536: **slower** (17 s→30 s), just more bignum scan.
- Resolution reduction does **not** help interior-heavy frames (still seconds at 85×65) —
  it's per-pixel rebase cost, not total-pixel work.

Confirmed non-fixes (don't retry): resolution/WORK_BUDGET reduction; aggressive BLA rebuild
on zoom-in; predictive/longer reference selection.

## The fix: multiple references, rebase onto the *nearest* one

Production deep-zoomers (Kalles-Fraktaler, Fraktaler-3) place several references across the
view. When a pixel exhausts/rebases, it rebases onto the reference whose current `Z` is
*closest* to the pixel's `z` (smallest δz) — so BLA keeps skipping instead of doing full steps.

What exists to reuse:
- **core**: `reference_orbit` / `best_reference` (pick + iterate one orbit), `build_bla_mandel`
  + `bla_to_gpu` (BLA per orbit), and the *CPU* multi-ref correction (`render_multiref_mandel`,
  `perturb_pixel_mandel`, Pauldelbrot detection) — the reference-*placement* logic (seed →
  glitch centroid → new ref) is reusable, but the iterative CPU re-render is **not** the GPU model.
- **gpu**: `mandelbrot.wgsl` mode-2 loop with single `reference` buffer (binding 1) + one BLA
  appended at `orbit_len`; rebase at ~line 852.

## Implementation plan (incremental — each step visually verified interactively)

1. **CPU: compute K references (K≈4–8).** Place ref 0 by `best_reference`; place refs 1..K at
   points spread across the view (grid, or reuse the glitch-centroid placement). Iterate each
   orbit + build its BLA. Pack all orbits+BLAs into one storage buffer with a per-ref
   `(orbit_off, orbit_len, bla_off, bla_levels)` header (small uniform/array). *Testable
   headlessly by timing a CPU multi-ref render of the spiral vs single-ref.*

2. **GPU buffer + uniforms.** Extend the reference buffer layout to hold K packed references;
   add `ref_count` + the per-ref header array to the iterate uniform (or a small second buffer).
   No shader logic change yet — render with ref 0 only, confirm identical output (regression gate).

3. **Shader: nearest-reference rebase.** At the rebase site, instead of `reference[0]`, loop the
   K references and pick the one minimizing |z − Z_j[0]| (or track a per-pixel current ref and
   switch on exhaustion). Use that ref's BLA thereafter. Start K=2 to validate the mechanism.

4. **Per-pixel initial reference.** Assign each pixel its nearest ref at start (not just on
   rebase) so δc is minimized from iteration 0.

5. **Live wiring + budget.** Compute the K references off-thread (extend `recompute_worker` /
   `RefCache` to hold K); keep the reproject freeze only as a fallback while the K-ref set is
   (re)computing. Remove the blanket mode-2 motion freeze once frames are fast.

6. **Validation.** `--selftest` golden images unchanged (single-ref path must stay bit-identical
   when K=1); interactive: deep spiral tour renders real detail at interactive rates; measure the
   ~15 s spiral frame → target < ~100 ms.

## Risks / open questions
- Per-pixel reference *selection* cost on the GPU (looping K refs per rebase) — keep K small.
- BLA storage: K BLA trees ≈ K× the orbit-buffer size; cap K / VRAM.
- Correctness of nearest-ref rebase across formulas (start Mandelbrot-only, mode 2, non-Julia).
- Interior views vs filament views may want different K / placement.

## Status
Committed 2026-07-03. Groundwork (diagnosis + design) done. Implementation is the next
focused session(s), starting at step 1. The offline `--render-tour` export path already renders
full detail (synchronous per-frame reference), so it is unaffected either way.
