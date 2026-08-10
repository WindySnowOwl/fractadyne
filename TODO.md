# Fractadyne — Development Tracking

## ▶ Announce readiness (fractalforums) — TRIAGE 2026-08-08

Goal restated by the user: a release **stable enough to announce publicly**, meaning (a) no easily
reproducible bugs, (b) no fundamentally missing features, (c) something substantively new in the
landscape, and (d) a runnable Linux build. Head is **v0.2.40-beta.50**, pushed; the latest
*published* prerelease is still beta.27 (28–50 are untagged — GitHub builds fire only on request).
⭐Status 2026-08-09: the A-list is essentially clear — A2 fixed (beta.47), A3 downgraded, the 2:58
device loss + black deep holds resolved (beta.49/50, livetest baseline all-green); A1's remaining
half (explicit iteration count still capped live) and the A4 ghost are what is left. Windows is
stable enough that **B (Linux) is unblocked** per the user's sequencing.

### A — BLOCKERS: easily reproducible bugs

Ordered by how fast a stranger hits them.

1. **An explicit iteration count is silently overridden** — PARTLY FIXED v0.2.40-beta.47; the
   remaining half is blocked, see the open item below. The cost model no longer collapses the
   frame: at an explicit 10,000,000 iterations the view now renders at full native resolution
   (measured 1445×1134, budget climbing 4.0e8 → 3.0e11 across ~160 readings) instead of the 16×16
   shrink floor an earlier attempt produced. What is still capped is `gpu_iter` itself — the status
   bar still reads `iter 82,741` — because removing the `boosted_cap` clamp lands on scripted
   playback, which sets `auto_iter = false` for every keyframe carrying a `max_iter`.
2. **A GPU without `TIMESTAMP_QUERY` renders at ~1/3 resolution forever — FIXED
   v0.2.40-beta.47.** Reproduced on the dev 3080 via the new `FRACTADYNE_NO_TIMESTAMPS=1`, which
   declines the feature the adapter offers: a 1445×1134 panel at 2,000 iterations rendered at
   **504×396 for 167 consecutive settled frames**, `budget=4.000e8` pinned at the bootstrap,
   **zero readings ever**, `can_tile=false`. Same view on the fix: **native 1445×1134**, tiled.
3. **The grand tour's own 1e55 hold spins forever — DOWNGRADED, could not reproduce
   (2026-08-09).** Two real holes in the refusal machinery were found and closed (see the open
   item below), but the reported symptom — the identical build repeating every ~450 ms — does not
   occur on the current build in any harness or window size tried. On the gauntlet the extension
   is refused **once** at each of 258,193 / 408,193 / 508,193 / 2,008,193 and the tour moves on.
   Not a blocker on this evidence; still worth a look if it recurs in the field, where the
   original report came from.
4. **Unexplained `0xc0000005` access violation** (08-07 17:00:25). One occurrence, no reproduction,
   no report at the time. Cannot block indefinitely on a ghost — but must not ship blind either.
   beta.43's unclean-exit reporting will now capture a recurrence; run the tour/export paths under
   load and see if it returns.

### B — BLOCKERS: runnable Linux build (d)

5. **Release artifacts** — `release.yml` needs an `ubuntu-22.04` job producing tar.gz + sha256.
   CI now proves the full workspace *compiles* there (2026-08-08, all jobs green, no code changes).
