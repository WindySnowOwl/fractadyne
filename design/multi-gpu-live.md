# Several GPUs for the live view and live zoom — design

Status: **design (2026-10-07), nothing built.** This replaces Phase 3 of `design/multi-gpu.md`
("the live view — research only"), whose premise has changed (§1). The plan's Phase 2 (per-device
renderer state) is a prerequisite and is shared. Line references are to `feat/remote-rendering`
at `6687dac`.

## 1. Why the live view after all

`design/multi-gpu.md` §1 concluded that a second GPU pays for exports and tours but not for deep
live navigation, which it took to be bound by the reference orbit on the CPU. Two things say
otherwise now:

1. **A moving deep frame is GPU-bound** (measured 2026-09-17, RTX 3080). `--divetest` at 1e20,
   1280×720: p50 88.1 ms a frame, build95 0.1 ms, so ~100% GPU. The motion resolution sat at its
   floor of 0.30 (82,944 px), a third of the 3080's ~262k-pixel occupancy knee. TODO.md:9349-9351
   records the same conclusion: the one device is saturated at realistic frame sizes, so the cheap
   single-GPU 1.5–2× does not exist and a second GPU is the honest path to 2×.
2. **The reference has become cheap.** The precision schedule, products split across cores, the
   pick sharing the centre's build, and the live reference lookahead along the glide's trajectory
   (render.rs:2535-2695) mean references arrive ahead of a zoom. At 1.82e5001 a 4K frame's reference
   built in ~1.6 s with MPFR, once, for many frames.

So during a deep zoom, extra GPU work buys real refreshes more often: held frames are magnified
less and the motion resolution can rise. When the view settles, it buys supersampling. At depth
each of the 24 samples is a whole walk of the frame.

## 2. How a live frame is made today (what the design depends on)

1. **One device, one submission.** eframe owns the only device (main.rs:331-437). Each view's
   `prepare` records its iterate, resolve and accumulate passes into egui's per-frame encoder.
   `paint` then draws `fs_color` or `fs_present`, and one `queue.submit` presents
   (lib.rs:2764-3672; egui-wgpu winit.rs:378-549). **No pixel data is ever read back in the live
   path.**
2. **The G-buffer is the interface between iterating and colouring.** It is `iter_tex` plus
   `aux_tex`, both `Rgba32Float`: 32 B per texel at res·ss per axis (lib.rs:31, 1157-1173).
   `fs_color` reads neighbourhoods (relief 3×3, palette-AA +1). When reprojecting it reads anywhere
   in the frozen frame, plus a whole-texture average (wgsl:3373-3530). **So colouring needs the
   complete G-buffer on the presenting device.**
3. **Moving frames are always ss = 1** (central.rs:1238-1247). They refresh by one of three paths:
   - **Reprojection.** The frozen frame is held and reprojected (`reuse_hold`, `will_reproject`,
     render.rs:7787-7812; freeze 8921-8987). The transform is `fs_color`'s `uv_scale`/`uv_off`.
   - **Live refresh** (df32 only). One priced pass of at most 4.5 ms (render.rs:7628-7786,
     10393-10462).
   - **Pinned refresh.** A chunked walk at a captured view (`PinnedRefresh`, render.rs:9612-9646)
     runs while the display shows the hold snapshot. It is adopted whole once complete
     (`pin_verdict`, 9871-9931); at most 2 are in flight.

   The motion resolution comes from a frame-interval AIMD (17/24 ms), a rate cap and the
   `visible_res` ladder (render.rs:7367-7583).
4. **Settle.** The AA ramp raises ss, then progressive supersampling takes over at
   log2 magnification ≥ 33 (main.rs:13956-14221):
   - 24 Halton-jittered samples;
   - each one re-renders the whole frame;
   - each sample's **colour** frame folds into a running average,
     `avg + (frame − avg)·inv_weight`, with the weight in a uniform (lib.rs:3582-3634;
     wgsl:3553-3559).
