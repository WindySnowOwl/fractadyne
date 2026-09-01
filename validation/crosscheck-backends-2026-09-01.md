# Backend x Fraktaler-3 corpus cross-check

Date: 2026-09-01 - machine: RTX 3080 / 3950X, idle - geometry 1280x720, Fractadyne --ss 2 (4 samples/px), F3 subframes=4 (4 samples/px) - single timed run per cell, engines sequential.

- astro: fractadyne 0.2.41-beta.2 (build 2350) [astro-float]
- mpfr: fractadyne 0.2.41-beta.2 (build 2351) [astro-float, rug/MPFR 4.2.2 + GMP 6.3.0]
- f3: vendored 3.1 x86_64

| # | location | zoom | iters | astro s | mpfr s | f3 s | mpfr/astro | f3/astro | f3/mpfr | pixels |
|---|---|---|---|---|---|---|---|---|---|---|
| 01 | home | 1e0.1 | 512 | 1.3 | 1.2 | 1.8 | 0.96 | 1.40 | 1.46 | identical |
| 02 | seahorse-1e4 | 1e4.1 | 1500 | 1.2 | 1.2 | 2.8 | 1.00 | 2.34 | 2.35 | identical |
| 03 | seahorse-1e6 | 1e6.1 | 3000 | 1.3 | 1.3 | 8.8 | 1.00 | 6.58 | 6.55 | identical |
| 04 | seahorse-1e12 | 1e12.6 | 60000 | 1.3 | 1.3 | 3.7 | 1.01 | 2.84 | 2.83 | identical |
| 05 | dendrite-8e17 | 1e18.0 | 50000 | 1.2 | 1.1 | 2.8 | 0.98 | 2.43 | 2.47 | identical |
| 06 | seahorse-1e24 | 1e24.1 | 20000 | 2.6 | 3.4 | 10.9 | 1.32 | 4.28 | 3.24 | identical |
| 07 | deep-1e30 | 1e30.0 | 600008 | 1.5 | 1.5 | 2.5 | 0.99 | 1.70 | 1.72 | identical |
| 08 | deep-6.6e43 | 1e43.9 | 60000 | 1.2 | 1.2 | 3.0 | 1.01 | 2.50 | 2.47 | identical |
| 09 | deep-6.1e500 | 1e500.9 | 150000 | 3.4 | 2.6 | 553.6 | 0.77 | 164.09 | 213.10 | identical |
| 10 | deep-4.6e1105 | 1e1105.8 | 250000 | 15.4 | 12.1 | 249.0 | 0.79 | 16.20 | 20.55 | identical |
| 11 | deep-1.7e124 | 1e124.4 | 600000 | 2.1 | 1.7 | 4.2 | 0.85 | 2.06 | 2.43 | identical |
| 12 | deep-4.6e132 | 1e132.8 | 600000 | 1.9 | 1.7 | 4.3 | 0.89 | 2.20 | 2.46 | identical |
| 13 | deep-3.7e141 | 1e141.7 | 600000 | 7.7 | 6.0 | 4.3 | 0.79 | 0.56 | 0.71 | identical |
| 14 | deep-1.2e148 | 1e148.2 | 800000 | 15.3 | 11.5 | 13.3 | 0.75 | 0.87 | 1.16 | identical |
| 15 | deep-3.7e163 | 1e163.7 | 1600000 | 27.3 | 21.0 | 14.6 | 0.77 | 0.53 | 0.69 | identical |
| 16 | deep-2.1e250 | 1e250.5 | 600008 | 3.9 | 3.6 | 7.9 | 0.93 | 2.03 | 2.19 | identical |
| 17 | deep-4.2e275 | 1e275.7 | 600008 | 4.3 | 4.0 | 7.8 | 0.92 | 1.81 | 1.97 | identical |
| 18 | deep-4.1e508 | 1e508.7 | 600008 | 7.5 | 6.4 | 324.8 | 0.85 | 43.43 | 51.13 | identical |
| 19 | deep-1.3e726 | 1e726.2 | 600008 | 10.7 | 8.9 | 376.4 | 0.83 | 35.21 | 42.34 | identical |
| 20 | deep-1.2e1008 | 1e1008.2 | 600008 | 20.0 | 15.8 | 179.3 | 0.79 | 8.94 | 11.36 | identical |
| 21 | m43-spar-1e27.7 | 1e27.7 | 30000 | 1.1 | 1.0 | 1.7 | 0.98 | 1.64 | 1.67 | identical |
| 22 | m43-spar-1e28.2 | 1e28.2 | 30000 | 1.0 | 1.0 | 1.8 | 0.99 | 1.75 | 1.77 | identical |
| 23 | nucleus-p145-1e27.7 | 1e27.7 | 30000 | 1.1 | 1.1 | 1.8 | 1.03 | 1.65 | 1.60 | identical |
| 24 | nucleus-p148-1e28.2 | 1e28.2 | 30000 | 1.1 | 1.1 | 1.8 | 1.02 | 1.61 | 1.58 | identical |
| 25 | misiurewicz-2-1 | 1e0.9 | 2000 | 1.0 | 1.0 | 1.5 | 1.03 | 1.50 | 1.46 | identical |
| 26 | misiurewicz-2-2 | 1e0.9 | 2000 | 1.0 | 1.0 | 1.8 | 0.99 | 1.77 | 1.78 | identical |
| 27 | misiurewicz-4-1 | 1e5.0 | 10000 | 1.1 | 1.1 | 1.8 | 1.00 | 1.60 | 1.60 | identical |
| 28 | misiurewicz-7-1 | 1e4.0 | 10000 | 1.1 | 1.1 | 2.0 | 0.99 | 1.83 | 1.85 | identical |
| 29 | nucleus-p2-basilica | 1e0.6 | 3000 | 1.1 | 1.1 | 2.2 | 1.02 | 1.98 | 1.93 | identical |
| 30 | nucleus-p3-airplane | 1e2.5 | 3000 | 1.1 | 1.1 | 2.3 | 1.02 | 2.05 | 2.01 | identical |
| 31 | nucleus-p3-rabbit | 1e1.3 | 3000 | 1.1 | 1.1 | 3.0 | 1.02 | 2.72 | 2.67 | identical |
| 32 | nucleus-p4 | 1e1.8 | 4000 | 1.1 | 1.1 | 3.8 | 1.00 | 3.48 | 3.47 | identical |
| 33 | nucleus-p5 | 1e3.0 | 5000 | 1.1 | 1.1 | 4.4 | 1.02 | 4.03 | 3.96 | identical |
| 34 | vger-dive-2p06e28 | 1e28.0 | 300000 | 1.7 | 1.9 | 6.4 | 1.09 | 3.74 | 3.42 | identical |
| 35 | vger-dive-1p47e77 | 1e77.2 | 300000 | 2.2 | 2.0 | 6.0 | 0.90 | 2.69 | 2.99 | identical |
| 36 | vger-dive-1p08e104 | 1e104.0 | 300000 | 3.0 | 2.7 | 6.7 | 0.90 | 2.22 | 2.45 | identical |
| 37 | vger-dive-3p11e114 | 1e114.5 | 300000 | 3.4 | 3.0 | 6.3 | 0.87 | 1.83 | 2.09 | identical |
| 38 | vger-dive-2p87e140 | 1e140.5 | 300000 | 2.5 | 2.1 | 5.9 | 0.84 | 2.36 | 2.80 | identical |

- **direct (df32 in-shader)** (n=8): mpfr/astro 1.01, f3/astro 2.21, f3/mpfr 2.19
- **perturbation, df32 delta (mode 0) + SA** (n=9): mpfr/astro 1.03, f3/astro 2.49, f3/mpfr 2.41
- **perturbation, floatexp delta (mode 2) + BLA** (n=21): mpfr/astro 0.88, f3/astro 3.44, f3/mpfr 3.91

Totals: astro 157.8s, mpfr 133.5s, f3 1837.1s (F3 over 38 of 38 locations)

PIXELS: all locations byte-identical across astro/mpfr and vs the blessed corpus