6. **Real-hardware validation** on the new rig, across the swappable GPUs — this is where (2) and
   the orbit-cap differences surface. ✅**The Linux machine is AVAILABLE as of 2026-08-09**, so this
   is unblocked and waiting on the Windows build stabilising first (the user's sequencing). Highest
   value once it starts: the no-`TIMESTAMP_QUERY` path is no longer theoretical — `IterTiming` is
   absent on the GL backend and on several Mesa/RADV/ANV combinations, and the beta.47 wall-clock
   fallback and settled-resolution invariant have only ever been exercised there via
   `FRACTADYNE_NO_TIMESTAMPS=1` on an NVIDIA card. Run `--selftest --selftest-filter live-res` and
   a `--play` of the grand tour on each GPU, and record adapter + resolved tunables per card.
7. **Four diagnostics are Windows-only stubs** — `process_memory`, `free_disk_bytes`,
   `cpu_topology`, `gpu_vram_bytes` return zero/None on Linux, so crash reports lose their memory
   line and the new disk-space warning never fires. Shipping Linux without these means shipping the
   platform where we are blind.
8. **`--version` is unrecognized** — prints `unrecognized option` then help. Trivial; it is also the
   first thing anyone types.
9. **CI cannot run the selftest** — the harness builds an eframe event loop, so it needs a display
   (`xvfb-run`). Until that lands, the Linux job gates compilation only.

### C — BLOCKERS: announce hygiene

10. **CHANGELOG stale** — the `0.2.40-beta` entry stops at beta.15; ~30 betas undocumented,
    including everything a reader cares about. Release notes come from here.
11. **Fresh-install smoke test** — ✅RUN 2026-08-09 (beta.51): wiped config dir, booted, verified
    classic home view (session records center (-0.5, 0) at classic zoom), all menus present,
    responsive, clean exit writes a valid session. ⚠A PrintWindow capture of the wgpu surface can
    show the view composited off-centre — capture artefact (endemic DWM), the session/viewport is
    correct; don't chase it from screenshots alone.
12. **Support website + email (user, 2026-08-09): `fractadyne.org` / `feedback@fractadyne.org`.**
    Update the in-app issue reporting (`ISSUES_URL` flow keeps GitHub Issues primary; the mailto /
    "Compose in Gmail" fallback currently points at `fractadyne@rithea.com` → change to
    `feedback@fractadyne.org`), the Help/About text, README, and the release-notes footer. Do this
    once the site/mailbox actually exist — a dead support address is worse than none.
13. **Pre-announce code hygiene review (user, 2026-08-09)** — a dedicated pass for dead code paths
    and refactoring opportunities for readability/maintainability. Natural candidates already
    known: the retired refusal machinery around `ref_ext_refused` (freeze guard v2 made the
    writer unreachable — the readers and `refusal_survives_install` are now vestigial), the
    `REFACTOR-PLAN` Phase 2/4 `too_many_arguments` markers, `cargo +nightly udeps` /
    `cargo-machete` for unused deps, and a `#[allow(dead_code)]` sweep. Run clippy at a stricter
    level and read what it says rather than silencing it.

### D — STRONGLY RECOMMENDED, not blocking

12. **First-run overlay** (design already agreed) — deep-zoom apps are opaque to newcomers and this
    is exactly the first-two-minutes friction a forum reader hits.
13. **Script playback is buried and forgets what you played.** Two small UX fixes to the feature
    that is one of the announce pitch's headline items (F), currently reachable only via
    *Tools → Play script…* (`ui/menus.rs`, `load_script()`), which opens a file dialog every time:
    - **A play-script button on the toolbar**, beside the existing icon actions in `ui/menus.rs`.
      The tours are the most demo-able thing in the app and right now nothing on screen suggests
      they exist.
    - **Default to the last script played.** Persist the path in `fractadyne-state` (the
      `export_dir: Option<String>` field is the precedent) so the toolbar button replays it with
      one click, and the menu item's dialog opens in its directory. Re-playing the same tour is
      the overwhelmingly common case — during this session's crash-hunting it meant a file dialog
      per attempt.
    - **A playback-speed control beside it.** ⚠One already exists — a `1×` button in the floating
      transport that CYCLES through speeds (`draw_playback_transport`, `ui/menus.rs`) — so this is
      about reach, not capability: the speed is only available once a script is already playing and
      only by clicking through values. Wanted on the toolbar, and worth reconsidering as a direct
      picker rather than a cycle. ⚠The transport's layout rule still applies to anything added
      there: nothing inside it may be sized from the available width, and the speed label is padded
      to its widest form so the widget cannot change size mid-playback.
    ⚠A missing or moved file must fall back to the picker rather than erroring, and the stored
    path should not be treated as trusted input (same handling as any loaded `.toml`).
    - **A progress bar in the Render Script dialog** (user, 2026-08-09). The 🎬 dialog launches
      the render as a child process (`--render-tour`) and today shows only text; the child already
      emits per-frame progress lines on stdout, so the dialog should parse them into a bar with
      frames-done / total, elapsed, and an ETA. ⚠The same stdout pipe is implicated in the silent
      4K render death (child died with `failed printing to stdout` when a parent went away —
      Open bugs) — whatever reads the pipe must tolerate the child outliving the dialog and vice
      versa, so do the two together.

### E — (b) fundamentally missing features: assessed, nothing blocking

Nothing *fundamental* is absent for a deep-zoom explorer. The honest gaps, all defensible to name
in the post rather than fix: no custom-formula scripting (Ultra Fractal's domain), no layers/blend
compositing, exports capped at the GPU texture size (KF tiles to wall-sized), SA covers Mandelbrot
+ Multibrot 3–5 only, no `.map` palette import yet (blocked on sample files), 3D explicitly out of
scope. The gradient editor, bookmarks, dual Julia, tours, `.fdn`/`.kfr` import and the export
pipeline are all present.

### F — (c) what is substantively new — the pitch

The **adaptive iteration budget** (nobody else raises the pixel budget from live GPU feedback),
**Newton-Raphson zoom** onto nuclei with the atom-size estimate, the cursor-driven **dual Julia at
depth**, the **script/tour system** with per-keyframe budgets and chapters, and the **F3-matched
corpus** offered as evidence rather than a boast. Secondary but unusual: the app explains its own
rendering limits in the status bar instead of leaving a black screen unexplained.

### Explicitly deferred, named not hidden

The 1e95 depth wall (self-reporting), the glitch-correction pathology, the offline tour path's
unbounded per-frame cost and memory, the autopilot rework, the tour editor, and the proposals
backlog.

Living backlog. Specs: [DESIGN.md](DESIGN.md), [UI-DESIGN.md](UI-DESIGN.md).
Mockups: [design/mockups/](design/mockups/).

## Open bugs

- [x] ⭐⭐⭐**RESOLVED v0.2.40-beta.49 (2026-08-09): the 2:58 device loss AND the black deep holds
  — one root cause, `design/reference-lifecycle.md` § 6 is the authoritative account.** Short
  form: `Playback::sample` ROUNDED a pinned-centre glide's centre to the current depth's precision
  (`lerp_bf(a, a, ease, p)` is not the identity), and every mysterious orbit length in this ledger
  is `escape(round(centre, p)) + 1` — the crash's 626 is the 78-bit rounding, hold-e72's "602,516
  accumulation" is the 206-bit rounding. Fixes: exact pinned-centre glides + the picker's cliff
  rescue (`best_reference_diag`, selftest group `ref-pick`) + the freeze guard now INSTALLS long
  partials at the floor budget (retiring the A3 refusal loop; e61/e63/e82 went green, baseline
  re-blessed). Verified: 0 losses in 4 fresh-dir runs (base rate was 3-in-4), 0 Event 153,
  e21000 soak clean, suite 106/106, goldens 17/17, bench-matrix 0 drift.
  ⚠Watch items: the incumbent/challenger cache (plan § 3 L2) is DEFERRED until the live
  `bla_skip` counters ever show a pinned-unfit reference again.
  **beta.50 follow-up (field report "past 3:35 it pixellates badly"): the install derate was
  firing on every deep-glide install** — truthful references grow ×2.3 per lookahead install
  where the old cliff orbits never changed length, so `big_jump` had never been exercised on a
  healthy stream. Pure partial→partial growth now does not derate (step-denominated budgets
  absorb clamp raises via resolution exactly); derates remain for clamp lifts and escaped-orbit
  jumps. Zero deep-chapter derates after; all six holds +0.0 pt; livetest `res` demoted from
  gate to context (flapped 4× in one day); the blessed baseline now carries NO recorded FAILs.
  ⚠e94 runs at the tour's own `max_iter = 4M` ceiling (view wants ~8M) — raising it is gated on
  the offline tour-render OOM item, not on the reference work.

- [ ] ~~⭐⭐⭐REDESIGN PLANNED~~ (superseded by the resolution above; kept for the forensic
  record) — **read `design/reference-lifecycle.md` FIRST; it supersedes the
  next-step lists inside the entry below.** 2026-08-09 (late): the mechanism is now fully
  measured, and it rewrites several of the entry's conclusions:
  - **A reference orbit's usable length is a function of PRECISION, with cliffs** (new experiment
    `escape_length_vs_precision` in fractadyne-core, `--ignored`): at the exact three-spar centre,
    128 bits → escapes at 570 · 160 → 84,941 · 207 → 570,711 · 286+ → survives 700k. Every
    "reference naturally escapes at N" in the spar family is a precision cliff, not a fact about
    the location. The `octaves + 64` precision policy is iteration-blind.
  - **`best_reference` goes blind below the cliff**: scoring at the caller's precision, every
    candidate "escapes" < the 4,096 quick scan ⇒ no survivor ⇒ longest-escaper fallback ⇒ a
    626-class pick. Fed adequate precision it picks the centre and is healthy (offline 26,465
    partial @286 bits; live 52,648 partial later in the same tour).
  - **The 626 entered via a `prec=78` lookahead build** (wall t≈172; the pacer-dilated clock puts
    the tour shallow then) and **reuse pinned it** — prec gate 206 ≥ 158 passes, drift ≈ 0, no
    fitness term — through the crossover into the device loss.
  - **A populated config dir MASKS the bug**: `refcache_persist` restores the previous session's
    deep reference, so the dive extends it and never picks badly. This explains the fresh-dir
    repro ingredient AND the 2-in-2-then-0-in-12 intermittency (those 12 runs reused the sandbox
    dir). ⚠And the reverse: `build_saved_ref` only skips PARTIAL references, so an escaped 626
    WOULD be persisted and poison the next session.
  - Plan: L0 pick/reuse traces → L1 escape-plateau probe + picker rescue (root fix) → L2
    incumbent/challenger cache (adopt-if-strictly-better; no destruction, so the e72 deadlock is
    structurally impossible) → L3 test/protocol hardening. Acceptance gates in the doc.

- [ ] ⭐**PRIORITY: live device loss with a 626-sample reference — EIGHT instances, still not
  root-caused, but now REPRODUCIBLE ON DEMAND in ~205 s.** Field report 2026-08-09 (two crashes
  at 250 s and 342 s of live tour playback) plus a deliberate A/B that crashed both builds. Every
  manifest across all eight shares one STATE, never one size:

      mode=2  orbit_len=626  partial=false  settled=false  boost=1.00

  a 626-sample ESCAPED reference against a ~27,000-iteration pixel budget — ~45 rebases per pixel
  and a 626-entry BLA that can skip essentially nothing.

  ✅**Not a regression from beta.47.** Settled by A/B on the same machine, same tour, same
  harness (`--play tours/grand-tour.toml`): **beta.46 died at 204.7 s, beta.47 at 205.9 s**, with
  identical manifests (236 vs 262 reference builds, 181 vs 187 of them `len=626`, 0 storm
  warnings). A crash on beta.46 is positive evidence and needs no reproducibility to stand, so
  this conclusion holds. `--play` is the only entry point that drives ON-SCREEN playback; neither
  `--livetest` nor `--divetest` has ever produced this.

  ⛔**IT IS INTERMITTENT AND CLUSTERED — do not trust a clean run.** Corrected the same session it
  was claimed: an earlier revision of this entry called it "reproducible on demand in ~205 s" on
  the strength of two consecutive crashes. The tally across every `--play` run that day was
  **2 crashes in the first 2 runs, then 0 in the next 9** (~25 minutes of playback), *including a
  re-run of the identical full grand tour as a control*. The two crashes came minutes after the
  user's own two, so all four sit inside one ~40-minute window; everything after it is clean.
  That points at a susceptible STATE (driver left degraded by a previous loss is the obvious
  candidate) on top of whatever the software trigger is.

  ⚠**Consequence: every negative result below is INCONCLUSIVE**, including the bisection. They
  were run believing the repro was reliable, and they only show the bug did not fire that time.
  Anyone resuming this needs a reliability model FIRST — run the unmodified grand tour N times,
  establish the base rate, and only then compare variants. Otherwise a "fix" is indistinguishable
  from the bug simply not firing, which is the exact trap the beta.38 entry warns about
  ("a null result on a bug that took 230–470 s to appear").

  Bisection attempted (all inconclusive per the above, none crashed): the real dive chapter alone
  (`t≥143`, 260 s, 1317 builds); features + gauntlet (`t≥95`, 300 s, 1347 builds); a hand-built
  dive with the dual-view / orbit-overlay / fractal-switch churn prepended; and a tour sweeping
  the e30↔e55 band three times at the grand tour's own glide rate (300 s, 2550 builds). All four
  reached the `len=626` precondition (~410 occurrences each). `scratchpad/trim_tour.py` produces
  prefix-trimmed copies of a v2 tour if the bisection is resumed.

  ⭐⭐**LOCALISED: "exactly 2:58" is the df32 → floatexp MODE TRANSITION, to the second.** Third
  field report 2026-08-09 gave the tour-clock time rather than an uptime, which is what cracked
  it — the earlier reports were wall-clock and the pacer dilates the clock, so nobody had tied the
  crashes to a tour POSITION. `PERT_FE_THRESHOLD` is 1e28 and the grand tour glides LINEARLY in
  log-zoom from 1e10 (t=153) to 1e30 (t=181), so it crosses at

      t = 153 + 28 × (28 − 10)/(30 − 10) = 178.2 s = 2:58.2

  and every recorded manifest interpolates to **1–5 s past it** (eff_iter 56 069 → t≈179.3,
  60 916 → 181.4, 63 600 → 182, 64 182 → 182.7). All of them are `mode=2`, i.e. already across.
  Six instances now agree. Note this also explains a dead end: the e30↔e55 sweep in
  `deep-dive-crash.toml` never fired because BOTH ends are above 1e28 — it stayed in mode 2 and
  never re-crossed.

  At the crossover, `budget_mode` derates the measured frame budget ÷8 and clears `fe_budget_ok`;
  measured here as 7.39e10→9.24e9, 1.05e11→1.32e10, 9.00e10→1.12e10 — landing squarely in the
  2.5e9–8.8e9 range the crash manifests record, with four of the five showing `steps` within 0.2%
  of `budget` (the shrink sizes the frame to exactly fill it). The tempting story is that the ÷8
  derate is smaller than the documented 10× data-dependence, so the first floatexp frames are
  mispriced. ⚠**But the measurement does not support it**: across a full oscillating run the
  slowest GPU iterate was **18.8 ms**, nowhere near the 900 ms target, because the motion
  res_scale keeps moving frames tiny. Frame cost stays ruled out. What the crossover changes
  besides the budget — reference/BLA rebuild, shader path, buffer and bind-group recreation while
  5–6 all-core bignum lookahead builds run per second — is the remaining surface.

  ⛔⛔**CORRECTION: it IS a driver-level GPU fault. My earlier "Windows logs no TDR" was a bad
  query, not a finding.** The System log carries **`nvlddmkm` Event ID 153** at the instant of
  every single loss — six events, six losses, **1:1**, and none during the ~12 clean runs:

      23:02:39 · 23:08:21 (field)   23:22:18 · 23:27:20 (the beta.46/47 A/B)
      04:46:47 (field)              08:02:37 (field)

  The earlier search missed them because it filtered on Event IDs 4101/4102/13/14 and a message
  regex, and Event 153 arrives with an EMPTY message body. There is no 4101 "display driver
  stopped responding", but there is unambiguously a GPU error. Frame cost is therefore back on
  the table, and the "not a TDR" reasoning built on top of the bad query is void.

  ⭐**And the frame rate collapses at the crossover.** From the 2026-08-09 08:02 report, with the
  new `since_mode_switch` instrument:

      mode switch 0→2  at +221.551s (frame 11838, mag 2^93 — the 1e28 threshold)
      DEVICE LOST      at +226.164s, `since_mode_switch=10 frames`

  **Ten frames in 4.6 seconds is ~460 ms per frame.** The instant it crosses into floatexp the app
  is submitting half-second dispatches — the ÷8 derate left the budget at 4.98e9 and the
  resolution shrink duly sized the frame to fill it. Against a ~2 s watchdog that is ~4× of
  margin, in the one regime where cost is least predictable (`orbit_len=626`, so BLA skips nothing
  and real cost tracks the nominal count instead of being a small fraction of it).

  ⛔~~**FRAME COST IS DEFINITIVELY OUT — and the mode-switch fix PROVED it rather than fixing
  the crash.** With the budget pinned to the bootstrap the very next crash (08:33, build 1273)
  read `135x106 ss=1 steps=3.969e8 budget=4.000e8` — the smallest frame the shrink allows, ~10 ms
  of GPU work on a 3080 — and died anyway, `since_mode_switch=8 frames`. A frame that small
  cannot be a 2-second dispatch. (The ledger had half of this already: one beta.36/37 crash died
  at `budget=4.000e8`, the FLOOR.)~~
  ⛔⛔**RETRACTED 2026-08-09 — see the beta.48 entry below. "~10 ms of GPU work" was an ASSUMPTION
  read off the step count, not a measurement, and it is wrong by ~80×.** The same false premise
  sits in the "budget=4.000e8, the FLOOR" observation and in every other place this ledger reasons
  from a small budget to a cheap frame. In this regime 4e8 nominal steps measures **~780 ms**.
  This file's own standing lesson — a nominal step is not a fixed amount of work — was written
  four bugs ago and I applied it to the resolution ratchet while still reading the budget as if it
  were milliseconds.

  ⭐⭐**ROOT CAUSE (mechanism identified, fix landed): LOOKAHEAD THREAD OVERSUBSCRIPTION STARVES
  THE RENDER THREAD.** The same 08:33 log: mode switch at +245.148 s, device lost at +248.986 s —
  **8 frames in 3.8 s, ~475 ms per frame, for ~10 ms of GPU work.** The frames were never
  GPU-bound; the main thread was starved. The starver is the reference lookahead:
  `playback_ref_prefetch` refills until the queue is full, so whenever it drains it spawns SIX
  builds in the same millisecond (five `building reference [lookahead]` lines share a timestamp in
  every crash log) — and `best_reference`'s candidate scoring uses `std::thread::scope` with
  `available_parallelism()` threads **per call, a fresh set each time rather than a shared pool**.
  Six concurrent builds therefore put six times the core count of bignum work against the
  renderer. The existing comment called this "harmless for compute-bound bursts"; that was the
  untested assumption, and beta.38 left "why a build storm takes the device out (CPU starvation of
  the driver?)" explicitly open. This is that answer.

  ✅**FIXED v0.2.40-beta.47: `PREFETCH_MAX_INFLIGHT = 2`** bounds builds IN FLIGHT (the queue still
  covers its full depth; slots fill in sequence). The effect on the dive is large and was
  measured with `--divetest`, not assumed:

  | band | before | after |
  |---|---|---|
  | 1e300 fps / p95 | 56.5 / 23.9 ms | **59.8 / 15.5 ms** |
  | 1e700 p95 | 15.9 ms | **9.9 ms** |
  | motion res @1e300 | 0.30–1.00 | **0.79–1.00** |
  | motion res @1e700 | 0.30–0.46 | **1.00–1.00** |

  The AIMD had been cutting motion resolution to 30% because frames measured slow — and they were
  slow from the oversubscription, not from GPU cost. Reference installs are unchanged (160), so
  the lookahead still keeps pace with the dive. **This is a real quality win independent of
  whether it cures the device loss.**

  ✅**Also kept, v0.2.40-beta.47: a mode switch now marks the budget UNMEASURED** rather than
  derating the old mode's measurement by 8 — the same invariant the no-`TIMESTAMP_QUERY` fix rests
  on, since a budget that has measured nothing *in this mode* may bound the first dispatch and
  must do no more. Measured: the crossover now goes `7.78e10 → unmeasured (bootstrap 4.00e8)`, and
  the slowest GPU iterate across a full oscillating run fell 18.8 ms → 14.0 ms. The controller
  re-climbs at ×1.5 per reading, so the cost is a few coarse frames during a dive, where motion
  resolution is already reduced. Suite 104/104, goldens 17/17, livetest 0 drifted.
  ✅**FIRST CLEAN PASS, 2026-08-09 (field, build 1275): the grand tour played through 2:58 and
  completed all 5:31 without a device loss** — the first full pass on record after six losses.
  Encouraging but NOT proof: the class is intermittent (2 in 2 runs, then 0 in 12 here), so one
  clean run is exactly the evidence the beta.38 entry warns against reading too much into. The
  base-rate measurement is still owed. `nvlddmkm` Event 153 remains the objective check.

  ⚠**Neither fix is VERIFIED against the crash** — it is intermittent (2 in 2 runs, then 0 in 12)
  and has not been reproduced locally since. The mode-switch change is now known NOT to cure it on
  its own; the in-flight cap targets the measured starvation and is untested against a live
  occurrence. `nvlddmkm` 153 is now the objective verdict signal and does not
  depend on the app: `Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='nvlddmkm'}`.

  ⭐⭐⭐**ROOT CAUSE, 2026-08-09 (build 1277, the fourth field crash — the one that carried BOTH
  beta.47 fixes): THE BUDGET'S LOWER CLAMP WAS `TDR_BOOTSTRAP_STEPS`, so the controller could not
  shrink below its own opening guess — and in this regime that guess is worth ~780 ms.**

  Build 1277 crashed at 2:58 with both fixes active, and its log is what cracked it because both
  fixes are visibly WORKING:

      mode switch  +219.821s → budget 7.78e10 → unmeasured (bootstrap 4.00e8)
      DEVICE LOST  +223.356s   since_mode_switch=7 frames
      lookahead spawns, last 4 s: 2, 1, 1, 1, 1      ← the cap holds (was 5–6 in one millisecond)
      frame: 146x115  steps=4.598e8  budget=4.625e8  orbit_len=626  partial=false

  So: lookahead bounded, budget unmeasured, frame 146×115 — and still ~500 ms per frame and still
  dead. That kills CPU starvation as the sole cause too. The tell is the budget itself: it had
  GROWN 4.00e8 → 4.625e8, a factor of 1.156, and the controller's factor is `900 / ms` — so a
  reading came back at **778 ms for a 4e8-step frame**. The frame was GPU-bound all along.

  `TDR_BOOTSTRAP_STEPS`' doc comment claimed 4e8 was "a few ms even on a GPU orders of magnitude
  slower than a desktop discrete part". At `mode=2` with `orbit_len=626` — an escaped reference so
  short that BLA skips nothing, the exact state all nine crashes share — it is ~780 ms on a 3080.
  Roughly 80× off, and every conclusion drawn from "the budget is small so the frame is cheap"
  inherits the error.

  Then the mechanism, which is the part that actually explains the deaths: the clamp was

      next.clamp(TDR_BOOTSTRAP_STEPS, TDR_STEPS_CEIL)

  with the OPENING GUESS as the FLOOR. Where that guess is worth 780 ms rather than a few ms, the
  controller measured slow frames, tried to shrink, and hit the floor every time. It then sat
  submitting ~0.8 s dispatches back to back — about 2× from the ~2 s driver watchdog, in the one
  regime where cost is least predictable — and any data-dependent spike takes it over. **A safety
  valve that cannot move in the direction of safety.** It was working exactly as designed; the
  design had no way down.

  This also retires the apparent paradox that made the class so confusing: the crash manifests
  cluster at tiny resolutions and near-floor budgets *because the controller was pinned at the
  floor*, and tiny resolution was read as evidence of a cheap frame when it was evidence of a
  controller in distress.

  ✅**FIXED v0.2.40-beta.48: `TDR_MIN_STEPS = 4_000_000`** (100× below the opening guess) is the
  new clamp floor, in `budget_step`, in the `tdr_steps` read-back that would otherwise re-raise a
  below-bootstrap budget, and in `install_recompute`'s discontinuity derate. The bootstrap is now
  documented as an opening guess, not a floor, with the 780 ms measurement recorded against its
  old claim. Property test `the_budget_can_shrink_below_the_opening_guess` pins the behaviour;
  `budget_never_leaves_its_clamps` updated. The resolution shrink's own 16×16 floor still bounds
  how small a frame can get, so the valve cannot collapse the view.

  ⚠**UNVERIFIED against a live occurrence, like the two before it** — the class is intermittent
  and has not reproduced locally since the first cluster. What is different this time is that the
  fix rests on a direct measurement of the failing quantity (778 ms at 4e8 steps, derived from the
  controller's own growth factor) rather than on a mechanism inferred from timing. `nvlddmkm`
  Event 153 remains the objective verdict.

  ⭐⭐⭐**REPRODUCED LOCALLY, 3/3, AND ATTRIBUTED — 2026-08-09. The frames are GPU-bound: ~1 s of
  GPU for a 69×54 frame, with the CPU idle.** This is the first on-demand local repro this class
  has ever had. Recipe, exactly:

      rm -rf $SANDBOX && FRACTADYNE_CONFIG_DIR=$SANDBOX         ./target/release/fractadyne.exe --play tours/grand-tour.toml

  A **fresh** config dir is the ingredient the earlier 0/12 attempts were missing. It dies ~3.2 s
  after the df32→floatexp switch, at +203.9–204.0 s, consistently to the tenth of a second.
  (`use_bla = true` in the sandbox session, so this is not the fresh-sandbox-has-BLA-off trap.)

  ⛔⛔**MANDATORY HYGIENE — `taskkill /F /IM fractadyne.exe` BEFORE AND AFTER EVERY RUN.** The
  device-loss handler AUTO-RESTARTS the app, and that restarted child is not a descendant of the
  `timeout` that launched the original, so **every crash leaves a survivor that outlives the run
  and races the next one.** Two GPU-heavy instances on one device is its own plausible cause of a
  loss, so a contaminated run proves nothing. Caught by the user, not by me: run 2 of a back-to-back
  pair had visibly interleaved out-of-order timestamps (201.770 → 201.370 → 202.121) because two
  processes were writing one log. ⚠**My first "3/3, reproducible on demand" was contaminated** —
  runs 2 and 3 raced run 1's survivor. Re-measured with kills between runs: **3 device losses in 4
  clean runs.** Still much the best repro this class has had, but it is ~75%, not deterministic,
  so a single clean run STILL does not clear a fix.
  → Worth fixing in the product too: auto-restart is right for a user whose GPU hiccuped and wrong
  for `--play` / automation, where it silently forks the experiment. Suppress it when a tour is
  being driven headlessly.

  The new `slow frame` attribution log settles what a frame interval never could — whether a long
  frame is CPU work inside `update` or a block outside it:

      slow frame 11267: dt=1018ms = body 0ms + outside(acquire/present/idle) 1018ms — mode=2 steps=9.860e7 budget=0.000e0
      slow frame 11268: dt=1018ms = body 2ms + outside 1016ms
      slow frame 11269: dt=1018ms = body 1ms + outside 1017ms
      DEVICE LOST

  **Zero CPU. One second blocked on the GPU, three frames running, at 69×54.** So:

  ✅**Confirmed blocked, not idle.** `1018/1016/1017 ms` three frames running looked like a 1 Hz
  timer — eframe falls back to ~1 Hz when the app requests no repaint, and an idle interval
  measures nothing about cost, so this had to be settled before reading anything into it. The log
  now carries `repaint_requested`, and it is **true** on every one of these frames: the app is
  asking for the next frame and not getting it. Genuinely blocked outside `update`.

  ⛔**Glitch correction is OUT.** It was the strongest remaining lead — `glitch_correct = true` in
  the session, `glitch=367` counted at this view offline, and a documented >1 h glitch-correction
  pathology already in this file. Tested by flipping it off in the sandbox session: **1 loss in 2
  runs, the same rate as with it on.**

  - ⛔**CPU starvation is OUT** as the cause — it was the beta.47 hypothesis and the lookahead cap
    it produced is a real quality win, but `body 0ms` ends it. Frame cost is IN, definitively.
  - ⛔**The resolution lever cannot reach this.** 69×54 is 3,726 pixels — ~2% occupancy on a 3080.
    At that width the pass is latency-bound, so shrinking further trades parallelism for nothing;
    `steps ∝ time` is false here exactly as [[topic-deep-zoom-gpu]] warns.
  - ⛔**Neither can the budget.** `budget=0.000e0`: no timing EVER came back before the device died.
    A healthy timestamp readback lands 2–3 frames later, and at ~1 s/frame that is ~3 s — longer
    than the watchdog allows. **The control loop's feedback is slower than the failure.** No value
    of `TDR_BOOTSTRAP_STEPS` fixes that: 4e8 died, 1e8 died.

  ⭐**The number to explain: 9.86e7 steps in 1018 ms = 9.7e7 steps/s.** The same view offline
  measured ~3.4e9 steps in ~0.4 s = **8.6e9 steps/s, 88× faster** — but offline reported `mode=0`
  at this zoom and a sweep of `--zoom-log2 92 → 96` showed no cliff, so the offline path may never
  enter mode 2 and that comparison is NOT yet apples-to-apples. Also measured there, and probably
  central: `bla_skip=0` with **46 rebases per pixel** (`rebase=5962666` over 129,600 px) — the
  626-sample reference means BLA skips nothing and every pixel grinds 26,464 iterations serially.

  ⭐**Step 1 DONE, and it deepens the mystery: offline mode 2 at 69×54 is FREE.** Same centre, same
  `--iter 26464`, `--zoom-log2 94` (94 not 93 — `magnification()` depends on the viewport, so at
  93 the mode flips with resolution), wall time minus the ~1500 ms constant of app start +
  reference build:

  | size | 69×54 | 138×108 | 480×270 | 960×540 |
  |---|---|---|---|---|
  | mode-2 render | ~0 ms | ~11 ms | ~196 ms | ~1403 ms |

  It scales with pixels, so the pass is NOT latency-bound at these widths, and the resolution lever
  does work — offline. **The live path spends 1018 ms where the offline path spends ~0 ms for the
  same view, mode, and iteration count.** That ~1000× gap, not the shader, is the bug.

  Ruled out while finding this: `ensure_orbit_capacity` only ever grows and rounds to a power of
  two, so at 626 samples it is not reallocating the ~1 GB cap buffer per frame.

  ⛔⛔**CORRECTION 2026-08-09 (late): the "period-626 nucleus" reading below is WRONG, and it came
  from a DEAD DIAGNOSTIC.** `finish_reference` sets `orbit_tail = Some(tail)` unconditionally
  (deliberately — an escaped orbit cannot be extended but can still be reused as-is, so a rebuild
  does not re-pick and make the view jump on zoom). The trace then printed
  `escaped = orbit_tail.is_none() && !partial`, which is therefore **hardcoded false for every
  reference ever logged**. `partial = !tail.escaped`, so `len=626 partial=false` means the orbit
  **ESCAPED at 626** — the original reading in this ledger was right. Fixed: the trace now prints
  `tail.escaped`. ⚠A diagnostic that cannot print one of its two values is worse than no
  diagnostic; it survived because nobody cross-checked it against `partial`, which was sitting on
  the same line saying the opposite.

  ⭐**What that same code path DOES reveal: an escaped reference is reused FOREVER.** Reuse is
  gated only on `orbit_tail.is_some()`, which is always true, with no check that the reference
  still suits the frame. That is why `len=626` is byte-identical across every build, every run and
  every field crash — one bad point, pinned, while the dive's appetite grows 600 → 34,000.

  ⛔⛔**ATTEMPTED FIX, REVERTED THREE TIMES — do not reach for a reuse-refusal predicate again
  without reading this.** `reference_reusable(...)`, refusing to reuse an escaped reference once
  `ask/len` exceeds N traversals: tried N=2, N=8, and N=8 plus a `len >= fresh_cap` guard. **All
  three regressed `--livetest hold-e72` from 0% → 100% black**, with byte-identical numbers every
  time (`orbit 602516 → 256001 PARTIAL`, budget 1011664 → 256000). Goldens 17/17 and
  `--bench-matrix` 0 drift throughout, so it is live-path-specific.

  ⭐**Mechanism, now understood (`try_reuse_reference` + `extend_reference_orbit`):**
  - An escaped orbit is returned by `extend_reference_orbit` UNCHANGED — it can never grow. But it
    can be LONG, because it grew by extension while it was still PARTIAL and escaped during one of
    those extensions. That is e72's 602,516-sample escaped reference, and it answers the question
    the previous revision left open.
  - **Refusing reuse is DESTRUCTIVE AND UNRECOVERABLE within a tour.** The 602k orbit is
    accumulated across the whole descent, one extension per rebuild. Refuse once, anywhere in the
    climb, and the replacement starts from a fresh build capped at `LIVE_REF_CAP` = 256,000 and
    never catches up — which is why the third attempt's `len >= fresh_cap` guard did not help: the
    refusal that does the damage fires EARLIER, while the reference is still short.

  ⛔**So the lever is wrong.** A predicate can only choose between "keep the bad reference" and
  "throw away the accumulation"; both are losing moves. Any real fix has to avoid the destruction:
  pick a new reference while KEEPING the old orbit until the new one is at least as good, or make
  a re-pick able to seed from the cached orbit. That is a design change in the reference cache, not
  a threshold. ⚠And whatever it is, the cost that actually kills is `bla_skip=0` at 54 traversals —
  verify against THAT counter, now that the live path publishes it.

  ⚠**`--livetest` FLAPS.** Twice today it reported `1 drifted` and then `0 drifted` on an
  IDENTICAL binary. The gate was narrowed to outcome-only fields precisely to stop this, so the
  narrowing is insufficient. Re-run before believing a single drift — and fix the flap, because a
  gate that cries wolf is one that gets ignored (its own doc comment says so).

  ⭐⭐⭐**THE ~90× IS THE REFERENCE, NOT THE PASS. Live and offline pick fundamentally different
  references for the SAME view, and the live one is a period-626 periodic orbit.**

  | | reference | kind | BLA | cost |
  |---|---|---|---|---|
  | offline `--render` | `len=26465 prec=286` | linear, `partial=true` (hit the cap, never escaped) | `bla_nodes=211724`, **`bla_skip=298829`** | **11 ms** @138×108 |
  | live at the crash | `len=626 prec=207` | **periodic**, `partial=false escaped=false` | `bla_nodes=5020` | **1018 ms** @~141×106 |

  `escaped` is logged as `orbit_tail.is_none() && !partial` (render.rs:503), so `partial=false
  escaped=false` means an orbit TAIL exists — the reference is a minibrot nucleus of period 626,
  not a truncated or escaped orbit. ⚠**This corrects an assumption carried through every earlier
  entry in this ledger, including mine hours ago: `len=626` was read as "an escaped reference too
  short for BLA to skip anything". It is a periodic reference, which is normally the BEST kind.**
  The 626 is constant across every build, run and field crash because it is a PROPERTY OF THE
  POINT — the same nucleus, correctly re-found each time.

  It is the right reference for pixels needing ~626 iterations and the wrong one for pixels needing
  **26,464** — ~42 wraps of the cycle, with a BLA tree that can only span 626 samples. The offline
  path sidesteps it by building a long linear reference instead. So `best_reference` is scoring
  "is this a good nucleus?" when the question at this view is "what will this cost the pixels?".

  **That reframes the whole class.** The device loss is downstream: a ~1 s dispatch every frame,
  ~2× from the watchdog, is what a period-626 reference costs at 26,464 iterations. Fixing the
  reference choice removes the cost, and with it the loss — and it is a rendering-quality fix
  independently, since these are the same holds that render black.

  ⚠Also re-confirmed here: **tracing suppresses it.** This traced run did not die (`device_lost=0`)
  where the four untraced clean runs before it went 3/4. Reproduce untraced, then instrument.

  **NEXT, in order** (all now cheap — there is a 4.5-minute repro):
  0. ⭐**Score references by what they will COST the pixels, not by nucleus quality alone.** When
     the iteration appetite greatly exceeds the period, a periodic reference needs `appetite/period`
     wraps and its BLA cannot span them; either extend it linearly (the offline path's shape) or
     prefer a longer non-periodic candidate. Check what `bla_skip` the live path actually achieves
     first — `bla_dc_max_log2=-88.1` against pixel offsets of ~2^-93 says the BLA radius COVERS
     these pixels, so if skips are still near zero the wrap is defeating the tree and that is the
     specific thing to fix.
  1. Find what the live frame submits that the offline frame does not. `body 0ms` +
     `repaint_requested=true` means the main thread is blocked in acquire/present, so the GPU has
     ~1 s of work queued from SOMEWHERE — the iterate pass cannot be it. The gap is ~90×: the live
     frames measure `steps=3.95e8` at ~26 464 iterations ⇒ ~141×106, and offline 138×108 in mode 2
     costs **11 ms**. Candidates: a second view still live from the dual-view
     chapter, the minimap, the orbit overlay, a pass running at window resolution rather than the
     iterate resolution, or the iterate pass running more than once per frame. Instrument by
     labelling every submit in the frame and logging the set when `outside > 200 ms`.
     ⚠Note the offline CLI renders TWO images (`cli-render-map` + `cli-render-julia`), so the table
     above already includes a second view's cost — which makes the live number stranger, not less
     strange.
  2. Ask why the reference is 626 samples when pixels need 26,464. A reference that escapes at 626
     is a bad pick, and `best_reference` scoring exists to avoid exactly that. This may be a
     rendering-quality bug that happens to also be the performance cliff.
  3. Only then consider a-priori throttles. An a-priori bound is the ONLY defence available given
     point 3 above, so it has to be right rather than tuned.

  ⭐**One long-standing assumption is still DISPROVEN — do not re-derive it:**
  - **It is not reference upload traffic** (the previous leading hypothesis, which this file told
    the next person to check first). `orbit_id` re-uploads only `orbit.len() + bla.len()` ACTUAL
    bytes — at 626 samples that is ~10 KB per install, not the ~1 GB `orbit_len_cap` binding.

  Still ruled out: **a build storm** (0.7 builds/s, against the ~400/s beta.38 cured). ⛔The
  companion claim that **frame cost** was ruled out — "the eight span budget 4.0e8 → 3.0e11 and
  resolutions 135×107 → 4631×2060" — is RETRACTED: it compared step counts and pixel dimensions
  across regimes where a step costs wildly different amounts, so it never measured cost at all.

  ⚠**`validation/deep-dive-crash.toml` reaches the PRECONDITION fast** — `len=626` on ~410 of its
  reference builds, prec 327, the `len=258193` e55 extension, in ~120 s — which is useful for
  working in the regime. It has never produced the crash, but neither has the full tour since the
  susceptible window closed, so that says nothing either way yet.

  ⚠**Timing-sensitive**: a grand-tour run under `FRACTADYNE_TRACE=ref,tile,gpu` did not reproduce
  in 300 s where an untraced one died at 205 s. Reproduce first, then instrument, and treat one
  clean traced run as inconclusive.

  ✅**Instrumented for the next occurrence** (beta.47): the live crash manifest now carries
  `since_mode_switch=N frames`, and every arithmetic-mode change logs one always-on line
  (`arithmetic mode 0 → 2 at frame 3627 (mag 2^93.0)` — 2^93 = 9.9e27, the threshold). Deliberately
  NOT trace-gated: the class is intermittent and a traced run has already been seen to suppress
  it. The next field crash report will say outright whether the death is frames or minutes from
  the crossover, which is the difference between a coincidence and a cause.

  **Next lever.** The clustering is now the most informative thing known about this bug, and it
  is cheap to test: run the grand tour repeatedly and see whether a loss makes the NEXT run more
  likely (the app auto-restarts, so a susceptible driver state would persist across the restart).
  If it does, the software trigger and the susceptibility are separable and can be attacked apart.
  Only then is instrumenting accumulation worthwhile — iteration-texture recreations (the
  motion-res AIMD changes resolution constantly, and `ensure_orbit_capacity` only ever grows),
  bind-group and buffer churn, VRAM.

- [ ] **A 4K tour render died silently at frame 5682/9931 (~t=3:09), 2026-08-08.**
  `--render-tour grand-tour.toml --out H:\frames --size 3840x2160 --fps 30 --ss 4 --resume -y`,
  beta.46. Ran 70 minutes, wrote 5,681 frames (24 GB), then vanished between two progress lines.
  Both known render failure modes are excluded: **not disk** (H: had 159 GB free — the beta.45
  case) and **not memory** (rss flat 261→288 MB, peak pinned at 591 MB since early — the beta.34
  OOM case). Frame cadence was steady at ~2.6 s to the last line. No panic, no crash report, no
  WER entry, no TDR; detected only by beta.43's unclean-exit marker, which is that machinery
  working as designed.
  One thread worth pulling: the GUI **parent** that launched it also ended uncleanly, and a
  beta.45 report shows a render child dying on `failed printing to stdout: The pipe is being
  closed` when its parent goes away — though this child then ran headless for 70 minutes, so it
  is not a clean fit. A render child that outlives its parent should not depend on the pipe;
  worth making it write its own completion marker either way.


- [x] ⭐**The live-path harnesses could not tell "unchanged" from "still broken" — FIXED
  v0.2.40-beta.47.** Raised after the beta.47 work, where establishing what a *clean* run of
  `--livetest` looks like cost two full 331 s runs of a rebuilt pre-change binary. Skipping that
  would have shipped an iteration-boost regression; the harness had all the data and no opinion.
  Two additions, both aimed at the same gap — **goldens are OFFLINE export renders, so no golden
  can ever see a live-path bug** (an offline render has no panel to be a fraction of, no settle
  grid, no frame budget, no measurement loop):
  1. **`--livetest` is graded against a blessed baseline** (`benchmarks/livetest-<tour>-<WxH>.json`,
     recorded with `--bless`), the contract `--bench-matrix` already used. It fails on CHANGE, not
     on failure: per checkpoint it compares verdict, live/offline black, budget, appetite, boost,
     orbit length + PARTIAL, and resolution — excluding `stale_ms` and `t`, which are timings
     rather than outcomes. This matters because three of the grand tour's checkpoints fail for a
     documented reason, so the old "exit non-zero if anything failed" rule could never go green.
     Now: `22 checkpoint(s) · 0 drifted · 0 new`, exit 0, with the 3 known FAILs recorded.
  2. **A settled-RESOLUTION property check** (`--selftest --selftest-filter live-res`, 2 checks),
     which is the single assertion that covers the whole beta.40/41/47 family — each of those
     ended as a settled view pinned at a fraction of its panel and upscaled. It drives the real
     `build_params` at a 1445×1134 panel / 2,000 iterations with `fe_budget = 0` and asserts BOTH
     halves of the invariant: the arm frame stays inside `TDR_BOOTSTRAP_STEPS` (so an unknown GPU
     is still safe) and the settled frame reaches ≥90% of the panel. ⭐**Verified against the bug,
     not just against the fix** — re-introducing the pre-fix `can_tile` gate makes it report
     35% of panel; the dispatch-bound check passes either way, so the pair cannot be satisfied by
     simply deleting the bound.
  ⚠**Two harness traps found writing it, both of which make a live check silently measure the
  wrong path**: `frame_idx` must ADVANCE between `build_params` calls (the tile turn token is
  keyed off it, so a pinned index yields exactly one tile and then holds forever — which looks
  identical to the bug), and the reference orbit must be allowed to LAND (it builds off-thread, and
  until it does `will_reproject` forces `can_tile` off).

  Still open on the harness side: `--divetest` has no baseline either (it was compared by hand
  against a rebuilt binary in beta.47), and the F3 corpus regression gate
  (`generate_corpus.py --check`) is still outside `--selftest`/CI — that, rather than a new
  shallow floatexp golden, is the right way to cover depth, since the deep bugs in this ledger are
  depth-specific (e142, e148, e163) and a golden at e40 would guard a band nothing has broken in.


- [ ] ⭐**PRIORITY: tunables + hardware-dependent factors are individually well-reasoned and
  collectively fragile.** ⭐**USER REQUIREMENT (2026-08-09): centralize the constants — "I don't
  like having critical numbers buried in random blocks of code."** Concretely: gather the fixed
  constants below into one `tunables` module (each with its doc comment, unit, and the incident
  that set it — the per-site comments move with the values), and allow OVERRIDING them for
  debugging via CLI (`--set TDR_BUDGET_MS=500`), an advanced settings file, and/or a diagnostics
  UI panel — overrides logged loudly at startup and stamped into crash-report manifests so a
  report from an overridden run can never masquerade as stock behaviour. ⚠Overrides are a
  debugging tool, not a supported configuration surface: defaults stay the only tested path, and
  the selftest/bench gates run at defaults. Raised 2026-08-08 after a week in which most incidents came from this
  machinery. An inventory first — three tiers:
  - **Fixed constants**: `TDR_BUDGET_MS` 900, `TDR_BOOTSTRAP_STEPS` 4e8, `TDR_STEPS_CEIL` 3e11,
    `TDR_GROW_MAX` 1.5 / `TDR_SHRINK_MAX` 0.5, `TDR_MAX_TILES`, `LIVE_REF_CAP` 256k,
    `MAX_ITER_LIMIT` 10M, `ITER_BOOST_MAX` 256, `ITER_STALL_LIMIT`, `PACE_LAG_LO/HI`,
    `PREFETCH_OCT/SLOTS`, `BUILD_STORM_PER_S`, `PREFETCH_BUILDS_PER_S`, `WORK_BUDGET`,
    `PERT_FE_THRESHOLD`, `SETTLE_DELAY`, `zoom_iter_cap`'s `2000 + 256/octave`.
  - **Hardware-derived at startup**: `orbit_len_cap` (from `max_storage_buffer_binding_size`),
    `TIMESTAMP_QUERY` presence, GL backend → downlevel limits.
  - **Measured at runtime**: `fe_budget` (GPU timestamps), `iter_boost` (capped-fraction counter),
    `motion_res` (AIMD on the frame interval).

  ⭐**The bug history says the values are mostly NOT the problem.** Everything that broke falls
  into five shapes, and only the last is "a constant is wrong":
  1. **A measured loop with a path where measurement never arrives**, falling back to a bootstrap
     constant that then BINDS something important. Three incidents: beta.41 (a static view never
     measures → 205×162 upscaled), beta.40's `is_fe` measurement gate, and the `TIMESTAMP_QUERY`
     case below. The dominant family by a wide margin.
  2. **Wrong cost model** — nominal steps ignore BLA skipping and latency-bound interiors.
     `steps ∝ time` is documented as false in these very files and still drives the settled
     resolution bound (the 16×16 collapse; the `1451ms IGNORED` discard).
  3. **Inconsistent capability gating** — five of six `is_fe` sites.
  4. **Unbounded waits/retries** — the lag hold with no timeout, the refused-reference loop, the
     lookahead spin.
  5. **A constant tuned at one depth failing at the next** — the spar family's eight bugs.

  ✅**Plan items 1 and 3 landed in v0.2.40-beta.47** — the invariant, a wall-clock fallback, and
  property tests. Details under the two bug entries below; the short version:
  - **`FRACTADYNE_NO_TIMESTAMPS=1`** declines the feature the adapter offers, so the whole
    no-timestamp path is now reproducible on the dev 3080. It is what turned this from a
    reasoned-about risk into a measured 504×396.
  - **The invariant is enforced where it binds**: `can_tile` no longer requires a measured budget,
    and a never-measured budget may arm a grid (the bootstrap is a CONSTANT, so the "a grid sized
    off a climbing budget restarts every measurement" objection does not apply to it).
  - **Measurement starvation is detected and fallen back on**, tripping on OBSERVATION rather than
    on the capability bit — so a device that advertises `TIMESTAMP_QUERY` but never delivers a
    reading (i.e. every bug in family 1) is covered by the same mechanism.
  - **Property tests** (`render::controller_props`, 7 checks in `cargo test --bins`) pin the
    shapes rather than the incidents: a slow reading always shrinks the budget, a slow *undersized*
    dispatch pulls it down to its own size (the beta.40 defect), growth is bounded, the clamps
    hold, starvation is measured from the last READING not the last dispatch, and an idle view
    never trips it.

  Items 2 (centralise the tunables with declared contracts), 4 (unify settled + motion on measured
  cost — half done, see below) and 5/6 (capability probe as an instrument; the probe itself now
  logs one line and is in the crash report, but `--bench-matrix` does not yet record it) remain.

  ✅**Reproducible consequence, found 2026-08-08, FIXED v0.2.40-beta.47: a GPU without
  `TIMESTAMP_QUERY` renders at ~1/3 resolution forever.** `IterTiming::new` returns `None`
  without the feature → no timing is ever published → the controller skips on `bits == 0` →
  `fe_budget` stays 0 → `tdr_steps` sits at the 4e8 bootstrap → and beta.40 made that floor drive
  the resolution shrink in EVERY mode (1920×1080 at 2k iters = 4.1e9 steps ⇒ ~600×337). `can_tile`
  also requires `fe_budget != 0`, so the tiled settle cannot recover it. Invisible on the dev
  RTX 3080; **certain to appear on the swappable-GPU Linux rig** (older Intel iGPUs, some
  Mesa/RADV/ANV combinations, and any fall back to the GL backend, which has no timestamp queries).

  **Plan, in value order** — note that a startup calibration is *not* first:
  1. **One invariant retires the largest family**: an UNMEASURED budget may bound the first
     dispatch but must NEVER bind resolution or quality downward. Plus a wall-clock fallback when
     timestamps are absent — the AIMD motion controller already proves that works.
  2. **Centralise the tunables** with a declared contract each: what it bounds, how it is measured,
     what happens when measurement is unavailable, where it was last validated. The prose today is
     good but scattered, and the contract is implicit — which is why one mistake recurred in three
     places.
  3. **Property tests for the controllers**, in the style that pinned the lookahead sign: with
     measurement disabled resolution must not collapse; a slow reading must lower the budget; every
     wait terminates. Catches families 1 and 4 structurally instead of one incident at a time.
  4. **Unify settled + motion on measured cost** — retires family 2 and unblocks explicit
     iteration counts.
  5. **Capability probe at startup** — query the adapter once, log one line, put it in crash
     reports and bench baselines. ⚠**Scope it to capabilities and a SAFE STARTING POINT, never to
     the cost model**: cost here is data-dependent, not hardware-dependent — the same nominal step
     count measured 114 ms at one location and 1147 ms at another on the SAME GPU. A calibration on
     the home view would produce a confident number that is wrong exactly where the app crashes,
     and would justify a larger first dispatch than the deliberately tiny bootstrap that currently
     makes an unknown GPU safe.
  6. **Make the swappable-GPU rig an instrument** — extend `--bench-matrix` to record adapter
     capabilities and the resolved tunables per card, so a GPU swap yields a comparable profile
     rather than an impression.

- [x] **A 4K tour render died at frame 1091 and the dialog wouldn't say why — FIXED
  v0.2.40-beta.45.** The render itself was not at fault: the log had the reason in plain words,
  `tour FAILED: frame 1091: There is not enough space on the disk. (os error 112)`. Measured after
  the fact: 4K frames cost **5.28 MB each**, so 9,931 of them need **~51 GB**, against ~6 GB free.
  The render was impossible before it started and spent 1h28m proving it.
  Two real defects, neither of them the disk:
  1. **The GUI threw the reason away.** `draw_tour_render_dialog` spawned the child with
     `.stderr(Stdio::null())`, and the child reports failures through `diag::log_line`, which
     writes to STDERR. So the dialog could only ever repeat its last stdout progress line —
     "Render failed or was stopped. frame 1091/9931 (1h28m35s elapsed…)" — which says where it
     stopped and not one word about why. Now stderr is piped, error-ish lines are tagged
     (`RenderLine::Error`), the FIRST one is kept (later lines are consequences), and the status
     quotes it: "Render failed: … not enough space on the disk".
  2. **Nothing estimated the output size.** The dialog costed the render in frames and seconds but
     never in bytes, which is the one resource a frame sequence actually exhausts. It now shows
     `≈N GB of frames · M GB free` from `sysinfo::free_disk_bytes`, amber with "not enough room for
     the whole sequence" when short. ~0.65 B/px, the measured PNG cost of a 4K fractal frame,
     labelled as an estimate — deep frames compress worse than shallow ones, so it is an order of
     magnitude, not a promise, and the 1.2× margin keeps it quiet when it is merely close.
  ⚠**This is the third time in two days that the diagnostic, not the failure, was the problem** (the
  beta.36 crash manifest knowing only export frames; the controller's silent `continue`). The
  pattern each time: a component states the reason correctly and something upstream discards it.

- [x] ⭐**Crash reporting was blind to ABORTS — FIXED v0.2.40-beta.43.** Three mechanisms now cover
  the classes that left nothing behind:
  1. **Global allocator wrapper** (`alloc.rs`). `set_alloc_error_hook` is nightly-only, so
     `ReportingAlloc` forwards to `System` and reports on the null return, just before the runtime
     aborts on it. Deliberately keeps NO per-allocation counters — an atomic RMW per allocation in
     a bignum-heavy, rayon-parallel program is a contended cacheline on the hottest path; process
     memory comes from `sysinfo::process_memory()` at report time instead, costing nothing until
     something fails.
  2. **`memory :` line in every crash report**, and memory on the reference-build breadcrumb
     (`building reference [live]: iter=1033260 prec=4061 (rss 289 MB, peak 1260 MB)`) — that build
     is the app's largest allocation by a wide margin and is what an OOM dies inside. Breadcrumbs
     only; never per-frame.
  3. **Unclean-exit marker.** Armed around the GUI event loop only and cleared by `crate::exit`,
     which ALL 30 `std::process::exit` call sites now route through — one choke point, because a
     single missed site would report a deliberate quit as a crash, and a false crash report teaches
     everyone to ignore the real ones. On the next launch a surviving marker produces a report
     quoting the previous session's last six log lines in chronological order. Worded as "no clean
     shutdown", not "crash": a hard kill lands here too.
  **Verified all three, not just compiled:** a clean `--selftest` arms nothing and reports nothing;
  a GUI run killed with `Stop-Process` is reported on the next launch with its log tail; and
  `--oomtest` (a new flag that requests an unsatisfiable allocation) exits with **`0xC0000409` —
  the exact signature of the three real crashes — and now writes a report naming the failed size,
  the memory, and the breadcrumb. Suite 101/101 + goldens 17/17; `--divetest` 59.5–60 fps, p95
  ≤ 15 ms per band, so the allocator wrapper costs nothing measurable.
- [ ] ⭐**PRIORITY: unexplained access violation, `08-07 17:00:25 0xc0000005`.** No crash report and
  never investigated. Distinct from the abort class above (a segfault, not a controlled exit), so
  it needs its own look once (c) above makes silent deaths visible.

- [~] **The live view cannot honour a high explicit iteration count — the COST MODEL half is FIXED
  (v0.2.40-beta.47); honouring the count itself is blocked on two couplings.** What landed, all
  measured at an explicit 10,000,000 iterations on a 1445×1134 panel:
  1. ⭐**The timing was being paired with the wrong dispatch's step count** — the defect under
     everything else here. `IterTiming` publishes ~2–3 frames after the pass it measured, while
     `fe_steps_last` is a single app-side slot overwritten by every dispatch; a tiled settle
     dispatches every frame, so each reading was priced against a LATER, usually clamped-edge tile
     and thrown away as undersized. The budget therefore froze wherever it happened to be
     (observed: five readings, then silence at 9.0e8, view stuck at 34×27). Fixed at the producer:
     `MandelbrotParams::nominal_steps` travels with the pass, `IterTiming` stashes it when it
     records the timestamps, and both come back together through `iterate_steps`. Steps are stored
     BEFORE the ms, since the app treats a non-zero `iterate_ms` as "a reading is ready".
  2. **A completed grid repeats its final rect** (so the GPU dedupes it) and that repeat was being
     priced as a dispatch. `next_settle_tile` now returns whether the tile actually ADVANCED.
  3. **A grid formed while the budget was climbing pinned that resolution forever**, because a
     settled view never changes its key and only an ss increase could unpin it. It now also
     re-forms on a materially larger affordable resolution. ⚠The margin must sit BELOW one budget
     step's worth of gain — `TDR_GROW_MAX` is 1.5× and cost goes as resolution², so a step buys
     exactly √1.5 = 1.2247×, and a 5/4 margin stalled the ratchet one notch short at 78×61. It is
     9/8.
  4. Net: the view ratchets 16×16 → 34×27 → 78×61 → … → 597×468 → **native 1445×1134** while the
     budget climbs 4.0e8 → 3.0e11, over ~160 readings and ~590 tiled frames.

  ✅**The SHARPNESS half landed 2026-08-09.** Reported from the field at 6.46e94× with Iterations
  4,000,000 and auto-scale off: a settled, parked view rendering visibly blocky. Cause was the one
  diagnosed above — `budget_res_scale` sizing a SETTLED frame from `6 × WORK_BUDGET`, a fixed
  constant that measures nothing (`want` ≈ 6.6e12 there, so the shrink lands at ~0.23 of native
  linear). A settled view is no longer sized by it; the measured `tdr_allowed` bound and the tiled
  settle size it instead. Measured at an explicit 4,000,000 iterations: the view ratchets
  281×221 → 776×609 → 951×746 → 1427×1120 → **native 1445×1134**.
  ⚠**Gated on `!tour_playing()`, and that gate is load-bearing.** The earlier attempt applied it
  during playback too and regressed the tour (1e61 hold 42.1% → 100% black) because tiled settle
  and the iteration boost compete for the same settled frames. The distinction is TIME: a scripted
  hold has seconds on a clock and must spend them on iterations; a view the user is parked at has
  as long as it likes and should spend it on resolution. A FINISHED tour is not "playing", so a
  parked tour sharpens — the reported case. Livetest 0 drifted, suite 104/104.

  ⛔**Still blocked — the ITERATION-COUNT half, and both blockers are worth knowing:**
  - **`gpu_iter` is still `.min(boosted_cap)`.** Removing that when `auto_iter` is off works in
    isolation (that is how the native-resolution result above was obtained) but CANNOT ship,
    because **script playback sets `auto_iter = false` for every keyframe carrying a `max_iter`**
    (`apply_sampled_settings`). So the branch lands on the flagship tour, and there it regressed
    the deep holds — `--livetest` on the grand tour: 1e61 went 42.1% → 100% black.
  - **Two copies of the same formula.** The boost probe recomputes the effective cap itself
    (`budget_now`, in the maxiter-counter drain) and compares it against the `armed_iter` the GPU
    reports, to reject stale pre-raise readings. Change `gpu_iter` without changing that
    expression and every reading is silently discarded as stale, the probe freezes, and the boost
    sits at its first step (measured: ×1.60 everywhere, against ×6.40/×16/×64). These two must be
    derived from one function before either is touched.
  - **Resolution and the iteration budget compete for the same settled frames.** The boost adapts
    on the capped-pixel fraction, which the GPU reports only for a FULL-FRAME iterate; a tiling
    view feeds it one coarse arm frame per grid instead of a reading per frame. This was masked by
    (1) — mispaired readings meant the budget never converged and `fe_budget_ok` gated tiling off.
    Fixing the pairing surfaced it. An explicit "iteration budget first, resolution second" gate
    (hold the grid off until the boost plateaus) was tried and did NOT restore the boost, so the
    coupling is not simply tiling-vs-probe; it needs its own investigation.
  ⚠Everything above is confined so the shipped auto/scripted path is unchanged: `--livetest` on the
  grand tour matches the pre-change build exactly (18 ok / 1 warn / 3 FAIL, same percentages, same
  boosts, same orbit lengths), `--divetest` is within run-to-run variance (1e300 56.7/55.0 fps vs
  baseline 56.5/56.4), suite 102/102 + goldens 17/17, bench-matrix 0 drift.

  Original diagnosis, kept — item (3) below is what beta.47 acted on:
  Reported at 8.8e94× (three-spar): Iterations set to **10,000,000** with auto-scale OFF, but the
  status bar reads `iter 82,741` and `⚠ iter exhausted`, and the picture is blocky. Two mechanisms,
  and the second is the real one:
  1. **The setting is overridden.** `gpu_iter = eff_iter.min(iter_cap).min(boosted_cap)` where
     `boosted_cap = zoom_iter_cap(l2) × iter_boost`. At this depth `zoom_iter_cap = 2000 + 315.4 ×
     256 = 82,742` — exactly the number displayed. The adaptive boost is what should lift it, and
     it does climb; but at the full appetite the frame is still >98% capped, so the `!cap_bound`
     branch declares "exhausted" and **reverts the boost to 1.0**, dropping the render two orders
     of magnitude below what was asked for. That revert is right for an AUTO budget (identical flat
     image, far cheaper) and wrong for an explicit one.
  2. **Simply honouring it makes things WORSE — measured, not predicted.** Removing the cap when
     `auto_iter` is off renders at 10M and the frame collapses to **16×16 pixels** (the shrink's
     own floor) upscaled to the panel: at 10M iterations one pixel is 1e7 nominal steps, so even
     16×16 is 2.56e9 against a 9.000e8 budget, and the tiled settle never arms (`can_tile=true
     tiling=false`) because the budget never converges. Strictly worse than the 82,741 being
     complained about. **The change was written, measured, and reverted** — do not re-attempt it
     without fixing (3) first.
  3. ⭐**ROOT CAUSE: the settled cost bound counts NOMINAL steps and ignores BLA skipping.** The
     field HUD at this very view: `iterate 1.3 ms (GPU)`, `steps/s 119,629 G`, `budget 3.00e11` —
     the frame's REAL cost is trivial because BLA skips nearly all of it, while the budget prices
     it as if nothing were skipped, and over-shrinks by orders of magnitude. The MOTION path
     already solved exactly this (the AIMD `res_scale`, driven by the measured frame interval,
     whose comment says the nominal model "over-shrinks the moving frame" where BLA skips); the
     SETTLED path never got the same treatment. Fix direction: size the settled resolution/tiling
     from measured cost like the motion path does. Then an explicit iteration count can be honoured
     at full resolution — the affordable range at this view is ~1.05M iterations before the shrink
     binds at all, i.e. ~12× what the view currently gets.

- [~] **A reference the freeze guard refuses is re-requested forever — TWO HOLES CLOSED
  v0.2.40-beta.47, but the SYMPTOM NEVER REPRODUCED (2026-08-09).** Both halves the original entry
  guessed at ("either something is clearing it each round or the re-request comes from a path that
  does not consult it") turned out to be real, and both are now closed:
  1. ⭐**The refusal record was amnesiac.** `install_recompute` cleared `ref_ext_refused` on EVERY
     install. After the extension is refused at 258,193 samples, the PRECISION and
     `bla_out_of_range` triggers rebuild anyway — at a shorter length, which passes the cap and
     installs — and that install wiped the record. Traced on the gauntlet: refusal at 258,193
     followed immediately by a stream of installs at 133,499 / 133,876 / 134,257 / … A shorter
     orbit cannot disprove "an orbit this long was still partial", so the record now survives
     unless the install actually reaches past it or the reference POINT moved (a re-anchor is a
     different orbit — the 6.5e94× spar depends on that path staying open).
  2. **The refusal was advisory, not authoritative.** `needs_quality`'s partial branch respected
     it; the precision branch and `bla_out_of_range` did not, so they could spawn the identical
     doomed build. A rebuild whose ask is no larger than the refused length, at the same point,
     is now suppressed at the spawn.
  ⚠**Scoped to settled builds past `LIVE_REF_CAP`, and that scoping is load-bearing.** Suppressing
  every rebuild instead looked like a fix and was a regression: the 1e61 hold's black fraction
  "improved" 42.1% → 0.1% purely because the view stopped rebuilding and froze on a **31.7-second
  stale reprojection at 144×81**, with the sRGB difference from the offline truth nearly doubling
  (33.4 → 60.2). Only a settled extension can be refused; a rebuild capped at `LIVE_REF_CAP`
  always installs, and those installs are what keep the BLA fresh and the view re-iterating.
  Caught by the livetest baseline landed the same day — the black metric alone called it a win.

  ⚠**The symptom did not reproduce, so treat this as hardening, not a demonstrated fix.** Tried:
  a static session seeded at the hold-e55 centre and 250,000 iterations (gives the exact reported
  numbers — `len=258193`, refused, keeping a 56,795-sample clamp — but 3 builds and 1 refusal in
  40 s, no loop); `--livetest --segment gauntlet` at 480×270 and at 2560×1440 (763 builds, 4
  one-off refusals). The guards measure as a no-op on that workload — 759–762 builds, 4 refusals,
  **0 suppressions** across repeat runs — and the livetest gate reports 0 drift, so the picture is
  unchanged. Six property tests (`render::controller_props`) pin both rules so a future change
  cannot quietly reintroduce either hole. If the field report recurs, the trace now says
  `rebuild suppressed: ask N ≤ refused M` when the guard fires.

  Original diagnosis, kept: At the grand tour's `hold-e55` keyframe (t=215 s, `zoom = "1e55"`,
  `max_iter = 250000`) the settled extension asks for `ref_build_iter = 250000 + 8192 = 258192`.
  That orbit never escapes, so it comes back **partial at 258,193 samples — 2,192 over
  `LIVE_REF_CAP` (256,000)** — and `install_recompute`'s freeze guard refuses it. Measured in the
  field log: the identical build repeats **every ~450 ms indefinitely** (`reference built [live]:
  len=258193 iter=258192`), burning a core forever. `ref_ext_refused` is supposed to stop exactly
  this ("re-fire only for strictly more than a refused attempt"), so either something is clearing
  it each round or the re-request comes from a path that does not consult it — that is the thread
  to pull. Consequence while it loops: no reference installs, pixels stay clamped to the previously
  installed orbit (48,772 iterations against a 250,000 ask), and the view is BLACK.
  ⚠Note the arithmetic: any script asking 248k–256k iterations lands in the refusal band purely
  because `ref_build_iter` adds 8,192 of headroom. The tour asks for 250,000.
- [x] **The tour clock could be held forever — FIXED v0.2.40-beta.42.** Reported from the field:
  the grand tour "seems to have hung at 3:35 (waiting for detail)" running full screen. The app was
  fine — responding, renderer busy, log advancing; it was the TOUR CLOCK that had stopped for good.
  `settle_timeout` has always bounded the settled-hold branch of the pacer, but the **lag-based
  dilation was bounded by nothing**: with the reference above never installing, the BLA never
  refreshes, `last_depth_lag` never falls, and `hold` stays at 1.0 for the rest of the run. Fix: a
  final backstop over every reason the clock can be held — once fully stopped for `settle_timeout`,
  the pacer gives up and lets the tour run. The release is STICKY until the pipeline genuinely
  recovers; releasing for one frame and re-arming the timer would only convert the stall into a
  15 s duty cycle, which is a hang with extra steps.
  ⚠**Two shortcut repros FAILED to reproduce it** and would have "proved" a fix that fixed nothing:
  starting cold at 1e55 builds a fresh reference (no lag, no stall), and a 1e30→1e55 dive at the
  tour's own rate also completed. The stall needs the whole tour's accumulated state AND the
  reported window size — the field report was full screen (5142×2182), ~7× the pixels of a default
  window, which is what makes the frames costly enough to matter. **Verified the only way that
  works: the actual grand tour, maximized, end to end — 5:31/5:31, past the 3:35 stall point, no
  storm, no device loss.** Suite 101/101 + goldens 17/17.

- [x] **A static view rendered at 205×162 upscaled to the panel — FIXED v0.2.40-beta.41.** Reported
  as "detail isn't resolving, it looks very pixelated" at a 6.6e18× df32 view. A beta.40 regression,
  and the mechanism is worth keeping: **the frame-cost budget could never bootstrap on a view that
  stops changing.** `fe_steps_last` — the nominal step count the controller prices a GPU timing
  against — was recorded only on `key_changed && !will_reproject`, and on a view that reaches its
  final state inside the reproject window that condition never held, so the number was never
  written at all. Timings kept arriving and kept being discarded: `no reading (bits=true, ms=0.28,
  steps=0)`, every frame, forever. The budget therefore sat at `TDR_BOOTSTRAP_STEPS` — a value
  chosen as a safe FIRST dispatch on unknown hardware, not a statement about what a view can
  afford — and beta.40 had just made that floor drive the resolution shrink on df32. Result: a
  permanent 205×162 render upscaled to a 1431×1134 panel. Fix: record `fe_steps_last` on
  `key_changed` alone; the timing only ever arrives from a dispatch that really ran, so pairing it
  with the last key change is sound, and a stale pairing costs one mis-priced measurement that the
  ratio search corrects. Now the budget climbs 4.000e8 → 3.460e10 in ~2.5 s and the view renders at
  full 1431×1134.
  ⚠**The `is_fe` gates hid this.** Before beta.40 the budget was equally unmeasurable on a static
  floatexp view, but nothing on the df32 path consumed it, so the dead controller was invisible.
  Generalising the bound is what turned a silent stall into a visible one.
  **Verified** by A/B at the reported location on beta.39 vs beta.40 vs the fix (same
  `session.toml` restored before each run): beta.39 sharp, beta.40 blocky at 205×162, fix sharp and
  pixel-comparable to beta.39. Suite 101/101 + goldens 17/17; `--divetest` 59.3–60 fps, p95 ≤ 16 ms
  per band; and the beta.40 crash repro still bounds — the same frame now tiles at **4631×2060
  ss=2** (better than the ss=1 it managed before) with no device loss. New `no reading (…)` trace
  under `FRACTADYNE_TRACE=gpu` names which of the three preconditions blocked a measurement — it is
  what found this, and the controller was previously silent on every one of them.

- [x] **A df32 frame had no cost bound at all — FIXED v0.2.40-beta.40.** Device loss 2026-08-07
  22:17 UTC (beta.39, build 1175), 49 s into a live tour in a maximized window:

      LIVE view=0 mode=0 4631x2060 ss=1 iter=12000 (boost=1.60)
      steps=1.145e11 budget=4.000e8 tile=false settled=true

  9.54 M pixels × 12,000 iterations = **1.145e11 nominal steps in one dispatch, 286× that frame's
  own budget**, submitted untiled; the log goes silent for eight seconds and then loses the device.
  `mode=0` is `Df32Pert`, and **five separate guards were gated on `is_fe`**, so on the df32 path
  there was nothing at all between the frame and the watchdog:
  1. the GPU timestamp SINK was only attached on floatexp (`is_fe.then(...)`) — the root: with no
     sink there is no measurement, hence no budget, hence nothing for anything else to size against;
  2. `fe_steps_last` (which the controller requires non-zero) was only recorded on floatexp;
  3. `can_tile` required `is_fe` — no tiled settle;
  4. the resolution shrink required `is_fe` — no shrink;
  5. the `tile` trace was floatexp-only, hiding the case that crashed.
  The one guard that did cover every mode, `max_ss_tdr`, floors at ss=1 and this frame was already
  ss=1 — and it measured against `TDR_STEPS_CEIL` (3e11), which 1.145e11 is *under*, so it could
  never have fired. Had the same frame been floatexp, the shrink would have taken it to ~273×121.
  That asymmetry was the bug. **Same shape as the beta.36 crash already in this ledger**, where
  `max_ss_tdr` was floatexp-only and got generalised — only the ss half was fixed then.
  Fix: every mode measures and is bounded; a MODE SWITCH derates the budget (`budget_mode`) exactly
  as `install_recompute` derates on a reference discontinuity, since a nominal step costs several
  times more in floatexp than in df32.
- [x] **The budget controller discarded the readings that mattered — FIXED v0.2.40-beta.40.** Found
  while verifying the above, in the trace of the repro: `gpu_iterate=1451.4ms IGNORED (steps=1.030e10
  < 0.7×budget)`. A dispatch 1.6× over the 900 ms target and most of the way to the ~2 s watchdog,
  thrown away — so the budget sat at 1.663e11 with no way to learn otherwise. The `< 0.7×budget`
  rule exists to stop a *fast* undersized tile (a clamped grid edge, latency-bound at depth) from
  inflating the budget; every word of its rationale is about readings that push the budget UP, but
  it was applied two-sided. It is now one-sided, and a slow reading also bases the shrink on
  `min(budget, that dispatch's own steps)` — a size that just measured slow cannot remain the
  budget. Assumes only "smaller is not slower", never `steps ∝ time`.

  **Verified** on a repro of the crash's exact frame (mode 0, 4631×2060, 12,000 iterations,
  1.145e11 steps, maximized): the slow reading is now acted on — `1146.9ms x0.78 cur=1.752e11 →
  next=1.218e10` — and the frame ends up **tiled at full resolution** (`budget=4.339e10
  tiling=true tile=Some([2875,0,1756,2060])`) instead of one dispatch. 74 s, no device loss. Deep
  path unregressed: `--divetest` 59.3–60 fps, p95 ≤ 22.2 ms per band. Suite 101/101 + goldens 17/17.
  ⚠The repro reproduces the frame's SHAPE, not its cost — the same nominal steps measured 114 ms at
  one location and 1147 ms at another, which is this codebase's standing lesson that `steps ∝ time`
  is false. The bound is what was verified, not a re-triggering of the original TDR.

- [x] **Live playback loses the GPU device a few minutes in — FIXED v0.2.40-beta.38.** Cause: the
  script-playback reference LOOKAHEAD was spinning. **Measured ~400 reference builds a second,
  sustained** — 13,529 of 14,311 builds in the 392 s crash log returned the same `len=626`, six
  worker threads spawned per frame, ~92,000 in a single 230 s playback, each fanning out across
  every core. Two independent defects in `playback_ref_prefetch`, both now fixed:
  1. **The hold/install test was read with the sign inverted.** The queue timed a slot by its BLA
     `dc_max` against the view's, and that lag GROWS as the dive descends (a tree built for a
     target Δ octaves ahead starts at `1 − Δ` and rises one per octave). So `lag < 0.86` means
     "the dive has not reached this slot yet" — the code took it for "window missed" and CULLED.
     Every result was discarded the moment it landed and the queue instantly rebuilt the same six
     targets. It only ever appeared to work because a deep build takes about as long as the dive
     takes to cover `PREFETCH_OCT`; wherever builds were cheap, it span. The test is now the
     slot's own `target_l2` against the script's current depth (`prefetch_reached`, pure and
     covered by a selftest that fails on the inverted reading), which also handles slots with no
     BLA at all — those had `NEG_INFINITY` lag, so the old form could not time them either.
  2. **The refill fought its own housekeeping on every non-diving stretch.** `max_ahead` is keyed
     off the CURRENT depth, so on the grand tour's back-out chapters a falling `cur_l2` swept down
     through the queued targets, culled them, and the refill rebuilt the same set for the next
     chapter's dive: **214 builds/s, none installed.** The refill is now gated on the tour
     actually descending (probe at +0.25 s), with a hard 30 builds/s backstop that does not depend
     on the queue logic being right.

  **The diagnostic gap is what made this expensive.** `recompute/s` counted INSTALLS only, so 400
  discarded builds a second read as a calm `recompute/s 2`, and every build logged an identical
  breadcrumb — the storm was visible only by counting lines in a 5 MB log after the fact. Now:
  a `ref builds/s` counter beside it, each breadcrumb tagged with its origin
  (`live` / `lookahead` / `export`), and a once-per-session log line when the rate passes 60/s.
  That tripwire is what caught defect (2), immediately, after (1) was fixed.

  **Verified 2026-08-07:** same tour, same machine, `--play tours/grand-tour.toml` — **722 s,
  responding, no device loss, no storm**, past all five recorded crash points (230/261/392/470 s);
  build rate 400/s → 3.6/s. Suite 101/101 + goldens 17/17; `--divetest` on the e1216 dive holds
  57.6–60 fps with p95 ≤ 21.6 ms in every band (2 frames >33 ms in the whole run), so the
  lookahead still does its job. ⚠**Not proof of the mechanism**: the run is a null result on a
  bug that took 230–470 s to appear, and *why* a build storm takes the device out (CPU starvation
  of the driver? allocator churn?) was never established — only that removing it removes the
  crash. If it recurs, the storm counter now names the source in one line.

  Original investigation, kept for the diagnosis trail:
  Three crashes on 2026-08-07 (beta.36 builds 1134/1149) at 261 s, 392 s and 470 s of live tour
  playback. The signature is NOTHING like the beta.36 crash that was fixed — this frame is far
  UNDER budget and tiny:

      LIVE view=0 mode=2 429x340 ss=1 iter=27697 (gpu_iter=27697, eff=60750, boost=1.00)
      steps=4.040e9 budget=7.785e10 tile=false orbit_len=626 partial=false settled=false

  4.04e9 nominal steps against a 7.785e10 budget — 5% — at 429x340, while MOVING, and it still
  lost the device. One of the three died at `steps=3.998e8` against `budget=4.000e8`, i.e. sitting
  exactly on the `TDR_BOOTSTRAP_STEPS` floor at 135x107: the controller had already shrunk as far
  as it can and that was still fatal. So the cost model is wrong here by one to two orders of
  magnitude, not by a little.
  - **`orbit_len=626` is the thread to pull.** The reference escapes after 626 samples while pixels
    iterate to 27,697, so each pixel traverses the reference ~44 times and the BLA table (626
    entries) can skip almost nothing — nominal steps stop predicting real cost. The budget is
    MEASURED, but it is measured on BLA-effective frames, so it is stale-high exactly when a
    degenerate reference appears. `install_recompute` already derates on a cost-discontinuous
    install, but only on orbit GROWTH (`res.orbit_len * 2 > old.orbit_len * 3`); a collapse to 626
    is just as discontinuous and does not trigger it.
  - **The prefetch thrashes on it.** In the 392 s run, 13,529 of 14,311 reference builds returned
    `len=626` — ~35 rebuilds a second for six minutes. `playback_ref_prefetch` culls a slot whose
    result has no usable BLA (`lag < 0.86`) and the queue immediately refills the same target, with
    no backoff and no memory of a futile target (unlike `ref_ext_futile` on the extension path).
  - **Ruled out:** coordinate precision. Keyframe centres were parsed at each keyframe's own depth
    (fixed in beta.37 for RATIONAL coordinates, with a selftest that fails at 9.8e-40 drift without
    it) — but a plain decimal literal is parsed from its own digit count, so the shipped tours were
    never truncated and this was not the cause.
  - **RULED OUT: frame cost / the watchdog budget.** A fifth crash (beta.37, build 1157, 230 s)
    died at `budget=3.000e11` — the CEILING — with `steps=3.949e9`, while an earlier one died at
    `budget=4.000e8`, the FLOOR, with `steps=3.995e8`. Across the five, resolution varies 10x
    (429x340 / 135x107) and the budget varies 750x, and it dies regardless. Shrinking the frame or
    re-budgeting it therefore cannot fix this; the earlier "the cost model is wrong" reading was
    incomplete.
  - **The invariant is the STATE, not the size.** All five: `mode=2`, `orbit_len=626`,
    `partial=false`, `iter` 27 049–27 697, `settled=false`. A 626-sample ESCAPED reference with a
    ~27 000 pixel budget, i.e. ~43 rebases per pixel and a BLA tree that can skip nothing.
  - **Also ruled out: an unbounded shader loop.** The mode-2 loop breaks on `iter >= max_iter` and
    increments once per pass; the rebase at the end of the body resets `ref_n` only. So a pixel
    really is bounded at ~27k iterations, which makes 4e8 nominal steps taking >2 s unexplained by
    iteration count alone.
  - **Leading hypothesis now: reference UPLOAD traffic, not iteration.** The prefetch rebuilds ~35
    references a second and every install bumps `orbit_id`, which re-uploads the orbit + BLA
    storage buffer. That buffer is sized against `orbit_len_cap()` (7 452 444 samples, ~1 GB
    binding). Re-uploading at that rate would swamp the bus and is independent of frame size and
    budget — which is exactly the observed signature. **Check what `orbit_id` actually re-uploads
    and how much, before changing anything else.**
  - The build carrying the beta.37 fix survived 500 s of the same tour, past all three earlier
    crash points, but it then crashed at 230 s on the next run — so that survival was luck, not a
    fix. Do not read it as one.

- [x] **Live tour playback lost the GPU device — FIXED v0.2.40-beta.36.** Reported by the user
  ("it crashed in the live view of the tour"); reproduced in 29 s with the new `--play` flag. A
  regression from beta.35's settling change plus a latent bug it exposed. The crash report's new
  LIVE manifest named the frame outright: `1431x1134 ss=8 iter=12000 (boost=1.60) steps=1.246e12
  budget=4.000e8 tile=false settled=true` — once holds settled, the progressive-AA ramp ran during
  playback and reached ss=8 (64 samples/pixel), and at a SHALLOW `mode=0` view nothing bounded it.
  Fixes: (1) no settle AA ramp during scripted playback — a tour's camera moves again in seconds,
  so the ramp buys nothing and costs quadratically; settling at a hold is for the iteration budget
  and the reference build. (2) The watchdog ss cap existed only on the floatexp path
  (`max_ss_tdr = if is_fe {…} else { u32::MAX }`) — **a latent crash reachable interactively**
  (high manual iteration count + AA 8 at a mostly-interior shallow view), now capped in every mode.
  (3) Live playback is no longer classified as an offscreen one-shot, so it obeys the measured
  budget and may tile. (4) The live path now sets a crash manifest at all — before this, every
  on-screen device loss recorded an EMPTY `manifest:` line, which is why this class has historically
  been diagnosed by inference. ⚠**Lesson: the crash report only knew about EXPORT frames. When a
  diagnostic is silent for the path that actually crashes, fix the diagnostic first — it turned
  three rounds of speculation into one line of fact.**

- [x] **Live view rendered BLACK during scripted playback — FIXED v0.2.40-beta.35.** Reported by
  the user as "sections that render as black" in the live view. `advance_playback_core` stamped
  the interaction timestamp on EVERY playback tick, so a tour's view was permanently
  "interacting" — right for a glide, wrong for a hold — and since the adaptive iteration budget
  only measures and adapts on SETTLED frames (`render.rs`, `if interacting { … capped_frac = None }`),
  the budget could never climb during a tour. Measured by the new `--livetest` harness at the
  three-spar holds: the live view ran at the unboosted depth cap (49k / 54k / 56k / 63k / 72k /
  83k iterations) against script budgets of 250k–4M, and was **100% black at 1e61×, 1e72× and
  1e82× where an offline render of the same view at the same budget is 0% black** (image pairs
  dumped by the harness). Fix: only a MOVING camera counts as interaction, so holds settle; plus
  `[playback] pace = "settled"`, which stops the tour clock at a hold until the view resolves
  (bounded by `settle_timeout`), because at depth the budget needs more settled frames to converge
  than a few seconds of hold provides.
- [ ] **A settled view re-dispatches identical frames instead of idling** (found 2026-08-09 by
  the ultra-dive exercise): after `ultra-dive-e200.toml` ends, the settled e200 view keeps
  submitting the SAME budget-sized dispatch (~234 ms, identical steps/rebase counts) at ~3.5/s
  indefinitely — ~80% GPU duty to render a picture that is already on screen. Harmless for
  correctness (frames are bounded, watchdog-safe) but a real power/thermals waste on laptops.
  Suspect: the boost probe / quality-gate loop never concluding at a view whose budget sits at
  the ceiling. `slow frame` lines with identical `steps`/`rebase` are the signature.

- [ ] **Tour/offline render has no per-frame cost bound** — the TDR no longer reproduces, but the
  hole is still there. Found 2026-08-07 rendering `tours/grand-tour.toml` with a script-wide
  `max_iter = 2000000`: at frame 16 a 2,000,001-sample (non-escaping) reference installed and the
  next frame lost the device (`DEVICE LOST (Unknown)`), even at 240x135. beta.34's per-keyframe
  budgets removed the trigger — that frame is a 1.33x home view and now asks for 2 000 iterations,
  and the full tour renders end to end — but nothing *bounds* a tour frame's cost, so a script
  that asks for millions at a shallow view can still do it. The LIVE path bounds per-frame cost
  (`fe_budget` + tiled settle + motion-res); the tour path has none of that, and `render_export`'s
  `TILE_WORK_BUDGET` evidently wasn't bounding that dispatch (240·135·2e6 = 6.5e10 nominal steps
  vs a 2e10 tile budget — check whether the tour render path actually goes through the tiled
  export). Fix direction unchanged: route tour frames through the tiled export path with a budget
  calibrated like the live one.
- [ ] **A deep tour frame's memory is unbounded too** (measured 2026-08-07). The offline path
  builds frame N+1's reference while frame N renders, so the peak is two references at once: at
  the 6.5e94x spar an 8M-sample reference is ~2.3 GB at 379-bit precision plus its BLA tables, and
  the pair OOM-killed the render mid-dive on a 32 GB machine (`memory allocation of 6520976 bytes
  failed`, after 221 of 233 frames). The grand tour's deepest hold is set to 4M as a result, which
  leaves it a few percent capped. Worth bounding the lookahead by available memory, or at least
  failing with a diagnosis instead of an allocator abort.

## Script format — requested extensions (user, 2026-08-09)

- [ ] **Tour files should be able to set every rendering parameter the UI can.** Today `[render]`
  covers size/fps/ss/max_iter/auto_iter/out/prefix/mp4/show_location and keyframes carry
  palette/max_iter — but UI-reachable settings like supersampling mode, glitch correction,
  series approx / BLA toggles, normalize-deep-colors, stripe/trap coloring parameters, and the
  watermark are not scriptable. Rule: **anything unset in the script defaults to the current UI
  settings** (already the effective behaviour for the unlisted ones — make that a documented
  contract rather than an accident, and emit each resolved value into the render log so a report
  shows what actually applied).
- [ ] **Tour-driven debug logging for remote diagnosis.** The ability to annotate a tour with
  debug/log directives — e.g. a `[[log]]` block (`t`, `note`, optional `dump = ["budget",
  "reference", "counters", "timings"]`) that appends structured lines to a per-run tour log file
  (`logs/tour-<name>-<timestamp>.jsonl`). Purpose stated by the user: **build tools that help
  debug issues on OTHER USERS' machines** — a bug reporter plays a diagnostic tour and sends one
  file back, giving us budget/reference/counter state at scripted moments without access to the
  hardware. The `--livetest` checkpoint machinery already gathers exactly this state at holds;
  this item is about exposing it to any tour, on any machine, from the script itself. Pairs with
  the in-app issue reporting (attach the newest tour log).

## Playback player — v0.2.40-beta.39 (2026-08-07)

Reported: "messages about waiting for detail adjust the width of the controls", plus asks for a
scrub bar, a player that outlives the tour, and a close button.

- **Scrub bar.** A slider under the transport row; dragging seeks the clock and the camera follows
  through the normal sampling path, so it needed no special case anywhere else.
- **Nothing on the player changes width any more.** Four things did: the "waiting for detail"
  notice appearing mid-hold, the elapsed clock crossing ten minutes, the speed label cycling to
  `0.5×`, and `▶`/`⏸` not being the same glyph width. Now the notice sits on the scrub row and is
  ALWAYS laid out (painted transparent when idle), the clock is padded to the total's width, the
  name is elided to a fixed field, and every button is `add_sized`. ⚠**The layout rule for this
  widget: nothing inside it may be sized from the available width** — inside an `egui::Area` that
  is unbounded, so a "fill the row" scrub bar or a right-aligned sub-layout blows the player up to
  the width of the screen and egui then clamps it against the screen edge, sliding the whole thing
  sideways with its right-hand buttons off-view. Both mistakes were made and measured here.
- **The player outlives the tour** (`Playback::finished`): a finished script parks at its final
  keyframe with the transport up, so you can scrub back in; the viewer's own iteration budget and
  coloring are handed back only on close, since restoring them earlier would recolor the frame
  being looked at. A finished tour is NOT "playing" (`tour_playing()`) — it settles its AA and
  arms frame-cost measurement like any idle view. The benchmark still tears down on completion:
  it owns the session and reports through a dialog.
- **✖ closes the player** (as does Esc and Tools → Close player); ⏹ now only stops and rewinds, as
  on any media player, and ▶ on a finished tour replays it.

⚠**Harness note, cost me several wrong conclusions:** a DPI-unaware PowerShell screenshot gets
VIRTUALISED coordinates from `GetWindowRect` while `PrintWindow` renders at true device size, so
the capture silently clips the right ~30% of the window — which reads exactly like a UI
overflowing its frame. Call `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)` first.

## Script format v2 — SHIPPED v0.2.40-beta.34 (2026-08-07)

Breaking change, explicitly sanctioned: no compatibility branch and no v1 reader — a v1 script is
rejected with a migration message (`check_format_version`), and all six shipped tours were
migrated in the same pass. Schema reference: [TOURS.md](TOURS.md), generated from `TOUR_SCHEMA`.

Delivered: **absolute keyframe times** (`t` = arrival, plus `hold`) · **stable `id`s** ·
**one `[[annotation]]` array** tagged by `kind` · **`[render]` block** (CLI flags override, so
`--render-tour x.toml` alone reproduces the intended render) · **per-keyframe `max_iter`**,
interpolated geometrically along each glide · **`[[location]]`** named coordinates (with a
reserved `thumb`) · **`zoom = "6.5e94"`** strings · **`[[segment]]` chapters + `--segment NAME`**
(global frame numbering kept) · **`[[palette]]` definitions** with a per-keyframe reference,
cross-faded between keyframes · **`editor` table** reserved and ignored. New `script` selftest
group (7 checks, suite 91 → 98) covers all of it, including that "Script to current view" still
generates a script this build can read — which is how the generator's v1 output was caught.

Effect on the grand tour: shallow chapters render at a few thousand iterations a frame (121
frames in 2.7 s at 480x270) while the spar holds get 250k–4M; the whole tour renders without the
TDR that killed it at frame 16.

Still open from the original design:
- **`[[audio]]` track** (music/narration timing is the usual next ask for video) and
  **per-property easing** (camera and palette want different curves — borrow glTF's
  channel/sampler model). Neither is reserved in the schema yet.
- **Thumbnail generation** — `thumb` parses but nothing writes or reads the cache. The app
  already renders bookmark/minimap thumbnails, so it's plumbing, not new machinery.
- **KF `.map` palette source** for `[[palette]]` — blocked on the `.map` import itself.

Not a compatibility target: there is no de-facto fractal tour-script format worth conforming to
(KF drives zoom videos from folders of numbered `.kfr` files; Ultra Fractal's `.upr` timeline is
proprietary and tangled with its formula system). The interop that matters for zoom video is
**exponential-map EXR for zoomasm** — spend the effort there. OpenTimelineIO is a plausible
future *export* if anyone wants to cut frames in an NLE, never the native format.

- [x] **wgpu device loss (TDR) crashes the app — FIXED v0.2.40-beta.29 (both halves).** Two crash
  reports on record, both `Surface::get_current_texture_view: Parent device is lost` → panic:
  2026-08-02 (v0.2.27, shallow view — external cause class) and 2026-08-06 (beta.27, the 2.6e72×
  spar — our own oversized dispatch: a 499,493-sample complete reference installed, lifting the
  pixel clamp while the frame budget was still calibrated on cheap clamped frames; the next
  3840×3042 ss2 frame at 500k iters blew the Windows GPU watchdog). Fixes: (1) `install_recompute`
  DERATES the measured frame budget (÷8, floor bootstrap) when an install changes the pixel cost
  model discontinuously — clamp lift or >2× orbit-length jump — so the controller re-climbs from
  safe sizes (its climb is overshoot-safe by design; gradual dive installs trip neither trigger);
  (2) device loss no longer panics: the handler writes the crash report, then AUTO-RESTARTS the
  app (session file preserves the exact view; "Recovered from a graphics device reset" toast via
  `FRACTADYNE_RESTARTED_AFTER_GPU_LOSS`), guarded to uptime > 60 s so a boot-time loss can't
  restart-loop. Verified live at the spar: derate fires on the big install, budget re-climbs to
  ceiling, 0 non-responding samples.

- [x] **Live pixel clamp for long-non-escaping references vs CLI — RESOLVED v0.2.40-beta.49/50
  (freeze guard v2: long partials now INSTALL at a re-measured budget; all grand-tour deep holds
  render at +0.0 pt vs offline, baseline all-green).** What remains is only the honest device-cap
  wall: a reference still partial at the full 7.45M-sample cap (the 2.05e95× spar) clamps pixels
  there, live and CLI alike — tracked under the depth wall, not here. Historical entry follows;
  ⚠its "the guard STAYS" conclusion was superseded once beta.48's budget floor made the install
  frame cost-boundable (the condition its own text set).
  ~~residual of the beta.27/28 freeze-guard design, accepted for now.~~ A reference still partial past `LIVE_REF_CAP` is refused
  (present-wedge safety, reproduced at e21000), leaving live pixels clamped at the last installed
  orbit (≤256k) while a CLI render of the same view clamps at the device cap — so live can
  under-resolve where an export succeeds. Bites when the picked reference's escape exceeds the
  device cap or genuinely never escapes — **measured 2026-08-07 at the 2.05e95× spar: the
  reference is still partial at the full 7.45M-sample cap, so the live view is BLACK there while
  a CLI render (pixels clamped at 7.45M) can still resolve.** **Experiment RUN 2026-08-07
  (guard bypassed at e21000, derate + restart nets active): the wedge is REAL and now fully
  explained** — the first frame after the 508k-partial install TDR'd (`DEVICE LOST (Unknown)` +5s
  after install), and the main thread then BLOCKED inside a wgpu wait on the dead device: no error
  ever surfaced, so no panic, no report, no restart — a permanent hang with the watchdog barking.
  So the guard STAYS (v0.2.40-beta.30 re-proves and documents it), and the experiment hardened two
  things it exposed: the device-lost CALLBACK now also writes the report and restarts (it fires on
  another thread and is the only recovery path in the blocked-main-thread case; `Destroyed` =
  clean teardown, never restarts), and the install derate trigger is ×1.5 (the killer install was
  a 1.985× length jump that sailed under ×2; dive installs are ×1.1–1.2, still never trip it).
  Real fix remains: cost-bound the first frame against a long non-escaping reference.
  ⭐**COST NOW QUANTIFIED (beta.35, `--livetest` on the grand tour's gauntlet at 480×270).** With
  the interaction bug fixed and the budget climbing (boost ×6.4–×16), the clamp is what is left,
  and it is the sole remaining cause of a black live view at depth:

  | hold | live black | offline black | live budget | script asks |
  |---|---|---|---|---|
  | 1.7e55 | 0.1% | 0.0% | 86 227 | 250 000 |
  | 3.3e61 | 42.1% | 0.0% | 146 112 | 400 000 |
  | 6.3e63 | 53.9% | 1.5% | **256 000** | 500 000 |
  | 2.6e72 | 100% | 0.0% | **256 000** | 1 200 000 |
  | 2.0e82 | 100% | 0.0% | **256 000** | 2 000 000 |
  | 6.5e94 | 100% | 0.3% | **256 000** | 4 000 000 |

  256 000 is `LIVE_REF_CAP` exactly: every extension past it came back still-partial and was
  refused. Note the reference is not always hopeless — at 6.5e94× a 3 631 055-sample attempt
  ESCAPED and installed, ~10 s after a 20 s settle timeout had already given up — so a refusal at
  a mid-climb budget must never be treated as terminal (only one at the device cap is).

- [x] **LIVE_REF_CAP truncates the reference below the live iteration budget → smooth "blobs"**
  — FIXED v0.2.40-beta.27 (reported 2026-08-06 at the 6.3e63× three-spar). The reference there
  naturally escapes at 256,753 iterations — 753 past the cap — so `LIVE_REF_CAP=256k` flipped it
  `partial`, clamping pixels to the short orbit (v0.2.26's pathology at the next scale up; the
  boost raising budgets past 256k is what re-exposed it). Fix (`live_orbit_cap` + install freeze
  guard): a SETTLED build's orbit cap follows the iteration budget so the orbit can run to its
  natural escape; motion keeps the cheap 256k cap; and `install_recompute` REFUSES a still-partial
  result longer than the cap (`ref_ext_futile` stops re-requests). That refusal is load-bearing:
  booting the e21000 tip and installing the extended 500k non-escaping reference (4M-node BLA)
  froze the window within one frame — the historical v0.2.15 freeze, reproduced live — while the
  refusal path stayed responsive for the whole run (0 non-responding samples, no watchdog).
  Verified: spar boots to full dendrite detail (escape-complete reference, no clamp); e21000
  boots, attempts the extension once off-thread, drops it, keeps the safe 256k clamp. Suite
  91/91, goldens 17/17, bench-matrix 0 drift. Residual accepted trade: non-escaping extreme views
  keep the 256k pixel clamp (exactly the pre-fix behavior) and pay one wasted ~27 s off-thread
  extension attempt per view; removing the clamp there requires fixing the present-wedge itself
  (root cause still unknown — it is NOT the build, it is the first frame against the long
  non-escaping reference).

- [x] **App freezes on load at extreme zoom (~1e2100×, center −2.0, the Mandelbrot filament tip) —
  FIXED v0.2.15 (`LIVE_ITER_CAP`).** The live preview now caps auto-iterations at 100k
  (`ui/central.rs` single + `nav_and_draw` dual); the export path keeps the full appetite. Verified:
  booting the actual e2100 session ran 45 s with NO watchdog / NO freeze (reference now builds to
  ~108k, not 500k); selftest 63/63 + goldens 17/17 unchanged (export path untouched). Diagnosis
  below kept for context. **Regression-guarded 2026-07-12** by a canonical extreme-zoom diagnostic
  case (the same tip, now at ~1e21000× / `units_per_pixel_e = −69770`, ~69769-bit): the exact view is
  `validation/extreme-zoom-tip-e21000.fdn` (load in-app via *Share location → Load .fdn…* to confirm
  it boots responsive / watchdog silent), and `validation/extreme-zoom.toml` is a `--profile --regions`
  region that exposes the cold reference cost: **measured ~250 s wall at 69828-bit** (mode 2), of which
  `--profile`'s `ref ms` column captures only ~410 ms (the orbit compute) — the ~99% remainder is
  `best_reference` scoring, re-confirming the throughput lever at extreme depth. NOT a golden or an F3 corpus entry — a render here
  is minutes (bignum-bound) and ~1e21000× is ~20× beyond F3's deepest matched pair (4.6e1105×). See
  DIAGNOSTICS.md "Canonical extreme-zoom diagnostic location".
- [~] **App freezes on load at extreme zoom (~1e2100×, center −2.0, the Mandelbrot filament tip) —
  DIAGNOSED 2026-07-12; NOT the reference build.** A saved session at this arbitrary-depth view
  (`units_per_pixel_e = −6986`, ~7000-bit precision, `max_iter = 500000`, `aa = 8`) leaves the
  window unresponsive on boot. **Diagnosis (booted the actual app with the v0.2.7+ watchdog +
  breadcrumbs):** the reference build is ALREADY off-thread (progressive coarse @10.6 s + full
  @12.3 s land from a worker thread — the "only the first reference is synchronous" comment in
  render.rs is STALE; the cold path was moved off-thread) so it does NOT freeze the UI. The main
  thread completes `update()` frames fine until ~16 s, then **wedges in the POST-`update()` GPU
  paint/present** — the watchdog's last activity is an end-of-`update()` breadcrumb (`update() done
  f=1204`), and no next frame ever starts. The preceding frames run ~1 s each (GPU-bound). **The
  isolated GPU render is HEALTHY** — a headless `--render` of this exact view is 22–117 ms at ss
  1/2/4 (maxiter=0, BLA skips 36M steps), so no pixel spin. So the freeze is **live-orchestration /
  GPU-queue specific**, not a render-cost or reference-build issue. **Strong hypothesis:** the
  tiled-settle + AA ramp (to ss 8) at this depth, driven by an over-provisioned `max_iter = 500000`
  (the view escapes at maxiter=0, so auto-iter's ~500k cap is absurd here → nominal ~8.1e11
  steps/frame → aggressive tiling → many/large dispatches across ss 1→2→4→8 back up the GPU present),
  same expensive-cumulative-dispatch class as the glitch-core spin. **Next step:** GPU-crate timing
  in `lib.rs` prepare/paint/present (fractadyne-gpu can read `FRACTADYNE_TRACE` directly) to pinpoint
  the hanging dispatch, then bound the settle/AA-ramp GPU work at extreme depth and/or fix auto-iter
  from over-provisioning where the view escapes fast. (Earlier "async-ify the cold reference"
  direction is MOOT — already async.) Higher severity than the XaoS enhancement.
  **ROOT CAUSE CONFIRMED 2026-07-12:** the LIVE preview uses the full-quality iteration appetite at
  extreme depth. `recommended_max_iter = (base + octaves·220).min(500_000)` (viewport.rs:244) → at
  ~6986 octaves always returns **500_000**; `zoom_iter_cap` (~1.79M there) doesn't bite; and the
  live path (`ui/central.rs:688`, + `nav_and_draw` for dual) uses `recommended_max_iter` DIRECTLY —
  the "live preview caps it lower (see build_params)" note in viewport.rs is STALE, no such cap
  exists. So the live settle + AA-ramp runs a 500k-iter reference (non-escaping at the −2.0 tip →
  full 500k orbit + 4M-node BLA) and overloads the GPU present. **Decisive test:** booting
  `auto_iter=false` + `max_iter=5000` (reference escapes at ~13k) → 45 s, NO watchdog, NO freeze;
  `auto_iter=true` (→500k) → freeze every time. **Fix:** give the LIVE preview a lower iteration cap
  than the export appetite (implement the cap the stale comment promises), targeted to bite only at
  extreme depth and tuned + verified across depths (too-low under-renders legitimate deep-interior
  live views). The export path keeps the full 500k and is already fast — leave it untouched.

- [x] **Glitch-correction pass went pathological (>1 hour) at extreme depth — FIXED v0.2.16
  (tiled dispatches + wall-clock time-box).** The multi-reference loop (`render_corrected_iter`,
  render.rs) re-rendered the WHOLE frame per correction pass with `bla_on=0`; deep-interior "dark
  dendrite core" pixels must iterate the FULL floatexp count that no acceleration skips (~50× normal
  cost — a 33×33 core window took >100 s = seconds/px), so un-tiled, one pass was a single
  **uninterruptible GPU dispatch** (>150 s for 47 px; ~10 min/pass at full res → >1 h over the loop)
  and a between-passes time-box couldn't bound it. **Fix = the approach this entry scoped:** new
  `fractadyne_gpu::render_iter_tiled` renders each correction pass in bounded per-tile dispatches
  (`CORRECT_WORK_BUDGET = 2e9`, ~10× smaller than the export path's because BLA-off cores cost ~50×),
  checking a `deadline` between tiles; `render_corrected_iter` / `render_export_corrected` take an
  `Option<Instant>` deadline (`GLITCH_CORRECT_BUDGET = 120 s` for exports, `None` for the fast
  selftest check) and the loop returns the best-effort merge when it fires. Verified: corpus 14
  glitch-on went from an infinite hang (killed at 600 s even at 160×90) to **144 s bounded** at
  320×180; selftest 63/63 + goldens 17/17 byte-identical (tiled iteration is per-pixel identical to
  un-tiled). Earlier reverted attempts (BLA-per-ref, `render_iter_region` window, span==0 guard, bare
  time-box) are superseded. ⚠**This does NOT clean corpus 14/15** — their noise is NOT glitches (the
  corpus render logs `glitch=0` at full res), so correction has nothing to fix there; see next item.

- [x] **Corpus 14/15 (1.2e148× / 3.7e163×) "interior SPECKLE" — RESOLVED 2026-07-12: it was COLORING
  (palette-cycle aliasing), NOT a render bug.** The escape VALUES are correct: a FEDUMP GPU dump
  (`reusetest.rs`, `FEDUMP=1`) of the smooth-iter row matches `tests/probe_fe.rs` — a faithful Rust
  transcription of the exact mode-2 kernel (df/Cdf/Fe Dekker/Knuth error-free transforms, shared exp,
  fe_add de>60, orbit_fe dip decode, Zhuoran rebase) — which is smooth (df32 == df64 == GPU). At these
  dense dendrite fields the smooth-iter counts are huge (~3e5–9e5) and vary steeply pixel-to-pixel, so
  `shade()`'s `palette(pv·cycle + offset)` with the fixed cycle (0.27) ALIASES a correct field into
  speckle. Colouring the SAME buffer with the frame's escape range mapped to the palette
  (auto-normalized: `cycle = N/range`, `offset = -min·cycle`) reveals the correct structure matching
  F3 arm-for-arm (spiral dendrites at 14; bulb boundary at 15). Corpus 14/15 renders regenerated with
  auto-normalized coloring (FEDUMP, 2560×1440 → 2×2 box downsample). ⚠**Three wrong diagnoses first,
  each disproven with a TOOL not a guess: glitches (glitch=0); reference coverage (a hill-climbed
  interior ref len=800001 still speckled — hill-climb reverted, 63/63 held); df32 precision (faithful
  df32/FTZ/NoFMA sims all smooth).** My earlier `cycle=2e-6` "coloring ruled out" step was WRONG — I
  sampled a smooth transition row (not the steep dendrite region) and mis-read aliased-gradient moiré as
  random noise. LESSON: dump the raw escape VALUES before blaming the perturbation. Probes
  (`probe_escape.rs`, `probe_fe.rs`) + FEDUMP are the harness.

- [x] **✅FIXED v0.2.18: corpus 15 (3.7e163×) deep dendrites — BLA-skip missed the rebase at orbit
  dips.** Loc 15's right-side dendrites were MISSING (smooth-orange where F3 shows dense dark spirals):
  those pixels escape at 928k–1.6M — PAST the reference orbit (~918k) — so they must rebase at the
  reference's near-zero orbit dips (|Z|≈1e-71 every 4383 iters). **Root cause: the GPU's BLA-skip path
  (`mandelbrot.wgsl`, mode-2 loop) never checked for a Zhuoran rebase after a skip** — it assumed
  "δz stays small in the BLA regime ⇒ rebasing never triggers here", which FAILS when a skip lands on a
  dip: |Z_nref|≈0 ⇒ |zfull|≈|δz| ⇒ the rebase condition `|zfull|<|δz|` holds. Without the check the deep
  pixels marched to the reference END and escaped prematurely there (measured GPU range **[885961,
  918967], interior=0**, 918967 ≈ the ref length). **Fix = a rebase check at the BLA landing** (mirrors
  the full-step rebase): after applying a skip, `zfull = Z_nref + δz`; if `|zfull| < |δz|`, rebase to
  index 0. Range restored to **[886019, 1599783], interior 70/57600**; the dendrites render, matching F3.
  **Goldens 17/17 byte-identical + selftest 63/63** (the rebase only fires at dip landings — deep-interior
  views — never on the shallow/normal goldens). ⚠**How it was found (disciplined):** ruled OUT the rebase
  magnitude precision (a df32/`sf_lt`-faithful `probe_fe` variant still reached 1.29M — the 24-bit
  comparison was NOT the bug), then SA (off), glitch (off in the export path), leaving BLA as the only
  difference from the correct CPU sim → read the BLA-skip code and found the missing rebase. The
  `best_reference` hill-climb (longer reference) was a DEAD END: the 1.54M interior ref overflows the GPU
  `max_buffer_binding_size` (128 MB / ~932k samples at 144 B/sample) → `create_bind_group` panic.
  ✅**Export panic GUARDED v0.2.22:** `check_orbit_binding` (fractadyne-gpu `export.rs`) checks
  `(orbit.len()+bla.len())·16 ≤ max_storage_buffer_binding_size` at the top of `render_export` /
  `render_iter` / `render_iter_tiled` and returns `GpuError::OrbitTooLarge` (the app already formats
  it as "Export failed: … reduce iterations") instead of letting `create_bind_group` panic. Uses the
  ACTUAL orbit+BLA sizes (not an estimate — loc 15's 918516-sample ref sits right at the 128 MB edge,
  so an estimated cap would regress it) and the device's real limit (bigger GPUs allow more). 63/63,
  no false-positive. ⚠**RESIDUAL: the LIVE view is only capped by `LIVE_ITER_CAP` (auto-iter=100k);**
  a loaded `.fdn` with `auto_iter=0` + a very high `max_iter` at a deep INTERIOR view could still build
  an oversized orbit and panic in the live `make_iter_bg` (returns a `BindGroup`, not a `Result`, so it
  needs a skip/cap not an error). All corpus `.fdn` have escaping refs (<932k) so they're safe; only a
  crafted pathological `.fdn` triggers it. Lower priority (rarer, more-involved).

- [~] **App: auto-normalize / adaptive-cycle coloring for extreme depth (the general fix behind
  14/15).** `shade()` (mandelbrot.wgsl:1277) does `palette(pv·cycle + offset)` with `pv` = the raw
  smooth-iter count; at deep zoom that is ~1e5–1e6 and varies steeply, so a fixed `cycle` aliases into
  speckle — for the corpus AND any deep dense view a USER zooms to. **EXPORT PATH SHIPPED v0.2.17
  (`--normalize`):** `render_export_normalized` (render.rs) is a two-pass — tiled iteration buffer
  (`render_iter_tiled`) → CPU min/max over escaped px → `cycle = (0.5 + slider·6)/range`,
  `offset = -min·cycle` → color → box-downsample. Gated by `coloring.normalize` (set by `--normalize`,
  transient); `render_export_view` routes to it; falls back for aux methods / all-interior / oversized
  supersampled buffers. `generate_corpus` reads a per-location `normalize = true` (14/15) and passes
  the flag, so 14/15 render natively; the FEDUMP scaffold was removed. selftest 63/63. **REMAINING:
  (a) the LIVE view still aliases at deep zoom — needs a per-frame GPU reduction for the range (touches
  the fragile live loop, deferred); (b) no UI toggle / session persistence yet (export/CLI only).**

- [x] **Uniform-exterior misrender past ~1e142× — FIXED in v0.2.6 (sub-f32 orbit dips).**
  Root cause: the 11–15 dive path's reference orbit passes within ~1e-71 of zero every 4383
  iterations; orbit samples are stored as plain df32, so those dips flushed to zero in the GPU
  buffer, dropping the `2Z·δz` recurrence term at exactly those iterations — past
  ~(dip ÷ per-period growth) zoom (~1e142×) that re-glued every pixel to the reference each
  period and frames rendered all-interior (the "uniform" color was the interior color). Fix:
  sub-1e-36 samples are stored extended-range as `[0.0, exponent, m_re+4, m_im]`
  (`pack_sample`/`sample_xy` in fractadyne-core) and decoded by the shader (`orbit_fe`/
  `orbit_cdf`). The marker is FINITE and provably unambiguous (a legit df32 pair with
  hi == 0.0 always has lo == 0.0, while lane 2 here is ≥ 2) — a first attempt used NaN
  and silently failed on the GPU: WGSL gives no NaN guarantees and compilers may fold
  `x != x` to false; the mode-2 rebase and Pauldelbrot-glitch comparisons also moved to scalar
  floatexp (`fe_abs_sf`/`sf_lt`) — both underflowed to `0 < 0` below dz.e ≈ −75 and were
  silently disabled. Diagnosed with two kept probe tests (`probe_orbit.rs`, `probe_escape.rs`:
  bignum dip profiles + CPU-perturbation escape times). Measured escape bands: loc 14
  304k–582k (cap now 800k), loc 15 894k–1.46M (cap now 1.6M) — the .exr's 6M was exploratory.

- [x] **Deep export throughput vs Fraktaler-3 — MEASURED 2026-07-11 (v0.2.10); the "~50× slower"
  claim is REFUTED: we are on par with F3.** A controlled render of corpus location 14 (= F3's
  me148) at F3's **2560×1440**, SA off / glitch off (corpus staging), on the 3080: **~15 s total**
  (render fn 14.8 s; 18 s incl. app boot) vs **F3's ~14 s** — roughly equal, not 50× (and not even
  2×). The `[fd-perf]` split: **GPU iterate 2.71 s** (nominal ~1.09e12 steps/s), GPU color 1 ms,
  and the remaining **~12 s is one-time CPU bignum setup** (reference orbit + BLA build, before the
  first tile — progress sits at 0% for ~8 s of that). Consistent with the `--profile`
  deep-interior-1e148 breakdown (~81% CPU setup). Counters confirmed the deep paths fire live:
  ext=281.9M (extended-range dip samples — the v0.2.6 fix executing), bla_skip=3.48e9 (would have
  wrapped u32 — validated the v0.2.10 per-tile-u64 fix), rebase=19.4M, maxiter=18,481. **Conclusion:
  the GPU iterate is fast and competitive; there is no 50× gap.** The old ~1e9-steps/s figure was
  stale (predated v0.2.x reference-reuse/BLA/extended-range, or was measured with glitch correction
  on = the separate >1 h pathology). **The cold CPU bignum setup (~12 s) breaks down (v0.2.11
  `[fd-ref]` timing split, me148 cold): `best_reference` candidate scoring ~9.3 s (79%!) + orbit
  build ~1.2 s + BLA build ~0.8 s.** So the ONE meaningful throughput lever is **`best_reference`
  scoring** (fractadyne-core), NOT the shader/orbit/BLA — the `--profile` path hides it because its
  timed reps reuse the reference and skip the scoring. Candidate ideas (all delicate — the scorer
  was tuned to avoid poor-reference glitches at depth, so any change needs a reuse-vs-quality golden
  check): score candidates to a much shallower iteration cap (a long-lived orbit is identifiable
  early; no need to run 800 k in bignum to rank), score fewer candidates, or reuse the boot-frame's
  reference for the export instead of re-picking cold. Not GPU-bound; mode-2 Fe per-iter cost is not
  the bottleneck.

## Autopilot / auto-zoom — target selection & path planning (user spec, 2026-08-07)

The current autopilot picks a direction from a geometric average of "interesting" pixels, which
oscillates and dead-ends. The machinery to do this properly already ships (Newton nucleus finding,
`nucleus_size`, the Misiurewicz solver, multiplier λ, DE and orbit-stat aux channels) — it just
isn't wired into target selection.

- [ ] **Target selection: score candidates, don't average.** Use Newton nucleus finding to locate
  the dominant period-*p* minibrot and its atom size, plus the Misiurewicz solve for nearby hubs,
  then SCORE the candidates. Useful score terms:
  - **Structure density** — local iteration-count entropy, or DE-gradient variance, in a window.
  - **Remaining depth** — atom size relative to the current view, i.e. how many decades of
    interest the target still has left in it.
  - **A penalty for near-tip locations.**
- [ ] **Aim beside the nucleus, not at it.** Landing exactly on a minibrot terminates the zoom in a
  dead end. Choose the aim point as an OFFSET from the nucleus — skirting it at a fraction of the
  atom size is precisely the Dinkydau shape-stacking manoeuvre that generates embedded Julia sets.
- [ ] **Prefer Misiurewicz targets as the other good class.** λ guarantees the descent never runs
  out: structure repeats every factor of |λ|, forever.
- [ ] **Path planning: receding horizon with commitment.** Pick a target several decades ahead,
  COMMIT to it, fly there smoothly, and only re-plan once within some fraction of the atom size.
  Commitment is what kills the oscillation.
- [ ] **Fly it with Van Wijk & Nuij (2003)**, "Smooth and efficient zooming and panning" — the
  closed-form optimal path in (center, log-zoom) space between two views such that apparent motion
  is perceptually uniform. Naive linear center interpolation with exponential zoom produces the
  "swing" artifact where the target lurches sideways before settling; Van Wijk–Nuij eliminates it.
  Plan in log-scale: zoom is exponential and everything is linear there. (This also applies to the
  tour camera, which interpolates centers linearly today — see `Playback::sample`.)
- [ ] **Two cautions specific to this stack:**
  - Center interpolation must happen in ARBITRARY PRECISION (or as a delta against a fixed
    reference), or the path quantizes into visible steps below ~1e15×.
  - Target re-planning must run OFF-THREAD — Newton refinement mid-flight would stall the live
    loop. Same concern the NR-zoom follow-up already flags
    (`NR_REFINE_MAX_BIT_ITERS` declines a synchronous refine past ~1 s of UI blocking).

## Design follow-ups (from mockup review, 2026-06-25)

- [ ] **CA 2-D Birth/Survive rows only go 0–5; must be 0–8.** A 2-D life-like cell
  has up to 8 neighbours, so rules like HighLife (B36/S23) can't be expressed.
  Fix in mockup `14` and in the real CA 2-D rule editor. (DESIGN.md §4.1)
- [ ] **Mockup `12` footer says "1 rule" but shows 2 rows** (the 2nd, `G →`, is an
  empty placeholder). Cosmetic — fix the count or drop the empty row.

## Milestone M0 — Foundations ✅ complete

Goal (DESIGN.md §15): workspace + wgpu/window/egui + a basic Mandelbrot
(f32 in-shader, f64 CPU viewport) + pan / wheel / box zoom.

- [x] Cargo workspace + 9-crate skeleton (DESIGN.md §10)
- [x] `fractadyne-core`: `Viewport` (f64) + pixel↔complex + pan/zoom + unit tests
- [x] `fractadyne-gpu`: Mandelbrot render pipeline (WGSL, smooth coloring) via an
      `egui_wgpu` paint callback
- [x] `fractadyne-app`: eframe shell (menu bar + canvas + status bar)
- [x] Pan (left-drag) + cursor-centered wheel zoom
- [x] **Box-zoom (right-drag rectangle)** — `Viewport::zoom_to_rect` + an amber
      selection overlay drawn over the canvas; verified by a unit test (6/6 pass).
- [x] **Continuous zoom** — hold **Space** (in) / **Shift+Space** (out),
      cursor-anchored; exponential rate (~2× per 1.5 s) with ease in/out for a
      relaxing glide. Tunable via `ZOOM_RATE` / `EASE_TAU` in `fractadyne-app`.
- [x] **Build verified** — `cargo build --workspace` succeeds; `cargo test -p
      fractadyne-core` passes **5/5**. rustc 1.96 + egui/eframe/wgpu 0.31; no code
      changes were needed (the API usage compiled clean) and the `fractadyne`
      binary links.
- [x] **Launch the GUI window** — verified live: Mandelbrot renders, drag-pan and
      wheel-zoom work (confirmed to 307× with crisp smooth-colored detail).

### Environment workarounds (this machine)

- **Small Windows page file → OS error 1455** when mmapping the large debuginfo
  rlibs (naga ~55 MB). Fixed via `[profile.dev] debug = false` in `Cargo.toml`
  (rlibs shrink ~2–10×). Re-enable `debug = "line-tables-only"` after enlarging
  the page file.
- **Cargo pipelining** produced unusable rmeta stubs here → disabled in
  `.cargo/config.toml`.
- Build at `-j 1`; it **self-resumes on retry** (transient `LNK1105` temp-file
  locks were a symptom of the same memory pressure).

### Build

```
# Requires the Rust toolchain (rustup). First build pulls wgpu/egui (~minutes).
cargo run -p fractadyne-app        # launch the app
cargo test  -p fractadyne-core     # viewport math tests
```

Pinned: egui / egui-wgpu / eframe **0.31** (eframe with the `wgpu` backend; wgpu is
used via `egui_wgpu`'s re-export so versions can't drift). If a different egui
release resolves, a few wgpu descriptor fields (`entry_point: Option<&str>`,
`compilation_options`, `cache`) may need tweaking — these match the wgpu that
eframe 0.31 ships.

## Milestone M1 — Coloring & state (in progress)

Goal (DESIGN.md §15): real coloring (palettes + the compute↔coloring split), tile
cache, adaptive iterations, auto-save/restore, and the first side panels.

- [x] **Preset palettes** — Ember / Ice / Nebula / Grayscale as gradient stops,
      interpolated in-shader, cyclic (`fractadyne-color`).
- [x] **Coloring side panel** — palette picker + Cycle / Offset / Max-iter sliders,
      live updates (first real panel from the mockups).
- [x] **Adaptive iteration count** — `Viewport::recommended_max_iter` scales iters
      with zoom octaves; Auto toggle + base slider in the Coloring panel.
- [x] **Compute↔coloring split** — iterate to an offscreen `R32Float` texture
      (recompute only on view/iter/size change); recolor every frame from it.
- [x] **Auto-save / restore** — session (location/zoom/coloring) persisted to TOML
      in the OS config dir, debounced + atomic on change/close (`fractadyne-state`).
- [~] **Tile cache + pan reprojection** — **pan reprojection DONE**: while dragging, the last
      settled iteration texture is frozen and translated in the color pass by the accumulated
      pixel offset (no bignum recompute, no re-iterate), so detail slides under the cursor; the
      revealed edge fills with the frame's average color; on settle it re-renders at full detail.
      Single + dual (left) at deep zoom. *(Scale reprojection now also exists — see the
      XaoS-style item below — but only as a deep-zoom stall fallback, not the primary zoom path.)*
- [~] **XaoS-style continuous-zoom pixel reuse (reuse-first zoom)** — the headline UX gap vs.
      XaoS. **PLAN + STAGE 0 landed 2026-07-11 (v0.2.12):** a full read of the live pipeline is
      written up as a concrete staged roadmap in **design/xaos-reuse.md** (Stage 0 verification →
      Stage 1 coordinate-keyed tile store → Stage 2 during-motion tile refine [the high-value,
      high-risk core] → Stage 3 shallow exact reuse), with the freeze/hang fragility constraints any
      new reuse path must respect. **Stage 0 shipped: `--reusetest`** (reusetest.rs) — a headless
      staleness harness measuring the color-pass reprojection vs a from-scratch render across
      Δ-octave zoom-ins. Data (RTX 3080): in fine detail the NEAREST reprojection loses real
      per-pixel iter fidelity fast (seahorse-1e6 ~61% of escaped px differ >2 iter by Δ=0.1 oct,
      rising slowly to ~68% by 1.0), a conservative raw-iter proxy (colored/perceptual staleness is
      lower). Quantifies WHY the reproject is only a placeholder and motivates the Stage-2 refine.
      **Stage 0 findings (v0.2.13–0.2.14):** a perceptual sRGB metric shows staleness is far below
      the raw-iter proxy (sRGBmean ~12–33/255) so REFRESH_OCTAVES=0.5 is validated; and a
      nearest-vs-bilinear comparison found **bilinear reprojection is WORSE by 4–16%** (it smears
      across escape-time bands) — nearest is correctly chosen, the filter is not the lever, and only
      the Stage-2 real-detail refine can reduce staleness. **Next: Stage 2 is its own dedicated task**
      (re-iterates during motion on a fragile loop — gate it with the `--reusetest` colored golden;
      needs live visual verification, so best done in a focused session). *Earlier shipped (v0.1.53–57,
      0.1.66): reuse-first refresh for mode-0 zoom, deep-dive reference **reuse** (extend the cached
      orbit, ~20× faster rebuilds), frozen-frame reprojection/hold, and adaptive motion resolution
      (AIMD) — deep zoom is smooth in motion; the full coordinate-keyed tile/mip reuse in (2) below
      remains open.* Today every zoom frame
      still re-renders from scratch on settle (GPU iterate → color), so a deep dive
      visibly pixelates/blanks until the frame settles; XaoS instead *remaps already-computed
      pixels* from the previous frame each step and only computes what's newly needed, so zooming
      feels continuous. **Foundation already present:** the color shader does an affine
      scale+translate of the frozen iteration texture (`uv_scale`/`uv_off` — `mandelbrot.wgsl`
      ~L1148), and `render.rs` computes `reproject_scale = 2^(l2_frozen − l2_now)` +
      `frozen_center`/`frozen_l2` (~L881–909). But it fires **only** as a stall fallback when the
      deep reference goes `too_stale` — not on shallow/normal zoom, and it just holds a *scaled*
      (upsampled, blurry) copy until a fresh reference snaps in. To make it XaoS-like:
      1. **Promote reuse to the primary zoom path** (all depths, every zoom frame): start each
         frame from the reprojected prior texture instead of black, so there's never a
         blank/pixellated intermediate.
      2. **Refine, don't just upscale** — a scaled frozen texture is upsampling, not real detail.
         Re-iterate only the newly-revealed annulus at the edges + progressively re-iterate the
         interior at correct resolution (center-out or priority tiles) so reused regions stay
         sharp while new detail streams in. Needs the long-planned **coordinate-keyed tile/mip
         cache** (the "persistent tile cache" noted above) so tiles survive across frames.
      3. **Shallow regime (mode 1, <1e4×) can reuse *exactly*** — direct per-pixel dwell means the
         iteration counts can be remapped by coordinate (true XaoS reuse: recompute only the
         rows/columns that moved past tolerance), cheap and lossless; the deep regime is the
         tile-refine path in (2).
      Effort: **large.** The live pipeline is fragile (per the freeze/hang history), so this needs
      a careful progressive-refinement scheduler + a reuse-vs-full-render golden check (a reused
      frame must converge to the same image as a from-scratch render). Biggest single win for
      perceived smoothness; orthogonal to the perturbation math already in place.
- [x] **Palette animation** — Coloring panel "Animate" (Off / Forward / Reverse /
      Ping-pong / **Random gradients**) + logarithmic Speed slider; modes shift the
      color offset, **Random** synthesizes & continuously morphs gradients (seamless
      endpoints) with a "Shuffle gradient" button. Mode + speed persist.
- [x] **Harmonious random palettes** — `gen_stops` now uses one base hue + a gentle
      analogous excursion + a smooth dark→bright→dark `sin(πt)` arc (seamless), moderate
      constant saturation, dim ends — flowing/tasteful instead of a clashing rainbow.
      (Later polish: random complementary-pair / monochrome flavors.)
- [x] **Deep floatexp blank render investigated** — forced floatexp shallow renders
      clean → floatexp is healthy; the reported blank was a featureless fast-escape
      region (uniform escape ~iter 276, fast frame), not a render bug or build regression.
      A localized single-reference *glitch* remains possible (needs multi-ref correction).
- [x] **Orbit overlay polish** — tapered gradient polyline (thick/warm at z₀ →
      thin/magenta at the tail) instead of a flat line, with green z₀ / red last dots.
- [~] **Real (high-precision) orbit at depth, cursor-following** — past ~1e12× the
      overlay iterates in **bignum from the cursor's arbitrary-precision coordinate**
      (`pixel_to_complex` → `reference_orbit`), recomputed on cursor/view change and
      **cached** (`orbit_cache`). Runs toward escape (cap `ORBIT_MAX_DEEP=8192`) so the
      divergent (cursor-sensitive) tail shows, and **trims the `|z|>4` blow-up** so the
      normalized fit isn't dominated by one escaping iterate. Below ~1e12× uses the f64
      cursor orbit. **PENDING: confirm it reshapes as the cursor moves at deep zoom
      (build 16).** If still static at extreme depth, raise the cap / cap to eff_iter.
- [x] **Zoom display formatting** — magnification shows scientific notation with a
      12-digit mantissa above 1e12×; large integer magnifications drop the cluttering
      `.00`; small zooms trim trailing zeros.
- [x] **Dual-view toolbar icon** — custom-painted "two side-by-side rectangles"
      (`dual_toggle_button`), so it reads as the split view regardless of font glyphs.
- [x] **Gradient stop editor** (custom palettes) — SHIPPED: "Edit gradient…" in the Coloring
      panel opens the stop editor (`palette_editor_open`); up to eight stops, each a colour +
      position, seeded from a preset. (Item was stale-open; verified in the running app.)
- [x] **Bookmarks / presets library** — Bookmarks menu (+ ★ toolbar button) saves the
      current view (full-precision center via the export view-metadata blob) to
      `bookmarks.toml` in the config dir; click any bookmark to jump back instantly.
      Manage… window adds (with optional name), lists with zoom, and deletes. Invaluable
      now that deep zoom reaches extreme depths (re-zooming to 1e30× by hand is painful).
- [~] **Left Parameters panel** (type, power, location, zoom) — SUPERSEDED by the right-hand
      Controls panel plus the status bar; keeping a second panel would duplicate both. Not a gap.

### Planned settings (Preferences UI — mockup 10)

- [x] **Continuous-zoom rate** — "Zoom speed" slider (0.25×–4×, log scale) in the
      side panel's NAVIGATION section; multiplies `ZOOM_RATE`, persisted with session
      state (`SessionState::zoom_rate`, `serde(default)` so old files still load).
      *(Requested 2026-06-26.)*

## Milestone M2 — Deep zoom (in progress)

Goal (DESIGN.md §5): extreme-depth zoom via arbitrary-precision reference orbit + GPU
perturbation + series approximation + glitch correction. The headline feature.

- [x] **Perturbation pipeline** — CPU reference orbit (`f64`) → per-pixel `δz` on
      the GPU. Verified live; pushes usable zoom well past the `f32`-direct limit.
- [x] **Reference picker** — choose a long-orbit/interior reference within the view
      (`best_reference`); fixed the short-orbit interior artifacts. **Now scores
      candidates in bignum** (`orbit_length_bf`, capped) — f64 scoring collapsed at deep
      zoom and made cold jumps (bookmark reload / Open view / `--render`) pick a poor
      reference → uniform/glitch. Added BigFloat string round-trip tests (the bookmark
      coordinate round-trips to ~1e-79, so it was never a precision problem).
- [x] **Rebasing** (Zhuoran) — single-reference glitch handling; killed the on-zoom
      speckle and self-heals short references.
- [x] **Supersampling (SSAA)** — Anti-alias control (Off / 2× / 3×); averages an
      ss×ss block of samples to remove boundary aliasing at depth.
- [x] **Double-single (df64) reference** — reference stored hi/lo; perturbation uses
      the `Z_lo` correction. Removes the reference-precision noise (dominant ~1e6×).
- Note: df64 reference + rebasing render **cleanly to ~5×10¹⁴×** (verified) — far
  past estimate; reference/delta are no longer the limit. The **f64 center
  coordinate** is now the wall.
- [x] **Double-double (df64) center** — the deep-zoom coordinate **jump is gone**.
      Verified clean *and* smooth to **~4×10³⁰×** (df64 reference + rebasing held far
      past estimate — no noise; the earlier ~10¹⁵× GPU-noise prediction was wrong).
- [x] **Arbitrary-precision center + reference (extreme-depth zoom)** — center is now
      `astro_float::BigFloat` at a mantissa size that scales with zoom
      (`precision_for_magnification` = octaves + 64 guard bits), so the coordinate
      never runs out of digits → no jump at *any* depth. The reference orbit is
      iterated in bignum on the CPU and handed to the GPU as df64 samples
      (`Arc<Vec<[f32;4]>>`). The GPU does no bignum (pure f32/df64 perturbation).
      Fast `BigFloat`→`f64` via direct mantissa/exponent bit-reconstruction
      (`core::to_f64`, validated by roundtrip test) — no slow string formatting.
- [x] **Reference-orbit caching** — bignum is slow, so the orbit is recomputed only
      when the reference leaves the view (>0.5 span) or, once the view settles, when
      precision/iterations grow (>0.4 span drift, or higher precision/iter). During
      motion the cached orbit is reused (smooth); refinement happens on settle.
      Caveat to watch: at very deep *continuous* zoom a recompute can micro-stutter;
      optimize later (precision headroom / async recompute) if it shows.
- [x] **Double-single (df32) perturbation delta** — `δz` (and per-pixel `δc`) carried
      as hi/lo f32 pairs with compensated add/mul (`two_sum`/`two_prod` via fused
      `fma`) in the shader. `δc` is built from the *exact integer* texel coordinate ×
      a df32 per-texel step (uniform `step` + `res`), so it isn't pre-truncated by
      f32. Removes the interior speckle that appeared past ~10¹⁵× (was f32 delta
      precision, **not** iterations); should hold clean to ~10²²–10²³×. Assumes a
      fused `fma` (true on NVIDIA/AMD/Intel targets). df64 delta later if needed.
- [x] **Full-precision persisted center** — session now stores `center_x_str`/
      `center_y_str` (decimal, full precision) and restores via `parse_bf` (fallback to
      the old f64 fields). Deep-zoom locations now survive quit/restart instead of
      truncating to f64 → a wrong spot → uniform screen. Also fixed the autosave
      debounce so an animating palette offset no longer blocks the idle save.
- [x] Re-add `zoom_to_rect` unit test (dropped in the dd rewrite) — two tests in
      `fractadyne-core` cover centered uniform 4× scaling, the max()-fit invariant for
      off-center/non-aspect boxes, and drag-direction independence.
- [x] **UI digit separators** — commas on zoom/iter, spaces grouping coordinate digits.
- [x] **Floatexp perturbation δ (extreme depth)** — the df32 δ has f32's *exponent*
      floor, so its low word denormalizes/underflows ~1e31–1e32× → speckle breakdown.
      Added a floatexp δ (df32 mantissa + i32 exponent) that never underflows. **Hybrid
      by depth**: direct df32 (<1e4×) → df32 perturbation (1e4–1e28×, fast) → floatexp
      perturbation (≥1e28×, ~1.7× costlier, only when needed). Shared base-2 `delta_exp`
      keeps the input δ mantissas (step / ref_offset) O(1) at any depth. Verified clean
      via `--render` at 1e15/1e25/1e27(df32)/1e29/1e32(floatexp); shallow unchanged;
      crossover seamless. Benchmark score held (3220). Depth now bounded by the center
      coordinate precision (auto-scales while zooming) + iteration budget, not f32.
- [x] **Lifted the ~1e308× render ceiling (extended-range `FloatExp` scale)** — the viewport
      scale was `f64` (`units_per_pixel` underflowed, `magnification()` overflowed near
      1e308×), which was the real live-zoom wall (the bignum center already had no fixed precision cap).
      Replaced it with a `FloatExp` (`m·2^e`, i32 exponent): `Viewport::units_per_pixel` is
      now `FloatExp`, with `log2_magnification` + `precision_for_octaves` driving precision,
      `complex_span_fe`/`gpu_scale` (O(1) span mantissa + shared `delta_exp`) and
      `ref_offset_mantissa` feeding the GPU (the shader was already exponent-aware — no WGSL
      change), `set_center_log2mag` + `--zoom-log2` for deep jumps, session persistence via a
      stored exponent, and `fmt_zoom_log2` for the readout. Verified: bit-identical to 1e30×
      (selftest goldens), GPU renders correctly at **1e331×**, no regression. *(Follow-ups:
      goto-dialog + exported-image metadata still take f64 zoom — fine to ~1e308×.)*
- [x] **Deep goto / exported-metadata zoom past 1e308×** — the "Go to location" dialog and
      the reloadable PNG/EXR view-metadata encoded zoom as `f64`, so a view deeper than
      ~1e308× lost its scale on reload (the center was fine). Goto now parses/formats via
      `log2(magnification)` (`parse_zoom_to_log2` / `fmt_zoom_field` — accepts `1.5e400`,
      clamped to a sane octave bound); the metadata blob carries an extended-range
      `upp_log2` (reconstructed on load, with the f64 `upp` kept for back-compat), so
      exported images and bookmarks restore deep views exactly. Round-trip unit-tested.
- [~] **Full glitch correction** (Pauldelbrot criterion + multi-reference recompute). Multi-ref
      correction is implemented + validated, **on by default**, and covers **single + dual exports**
      (side-by-side + separate; ≤ ~32 MP / texture limit, non-aux; VRAM-capped; `--glitch`/
      `--no-glitch`). *Remaining:* live-view correction (settle-time / async — the live pipeline is
      fragile, so deferred).
- [x] **AA auto-drop during motion** — full AA only when the view settles (smooth
      deep zoom; sharp still image).
- [x] **Reference refresh during motion (anti-"impressionist")** — the reference orbit
      now refreshes while zooming (not just on settle) when out of view / under-precise,
      adaptively throttled (~2.5× last recompute duration) so deep zoom stays sharp in
      motion without stalling FPS. Supersedes the earlier "defer entirely during motion"
      that left stale references → blotchy frames. AA still applies only on settle.
- [x] **Hybrid direct/perturbation** — below **1e4×** iterate `z²+c` directly in df32
      (glitch-free); perturbation at/above 1e4×. `mode` uniform + df32 `center`; direct
      path shares the coloring/AA pipeline. Crossover is conservative: direct iteration
      accumulates rounding error and breaks down ~1e6× (random noise — that's *why*
      perturbation exists), so hand off long before. Verified clean at ~2e6×.
- [x] **Higher AA (4×/8×) + persisted AA** — exterior "speckle" was diagnosed (via
      the glitch-free direct path) as **undersampling of real sub-pixel exterior
      dust**, NOT precision or glitches: it persisted without perturbation and
      cleaned up at 8×. Added 4×/8× options (8× auto-reduced to fit the GPU texture
      limit) and persisted the AA choice (`SessionState::aa`). AA only runs on settle,
      so motion stays smooth.
- [ ] **Smarter exterior sampling** — adaptive/jittered supersampling or higher
      export-time AA so the dense dust sea is clean without brute 8× every frame.
- [ ] **Coloring tuning** — optionally scale color cycling with zoom so steep
      escape-time gradients don't read as grain at default Cycle.

## Milestone M3/M4 — Fractal variety & dual view (in progress)

- [x] **Fractal type system** — `Fractal` menu lists 10 escape-time families:
      Mandelbrot, Multibrot 3/4/5, Tricorn (Mandelbar), Burning Ship, Celtic, Buffalo,
      Phoenix, Newton (z³−1). Shader carries a `formula` id + `julia` flag (decoupled
      from the formula), with complex-df32 helpers (mul/sqr/div, `Cdf` struct) for
      powers and Newton. **Julia mode** is a toggle (Fractal menu) for any family; the
      **dual view** shows each family's map ↔ its Julia. Mandelbrot/Multibrot/Tricorn
      and the abs families (Burning Ship/Celtic/Buffalo) all deep-zoom at floatexp
      range; Phoenix/Newton are direct df32 (clean to ~1e6×).
- [x] **Per-fractal info panel** — collapsible section atop the side panel with the
      formula, a short background, and a reference hyperlink (Wikipedia / Paul Bourke),
      sourced from `FractalKind::info()`.
- [x] **Dual linked view** — View menu → "Dual view (Mandelbrot ↔ Julia)". GPU
      renderer refactored to per-view resources keyed by `view_id` (each panel has
      its own texture/uniforms/orbit/caching). Left = Mandelbrot, right = Julia;
      hovering the Mandelbrot sets the Julia `c` live. Each panel pans (drag) and
      wheel-zooms independently.
- [x] **Dual-view interaction** — per-panel drag-pan + wheel-zoom + continuous (Space)
      zoom toward the cursor; hovering the Mandelbrot drives the Julia `c` live (uses
      the global pointer position — per-widget hover was unreliable since both panels
      allocate from the same source line). Reset resets both panels in dual mode.
- [x] **Performance overlay + diagnostics** — draggable overlay (FPS, cpu vs gpu/idle,
      reference-recompute ms/rate, mode/iter/precision/orbit/zoom) + `[perf]` stderr
      log. On by default; toggle in View, or `--no-perf`. Caught a per-frame reference
      recompute loop (`best_reference` sits ~0.4 span off-center, which the stale check
      mis-flagged) that pinned the app at ~2 FPS; fixed → ~60+ FPS.
- [x] **Frame-rate cap** — View → Frame-rate cap (Uncapped/30/60/120, default 60),
      persisted; enforced by pacing the main loop (request_repaint_after is a deadline,
      not a throttle).
- [x] **Deep zoom for the analytic families** — perturbation generalized to
      Multibrot 3/4/5 and Tricorn (exact polynomial / anti-holomorphic δz series),
      in **both Mandelbrot and Julia modes**, sharing the bignum-reference + rebasing
      pipeline. Core `reference_orbit`/`best_reference`/`orbit_length` are now
      formula+mode aware (`step_bf`/`step_f64`, `cmul_bf`); the shader's perturbation
      branch carries δz in `Cdf` with the per-formula series. Verified vs the direct
      path at boundary regions (mean diff 0.001–0.4%).
- [x] **Per-view reference caches** — `ref_cache[2]` (main/left + dual Julia), each
      with its own orbit / ref_pt / orbit_id, so **both dual-view panels deep-zoom with
      perturbation** independently (previously dual was direct-only → the deep panel
      pixelated). `invalidate_refs()` drops both on formula/mode/center change.
- [x] **GPU watchdog (TDR) guard** — a heavy live render (high AA × deep iterations,
      esp. both dual panels) could exceed the OS GPU watchdog (~2 s) and crash with a
      device-lost error during `Queue::submit`. Added a per-render `WORK_BUDGET`
      (texels × iterations): supersampling auto-reduces on heavy frames (and, only on
      a very large window at extreme depth, the GPU iteration count is clamped) so a
      single submission stays well under the watchdog. Verified the previously-crashing
      deep dual Multibrot 5 8× case now survives. (Export already tiles, so it's safe.)
- [x] **Julia deep-zoom rebasing fix** — Zhuoran rebasing reset `δz = z_full`, which
      assumes `reference[0] = Z₀ = 0`. True for Mandelbrot, but a Julia reference orbit
      starts at `Z₀ = ref_point ≠ 0`, so every rebase offset the perturbation by `Z₀`
      and corrupted deep Julia renders (worse the deeper you go, as rebasing fires more
      often). Fixed by rebasing to `δz = z_full − reference[0]` (no-op for Mandelbrot).
      Applies to all analytic families in Julia mode + exports (shared shader).
- [x] **Burning Ship / Celtic / Buffalo perturbation** — sign-aware (abs) deep zoom
      for the non-analytic families. The abs fold on a z² component becomes a `diffabs`
      step `|c+d|−|c|` (KF/Zhuoran), evaluated branch-wise to avoid catastrophic
      cancellation: exactly `±d` when the reference and perturbed component share a sign,
      `±(2c+d)` across a sign flip (a wrong branch at a near-fold is the inherent glitch).
      Core `step_bf` gained the bignum reference iterations (5/6/7). The shader folds
      each abs component with diffabs in BOTH render paths: `df_diffabs` in the df32
      loop (mode 0, ~1e4×…~1e28×) and a scalar-floatexp `sf_diffabs` in the floatexp
      loop (mode 2, past ~1e28× — the complex `Fe` shares one exponent across re/im, so
      the per-component fold drops to a scalar `Sf` then recombines via `fe_from_sf`).
      So they now deep-zoom at floatexp range like the analytic families (vs ~1e6×
      direct before). Validated in `--selftest`: perturbation == direct at 1e5× (exact
      Burning Ship/Buffalo, mean Δ 0.18 iter Celtic, 0 px >2 iter), floatexp == df32 at
      1e10× (exact, all three), finite + structured at 1e35×. Lighting/DE stay off
      (non-holomorphic). Remaining: multi-reference glitch correction for the residual
      speckle at the abs folds (where a tiny df32 reference z² component flips the
      diffabs branch — same root cause as Mandelbrot perturbation glitches).
- [~] **Newton / Phoenix deep zoom** — **Phoenix DONE (v0.1.2):** perturbation deep zoom in df32
      (mode 0) + floatexp (mode 2), with the two-term `δz_{n-1}` register and previous-term rebasing
      (rebase-to-0 valid since the reference's `z_{-1}=0`); bignum reference + `orbit_length_bf` made
      Phoenix-aware. Validated in `--selftest` (mode 0 vs direct mean Δ 0.007 iter @1e5×; mode 2 vs
      mode 0 exact) + a core unit test. **Newton stays direct-only** — convergence-based with a
      nonlinear, coloring-incompatible perturbation (revisit only via a separate higher-precision
      *direct* path if there's demand).
- [x] **Click-to-pin Julia `c`** — in dual view, click the Mandelbrot to freeze the
      Julia at that point (a marker is drawn there); click the marker to release and
      resume live cursor-follow. Pinning also stops the per-move Julia reference
      recompute, so it's smoother at depth.
- [x] **Export hotkey (Ctrl+S)** — quick-saves the current view to the last-used folder
      with an auto timestamped name (no dialog), using current export settings.
- [x] **Dual export layouts** — Export dialog "Dual layout": Side-by-side (one
      stitched file, default), Separate files (`…_map` / `…_julia`), or Map only.
      Persisted. Verified the side-by-side stitch.
- [x] **Action toolbar** — fractal **dropdown** (the name is a picker), Julia + Dual
      toggles, Export / Gallery / Open… / Reset, Perf toggle. **Merged with the menu
      bar** on one `horizontal_wrapped` row: shares the menu line when the window is
      wide, wraps below when narrow. Action buttons use **emoji icons** (💾 🖼 📂 🔄
      📷 🔍± 🎨 📊 🖥) with tooltips; File-menu items are icon-prefixed.
- [x] **Docked performance panel** — the perf diagnostics render as a "PERFORMANCE"
      section at the **bottom of the right-hand control panel** (toggle via the Perf
      button) instead of a floating window. The whole right panel is **hidden in
      fullscreen** for an edge-to-edge view.
- [x] **More toolbar buttons** — Snap (quick export), Zoom +/− (about center),
      Palette (cycle preset), Fullscreen toggle, **Home 🏠 (animated zoom-out)**.
      (AA cycle / pin-release still candidates if wanted.)
- [x] **Animated "zoom home"** — 🏠 smoothly glides back to the default view (vs the
      instant 🔄 Reset). `Viewport::home_lerp` sets magnification from a log-mag track
      and lerps the center with `frac = 1 − 1/mag` so the focal point stays on-screen
      during the zoom-out (a linear lerp flings it off at depth). Eased (smoothstep),
      duration scales with depth (1.5–9 s), animates both panels in dual, treated as
      interaction (AA off / references deferred), and any pan/zoom/Space cancels it.
- [x] **Esc exits fullscreen**, in addition to the 🖥 toolbar toggle.
- [x] **Orbit overlay** — View → "Show orbits" draws the iteration path of the point
      under the cursor (`core::orbit_points`, f64, matches the shader's per-formula
      direct step incl. Burning Ship/Celtic/Buffalo/Phoenix/Newton). z₀ green, last
      iterate red, connecting polyline; works in single and dual (hovered panel).
- [x] **Higher max iterations** — base slider raised 4000 → **50,000** (logarithmic)
      to match the auto-scale cap; useful for deep minibrots / thin filaments.
- [x] **Dual-view polish** — draggable splitter (persisted `dual_split` fraction; drag the
      separator between panels, clamped 15–85%).
- [x] **Release build** — `[profile.release]` (debug=false, lto=false, codegen-units=16
      to bound compile memory) builds clean here. Measured via `--benchmark`: bignum
      **reference recompute 374 ms → 45 ms (~8×)**, avg CPU 2.5 ms → 0.33 ms (~7.6×),
      score 2750 → 3031. Deep-zoom recompute stutter cut ~8×; steady-state FPS is
      GPU-bound so only +10%. Build with `cargo build --release -p fractadyne-app -j 1`.
- [ ] L-systems, cellular automata (1-D & 2-D).

## Milestone M5 — High-res export (in progress)

- [x] **PNG / OpenEXR export** — `File → Export image…`: pick width (1280–7680),
      supersampling (1–4×), and format. Renders the current view offscreen at the
      chosen resolution (`fractadyne_gpu::render_export`: iterate → color → readback),
      then encodes via `fractadyne-export` (8-bit sRGB PNG with the linear→sRGB OETF;
      32-bit float linear EXR). Saves to the user's Pictures dir, timestamped; the
      dialog shows the resulting path. Reuses the live precision/AA/coloring pipeline
      so deep Mandelbrot exports use perturbation. Verified end-to-end (PNG + EXR).
- [x] **Tiled export** — renders in ≤2048-px tiles (sized to the texture + buffer
      limits) via a per-tile `px_offset` uniform, assembled on the CPU. Removes the
      ~8192 single-texture cap and fixes the large-size crash (was exceeding
      `max_buffer_size`). Verified seamless at 3840×2903.
- [x] **Native save/open dialogs** (`rfd`) — Export uses a Save dialog; `File ▸ Open
      view…` uses an Open dialog. Export width / supersampling / format and the **last
      save directory** persist in the session and default the dialogs.
- [x] **Reloadable PNG metadata** — exported PNGs embed a `tEXt` chunk with the full
      view state (fractal, Julia mode + c, **full-precision** center via
      `core::to_decimal_string`/`parse_bf`, units-per-pixel, iterations, palette,
      cycle/offset, AA). `File ▸ Open view…` restores it to continue exploring.
- [x] **EXR metadata** — same view state embedded as a custom `Fractadyne` OpenEXR
      attribute (write + read); `File ▸ Open view…` now accepts PNG *and* EXR.
- [x] **Background export** — render + encode run on a worker thread; the UI stays
      responsive (status polled via a channel; Export button disabled while busy).
      *(Cancelation is still TODO.)*
- [x] **Richer metadata + Notes** — exports now embed `app=Fractadyne`, `version`,
      `saved` (UTC date) + `saved_unix`, and a user **Notes** field (≤120 chars, in the
      Export dialog) alongside the view state.
- [x] **Export progress bar + cancel** — the Export dialog shows a live `ProgressBar`
      (% of tiles done) while rendering, with a **Cancel** button. `render_export`
      reports per-tile progress (permille via `AtomicU32`) and checks a cooperative
      `AtomicBool` cancel flag each tile (returns "canceled"). Verified: normal render
      reaches 100%, pre-set cancel aborts. Encode/write shows a distinct "Saving…"
      phase (progress sentinel ≥2000); default filename is now `..._YYYYMMDD_HHMMSS`.
- [x] **Gallery / metadata browser** — `File ▸ Gallery…` scans a folder (default
      Pictures, switchable) for exported PNG/EXR with Fractadyne metadata, newest
      first, showing a **thumbnail** + parsed metadata (fractal, zoom, saved date,
      notes, app/version) and a **one-click "Open this view"** to jump back in.
      Thumbnails decode lazily (one/frame, box-downsampled, cached as egui textures).
- [x] **EXR thumbnails** — gallery now decodes EXR too (`read_first_rgba_layer`,
      box-downsampled, linear→sRGB), so EXR entries get real thumbnails like PNG.

## Tooling, scripting & versioning (M7)

- [x] **Versioning + changelog** — workspace at **0.1.0**; `build.rs` auto-increments a
      per-build counter (`FRACT_BUILD`) shown as `v0.1.0 (build N)` in the title bar,
      Help menu, and export metadata. [CHANGELOG.md](CHANGELOG.md) tracks changes.
- [x] **Release / beta tracks (GitHub)** — DONE (2026-08-04): `release.yml` publishes a plain
      `vX.Y.Z` tag as a stable release marked *latest* (release track), and a `vX.Y.Z-beta.N` /
      `-rc.N` / `-alpha.N` tag as a GitHub *pre-release* (beta track). An update checker can then
      read `/releases/latest` for stable or the newest of `/releases` (incl. pre-releases) for beta.
- [x] **In-app update check (release / beta track selector)** — DONE (v0.2.39, 2026-08-04): the
      repo was made **public**, so `update.rs` queries the GitHub Releases API (`ureq` + rustls) on a
      background thread — Stable = `/releases/latest`, Beta = newest of `/releases` — semver-compares
      (incl. prerelease ordering) against the running version, and offers a "vX.Y.Z available →
      Download" link (opens the release page; no auto-install). Persisted **Stable / Beta** track +
      "check on launch" (off by default) in View → Settings; Help → "Check for updates"; CLI
      `--check-updates`. Validated end-to-end against the live API.
- [x] **Scripting (camera tours)** — Tools → "Play script…" loads a TOML of keyframes
      (`secs`, `center_x/y`, `mag`, `fractal`, `julia`; centers inherit if omitted) and
      glides center (BigFloat lerp) + log-magnification (eased) along the timeline.
      `core::set_center_mag` / `lerp_bf` drive it; Esc or Tools → Stop ends it.
- [x] **Guided-tour scripting (narrated, annotated tours)** — DONE (all five sub-items below):
      grew the keyframe format into an authored, self-documenting tour — captions, coordinate-
      anchored callouts, spotlight vignettes, per-segment easing + holds, and schema version
      tracking — rendered live and burned into `--render-tour` movie frames. Optional extras noted
      per sub-item (pause-until-dismissed captions, off-screen callout arrows, rect spotlights):
      - [x] **On-screen commentary / text** — DONE: `[[caption]]` entries (timed independently of
        keyframes) with `text` (multi-line), `at`/`secs`, `pos` (top/center/bottom), `fade`, and
        `size`. Eased fade in/out; wrapped + centred on a soft dark backing. Renders live
        (`draw_captions`, egui painter) **and** burned into exported tour frames (`stamp_caption`,
        rasterized from the font atlas). *(Remaining: optional pause-until-dismissed.)*
      - [x] **Callouts** — DONE: `[[callout]]` entries with a target `center_x`/`center_y` (fractal
        coordinate), `text`, `at`/`secs`, `fade`, `size`. Drawn as an amber marker ring + leader
        line + label, **anchored in fractal space** (new `Viewport::complex_to_pixel`, exact at any
        depth) so they track the point as the view pans/zooms; off-screen anchors are skipped. Live
        (`draw_callouts`) + exported frames (`stamp_callout`). *(Remaining: off-screen edge arrows.)*
      - [x] **Vignettes / spotlights** — DONE: `[[spotlight]]` entries dim everything outside a soft
        circle centred on a fractal coordinate (`center_x`/`center_y`), with `radius`/`softness`
        (frame-height fractions), `dim`, and `at`/`secs`/`fade`. Applied in the color shader
        (aspect-corrected round circle) so live + export are identical; anchored via
        `complex_to_pixel` so it tracks the point; the dim eases with the fade window.
        *(Remaining: rectangular regions.)*
      - [x] **Eased transitions** — DONE: per-keyframe `ease` (`smooth` default, `linear`,
        `smoother`, `in`, `out`) for the glide arriving at it, plus `hold` seconds to pause at a
        keyframe before the next glide. `Playback::sample` now splits each segment into a hold phase
        + an eased move phase (log2-mag + BigFloat-lerp as before). Verified: hold-window frames are
        identical, the hold extends the timeline.
      - [x] **Targeted version tracking** — DONE: scripts declare `format_version`; loading (live +
        `--render-tour`) warns when it exceeds this build's `SCRIPT_FORMAT_VERSION` (like the `.fdn`
        / export `NewerFormat` path). Schema is additive (unknown keys ignored, missing default), so
        old scripts still play.
      Pairs with the existing live playback + `--render-tour` movie export (annotations should
      render in exported frames too). *(Design the schema additively — new keys, old scripts still
      play — and reuse the hardened `meta_get`/version-check machinery for untrusted script files.)*
- [ ] **Tour editor (visual timeline)** — an in-app editor for camera tours: a scrubbing timeline
      with draggable **keyframes** (center/zoom/ease/hold) and separate lanes for **narration**
      (captions) + callouts/spotlights, live preview, and save back to the `.toml` script format.
      Today tours are hand-authored TOML ([TOURS.md](TOURS.md)); this makes authoring WYSIWYG. Builds
      on the existing `Playback` engine + `ScriptFile` schema. (Requested 2026-08-04.)
- [ ] **Start an offline render from the UI** — a dialog/button to kick off a headless
      `--render-tour`-style frame render (a chosen script → PNG sequence + optional mp4) as a
      **background job** inside the app, with progress + cancel, instead of the CLI. Pairs with the
      tour editor (render the tour you just authored). (Requested 2026-08-04.)
- [x] **Benchmark** — Tools → "Run benchmark" plays a fixed deep-zoom tour and samples
      FPS (avg/min/max), CPU ms, GPU ms (frame−cpu), and RAM (working set + peak via
      `K32GetProcessMemoryInfo`), reporting aggregates + score in a copy/save-able window.
- [x] **Benchmark system info** — report includes CPU brand (CPUID), physical/logical
      cores + L2/L3 cache (GetLogicalProcessorInformation), GPU name (wgpu adapter), and
      VRAM (display-adapter registry). Verified: Ryzen 9 3950X / RTX 3080 / 10 GB.
- [x] **CLI benchmark** — `fractadyne --benchmark [--out PATH]` runs the tour on
      startup, prints + saves the report, and quits (skips session autosave). Enables
      automated build/machine evaluation. Default out `fractadyne_benchmark.txt`.
- [ ] **Benchmark presets** — multiple scenes (Julia deep, Multibrot, dual) + CSV/JSON
      output and a results-history compare view.
- [x] **Headless render** — `fractadyne --render --out IMG [--fractal N --center X Y
      --zoom M --size W --ss N --iter K --julia --julia-c RE IM --palette I]` renders one
      image (reusing the tiled export + perturbation pipeline) and quits. PNG/EXR by
      extension; full-precision center. For debugging / automated golden-image checks.
- [x] **Development profiling harness** — `--profile` times the render stages (bignum
      reference orbit, series-approximation setup, GPU iterate / full render) per benchmark
      region and writes a JSON log to `logs/` with run context; `scripts/profile.ps1` runs it
      and `scripts/profile-compare.ps1` diffs before/after to validate optimizations. Logic in
      a `profile` module. **GPU timestamp queries DONE (v0.1.34):** `--profile` now reports pure-GPU
      `gpu-it` / `gpu-col` per-pass time (wgpu `TIMESTAMP_QUERY` + a `fractadyne_gpu::timing`
      thread-local capture bracketing `render_export`'s iterate/color passes), independent of the
      CPU submit/poll/readback overhead. *(Follow-ups: opt-in per-frame logging of live interactive
      sessions incl. a live perf-overlay GPU-timestamp row; fold the series coefficients into the
      reference-orbit pass to cut the ~100 ms series-skip setup at depth.)*
- [x] **Record-to-video / frame export** from a script (offline, deterministic) — done via
      `--render-tour` (see the Zoom-movie entry below).
- [x] **Ship compiled binaries on GitHub (Releases)** — `.github/workflows/release.yml`
      builds the Windows `x86_64` binary on `windows-latest` and, on a `v*` tag push,
      packages `fractadyne.exe` + README + both licenses into a versioned zip with a SHA-256
      sidecar and publishes a GitHub Release (auto-generated notes) via the `gh` CLI. A
      manual `workflow_dispatch` run instead uploads the zip as a downloadable artifact (no
      publish) for testing. Uses the standard `--release` profile (the local `-j1`/no-LTO
      constraints are this machine's page-file workaround; runners don't need them). Verified
      locally: the build command, output path, and the packaging/zip/checksum steps.
      README gained a **Download** section. Possible later: Linux/macOS jobs (need GTK/X11
      runner deps for `rfd`), code signing, and a more-optimized `dist` profile (LTO).
- [x] **Continuous integration** — `.github/workflows/ci.yml` gates every push to `main` and
      every PR: a **core-tests** job (`cargo test -p fractadyne-core --release` on Linux — the
      exact-math suite is pure Rust, no GPU/GUI/system deps) plus a **build** job
      (`cargo build --workspace` on Windows) confirming the GPU/egui crates still compile on
      the target. `concurrency` cancels superseded runs. The GPU `--selftest` needs a real
      GPU (runners have none → flaky), so it stays a local/manual gate. Verified both commands
      locally (29 core tests pass; workspace compiles). Possible later: a software-adapter
      `--selftest` job, `clippy`/`fmt` checks.
- [x] **File format versioning + minimum-version validation** — the reloadable view metadata
      (exports / `.fdn` / bookmarks) now has a single source-of-truth `VIEW_FORMAT_VERSION`
      (export.rs); the writer emits it and `load_view_metadata` returns a `ViewLoad`
      (`Ok` / `NewerFormat(v)`). A file whose `format_version` exceeds this build's loads
      best-effort (the format is additive key=value, so core fields still parse) but the
      untrusted callers — Open-view and Apply-location — surface a clear "saved by a newer
      Fractadyne; some settings may not apply, consider updating" message instead of
      silently mis-loading. Same pass **hardens the untrusted parser**: `max_iter` clamped
      to ≤1e7, `aa` to 1..16, zoom depth to ≤3.4e7 octaves (prevents a hostile `upp_log2`
      from ballooning bignum precision into a memory DoS), and `cycle`/`offset` rejected if
      non-finite. `--selftest` covers round-trip, newer-version detection, and clamping.
      (A file is missing `format_version` ⇒ treated as v1; legacy files still load.)
      Possible later: an explicit `min_app_version` for forward signaling of hard breaks.
- [ ] **In-app editor for the authorable files** — a text/TOML editor (probably a Tools →
      "Edit file…" panel) for the file types the app reads: tour scripts (`.toml`),
      profiling region files (`.toml`), `.fdn` locations, and response/`@args` files.
      Ultimately it should offer **schema validation** (flag unknown keys, out-of-range
      values, malformed sections before the file is used), **autocomplete** (key names,
      enum values, section templates), and **pasteable sample snippets** (a palette of
      ready-made keyframes / sections / whole example files to insert). The existing
      `TOUR_SCHEMA` in scripting.rs (which already generates TOURS.md) is the natural
      source of truth for the tour-script validation/autocomplete/samples; the untrusted
      parsers elsewhere give the range clamps to surface as validation errors. Could start
      as a validate-on-load "problems" list and grow toward live editing.

## Branding & UI (M7)

- [x] **Fractadyne theme + branding** — dark deep-space theme with cyan/magenta accents
      (`apply_brand_theme`), painted brand mark + wordmark in the top bar
      (`brand_wordmark`), and a procedural window icon (`brand_icon`).
- [x] **Animated relief lighting** — "Rotate light" spins the light direction over time
      (shares the Speed slider), complementing the animated distance glow + palette cycle.
- [ ] **Theme polish** — optional light/preset themes, custom font, accent picker.
- [ ] **Internationalization (user, 2026-08-09)** — externalize user-facing strings and support
  translated UI. Substantial: strings are inline across `ui/`, help.rs, dialogs, tooltips, and
  the status-bar diagnostics. Sequence AFTER the announce (an English-only announce is fine; a
  half-translated UI is not). Groundwork that is cheap now and pays later: route new user-facing
  strings through a single `tr!`-style shim instead of bare literals, and keep formatting
  (`format!` argument order) locale-safe. egui has no built-in i18n — likely `fluent` or a thin
  key→string table with a language picker in Preferences.

## Survey-driven roadmap (2026-06-28)

Gaps vs. Ultra Fractal / Kalles Fraktaler / XaoS / Mandelbulber / Apophysis, prioritized
for fun, informative value, and ease of use.

### Tier 1 — best value, good fit for the escape-time engine
- [x] **Distance-estimate slope/relief lighting** — tracks the derivative `dz/dc`
      (`dz/dz0` in Julia mode) and shades by the slope normal → embossed, lit 3D look.
      Works on the **direct path** (Cdf derivative) and the **perturbation paths**
      (floatexp derivative, so it holds at any depth — verified at 1e8×). Holomorphic
      families (Mandelbrot / Multibrot 3/4/5). Iter texture now RGBA32F (r=iter,
      g/b=normal, a=reserved for DE); light angle/relief live in the color pass so they
      re-light without re-iterating. Coloring panel toggle + angle/relief sliders;
      `--light [--light-angle R]` CLI; persisted.
- [x] **Distance-estimate glow + animation** — the derivative magnitude → distance
      estimate (stored as log2(pixels) in the iter texture's alpha); the color pass draws
      bright distance-contour bands that densify into glowing filaments near the boundary.
      Coloring panel: "Distance glow" toggle + Glow strength + Band width + "Animate glow"
      (flows the bands, shares the Speed slider). `--de` CLI; persisted. Works direct +
      perturbation (verified at 1e8×).
- [x] **More coloring methods** — Coloring → "Method": stripe average (+ density),
      triangle-inequality average (TIA), orbit trap (point/cross/circle, colors interior),
      distance estimate, and decomposition. Orbit stats accumulate into a second GPU
      render target (only when a method needs it); works at any depth (direct + both
      perturbation paths). Persisted; `--method/--stripe-freq/--trap` CLI. *(Follow-up:
      histogram/equalized auto-coloring still open.)*
- [x] **Goto-location dialog + navigation undo/redo** — View → "Go to location…":
      view/edit/paste/copy the exact center (full precision) + zoom, with validation.
      Navigation history records each settled location (+ discrete jumps); **Backspace**
      = undo view, **Shift+Backspace / Ctrl+Y** = redo (also in the View menu), gated so
      it doesn't fire while typing. (Single view; dual skipped.) *(Follow-up: this is
      the basis for the `.fdn` share format — same key=value, hardened parse.)*
- [x] **Period / minibrot finder ("zoom to center")** — View → "Find minibrot center"
      (or **M**) snaps the view center to the nearby minibrot's exact nucleus and reports
      its period via a transient toast. Detects the atom-domain period (global argmin of
      |Zₙ|), Newton-refines `c` so the critical orbit closes (`Z_period(c)=0`) in
      arbitrary precision, then recovers the true smallest period; rejects runaway Newton
      / non-nuclei. Holomorphic families (Mandelbrot / Multibrot). Unit-tested
      (period-2 → c=−1, period-3 bulb); verified deep (period-998 at 2e7×). Headless
      `--find-minibrot --center X Y [--zoom M] [--fractal NAME]`.
- [x] **Minimap / "you are here" overview + zoom-depth context** — View → "Minimap
      overview" shows a small static home-view thumbnail (rendered once per fractal/
      palette/method via the export pipeline) in the bottom-left, with a "you are here"
      marker (view rectangle when shallow, crosshair when sub-pixel deep) and the live
      zoom-depth label. Click to jump to a region at home zoom. Persisted; single
      Mandelbrot-mode only (hidden in dual / Julia).
- [x] **Gradient / palette editor** (custom palettes) — Coloring → "Edit gradient…" (or
      the "Custom" palette entry) opens an editor with a live gradient preview, per-stop
      color picker + position slider, add/remove stops (up to 8), and "Copy preset…" to
      seed from a built-in. Custom gradient persists and renders everywhere (live, export,
      minimap). Verified end-to-end via a custom-palette render.
- [x] **Famous-locations tour + "random interesting location" + help/keyboard overlay** —
      a **Locations** menu with curated named Mandelbrot spots (Seahorse/Elephant Valley,
      spirals, mini-Mandelbrot, a deep seahorse) that jump (full-precision) + a "🎲 Random
      location" that bisects to a random detail-rich boundary point and zooms in. A
      **Keyboard & controls** overlay (Help menu / **F1** / **?**) lists all shortcuts and
      feature tips. Famous coordinates verified to render detail.

### Tier 2 — high value, larger effort
- [x] **Shareable location `.fdn` + paste-text** — **File → "Share location…"** opens a
      dialog with the current view as a self-contained `.fdn` text blob (fractal,
      full-precision center, `upp_log2` so depths past 1e308× round-trip, zoom, coloring):
      **Copy** to clipboard, **Apply** a pasted/edited one, **Save .fdn… / Load .fdn…**.
      Untrusted input is handled safely — size-bounded (`SHARE_MAX`, plus a file-size check)
      and parsed through the existing **hardened, fuzzed** `load_view_metadata`/`meta_get`
      chain (key=value allow-list, every field validated/clamped, unknown keys ignored, no
      paths/code). *(Optional follow-up: QR-code generate/scan for the compact string.)*
- [x] **Auto-zoom (autopilot) — follow interesting areas downward** — hands-free continuous
      deep zoom that re-steers toward detail (XaoS-style), via **View → "Auto-zoom"** or the
      **A** key (Esc / any navigation input stops it). Every ~0.35 s it renders a small
      (56×56) iteration field of the current view through the live perturbation pipeline and
      scores each cell by **boundary adjacency + escape-time gradient**, center-biased for a
      stable dive; it eases the target and zooms toward it each frame (reusing `zoom_at` +
      the continuous-zoom rate). Stops on a dead end (no boundary detail → flat interior/exterior)
      or the user-set **dive limit** (Navigation-panel slider, persisted, 1e30×–1e5000×). Up to
      ~1e271× (the smooth regime) it glides; past that it switches to a **stepped dive** (jump ×4 →
      render → hold the last full frame while the next computes) so it reaches extreme depth without
      staring at a blank; re-evaluation is adaptive (spaces out as frames slow). Started/stopped by
      **A**, the **🛸 toolbar button** (highlighted while running), or the View menu; **Esc** stops
      it. *(Follow-ups: minibrot-seeking / boundary-tracking steering modes.)*
- [x] **Zoom-movie / frame→video export** — `--render-tour FILE [--fps N] [--size W]
      [--height H] [--ss N] [--out DIR]` renders a keyframe-tour TOML to a numbered PNG
      frame sequence (`frame_00000.png …`) for assembly into a video (prints an ffmpeg
      one-liner). Reuses the scripting keyframe interpolation — factored into
      `Playback::sample(t)` shared with live playback — and the offscreen export path; steps
      the timeline at fixed `fps`, recomputing a fresh deep reference per frame. Deep-correct
      (`set_center_log2mag`, octave-based precision) so dives past 1e308× sample exactly; this
      also upgraded **live** playback to the log2 path (was `set_center_mag`, which saturated
      at 1e308×). Example: `scripts/tour.example.toml`. Verified: a 9-frame 1→1e3× test dive
      renders correctly. Now prints live progress (frames done / elapsed / ETA / fps) and, with
      `--mp4 [PATH]`, assembles the frames into an H.264 mp4 via ffmpeg (frames kept; falls back to
      printing the assemble command if ffmpeg is absent). *Follow-up: in-app "Render tour…" UI.*
- [ ] **Layers + blend modes** (Ultra Fractal-style compositing).
- [ ] **Formula DSL / custom formulas** (M6).
- [~] **Series approximation** — order-3 polynomial (`δz ≈ A·δc + B·δc² + C·δc³`) seeds the
      perturbation and skips the early iterations. **Done for the holomorphic polynomial
      families — Mandelbrot + Multibrot 3/4/5 — on both perturbation paths: mode 2 (floatexp,
      ≥1e28×) and mode 0 (df32, 1e4–1e28×, the common range)**, non-Julia, non-aux coloring.
      Coefficients iterated in bignum alongside the reference (mode-independent), generalized
      to `z^d+c`: `A'=d·Z^{d-1}·A+1`, `B'=d·Z^{d-1}·B+C(d,2)·Z^{d-2}·A²`,
      `C'=d·Z^{d-1}·C+2C(d,2)·Z^{d-2}·AB+C(d,3)·Z^{d-3}·A³`. Skip chosen from the worst-case
      corner `|δc|` (cubic ≤ 2⁻¹⁶ of linear ⇒ no premature escape), cached per reference. The
      mode-0 seed is evaluated in floatexp (coeffs overflow f32) then collapsed to absolute
      df32 via `fe_to_cdf`; the GPU seed is formula-agnostic. Validated: core tests of the
      series vs exact perturbation for d=2 and d=3 (rel err <1e-3); seed vs full iteration
      `maxΔ 0` at 1e30× (mode 2) and 1e20× (mode 0); Multibrot 3/4/5 SA engages + matches
      SA-off. (Tricorn/abs families have no such δc expansion — anti-holomorphic / non-analytic.)
- [x] **BLA (bilinear approximation)** — skips iterations *throughout* the orbit (SA only skips
      the start). **On by default; ~5× faster GPU render at 1e30×.** A binary tree of merged linear maps `δz' ≈ A·δz + B·δc` (A=2Z, B=1 per
      Mandelbrot step) with validity radii; a pixel skips 2^l steps when `|δz| ≤` the merged
      radius (Zhuoran's BLA; KF2+/Fraktaler-3). **Phase 1 DONE (core, fractadyne-core):**
      `CFloatExp` (complex extended-range), `BlaNode`, `bla_merge`, `build_bla_mandel`
      (level tree, odd-tail carry); merged radius `min(r₁,(r₂−|B₁|·δc_max)/|A₁|)`. Validated:
      `bla_reproduces_exact_perturbation` — a BLA traversal matches full-step perturbation
      (rel err <1e-3) while skipping >¾ of iterations on a main-cardioid reference.
      **Phase 2a DONE (core reference algorithm):** `bla_iterate` — the exact per-pixel render
      the shader will mirror: skip with the highest valid level, **revert to a lower level /
      full step on escape overshoot**, full step when `|δz|` exceeds even the level-0 radius.
      Validated by `bla_matches_naive_including_escapes` (BLA == naive perturbation on the
      escape iteration for both BLA-engaged tiny-δc pixels and large-δc fast escapers).
      **Phase 2b DONE (GPU port, off by default):** the tree is appended after the reference
      in the SAME storage buffer (no new binding) — 4 `vec4` per node (`[A],[B],[a_exp,b_exp,
      r_exp,r_mant],[span]`); the shader reconstructs per-level offsets from `orbit_len` and
      ports `bla_iterate` into the mode-2 loop (skip highest valid level → revert on escape
      overshoot → full step), updating the derivative `D=A·D+B` on skips. Core packers
      `CFloatExp::to_mantissa_exp`/`FloatExp::to_f32_exp`/`bla_to_gpu`; one uniform flag
      `bla_on`; app gate `self.use_bla` (mode 2, Mandelbrot, non-Julia, non-aux). **Phase 2c DONE
      (user-facing + escape-path validated):** `use_bla` is now a persisted **View-menu toggle**
      (`SessionState::use_bla`), and the GPU escape-overshoot revert is validated — a new selftest
      "BLA escape path == non-BLA @1e30× (boundary)" renders a deep boundary view (**48400
      escapers**, 0 mismatch) alongside the all-interior nucleus test (48400 interior, 0 mismatch),
      so both BLA code paths are covered. **Measured (2026-07-01, RTX 3080 / Ryzen 3950X, via the
      new `scripts/profile-bla.ps1` + `--bla` flag):** at 1e30× (mode 2, SA on) BLA cuts the GPU
      render **73.4 → 12.7 ms (5.8×)**; the tree build costs **~20 ms** (CPU, currently per frame).
      Net: **2.2× faster even uncached** (build-every-frame + render, 73.4 → 33.1 ms) and **5.8×
      with a per-reference cache** (build amortized). Zero cost where it doesn't apply (mode 0/1,
      aux, Julia — `build_bla` returns early). **Conclusion: enabling by default is justified.**
      **Phase 2d DONE (per-reference caching):** `build_bla` split into `bla_eligible` + a
      **conservative, offset-independent `bla_dc_max`** (2.5× the view diagonal — covers the whole
      region a reference stays valid over, up to the ~1.5-span "gone" recompute threshold, with
      margin) + the tree build; the live path (`build_params`) now caches the tree in
      `RefCache.{bla, bla_id}` and rebuilds only when `orbit_id` changes. Validated: selftest still
      0 mismatch (interior + boundary) with the conservative bound, and the profile shows it barely
      costs skips (render 12.7 → 13.6 ms, still **5.4×** vs off). Effect: a settled deep view drops
      from build-every-frame (~35 ms, 2.2×) to render-only (~13.6 ms, **5.4×**), and the ~20 ms tree
      build becomes a one-time per-reference cost (like the reference orbit) instead of per-frame —
      removing the weak-CPU risk. **Phase 2e DONE (on by default):** `SessionState::use_bla` now
      defaults **on**; a `--no-bla` flag forces it off for profiling (`--bla` still forces on). The
      cache was hardened against the zoom-out edge case — it rebuilds when the view needs a larger
      `dc_max` than the cached tree was built for (compared in log2 space to avoid underflow), with
      2× headroom so continuous zoom-out doesn't thrash. Verified: `--profile` (no flag) engages
      BLA (render 70→13 ms), `--no-bla` disables it, selftest 53/53. The View-menu toggle disables
      it if an artifact ever shows. **Phase 3:** mode-0 (df32, 1e4–1e28×) + Multibrot.
- [x] **Multi-reference glitch correction** (Pauldelbrot criterion + per-glitch recompute) —
      beyond the current single-reference Zhuoran rebasing. **Shipping for single-view exports**
      via a "Glitch correction (export)" preference (View menu, persisted): detects perturbation
      glitches and re-renders those pixels against extra references until clean. **Phase 2c DONE
      (color + wire):** `fractadyne_gpu::color_iter_buffer` colors the merged glitch-free iteration
      buffer (non-aux methods); `FractadyneApp::render_export_corrected` = correction → color →
      `ExportResult`, wired into both the headless (`render_to_file`) and interactive
      (`start_export_to`, run synchronously) export paths, gated by `glitch_correct`
      (`SessionState`). Selftest "corrected buffer colors to a valid image" (52/52, goldens 4/4).
      *(Follow-ups: tiling so it applies past the GPU max texture dim; aux coloring methods
      (stripe/TIA/trap/decomp) — need per-orbit stats merged too; apply to the live settled view;
      dual-view layouts.)* **Phase 1 DONE (core algorithm,
      fractadyne-core, Mandelbrot):** `Perturb` outcome, `reference_orbit_f64`,
      `perturb_pixel_mandel` (Zhuoran rebasing + Pauldelbrot detection, δz carried in **f32** to
      mirror the GPU's df64-reference/df32-δz precision gap — the gap that makes glitches real and
      fixable), and `render_multiref_mandel` (detect glitched pixels → place a new reference at the
      glitch region's centroid → re-render + merge → repeat to convergence). Validated: a real
      period-3 minibrot with an off-nucleus reference induces glitches, correction converges (≥2
      references, 0 unresolved), and the result matches a bignum per-pixel oracle exactly
      (`multi_reference_resolves_glitches`); plus a perturbation-vs-direct accuracy test. As with
      BLA, the core algorithm is validated first. **Phase 2a DONE (GPU detection):** shader gains
      a `glitch_on` uniform + Pauldelbrot check (`|z|² < GLITCH_TOL2·|Z|²`) in both perturbation
      loops (mode 0 df32 + mode 2 floatexp), flagging glitched pixels with a `-2` sentinel in the
      iteration texture's `r` channel (the color pass already treats `r<0` as interior, so it's
      harmless when uncorrected). `ExportRequest.glitch_on` plumbs it through `render_iter`/
      `render_export`; live rendering leaves it 0. Selftest "glitch detection responds to
      reference quality" confirms detection fires and a far-offset reference flags ≥ the auto
      reference (50/50, goldens 4/4). **Phase 2b DONE (correction orchestration):**
      `FractadyneApp::render_corrected_iter` renders the iteration buffer with `glitch_on`, then
      repeatedly drops a fresh reference (bignum, via `compute_reference`) at the glitched region's
      centroid, re-renders, and adopts the newly-resolved pixels — until nothing is glitched or
      `max_refs`. Seeding at the exact pixel *center* (the +0.5 texel offset) makes δc = 0 there,
      so each pass resolves at least its seed ⇒ guaranteed convergence. Selftest "multi-reference
      correction resolves glitches" (seahorse 1e8×): 9 flagged → **0 residual** with 7 references
      (51/51, goldens 4/4). **Phase 2c (next):** color the corrected buffer (GPU color-only pass
      over an uploaded iteration texture) + wire into the export path behind a "Glitch correction"
      preference + tiling for exports larger than the GPU max texture dim.

### Tier 3 — big bets (separate engines)
- [ ] **3D fractals** (Mandelbulb / Mandelbox, ray-marched).
- [ ] **Flame / IFS fractals; L-systems; cellular automata.**

## UI test automation

- [x] **`--resizetest`** (2026-08-05): headless window-resize regression harness — scripted
  drag-resize through the real frame logic; asserts every painted frame is either an aspect-fit
  reproject or a re-iterate at the current size (exit 0 = pass). First run proved the app-side
  paint path aspect-correct, isolating the residual live "squashed resize" to compositor-level
  present lag.
- [ ] **Adopt `egui_kittest`** (egui's official test harness, same 0.31 workspace as our pinned
  egui) for pixel-level UI regression tests: drives `update()` headlessly via AccessKit
  (synthetic clicks/keys/resizes) and renders REAL wgpu snapshots (custom paint callbacks
  included) for image-diff assertions. Needs a small refactor: our `update()` uses
  `frame.wgpu_render_state()` only — extract `update_impl(&mut self, ctx, Option<&RenderState>)`
  so the harness can drive the app without an `eframe::Frame`. First tests: resize sequence
  snapshots (this bug class), dialog smoke tests (goto/share/report/update), menu navigation.
- [x] **Real-window smoke test** — `scripts/resize-smoke.ps1` (2026-08-05): launches the app
  sandboxed (`FRACTADYNE_CONFIG_DIR`), drags the window corner with OS-level synthetic input,
  and analyzes the per-resize-frame present cadence the app records (`FRACTADYNE_PERF=1` →
  `kind:"resize"` JSONL). Verdict thresholds built in; `-Session` measures at a real deep view.
  First results: deep session median **18 ms** (~vsync — presents keep pace; residual visible
  stretch is the endemic wgpu/DWM one-frame compositor scale), shallow default median **32 ms**
  (lags — per-tick re-iterate + texture realloc; see next item).
- [ ] **Shallow-view resize pacing** — the direct-mode path re-iterates + reallocates the
  iteration texture at every resize tick (~32 ms median present cadence vs 18 ms deep). Options:
  extend the hold/reproject-with-aspect-fit path to direct mode during resize bursts, or debounce
  texture reallocation (reproject between reallocs). Cosmetic polish; the deep case (the original
  report) is already at parity.

## Feature gaps vs. peer renderers (survey 2026-08-05: KF2 / Fraktaler-3 / Ultra Fractal / XaoS / Imagina)

- [ ] **KF `.map` palette import** — cheapest high-value gap: connects Fractadyne to the large
  existing KF palette culture (plain 256-entry RGB text files). Import into the custom-gradient
  editor; offer as a palette source alongside presets.
- [ ] **Custom formula / coloring scripting** — Ultra Fractal's killer feature (user-written
  formulas, layered coloring, transform stacks, community formula DB). Ours: 10 fixed families +
  6 coloring methods. A big lift (needs a shader-codegen or interpreter story for the deep-zoom
  perturbation path, not just direct mode) — scope carefully; maybe start with direct-mode-only
  user formulas.
