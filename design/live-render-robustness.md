# Live-render robustness — architecture and instrumentation

Status: **analysis + strategy, nothing implemented.** Written 2026-09-22 against `main` =
`4e86e0e`, workspace version v0.2.41-beta.112. That is five commits past `bb27d5a` — the
budget-blind instrumentation merge that shipped as beta.112 — and all five are bench-kit / Imagina
palette work that does not touch the live path. Every line number below was taken from `4e86e0e`.

The brief this answers: *we want to develop a more robust architecture and add instrumentation to
help diagnose and validate issues and fixes.* Sections 5 and 6 are the answer; sections 2 and 3 are
the evidence that the answer is the right shape; sections 7 to 10 are how it gets built and how we
find out if it worked.

Evidence base: 201 merged incidents across the release notes and the post-mortem notes, 59 findings
each upheld by two independent adversarial reads of the tree, and eleven subsystem maps. Every code
claim below carries a `path:line` or a symbol that exists on `main` today. Where the only evidence
is a memory note rather than code, the sentence says **not verified**.

One sentence of thesis, so the rest can be read against it:

> The live path does not fail because its controllers are wrong. It fails because no single
> component owns a price, a frame's identity, or the record of what was decided — so a number
> learned by one loop is consumed by four others as a denominator, a gate and an actuator switch,
> and when it goes wrong nothing on disk can say so.

---

## 1. Scope

### In scope

Everything that puts a frame on the screen while the view is, or may be, moving — and the
controllers behind it.

| path | entry points |
|---|---|
| Manual zoom | Space glide (`central.rs:1058-1087`, `central.rs:98-156`), wheel `zoom_at` with no per-frame bound (`central.rs:1032-1056`, `main.rs:8375-8380`), Shift box and right-drag `zoom_to_rect` (`central.rs:972-1017`, `1089-1110`) |
| Pan | left-drag `pan_pixels` 1:1 (`central.rs:1026-1030`, `main.rs:8371-8373`), minimap drag/wheel (`central.rs:726-738`) |
| Click-to-zoom | `click_zoom_at` / `_view` 2–100× (`main.rs:7652-7670`, `main.rs:8326-8352`) |
| Keyboard | Home glide `home_lerp` (`main.rs:7889-7953`), Backspace/Ctrl+Z `apply_snapshot`, Esc, `M`, `A` (`main.rs:14237-14300`) |
| Tour playback | `advance_playback_core` → `set_center_log2mag` per tick (`scripting.rs:3631-3700`, `3785-3800`) |
| Autopilot | `autopilot_step` (`autopilot.rs:410-620`) in Speed, Quality and Misiurewicz-target modes; `autopilot_zoom` / `autopilot_pan` (`autopilot.rs:620-650`) |
| Dual view | the same path at `view_id 1`, own `ViewResources`, own `settle_t[1]`, shared `tile_turn` (`central.rs:84-260`, `main.rs:8178`) |

and, for all of them, the controllers: frame cost (`render.rs:4615-5300`, `9015-9700`), the
iteration ask (`render.rs:4051-4614`), the reference lifecycle (`render.rs:1341-1700`, `621-1003`),
motion presentation (`render.rs:5301-5427`, `6180-6440`, gpu `lib.rs:1921-2540`), colour
normalisation on the live path, and the harnesses that grade any of it.

### Out of scope

- **Content correctness past 1.6e15×** (U37). Exact-image verification stops there, and past it
  every check either shares the live path's ask or switches off the machinery under test — the F3
  corpus pins `series_approx = false` and `glitch_correct = false`. Closing that means authoring
  deep goldens and a corrector gate; it is a multi-week project with a different shape and deserves
  its own document.
- **The offline `--render` / export path**, except where it shares the live admission gate
  (§5.1) and except for the concurrent-export aggregate case (MEM-37), which is named as a
  regime with no controlled reproduction and is deliberately not scheduled.
- **Issue #2** (monitor-drag growth, upstream) and the renderer's arithmetic, the shader and the
  bignum layer. Byte-identity on the corpus is a *constraint* on every commit here, not a subject.
- **New UI**, beyond the single quarantine notice §5.7 requires.
- **Closing issue #1.** Nothing in this document is a reproduction. The claim is that it makes one
  obtainable and the next field capture decisive.

---

## 2. What has gone wrong

### 2.1 By class

201 incidents, each filed under exactly one class. "Found by" is derived from the catalogue's
detection column per class, not from a per-incident field, so read it as the dominant route rather
than a count.

