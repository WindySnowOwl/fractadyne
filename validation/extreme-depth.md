# Extreme-depth validation (to 1e1000000×)

How deep can we *validate*? The answer splits in two, because the binding constraint
changes with depth.

## What limits validation depth

The independent ground truth is the arbitrary-precision **CPU bignum dwell oracle**
(`naive_dwell_bf`): it iterates `z → z² + c` in full `astro-float` precision, with no
perturbation and no reference orbit, so it is *exact at any depth* — there is no precision
ceiling to correctness. The practical limit is **CPU time**, and it differs by check type:

- **Per-pixel oracle (full image)** — used by `--selftest` and `--crosscheck-f3`. Cost is
  `pixels × iterations × M(precision)`. Feasible to ~1e30× (committed selftest battery) and
  demonstrable deeper, but it does **not** scale to 1e1000000×: at that depth one iteration
  of one pixel costs ~32 ms (see below), so a full frame would take years.

- **Single-point precision self-consistency** — used by `--validate-deep`. Cost is just
  `k × M(precision)` for a handful of points, which *is* feasible to 1e1000000× and beyond.

## The bignum cost is FFT-bounded, not quadratic

`astro-float` switches to **FFT multiplication** above ~5400 limbs, so the per-iteration
cost grows ~linearly (× log) with precision rather than quadratically. Measured on this
machine (Ryzen 9 3950X), full-precision `z²+c` iterations:

| Magnification  | Precision (bits) | Limbs      | ns / iteration          |
| -------------- | ---------------- | ---------- | ----------------------- |
| 1e30×          | 163              | 2          | ~680                    |
| 1e3000×        | 10,029           | 156        | ~15,700                 |
| 1e30000×       | 99,721           | 1,558      | ~550,000                |
| 1e300000×      | 996,642          | 15,572     | ~7,970,000              |
| **1e1000000×** | **3,321,992**    | **51,906** | **~32,000,000** (32 ms) |

The 1e300000→1e1000000 step (×3.33 limbs → ×4.0 time) confirms FFT scaling, not schoolbook
`O(n²)` (which would be ×11).

## The precision-doubling validation technique

With no external corpus at this depth, `--validate-deep` uses the standard
arbitrary-precision validation method: compute a quantity at precision `p`, then again at
`p + 256` bits, and require they agree. If the `p`-bit answer is stable under a precision
increase it is almost certainly correct; a precision-propagation or arithmetic bug at that
bit-width collapses the agreement. The probe iterates `z²+c` from a **full-mantissa**
interior point (seeded by `√½`, so the multiply exercises real carries across all limbs)
and also round-trips a coordinate through decimal (`to_string → parse`) to validate the
persisted-location I/O path at extreme precision.

## Results (this machine)

```
      magnif.        bits   limbs       k   agree(bits)      rt(bits)   time(s)  result
      1e1000         3386      52   20000          3392          3395      0.45  PASS
      1e10000       33284     520    4000         33346         33346      2.89  PASS
      1e100000      332257    5191     800        332289        332291     20.06  PASS
      1e1000000     3321993   51906     200       3322048       3322052     64.94  PASS
```

Every case agrees to its full working precision (≳ `p` bits) and round-trips losslessly —
the arithmetic and precision machinery are sound at **3.3-million-bit** precision.

## Reproduce

```sh
fractadyne --validate-deep [--out report.md]      # exit 0 = all passed
cargo test -p fractadyne-core deep_precision_self_consistent_1e1000
cargo test -p fractadyne-core -- --ignored deep_precision_self_consistent_1e100000
```

## Scope / caveat

This validates the **arithmetic and precision core** (the depth-critical numerics), not a
full rendered image at 1e1000000×: a per-pixel arbitrary-precision dwell oracle is
computationally infeasible there (see the cost table above).

The renderer's *scale* is no longer the limit — the viewport now uses an extended-range
`FloatExp` (`m·2^e`) for `units_per_pixel`, so live zoom runs past the old `f64` ~1e308×
ceiling (verified rendering correctly at **1e331×**; `--render --zoom-log2 L` drives it).
What now bounds a *rendered* deep image is the reference-orbit / iteration cost and the
coordinate precision you supply, not the scale representation.