- [ ] **Huge tiled exports** — KF renders wall-sized images by tiling; we cap at the GPU texture
  limit (~8–16k). The tiled-iterate machinery exists (`render_iter_tiled`) — an export path that
  stitches tiles to disk (PNG rows / EXR scanline blocks) would lift the ceiling to RAM/disk.
- [ ] **Acceleration breadth** — SA covers Mandelbrot + Multibrot 3–5 only; Fraktaler-3's
  bilinear/bivariate approximation family is broader and better tuned. Evaluate extending BLA
  variants to the abs-family (Burning Ship etc.) deep paths.
- [ ] **Glitch correction robustness at depth** — KF's Pauldelbrot correction is mature; ours has
  the >1 h deep-interior pathology (see Open bugs) and the corpus renders with correction off.
- [ ] **Ecosystem/platform** — no plugin system; Windows-only release builds (code is portable
  wgpu/egui — a Linux/macOS CI target is mostly release.yml work); KF interop partial (.kfr
  import yes, .kfb map files no).
- [ ] **Linux build target** — QUEUED (user is setting up a separate test machine). Build on
  **Ubuntu 22.04 LTS**: glibc compatibility only runs backwards, so a binary built there
  (glibc 2.35) runs on newer distros while one built on 24.04/Fedora will not run on 22.04 —
  build on the oldest glibc worth supporting. It also has the most trodden path for proprietary
  NVIDIA drivers and the widest wgpu/Vulkan testing. Work is mostly `release.yml` (add a
  ubuntu-22.04 job producing a tar.gz + sha256) plus whatever the first real run turns up
  (file dialogs via rfd need a portal/GTK dep; check the icon/font loading paths).
