# Live zoom smoothing — where the lag is, and the plan to remove it

*2026-09-16, build 3202 (`ee5cde9`), RTX 3080 · Vulkan, 1280×800 pt window @1.5×, 60 Hz. All
numbers are `--zoomtest` runs (`DIAGNOSTICS.md`); the JSON logs and the trace transcripts are in
the session scratchpad, the target files are `misi-49-3.fdn` (the hero Misiurewicz (49,3) spiral
centre, `max_iter=500000`, auto-iter on) and `mini-p1411.fdn` (the period-1411 island at 1.2e123,
`validation/dive-to-minibrot-1.6e123.toml`). Predecessor: `local/live-zoom-pipeline-analysis-2026-09-15.md`
(the interactive lookahead, P1, is now shipped in `b194e51`).*

## 1. Summary

A continuous 1× → 1e100 dive is **smooth from 1e8 down** (17.7–18.3 ms mean, ≤52 ms max, both at
1.0× and 4.0×, all the way to 1.2e123). Every stutter that remains sits in one of five places, and
four of them share one cause: **the live path has no smoothness-targeted sizing of its GPU work**.
Motion and pinned-refresh dispatches are sized either to the *TDR safety* budget (a 400 ms real
target) or, by accident, to the bootstrap floor when the "motion jam" gate trips. Which of the two
regimes a dive lands in depends on controller history, not on what the viewer is looking at.

| # | where | what the viewer sees | measured | root cause (traced) |
|---|---|---|---|---|
| F1 | 1e0–1e4, direct mode, a deep file's `max_iter` | 4–6 fps for the first 18–31 s at 1.0× | 165 ms mean (1e0–1e2), 75 ms (1e2–1e4); 200–283 ms frames of pure GPU wait; base 256 = flat 17.7 ms | the settled iteration probe climbs the boost ladder toward `eff=500000` with the set interior in view (`gpu_iter` 2000 → 145k), the budget controller *discards* the 274 ms readings (`steps < 0.7×budget`, and 274 < 400 ms is not "slow"), direct mode never runs the AIMD motion-resolution loop |
| F2 | the mode hand-overs, 1e4 (direct→df32) and 1e28 (df32→floatexp), at 4.0× | the old frame magnified 10× (1e4) / 24–53× (1e28) for 1.1–1.7 s, then a pop | held 3.3 oct @1e4; 4.5–5.7 oct @1e28; 1.26 oct at 1.0× | the budget resets to "unmeasured" at the switch, the pin's chunk ladder restarts at 256 steps ×1.5/pass, and **every reference install aborts the pin** (`PinStop::Orbit`) — including no-op installs of the identical orbit (`reuse-extend … ⚠DID NOT GROW`, `install v0: len=11412 (was len=11412)`) every 0.12–0.19 s from the reactive path or the lookahead alike |
| F3 | a JUMP to a deep view, then zoom ("paste a location, hold Space") | 26 fps with 200–445 ms stalls, every real refresh | 1e26→1e32 @1.0×: mean 39, p95 216, max 445 ms, 89 frames >100 ms; @4.0×: max 215. The continuous dive through the same band: 17.7 / 39 ms, none >100 | after the switch the budget converges on the 400 ms TDR target and **motion/pin passes are sized to the full budget** (`pace = 1.0` for interacting and pin frames, `budget_step = tdr_steps / px`), so each real refresh IS a 400 ms dispatch. The continuous dive only escapes because its budget sits at the 3e11 ceiling, every dispatch counts as "full-size", the motion-jam gate trips and clamps motion to bootstrap (1.5e8 steps) — smoothness by accident |
| F4 | steady deep dive at 4.0× | periodic blur→sharp pop | held frames grow 0.64–0.9 oct (1.55–1.9×) between refreshes at 4×; at 1.0× 70 % of frames are held, a refresh every ~68 ms | presenter policy `REFRESH_OCTAVES = 0.5`, `REFRESH_MAX_SECS = 0.15` is rate-blind; held frames are nearest-reprojected, no filtering |
| F5 | ~1e6, once per dive | one 218–318 ms held frame, then ~100 ms frames for a second | 3 of 3 untraced continuous runs (t ≈ 2.5 s after the switch); 0 of 2 traced or jump-started runs | OPEN — a held (no-dispatch) frame whose present blocked; intermittent, not yet captured under trace |

Not a cause: the interactive lookahead. `FRACTADYNE_NO_PREFETCH=1` at both hand-overs gives the
same held-frame maxima (2.98 vs 3.27 oct at 1e4; 4.67 vs 4.38 at 1e28) — the reactive path installs
just as often (69 installs in 8.5 s). The pacer never throttled in any run (0 %).

## 2. The measurements

### 2.1 The two test cases, 1× → 1e100 (332.19 octaves)

`--zoomtest 332.19 --zoomtest-rate R --zoomtest-location F.fdn --zoomtest-start-log2 0`