Independent per-pixel cross-checks (`--selftest` vs the bignum oracle to 1e30×;
`--crosscheck-f3` vs Fraktaler-3) cover the renderable depth range; `--validate-deep` covers
the arithmetic far beyond it.

## A rendered image at 2.37e4000×, cross-checked against Fraktaler-3 (2026-08-31)

A user's Misiurewicz spiral, found with the log-space feature solver and rendered by both apps from
the same 4,031-digit centre. This is the deepest *structural* render the project has produced — past
the deepest F3-matched corpus pair (4.6e1105×) — and the deepest point at which we have an
independent second opinion.

| | file | notes |
|---|---|---|
| location | `e4000-misiurewicz.fdn` | 4,031-digit centre, `upp_log2 = -13294.9`, 2,008,192 iterations. ⚠Depth lives in `upp_log2`; `zoom` is `f64` and cannot hold 1e4000. |
| ours | `e4000-fractadyne.png` | 400×250, `--iter 2008192`, ss=2 |
| Fraktaler-3 3.1 | `e4000-fraktaler3.png` | 400×250, `subframes = 1`, same iteration/reference/perturb caps |

**They agree structurally** — same spiral, same arm count, same arrangement. F3's own colouring
makes a pixel diff meaningless (see `corpus/README.md`), so this is a visual cross-check, which at
this depth is the only kind available.

⛔**F3 is much faster here, and the gap is ours to own.** F3: **60.3 s**. Us: **831 s** originally,
of which **830 s was reference building** — the GPU render itself was **0.58 s**.

Chasing that found the export path building the same reference two and three times over; both
duplicates are now gone (`eaa16ec`, `f21683f` — the colour pass and the correction pass each reused
`current_export_request_for` instead of the request already in hand). Measured here:

| | e4000, 400×250 | hero, e500 |
|---|---|---|
| 3 builds (original) | 831 s | 24.3 s |
| 2 builds | 771 s | 13.1 s |
| 1 build | **529 s** | **10.3 s** |

⚠**F3's lead is now ~2.9×** (2026-09-01): the 529 s row below predates the SA cost budget
(`SA_COST_BUDGET`, fractadyne-core), which cut the series-approximation walk 258.2 s → 48.2 s at
this location — same command, **333 s**, corpus 38/38 byte-identical, the spiral unchanged. ⚠A
re-render of this location now differs from the committed PNGs in low bits (the skip moved
439,915 → 84,126); the committed pair documents the 2026-08-31 comparison and stays. What remains
of the gap is the reference PICK (113.7 s — deep survivor scoring), filed in TODO.md with the
redesign options. The old note blamed candidate scoring for "~99%"; that is true only at shallow
depth — at e4000 the split was pick 28% / orbit 8% / **SA 64%** / BLA 0.1%, which is why the SA
budget was the first fix.

✅**Re-measured 2026-09-02 (0.2.41-beta.11), same command** — the pick redesign
(`23d03f7`) and its backend dispatch (`52b5b72`) landed since the 333 s row:

| | default (astro) | accelerated (MPFR) | Fraktaler-3 |
|---|---|---|---|
| pick / orbit / SA / BLA | 99.4 / 33.3 / 47.4 / 0.4 s | 23.5 / 7.6 / 47.5 / 0.5 s | — |
| **wall** | **183.8 s** | **82.2 s** | **60.3 s** |
| **F3's lead** | **3.05×** | **1.36×** | — |

astro↔MPFR decoded-RGB 0/100,000 pixels differ, same winner (443,199), identical pick
counters. ✅**The SA lever landed (0.2.41-beta.12, 2026-09-03)**: the coefficient walk now
dispatches through the backend like the pick — accelerated SA 47.5 → 13.6 s, **wall 82.2 →
47.5 s, ahead of F3's 60.3 s at this location** (pick 23.3 / orbit 7.3 / SA 13.6 / BLA 0.5;
pixels still 0/100,000 differ vs astro, same skip 84,126). Default build unchanged
(183.8 s — the astro walk is kept verbatim). One location at 400×250 is not an "owns
extreme" claim; publish per-config only.

⚠**Supersampling was not matched in the original run** (ss=2 vs F3's 1); the 771 s and 529 s rows
are ss=1. It changes nothing either way — all the pixel work together was 0.58 s of 831 s. This is
a reference-BUILD comparison, not a renderer comparison.

