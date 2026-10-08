# Several GPUs in one machine — plan

Status: **plan (2026-10-05). Phase 1 built 2026-10-08 (§3, "Phase 1 as built"); the live view's
second GPU (L2, `design/multi-gpu-live.md`) built 2026-10-07.** The test machine (PLUTO) now has an
RTX 3070 beside its RX 6800 XT. This document says what is worth parallelising across them, in what order, and what
has to change first. Line references are to `feat/bench-ladder` unless marked `[RR]`
(`feat/remote-rendering`, not merged).

## 1. Where the time goes, and what a second GPU can and cannot buy

A deep render has two costs, and only one of them is GPU work:

- **The reference orbit** (CPU, bignum, strictly sequential step to step). At 1e30000 it was
  220 s of a 223 s render before this week's work, 99 s after it (`db10d47`: the step's products
  on several cores). A second GPU buys nothing here.
- **Iterating the pixels** (GPU). Dominant for big exports, high iteration caps and interior-heavy
  views: the period-951,094 ladder scene spends 17 s of 19 s in the GPU passes, a 4K export of a
  dense field is almost all GPU, and a tour is that times every frame.

So the second GPU pays for **exports and tours**, not for deep live navigation, where the
bottleneck is the orbit. That agrees with the 2026-07-01 assessment in `TODO.md:9197-9201` and
the deferred item at `TODO.md:9413-9423` ("Multi-GPU — offline/export only").

**Revised 2026-10-07:** the live half of that conclusion no longer holds. A moving deep frame
measured ~100% GPU (2026-09-17), and the reference has since become cheap. The live view now has
its own design: `design/multi-gpu-live.md`.

## 2. Facts that shape the design

1. **One device per process.** eframe creates the only Instance, Adapter, Device and Queue
   (`main.rs:305-411`). `--render` goes through the same window and device (`cli.rs:1597-1617`).
   The live view paints through an `egui_wgpu` callback on that one device
   (`fractadyne-gpu/src/lib.rs:3435-3465`). wgpu has no cross-device resource sharing.
2. **The export path is device-agnostic already.** `render_export` builds its own shader,
   pipelines and buffers from whatever `&Device` it is given (`fractadyne-gpu/src/export.rs:943-1025`).
   It splits the frame into tiles and reads each back to a CPU buffer
   (`:907-941`, `:1295-1375`). A tile is therefore a natural unit to hand to another device.
3. **Process-wide state that assumes one GPU:**
   - `ORBIT_LEN_CAP`: first value wins, from this device's storage-binding limit (`render.rs:469-488`).
   - The dispatch-ceiling calibration, keyed by adapter name (`calibration.rs:70,171-185`).
   - `CUSTOM_FACTOR` (`calibration.rs:155`).
   - The `TAIL_DF32` and `TILE_OCCUPANCY` capability flags (`fractadyne-gpu/src/lib.rs:89,103`).
     `TAIL_DF32` follows whether this GPU's compiler folds df32 error-free transforms, which is a
     property of the GPU, not of the process.
   - A timing `thread_local` (`timing.rs:34`).
   - The app's `gpu_name` and `max_texture_dim` (`main.rs:6074-6082`).
4. **Tours already run as child processes**, partly so each has its own device
   (`main.rs:4825-4829`).
5. **The render farm `[RR]` already does multi-GPU, between processes.**
   - `--adapter N` pins a process to one adapter, and `--list-adapters` lists them
     (`gpu_choice.rs`).
   - `--adapters all|N,M` runs one farm session per GPU, each spawning `--render-tour … --adapter N`
     children (`farm/client.rs:288-319,662-687`).
   - A **GPU class** is adapter + driver + probe (`farm/controller.rs:335-338`).
   - The scheduler keeps a held shot on one class (`fractadyne-farm/src/sched.rs:675-684,754-759`).
6. **Different GPU classes render different pixels.** This is measured, not assumed
   (`design/remote-rendering.md` `[RR]` §12):
   - RADV against the RTX 3080: up to 0.5% of pixels on the gate tour, and up to 6.6% at 2.37e4000.
     The probe matched, yet the frames still differed.
   - The same RX 6800 XT under the Windows AMD driver: up to 26%.
   - The differences are deterministic per class. They are masked in motion, but show as a band in
     a held shot.
7. **Device loss ends the process.** Both detectors write a crash report and relaunch or exit
   (`main.rs:5998-6068`). Nothing survives the loss of one device.

## 3. What to build, in order

### Phase 0 — measure on PLUTO (no code; the published farm build already has the tools)

The share's `v0.3.0-beta.18` is a `feat/remote-rendering` build, so it has `--list-adapters`,
`--adapter` and `--adapters`. Through the field agent (ask before queueing):