- [ ] **First-run experience** — AGREED DESIGN (2026-08-07): a welcome overlay on first launch
  covering navigation (drag to pan, wheel to zoom, click-to-zoom, `M` for the minibrot jump) and
  a couple of preset destinations, since deep-zoom apps are opaque to newcomers and this is the
  friction a forum reader hits in the first two minutes. ⚠**Not** a Simple/Advanced *mode*:
  hiding controls creates "where did that go" support questions and doubles the UI states to
  test. Instead ship the advanced sections **collapsed by default** (the Controls panel already
  labels one "Accelerators (advanced)") — same benefit for newcomers, nothing hidden, no mode
  flag to maintain.
- Out of scope (named deliberately): 3D fractals (Mandelbulber's domain).

## Proposed feature set — reconciled backlog (2026-08-06)

Source: `local/fractadyne-proposed-features-2026-08-05.md` (competitive synthesis), reconciled
against the implementation in `local/fractadyne-proposals-reconciliation-2026-08-06.md` — that
document carries file:line evidence for every status below. 44 proposed items: 3 already done,
17 partial, 24 absent. Priorities are the proposal's own tiers (**P0** core differentiator,
**P1** high-value follow-on, **P2** niche/research).

Two derivations block a disproportionate share of the list and are therefore sequenced first:
the **atom-size estimate** (unlocks Newton-Raphson zoom, nucleus size/orientation reporting,
embedded-Julia estimates) and the **multiplier λ** at a Misiurewicz point (unlocks λ-guided
descent and the Tan Lei self-checking test).

