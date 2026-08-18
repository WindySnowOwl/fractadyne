# Torture suite — design

Status: **DESIGN ONLY, nothing implemented.** Written 2026-08-15 after the RX 6800 XT device loss
(`reports/fractadyne-report-2026-08-15.txt`), which no existing harness could have caught.

Goal: one systematic suite that walks *increasing difficulty* across live interaction, tours and
offline rendering at several resolutions, stops at the first thing that breaks in each lane, and
emits a failure artifact good enough to diagnose from — the way that field crash report was.

---

## 1. What exists today

| Harness | Lane | Points | Depth range | Escalates? | Targetable? | Failure output |
|---|---|---|---|---|---|---|
| `--selftest` | mixed | 116 checks + 17 goldens | goldens ≤ **1e6** | no — flat | yes (`--selftest-filter`) | pass/fail + `validation/report.md` |
| `--livetest` grand tour | live | 24 checkpoints | e55–e94 holds | no — fixed tour | no | drift vs blessed JSON, stdout |
| `--livetest` ultra dive | live | 5 checkpoints | e30–e200 | no | no | same |
| F3 corpus | offline | 20 locations | e0–**e1008** | no | `--check` | maxΔ per location |
| `--bench-matrix` | perf | 22 segments | mixed | no | ? | JSON baseline, exit 2 on drift |
| `--gputest` | GPU primitives | op sweep | n/a | sweeps inputs | ? | per-op max error |
| `--uitest` / `--juliadive` | live UI | scripted walk | shallow | no | no | screenshots |
| `--reusetest`, `--resizetest`, `--frametest`, `--divetest`, `--oomtest`, `--burnin` | various | ad hoc | — | no | no | ad hoc |

Coverage is genuinely good. The problem is **shape**, not quantity.

### 1.1 Gaps, in the order they have cost us

**G1 — Nothing samples the mode 0→2 crossover.** `PERT_FE_THRESHOLD` (1e28) is where the 2:58
device-loss class lives and where the RX 6800 XT died. The grand tour *glides through* it at
t = 178.2 s and never holds there; no checkpoint, no golden, no corpus entry sits at e28. The single
most dangerous point in the app is unsampled — it is crossed at speed and never measured.