- **Enumeration:** confirm `--list-adapters` shows both cards under Vulkan on Windows, with both
  drivers loaded together. Repeat under Linux: RADV beside the NVIDIA proprietary driver.
- **Per-GPU speed:** run the bench-ladder scenes with `--adapter` on each card alone. The RTX 3070
  needs its own dispatch-ceiling calibration; it has no entry in
  `validation/calibration/ceilings.toml`.
- **Cross-GPU differences:** render the same scenes on each card and count differing pixels, as the
  farm did for RADV. That number decides the still-export policy in Phase 2.
- **Both cards at once:** a local farm run, with a controller and one client on `--adapters all`,
  splits a tour across the two cards. `--farmtest` doesn't do this: its client D runs
  `--adapters N,N`, the same adapter twice. That gives the first real two-GPU number for tours.

### Phase 1 — tours on every local GPU (frame-level; reuses the farm)

A tour's frames are independent. Splitting **frames** across GPUs gives near-linear gains, and no
single frame mixes classes. The farm already provides:
- per-adapter sessions;
- holds kept on one class;
- shared orbits;
- crash isolation (each GPU is its own child process).

What's missing is a local, zero-configuration entry point:
- **CLI:** `--render-tour FILE --gpus all`, which starts an in-process controller on loopback, one
  session per adapter, and writes the same frame sequence a single-GPU render would.
- **GUI:** a "Use all GPUs" checkbox in tour export.

**Depends on merging `feat/remote-rendering`.**

**Expected:** about the sum of the two cards' per-frame throughput (Phase 0 measures each).

**Acceptance:**
- A two-GPU tour has the same frame count and order as a single-GPU one.
- Held frames all come from one class.
- `--farmtest`'s checks pass with `--gpus all` on PLUTO.

**Phase 1 as built (2026-10-08, `farm/local.rs`).** `--render-tour TOUR --gpus all | N,N,…` and the
Render tour window's *Use all graphics cards (N)* (shown for two or more cards, counted by a
`--list-adapters` child when the window opens).
- **Not in-process.** The render process is an orchestrator: a `--farm-render` controller on
  `127.0.0.1:0` (`--no-discovery`, `--min-clients N`, `--ui-status`) and one `--render-client
  --adapters … --one-job --ui-status`, both children through `status::Link`, so their stdin is the
  orchestrator's: however it ends (the window's Stop kills it), the controller stops and the client
  leaves. Measured: kill the orchestrator mid-render and the controller, the client and its render
  process are all gone within 12 s.
- **Single-render surface.** It resolves the tour as `--render-tour` does, keeps the
  existing-frames rule (refuses without `-y`/`--resume`; `-y` clears those frames and `DIR/farm/`,
  since the controller adopts any complete frame it finds), prints `frame K/N` lines from the
  controller's status (the window's bar reads them), writes `render-status.txt`, and runs the mp4
  step (`scripting::assemble_mp4`, now shared). Refused, not ignored: what a farm job has no
  notion of (`--segment(s)`, `--frames`, `--order`, `--dry-run`, the anchor flags) and the overrides
  a farm takes from the session instead (`--watermark`, `--show-location`, `--bla`, `--set`).
- **Its own key and client**, in `<config>/farm/local-gpus/`: the controller pins a client's name to
  its identity, so a client made afresh per run would be refused the second time (checked: a second
  `-y` run into the same folder renders). The user's `farm-key.txt` is neither read nor made.
- ⛔**No `FRACTADYNE_LOG_DIR` for the children.** Their render processes inherit it; one the client
  stopped when the job closed left its "still running" marker there, the next client reported it as
  a crash, and relayed child lines in the orchestrator's own log then failed ITS log check. The
  children log as guests, the client's renders in their job folders; child lines go to stdout only.
- **Same-class identity.** `--gpus 1,1` (two sessions on this machine's one RTX 3080) renders the
  farm test tour's 19 frames identical to a `--farm-child` render, and identical again after `-y` and
  after `--resume` with two frames deleted. Against a plain `--render-tour` the farm's frames differ
  only in the watermark (52–91 px a frame, all inside the mark: a farm draws it at a fixed scale).
- **PLUTO, 2026-10-08 (build `g08d0431`, field agent v19).** `tours/deep-minibrot-dive.toml` at
  1280×720, 6 fps, 265 frames, Windows, both cards Vulkan:

  | Render | Time | Frames/s |
  | --- | --- | --- |
  | `--gpus all` (RX 6800 XT + RTX 3070) | 2 min 27 s | 1.80 |
  | RX 6800 XT alone (`--adapter 1`) | 6 min 00 s | 0.74 |
  | RTX 3070 alone (`--adapter 2`) | 4 min 08 s | 1.07 |

  The sum of the two solo rates predicts 147 s; it took 147 s, farm start-up included (1.7× the
  faster card, 2.4× the slower). The 3070 took 176 frames (median 379 ms), the 6800 XT 89 (median
  1,453 ms). Same 265 frame names; every one of the farm's frames is identical to the solo render
  of the card that drew it outside the watermark's corner (the farm's mark is drawn at a fixed
  scale), and the two cards differ from each other on a median 29% of a frame's pixels. All three
  runs exit 0 and pass their log checks. The 3 held shots stayed on one card each.
