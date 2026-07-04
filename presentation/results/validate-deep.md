# Extreme-depth precision validation

Fractadyne 0.1.10 (build 388)

Precision-doubling self-consistency of the arbitrary-precision arithmetic core, at magnifications beyond `f64` range. Iterate `z²+c` (full-mantissa interior point) at precision `p` and at `p+256`; `agree` = leading base-2 bits that match (sound ≈ `p − log₂(k)`). `rt` = bits preserved through a decimal `to_string → parse` round-trip. No GPU, no external data.

| magnification | bits | limbs | k iters | agree (bits) | round-trip (bits) | time (s) | result |
|---|---|---|---|---|---|---|---|
| 1e1000 | 3386 | 52 | 20000 | 3392 | 3395 | 0.34 | PASS |
| 1e10000 | 33284 | 520 | 4000 | 33346 | 33346 | 2.52 | PASS |
| 1e100000 | 332257 | 5191 | 800 | 332289 | 332291 | 17.39 | PASS |
| 1e1000000 | 3321993 | 51906 | 200 | 3322048 | 3322052 | 49.52 | PASS |

**Overall: PASS**

## Scope

This validates the *arithmetic and precision machinery* at extreme bit-width (the depth-critical numerics), not a full rendered image: a per-pixel arbitrary-precision dwell oracle is computationally infeasible this deep, and the renderer's `f64` `units_per_pixel` caps live zoom near 1e308× regardless. Independent per-pixel cross-checks (`--selftest`, `--crosscheck-f3`) cover the renderable depth range.
