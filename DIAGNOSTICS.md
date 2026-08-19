# Diagnostics — how to see what Fractadyne is doing

One page for debugging and performance work. The design rationale and failure catalog live
in [design/diagnostics.md](design/diagnostics.md).

## Files written automatically

| File | What | When |
|------|------|------|
| `<config>/logs/fractadyne.log` | Every `[fd-*]` diagnostic line, timestamped `[+12.345s]`; session header with version/args | Always (disable: `FRACTADYNE_LOG=0`); rotates to `.log.1` past ~5 MB |
| `<config>/logs/crash-<unix>.txt` | Panic message, backtrace, current activity, last render manifest, version | On any panic (including wgpu uncaptured errors, which log then panic). The manifest covers **both** paths: `req`-style for export/offline frames and a `LIVE …` line for on-screen frames stating resolution, ss, iterations, boost, nominal steps vs the watchdog budget, tiling, orbit length/partial and settled-ness — a live device loss used to record an EMPTY manifest, which made that crash class diagnosable only by inference |
| `<config>/logs/perf.jsonl` | One JSON record per export render: size/ss/mode/iterations, pure-GPU iterate+color ms, nominal Gsteps/s, event counters | Only with `FRACTADYNE_PERF=1` |

`<config>` is the session directory (`%APPDATA%\Fractadyne\Fractadyne\config` on Windows;
override with `FRACTADYNE_CONFIG_DIR`). **Note:** `--reset-state` deletes the whole config
dir, logs included.

## Environment variables

| Var | Values | Effect |
|-----|--------|--------|
| `FRACTADYNE_TRACE` | `1` (all) or `req,ref,gpu,tile,glitch` | Stderr + log-file tracing by category (below) |
| `FRACTADYNE_LOG` | `0` | Disables the log file (stderr unchanged) |
| `FRACTADYNE_PERF` | `1` | Appends per-render perf records to `logs/perf.jsonl` (regression tracking across builds) — plus, during script playback, one `kind:"live"` record per frame (tour time, depth, frame/cpu ms, pipeline lag) for live-judder analysis |
| `FRACTADYNE_CONFIG_DIR` | path | Relocates config dir (and therefore `logs/`) |
| `FRACTADYNE_NO_TIMESTAMPS` | `1` | Decline `TIMESTAMP_QUERY` even where the adapter offers it — the only way to exercise the no-timestamp frame-budget path on a GPU that has it. That path had a reproducible bug (budget stuck at the bootstrap ⇒ ~1/3 resolution forever) that was invisible on the dev 3080 for exactly that reason; older Intel iGPUs, some Mesa/RADV/ANV combinations and the GL backend all land on it for real. Expect `capability: TIMESTAMP_QUERY=false` in the log, then `pricing frames by wall clock` |
| `FRACTADYNE_DIVETEST_WINDOWS` | `"300,700"` | `--divetest`: override the default every-100-decades window sweep (targeted bands) |
| `FRACTADYNE_DIVETEST_SESSION_RES` | `1` | `--divetest`: keep the session's `min_motion_res` instead of pinning the 0.30 default (user-repro runs) |
| `FRACTADYNE_LIVETEST_SESSION_RES` | `1` | `--livetest`: same, for the live-output harness |
| `FRACTADYNE_NO_PREFETCH` | `1` | Disable script-playback reference prefetching (both the dive lookahead and the hold prefetch), so a tour is served by the REACTIVE rebuild path alone — the path a GUI user parked at a deep view has, since no script tells the app where the camera is going. ⭐This is what made the e72/e82 reference family measurable: with prefetching on, a hold's verdict is a race between the prefetch install and the checkpoint sample, so the gate flips on any recompile; with it off the same run is deterministic and the defect is in the open (it found a motion-time rebuild TRUNCATING a 1,208,193-sample reference to 256,001 and blacking out the next hold). Start here on any reference-lifecycle bug |
| `FRACTADYNE_FAKE_VERSION` | semver | Pretend the running build is that version — exercises the "update available" path (CLI + the in-app prompt) while current |
| `RUST_LOG` | env_logger spec | wgpu/naga internal logging (stderr) |

