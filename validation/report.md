# Fractadyne validation report

- **Version:** 0.1.0 (build 95)
- **Generated:** 2026-06-29 12:38:29 UTC (unix 1782736709)
- **GPU:** NVIDIA GeForce RTX 3080
- **CPU:** AMD Ryzen 9 3950X 16-Core Processor (16 cores / 32 threads, L2 8192 KB, L3 65536 KB)
- **OS:** windows / x86_64
- **Mode:** VALIDATE

All checks use exact mathematics (arbitrary-precision dwell, closed-form properties) or internal cross-checks — no external data. Anyone can reproduce a golden image with the listed command and compare it to `golden/`.

## Numeric, deep-zoom & invariant checks

| Category | Check | Parameters | Result | Threshold | Verdict |
|---|---|---|---|---|---|
| Numeric | df32 perturbation vs CPU f64 dwell | seahorse, 2e4×, 5657 iter, n=6913 | 95.5% agree within 1 iter | ≥90% within 1 iter | ✅ PASS |
| Finiteness | dwell finite (perturbation @2e4×) | all sampled pixels | all finite | all finite | ✅ PASS |
| Numeric | floatexp vs df32 perturbation | seahorse, 1e10× | mean Δ=0.0000 iter, >2iter 0.000% | mean<0.5, <2% differ | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e6x | mode 0, 7102 iter, 25 samples | 14 agree, 11 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e12x | mode 0, 12204 iter, 25 samples | 15 agree, 10 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e16x | mode 0, 15606 iter, 25 samples | 25 agree, 0 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e24x | mode 0, 22409 iter, 25 samples | 25 agree, 0 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e30x | mode 2, 27512 iter, 25 samples | 25 agree, 0 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Glitch | reference independence (3-ref majority) | seahorse, 1e8×, auto vs 2 offset refs (smooth region) | 16650 smooth px: auto dissent 0, no-majority 0 (0.0000%) | <0.2% of smooth pixels | ✅ PASS |
| Invariant | real-axis mirror symmetry | home view (-0.5, 0) | mean Δ=0.00000 iter | mean<0.05 | ✅ PASS |
| Invariant | home has interior + exterior | home view | interior=true, exterior=true | both present | ✅ PASS |
| Symmetry (render) | Multibrot-3 180° rotation | origin view, span 3, 44944 smooth px | 0 asymmetric | 0 asymmetric | ✅ PASS |
| Symmetry (render) | Tricorn real-axis reflection | origin view, span 3, 44846 smooth px | 0 asymmetric | 0 asymmetric | ✅ PASS |
| Symmetry (render) | Celtic real-axis reflection | origin view, span 3, 42414 smooth px | 0 asymmetric | 0 asymmetric | ✅ PASS |
| Consistency | resolution independence (N vs 3N) | seahorse, 1e6×, 25551 smooth px | 0 differ | 0 differ | ✅ PASS |
| Consistency | max-iter monotonic stability | seahorse, 1e6×, 500→3000 iter, 15722 escaped px | 0 changed dwell | 0 changed | ✅ PASS |
| Consistency | zoom-sequence across direct→df32 seam | seahorse, 4e3×↔1.2e4×, 1814 overlap px | 0 differ | <0.1% differ | ✅ PASS |
| Consistency | pan consistency | seahorse, 1e6×, +55px, 23099 overlap px | 1 differ | <0.1% differ | ✅ PASS |
| Consistency | render determinism (2 runs) | seahorse, 1e6× | bit-identical | bit-identical | ✅ PASS |
| Derivative | distance-estimate self-consistency | seahorse, 1e6×, 1037 boundary px | 0 with DE>16px at boundary | <0.5% of boundary px | ✅ PASS |
| Derivative | DE lower bound (Koebe ¼) | seahorse, 1e6×, 12 sampled exterior px | 0 disks contain interior | 0 | ✅ PASS |
| Catalog | period-2 disk center (c = -1) | zoom 5e1 | period 2 (want 2), nucleus Δ=1.6e-23 | period + nucleus | ✅ PASS |
| Catalog | period-3 bulb nucleus | zoom 8e1 | period 3 (want 3), nucleus Δ=3.6e-16 | period + nucleus | ✅ PASS |
| Catalog | period-3 antenna minibrot (real axis) | zoom 3e2 | period 3 (want 3), nucleus Δ=6.0e-17 | period + nucleus | ✅ PASS |
| Catalog | period-4 window nucleus (real axis) | zoom 3e2 | period 4 (want 4), nucleus Δ=8.4e-17 | period + nucleus | ✅ PASS |
| Catalog | deep period-998 minibrot (Seahorse Valley, 2e7x) | zoom 2e7 | period 998 (want 998), nucleus Δ=3.4e-41 | period + nucleus | ✅ PASS |
| Catalog | main-cardioid interior (c = -0.5) | interior expected true | oracle says interior=true | matches catalog | ✅ PASS |
| Catalog | exterior point (c = 1) | interior expected false | oracle says interior=false | matches catalog | ✅ PASS |
| Catalog | deep minibrot nucleus interior (full precision) | interior expected true | oracle says interior=true | matches catalog | ✅ PASS |

