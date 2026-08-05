# Fractadyne validation report

- **Version:** 0.2.40-beta.12 (build 997)
- **Generated:** 2026-08-05 12:03:02 UTC (unix 1785931382)
- **GPU:** NVIDIA GeForce RTX 3080
- **CPU:** AMD Ryzen 9 3950X 16-Core Processor (16 cores / 32 threads, L2 8192 KB, L3 65536 KB)
- **OS:** windows / x86_64
- **Config:** fractal=Mandelbrot julia=false auto_iter=true max_iter=4000 sa=true bla=true glitch=true color=Smooth
- **Mode:** VALIDATE

All checks use exact mathematics (arbitrary-precision dwell, closed-form properties) or internal cross-checks — no external data. Anyone can reproduce a golden image with the listed command and compare it to `golden/`.

## Numeric, deep-zoom & invariant checks

| Category | Check | Parameters | Result | Threshold | Verdict |
|---|---|---|---|---|---|
| Numeric | df32 perturbation vs CPU f64 dwell | seahorse, 2e4×, 5763 iter, n=6913 | 95.7% agree within 1 iter | ≥90% within 1 iter | ✅ PASS |
| Finiteness | dwell finite (perturbation @2e4×) | all sampled pixels | all finite | all finite | ✅ PASS |
| Numeric | floatexp vs df32 perturbation | seahorse, 1e10× | mean Δ=0.0000 iter, >2iter 0.000% | mean<0.5, <2% differ | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e6x | mode 0, 7208 iter, 25 samples | 14 agree, 11 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e12x | mode 0, 12311 iter, 25 samples | 15 agree, 10 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e16x | mode 0, 15712 iter, 25 samples | 25 agree, 0 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e24x | mode 0, 21631 iter, 25 samples | 25 agree, 0 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Bignum oracle | naive bignum dwell vs GPU @1e30x | mode 2, 26016 iter, 25 samples | 25 agree, 0 boundary, 0 mismatch | 0 hard mismatches | ✅ PASS |
| Series approximation | SA seed vs full iteration @1e30× | Mandelbrot, 1e30×, skip 26015 of 26016 iter | max Δ 0.0000 smooth iter | skip>0 and max Δ < 0.05 | ✅ PASS |
| Series approximation | SA gated off when BLA active @1e30× | Mandelbrot mode 2, SA toggle on, BLA on | sa_skip 0, bla_on 1 | sa_skip == 0 and bla_on == 1 | ✅ PASS |
| Series approximation | SA seed vs full iteration @1e20× (mode 0) | Mandelbrot, 1e20×, mode 0, skip 18706 of 18707 iter | max Δ 0.0000 smooth iter | mode 0, skip>0, max Δ < 0.05 | ✅ PASS |
| Glitch | reference independence (3-ref majority) | seahorse, 1e8×, auto vs 2 offset refs (smooth region) | 16649 smooth px: auto dissent 1, no-majority 0 (0.0060%) | <0.2% of smooth pixels | ✅ PASS |
| Glitch | glitch detection responds to reference quality | seahorse, 1e8×, auto vs far-offset reference | auto-ref flagged 9, far-ref flagged 10 | detection fires (>0) and far-offset flags ≥ auto | ✅ PASS |
| Glitch | multi-reference correction resolves glitches | seahorse, 1e8×, auto seed + correction | 7 references, 0 residual glitches | 0 residual glitches | ✅ PASS |
| Glitch | corrected buffer colors to a valid image | seahorse, 1e8×, render_export_corrected | finite true, dark true, bright true, plain interior px 92 | finite + structured (interior & exterior) | ✅ PASS |
| Invariant | real-axis mirror symmetry | home view (-0.5, 0) | mean Δ=0.00000 iter | mean<0.05 | ✅ PASS |
| Invariant | home has interior + exterior | home view | interior=true, exterior=true | both present | ✅ PASS |
| Symmetry (render) | Multibrot-3 180° rotation | origin view, span 3, 44944 smooth px | 0 asymmetric | 0 asymmetric | ✅ PASS |
| Symmetry (render) | Tricorn real-axis reflection | origin view, span 3, 44846 smooth px | 0 asymmetric | 0 asymmetric | ✅ PASS |
| Symmetry (render) | Celtic real-axis reflection | origin view, span 3, 42414 smooth px | 0 asymmetric | 0 asymmetric | ✅ PASS |
| Abs-family deep zoom | Burning Ship perturbation vs direct | 1e5×, mode 0 vs 1, n=48400 | mean Δ=0.0000 iter, >2iter 0.000% | mode 0, mean<0.5, <2% differ, n>0 | ✅ PASS |
| Abs-family deep zoom | Burning Ship floatexp vs df32 | 1e10×, mode 2 vs 0, n=48400 | mean Δ=0.0000 iter, >2iter 0.000% | mean<0.5, <2% differ, n>0 | ✅ PASS |
| Abs-family deep zoom | Burning Ship deep finiteness @1e35× | 1e35×, mode 2 | finite dwell, 48400 escaped / 0 interior, spread 0.0 iter | mode 2, all finite | ✅ PASS |
| Abs-family deep zoom | Celtic perturbation vs direct | 1e5×, mode 0 vs 1, n=31 | mean Δ=0.1837 iter, >2iter 0.000% | mode 0, mean<0.5, <2% differ, n>0 | ✅ PASS |
| Abs-family deep zoom | Celtic floatexp vs df32 | 1e10×, mode 2 vs 0, n=1 | mean Δ=0.0000 iter, >2iter 0.000% | mean<0.5, <2% differ, n>0 | ✅ PASS |
| Abs-family deep zoom | Celtic deep finiteness @1e35× | 1e35×, mode 2 | finite dwell, 9319 escaped / 39081 interior, spread 1217.2 iter | mode 2, all finite | ✅ PASS |
| Abs-family deep zoom | Buffalo perturbation vs direct | 1e5×, mode 0 vs 1, n=48400 | mean Δ=0.0001 iter, >2iter 0.000% | mode 0, mean<0.5, <2% differ, n>0 | ✅ PASS |
| Abs-family deep zoom | Buffalo floatexp vs df32 | 1e10×, mode 2 vs 0, n=48400 | mean Δ=0.0000 iter, >2iter 0.000% | mean<0.5, <2% differ, n>0 | ✅ PASS |
| Abs-family deep zoom | Buffalo deep finiteness @1e35× | 1e35×, mode 2 | finite dwell, 48400 escaped / 0 interior, spread 0.0 iter | mode 2, all finite | ✅ PASS |
| Phoenix deep zoom | Phoenix perturbation vs direct | 1e5×, mode 0 vs 1, n=34635 | mean Δ=0.0072 iter, >2iter 0.000% | mean<0.5, <2% differ, n>0 | ✅ PASS |
| Phoenix deep zoom | Phoenix floatexp vs df32 | 1e5×, mode 2 vs 0, n=34635 | mean Δ=0.0000 iter, >2iter 0.000% | mean<0.5, <2% differ, n>0 | ✅ PASS |
| Series approximation | Multibrot 3 SA engages + matches SA-off @1e7× | mode 0, skip 3999 of 4000 iter, 0 escaped | 0 mismatch, finite | skip>0, mode 0, finite, 0 mismatch | ✅ PASS |
| Series approximation | Multibrot 4 SA engages + matches SA-off @1e7× | mode 0, skip 3999 of 4000 iter, 0 escaped | 0 mismatch, finite | skip>0, mode 0, finite, 0 mismatch | ✅ PASS |
| Series approximation | Multibrot 5 SA engages + matches SA-off @1e7× | mode 0, skip 3999 of 4000 iter, 0 escaped | 0 mismatch, finite | skip>0, mode 0, finite, 0 mismatch | ✅ PASS |
| BLA | BLA render == non-BLA @1e30× | Mandelbrot mode 2, bla_on 1, 0 escaped / 48400 interior | 0 mismatch | bla engaged, 0 mismatch | ✅ PASS |
| BLA | BLA escape path == non-BLA @1e30× (boundary) | seahorse boundary, mode 2, bla_on 1, 48400 escaped | 0 mismatch | bla engaged, escapers>100, 0 mismatch | ✅ PASS |
| BLA | orbit-trap-point: BLA-fold == non-BLA @1e30× | bla_on 1, maxΔ 0.0000 | 0/193600 channels >2% | bla engaged, maxΔ<0.1, <1% differ | ✅ PASS |
| BLA | orbit-trap-cross: BLA-fold == non-BLA @1e30× | bla_on 1, maxΔ 0.0000 | 0/193600 channels >2% | bla engaged, maxΔ<0.1, <1% differ | ✅ PASS |
| BLA | orbit-trap-circle: BLA-fold == non-BLA @1e30× | bla_on 1, maxΔ 0.0000 | 0/193600 channels >2% | bla engaged, maxΔ<0.1, <1% differ | ✅ PASS |
| BLA | triangle-ineq: BLA-fold == non-BLA @1e30× | bla_on 1, maxΔ 0.0000 | 0/193600 channels >2% | bla engaged, maxΔ<0.1, <1% differ | ✅ PASS |
| BLA | stripe: BLA-fold == non-BLA @1e30× | bla_on 1, maxΔ 0.0000 | 0/193600 channels >2% | bla engaged, maxΔ<0.1, <1% differ | ✅ PASS |
| Consistency | resolution independence (N vs 3N) | seahorse, 1e6×, 25544 smooth px | 0 differ | 0 differ | ✅ PASS |
| Consistency | max-iter monotonic stability | seahorse, 1e6×, 500→3000 iter, 15737 escaped px | 0 changed dwell | 0 changed | ✅ PASS |
| Consistency | zoom-sequence across direct→df32 seam | seahorse, 4e3×↔1.2e4×, 1814 overlap px | 0 differ | <0.1% differ | ✅ PASS |
| Consistency | pan consistency | seahorse, 1e6×, +55px, 23092 overlap px | 1 differ | <0.1% differ | ✅ PASS |
| Consistency | render determinism (2 runs) | seahorse, 1e6× | bit-identical | bit-identical | ✅ PASS |
| Derivative | distance-estimate self-consistency | seahorse, 1e6×, 1037 boundary px | 0 with DE>16px at boundary | <0.5% of boundary px | ✅ PASS |
| Derivative | DE lower bound (Koebe ¼) | seahorse, 1e6×, 12 sampled exterior px | 0 disks contain interior | 0 | ✅ PASS |
| Counters | BLA skips fire @1e30× (execution proof) | bla_on=1 iter=4000 | bla_skip=774400 rebase=0 maxiter_px=0 | bla_on and bla_skip > 0 | ✅ PASS |
| Counters | extended-range samples + rebases fire on a dip orbit @1.2e148× | orbit_len=5001 render_iter=20000 sa=off bla=off | ext=48400 rebase=90798400 maxiter_px=48400 | ext > 0 and rebase > 0 | ✅ PASS |
| Catalog | period-2 disk center (c = -1) | zoom 5e1 | period 2 (want 2), nucleus Δ=1.6e-23 | period + nucleus | ✅ PASS |
| Catalog | period-3 bulb nucleus | zoom 8e1 | period 3 (want 3), nucleus Δ=3.6e-16 | period + nucleus | ✅ PASS |
| Catalog | period-3 antenna minibrot (real axis) | zoom 3e2 | period 3 (want 3), nucleus Δ=6.0e-17 | period + nucleus | ✅ PASS |
| Catalog | period-4 window nucleus (real axis) | zoom 3e2 | period 4 (want 4), nucleus Δ=8.4e-17 | period + nucleus | ✅ PASS |
| Catalog | deep period-998 minibrot (Seahorse Valley, 2e7x) | zoom 2e7 | period 998 (want 998), nucleus Δ=3.4e-41 | period + nucleus | ✅ PASS |
| Catalog | main-cardioid interior (c = -0.5) | interior expected true | oracle says interior=true | matches catalog | ✅ PASS |
| Catalog | exterior point (c = 1) | interior expected false | oracle says interior=false | matches catalog | ✅ PASS |
| Catalog | deep minibrot nucleus interior (full precision) | interior expected true | oracle says interior=true | matches catalog | ✅ PASS |
| View format | metadata round-trips a deep view | serialize → scramble → load | iter 1234 aa 3 upp_log2 -120.000 cx -0.743643887037151 | clean load; fractal/iter/aa/zoom/center preserved | ✅ PASS |
| View format | newer format_version flagged | format_version=999 | saved by a newer Fractadyne (format v999); some settings may not apply — consider updating | newer == Some(999) | ✅ PASS |
| View format | hostile fields clamped + reported | upp_log2=-1e30, max_iter=4e9, aa=9999, cycle=inf, bogus_field | iter 10000000 aa 16 upp_log2 -3.40e7; clamped [zoom depth, max_iter, cycle, offset, anti-aliasing]; unknown [bogus_field] | clamped & finite; report lists clamped + unknown | ✅ PASS |
| Formatting | zoom mantissa grouped | 3.38050027227e15 | 3.38050 02722 7e15 | "3.38050 02722 7e15" | ✅ PASS |
| Formatting | deep coordinate elides middle | 32-digit center @ ~1e30×; and -0.5 | -0.74364 38870 … 11477 40000  |  -0.5 | leading … frontier; short coord safe | ✅ PASS |
| bench-matrix | direct-1e2 | path signature vs baseline | mode 1 eff-it 512 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | df32-1e8 | path signature vs baseline | mode 0 eff-it 3000 sa-skip 78 counters ok | exact | ✅ PASS |
| bench-matrix | df32-1e20 | path signature vs baseline | mode 0 eff-it 15000 sa-skip 7954 counters ok | exact | ✅ PASS |
| bench-matrix | floatexp-1e30-sa | path signature vs baseline | mode 2 eff-it 30000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | floatexp-1e30-nosa | path signature vs baseline | mode 2 eff-it 30000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | floatexp-1e30-nobla | path signature vs baseline | mode 2 eff-it 30000 sa-skip 16014 counters ok | exact | ✅ PASS |
| bench-matrix | color-smooth | path signature vs baseline | mode 0 eff-it 15000 sa-skip 7954 counters ok | exact | ✅ PASS |
| bench-matrix | color-stripe | path signature vs baseline | mode 0 eff-it 15000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | color-trap | path signature vs baseline | mode 0 eff-it 15000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | color-decomposition | path signature vs baseline | mode 0 eff-it 15000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-mandelbrot | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-multibrot3 | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-multibrot4 | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-multibrot5 | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-tricorn | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-burningship | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-celtic | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-buffalo | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-phoenix | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |
| bench-matrix | fractal-newton | path signature vs baseline | mode 1 eff-it 2000 sa-skip 0 counters ok | exact | ✅ PASS |