### Sequenced first — small, high-value, hard to get wrong

- [x] **Exact rational / complex coordinate entry** (P0, §4.1) — v0.2.40-beta.23. `parse_bf` now
  evaluates rational expressions (`-3/4`, `(1+2)/8`), and `parse_complex_prec` parses complex ones
  (`(37+16i)/100`) — the Go-to dialog's real field accepts a whole complex value and fills both
  coordinates. Both entry forms honour a caller-supplied precision floor taken from the target
  zoom, so an inexact rational carries enough digits for the depth it's viewed at (astro-float's
  `FromStr` sizes precision from the input's digit count, which left `0.37` at ~64 bits). Also
  closed a latent hazard: `FromStr` accepts `"1 2"` as `1`, so coordinates are now shape-validated
  before parsing. Verification: 5 core unit tests (exact values, precision floor, complex division,
  malformed rejection, 20k-case fuzz) + selftest group `coords` (4 checks; suite 83 → 87).
- [x] **Atom size + orientation at a nucleus** (P0, §3.2) — v0.2.40-beta.24. `nucleus_size`
  implements Munafo's estimate (`Λ = ∏2z_i`, `B = 1 + Σ1/Λ_i`, `size = 1/(BΛ²)`), returning
  `log2_size` (a log, so a 1e-1000-scale atom can't underflow) and `orientation`. Quadratic only —
  the Multibrot families need a different derivative recurrence and return `None` rather than a
  plausible wrong number. Also added `log2_abs` / `arg_bf` (exact past f64's range) and
  `refine_nucleus` / `nucleus_residual_log2`.
- [x] **Newton-Raphson zooming** (P0, §3.3) — v0.2.40-beta.24. "Find minibrot center" (M) and the
  Go-to feature finder now jump to the minibrot's own scale, not just its center: one click took a
  1e6× view to 3.2e15× onto a period-998 minibrot. The subtlety that made this more than a
  one-liner: `find_nucleus` stops when the Newton step falls below a tolerance derived from the
  *current view span*, and works at that view's precision, so its center is only view-accurate —
  useless for a destination whose whole span is 1e-16 or smaller. `newton_raphson_target` runs two
  passes (size → refine at the destination's precision → re-size), and the zoom never goes
  backwards. Verification: selftest group `nr-zoom` (2 checks) + 3 core unit tests; suite 87 → 89.
- [ ] **Newton-Raphson zoom follow-ups** — (a) run the refine off-thread; a period-100k atom at
  several thousand bits will block the UI thread the way the reference build used to. (b) Expose
  the framing fraction (`ATOM_FILL`, currently 0.25) or a "fraction of the way there" control, as
  the proposal's §4.4 guided descent wants. (c) Extend the size estimate to Multibrot.
- [x] **Multiplier λ at Misiurewicz points** (P1, §3.4) — v0.2.40-beta.25. `misiurewicz_multiplier`
  runs the critical orbit to the cycle then accumulates the derivative around it, reporting
  `log2|λ|` (the zoom period — the view repeats every that-many octaves) and `arg λ` (the twist per
  repeat). The Go-to dialog reports both after a solve. Verification: both closed-form cases exact
  to 1e-9 — c = −2 gives λ = 4 real (which is *why* the antenna tip repeats without spiralling),
  c = i gives 4(1+i) = 45° twist. Core unit test + selftest (suite 89 → 90). This unblocks the Tan
  Lei invariant test and λ-guided descent, which both need λ.
- [ ] **Landmark library expansion** (P1, §4.3 partial) — we ship 12 curated points; add the
  parabolic entrances (1/4, −3/4 as exact rationals), Feigenbaum point, Douady rabbit, Basilica,
  airplane minibrot, golden Siegel point, and the Pythagorean boundary point (37+16i)/100. Also
  wanted: search/filter, and animated fly-to (the zoom-home button already animates a fly-back).
  Verification: a self-checking test that each landmark *is* what it claims — nuclei re-converge
  under `find_nucleus` at their stated period, Misiurewicz points under `find_misiurewicz` at
  their stated (k,p).
- [ ] **KF `.map` palette import** (P1, §2.6 gap) — plain 256-entry RGB text; import into the
  custom-gradient editor and offer as a palette source alongside the presets. **NEXT UP after the
  format work.** Verification bar set by the user: exactness, not just "it loads" — obtain
  reference `.map` files AND the KF-rendered images that use them, then assert our render through
  the imported palette matches KF's colours with no artifacts (interpolation between the 256
  entries, and gamma/linear handling, are the two places this silently goes wrong). Needs those
  reference assets before starting.
- [x] **Grand-tour script: landmarks + pathological locations** — SHIPPED v0.2.40-beta.33 as
  `tours/grand-tour.toml` (~232 s): the whole set, the famous valleys, the exactly-known
  Misiurewicz points (c = −2, c = i, with their multipliers narrated), the feature showcase
  (dual Julia, orbit overlay, a second fractal family), then a single continuous dive through
  the three-spar family HOLDING at every depth that has broken us — 1.7e55 / 3.3e61 / 6.3e63 /
  2.6e72 / 2.0e82 / 6.5e94 — since they share one center. A flat frame at any hold is a
  regression. It found a real bug on its first run (offline iteration starvation, fixed in the
  same release). ⚠**Not yet fully validated end-to-end**: the deep chapter needs a bigger
  iteration budget than the renderer can currently survive (see the tour-render TDR bug in Open
  bugs) — validate the deep holds once that lands. Remaining from the original idea, now its own
  item: an automated PASS/FAIL harness over the rendered frames (per-frame escaped-pixel and
  capped fractions vs thresholds) so "it went black at depth" becomes a failing check.
- [ ] **Automated tour-frame verdict harness** — render `grand-tour.toml` (low fps/size) and
  assert per-frame flatness metrics: standard deviation and distinct-colour count over a sampled
  grid, plus the GPU capped fraction where available. The ad-hoc PowerShell version used during
  beta.33 development worked (flat frames read sd ≈ 0–8 and 2–12 colours against 60–100+ for a
  healthy frame, and PNG size is a good cheap proxy: 5–8 KB vs 150–240 KB) — make it a committed
  script or a `--tour-verify` flag so it runs unattended.
- [ ] **Original grand-tour spec (superseded, kept for the unbuilt parts)** — one tour that visits the known
  points of interest AND the locations that have historically broken us, serving as both the demo
  reel and a regression test. Candidates from this repo's own scar tissue: the Misiurewicz spar
  fields where the iteration cap starves and the view goes flat/black (the 1.7e55× / 5.17e55× /
  3.3e61× / 6.3e63× / 2.6e72× / 2e82× / 6.5e94× three-spar family — SEVEN separate bugs,
  beta.19/22/26/27/28/30/32; the recurring shape is a heuristic tuned at one depth failing at the
  next, so the tour must sample MANY depths of this family, not one),
  the e1216 dive the pacer/lookahead work targeted,
  the corpus-14/15 glitch-and-aliasing locations, the deep interior minibrots whose cores cost
  ~50× (the glitch-correct pathology), and the extreme-zoom tip at e21000. Wanted: a `--render-tour`
  pass that must complete without a black/flat frame, so "it went black at depth" becomes a
  failing check rather than a user report. Pairs naturally with the landmark-library expansion and
  with `--divetest` (which already plays real windows at every 100 decades but judges *timing*,
  not image content). Verification: per-keyframe frame statistics — escaped-pixel fraction and
  capped fraction — asserted against thresholds, so starvation and flatness both trip it.

### Rendering data architecture

- [ ] **Complete the G-buffer** (P0, §1.1 partial) — the iterate target already carries smooth
  iteration + slope normal + DE, with an aux target for orbit statistics; but the accumulators are
  *method-selected at iterate time* (`aux_on` / `color_method`), so only the active statistic
  exists. Emit all of them unconditionally, plus |z| and arg(z) at escape and a period/atom-domain
  ID. Cost: aux bandwidth. Payoff: the next item falls out for free.
- [ ] **Method switching without re-iterating** (P0, §1.2 partial) — palette, cycle, offset and
  lighting already recolor from the cached iterate texture; `color_method`, `stripe_freq` and
  `trap_type` are still in `IterKey`, so switching method or trap shape forces a full re-iterate.
  Drops out of the completed G-buffer above. Verification: assert the iterate pass does not run
  across a method change (GPU event counters / IterKey identity).
- [ ] **Raw-channel EXR export** (P1, §1.3 partial) — we write a linear *color* EXR with embedded
  view-state metadata, and can already *read* F3's named raw channels for cross-checks. Writing
  named raw channels (KF/zoomasm layout) would also upgrade the F3 corpus comparison from the
  image domain to per-channel tolerance (§5.4).
- [ ] **Exponential-map render mode** (P1, §1.4 absent) — zoomasm-compatible zoom-video assembly.
- [ ] **Resumable/checkpointed tiled export** (P2, §1.5 partial) — tiled iterate exists; see
  "Huge tiled exports" above for the disk-stitching half.

### Coloring & post-processing

- [ ] **Dithered 8-bit export** (P0, §2.1 gap) — banding is the #1 newcomer complaint; smooth
  coloring itself ships. Verification: histogram/banding metric on a shallow gradient render.
- [ ] **Interior distance-estimation coloring** (P0, §2.2 gap) — exterior DE ships and is stable
  past e300; interior is a flat color today.
- [ ] **Log-scaled + histogram/percentile palette mapping** (P0, §2.4 partial) — `--normalize` and
  the live "Normalize deep colors" toggle do *linear* min/max range mapping from GPU escape
  min/max. Log and histogram mapping are what keep color perceptually stable across a zoom video.
- [ ] **Curvature-average coloring** (P1, §2.5 gap) — stripe, triangle-inequality and orbit traps
  ship; curvature is the missing fourth. Also: line/stalk trap shapes (we have point/cross/circle).
- [ ] **Gradient editor depth** (P1, §2.6 partial) — curve-based stops, HSL editing, alpha. The
  editor, duotone/binary, random-palette generation and cycling animation already ship.
- [ ] **Layer compositing** (P1, §2.7 absent) — multiple colorings over one G-buffer with masks and
  blend modes (the Ultra Fractal moat; cheap once the G-buffer is complete since data is resident).
- [ ] **DE-adaptive anti-aliasing** (P1, §2.8 absent) — supersample only where DE says the boundary
  is dense, plus a proper downsampling filter. AA today is a fixed 1/2/3 supersample factor.
- [ ] **User-scriptable coloring** (P2, §2.9 absent) — a WGSL snippet slot over the G-buffer; see
  "Custom formula / coloring scripting" above for the shared codegen story.
- [ ] **Interior colorings** (P2, §2.10 absent) — period coloring, atom domains, interior-coordinate
  (multiplier map) shading.

### Structural mathematics

- [ ] **Live period readout in the HUD** (P0, §3.1 partial) — period is computed by the nucleus
  finder and shown in a transient toast; the proposal wants it ambient. Note detection is argmin
  of |Z_n| over the critical orbit, not box-period/ball arithmetic.
- [ ] **Misiurewicz (k,p) discovery** (P1, §3.4 partial) — the Newton solve ships but the user must
  supply k and p; derive them from the view instead.
- [ ] **Embedded-Julia size/orientation estimates** (P1, §3.5 absent) — shape-stacking navigation;
  depends on the atom-size derivation.
- [ ] **Orbit cycle detection + multipliers** (P1, §3.6 partial) — the cursor-point orbit plot,
  inset normalize and animated racing dot ship; periodic-cycle highlighting and multiplier
  reporting do not.
- [ ] **Critical-orbit overlay in the Julia panel** (P2, §3.7 gap) — the dual linked view otherwise
  matches the proposal: Julia c is driven live by the Mandelbrot cursor, pinnable, per-panel
  reference caches.
- [ ] **Guided descent modes** (P1, §4.4 partial) — approach-nucleus, λ-stepped Misiurewicz
  orbiting, minibrot skirting. Autopilot and scripted tours cover automated descent; these are the
  math-driven variants, gated on atom size and λ.
- [ ] **External angles and rays** (P2, §3.8 absent) — landing angles, parameter rays, equipotentials.
- [ ] **Internal addresses and tuning navigation** (P2, §3.9 absent) — "go to p/q bulb", display the
  current minibrot's internal address.
- [ ] **Verified computation mode** (P2, §3.10 absent) — interval/ball arithmetic for certified
  escape/membership and DE error bounds. Turns images into evidence for computer-assisted work.
- [ ] **Numeric egress** (P2, §3.11 absent) — iteration grids, orbits, ray data as CSV/NumPy via the
  headless CLI (which today emits images plus JSON perf/bench logs).

### Navigation

- [ ] **Auto-stretch** (P2, §4.5 absent) — unskew sheared deep locations from iteration histograms
  (F3-style). Prerequisite: no skew/rotate/stretch transform exists in the view model at all.
- [ ] **`.kfr` export / `.kfb` support** (§4.2 gap) — `.kfr` import ships (hardened + fuzzed);
  writing them back, and `.kfb` map files, do not.

### Testing & verification

- [ ] **Tan Lei invariant goldens** (P0, §5.1 absent) — at a Misiurewicz landmark, render at
  magnification m and at m·|λ| rotated by arg λ, and assert the two images converge. A
  self-checking correctness invariant needing **no stored reference render** — all 17 goldens
  today are blessed-image comparisons. Gated on λ (above) *and* on view rotation, which the
  viewport does not support (shares the prerequisite with auto-stretch).
- [ ] **Landmark benchmark regimes** (P1, §5.2 partial) — `--bench-matrix` covers zoom bands,
  coloring and per-fractal paths including a deep Misiurewicz (4,1) segment. Missing regimes:
  minibrot approach (period detection / rebasing / iteration blowup), parabolic valleys (quadratic
  iteration growth at 1/4 and −3/4), Siegel points (near-neutral dynamics).
- [ ] **λ-scaling zoom-loop test** (P1, §5.3 absent) — a Misiurewicz-centered zoom must become
  periodic; automated frame-difference check across one |λ| cycle.
- [ ] **Per-channel cross-tool differential testing** (P2, §5.4 partial) — the F3 corpus (20
  locations to 4.6e1105×, `generate_corpus.py --check` gate) compares in the image domain; raw
  EXR channel export would allow numeric per-channel tolerance.

Non-goals restated from the proposal: 3D/DE fractals, flame fractals, built-in video encoding
(export exponential-map EXR and let zoomasm own assembly).

## Performance & throughput (M7)

- [ ] **GPU-assisted reference-candidate scoring (the practical "bignum on GPU").** Full GPU bignum
  is a poor fit for the reference ORBIT itself — `z_{n+1} = z_n² + c` is inherently sequential, and
  WGSL lacks 64-bit ints / add-with-carry / extended-mul (16-bit-limb schoolbook arithmetic is
  possible but a single dependent chain runs at GPU-scalar speed, no faster than the CPU bignum we
  have; this is why KF / Fraktaler-3 / Imagina all keep references on CPU too). What DOES map to the
  GPU: the **parallel** bignum workload — `best_reference` scoring. Idea: score the ~101 candidates
  as a tiny GPU "render" — iterate each candidate's δ against the EXISTING reference orbit with the
  same floatexp perturbation kernel the pixels use (a candidate is just a point; its escape
  iteration IS its score). Needs a valid covering reference (fine during a dive; CPU bootstraps the
  cold start), and the scorer only needs coarse "who survives longest", so perturbation accuracy
  suffices. Would make re-picks ~instant at any depth (vs ~0.6 s CPU-parallel at e1216). Moderate
  complexity: a small compute pass + readback, reusing the existing iterate kernel. Alternative
  considered and rejected: limb-parallel bignum multiplication per orbit step on GPU (workgroup-wide
  carry propagation each iteration — latency-bound, not competitive).

- [x] **Deep-dive live "monocolor" past ~1e400× — FIXED v0.2.40-beta.7 (parallel `best_reference` +
  pipeline pacing).** Diagnosis: on a centered dive the picked reference (a nearby nucleus, a
  fraction of a span off-center) leaves the 0.7-span drift window every ~1 octave → a full
  `pick_reference` re-fires; its **sequential** candidate scoring crossed ~1 s at e400 (796 ms; 7.6 s
  at e1216), the async worker fell permanently behind the ~5–10 oct/s dive, and the screen reprojected
  an ever-staler frame into a monocolor blur. Fix 1: `best_reference` phase-1/phase-2 scoring now runs
  across ALL CORES (result-identical — bench-matrix 0 drift, goldens 17/17): **796→55 ms @e400,
  3.3 s→272 ms @e800, 7.6 s→609 ms @e1216 (~12–14×)**. Fix 2: `last_depth_lag` (octaves the view
  outran the cached BLA) now paces both the script-playback clock (dilates in the `PACE_LAG_LO=1.5 →
  HI=2.8` window) and the interactive zoom velocity (`paced_zoom_vel`) — the dive slows instead of
  blurring, so the screen always shows a fresh reference. Remaining ideas (not yet done): reduce
  re-pick frequency structurally (bias the pick toward near-center candidates so references survive
  more octaves; or pipeline the NEXT pick concurrently with serving the current one); make the pacer
  visible (status-bar "paced" tag); SA coefficient pass at depth (~1 s at e1216) is the next cold-path
  cost after the pick; live per-frame palette normalization at extreme depth (the deferred LIVE
  `--normalize` analogue) for coloring-compression cases the pacer can't help.

- [ ] **Deep floatexp *settled* frames are slow in filament fields — a shader-speed fix, NOT multi-reference.**
  *Update (v0.1.57–0.1.68): interactive MOTION is now smooth — reference-orbit reuse (~20× faster
  rebuilds), frozen-frame reprojection/hold, and adaptive motion resolution (AIMD) replaced the old
  "blank during deep dives." What remains is a full-detail SETTLED frame in filament/Misiurewicz
  fields.* Full profiling in the archived `multiref-live` design note (git history). Deep mode-2
  frames cost seconds; this forced the v0.1.10 "reproject during mode-2 motion" hang fix (responsive
  but **blank** during deep dives). **Multi-reference was validated and abandoned (2026-07-03):** a
  `--refdiag` prototype showed the deep-spiral/Misiurewicz views have **zero long/interior references**
  (every point escapes at ~2400–6490 iters; at 1e75× all collapse to 6490), so there's nothing to
  rebase onto — multi-ref can't help. A finer sweep showed the cost is flat iter 4000→10000, so it's
  **BLA failing to skip in the filament structure**, not rebasing. Confirmed non-fixes (don't retry):
  resolution/WORK_BUDGET reduction, BLA rebuild on zoom-in, longer/predictive reference selection
  (`REF_SCORE_SCAN` 4096→65536 was *slower*), multi-reference. **Real levers:** (1) cheaper floatexp
  ops in `mandelbrot.wgsl` — proportional speedup to every deep frame, best next candidate; (2) GPU
  occupancy (register pressure); (3) iteration cap during motion (trades detail); (4) accept it —
  export (`--render-tour`) already renders full detail per frame. `--refdiag` CLI added as a dev tool.
  - **Confirmed (2026-07-06): op-level shader micro-opts are noise vs the floatexp iteration.** A perf
    recon proposed gating the per-iteration aux stats on the selected color method and a 3-mul `c_sqr`.
    Both landed bit-exact (pixel-verified; goldens 17/17) but measured **perf-neutral** on the RTX 3080:
    removing *all* of decomposition's per-iteration aux work (atan2+sin+pow) moved 1e30× iterate
    298.2→297.0 ms, and `c_sqr` is CSE'd by the compiler. So the aux-at-depth cost is **not** the aux
    transcendentals — it's that stripe/TIA/decomp disable BLA/SA → 25k full floatexp steps (=the 297 ms;
    trap keeps BLA → 11 ms).
  - **Reranked by GPU timestamps (v0.1.34, RTX 3080, 1e30× mode-2 seahorse).** The new per-pass
    `gpu-it`/`gpu-col` split shows pure-GPU iterate is **1.4 ms WITH BLA vs ~316 ms without** — BLA is
    a **~220× lever**, and aux is slow at depth *only* because it disables BLA. So **(b) aux⇄BLA
    coexistence is #1** (a cheaper aux accumulation that lets SA/BLA skip, or per-orbit stats that
    survive skips); **(a) cheaper floatexp arithmetic is #2** — it only matters for the filament views
    where BLA genuinely can't skip. The color/downsample pass is **~0.01 ms** at 512² (negligible), and
    the CPU-timed `iter/render ms` columns carry ~9 ms of fixed submit+poll+readback overhead (smooth's
    true GPU iterate is 1.4 ms, not the 10.7 ms the CPU clock showed) — which is why op-counting on the
    CPU columns mis-ranked the earlier pass. Measure GPU levers with `gpu-it`, not the CPU columns.

- [x] **FIXED (v0.1.10): fast live dive hung ("Not Responding") crossing into floatexp (~1e28×+).**
  Reproduced 2026-07-03 by auto-playing `tours/deep-spiral-dive.toml` with per-frame stderr timing.
  **Root cause:** the mode-2 (floatexp) iterate shader spins **~5 s/frame** when its reference/BLA are
  even ~0.5–2 octaves depth-stale (a fresh reference renders the same view in ~18 ms; stale, the
  perturbation rebases/does full steps per pixel). Since GPU pixels run in parallel, the frame time is
  the *slowest pixel's* shader duration, so it's **independent of resolution** (proven: 24×18 iterate
  texture still spun) — the ~1 s frame present blocks the UI thread, and because `update()` can't run
  during the block the off-thread reference recompute can't install → feedback loop pinning the dive at
  ~1 fps. On a *centered* dive the existing positional `too_stale`/reproject freeze never fired
  (`drift ≈ 0`), so it always painted the stale reference. **Fix (`render.rs`):** in mode 2, (a) freeze
  = reproject (which skips the iterate pass) for **all interacting frames** — on a dive faster than the
  recompute latency every reference is stale on arrival and the spin onset is data-dependent, so no
  threshold safely lets a real frame through; and (b) also freeze while a `bla_dc_max`-based
  `depth_lag > 1.2` so a *settle* holds until the freshly-recomputed reference lands, then snaps to
  full detail. Result: max mode-2 frame **5167 ms → 32 ms**, tour dives smoothly to 1e193× live;
  selftest 55/55, goldens 4/4. Tradeoff: live mode-2 *motion* is soft (reprojected) and sharpens on
  pause — the offline `--render-tour` export path is unaffected (fresh sync reference per frame = full
  detail). **Dead ends (don't retry):** shrinking the mode-2 `WORK_BUDGET` (even /4000) does nothing
  (cost is per-pixel spin, not total work); a `depth_lag` threshold that still allows real motion
  frames is fragile (spin onset overlaps the "fresh" range). *Possible follow-up:* bound the mode-2
  shader's worst-pixel step count so real motion frames become safe (would restore live detail while
  diving), verified against goldens.

**Update (2026-07-02): the bottleneck moved.** The off-thread reference recompute (below) took the
bignum orbit off the render thread — `--benchmark` now shows **avg CPU 0.38 ms, avg GPU 20.3 ms**,
so the live cost is now the **GPU iterate pass**, not the reference. `--profile` breakdown (render ms
= GPU): home/1e4 ≈ 10 ms · 1e6 17 · 1e12 16 · 1e20 19 · 1e30 (mode 2, BLA) 12 · **1e30 stripe
(aux, no BLA) 214**. Findings:
- **BLA is the GPU lever, but only past ~1e28×.** Measured: forcing the floatexp+BLA path down to
  1e12/1e20 (lowering `PERT_FE_THRESHOLD`) made it **2.5–4× slower** (iter 40/73 ms vs df32 15/18 ms)
  — floatexp's per-op cost dwarfs the BLA skip until the skip becomes huge (~1e28×). The 1e28
  crossover is well-chosen; don't lower it. Getting BLA into the df32 range would need BLA applied in
  the df32 loop (coefficients overflow f32 → need an fe hybrid) — uncertain payoff, high risk.
- **Aux coloring (stripe / distance / orbit-trap) can't use BLA** (it needs every iteration's z), so
  it's ~10–17× slower at depth (214 ms at 1e30). Inherent; the fix is a cheaper aux accumulation, not
  BLA.
