# Diagnostics — how to see what Fractadyne is doing

One page for debugging and performance work. The design rationale and failure catalog live
in [design/diagnostics.md](design/diagnostics.md).

## Files written automatically

| File | What | When |
|------|------|------|
| `<config>/logs/fractadyne.log` | Every `[fd-*]` diagnostic line, timestamped `[+12.345s]`; session header with version/args | Always (disable: `FRACTADYNE_LOG=0`); rotates to `.log.1` past ~5 MB |
| `<config>/logs/crash-<unix>.txt` | Panic message, backtrace, current activity, last render manifest, version | On any panic (including wgpu uncaptured errors, which log then panic) |
| `<config>/logs/perf.jsonl` | One JSON record per export render: size/ss/mode/iterations, pure-GPU iterate+color ms, nominal Gsteps/s, event counters | Only with `FRACTADYNE_PERF=1` |

`<config>` is the session directory (`%APPDATA%\Fractadyne\Fractadyne\config` on Windows;
override with `FRACTADYNE_CONFIG_DIR`). **Note:** `--reset-state` deletes the whole config
dir, logs included.

## Environment variables

| Var | Values | Effect |
|-----|--------|--------|
| `FRACTADYNE_TRACE` | `1` (all) or `req,ref,gpu,tile,glitch` | Stderr + log-file tracing by category (below) |
| `FRACTADYNE_LOG` | `0` | Disables the log file (stderr unchanged) |
| `FRACTADYNE_PERF` | `1` | Appends per-render perf records to `logs/perf.jsonl` (regression tracking across builds) |
| `FRACTADYNE_CONFIG_DIR` | path | Relocates config dir (and therefore `logs/`) |
| `RUST_LOG` | env_logger spec | wgpu/naga internal logging (stderr) |

### Trace categories (`[fd-<cat>]` prefixes)

| Category | Fires | Tells you |
|----------|-------|-----------|
| `req` | every export-request build | The **effective** render manifest: mode, iterations, orbit length, SA skip, BLA, span, precision, size. The record that catches "rendered the wrong view" |
| `ref` | every reference build (fresh *and* reused/extended) | Orbit length/iterations/precision, escaped/partial, SA skip, BLA nodes, and the build-time split `orbit_ms`/`sa_ms`/`bla_ms` + `pick_reference` scoring ms — the cold deep-export cost lives in `pick_reference` (~9 s at me148), see TODO.md |
| `gpu` | live floatexp budget controller | Measured iterate ms per dispatch, budget grow/shrink, convergence |
| `tile` | live floatexp frame sizing | Per-frame resolution/ss/iterations/steps vs budget, reprojection, tiled-settle grid state |
| `glitch` | multi-reference correction | Per-run summary: references used, residual glitched px, elapsed |

Always-on prefixes (not gated): `[fd-start]` session header, `[fd-render]` CLI render
manifest + failures, `[fd-progress]` CLI render progress (~2 s cadence), `[fd-watch]`
possible-hang warnings, `[fd-wgpu]` device errors/loss, `[fd-panic]` crash reports,
`[fd-perf]` per-export GPU times + counters, `[selftest …ms]` streamed check results.

### GPU event counters

Every render reports shader event counts (in `[fd-perf]`, `perf.jsonl`, and
`ExportResult.counters`): **rebase** (Zhuoran rebases), **ext** (extended-range orbit
samples decoded), **glitch** (Pauldelbrot flags), **bla_skip** (BLA multi-steps),
**maxiter** (pixels that exhausted the budget). Totals are accumulated in **u64** across
all tiles (the GPU-side u32 slots are zeroed + read per tile), so a deep multi-tile export
does not wrap.

These count **main perturbation-loop events**, so they are legitimately *low* when series
approximation and BLA cover most of the work — a deep view with a large `sa_skip` can escape
in a handful of counted iterations (a fast `gpu_iterate` ms confirms it). To use them as
execution proof (e.g. "did the extended-range path fire?") disable SA/BLA so the main loop
runs, the way the selftest "counters" group does — otherwise a genuinely SA-dominated render
and a dead code path both read near-zero. With SA/BLA off, zero on a path a deep render must
exercise means dead code (exactly how the v0.2.6 NaN-marker regression would have shown).

## Reading common symptoms

- **App window "Not Responding" / closed by itself** → open `logs/fractadyne.log`. A crash
  leaves `[fd-panic]` + a `crash-*.txt`; a hang leaves `[fd-watch] possible hang: <activity>`
  every 30 s naming the wedged phase. Nothing at all = killed externally (driver TDR, OOM
  killer, user).
- **CLI render slow vs hung** → the `[fd-progress]` line updates every ~2 s while tiles
  finish; a frozen percentage + `[fd-watch]` lines = hung. `--render` now exits non-zero on
  failure (it used to exit 0 unconditionally).
- **A batch rendered the wrong thing** → check the un-gated `[fd-render]` manifest line
  (center/zoom/iterations/out) printed before each CLI render, or `[fd-req]` under trace.