- ⚠**Class switches.** The frames came back in 5 runs: 6800 XT 0–21, 3070 22–196, 6800 XT 197–199,
  3070 200, 6800 XT 201–264. Frame 200 is one frame of the other card between the 6800 XT's, from
  the scheduler taking over the end of a run. With classes this far apart (29% of pixels) that can
  show as a one-frame change of fine grain in motion; the farm's flicker measurement (3080 against
  6800 XT, 2% of pixels) was made on a smaller difference.
- **The switches at 30 fps (2026-10-08, user: "yes" to checking them).** The same tour at 1280×720
  and 30 fps, 1,321 frames: `--gpus all` 11 min 06 s, the RTX 3070 alone 20 min 39 s (1.86×). The
  farm made four runs (6800 XT 0–105, 3070 106–816, 6800 XT 817–1135, 3070 1136–1320): three
  switches, none a single frame. Measured against the 3070's own sequence at the same frames:
  - 106 ends the opening held shot (frames 0–105, kept on one card), where the cards differ by
    0.09/255: nothing to see.
  - 817 and 1136: the class difference is 4.5 and 5.5/255 (2.9 and 3.9 after a 2×2 average)
    against a frame-to-frame motion of 42 and 50/255 (38 and 40): about a ninth, as pixel speckle
    in the filaments, which the zoom rearranges on every frame anyway. The farm's change into the
    switch frame equals the single card's to 0.1%, and the amplified change images of the two
    sequences look the same (`local/l2-ab/switch/switch-00817.png`; clips of a second either side,
    farm and single card, beside it).
  So no shimmer to expect at these switches. The risky case would be a switch in a slow or held
  stretch, and the farm keeps holds on one card; the 6 fps run's one-frame switch came from its
  shorter tour (265 frames) ending in stolen work.
- **Not yet:** a `--farmtest` variant with `--gpus`.

### Phase 2 — one still across GPUs (tile-level, in one process)

**Phase 2a as built (2026-10-08, user: "yes" to step 3).** The export path needed far less than this
section expected: L2 had already shown `render_export` runs unchanged on a second device in the same
process, with no per-device capability state (`design/multi-gpu-live.md` §8).
- `render_export_impl` is split into `export_plan` (tile size, chunk scope, occupancy: decided once,
  from the SMALLEST limits among the devices), a per-device `TileRenderer` (pipelines, its own upload
  of the reference, uniforms, its own chunk/step pricers, which change dispatch shapes, never pixels)
  and the drivers. The single-device loop is the old one, wall-adaptive cap included: the full
  selftest is 510/510 with 31/31 goldens on the refactor.