- The original premise below (reference is the bottleneck) is now **historical** — kept for context.

Prioritized after a multi-GPU assessment (2026-07-01). The deep-zoom bottleneck during motion is
the **serial arbitrary-precision reference orbit** (bignum CPU, ~45–77 ms recompute) — *not* GPU
work — so it can't be parallelized across GPUs (or even threads: a single orbit `z_{n+1}=z_n²+c`
is a sequential dependency chain). The live GPU render is already frame-capped (~100 FPS via
`WORK_BUDGET`). So these attack the real bottlenecks first; **multi-GPU is deferred** (see below).

- [~] **Off-thread reference recompute** — **DONE for the live view:** the deep-zoom recompute
  (reference orbit + series approximation + BLA tree — all bignum) now runs on a worker thread
  (`recompute_worker`); `build_params` keeps drawing with the cached reference and installs the
  result when it lands (only the very first, cold-start reference is synchronous). Validated with
  the new `--frametest` harness (dive → 1e30×): **recompute stalls 27 → 1**, build-time p95
  **91.8 → 0.1 ms**, max **196 → 30 ms** (the lone remaining stall is the cold start). Selftest
  53/53, goldens 4/4 (sync export path unchanged). **Remaining:** compute the **multiple glitch-
  correction references concurrently** (rayon) and **speculative** next-frame references; thread
  the glitch-corrected export so it doesn't block the UI.