### Trace categories (`[fd-<cat>]` prefixes)

| Category | Fires | Tells you |
|----------|-------|-----------|
| `req` | every export-request build | The **effective** render manifest: mode, iterations, orbit length, SA skip, BLA, span, precision, size. The record that catches "rendered the wrong view" |
| `ref` | every reference build (fresh *and* reused/extended) | Orbit length/iterations/precision, escaped/partial, SA skip, BLA nodes, and the build-time split `orbit_ms`/`sa_ms`/`bla_ms` + `pick_reference` scoring ms (scoring is parallel across all cores since 0.2.40-beta.7 — ~0.6 s at 1e1216× where it was ~7.6 s); also `lookahead install:` lines when a playback-prefetched reference installs. Every build line is tagged with its ORIGIN — `[live]` (reactive), `[lookahead]`, `[hold]`, `[export]`, `[test]` — without which a log of four concurrent builders cannot be read at all (two e72 root-cause attempts died on exactly that ambiguity). Reactive spawns also log `interacting`, `cap_now`, the installed `orbit_len`, and which of the three triggers (`out_of_view`/`needs_quality`/`bla_out_of_range`) fired; during playback, `holding=` transitions mark each hold's boundary |
| `gpu` | live floatexp budget controller | Measured iterate ms per dispatch, budget grow/shrink, convergence; `aimd:` lines show the motion-res controller's real-frame cost signal + resolution decisions |
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
**maxiter** (pixels that exhausted the budget). Slots 5/6 carry the frame's escaped
smooth-iter **min/max** (f32 bits) — the LIVE path reads maxiter + range back per
settled full frame to drive the **adaptive iteration budget** (`[fd-gpu] adaptive
iter:` trace) and **live palette normalization** (the "Normalize deep colors" toggle). Totals are accumulated in **u64** across
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
| `--selftest [--out report.md] [--bless]` | 113 checks + 17 goldens, streamed live; hermetic (resets config at entry, echoes it); GPU errors are printed, never silently skipped; data files resolve relative to the repo even when run elsewhere. The last 20 checks are the `bench-matrix` group — deterministic path-signature tripwires (see `--bench-matrix`) |
| `--selftest-filter <substr>` | Run only matching check groups / goldens (fast iteration on one failure; not a release verdict — groups share state) |
| `--selftest-list` | Print the group tags usable with `--selftest-filter` |
| `--profile [--regions file.toml] [--reps N]` | Per-region reference/SA/BLA build ms + pure-GPU pass ms (TIMESTAMP_QUERY); includes a corpus-14-class `deep-interior-1e148` region (dip orbit, 800k iters — the export-throughput-gap regime) |
| `--bench-matrix [--bless] [--reps N]` | Path-coverage perf + regression suite (zoom bands, fractals, coloring). Per-segment CPU-build vs GPU split + deterministic GPU event counters, compared against `benchmarks/bench-matrix-baseline.json`. Algorithmic drift → exit 2; timing regression → warn. `--bless` records the baseline. See [design/bench-matrix.md](design/bench-matrix.md) |
| `--divetest tour.toml [--out log.json]` | Headless live-dive perf harness: plays real-time 18 s windows of a tour at every 100 decades of depth through the ACTUAL playback machinery (pacer, lookahead, reuse-hold, motion-res controller) with real GPU iterates, vsync-paced. Per band: fps, p50/p95/max frame ms, >33 ms hitches, real-refresh rate/cost (CPU vs pure-GPU), reference installs, pacer engagement, achieved oct/s. The dive-smoothness regression harness — diff the JSON across builds |
| `--livetest tour.toml [--segment NAME] [--size WxH] [--out DIR] [--quick]` | Headless live-OUTPUT harness: plays a tour through the SAME live machinery `--divetest` drives, but keeps the pixels and, at every keyframe hold, renders that view through the offline path as an oracle. Enforces the contract *the live view should show what an offline render of the same view at the same iteration budget shows*: reports excess black % and sRGB difference per checkpoint with the context that attributes it (budget vs appetite, boost, orbit length + PARTIAL flag, motion resolution, staleness), dumps live/truth PNG pairs for failures, exits 1 if any checkpoint fails. This is the harness that caught the live view rendering 100% black at 1e61–1e82x where the offline render is 0% black (beta.35). `--quick` skips the oracle (metrics + context only). **Graded against a blessed baseline** (`benchmarks/livetest-<tour>-<W>x<H>.json`, written by `--bless`): a run passes when every checkpoint matches what was recorded, INCLUDING recorded FAILs — the tour's deep holds fail for a known reason (the `LIVE_REF_CAP` pixel clamp), and a gate that stays red on a known problem cannot report a new one. Without a baseline it falls back to grading raw FAILs |
| `--play validation/deep-dive-crash.toml` | Focused diagnostic tour for the `orbit_len=626` live device-loss class: reaches the precondition state (626-sample escaped reference vs a ~27k pixel budget) in ~120 s instead of the grand tour's ~205 s. Reproduces the STATE, not yet the crash — its header records the negative runs, read it first |
| `--play tour.toml` | Start the GUI with a tour already playing in the LIVE view. The only way to drive on-screen playback — present, watchdog budget, settle ramp, tiled settle — from a command line; every other tour entry point is headless or offscreen. This is what reproduced the beta.36 device loss in 29 s, and what verified the fix |
| `--autodive [LOG10] [--autodive-timeout SECS]` | **UNPACED frame-cost controller hammer.** Drives the auto-zoom autopilot from the CLI with auto-iter on, so frames go out as fast as they complete with no tour clock to dilate the pressure away. Reports deepest depth, controller readings, peak measured iterate and lethal count. **Exit 0 = a lethal reading occurred (the experiment ran); exit 2 = it did not, so nothing was tested — never read that as a pass.** Use this, not `--play`, to chase device-loss/TDR behaviour: a tour dilates its clock on a slow frame, and measured on a 3080 `repro-e28-crossover` peaks at ~195 ms against a 900 ms lethal band |
| `--motiontest` | **Motion-PRESENTATION gate for chunked deep views** (design/mode2-chunking.md §11). Self-contained: jumps to corpus loc 07 at 1.3e31× (mode 2) with an explicit 1M ask, waits for the reference, then drives a 6 s wheel-style dive and a full Home glide while asserting invariants over the adoption counters: a partial chunk progression is never adopted as the frozen texture (A1 — the "interior looks like noise" regression `--livetest` cannot see, because its checkpoints measure settled results), complete refreshes keep streaming during motion (A2 — the anti-freeze half), and no frame displays a texture that diverged from the frozen bookkeeping (A3). Fails as VACUOUS if the run never produced interacting chunk-eligible frames. Exit 0 pass / 2 assert-fail (never a pass) / 4 watchdog. ~1–3 min; run with a wiped `FRACTADYNE_CONFIG_DIR` |
| `--frametest [--center X Y]` | Stepped-dive stutter harness (build_ms stalls; its "gpu" column is CPU wall-clock — trust `--profile`/`--divetest` for GPU numbers). `--center` dives a real deep line instead of the 34-digit seahorse (precision-noise past ~1e34×) |
| `--benchmark-std` | Standardized dive benchmark with report |
| `--render --out X …` | One-shot render; prints manifest + progress; non-zero exit on failure |
| `--set NAME=VALUE` | Override one frame-cost tunable **for this run** (repeatable). See below |

