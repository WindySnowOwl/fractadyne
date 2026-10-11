# The reference orbit's precision schedule — and extending it past Mandelbrot

Status: the Mandelbrot schedule shipped in 0.3.0-beta.18 (`8e77046`). This note records what it is,
what the 2026-10-11 measurements found, and the plan to extend it to the other families.

## What it is

A step `Z_{n+1} = Z_n² + c` rounded to `p` bits injects an error `e ≈ 2^-p · max(|Z_n|², |c|)` (the
products are rounded before `c` is added, so a near-return does not make it small). Perturbation
carries `e` into every pixel exactly, where it acts as a shift of the pixel's `c` by
`e / |dZ_{n+1}/dc|`. The build's precision `p0` is set for the pixel spacing at the start, where
`dZ/dc = 1`; as `|dZ/dc|` grows the same pixel-relative accuracy needs fewer bits:

    p_{n+1} = p0 + log2 max(|Z_n|², |c|) − log2 |dZ_{n+1}/dc| + 112

in whole 64-bit words, moving in steps of `p0/64`, rising at once when `dZ/dc` falls back near a
return. The code: `Schedule` in `crates/fractadyne-core/src/reference.rs`; both backends; Mandelbrot
from `Z_0 = 0` only. The 112-bit margin keeps the stored samples byte-identical with the schedule on
and off (checked on all 31 bench scenes).

**Prior art (searched 2026-10-11).** Not original. github.com/gcollombet/mandelbrot (a Rust/WASM
explorer) has computed its reference with the same derivative-based descending precision since a
commit dated 2026-06-30, with a design note (`openspec/changes/fix-reference-precision-budget/
design.md`, 2026-07-01) deriving `p_k > −log2(scale) − log2|dZ_k/dC| + guard`; GitHub's own Pages
build of 2026-07-06 contains both. Its version has no operand-size term, works in decimal digits and
takes its budget from a user-set target depth. The backward-error comparison itself goes back to
knighty's 2016 glitch test (fractalforums.com, the *Continued* SuperFractalThing thread, msg91505:
`2^-pmax max(2|z_n|, |δ_n|)` against derivative × pixel) and gerrit's 2018 "Notes on Mandelbrot set"
("step method"); both used it for glitch detection at fixed precision. No established renderer
(Imagina, FractalShark, KF2, Fraktaler-3, Fractal Zoomer, rust-fractal, DeepDrill, SFT, mightymandel)
varies the reference's precision per step.

## Measurements, 2026-10-11

`FRACTADYNE_SCHED_TRACE=<dir>` makes every scheduled walk of ≥ 10,000 steps write one CSV row per
change of its bits (step, bits, `log2|Z_n|`, `log2|dZ/dc|`, ns). `local/bench-ladder/sched_measure.py`
renders kit scenes with it (MPFR build, 640x360, so the time is the orbit):