- [~] **Faster / adaptive bignum reference** — the reference orbit is the deep-zoom wall.
  **Done so far:** the per-iteration `2xy` was formed with a *full bignum multiply by 2*; replaced
  with `double_bf` (exact base-2 exponent bump) in `step_bf` + `iter_zsq_c`. Measured via
  `--profile`: reference-orbit compute **−13–17% at 1e6–1e20×** (deep-1e20 14.5→12.6 ms), −6% at
  1e30×; goldens bit-identical (exact change). **Remaining:** audit the +64 guard bits (trim where
  safe), find a dedicated bignum square (x²/y² are still general muls), profile `astro_float` hot
  paths, and evaluate a GPU-bignum / fixed-point reference pass to move it off the single CPU core.
- [~] **Pipeline the export** — overlap render → encode so the CPU/GPU never idle waiting on I/O.
  - [x] **Tour frame encode** — `--render-tour` now hands finished frames to a background PNG
    encoder pool (bounded ~1 GB in-flight for backpressure), so deflate overlaps the next frame's
    render. Byte-identical output; win scales with resolution.
  - [ ] **Tile-level export pipeline** — overlap tile N+1 iterate with tile N async readback +
    encode inside `render_export` (still `poll(Wait)`-serial per tile). Smooths the synchronous
    glitch-corrected export too.
  - [x] **Reference precompute overlap (tours)** — DONE: the export path now routes through
    `recompute_worker` (unified with the live path), and `--render-tour` computes frame N+1's
    reference on a worker while frame N renders on the GPU, feeding it to
    `current_export_request_with_ref`. Gated to single-view successors with matching fractal/Julia
    state (falls back to synchronous otherwise — always correct). Verified byte-identical output
    (0/37 frames differ); measured ~1.2× on a 1e12 mode-0 dive at 1000px, more on deep mode-2
    large frames where the bignum reference is a larger share of the per-frame cost.