## Moving a tunable for one run (`--set`)

Every critical number lives in [`crates/fractadyne-app/src/tunables.rs`](crates/fractadyne-app/src/tunables.rs),
each with its unit and the incident that set it. Twelve of them — the frame-cost controller family
that every device loss in this project involved — can be overridden from the command line, so a
field diagnosis can answer *"does this still reproduce at a 400 ms target?"* without a rebuild:

```
fractadyne --set TDR_EXPLICIT_BUDGET_MS=200 --set TDR_MAX_TILES=64 --play tours/grand-tour.toml
```

`TDR_BUDGET_MS`, `TDR_EXPLICIT_BUDGET_MS`, `TDR_LATENCY_ACCEPT_MS`, `TDR_GROW_MAX`,
`TDR_SHRINK_MAX`, `TDR_BOOTSTRAP_STEPS`, `TDR_MIN_STEPS`, `TDR_STEPS_CEIL`, `EXPLICIT_STEPS_CEIL`,
`EXPLICIT_DISPATCH_CAP`, `TDR_MAX_TILES`, `TDR_TILES_CEIL`.

- **Not a configuration surface.** The defaults are the only tested path: the self-test, the
  goldens, `--bench-matrix` and `--livetest` all assume them. `--selftest` carries a check that
  FAILS when any override is in effect, so an overridden run can never be quoted as a clean verdict.
