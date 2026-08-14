# Diagnostics & Observability Plan

Plan for making analysis, debugging, and performance tuning effective — grounded in the
failures that actually cost us time, not hypothetical needs. Each provision below cites the
incident that motivates it.

## The failure catalog (what happened, what it cost, what was missing)

| # | Incident | Cost | Missing tooling |
|---|----------|------|-----------------|
| F1 | wgpu device-lost panic on load at a deep interior minibrot (v0.2.1). The user reported "hangs on load" — the panic text existed only on stderr, invisible for a GUI-subsystem launch. | Two sessions + a reboot round-trip to even classify crash vs hang vs driver | No log file; no crash report; no record of what the app was doing when it died |
| F2 | True UI hangs: a budget-sized GPU dispatch blocked the frame loop ("Not Responding", CPU≈0). A 60 s soak that grepped stderr for panics **passed a hung app**. | One session; a wrong "fixed" claim shipped | Nothing in-app notices a stalled frame loop; no heartbeat; verification had no responsiveness signal to consume |
| F3 | Offscreen `--render` hung >1 h (glitch-correction pathology at 1e500×; separately, a home-view render at 6M iterations). The export path *has* a progress `AtomicU32` — the CLI never prints it. | Hours across three separate timeout round-trips | No progress output, no per-phase timings, so "slow" and "hung" were indistinguishable without killing the process |
| F4 | A shader fix was silently dead: WGSL compilers may fold `x != x` (NaN test) to false; the marker branch never executed. Output was **byte-identical to pre-fix** — which was itself the tell, noticed late. | Half a session chasing phantom secondary causes | No GPU-side event counters (rebases fired, extended samples decoded, glitch flags raised) — a dead code path is invisible |
| F5 | Three successive frame-budget controllers failed because wall-clock frame intervals carry no GPU signal (repaint scheduling dominates at ~420 ms regardless of frame size). | One session of controller rewrites | GPU timestamps existed only in `--profile`; nothing measured the live iterate until it was added mid-crisis |
| F6 | Selftest read the live session file and produced spurious failures **twice**: `color_method` (58/60, v0.2.1) and `series_approx` (58/61, v0.2.6) — both times a stripe/staged session leaked into "hermetic" checks. | Two full debugging rounds, each initially read as a real regression | Checks pin fields ad hoc instead of snapshotting the whole config; no effective-config echo at suite start |
| F7 | Selftest ran 20+ minutes with **zero output** (post-v0.2.6, deep checks doing honest work). Which check was slow? Unknowable without killing it. | Live, as this plan is written | Results are buffered to the end; no per-check duration; no per-check timeout; no subset filter |
| F8 | `--render` silently rendered the home view for an entire corpus batch (its CLI flags always reset center/zoom; the session-staged location was ignored). Twenty "renders" of the wrong image, detected only by eye. | One session (compounded F3) | No render manifest: nothing prints the *effective* center/zoom/iterations/mode before rendering |
| F9 | Orbit-level forensics (dip magnitudes, escape-time distributions) had to be improvised as env-var-gated test files mid-investigation. | Mitigated — but only because the session invented them | No first-class probes; `probe_orbit.rs` / `probe_escape.rs` now exist but are undocumented |

Additional gaps confirmed by a code inventory (2026-07-10):