| run | secs | frames | mean / p99 / max ms | >33 / >50 / >100 ms | held max | installs (lookahead) |
|---|---|---|---|---|---|---|
| Misiurewicz → 1e100 @4.0× | 127 | 6785 | 18.8 / 32 / 246 | 64 / 46 / 32 | 5.74 oct (53×) | 1670 (637) |
| minibrot → 1e100 @4.0× | 125 | 6821 | 18.3 / 35 / 218 | 70 / 44 / 4 | 4.53 oct (23×) | 1683 (637) |
| minibrot → 1.2e123 (409 oct) @4.0× | 154 | 8429 | 18.3 / 26 / 318 | 68 / 45 / 5 | 4.61 oct | 2150 (791) |
| Misiurewicz → 1e100 @1.0× | 510 | 27088 | 18.8 / 34 / 256 | 281 / 190 / 133 | 1.26 oct (2.4×) | 2273 (637) |

Per depth band, Misiurewicz (mean ms; `scripts/zoomtest_report.py`):

| band | 1e0–1e2 | 1e2–1e4 | 1e4–1e6 | 1e6–1e8 | 1e8–1e20 | 1e20–1e40 | 1e40–1e100 |
|---|---|---|---|---|---|---|---|
| 1.0× | 165 (18 s) | 75 (13 s) | 19.1, max 114 | 18.4 | 18.3–18.5, max ≤73 | 17.7, held max 1.26 | 17.7, max ≤63 |
| 4.0× | 118 | 93 | 18.2, held max 3.74 | 18.1 | 18.1–18.3 | 17.8, held max 5.74 | 17.7, max ≤37 |

### 2.2 The controls that isolate each cause

| experiment | result | isolates |
|---|---|---|
| 14 oct @4× from 1×, base 500k vs base 256 (`FRACTADYNE_TRACE=req` asserts `eff=`) | 500k: 96 frames in 5 s (vacuous), `gpu_iter` 2000→145k, up to 6e10 steps/frame; 256: 304 frames, mean 17.7, max 21 ms, **0 > 33 ms** | F1 = the appetite |
| jump to 1e26 → 1e32, with and without `FRACTADYNE_TRACE=tile` | identical: @1.0× 38.3 / 38.7 ms mean, 433 / 445 max; @4.0× 28 / 25 mean | F3 is the regime, not the trace |
| same band inside the continuous dive | 17.7 mean, 39 max, 0 > 100 ms, res 1.00 (both rates) | F3 is controller history |
| `FRACTADYNE_TRACE=tile` pin counts, 1e26→1e32 | @4.0×: 14 pins, 6 adopted, **8 abandoned `reason=Orbit`**; @1.0×: 69 / 57 / 11 | F2 = installs abort pins |
| `FRACTADYNE_NO_PREFETCH=1` at 1e3→1e8 and 1e26→1e32 @4× | held max 2.98 (vs 3.27) and 4.67 (vs 4.38) oct | the lookahead is not the cause |

### 2.3 What the traces say (verbatim lines that matter)

*F1 (shallow, direct mode):*
`slow frame 259: dt=207ms = body 0ms + outside(acquire/present/idle) 207ms … mode=1 steps=5.996e10 budget=1.168e11`
`view=0 gpu_iterate=273.6ms IGNORED (steps=5.989e10 < 0.7×budget)`
`LIVE view=0 mode=1 … iter=145304 (gpu_iter=145304, eff=500000, boost=72.65)` (base 500k) vs
`iter=3184 (gpu_iter=3184, eff=3184, boost=1.00)` (base 256, same depth)

*F2 (the 1e4 hand-over at 4×, one pin per 0.18 s, none complete for 1.1 s):*
`mode switch to 0: budget 1.17e11 → unmeasured (bootstrap 4.00e6, ceiling 4.00e8), re-converging`
`pin-start v=0 f=208 ask=57849 step0=256` → `lookahead install: len=11412 … (was len=11412)` →
`pin-abandon v=0 f=218 reason=Orbit cur=2560 ask=57849` → `pin-start … step0=256` → … ×6 →
first `pin-adopt … f=266 lag_oct=0.40` 1.1 s later. Every install in that stretch:
`reuse-extend [lookahead]: prefix=11412 → len=11412 … ⚠DID NOT GROW`.

*F3 (after a jump, mode 2):*
`pin-start v=0 f=1303 ask=35035 step0=24852 res=1460x1102` → `slow frame 1303: dt=361ms = body 1ms +
outside 360ms … mode=2 steps=3.998e10 budget=3.999e10` → `pin-adopt f=1305` — a two-pass pin whose
opening pass is the whole budget.

## 3. Why the machinery behaves this way (code, with line references at `ee5cde9`)

1. **Two budgets, neither about smoothness.** `budget_step` (`render.rs:7730`) converges the
   per-dispatch step budget on `TDR_BUDGET_MS = 400 ms` real and discards any reading under
   `PRICE_REPRESENTATIVE_FRAC = 0.7` of the budget unless it is *slow*, i.e. > 400 ms. A 274 ms
   frame is therefore invisible to it. The motion resolution AIMD (`motion_res_step`,
   `render.rs:7847`, target 17–24 ms, floor `min_motion_res` 0.30) is the only smoothness loop —
   and it runs only for `interacting && is_pert` (`render.rs:5619`); direct mode uses the static
   `budget_res_scale = sqrt(WORK_BUDGET / (px·gpu_iter))` (`render.rs:5546`), which is why the
   shallow phase sat at `res=0.60` for 30 s of 200 ms frames.
