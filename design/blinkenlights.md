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
  - **Panel:** the five instruments.
  - **Full:** the panel plus the engine room beneath it.
- **Tooltips** on every instrument and lamp say what it shows and what it means (the understanding
  goal); a **legend** lamp lights a key over the panel while held.

### The look

A rack instrument from a 1970s lab: charcoal anodized panel, a fine brushed grain, four corner
screws, engraved labels in small caps, a thin bevel. Instruments are the period's own:

| instrument | period form |
|---|---|
| light bank | a PDP-8 / Altair row of round lamps under engraved labels: red, amber and green, with a warm incandescent glow that fades over ~150 ms when the event is past |
| walk dial | a **radar PPI scope**: round green-phosphor screen, the iteration count on a log scale round the rim from 256 at the top, octave bands as range rings' arcs; the walk's cursor is the sweeping beam, each pass leaves an afterglow arc coloured by its price (pale green cheap → amber → red over target), the verification wait blips at the rim, adoption flashes the whole screen |
| freshness gauge | a **VU meter**: cream face, black scale in octaves (0 to 2), red zone past 1.0, a mark at 0.5; the needle has meter ballistics. Behind it, under glass, a small phosphor strip of the last few seconds' sawtooth |
| GPU oscilloscope | a **phosphor scope**: green P31 trace on a graticule, one column per frame scrolling left, the passes as stacked bars by kind (bright = priced, hollow = unpriced), the 16.7 / 33 / 50 ms lines on the graticule; above it the three **timer-ring lamps** |
| depth ruler | a **radio tuning dial**: backlit amber glass with a log2-magnification scale, the view as the red pointer gliding along it; scale bands shaded by arithmetic (direct, df32, floatexp) with the 2^93 crossing marked; the reference builders as three small lamps riding the dial ahead of or at the pointer, an install pulsing, an overtaken build blinking out |
| readouts | **nixie tubes** for the headline numbers (depth 2^N, iterations, frame ms), amber **seven-segment** for the rest |
| engine room | a **dual-needle tachometer** (nominal and executed steps/s, log scale, redline at the TDR budget) with a **mechanical odometer**; **two engine gauges** for the GPUs with dots running down a glass tube from each to a screen glyph; **thermometer columns** for RAM and GPU memory; an **LED bar graph** of cores with a small round scope beside it where busy threads orbit their role; a **dot-matrix window** where the pixel stream falls |

Colour: warm amber and red lamps, phosphor green, cream meter faces, on charcoal. The price colours
run green → amber → red and also brighten, so they read without colour vision. Everything is drawn as
vectors, so it scales with the display.

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
  (live or hold), SSAA sample, palette feed, timer armed / read (by slot). The emitters are the
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

### Build order

1. **Plumbing:** the event ring and its emitters, the frame history, timer-ring states, the thread
   registry, the GPU ledger. Unit tests for the ring (order, bounds, wraparound) and for each
   emitter's event at the trace line it mirrors.
2. **The panel and the light bank:** the overlay, the `B` cycle, corners and dragging, the panel art,
   lamps, nixie and seven-segment readouts. `--uitest` screenshots of each size, looked at.
3. **The three scopes and the meter:** the radar walk dial, the VU freshness gauge, the GPU
   oscilloscope with the timer-ring lamps.
4. **The tuning dial:** the depth ruler with the reference builders.
5. **The engine room.**
6. **The cost check:** the P-G rows with the overlay full against off, on the RTX 3080 and on PLUTO.
