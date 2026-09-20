# Verified presentation + autopilot priority (speed / quality)

Status: implementation plan, 2026-09-20. Lands as beta.110.

## 1. The report

Screen recording `autopilot-diev-2026-09-20_12-55-06.mp4` (24.3 s, 731 frames at 30 fps) of an
autopilot dive from 8.1e175× at zoom speed 4×, `prefer_detail`, `min_motion_res` 0.83,
`work_budget_scale` 7.5, auto-iter (ask ≈ 152k–180k). The user's words: "the screen flashing as
it zooms and also going to flat color planes".

Measured on the recording (canvas interior, dominant-colour fraction > 97% = flat):

| what | value |
|---|---|
| flat frames | 255 of 731 (35%) |
| flat episodes | 22, of 2–32 frames (0.07–1.07 s) |
| black flats (5,4,7) | 16 episodes |
| single-palette-colour flats | 6 episodes, each the colour of the previous frame's centre |
| frame-to-frame change at the transitions | mean absolute difference 80–130 per channel — the "flash" |

The log of the same session (build 3445, `FRACTADYNE_TRACE=autopilot,tile,gpu`), +36 s to +59 s:
the view's escape range reads `[3802, 4416]` while the ask is ≈155k; 460 `norm reading EMPTY`
(a pass committed no escaped pixel); 27 `visible-res` rung changes, mostly 1.0 → 0.125 → 1.0
within 12 ms; 50 pin starts, 16 adopts, 26 `Orbit` abandons (a lookahead install at another
point), 7 `Drift` abandons; pins open cold bands with 12-iteration passes (`pin-price band=1
size=12`).

## 2. Mechanism

At this depth every moving refresh is a pinned chunked walk (`pin_frame`): the display serves a
hold snapshot of the last complete frame while the walk runs, and the walk must reach the FULL
ask (155k) to adopt. The picture is complete at ≈4.4k iterations; the remaining 150k only confirm
interior pixels. So adoptions are rare (~1/s at best) and most pins die first.

Black flats. The hold snapshot is copied from the live texture on the frame a pin starts.
`fractadyne-gpu`'s `prepare` runs the RESIZE (which clears the texture when the content stamp is
not this view's — always, during a dive) BEFORE the hold copy, so a pin that starts at a new
resolution snapshots a cleared texture and serves black for its whole walk. The resolution changed
every ~1 s (the `visible_res` ladder flip-flopping on a motion pass budget that swung 50× between
consecutive frames). Nothing in the pipeline checks that a texture has content before it is
displayed or snapshotted; the only content signal (`CTR_ESC_MIN == 0xFFFFFFFF`) is traced and
discarded.

Coloured flats. Between adoptions the snapshot is magnified to follow the zoom. Two to three
seconds without an adoption at 2.65 oct/s is 30–80×: the snapshot's centre pixel fills the
screen. That is the "flat colour plane"; it is the previous frame's centre colour, measured.

Flashing = the alternation: crisp adopted frame → magnified snapshot → black snapshot → crisp.

Contributors that make adoption rare: (a) the full-ask completion rule at a 36× auto-iter
overshoot; (b) the pin floor `MOTION_PIN_TARGET_MS × worst_rate` where `worst_rate` is a
session-long minimum poisoned by one slow reading (12-iteration passes, ~1 s per pin to reach
4.4k); (c) lookahead installs at a different reference point abort the pin (`Orbit`) and reset the
band ledger, so the next pin starts cold again; (d) the 2-octave `Drift` abandon at 4× is 0.75 s.

## 3. Design

### 3.1 Content verification (both modes, all diving)

- GPU: `CTR_ESC_COUNT` (counter slot 7) = escaped pixels in the resolved frame, un-subsampled,
  committed in `fs_resolve` and at the escape return of the single-pass iterate. Published by the
  counter readback with the frame's pixel count, the app's `content_tag` (echoed like `norm_sig`)
  and the chunk cursor the pass ended at (`content_cursor`).