5. **The reference on the GPU** is one buffer per view: the orbit, then the BLA tree (16 B a
   sample, 64 B a node, ≈9× the orbit). It is uploaded when `orbit_id` or the BLA tree changes
   (lib.rs:3081-3104). A moving reference is capped at `LIVE_REF_CAP` = 256k samples (≈37 MB with
   its BLA tree). The device cap is ≈7.45M samples (1 GiB).
6. **Learned state is per VIEW, and implicitly this one device.** That covers the step budget,
   mode rates, refresh prices, chunk band ledger, dispatch ceiling (one adapter's calibration,
   calibration.rs:70-185), dead-man and timing witness (main.rs:1120-1823,
   render.rs:11371-11533, timing_witness.rs).
7. **Whole-frame counters** (228 B: escape min/max, escaped count, gradient and escape histograms)
   feed live normalization and the verified-present content check (render.rs:5051-5409,
   9755-9866). Their readback is paired with one pass.
8. **Losing the device ends the process** (main.rs:6083-6121).
9. **The export renderer is already device-agnostic.** It builds its own pipelines on any device,
   renders tiles with `px_offset` into its own textures, prices its own step-bounded passes and
   reads tiles back to the CPU (export.rs:1541-1580; `design/multi-gpu.md` §2.2).

## 3. The shape of the design

**Principles.**
1. **The window's device presents everything.** A second GPU (a *worker*) never touches the
   screen.
2. **Only whole products cross between devices:** a complete G-buffer for a captured view, or a
   partial colour average. Never chunk state, never a half-walked frame. These are exactly the two
   places the live path already adopts finished work (§2.3 pins, §2.4 the fold).
3. **A job carries everything that defines its pixels:** the live iterate uniform verbatim, the
   reference (`orbit_id`), the geometry and the jitter. The same job on the window's device would
   produce the same G-buffer, byte for byte on the same GPU class. So the single-GPU path is the
   test oracle for every multi-GPU result.
4. **Latency-tolerant by construction.** Results are adopted through paths that already show a
   held or reprojected frame while they wait.
5. **A worker failure** (device loss, hang, error) drops the worker and leaves single-GPU
   rendering. It never ends the process.

### 3.1 The worker (`GpuWorker`)