> **CORRECTNESS HALF CLOSED — beta.105 `b14058c`.** `--selftest` now brackets the switch: bignum-oracle
> entries at 1.3e26×, **9.3e27×** (the deepest view the selector still hands to df32) and **1.3e28×**
> (mode 2's floor), a df32-vs-floatexp comparison at that ceiling, and a bracket check asserting the
> selector still picks mode 0 below and mode 2 above — without which a threshold move would silently
> reduce the comparison to mode 2 vs mode 2, vacuously identical. The two representations agree
> *exactly* at 9.3e27× (mean Δ 0.0000 iter). Suite 116 → 121 checks.
>
> Two traps this uncovered, both worth knowing before authoring any threshold check:
> - **`make`'s `mag` argument is not the view magnification.** It builds `units_per_pixel` from a
>   3-unit span while `Viewport::magnification()` measures against `REFERENCE_HEIGHT = 4`, so the view
>   lands at 4/3 × `mag`. A crossover check written at 7.9e27 renders at 1.06e28 — the wrong side.
>   Scale by 3/4, and report actual magnifications in the check output.
> - **The battery's 38-digit minibrot nucleus is interior-filled at depth** — measured, all 48400
>   pixels reach `max_iter` at 9.3e27×. The oracle agrees there only on "never escapes"; it never
>   compares an escape *count*. Dwell checks need a structure-rich center (corpus location 07 escapes
>   on every pixel at the same depth).
>
> Still open: the **cost/stability** half. Nothing *holds* at e28 under a frame budget, which is the
> regime the device losses live in — that remains the `live/crossover/*` rungs below.

**G2 — No BLA-state axis.** Per-step cost swings ~100× on whether a BLA tree is live. Every
device loss has landed in the no-BLA regime (`bla_skip=0`, short escaped reference — the documented
`orbit_len=626` class). Grand-tour holds run `orbit_len` 258k–4M; the ultra dive runs 5k–24k.
**Neither harness can produce the regime that kills.** A suite that sweeps depth, resolution and
fractals but not this would have missed the 2026-08-15 crash entirely.

**G3 — `--livetest` ran at exactly one resolution, 480×270. ✅PARTLY CLOSED 2026-08-16.**

⚠**The original wording here was imprecise and the imprecision mattered.** It implied the small
window left the frame-cost controller untested. Measured, it does not: at 480×270 the deep holds
e63/e72/e82/e94 already settle at 461×259 / 298×167 / 230×129 / 163×91 — budget-bound — and at
1920×1080 they settle at *exactly the same sizes*, because the budget is denominated in steps and
the resolution that fits it is window-independent.

What the large window genuinely adds, and why it was still worth blessing:

- **e55 and e61 become budget-bound** (653×367, 516×290) where the small window capped them first,
  so two further holds exercise the controller rather than the window.
- **The shallow checkpoints render at true full resolution**, which is the only route to the
  **tiled settle at full size** — and both 2026 field device losses were on that path (the 08-16
  one explicitly `settled=true`, `tile=true`, 2247×1485). A 480×270 window never needs tiling, so
  no gate had ever run it.

`benchmarks/livetest-grand-tour-1920x1080.json` is blessed (24/24) and verified reproducible
(0 drifted on a re-run), wired in as `tour/grand/gauntlet-1080p` and `tour/grand/full-1080p`.

Still open: the no-`TIMESTAMP_QUERY` path (1445×1134 + `FRACTADYNE_NO_TIMESTAMPS=1`), the
beta.102/103 pixellation size (1920×1102), and a window at the 2247×1485 class the 08-16 loss
actually used.

**G4 — Goldens stop at 1e6.** There is no exact-image check anywhere in e6–e1008. Deep correctness
rests entirely on the F3 corpus (a different oracle, offline only) and on livetest black-fraction
drift (a summary statistic, not an image).

**G5 — Depth ladder has holes exactly where regimes change.** Corpus goes e24 → e30 → e43 → e124.
Nothing at **e13.7** (the f32 cliff / direct→perturbation switch), nothing at **e28**, nothing
between e30 and e43, nothing at e95 (the self-reported wall) or e100 (the beta.101 loss).

**G6 — Only the forward direction is tested.** `render.rs` names floatexp→df32 as "the dangerous
one" — a budget earned where BLA skips most work, carried into a mode where it does not. No harness
zooms out through a crossover.

**G7 — No unified failure artifact and no per-rung deadline.** Each harness reports differently, and
a hung rung looks like a slow one. The banked lesson is explicit: *a soak that greps only crashes
passes a hung app.*

---

## 2. Design principles

**P1 — The runner is a supervisor; every rung is a child process.** Forced, not preferred: the
failures worth testing kill the process (device loss → `exit(2)` + relaunch) or wedge it (present
wedge). An in-process runner dies at the first interesting result. A supervisor also buys per-rung
isolation, timeouts, and resume.

**P2 — A timeout is a FAILURE with a full report.** Never a skip, never a pass. Every rung carries
an explicit deadline sized from its blessed duration.

**P3 — Difficulty is ordered and the suite reports the highest rung passed.** One headline per lane:
`live: passed to e94, FAILED at e100`. This matches how the project already talks about its limits.

**P3a — A failure does not end the run. ⭐** The default is to keep going and collect *every*
independent failure in one pass, because these runs cost 40+ minutes and finding one bug per run is
too slow a loop. Stopping at the first failure is available (`--stop-at-first`) but is not the
default.

This needs dependency awareness, or "continue" degrades into noise — if the e28 crossover loses the
device, every deeper live rung will fail for the same reason and bury the one novel failure. So each
rung may declare `requires`:

- a rung whose prerequisite **failed** is reported `blocked-by <rung-id>` — recorded explicitly,
  never silently skipped, and never counted as a pass;
- a rung with no failed prerequisite **runs**, even if something earlier in the ladder failed.

Depth rungs chain (`e55` requires `e28-hold-blaLive`), so one crossover failure blocks the depth
spine and reports once — while the resolution, iteration-regime, Julia and offline families keep
running, because none of them depends on it. That is what makes a single run able to surface several
genuinely distinct defects.

`--stop-at-first` remains useful when bisecting a known failure, where everything after the first
one is noise.

**P4 — Every rung is independently reachable by ID.** No "run the first 40 to get to the 41st".

**P5 — Hermetic, and WIPED per run. ⭐Proven necessary 2026-08-15.** Each rung gets a throwaway
`FRACTADYNE_CONFIG_DIR` that is *deleted and recreated* before it starts.

Not a precaution — a measured defect. `--livetest` boots into whatever view the session holds, and
the dev session held a 1e102 view (2.4M-sample reference, ~1 fps). Under it the tour **never started
at all**: 691 s and 1063 frames with no checkpoint header, the run abandoned. The same binary with a
fresh config dir reached the home view (`mag 2^-0.2`) and began the tour in **0.85 s**.

Two consequences, both uncomfortable:

- Every livetest run made today, *and the blessed baseline itself*, was recorded under whatever view
  happened to be saved at the time. The gate has never been reproducible. The 2267 s / 3478 s
  spread first attributed to CPU contention was substantially this.
- A per-rung directory that merely *persists* is only half-hermetic: the app writes a session on
  exit, so run N+1 of a rung starts where run N stopped. It must be wiped, not reused.

The baseline should be re-blessed under a wiped config dir before it is trusted again.

**P6 — Compare decoded RGB, never file hashes.** `--render` embeds metadata; four identical renders
produce four different sha256s.

**P7 — Rungs are stock-only.** `--set` overrides fail the suite, exactly as they fail `--selftest`.
The overrides exist to *bisect* a failure afterwards, not to produce a green run.

**P8 — One rung at a time, on an otherwise idle machine. ⭐Learned the hard way, 2026-08-15.** A
`--livetest` run made concurrently with `cargo check`/`cargo test` spent **1400 s** in an ultra-dive
hold that the same build had cleared in **~200 s** minutes earlier, sitting on a byte-identical
frame (`rebase` and `bla_skip` unchanged across thousands of frames) and looking exactly like a
hang. It was CPU contention: checkpoint resolution is derived from the measured frame-cost budget,
so a loaded machine changes the *result*, not merely the duration.

Consequences for the suite: **no `--jobs`, ever** — parallel rungs would make every timing-derived
checkpoint meaningless. The supervisor should sample machine load before starting and record it in
the summary, and a rung that exceeds its blessed duration by a wide margin should say
`fail-deadline (machine may have been loaded — blessed 200s, actual 1400s)` rather than assert a
product bug. A harness that manufactures failures is worse than one that misses them.

---

## 3. Architecture

```
fractadyne --torture [SELECTOR ...] [--stop-at-first] [--list] [--torture-out DIR]
```

Supervisor responsibilities: resolve selectors → ordered rung list; spawn one child per rung with a
deadline; classify the outcome; write artifacts; propagate `blocked-by` to dependents of a failed
rung while letting independent families continue (P3a); print the ladder summary.

**Outcome classification** (from exit code + log + artifacts):

| Outcome | Detected by |
|---|---|
| `pass` | exit 0 and all assertions met |
| `fail-assert` | exit 0 but drift/Δ/verdict outside tolerance |
| `fail-deadline` | wall clock exceeded the rung deadline → child killed |
| `fail-device-lost` | `exit(2)` + `[fd-wgpu] DEVICE LOST` in the log |
| `fail-crash` | non-zero exit, panic, or `0xc0000409`/`0xc0000005` |
| `fail-hang-watchdog` | `[fd-watch] possible hang` present |
| `blocked` | a declared prerequisite failed — reported with the blocking rung ID, never a pass |
| `skip-unsupported` | capability probe says the rung is inapplicable (recorded, never silent) |

### 3.1 Rung IDs

```
<lane>/<family>/<rung>
live/crossover/e28-into-fe-noBLA
live/depth/e72-hold
live/resolution/1445x1134-no-timestamps
tour/grand/chapter-orbit
offline/render/e100-explicit-over-escape
offline/resume/one-byte-short
```

Prefix matching: `--torture live` (lane), `--torture live/crossover` (family), or a full ID.
`--torture --list` prints the ladder with blessed durations so a targeted re-run is one copy-paste.

---

## 4. The difficulty ladder

Difficulty is a product of independent axes. The ladder is a *path* through that space, ordered so
that the first failure localises the cause.

### Axis A — depth, with regime boundaries named

| Rung | Why this exact depth |
|---|---|
| e0 | framing/palette anchor |
| e6 | double precision exhausted (classic KF benchmark) |
| **e13.7** | **f32 cliff / direct→perturbation switch** — the df32 fold makes this a real boundary |
| e24 | deep df32, still mode 0 |
| **e28** | **`PERT_FE_THRESHOLD`, mode 0→2.** The 2:58 class; the 2026-08-15 loss. **G1** |
| e30, e43 | early floatexp, corpus-backed |
| e55, e61, e63 | grand-tour holds; the reference-extension refusal band lives here |
| **e72, e82, e94** | the motion-cap truncation family (beta.98) |
| **e95** | the self-reported depth wall — must fail *honestly*, with the status message |
| **e100** | beta.101 device loss (explicit count > reference escape length) |
| e200, e500, e1008 | corpus-backed extreme depth |
| **e2100** | `LIVE_REF_CAP` freeze |

### Axis B — reference / BLA state ⭐ the one nothing currently tests

| State | How to construct | Motivates |
|---|---|---|
| BLA-live, long ref | current tour holds | baseline |
| **short escaped ref (`orbit_len` ~600, `bla_skip=0`)** | centre just outside a minibrot so the reference escapes early relative to the pixel ask — the documented three-spar `orbit_len=626` | **every device loss** |
| partial reference | ask above `LIVE_REF_CAP` | freeze guard v2 |
| collapse-on-install | wheel-jump that shortens the ref >1.5× | `install_collapse` derate |
| rebase grind | high rebase count, low skip | crash-1786506241 |
| glitch-correction storm | the documented >1h pathology point | correction pathology |

### Axis C — iteration regime

auto-iter · explicit small · **explicit 10M** · **explicit above the reference's escape length**
(beta.101) · **the 248k–256k refusal band** · boost-climbing vs pinned.

### Axis D — resolution

320×200 · 1280×720 · **1445×1134** (beta.47 repro) · **1920×1102** (pixellation repro) · 1920×1080 ·
3840×2160 · **resize-during-settle** · odd/prime widths.

### Axis E — motion

settled hold · slow glide · **fast glide across a crossover** · wheel-jump · pan-at-depth ·
**zoom-out through a crossover** (G6) · dual-Julia cursor slide at depth.

### Axis F — breadth

10 fractals · dual Julia · Julia at the 100× `PERT_JULIA_THRESHOLD` · Multibrot 3/4/5 SA scope ·
Newton/Phoenix (different escape semantics).

### 4.1 Composed ladder (first cut)

**Lane `live`** — escalating, stop at first failure:

```
live/warmup/e0-home                        A=e0
live/warmup/e6-seahorse
live/cliff/e13.7-direct-to-pert            A=e13.7           G5
live/crossover/e28-approach                A=e28  E=glide
live/crossover/e28-hold-blaLive            A=e28  B=BLA-live      ⭐G1
live/crossover/e28-hold-noBLA              A=e28  B=short-escaped ⭐G1+G2  ← the 2026-08-15 crash
live/crossover/e28-fast-glide              A=e28  E=fast
live/crossover/e28-zoom-out                A=e28  E=reverse       ⭐G6
live/depth/e55-hold … e63 … e72 … e82 … e94
live/depth/e95-wall-reports-honestly
live/depth/e100-explicit-over-escape       C=explicit>escape      ⭐
live/depth/e200 … e1008
live/depth/e2100-refcap-freeze
live/iter/refusal-band-248k-256k           C
live/resolution/{320x200,1280x720,1445x1134,1920x1102,3840x2160}
live/resolution/1445x1134-no-timestamps    (FRACTADYNE_NO_TIMESTAMPS=1)
live/resolution/resize-during-settle
live/motion/wheel-jump-collapse            B=collapse
live/julia/dual-slide-at-depth
live/julia/pert-threshold-100x
```

**Lane `tour`** — the 9 shipped tours plus purpose-built ones, each `--play` under deadline with
checkpoint drift vs blessed JSON. Adds `tour/grand/*` chapters as individually targetable rungs so a
chapter regression does not require a 38-minute run.

**Lane `offline`** — deterministic, no GPU timing, cheapest to run and the natural CI candidate:

```
offline/corpus/01-home … 20-deep-1.2e1008     (existing 20, now rungs)
offline/render/e28-crossover-{blaLive,noBLA}  ⭐ new, mirrors the live rungs
offline/render/e100-explicit-over-escape      ⭐ beta.101
offline/render/glitch-storm                   the >1h pathology, with a deadline
offline/res/{720p,1080p,4k}
offline/resume/one-byte-short                 the IEND-CRC truncation
offline/resume/interrupted-midframe
offline/writer/{png,exr}-metadata-roundtrip
offline/tour/render-tour-determinism          two runs, decoded-RGB identical
```

---

## 5. New pathological points to author

These do not exist yet and are the substance of the work. Each needs coordinates derived and then
blessed.

| ID | Construction | Guards against |
|---|---|---|
| **PP1 e28-noBLA** | centre where the reference escapes in ~600 iterations at a pixel ask of ~10⁵, sitting at 1e28 | the 2026-08-15 device loss |
| **PP2 e28-blaLive** | same depth, long non-escaping reference | isolates B from A |
| **PP3 f32-cliff** | 1e13.7 straddle, direct and perturbation both valid | the df32 fold cliff |
| **PP4 explicit-over-escape** | e100 view, auto-iter OFF, count above the reference's escape length | beta.101 |
| **PP5 refusal-band** | ask 248k–256k at a depth needing extension | the beta.42 hang |
| **PP6 collapse-jump** | wheel jump shortening the ref >1.5× | `install_collapse` |
| **PP7 latency-floor minibrot** | interior of a deep minibrot, tiny frame | "steps ∝ time is FALSE" |
| **PP8 glitch-storm** | the documented pathology centre | correction >1h |
| **PP9 e95-wall** | just past the honest limit | must degrade with a message, not a black screen |
| **PP10 zoom-out-crossover** | start e30, zoom out through e28 | G6 |
| **PP11 julia-100x** | dual Julia at `PERT_JULIA_THRESHOLD` | beta.88 |
| **PP12 deep-golden** | one exact-image golden at e28 and one at e72 | **G4** |

PP12 is worth calling out: adding even two deep goldens closes the widest correctness gap in the
project, and the cross-GPU tolerance machinery (beta.93) already exists to make them portable.

---

## 6. Failure artifact

`validation/torture/<rung-id>-<utc>.txt`, deliberately the same shape as a crash report:

```
rung      : live/crossover/e28-hold-noBLA
outcome   : fail-device-lost
repro     : fractadyne --torture live/crossover/e28-hold-noBLA
duration  : 41.2s (deadline 120s)
version   : 0.2.40-beta.NNN (build N)
adapter   : <name> · <backend> · TIMESTAMP_QUERY=<bool>
tunables  : stock
expected  : verdict ok, black ≤ 0.31, res 480x270
actual    : device lost at frame 4303
manifest  : <the live manifest line>
log tail  : <N lines>
artifacts : actual.png, expected.png, diff.png (decoded-RGB Δ: max/mean)
```

Plus `validation/torture/summary.json` — machine-readable, one record per rung, so trends across
runs and across machines are diffable. On the Radeon box this is what gets zipped and shared.

---

## 7. Implementation increments

1. **Supervisor + ladder + report**, rungs composed only from existing flags (~25 rungs). No engine
   changes. Delivers targeting, deadlines, artifacts, highest-rung-passed.
2. **New scenario flags** for the regimes nothing can reach — above all forced short-escaped
   reference (B). This touches the engine and needs its own gating.
3. **Author PP1–PP12**, bless them, add the two deep goldens.
4. **CI/offline lane** wired into `ci.yml` (offline is deterministic and headless-safe).

---

## 8. Open questions

- Supervisor in Rust (`--torture`, re-execs itself per rung) or a script driving the CLI? Rust keeps
  the ladder next to the code that defines the regimes and makes `--list` trivial; a script is easier
  to run against an *installed* build. Leaning Rust with the ladder as committed data.
- Blessing policy for the deep goldens: which card is authoritative, and does `BLESSED-GPU.txt`
  generalise to per-rung tolerance?
- Does the full suite have a "quick" tier (< 5 min, offline only) for pre-commit, distinct from the
  overnight soak?
- How do rungs that *intentionally* lose the device get scored — expected-failure, or excluded from
  the default run and only present under `--torture --include-lethal`?