- App: `Perf::content[view]` tracks the reading for the LIVE texture's content (`live_tag` stamped at
  every dispatch) and whether the HOLD snapshot holds verified content. A reading has DETAIL when
  `escaped ≥ CONTENT_MIN_PX` (0.2% of the frame) and the escape range spans ≥ 0.5 iteration.
- Pin adoption requires detail: `Adopt` only when a reading tagged for this pin shows detail
  (escapes are monotonic through a walk, so any non-blank reading proves the texture). A walk that
  completes with its final reading blank stops as `PinStop::Blank`; the hold stays. Three
  consecutive blank walks stop the autopilot with the iteration-cap message.
- The hold snapshot is taken only from a live texture whose content is verified, and the GPU takes
  the copy BEFORE any resize. If no verified snapshot exists (cold start) the display shows the
  live texture — the honest fallback, unchanged.

### 3.2 Refresh policy per mode

`RefreshPolicy::Full` (manual glides, and the autopilot in Quality): adopt at the full ask, native
resolution when `prefer_detail` (unchanged).

`RefreshPolicy::Converged` (autopilot in Speed): adopt as soon as the walk has detail AND the
escaped count has stopped growing across two readings whose cursors differ by ≥ 25% (the picture
stopped changing), or the cursor passed 1.25 × the view's last known escape-range top, or the full
ask. Resolution adaptive (the existing AIMD / `visible_res` cap under the user's floor).
Counted as `adopt_converged`, never as `adopt_partial` (`--motiontest` A1 keeps its meaning).

### 3.3 Autopilot priority

`AutopilotPriority { Speed, Quality }`, persisted as `autopilot_priority` in `session.toml`
(default `speed`), toggled in the Navigate panel next to the dive limit, in Tools ▸ Auto-zoom, and
by `--autopilot-priority speed|quality`.

Quality pacing: `glide_step` takes a speed cap. `cap = HELD_MAX_OCT·ln2 / refresh_period` where
`refresh_period` is the EMA of seconds between adoptions, scaled down further by
`HELD_MAX_OCT / held_age` when the held frame is already older than the bound; floored at 10% of
the slider rate so the dive never stalls. Speed mode: no cap.

### 3.4 Keeping pins alive

- A reference install at a DIFFERENT point is deferred while the view has a pin younger than
  `PIN_INSTALL_DEFER_FRAMES` (30); it lands the frame the pin adopts or stops. A same-point
  extension installs at once (the pin re-keys and continues). A parked result counts as in flight
  for the reactive spawn gate.
- The band ledger survives a point change at half its licences instead of resetting to the
  floor (priced knowledge of the neighbouring orbit's cost structure; the cliff rule still
  quarters a surprise).
- A frame the freeze verdict turns into a reprojection is unstamped as a dispatch
  (`fe_dispatch_frame`/`fe_steps_last`): the wall-clock lift in `motion_wall_cut` was pricing
  never-run work, which is what swung the motion pass budget 50× between consecutive frames and
  flipped the `visible_res` ladder 49 times in 80 s.

### 3.5 Confirmed black path (from the user's log)

Adoption at 364×275 (f=2015, +40.76 s) → next refresh starts at 1456×1102 (f=2019, +40.80 s) →
`prepare` resizes (clears; the content stamp never matches mid-dive) and THEN copies the hold →
a black snapshot, served through the abandons that follow (f=2025 Orbit, 2087 Orbit, 2181 Drift)
until the next adoption. The video is black from +40.79 s.

## 4. Measurement

- `scripts/dive-capture/capdive.sh` from `dive-2p584.kfr` (the recorded start view) with the
  user's session, speed and quality, 3 runs each; `blankscore.py` flat frames + a new
  `flashscore.py` (frame-to-frame mean |Δ| > 60 = a flash).
- Gates: `cargo test -p fractadyne-app`, `--selftest`, `--motiontest`, `--uitest`,
  `--livetest tours/grand-tour.toml --size 480x270`, `--zoomtest` full dive at 4×.