- [x] **BLA on by default** — shipped (`SessionState.use_bla` defaults true). Confirmed by
  `--profile` as the key GPU lever at ≥1e28× (1e30 iter 10 ms with BLA vs 174 ms without). Note it
  only helps past the floatexp crossover (see the update note above) and not for aux coloring.
- [ ] **Better single-GPU utilization** — before adding GPUs, check the live dispatch actually
  saturates the one GPU (occupancy, workgroup sizing, async compute for the iterate vs. color
  passes). Often a cheaper 1.5–2× than a second device.
- [ ] **Multi-GPU — offline/export only (deferred)** — a second GPU gives near-linear speedup for
  **embarrassingly-parallel batch work** (high-res export tiles, movie/tour frame sequences), and
  the export path already does CPU readback so there's no shared-texture problem. It does **not**
  help the serial reference orbit or the frame-capped live view, and interactive multi-GPU is very
  invasive (the `egui_wgpu` paint callback is single-device; wgpu has no cross-device sharing).
  Low priority: most users are single-GPU, and the items above are higher-ROI. Revisit only if
  batch-render throughput becomes the pain point — and only for the offline path. Measure the
  GPU-vs-reference split with `--profile` / `--benchmark` before investing.

## Backlog (later milestones — DESIGN.md §15)

- **M4** more fractal variety: L-systems, cellular automata
- **M5** high-res tiled export (PNG / OpenEXR)
- **M6** programmable engine (formula DSL → WGSL + CPU; custom coloring)
- **M7** polish & perf

## Stub crates (created, awaiting their milestone)

`fractadyne-color` (M1) · `fractadyne-render` (M1/M2) · `fractadyne-state` (M1) ·
`fractadyne-fractals` (M4) · `fractadyne-export` (M5) · `fractadyne-ui` (panels, M1+).