- **Loud and traceable.** Overrides are logged at startup (`⚠TUNABLES 2 OVERRIDE(S) — …`) and
  stamped into every crash report (`tunables:` line, which reads `stock` otherwise) — a report from
  an overridden run cannot masquerade as stock behaviour.
- **Never a silent no-op.** An unknown name, a non-numeric or non-positive value, or a pair that
  would invert a floor and its ceiling is a fatal startup error.
- ⚠**Dangerous values are permitted on purpose** — raising a budget until the device is lost is a
  legitimate experiment, and the ~0.9 s lethal band is reachable from here. Nothing clamps you.

## Validating a new machine / GPU (the B6 battery)

Run **one** command on the machine under test; it produces a single bundle to send back.

```powershell
# Windows                                    # Linux (works over bare SSH)
.\scripts\gpu-validate.ps1 -Label rx6800xt-windows    ./scripts/gpu-validate.sh --label rx6800xt-linux
.\scripts\gpu-validate.ps1 -Label foo -Quick          ./scripts/gpu-validate.sh --label foo --quick
.\scripts\gpu-validate.ps1 -Label foo -Backend dx12   ./scripts/gpu-validate.sh --label foo --backend gl
```

Both scripts run the same six steps in the same order and write the same file names, so two
machines' bundles diff directly: `--gputest` (arithmetic per backend), `--selftest` (suite +
goldens), `--selftest-filter live-res` (the settled-resolution invariant), `--bench-matrix`
(determinism), `--livetest` (live vs offline truth) and `--uitest` (screenshots). `-Quick` drops
the last two, taking the run from ~15 minutes to ~3. They find the binary beside themselves
(extracted release zip) or in `target/release`, so testers need no repo and no toolchain.

Three properties worth preserving if you edit them:

- **Hermetic.** Everything runs against a private config dir inside the bundle
  (`FRACTADYNE_CONFIG_DIR`), so the tester's own session is untouched *and* every machine renders
  with identical settings. Without this, results are not comparable — the F3 corpus check used to
  inherit the developer's live session, and its baselines drifted into meaninglessness as a result
  (fixed 2026-08-14 the same way: a committed session template copied into a throwaway
  `FRACTADYNE_CONFIG_DIR`; `--check` is 20/20 again). The app now logs which session it loaded —
  `[fd-start] session: <path> — loaded / none (defaults) / UNREADABLE, ignored (defaults)` — so a
  harness can PROVE its staging took effect instead of assuming it.
- **A failing step never aborts the battery.** A card can fail the goldens and still pass the
  live-resolution check; you want both. Hence `set -uo pipefail` (not `-e`) and
  `$ErrorActionPreference = "Continue"` — with `Stop`, the app's stderr banner alone kills the run.
- **ASCII-only in the `.ps1`.** Windows PowerShell 5.1 reads scripts as ANSI unless they carry a
  UTF-8 BOM, so an em-dash becomes a parse error on a stranger's machine.

**Reading the results** — `summary.txt` leads with the step/exit/duration table and then explains
which failures are expected off the reference card. The essentials: `live-res` must pass
everywhere; the 113 non-golden checks should pass everywhere; the 17 goldens are compared
*exactly* and were blessed on an RTX 3080, so cross-vendor deltas there are expected rather than
bugs (judge by count and magnitude); `bench-matrix` timings are meaningless across machines but
exit 2 means algorithmic drift; `livetest` compares live against offline *on that machine*, so its
FAILs are meaningful even on unfamiliar hardware while its "drift" lines are not.
`adapter.txt` records what the app itself resolved — adapter, backend, `TIMESTAMP_QUERY` — which
is the "record adapter and resolved tunables per card" half of B6.

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