- **Device.** A headless device on another adapter, with its own `wgpu::Instance` (wgpu 24.0.5
  shares nothing across devices; eframe owns the window's instance).
- **Thread.** Its own thread, submitting and polling its own queue.
- **Per-device state:**
  - its pipelines;
  - one reference buffer per `orbit_id` it has been sent;
  - the dispatch ceiling for **its** adapter;
  - its own capability flags (`TAIL_DF32` etc., process-wide atomics today, lib.rs:175-211);
    **2026-10-07: not needed.** Those three atomics are configuration switches (`--set` tunables),
    the same for every device, not detected capabilities; see §8;
  - its own pass pricing.
- **Rendering.** It uses the export path's tiling and TDR-safe pricing, fed the **live** job's
  iterate uniform rather than an export request's. ⚠The export and live paths set some fields
  differently (iteration budget, series and BLA choices). The job must carry the live values, and
  L1's byte-identity test is what catches a mismatch.
- **References.** Uploaded once per `orbit_id`: ≈37 MB for a moving reference, up to ~1 GiB for a
  deep settled one. The orbit cap becomes the minimum over all devices, as the farm already does.
- **Output.** The G-buffer, plus the counter block (228 B) and the pass timings:
  - `iter` channel: 16 B/px;
  - `aux` channel: another 16 B/px, sent only when the colour method reads it.
- **Interface:**
  - `submit(job) → receiver`;
  - `cancel(generation)`: a new view generation drops stale jobs;
  - `health()`: alive or lost.

### 3.2 Transfer

- **Path.** `map_async` on the worker, then `queue.write_texture` on the window's device. The upload
  runs on a helper thread (`wgpu::Queue` is `Send + Sync`), so the UI thread copies nothing.
- **Sizes:**
  - A motion refresh at ss = 1, at 2560×1600 and res 0.5, is 1.02M px × 16 B = **16 MB**
    (32 MB with aux).
  - A settle delivery is a colour partial average at base resolution. As `Rgba16Float` (8 B/px)
    that is **33 MB** at 2560×1600, sent every few samples rather than every sample.
- **Expected cost:** a few ms of PCIe each way, plus about one frame of latency. L0 measures it.

### 3.3 The two adoption points

- **(A) Settle samples.** A worker's partial average of m samples folds into n with
  `inv_weight = m/(n+m)`. `fs_accum` already computes `avg + (frame − avg)·inv_weight`, so no
  shader change is needed: the fold pass reads the uploaded partial average in place of this frame's
  colour.
- **(B) Motion refreshes.** A worker G-buffer for a captured view is adopted the way a completed
  pin is (render.rs:9104-9119). It becomes the frozen frame, which `fs_color` reprojects to the
  current camera.

## 4. What to build, in order

### L0 — measure (no change to the live path)

A headless `--multigpu-probe`, for each pair of adapters:
- creates both devices in one process: on PLUTO, NVIDIA and AMD Vulkan together on Windows;
- measures each device's throughput on live-shaped work: the divetest window at 1e20, and the
  interior-df32 fixture at motion resolution;
- measures transfer: readback and upload of 4, 16 and 64 MB, and the latency from submit to
  adoption;
- renders the same jobs on both devices and counts differing pixels, i.e. class differences at
  live sizes.

**Two devices on one adapter.** wgpu can open two devices on the same adapter (the farm's
`--farmtest` client D already runs `--adapters N,N`). So the whole worker path can be built and
tested **on this machine**, on the RTX 3080 twice. That gives no speed-up, but it is the
byte-identity rig. PLUTO is for speed and mixed-class checks (ask before queueing).

**Gate:** if transfer and per-frame overhead eat the gain at live sizes, stop after L2.

### L1 — the worker and per-device state

- Every per-process GPU value becomes per-device. This is shared with `design/multi-gpu.md`
  Phase 2: that plan's §2.3 list, plus the orbit cap (minimum over devices), calibration per
  adapter, capability flags and the timing witness.
- `GpuWorker` as in §3.1.
- A setting "Use other GPUs for the live view", **off by default** until validated.
- **Acceptance:**
  - A job rendered on a twin device gives a G-buffer byte-identical to the window's device
    rendering the same job.
  - An instrumented worker loss (`FRACTADYNE_WORKER_LOSE_AFTER=n`) leaves the session running on
    one GPU, with a log line saying so.

### L2 — settle supersampling on both GPUs (recommended first)

- When the view settles (`drive_accumulation`), the worker takes every other sample of the 24
  (its own Halton indices).
- It renders each one whole: iterate, then colour with the **window's colour uniform verbatim**,
  so normalization is identical.
- It folds its samples locally and ships a partial average plus its count every few samples.
- The window's device keeps rendering its own samples and folds the worker's partials (§3.3 A).
- A new settle (`view_gen`) cancels the worker's job by generation.
- **Mixed classes are safe here.** Every output pixel averages samples from both GPUs, so the
  class differences (sparse boundary pixels) average inside each pixel instead of forming regions:
  no seam, and no flicker, since the average only converges. This is option (b) of
  `design/multi-gpu.md` §3, applied to the live view.
- **Acceptance:**
  - On a twin device, the converged image matches a single-GPU run within 1/255 per channel. The
    fold order differs, so it can't be bit-equal.
  - On PLUTO, the time to 24 samples falls to about 1/(1 + s₂/s₁) of one GPU's, where s₁ and s₂
    are the two cards' sample rates.
- **Why first:**
  - Latency doesn't matter on a settled view.
  - The gain is clear: at depth each sample is a whole walk.
  - It exercises the worker, the transfer and adoption end to end before any motion work.

### L3 — motion refreshes from the worker

- **Where the worker renders.** It renders refreshes at **predicted** views: where the camera will
  be when the result can be adopted, which is render time plus transfer, learned per device. The
  prediction is `Trajectory::Glide` (render.rs:274-300, the closed form of the app's own easing)
  or a tour's playback, the same oracle the reference lookahead already uses.
- **Adoption.** The window's device adopts, as the frozen frame, the newest worker frame whose view
  is closest to the camera. Its own pins and live refreshes carry on. The display always
  reprojects the newest real frame from either GPU.
- **Effect.**
  - The real refresh cadence is about the sum of both devices' rates, so held frames are magnified
    less.
  - At the same cadence each device gets more time per refresh, so the AIMD motion resolution can
    rise (render.rs:7484-7529).
- **Counters travel with the frame.** The verified-present check and live normalization read the
  worker's counter block as if the pass had been local.
- **Mixed classes are masked in motion.** The farm's flicker check measured it: switching GPU class
  on every frame of a moving zoom added 0.09% of pixels changing by more than 48, against the
  motion's own 1.57%. When motion stops, settle work stays on the window's device or goes through
  L2's averaging, so **a held view never alternates between classes.**
- **Acceptance:**
  - On a twin device, every adopted worker frame is byte-identical to a pin of the same captured
    view on the window's device.
  - `--zoomtest` on PLUTO: the real-refresh cadence rises, held frames are magnified less, and
    there are no more >33 ms hitches than before.
  - The motion flicker metric (the farm's method: change from GPU switching alone, against change
    from motion) stays within the band measured above.

### L4 — the dual view's Julia panel on the worker (optional, small)

The Julia panel is an independent view (`Renderer.views`, keyed by `view_id`, lib.rs:1071). In
dual view the worker could render that panel whole and ship it. That is a split by view: no
panel ever mixes GPU classes.

### Not planned

- **Splitting one moving frame by area across GPUs.** Both halves must arrive before the frame can
  show, mixed classes would seam at the boundary, and one frame would have two budgets.
- **Sharing chunk state between devices.**

## 5. Mixed-class policy for the live view

- **Same class** (two identical cards, or the twin test): everything is allowed, and results are
  byte-identical.
- **Different classes** (PLUTO): L2 averages inside each pixel, and L3 works only in motion. By the
  measurements above both are safe, and nothing ever presents a held view that alternates between
  classes.
- The setting lists which GPUs are in use.

## 6. Risks

- **Two vendors' drivers in one process.** L0 tests this first. The fallback is a worker process
  (the farm's child model), with the reference shipped as an orbit blob. That costs latency: fine
  for L2, probably too slow for L3.
- **Transfer latency eating L3's gain.** A late frame is still a real frame, reprojected, so it is
  no worse than today's hold; only the gain shrinks. The prediction aims the worker where the camera
  will be.
- **Memory bandwidth.** 16–65 MB copies per delivery. They run on the upload thread, never the UI
  thread.
- **Budget machinery is per view.** The worker prices its own passes (the export path's pricing),
  so the live controllers keep reading only the window device's passes.
- **The RX 6800 XT's open device-loss issue (#1).** Worker isolation (§3 principle 5) is what makes
  a loss survivable.
- **Power and thermals** with both cards loaded (also noted in `design/multi-gpu.md` §4).

## 7. Decisions needed

1. **Order:** L2 (settle supersampling) first, as I recommend, or L3 (motion) first?
2. **Development rig:** a twin device on this machine's RTX 3080 for byte-identity, and PLUTO for
   speed and mixed-class checks (each queued run asked first)?
3. **Where:** on `feat/remote-rendering`, which already has `--adapter`, `--list-adapters` and
   `gpu_choice`, or a new branch off it?

**Answered 2026-10-07:** L2 first, the twin-device rig, on `feat/remote-rendering`.

## 8. L2 as built (2026-10-07)

**How to run it.** `--worker-gpu same | N | NAME` (a dev flag; there is no setting yet). `same` opens
a twin device on the window's adapter, the test rig. Otherwise it takes a number from
`--list-adapters` or part of an adapter's name. Without the flag nothing changes.

**What it is.**
- `gpu_worker.rs`: a headless device with its own `wgpu::Instance` and thread. It renders whole jobs
  through `render_export`, so it has its own tiling, TDR-safe pricing and pipelines.
- A job is one jittered sample of the settled frame. It is built from sample 0's confirmed
  `MandelbrotParams` by `params_to_request_exact`: the frame's own normalization map, palette
  anti-aliasing and custom formula. `ExportRequest::jitter` places it where the live view would:
  `px_offset = jitter · ss`.
- The worker's loss is its own. The device-lost and error callbacks mark it dead and log it. The app
  drops it, gives back the sample index it held, and carries on with one GPU.
  `FRACTADYNE_WORKER_LOSE_AFTER=n` simulates a loss for tests.

**Where it differs from §4 L2, and why.**
- **Sample indices are shared, not split.** The window's device and the worker take the next free
  Halton index when each is ready, not every other one. Two cards of different speeds both stay
  busy. A failed or cancelled sample gives its index back.
- **Each sample is shipped, not a partial average.** Every fold then uses the one weight rule,
  `1/(n+1)`. At most one fold runs per frame, because the folds share the accumulator's weight
  uniform.
- **A sample travels as floats (16 B/px), and the window's device rounds it.** A local sample
  reaches the average through that device's write into the 8-bit `frame` target. That write rounds
  the way the vendor rounds, not as `round(v · 255)`. On the RTX 3080 (Vulkan), a float ramp
  written to `Bgra8Unorm` came out one lower than `round(v · 255)` in 8,351 of 262,144 channels,
  and never higher. The first version rounded worker samples on the CPU, and its converged image
  read 1/255 brighter on about 10% of the pixels, all in one direction. So the window's device now
  writes the floats into a `frame`-format target with the present pipeline's blit, which is a
  `textureLoad` and that same write. Every sample in the average is then rounded by one device's
  hardware, whatever vendor made it.
- **Generations are run ids.** Each accumulation run has an id. A run that ends cancels the job in
  flight, and an answer from an old run is dropped.

**Not needed: per-device capability flags.** `TAIL_DF32`, `TILE_OCCUPANCY` and `TILE_PACK` are
configuration, the same for every device. The export path keeps no other per-process GPU state
(the remaining statics are a trace switch and the pipeline-constant maps). Nothing had to become
per-device for L2.

**`--shot` waits for a due run to begin** (`Perf::accum_due`). Its gate waited only on an *active*
run. Between the settle and `accum: begin` it passed, and a 9.3e78× shot was written 80 ms after
`begin`, an unaveraged frame.

**Tests.**
- `--selftest-filter worker` (4 checks):
  - a twin device's jittered sample is bit-identical to the window's device rendering the same
    request;
  - control: the jitter changes the sample;
  - control: a whole-pixel jitter is a whole-pixel shift (sign and units);
  - a cancelled run's job answers without a sample, even when the cancel arrives before the job,
    and the next run's job renders. It went red with the queued-cancel branch disabled.
- `--shot` A/B at 1600×1000, with and without `--worker-gpu same`, comparing view 0's pane only:

| View | Solo: 24 samples | With the twin worker | Worker's share | Pane pixels differing (382,100) |
| --- | --- | --- | --- | --- |
| 9.3e78× minibrot (4 runs) | 68.6 s | 18.0 s | 18 of 24, ~0.78 s each | 11 to 140 (max 3 to 5/255); solo vs solo: 0 |
| 6.8e3999× Misiurewicz | 24.3 s | 4.2 s | 20 of 24, ~0.13 s each | 0 |
| 9.3e78×, worker lost after 3 jobs | — | 63.0 s | 2 of 24 | 71 (max 3/255) |

  The CPU-rounded first version differed at 53,680 pixels of the 9.3e78× pane, 23 of them by more
  than 1.

  ⚠**The worker arm is not reproducible run to run, though uncontended solo is.** Explained on
  2026-10-07, at the 9.3e78× view, where every differing pixel sits within a few pixels of
  interior (102 of 140 within 2 px):
  - The worker is deterministic. Each sample index hashed the same in three runs (the
    `#hash` on its log line), and the template (sample 0's iterate inputs, now logged) was
    identical.
  - Nothing is lost or doubled. The GPU reports its own fold count, and the app checks it at
    convergence: "the average holds all 24 samples".
  - **The live path's samples move with GPU timing.** Two solo runs (no worker) sharing the GPU
    with each other differ from an uncontended solo by 192 and 251 pixels (max 5/255). A
    "shadow" worker whose samples are thrown away (`FRACTADYNE_WORKER_SHADOW=1`) moves the solo
    image by about 100. This is pre-existing and not L2's. The likely mechanism is the live path's
    timing-priced choice of passes, which (as noted before) differ at a few hundred pixels; that
    part is unproven.
  - The live and export paths disagree, sample for sample, at boundary pixels: moving one index
    from the window's device to the worker moved 264 pixels (max 6/255).

  None of this shows away from interior (the 6.8e3999× view, with none, matched to the pixel), and
  all of it is within a few 1/255 at under 0.1% of a pane.
- `FRACTADYNE_WORKER_LOSE_AFTER=3`: the worker dropped out after its jobs, the app said so ("one
  GPU from here"), gave the lost job's sample back to the window's device, and converged.

**⚠The speed-up on ONE GPU is a finding, not a result.** A twin on the same adapter should add
nothing, yet it finished the run 3.8× and 5.8× sooner. That means a live settle sample takes far
longer than the GPU work in it: 2.9 s against 0.78 s, and 1.0 s against 0.13 s. The live path paces
samples by its frame budget; the export path has no such pacing. Every timing here was taken while
a farm render loaded the same GPU, and time-slicing favours big batches over frame-paced work. So
these ratios are an upper bound until they are re-measured on an idle machine. If they hold, a
faster single-GPU settle is open, at the cost of how quickly the first input after a settle can
interrupt a long dispatch.

**The live pace is set by frames, not by the GPU.** A live sample takes ~158 frames (~3 s here)
whatever else the GPU is doing: a 100 ms dispatch budget, another process's shot, and the worker
all left it unchanged. That is why a twin on the same card sped the run up.

**Two more fixes found on the way.**
- `--shot` captured an unaveraged frame when a run restarted (live normalization settling the map
  restarts it, up to three times in 75 ms): the restart frame passed the `!allowed` arm with no run
  active, and `accum_due` read false there. A run is now due whenever the view qualifies.
- Every run restarts once about 3 s in, in solo too, and re-renders sample 0 (one sample's time
  lost). Pre-existing; not changed here.

**Idle re-measure (2026-10-07, after the farm render).** Unchanged: 9.3e78× solo 68.4 s → twin
17.6 s (3.9×); 6.8e3999× solo 24.2 s → 4.1 s (5.9×). The contention had not inflated the ratios.

**PLUTO (2026-10-07, field agent v18, build `g8bd5929`).** RX 6800 XT and RTX 3070, both Vulkan, in
ONE process on Windows: no errors, no device loss. That settles §6's first risk.
- The twin self-test passes on each card (`--adapter 1` and `--adapter 2`): 4/4 each.
- `--shot` at 1600×1000, 24 samples:

| View | Window card | Alone | Other card as worker | Worker's samples |
| --- | --- | --- | --- | --- |
| 6.8e3999× | RX 6800 XT | 29.0 s | 3.3 s (8.8×) | 20, 130 ms each on the 3070 |
| 6.8e3999× | RTX 3070 | 27.5 s | 4.8 s (5.7×) | 19, 198 ms each on the 6800 XT |
| 9.3e78× | RX 6800 XT | 95.0 s | 13.3 s (7.1×) | 20, 512 ms each |
| 9.3e78× | RTX 3070 | 78.5 s | 13.1 s (6.0×) | 20, 493 ms each |

  "The average holds all 24 samples" in every worker run.
- **Mixed classes, the image.** The two cards alone differ at the 6.8e3999× view on 72% of
  pane pixels (mean 2.3/255, max 23; the 6800 XT +0.2–0.35/255 brighter): fine grain across the
  detailed filaments, faint arcs in the smooth background, nothing visible side by side. At 9.3e78×
  they differ on 9% (max 9). A mixed run lands between the two (window alone vs mixed: 56% and
  0.5%), the per-pixel averaging §5 predicted: no seam, no region from either card.

**⭐What this says to do next.** §4's acceptance expected ~1/(1 + s₂/s₁) of one GPU's time; the
measured 6–9× is far beyond it, because the worker is not frame-paced and the live path is (~158
frames a sample, §8 above). So the largest lever is single-GPU: render the settle's samples off the
frame pacing (on the window's own device, or a twin of it), which every user gets. A second GPU then
adds its own sample rate on top.

**Pacing, step 1 (2026-10-07): a new sample keeps the band ledger.** The tile trace of one sample
at 9.3e78× showed the mode-2 walk crossing ~9 iteration bands, each re-opened at the 256-iteration
floor and doubled once a frame: ~90 of the sample's 158 frames were ramps. The cause: the walk's sig
includes the supersampling jitter (right — the per-pixel state is position-specific), and a sig
change also cleared the ledger, although the ledger is the orbit's (it already survives pins and
same-sig restarts). `chunk_restart_clears_ledger` now spares a jitter-only change. Output-neutral
(the chunked iterate's bit-identity contract); measured pixel-identical:
- 9.3e78×: 158 → 81 frames a sample; the settle 68.4 → 35.9 s (1.9×).
- 6.8e3999×: 24.2 → 9.0 s (2.7×).

**What is left.** Every pass now runs at the cap: `EXPLICIT_STEPS_CEIL` (6e10 nominal steps,
pixels × iterations over the WHOLE frame) — 125,164 iterations at 551×870. A pass costs ~4–6 ms of
GPU; the frame is ~18 ms (the swap chain's vsync wait, no fps cap), so the walk is one pass per
displayed frame and the card idles ~70%. At the tail only the interior's chains run, latency-bound
(~40 ns an iteration), so a sample's GPU work (~0.4 s) is already near its floor; the pacing is the
remainder. Both ways past it touch the dispatch-size safety model:
- **Count nominal steps over the pixels still running** (a live `CTR_CHUNK_RUNNING`-style count,
  monotone within a walk so a stale reading is an upper bound), charged at least the occupancy knee
  as the calibration ceiling already does: the same worst-case model, an honest pixel count. ~1.8×
  at this window, far more at 4K. Needs the shader count, its readback, and a deliberate revision of
  the "a settled chunked pass stays inside ONE dispatch budget" selftest's units.
- **Several passes a frame** while accumulating: each pass inside the ceiling, but one submission's
  sum is not, which the serialized walk exists to prevent.

**Pacing, step 2 (2026-10-07, user: "count only pixels still iterating"): charge the pixels still
running.** A live mode-2 chunk pass now counts the pixels it leaves running (`count_running`, the
former `rn_pad1`; the same `CTR_CHUNK_RUNNING` the export's step-capped passes fill), and the
readback carries the walk's number (`MandelbrotParams::chunk_walk`). A settled pass is charged
`max(running, knee)` texels instead of the whole frame (`walk_charged_px`), and only from a count of
the SAME walk at an earlier cursor (`walk_running_feed`; every restart, pin start and pin discard
bumps the number, and a cursor found behind its reading drops the bound). The worst case is the
calibration ceiling's own model: below the knee a pass is latency-bound and charged the knee. No
knee for the adapter, no count, or `--set CHUNK_CHARGE=0`: the whole frame, as before.
Decision (user, 2026-10-08): `CHUNK_CHARGE` stays ON by default while the timing-dependent difference below is investigated; `--set CHUNK_CHARGE=0` remains the off switch.
- 9.3e78×: 81 → 48 frames a sample; the settle 35.9 → ~21 s (68.4 s before step 1: 3.2×).
- 6.8e3999×: 9.0 → 5.3 s (24.2 s before step 1: 4.6×), pixel-identical.
- At 1600×1000 the pane is 479k px, so the knee (262k on the RTX 3080, 524k by default and on the
  RX 6800 XT) caps the gain near 1.8×; a 4K pane's late passes can grow far more.

**A walk's identity now includes its SERIES SEED** (`series_seed_bits`). Found while checking
step 2's images: at 9.3e78× the series skip is refined (5513 → 5535, with a new reference) a few
frames after the settle walk starts, and the walk carried on, its pixels seeded from the old series
while every later walk seeds from the new one. A supersampling run whose quick colour restarts
reused that settle walk as sample 0 converged 79 pixels away from one that re-walked it. A
pre-existing bug; it touched ordinary settled frames too.

**⚠Open, pre-existing: the live walk is not reproducible under every timing.** With live
normalization OFF (the sensitive test: the palette cycles fast through the deep counts), identical
settings sometimes converge a few dozen pixels apart (≤ 3/255): uniform 209k-iteration windows with
`CHUNK_CHARGE=0` gave 65 and 16 pixels against the old build, 78 between the two runs; default
windows did not show it here, but two runs sharing the GPU did (192/251, earlier). Longer passes
make it likelier, so step 2 surfaces it more often (1 run in 5 at default normalization-off; with
normalization on, 0–3 pixels). Ruled out: window size and placement (`--set CHUNK_SHUFFLE=1`,
deterministic uneven windows, is pixel-identical), the colour mapping, the series seed (fixed), the
running count's data flow (no shader code reads counters), and every iterate input in the frame
record. Next: read back each folded sample's iteration texture and hash it, so two runs name the
sample that differs.

**A run begins only once the iteration limit and the colour range are decided (2026-10-07, user:
"start supersampling only after the iteration limit stops rising").** Asked "is it speckled?" at a
user's 2.3e11× view (df32, an escaping 6,286-long reference): the converged 24-sample average keeps
a fine grain (128 samples halve it, as 1/√N predicts), but for minutes the screen showed single
samples, because every run restarted. Two causes, both in the colour signature a run is held to:
(1) the adaptive limit climbed ×1 → ×42.9 and fell back to ×10.5, and every raise re-decides the
colour range at the new ask; (2) at a fixed limit a run began before the range had its whole-walk
reading, so each jittered sample's partial reading nudged the unlocked range. The reading both
decide on comes from the walk's last pass, and the counter readback arms only when idle, so about
every other pass goes out unarmed; when the last one did, nothing re-dispatched on a still view,
and the restarted runs were what retried it.
- `accum_inputs_pending`: on a chunk-walked settle a run waits until the climb has a verdict at
  the planned budget (`Perf::iter_verdict`: a reading at that budget that left the boost alone) and
  a reading of the whole walk at this view has landed (`walk_read_sig`). It gives up after
  `ACCUM_DECIDE_WAIT_FRAMES` (120) idle frames with a `⚠… undecided` line. A one-pass settle
  begins as before.
- `tail_read_retry`: a finished settled walk re-sends its empty tail (fresh probe nonce, as a
  finished pin does) until that reading lands, at most `TAIL_READ_RETRIES` (30) per walk.
- ⛔`read_complete` is not "the walk was read": a moving frame's one-pass preview is complete by
  definition, and the first build counted the startup [0, 253) preview (no escapes) as the view's
  reading, so the tail stopped re-sending. Only a reading the GPU published as REACHING ITS ASK
  counts (the maxiter reading's own condition).
- Measured at the user's view, single view 1431×1102 as they run it (normal launch, scratch copy
  of their session; `--shot` forces dual view): before 20 begins, converged at 122.5 s; after ONE
  begin at 28.1 s (limit 122,295), converged at 85.2 s. The climb itself ended at 25.6 s instead
  of 44.2 s, with the same readings at every step: nothing restarted under it, and each step's
  reading came within a few frames. Lethal-band passes 9 → 3. In dual view (`--shot`): 22 begins
  and 59.0 s before, one begin and 46.1 s after, converged images 3 px apart by 1/255.
- Left: 24 samples there take ~57 s (2.4 s each: a 122k df32 walk with rebase storms; the running
  pixel charge is mode 2 only), and each climb step re-walks from 0 although a raise only extends
  the pixels still running.

**Next.**
1. Find the timing-dependent difference (per-sample hashes, above).
2. A setting in place of the flag.
