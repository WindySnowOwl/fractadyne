# Diagnostics — how to see what Fractadyne is doing

One page for debugging and performance work. The design rationale and failure catalog live
in [design/diagnostics.md](design/diagnostics.md).

## Files written automatically

| File | What | When |
|------|------|------|
| `<config>/logs/fractadyne.log` | Every `[fd-*]` diagnostic line, timestamped `[+12.345s]`; session header with version/args | Always (disable: `FRACTADYNE_LOG=0`); past ~5 MB rotates into `.log.1`–`.log.3`, checked while running (beta.113) as well as at startup |
| `<config>/logs/crash-<unix>.txt` | Panic message, backtrace, current activity, last render manifest, version | On any panic (including wgpu uncaptured errors, which log then panic). The manifest covers **both** paths: `req`-style for export/offline frames and a `LIVE …` line for on-screen frames stating resolution, ss, iterations, boost, nominal steps vs the watchdog budget, tiling, orbit length/partial and settled-ness — a live device loss used to record an EMPTY manifest, which made that crash class diagnosable only by inference |
| `<config>/logs/perf.jsonl` | One JSON record per export render: size/ss/mode/iterations, pure-GPU iterate+color ms, nominal Gsteps/s, event counters | Only with `FRACTADYNE_PERF=1` |
| `<config>/logs/frames.bin` | **The frame record**: the last 4,096 frames, one fixed 512-byte slot each (see below). Written in place with no flush, so it survives a process abort that the panic hook never sees | Always, from the first live frame (beta.113). Truncated at the start of each session, after the previous one's has been read |
| `<config>/logs/crash-<unix>-<n>-frames.jsonl` | Every frame record the crashing process held, as JSON Lines — a header line, then one record per line | Beside every crash report that had frames to write, including the report for a previous session that died without one |
| `<config>/logs/frames.jsonl` (+ `.1`–`.3`) | **The long-horizon log**: a header line, one compact `"kind":"summary"` row per second (frames, mean/max interval, slow frames, readings by verdict, what was presented, events written and dropped), and a full frame row for each EVENT — a slow frame, a lethal reading or a watchdog stall (up to 20/s), a budget move, growth refusal or mode switch (up to 2/s). ~13 KB for an 18 s run. Written by its own thread (beta.114), so a slow disk or a network share never delays a frame; rows reach the OS within milliseconds, and a clean exit or a panic waits briefly for the last ones | Always. The previous session's file rotates to `.1` at startup, and a file past 32 MB rotates while running |

`<config>` is the session directory (`%APPDATA%\Fractadyne\Fractadyne\config` on Windows;
override with `FRACTADYNE_CONFIG_DIR`). **Note:** `--reset-state` deletes the whole config
dir, logs included.

## Environment variables