2. **Motion and pin passes are budget-sized.** In the chunk walk, `pace = 1.0` when
   `interacting || pin_frame` (`render.rs:4538`) and `budget_step = tdr_steps·pace / px`; only
   *settled* passes are wall-clock priced through `chunk_step_factor(pass_dt, target)`
   (`render.rs:7295`, and even there the target is the 400 ms one). A pin's opening pass "runs at
   the budget's size" by design (`chunk_step_factor` returns 1.0 with no reading).
3. **The jam gate is the accidental smoother.** `motion_jam_counts` counts a dispatch as full-size
   when it is ≥ 0.7× the learned budget; three unpriced full-size dispatches clamp motion to
   `bootstrap_steps` (`render.rs:5901–5920`, `MOTION_UNPRICED_MAX = 3`). With the budget at the
   `TDR_STEPS_CEIL = 3e11` ceiling (a skip-rich deep regime makes nominal steps cheap), every
   dispatch is "full-size" and the gate stays tripped — the continuous dive rides bootstrap-size
   passes and is smooth. After a jump or a mode switch the budget is small and climbing, each
   350 ms pass retires before the next is issued, the gate never trips, and every pass is 400 ms.
4. **A pin dies on any reference change.** `pin_verdict` (`render.rs:7246`) returns
   `Stop(PinStop::Orbit)` whenever `orbit_id` or `orbit_len` differ from the pin's. `install_recompute`
   (`render.rs:1306`) bumps `orbit_id` on every install, and neither the lookahead
   (`lookahead_collect_install`, `render.rs:1636`) nor the reactive path checks whether the result
   is the orbit already installed. The display keeps serving the hold snapshot of the last complete
   frame until a pin adopts (`render.rs:5373`, `:5458`), so an aborted pin costs the viewer the whole
   pin's worth of zoom in extra magnification.