- `render_export_multi`: a fixed grid (`multi_tile_rects`: the single device's equal grid on the
  step-bounded mode-2 path; elsewhere `tile` squares, the single loop's layout before its cap moves),
  one scoped thread per device taking the next tile from a shared counter (a handed-back tile
  first), assembly and progress in the caller's thread. A device that fails is retired and its tile
  goes back; one handed back after the others finished is rendered at the end on a device still
  standing; the render fails only with none left. Only the chunked iterate is split
  (`export_splits`): outside it a tile is one unbounded dispatch, safe only under the single loop's
  wall-adaptive size.
- CLI: `--render … --gpus all | N,N`. The window's card always renders; the extra cards are opened
  at start as `gpu_worker::open_headless` (the device code the live worker now shares), Vulkan only;
  `export_extra_cards` resolves the spec (the window's number once = the window, twice = a twin).
  The log names who rendered what. Not yet in the Export dialog: the mixed-model default is open.
- **Checked on this machine (RTX 3080 + a twin device):** 9.3e78× at 1280×720 ss 2 and 3840×2160 ss 2
  (12 tiles), and 1e12× df32 at 1920×1080 ss 2 (93 tiles): 0 differing pixels against the card alone,
  event counters equal; with the twin failing after 2 tiles (`FRACTADYNE_SPLIT_LOSE_AFTER=1:2`) the
  window took its tile and the image was identical. Selftest `worker` group +2: the split, and the
  split whose twin fails, both bit-identical with both devices rendering — proven red by corrupting
  one channel of the twin's tiles (both FAIL, "1 of 92160 channels differ").
- **Next:** PLUTO, a still on the RX 6800 XT, the RTX 3070 and both (field agent v21 delivers a
  `--render`'s image): the speed, and whether a seam between the two models' tiles shows.

Big stills (8K and up, high caps, interior-heavy minibrots) are where a single frame waits on the
GPU. The work splits like this:
- **Share the reference.** The parent builds the reference orbit, series skip and BLA tree once on
  the CPU. Each extra device gets its own upload; the orbit and BLA together are up to 1 GiB.
- **One headless device per extra adapter,** each with its own export pipelines (`render_export`
  already takes a device).
- **A shared tile queue.** Each device pulls the next tile when it finishes one, so a fast and a
  slow card balance themselves, and interior-heavy tiles don't stall the split.
- **CPU assembly,** exactly as now (`export.rs:1365-1375`).

**Must change first:** every per-process GPU value in §2.3 becomes per-device state. That covers
calibration, the capability flags, the timing state and the texture and binding limits. The orbit
cap becomes the **minimum** over participating devices, as the farm already does.

**Device loss:** a secondary device's loss must re-queue its in-flight tiles to the others and
drop that device, not exit the process. Only the window's own device keeps today's handling.

**Mixed-class policy.** The first Phase 0 result decides this:
- **Same class** (e.g. two identical cards): split freely. Acceptance is byte-identical to a
  single-GPU export.
- **Different classes**: a tile from one card beside a tile from the other can show a seam in
  boundary-dense regions. Options:
  - **(a) Default:** split only across same-class devices, and say which GPUs were used.
  - **(b) Split by sample, not by area,** for supersampled exports. Each GPU renders the whole frame
    at a subset of the ss×ss sample offsets, and the average hides class differences inside each
    pixel. This needs the export's fixed sample grid to become per-sample passes.
  - **(c) An explicit "fastest, mixed GPUs" option,** with a warning.

  I'd ship (a) first, and add (b) if PLUTO's numbers show the 3070 and 6800 XT together are worth it.

**Why not child processes here (as tours use):** each child would build or load the reference
itself. At depth that is the expensive part, so it would cost CPU (and a minute or more at 1e30000)
for every GPU. Periodic references are also not written to the disk cache (`9fdf3da`). In-process
devices share the one reference for free.

### Phase 3 — the live view

**Superseded by `design/multi-gpu-live.md` (2026-10-07).** That design keeps both uses below:
supersampling on settle (its L2) and refreshes during motion at predicted views (its L3). It
adopts only whole products through paths the live view already has. The original note follows.

The window can only paint from its own device, and moving pixels between devices goes through
CPU readback. At 1080p RGBA32F that's about 33 MB per pass. Two candidate uses of the second GPU
that tolerate that latency:
- **Progressive supersampling:** the second GPU renders accumulation passes, which are uploaded
  to the presenting device.
- **Dive prefetch:** the second GPU renders the next zoom level for XaoS-style reuse.

Both only pay where the live view is GPU-bound, which at depth it usually isn't (§1). Decide after
Phase 2 has a per-device renderer to build on.

## 4. Risks and unknowns

- **Two vendors' Vulkan drivers in one process** (Phase 2). The RX 6800 XT already has an open
  device-loss issue (#1) and readings that lag one dispatch (see the RX 6800 XT notes). If mixing
  drivers in one process proves fragile on PLUTO, Phase 2 falls back to one worker process per
  device. The parent would ship the reference to them in a file (the orbit blob format), trading
  some CPU for isolation.
- **Calibration per device.** The live-path ceiling is per adapter. The export pricing starts from
  constants on every render (`fractadyne-gpu/src/export.rs:486-568`), so it adapts per device
  without new data.
- **Memory.** Each device holds its own orbit+BLA copy and tile textures. The 1 GiB binding cap
  applies per device.
- **Power and thermals** on PLUTO with both cards loaded: watch for clock drops in Phase 0's
  `--adapters all` run.

## 5. Decisions needed

1. **Mixed-class still policy:** (a) same-class only by default, (b) the sample split, or (c) a
   mixed option. My recommendation is (a), then (b) once measured.
2. **Merging `feat/remote-rendering`:** Phase 1 needs it. Should Phase 1 go on that branch, or after
   it merges?
3. **PLUTO OS for the first runs:** Windows (the field agent, both drivers) or Linux (the farm
   watcher, RADV plus NVIDIA's driver)? I'd start on Windows.