| class | n | open | found mostly by | example |
|---|---|---|---|---|
| cost-model-blind | 15 | **yes** | field crash reports, `--zoomtest` | CL-1 (2026-09-21, RX 6800 XT lost the device at 1.76e6×, budget frozen at 1.515e11 through 20 slow frames) |
| measurement never arrives ⇒ a bootstrap constant binds | 7 | **yes** | measurement on affected hardware | CL-125 (a 1445×1134 panel rendered 504×396 permanently, since beta.40) |
| stale or misattributed measurement | 7 | no | user report, then trace | CL-126, CL-85–88 (readbacks land 2–3 frames late) |
| unbounded or oversized dispatch | 9 | no (CL-165 residual) | user report, `--motiontest` | CL-95 (Home from depth reset the graphics card) |
| guard scope / capability-gating hole | 9 | no | user report | CL-83 (every view past 1e308× blank, live four days) |
| coarse or unsatisfiable identity key | 6 | mechanism closed; CL-29 **open-and-unreproduced** | user report | CL-29 (a translucent copy of another location after a small pan) |
| normalization source & wrong statistic | 13 | no | user report | CL-15 (every colour swelling and shrinking ~2.5×/s) |
| iteration starvation / misjudged interior | 11 | **yes** | screen recording, field log | CL-13 (36% of a dive's length blank), MEM-61 (motion `[0,step]` false, **not fixed**) |
| reference lifecycle | 16 | no; MEM-71 standing | validation script, trace | CL-102 (100% black at 2e82× after a pan) |
| presentation ≠ render | 10 | **yes** | screen recording | CL-3 (one frame in three a flat colour during a dive) |
| UI reflow feeding back into the render | 3 | no | user report | CL-17 / CL-69 / CL-115 — the same loop three times |
| main-thread blocking & unbounded aggregate | 5 | **yes** | user report, crash reconstruction | MEM-38 (~1 fps grind, no device loss, no watchdog line), MEM-37 |
| numeric representation limit | 8 | mechanism closed; CL-108 **open** (workaround) | user report, code reading | CL-108 (df32 running at f32 for the product's whole history) |
| perturbation & formula correctness | 5 | no | user-visible | CL-141 (a deep exterior as distorted tiling) |
| steering & search logic | 9 | no | user's own dive + screen recording | CL-10 (aim slid off the edge at 4× speed) |
| verification-harness defect / gate blind spot | 16 | **yes** | self-review | CL-52 (an export-aspect check passed against the defect it was written for, twice over) |
| retracted or refuted diagnosis | 6 | n/a | measurement | CL-2 (atomic contention on the escaped counter: Δ=0 ms, pixel-identical) |
| driver / GPU fast path / upstream | 5 | **yes** | new harness's first run | CL-77 then CL-39 (a ~200× cliff at odd height, then at odd width) |
| actuator & retreat blind spots | 4 | **yes** | field log | MEM-38 (the guard logged on every pass and backed nothing out) |
| composition defect | 3 | no | `--zoomtest` | CL-28 (`model.min(measured)` latched 9% of the pixels for a whole dive) |
| path divergence | 3 | no | the user walking the feature list | MEM-63, MEM-64 |
| crash & report invisibility / recovery | 5 | CL-168 open | field | CL-168 (one `0xc0000005` with no report) |
| silent default / unread value | 2 | no | user report | CL-124 (10,000,000 iterations rendered as ~82,000) |
| heuristic tuned in one direction or at one depth | 3 | no | later audit | MEM-73 (a derate calibrated while a data bug was live) |
| open / unexplained | 2 | **yes** | user report, harness | CL-163 (issue #3, dual-view Julia), CL-165 |
| file, format & session contract | 4 | no | a shipped-file sweep | CL-50 (coordinates silently dropped for 52 releases) |
| colouring quality & mapping | 4 | CL-167 = ceiling | user report | CL-167 (speckle with one clean central diamond) |
| miscellaneous | 11 | no | mixed | — |

**Nine** classes remain open — the nine bolded **yes** above. Of those nine, **five have no check on
`main` that could go red**: cost-model-blind, iteration starvation, main-thread blocking /
aggregate, actuator blind spots, and open/unexplained; the other four (measurement never arrives,
presentation ≠ render, verification-harness defect, driver / GPU fast path) are covered only
partially, by a model or in one regime. The count is nine, not the eight that
`findings-verifiability.md`'s summary paragraph states: that paragraph miscounts against its own
table, where "five with no red check" plus four "partial" rows is nine. Recorded here so the next
reader does not re-derive it.

### 2.2 The observations that matter

**Who found them.** Roughly two thirds of the 201 rows were found by a user report or by the
maintainer's own dogfooding, not by a gate. The gates that did find things are almost all live-path
harnesses built *after* an incident: `--livetest` (CL-134, CL-112), `--motiontest` (CL-95, CL-165),
`--zoomtest` (CL-23, CL-28), `--uitest` (CL-106, CL-161, CL-166, CL-167), `--gputest` (CL-108),
`--divetest` (CL-33), `--deviceloss-repro` (CL-38, CL-39). CL-134's own note explains the shape of
the whole catalogue: *goldens are offline export renders, so none of them can see a live-path bug.*

**Screen recordings are a distinct instrument.** CL-3/4/5/6, CL-7, CL-11 and CL-13 were all
diagnosed from a recording lined up against the log, which is why `--show-timestamp` exists. A
per-eval replay at 0.35 s cannot see a 0.4 s transient.

**How long defects lived.** CL-50, coordinates silently dropped, 52 releases. CL-108, df32 running
at f32, the entire history of the product, found on the first run of a new harness. CL-125, a panel
permanently at 504×396, since beta.40. CL-83, every view past 1e308× blank, four days across a
release. Two of these were invisible precisely because nothing was asking.

**What recurred.** Six shapes, across years:

1. Iteration starvation read as "black view" — CL-151 (0.1.0) → CL-13 (beta.109), eleven months
   apart, same subsystem, because the earlier fix addressed the cap and not the measurement that
   sizes it.
2. UI reflow → canvas resize → render restart — CL-69, CL-115, CL-17. Each fixed for its specific
   element; the structural rule arrived only after the third.
3. Buffer-dimension parity costing ~200× — CL-77 (odd height) then CL-39 (odd width). The width
   twin survived two months because nobody swept the other axis.
4. Normalisation from unrepresentative data — nine incidents, beta.8 → beta.109.
5. A measurement paired with the wrong subject — CL-126, CL-85, CL-87, CL-88, CL-59, and still
   unresolved in CL-1.
6. A gate that cannot go red — CL-52 (twice in one check), CL-67, CL-81, CL-101, CL-150.

**What one fix broke elsewhere.** The record is unusually explicit, and this is the single strongest
argument for the migration discipline in §5.10:

- Chunked refinement, introduced for device-loss safety, immediately broke colour: CL-90 and CL-88
  exist only because the palette window now saw pieces instead of frames, and CL-89 (black screen,
  no spinner) because the spinner's idle-gap detector was tuned for the old repaint cadence.
- CL-114: "Prefer detail while zooming" stage A froze every moving frame, disabling the refresh
  cadence and making a long dive worse than the toggle off.
- CL-113 → CL-112: the beta.79 explicit-cost regime collapsed the grand tour's deep holds to 16×16
  one release later; the same-day attempt to fix that rendered spar holds 100% black and was
  reverted.
- CL-3: the motion-resolution ladder — itself a smoothness fix — is what made the
  snapshot-after-clear bug fire. The snapshot was only ever black when the resolution had just
  changed.
- CL-102 supersedes CL-104: beta.97's "skip the futile BLA rebuild" was treating a symptom of a
  truncation fixed in beta.98, and it cost the descent its up-to-date approximation tables.
- CL-141: "BLA subsumes SA" was reasoning rather than measurement.

---

## 3. Why

The 59 verified findings group into eight themes. Severity is the verified severity; the failure
scenario is one line each; the finding ids are the anchor into Appendix B.

### 3.1 Controller blindness and one-sidedness — critical, UNRESOLVED

`fe_budget[v]` is four things at once (U10): the quantity being learned; the denominator of the rule
deciding which readings may update it (`PRICE_REPRESENTATIVE_FRAC` 0.7, `render.rs:9339-9341`); the
denominator of the motion-jam backlog threshold, with the same 0.7 duplicated as a bare literal at
`render.rs:9409` rather than read from the constant at `render.rs:9299`; and the gate deciding
whether a frame is chunked at all (`chunk_over`, `render.rs:6932-6978`). All four fail together and
in the same direction. The duplication is not an oversight — `motion_jam_counts`' own doc
(`render.rs:9404-9407`, read this pass) says the threshold is *"deliberately the SAME
representativeness threshold `budget_step` uses to accept a reading, so 'counts as backlog' and
'would move the budget' are one definition"*. The intent is one definition; the code has two copies
of it, and that is the whole shape of this theme.

The failure is one-sided. When the budget is too small the frame is shrunk or chunked, which is
safe. When it is too large, `chunk_over` says "this fits in one dispatch" — and because the
resolution shrink (`&& !chunk_over`, `render.rs:7012`) and the settle tiles (`&& !chunk_over`,
`render.rs:7120`) are both suppressed by it, all three bounds stand down in the same instant
(U1, U22). The supersample caps are not an independent bound either: `max_ss` / `max_ss_tdr`
(`render.rs:7023-7035`) are denominated in the same budget and floor at 1, and the in-code comment
says so outright — it "floors at ss=1 and could never bound the ss=1 frame that lost the device
anyway".

The wall-clock path that exists to catch this is defeated three separate ways:

- any arriving reading of any age clears the probe before it can price (U2: `main.rs:14206-14214`
  against `render.rs:8005-8023`);
- five non-dispatching frames set `present_throttle >= 5`, which freezes wall pricing
  (`main.rs:13795-13796`), blocks the starvation latch (`main.rs:14181`) and vetoes the lethal shed
  through `!throttled` (`render.rs:9263-9265`) — and the price-serialized hold *manufactures* exactly
  those frames (U11, U12). `present_throttle_step`'s own doc states the premise being violated:
  "a genuinely busy queue cannot produce them" (`render.rs:9561-9562`, read this pass);
- the 1000 ms wgpu `FRAME_TIMEOUT_MS` right-censors the interval, so a 3 s overrun and a 1.05 s
  overrun price identically.

Alongside these: `mode_rate` is a session-long running minimum fed by every kind of dispatch,
feeding three floors, with no recovery (U23); the motion pacer's wall LIFT is unbounded and
kind-blind (U40); the steering loop's cadence is silently rescaled by the cost loop's failures
(U53); the budget-blind tripwire is reset by a budget moving in the *wrong* direction and reports
one view's state for both panels (U15).

*Failure scenario.* The budget converges at a benign per-step rate, the location turns 10–70× hotter
per nominal step, and the frame is now 4× the real budget while reading 0.27× the nominal one.
Chunking, tiling and the shrink are all off at the same instant, by the same number. One submission,
device lost.

**UNRESOLVED.** U9 lists five candidate mechanisms for the 2026-09-21 stall and the code that would
discriminate them: (a) timestamps never armed; (b) armed on the wrong dispatch; (c) the bracket does
not cover the work; (d) the cost is outside the timed pass; (e) readings arrived and could not move
the budget. Beta.112's 24-entry ring can separate three of them. Mechanism (d) is unmeasurable by
construction — `fs_resolve` carries `timestamp_writes: None` (gpu `lib.rs:2367`, read this pass), as
do the seed, `hold_copy`, `accum_color` and `accum_fold` passes. No reaction has shipped, and none
should ship on a mechanism argument.

### 3.2 The picture and the provenance gap — critical to medium

The content guard shipped in beta.110 is wired to two consumers (pin adoption, hold snapshot) out of
five presenters. The other three — the reuse-hold reprojection, the settled compose gate, the
cold-start live fallback — are outside it entirely (U20, U42, U45). On the two it *is* wired to, it
can be short-circuited by a completeness test: `live_verified()` short-circuits on `live_complete`
with `||` before it ever looks at an escaped count, so the predicate that actually runs for the
settled path is "did this render finish its ask?" — and an all-interior frame is a verified picture
(U19).

The signal that would answer the other half of the question — *and is it a picture of this view?* —
already exists. The GPU stamps every texture with `view_stamp = pos_sig` and reports `content_stamp`
/ `content_foreign`, and that signal has exactly two consumers tree-wide: `drive_accumulation`
(`main.rs:13362-13364`) and `seed_allowed` (gpu `lib.rs:1079`). So the present path verifies content
but not provenance, and the accumulation path verifies provenance but not content (U42) — the
MEM-77 shape, a guard that passes noise and passes the wrong view, reproduced inside the app.
`ContentTrack::hold_verified` is written on three paths and read only inside three `format!` calls
(`render.rs:4322`, `5878`, `7730`) (U59); the GPU display gate is `self.display_hold &&
view.hold.is_some()` (gpu `lib.rs:2077`) with no content and no provenance term.

Two more in the same family: a held or reprojected frame is re-coloured every frame under the
*current* normalisation map, so under the log map every pixel below `norm_lo` collapses to one
colour (U43); and the converged SSAA average is presented under a colour signature that omits the
window actually mapped (U58).

*Failure scenario.* A hold snapshot is taken from a texture that was cleared for a resize, and
nothing anywhere asks whether it contains pixels. This is CL-3 exactly, and the beta.110 fix closed
the ordering, not the question.

### 3.3 Walk-signature ownership and frame identity — high

Resolution is part of the walk signature, so changing it discards the iteration cursor and that
frame restarts however far it could have walked. Three independent writers set it — `res_scale`,
`motion_res` and `visible_res` — and `build_params` reaches them through staged mutation of
`&mut self` over ~120 `Perf` fields across five stages (`render.rs:5890-8004`, plus `bp_frame_budget`
4051, `bp_chunk_tiling` 4615, `bp_present_gate` 5301, `bp_finish_params` 5428).

The costs are visible throughout: `fe_dispatch_frame` is stamped at four sites (`render.rs:5174`,
`5291`, `5629`, `7132`) and actively rewound eight frames at a fifth (`render.rs:5843-5844`), which
is the mechanism behind the present-throttle misclassification (U11); the moving frame's resolution
is decided by the palette normalisation window (U24, `render.rs:6412`), which is a guess reading a
value downstream of what it sizes; view 1's moving resolution is a view-0 quantity that view 1 can
never move (U56, `main.rs:1547`); and `hold_scale_floor(jumped)` itself is a clean pure function taking the
flag (`render.rs:8863`, whose doc says "One definition, three callers"), but all three callers infer
the flag the same wrong way — `hold_scale_floor(self.pointer.zoom_vel.abs() < 1.0e-9)` at
`render.rs:7579`, `7745` and `7776` — so every autopilot dive and every wheel zoom takes the JUMP
branch (`autopilot_zoom`, `autopilot.rs:632`, never writes `zoom_vel`), while `--motiontest` sets
`zoom_vel = 2.5` (`motiontest.rs:191`) and exercises the other one (U39). Cross-frame numeric state
is reset on the wrong event or never
(U51): a session-lifetime running minimum, a boost that outlives its view, and an identity key
written but never read.

*Failure scenario.* A motion rung changes, the walk sig changes, the cursor is discarded, the frame
is blank — and every per-pass metric improves while the screen goes from 8% blank to 36%.

### 3.4 Reference fitness — critical

`try_reuse_reference` applies exactly one test — build precision ≥ view precision AND per-axis drift
≤ `REUSE_MAX_DRIFT` (`render.rs:621-634`, `944-946`) — and `needs_quality`'s length term is gated on
`partial` (`render.rs:7341-7344`), so a complete (escaped) orbit is never compared with the ask
again (U6). At 4,627 iterations against a 655-sample orbit that is about seven wraps per pixel and
25–33 M rebases per frame with `bla_skip = 0` (the 2026-09-21 field manifest — **not verified in
code**, it is a memory note plus a crash artefact), and nothing in the ask, the nominal step price
or `chunk_over` carries a rebase term.

Three more, all structural: the reuse gate's precision comparand is the *view* precision while a
fresh build runs at `inp.precision + REF_PREC_HEADROOM` (+128 bits, `render.rs:764`,
`tunables.rs:796`), so a reused orbit can sit up to 128 bits below what the picker's cliff rescue
would use, with no rescue on the reuse path at all (U13). The disk cache's `find` ranks candidates
by `orbit_len` alone and never reads the header fields — `partial`, `iter`, `req_prec` — that it
already stores (U25, `refcache_persist.rs:249-283`, `orbit_blob.rs:25, 78`). The series
approximation has no validity re-check while its twin the BLA has one every frame
(U41, `render.rs:7870`); in mode 0 nothing at all watches reference staleness. And `install_collapse`
(`render.rs:9288-9290`) is transition-only, so a reference unfit from its first install, or an ask
that grows with no install, never derates (U50).

`design/reference-lifecycle.md:250-253` defers the incumbent/challenger fitness rule "until a real
pinned-unfit case is observed". That condition has been met.

*Failure scenario.* A reference escapes short, the ask grows past it by a factor of seven, every
pixel rebases repeatedly, the nominal step count does not change at all, and the frame is 10–70×
more expensive than the number the controller is steering on.

### 3.5 The iteration ask versus the reference — high

The iteration ask has no always-on per-frame record, and the two always-on artefacts that describe
it are stamped at different points and disagree in the one open field incident: the LIVE manifest
reads `iter=4627` while the crash view `.fdn` reads `max_iter=25000 auto_iter=1`, same crash, same
second (U28). The only directly measured cost multiplier in the app — `CTR_REBASE` and
`CTR_BLA_SKIP`, which come back every readback into `work_sink` (`render.rs:5685`) — is consumed at
exactly three print-only sites (`main.rs:13780-13787`, `14743-14744`, `14768`) and steers nothing
(U17). Iteration starvation is invisible to the gates by construction: `--livetest`'s contract is
"the same view at the same iteration budget" (`livetest.rs:15-16`), so a view starved by its ask
matches a starved oracle and passes, and **no harness** reads `capped_frac` or `iter_exhausted`
(U34). The app itself does: `Perf::view_resolved` branches on both (`render.rs:3773`, `3779`) and
`limit_status` turns them into the user-facing "iter capped" message (`ui/menus.rs:1404`, `1409`),
red-gated by `limit_status_matches_measured_regimes` (`ui/menus.rs:1819`). Both read this pass. That
is good news for §7.8: the fields exist, are maintained, and already have one red-capable test, so
putting them into a harness verdict is cheaper than building the measurement. With auto-iterations
on, the autopilot cannot even *name* starvation, because both witnesses in `dead_end_message`
require auto off (U27).

MEM-61 is open: a motion chunk's `[0, step]` contract is false wherever the view's first escape
exceeds `step`, and the user confirmed two separate bugs. **Not verified in code this pass.**

### 3.6 Settings that persist harm — high

Learned controller state (`fe_budget`, `mode_rate`, `motion_res`) is not persisted or logged at
shutdown, so a relaunched process cannot know what the previous one learned. What *is* persisted is
the ask: the session saves `max_iter`, so a relaunch re-grinds, and `crash-view-1789960937-0.fdn`
records `max_iter=25000 auto_iter=1`, so loading the crash view recreates the lethal ask (U18). The
relaunch resets only the view (`main.rs:6184-6190`).

*Failure scenario.* The app dies in a grind; the user reopens it; it walks straight back into the
grind. Observed twice.

### 3.7 Actuator reachability — critical

A guard can be perfectly correct, fire on time, and change nothing.

- A chunk pass is priced `spx * ss^2 * (end - real_lo)` (`render.rs:5165-5174`), a pure function of
  the iteration window. Its real cost is not: every pass clears and stores 3–4 full-size
  `Rgba32Float` attachments and runs a full-frame `fs_resolve` that is invisible to the only cost
  signal the controller has (gpu `lib.rs:2367`). The project has measured this twice and filed it as
  unexplained — the chunk-sweep parity axis, and `1440×1102 = 4.5 ms/pass` against
  `1441×1102 = 948 ms/pass`, quoted verbatim in `make_state_textures` (gpu `lib.rs:1204-1231`). That
  is why the retreat to the 256-iteration floor did not save the device: the floor's nominal price
  is four orders below the fixed term (U4).
- `chunk_ok` / `chunk_fe_ok` are set once, at `frame_idx == 0`, only if `frame.wgpu_render_state()`
  is `Some`, defaulting to `false` (`main.rs:1918-1919`, `14110-14137`). There is no retry, and no
  log line states the verdict — the adapter line logs the capability *inputs* and never whether
  chunking is available (`main.rs:14121-14136`, read this pass) (U1).
- `PIN_COMPLETION_TIMEOUT_US` = 170 ms (`tunables.rs:647`) is a latency timeout used as a
  "the callback machine is not running" escape, so on hardware whose frames legitimately run
  200–1027 ms it opens the pin serialization gate on every single frame (U3, U47).
- The autopilot steering probe calls `render_iter` synchronously inside `autopilot_step`
  (`autopilot.rs:676`, called at `:532`) — one `fs_iterate` pass at `req.max_iter` with no chunking,
  no budget and no TDR sizing, on the UI thread (U38). Read this pass, the probe is *size*-bounded:
  `autopilot.rs:658-664` clamps it to at most 128×128 at `ss = 1` (≤ 16,384 px) and its comment
  records that it deliberately borrows the live view's resident reference rather than building a
  full-appetite one on this thread. At the field ask of 4,627 that is ~7.6e7 nominal steps against a
  budget of 1.5e11, so this dispatch is not the load. What is unbounded is the **aggregate**: nothing
  anywhere bounds in-flight GPU work across the live path, the probe and an export together, and
  nothing accounts for the probe's submission in any budget at all.
- The lethal shed and the always-on in-flight lethal-band line are gated off for the whole of
  unpinned motion (`render.rs:4721`) (U49), and `wall_clock_budget_tick` zeroes the motion-jam
  backlog that only real completions are supposed to retire (`main.rs:13833-13835`) (U46).

### 3.8 Harness blindness — high

This theme is why the others survived.

- No harness can produce the regime that has killed every device. `--torture`'s ladder header says
  so explicitly (`torture.rs:211-217`), and the one instrument that could —
  `FRACTADYNE_BLA_DROP_FRAMES` — is armed only inside the mode-switch block
  (`render.rs:6577-6589`), while the 2026-09-21 loss was at a shallow mode-0 view with no mode
  switch (U7).
- `--autodive`, the only gate claiming to test device loss, builds its regime verdict from
  `perf.last_iterate_ms[0]` (`autopilot.rs:1344-1360`, exit at `:1511-1512`) — the exact instrument
  the field failure proved short. The more successfully a run enters the blind regime, the more
  confidently it reports "DID NOT REACH THE REGIME", and `torture::classify` maps that exit to
  `FailAssert` (`torture.rs:647-663`), the same result as a healthy machine that never got hot
  (U8, U16).
- Nothing automated looks at the pixels that reached the screen during motion. `--motiontest`
  computes `blank_walks_total` and prints it without asserting on it (`motiontest.rs:220-223`);
  `--zoomtest` prints "held frame max" (`zoomtest.rs:648`) and exits 0 (`:697`) (U21, U44).
- On the hardware that actually fails, every image and signature comparison is informational by
  construction: `bench_matrix.rs:732` reads `let pass = cur == base || !same_gpu;` (read this pass),
  and the cross-GPU golden tolerance at `selftest.rs:5969-5975` is documented as never a release
  gate. An RX 6800 XT legitimately reaches meanΔ 19.15 against a 24.0 limit (U30).
- Each live harness mirrors or re-implements the loop it grades. `--livetest` keeps its own copy of
  the controller loop (`livetest.rs:355-395`), whose comment records the cost: a build gate was
  added to the app and missed in the copy for one commit, and the livetest then reported 0 drift on
  a path that did not contain the change at all (`livetest.rs:383-392`). A second mirror is drifted
  right now: `profile.rs:748` declares `SETTLE_DELAY = 0.35` against the app's 0.18
  (`tunables.rs:994`) (U52).
- The live gates dive with auto-iter off, at one location, in one mode (U33) — `--autodive` switches
  to an explicit count at `EXPLICIT_FROM_LOG10 = 6.0` (`autopilot.rs:1259`) and the 2026-09-21 loss
  was at log10 6.25, the first quarter-decade after the harness stops using auto-iter.
- Every gap above already has a written plan; none of the three is built, and the one tool that
  counts unbuilt work reports zero, because `release_checklist.py` contains no `planned:` rows at
  all while `checklist_coverage.py:171-172, 233-235` implements that class fully (U36).
- `harness:<flag>` means only that a flag exists in the argument parser
  (`release_checklist.py:657-661`, read this pass), and the Deep-zoom row — which stands for the
  entire device-loss regime — is enforced by exactly that (U35).

---

## 4. Principles

The rules this strategy obeys. Each is the maintainer's own, each carries the incident that taught
it, and every item in §5 and §7 is checked against the list.

| # | rule | why |
|---|---|---|
| P1 | A price is a duration, never a count. | The same nominal count measured 114 ms at one location and 1147 ms at another on the same GPU (MEM-7, MEM-16). |
| P2 | A model seeds; it never bounds a measurement — but the model is what lets a pin finish. | `model.min(measured)` latched 9% of the pixels for a whole dive (MEM-49); removing the bound produced a 50-octave-stale uniform smear (MEM-50). The optimistic path uses `measured.unwrap_or(model)`; the pessimistic path uses `min(all sources)`. These are different questions and must not share a number. |
| P3 | A guess must not read a value downstream of what it sizes. | The adaptive iteration budget (MEM-12); and U24, live today. |
| P4 | A guard that only logs is not an actuator, and "there was nothing to pull" must be a first-class recorded value. | The 2026-09-18 lethal-band grind logged on every pass and backed nothing out (MEM-38). |
| P5 | A dispatch already in flight cannot be recalled. | Every retreat makes the *next* pass smaller; none saves the frame already running (MEM-36). |
| P6 | A check that cannot go red is not a gate, and a check that cannot tell "did not run" from "ran clean" is worse than none. | CL-52; U8's inverted verdict. |
| P7 | A control that cannot express the variable under test is not a control. | Four A/Bs on chunking returned a clean, confident, meaningless zero (MEM-78). |
| P8 | Measure a new gate's engagement rate on a *passing* run before its threshold ships. | `MOTION_UNPRICED_MAX = 2` engaged 1,323 times per autodive inside healthy pipelining (MEM-30). |
| P9 | Every assertion's red must have been observed. | A gate whose red has never been seen is an assertion, not a gate. |
| P10 | Instrument the quantity the predicate tests, and the value the consumer reads. | The trace printed `phase/px 0.000` for a session because it repeated the predicate's own bug (MEM-45); `norm_range` was measured instead of `norm_shown` and the glide read as a no-op for two build cycles (MEM-44). |
| P11 | Measure the screen; three pairs minimum; never judge a motion fix by a per-pass metric. | The screen went 8% → 36% blank while per-pass metrics improved; single runs vary 8–14% (MEM-60). |
| P12 | My replays are milder than the user's live sessions. | Same seed, same location, their motion budget 6× smaller (MEM-61). A gate that cannot enter the regime is not coverage; a gate that cannot *prove* it entered is not a gate. |
| P13 | A probe whose reading another rule discards never terminates — gate the producer on the consumer's predicate, from one constant. | The climb probe on the default home view (MEM-66). |
| P14 | Byte-identity on the corpus for refactors, and watch the test count. | 151 → 149, still "ok", still committed clean (MEM-69). Never compare renders by file hash. |
| P15 | Never close a bug on a mechanism argument without a reproduction — and deleting a guard on a mechanism argument is the mirror image of it. | The flat-grey closure was disproved in one day (MEM-42). |
| P16 | Record first, reproduce second, react third. | Beta.112 shipped instrumentation and deliberately no reaction, because inferring a controller rule from one field log on hardware the dev box cannot reproduce is how a controller gains a rule nobody can explain (MEM-39). |
| P17 | A feature that can decline must say it declined, and a control that is offered and does nothing is worse than one that is absent. | The dual-view click-zoom checkbox; `M` in dual view (MEM-63). |
| P18 | After fixing a long-standing data bug, audit every heuristic tuned while it was live. | Beta.50's derate was calibrated against frozen-length references (MEM-73). |
| P19 | A judge must not run on the thread it judges. | `--soak`'s liveness window was counted inside the per-frame hook, so a wedged UI thread produced **no verdict at all** (MEM-68). |
| P20 | A timeout, a wedge or a missing verdict line is a FAILURE, never a skip and never silence. | A soak that greps only crashes passes a hung app (MEM-68). This is P6's other half: "did not run" must be as loud as "ran red". |

---

## 5. The architecture

Eight structural changes (§5.1–§5.8). Each states what it replaces, what it creates and deletes by
name, which incident classes it closes and why they cannot recur under it, its place in the
migration ladder, and what is lost if it is abandoned half-way. §5.9 says what is deliberately
rejected and how the findings this document does not act on are disposed of; §5.10 is the migration
discipline the whole section depends on.

The shape of all eight is the same: move a property from *review* to *construction*. A rule that the
compiler or the type system enforces cannot be forgotten in the next refactor, which is what
happened to `--livetest`'s controller copy and to the third, fourth and fifth presenters of the
content guard.

### 5.1 One pricing authority, with an un-bypassable wall backstop

**Replaces.** The current arrangement in which `fe_budget[v]` is simultaneously a learned quantity,
two denominators and an actuator switch, and in which three separate mechanisms can switch the
wall-clock backstop off.

**Creates.** `crates/fractadyne-app/src/render/pricing.rs` — a real `pub(crate) mod pricing;` under
`render/`, the same shape as the existing `render/orbit_blob.rs` declared at `render.rs:131`. It
owns every function in the family, moved verbatim: `budget_step` (`render.rs:9323`),
`budget_after_build_gate` (`:9525`), `probe_would_price` (`:9306`), `budget_base` (`:9273`),
`wall_probe_step` (`:8005`), `measurement_starved` (`:9692`, read this pass), `present_throttle_step`
(`:9552`), `wall_shed_now` (`:9263`), `budget_blind` (`:9438`), `motion_jam_counts` (`:9406`),
`chunk_band_license` / `chunk_band_update_with_lane` / `chunk_band_retreat` (`:9046-9270`), plus the
rate latches `record_mode_rate` (`main.rs:1637`) and `record_motion_rate` (`main.rs:1675`) as methods
on a new `Meter` struct holding the fields they already own.

Inside it, three budgets and two verbs.

```
struct Budgets { fe: [u64; 2], wall: [u64; 2], ceiling: u64 }

fn size(view, kind) -> Steps       // optimistic:  measured.unwrap_or(model)
fn admit(view, plan) -> Admission  // pessimistic: min(fe, wall, ceiling)
```

- `fe_budget[v]` learns exactly as it does today.
- `wall_budget[v]` is a second, independent estimator learned only from `last_dt_ms` over frames
  that dispatched, using the max-over-two-frames rule that is written down *and implemented* at
  `main.rs:13798-13815` — by `wall_probe_step` (`render.rs:8005`), which `wall_clock_budget_tick`
  calls at `main.rs:13818-13824` — but which is reachable only through the `wall_fallback` latch,
  because the whole function returns early unless that latch is up (`main.rs:13789`, read this
  pass). The rule is not missing; its reachability is. `wall_budget` runs always, on every frame,
  never latched and never frozen.
- `ceiling` is **not learned**. It is an absolute per-dispatch bound denominated in the thing the
  hardware actually fails on — `px × ss² × window` — seeded per adapter from `--chunk-sweep`
  (§7.3's calibration file, never from a converged budget), and no measurement, confidence rule or
  latch may raise it. It exists because of the failure scenario §3.1 states: on that frame the
  readings *were* representative in steps and recent, and `wall_budget` had seen only fast frames,
  so two learned terms both say yes and the dead-man latch arms on the frame *after* the fatal one.
  P5 is exactly this: a dispatch already in flight cannot be recalled. A `min()` over two learned
  numbers cannot bound a first surprise; a `min()` over *learned, learned, fixed* can.

**And `admit()` is made un-bypassable by the type system, not by review.** `Admission` is a struct
with no public constructor outside `render/pricing.rs`, and every dispatch entry point in
`fractadyne-gpu` takes one as an argument: the live paint path (`add_mandelbrot`, gpu
`lib.rs:2606`), the offscreen family (`export::render_iter` `:2018`, `render_iter_tiled` `:1222`,
`render_iter_gather` `:1617`, `render_iter_chunked` `:2274`, `render_iter_chunked_timed` `:2312`) —
which is the path the autopilot steering probe calls at `autopilot.rs:676` and the export path §1
says shares this gate — and the settle-tile path. A submission without an `Admission` does not
compile. That is the difference between this section and a better-organised guard. "There is no
code path from a `Plan` to a submission that does not pass through `admit()`" would be a claim about
the tree, checked by review — and review is precisely what failed for the third, fourth and fifth
presenters of the content guard (U20, U42, U45) and for `--livetest`'s controller copy. The token
makes the claim a property of the type system instead.

Two named predicates sit beside it, both new and both cheap:

```
fn budget_confident(v) -> bool
// true only when a reading PAIRED to a real dispatch landed within k frames
// AND that reading measured the whole of what was sent:
//     reading.pass_steps == plan.pass_steps  (both carried by the reading's provenance)
// When false: chunk at the bootstrap size, whatever the budget says.

fn budget_blind_latched(v) -> bool
// the existing budget_blind (render.rs:9438) with its reset rule corrected:
// reset on a DECREASE only, not on any movement (U15), and per view.
```

**Why the pairing, and not a fraction of the budget.** The obvious definition — "steps ≥
`PRICE_REPRESENTATIVE_FRAC` × `cur`" — is the defect this section indicts: `cur` *is* the budget, the
number under suspicion (U10), and a cost expressed as a fraction of a moving denominator expires the
moment the denominator moves. It would also fork a constant the code says must not fork.
`PRICE_REPRESENTATIVE_FRAC`'s doc (`render.rs:9292-9299`, read this pass) states that it "is named
here only so the two readers cannot drift", and `budget_step` (`render.rs:9323`) does not apply the
bare fraction at all: it applies

```
if !probe_would_price(steps, cur) && !slow { return None; }
```

— a slow undersized dispatch is deliberately *kept*, as "the strongest evidence the budget is too
high". A third reader with a third predicate would call exactly that reading un-confident, in a
section whose other hand deletes a duplicated literal to stop a drift. So `budget_confident` asks
the question its consumer actually has — *did this reading measure the whole of what I sent?* — by
comparing the reading's recorded `pass_steps` against the plan's, which is not a fraction of
anything. The single shared predicate, including the `&& !slow` exception, is stated once in
`pricing.rs` and used by `budget_step`, `probe_would_price` and `motion_jam_counts` alike.

This makes a **minimal reading provenance** a precondition of this section rather than a later rung:
`ReadingProvenance { dispatch_frame, view, pass_steps, res, ss }`, attached to every published
reading (`IterTiming`, gpu `lib.rs:177`; `CounterRead`; `ContentReading`, gpu `lib.rs:812`) and
landing with B4. The full `Provenance` of §5.5 — which additionally carries `FrameKind` and
`WalkSig`, and therefore waits for §5.3 — is a widening of this struct, not a separate one.

`budget_confident` goes beside `measurement_starved` (`render.rs:9692`) and is P2 applied in the
safe direction: it reads only what came back paired to what went out, and it is the named input for
"no reading has landed recently", which is the actual field state and which the pricing module
otherwise has no word for.

**The wall-clock dead-man, with a stated clear-condition.** This is the reaction that should ship for
the 2026-09-21 stall, and it is chosen precisely because it does not require knowing which of U9's
five mechanisms fired — mechanism (d), the cost being outside the timed pass, is unmeasurable by
construction (gpu `lib.rs:2367`).

> When `budget_blind_latched(v)` is true, `chunk_over[v]` becomes true regardless of `tdr_steps`;
> the next dispatch is floored at the chunk-band floor; and `budget_after_build_gate`
> (`render.rs:9525`) refuses budget growth until the latch clears — which happens only on a frame
> whose wall interval is under **half** `tdr_budget_ms`.

It changes no arithmetic, adopts no theory of the queue, and is falsifiable in one line of the log.

**Deletes.** `Perf::wall_fallback` (the global scalar latch, `main.rs:1103`) and its unlatch
(`main.rs:14206-14214`); the three `present_throttle` vetoes (`main.rs:13795-13796`,
`main.rs:14181`, and the `!throttled` term in `wall_shed_now`, `render.rs:9263-9265`) —
`present_throttle_step` survives as a *record field*, never as a gate; the two `&& !chunk_over`
suppressions (`render.rs:7012`, `7120`), so `chunk_over` is renamed `chunk_needed` and stops being
an actuator switch; the duplicated `0.7` literal at `render.rs:9409`, which becomes a read of
`PRICE_REPRESENTATIVE_FRAC` and so makes `motion_jam_counts`' own "one definition" comment true.

Two existing tests encode the premise being attacked and must be rewritten rather than deleted, with
the test count watched (P14): `present_throttle_tests.rs:37-46` asserts the throttle verdict must
arrive on 969 ms no-dispatch frames, and
`wall_shed.rs:51::present_throttling_vetoes_the_shed_however_large_the_accumulator` asserts the veto
as correct.

**Classes closed, and why they cannot recur.** *cost-model-blind* (15, open) and *measurement never
arrives ⇒ bootstrap binds* (7, open). The recurrence argument is structural, not procedural, and it
has three legs. A submission cannot be made without an `Admission`, and `Admission` cannot be
constructed outside `render/pricing.rs` — a future refactor that routes around `admit()` fails to
build, rather than passing review. `admit()` takes a `min()` over a term no other rule can freeze
(`wall`) and a term nothing can raise (`ceiling`), so neither a frozen budget nor a first surprise
can authorise an unbounded dispatch. And a budget that is too large can no longer switch off the
shrink or the tiles, because those are no longer gated on the chunk decision at all; a controller
that is unfed can no longer bind a bootstrap constant *upward*, because `budget_confident` names
that state and chunks.

The honest limit of the claim: the fixed ceiling is what stands between this design and "cannot
recur *after the first slow frame*". If the ceiling is ever judged unaffordable and dropped, this
paragraph must be downgraded to that weaker sentence and §10's falsifiers amended to match.

**Migration.** M2 (move the thirteen pure functions verbatim with their `#[cfg(test)]` siblings), M3
(move the rate latches), B4 (`wall_budget`, the fixed `ceiling`, `ReadingProvenance`, the
`Admission` token + `admit` at every entry point, and the three deletions), B5 (the actuator
un-suppression and the literal). Each rung leaves `main` shippable.

**If abandoned half-way.** After M2/M3 the tree is strictly better organised and behaviourally
identical — a safe stopping point. After B4 but before B5 the wall backstop exists but the
suppressions remain, so the min() can reduce the budget and the frame still will not shrink; that is
a *worse* intermediate than either end and must not be left as a release boundary. B4 and B5 ship
together or not at all.

### 5.2 A chunk-pass price with a constant term

**Replaces.** `render.rs:5165-5174`, which prices a pass as a pure function of the iteration window,
so a retreat asymptotes at a cost the model calls approximately zero.

**Creates.** `pricing::pass_price(fixed, spx, ss, window)`, where `fixed` is a **per-adapter
calibration value, not a tunable**: it lives in `validation/calibration/<adapter-slug>.json` beside
the per-adapter golden directory §7.8 already creates, is written by `--chunk-sweep` on that card,
and is read at startup by adapter slug with a conservative built-in default when no file matches.
It is deliberately not a `--set` override, because `--selftest` emits a failing "tunables are stock"
check under any override (`selftest.rs:3253-3260`) and §6.6.2's anti-vacuity guard refuses a verdict
unless tunables are stock — a per-card value delivered as an override would make every gate on that
card vacuous.

Thereafter it is refined per session as the intercept of the two cheapest and two most expensive
priced passes, **bounded**: the learned intercept is clamped to [0.25×, 4×] the calibrated seed and
may never be negative, because a two-point fit on a signal this document elsewhere calls late and
sparse has no business moving a floor by an order of magnitude. A clamp hit is a recorded field and
a `--logcheck` rule, not a silent saturation.

**Reconciling this with MEM-16.** The standing rule is to scope startup calibration to capability
discovery and a safe starting point, never to the cost model, because a calibration that depends on
*content* mis-sizes every later frame at a different location. The fixed term is the part of the
price that does not depend on content at all: it is the per-pass cost of clearing and storing 3–4
full-size `Rgba32Float` attachments and running `fs_resolve`, which is a function of the adapter and
the resolution and nothing else. That is the exemption argued explicitly, and it is falsifiable:
if `--chunk-sweep` at two different locations on the same card returns intercepts differing by more
than the clamp, the exemption is wrong and the term must be measured per view or abandoned.

**What `admit()` does *not* read.** `CTR_REBASE` / `CTR_BLA_SKIP` from `work_sink`
(`render.rs:5685`) are the only directly measured cost multiplier in the app, which makes them an
obvious secondary term inside `admit()`. They cannot be one, for two independent reasons, both read
this pass. The counter readback is armed only for full-frame iterates
with nothing already in flight — `arm_ctr` at gpu `lib.rs:2274-2276`, whose comment says a scissored
settle tile's counts are not a frame fraction — and it lands 2–3 frames after the render that
produced it (gpu `lib.rs:424-426`), with `work_sink` holding only the newest packed pair. So the
signal is sparse, late and absent on exactly the tiled settle frames. And U26 is unrefuted: the
count is shader arithmetic that folds differently per vendor, and the only cross-vendor gate is
deliberately loosened for that reason, so pricing admission on it would price the failing card on a
number we cannot show means the same thing there. It is therefore a **recorded, session-level
estimate** used for diagnosis and for W9's RED criterion (`rebase / sampled_px` above 4 while
`chunk_needed == false`), never a per-frame term in `admit()`. Its detect half gets a red-capable
check of its own in W8, in `--gputest`, which already grades op families per backend.

**And one band ledger per target and lane (U48).** `chunk_band_license` / `_update_with_lane` /
`_retreat` (`render.rs:9046-9270`) keep one ledger across two targets, two clocks and two lanes. M2
moves them verbatim; this rung then keys the ledger on `(target, lane)` so a retreat in one lane
cannot spend another's licence, with `--selftest pricing-props` asserting that two lanes driven from
one recorded stream never share a band index.

**Classes closed.** *actuator & retreat blind spots* (4, open) and the residual of *unbounded or
oversized dispatch*. A retreat cannot asymptote at a fictional zero once the model's floor is the
measured floor.

**Caveat, kept.** The claim that the same parity cliff applies to the *render* textures as to the
state textures is inferred, not measured — gpu `lib.rs:1220-1222` is explicit that padding "does NOT
round the RENDER resolution". Do not act on that half without a measurement.

**Migration.** One B rung after B5, gated by `--chunk-sweep` on both cards.

**If abandoned half-way.** A fixed term seeded and never learned is still an improvement over zero,
provided the seed is per card — and since the seed is a calibration file rather than an override,
"seeded and never learned" is a shippable end state, not a stub. A fixed term seeded too *high*
collapses the settled walk, which is why the clamp exists and why "median settled walk passes per
second" is on the scorecard.

### 5.3 A frame-kind state machine, with one owner of the walk signature

**Replaces.** `build_params` producing nine distinct frame kinds through staged mutation of
`&mut self` (`render.rs:5890-8004` plus the four `bp_*` stages), with resolution written
independently in three places and `fe_dispatch_frame` stamped at four sites and rewound at a fifth.

**Creates.** `crates/fractadyne-app/src/render/frame_plan.rs`:

```
enum FrameKind { Settled, Motion, Reproject, PinWalk, SettledWalk, Tile, Hold, Placeholder, Accum }

struct Plan {
    kind: FrameKind, view: ViewId,
    res: Resolution,          // PRIVATE to this module
    ss, ask, chunk, tile, present, hold_uv, ...
}

impl Plan {
    fn set_resolution(&mut self, ...) -> WalkEffect   // the ONLY resolution writer
}

struct WalkSig(u64);
impl WalkSig { fn of(p: &Plan) -> WalkSig }           // the ONLY sig constructor
```

`Plan::res` being private to the module means "nothing else can restart a walk" is enforced by the
compiler rather than by review. That is the strongest kind of gate available and the cheapest to
keep: a new resolution writer outside the module fails to build.

One `fe_dispatch_frame` stamp in the submit path, keyed on `FrameKind`; the eight-frame rewind at
`render.rs:5843-5844` deleted. `jumped` becomes `kind == Reproject && prior.kind != Motion` rather
than an inference from `pointer.zoom_vel` — which closes U39 by removing the inference, not by
tuning it. Concretely, that is three call sites to change, not one function:
`hold_scale_floor(self.pointer.zoom_vel.abs() < 1.0e-9)` at `render.rs:7579` (the freeze path),
`:7745` and `:7776` (the two `hold_uv` snapshots). `hold_scale_floor` itself (`render.rs:8863`)
already takes the flag and does not move; its doc's "one definition, three callers" becomes true of
the *input* as well as the policy. `Plan` is per view by construction, which removes the view-0 substitution in the second
panel's moving resolution (U56) and in the slow-frame and BLIND lines (U15,
`main.rs:14739-14744`, `14785-14795`).

`visible_res` and `motion_pass_steps_last` join the mode-switch reset list at
`render.rs:6554-6647`, which today clears `motion_res_measured`, `pin_frames_last` and
`motion_step_last` but not these (U51).

**Deletes.** The three independent resolution writers as writers (they become callers of
`set_resolution`); the four extra dispatch stamps; the rewind.

**Classes closed.** *coarse or unsatisfiable identity key*, the walk-restart half of *iteration
starvation*, and the view-substitution half of *presentation ≠ render*. Recurrence is prevented by
the privacy of `Plan::res` and by `WalkSig::of` being the only constructor — a future change that
forgets the sig cannot compile.

**Migration.** M6 (introduce `Plan` / `FrameKind` / `WalkSig`; `build_params` returns a `Plan` and a
thin shim converts it to `MandelbrotParams` — byte-neutral by construction), then B7 (make
`Plan::res` private, route the three writers, collapse the stamps).

**If abandoned half-way.** M6 alone is a pure reorganisation and safe to leave. B7 half-applied — two
writers routed, one not — leaves an unowned writer and a false sense that the invariant holds, which
is the worst state of the three. B7 is one commit.

### 5.4 `decide(inputs) -> plan` as a pure function over a recorded struct

**Replaces.** The absence of any path from a recorded field session into the controller functions.
Every controller in the frame loop is already pure (`render.rs:8005-9692`, plus `choose_goal` at
`autopilot.rs:906`) and each has unit tests; what does not exist is a way to feed them a field log
(U32). A field log becomes a test today only by a human transcribing numbers —
`render/chunk_bands.rs:26-40`, `wall_shed.rs:16`, `sa_skip_budget.rs:16` are literally that.

**Creates.** `crates/fractadyne-app/src/render/decide.rs`:

```
struct FrameInputs { /* every value the decision reads, incl. RefState */ }
struct RefState { orbit_id, len, partial, prec, escaped_at, installed_at_frame }

#[must_use]
fn decide(&FrameInputs) -> Plan
```

and a `--replay <jsonl>` CLI mode that feeds recorded `FrameInputs` through `decide` and diffs the
recomputed `Plan` against the recorded one, field by field.

**Deletes.** `--livetest`'s controller copy (`livetest.rs:355-395`) and `profile.rs:748`'s
`SETTLE_DELAY = 0.35`. Both call `decide` instead. Both harnesses' synthetic frame interval
(`livetest.rs:417-419`, `profile.rs:611-616`) is replaced with a recorded interval, or failing that
an explicit `SyntheticInterval` marker in the record that makes the analysis tool *refuse* to score
any wall-dependent metric from that run.

**Boundary, stated.** `decide` cannot be pure over the whole of `build_params` in one step:
reference installs spawn threads (`render.rs:1954`, `2239`, `3013`), the GPU sinks are drained
mid-frame, and the readback machines mutate on their own schedule. The realistic scope is the cost,
pacing, resolution, chunking, pin and present decisions; the reference lifecycle stays imperative
and enters as an *input* (`RefState`). This boundary is named here because it is where a later
reader will be tempted to overreach.

**Classes closed.** *verification-harness defect* (16, open), specifically the mirror half: a harness
cannot drift from the controller it grades if there is only one controller.

**Migration.** M8 — the staged bodies of `bp_frame_budget` / `bp_chunk_tiling` / `bp_present_gate`
are lifted into `decide` one stage at a time, each stage a separate move commit — then B9 (delete
both mirrors).

**If abandoned half-way.** M8 is the single highest-risk rung in the plan: lifting stages out of a
2,100-line `&mut self` function. Each stage is byte-neutral under the corpus and `--selftest
live-res` already pins the arithmetic those stages produce, so a partial lift is safe to ship; but a
partial lift with `--livetest`'s mirror *already deleted* is not. B9 follows the last stage, never
an intermediate one.

### 5.5 Provenance on every texture and every reading

**Replaces.** The current split in which the present path verifies content but not provenance and
the accumulation path verifies provenance but not content (U42), and in which nothing records which
dispatch a timestamp described (U14).

**Creates.** One struct, defined once in `fractadyne-gpu` — the widening of §5.1's
`ReadingProvenance`, which lands earlier with B4 and already carries the first five fields:

```
struct Provenance { frame_idx: u64, view: u8, pass_steps: u64, res: (u32,u32), ss: u8,
                    kind: FrameKind, walk_sig: WalkSig, view_stamp: u64, ask: u32 }
```

attached to (a) every dispatch; (b) every published reading — the type definitions are `IterTiming`
at gpu `lib.rs:177`, `CounterRead`, and `ContentReading` at gpu `lib.rs:812`, which is where the
field is added; their arming and construction sites (gpu `lib.rs:2411-2417`, `424-449`) are where it
is filled; (c) every texture (`ViewResources::content_stamp` widened); (d) every record row (§6).

A consumer may apply a reading only when its provenance matches what the consumer is steering. A
mismatch is **counted and recorded, never silently dropped** — because a probe whose reading another
rule discards never terminates (P13), the mismatch counter feeds the starvation and blind detectors,
and W6's gate puts a bound on the **discard rate** itself, not only on the count of mismatches
applied. That bound is what U54 is about: the adaptive-iteration probe's staleness gate discards
readings silently today, so a gate that only counted accepted readings would be green in exactly the
state where nothing is being measured.

**Changes to existing gates.** `bp_present_gate`'s settled branch — today
`let fresh = !self.perf.hold_active[vs];` (`render.rs:5366`) — gains a verified-plus-stamp test. Be
clear about the cost: the pin branch fifteen lines *below* it (`render.rs:5381-5383`, read this
pass) reads

```
let fresh = !self.perf.hold_active[vs] && self.perf.content[vs.min(1)].live_verified();
```

which is a **content** test only — no `view_stamp`, no `content_stamp`, no provenance term at all,
consistent with §3.2's finding that the stamp has exactly two consumers tree-wide and neither is
here. So the settled branch is not inheriting an existing test: the stamp half has to be built, once,
for both branches. The coarse-preview gate (pinned today by `render/coarse_preview_gate_tests.rs`),
which asks a different question at a different sampling and fails open (U45), takes the same
verified-plus-stamp test rather than its own. The GPU display gate (gpu `lib.rs:2077`) requires
`hold_verified`, giving that field its first consumer that is not a `format!`. `live_verified()`'s
`||` short-circuit on `live_complete` (`render.rs:8322-8326`) is replaced so `content_has_detail`
(`render.rs:8260`) runs on the fast path too. The held frame's colour uniform is frozen with the
snapshot rather than rewritten every frame (gpu `lib.rs:2152`; `mandelbrot.wgsl:2388-2391`) for the
log branch (U43).

**Classes closed.** *stale or misattributed measurement* (the unresolved half of CL-1),
*presentation ≠ render* (10, open), and the hold half of *normalization source & wrong statistic*.
Recurrence is prevented because provenance is a *required field on the reading type*: a new
consumer that forgets to check it has to construct one, and a new presenter that forgets to verify
has no path to `display_hold` without a `hold_verified` term.

**Known risk this widens.** `CONTENT_MIN_ESCAPED` 0.1% on a 4×4 subsample can call a genuine
thin-filament frame blank, and three of those stop the dive (U57). Putting the content predicate on
the fast path widens that constant's blast radius. The mitigation is P10's calibration rule:
recalibrate across at least six measured views *in the middle of the range*, not only at the
extremes — a calibration checked only at its extremes regressed the middle once already (MEM-45).

**Migration.** Two rungs, and the split is the point. `ReadingProvenance` — frame, view,
`pass_steps`, res, ss on every published reading — lands with **B4**, because §5.1's
`budget_confident` is defined against it and cannot ship without it. The widening to the full
`Provenance`, with `kind` and `walk_sig` and the texture and present-gate work, is a B rung **after
B7**, because those two fields do not exist until §5.3 creates them. There is no interim in which
`Provenance.kind` holds a placeholder: the struct that exists before B7 simply does not have the
field, and `--selftest provenance`'s RED criteria that mention `kind` or `walk_sig` arrive with the
widening. §7.6 and §8 state the same order.

**If abandoned half-way.** Provenance attached to readings but not to textures still closes U14 and
is worth shipping alone. Provenance attached to textures while the present gate still fails open is
not: it adds a field with no consumer, which is exactly the U59 shape this change exists to remove.

### 5.6 Typed capabilities, so a guard cannot be offered an actuator it cannot reach

**Replaces.** A set of booleans read as though they were always available, set once at frame 0 and
defaulting to false with no retry and no logged verdict (`main.rs:1918-1919`, `14110-14137`).

**Creates.** `crates/fractadyne-app/src/render/actuators.rs`:

```
struct Actuators { chunk: Option<ChunkCap>, shrink: ShrinkCap, tiles: Option<TileCap>, ss: SsCap }

enum RetreatOutcome { Applied { actuator, from, to }, AlreadyAtFloor(actuator), NoActuatorReachable }

#[must_use]
fn retreat(&Actuators, reason: RetreatReason) -> RetreatOutcome
```

`NoActuatorReachable` is an always-on log line and a record field — the first-class value P4
demands. The capability *verdict* is logged at startup beside the existing adapter line
(`main.rs:14121-14136`), and the probe is retried until `frame.wgpu_render_state()` is `Some`.

`PIN_COMPLETION_TIMEOUT_US` (`tunables.rs:647`) is demoted from a latency escape to a genuine
"the callback machine is not running" escape, keyed on `retire_synchronous_dispatches` being armed
(`main.rs:1730-1735`, called at `livetest.rs:351`) or a device-lost flag, and otherwise scaled as
`max(PIN_COMPLETION_TIMEOUT_US, 4 × last_dt_ms)`. The headless harnesses already retire the gate
explicitly, so the absolute timeout is serving no live case it was designed for.

The autopilot steering probe (`autopilot.rs:676`) is routed through `admit()` like any other
dispatch. Be precise about what that buys: the probe is already size-bounded to ≤ 128×128 at ss = 1
(`autopilot.rs:658-664`), so routing it does not remove a large dispatch — it makes the probe's work
*visible to the same accounting as everything else*, which is the first half of the aggregate bound
U38 asks for. The second half, a bound on total in-flight work across the live path, the probe and a
concurrent export, is **not** delivered here and is not scheduled (§5.9, §8): MEM-37 has no
controlled reproduction. So this change does not close *main-thread blocking & unbounded aggregate*;
what makes that class red-capable is the record's watchdog heartbeat and the `--logcheck` rules of
§6.4 and §6.5, which turn a grind or a wedge into a positive artefact.

**Classes closed.** *actuator & retreat blind spots* (4, open). Recurrence is prevented by
`#[must_use]` on `retreat` plus the `RetreatOutcome` enum: a caller that ignores the outcome does
not compile, and "nothing happened" is a value the record carries rather than a silence.

**Migration.** A B rung, after B5 (the un-suppression). Order matters: if `retreat` is asked to reach
an actuator while `&& !chunk_over` still suppresses two of the three, `NoActuatorReachable` becomes
the normal answer and the signal is worthless.

**If abandoned half-way.** The enum and the log line alone are useful and safe. The demotion of
`PIN_COMPLETION_TIMEOUT_US` without the record to measure it is not — it changes pin serialization
on exactly the hardware we cannot observe.

### 5.7 A settings layer that cannot persist a lethal configuration

**Replaces.** A relaunch that resets the view (`main.rs:6184-6190`) and restores the ask.

**Creates.** In `crates/fractadyne-state/src/lib.rs:75-146`:

```
struct SessionOutcome { clean: bool, crash_kind, at_log2mag, max_iter, auto_iter,
                        admitted_budget, mode_rate }
last_session: Option<SessionOutcome>
```

written on clean shutdown and by the crash path. On a launch following an unclean exit, the loader
**quarantines** the ask that was live at the loss: it is not restored, it is offered — and it says
so, because a feature that can decline must say it declined (P17).

**Where the quarantine lives, and where it must not.** It is scoped to the crash-recovery *launch*
path, beside the existing `FRACTADYNE_RESTARTED_AFTER_GPU_LOSS` handling that already opens at Home
rather than at the view that lost the device (`main.rs:6184-6190`, read this pass). It is **not** a
change to `.fdn` loading in general, and `crash_view_fdn()` (`main.rs:708-724`) — which is the
*writer*, formatting `max_iter=` and `auto_iter=` into the file at `:718-719` — keeps emitting both
keys. A key honoured by the writer and ignored by the reader is precisely the CL-50 shape this
project gates against: `selftest.rs:4801-4812` asserts *writer ⊆ `KNOWN_VIEW_KEYS`*, and
`relaunch_policy.rs:8-9` (`crash_view_fdn_is_a_loadable_location`) pins that a crash view loads as a
location with its iteration count intact. Both move with this change only in the sense that the
quarantine test is added beside them; neither assertion is weakened. Reproducing the lethal
configuration deliberately — opening the crash `.fdn` from the file dialogue — still restores it
verbatim, which is what `--deviceloss-repro` wants anyway.

**Classes closed.** *crash & report invisibility / recovery*, the recovery half.

**Argument against, honestly.** This is the only user-visible behaviour change in the whole document
and the one place P17 can bite in the other direction — a quarantine the user did not ask for is a
surprise. It is included because the alternative, a relaunch that walks straight back into the
grind, has already been observed twice (MEM-38, U18). The notice is part of the gate's RED criterion
rather than a nicety.

**Migration.** A late B rung; it needs `admitted_budget` from §5.1 to have a price to compare.

**If abandoned half-way.** Writing `SessionOutcome` without the quarantine is pure gain — it is the
first time learned controller state survives the process. The quarantine without the notice is a
regression.

### 5.8 Reference fitness — shipped behind a tunable, default off

**Replaces.** Admissibility-by-drift as the only test an installed reference faces (§3.4).

**Creates.** A fitness term in `reuse_drift` (`render.rs:621-634`) and `try_reuse_reference`
(`render.rs:937-1003`): refuse reuse when `!partial && ask > REUSE_MAX_WRAPS * orbit_len` (new
tunable; the field value is about 7 wraps). `needs_quality`'s length term (`render.rs:7341-7344`) is
ungated from `partial` for the wrap case. `install_collapse` (`render.rs:9288-9290`) derates every
frame rather than only at a transition. `refcache_persist::find` (`:249-283`) ranks by fitness
against the ask using the header fields it already stores — `partial`, `iter`, `req_prec`, all
present in the blob header (`orbit_blob.rs:70-83`) and none of them read today. `sa_dc_max_log2` is
stored beside `bla_dc_max_log2` in `RefCache` (`main.rs:4617-4634`) and re-validated the way the BLA
is at `render.rs:7870`. And **install identity stops being "longer wins"** (U55): an incumbent and a
challenger escaped at *different points* are not comparable by length, so the install comparison
takes the same fitness term as the reuse gate — length relative to the ask, plus the point — rather
than the raw orbit length. This is listed as a change, not only as a risk, because the wrap-count
refusal without it re-opens the door it closes: a refused reuse asks for a build, and "longer wins"
is what decides whether the build replaces the incumbent.

The reuse precision comparand (`render.rs:623-625`) is raised to `inp.precision +
REF_PREC_HEADROOM` — and while there, **settle by measurement** the verbatim contradiction the tree
carries next to it: `render.rs:538-540` states that a reference built at a different precision rounds
to different df32 mantissas so "at least as precise" would not be pixel-identical, while
`tunables.rs:794-795` states that building at a higher precision leaves the df32 samples
byte-identical. Both read this pass; they cannot both be true. The calibration data already exists —
128 bits escapes at 570, 160 at 84,941, 207 at 570,711, 286+ survives 700k (memory
`topic-spar-family.md`, **not verified in code**). Instrument the intermediate value before
theorising, and record the answer in this document's successor.

**Default off.** This is the one change in the architecture that touches the picture and can re-enter
the build-storm and lookahead-clobber families (U50, U55; CL-102, CL-127), and P18 says to audit
every heuristic tuned while the old behaviour was live — the iteration-boost seed and all three
install derates were calibrated against it. The tunable flips on only when: the reference-rebuild
rate on the grand tour is within 10% of beta.112; `--selftest` goldens are byte-identical; and
`--bench-matrix` reports 0 algorithmic drift.

**Expect a red by design.** `reuse_tests.rs:82-95` asserts that an escaped orbit is reused as-is and
that its length is unchanged. That test encodes the defect. It moves with the fix, and the test count
is checked before and after (P14).

**Classes closed.** *reference lifecycle* (16), and the entry path into *cost-model-blind*.

**If abandoned half-way.** The precision comparand and the disk-cache ranking are independently
shippable and low-risk. The wrap-count refusal without the derate is fine; the derate without the
tunable is not.

### 5.9 What is rejected, and why

| rejected | why |
|---|---|
| A parallel telemetry system, a network sink, uploads. | `diag` already has a teed log, a crash composer, an unclean-exit marker, an OOM reserve and `perf.jsonl`. A second system would be a second thing to keep honest. One record, three sinks, all local. |
| Tightening the shared cross-GPU golden tolerance so something can fail on the RX 6800 XT. | An RX 6800 XT legitimately reaches meanΔ 19.15 against a 24.0 limit (U30). A tighter shared constant manufactures red. The answer is a per-adapter baseline (§7.8), never a tighter shared constant. |
| Pricing on the rebase counter, primary **or** secondary. | Two reasons, either sufficient. It is shader arithmetic that folds differently per GPU vendor, and the only cross-vendor gate is deliberately loosened for exactly that reason (U26). And the reading is armed for full-frame iterates only, one in flight per view, landing 2–3 frames late (gpu `lib.rs:2274-2276`, `424-426`), so it cannot be a per-frame term at all. It is a recorded session estimate and a gate criterion (§5.2, W9), never an input to `admit()`. Its detect half gets a `--gputest` check (W8). |
| Making `decide` pure over the reference lifecycle. | Installs spawn threads and land asynchronously; forcing them into a pure function would either serialise them or lie about them. `RefState` enters as an input (§5.4). |
| A chunking heuristic keyed on depth. | A device loss is not a depth problem — the 2026-09-21 loss was at 1.76e6× (MEM-39). |
| Rewriting the renderer, the shader or the bignum layer. | Byte-identity on the corpus is a constraint on every commit here. |
| Fixing U27 (the autopilot cannot name starvation with auto-iter on) as part of this. | A message-correctness bug with a small local fix and no architectural content. It belongs in the ordinary bug queue; its *detection* half is covered by `capped_frac` / `iter_exhausted` entering the record and the harness verdicts (§3.5: both fields already have live consumers and one red-gated test). |
| Fixing MEM-61 — a motion chunk's `[0, step]` contract being false wherever the view's first escape exceeds `step` — as part of this. | It is open, user-confirmed, and in an open class, so it gets a stated disposition rather than silence. It is a contract bug in one predicate, not an architectural one, and it is **not verified in code this pass**; verifying and fixing it belongs in the ordinary bug queue, ahead of most of it. The interim answer here is *detection*: W6's screen gate scores the flat fraction that is its symptom, and W8's `--livetest` second oracle plus `capped_frac` / `iter_exhausted` in the verdict make an iteration-starved view fail rather than match a starved oracle. §10 repeats this so it is not lost. |

### 5.10 Migration: byte-neutral steps in an 814 KB and a 595 KB file

`main.rs` is 814,304 bytes and `render.rs` is 595,531 bytes. The refactor risk is real, so the
discipline is fixed.

**Rule.** Every commit is exactly one of:

- **(M) a pure move** — code relocated into `render/<mod>.rs` or `diag/<mod>.rs` with no logic
  change, gated on *signature* identity, appended to `.git-blame-ignore-revs`; or
- **(B) one behaviour change**, with a gate that can go red, named in the commit message.

Never both.

**The move gate** is `--bench-matrix` **signatures** — not timings. One control run cannot attribute
~2 ms and the fixed term drifts 10–13 ms on identical code, so a timing number on a refactor commit
is noise; build both exes and interleave ABABAB median-of-3 only when timing is the question.
Plus: `--selftest` goldens byte-identical; the F3 corpus compared pixel-wise, never by file hash; the
`test result:` count must not fall; and EOLs re-checked before any anchored append, because a branch
checkout under `autocrlf=true` re-checks files out as CRLF.

| step | kind | what it does to what exists |
|---|---|---|
| M0 | move | `diag/frame_record.rs` created; `diag::budget_note` (`diag.rs:539`) kept as a thin adapter that also fills a `FrameRecord`. Nothing else moves. |
| B1 | behav | One `diag::frame_record()` call at the end of `update()` (`main.rs` ~14800). `BUDGET_LOG_CAP` 24 (`diag.rs:58-62`) becomes a typed 4096-entry ring. Crash report gains a `frames:` section. |
| M2 | move | `render/pricing.rs`: the thirteen pure functions at `render.rs:8005-9692` move verbatim, with their `#[cfg(test)]` siblings — `chunk_bands`, `wall_shed`, `controller_props`, `motion_pace`, `probe_price_tests`, `present_throttle_tests`, `budget_blind_tests`, and also `motion_jam` (because `motion_jam_counts` is in the move set), `sa_skip_budget` and `frame_cost_tests`, which sit in the same neighbourhood. Zero logic change. |
| M3 | move | `record_mode_rate` / `record_motion_rate` / `motion_wall_cut` / `bootstrap_steps` (`main.rs:1616-1814`) move into `render/pricing.rs` as methods on `Meter`. |
| B4 | behav | `wall_budget` and the fixed per-adapter `ceiling` added; `ReadingProvenance` attached to every published reading; the `Admission` token introduced and required by every dispatch entry point; `wall_fallback` latch, unlatch and the three `present_throttle` vetoes deleted. |
| B5 | behav | `&& !chunk_over` deleted at `render.rs:7012` and `7120`; `chunk_over` renamed `chunk_needed`; `render.rs:9409`'s literal replaced by `PRICE_REPRESENTATIVE_FRAC`. `budget_confident` and the blind dead-man land here. **Carries a P18 audit and a screen measurement of its own** — see below. |
| M6 | move | `render/frame_plan.rs`: `Plan`, `FrameKind`, `WalkSig`. `build_params` returns a `Plan`; a thin shim converts it to `MandelbrotParams`. Byte-neutral by construction. |
| B7 | behav | `Plan::res` made private; the three resolution writers routed through `set_resolution`; the four `fe_dispatch_frame` stamps and the rewind collapsed to one. |
| M8 | move | `render/decide.rs`: the staged bodies of `bp_frame_budget` / `bp_chunk_tiling` / `bp_present_gate` lifted into `decide(&FrameInputs) -> Plan`, one stage per commit. |
| B9 | behav | `--livetest`'s controller copy (`livetest.rs:355-395`) and `profile.rs:748` deleted; both call `decide`. |
| B10+ | behav | Provenance (§5.5), actuators (§5.6), settings quarantine (§5.7), reference fitness behind its tunable (§5.8) — each its own commit with its own gate. |

The single highest-risk rung is M8. Its mitigation is that it is performed stage by stage, each stage
is byte-neutral under the corpus, and `--selftest live-res` plus the goldens already pin the
arithmetic those stages produce.

**B5 carries a P18 audit, on the commit itself.** Deleting `&& !chunk_over` at `render.rs:7012` and
`:7120` makes the resolution shrink and the settle tiles engage in frames where they were suppressed
— and every constant that decides what a moving screen looks like was calibrated with those
suppressions live: `112a088`'s seven-rung sticky ladder, `MOTION_CUT_MAX` 2.0, the 40% visible
headroom, `min_motion_res`. P18 says to audit every heuristic tuned while the old behaviour was
live, and it applies here at least as much as it applies to §5.8. So B5 is the one behaviour rung
that ships with a screen measurement attached rather than deferring it to a later workstream: three
pairs through `scripts/dive-capture/`, control scored, blank and stale fractions before and after,
and each of the four constants either re-measured or explicitly recorded as unchanged with the
engagement rate that justifies it.

---

## 6. The instrumentation

Three jobs, one record:

1. **Diagnose a field failure from the artefacts alone** — no trace flag, no reproduction, no
   correspondence with the reporter.
2. **Validate a fix by measurement before and after**, rather than by argument.
3. **Feed the replay tests as their input**, so a field failure can become a regression test.

Everything below builds on what `diag` already has. A parallel system is rejected (§5.9).

### 6.1 Build identity — the field that makes every other artefact attributable

Every exit criterion in §7 reads "green on the Radeon three runs running", and today nothing ties a
field crash report to a commit. On 2026-08-16 a six-phase run started 53 s after a push measured the
previous binary and nobody could tell.

- `crates/fractadyne-app/build.rs` — beside the existing `FRACT_BUILD` counter, emit
  `cargo:rustc-env=FRACT_GIT=<short sha>[-dirty]` from `git rev-parse --short HEAD` plus
  `git status --porcelain`, falling back to `unknown` outside a work tree (the tarball case, which
  `publish-share.ps1` builds from `git archive`). `build.rs` runs on every recompile, so this must
  not shell out slowly.

  **And it must be told when to re-run.** The file as it stands (27 lines, read in full this pass)
  emits **no `cargo:rerun-if-changed` at all**, so Cargo falls back to "re-run if any file in the
  package changed" — which does not include `.git/HEAD` or `.git/index`. A `git commit` with the
  tree unchanged, a checkout of a branch with identical sources, or a `git stash` would then leave
  `FRACT_GIT` holding the *previous* sha and the previous `-dirty` state: the exact failure this
  section opens with, reproduced by the fix for it. So the script emits
  `cargo:rerun-if-changed=../../.git/HEAD` and `cargo:rerun-if-changed=../../.git/index`, and the
  honest residue is stated rather than hidden — a staged-but-uncommitted change can still move the
  dirty flag between index writes, and the `git archive` tarball has no `.git` at all, which is why
  `unknown` is a first-class value and why `BUILD-ID.txt` (below) is what the share actually trusts.
- `crates/fractadyne-app/src/sysinfo.rs:8-14` (read this pass: `APP_VERSION`, `BUILD_SEQ`,
  `version_string()`) — add `pub(crate) const BUILD_GIT` and extend `version_string()` to
  `0.2.41-beta.113 (build 1934, g1a2b3c4)`.
- `crates/fractadyne-app/src/tests.rs:599-612` (read this pass: `title_string_matches_version`,
  which today asserts the title carries `CARGO_PKG_VERSION` and a `(build N)` number) — extend to
  require the git field's **presence and shape**, and no further. It must **not** fail on `-dirty`:
  it is an ordinary unit test, so a `-dirty` assertion would turn every `cargo test` run during
  development red, including the runs §5.10's move gate depends on ("the `test result:` count must
  not fall") and P14's count-watch, both of which happen on dirty trees by construction. The
  `-dirty` refusal belongs where a dirty tree is genuinely wrong: `scripts/publish-share.ps1` and a
  `run:` checklist row, which is also where a release is actually assembled.
- `scripts/publish-share.ps1` writes `BUILD-ID.txt` (tag, sha, build seq, UTC, the four package
  sha256s) into `<Share>\builds\<tag>\`.
- `scripts/gpu-validate.ps1` gains step `00-build-id`, comparing the installed app's `--version`
  against `BUILD-ID.txt` from the folder it was installed from.

One day of work. Every existing consumer inherits it free: the crash report header
(`diag.rs:605-618`), `perf.jsonl` (`diag.rs:482-483`), the unclean-exit report (`diag.rs:686`),
`window_title()`, `--uitest` screenshot metadata.

### 6.2 What exists today, and the precise gaps

`diag.rs` already has a teed log file with `[+s]` stamps (`149-264`), a 24-entry always-on budget
ring (`58-62`, `539-556`) dumped into the crash report, a breadcrumb (`499-521`), a manifest global
(`526-530`), a crash-report composer (`601-665`), an unclean-exit marker that survives `0xc0000409`
and `0xc0000005` (`667-743`), an OOM reserve (`alloc.rs`), a hang watchdog (`827-864`) and
`perf_jsonl` (`472-490`). The frame loop emits always-on lines for the lethal band, the blind
tripwire, slow frames, starvation, mode switches and jams. There is a timestamp overlay for lining
a screen recording up against the log.

The gaps are precise, not general:

| gap | evidence |
|---|---|
| No join key | `slow frame N`, `[fd-glide] fN`, `chunk f=`, `pin-* f=` and the overlay carry a frame index; the budget ring, the LETHAL-BAND line, the manifest and the crumbs do not. |
| Coverage | The ring holds 24 entries at ~17 decisions/s ≈ **1.4 s**; the field episode ran **33 s**. |
| Missing inputs | The ring carries ms/steps/budget only — no `iter`, `eff`, `boost`, frame index, `explicit` or `building`, the last two being inputs `budget_step` and `budget_after_build_gate` actually consume. |
| Contradictory artefacts | Manifest `iter=4627` against `.fdn` `max_iter=25000 auto_iter=1`, same crash, same second (U28). |
| Everything richer is trace-gated | `FRACTADYNE_TRACE` is read once at startup through a `OnceLock` (`diag.rs:323-333`), so a field session is untraced by definition. The 2026-09-21 session contains 0 trace lines. |
| Nothing machine-reads it | Five log-reading sites, all PowerShell, all copy-or-grep-a-literal. The only parsers in the tree are `scripts/zoomtest_report.py` and `scripts/dive-capture/motion.py` (U29). |
| The log drowns | `[fd-accum] view 0: begin` (`main.rs:13418-13421`) fired 10,323 times at ~31/s for 740 s — about 65% of the crashing session's 1.06 MB — because `accum_active` is cleared again every frame the content stamp does not match (`main.rs:13374`, and on a signature change at `:13338`), so the episode re-begins and re-logs. |
| No colour or normalization field anywhere | The record this section proposes must close the *engagement* half of the longest-running class in the catalogue (13 incidents), and `norm_complete` (`render.rs:5741`) and the normalization window appear in **no** `diag` artefact today (grepped this pass). |

### 6.3 `FrameRecord` — the exact fields

One `#[repr(C)] #[derive(Copy)]` struct, **about 360 bytes packed** by inspection of the field list
below (identity ~16, ask ~26, reference ~32, price ~77, state ~33, signals ~28, counters ~36,
normalization ~24, plan ~30, read ~20, retreat ~8, input ~31, plus padding), written once per frame
per view at the end of `update()` after the slow-frame line (`main.rs` ~14800), so it sees the plan,
the reading and the interval together. Every size, cadence and coverage figure in §6.4 is derived
from that number and a **512-byte slot**, not from a guess; the slot is deliberately larger than the
struct so a schema addition does not re-key an existing file.

```
// identity — the join key nothing has today
frame: u64, t_ms: u32, view: u8, kind: FrameKind, schema: u16,
refusal: Option<RefusalReason>          // see below: NOT a FrameKind variant

// the ask (U28 — no always-on record today)
max_iter, auto_iter, explicit, building, gpu_iter, eff_iter, boost,
capped_frac, iter_exhausted, budget_maxed

// the reference (U6, U13, U17, U25) — the field shape was orbit_len 655 against ask 4627
ref_orbit_id, ref_len, ref_partial, ref_prec, ref_installed_frame, ref_origin,
install_trigger, sa_skip_eff, bla_on

// the price (U1, U4, U10, U22, U23)
fe_budget, wall_budget, admitted_budget, bootstrap, mode_rate, motion_rate,
band_idx, band_licence, pass_fixed_steps, nominal_steps, priced_steps

// the state (U24, U39, U51, U56) — per view, never a view-0 substitution
res_w, res_h, ss, visible_rung, motion_res, walk_sig, walk_restarted,
chunk_lo, chunk_hi, tile_idx

// the signals (U11, U12, U14, U15)
last_dt_ms, body_ms, acquire_ms, present_ms, idle_ms, repaint_requested,
present_throttle, pin_inflight, ts_state, frames_since_reading, blind_slow_frames

// the counters the app measures and throws away (U17) — each an Option, and each
// carrying the frame it describes: the readback is armed for FULL-frame iterates
// only, one in flight per view, and lands 2-3 frames late (gpu lib.rs:2274-2276,
// 424-426). Recorded as plain per-frame fields they would attribute one frame's
// rebase count to a dozen others — §2.2's "a measurement paired with the wrong
// subject", manufactured inside the instrument built to detect it.
ctr: Option<{ src_frame: u64, rebase, bla_skip, escaped_px, sampled_px,
              maxiter_capped, glitch }>

// colour and normalization — the engagement half of a 13-incident class,
// absent from every diag artefact today
norm_lo, norm_hi, norm_complete, norm_shown, colour_sig

// the plan (the output of decide())
chunk_needed, budget_confident, shrink_applied, tiles_armed, ss_cap, pass_steps,
pin_verdict, present: Real|Hold|Reproject|StaticBlack, hold_scale_oct, hold_verified

// what came back
read_src: Gpu|Wall|None, read_ms, read_steps, read_prov_frame,
read_verdict: Accept|Discard(reason)|ClampToCur

// what was attempted
retreat_reason, retreat_outcome: Applied|AlreadyAtFloor|NoActuatorReachable

// the user's input this frame (never logged today)
input: { wheel_delta, space_held, drag_dx, drag_dy, click: Option<(x,y,factor)>,
         key: Option<KeyKind> }
```

Three additions deserve their own note, because each removes a specific ambiguity:

- **The user's input stream.** Without it a replay can rerun the controllers but not the camera, so
  the 33 s approach to the 2026-09-21 loss stays unreconstructible. It is the difference between a
  controller-only regression test and a camera-and-controller one.
- **A refusal is a recorded reason, not an inferred silence.** Because `budget_note` is emitted
  *after* the build gate, a growth refused for a reference rebuild (`main.rs:13702-13710`) is
  written into the ring as an indistinguishable `" (unchanged)"` (`main.rs:13747-13752`), and the
  reason at `:13702-13710` is `diag::trace`-gated, i.e. absent from every field session. It is
  carried here as `refusal: Option<RefusalReason>` rather than as a `FrameKind` variant, because
  `FrameKind` is the *shape of the frame* — the type `Provenance` and `Plan` both carry — and a
  frame whose budget growth was refused is still a Settled or a Motion frame. `explicit` and
  `building` are recorded beside it for the same reason §6.2's gap table names them: they are the
  inputs `budget_step` and `budget_after_build_gate` actually consume.
- **A header block, written once per session** and repeated at the head of every rotated file:
  session id, app version + `FRACT_GIT`, adapter name + backend + `TIMESTAMP_QUERY` +
  `attach_bytes/sample granted of available` (`main.rs:14121-14136` — the one line that decides
  which code is under test), `tunables::status_line()`, bignum backend, window size, and the
  `session.toml` controller constants a replay needs: `max_iter`, `auto_iter`, `zoom_rate`,
  `work_budget_scale`, `min_motion_res`, `prefer_detail`, `autopilot_priority`. Without this a
  replay can be scored against the wrong code path.

Two constraints on every field, both from §4:

- Guarded quantities are populated **from the guard's own return value**, never recomputed (P10).
  `chunk_needed`, `admitted_budget`, `hold_verified`, `pin_verdict`, `escaped_px` and
  `retreat_outcome` are the values the guards returned.
- What the **consumer** reads, not the target (P10). Hence `visible_rung` and `res_w/res_h` as
  dispatched, not as requested.

### 6.4 Where it goes, at what cadence, and how it survives

Three sinks, one record.

**The encoding, stated once, because three other numbers depend on it.** Fixed-width scalars in
declared order; every enum a `u8` with `0 = unset` reserved, so §6.6.1's "a required field at its
default sentinel" check has something to test; every `Option<T>` as `(present: u8, T)` with the
payload zeroed when absent; `f64` where the app holds an `f64` and no lossy narrowing anywhere.
`#[repr(C)] #[derive(Copy)]` alone does **not** give a stable file layout for
`input.click: Option<(x,y,factor)>`, `input.key: Option<KeyKind>`, `read_verdict: Discard(reason)`
or `present: Real|Hold|Reproject|StaticBlack`, so the file format is this encoding, written and read
by generated code, and §6.6.3's round-trip test is a test of *it*.

**(1) In-memory ring — every frame, unconditionally.** `[FrameRecord; 4096]`, pre-allocated, single
writer (the UI thread owns `update()`), no allocation, no lock. 4096 × 512 B = **2 MB resident**.
The coverage property is the one that matters, and it is stated **per view**, because the record is
written once per frame per view and the slot key below carries the view: with one panel the ring
holds 4096 frames — 68 s at 60 fps, and **14–68 minutes** at the failing cadence of 1–5 fps. In dual
view each panel gets 2048 slots, so the same figures halve: 34 s at 60 fps, 7–34 minutes at the
failing cadence. Either way the ring lengthens in time exactly as the app slows down, which is the
opposite of the current 24-entry ring's behaviour and covers the entire 33 s approach plus the 860 s
session that led to it. (2 MB resident buys the headroom; if it ever needs to be 1 MB, halve the
slot, not the slot count.)

**(2) `frames.bin` — a fixed-size circular file, for the deaths the panic hook never sees.**
`0xc0000409` (fastfail) and `0xc0000005` (access violation) bypass the hook entirely; only OOM is
covered today, and the rest is inferred from a surviving `session.running` marker quoting six log
lines (`diag.rs:667-743`). A **2 MB** file of 4096 fixed 512-byte slots, one positional write per
record at `((frame * 2 + view) % 4096) * 512`, **no fsync** — the page stays in cache and the file
is not deleted, so a process abort leaves the tail intact. The slot key carries the view
deliberately: keyed on the frame index alone, view 1 would overwrite view 0 every frame and dual
view — which is in scope (§1) and is one of the two open/unexplained incidents — would leave exactly
half a record on disk. On the next launch the unclean-exit path reads `frames.bin` and folds the
last N records into the report it already writes. Honest caveat: a hard power-off or a bugchecked
machine loses the tail; a process abort does not. A **device loss** is not a process death — the app
survives it and can write the record itself — so `frames.bin` is the backstop for the abort case,
not the device-loss case.

**(1b) A heartbeat row, written by the watchdog thread.** The ring, `frames.bin` and `frames.jsonl`
all have exactly one writer — the UI thread inside `update()` — which is the thread that stops in
the failure this record is credited with catching. That is fine for a *grind*, where frames still
tick at ~1 fps, and useless for a *wedge*, and MEM-38's report was "the app is no longer
responding". P19 says a judge must not run on the thread it judges. So the existing hang watchdog
(`diag.rs:827-864`, a separate `fd-watchdog` thread that already logs `possible hang` after 10 s and
re-warns every 30 s) owns a `last_record` timestamp and writes a heartbeat row of its own — kind
`Stall`, carrying the duration, the breadcrumb and the last frame index it saw — so the absence of
frames becomes a positive record with a length rather than a gap a human has to notice. It is the
one writer that is not the UI thread, it writes only when the UI thread has not advanced for N
seconds, and it is the reason a wedged session produces a red rather than a silence.

**(3) `frames.jsonl` — always on, event-triggered, rate-limited.** Not gated on
`FRACTADYNE_PERF=1`; `diag::perf_jsonl` (`472-490`) is the vehicle, with the gate moved off event
records. Write one record per frame while **any** of these holds — slow frame, lethal band, blind
episode, retreat attempted, `present != Real`, reading discarded, budget moved, mode switch, pin
stop, walk restarted, provenance mismatch — and otherwise one **per-second summary** row (min /
median / max of the numeric fields plus counts). Estimated size for a 14-minute field session: about
1,000 event rows plus about 840 summary rows at ~200 B = **under 0.4 MB**, against the 1.06 MB the
current log spends mostly on the accum flood. Rotation: 32 MB, four slots, checked mid-run —
replacing the single-slot, startup-only rotation at `diag.rs:200-203` that lets a long session
overwrite its own head.

**Cost per frame.** About 40 stores, one ring write, one page-cache-resident 512-byte `write_at`.
The honest position is that this is an *estimate* of well under 10 microseconds — under 0.06% of a
16 ms frame and under 0.001% of a 1000 ms frame — and that it must be **measured before shipping**,
by building both binaries and interleaving ABABAB median-of-3, because a single control run cannot
attribute a couple of milliseconds. That measurement is a named exit criterion in §7.1.

**Log hygiene, so the record is findable.** `[fd-accum] begin` (`main.rs:13418-13421`) becomes
edge-triggered — the flood is the episode restarting, because `accum_active` is cleared again on
every frame whose content stamp does not match (`main.rs:13374`) — with `accum_transitions` carried
as a count in the record, which is the information the flood was accidentally conveying. The issue
reporter (`main.rs:9478-9519`) attaches the `frames.jsonl` tail instead of 48 KB of log, which at
the field cadence was about 12 s of history. **What the reporter strips, said explicitly:** the
repository is public and the user's home path is their account name, so the reporter redacts the
home path from every line of the attached tail and from the header block's paths, exactly as the
site-publishing rule requires; the session id, version, `FRACT_GIT`, adapter line and
`tunables::status_line()` are kept, because they are what makes the report attributable and none of
them names the user.
Reference-build workers (`render.rs:1954`, `2239`, `3013`) get thread names so `[crumb]` stops
printing `(?)`. `DIAGNOSTICS.md`'s tables are completed — `[fd-glide]`, `[fd-accum]`, `[fd-view]`,
`[fd-cache]`, `[crumb]` and `refwaste` are all absent today.

### 6.5 How it is read back

Three readers, at three distances from the machine.

**`scripts/framelog.py`** — one tool beside `zoomtest_report.py` and `dive-capture/`, three
subcommands answering the three jobs.

- `framelog.py summarize <frames.jsonl|frames.bin|crash-*.txt>` — the scorecard of §9, plus the
  five-way discrimination U9 asks for: *timestamps never armed* (ring empty, `ts_state` pinned,
  `frames_since_reading` climbing); *armed on the wrong dispatch* (`read_prov_frame` against
  `frame`, `read_steps` against `nominal_steps`); *the bracket does not cover the work* (plausible
  `read_steps`, implausible `read_ms`); *the cost is outside the timed pass*
  (`last_dt_ms - read_ms` large and persistent); *the readings could not move the budget*
  (`read_verdict` histogram). Two of those five are not answerable from beta.112's ring. All five
  are answerable from this record. That is the specific reason to build it.
- `framelog.py compare A/ B/ --pairs 3` — the A/B instrument. Requires at least three pairs and
  reports each metric's delta against its own run-to-run variance, because single runs vary 8–14%
  (P11). It **scores the control** — every arm against every other — because a metric that cannot
  tell the right scene from the wrong one is measuring nothing.
- `framelog.py replay <frames.jsonl>` — feeds each recorded `FrameInputs` through `decide()` via the
  app's `--replay` mode and diffs the recomputed `Plan` against the recorded one, field by field. A
  field log dropped into `validation/replays/` is thereby a regression test.

**`--logcheck`** (`crates/fractadyne-app/src/logcheck.rs`) — the cheap bridge to red capability,
available in under a week and covering every gate run in the interim while the record is being
built. Every in-process harness calls it on its own log at finish **and it is runnable
out-of-process**, `--logcheck <path>` against a log left by a run that was killed, wedged or
watchdogged. That is not a convenience: a harness that wedges never reaches "at finish", so a
verdict computed only on the judged thread produces silence rather than red — MEM-68's shape, and
P19. The field loop needs the out-of-process form anyway, since `fieldcheck.py` reads logs from the
share. Rules live in a committed `validation/logcheck-rules.toml`:

| rule | bound |
|---|---|
| `⚠FRAME BUDGET IS BLIND` | 0 allowed, outside the rung that expects it |
| `⚠LETHAL-BAND FRAME` / `⚠IN-FLIGHT PASS IN THE LETHAL BAND` | allowed only in declared rungs |
| `no GPU iterate timing after 30 frames` / `timing resumed` | rate-bounded (the field session flipped 58 times in ~35 s) |
| `motion jam`, `⚠FOLD AT ANOTHER VIEW`, `reference-build storm`, `[fd-watch] possible hang` | bounded / 0 allowed |
| `[fd-accum] … begin` | rate-bounded |
| each harness's own header and verdict line | **required** — the general mechanism for "verify a harness actually ran", today asserted in only five places |

Three cautions travel with it. Match on a stable prefix token, with a unit test that each rule
matches a line the code can actually emit. Keep the tripwire's **predicate** where it is, so
`--logcheck` never becomes a second definition of the condition. And the rules file is read
**strictly**: an unknown key, an unknown rule name or a malformed bound is a load error, not a
skipped rule. That is the same decision §6.6.3 makes about the record's reader and for the same
reason — SCRIPT FORMAT v2 silently ignores unknown keys, and a typo in a rule name that silently
disarms a gate is the CL-124 / silent-default family reproduced inside the gate layer.

**`scripts/fieldcheck.py`** — reads a `gpu-validate` / `radeon-verify` bundle from the share and
emits `verdict.json` plus a non-zero exit, so the dev box reads one exit code and not a 1 MB log.
Its rules are `--logcheck`'s plus the field-only ones: every BLIND episode must be followed within k
frames by a recorded budget **decrease**; every lethal-band line must be followed by a visible
retreat; any `crash-*.txt` is RED; the bundle's build must be non-dirty and must match
`BUILD-ID.txt`; and **"no verdict line from harness X" is its own non-zero outcome**, distinct from
that harness passing and from it failing. A bundle whose `--motiontest` log simply stops is not a
bundle with nine green harnesses; P20 says a missing verdict is a failure, and the only way to keep
that true is to enumerate the harnesses the bundle was supposed to run and check each one off.

### 6.6 Keeping the instrumentation honest

An instrument that cannot be checked is the failure mode this whole document is about. Four
mechanisms.

1. **A check that the recorder recorded.** `--selftest record` — a normal group, **not** `opt_in`, so
   it runs in every unfiltered `--selftest`; `opt_in(tag)` is false without a filter
   (`selftest.rs:355-358`), which is why `deep-location` never runs. It drives 120 live frames and
   goes RED if: the record count differs from the frame count; any frame index is missing or
   duplicated; any required field holds its default sentinel; the schema round-trip is lossy; or
   `frames.bin` read back after a simulated abrupt exit is short.
2. **An anti-vacuity guard, so an empty record cannot read as a clean run.** Every consumer —
   `framelog.py`, `--selftest replay`, every harness verdict built on the record — refuses to return
   a verdict on a set that fails a richness test: at least K frames, at least one dispatching frame,
   at least one reading, at least one non-`Real` present if the session moved, `tunables: stock`
   (`tunables.rs:122-133`), and the `session:` line present. Failure is **exit 2 / VACUOUS** — never
   exit 0, and never the same code as a pass. U8's lesson is that a verdict which cannot distinguish
   "did not reach the regime" from "healthy" is worse than no verdict, so VACUOUS must also be
   distinct from FAIL.
3. **A strict schema.** A `schema` field on every row, a Rust round-trip test over every field, and
   `framelog.py --schema-check` failing on an unknown **or** missing key. Today `budget_note` lines
   are free-form and the ring test asserts only substrings (`diag.rs:960-976`), so nothing guards a
   format change against its reader — and the project has been bitten by the opposite policy
   (SCRIPT FORMAT v2 silently ignores unknown keys). The reader is strict on purpose.
4. **Per-assertion reachability.** `validation/reachability/` holds a deliberately-corrupted input
   per rule, and a `--selftest --selfcheck` / `--replay --selfcheck` mode requires each one to fail.
   This is narrower and stronger than a per-harness sabotage: the meta-gate proves a *harness* can
   fail; this proves each individual RED criterion inside `--selftest record`, `--selftest replay`
   and `pricing-props` can actually fire. `checklist_coverage.py` is extended to require a
   reachability fixture and an exit criterion for every row claiming a harness, so P9 becomes
   enforced rather than asserted.

### 6.7 Instruments for reaching the regimes

Instrumentation is not only recording. Three bounded, opt-in instruments make the regimes that kill
reachable on demand; all three are documented in `DIAGNOSTICS.md` and none may appear in a default
ladder selector.

- **`FRACTADYNE_REF_ESCAPE_AT=N`** — `Perf::ref_escape_at()`, the same shape as the existing
  `Perf::bla_drop_frames()` (`main.rs:1792-1801`): a read-once `OnceLock`, 0 = off. It truncates the
  installed orbit to N samples and marks it **complete** (`partial = false`) — which is exactly the
  field shape, `orbit_len=655 partial=false` against `iter=4627`. One place, one path, no effect when
  the variable is absent.

  **The insertion point is the whole difficulty, and it is not where the obvious reading puts it.**
  Truncating inside `install_recompute` upstream of the install bookkeeping would be seen by
  `install_collapse(old.orbit_len, res.orbit_len, res.partial)` at `render.rs:1569` — which is true
  exactly for short-and-complete, i.e. for the shape being manufactured — and the next lines
  (`render.rs:1575-1580`, read this pass) slam the budget to at most `tdr_bootstrap_steps` with
  `fe_budget_ok = false`. The field state this rung exists to recreate is a budget **frozen at
  1.515e11 with `chunk_over` false**; a bootstrap budget makes `chunk_over` true and the frame is
  chunked, which is the opposite regime. The rung would run, look busy, and test nothing.

  So the instrument sets an explicit `instrument_truncated` flag on the install result, and
  `install_recompute` skips the `big_collapse` derate for a truncation it made itself — the derate
  is a reaction to a *picker* collapse, and the picker did not collapse. The perturbation is then
  one line, declared in the record's header block, and visible to anyone reading the run. It is
  named here rather than left to implementation because "the instrument perturbs the controller it
  measures" is precisely the class §6.6 exists to prevent, and this is the instrument the largest
  workstream rests on.

  It also changes the rung's anti-vacuity predicate: reaching the regime requires the budget to have
  **stayed large**, so `fe_budget` unchanged across the install joins `bla_skip == 0` and
  `ref_len < ask/4` in §7.9's proof-of-entry. Without that term the gate can read green having
  derated itself into safety.
- **`FRACTADYNE_BLA_DROP_FRAMES`, armed outside the mode-switch block.** Today
  `bla_suppress_until[vidx]` is set only after a `prev → m` transition (`render.rs:6577-6589`), so
  `bla_skip = 0` is unreachable at a shallow mode-0 view with no crossover — which is the field
  regime.
- **`FRACTADYNE_EXTRA_GPU_LOAD=N`** — N dummy full-size passes per frame, so that with
  `desired_maximum_frame_latency: 1` the acquire of non-dispatching frames stalls. This manufactures
  the frames `present_throttle_step`'s own doc says a busy queue cannot produce
  (`render.rs:9561-9562`) and, as a bonus, frames slower than 170 ms for the pin-timeout regime
  (U3, U47). It exists because §5.1 *deletes* three vetoes and rewrites the two tests that encode
  their premise: deleting a guard on a mechanism argument is the mirror image of closing a bug on one
  (P15), so the regime must be entered before the guard goes.

**The regime-reached predicate** is read from signals independent of the timing instrument — this is
the whole lesson of U8 and U16. Reached ⟺ `!partial && gpu_iter >= 4 * orbit_len && bla_skip == 0`
**and the frame's budget was not derated by the instrument's own install**, held for at least M
frames, taken from the LIVE manifest fields that exist today (`render.rs:5520-5630`), **not** from
`last_iterate_ms`. It is buildable before `FrameRecord` lands.

Carry the caveat: the short-reference rung deliberately destroys the picture, so it must never feed
a golden or a signature gate, and a *pass* proves only that the app survives the cost shape, not that
the picker produces it.

---

## 7. The gates

Ten workstreams. Each names the classes and findings it closes, one gate with a RED criterion and a
place it measures, cost, exit criteria and risks.

A standing precondition applies to **every new constant** any of them introduces —
the per-adapter chunk-pass fixed term and its clamp, the fixed admission
ceiling, the blind latch's k, `budget_confident`'s k, `wall_budget`'s acceptance
rule, the `FrameRecord` event-trigger set, the screen thresholds: a measured engagement rate of 0,
or a stated healthy distribution, across `--livetest grand-tour`, `--motiontest` and a 20-minute
healthy dive on **both** cards, before the threshold is fixed (P8). `MOTION_UNPRICED_MAX = 2` engaged
1,323 times per autodive inside healthy pipelining; a scorecard that watches for a new latch after
the fact is not enough.

### 7.1 W1 — `FrameRecord`: one always-on structured per-frame record

**Closes.** Classes *crash & report invisibility / recovery*, the diagnosis half of *measurement
never arrives ⇒ bootstrap binds*, *verification-harness defect*. Findings U9, U15, U28, U29, U32.

**Changes.** §6.1 (build identity), §6.3, §6.4; new `crates/fractadyne-app/src/diag/frame_record.rs`
beside `diag/console.rs`; `diag.rs:58-62`'s 24-entry ring replaced by the typed 4096 ring;
`diag::budget_note` (`:539`) retained one release as an adapter then deleted;
`diag::compose_crash_report` (`:601-643`) gains a `frames:` section replacing the `budget :` block;
`diag.rs:667-743` folds `frames.bin`'s tail into the unclean-exit report; `diag::perf_jsonl`
(`:472-490`) loses the `FRACTADYNE_PERF=1` gate for event rows; rotation (`:200-203`) becomes four
slots with a mid-run size check; one emit site at the end of `update()`; new `scripts/framelog.py`.

**Gate.** `--selftest record` (§6.6.1) on the dev GPU, plus a `run:` checklist row executing
`framelog.py --schema-check` against the session's own jsonl. **RED:** record count ≠ frame count; a
gap or duplicate in `frame`; a required field at its default sentinel; a lossy schema round-trip; a
short `frames.bin` after a simulated abrupt exit. *Where:* the app's own artefacts, dev GPU.

**Cost.** ~3 days plus one measurement day for the overhead A/B. **Order:** first; nothing else here
can be validated without it.

**Risks.** Log volume — mitigated by the cadence rule and four-slot rotation. Allocation on the crash
path — the ring is pre-allocated and the OOM reserve already exists (`diag.rs:69-79`). Per-frame
cost — must be measured ABABAB, not argued.

**Exit criteria.** A 14-minute dive produces a `frames.jsonl` under 5 MB; the interleaved ABABAB
overhead measurement is under 0.5% of median frame time, with the control scored; `framelog.py
summarize` on a synthetic device-loss capture names which of U9's five mechanisms fired. **And one
bundle produced on the RX 6800 XT**, through `gpu-validate.ps1`, read end to end by `fieldcheck.py`
to a `verdict.json`, with `frames.bin`'s tail recovered after a deliberate abort **on that machine**.
W2, W3 and W9 all depend on the record, the bundle and the share loop working on the card that
fails; exercising them only on the dev box would make the first field run the first test of the
instrument as well as of the app, which is how a capture comes back unreadable.

### 7.2 W2 — One pricing authority with an un-bypassable wall backstop

**Closes.** *cost-model-blind* (15, open), *measurement never arrives ⇒ bootstrap binds* (7, open),
*composition defect*. Findings U1, U2, U5, U10, U11, U12, U22, U23, U40, U46, U49, U53.

**Changes.** §5.1 in full: M2, M3, B4, B5. `record_mode_rate`'s running minimum
(`main.rs:1637-1638`) gains a kind filter and a decay; `motion_wall_cut`'s LIFT branch
(`main.rs:1756-1765`) is bounded and kind-filtered the way the CUT already is.

**Gates.** Four, because one is deterministic, one injects the variable under test, one is the
regime, and one is the anti-latch invariant that an existing group already knows how to check.

1. `--selftest pricing-props`, extending `render/controller_props.rs`, driven from
   `validation/replays/*.jsonl`. **RED:** on any recorded window of ≥ 8 consecutive frames with
   `last_dt_ms > TDR_BUDGET_MS`, the `admitted_budget` did not decrease within k = 4 frames. This is
   verbatim the assertion the catalogue records as "would have caught CL-1".
   **Anti-vacuity — and this gate needs it more than any other in the plan, because it is the gate
   for the largest open class:** if no file in the corpus contains such a window, the result is
   **exit 2 / VACUOUS**, distinct from both pass and fail. A green here on a corpus of healthy
   replays is the precise thing this document exists to abolish. The fixture that must contain one
   is named in the corpus manifest — initially `validation/replays/blind-queue-*.jsonl`, produced by
   gate 3's `EXTRA_GPU_LOAD` rung, and replaced by the first field capture carrying the CL-1
   signature as soon as one exists. *Where:* deterministic replay, CPU, CI.
2. `--selftest budget-confidence`, scoped to the case that is actually uncovered. The two existing
   checks are **not** both "a correct budget": `selftest.rs:3211-3222` injects `fe_budget = 0` (set
   at `:3159`) and asserts the first dispatch stays within `tdr_bootstrap_steps` — that is the
   *never-measured* axis, and it is close to what this gate would otherwise duplicate — while
   `selftest.rs:3556-3575` injects a converged `FIELD_BUDGET` (`:3526`) and grades sizing given a
   large budget. Neither touches a budget that was **learned large and has gone stale**: measurement
   arrived, then stopped. That is the 2026-09-21 shape and it is this gate's subject. Drive
   `build_params` with a converged budget and then no paired reading for k frames, and assert the
   frame was chunked and its dispatch ≤ bootstrap. **RED:** a frame goes out un-chunked, un-tiled
   and un-shrunk with `budget_confident == false`. Gate 1 asserts the budget decreased; this asserts
   the actuator **fired**, which is the other half and the one P4 is about. *Where:* dev box,
   offline model.
3. `--torture blind-queue` on the **RX 6800 XT through the share**. **RED:** a device loss; any
   8-frame wall-slow window with an unchanged admitted budget; a `NoActuatorReachable` outcome.
   **Anti-vacuity:** RED, not skip, unless the run recorded at least one frame with
   `last_dt_ms > 500` — otherwise it did not reach the regime and must say so distinctly (exit 3).
   *Where:* field hardware.
4. `--selftest live-res`, extended to the admission path. `admit()` is a `min()` over sources, and
   `model.min(measured)` is the arithmetic that ran a whole 1× → 1e100 dive at 30% linear resolution
   with the GPU idle at 17.8 ms/frame (CL-28 / MEM-49). The gate written for that incident already
   exists and already pins **both halves** of the invariant — bounded on the first dispatch, and
   reaching native resolution once measured. Extending it to cover `admitted_budget` costs a day and
   turns the plan's most obvious new risk from a watched scorecard line into a red-capable check.
   A watched metric is not a gate; P6 says so, and this document should not exempt itself.

**Cost.** ~6 days, most of it in M2/M3 review rather than new logic. **Order:** second; needs W1's
replay corpus and its metrics.

**Risks.** A new latch on the admission path — gated by gate 4 above and additionally watched by
"fraction of settled frames admitted below native resolution", which is how MEM-49's latch was
eventually seen. Removing the present-throttle
veto re-exposes the 2026-09-04 compositor case (CL-58/MEM-65) — mitigated because `wall_budget`
learns from `last_dt_ms` only on frames that **dispatched**, and an occluded window at ~1 Hz presents
does not dispatch; the discriminator becomes "did anything get submitted recently", not "did this
frame dispatch".

**Exit criteria.** The next field capture carrying the CL-1 signature — an episode of wall-slow
frames with a budget that does not move — replays to a **decreasing** admitted budget. Note what
this cannot be: the 2026-09-21 session itself is not replayable, because the user's input stream is
not logged today and beta.112's 24-entry ring holds ≈ 1.4 s of a 33 s episode. Only a beta.113+
capture can serve, which is why §8 makes W2's shipment conditional on one arriving. Plus:
`--torture blind-queue` green on the Radeon three runs running; zero `NoActuatorReachable` in a
20-minute dive; engagement rate 0 for the blind latch on all three healthy runs per card.

### 7.3 W3 — A price with a constant term, and a retreat that must reach an actuator

**Closes.** *actuator & retreat blind spots* (4, open), *cost-model-blind*, the residual of
*unbounded or oversized dispatch*. Findings U3, U4, U17, U47, U48, and U38 **in part only**.

It does **not** close *main-thread blocking & unbounded aggregate* (5, open), tempting as it is to
count it here. Routing the steering probe through `admit()` accounts for a
dispatch that is already bounded to ≤ 128×128 at ss = 1 (`autopilot.rs:658-664`); the probe is not
the load. The class's red-capable half comes from W1's watchdog heartbeat row and W8's `--logcheck`
rules (a grind and a wedge both become positive artefacts with a duration); its aggregate half —
concurrent export plus live path — has no controlled reproduction and is a named non-goal (§8, §10).

**Changes.** §5.2 and §5.6, including the per-adapter calibration file, the clamped learned
intercept, and the band ledger keyed per target and lane (U48).

**Gates.**

1. `--chunk-sweep` promoted from a report to a gate, and made the producer of the per-adapter
   calibration file `validation/calibration/<adapter-slug>.json`. `chunksweep.rs` contains **no
   `exit(` call at all** today; it gains one. **RED:** the measured per-pass fixed term differs from
   the calibrated value for this adapter by more than 2×, or the session-learned intercept hits its
   [0.25×, 4×] clamp, or a dimension-parity cliff above 10× reappears on any swept axis.
   **Anti-vacuity (P7):** the run first asserts that chunking
   actually happened (`TRACE=tile` `chunks=`, or the record's `chunk_needed`), because four A/Bs on
   chunking once returned a clean, confident, meaningless zero.
2. `--selftest retreat`: inject a lethal reading with every actuator at its floor and assert the
   outcome is `NoActuatorReachable` **and** that a log line exists. **RED** if a retreat reports
   success having changed nothing.
3. `--deviceloss-repro` gains a criterion, not just an exit code. It already computes a floor-window
   cost extrapolation. **RED:** the extrapolated single-dispatch cost exceeds `tdr_lethal_ms` while
   the live path would **not** have chunked. Run on both cards, from the field `.fdn`. An exit code
   without a criterion is the vacuous gate this plan exists to abolish.

*Where:* dev GPU and the Radeon via `scripts/radeon-verify.ps1`.

**Cost.** ~5 days. **Order:** third; B5 must land first or the retreat has nothing to reach.

**Risks.** A fixed term seeded too high makes every pass look expensive and collapses the walk —
hence "learned per session as an intercept", and hence "median settled walk passes per second" on
the scorecard.

**Exit criteria.** A shed to the window floor measurably reduces wall time per pass on the Radeon
(today it does not: 243 sheds at size=2 over 700 s — memory note, **not verified**);
`NoActuatorReachable` appears in the record when and only when it is true.

### 7.4 W4 — Frame-kind state machine and one owner of the walk signature

**Closes.** *coarse or unsatisfiable identity key*, *presentation ≠ render*, the walk-restart half of
*iteration starvation*. Findings U24, U39, U51, U56. (U20 is W6's, with the rest of the
content-guard presenters.)

**Changes.** §5.3: M6, B7, including the three `hold_scale_floor` call sites at `render.rs:7579`,
`7745` and `7776`.

**Gate.** Two terms, and the order of authority between them is the point of this workstream's
history.

1. **The screen, median of three pairs**, through `scripts/dive-capture/`: blank and `stale`
   fractions, control arm scored. This is a **RED term of W4's own gate**, not a later
   workstream's. The three failed attempts that produced 128–216 restarts were found by capturing
   the screen, and they are on record as *improving every per-pass metric while the screen went 8%
   → 36% blank* (P11, MEM-60). A workstream whose verdict is a per-pass counter would have passed
   all three.
2. `--motiontest` gains **A4: walk restarts per dive**, computed from the record's `walk_restarted`
   field rather than from a trace. **RED:** more than 12 restarts per dive — the measured
   post-`112a088` figure is 7–10. A4 is the *explanatory* metric: it says why the screen moved, and
   it must never be the verdict on its own.

Plus a compile-time gate: `Plan::res` being private means a new resolution writer outside the module
fails to build; and a `--selftest walk-sig` property test asserts that two plans differing in
resolution never share a sig.

**Which card.** The blank thresholds below are dev-box numbers, and P12 is explicit that these
replays are milder than the user's sessions — same seed, same location, their motion budget 6×
smaller. So one arm of the screen measurement runs on the **RX 6800 XT** through `gpu-validate.ps1`
(`radeon-verify.ps1 -Phase 5` already needs a real desktop there, so the capture path exists), and
where that is not available for a given commit the dev-box numbers are stated for what they are: a
bound on *regression*, not evidence about the reported regime.

**Cost.** ~5 days, of which M6 is the bulk. **Order:** fourth; independent of W2/W3 but wanted before
W5, since `Plan` is `decide`'s output type.

**Risks.** M6 touches the hottest function in the tree; mitigated by being byte-neutral under the
corpus and by `--selftest live-res`, which already pins the sizing arithmetic.

**Exit criteria.** Zero resolution writers outside `render/frame_plan.rs`, stated as a falsifiable
grep rather than a slogan: `rg -n 'res_scale\s*=|visible_res\[[^]]*\]\s*=|motion_res\s*=' crates/`
must return hits **only** inside `render/frame_plan.rs`. Today that pattern finds at least four
writing sites — `let res_scale` at `render.rs:6337` (pin) and `:6408` (motion),
`self.perf.visible_res[vbi] = held` at `:6436`, and `Perf::motion_res` at `main.rs:1547`. Plus: the
three-pair screen measurement green on both arms, with the control scored; `--motiontest` A4 green
over three runs; `--zoomtest`'s held-frame magnification measured on the **dive** branch for a wheel
gesture, which the harness cannot drive today.

### 7.5 W5 — `decide(inputs) -> plan`, and replay as a first-class test input

**Closes.** *verification-harness defect*. Findings U32, U35, U52, and U9's discrimination — a replay
makes the five mechanisms separable off-line.

**Changes.** §5.4: M8, B9; a `--replay <jsonl>` CLI mode; `validation/replays/` seeded with a
scripted dive, a `--motiontest` run, a `--zoomtest` run and — as soon as one exists — a post-beta.112
field capture.

**Gate.** `--selftest replay` over `validation/replays/*.jsonl`. **RED:** the recomputed `Plan`
differs from the recorded `Plan` on any frame in any field, or any replay file's declared invariants
are violated, or the per-function coverage bitmap shows any controller with zero replayed decisions
across the whole fixture set. **Anti-vacuity — three terms, not one:** exit 2 if the directory is
empty; if any file holds fewer than 200 frames; **or if no fixture in the set satisfies §6.7's
regime-reached predicate**, and — once one exists — if the set contains no field capture at all. The
seed corpus is a scripted dive, a `--motiontest` run and a `--zoomtest` run: the maintainer's own
replays, at the one location, mode and auto-iter setting U33 indicts, and P12 says they are milder
than the user's sessions by a factor of six. A corpus that can be satisfied entirely by mild runs
grades the controller only where it already works. The regime predicate is buildable before
`FrameRecord` lands (§6.7), so this term costs nothing to state now. *Where:* CPU only,
deterministic, no GPU — the first live-path gate in this project that can run in CI.

**Companion red-check.** Flip `PRICE_REPRESENTATIVE_FRAC` (`render.rs:9299`), or hard-code the
duplicated literal at `render.rs:9409` to 0.5; the fixture test must go red on a real field trace.
Today `render/motion_jam.rs:4-11` asserts the boundary with a literal 70/100 while
`render/probe_price_tests.rs:25-30` derives it from the constant, so the two definitions can split
with every test green. Two clauses travel with this. `PRICE_REPRESENTATIVE_FRAC` is a plain `const`,
not a tunable, so the flip means edit-and-rebuild, not `--set` — unlike W8's `--set TDR_LETHAL_MS=1`
recipe, which reads like the same mechanism and is not. And its doc says the **value** is beta.11
determinism-critical and must not move without that lens: so the sabotage arm is a scratch build
that never blesses a golden, never feeds a baseline, and never runs inside a release gate. It exists
to prove the fixture can go red, and its binary is thrown away.

**Cost.** ~7 days. **Order:** fifth. **Risks.** M8 is the highest-risk refactor in the plan;
mitigated by stage-at-a-time moves, corpus byte-identity per stage, and leaving the reference
lifecycle outside the pure boundary.

**Exit criteria.** Every harness's controller is the app's; `grep -c SETTLE_DELAY` finds one
definition; a recorded session replays to a bit-identical plan stream.

### 7.6 W6 — Provenance, content, and the screen

**Closes.** *stale or misattributed measurement*, *presentation ≠ render* (10, open), the hold half
of *normalization source & wrong statistic*. Findings U14, U19, U20, U42, U43, U45, U54, U57, U58,
U59.

**Changes.** §5.5.

**Gates.** Two, one of them on the screen.

1. `--selftest provenance`: over a 300-frame scripted dive, **RED** if the provenance-mismatch
   counter is non-zero, or any frame displayed a hold with `hold_verified == false`, or any reading
   was applied to a consumer whose `read_prov_frame` differs from the frame it steered, **or the
   discard rate exceeds its bound** — readings dropped for a provenance or staleness mismatch, as a
   fraction of readings published. That last term is U54 and P13: a rule that silently discards
   every reading produces no mismatches at all, so a gate counting only applied readings would be
   greenest exactly when nothing is being measured. Additionally, `--uitest`'s deep "frame not
   blank" check (`uitest.rs:1383-1394`, `deep_ambiguous` for `RenderMode::Floatexp`) is promoted
   from WARN to FAIL once a per-adapter reference screenshot exists (§7.8).
2. **Screen-level**, wired into `scripts/release_checklist.py`, which today contains no
   `dive-capture` row at all. The scorers are `blank`, `flash`, `flat` and a new one the Python set
   lacks: **`stale`** = a capture pixel-identical to the previous one while the *recorded view
   moved* — computable only because the record carries frame index and view span. `stale` is the
   signature of the 50-octave smear (MEM-50) and of a frozen pin: the exact failures U39 and U44
   describe, which a blank score cannot see because a smear is not blank. **RED:** median-of-three
   blank fraction above 7% (the measured post-`112a088` band is 1–7%; single runs vary 8–14%, hence
   three pairs), or any of flash/flat/stale above its threshold, or a three-run spread wider than
   the calibration. *Where:* the screen, via PrintWindow by PID, **with one arm on the RX 6800 XT**
   through `gpu-validate.ps1` — the failures this gate grades (MEM-60's 36% blank, MEM-61's
   flat-colour pan) were reported from that machine, and a blank-fraction threshold blessed only on
   the dev box is a threshold calibrated in the mild regime (P12). Where a commit cannot get a
   Radeon arm, the dev-box numbers bound regression only and the run says so.

   The repro trap is part of the gate: the run's own log must carry the bignum-backend line the
   fixture declares. Copying only `fractadyne.exe` out of the accelerated dist drops the MPFR DLLs
   and scored the first beta.110 baseline at 3% blank against the user's 35%.

**Cost.** ~6 days. **Order:** sixth, and the dependency on W4 is **hard, not soft**: `Provenance`
carries `FrameKind` and `WalkSig`, which do not exist until §5.3 creates them. The half that does
not wait is already elsewhere — `ReadingProvenance` (frame, view, `pass_steps`, res, ss) lands with
W2's B4, because `budget_confident` is defined against it. So W6 is the *widening*, and there is no
interim in which `Provenance.kind` holds a placeholder. §5.5 and §8 state the same order.

**Risks.** `CONTENT_MIN_ESCAPED` (U57) — see §5.5. Mitigation is a six-view recalibration in the
middle of the range.

**Exit criteria.** Zero provenance mismatches over a 20-minute dive; blank fraction under 7% over
three pairs; `hold_verified` has a consumer that is not a `format!`.

### 7.7 W7 — Typed capabilities and a settings layer that cannot persist a lethal configuration

**Closes.** *actuator & retreat blind spots*, *crash & report invisibility / recovery*. Findings U18
and U1's capability latch.

**Changes.** §5.6 and §5.7.

**Gate.** `relaunch_policy.rs` extended — it has four tests today, all about policy, none about
survivability. **RED:** given a recorded `SessionOutcome` whose admitted price exceeded the wall
budget at the loss, the restored configuration's admitted price still exceeds it. Plus
`--selftest settings-quarantine`: **RED** if a quarantine occurs without a user-visible notice.

**Cost.** ~3 days. **Order:** seventh; needs W2's `admitted_budget`.

**Risks.** The only user-visible behaviour change in the plan; a silent quarantine is worse than
none, which is why the notice is in the RED criterion.

**Exit criteria.** A relaunch after a recorded loss opens at a configuration whose admitted price is
below the wall budget at which the previous session died, and says what it changed.

### 7.8 W8 — Turn the report-only harnesses into gates that can go red

**Closes.** *verification-harness defect / gate blind spot* (16, open) and, with W1–W3 and W6, the
five open classes with no check that can go red. Findings U8, U16, U21, U26 (its detect half only),
U29, U30, U31, U34, U35, U36, U44.

**Changes.**

- `--autodive`'s verdict rebuilt on the **record** — wall-slow frames, retreat outcomes,
  `NoActuatorReachable` — instead of `perf.last_iterate_ms[0]` (`autopilot.rs:1344-1360`, exit at
  `:1511-1512`). Three distinct codes replace the current two: **0** reached and survived, **2**
  reached and the budget did not move (a real RED), **3** not reached (vacuous). Fix the in-tree doc
  drift while there: `torture.rs:554`'s rung motivation tells the operator the flag "exits 3 if no
  lethal reading occurs", while the code exits 2 and `classify` maps 2 to `FailAssert`
  (`torture.rs:647-663`).
- **`--motiontest` gets the same three codes**, for the same reason. It already computes a VACUOUS
  term — "only N interacting chunk-eligible frames … the regime was not reached"
  (`motiontest.rs:226-229`) — and then discards the distinction, because `motiontest_verdict` exits
  `if pass {0} else {2}` (`:271`), the identical code a real A1/A2/A3 assertion failure produces. A
  harness that can tell "did not run the experiment" from "ran it and failed" and then throws the
  answer away is U8's lesson with the evidence already in hand.
- Exit codes and criteria for `--zoomtest` (`design/live-zoom-smoothing.md:215-228` already specifies
  the matrix: max ≤ 50 ms, frames > 100 ms = 0, held max ≤ 1 octave), `--chunk-sweep`,
  `--deviceloss-repro`. `--reusetest`, `--divetest`, `--frametest`, `--juliadive` and `--dualsettle`
  get either a criterion or an explicit REPORT-ONLY banner and are removed from any checklist row
  implying a gate. Note the trap: adding `crate::exit` puts a harness on the choke point that
  disarms `session.running`, which is correct and must be checked.
- `--motiontest` asserts `blank_walks_total` (computed and printed at `motiontest.rs:220-223`, with
  no matching `fails.push`) and the held-magnification maximum.
- `--livetest` gains a second oracle at 4× the ask, so an iteration-starved view is no longer black
  on both sides (U34); `capped_frac` and `iter_exhausted` enter the verdict from the record
  (today `orbit_len` and `PARTIAL` are context only, `livetest.rs:769-778`). This half is cheaper
  than it looks: both fields are already live app state with real consumers — `Perf::view_resolved`
  (`render.rs:3773`, `3779`) and `limit_status` (`ui/menus.rs:1404`, `1409`), the latter red-gated by
  `limit_status_matches_measured_regimes` (`:1819`) — so the work is wiring a verdict to a
  maintained measurement, not building one.
- **Checklist enforcers.** Add `planned:` rows for torture increment 2, the `--zoomtest` P-G matrix
  and TODO T-6, so `checklist_coverage.py` reports non-zero OUTSTANDING — the class is fully
  implemented (`checklist_coverage.py:10-12`, `171-172`, `233-235`) and the file contains none. Add a
  `run:<cmd>` class requiring a recorded exit code **from this release's binary**, and move the
  device-loss row onto it. Precisely: the Deep-zoom area has eight enforcers
  (`release_checklist.py:686-693`), six of them `selftest:` rows that do resolve to real checks;
  the claim here is scoped to the **device-loss regime**, which is enforced by exactly one of them,
  `("Deep zoom", "harness:--autodive-home")` at `:693` — and `harness:<flag>` means only that a flag
  exists in the argument parser (`release_checklist.py:657-661`). Add a `dive-capture` row while
  there; the file contains none today.
- `--selftest` prints which `opt_in` groups did **not** run, and a checklist row requires each to have
  run green once per release.
- `checklist_coverage.py` asserts `validation/golden/BLESSED-GPU.txt` against the declared reference
  adapter, and `--selftest --bless` requires a new `--bless-new-adapter` to change the adapter
  (`selftest.rs:6051`).
- **Per-adapter baselines**, so something can go red on the card that fails: `validation/golden/<adapter-slug>/`
  and `benchmarks/bench-matrix-baseline-<adapter-slug>.json`; delete `|| !same_gpu` at
  `bench_matrix.rs:732` and make "no baseline for this adapter" a distinct, loud, non-passing
  outcome; per-adapter tolerance selection at `selftest.rs:5969-5975`; and
  `scripts/gpu-validate-expected.json` declaring each step's expected exit set per adapter, so
  `gpu-validate.ps1` can aggregate instead of always exiting 0 with prose explaining away the reds.
- **A red-capable check for the rebase counter's vendor dependence (U26's detect half).**
  `--gputest` already grades op families per backend; the rebase and BLA-skip counter arithmetic
  joins that set, so "this counter does not mean the same thing on this backend" becomes a reading
  rather than a standing caveat. That is what makes "detect, do not fix" an honest disposition
  instead of an unassigned finding — especially now that §5.2 keeps the counter out of `admit()`.
- `--logcheck` and `validation/logcheck-rules.toml` (§6.5), called by every in-process harness
  against its own log **and runnable out-of-process** over a log a killed or wedged run left behind
  (P19, P20); "no verdict line from harness X" is a distinct non-zero outcome in `fieldcheck.py`.

**Gate.** A meta-gate, because this workstream's product *is* gates: `scripts/gate_selfcheck.ps1`
runs each harness named in the checklist against a deliberate sabotage — a `--set` override known to
break it, or a corrupted golden — and is **RED if any of them still exits 0**. One trap to avoid
inside it: `--selftest` does not *refuse* to run under an override; it emits a failing check,
"tunables are stock (no --set overrides)" (`selftest.rs:3253-3260`), inside the `live-res` group. So
a `--set` sabotage of `--selftest` goes red on the stock check rather than on the thing being
sabotaged, and the meta-gate passes for the wrong reason. `--selftest`'s sabotage arm is therefore a
corrupted golden or a reachability fixture, never a `--set`. Beneath it sits the per-assertion
reachability fixture set of §6.6.4, which is the stronger claim.

**Red-check.** `--set TDR_LETHAL_MS=1` — an existing recipe that produced 46 sheds under `--livetest`
— must turn a `--livetest` run RED through `--logcheck`, not merely noisy.

**Cost.** ~6 days spread across the other workstreams; each new gate lands with the change it grades.
**Order:** continuous; the `--autodive` verdict rebuild specifically after W1.

**Risks.** Turning informational reds into failures will surface pre-existing failures — `--uitest`
step 30 was red for weeks, including a shipped release, normalised as "the known pre-existing
failure". Budget for that: the first `gate_selfcheck` run is expected to find several, and each must
be **filed** rather than re-normalised.

**Exit criteria.** Every harness in the checklist returns non-zero on its sabotage arm; every
assertion has a reachability fixture that has been observed red; `--autodive` distinguishes pass /
fail / did-not-reach; `checklist_coverage.py` reports a non-zero OUTSTANDING count.

### 7.9 W9 — Reach the regime that has killed every device, on the card that fails

**Closes.** *reference lifecycle*, *verification-harness defect* (G2), *cost-model-blind*. Findings
U6, U7, U13, U25, U33, U41, U50, U55.

**Changes.** §6.7's three instruments and the regime-reached predicate — including the
`instrument_truncated` flag that keeps `install_recompute`'s collapse derate from neutering
`REF_ESCAPE_AT`; §5.8's fitness rule behind its tunable; new `--torture` rungs
`escaped-ref-shallow` (the field geometry from the share's crash view — 1676×1295, ss=2, mode 0,
~1.76e6×, **auto-iter ON** via `--autodive-iter 0`, which exists at `main.rs:5929-5932` and
`help.rs:643` and which no gate passes today), `auto-iter-deep`, `crossover-hold` (G1),
`zoom-out-crossover` (G6), `queue-sat` (`EXTRA_GPU_LOAD`), `motion-mode1` (CL-165's unreproduced
`--motiontest` loss, with a regime assertion rather than a repro), `click-jump-deep`.
`torture/tests.rs` already asserts ladder shape (unique ids, prerequisites earlier, skip ≠ pass) and
is extended for the new rungs.

**Why auto-iter ON, against the measurement that set the default.** `--autodive`'s help text records
the reason it defaults to an explicit count: *"the live path caps auto-iter lower for
responsiveness, so auto-iter frames are too cheap to reach the lethal band (measured: 19.5 ms peak
over 900 s)"* (`help.rs:643`, read this pass). That measurement stands, and it is exactly why this
rung pairs auto-iter with `REF_ESCAPE_AT`: what makes an auto-iter frame cheap is a *fit* reference,
where the nominal step count is the cost. With a short escaped reference the same nominal count
costs 10–70× more per step, which is the 2026-09-21 shape — a shallow view, auto-iter on, a budget
sized for a per-step rate the frame no longer runs at. The rung tests the cost shape the field
produced, not the one the explicit-count default manufactures. If it still peaks at tens of
milliseconds with the instrument armed, that is a finding about the instrument, and it must be
reported as vacuous rather than as a pass.

**The field loop, concretely.** The largest item here is also the one with the most round-trip
latency, so the loop is named rather than assumed:

```
dev box              cargo build -j1  →  scripts\publish-share.ps1
                        ↓
share  D:\share\Fractadyne\builds\<tag>\   + BUILD-ID.txt + TEST-PLAN.md
                        ↓
RX 6800 XT box       one command: gpu-validate.ps1 -Label rx6800xt
                     (installs the accelerated zip, hermetic config dir)
                     radeon-verify.ps1 -Phase 5      (needs a real desktop)
                     python fieldcheck.py <bundle>
                        ↓
share  results\<tag>\rx6800xt-<utc>\{verdict.json, summary.txt, app.log, crash\}
                        ↓
dev box              reads one exit code, not a 1 MB log
```

Turnaround target: publish before the user's evening, verdict on the share by the next morning.
Nothing in this loop needs CI (`ci.yml` is `workflow_dispatch` and has no GPU) and nothing needs the
dev box to reach the Radeon box. The later automation is TODO T-6: `scripts/job-watcher.ps1` running
in the **logged-on desktop session** — remoting alone usually cannot create a swapchain, or lands on
a software adapter and gives plausible timings that describe nothing. Job files carry **parameters,
not commands**, the watcher invokes only a whitelisted script, and anything unrecognised is a
**refusal**, never a skip.

**Gate.** `--torture escaped-ref-shallow` on the **RX 6800 XT through the share**. **RED:** a device
loss; ≥ 8 consecutive wall-slow frames with no retreat outcome; or a recorded `rebase / sampled_px`
above 4 while `chunk_needed == false`. **Anti-vacuity, and this is the crux:** RED — not skip, not
pass — unless the run recorded at least one frame with `bla_skip == 0`, `ref_len < ask/4`, **and a
budget the instrument's own install did not derate**. That third term is not decoration: without it
the rung can satisfy its entry test while sitting at a bootstrap budget (§6.7), which is the
chunked, safe regime and precisely not the one under test. All three together are the proof the
regime was entered. `--torture`'s own ladder header states that a green run does
**not** cover this regime (`torture.rs:211-217`); after this workstream it does, or it goes red
saying it did not.

**Cost.** ~8 days, the largest single item. **Order:** after W1–W3, but its harness rungs can be
built in parallel from day one, since they are composed of flags that already exist.

**Risks.** U6's fix turns an existing test red by design (`reuse_tests.rs:82-95`) — rewrite, do not
delete, and watch the test count. The instrument perturbs the thing it measures: a truncated orbit is
not a naturally escaped one, so a pass proves the app survives the cost shape, not that the picker
produces it. That is acceptable — the cost shape is what killed the card — but the rung must never
feed a golden.

**Exit criteria.** A run that provably entered `bla_skip == 0` with a short escaped reference and
survived, three runs running, on the card that has lost the device twice; and the fitness tunable's
flip conditions met (rebuild rate on the grand tour within 10% of beta.112, goldens byte-identical,
`--bench-matrix` 0 algorithmic drift).

### 7.10 W10 — Log hygiene, so the record is findable

**Closes.** The field-observability fragilities. Supports U29, U32.

**Changes.** §6.4's hygiene paragraph.

**Gate.** `--selftest log-budget`: over a 300-frame scripted dive, **RED** if any always-on category
exceeds N lines per second, or if any `[fd-*]` prefix present in the log is absent from
`DIAGNOSTICS.md`'s tables. That second clause is a documentation check that can genuinely go red.

**Cost.** ~2 days. **Order:** alongside W1.

**Risks.** Minimal; the only trap is that an edge-triggered `begin` could hide a genuinely flapping
predicate, so the record carries `accum_transitions` as a count.

**Exit criteria.** A 14-minute session's log under 300 KB with the accum share under 5%; no
undocumented prefix.

### 7.11 Class × gate coverage, before and after

The headline claim of this document is that the **five of the nine** open classes with no
red-capable check go to zero — with two of those five reaching only *partial* coverage, named in the
rows below and in §10, because their residual halves have no controlled reproduction. (Nine, not
eight: §2.1 bolds nine open classes, the catalogue bolds the same nine, and
`findings-verifiability.md`'s summary paragraph miscounts against its own table.) The claim is only
demonstrable in this shape.

| class (count, state) | red-capable today? | the gate that will exist | W |
|---|---|---|---|
| **cost-model-blind** (15, open) | **none** | `--selftest pricing-props` (budget must decrease within k of 8 wall-slow frames); `--torture blind-queue` / `escaped-ref-shallow` on the Radeon; `--chunk-sweep` fixed-term criterion | W2, W3, W9 |
| **measurement never arrives ⇒ bootstrap binds** (7, open) | partial — `live-res` drives the app's own `build_params` at `fe_budget = 0` (`selftest.rs:3211-3222`), so the *never-measured* axis is covered; the **learned-then-stale** axis is not | `--selftest budget-confidence` scoped to learned-then-stale (asserts the actuator fired); `--logcheck` bound on starve/resume pairs | W2, W8 |
| **iteration starvation** (11, open) | **none** | `--livetest` second oracle at 4× the ask + `capped_frac` / `iter_exhausted` FAIL terms; the screen gate's flat fraction | W8, W6 |
| **presentation ≠ render** (10, open) | partial, one regime | the screen gate's blank / flash / flat / **stale**, median-of-three; `--zoomtest --gate` held-max; `--motiontest` blank-walk term | W6, W8 |
| **main-thread blocking & aggregate** (5, open) | effectively none | → **partial**: the watchdog thread's heartbeat row makes a *wedge* a positive record with a duration (the UI thread is the record's only other writer, so a grind is visible and a wedge would otherwise be a gap); `--logcheck` on `[fd-watch] possible hang`, out-of-process; the probe accounted for in `admit()`. The **aggregate** half (MEM-37, concurrent export) stays uncovered — no controlled reproduction, named non-goal | W1, W8, W3 |
| **verification-harness defect** (16, open) | partial | `gate_selfcheck.ps1` sabotage arm; per-assertion reachability fixtures; `--logcheck` required-line rules; `planned:` and `run:` enforcer classes | W8 |
| **driver / GPU fast path / upstream** (5, open) | partial, unusable where it matters | per-adapter goldens and bench-matrix baselines; `gpu-validate` aggregate exit against a declared expected-exit set | W8 |
| **actuator & retreat blind spots** (4, open) | **none at system level** | `--selftest retreat` (`NoActuatorReachable` + a log line); `--deviceloss-repro` criterion; the record's `retreat_outcome` asserted by every regime rung | W3, W1 |
| **open / unexplained** (2, open) | **none** | → **partial**: the `motion-mode1` rung (CL-165) with a regime assertion, so a run that cannot enter mode-1 motion says so instead of passing. Issue #3 (dual view) gets **nothing** — unreproduced in 12 trials, deliberately not scheduled (§8), and the honest state stays OPEN-AND-UNREPRODUCED | W9 |
| stale / misattributed measurement (7) | partial | `--selftest provenance`; replay fixtures pin the pairing | W6, W5 |
| unbounded / oversized dispatch (9) | yes (offline) | unchanged, plus "assert chunking happened" on every chunking A/B | W3 |
| guard scope / capability gating (9) | partial | unchanged, plus `chunk_ok` / `chunk_fe_ok` recorded, logged and asserted per adapter | W7, W8 |
| coarse / unsatisfiable identity key (6) | partial | `Plan::res` private (a compile-time gate); `--selftest walk-sig`; replay pins the sig composition per frame | W4, W5 |
| normalization source & wrong statistic (13) | rules yes, engagement no | the record carries `norm_lo/hi/complete/shown` per frame; engagement asserted on a real dive | W1, W6 |
| reference lifecycle (16) | partial | `--selftest ref-fitness`; `escaped-ref-shallow` asserts `ref_len < ask/4` was entered | W9 |
| UI reflow → render (3) | yes | unchanged, plus `--logcheck` on `panel resize` | W8 |
| numeric representation limit (8) | yes on the dev box, **no** where it matters — `--gputest` found CL-108 and is red on every NVIDIA backend by construction, and CL-108 itself is open behind a workaround | per-adapter expected-exit sets, so `--gputest`'s known red becomes a *declared* red and a new one is a failure; the rebase/BLA-skip counter arithmetic joins its op families (U26) | W8 |
| perturbation & formula correctness (5) | yes (offline), depth ceiling | unchanged; the ceiling is a named non-goal | — |
| steering & search logic (9) | yes (pure) | `choose_goal` joins the replay set, so a field dive becomes a steering fixture | W5 |
| retracted / refuted diagnosis (6) | n/a | n/a | — |
| composition defect (3) | partial | replay drives the composition (`present_throttle_step` → `wall_shed_now` → `measurement_starved`) from one recorded stream, which unit tests in isolation cannot see | W5 |
| path divergence (3) | **none** | `--logcheck` required-line rules make a mover that skips the common finisher leave a missing line | W8 |
| crash & report invisibility / recovery (5) | partial | `--selftest record` (`frames.bin` survives an abrupt exit); `relaunch_policy.rs` survivability test | W1, W7 |
| silent default / unread value (2) | yes | unchanged, plus the job schema's refuse-unknown-keys rule | W9 |
| heuristic tuned in one direction / at one depth (3) | partial | the screen gate's three depth fixtures plus a shallow control — calibrate in the middle, not at the extremes | W6 |
| file, format & session contract (4) | yes | unchanged, plus the record's schema version and strict reader | W1 |
| colouring quality & mapping (4) | partial | the screen gate's flat and flash fractions on a real dive | W6 |
| miscellaneous (11) | partial | unchanged | — |

---

## 8. Sequencing

Three betas with a designed failure mode, then the rest. Beta boundaries matter here because the
ladder is **about 52 working days** for a solo maintainer with `-j1` builds — the ten workstreams
sum to 51 (3 + 6 + 5 + 5 + 7 + 6 + 3 + 6 + 8 + 2) plus W1's measurement day — two to three months
part-time, and that is the plan's weakest axis. Release boundaries make "abandoned half-way" a
named, harmless, already-shipped state rather than a risk to argue about.

### beta.113 — evidence only, no controller reaction

| item | from |
|---|---|
| Build identity (`FRACT_GIT` with its `rerun-if-changed` directives, `BUILD-ID.txt`, `gpu-validate` step 00, the `tests.rs:599-612` presence assertion, the `-dirty` refusal in `publish-share.ps1`) | §6.1 |
| `FrameRecord` M0 + B1: the ring, `frames.bin`, `frames.jsonl`, the crash-report `frames:` section, `scripts/framelog.py` | W1 |
| Log hygiene: edge-triggered accum, four-slot rotation, `DIAGNOSTICS.md` tables, thread names | W10 |
| The blind-tripwire reset fix (decrease only) and per-view counters — report-only, no actuator | W2 (partial) |
| The regime instruments (`REF_ESCAPE_AT`, `BLA_DROP_FRAMES` armed outside the switch, `EXTRA_GPU_LOAD`) and the `escaped-ref-shallow` rung, on stock beta.112 controller logic | W9 (partial) |
| `--logcheck` and `validation/logcheck-rules.toml`; the `planned:` and `run:` checklist rows | W8 (partial) |
| The field loop: `publish-share` → one command on the Radeon → `fieldcheck.py` → `verdict.json` | W9 |

Every item is red-gateable on the dev box alone if the field run slips. Nothing here changes what the
app *does*.

### beta.114 — the reaction, if and only if the evidence arrived

W2 in full (M2, M3, B4 with the `Admission` token, the fixed ceiling and `ReadingProvenance`, B5
with its P18 audit and screen measurement, `budget_confident`, the wall dead-man); W3 (the constant
term with its per-adapter calibration file, the retreat outcome, the pin-timeout demotion, the probe
routed through `admit()`); the screen gate with its thresholds and three-pair discipline; §5.8's
reference fitness **behind a tunable, default off**. Re-run `escaped-ref-shallow` on the Radeon:
survival on stock-new where stock-old produced the signature is the proof.

**The proof standard for that one comparison, stated, because §9's standing rule cannot apply.**
Three pairs minimum with the control scored is right for every metric on the scorecard, and it is
the wrong instrument here: three "before" arms means three deliberate device losses on the user's
card. So a device-loss survival test is scored as **one before-arm and three after-arms** — the
stock-old signature reproduced once (which is what W9's anti-vacuity already proves), then three
stock-new runs that provably entered the regime and survived. The asymmetry is deliberate and its
cost is stated: a single before-arm cannot distinguish a fix from a flaky reproduction, which is why
the *entry* proof, not the survival, carries the weight.

**The designed failure mode.** If the beta.113 evidence did not arrive — the rung never reached the
regime, or the record is inconclusive — beta.114 ships the screen gate and the verifiability work
only, and the pricing reaction waits. That is a planned state, not a slip, and it is how P16 becomes
a schedule rather than a promise.

### beta.115 — close the loop

W4 (M6, B7); W6 (the `Provenance` widening, the settled compose branch, the coarse-preview gate,
`hold_verified`'s first real consumer); W7 (actuators, the settings quarantine); W5's `--selftest
replay` with **the first field capture carrying the CL-1 signature** checked in — not the
2026-09-21 session, which cannot be replayed because the input stream was never logged and the
24-entry ring holds ≈ 1.4 s of a 33 s episode; §5.8's tunable default **on**, once its flip
conditions are met; W8's per-adapter baselines and the `gpu-validate` aggregate exit.

### After

W5's M8/B9 (the `decide` lift and both mirror deletions) and the T-6 job watcher. Explicitly **not**
scheduled: the aggregate-load case (MEM-37, the second half of U38) beyond the probe's admission,
because no controlled reproduction exists and it is a different failure shape; dual view (issue #3),
unreproduced in 12 trials; cross-vendor arithmetic (U26) — detect only, with the detection itself
now red-capable through `--gputest` (W8); and **MEM-61's `[0, step]` motion-chunk contract**, which
is a one-predicate bug for the ordinary queue rather than an architectural change, and whose interim
answer here is detection (§5.9, §10).

### Dependencies, and what each step unblocks

- W1 unblocks everything. W2's replay gate, W4's restart count, W6's stale score, W8's `--autodive`
  rebuild and W9's anti-vacuity predicate all read the record. (W9's predicate is the exception that
  can be built first, from the LIVE manifest fields that exist today — §6.7.)
- B5 (the un-suppression) must precede W3, or `NoActuatorReachable` is the normal answer.
- W2's `admitted_budget` must precede W7, or the quarantine has no price to compare.
- W4's `Plan` must precede W5, since it is `decide`'s output type, and **must** precede W6, since
  `Provenance` carries `FrameKind` and `WalkSig`. The part of W6 that cannot wait is not in W6:
  `ReadingProvenance` ships inside W2's B4, because `budget_confident` is defined against the
  reading's `pass_steps` (§5.1, §5.5). Three places in this document say this; they agree.
- The minimum set that changes the odds on issue #1 is **W1 + W2 + W3 + W9**, about 23 days
  including W1's measurement day. Of those, W1 alone changes what the *next* field failure can tell
  you.

### The 2026-09-21 budget stall — disposition

The instrumentation exists (beta.112's ring, `diag.rs:58-62`, `539-556`; the `FRAME BUDGET IS BLIND`
tripwire, `main.rs:14711-14748`). The reaction was deliberately withheld, and correctly.

**What ships:** the wall-clock dead-man of §5.1 — the blind latch arms the chunker, not the budget,
with the clear-condition stated (a frame whose wall interval is under half `tdr_budget_ms`). It
requires no theory of which mechanism fired, which matters because mechanism (d) is unmeasurable by
construction.

**What must be in hand first — all four:**

1. A beta.113+ record covering the **whole** slow episode, from the RX 6800 XT. Today's ring is 24
   entries ≈ 1.4 s at ~17 decisions/s; the episode ran 33 s.
2. `--chunk-sweep` on the RX 6800 XT at the field geometry — the per-card constant term, and the only
   way to test whether RDNA2 Vulkan places the pass timestamps where we assume. Plausible `steps=`
   with implausibly small `ms=` is the signature.
3. The `escaped-ref-shallow` rung reaching the regime on that card at least once, on **stock
   beta.112 logic**, producing the field signature. Without this there is no control that can express
   the variable under test.
4. A measured engagement rate of 0 for the latch on three healthy runs per card.

If (1) comes back with the record showing timestamps never armed, that is mechanism (a) — the timing
machine never returned to Idle (gpu `lib.rs:2266-2269`). The dead-man still ships unchanged; a
second, separately gated change — re-arming independent of `Idle` (U14) — then becomes the next item,
and not before.

---

## 9. How we will know

All computed by `framelog.py summarize` from the record, so the same numbers appear in a harness run,
a dev A/B and a user's field capture. Three pairs minimum for any comparison; the control is always
scored. The one stated exception is the device-loss survival test, where three "before" arms would
mean three deliberate device losses on the user's card: that comparison is one before-arm and three
after-arms, with its cost written down in §8.

| metric | baseline (source) | target |
|---|---|---|
| Consecutive wall-slow frames with an unchanged admitted budget | **20** (field, 2026-09-21; budget frozen at 1.515e11 through 200–1027 ms frames) | 0 |
| BLIND episodes followed by a budget **decrease** within k frames | **0 of 4** | 100% |
| Lethal-band lines fired during a fatal episode | **0**, while `slow frame` fired 20 | ≥ 1, each followed by a visible retreat |
| Blind episodes per hour of dive | 1 episode of 33 s in a 14-minute session; the tripwire recorded 339 decisions in 20 s with 0 false trips | 0, and any episode fully recorded |
| Starve/resume pairs per minute | 58 in ~35 s ≈ 99/min | baseline it on three healthy runs per card first, then assert at **2× the healthy 95th percentile** — a fixed "< 5/min" would have no derivation, and §7's standing precondition forbids fixing a threshold before the healthy distribution is known |
| Record coverage of an episode | 24 entries ≈ 1.4 s of 33 s | ≥ the whole episode |
| Fraction of dispatching frames with a provenance-matched reading | **unmeasurable today** — nothing records which dispatch was timed | measure it on the first three healthy runs, publish that as the baseline, then assert **no worse than the baseline minus its run-to-run spread**, with the shortfall attributed. No fixed fraction ships before the distribution is known (§7's standing precondition), and a round number like 0.9 chosen in advance is exactly that |
| `NoActuatorReachable` occurrences | unmeasurable today; MEM-38's session would read 100% | 0 |
| Walk restarts per dive | **7–10** post-`112a088`; 128–216 in the three failed attempts | ≤ 12 |
| Screen blank fraction (3 pairs) | **1–7%** post-`112a088`; 8–14% run-to-run variance | ≤ 7%, asserted |
| Screen `stale` fraction | not measured today | baselined, then asserted |
| Held-frame max magnification | printed by `--zoomtest`, asserted nowhere; design target ≤ 1 octave | ≤ 1 octave, asserted |
| Rebase per sampled pixel, session 95th percentile (the counter is full-frame only and lands 2–3 frames late, so it is a session statistic, not a per-frame one) | **~7 wraps/px** at 1.76e6× (25–33 M/frame, `bla_skip=0`, `orbit_len=655` against ask 4627) — memory note, **not verified in code** | **< 1.0 outside a declared rung**, and any frame above 4 with `chunk_needed == false` is W9's RED |
| Fraction of settled frames admitted below native resolution | the MEM-49 latch ran **9% of the pixels** for a whole dive at 17.8 ms/frame with the GPU idle | **gated** by `--selftest live-res` extended to the admission path (W2, gate 4), *and* watched here as the anti-latch canary. A watched number alone would have been P6's own mistake |
| Open classes with no check that can go red | **5 of 9** (§2.1 and the catalogue both bold nine open classes; `findings-verifiability.md`'s summary paragraph says eight and miscounts its own table) | 0 of 9, with two of the five reaching *partial* only — the aggregate half of main-thread blocking, and issue #3 inside open/unexplained — both named non-goals |
| Log bytes per minute / accum share | 1.06 MB per 14 min, **65% accum** | < 300 KB, < 5% |
| dev-box build → Radeon verdict on the share | days, hand-assembled | ≤ 24 h |
| Device losses per field session at the field geometry | 2 in ~5 weeks, both RX 6800 XT | 0 across 3 sessions ≥ 30 min — **and read it for what it is**. Against a base rate of two losses in five weeks, three half-hour sessions cannot distinguish a fix from luck; an error that runs in your favour is the one you will not notice (MEM-77). A green here means *the regime was entered and survived* (W9's entry proof carries that), and it does **not** mean issue #1 is closed |
| `test result:` total | a **`test result:`-line** count from a full `cargo test` run, not a `#[test]` grep — the `#[test]` counts on `4e86e0e` are 486 app / 117 core / 34 gpu / 20 state, against a beta.111 snapshot of 479 / 106 / 34 / 20, so the figure must be re-measured per release and labelled with how it was obtained | never decreases |
| `--bench-matrix` algorithmic drift on a refactor commit | **0** (the beta.111 matrix) | 0 |
| Record overhead, interleaved ABABAB median-of-3 | not yet measured | < 0.5% of median frame time |

---

## 10. Non-goals and risks

### Non-goals

- **Not closing issue #1.** Nothing here is a reproduction. The claim is that W9 makes one obtainable
  and W1 makes the next capture decisive.
- **Not tuning any controller constant from the 2026-09-21 log.** A constant moves when a replay or a
  Radeon run says so.
- **No parallel telemetry system, no network, no uploads.**
- **No rayon, no new threads** beyond the reference workers that already exist.
- **Not rewriting the renderer, the shader or the bignum layer.**
- **Not issue #2** (upstream) or **#3** (dual-view Julia — the honest state is
  OPEN-AND-UNREPRODUCED, not "probably this").
- **Not deep golden authoring** (U37), which is its own project.
- **Not fixing MEM-61** — a motion chunk's `[0, step]` contract being false wherever the view's
  first escape exceeds `step`, which the user confirmed and which is **not verified in code this
  pass**. It is open, it is in an open class, and it gets a disposition rather than silence: it is a
  contract bug in one predicate, not an architectural one, so it belongs in the ordinary bug queue
  and near the front of it. What this document does for it is *detection* — W6's screen gate scores
  the flat fraction that is its symptom, and W8's `--livetest` second oracle plus `capped_frac` /
  `iter_exhausted` in the verdict stop a starved view from matching a starved oracle. Detection is
  not a fix, and this bullet exists so the next reader cannot mistake one for the other.
- **Not the aggregate-load case** (MEM-37): a concurrent export plus the live path, which has no
  controlled reproduction. The probe's admission (§5.6) is accounting, not a bound on total
  in-flight work.
- **Not new UI**, beyond §5.7's quarantine notice.

### Risks in the plan

| risk | mitigation |
|---|---|
| M8 — lifting stages out of a 2,100-line `&mut self` function | one stage per commit, byte-neutral under the corpus, `--selftest live-res` pins the arithmetic, and B9 follows the last stage only |
| A new latch on the admission path | `--selftest live-res` extended to `admitted_budget` as W2's fourth RED criterion — the gate written for MEM-49 already pins both halves of that invariant — *plus* "fraction of settled frames admitted below native" on the scorecard as a canary |
| B5 un-suppresses the shrink and the tiles in frames that never saw them, and the motion constants were calibrated with the suppressions live | the P18 audit rides on the B5 commit itself, with a three-pair screen measurement and each constant re-measured or explicitly recorded as unchanged (§5.10) |
| A new gate engaging on healthy runs | the standing precondition at the head of §7: engagement rate 0, or a stated healthy distribution, on both cards before any threshold is fixed |
| Deleting the present-throttle vetoes re-exposes the compositor case | `wall_budget` learns only from frames that dispatched; the discriminator becomes "did anything get submitted recently" |
| `CONTENT_MIN_ESCAPED` on the fast path calls a thin-filament frame blank | recalibrate across six views in the middle of the range, not at the extremes |
| Turning informational reds into failures surfaces pre-existing failures | expected; each is filed, none is re-normalised |
| The reference-fitness change re-enters the build-storm family | default off behind a tunable with named flip conditions; audit every heuristic calibrated against the old behaviour |
| 45–50 days is long for one maintainer | the beta cadence of §8, with a stated failure mode at each boundary |

### What would make this strategy wrong

Stated so it can be checked rather than defended.

1. **If the first field capture shows the fatal frames were not in the blind regime at all** —
   timestamps armed, readings representative, the budget moving — then the pricing workstream's
   premise is wrong, and U9's five candidates must be re-discriminated before the admission change
   ships. W1 is designed to be able to produce exactly that answer.
2. **If the blind latch or `budget_confident` engages at a high rate on a healthy passing run**, the
   constant is wrong and shipping it would add a rule nobody can explain. That is the
   `MOTION_UNPRICED_MAX` shape, 1,323 engagements per autodive, and it is why the engagement rate is
   a precondition rather than a follow-up.
3. **If the reference-fitness derate costs enough rebuilds to make dives visibly worse on the screen
   scorer**, the derate is the wrong actuator and chunking alone must carry it.

A plan whose whole argument is that this project's failures come from unfalsifiable claims should
state its own falsifiers. The one-line test of completeness is that every workstream above names a
gate, a place it measures, and a condition under which it goes red; these three are the test of
correctness.

---

## Appendix A — automated-check inventory

"Real loop" = drives the app's own `update()` / `build_params`; "model" = drives a pure function or a
harness copy. "Screen" = scores captured pixels.

| harness / gate | what it measures | pass criterion today | real loop or model | screen or per-pass | GPUs | can catch | cannot catch |
|---|---|---|---|---|---|---|---|
| `--selftest` (~168 checks — a runtime count from the last full run, not readable from the source, since `push_check` appears at 133 call sites and several are inside loops — plus **19** goldens in `validation/golden/`) | arithmetic, sizing, reference picking, formats; goldens ≤ 1e6 | exit code + `validation/report.md` | model mostly; the `live-res` group drives the app's own `build_params` | per-pass | single, blessed | numeric limits, format contracts, sizing at `fe_budget = 0` and at a converged budget | anything live-path; a budget learned large and gone stale; `opt_in` groups never run unfiltered |
| `--livetest` grand tour / ultra dive | 24 + 5 settled checkpoints, e30–e200 | drift against a blessed JSON | real loop, **own controller copy** (`livetest.rs:355-395`) | per-pass | single | settled regressions at depth | frames between checkpoints; starvation (oracle shares the ask) |
| `--motiontest` | motion counters at corpus loc 07, 2^103.3, mode 2, explicit 1M ask | four-term verdict; `blank_walks_total` printed, not asserted; VACUOUS computed (`motiontest.rs:226-229`) then collapsed into the same exit 2 as a real assertion failure (`:271`) | real loop | per-pass | single | mode-2 motion regressions, one device loss (CL-165) | the screen; other depths, modes, auto-iter; the difference between "did not reach the regime" and "failed" |
| `--zoomtest` | on-screen update cadence, held-frame magnification | **none** — prints and exits 0 | real loop | per-pass timings | single | cadence data | nothing; it has no criterion |
| `--uitest` / `--juliadive` | scripted UI walk, screenshots | per-check; deep band is WARN-not-FAIL | real loop | screen (stills) | single | reflow, panel layout, appearance | deep-band failures (downgraded by CL-166) |
| `--autodive` | device-loss regime | `exit(if lethal > 0 {0} else {2})` | real loop | per-pass | single | a lethal *reading* | a blind loss — reports "did not reach the regime" (U8) |
| `--deviceloss-repro` | cost curves, actuators, parity axes | **none** — report only | real loop | per-pass | both, by hand | cost characterisation | nothing; no criterion |
| `--chunk-sweep` | per-pass cost against window size and dimension parity | **none** — no `exit(` in the file | real loop | per-pass | both, by hand | the fixed term, parity cliffs | nothing; no criterion |
| `--soak` | liveness over a 20 s window | crash/hang census | real loop | none | single | crashes, hangs | a ~1 fps grind passes |
| `--torture` ladder | escalating live/offline rungs | rung exit codes; ladder shape tested | real loop | mixed | single | what its rungs cover | the short-escaped-reference regime — stated in its own header |
| `--gputest` | GPU op families per backend | per-op max error | model | per-op | both; red on NVIDIA by construction | df32/EFT folding (found CL-108) | cannot gate the dev box |
| `--bench-matrix` | 28 segments: signatures + timings | signature equality, `|| !same_gpu`; timing warns at 1.35× | real loop | per-pass | single effectively | algorithmic drift on the blessed card | anything on a second adapter |
| F3 corpus `--check` | 38 locations, e0–e1105 | maxΔ per location | offline | per-pixel | single | perturbation correctness | live path; SA and the corrector are pinned off |
| `scripts/dive-capture/` | blank / flash / flat fractions | **manual, no thresholds** | real loop | screen | single | blanking and flashing | nothing automatically |
| `scripts/gpu-validate.ps1` | the cross-GPU battery | records exit codes, **always exits 0** | n/a | n/a | multi | data collection | nothing; red is normalised in prose |
| `checklist_coverage.py` | that each checklist enforcer name resolves | name resolution | n/a | n/a | n/a | a missing enforcer | a vacuous one; reports 0 outstanding |

**After this plan**, the additions are: `--selftest record`, `budget-confidence`, `pricing-props`,
`retreat`, `provenance`, `walk-sig`, `replay`, `log-budget`, `ref-fitness`, `settings-quarantine`;
`--replay` and `--logcheck` as modes; the screen gate with thresholds and a `stale` score; criteria
and exit codes for `--zoomtest`, `--chunk-sweep` and `--deviceloss-repro`; the new `--torture` rungs;
per-adapter baselines and an aggregate `gpu-validate` verdict; `gate_selfcheck.ps1` and
`validation/reachability/`; `fieldcheck.py` and the share loop.

---

## Appendix B — the verified findings

Fifty-nine findings, each upheld by two independent adversarial reads (code-truth, and
coverage-and-significance) of the tree that shipped as beta.112 (`bb27d5a`); every citation was
re-checked against `main` = `4e86e0e` for this revision. Listed in full below by id, severity, subsystem,
one-line claim and the workstream that acts on it; the complete descriptions, evidence blocks,
failure scenarios and gate-coverage notes live in the assembled findings file that this document was
written from.

### Critical

| id | subsystem | claim | W |
|---|---|---|---|
| U1 | frame-cost | `chunk_over` is downstream of the budget, so a budget that cannot shrink switches off the only actuator that bounds an un-chunked frame | W2, W3, W7 |
| U2 | frame-cost | The wall-clock fallback can essentially never complete a probe: an arriving reading destroys it before it prices | W2 |
| U3 | motion | `PIN_COMPLETION_TIMEOUT_US` (170 ms) permanently opens the pin serialization gate on any hardware slower than 170 ms | W3 |
| U4 | frame-cost | The chunk-pass price model has no constant term, so every retreat asymptotes at a cost the model calls ~zero | W3 |
| U5 | frame-cost | A reading the budget throws away still counts as "measurement arrived" for two other loops | W2 |
| U6 | reference | An escaped reference is pinned for any iteration ask, and the deferral that was to end when this appeared has not been claimed | W9 |
| U7 | verification | No harness can produce the regime that has killed every device, and the one instrument that could is armed only by an event the field failure did not contain | W9 |
| U8 | verification | `--autodive`, the only gate claiming to test device loss, reads the exact instrument the field failure proved blind | W8 |
| U9 | frame-cost | The 2026-09-21 budget stall: five candidate mechanisms, three discriminable by beta.112's ring | W1 |
| U10 | frame-cost | Three safety gates share one denominator, and it is the number under suspicion | W2 |
| U11 | frame-cost | The present-throttle detector's discriminator is inverted for a backlogged queue, and it disables every wall-clock safeguard exactly there | W2 |
| U12 | frame-cost | The serialized walk manufactures the evidence that makes the present-throttle detector switch off every retreat | W2 |
| U13 | reference | Reuse and the disk cache admit an orbit up to 128 bits below a fresh build's precision | W9 |

### High

| id | subsystem | claim | W |
|---|---|---|---|
| U14 | gpu | Timestamps arm only when the readback machine is Idle, and nothing records or asserts which dispatch was timed | W6 |
| U15 | diag | The budget-blind tripwire is reset by a budget moving in the wrong direction, and reports one view's state for both | W1, W2 |
| U16 | verification | The `--autodive` regime verdict is built from the same readings the field loss proved short | W8 |
| U17 | reference | Nothing refuses a frame whose reference escaped far below the ask, and the only directly measured cost multiplier is print-only | W3 |
| U18 | state | Recovery reinstates the lethal configuration: only the view is reset, and the crash-view `.fdn` carries the ask back | W7 |
| U19 | presentation | "Verified by construction" verifies completeness, not content: an all-interior frame is a verified picture | W6 |
| U20 | presentation | The reuse-hold reprojection source is outside content verification entirely | W6 |
| U21 | verification | No automated gate looks at the pixels that reached the screen during motion | W8, W6 |
| U22 | frame-cost | `chunk_over` is the only place the iteration ask meets the cost model, and all three actuators hang off it | W2 |
| U23 | frame-cost | `mode_rate` is a session-long running minimum, fed by every kind of dispatch, feeding three floors, with no recovery | W2 |
| U24 | motion | The moving frame's resolution is decided by the palette normalisation window | W4 |
| U25 | reference | The on-disk orbit cache ranks by length alone and never reads the header fields that would say the orbit is unfit | W9 |
| U26 | gpu | The rebase count is shader arithmetic that folds differently per vendor, and the only cross-vendor gate is loosened for that reason | W8 (detect only: the counter arithmetic joins `--gputest`'s per-backend op families). The fix half stays out of scope, and §5.2 keeps the counter out of `admit()` accordingly |
| U27 | autopilot | With auto-iterations on, starvation cannot be named: both witnesses in `dead_end_message` require auto off | *not assigned* |
| U28 | diag | The iteration ask has no always-on per-frame record, and the two always-on records disagree in the open field incident | W1 |
| U29 | diag | Beta.112's always-on tripwires are report-only and unread; nothing greps a harness log and no test pins the wiring | W1, W8 |
| U30 | verification | On the hardware that actually fails, every image and signature comparison is informational by construction | W8 |
| U31 | verification | Motion is graded by counters, settled checkpoints and a synthetic frame interval | W8, W6 |
| U32 | verification | A field log cannot be replayed into the controller, so a field failure can never become a regression test | W1, W5 |
| U33 | verification | The live gates dive with auto-iter off, at one location, in one mode | W9 |
| U34 | verification | Iteration starvation is invisible **to the harnesses**: the oracle shares the live frame's appetite, and no harness reads `capped_frac` / `iter_exhausted` — though the app itself does, in `Perf::view_resolved` and `limit_status` | W8 |
| U35 | verification | Several named gates cannot go red; "it ran" is asserted in a handful of places; a bless silently re-anchors the suite | W8 |
| U36 | verification | Every gap above already has a written plan; none is built, and the tool that counts unbuilt work reports zero | W8 |
| U37 | verification | Exact-image verification stops at 1.6e15×; past it every check shares the ask or switches off the machinery | *out of scope* |
| U38 | frame-cost | A synchronous, un-budgeted, unaccounted dispatch on the UI thread — **size-bounded** to ≤ 128×128 at ss = 1 (`autopilot.rs:658-664`), so the exposure is the unbounded **aggregate**, not this pass | W3 for the accounting half; the aggregate half is a named non-goal |
| U39 | presentation | `hold_scale_floor` takes the JUMP branch for every dive and every wheel zoom, because only Space writes `zoom_vel` | W4 |
| U40 | frame-cost | The motion pacer's wall LIFT is unbounded, kind-blind, and can be proved by a frame that never waited for the GPU | W2 |
| U41 | reference | The series-approximation skip has no validity re-check while the BLA has one every frame | W9 |
| U42 | presentation | The settled compose gate snapshots and serves a texture with neither a content nor a provenance check | W6 |
| U43 | colour | A held or reprojected frame is re-coloured under the current normalisation map; under the log map everything below `norm_lo` collapses | W6 |

### Medium

| id | subsystem | claim | W |
|---|---|---|---|
| U44 | presentation | Nothing bounds how far a held frame may magnify on the branch the design meant to leave free | W8 |
| U45 | presentation | The coarse-preview gate asks a different question at a different sampling, and fails open | W6 |
| U46 | frame-cost | The wall fallback zeroes the motion-jam backlog that only real completions may retire | W2 |
| U47 | motion | The pin serialization gate's escape hatch opens precisely on the hardware it protects | W3 |
| U48 | frame-cost | One band ledger, two targets, two clocks, two lanes | W3 |
| U49 | frame-cost | In-flight pricing and the lethal shed are switched off for the whole of unpinned motion | W2 |
| U50 | reference | Install-time reactions are transition-only, so a reference unfit from its first install never derates | W9 |
| U51 | frame-loop | Cross-frame numeric state reset on the wrong event, or never reset | W4 |
| U52 | verification | Each live harness mirrors the loop it grades; one mirror is drifted right now | W5 |
| U53 | autopilot | The steering loop's cadence and pacing are silently rescaled by the cost loop's failures | W2 |
| U54 | iteration | The adaptive-iteration probe's staleness gate discards readings silently, and three rules break the tag it compares | W6 |
| U55 | reference | Reference identity at install is "longer wins", which is wrong for escaped orbits at different points | W9 |
| U56 | presentation | The dual-view second panel's moving resolution is a view-0 quantity view 1 can never move | W4 |
| U57 | presentation | `CONTENT_MIN_ESCAPED` can call a genuine thin-filament frame blank, and three of them stop the dive | W6 |
| U58 | colour | The converged SSAA average is presented under a colour signature that omits the window actually mapped | W6 |

### Low

| id | subsystem | claim | W |
|---|---|---|---|
| U59 | presentation | `hold_verified` is written on three paths and read only by trace formatting | W6 |

### Contested

None. Both lenses agreed on all sixty candidate findings.

### Refuted

- **"At the shipped default iteration base the adaptive probe can never engage, and the diagnostic
  that would say so is keyed on the same predicate."** Refuted by both lenses and by a third read.
  The arithmetic half is right — with `eff_iter = base + min(220·octaves, 2M)`
  (`viewport.rs:303`) against `zoom_iter_cap = 2000 + 256·octaves` (`main.rs:2047-2050`), a default
  base of 256 leaves `cap_bound` false at every depth. The observability half is backwards, and it is
  the half the finding was named for: `budget_maxed` (`render.rs:4209-4210`) is written and **true**
  in exactly that regime, under an in-code comment saying that is precisely when the user most needs
  telling; `limit_status` then emits the "iter capped" message (`ui/menus.rs:1202-1224`), red-gated
  by `limit_status_matches_measured_regimes` (`menus.rs:1819-1860`). The failure scenario is also
  arithmetically impossible, since `live_iter_budget` is a `min` against `eff_iter`
  (`render.rs:8041-8045`).
  - **Salvaged, and not lost:** (a) the boost seed is not gated on `cap_bound`
    (`render.rs:4157-4180`) and can leave a large, permanently inert multiplier — that is U51(b);
    (b) `--selftest iter-budget`'s boost ceiling of 16.0 (`selftest.rs:3627`) predates
    `ITER_BOOST_MAX = 256.0` (`tunables.rs:938`), so that check passes on an outdated model of the
    controller — a real, cheap, separate coverage defect worth filing on its own.

---

## Appendix C — the incident catalogue

201 incidents were merged from the release notes and the post-mortem notes and classified in §2.1.
Reproduced here: every incident that is **open**, plus the exemplar for each recurring shape and each
"one fix broke another" pair — the rows this document's workstreams act on. Ids prefixed `CL-` come
from the release-note catalogue and `MEM-` from the post-mortem notes; a row carrying both is the
same incident seen from both vantage points.

| id | version / date | symptom | class | detected by | gate added |
|---|---|---|---|---|---|
| CL-1 · MEM-39 | beta.112 / 2026-09-21 — **OPEN** | RX 6800 XT lost the GPU at a shallow 1.76e6×; frames 200 → >1000 ms over half a minute | cost-model-blind | field crash report (user's own zoom) | none — diagnostics only (budget-history section, blind tripwire) |
| CL-2 · MEM-39 | beta.112 / 2026-09-21 | (no symptom) — the escaped-pixel atomic suspected of costing CL-1; measured Δ=0 ms, pixel-identical | retracted diagnosis | code reading | none |
| MEM-61 | 2026-09-20 — **OPEN** | "it starts to pan and the screen goes to a flat colour so it seems to go off screen" | iteration starvation | field log | none |
| CL-3 · MEM-62 | beta.110 / 2026-09-20 | dive from 8e175×: one frame in three a flat colour, black or the previous centre's colour | presentation ≠ render | screen recording, lined up against the log | escaped-pixel content reading gates adoption and snapshotting |
| CL-4 · MEM-62 | beta.110 / 2026-09-20 | same dive — flat/black frames shown *and adopted* | guard scope | screen recording | `TRACE=gpu` content reading per frame |
| CL-10 · MEM-56 | beta.109 / 2026-09-19 | at 4× zoom speed the auto-zoom's aim slid off the edge | steering & search | the user's own 4× dive at 1e118 | target-motion replay: 20/81 → 2/78 looks |
| CL-11 · MEM-59 | beta.109 / 2026-09-19 | the picture slid sideways for the first half second of every dive | cost-model-blind (proxy input) | user report + `--show-timestamp` recording | sideways travel over the opening half second: 2.84% → 1.03% |
| CL-12 | beta.109 / 2026-09-20 | auto-zoom stopped with "no detail ahead" on a 1e590 dive that had detail | guard scope | user report | none automated |
| CL-13 · MEM-60 | beta.109 / 2026-09-20 | at ~1e240 a dive showed **36% of its length blank**, in stretches up to 2 s | iteration starvation | screen recording of the reported dive | `scripts/dive-capture/` blank measurement, 3 runs per arm |
| CL-15 · MEM-46 | beta.109 / 2026-09-20 | every colour on screen swelling and shrinking ~2.5×/s during a deep zoom | normalization source | user report at a 2e13 view | `render/norm_partial.rs`; palette-scale swing 1.35 → 0.011/s |
| CL-17 · MEM-21 | beta.109 / 2026-09-19 | one black frame at the start of every Space zoom with the Performance section open | UI reflow → render | user report | live readouts `.truncate()`; status-bar width invariants |
| MEM-38 | 2026-09-18 — **OPEN** | "the app is no longer responding" — ~1 fps, no device loss, no watchdog line | actuator blind spot | field log | none |
| CL-19 · MEM-47 | beta.108 / 2026-09-18 | a 100× click-to-zoom flashed the panel black | presentation ≠ render | user report | `hold_scale_floor`; `--zoomtest` records held magnification |
| CL-23 | beta.108 / 2026-09-18 | live zoom jerky: 246 ms worst frame, 32 frames over 100 ms across 1× → 1e100 | cost-model-blind | `--zoomtest`, a new harness | blessed cadence figures |
| CL-28 · MEM-49 | beta.108 / 2026-09-18 | a whole 1× → 1e100 dive at 30% linear resolution while the GPU sat idle at 17.8 ms/frame | composition defect | `--zoomtest` | `live-res` selftest group, both halves of the invariant |
| CL-29 · MEM-48 | beta.108 / 2026-09-18 — **open-and-unreproduced** | after a small pan at extreme depth, a translucent copy of another location stayed on screen | coarse identity key | user report | `FOLD AT ANOTHER VIEW` names the sample and both views |
| MEM-50 | 2026-09-17 — tried, reverted, **open** | a 50-octave-stale frame = a uniform smear; a shallow glide frozen at res 0.57 | presentation ≠ render | the **user**, live, in the maintainer's own session | `zoomtest-full4`/`full1` fence; watch "held frame max" |
| CL-33 | beta.107 / 2026-09-17 | 7–22% of frames during a deep dive held or reprojected, stalls 140–240 ms | reference lifecycle | `--divetest` glide mode | held-frame fraction and longest stall |
| MEM-37 | 2026-09-13 — **OPEN** | device lost at ~68 min on the already-fixed driver, with concurrent export | main-thread blocking & aggregate | crash report + log reconstruction | none; no `export_active` gate found |
| CL-38 · MEM-32 | beta.99 / 2026-09-11 | re-zooming a parabolic exact point settled then sat "computing" for minutes | guard scope | user report | `--deviceloss-repro` reproduces it headlessly |
| CL-39 · MEM-31 | beta.99 / 2026-09-11 | the same view, ~200× per-pass cost for no visible reason (odd **width**) | driver fast path | `--deviceloss-repro` with a new parity axis | the parity axes are the regression |
| MEM-35 | 2026-09-10 | shedding the chunk window to its 256 floor did **not** save the device | actuator blind spot | field crash report + `--deviceloss-repro` | `--deviceloss-repro` characterises the cost curve |
| CL-49 | beta.75 / 2026-09-02 | Save on a `.fdn` at 9.98e60205× put the app into "(Not Responding)" for minutes | main-thread blocking | user report | refusal in 0.3 ms with nothing resident |
| CL-50 | beta.73 / 2026-09-02 | every `.fdn`, PNG and EXR written before v0.2.20 loaded with coordinates silently dropped — **52 releases** | file/format contract | a new shipped-file sweep over 81 `.fdn` and 39 `.kfr` | that sweep, asserting a silent load |
| CL-52 | beta.71 / 2026-09-01 | an export-aspect check passed against a known defect, twice over | vacuous gate | self-review | verified red against the rule it replaced |
| CL-58 · MEM-65 | beta.16 / 2026-08-29 | the dual-view Julia panel came back blurry after the app sat in the background overnight | stale measurement | user report | `--dualsettle` |
| CL-69 · MEM-21 | beta.149 / 2026-08-27 | a deep view could sit black indefinitely, never finishing | UI reflow → render | user report | `status_readouts_never_change_width` |
| CL-77 · MEM-31 | beta.137 / 2026-08-24 | deep renders up to 20× slower at half of all window sizes (odd **height**) | driver fast path | investigation of "the fractal simply got harder" | crash-report render size; the parity axes |
| CL-78 | beta.136 / 2026-08-24 | the same view rendered a few dozen pixels differently by tile budget and machine speed | schedule-dependent output | corpus regression, three views drifting | a self-test fails if a render mixes the two paths |
| CL-83 | beta.125 / 2026-08-21 | every view deeper than 1e308× rendered blank — live four days | numeric limit + guard scope | user report | none named at the time |
| CL-89 · MEM-29 | beta.113 / 2026-08-20 | a deep refinement showed a solid interior-colour screen with no spinner | presentation ≠ render | field report | state-aware FPS row; pure tests |
| CL-90 · MEM-29 | beta.112 / 2026-08-20 | deep views near minibrots rendered flat, or noise, depending on timing | normalization source | user report with screenshots | pure tests only |
| CL-95 · MEM-25 | beta.106 / 2026-08-19 | Home from a deep view reset the graphics card; the first fix made deep interiors look like noise | unbounded dispatch | user report | **`--motiontest`** |
| CL-102 | beta.98 / 2026-08-18 | zooming or panning at extreme depth left the picture blank — 100% black at 2e82× | reference lifecycle | a 6-minute deep-zoom validation script | five deep stops each gated |
| CL-108 · MEM-2 | beta.88 / 2026-08-14 — **OPEN** | (no direct symptom) — "df32" has been running at ~f32 on the shipping configuration all along | driver / compiler | **`--gputest`**, first run | `--gputest` grades every op family per backend |
| CL-112 · MEM-22 | beta.80 / 2026-08-13 | the grand tour's six deep holds collapsed from 480×270 to 16×16 | measurement ⇒ bootstrap binds | `--livetest`, first run since beta.69 | `--livetest` graded against a blessed baseline |
| CL-113 | beta.79 / 2026-08-13 | a scripted deep dive rendered ~26-pixel blocks for most of the descent | cost-model-blind | user report of a scripted dive | `--livetest` baseline |
| CL-114 · MEM-22 | beta.78 / 2026-08-12 | "Prefer detail while zooming" made a long dive worse than the toggle off | presentation ≠ render | user report | held frame never more than half an octave past a sharp frame |
| CL-115 · MEM-21 | beta.71/70 / 2026-08-12 | a deep view never finished: label toggles → bar reflow → "interaction" → grid discarded | UI reflow → render | user report | the slot renders the identical monospace widget in both states |
| CL-124 | beta.53 / 2026-08-09 | with auto-iterations off, 10,000,000 in the Iterations box rendered as ~82,000 | silent default | user report | selftest pins 10M honoured verbatim |
| CL-125 · MEM-15 | beta.47 / 2026-08-09 | a 1445×1134 panel at 2,000 iterations rendered at **504×396**, permanently | measurement ⇒ bootstrap binds | measured on affected hardware | `live-res` selftest group |
| CL-127 | beta.47 / 2026-08-09 | a doomed reference build was requested again and again | reference lifecycle | trace analysis | none named |
| CL-134 | beta.35 / 2026-08-07 | the live view rendered **100% black at 1e61×, 1e72× and 1e82×** where offline was correct | presentation ≠ render | **`--livetest`**, a new headless harness | `--livetest` against a blessed baseline |
| CL-141 | 0.2.0 | a deep exterior view rendered as distorted tiling | perturbation correctness | user-visible | none named at the time |
| CL-151 | 0.1.0 / 2026-07-01 | holding space past ~1e420× the view went solid black until it settled | iteration starvation | user report | none named |
| CL-163 | **open, issue #3** / 2026-09-04 | dual view: the Julia panel stays blurry after motion, or lands on the wrong content | unexplained | user report with screenshot | `--dualsettle`, report-only |
| CL-164 | **open, issue #1** / 2026-08-22 | an interactive deep zoom lost the device on stock tunables (beta.129, RTX 3080) | cost-model-blind | field crash report | crash-manifest stamps for `res=`, `sa_skip`, in-flight state |
| CL-165 | **open** / 2026-08-21 | one device loss in `--motiontest` at Home→Settle in mode 1, never reproduced | unexplained | harness, during a gate battery | crash report preserved |
| CL-166 · MEM-76 | **open** / 2026-08-11 | the same view renders completely differently by machine: black on one, structured on another | hardware-dependent adaptation | `--uitest` deep band | the deep band is **WARN-not-FAIL** because of this |
| CL-167 | **open** / 2026-08-10 | uniform speckle with one clean central diamond at a ~1e103× three-spar view | quality ceiling, not a defect | user report | `--uitest`'s `live-floatexp-1e30` reproduces it visually |
| CL-168 | **open** / 2026-08-07 | one unexplained access violation `0xc0000005`, no crash report | unexplained | field, one occurrence | abort/OOM reporting shipped in beta.43 |
| CL-169 · MEM-40 | **open, issue #2, upstream** / 2026-08 | dragging the window between differently-scaled monitors multiplies its size every crossing | upstream | three field crash reports | `TRACE=dpi` logs scale and size on every change |
| MEM-71 | standing | a single bad reference pick is pinned for ~32 s and kills the device | reference lifecycle | measured on the grand tour; reproduced 3-in-4 | live `bla_skip` counters only |
| MEM-77 | 2026-09-21 (bench kit) | the structure guard passed ten wrong-view renders and one of pure noise, then failed a correct render as noise | vacuous gate | the independent check, then re-examination at native resolution | `verify-views.py`, verified RED 10/10 and GREEN 9/10 |

## Appendix D — implementation log

Where the build departed from this document, and why. Each departure is also recorded at the code
it concerns.

**beta.113 — §6.1 build identity** (merged `f49ff69`).

- `build.rs` lists Cargo's default watch set explicitly: emitting *any* `rerun-if-changed` switches
  off the default "any file in the package", which would have frozen the build counter. The watched
  set and the dirty check's pathspec are one list. It watches `refs/heads` as a directory (a commit
  rewrites the branch ref, not `HEAD`) and not `.git/index` (`git add` would force a rebuild), and
  runs `git --no-optional-locks status` so the check cannot rewrite the index it depends on.
- The identity is used only when git's top level is this workspace: a source tarball unpacked
  inside another repository was stamped with *that* repository's commit, as clean (measured).
- The tarball carries `BUILD-COMMIT.txt`, so the Linux build reads `g<sha>-archive` rather than
  `unknown`.

**beta.113 — W1 `FrameRecord`** (merged `e5bc3a5`, `019678f`).

- `frames.bin` slots are keyed on a record sequence number, not `(frame × 2 + view) % 4096`, which
  gave a single view only the even slots — half the coverage §6.4 promises.
- The gate is `--recordtest`, not `--selftest record`: `--selftest` renders offline inside one
  `update()` and never drives a live frame.
- The overhead criterion is measured by an in-process timer carried in every record (`rec_us`), not
  a whole-frame ABABAB interleave: the cost is tens of microseconds and an A/B's noise floor is
  milliseconds. It is judged as the p99 against a 60 Hz frame; the harness's own frames run
  uncapped at ~1.1 ms, against which the first version went red at 2.3% on a 16 µs cost. Measured
  on the RTX 3080: p99 36–46 µs, 0.22–0.28% of a 60 Hz frame.
- `frames.jsonl` writes a full row only for rare events (slow, lethal, stall — up to 20/s; budget
  moved, refusal, mode switch — up to 2/s) plus one compact summary per second. §6.4's trigger list
  includes `present != Real`, which fires on every frame of a dive (~120 KB/s) against the < 5 MB
  per 14-minute criterion. Measured: ~13 KB for an 18 s run.
- Two pre-existing watchdog defects found by the gate: `last_warn_ms` started at 0, so no hang in a
  process's first 30 s was ever reported; and the unclean-exit report read the log's NEW file after
  a startup rotation, quoting nothing of the dead session.
- A Rust reader of the JSON must not use `serde_json`'s default float parser (best-effort; 22 of 200
  f64s off by one ULP, measured). This matters for W5's replay.
- The issue report keeps the log tail beside the new frame-record section rather than replacing it,
  and the whole report is home-path-redacted.

**beta.113 — W10 log hygiene** (branch `feat/log-hygiene`).

- `--selftest log-budget` is folded into `--recordtest` (the same reason as above). The accumulation
  flood cannot occur there — accumulation is off under harnesses — so the rate limiter is pinned by
  a unit test, and the harness guards against any category exceeding 10 lines/s over 5 s.
- The documentation half is a static scan of the source (`every_log_category_is_documented`), which
  covers categories a given run never exercises; the design's version checked only prefixes present
  in one run's log.

**beta.114 — `frames.jsonl` off the UI thread; the battery runs on local disk** (branch
`fix/record-cost-and-local-battery`).

- The RX 6800 XT's beta.113 battery failed `--recordtest` on cost alone (p99 706/878 µs, limit 83):
  its logs were on `\\vger\share`, because `gpu-validate` built the bundle, config and logs
  included, in `-Out`. The record attributed the cost to itself: `rec_us` is the previous emit's
  cost, and all 68 emits above 300 µs followed a `frames.jsonl` flush (summary 1/s, severe event),
  at ~650 µs each, while no other frame went above 172 µs. A dev-box A/B, interleaved on the same exe,
  reproduced it (local p99 62–65 µs PASS, `\\vger\share` 98 µs FAIL, 2/2 each).
- §6.4's sinks are now split by what they must survive: `frames.bin` stays synchronous on the
  recording thread (an abort must find the slot on disk), and `frames.jsonl` moves to a writer
  thread fed by a bounded queue (4,096 rows, `try_send`, drops counted and logged once). The row is
  still DECIDED on the recording thread (the summary accumulator and rate limits), and the event row is
  formatted on the writer. Clean exit and the panic hook wait ≤ 500/250 ms for the queue.
- `--recordtest` prints where its logs are. With the logs on a network share, a cost over the limit is
  VACUOUS rather than a failure, because it measures the share. A cost within the limit there is still a pass.
- The battery builds its bundle under the system temp folder and copies it to `-Out` at the end.

**Evidence, 2026-09-23 — §8 prerequisite 2, `--chunk-sweep` on the RX 6800 XT** (run through the
field agent at `crash-view-1789960937-0.fdn`, staged as the session; 3 runs per card, beta.114;
results in `<share>\field\results\`, the comparison in `devbox-rtx3080-chunk-sweeps-20260923\`).

- **The per-pass constant term is card-dependent, and large on the Radeon:** eight 256-iteration
  passes at 160×90 cost 17.7–19.2 ms there (~2.3 ms/pass) against 3.0–3.2 ms on the RTX 3080
  (~0.4 ms/pass). A price proportional to steps under-prices every small pass on that card by about
  2 ms. That is §5.2's term, now measured on both cards.
- Full-size work is ~2.2× the 3080's (8 passes at 1280×735: 88–90 vs 40–44 ms). Cost grows 1.42–1.45×
  per window doubling (3080: 1.29–1.30×), and scales as area^0.36–0.39 (3080: area^0.59–0.62). So on
  the Radeon, shrinking the window or the pixel count buys less.
- No ≥8× cliff within a band on either card (the 256-window spread is 3.4–3.6× and 2.8–3.1×).
- ⚠**Not the fatal regime.** The live view settled on a still-growing 37,9xx-iteration reference (the
  sweep's own "NOT SETTLED" warning), not the field session's ESCAPED 655. Prerequisites 1 and 3 still
  need W9's regime instrument (`REF_ESCAPE_AT`) to put the Radeon in the escaped-reference storm on
  purpose.