5. **The mode switch forgets what it knew.** `fe_budget → 0` ("unmeasured", `render.rs:5827`), and
   the pin ladder restarts from `step0 = 256` (the "bootstrap-collapsed 256-step opening grows ×1.5
   per measured pass", `PIN_MAX_FRAMES` doc) while readings keep being discarded as
   unrepresentative — 15 passes to reach a 60k ask, longer than the interval between installs at 4×.
6. **The appetite is depth-blind for a loaded file.** With auto-iter on, `eff_iter` is the file's
   `max_iter` at every depth; the boost ladder (`ITER_BOOST_MAX = 256`) climbs toward it while the
   view is settled and the interior is capped, and the boost is *kept* into motion
   (`render.rs:3994–4001`), so the shallow dive inherits a 145k–179k ask it does not need.
7. **The presenter is rate-blind.** `reuse_hold` (`render.rs:5691`) refreshes on
   `frozen_drift < REFRESH_OCTAVES (0.5)` and `REFRESH_MAX_SECS (0.15)`; at 2.67 oct/s that is a
   0.4–0.64 oct held frame between refreshes, nearest-sampled.

## 4. The plan

Ordered by expected effect per unit of risk. Each item names the code it touches, the invariant it
must keep, and the measurement that proves it.

### P-A  A motion smoothness budget (the core fix; F2, F3, most of F1)

Size every *motion* and *pin* pass to a real-time target instead of the TDR budget:

- New tunable `MOTION_PASS_MS` (≈ 10 ms; the present interval is 16.7 ms and the pass must leave
  room for the colour pass and present). In the chunk walk (`render.rs:4538`) replace
  `pace = 1.0 for interacting || pin_frame` with `chunk_step_factor(motion_pass_dt, MOTION_PASS_MS)`,
  where `motion_pass_dt` is the measured wall/timestamp cost of the previous *motion* pass in this
  mode (a per-view EMA seeded from the last settled reading — never "no reading ⇒ full budget").
- The pin's opening pass uses the same seed. A fresh progression must not start at the budget's size
  (`chunk_step_factor(0, ·) = 1.0` stays for *settled* walks, which the retreat and the TDR budget
  already govern).
- The TDR budget stays the hard ceiling (`min(tdr_steps, smooth_steps)`), so nothing here can size a
  larger dispatch than today. The device-loss rule ("the budget's only actuator is resolution; do not
  cap the iteration ask") is about the TDR controller and is untouched: this is a *pacing* of chunk
  passes, the walk still reaches the full ask.
- The motion-jam clamp keeps its role (an unpriced backlog) but should stop being the thing that makes
  dives smooth; once P-A lands, `--set MOTION_UNPRICED_MAX=999` must not change the zoomtest numbers.

Invariants: byte-identical settled output (the corpus goldens and `--livetest` 0-drift — pass
sizing never changes the iteration result); `--motiontest` "no partial pin adopted" still holds.
Proof: `--zoomtest 20 --zoomtest-start-log2 86.37` at 1.0× and 4.0× — max ≤ 50 ms, frames > 100 ms
= 0 (today 445 / 89 at 1.0×); the 1e4 hand-over band (`--zoomtest 16.6 --zoomtest-start-log2 9.97`)
held max ≤ 1 oct at 4× (today 3.3); the continuous 1× → 1e100 tables unchanged or better.

### P-B  Pins survive references that did not change (F2)

1. `install_recompute`: if the result is the *same* reference point (compare the pick's centre or
   the orbit's identity hash) with `orbit_len` equal to the installed one, drop it as a no-op — no
   `orbit_id` bump, no BLA re-upload. The trace shows every install between 1e4 and 1e6 and most at
   1e28–1e30 is this case (`⚠DID NOT GROW`).
2. `pin_verdict`: a pure extension of the same orbit (same point, `orbit_len` grew, prefix identical
   by construction — bignum iteration is deterministic) should `Continue`, not `Stop(Orbit)`. The
   chunk `sig` carries `orbit_len` for the rebase trigger; a resumed pass on an extended orbit is
   valid up to the old length, and mode 2 rebuilds the BLA table per pass anyway. Needs the resume
   contract re-read (`design/mode2-chunking.md` §12–13) before it is relied on; if it cannot be
   guaranteed, fall back to item 3.
3. Defer installs while a pin is within its last `PIN_INSTALL_GRACE` passes (≈ 3): park the result
   and install it at adopt/abandon. Costs at most one pin length of reference staleness — bounded by
   `PACE_LAG_LO = 1.5` against the lookahead's 0.5-oct spacing.

Proof: `FRACTADYNE_TRACE=tile` on 1e26→1e32 @4.0× — `pin-abandon reason=Orbit` from 8/14 to ≤ 1;
`--zoomtest` held max at 1e28 @4.0× from 5.7 oct to < 1; the 1e4 band likewise.

### P-C  The shallow phase and the appetite (F1)

1. Run the AIMD motion-resolution loop in direct mode as well (`render.rs:5619`: drop the `is_pert`
   condition for the motion branch; keep `budget_res_scale` as the safety floor).
2. A depth-relative appetite for a loaded file: with auto-iter on, treat the file's `max_iter` as the
   appetite *at the file's own depth* and scale it down for shallower views by the same slope
   `zoom_iter_cap` uses (256/octave), floored at the app default. A 1× view of a 1e246 file then
   starts near 2000 iterations instead of 500,000; the settled image at the file's depth is
   unchanged. (`ColoringConfig`/session: keep `max_iter` as saved; the scaling is a live-path rule,
   so the `.fdn` document is untouched.)
3. Do not carry the boost into motion when the view is shallower than the depth the boost was
   measured at (`render.rs:3994`): reset it on a mode switch downward, or key it to `log2mag`.

Proof: the base-500k 14-octave run @4× — 0 frames > 33 ms (today vacuous, 96 frames in 5 s); the
1.0× 1× → 1e100 run's 1e0–1e4 bands at ≤ 25 ms mean; the settled renders of the corpus and the hero
views byte-identical (the appetite rule must be a no-op at the file's depth).

### P-D  Presenter policy for fast zooms (F4)

1. Make the refresh cadence rate-aware: refresh when the held frame would exceed a target
   magnification (`HELD_MAG_MAX` ≈ 1.2×, i.e. 0.26 oct) *or* `REFRESH_MAX_SECS`, whichever first —
   feasible only once P-A makes a refresh cost ~10 ms.
2. Box-filter (or bilinear-with-mip) the held reprojection when it is magnified > 1.2× (the P4 item
   of the previous analysis; `aa_filter` gated on `will_reproject`). Nearest sampling is what makes
   the pop read as "blocky then sharp".

Proof: held max at 4.0× in 1e8–1e100 from 0.9 oct to ≤ 0.3; a user A/B at the Zoom speed slider's
top end.

### P-E  Remember across the hand-over (F2, F3 tail)

`bootstrap_steps(rate_this_mode, rate_other_mode)` already exists; make the mode switch seed the
new mode's `fe_budget` *and* the pin ladder's opening step from the measured rate of the mode being
left (scaled by the known per-iteration cost ratio df32 → floatexp), instead of `fe_budget = 0`.
Lower priority once P-A prices passes on wall time.

### P-F  The ~1e6 intermittent stall (F5)

Instrument before changing anything: record per frame whether the present wait or the body took the
time (the app's `slow frame` split, but into the zoomtest JSON), then run five untraced continuous
1× → 1e8 dives at 4× and keep the first that shows it. Candidates: a swap-chain acquire behind a
jam-released full-budget dispatch right after the switch; a lookahead build landing with the first
BLA upload. Not on the critical path — it is one frame per dive.

### P-G  Gates (all of the above)

Add a `zoomtest` matrix to the release checklist (and, once stable, a `--selftest` opt-in):

| case | command | pass |
|---|---|---|
| continuous, both rates | `--zoomtest 332.19 --zoomtest-location misi-49-3.fdn --zoomtest-start-log2 0 --zoomtest-rate {1.0,4.0}` | max ≤ 50 ms, > 100 ms = 0, held max ≤ 1 oct (4×) / 0.5 (1×) |
| jump-start | `--zoomtest 20 --zoomtest-start-log2 86.37 --zoomtest-rate {1.0,4.0}` | same |
| hand-over | `--zoomtest 16.6 --zoomtest-start-log2 9.97 --zoomtest-rate 4.0` | same |
| appetite | `--zoomtest 14 --zoomtest-start-log2 0 --zoomtest-rate 4.0` on the base-500k file | not vacuous, > 33 ms = 0 |

alongside the existing gates: `--selftest` + goldens, `--livetest tours/grand-tour.toml --size
480x270` 0-drift, `--motiontest`, `--autodive 32 --autodive-home 3` (no device loss — every change
above sizes dispatches *smaller*, never larger, and the check is cheap insurance).

## 5. Implementation status (2026-09-16, beta.108, uncommitted at time of writing)

Landed on `feat/deep-zoom-progressive-ssaa`, measured with the same harness at 0.5×, 1.0×, 2.0×
and 4.0× on the Zoom speed slider (`--zoomtest-rate`):

| what shipped | where |
|---|---|
| P-B.1 no-op installs dropped (`same_reference`: same point, length, completeness, tables, precision) | `install_recompute` |
| P-B.2 a same-point extension keeps the pin (`PinnedRefresh::ref_pt`, `PinInputs::same_point`, re-keyed on continue) | `pin_verdict`, the verdict site |
| P-A every moving refresh past 512 iterations is a pinned walk; pin passes use the settled walk's ledger — dispatch gated on GPU **completion callbacks** (`Perf::pin_inflight`, `PIN_INFLIGHT_MAX` 2 in flight), each pass **priced by its own GPU timestamp reading** (paired by nominal steps, `Perf::pin_pass_steps`; the completion stamp less `PIN_PASS_FRAMES` of latency stands in after `PIN_PRICE_WAIT_FRAMES`), per-band licence with a price-proportional fast lane for pins (`pin_fast_lane`, ×4..×16) and cold bands past the first-wrap storm inheriting half their predecessor's licence (`pin_band_license`), the wall-price factor (`chunk_step_factor` vs `MOTION_PIN_TARGET_MS` 20 ms), a growth limiter (×2, `MOTION_STEP_OPEN` 512), passes clipped at the band edge (`chunk_band_end`), a pin floor sized from the pin target and the mode's worst measured rate; the band ledger persists across the pins of a glide and resets when the reference POINT or LENGTH changes | `bp_chunk_tiling`, `chunk_over`, the pin-start block |
| harness fidelity: the headless `--livetest` / `--divetest` loops retire the pin gate after their synchronous readback (`Perf::retire_synchronous_dispatches`) and feed readings through the app's own `Perf::record_gpu_reading` | `livetest.rs`, `profile.rs` |
| P-A/P-D rate-aware refresh resolution: `Perf::zoom_oct_s` (observed oct/s), `refresh_passes_allowed` (`HELD_MAX_OCT` 0.5 / `REFRESH_TARGET_S` 0.25 ⇒ 11 frames at 4×, 15 at ≤1×), `refresh_res_cap` from the nominal-rate hint (`Perf::motion_rate`, GPU readings + wall lift/cut) **and** the measured duration of the last pin (`pin_frames_last`/`pin_res_last`, cost ∝ pixels, growth ≤ ×1.25 per pin); applied as a cap on the AIMD scale (deep motion) and on the work-budget scale (direct mode) | the `res_scale` block of `build_params` |
| harness: `res_px`, `oct_s`, `adopts` per frame; `pinned refreshes adopted` in the summary | `zoomtest.rs`, `scripts/zoomtest_report.py` |

Measured (build 3247 — the final build, with the pin licence rules of finding 8 below; RTX 3080,
1280×800 pt @1.5×; "before" = build 3202; the 1.0× continuous row is from build 3240, the licence
rules changed nothing measurable on the other rows):

| case | rate | mean / p99 / max ms before → after | > 100 ms before → after | held max (oct) before → after |
|---|---|---|---|---|
| continuous 1× → 1e100 | 4.0× | 18.8 / 32 / 246 → **17.8 / 19 / 61** | 32 → **0** | 5.74 → 4.55 (the 1e28 hand-over; ≤ 1.9 elsewhere) |
| continuous 1× → 1e100 | 1.0× | 18.8 / 34 / 256 → **17.8 / 19 / 45** | 133 → **0** | 1.26 → 1.33 |
| hand-over 1e3 → 1e8 | 4.0× | 20.0 / 45 / 57 → **17.8 / 19 / 26** | 0 → 0 | 3.27 → 3.25 |
| hand-over 1e3 → 1e8 | 0.5 / 1.0 / 2.0× | — → 17.7–17.8 / 18–19 / 62 · 25 · 26 | 0 | 0.53 / 0.92 / 1.83 |
| jump 1e26 → 1e32 | 4.0× | 25.1 / 179 / 215 → **18.1 / 26 / 38** | 13 → **0** | 4.38 → 2.67 |
| jump 1e26 → 1e32 | 2.0× | — → 18.6 / 43 / 69 | 0 | 1.08 |
| jump 1e26 → 1e32 | 1.0× | 38.7 / 391 / 445 → 20.4 / 91 / 126 | 89 → 6 | 0.94 → 1.02 |
| jump 1e26 → 1e32 | 0.5× | — → 23.0 / 187 / 293 | 84 | 0.63 |
| stripe (aux, unchunked) e246, 20 oct | 1.0 / 4.0× | — → 17.7 / 19 / 31 · 17.9 / 19 / 29 | 0 | 0.80 / 2.55 |
| `--motiontest` (1M explicit ask, 1e31×, 3.6 oct/s dive + 19 oct/s home glide) | — | A2: 1 adopt in 213 pins / 0 in 211 (a coin flip) → **2 adopts, PASS**, pins reach 130k–950k of 1M in the 36-frame window (were 33k–130k) | | |
| `--livetest` grand tour, headless | — | 24/24 checkpoints, 0 drift, both builds; pins 160 started / 0 adopted → **167 / 15 adopted** (the tour's glides run 5–19 oct/s, the drift window is 6–36 frames) | | |
| continuous 1× → 1e100, dispatched RESOLUTION (build 3263) | 1.0× | 30% linear (9% of pixels) for the whole dive → **native (1460 of 1460 px) from 1e6 down**, 18.0 / 18.1 / 63 ms, 0 > 100 (the unpaced build: 133 > 100) | | ≤ 1.2 oct |
| continuous 1× → 1e100, dispatched RESOLUTION | 4.0× | 30% linear → 1029 px at 1e0–1e4, **still 438 px (the floor) below 1e4** — a refresh has half an octave to finish at this rate, and the bands where no pin completes fall back to the model (finding 13); 17.8 ms, 0 > 100 | | 4.51 oct |
| `--autodive 32 --autodive-home 3` (the device-loss experiment, autopilot dive, explicit 250k) | — | before: 31 lethal readings, peak iterate 2314 ms, 14 lethal-band frames, home-glide peak 1197 ms → **0 lethal readings, peak iterate 138 ms, 0 lethal-band frames, home-glide peak 66 ms**; no device loss on either. The experiment now exits 2 ("did not reach the regime") on the paced build — its repro value for issue #1 is gone with the 400 ms passes it relied on. One earlier run of the paced build (3245) logged 45 in-flight lethal-band warnings at the settled 1e32 view; the final build's run logged none — run-to-run variance not yet characterised | | |

What the round found, beyond the plan (each cost a build cycle and is recorded in the code):

1. **No gate on the estimator's upward path.** "Only a reading ≥ ¼ of the current sizing may grow
   the rate" and "only a reading ≥ 1 ms may grow it" both deadlocked: a pessimistic cut sized the
   next passes below the gate, nothing could qualify again, the pin crawled at its floor, and the
   frozen frame aged 13 octaves (11,000×) on screen while the cadence read perfect. The rate is a
   hint; the licence, the limiter, the completion gate and the TDR budget are the guards.
2. **The present interval cannot price a pass.** The swap chain hides a pass's cost for two frames,
   so the 17 ms interval after a 100 ms pass priced it "cheap", it doubled, and three stacked before
   the acquire blocked for the whole backlog. The settled walk's quick-present drain proof has the
   same flaw, tolerated by its 400 ms target; pins price on the completion callback instead.
3. **Nominal steps are not cost** in two regimes: fast-escape regions (chain-bound, 0.2 ms for
   4e9 nominal) and the rebase storm at the end of an ESCAPED reference (cur ≈ orbit_len, 10–70×).
   A pass licensed in one band must not run into the next (band clip), and the residual stalls at
   0.5×–1.0× after a jump are that storm: the lookahead picks a reference that escapes at 29,159
   iterations against a 32k ask — a reference-pick property (`design/pick-redesign.md`), not pacing.
4. **A pin's opening pass** is an ordinary moving frame and was exempt from the licence; with an
   inflated hint it went out at the TDR budget's size — once per pin, at every rate.
5. **Per-band licences reset per pin** (each pin has a new signature) meant ~50 passes per pin;
   at 4× that is past the 2-octave drift abandon, so no pin adopted. The ledger now survives the
   pins of a glide.
6. **Completion callbacks are delivered on the queue's poll cadence**, so a completion stamp
   reads ~2 frames (33.5 ms) for a 1.7 ms pass, and a frame-counted price is quantized the same
   way: a 30 ms pass reads as "free", the ×4 fast lane makes it 120 ms, the cliff quarters it,
   and the cycle puts a 150 ms frame on screen every few pins. Only the GPU timestamp reading
   prices a 20 ms target; the pass is paired with its reading by nominal step count.
7. **One pass in flight is too few at 4×**: with the price loop three frames long, a 60k-iteration
   refresh outran the drift abandon and the held frame aged 12 octaves. Two in flight pipeline the
   latency away and bound the worst stall to two target-sized passes.
8. **The cold-band floor is the throughput limit of a cheap pin.** `--motiontest` (a 1M explicit
   ask at 1e31×, 3.6 oct/s — faster than the slider's top) went red on A2: 58 pins, 0 adopted, all
   abandoned for drift at cur ≈ 33k–130k of 1M. The trace: every pass priced 0.1–1 ms against the
   20 ms target, yet each cold band opened at 256 and climbed ×4 per price, and a price lands ~4
   frames after its dispatch (the timestamp readback), so a band cost ~16 frames and six cold bands
   between 16k and 1M cost 64+ of the 36 frames the drift window allows. The reference also moved
   with the zoom anchor, so the ledger reset (new point) between pins. Two rules, both
   rate-derived: a pin's licence grows in proportion to how far under target it priced
   (`pin_fast_lane`, ×4..×16, the grown pass predicted at half the target), and a pin's cold band
   inherits HALF its predecessor's licence once the pass starts past `2 × orbit_len` — beyond the
   first-wrap rebase storm, the one hot region at a KNOWN cursor that the floor rule exists for
   (`pin_band_license`); bands up to the storm still open at the floor, and the settled walk is
   untouched. Result: pins reach 130k–950k in the same window, 2 of 10 dive-phase pins adopt, the
   gate is green. The before binary passed A2 by ONE adopt in 213 pins and failed it (0 in 211) on
   the next run — it was a coin flip there, at 400 ms per pass.
9. **A headless harness must retire what its readback completed.** `--livetest` and `--divetest`
   render each frame synchronously and never run `update()`, so the completion callbacks the pin
   gate waits on were never armed: pins held for `PIN_COMPLETION_TIMEOUT_US` per pass and drifted
   out (160 started, 0 adopted, 150 for drift). `Perf::retire_synchronous_dispatches` after the
   readback, and the shared `Perf::record_gpu_reading` (mode rate, smoothness rate, the pin pass's
   price) in place of the harness's private copy of two of its three consumers. The GUI path was
   never affected; the harness now measures the same pins the GUI runs.

10. **"A transparent overlay of another location/zoom level" after a zoom (field report,
    2026-09-16, dual view at 2^49, prefer detail, 2× AA, 4.0×) — OPEN AND UNREPRODUCED.** The
    only blend the display can produce is the on-settle average; the ghost is a sample folded at
    another view. Seven harness runs under the reporter's own settings (`FRACTADYNE_ZOOMTEST_
    PROFILE=user`, the dual view driven by the virtual key), continuous glides and the log's
    tap-and-rest pattern (`FRACTADYNE_ZOOMTEST_TAPS`), at 1.0× and 4.0×, 2^48 to 2^71, all
    converged with every fold at sample 0's view, and the one-sample vs converged captures of the
    same view (`FRACTADYNE_ZOOMTEST_SETTLE`, `scratchpad/ghost_diff.py`) differ only by the
    de-speckle texture (blurred mean 1.8–2.5 / 255, 0.1–0.2 % of pixels over 16). What the review
    DID find and fix: the settled chunk walk's signature hashed the **f64 centre**, whose ulp is
    1/190 of the view width at 2^49 and one value for the whole neighbourhood at 2^800 — a small
    pan, or a re-picked reference of the same length at the same f64 point, kept the signature and
    the walk RESUMED its per-pixel state against the moved view: the old view's escaped pixels
    stay on screen under the new one, which is exactly a translucent copy of another location.
    The signature now carries the view's exact position (`pos_sig`: offset from the reference
    point as a 2^-delta_exp mantissa, span exponent and mantissa). Whether that was the reported
    ghost is unknown (the report says "zooming", the mechanism needs a walk resumed across a
    move); an always-on tripwire (`⚠FOLD AT ANOTHER VIEW` in the log, `Perf::accum_view0`) now
    names the sample and the two views the next time it happens, with no trace flag.

11. **A model that BOUNDS a measurement is a latch — the shallow-detail regression (field report,
    2026-09-16: "at low zooms there was both high detail and high frame rates; now detail is
    lost").** The moving-refresh size was `model_cap.min(measured_cap)`. The refresh renders at
    the capped size, so the measurement comes back capped, the measured half may raise it only
    ×1.25, and the model floors it again next frame: a latch at `min_motion_res` that no headroom
    escapes. Measured on the 1× → 1e100 dive at 4.0× (build 3247): the AIMD read **1.00 — native
    fits — at every depth past 1e8 while the dispatch went out at 438 of 1452 px (9% of the
    pixels) for the WHOLE dive**, at a flat 17.8 ms with the GPU idle. The same shape as the
    bootstrap floor in `budget_step` and the opening guess in `motion_rate_now`, both already
    recorded here: **a model is what you use until you have measured, never a bound on what you
    measured.** Fixed by `measured.unwrap_or(model)`, plus running the AIMD for shallow/direct
    motion at all — it was gated on `is_pert`, so a shallow zoom had no measured controller and
    the model's word was final. Measured after (1× → 1e4, RTX 3080, `gates.ps1 -Step shallow`):

    | case | before this round | the pacing round | now |
    |---|---|---|---|
    | ordinary ask (base 256), 1.0× | native, ~118 ms at a big ask | 0.52 of native (27% of pixels), 21.5 ms | **native (100%), 18.0 ms** |
    | ordinary ask (base 256), 4.0× | native | 1.00 median, p25 0.51, 21.6 ms | **1.00 median, p25 1.00, 17.8 ms** |
    | explicit 500,000 ask, 4.0× | native, 118 ms (8 fps) | 0.30 floor (9% of pixels), 17.9 ms | **0.88 (78%), 27.0 ms** |

    The last row is a real trade and the honest one: native at that appetite is 118 ms, so the
    loop finds the resolution that fits ~20 ms instead of either extreme.
12. **Two limits of the frame-INTERVAL signal, both measured 2026-09-17, both open.** They bound
    any controller that reads it, and the fix for both is the same:
    - **Quantized.** Under vsync a frame either fits (~17 ms) or misses (~33 ms), so the 17..24 ms
      deadband meant to let the loop settle is often unreachable: it can only find the edge by
      growing into a miss, and a frame parked in the deadband can never climb out (measured: a
      shallow 4.0× glide frozen at 0.57 of native for a whole run because every interval read
      22–23 ms). A remembered "last miss" ceiling was implemented, tested and **rejected**: it
      damped the sawtooth (0.88 → 0.70 of native, 23 late frames → 0) but paid for it in exactly
      the currency the report was about, and bought nothing in the case below.
    - **Not attributable.** At a high zoom rate the per-frame cost that is NOT pixels dominates:
      the same shallow view renders at 17.9 ms at FULL resolution at 1.0× and costs 22–23 ms at a
      THIRD of the pixels at 4.0× — four times the cost per pixel. The reference lookahead is
      ruled out (`FRACTADYNE_NO_PREFETCH=1`: 22.7 ms, unchanged) and so are installs (1 per run).
      Cutting resolution against that cost loses detail and buys nothing.

    **Next lever: drive the moving resolution from the GPU iterate TIMESTAMP** rather than the
    frame interval — unquantized, and attributable to pixels, so it can tell "this frame is
    expensive because of pixels" from "this frame is expensive for some other reason" and stop
    spending detail on the second. The readings already exist (`apply_iterate_measurement`); what
    is missing is pairing each one with the resolution its dispatch ran at, the way
    `pin_pass_steps` already pairs a pin's pass with its own reading.
13. ⛔**A PINNED refresh must fall back to the MODEL, never to the measured loop — and that is not
    a wart, it is the model's remaining job.** Item 11's fix left the 1e4–1e12 bands of a dive on
    the floor wherever no pin had completed (no pin measurement ⇒ model ⇒ floor), so the fallback
    was widened to "use the AIMD instead". That put a **50-octave-stale frame on screen**: a
    uniform smear of a frame magnified 10^15×, spotted live at 2^262 during a 1.0× dive (build
    3262) — the offline renderer draws a rich spiral for the same view. The deadlock is exact: a
    pin measurement can only be produced BY a pin that completes, and a pin completes only if
    something bounds its size. Unbounded, the first pin at a new depth goes out at native
    resolution against a 557,000-iteration ask, never finishes inside the 2-octave drift window,
    abandons, and the frozen frame it was going to replace ages without limit while the view keeps
    zooming. Same family as "a probe whose reading another rule discards never terminates": a
    measurement that can only be produced by the thing it is meant to size needs a guess to get
    started. The bands that never pin stay on the floor, which is the honest residual — the fix
    for them is the GPU-timestamp signal above (it needs no completed pin), not removing a bound.
    ⚠A jump straight to 2^250 followed by a glide does NOT reproduce it (both binaries: held frame
    max 0.83 oct, 48/49 pins adopted) — only the long continuous dive does, so the regime fence
    for this class is `zoomtest-full4`/`full1`, and the number to watch is `held frame max`.

Open after this round: the storm at slow rates after a jump (item 3 — pick a longer-lived
reference, or cap the ask to the escaped reference's length); the pacer now throttles 12–13 % of
frames at 4.0× (depth lag peaks ~3; the continuous descent and the e246 stripe run) where it
never did before — the effective rate is unchanged (2.62 oct/s), but it is a change in a
controller this round did not touch and should be traced; the headless tour's glides (5–19 oct/s)
still abandon most pins for drift — a harness fact, not a GUI one, since the slider tops out at
2.67 oct/s;
P-C (the shallow appetite for a loaded deep file) is now moot for cadence — the direct-mode phase
runs at the resolution floor and 17.7 ms — but still softens the first octaves of such a dive.

## 6. Order of work and what each step buys

| step | effort | removes |
|---|---|---|
| P-B.1 (no-op installs) | small | most pin aborts at 1e4–1e6 and 1e28–1e30; safe on its own |
| P-A (motion pass pacing) | medium | the jump-start 445 ms stalls, the hand-over blur, the reliance on the jam accident |
| P-C.1 + P-C.3 (AIMD in direct mode, boost not carried) | small | the 200 ms shallow frames for a loaded deep file |
| P-C.2 (depth-relative appetite) | medium, needs the corpus fence | the shallow appetite at its root |
| P-B.2/3 (pins survive extensions / deferred installs) | medium | the residual aborts at 4× |
| P-D (presenter) | medium | the blur→pop at high zoom rate |
| P-E, P-F | small / investigation | the hand-over tail; the one-frame 1e6 stall |

Every step is measured by the same harness that found the problem, so a step that does not move
its number is a finding, not a merge.
