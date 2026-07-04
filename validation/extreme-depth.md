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