| Var | Values | Effect |
|-----|--------|--------|
| `FRACTADYNE_TRACE` | `1` (all) or `req,ref,gpu,tile,glitch,idle,dpi,autopilot` | Stderr + log-file tracing by category (below) |
| `FRACTADYNE_LOG` | `0` | Disables the log file (stderr unchanged) |
| `FRACTADYNE_PERF` | `1` | Appends per-render perf records to `logs/perf.jsonl` (regression tracking across builds) — plus, during script playback, one `kind:"live"` record per frame (tour time, depth, frame/cpu ms, pipeline lag) for live-judder analysis |
| `FRACTADYNE_CONFIG_DIR` | path | Relocates config dir (and therefore `logs/`) |
| `FRACTADYNE_BIGNUM` | `auto` / `astro` / `rug` | Which arbitrary-precision backend iterates the reference orbit (same as `--bignum`, which outranks it). **Use the variable, not the flag, for any batch or gate run**: harnesses launch fractadyne as CHILD processes, so a flag on the parent never reaches them. Asking for a backend this build does not contain is a FATAL error, never a quiet fall back — a silent downgrade would let a benchmark report numbers for arithmetic it never ran. `rug` exists only in the optional accelerated build |
| `FRACTADYNE_NO_SOUND` | `1` | Silences the render-finish tone. Same child-process reasoning as above; an explicit `--sound` outranks it without unsetting it |
| `FRACTADYNE_NO_TIMESTAMPS` | `1` | Decline `TIMESTAMP_QUERY` even where the adapter offers it — the only way to exercise the no-timestamp frame-budget path on a GPU that has it. That path had a reproducible bug (budget stuck at the bootstrap ⇒ ~1/3 resolution forever) that was invisible on the dev 3080 for exactly that reason; older Intel iGPUs, some Mesa/RADV/ANV combinations and the GL backend all land on it for real. Expect `capability: TIMESTAMP_QUERY=false` in the log, then `pricing frames by wall clock` |
| `FRACTADYNE_DIVETEST_WINDOWS` | `"300,700"` | `--divetest`: override the default every-100-decades window sweep (targeted bands) |
| `FRACTADYNE_DIVETEST_SESSION_RES` | `1` | `--divetest`: keep the session's `min_motion_res` instead of pinning the 0.30 default (user-repro runs) |
| `FRACTADYNE_NO_ACCUM` | `1` | Disable progressive on-settle supersampling (the deep-zoom despeckle: once a view ≥ ~1e10× settles, sub-pixel-jittered re-renders are folded into a running average until `FRACTADYNE_ACCUM_TARGET` samples). Runs in the single and dual live views; off under every task invocation except `--shot`, whose capture gate waits for convergence. A fold is only taken for a real, un-held, un-pinned frame with no grid/chunk/reference build in flight, and the average restarts on any colouring change (palette, cycle/offset, method, effects, normalization, AA) |
| `FRACTADYNE_ACCUM_TARGET` | `24` | Samples to fold before the view quiesces (clamped 2..=256). Each is a full deep re-render, so this trades convergence against idle GPU load |
| `FRACTADYNE_DIVETEST_GLIDE` | `1.0` | `--divetest`: drive each window as an INTERACTIVE hold-Space glide at that zoom-rate slider value (about the tour's final centre, through the GUI's own pacer and the interactive reference lookahead) instead of the tour camera — the tour only supplies the window start views. Pair with `FRACTADYNE_NO_PREFETCH=1` for the A/B baseline; a run whose `look` column is 0 has not exercised the lookahead |
| `FRACTADYNE_ZOOMTEST_SETTLE` | `out.png` | `--zoomtest`: after the glide, release the key, let the view settle and the on-settle supersampling converge, and capture the window twice — `out-s0.png` with one sample folded (the classic settled frame) and `out.png` converged. The two are the SAME view; their blurred difference is the de-speckle change plus any ghost (a sample folded at another view survives the blur, speckle does not). Accumulation, off under every other task invocation, is admitted for this run |
| `FRACTADYNE_ZOOMTEST_TAPS` | `6,1.0,2.5` | `--zoomtest`: `n` short glides of that many octaves with that many seconds of rest between them instead of one continuous glide — the wheel-tap pattern, where the accumulation begins at every rest and the next tap interrupts it. The last release goes straight to the settle tail when one is configured |
| `FRACTADYNE_ZOOMTEST_PROFILE` | `user` | `--zoomtest`: apply the 2026-09-16 field-report settings (prefer detail while zooming, 2× AA, motion-resolution floor 0.83, live normalization, the dual view at split 0.588) so a report under those settings is reproduced under them. With `FRACTADYNE_NO_ACCUM=1` a settle tail captures the quiet single render instead of a converged average |
| `FRACTADYNE_LIVETEST_SESSION_RES` | `1` | `--livetest`: same, for the live-output harness |
| `FRACTADYNE_NO_PREFETCH` | `1` | Disable script-playback reference prefetching (both the dive lookahead and the hold prefetch), so a tour is served by the REACTIVE rebuild path alone — the path a GUI user parked at a deep view has, since no script tells the app where the camera is going. ⭐This is what made the e72/e82 reference family measurable: with prefetching on, a hold's verdict is a race between the prefetch install and the checkpoint sample, so the gate flips on any recompile; with it off the same run is deterministic and the defect is in the open (it found a motion-time rebuild TRUNCATING a 1,208,193-sample reference to 256,001 and blacking out the next hold). Start here on any reference-lifecycle bug |
| `FRACTADYNE_AUTOPILOT_DUMP` | directory | Write every autopilot steering probe there as raw little-endian `f32` (`probe-NNNNN.f32`, `w×h×4`: smooth iteration, normal x/y, distance-estimate log₂ in cells; the dims are the `n=` of the matching `autopilot` trace line). Pairs with `FRACTADYNE_TRACE=autopilot` and `--import-kfr LOC --autodive L --autodive-iter 0 --autodive-home 0` to replay a user's auto-zoom from their view: the 2026-09-19 "zooms into a flat region" report was measured this way (every pick was detail; the pivot averaged picks from opposite corners into the flat gap between them) |
| `FRACTADYNE_FAKE_VERSION` | semver | Pretend the running build is that version — exercises the "update available" path (CLI + the in-app prompt) while current |
| `FRACTADYNE_SHARE` | path, e.g. `\\host\share\Fractadyne` or `/mnt/share/Fractadyne` | The team file share the validation scripts and the field agent use: `gpu-validate` delivers its bundle there and looks there for `BUILD-ID.txt`, `publish-share.ps1`, `field-request.ps1` and `field-agent-setup.ps1` default to it, and `--uitest` stages its screenshots under `<share>/uitest`. Unset = none of that (bundles stay beside the script, `-Share`/`-Out` must be given) |
| `FRACTADYNE_REF_ESCAPE_AT` | samples, e.g. `655` | ⚠**INSTRUMENT — can cause a device loss.** Every installed reference is cut to that many samples, marked complete (escaped) and loses its BLA: the 2026-09-21 RX 6800 XT field shape (`orbit_len=655 partial=false` against a 4,627 ask, 25–33 M rebases a frame), which a view that picks its own reference does not reach on demand. The install derate that a picker collapse would trigger is skipped for these cuts, so the budget stays where the controller had it; each cut is logged as `[fd-instrument]`. Pair with a staged view and `--soak N --soak-depth session`, whose `soak-regime:` line says whether the regime was ENTERED; armed but not entered ⇒ exit 2 (VACUOUS). ⚠A freshly opened view starts with an UNMEASURED budget, so its storm frames are chunked: the safe variant, reported as `0 of N dispatches UN-chunked`. The field's frames went out whole on a budget learned high elsewhere (1.515e11). To recreate that, add `--set TDR_BOOTSTRAP_STEPS=151500000000` (measured on the dev box: `1 of 1 dispatches UN-chunked`). ⚠The still rung does NOT reproduce the field's failure: on the RX 6800 XT (2026-09-25) its budget came down at the first timing reading with the instrument armed or not, and its control was indistinguishable from it. The field was ZOOMING, in taps. The motion rung is `--zoomtest --zoomtest-location session --zoomtest-start-log2 13.3 --zoomtest-taps 40,0.08,1.0 --zoomtest-hold 60 --zoomtest-rate 4.0 --window 1676x1360` on the staged crash view: taps of ~0.29 oct a little over a second apart (the field's pattern; a 0.08 tap coasts to ~0.29 at rate 4) from 2^13.3 until it arrives at the staged view's depth, then a minute's rest there (the field was lost ~20 s after the last tap), at the field's 1676x1295 render (`--window` is in points: 1676x1360 at 1.0x scale); it prints `zoomtest-regime:` as above and `zoomtest-stall:`, the longest run of slow frames across which the budget never came down (field: 20 over 32.8 s). `--soak` prints the same `soak-stall:` line. Marks the run non-stock, like `--set` (the self-test fails it, `--recordtest` calls it VACUOUS). Never for a golden or a baseline |
| `FRACTADYNE_BLA_DROP_FRAMES` | frames | INSTRUMENT: suppress BLA for that many frames after an arithmetic-mode switch, so `bla_skip=0` is reachable in a crossover on purpose. Non-stock, like the above |
| `FRACTADYNE_SEED_BUDGET` | millions of steps, e.g. `151500` | ⚠**INSTRUMENT — can cause a device loss.** The session's first frame installs a frame budget of that many million steps AS IF LEARNED, logged as `[fd-instrument] SEED_BUDGET=…`. The 2026-09-21 budget (1.515e11) was learned somewhere cheap and carried to the crash view, where every whole-frame render is under 0.7× of it and under the slow mark, so the budget rule discarded every reading. `TDR_BOOTSTRAP_STEPS` cannot recreate that, because it seeds only the unmeasured guess, which the first reading replaces. Pair with the crash view as the zoomtest location (no mode switch, which would reset it) and read `zoomtest-stall:`. Non-stock, like the others |
| `FRACTADYNE_PASS_CLOCK` | `1` | INSTRUMENT, observation only: GPU timestamps on EVERY live iterate pass (the frame budget's own timer brackets about one in three), each logged as a `[fd-passclock]` line with its frame, iteration range, nominal steps, GPU ms, the chunked resolve pass's ms, and the GAP since the previous pass ended (negative = it began before that one finished). `PRICED` marks the passes the budget saw, which read the same ticks as the line. From beta.127 each line ends with the calibration markers, `pre X (to iter Y) post X (after Y)`: empty passes recorded just before and just after, which read ~0 and sit within microseconds of the pass on a faithful clock. ⚠It perturbs what it measures on the RX 6800 XT: with it on, that card showed 470–559 ms frames, and pass timings that disagree with the CPU clock by hundreds of ms, none of which appear with it off. Use it to find which pass owns time on the RTX 3080, and take timing verdicts from the timing witness (`[fd-wgpu] timing witness`, record schema 3), never from a clock-on run. Changes no decision, but marks the run non-stock like the others. Needs `TIMESTAMP_QUERY` |
| `RUST_LOG` | env_logger spec | wgpu/naga internal logging (stderr) |

### Trace categories (`[fd-<cat>]` prefixes)

| Category | Fires | Tells you |
|----------|-------|-----------|
| `req` | every export-request build | The **effective** render manifest: mode, iterations, orbit length, SA skip, BLA, span, precision, size. The record that catches "rendered the wrong view" |
| `ref` | every reference build (fresh *and* reused/extended) | Orbit length/iterations/precision, escaped/partial, SA skip, BLA nodes, and the build-time split `orbit_ms`/`sa_ms`/`bla_ms` + `pick_reference` scoring ms (scoring is parallel across all cores since 0.2.40-beta.7 — ~0.6 s at 1e1216× where it was ~7.6 s); also `lookahead install:` lines when a playback-prefetched reference installs. Every build line is tagged with its ORIGIN — `[live]` (reactive), `[lookahead]`, `[hold]`, `[export]`, `[test]` — without which a log of four concurrent builders cannot be read at all (two e72 root-cause attempts died on exactly that ambiguity). Reactive spawns also log `interacting`, `cap_now`, the installed `orbit_len`, and which of the three triggers (`out_of_view`/`needs_quality`/`bla_out_of_range`) fired; during playback, `holding=` transitions mark each hold's boundary |
| `gpu` | live floatexp budget controller | Measured iterate ms per dispatch, budget grow/shrink, convergence; `aimd:` lines show the motion-res controller's real-frame cost signal + resolution decisions |
| `tile` | live floatexp frame sizing | Per-frame resolution/ss/iterations/steps vs budget, reprojection, tiled-settle grid state |
| `live` | every moving df32 frame the live-refresh rule does not decline, every price, every split pass | `live-refresh … Priced/Split(k)/Probe/Reprobe(k) steps=S of N pred_ms=P scale=K price_mode=M probe_cap=C` (the verdict, the steps it may render after the ceiling fit, the predicted GPU ms, the smoothness scale); `refresh-price … from=F Xms steps=S ns/step=Y` when a live pass's own timing prices the view (a split pass carries its set's share of the steps; `(drop bounded: Z)` when the reading was more than `LIVE_PRICE_DROP_MAX` = 4× under the view's last price and Z was used instead, from 0.3.0-beta.6); `split v0 f… pass j/k [start]` and `split v0 f… adopt k passes from f… lag_oct=L` for a split refresh; `live-refresh v=V f=F No: REASON …` once each time a reason for NOT rendering live starts to apply (`OverCeiling`, `NoTiming`, `ProbeWaits`, `ProbeTooSmall`, `BoundTooDear`, `MeasuredDear`, from 0.3.0-beta.5 `UnderKnee`, from 0.3.0-beta.8 `BackedOff`, from 0.3.0-beta.9 `NoSplitRoom`; from 0.3.0-beta.4 — before it, a glide that never went live traced nothing). One line per split pass: an 80-octave glide writes ~3,000, and one traced run's deepest band ran more frames over 20 ms than its untraced twin, so judge smoothness from untraced runs |
| `glitch` | multi-reference correction | Per-run summary: references used, residual glitched px, elapsed |
| `dpi` | every real change of scale factor or window size | Scale factor, logical size and **physical** size, before → after, one line per change (jitter suppressed). For the monitor-drag resize report: a healthy DPI transition HOLDS the logical size and rescales the physical one by exactly the new factor; the defect is the physical size ratcheting up beyond that, and the upstream reports describe the scale factor flipping repeatedly (1.0 ↔ 1.75) as it happens, which shows here as a burst of lines |
| `idle` | every frame, while the performance overlay is on | Why the app is still drawing: the quiescence verdict and each input to it (animation/playback clock, per-view tiled-settle and chunk-walk pending, reference build in flight, frames since each view last dispatched). ⭐Answers "a settled app is still burning GPU — what is holding it awake?", which no other channel can: the 2026-09-04 climb-probe loop dispatched a real 36.9 ms pass every 3 frames forever while `tile` reported `iterates=false` (true — the iterate KEY was deduped; the probe re-keys by nonce) |
| `autopilot` | every auto-zoom target evaluation | Depth, the tracked goal before the look (`goal_was`), the aim and zoom speed, the new goal (`pick`), whether it moved to a different region (`retarget`), the probe dims and the full-precision centre — enough to re-render any point of the dive with `--render --center X Y --zoom-log2 L`. A healthy dive shows `goal_was` ≈ `pick` and rare retargets |

### Every log-line prefix

A test (`every_log_category_is_documented`) scans the source for each category the app logs under
and fails if one is missing from this table.

| Prefix | Gated? | What it says |
|--------|--------|--------------|
| `[fd-start]` | always | Session header: version (with commit), arguments, bignum backends compiled in, which session loaded, where logs were directed |
| `[fd-render]` | always | CLI render manifest and failures; on the live path the always-on alarms — `slow frame N` (with body vs time outside it), `⚠LETHAL-BAND FRAME`, `⚠IN-FLIGHT PASS IN THE LETHAL BAND`, `⚠FRAME BUDGET IS BLIND` (from beta.131 followed by `DEAD-MAN: … budget X → Y`, and later `DEAD-MAN cleared`, see `DEAD_MAN`), `motion jam`, and from beta.135 `dispatch ceiling BINDS` / `released` (see `DISPATCH_CEILING`) |
| `[fd-wgpu]` | always | The adapter + capability line (`TIMESTAMP_QUERY`, attach bytes granted), from beta.135 followed by the `dispatch ceiling (…)` this adapter resolved to (see `DISPATCH_CEILING`), device errors and device loss; from beta.128 a `timing witness:` line every 30 s of GPU readings: how many of the frame budget's timestamp readings were held against their pass's CPU-side window (timer armed to the frame's completion callback, an upper bound), how many were IMPOSSIBLE (longer than it), and how much of the window a reading left unexplained when the queue was empty (RTX 3080: 13–17 ms at the median, callback latency). Per reading in the frame record (schema 3: `read_window_ms`, `read_queue_empty`) |
| `[fd-panic]` | always | A panic, and where its crash report was written |
| `[fd-oom]` | always | An allocation failure (the report is written from an 8 MB reserve) |
| `[fd-unclean]` | always | The previous session ended without a clean shutdown — with its last log lines and, from beta.113, how much of its frame record survived |
| `[fd-exit]` | always | A console-initiated shutdown (Ctrl+C, the console window closed) — recorded as NOT a crash; and, for a harness or offline job, `<harness> exit <code>` as it ends (from beta.120). A harness session without that line never reached its end — `--logcheck` calls it NO VERDICT |
| `[fd-harness]` | harness | `begin <harness> pid <pid> — tunables: <stock \| overrides \| INSTRUMENT …>`, first thing: what the run is, and the only record of an instrument armed from the environment |
| `[fd-verdict]` | harness | A harness's verdict line, also printed to stderr (`soak-liveness:`, `soak-regime:`, `soak-stall:`, `zoomtest-regime:`, `zoomtest-stall:`, `recordtest:`); `validation/logcheck-rules.toml` `[[harness]]` requires them |
| `[fd-logcheck]` | harness | The run's own log held to `validation/logcheck-rules.toml` as it exits: `PASS`, `FAIL`, with any `KNOWN` pre-existing conditions counted (see `--logcheck`) |
| `[fd-watch]` | always | `possible hang` — nothing stamped liveness for 10 s — with the last activity; also written into the frame record as a STALL row |
| `[fd-instrument]` | only with an instrument armed | What a diagnostic instrument did, e.g. `REF_ESCAPE_AT=655: v0 reference 37936 → 655 samples, complete, BLA off … install derate SKIPPED` |
| `[fd-passclock]` | only with `FRACTADYNE_PASS_CLOCK=1` | One line per live iterate pass: `v0 f1880 chunk [4627,4627) steps=2.206e6 gpu 0.412 ms gap +0.020 ms resolve 0.310 ms (gap +0.004) PRICED t0=…` |
| `[fd-frames]` | always, once | `frames.jsonl is falling behind` — its writer thread fell 4,096 rows behind (a slow or unreachable disk) — or `writer thread has stopped`; either way rows are being dropped. `frames.bin` and the in-memory record are unaffected |
| `[fd-accum]` | always | Progressive supersampling: `begin`, `waiting`, `abandoned`, `converged`, `⚠FOLD AT ANOTHER VIEW`. On a chunk-walked settle a run waits for the adaptive iteration limit and the colour range to be decided, and `begin` then says so (`limit N after S s`); `⚠… beginning with IterationLimit|ColourRange undecided` means their reading never came and the run began anyway. `begin` and `abandoned` print at most once per 5 s per view and say how many were held back; every begin is counted in the frame record (`accum_begins`). Once per run: `⚠… read other iterate inputs than sample 0` (a later sample read another mode, cap, orbit, series or BLA) and `⚠converged at N samples, but the average holds M`. With `--worker-gpu`: sample 0's inputs (`the worker's template`), each worker sample with its time and hash (`sample N rendered on the worker GPU in … ms (#…)`), and `the average holds all N samples` |
| `[fd-worker]` | only with `--worker-gpu` | The second GPU (design/multi-gpu-live.md L2): `worker GPU ready`, or why there is none; its loss (`lost`, `failed`) and `dropping the worker GPU — one GPU from here`; a sample that failed on it; and the `FRACTADYNE_WORKER_LOSE_AFTER` test hook firing |
| `[fd-glide]` | always, bounded | What was presented on each of the first 20 frames of every glide (`present=real\|reproject\|static(BLACK)` …) |
| `[fd-view]` | always | `⚠JUMP view` — the view moved without a deliberate jump |
| `[fd-cache]` | always | The on-disk orbit cache: size at startup, evictions, clears |
| `[fd-export]` | always | Tour export writes that waited on a slow destination |
| `[fd-farm]` | render farm | `--farm-render` / `--render-client`: connections, refusals and why, self-checks, admissions, assignments, strikes, removals, drops, resumes. The same lines go to the job's `<out>/farm/events.jsonl` (design/remote-rendering.md) |
| `[fd-console]` | always | Console output switched on or off from the Diagnostics window |
| `[fd-formula]` | always | A custom formula applied — its source, parameters, precision tier and the dispatch ceiling's cost-factor estimate — or a switch to Custom refused because none has been applied |
| `[fd-perf]` | always | Per-export GPU iterate/colour ms and event counters (every export path since beta.149, the normalized and glitch-corrected ones included), `tiles=`/`passes=` (beta.150); `file-write:` the CLI render's PNG/EXR encode + write ms and bytes |
| `[fd-progress]` | always (CLI) | CLI render progress, ~2 s cadence (`[progress]` in the log file) |
| `[fd-autodive]` `[fd-motiontest]` `[fd-zoomtest]` | harness | Each harness's own progress and verdict lines |
| `[fd-req]` `[fd-ref]` `[fd-gpu]` `[fd-tile]` `[fd-live]` `[fd-glitch]` `[fd-idle]` `[fd-dpi]` `[fd-autopilot]` `[fd-refwaste]` | `FRACTADYNE_TRACE` | The trace categories in the table above; `refwaste` accounts every reference build's CPU cost as `USED` / `SUPERSEDED` / `DROPPED` |
| `[crumb]` | always, file only | Breadcrumbs — phase transitions — tagged with the writing thread's name (reference builds run on `fd-ref-live`, `fd-ref-lookahead`, `fd-ref-hold`, `fd-ref-export`) |

`[selftest …ms]` lines stream `--selftest` check results. `fractadyne.log` rotates past ~5 MB
into `.1`–`.3`, checked while running as well as at startup.

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

### The frame record (`frames.bin`, the crash report's `frames:` section)

One structured row per frame per view, always on — no trace flag. Each row carries the frame
index (the same number the slow-frame line, `[fd-glide] fN`, the chunk/pin traces and the
`--show-timestamp` overlay show), what was asked (iterations, boost, auto-iter), the reference
(length, partial, SA skip, BLA), the price (learned budget, the budget the plan was sized against,
the dispatch's steps, the per-mode rates), what was dispatched and presented (resolution, ss,
chunk range, tile, live/reproject/hold), the frame interval and body time, the GPU reading the
budget controller judged that frame and its verdict (`DISCARDED`/moved/unchanged, and why growth
was refused), the palette-normalization window, and the user's input. It holds **4,096 frames** —
about a minute at 60 fps, and 14–68 minutes at the 1–5 fps of a failing session — where the
`budget :` decision ring beside it holds 24 decisions.

⚠**The GPU work counters (`ctr_rebase`, `ctr_bla_skip`, escaped pixels) appear only on the frame a
reading ARRIVED, tagged with the render they describe (`ctr_tag`).** A readback lands 2–3 frames
late; reading them as a per-frame value would pin one frame's rebase count on a dozen others.

**A wedge is a row, not a silence.** The record's only other writer is the UI thread — the thread
a wedge stops — so the watchdog thread writes a `kind: STALL` row each time it logs `possible
hang`, carrying how long nothing was recorded (`stall_ms`) and the last frame that was.

A crash report prints the last 40 rows under `frames  :` and writes all of them to the
`-frames.jsonl` beside it. `session :` in the report joins the three files. The encoding is
`validation/frame-schema.json` (`--dump-frame-schema` regenerates it; a test fails if it is stale),
and `--recordtest` is the gate that proves the record is being written.

**Reading it: `scripts/framelog.py`** (standard library only).

- `summarize <frames.bin | crash-…-frames.jsonl>` — which build and adapter produced it, then
  the scorecard (intervals, what was presented, readings by verdict, the budget's range, counter
  readings, frames in the escaped-reference shape, the record's own cost) and every **slow
  episode** with what the budget controller was told during it, labelled with which of the five
  candidate mechanisms of the 2026-09-21 stall it fits: (a) no reading arrived, (b) readings
  priced the wrong dispatch, (c) the timed bracket missed the work, (d) the time went where no
  iterate is timed (slow frames that dispatched nothing), (e) slow readings did not move the budget.
- `compare --a A1 A2 A3 --b B1 B2 B3` — before/after, three runs per arm minimum (VACUOUS
  otherwise); each metric against its own run-to-run range, with arm A split against itself as the
  control. A metric the control "separates" is noise on that run and its verdict carries a `?`.
- `schema-check <file>` — strict: an unknown or a missing key is a failure, never a skipped field.
- `decode <frames.bin> [-o out.jsonl]`, and `selftest`, which feeds the mechanism labelling one
  synthetic episode per mechanism plus a healthy control and fails on any mislabel.

### The `bignum` line in a crash report

Crash reports and `--selftest` name the arbitrary-precision backend that produced the run, and
the value is taken from the arithmetic that ACTUALLY ran rather than from a flag or a setting
that could disagree with it. `none (no reference orbit built yet)` is a real answer, not a
missing field — it means the process died before any deep-zoom work happened.
`--selftest` fails if a single run is attributable to more than one backend, because every
golden and blessed baseline is the output of exactly one.

## Reading common symptoms

- **App window "Not Responding" / closed by itself** → open `logs/fractadyne.log`. A crash
  leaves `[fd-panic]` + a `crash-*.txt`; a hang leaves `[fd-watch] possible hang: <activity>`
  every 30 s naming the wedged phase. Nothing at all = killed externally (driver TDR, OOM
  killer, user) — or an abort the panic hook cannot see (`0xc0000409`, `0xc0000005`): the NEXT
  launch then writes an `[fd-unclean]` report which, from beta.113, carries the dead session's
  own frame record recovered from `frames.bin` (`its frame record survived: … N records`).
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
| `--show-timestamp` (also View ▸ Show timestamp) | **Draws a large elapsed-time clock over the live view**, reading the *same* `+12.345s` the log stamps every line with, plus the frame number. For anything that only shows itself in MOTION — a slide, a flash, a blank frame — record the screen, then line the recording up against the log frame by frame instead of guessing which log line the eye caught. Persists in the session; `--no-show-timestamp` forces it off |
| `scripts/dive-capture/` | **Score what a live dive put on screen.** `capdive.sh` runs `--autodive` from a `.kfr` in a wiped scratch config and PrintWindow-captures the window; `blankscore.py` counts captured frames whose canvas is a single flat colour. This is the measurement that caught the blank-motion-frame bug after every per-pass log metric had called three fixes good — resolution is part of the walk's signature, and a controller that re-chooses it restarts the walk. `dive-2p800.kfr` + `session-seed.toml` reproduce the regime; compare three pairs, single runs vary 8–14% |
| `--autopilot-target detail\|misiurewicz` (also the panel's "Auto-zoom target") | **What the auto-zoom aims at.** `misiurewicz` runs the Go-to dialog's detect + Newton solve off-thread from the start view (`autopilot` log lines `misiurewicz target: detecting from 2^… solving to 2^…` / `… solved to 2^… in N s, repeat R oct`), holds the view until it lands, then dives into the coordinate with the goal re-projected exactly every frame (`[fd-autopilot] eval … misi=(k,p)`); a miss stops the dive with the reason in the toast and the log. `detail` is the steering probe described above |
| `--autopilot-priority speed\|quality` (also the panel's "Auto-zoom priority") | **Which the auto-zoom puts first**, for harness dives (`--autodive`, `capdive.sh`) as well as the GUI. `speed`: the slider rate; a pinned refresh is adopted as soon as its picture has stopped changing (`pin-adopt … mode=converged`). `quality`: full-ask native-resolution refreshes, and the zoom is paced by the measured refresh period (`pin-adopt … period=`) so the held frame stays under `HELD_MAX_OCT`. Under both, a frame reaches the screen — or becomes the hold snapshot — only when a reading of its own render shows escaped pixels: `FRACTADYNE_TRACE=gpu` prints `content: view= tag= cursor= escaped=N/px verdict=detail\|BLANK live_ok= hold_ok=` per readback, `tile` prints `present f= shows=live\|hold\|reproject live_ok= hold_ok=` per frame and `pin-abandon reason=Blank` for a walk that finished with nothing escaped (kept off the screen; three in a row stop the autopilot), and `ref` prints `install DEFERRED` when a reference at another point is parked behind a young pin (design/verified-present.md) |
| `--show-zoom-target` (also View ▸ Show auto-zoom target) | While the auto-zoom runs, outlines the region it is zooming into (what fills the screen after 4× more magnification, about the current aim) with lines to the screen corners, and rings the steering goal. Pair it with `--show-timestamp` when a dive "goes somewhere odd": the box says where the camera is going, the ring says where the steering wants it, and the clock lines the recording up against the `[fd-autopilot]` evals. Persists in the session; `--no-show-zoom-target` forces it off |
| `--selftest [--out report.md] [--bless]` | The full correctness suite (~170 checks + 18 goldens; it prints its own totals, so this text cannot go stale), streamed live; hermetic (resets config at entry, echoes it); GPU errors are printed, never silently skipped; data files resolve relative to the repo even when run elsewhere. The run's tail is the `bench-matrix` group — deterministic path-signature tripwires (see `--bench-matrix`) |
| `--selftest-filter <substr>` | Run only matching check groups / goldens (fast iteration on one failure; not a release verdict — groups share state) |
| `--recordtest [FRAMES]` | **Is the frame record being written?** Drives FRAMES live frames (default 240) at 1e12× with a jump to 1e13.5× halfway and a deliberate 13 s wedge of the UI thread three quarters through, then checks, against its own count: every frame has exactly one view-0 record; no required field is left unset; `frames.bin` read back is exactly the in-memory ring; the watchdog wrote a stall row of ≥ 10 s naming the last frame recorded; `frames.jsonl` opens with this session's header, every row carries exactly its keys, and the summaries' frame counts equal the frames recorded, with no row dropped by its writer thread; the record's p99 cost is under 0.5% of a 60 Hz frame (it prints where the logs are, and if they are on a network share an excess is VACUOUS, not a failure — that measures the share); and a child process that records 60 frames and then `abort()`s (`0xc0000409`) leaves all 60 in its `frames.bin`, untorn. Exit 0 pass / 1 fail / **2 VACUOUS** — no dispatch, judged reading or counter reading was exercised, or `--set` was in force. ~20 s; run with a wiped `FRACTADYNE_CONFIG_DIR` |
| `--dump-frame-schema` | Print the frame record's encoding (fields, types, slot layout) as JSON — regenerates `validation/frame-schema.json` (in bash; PowerShell `>` writes UTF-16) |
| `--selftest-list` | Print the group tags usable with `--selftest-filter` |
| `--profile [--regions file.toml] [--reps N]` | Per-region reference/SA/BLA build ms + pure-GPU pass ms (TIMESTAMP_QUERY); includes a corpus-14-class `deep-interior-1e148` region (dip orbit, 800k iters — the export-throughput-gap regime) |
| `--bench-bignum [--iters N]` | Reference-orbit cost per arbitrary-precision backend, at precisions from 64 to 8256 bits. **CPU only - no GPU**, so it runs on a CI box. On a build with more than one backend it times each over the SAME work in one process and **asserts the orbits are byte-identical** (exit 1 if not): a speed ratio between backends that computed different orbits is meaningless. Marks any row whose test orbit escaped as INVALID rather than reporting the meaninglessly fast number that produces. `--iters` scales the counts (fatal if unreadable, never a silent default) |
| `--bench-matrix [--bless] [--reps N]` | Path-coverage perf + regression suite (zoom bands, fractals, coloring). Per-segment CPU-build vs GPU split + deterministic GPU event counters, compared against `benchmarks/bench-matrix-baseline.json`. Algorithmic drift → exit 2; timing regression → warn. `--bless` records the baseline. See [design/bench-matrix.md](design/bench-matrix.md) |
| `--zoomtest [OCTAVES] [--zoomtest-rate R] [--zoomtest-location FILE.fdn] [--zoomtest-start-log2 L] [--out log.json]` | **On-screen update-latency harness for live zooms** — the one that measures what a viewer feels. Runs the real window, holds a virtual Space key through the production glide (pacer + interactive reference lookahead) for OCTAVES octaves (default 40) at zoom rate R (default 1.0) from a `.fdn` (default: a deep corpus centre at ~1e31×; with `L`, from 2^L on that file's centre — `L=0` is the whole descent from 1×, e.g. `--zoomtest 332.19 --zoomtest-start-log2 0` = 1× → 1e100), and stamps every presented frame with the wall interval since the previous one, whether it was a real re-iterate or a held reprojection (and how many octaves old the held frame was), the per-frame zoom step, depth lag, installs / lookahead installs and pacer throttle. Summary: mean / p50 / p95 / p99 / max interval, >33 / 50 / 100 ms hitch counts, the longest stall and where it happened, real-refresh cadence, held-frame magnification, zoom-step spread. `FRACTADYNE_NO_PREFETCH=1` = A/B without the lookahead. Exit 2 = never reached the regime, 4 = watchdog. Unlike `--divetest` (headless, synthetic clock) this is real presents. Each frame also records the iterate resolution actually dispatched (`res_px`), the observed zoom speed the refresh sizing used (`oct_s`) and the running count of pinned refreshes adopted (`adopts` — the proof the held frame is being replaced, not merely re-iterated). `python scripts/zoomtest_report.py A.json [B.json] [--worst N]` prints the summary, an interval histogram, a per-depth-band table (where in the dive the stutter lives) and the worst frames with their context (held or real, lag, install landed) |
| `--divetest tour.toml [--out log.json]` | Headless live-dive perf harness: plays real-time 18 s windows of a tour at every 100 decades of depth through the ACTUAL playback machinery (pacer, lookahead, reuse-hold, motion-res controller) with real GPU iterates, vsync-paced. Per band: fps, p50/p95/max frame ms, >33 ms hitches, real-refresh rate/cost (CPU vs pure-GPU), reference installs, pacer engagement, achieved oct/s, and smoothness AS SEEN: `hold%` (frames that reprojected a held frame), `gap p95/max` (wall ms between consecutive REAL frames — the visible stall), `mag max` (how far the presenter magnified a held frame — faithful for un-pinned frames only: a pinned mode-2 refresh latches on adoption, which this harness never performs, so there a held frame measures against the window's first latch), `vel%` (share of the selected zoom speed the pacer let through), `look` (lookahead installs). `FRACTADYNE_DIVETEST_GLIDE=<rate>` swaps the tour camera for an interactive glide (see the env table). The dive-smoothness regression harness — diff the JSON across builds. ⚠Tour-mode JSONs written before 2026-09-15 are not comparable: the harness never reset the lookahead's per-second spawn backstop (`PREFETCH_BUILDS_PER_S`), so the tour lookahead switched itself OFF a couple of seconds into every window (measured: 32 vs 153 installs at 1e40) |
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
`EXPLICIT_DISPATCH_CAP`, `TDR_MAX_TILES`, `TDR_TILES_CEIL`, `PASS_FIXED_MS` — the per-pass fixed
GPU cost in ms (default 0.2; `0` = off). With it, a moving frame's pass is priced as a fixed part
plus a per-step part instead of per step alone, so a card whose small passes are mostly fixed cost
(the RX 6800 XT: ~1 ms of a 1.14 ms pass) gets larger, sharper motion frames. Bounded: at most three
quarters of a reading or of the 10 ms pass target is treated as fixed — and `MOTION_NEED_QUANTILE`
(default 1): the fraction of the picture a moving frame is sized to reach. At 1 its walk must reach
the view's slowest escaping pixel; below 1 (e.g. 0.9), only the iteration by which that fraction of
the last complete walk had escaped, so deep views with a few stragglers get larger moving frames and
the stragglers show unfinished until the view settles — and `READING_POOL` (default 1 from beta.129;
`0` = off): GPU timings the frame budget would discard as too small to count are added together until
they are representative, then priced as one. While the view moves every timing is a small one, so
without it the budget cannot learn during a zoom. A timing the timing witness proves impossible never
joins the pool, and neither does one it judges possibly too short — and `DEAD_MAN` (default 1 from
beta.131; `0` = report only): when the budget-blind tripwire fires (eight frames over `TDR_BUDGET_MS`
by the wall while no GPU timing looked slow, or from beta.132 ONE frame at or past `TDR_LETHAL_MS`;
from beta.136 a frame's time inside a synchronous offscreen render — a CLI `--render` or
`--render-tour`, a glitch-corrected export — is not counted, since that is the app's own blocking work
and not a live dispatch), the view's frame budget drops to its bootstrap, which
bounds every dispatch path at once, and may not grow until a frame comes in under half
`TDR_BUDGET_MS`. Logged as `DEAD-MAN: view=… budget X → Y` and `DEAD-MAN cleared` — and
`DISPATCH_CEILING` (default 1 from beta.135; `0` = off, for a before/after measurement only): a
fixed per-adapter cap on one dispatch's nominal steps that nothing learned can raise, sized so a
ceiling-sized dispatch at the adapter's worst MEASURED per-step cost takes ~400 ms. The costs, the
per-formula factors and each card's occupancy knee live in `validation/calibration/ceilings.toml`,
which also says how to calibrate a new card; an adapter without an entry gets the table's
conservative default. Direct and df32-perturbation dispatches only (floatexp's BLA makes its
nominal steps unrepresentative). The adapter line is followed by `dispatch ceiling (calibrated for
…)`, and a view logs `dispatch ceiling BINDS on v0: learned X → Y` when the ceiling starts to limit
its budget and `dispatch ceiling released` when it stops. The value itself is NOT overridable: a
per-card number as an override would make every gate on that card non-stock. — and `TAIL_DF32`
(default 1 from beta.141; `0` = off, for a before/after measurement): a deep (floatexp) Mandelbrot
step whose offset from the reference has grown past 2^-60 runs in ordinary df32 arithmetic instead
of floatexp, and back if it shrinks. The share of steps taken that way is the `in df32` figure on a
CLI render's `perf` line, and `--tail-audit` checks the pixels the phase changes against the
arbitrary-precision oracle. — and `REF_OVERLAP` (default 1 from beta.143; `0` = the sequential
order, for a before/after measurement): a fresh reference builds the view centre's orbit and
series skip on their own threads while the pick scores candidates, and the pick's centre rescue
reads its score from that build instead of walking the centre again. The result is byte-identical
either way (selftest `ref-overlap`); `FRACTADYNE_TRACE=ref` prints `overlap: centre build USED` or
`DISCARDED` per build, with `sa=overlap` when the series skip came from the parallel walk. ⚠A
build's own `orbit_ms` (and `--bench-matrix`'s `ref ms` column) is then clocked while the pick's
phase 1 has every core busy, so it reads 10–45% higher than a sequential build's while the whole
reference window is shorter: at beta.143 the bench-matrix's 16 builds took 3,942 ms from
`building reference` to `reference built`, against 4,343 ms with `--set REF_OVERLAP=0`, none of
them slower. Judge the overlap by that window, not by the build's clock. — and `EARLY_REF`
(default 1 from beta.147; `0` = off, for a before/after measurement): a plain single-view
`--render` starts its reference build in `main`, before eframe creates the window and GPU device
(~0.7 s), and the export uses it only when its own reference inputs match field for field. The log
says `early reference USED — started N ms before the export asked for it, which then waited M ms`,
or `early reference DISCARDED — its <field> differs` (the render then builds as before; a discard
is a missed speed-up, never a different picture). The window/device creation itself reads ~0.1–
0.2 s slower while the build competes with it for the CPU; the render as a whole is faster. — and
`TILE_OCCUPANCY` (default 1 from beta.150; `0` = the old tiles, for a before/after measurement):
a mode-2 export (plain or normalized) tiles the frame into equal parts of up to 1024 samples a
side instead of sizing each tile as if every pixel ran every iteration (158 px at 800,000
iterations), and runs each tile as STEP-BOUNDED passes: every pixel stops after `step_cap`
executed steps and resumes in the next pass, until none is running. The cap is sized from the
measured cost per active pixel-step so a pass lands near 200 ms, and never exceeds 6e8
pixel-steps. The `[fd-perf]` line reports `tiles=` and `passes=`; `FRACTADYNE_TRACE=tile` prints
one `steps pass N cap=C wall=W running=R` line per pass. Pictures are the same whichever way the
frame is split (selftest `occupancy tile: step-bounded passes resume bit-identically`). Modes 0/1
keep their tiles. — and `LIVE_REFRESH` (default 1 from 0.3.0-beta.2; `0` = every moving
perturbation frame holds and refreshes through the pinned chunk walk, as before): a moving df32
frame whose whole refresh has been MEASURED cheap renders live. The price is the GPU time of a
recent live pass of the same view (same mode, navigation epoch and depth window), timed by its own
timer; a refresh up to 4.5 ms renders every frame in one pass. From 0.3.0-beta.3 a dearer one, up
to 36 ms, renders as a SPLIT: `k` = ⌈cost / 4.5 ms⌉ passes (at most 8), one per frame, each one
checkerboard set of 16-texel tiles of the frame at the view the split started at, the display
serving the last complete frame (reprojected to the live view) until the last set has run.
Without "Prefer detail while zooming" a frame that shrinks to 4.5 ms at no less than 0.75 scale
renders every frame that way instead. A live frame never exceeds the dispatch ceiling. With no
price (a mode switch, a jump), one probe measures it, then one full-size split. The probe carries
a 40 ms share of the ceiling or, from 0.3.0-beta.4, the bound the view's own walk passes already
dispatch at, whichever is larger, and runs only if that holds it at 0.35 scale or more. (The
share alone never fitted on the RX 6800 XT: below its 524k-px occupancy knee a dispatch costs its
iterations over the knee's pixels, so shrinking a probe stops helping, and 0.3.0-beta.3 rendered
no live frame there in a 30-octave glide.) From 0.3.0-beta.5 both the split and the shrink count
that knee: a set of `px/k` pixels is predicted at the frame's cost × max(1/k, knee/px) and must
stay within `LIVE_SPLIT_SET_MAX_MS` (6.75 ms), no split goes finer than brings a set down to the
knee, and a shrunk frame may not go below it. On the RX 6800 XT at 1280×800 that leaves halves of
frames up to ~9 ms and sends dearer ones to the walk (an eighth of a 15–27 ms frame had cost
6–27 ms); on the RTX 3080 it changes little (sevenths of 1457×1102 still split). From
0.3.0-beta.7 a stale price's re-measurements scale the same way: a reprobe below 4/3 and a probe
below 8/3 of the dearest frame the adapter can render live at this size (`live_max_ms`: 36 ms on
the RTX 3080, ~12 on the RX 6800 XT at 1280×800), nothing past that until the price lapses; and
from 0.3.0-beta.6 one reading may lower a view's price by at most 4× (`LIVE_PRICE_DROP_MAX`).
From 0.3.0-beta.8 the probe window ends at `LIVE_HOPE_X` (2) × that live maximum, and a price past
it BACKS OFF the view: it still stands after it lapses, however deep the zoom goes, until the render
mode or navigation epoch changes or its prediction falls under that (`BackedOff`), so a card that
cannot render a view live stops probing it. From 0.3.0-beta.9 a frame under twice the knee has no
room to split (`split_room`; the RX 6800 XT at 1280×800): no probe, reprobe or split measures it
(`NoSplitRoom`), and only a price it already has — a settled frame's own pass — may license a frame
at the share, or shrunk to it. With live refresh off (`--set LIVE_REFRESH=0`) 0.3.0-beta.8 matched
0.3.0-beta.3 exactly on that card; on, even two probes a glide slowed the walk after them.
`FRACTADYNE_TRACE=live` shows every verdict (`Priced`,
`Split(k)`, `Probe`, `Reprobe(k)`), each reason for `No` as it starts to apply, every price, and
each split's passes (`split v0 f… pass j/k [start]`, `… adopt k passes from f… lag_oct=…`).
Selftest group `live-split`: the sets compose the frame bit for bit, one set writes only its tiles,
one set costs about its share (wall-clock timed), GPU timestamps describe their own pass (both
draws, timestamp against wall — the price rides on them), and the tile geometry costs what the
full-screen pass does.

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

For a tester who will not touch a command line at all, **Help ▸ Diagnostics** runs the two headless
checks that need no context — the self-test and the **GPU arithmetic check** (`--gputest` behind a
button) — streams progress, and attaches the result to an issue report. The arithmetic check is
deliberately informational: on NVIDIA it reports that the compiler folds the error-free transforms,
which is the finding to collect, not a fault, so the dialog says "result captured" rather than
grading it. It is the one-click form of the df32 corroboration request.

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
everywhere; the non-golden checks should pass everywhere; the 18 goldens are compared
*exactly* and were blessed on an RTX 3080, so cross-vendor deltas there are expected rather than
bugs (judge by count and magnitude); `bench-matrix` timings are meaningless across machines but
exit 2 means algorithmic drift; `livetest` compares live against offline *on that machine*, so its
FAILs are meaningful even on unfamiliar hardware while its "drift" lines are not.
`adapter.txt` records what the app itself resolved — adapter, backend, `TIMESTAMP_QUERY` — which
is the "record adapter and resolved tunables per card" half of B6.

- **Local.** The battery runs under `%TEMP%\fractadyne-validate\` (`${TMPDIR:-/tmp}` on Linux) and
  copies the finished bundle to `-Out` at the end (beta.114). Every step logs into the bundle, and
  `frames.bin` is written on the UI thread each frame, so a bundle built on a network share
  measures the share.

### Running it from the dev box (the field agent)

A test machine can run the battery, and a few harnesses, on request, with no one at it and no
inbound connection. The only channel is the share it already reads builds from.

- **On the test machine, once:** `pwsh -ExecutionPolicy Bypass -File
  \\<host>\share\Fractadyne\field\setup\field-agent-setup.ps1 -Share \\<host>\share\Fractadyne`
  (or set `FRACTADYNE_SHARE` first). That installs `field-agent.ps1` into
  `%LOCALAPPDATA%\Fractadyne-field\` plus a scheduled task that starts it, hidden, in the user's
  own session at logon. `-Disable` / `-Enable` / `-Status` / `-Uninstall` manage it.
- **From the dev box:** `scripts\field-request.ps1` files a request in `<share>\field\requests\`
  and, with `-Wait`, watches `<share>\field\results\<id>\status.json`. With no arguments it shows
  each agent's heartbeat (`<share>\field\agent\<COMPUTER>.json`), the queue, and recent results.

It runs **only published packages**: a zip in `builds\<tag>\` whose sha256 matches `BUILD-ID.txt`,
copied and extracted locally. Three actions:
- `battery` — `gpu-validate.ps1`, `-Quick` optional.
- `harness` — `fractadyne.exe` with an allow-listed mode such as `--recordtest`, `--zoomtest`,
  `--motiontest`, `--chunk-sweep`, `--selftest`, `--livetest` or `--soak`; optionally several
  builds interleaved A,B,A,B for an A/B.
- `events` — the event log's display-driver events, LiveKernelEvent (GPU reset) reports and
  Fractadyne crash reports.

A harness request can carry a **view** (`-ViewFdn some.fdn`), which becomes the run's session: a
windowed harness renders the SESSION's view, and `--center`/`--zoom` are read only by the headless
modes. The session is the committed corpus template (`validation/corpus/session-template.toml`,
published beside the agent) with the view keys pinned. A run whose log lacks `session: … loaded` is
failed, not reported. From agent v3, a newer `field-agent.ps1` in `<share>\field\setup\` replaces
the running one between jobs.

Anything else is `rejected`, with the reason. Deliberately absent: `--deviceloss-repro`,
`--autodive`, and anything that writes outside its own run folder. A job starts only when the
machine is unlocked, idle for 5 minutes, and Fractadyne is closed. Each harness run records
whether anyone used the machine during it.

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
Fraktaler-3's demonstrated range (the deepest F3-matched corpus pair is 6.13e1105×), so there is no
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