| # | Finding | Anchor |
|---|---------|--------|
| F10 | Selftest CPU oracles run at the **session's** `max_iter` (unpinned): with a 500k session, the naive bignum dwell checks take ~14 s *each* — the suite went from ~3 min to 35+ min when v0.2.5 removed the iteration cap on explicit counts. Only `fractal`/`julia_mode` are pinned at suite entry. | selftest.rs:112, render.rs:801 |
| F11 | GPU errors are swallowed suite-wide: every render closure is `render_iter(..).ok()` — a device-lost mid-suite is indistinguishable from any failure, and half the checks silently *skip* on `None` (no FAIL, no message), so the check count itself varies. | selftest.rs:121 |
| F12 | Catalog checks read `validation/catalog.toml` by CWD-relative path inside `if let Ok(..)` with no else: run from any other directory and the whole category silently vanishes — the other source of run-to-run check-count variance. | selftest.rs:1522 |
| F13 | The suite *mutates* live app config destructively (e.g. hard-sets `use_bla=false, series_approx=true` at block exits rather than restoring saved values); later blocks depend on those hard-sets. Only `process::exit` (no session save) keeps this from corrupting the session file. | selftest.rs:1160, main.rs:3868 |
| F14 | Every export-path reference build already measures `reference_ms/series_ms/bla_ms` (ProfSetup) — and drops them unread except under `--profile`. The glitch-correction loop (the >1 h sink) has **no timing hook at all**. | render.rs:833 |
| F15 | `--frametest`'s "gpu" column is CPU wall-clock around `render_iter` — the exact measurement the codebase itself documents as signal-free (~440 ms fixed overhead). | profile.rs:416 |
| F16 | `--profile` has no region deeper than ~1e30× and never engages the glitch pass or the tiled export path — the regimes where every recent pathology lived are unprofiled. | profile.rs:46 |
| F17 | `timing.rs` capture is thread-local — GUI background exports can never produce GPU timings; sums across tiles, losing per-tile attribution. | timing.rs:29 |

Two cross-cutting lessons:

- **Byte-identical output across a change means the changed code did not execute.** Tooling
  should make execution observable (counters), not inferable.
- **Silence is ambiguous.** Every long-running operation must distinguish "working" from
  "stuck" without being killed: progress, phase transitions, or a heartbeat.

## What exists today

> **Snapshot as of when this plan was written (v0.2.2–0.2.9).** Deliberately left as-is — the
> point of the list is what was available when the plan was made, and rewriting it would erase
> the reasoning. For the *current* tooling see [DIAGNOSTICS.md](../DIAGNOSTICS.md); the counts
> below have since grown (the self-test is 113 checks + 17 goldens), and see "After this plan"
> at the end for the tooling added later.

- `FRACTADYNE_TRACE=1` — ad-hoc but proven stderr tracing: `[fd-trace]` per-frame GPU sizing,
  `[fd-req]` export request manifest, `[fd-ref]` reference-build stats, `[fd-gpu]` budget
  controller. Added during the v0.2.2–0.2.6 investigations; solved in one run what constant-
  guessing could not. Undocumented, no timestamps, uneven coverage (live-path-centric).
- `--profile` harness — per-region reference/series/BLA build timings + pure-GPU pass times via
  `TIMESTAMP_QUERY` (`timing.rs`); the only trustworthy GPU numbers before the live-path
  `IterTiming` was added.
- `--selftest` — 61 checks + 17 goldens, report to `validation/report.md` at the end; partial
  per-check hermeticity pins (`color_method`, now `series_approx`).
- `--benchmark-std` — standardized dive benchmark.
- Live `IterTiming` (v0.2.3) — GPU timestamps on the live iterate pass feeding the frame budget.
- Perf HUD panel — FPS, frame/cpu ms, recompute stats, mode/iter/precision.
- Export progress `AtomicU32` — written by `render_export`, surfaced only in the GUI export
  dialog; never in CLI mode.
- Orbit forensics tests — `probe_orbit.rs` (stored-sample dynamic range, dip periods, extended-
  marker counts), `probe_escape.rs` (CPU floatexp perturbation + rebasing escape times).

## The plan

Phased by measured cost of the failures each phase would have prevented.

### D1 — Crash & hang visibility (prevents F1, F2, F3, F9-class losses) — ✅ SHIPPED v0.2.7 (except D1.6 stretch)

1. **Log file.** `env_logger` output additionally to a rotating `logs/fractadyne.log` under the
   config dir (the `directories` crate is already a dependency). Level via `FRACTADYNE_LOG`
   (default `info`). GUI launches on Windows currently discard stderr entirely — this is the
   single highest-value change.
2. **Breadcrumb cell.** A global "current activity" slot (`AtomicPtr`/mutexed `String`, written
   cheaply): `building reference 150000@1727bit`, `export tile 3/16`, `selftest: Multibrot 4 SA
   engages`, `settle tile 7/19`. Costs one store per phase transition.