| scene | walk | steps | mean bits | time at full bits | time in return stretches | near-return depth (median / max bits) |
|---|---|---|---|---|---|---|
| 55, 1e30000 spiral path | 55.0 s | 821,552 at 99,904 | 51% | 20.7% (1.8% of steps) | 0.0% | 2 / 5 |
| 60, 1.5e30000 minibrot | 44.6 s | 410,594 at 99,904 | 76% | 26.6% (3.6% of steps) | 0.0% | 2 / 50,010 (the period's own close) |
| 54, 1e10000 spiral path | 7.4 s | 275,085 at 33,472 | 51% | 31.8% (2.7%) | 0.1% | 2 / 5 |
| 46, 3.4e10002 minibrot | 6.1 s | 137,396 at 33,472 | 76% | 36.3% (5.4%) | 0.1% | 2 / 16,792 |

**Self-perturbed return stretches: dropped.** mathr's 2023 idea (fractalforums.org topic 5142: after
a near-return compute the reference as a perturbation of its own start at "full precision +
exponent(ΔZ)") targets the steps where the schedule climbs back to full. These orbits' near-returns
are shallow (|Z| a few bits below 1); the only deep one in a minibrot scene is the period closing
at its last step. Time in return stretches ≈ 0, so there is nothing for it to save here.

**The orbit's opening is slow for another reason: the pick's phase 1.** The full-precision time is
the opening stretch (the first 14,811 steps, before `|dZ/dc|` has grown), and it runs at 761–802
µs/step where the next stretch, 1.6% fewer bits, runs at 113 µs/step. Phase 1 of the reference pick
(`pick_pass`) walks 101 candidates × `REF_SCORE_SCAN` = 4,096 steps at full precision on one
thread per logical CPU (`par_orbit_scores`): ~414,000 full-precision steps, half the length of the
whole orbit, beside the build's 9 split-product threads. Interleaved A/B on scene 55 with
`FRACTADYNE_PICK_SCAN_THREADS=6` (a diagnostic, not kept): opening 761–781 → 139 µs/step, build
walk 57.0 → 51.1–53.8 s, wall 58.5–63.1 → 52.7–55.6 s; images identical, same pick.

## Plan

### P0 — the pick's phase 1 (Mandelbrot, every deep render)

Two options, smallest first:

1. **Leave the build its cores.** When the centre's build runs beside the pick (`centre_src.whole`),
   cap phase 1 at `available − build threads` (9 with split products, else the walk's own). Measured
   above: −4 to −6 s at 1e30000. Must be checked at 1e1000–1e10000, where a slower phase 1 could
   delay a pick whose build is short.
2. **Score phase 1 by perturbation.** At depth every candidate is within half a view of the centre;
   their 4,096-step bignum walks are near-duplicates. Phase 2 already δ-scores survivors in floatexp
   against the centre's recorded walk with a trust rule (design/pick-redesign.md). Doing the same for
   phase 1 removes its ~414,000 full-precision steps: the build would run uncontended (scene 55's
   opening alone would save ~9.5 s; the whole saving is unmeasured). Same gate as phase 2 (`!julia`,
   polynomial, deep spans); `--pickcheck` must show the same winner on the ladder and the corpus.

### P1 — Multibrot 3–8 (c plane)

`Z_{n+1} = Z_n^d + c`: error `2^-p · max(|Z_n|^d, |c|)` (the margin absorbs the `log2 d` from the
power's partial products); derivative `dZ_{n+1}/dc = d·Z_n^{d−1}·dZ_n/dc + 1`. These run in the
generic loop (`run_orbit_gen`), which already takes each step's bits from the schedule, for both
backends. Changes: `SchedState::fresh` admits the polynomial family with its power; `Schedule::advance`
uses `d`. The Mandelbrot arm stays exactly as it is (byte-identical schedule, so byte-identical
samples and blobs).

### P2 — Julia views of the polynomial families

`Z_0` is the pixel, `c` the constant: derivative with respect to `Z_0`, `D_0 = 1`,
`D_{n+1} = d·Z_n^{d−1}·D_n` (no `+1`), same error term. `D` can fall below 1 (near the critical
point, attracting cycles); the bits then cap at full, which is today's behaviour. The pick's Julia
walk and the build both pass `from_zero = false` today; they need the Julia flag instead.

### P3 — the fold families (Tricorn, Burning Ship, Celtic, Buffalo; powers 2–5; both planes)

Not holomorphic, so `dZ/dc` becomes a real 2×2 Jacobian `J`: per step `J' = F · P · G · J + I` (c plane;
no `+ I` for Julia), with `P` the 2×2 of multiplying by `d·w^{d−1}`, `G` the fold before the power
(Burning Ship `diag(sgn x, sgn y)`, Tricorn `diag(1, −1)`) and `F` the fold after (Celtic
`diag(sgn Re w, 1)`, Buffalo `diag(sgn Re w, sgn Im w)`). A rounding error `e` moves a pixel's
parameter by `J^{-1}e`, bounded by `|e| / σ_min(J)`; use `log2 σ_min ≥ log2|det J| − log2‖J‖_F` (a
Burning Ship Jacobian can be far from conformal, so `‖J‖` alone would under-provision). The fold is
Lipschitz, so an error at a fold crossing stays `O(e)`; the margin and the byte-identity tests are
the check. Power 2 runs in the MPFR in-place loop (`try_run_orbit_inplace`, already per-step bits;
gate `formula == MANDELBROT` today), powers 3–5 in the generic loop. `SchedState` grows the 2×2
state, so `OrbitTail` and the orbit blob change: blob v4, still reading v3.

### Not planned now

- Phoenix (`z² + c + p·z_{n−1}`): two-step derivative state; first confirm it renders deep at all.
- Newton: convergent, no deep reference worth scheduling.
- Custom formulas: would need a derivative program from the IR; the perturbation rewriter
  (`ir/perturb.rs`) is the place to start. Later.

### Gates, every phase

- Core tests, both backends: stored samples byte-identical with the schedule on and off over long
  chaotic orbits and minibrot (nucleus) orbits, at 1,024–4,096 bits; `extend == fresh`; the MPFR
  identity test; and an anti-vacuity check that the schedule actually reduced bits (`reduced > 0`),
  since a test that passes with the schedule off proves nothing.
- Deep scenes per family (none exist: all 69 view files in the repo are Mandelbrot, c plane). Make
  them with the nucleus finder (polynomial families) and the app (folds, Julia) at ~1e1000, 1e5000
  and 1e10000; render A/B on and off: images byte-identical, times recorded.
- Selftest, goldens, the F3 corpus `--check`: unchanged.

### Expected gain

Where the orbit dominates, the same as Mandelbrot at the same depth: at 1e10000 the walk took
14.5 → 8.4 s and at 1e30000 140 → 69 s at 720p. Below ~1e1000 the orbit is a small part of a render
and the schedule changes little. The generic loop allocates per operation, so its steps cost more
than the in-place Mandelbrot walk's, and the saving in seconds is larger for the same ratio.
