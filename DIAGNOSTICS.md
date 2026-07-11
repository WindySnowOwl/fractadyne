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
| `ref` | every reference build (fresh *and* reused/extended) | Orbit length/iterations/precision, escaped/partial, SA skip, BLA nodes |
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
**maxiter** (pixels that exhausted the budget). They are execution proof: a deep render
whose `ext`/`rebase` counters are zero is running with those paths dead (exactly how the
v0.2.6 NaN-marker regression would have been caught in one render). The selftest
"counters" group asserts them on known views.

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
| `--selftest [--out report.md] [--bless]` | 61 checks + 17 goldens, streamed live; hermetic (resets config at entry, echoes it); GPU errors are printed, never silently skipped; data files resolve relative to the repo even when run elsewhere |
| `--selftest-filter <substr>` | Run only matching check groups / goldens (fast iteration on one failure; not a release verdict — groups share state) |
| `--selftest-list` | Print the group tags usable with `--selftest-filter` |
| `--profile [--regions file.toml] [--reps N]` | Per-region reference/SA/BLA build ms + pure-GPU pass ms (TIMESTAMP_QUERY); includes a corpus-14-class `deep-interior-1e148` region (dip orbit, 800k iters — the export-throughput-gap regime) |
| `--frametest` | Stepped-dive stutter harness (build_ms stalls; its "gpu" column is CPU wall-clock — trust `--profile` for GPU numbers) |
| `--benchmark-std` | Standardized dive benchmark with report |
| `--render --out X …` | One-shot render; prints manifest + progress; non-zero exit on failure |

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