3. **Panic hook + wgpu handlers.** On panic: write `logs/crash-<timestamp>.txt` with the panic
   message, backtrace, breadcrumb, and an effective-config snapshot (view, iterations, mode,
   budget state). Install `Device::on_uncaptured_error` and the device-lost callback to log the
   in-flight dispatch parameters *before* the panic unwinds. F1 becomes a one-file bug report.
4. **Heartbeat watchdog.** A thread stamped by `update()`; if stale >10 s it logs
   `possible hang: <breadcrumb>` (once per 30 s). CLI paths stamp per-phase instead. A hung app
   now *names its own hang* in the log — externally observable without a debugger.
5. **CLI progress.** `--render`/`--render-tour`/`--selftest` print phase transitions and the
   existing progress atomic (`\r` line, ~2 s cadence): `reference 42% … iterate tile 5/16 …`.
   "Slow vs hung" becomes readable at a glance (F3).
6. **Hard-fault stretch goal.** Rust panics and wgpu callbacks don't catch access violations or
   driver-level kills. Evaluate `crash-handler`/`minidumper` for a SetUnhandledExceptionFilter
   minidump writer; low priority since no such fault has occurred yet, but the hook placement
   (D1.3) should leave room for it.

### D2 — Selftest hardening (prevents F6, F7) — ✅ SHIPPED (D2.1–D2.4 v0.2.6, D2.2/D2.5–D2.7 v0.2.8, D2.8 v0.2.9)

1. **Whole-config snapshot + hermetic baseline.** At suite start: reset `render_cfg`,
   `coloring`, `effects`, `fractal`, `julia_mode` to a documented baseline (no restore needed —
   `--selftest` exits via `process::exit`, main.rs:3868, and never saves the session). Deletes
   the per-field pin whack-a-mole class (two incidents — and F10 shows the third was already
   here: unpinned `max_iter` turned the CPU-oracle checks into a 35-minute suite).
2. **Echo the effective config** at suite start (one line into stdout + report) so any residual
   leak is visible in the failure report itself.
3. **Stream results.** Print each check's line (name, pass/fail, duration ms) as it completes,
   flushed. Today's 20-minute dark run becomes a live scoreboard, and the slow check names
   itself (F7).
