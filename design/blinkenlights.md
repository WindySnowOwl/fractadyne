# Blinkenlights: the live renderer, shown live

A diagnostic panel that draws what the live renderer is doing, frame by frame: which steps run, in
what order, on which thread or GPU, and what they decide. Three goals, in the order they constrain the
design: help a user understand why the picture behaves as it does; help diagnose it; and above all,
be good to watch.

Part 1 (this document): what is worth showing. Part 2, once that is agreed: how it looks and where.

## Principles

- **Every light is an event or a measurement.** Nothing blinks for decoration. If a dot moves, a
  pass ran; if a needle drops, a refresh was adopted. A panel that animates on its own would teach
  the wrong thing and hide the right one.
- **Sequence is the point.** Events carry the frame they happened on, so the panel can show order
  ("the walk's last pass, then its reading, then the adoption three frames later") and not just
  totals.
- **It reads, it does not perturb.** It draws from state the app already keeps (the frame record,
  `Perf`, the reference and worker channels) and from one new bounded in-process event ring. It must
  cost well under a millisecond a frame on the UI thread, and nothing on the GPU passes it watches.

## What happens in a live frame (the inventory)

| | process | what it does, and the signals already kept |
|---|---|---|
| A | **The frame loop** (UI thread) | input → plan (resolution ladder, iteration ask, budget) → dispatch → present (live, reprojected or held) → drain the readings. Frame interval, CPU body time, frame kind, present kind, interacting. |
| B | **The GPU queue** | 0–N passes a frame: the moving frame, a pin pass (an iteration window), a settle tile, an SSAA sample, a live probe, a split set. Kind, iterations, pixels, GPU ms, passes in flight. |
| C | **The pinned refresh (the walk)** | start → passes through octave bands from 256 to the ask → last pass → its verification reading → adopted, or abandoned (drift, age, superseded, re-keyed). Cursor, ask, each pass's price against the 20 ms target, band licences. |
| D | **The picture on screen** | live, held or reprojected; its age in octaves against the 0.5 target and the 1.0 gate; the reuse-hold waiting out its drift; the resolution of what is shown. |
| E | **The reference pipeline** (threads) | live, lookahead and reactive builds: orbit length, precision, series walk, BLA; installs, overtaken drops, re-keys; the reference's depth against the view's. |
| F | **Arithmetic** | direct → df32 → floatexp (the 2^93 switch), the pre-switch probe, BLA skipping, rebases. |
| G | **Budget and pricing** | the frame budget's ×1.5 climb, the dispatch ceiling, mode and motion rates, admission, retreats from the lethal band. |
| H | **GPU timing** | the three-timer ring (idle, recorded, mapping, ready), readings landing, passes left unpriced. |
| I | **The palette** | the escape-range histogram and the normalization window (fed, shown, held) over it. |
| J | **Settling** | the tiled settle grid, SSAA samples accumulating to 24, convergence. |
| K | **The worker GPU** | frames offered, in flight, delivered, adopted live or into the hold; pins it superseded. |
| L | **The iteration climb** | the auto-iteration ask, the capped fraction, boosts and probes. |

## Scoring

Understanding (U), diagnostic value (D), visual interest (V), each 1–3; and whether the data is there
per frame today (A: ✓ kept, ~ kept in a trace only, ✗ needs plumbing).