**29/29 checks passed.**

## Coverage & scope

What each oracle independently verifies, and its validity range:

- **Naive bignum dwell** (arbitrary precision, no perturbation/reference): exact integer escape count at **any depth** — the only fully independent deep-zoom oracle. Tested 1e6×–1e30× across the real render modes (df32 + floatexp).
- **CPU f64 dwell**: exact only to ~f64 coordinate resolution (≲1e13×); used for the shallow cross-check.
- **floatexp ↔ df32 agreement**: internal consistency in the overlap band; not an external oracle by itself.
- **Reference independence**: oracle-free glitch detection (multi-reference majority); confirms the chosen reference is clean, doesn't prove a coordinate.
- **Symmetries / landmarks / consistency / derivative checks**: exact mathematics, any depth, but each only constrains the property it tests.
- **Catalog**: full-precision locations with externally known answers (period, nucleus, membership) — reproduce independently from `validation/catalog.toml`.

**Not independently oracle-checked:** non-Mandelbrot family *dwell* at depth (only their symmetry is checked); interior-coloring/decomposition exactness; coloring beyond the integer dwell. Aim scrutiny there.

## Golden images (320×240)

Stored in `validation\golden`. Compared against; current renders written to `current/` for side-by-side review. pixel tolerance: max ≤ 10, mean ≤ 2.0 (8-bit sRGB).

| Image | Max Δ | Mean Δ | Checksum (FNV-1a) | Verdict | Reproduce |
|---|---|---|---|---|---|
| home | 0 | 0.000 | `c387ba2f582c426d` | ✅ match | `fractadyne --render --out home.png --fractal Mandelbrot --center -0.5 0.0 --zoom 1 --size 320 --iter 800 --ss 1 --method smooth --palette 0` |
| seahorse | 0 | 0.000 | `876e75bc70f76d84` | ✅ match | `fractadyne --render --out seahorse.png --fractal Mandelbrot --center -0.743643887037151 0.131825904205330 --zoom 2000 --size 320 --iter 1500 --ss 1 --method smooth --palette 1` |
| seahorse-stripe-1e6 | 0 | 0.000 | `c7fce0b7f354080c` | ✅ match | `fractadyne --render --out seahorse-stripe-1e6.png --fractal Mandelbrot --center -0.743643887037151 0.131825904205330 --zoom 1000000 --size 320 --iter 4000 --ss 1 --method stripe --palette 1` |
| elephant | 0 | 0.000 | `1017955690592063` | ✅ match | `fractadyne --render --out elephant.png --fractal Mandelbrot --center 0.2925755 -0.0149977 --zoom 1500 --size 320 --iter 1500 --ss 1 --method smooth --palette 2` |

**4/4 golden images within tolerance.**

## Summary

✅ ALL CHECKS PASSED
