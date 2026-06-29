# Fractadyne validation report

- **Version:** 0.1.0 (build 88)
- **Generated:** 2026-06-29 11:55:07 UTC (unix 1782734107)
- **GPU:** NVIDIA GeForce RTX 3080
- **CPU:** AMD Ryzen 9 3950X 16-Core Processor (16 cores / 32 threads, L2 8192 KB, L3 65536 KB)
- **OS:** windows / x86_64
- **Mode:** BLESS (recording references)

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

**19/19 checks passed.**

## Golden images (320×240)

Stored in `validation\golden`. Recorded this run. pixel tolerance: max ≤ 10, mean ≤ 2.0 (8-bit sRGB).

| Image | Max Δ | Mean Δ | Checksum (FNV-1a) | Verdict | Reproduce |
|---|---|---|---|---|---|
| home | 0 | 0.000 | `c387ba2f582c426d` | 📷 recorded | `fractadyne --render --out home.png --fractal Mandelbrot --center -0.5 0.0 --zoom 1 --size 320 --iter 800 --ss 1 --method smooth --palette 0` |
| seahorse | 0 | 0.000 | `876e75bc70f76d84` | 📷 recorded | `fractadyne --render --out seahorse.png --fractal Mandelbrot --center -0.743643887037151 0.131825904205330 --zoom 2000 --size 320 --iter 1500 --ss 1 --method smooth --palette 1` |
| seahorse-stripe-1e6 | 0 | 0.000 | `c7fce0b7f354080c` | 📷 recorded | `fractadyne --render --out seahorse-stripe-1e6.png --fractal Mandelbrot --center -0.743643887037151 0.131825904205330 --zoom 1000000 --size 320 --iter 4000 --ss 1 --method stripe --palette 1` |
| elephant | 0 | 0.000 | `1017955690592063` | 📷 recorded | `fractadyne --render --out elephant.png --fractal Mandelbrot --center 0.2925755 -0.0149977 --zoom 1500 --size 320 --iter 1500 --ss 1 --method smooth --palette 2` |

**4/4 golden images recorded.**

## Summary

✅ ALL CHECKS PASSED