| | U | D | V | A | note |
|---|---|---|---|---|---|
| C the walk | 3 | 3 | 3 | ~ | why a refresh takes N frames; the band-by-band sweep is the most watchable thing in the renderer |
| D picture age | 3 | 3 | 2 | ✓ | the sawtooth of age and adoption *is* the zoom blur users notice |
| B GPU queue | 2 | 3 | 3 | ✓ | pass kinds stacked per frame, against the 16.7 / 33 / 50 ms lines |
| E references | 2 | 3 | 3 | ~ | builders racing the view down a depth ruler; installs, drops, re-keys |
| A frame loop | 2 | 2 | 2 | ✓ | the step order of one frame; best as LEDs, not text |
| K worker GPU | 2 | 3 | 3 | ~ | frames travelling from a second device into the hold |
| I palette | 2 | 3 | 2 | ✓ | the window over the histogram (it would have shown today's frozen range at a glance) |
| H timer ring | 1 | 3 | 3 | ✗ | three literal blinking lights |
| F arithmetic | 2 | 2 | 2 | ✓ | a mode lamp and the 2^93 crossing |
| J settling | 2 | 1 | 2 | ✓ | a ring filling to 24 samples |
| G budget | 1 | 3 | 1 | ✓ | numbers more than motion; a diagnostics tab, not the centrepiece |
| L iteration climb | 1 | 2 | 1 | ✓ | |

## The proposal: five instruments

1. **The walk dial** (C). A round dial whose rim is the iteration count on a log scale, 256 at the
   top to the ask, divided into its octave bands. A hand sweeps as the walk's cursor advances; each
   pass paints the arc it covered, coloured by its price against the 20 ms target (cool to hot), so
   a cheap band fills in one stroke and a dear one in several. The verification wait is a pulse at
   the rim; adoption flashes the dial white and it resets for the next pin. An abandoned walk fades
   red with its reason. A small second dial can show the worker GPU's walk beside it.
   *(Added 2026-10-21, on the user's 2^704 view, where the dial sat empty through two minutes of
   settling:)* a still view's walk sweeps the dial too, tagged SETTLE, and so does each
   supersampling sample's, tagged SSAA (`WalkStart`, `WalkPass`): each arc is coloured against the
   time the pass was sized for (the 400 ms settle budget, not the pin's 20 ms), and the walk ends
   when its last pass is timed.
2. **The freshness gauge** (D). A needle for how far the picture on screen lags the view, in
   octaves, with the 0.5 target and the 1.0 gate marked, and the last few seconds as a trace behind
   it: the sawtooth of ageing and adoption that is the zoom blur. Reprojection and hold show as the
   needle's colour.
3. **The GPU oscilloscope** (B, H). A scrolling strip, one column a frame: the frame's passes stacked
   by kind (moving, pin, settle, SSAA, probe, worker), height in GPU ms, against the 16.7, 33 and
   50 ms lines. A pass with no price is drawn hollow. Above it, the three timer-ring lights cycle
   through armed, mapping and ready.
4. **The depth ruler** (E, F). The view's depth as a marker sliding along a log2-magnification
   ruler, shaded by arithmetic (direct, df32, floatexp, with the 2^93 crossing and the pre-switch
   probe's blink). Reference builds launch from the three builders as pucks that land ahead of the
   view (lookahead) or at it (live, reactive); an install pulses, an overtaken build is dropped with
   an X, a re-key ticks.
5. **The light bank** (A, E, J, K, I). The literal blinkenlights: a row of labelled lamps that light
   when their process acts and decay over a few frames: plan, dispatch, present (live / held /
   reprojected), reading landed, pin pass, adoption, reference build ×3, series walk, worker frame,
   SSAA sample, palette feed. The order they light in, frame after frame, is the sequence.

Close behind: the **palette window** (I: the histogram with the normalization bracket gliding over
it), the **settling ring** (J), and a scrolling **event ticker** of the same events in words, for a
user who wants to read what the lights mean.

## The engine room: threads, work, GPUs, memory, throughput

A sixth cluster for the machine underneath the picture: how much of it is busy, and with what.

| quantity | where it comes from | instrument |
|---|---|---|
| **Threads at work** | Named threads exist (`fd-ref-live`, `fd-ref-lookahead`, `fd-ref-hold`, the worker GPU's, `fd-preswitch`, exports), plus the scoped pools that fan out across cores (the reference pick's scoring, the MPFR split products, the series walks). ✗ No one counts them: a small registry of atomic busy counts per role, bumped where work starts and ends. System CPU use is measured already (`sysinfo::read_cpu_times`). | **The core strip:** one cell per logical core, lit by CPU use. Beside it a **constellation**: each busy thread a dot orbiting the core of its role (reference, worker, scoring pool), so a pick fanning out to 16 threads is a burst of dots, and a lone reference build is one. |
| **Computational work** | ✓ Each GPU reading pairs nominal steps with milliseconds; the armed frames' work counters give BLA skips and rebases; reference builds report length, precision and time. | **The tachometer:** nominal steps per second on a log dial (1e9–1e13), with a second needle for the iterations actually executed. The gap between the two needles is BLA's multiplier made visible. An **odometer** rolls the session's total. A smaller dial for the CPU's bignum work (orbit iterations × precision bits per second). |
| **Split between GPUs** | ✓ The window's GPU time per frame; the worker's walk time, frames offered, delivered, and adopted live or into the hold (`LiveStats`). On PLUTO, two cards; elsewhere, two devices on one. | **Two engines:** a busy gauge per device, and frames flowing as dots from each device to a screen glyph, so the share of what you are looking at is visible. A balance bar gives the split as a number. |
| **Memory** | ✓ Process working set and peak (`sysinfo::process_memory`). ✗ GPU memory: wgpu does not report it, but the app knows every large thing it allocates (G-buffers, hold and SSAA textures, orbit and BLA buffers, the worker's reference copy, capped at a quarter of the card), so a ledger of its own allocations gives a true lower bound. | **Two tanks:** system RAM (working set, the peak as a line) and GPU memory as a stacked column by kind, against the card's size. |
| **Pixel throughput** | ✓ Resolution, supersampling and passes per frame give pixel-passes per second; content readings give how many pixels finished (escaped) per walk. | **The pixel stream:** particles flowing into the screen at a rate set by pixels per second, coloured by arithmetic mode, with Mpx/s beside it. Finished pixels fall in brighter than passes still walking. |

## What it needs that is not there

- **An in-process event ring.** Pin starts, passes, prices, adoptions and abandons; reference
  launches, installs and drops; worker deliveries; readings. Today these are trace lines, which are
  opt-in and go to the log file. The panel needs them always, bounded (a few thousand), cheap (a push
  per event), each stamped with its frame.
- **The timer ring's states** exposed per frame (one byte a slot).
- **A thread-activity registry:** an atomic busy count per role, and the size of each fan-out.
- **A GPU allocation ledger:** bytes the app holds on each device, by kind, updated where it creates
  and drops its large buffers and textures.
- **A frame-record history** for the scrolling instruments, a few seconds of the records already
  built each frame (most fields are there now).

## Part 2: an overlay, styled as a retro front panel (the user's choices, 2026-10-19)

### Where it sits and how it is called

- **An overlay on the fractal view**, drawn by the UI pass (an `egui::Area`), never into the fractal
  texture: exports, `--shot` renders and the measured passes never see it. It snaps to any corner of
  the view (bottom-left by default, clear of the status bar and the click-to-zoom reticle) and can be
  dragged; its opacity is a View-menu slider.
- **`B` cycles it** through off → strip → panel → full (and the View menu has the same entry).
  - **Strip:** the light bank and three readouts in one band along the bottom edge.
  - **Panel:** the five instruments and the tile plane.
  - **Full:** the panel plus the engine room beneath it.
- **Help cards** (the understanding goal; asked for on review, 2026-10-19): hovering, focusing or
  tapping any instrument, lamp or readout opens a card saying what it shows, what **healthy** looks
  like and what signals **trouble** (e.g. the picture-age needle living past 1.0; red arcs on the walk
  dial; PALETTE dark while zooming). The mockup carries the first draft of every card's text. A
  **legend** lamp lights a key over the panel while held.
- **Even rows:** the instruments in a row share one height and title line; each draws to the box it
  is given.

### The look

A rack instrument from a 1970s lab: a slate hammertone case (first built as charcoal anodizing with
a brushed grain; see "As built"), four corner screws, engraved labels in small caps, a thin bevel.
Instruments are the period's own:

| instrument | period form |
|---|---|
| light bank | a PDP-8 / Altair row of round lamps under engraved labels: red, amber and green, with a warm incandescent glow that fades over ~150 ms when the event is past |
| walk dial | a **radar PPI scope**: round green-phosphor screen, the iteration count on a log scale round the rim from 256 at the top, octave bands as range rings' arcs; the walk's cursor is the sweeping beam, each pass leaves an afterglow arc coloured by its price (pale green cheap → amber → red over target), the verification wait blips at the rim, adoption flashes the whole screen |
| freshness gauge | a **VU meter**: cream face, black scale in octaves (0 to 2), red zone past 1.0, a mark at 0.5; the needle has meter ballistics. Behind it, under glass, a small phosphor strip of the last few seconds' sawtooth |
| GPU oscilloscope | a **phosphor scope**: green P31 trace on a graticule, one column per frame scrolling left, the passes as stacked bars by kind (bright = priced, hollow = unpriced), the 16.7 / 33 / 50 ms lines on the graticule; above it the three **timer-ring lamps** |
| depth ruler | a **radio tuning dial**: backlit amber glass with a log2-magnification scale, the view as the red pointer gliding along it; scale bands shaded by arithmetic (direct, df32, floatexp) with the 2^93 crossing marked; the reference builders as three small lamps riding the dial ahead of or at the pointer, an install pulsing, an overtaken build blinking out |
| readouts | **nixie tubes** for the headline numbers (depth 2^N, iterations, frame ms), amber **seven-segment** for the rest |
| tile plane | an **amber lamp matrix** behind smoked glass, in the picture's own aspect: the cells this frame's pass covers light up, those done earlier in the same refresh or grid stay lit, and the tile size and tiles per frame read out beneath it in amber (see below) |
| engine room | a **dual-needle tachometer** (nominal and executed steps/s, log scale, redline at the TDR budget) with a **mechanical odometer**; **two engine gauges** for the GPUs with dots running down a glass tube from each to a screen glyph; **thermometer columns** for RAM and GPU memory; an **LED bar graph** of cores with a small round scope beside it where busy threads orbit their role; a **dot-matrix window** where the pixel stream falls |

Type (chosen on review, 2026-10-19, after the user's machinist-tool references: a steel rule and a
Starrett dial indicator):

| role | face | why |
|---|---|---|
| engraved legends: panel and instrument titles, lamp labels, screen captions | **Routed Gothic** (regular; narrow on screens and lamps), SIL OFL 1.1, © 2017 Darren Embry | a tracing of K&E Leroy lettering, the single-stroke drafting and engraving standard; drawn paint-filled (a dark lip above, a faint highlight below) |
| printed dial faces: meter and gauge numerals, scale labels, the odometer | **Barlow Condensed**, SIL OFL | the narrow, slightly rounded DIN-like numerals of a printed dial |
| maker's mark on the plate | Barlow Condensed Bold Italic, in red | as a toolmaker signs a dial |
| nixie readouts: depth, iterations, frame ms, date and time | Share Tech Mono, with the neon glow (the user: keep it) | |

Routed Gothic's 1 is narrower than its other digits, so a number that changes is drawn with every
digit in a cell as wide as a 0; fixed labels are drawn as set. The nixie row sits at the top of the
panel with a clock (`yyyy-mm-dd hh:mm:ss.s`, local time), which stamps any screenshot of the panel.

Colour: warm amber and red lamps, phosphor green, cream meter faces, on charcoal. The price colours
run green → amber → red and also brighten, so they read without colour vision. Everything is drawn as
vectors, so it scales with the display.

### The tile plane (added on review, 2026-10-19)

The user asked for tile size and tiles per frame. The walk dial shows how a refresh is split along
the iteration axis; the tile plane shows how a frame is split in space. The live view does this in
three ways, and the plane draws each one the way it happens:

| how the frame is split | in the code | the plane shows | tile size · tiles a frame |
|---|---|---|---|
| **The whole frame in one pass:** a moving frame, a priced live refresh, each pass of a pinned walk or of a still view's walk (the settle's, or a supersampling sample's) | one full-screen draw | the whole plane flashes on each pass; for a walk, the band (e.g. 4,096–8,192) captions it, named PIN PASS, SETTLE PASS, or SSAA with the samples done. *(2026-10-21, the maintainer: "Tiles instrument seems to always look like a single rectangle".)* A still view's walk pass covers the whole frame but iterates only the pixels not yet escaped: the plane fills from the bottom to that share (`Perf::chunk_running` over the walk's texels, log scale, 0.01% at the floor) and reads it ("RUNNING 1.5%"), so it drains as the walk goes deeper | the resolution · 1 |
| **A split refresh:** a live refresh priced dearer than one frame's share goes out in `k` passes, one a frame, each about `LIVE_REFRESH_MS` (4.5 ms), up to `LIVE_SPLIT_MAX` (8) | `LiveRefresh::Split(k)`, `PinnedRefresh::split`; set `j` = the 16-texel tiles with (x + y) mod k = j (`SPLIT_TILE`, `vs_split_tiles`) | the interleaved sets fill pass by pass (a checkerboard at k = 2, diagonals above), this frame's set brightest | 16 px · tiles ÷ k (about 2,100 at 1455×1102, k = 3) |
| **The settle grid:** a settled frame too dear for one dispatch | `TileGrid`: side √(admission ÷ (iterations × ss²)), at least 16 px, frozen for the grid; 16 to 512 tiles (`TDR_MAX_TILES`, `TDR_TILES_CEIL`); one a frame, centre-out; a tile over the admission goes out as strips (`settle_strip`) | lamps fill from the centre out, the next tile outlined, a stripped tile filling row by row; then SSAA samples pulse the whole plane | the side · 1 (or 1/strips) |

A frame with no pass (reprojected, held, waiting on a reading) leaves the plane dark and says why.
The GPU oscilloscope gets a fourth bar kind (settle tiles), and the light bank gets **SPLIT** and
**TILE** lamps.

The frame record already has `tiled`, `tile_w`, `tile_h`, `tile_pending` and the chunk range. The
event ring needs **grid start** (cols, rows, side), **tile sent** (index, strip of strips) and **split
pass** (j of k). Later, an export's own tiles could drive the same plane while it runs: 2048-px
occupancy tiles with packed tails (design notes in the GPU-tail work).

### Motion: real events, honest ballistics

Every movement starts from a real event or reading. What the panel adds is display physics, the same
for every signal: lamp decay (τ ≈ 150 ms), phosphor persistence (τ ≈ 0.8 s), needle spring and
damping (≈ 120 ms, critically damped, so a needle never shows a value that was not read). A frozen
renderer gives a still panel; a frozen panel next to a moving picture is a bug.

### What feeds it

- **The event ring** (new): a bounded ring (4,096 events, ~32 bytes each) every emitter pushes to
  whether or not the overlay is open, so opening it shows the last seconds at once, and a crash
  report could carry it later. Each event: frame, time, view, kind, two integers and a float. Kinds:
  frame plan, dispatch (by pass kind), present (live / reprojected / held), reading landed, pin
  start / pass / price / adopt / abandon (with reason), reference launch / built / installed /
  dropped / re-keyed (by builder), mode switch, pre-switch probe, worker offer / delivery / adoption
  (live or hold), SSAA sample, settle grid start / tile sent (with its strip), split pass (j of k),
  palette feed, timer armed / read (by slot). The emitters are the
  places that write the `tile`, `ref`, `live`, `gpu` and `accum` trace lines today; they push whether
  or not tracing is on. Builders on other threads push through the same lock (a few events a second).
- **A slim frame history** (new): the last 600 frames (~10 s) of a dozen frame-record fields: time,
  interval, CPU body, GPU ms, nominal steps, frame kind, present, picture age, mode, resolution,
  supersampling. The frame record already computes all of them.
- **The timer-ring states** (new): one byte per slot published per frame by the GPU crate.
- **The thread registry** (new): an atomic busy count per role and the size of each fan-out.
- **The GPU allocation ledger** (new): bytes held per device by kind, kept where the app creates and
  drops its large buffers and textures.
- **Existing:** `sysinfo::process_memory` and `read_cpu_times`, the worker's `LiveStats`, the GPU
  work counters, `Perf`'s rates, budgets and normalization window.

### Cost

The overlay is a passenger: when open it may cost **0.5 ms of UI-thread time a frame at its largest**,
and nothing on the GPU passes it watches. Emitters cost a push each (well under a microsecond) and
run whether or not it is open. The overlay asks for repaints only while its lights are still
decaying (≤ 1 s after the last event), so an idle app still idles. Measured, not assumed: the P-G
rows with the overlay at full size against off, for frame interval and the picture's age.

### As built (2026-10-19)

What the implementation settled, where it differs from or adds to the above:

- **Where it lives:** `crates/fractadyne-app/src/blinken.rs` and `blinken/` (`events`, `registry`,
  `model`, `panel`, `paint`, `help`); the app side in `ui/blinken_overlay.rs`. The model is pure
  and unit-tested (`tests/unit/blinken/`).
- **The third builder is the tour's hold prefetch** (`fd-ref-hold`), lamp REF HOLD. The code's
  "reactive" build is `fd-ref-live` itself, so REF LIVE and REF REACT would have been one lamp.
  Builds carry `(role, id)` from launch (`RecomputeInputs::blinken`) through the build thread
  (built, series walk) to install or drop (`RecomputeResult::blinken`); a marker whose end is
  never seen leaves the dial after 30 s.
- **Picture age** is the zoom harness's own measure: how far the view has zoomed past the picture
  frozen on screen (`ref_cache.frozen_l2`).
- **GPU time**: the iterate timer's and the extra timers' readings, each filed under the kind of
  pass its frame ran (`PASS_OF_FRAME`); the live-refresh timer shares the iterate timer's ticks
  and is not counted twice. The worker's walk time feeds the second engine, and so do its
  supersampling samples: each export pass reports its GPU time on the rendering thread
  (`fractadyne_gpu::pass_sink`), batched per UI frame (`gpu_worker::PassBatch`) into the worker's
  `PassTimed` (added 2026-10-21: the gauge read 0% while the worker rendered a sample every
  ~20 s).
- **Two new GPU sinks** on `MandelbrotParams`, dropped by the worker's twin like the others:
  `timer_states_out` (the ring's slot states, two bits each; the T0–T2 lamps) and `step_out`
  (the armed frame's `StepStats`: loop trips against iterations advanced, the tachometer's BLA
  multiplier).
- **GPU memory** is reckoned from the sizes the renderer allocates (G-buffers, hold and SSAA,
  orbit and BLA, the worker's copy) rather than a ledger of every allocation.
- **CPU per core**: `sysinfo::CoreSampler` (`NtQuerySystemInformation` on Windows, `/proc/stat`
  on Linux), four times a second. The clock: `sysinfo::local_now` (`GetLocalTime` /
  `localtime_r`).
- **Cost**: the panel logs its own UI-thread time every 600 frames it is open (`blinken` log
  lines, with a per-section breakdown), and asks for frames only while something on it moves
  (else once a second, for the clock), so an idle app idles. Measured on the RTX 3080 at full
  size over a 1920×1200 window, in four steps:
  - first build: 1.5–1.7 ms a frame of the panel's own time (each tracked letter was its own
    text shape, three times for the engraving); the P-G rows' frame interval +0.6 ms;
  - one layout per legend (`extra_letter_spacing`), one per nixie row, one per tabular number:
    0.67 ms;
  - a STATIC layer (case, faces, scales, titles, labels, bezels) drawn once and replayed:
    0.25–0.28 ms, but the frame interval still +0.5–0.7 ms, because egui tessellates every
    replayed shape every frame;
  - the static layer kept already tessellated (re-tessellated when the font atlas grows or
    anything it depends on changes): 0.27 ms of its own, the frame interval +0.2–0.4 ms
    (df32-4 17.8 → 18.2 ms, deep-4 18.0 → 18.4 ms), and the picture's age unchanged within
    run-to-run noise (df32-4 1.00 → 1.02 octaves at most, deep-4 0.92–0.96 → 0.94).
  - PLUTO's RX 6800 XT (dev.32, agent v29, off/full interleaved, a 40-octave 4× dive and one
    from 2^80 through the floatexp switch, 2560×1600 display): the panel's own time 0.54–0.64 ms
    a frame (its CPU is slower), the median frame interval +0.2–0.4 ms (19.1 → 19.3–19.5 ms;
    17.8 → 18.1 ms), refreshes completed the same (84–88; 54–55), and the picture's age within
    that card's wide run-to-run spread (1.17–1.89 octaves off, 1.32–2.16 full). The battery on the
    same build: PASS (every step as declared for the card). A `--shot` with the panel open showed
    it drawn correctly there; its log check failed on the startup wall-clock-pricing flaps every
    earlier PLUTO shot has (beta.18, no panel: 32–40 of them; this one 22).
- **Harness hooks**: `--live-panel off|strip|panel|full`, and `--uitest` screens
  `live-panel-strip`, `live-panel`, `live-panel-full`.
- **Hammertone (2026-10-20, on review):** the case is a 512-texel tile generated once
  (`paint::hammertone`: small round dents with defined rims, `(1 − d²/r²)²`, pressed in round the
  tile's edges so it repeats without a seam, lit from the top left with a glint and a speckle of
  flake) and uploaded with a repeating sampler; one texel is one panel unit, so the dents scale
  with the panel, and it lives in the static layer, so it costs nothing a frame. Soft Gaussian
  dents read as smoke at 1:1 and were dropped. Legibility is measured, not judged: every legend
  printed on the case carries a dark halo a pixel round it, the legend colours were raised
  (`ENGRAVE_DIM` #8f8875 → #ccc2a8), and a unit test holds each at 4.5:1 or better against the
  tile's median texel and against its p99.5 glint under the halo; the maker's mark moved onto a
  dark enamel badge (red on the hammertone itself would read at 1.5–2:1).
- **Hammertone, second pattern (2026-10-21, from the maintainer's photograph of the paint):** the
  round dents became the paint's own structure: irregular rounded cells parted by thin dark
  creases, each gently domed, with small round bubbles crowding inside them. Weighted Voronoi
  cells (`d − w`, a slow periodic warp) on the torus give the cells and their border-distance
  field; blurring that field turns every corner an arc; a dome per cell and a hemisphere per
  bubble make the relief, lit as before; the creases and the bubbles' rims are darkened on top.
  Fifteen rounds in a NumPy prototype beside the photograph (rendered in silver to compare the
  pattern, in the case's slate to ship it): high-relief cells read as stone, a strong warp as
  worms, a morphological opening erased the small cells. The case keeps its slate colour, since
  the legends are light, and the same unit tests hold (in the prototype the dimmer legend read
  7.2:1 against the median texel and 5.6:1 against the halo-darkened glint; no seam).
- **Review fixes (2026-10-20):** supersampling passes are lavender on the scope, with their own
  legend entry (they were a green the eye could not tell from moving frames), and the HELD card
  says it is lit steadily while a still view sharpens under Prefer detail (it had called that a
  fault).
- **The tile plane, checked at every tiling (2026-10-20, on review: "only a full or half panel"):**
  the instrument is drawn headlessly in a unit test (egui's own shapes, tessellated, rasterized in
  software) for settle grids of 16 to 1000-px tiles at several stages, strips, splits of 2 to 8
  passes, whole frames, SSAA and idle, and read back tile by tile. The geometry was right; what
  the eye saw was not. Its levels were amber at an alpha over the dark face, and egui premultiplies
  a faint colour in linear light: the plane at 0.04 showed as a quarter-bright brown, done cells at
  0.3 almost matched the pass's own at 0.95, so a two-pass split read as half the plane, then all
  of it. The levels are now opaque mixes into the face (plane 0.07, to come 0.13, done 0.38, the
  pass 0.7 to 1). At the instrument's own size a 16-px tile is ~2 px: grids of tiles under 6 px
  meet edge to edge (with a gap each, a finished grid read as dots), and a split whose 16-px cells
  would be under 5 px is drawn as the same interleave at 5-px cells, the captions keeping the
  truth ("TILE 16 PX", the tiles a frame).

### Build order

1. **Plumbing:** the event ring and its emitters, the frame history, timer-ring states, the thread
   registry, the GPU ledger. Unit tests for the ring (order, bounds, wraparound) and for each
   emitter's event at the trace line it mirrors.
2. **The panel and the light bank:** the overlay, the `B` cycle, corners and dragging, the panel art,
   lamps, nixie and seven-segment readouts. `--uitest` screenshots of each size, looked at.
3. **The three scopes, the meter and the tile plane:** the radar walk dial, the VU freshness gauge,
   the GPU oscilloscope with the timer-ring lamps, the tile plane.
4. **The tuning dial:** the depth ruler with the reference builders.
5. **The engine room.**
6. **The cost check:** the P-G rows with the overlay full against off, on the RTX 3080 and on PLUTO.