4. **Per-check duration + soft timeout.** Record ms per check into the report; warn on stderr
   when a check passes 60 s (configurable) — with the breadcrumb (D1.2) naming the phase inside
   the check. *(Streaming + duration landed 2026-07-10: `push_check` prints each result live —
   it identified F10's 14-second oracle checks on its first run.)*
5. **Stop swallowing GPU errors (F11).** The render closures must print the `GpuError` and
   record an explicit FAILED/SKIPPED check instead of silently omitting it; a device-lost
   mid-suite should abort the suite with a clear message, not shrink the check count.
6. **Anchor data files to the executable/repo, not CWD (F12)** — or fail loudly when
   `validation/catalog.toml` / goldens are absent.
7. **`--selftest-filter <substr>` / `--selftest-list`.** Re-running one failing check should
   cost seconds, not the full suite.
8. **Execution-proof assertions (F4).** With D3.3's GPU counters: deep checks assert the paths
   they exercise actually fired (rebases > 0 at a rebase-heavy view; extended samples decoded >
   0 on a dip-carrying orbit). A silently-dead shader branch fails loudly.

### D3 — Performance observability (feeds the ~50× export-throughput investigation; prevents F5-class) — ✅ SHIPPED v0.2.9 (live-path counter readback deferred)

1. **Export-path timestamps.** Per-tile GPU time via the existing `timing.rs` (not just under
   `--profile`): end-of-render summary `iterate: N tiles, X.Xs GPU, Y.Y Gsteps/s`. The
   steps/s figure is the metric the F3-vs-us gap is measured in.
2. **Perf log.** `FRACTADYNE_PERF=1` appends JSONL records (per render: view depth, mode,
   iterations, tile count, GPU ms, steps/s) to `logs/perf.jsonl` — regression tracking across
   builds becomes greppable history instead of memory.
3. **GPU event counters.** A small storage buffer of atomics incremented by the shader:
   rebases, extended-sample decodes, glitch flags, BLA skips taken/levels, max-iter exhaustions.
   Read back per render; shown under trace and in the perf HUD. This is the F4 dead-code
   detector and doubles as a tuning instrument (e.g. BLA skip efficiency at depth).
4. **`--profile` deep-interior region.** Add a corpus-14-class region (dip-carrying orbit,
   no-BLA-skip, latency-bound) — the regime where the 50× gap lives is currently not profiled
   at all.
5. **HUD additions.** Budget, measured iterate ms, steps/s, counter summary — the live view of
   D3.1–D3.3.

### D4 — Trace unification & documentation (cheap, do alongside D1) — ✅ SHIPPED v0.2.7 (see DIAGNOSTICS.md)

1. **Categories + timestamps.** `FRACTADYNE_TRACE=req,ref,gpu,tile` (or `1` for all); each line
   stamped `[+12.345s]`. The existing `fd-*` prefixes become the category names.
2. **Coverage.** `fd-req` (render manifest) prints for tour frames too; `fd-ref` gains the
   reuse/extend path. The render manifest (center digits, zoom, iterations, mode, ss, size)
   also prints **un-gated at info level** for one-shot CLI renders — F8 dies permanently: a
   home-view render would have announced itself on the first line.
3. **`DIAGNOSTICS.md`.** One page documenting: env vars (`FRACTADYNE_TRACE`, `FRACTADYNE_PERF`,
   `FRACTADYNE_LOG`), CLI flags, the probe tests (`probe_orbit`, `probe_escape` and their
   `PROBE_*` env specs), log/crash-file locations, and the reading list for common symptoms
   (uniform frame → check interior vs escaped first; byte-identical output → counters; hang →
   breadcrumb + progress).

## Sizing and order

| Phase | Est. size | Order rationale |
|-------|-----------|-----------------|
| D1 | ~250–350 lines | Highest measured cost (F1+F2+F3 ≈ multiple sessions); everything else reports *into* it |
| D2 | ~150–200 lines | Two spurious-failure incidents + the current dark run; cheap once D1's breadcrumb exists |
| D4 | ~50 lines + doc | Mostly formalizing what already proved itself |
| D3 | ~200–300 lines | Feeds the open throughput investigation; the counters unlock D2.6 |

Suggested landing: D1+D4 together (one release), then D2, then D3 alongside the export-
throughput work that consumes it.

**Outcome (2026-07-11):** landed as v0.2.7 (D1+D4), v0.2.8 (D2 remainder), v0.2.9 (D3+D2.8);
operator manual in DIAGNOSTICS.md. Notable: the v0.2.7 panic hook's first catch was a real
pre-existing bug (stamp_watermark clamp panic at h<80); the D2.8 counter checks initially
passed the full suite while failing filtered — state leakage (F13) observed live and fixed
by pinning reference length per check. Open: D1.6 minidumps (no hard fault yet observed),
live-path counter readback (needs the non-blocking pump), F13's full fix (per-block config
ownership), F17 per-tile attribution.

## After this plan (2026-08)

The plan's own failure catalog was written from incidents on one machine. The next class of cost
came from the opposite direction — behaviour that differs *between* machines — and the tooling
added since is aimed there. Recorded here because it is the same idea (build the instrument the
failure demanded), not a new plan:

- **`--gputest`** — the WGSL df32/floatexp primitives against CPU oracles, swept over every
  backend. Built because "df32" was silently degrading to plain `f32`: NVIDIA's shader compiler
  folds the error-free transforms that make double-float more than float, on all three backends,
  and no image-level test could have localized that. AMD's Vulkan/GL preserve them, which is how
  we know the shader source was right all along.
- **`--uitest`** — scripted UI + live-render walk with screenshots; the harness that caught the
  deep-band capture racing a progressive reference build.
- **Help → Diagnostics…** (`ui/diagnostics.rs`) — the self-test and UI test run from the
  interface as child processes, with the result attachable to an issue report. The audience is
  people testing on hardware we do not own, who will never pass a CLI flag.
- **`scripts/gpu-validate.ps1` / `.sh`** — the whole battery in one command, same steps and file
  names on both OSes, hermetic so two machines' bundles are comparable.
- **Cross-GPU tolerance** on the goldens and the path-signature baseline (see ARCHITECTURE.md
  §12). The lesson worth carrying: a gate that reddens on every non-reference GPU trains testers
  to ignore it, so it stops reporting anything at all.