**83/83 checks passed.**

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

Stored in `validation/golden`. Compared against; current renders written to `current/` for side-by-side review. pixel tolerance: max ≤ 10, mean ≤ 2.0 (8-bit sRGB).

| Image | Max Δ | Mean Δ | Checksum (FNV-1a) | Verdict | Reproduce |
|---|---|---|---|---|---|
| home | 0 | 0.000 | `13d3fde7f7ebb199` | ✅ match | `fractadyne --render --out home.png --fractal "Mandelbrot" --center -0.5 0.0 --zoom 1 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| seahorse | 0 | 0.000 | `9b392fa6aac0138f` | ✅ match | `fractadyne --render --out seahorse.png --fractal "Mandelbrot" --center -0.743643887037151 0.131825904205330 --zoom 2000 --size 320 --iter 1500 --ss 1 --method smooth --palette 1 --no-watermark` |
| seahorse-stripe-1e6 | 0 | 0.000 | `4ef06b8485bc96e7` | ✅ match | `fractadyne --render --out seahorse-stripe-1e6.png --fractal "Mandelbrot" --center -0.743643887037151 0.131825904205330 --zoom 1000000 --size 320 --iter 4000 --ss 1 --method stripe --palette 1 --no-watermark` |
| elephant | 0 | 0.000 | `e2a5b19d2794df96` | ✅ match | `fractadyne --render --out elephant.png --fractal "Mandelbrot" --center 0.2925755 -0.0149977 --zoom 1500 --size 320 --iter 1500 --ss 1 --method smooth --palette 2 --no-watermark` |
| multibrot3 | 0 | 0.000 | `63dbf0f65f007a5d` | ✅ match | `fractadyne --render --out multibrot3.png --fractal "Multibrot 3" --center 0.0 0.0 --zoom 0.8 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| multibrot4 | 0 | 0.000 | `fdfc9e196fb14a2d` | ✅ match | `fractadyne --render --out multibrot4.png --fractal "Multibrot 4" --center 0.0 0.0 --zoom 0.8 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| multibrot5 | 0 | 0.000 | `728fc000006513cd` | ✅ match | `fractadyne --render --out multibrot5.png --fractal "Multibrot 5" --center 0.0 0.0 --zoom 0.8 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| tricorn | 0 | 0.000 | `0b01d8cc19a5eb4d` | ✅ match | `fractadyne --render --out tricorn.png --fractal "Tricorn" --center 0.0 0.0 --zoom 0.8 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| burning-ship | 0 | 0.000 | `f46c1f1c2cbb8874` | ✅ match | `fractadyne --render --out burning-ship.png --fractal "Burning Ship" --center -0.5 -0.5 --zoom 0.7 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| celtic | 0 | 0.000 | `c7b9d4df52597b3d` | ✅ match | `fractadyne --render --out celtic.png --fractal "Celtic" --center -0.5 0.0 --zoom 0.8 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| buffalo | 0 | 0.000 | `1fc4cc68876c8ad6` | ✅ match | `fractadyne --render --out buffalo.png --fractal "Buffalo" --center -0.5 -0.5 --zoom 0.7 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| phoenix | 0 | 0.000 | `7877037c047b6fd1` | ✅ match | `fractadyne --render --out phoenix.png --fractal "Phoenix" --center 0.0 0.0 --zoom 0.7 --size 320 --iter 800 --ss 1 --method smooth --palette 0 --no-watermark` |
| newton | 0 | 0.000 | `bb305b3ab800e151` | ✅ match | `fractadyne --render --out newton.png --fractal "Newton" --center 0.0 0.0 --zoom 0.7 --size 320 --iter 400 --ss 1 --method smooth --palette 0 --no-watermark` |
| mandelbrot-1e6 | 0 | 0.000 | `7e5fd1f4203227a6` | ✅ match | `fractadyne --render --out mandelbrot-1e6.png --fractal "Mandelbrot" --center -7.219621882920463979621343199249635039400777157391994056859e-1 2.406540627640154659873781066416545013133592385797331352286e-1 --zoom 1000000 --size 320 --iter 3000 --ss 1 --method smooth --palette 0 --no-watermark` |
| multibrot3-1e6 | 0 | 0.000 | `6e8bf2f7ad4cf23e` | ✅ match | `fractadyne --render --out multibrot3-1e6.png --fractal "Multibrot 3" --center 2.19533102209775940218788168856401426185991366731348781648e-1 7.317770073659198278104833118192370226116695264984596408352e-1 --zoom 1000000 --size 320 --iter 3000 --ss 1 --method smooth --palette 0 --no-watermark` |
| multibrot4-1e6 | 0 | 0.000 | `ddd2489ed04de2ad` | ✅ match | `fractadyne --render --out multibrot4-1e6.png --fractal "Multibrot 4" --center 2.28757960884408080137002307307431367850187620104115769219e-1 7.625265362813602953424916065993043372187655480595946595141e-1 --zoom 1000000 --size 320 --iter 3000 --ss 1 --method smooth --palette 0 --no-watermark` |
| multibrot5-1e6 | 0 | 0.000 | `bad6fc4493cc5840` | ✅ match | `fractadyne --render --out multibrot5-1e6.png --fractal "Multibrot 5" --center 2.320768669674853369085651557338865001525750889159483426277e-1 7.735895565582844849904484291320284693154748744446630197764e-1 --zoom 1000000 --size 320 --iter 3000 --ss 1 --method smooth --palette 0 --no-watermark` |

**17/17 golden images within tolerance.**

## Summary

✅ ALL CHECKS PASSED
