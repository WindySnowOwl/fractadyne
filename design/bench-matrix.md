# Path-matrix benchmark (`--bench-matrix`)

A path-coverage performance **and** regression suite. Where the standardized benchmark
(`--benchmark-std`, `Tools → Benchmark → Standardized`) gives one comparable score from a single
Seahorse dive to 1e12×, this suite runs a **matrix** of short renders, each pinned to exercise one
rendering path, and reports enough per-segment detail to (a) tune performance and (b) catch
regressions in any build that touches the rendering pipeline.

## Why it exists

The standardized dive spends almost all its frames in the shallow-to-mid regime and measures GPU
render time only. It barely touches the deep floatexp / BLA / Zhuoran-rebasing machinery, never
varies fractal or coloring, and hides the cold bignum **reference build** — which our own profiling
shows dominates deep renders (`best_reference` scoring). So a slowdown or an algorithmic change in a
deep path could ship invisibly. The matrix makes every path a first-class, individually-measured,
individually-regression-checked segment.

## What each segment reports

| field | source | nature |
|---|---|---|
| `mode` | `RenderMode::select` | deterministic — 1 = f64 direct, 0 = df32 perturbation, 2 = floatexp |
| `eff-it` / `sa-skip` / `orbit_len` | the built `ExportRequest` | deterministic |
| `counters` = rebase / ext / bla-skip / maxiter | GPU event counters (`CTR_*`) | **deterministic** — same math ⇒ same counts |
| `ref` / `series` / `bla` ms | `ProfSetup` (CPU bignum setup) | hardware-dependent |
| `gpu` ms (+ pass split via `TIMESTAMP_QUERY`) | wall-clock + GPU timestamps | hardware-dependent |

The **deterministic** fields are the machine-independent signal: they encode exactly which path ran
and how hard, and are reproducible regardless of GPU speed. The **timings** are only comparable on
the same hardware.

## The matrix

Defined in [`crates/fractadyne-app/src/bench_matrix.rs`](../crates/fractadyne-app/src/bench_matrix.rs)
`matrix()`. Three groups:

- **zoom-band** — a Mandelbrot sweep across the numeric regimes plus accelerator variants:
  `direct-1e2` (f64), `df32-1e8`, `df32-1e20`, `floatexp-1e30-{sa,nosa,nobla}` (isolates SA vs BLA),
  `deep-interior-1e148` (extended-range samples + heavy rebasing — the pathology regime), and
  `floatexp-1e300` (a ~330-digit bignum reference — the cold-reference-build lever).
- **coloring** — Mandelbrot @1e20× under `smooth` vs the iteration-skip-blocking methods
  (`stripe`, `trap`, `decomposition`), which drop `sa-skip` to 0 and cost ~2× on the GPU.
- **fractal** — every family's home view (direct path; a mix of interior / boundary / exterior).

Glitch correction is deliberately excluded (data-dependent cost + the known deep-interior multi-ref
pathology), so every segment is a single deterministic dispatch.

Segments flagged `deterministic: true` are fast and stable enough to run as a selftest sanity check;
the two heavy deep bands (`1e148`, `1e300`) are `--bench-matrix`-only.

## Regression model — two tiers

1. **Algorithmic (machine-independent) — hard fail.** Any change to a segment's deterministic
   signature (mode / skip / orbit-len / eff-iter / counters) means a rendering path changed its
   executed work. `--bench-matrix` exits **2**; the selftest check FAILS. If intended, re-bless.
2. **Performance (same-GPU) — soft warn.** A timing above `baseline × 1.35` (and ≥ 1 ms) warns but
   does not fail. Generous, because run-to-run variance on the small CPU bignum builds is real.

Baseline: `benchmarks/bench-matrix-baseline.json` (committed). `--bench-matrix --bless` records it;
a plain `--bench-matrix` compares against it. A different GPU skips the timing comparison (noted in
the output) but still checks the deterministic signatures.

## Usage

```
fractadyne --bench-matrix              # run the full matrix, compare vs baseline
fractadyne --bench-matrix --reps 8     # more timed reps per segment (median; default 5)
fractadyne --bench-matrix --bless      # re-record the baseline (after an intended change)
```

The 20 deterministic segments also run inside `--selftest` (group tag `bench-matrix`, part of the
83-check suite) — so any build that touches the rendering pipeline trips the algorithmic tripwire as
part of normal validation. CI's GPU-less runners can't run it; it belongs to the dev-machine GPU
validation gate, same as the golden images.

## Caveats

- Timings are **not** cross-machine comparable; only the deterministic signatures are.
- The committed baseline was blessed on the maintainer's GPU (RTX 3080). On a different GPU the
  deterministic counters *should* still match (same math), exactly as the golden images do; if a
  vendor's floating-point differs at an escape margin, re-bless locally.