- **Uniform/flat frame at depth** → check interior-vs-escaped first (compare against the
  session's interior color), then `FRACTADYNE_TRACE=ref` for orbit length and escape state.
- **Byte-identical output across a shader "fix"** → the changed code did not execute. Check
  the `[fd-perf]` counter line for the path's counter (ext/rebase/bla_skip/glitch): zero on
  a view that must exercise it = dead code (the v0.2.6 WGSL NaN lesson, F4).
- **Selftest wedged or slow** → the streamed `[selftest …ms]` line names the last completed
  check; the watchdog breadcrumb says `selftest: after '<check>'`.

## CLI validation & profiling flags

| Flag | What |
|------|------|
| `--selftest [--out report.md] [--bless]` | 83 checks + 17 goldens, streamed live; hermetic (resets config at entry, echoes it); GPU errors are printed, never silently skipped; data files resolve relative to the repo even when run elsewhere. The last 20 checks are the `bench-matrix` group — deterministic path-signature tripwires (see `--bench-matrix`) |
| `--selftest-filter <substr>` | Run only matching check groups / goldens (fast iteration on one failure; not a release verdict — groups share state) |
| `--selftest-list` | Print the group tags usable with `--selftest-filter` |
| `--profile [--regions file.toml] [--reps N]` | Per-region reference/SA/BLA build ms + pure-GPU pass ms (TIMESTAMP_QUERY); includes a corpus-14-class `deep-interior-1e148` region (dip orbit, 800k iters — the export-throughput-gap regime) |
| `--bench-matrix [--bless] [--reps N]` | Path-coverage perf + regression suite (zoom bands, fractals, coloring). Per-segment CPU-build vs GPU split + deterministic GPU event counters, compared against `benchmarks/bench-matrix-baseline.json`. Algorithmic drift → exit 2; timing regression → warn. `--bless` records the baseline. See [design/bench-matrix.md](design/bench-matrix.md) |
| `--frametest` | Stepped-dive stutter harness (build_ms stalls; its "gpu" column is CPU wall-clock — trust `--profile` for GPU numbers) |
| `--benchmark-std` | Standardized dive benchmark with report |
| `--render --out X …` | One-shot render; prints manifest + progress; non-zero exit on failure |

## Canonical extreme-zoom diagnostic location

The Mandelbrot **real-axis tip** (`c = -2` exactly) at **~1e21000×** (`units_per_pixel_e = -69770`,
~69769-bit working precision) is the project's canonical extreme-depth stress point — the deepest
routinely-exercised view. Two committed forms live in `validation/`:

- **`extreme-zoom-tip-e21000.fdn`** — the exact view. Load it in-app via *Share location →
  Load .fdn…* (or paste the text and Apply) to reproduce it. Its purpose is a **responsiveness /
  no-freeze** check: it regression-guards the v0.2.15 load-freeze fix (`LIVE_ITER_CAP`). Before
  that fix, auto-iter over-provisioned the *live preview* to 500k iterations and the app froze on
  load at this depth; now it boots responsive — the reference builds off-thread, so the first sharp
  frame is delayed by the reference cost below, but the UI never wedges and the watchdog stays silent.
- **`extreme-zoom.toml`** — a `--profile --regions` region for the same point, to quantify the cold
  reference cost. Run `fractadyne --profile --regions validation/extreme-zoom.toml --reps 1`.
  **Measured (3080 / 3950X): ~250 s wall for the cold build at 69828-bit precision** (mode 2 /
  floatexp). ⚠The `--profile` table's `ref ms` column reports only **~410 ms** — it times just the
  arbitrary-precision *orbit compute*; the ~99% remainder is `best_reference` **candidate scoring**
  (fractadyne-core — the throughput lever), which `--profile` does *not* attribute to that column.
  What exposes the true cost is the **watchdog breadcrumb** `building reference … [main]`, firing
  every 30 s through ~240 s. (That is the headless *main-thread* build; in the live app the same
  build runs off-thread, so the UI stays responsive and the watchdog stays silent — cf. the `.fdn`
  case above. This region thus doubles as a live demonstration of the `--profile` scoring blind spot.)

Deliberately **not** a selftest golden or an F3 corpus entry: a full render here is minutes
(bignum-bound), far too slow for the byte-identical goldens; and ~1e21000× is ~20× beyond
Fraktaler-3's demonstrated range (the deepest F3-matched corpus pair is 4.6e1105×), so there is no
F3 image to compare against. Its value is diagnostic (responsiveness + cost), not render-comparison.

## Orbit forensics (CPU probes, no GPU)

Env-gated tests in `crates/fractadyne-core/tests/`, built for the deep-zoom investigations:

```
PROBE_ORBIT="label|cx|cy|iters|prec"        cargo test -p fractadyne-core --test probe_orbit -- --nocapture
PROBE_ESCAPE="label|cx|cy|mag_log10|max_iter|prec"  cargo test -p fractadyne-core --test probe_escape -- --nocapture
```

- `probe_orbit` — stored-sample dynamic range, orbit-dip periods, extended-range marker
  counts vs the f64 truth (the tool that found the ~1e-71 dips flushing to zero).
- `probe_escape` — floatexp-perturbation escape times at 8 directions × 3 radii around a
  center: the oracle for sizing per-location iteration counts (corpus 14 → 800k, 15 → 1.6M).
