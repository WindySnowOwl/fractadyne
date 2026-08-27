# Changelog

All notable changes to Fractadyne. Versioning is `MAJOR.MINOR.PATCH` (Cargo) plus an
auto-incrementing **build** number (bumped by `build.rs` on every recompile) shown as
`v0.2.0 (build N)` in the title bar, Help menu, and exported image metadata.

The project enters tracked versioning at **0.1.0**; entries below summarize the state
at that point and changes after it. From **0.1.1** on, the patch version is bumped for each
new functional enhancement; a **minor** bump (e.g. 0.2.0) marks a milestone that rolls up a
run of patch releases. The `0.2.0` entry summarizes **0.1.29 – 0.1.68** by theme — per-version
detail is in the git history.

## 0.2.40-beta (in progress)

The post-0.2.36 series (**0.2.37 – 0.2.40-beta.53**, published as `v0.2.40-beta.N`
pre-releases on the Beta update track). Grouped by theme, newest first; per-version detail is
in the git history.

- **The faster arithmetic is now a download you can actually get** (beta.157) - the optional
  accelerated build described below is packaged as its own zip, and Help now has a "Faster deep
  zoom" entry that explains it and links to the version matching the one you are running. Extract
  it, run it, and your settings, saved session and locations carry over untouched - they live in
  your user profile rather than beside the program, so you can move between the two builds freely.
  If you are already running the accelerated one, the same dialog tells you so. The images the two
  produce are identical; only the speed differs.

- **An optional second arbitrary-precision engine, off by default** (beta.153) — deep zoom is
  carried by arbitrary-precision arithmetic on the CPU, and on the deepest views that work takes
  longer than everything the graphics card does for the same frame. Fractadyne can now be built
  against MPFR (the GNU multiple-precision float library) instead of its usual pure-Rust one, which
  on this machine computes the same orbits roughly 2.5x to 7x faster, depending on depth. It is a
  build-time option that is **off unless asked for**, for two reasons that are not going away: the
  library does not build with the Microsoft compiler Fractadyne normally uses on Windows, and it
  carries a copyleft licence that would place conditions on a released binary which Fractadyne's own
  licence does not. Nothing about a normal build changes. Where it is enabled it is used by default
  — a build option that then needs a second flag to do anything is a build option that does nothing
  — and `--bignum` overrides that either way; asking for an engine that is not in the build is a
  clear error rather than a silent fall back to the other. The two engines produce **byte-identical**
  results, verified across every fractal formula at six precisions and again at the very large
  arithmetic widths extreme zoom uses, so the existing reference images check both.

- **Crash reports, the self-test and bug reports now name the arbitrary-precision backend that
  produced them** (beta.152) — deep zoom is carried by arbitrary-precision arithmetic, and every
  golden image, corpus render and blessed benchmark is the output of one particular arithmetic
  library. Until now nothing recorded which one, which was harmless while there was only ever a
  single choice and becomes a real problem the moment there is more than one: a result you cannot
  attribute is a result you cannot compare. Reports now carry it, the self-test asserts exactly one
  backend produced the run, and the startup log records which backends the build contains. The
  value is taken from the arithmetic that actually ran rather than from a setting, so it cannot
  claim a backend the program did not use. There is also a new `--bench-bignum` for measuring what
  that arithmetic costs at each precision; it needs no GPU, and it marks any measurement whose
  test orbit escaped as invalid instead of reporting the meaninglessly fast number that produces.

- **The smallest piece of work the renderer will issue now adapts to how slow the region is**
  (beta.142) — the renderer never split work below a fixed size of 256 steps, on the reasoning that
  256 steps is quick no matter what. Measurements from a recorded device loss show that reasoning
  was about five times optimistic: in that region 256 steps would have taken roughly a third of a
  second, and a region three times harsher would make even the smallest allowed piece dangerous on
  its own. The minimum is now the smaller of 256 and however many steps actually fit the time
  budget at the worst speed measured, so it only ever shrinks, only in regions whose own
  measurements call for it, and never slows an ordinary render.

- **Auto-normalized colour no longer buries the picture at shallow zoom with a big iteration
  budget** (beta.150) — "Normalize deep colors" spreads the palette across the range of escape
  values in view. That works when those values sit in a narrow high band, which is what deep views
  give you. At a shallow view with a large iteration budget the range is lopsided instead: nearly
  everything escapes within a few dozen steps while a thin fringe along the boundary runs to tens
  of thousands. Spreading the palette evenly across that put the whole visible exterior into a
  fraction of a percent of the colours — a near-black field with only the filaments lit. A lopsided
  range now uses the logarithmic mapping (the same one the "Log color scale" checkbox selects)
  automatically, so the exterior gets its colours back. Deep views are unaffected.

- **A deep view no longer restarts itself forever because the status bar changed width**
  (beta.149) — the iteration count and the zoom shown along the bottom were drawn at whatever width
  their digits happened to need. On a window where the bar only just fits, one extra digit wrapped
  it onto a second line, which made the picture area shorter, which the renderer treated as though
  you had resized the window — so it threw away the work in progress and started again, which
  changed the numbers, which changed the width. A deep view could sit black indefinitely, never
  finishing. Both fields now reserve a fixed amount of room, so the bar's height stops depending on
  what the numbers happen to say. It is the same failure the status-bar labels were fixed for
  earlier; these two numeric fields had been left out.

- **A deep view no longer turns black when the iteration count drops** (beta.147) — at extreme
  zoom the renderer skips ahead past the first few hundred thousand iterations using a shortcut
  computed from the reference orbit. If something then lowered the iteration budget below where
  that shortcut lands — toggling "Auto-scale iterations with zoom" is the easy way to do it — every
  pixel started out already past its own limit, counted as inside the set, and the frame rendered
  solid black. The shortcut is now declined whenever it would land at or past the budget, and the
  view simply computes the ordinary way instead: correct picture, slightly slower.

- **3D relief lighting no longer speckles deep views black** (beta.145) — with relief lighting on,
  deep zooms came out stippled with near-black pixels. The shading was computed from the surface
  slope at each pixel on its own, and at that depth the slope varies faster than the pixel grid, so
  neighbouring pixels disagreed about which way the surface faced and the light flickered between
  lit and shadowed. Extra anti-aliasing only helped in proportion to the number of samples, because
  each sample was still asked to decide on its own. The shading now reads the slope over a small
  neighbourhood and, where that neighbourhood cannot agree on a direction, fades the lighting out
  instead of picking one at random. Deep views keep their relief without the speckle, ordinary
  views look as they did, and there is no measurable cost.

- **"Normalize deep colors" now actually engages where the picture needs it** (beta.144) — deep
  views could come out as coloured static: a correct image, mapped through a palette that repeated
  many times between one pixel and the next. The setting meant to prevent exactly that decided
  whether to act by looking at the total spread of escape values across the frame, which turns out
  not to answer the question — a shallow view can span a hundred palette repeats and look perfect,
  because neighbouring pixels barely differ. What matters is how far the palette moves from one
  pixel to its neighbour. The renderer now measures that directly and normalizes when the palette
  would advance more than half a cycle per pixel, which is the point past which the banding cannot
  be seen as banding at all. Deep dense fields resolve; ordinary views keep their classic colouring.

- **A stalling render now reacts without waiting to be asked** (beta.141) — the renderer judged how
  expensive a piece of work had been only once that work reported back, and it treated a frame
  arriving promptly as the signal that the GPU had caught up. On a machine sliding toward a driver
  reset, frames stop arriving promptly — so the one measurement that would have told it to back off
  was the one it could never collect, and its fallback took roughly twenty-four seconds. It now
  treats elapsed time alone as sufficient evidence: if a piece of work has already been running long
  enough to be dangerous, that is known without waiting for it to finish, and the size limit for
  subsequent work is reduced immediately. Rendered images are unaffected.

- **The renderer's cost memory no longer gets blurrier the deeper you go** (beta.140) — to keep
  each piece of GPU work short, the renderer remembers how expensive each stretch of a computation
  has been and sizes the next piece from that. It divided the whole computation into sixteen equal
  stretches — so the deeper the zoom, the longer each stretch, and the vaguer the memory. At the
  depth of a device loss recorded in August, each stretch covered about fourteen thousand steps,
  which meant a piece of work could be sized from measurements taken in a completely different and
  much cheaper part of the computation. The stretches are now fixed, doubling in size as they go,
  so the memory stays sharp exactly where the expensive surprises happen. Rendered images are
  unaffected.

- **After a near-miss, the renderer now forgets what the region "afforded"** (beta.139) — when a
  frame comes within about 2x of losing the graphics device, the app cuts its work budget. But the
  size of each piece of work is also limited by a separate allowance the renderer earns from how
  cheap that part of the image has been so far, and the emergency cut could not lower that
  allowance. Worse, the routine that normally lowers it only runs on work that finished promptly —
  and a machine about to lose its device is precisely one where nothing is finishing promptly. So
  the allowance could survive the emergency that should have removed it. It is now discarded
  outright on a near-miss, and re-earned from scratch. The rendered image is unaffected: the
  allowance controls only where a computation pauses and resumes, never what it computes.

- **Crash reports from a near-miss now record the render size** (beta.138) — when a frame comes
  within about 2x of losing the graphics device, the app always writes a line about it, so that a
  report from the field carries the evidence even if nobody had diagnostics turned on. That line
  did not include the resolution being rendered, which turned out to matter: a recent fix showed
  that certain render sizes cost roughly 20x more than their neighbours, and the existing field
  report could not be checked against it. It can be now.

- **Deep renders were up to 20x slower at half of all window sizes, for no visible reason**
  (beta.137) — at depth the renderer splits a long computation into bounded pieces so the GPU stays
  responsive, and it keeps the work-in-progress in off-screen scratch buffers between pieces. When
  those buffers happened to have an odd number of rows, the graphics driver fell off a fast path
  and every piece paid a large fixed penalty, whatever it was actually computing. The same view one
  pixel taller took 4.6 seconds instead of 0.2. It was invisible because nothing about the picture
  changes: the app picks its resolution to hit a frame-time budget, so the penalty switched on and
  off as you zoomed, looking like the fractal had simply got harder. The scratch buffers are now
  allocated with an even number of rows. The rendered image is unchanged — byte for byte, checked
  against the reference corpus — and the penalty is gone.

- **An exported image no longer depends on how the render was scheduled** (beta.136) — since
  beta.122 an offline render splits each expensive tile along the iteration axis, so that no single
  piece of GPU work runs long enough for the driver to kill it. Whether to do that was decided tile
  by tile, from measured timings. That turned out to matter to the picture: the split and unsplit
  paths are separate programs on the graphics card, this hardware's driver optimizes the two
  slightly differently, and a frame could therefore run some of its tiles through one and the rest
  through the other. The result was an image that depended on the tile budget, on how fast the
  machine happened to be, and on where the expensive part of the view sat — the same view could
  come out a few dozen pixels different for no reason the picture itself could explain. Three views
  in the comparison corpus had been drifting from their references this way since beta.122.

  The choice is now made once, before the first tile, from the requested render alone, so a frame
  is always drawn by a single program. The same view rendered at half, standard and double the tile
  budget is now byte-for-byte identical, where those three settings used to disagree. There is no
  measured speed cost: a tile that does not need splitting still runs as the one piece of work it
  always did. The three affected reference images have been regenerated against the new,
  schedule-independent output, and a self-test now fails if any future render mixes the two paths
  again.

- **A mistyped option is now an error instead of a different picture** (beta.135) — fixing `--zoom`
  (below) turned out to be one instance of a pattern repeated across the command line: an option
  whose value could not be read was quietly replaced by a default, and the program then did the
  full expensive job and reported success. The sharpest case was `--center`, where a single stray
  character in a pasted coordinate — or simply forgetting the second number — threw the whole
  location away and rendered the default view at the depth you asked for, which at deep zoom is a
  plausible-looking solid frame. `--size`, `--iter`, `--ss`, `--zoom-log2`, `--fractal`,
  `--method`, `--trap`, `--palette` and the tour renderer's `--fps`, `--height` and sharding
  options behaved the same way, as did the coordinates in the headless comparison and profiling
  modes — including the cross-renderer check, where a bad value produced a wrong VERDICT about
  whether two renderers agree rather than a wrong picture. All of them now say what they could
  not read and stop. Leaving an option out still means what it always did: use the default.

- **The benchmark, re-measured fairly — and it changes the answer** (beta.135) — with both
  renderers finally sampling each pixel once, Fraktaler-3 is FASTER than Fractadyne on the two
  heaviest scenes: the 1e148 view by 1.18x and the extreme 1e1105 view by 2.20x. Fractadyne still
  leads the other eight, but by 1.12x to 2.29x rather than the 1.9x to 6.8x the handicapped run
  suggested. The earlier summary of "Fractadyne owns deep and extreme" was an artefact of the
  sampling defect and should not be repeated.

  Where the architecture does show a large advantage is a zoom SEQUENCE, which is what it was
  built for: rendering forty 4K frames of a dive as one continuous descent takes 0.24 s a frame,
  against 2.55 s a frame to render the same forty one at a time — and against 4.11 s a frame for
  Fraktaler-3, whose batch command has no sequence mode. So: comparable on a single frame,
  behind at the deepest, and roughly seventeen times ahead across a dive.

- **Benchmark kit: Fraktaler-3 had been given four times the sampling work** (beta.135) — the kit
  compares renderers on scene files borrowed from the correctness corpus, and those files carry
  the corpus's own antialiasing setting: four samples per pixel for Fraktaler-3, to pair with the
  matching setting on Fractadyne's side. The benchmark renders one sample per pixel, and said so
  in its README, but only rewrote the image SIZE when it copied the scene — so every published
  number had one renderer sampling four times as hard as the other. It now restates the sample
  count too, everywhere a scene file is written. **Previously published comparisons should be
  treated as void until re-measured.** Re-measured at two reps on 2026-08-23: the corrected
  picture holds. This is the second defect of exactly this shape (the first
  was resolution), and the rule it teaches is that a correctness fixture is not a benchmark input.

- **A zoom sequence is now part of the benchmark** (beta.135) — every other scene in the kit is a
  single frame, which cannot see the thing that matters most for zoom video: diving toward a fixed
  point lets a renderer reuse its expensive setup across many frames instead of rebuilding it for
  each one. The new lane renders a ladder where every frame is the same picture at a different
  scale, so any cost ramp is the renderer and not the scenery, and reports how much each app
  saved measured against itself. On this machine Fractadyne renders that 4K ladder 8.2x cheaper
  per frame in sequence than one frame at a time. Fraktaler-3's batch command renders one image
  per invocation, so its figure is 1.0 by construction — a property of its command line, not of
  its engine, and the report says so.

- **Fixed: `--zoom` could silently render the wrong place** (beta.135) — a magnification the
  option could not read was quietly treated as 1x, so the command rendered the whole Mandelbrot
  set and reported success. Perfectly ordinary values triggered it: `1.0e23.9`, the form a zoom
  ladder naturally produces, is not something a plain number parser accepts, and anything past
  `1e308` became infinity. Both now go through the same parser the app's own go-to box uses, and
  a value that cannot be read is a clean error instead of a picture of somewhere else.

- **A crash report now says whether the frame that died was doing any work at all** (beta.135) —
  when the graphics device is lost, the report records a "manifest": exactly what the frame was
  asked to draw. Two numbers were missing from it, and both can change what a frame costs by
  thousands of times. The first is the series-approximation skip, which lets the renderer start
  a pixel thousands of iterations in; a slice of work that ends below the skip finishes instantly,
  so a run of such slices looks fast and earns the renderer permission to ask for far more next
  time. The second says whether the acceleration table was in use. Both are now stamped on the
  crash manifest and on the warning line the log always prints when a frame comes close to the
  driver timeout, so a report sent in from the field carries them without anyone having had to
  turn diagnostics on beforehand. The live manifest is also traceable per frame
  (`FRACTADYNE_TRACE=req`) for watching a zoom as it happens. Rendering is untouched.

- **Tour renders now actually use the reference they prepared ahead of time** (beta.132) — the
  companion to beta.130. While rendering one frame the tour prepares the next frame's expensive
  setup on a background thread, but it prepared it using the CURRENT frame's iteration count, and
  a tour changes that count on nearly every frame of a dive — so the prepared copy no longer matched
  the frame it was for and got rebuilt from scratch anyway. Measured on a 45-frame dive: 30
  references prepared and 25 still rebuilt on the drawing thread; now **all 30 are used and none
  are rebuilt**. Frames are byte-for-byte identical.

- **The finish tone can be turned off** (beta.131) — `--no-sound` silences the tone that marks a
  finished render (`--sound` turns it back on). For batch work there is also a `FRACTADYNE_NO_SOUND`
  environment variable, which is the one you want when running the test suites: they launch the app
  as separate processes per case, so a flag on the command you typed never reaches them, while the
  environment is inherited. A twenty-location validation run is silent with it set.

- **Fixed: the renderer's reference-reuse check could never say yes** (beta.130) — several parts of
  the program prepare an expensive piece of setup (a "reference orbit") ahead of time on a
  background thread so the next frame does not have to wait for it. The test that decided whether
  that prepared work was still valid compared two numbers that are never equal by construction, so
  the answer was always no: the prepared copy was thrown away and rebuilt from scratch, on the
  thread that draws the window. Nothing failed and nothing warned — the work was simply done twice.
  The check now compares what was actually asked for. Renders are unchanged (a reused reference is
  the same one the rebuild would have produced), and a test now pins the check so it cannot quietly
  become impossible again.

- **Auto-zoom dives deeper in the same time** (beta.129) — to decide where to zoom next, the
  autopilot renders a tiny 56x56 preview of the current view. It was computing a brand-new
  full-precision reference orbit for that preview — the most expensive thing the program does —
  on the UI thread, several times a second, and then throwing it away. It now borrows the
  reference the view on screen is already using, which is both free and a better answer, since the
  preview is meant to reflect what you are looking at. Measured over a 240-second dive: **553
  reference builds down to 0**, and the dive reaches **1e47 instead of 1e38** in the same time.
  The graphics card had been sitting idle waiting for those builds; it is now doing the work.

- **Glitch correction is no longer the slowest part of an offline render** (beta.128) — a
  correction pass exists to repair a few hundred wrong pixels, but it used to ask the GPU for the
  whole frame to do it. It now asks for exactly the pixels it is repairing: their coordinates go
  into a list, and the renderer draws a tiny image — a few dozen pixels on a side — in which each
  texel stands for one entry in that list. On the 1.3e6× benchmark scene correction drops from
  **9.0 s to 0.4 s** and the whole render from 11.9 s to 2.4 s; the 6.6e43× scene goes from 15.2 s
  to 2.5 s. The repaired image is **byte-for-byte the one it produced before** — same references,
  same corrected pixels — because both paths run the identical arithmetic on the identical
  coordinate. Frames whose iteration count is high enough that a single pass has to be split into
  bounded pieces (above 400,000) keep the previous method, which can split them.

- **Glitch correction only re-renders where it needs to** (beta.127) — each extra reference pass
  used to re-compute the entire frame in order to repair a handful of pixels; it now skips the
  areas with nothing left to fix. Where the correction was running out of passes, that makes the
  render faster (the 1.3e6× benchmark scene, whose correction was 10.2 s of an 11.1 s render,
  drops to 9.0 s with identical output); where it was running out of time, the saved effort goes
  into more correction instead — the 4.6e1105× scene now fits 57 reference passes into the same
  104 s where 37 fit before, leaving 91 unresolved pixels instead of 122.

- **Glitch-corrected exports are faster again** (beta.126) — the beta.124 chunking set up its
  machinery on every correction pass, even for frames whose whole iteration count fits a single
  bounded dispatch and so are never actually split. On a 60,000-iteration frame that was pure
  overhead: the 1e12× benchmark scene went from 14.3 s to 21.7 s. The setup is now built only
  once a render genuinely needs to split a tile, and an unsplit tile still reports its cost, so
  a frame that turns out slower than expected starts splitting from the next tile on. Same
  scene now renders in 12.6 s — faster than before chunking existed.

- **Fixed: views deeper than 1e308× rendered a BLANK image** (beta.125) — a regression that had
  been live since 2026-08-17. A guard added for corrupted sessions (where a NaN zoom used to
  select the most expensive arithmetic) rejected every *non-finite* magnification, not just
  NaN — but the magnification figure is a plain 64-bit float, so it saturates to infinity past
  about 1e308×. Every genuinely extreme view therefore looked like garbage input to the mode
  selector and was quietly demoted to the shallow, non-perturbation renderer, which at that
  depth produces an empty frame. Views past 1e308× now correctly use the extended-range
  arithmetic they were always meant to. If you visited a very deep location since mid-August
  and found it blank, this was why.

- **Fixed: glitch-corrected exports could still lose the device on a single tile** (beta.124) —
  the last piece of the export watchdog family. The beta.122 fix bounded every dispatch in the
  plain export path, but the multi-reference glitch corrector's own passes were out of scope
  (its shaders had no way to say "this pixel is glitched" mid-progression), so its base pass —
  which runs with acceleration disabled, over exactly the deep-interior pixels that cost the
  most — still issued one unbounded dispatch per tile. Its 120-second budget only checks
  *between* tiles, so a single tile could overrun the OS watchdog inside it. Glitch detection
  now works across chunk boundaries: a glitched pixel settles into a fourth state that carries
  through the rest of the progression and resolves to the same marker as before. Output is
  bit-identical to the old single-dispatch path (new self-test gate), correction still resolves
  every glitch, and the chunking costs 0.5% overhead.

- **The finish tone is now synthesized band-limited, through a PC-speaker model** (beta.123) —
  the tune is unchanged (FRACTINT's `buzzer0`: 1047/1109/1175 Hz, 100 ms each), but it no longer
  goes through the `kernel32 Beep` shim's naive square. The square is generated by a phase
  accumulator with polyBLEP band-limited edge corrections (sub-sample timing, O(1) per sample,
  aliasing around −40 dB), one continuous phase across all three notes so the joins are
  click-free, then high-passed at 450 Hz to account for the frequency response of the little
  PC-speaker cone the original played through. Millisecond edge fades declick the start/stop,
  the peak is normalized so nothing can clip, and playback is an in-memory WAV via `PlaySound`.
  The synthesis chain is unit-tested (pitch, DC rejection, peak, container).

- **Fixed: a deep high-iteration export could lose the GPU device hours in** (beta.122) — the
  crash behind the 5K/2xSS/4,000,000-iteration report: near a minibrot interior a tile is
  latency-bound (its cost is the deepest pixel's iteration chain, no matter how small the
  tile), so the wall-priced tile cap — which can only shrink AREA — could not keep a single
  dispatch under the OS watchdog, and at 2.6 h in one tile crossed it. Export tiles now split
  along the ITERATION axis instead: each tile runs as a sequence of resumable, wall-priced
  iteration windows (the same chunk shaders the live view uses, bit-identical output — gated
  by a new selftest), every window is bounded by the worst serial cost observed so far, and
  the exact crash workload now completes with its longest single dispatch under 300 ms. As a
  bonus, dwell-heavy regions render ~4x faster (the cap no longer shrinks tiles in a regime
  where that only multiplied the work), interrupted exports cancel promptly, and the per-export
  perf line now records `max_dispatch=` for field diagnosis. Scope: smooth/distance/relief/glow
  coloring on the holomorphic formulas; aux colorings and the glitch-corrected path keep the
  previous tiling for now.

- **Fixed: an export no longer dies if its output folder disappears mid-render** (beta.122) —
  a tour rendering to a network share or USB drive that dropped out lost the whole render at
  the final write. File writes now retry with escalating backoff while the destination is
  unreachable, and give up with a clear error only after the retry budget is exhausted.

- **The finish sound is now FRACTINT's actual completion tune** (beta.121) — sourced from the
  DOS original's `general.asm` rather than guessed: three rising 100 ms notes at 1047, 1109 and
  1175 Hz (C6, C#6, D6), played through Windows' PC-speaker shim exactly as `buzzer0` encoded
  them. The beta.120 system chime is replaced; the same "Sound when a render finishes" checkbox
  controls it.

- **New: a sound plays when a render finishes** (beta.120) — the FRACTINT tradition, by request.
  Fires when a GUI export, a tour render, or a command-line `--render` completes (success or
  failure — either way the wait is over), via the system notification sound so it respects your
  Windows sound scheme. Off-switchable next to the other render settings ("Sound when a render
  finishes").

- **Fixed: the reported VRAM figure could belong to a GPU no longer in the machine** (beta.120).
  Windows keeps a registry entry for every display adapter ever installed, and the probe took
  the largest figure among the first few entries — on a bench that has seen several cards, that
  could be a stale one (the RX 6800 XT field report said 8192 MB for a 16 GB card). The probe
  now scans the full class list and prefers the entry matching the active adapter's name,
  falling back to the widest scan only when no name is available.

- **Fixed: the beta.117 motion backpressure could be re-armed by stale measurements, letting
  oversized work stack anyway** (beta.119). Found by the fourth validation crash on the same
  hardware: the backlog of unmeasured full-size dispatches was cleared whenever any measurement
  returned — but a measurement only vouches for work submitted before it, and the measurements
  arriving during a zoom-home's budget climb belong to the small frames from seconds earlier.
  Each stale arrival re-credited the gate to admit more full-budget dispatches, three of which
  queued back-to-back at the ceiling right before the loss. Retirement is now driven by the
  GPU's own completion signal (one ordered callback per counted dispatch), so measurements play
  no part in admission control at all: work is admitted when prior work is confirmed done, and
  measurements only price it. The crash manifest also now prices a piece-by-piece frame by its
  actual piece, not the whole frame's nominal cost, which had made the killer frame read as 3x
  over a budget it was honoring.

- **New: `--log-dir DIR` (and `FRACTADYNE_LOG_DIR`) redirect the log file and crash reports**
  (beta.118) — for example onto a network share while validating on another machine. Only the
  logs move; the session and settings stay in the config directory (unlike
  `FRACTADYNE_CONFIG_DIR`, which relocates everything and so also resets the session). The flag
  wins over the variable; an unwritable directory falls back to the default location loudly, and
  a missing value is a fatal error rather than a silent default.

- **Fixed: a fast zoom-out from depth could stack oversized GPU work faster than measurements
  could brake it** (beta.117). The companion to the beta.116 fix, from the same validation
  campaign's second crash: with the bookkeeping honest, the cost controller is still fed nominal
  step counts, and what a nominal step costs for real changes several-fold across a zoom home.
  Measurements arrive a few frames after the work they describe, so the sweep queued five
  full-budget dispatches before the first "this regime is slower" reading could land — and by
  then each queued dispatch ran over a second. Motion frames now stop submitting full-size work
  while two of them are still unmeasured, dropping to the small opening-guess size until a
  measurement returns; the queue stays shallow, the retreat gets to act, and no single dispatch
  approaches the driver's watchdog. Settled views are untouched (their refinement already
  self-serializes), and the new `MOTION_UNPRICED_MAX` is `--set`-overridable for field bisection.

- **Fixed: the frame-cost budget could inflate on misrecorded step counts, oversizing dispatches
  on the way home from a deep dive** (beta.116). Found in an RX 6800 XT validation run: a
  refinement pass's bounded cost was overwritten with the full frame's nominal count whenever a
  reference install re-keyed the same frame, so ~210 ms passes were priced as if the GPU ran
  ~875 billion steps per second. The budget — already correctly converged — walked to its ceiling
  in three seconds on those fantasy rates, and the shallow side of the zoom home then dispatched
  real 1–2 s frames until the device was lost. A frame under refinement now always keeps its
  bounded pairing (the misrecording is gone), and the crash report's manifest now names the
  autopilot's small target probe honestly instead of inheriting full-panel export dimensions —
  the misdirection that pointed this triage at the export path first.

- **Fixed: correct colors could flash and then flatten on deep views** (beta.115). The remaining
  half of the beta.112 normalization fix. The palette window was fed correctly the moment a
  refinement completed — the brief flash of proper coloring — but each measurement arrives a few
  frames after the work it describes, so the final piece's late reading landed after completion
  and was mistaken for a whole-frame range, dragging the window onto a sliver and flattening the
  picture. Measurements are now classified by whether the view is under piece-by-piece
  refinement, not by refinement progress: while it is, they only ever widen the pending window,
  which is applied exactly once, complete, at the moment of completion — verified live at the
  reporting view, late reading and all.

- **The performance panel now watches the danger zones for you** (beta.114). New rows for
  process memory (current and peak — deep reference builds have quietly peaked over 2 GB) and an
  estimated GPU-resident figure assembled from the allocations the app makes. And the statistics
  with documented danger bands now carry annunciators: measured frame cost turns amber past the
  controller's target and red inside the band where cards have been lost; the budget shows amber
  while it is still calibrating; the timing row turns red if cost measurement falls back to wall
  clock (the blind state behind several crashes); and the reference-orbit length warns as it
  approaches this GPU's ceiling — the practical depth wall — before you hit it. Every threshold
  is read live from the same constants the engine itself uses.

- **Deep refinements no longer show a black screen with no cue** (beta.113). Two halves,
  both from one field report. The busy spinner had stopped appearing during long refinements:
  its idle-gap detector was tuned for rapid repaints, and the new piece-by-piece refinement
  paces frames a few hundred milliseconds apart — every frame read as a gap, so the spinner
  never armed. And the refinement itself drew progressively, which at a deep view means
  nothing at all is drawable until the work crosses the view's minimum escape count — a solid
  interior-color screen that is indistinguishable from a hang. Deep refinements now keep the
  previous image on screen (tracking position correctly) and reveal the finished frame whole,
  the same behavior "prefer detail" always had; a brand-new view with nothing to hold shows
  the honest progressive fill with the (now working) spinner and the "refining N%" readout.

- **Fixed: deep views near minibrots sometimes rendered flat instead of detailed** (beta.112).
  With "Normalize deep colors" on, the palette window is measured from what escapes on each piece
  of GPU work — and since deep refinement became piece-by-piece, the window was following the
  LAST few pieces of a refinement rather than the whole picture: when those pieces contained
  escapes, the window collapsed onto a sliver of the range and the entire exterior mapped to one
  color (the reported "sometimes flat, sometimes noise" — which of the two you got depended on
  timing). The window is now accumulated across the whole refinement and applied once, complete —
  the same behavior deep views had before the piece-splitting existed.

- **Exports now pace themselves by measured cost, and bookmark thumbnails are instant**
  (beta.111). Completes the beta.110 fix pair: every export (and the glitch-correction pass)
  now watches how long each piece of work actually took and halves the next piece when one runs
  hot — so a 4K export of a worst-case deep view stays watchdog-safe end to end instead of
  trusting estimates. And adding a bookmark no longer renders anything at all: the thumbnail is
  snapped from what is already on screen (exactly what you bookmarked — palette, zoom state and
  all) and saved in milliseconds, where it previously rebuilt a full-precision reference and
  re-rendered on the spot.

- **Fixed: adding a bookmark at a deep view could reset the graphics card** (beta.110). The
  bookmark thumbnail renders through the export path, and two inherited settings made a 160-pixel
  preview into heavyweight work: it picked up the export dialog's supersampling (so 2x the pixels
  in each direction), and the export path's tile-size floor — an efficiency optimization — was
  allowed to override its own safety budget, making each piece of work up to 3x larger than
  intended. At extreme views where estimated cost equals real cost, those pieces ran for seconds
  each while the live view was still rendering on the same card. Thumbnails now render without
  supersampling under a small strict budget, and the floor can no longer override the budget for
  any export. The remaining refinement (pacing export pieces by their measured cost, like the
  live path now does) is queued.

- **Fixed: the deep-minibrot crash is closed — the worst view we know of now settles to
  completion** (beta.109). The beta.108 note admitted the danger was reduced but not eliminated;
  this closes it. Two final causes: refinement work was entering unexplored cost territory at
  sizes it had only earned in cheap territory (now: every region of the refinement starts at a
  provably safe size and earns growth from its own measured prices, with at most one unpriced
  piece ever in flight); and a second, older refinement path — the spatial tiling — was quietly
  taking over whenever the scheduler felt confident, and each of its tiles pays the FULL
  iteration depth, which at these views is exactly the unbounded cost the piece-splitting exists
  to avoid. Deep refinements now always split along the iteration axis when available. The crash
  recipe (minibrot, maximum iterations, wait) now runs to a fully-settled image and stays up
  indefinitely — the first time that view has ever finished rendering on this hardware.

- **Deep settled views now pace their own work by what it actually costs** (beta.108). Follow-up
  to the beta.107 crash: the same recipe (a minibrot at maximum iterations, left to settle) could
  still reset the card, because the view refined itself with back-to-back pieces of work and the
  cost of a piece varies enormously across one refinement — the cheap stretches kept talking the
  scheduler into sizes the expensive stretches then blew, and a saturated card also silences the
  very measurements that would have corrected it. Each piece is now priced by the wall-clock cost
  of the previous one (a signal saturation cannot silence), grows only on evidence, sheds its
  size the moment a stretch turns expensive, and leaves the card idle gaps under pressure.
  ⚠Honestly: this measurably reduces the danger but does not yet eliminate it at the worst case —
  a minibrot interior at a multi-million iteration count can still reset the card while settling
  (the fix that closes it fully is designed and queued). Until then, such views are safe at
  moderate iteration counts or with automatic iterations on.

- **Fixed: sitting on a deep minibrot at maximum iterations could still reset the graphics card**
  (beta.107). Reported the same day beta.106 landed: zoom into a minibrot, set iterations to
  maximum, wait — the card reset within minutes. The work-splitting introduced for deep views had
  one mis-sized case: on a resting (settled) view, each piece was accidentally sized from the
  budget meant for a whole GROUP of pieces — sixteen frames' worth of work in a single submission,
  at the exact moment a minibrot interior makes every unit of work cost its full price. Each piece
  is now sized from a single frame's budget. A new self-test pins this permanently, and the fix
  was verified by replaying the crashed session unharmed.

- **Very deep views now stay safe for the graphics card while zooming, without the picture turning
  to noise** (beta.106). Deep views with very high iteration counts used to be drawn in a single
  huge piece of GPU work per frame — occasionally so large that Windows reset the graphics card
  mid-zoom (reported when pressing Home from a deep view). The work is now split into many small,
  safely-sized pieces. The first version of that split had a visible cost: while the view was
  moving, the screen showed whichever piece had finished, so deep interiors looked like noise.
  Now, when a moving view is due a detail refresh, the refresh is computed invisibly over several
  frames at a fixed point of the dive and shown only once it is COMPLETE — the picture you see is
  always a whole frame, smoothly following the zoom, with fresh detail streaming in a beat behind.
  A new automated gate (`--motiontest`) drives a zoom and a Home glide and fails the build if a
  partial frame is ever shown or adopted, so this cannot quietly regress.

- **Fixed: two crashes where the app pushed the graphics card too hard and lost it** (beta.105).
  Both were reported from the same machine and both ended the same way — the window vanishing and
  reappearing with your view restored. The app decides how much work one frame may ask of the card
  by measuring how long recent frames took and steering toward a target. That target was set close
  to the point where the driver gives up on a frame, so a steady state that *aimed* at roughly nine
  tenths of a second was working as designed while leaving almost no margin. It now aims at less
  than half that. Three related changes: the app also backs off in a single step rather than
  gradually once a frame does run long, and it no longer raises its estimate of what the card can
  afford using frames measured while it is rebuilding the deep-zoom reference — those frames are
  unusually cheap, and treating them as typical is what inflated the estimate just before the work
  became expensive. Expect slightly lower resolution on very deep views that were previously being
  drawn at the edge of what the card could manage.
- **New: a diagnostics tool for testing the renderer's limits** (beta.105). `--torture` runs an
  escalating ladder of scenarios — increasingly deep zooms, resolutions and rendering regimes —
  with each one launched as a separate process under a time limit, so a scenario that hangs or
  loses the graphics device is recorded rather than taking the whole run down. Failures do not stop
  the run, so a single pass can surface several unrelated problems. Aimed at contributors and at
  anyone reporting a hardware-specific issue; see `design/torture-suite.md`.
- **Fixed: the Linux download's self-test reported normal graphics-card differences as failures.**
  The Linux archive was missing the small file that records which card the reference images were
  made on. Without it the self-test has no way to know it is running somewhere else, so it applies
  the strict same-card comparison and flags the ordinary differences between one vendor's
  arithmetic and another's — which is precisely what that file exists to prevent. The Windows
  archive has always included it. Also, release publishing was reorganised so the Windows and
  Linux builds can no longer race each other: previously the Linux archive was attached to a
  release the Windows build had to create first, and a slow Windows build could have caused a
  release to be published without the Linux download at all.
- **The tour clock now waits for a hold's own reference** (beta.88). The intermittent
  hold-e72 livetest failure (27.6% black whenever the ~52 s deep extension lost its race
  against the tour clock): Adaptive pacing's lag dilation tracks BLA validity but not "the
  reference is still an ask short of this hold's budget", so the clock could walk through a
  hold showing the previous ask's clamped render. The pacer now holds the clock inside a hold
  window while that hold's prefetched reference is in flight — the in-flight build is a live
  progress signal (a dead worker culls it), and the sticky give-up backstop still applies with
  a 6× allowance for this demonstrably-progressing case.
- **Every critical number now lives in one file, and twelve of them can be moved for a single run**
  (beta.104, user-requested: *"I don't like having critical numbers buried in random blocks of
  code."*). The values that govern how much work one frame may ask of the graphics card — the ones
  behind most of this release's rendering incidents — were scattered through two large source files,
  each with a long comment explaining the incident that set it. They are now collected in
  `tunables.rs`, moved verbatim with those comments, with no change to behaviour whatsoever. For
  diagnosis there is also a new developer flag, `--set NAME=VALUE`, that moves one of the twelve
  frame-cost numbers for that run only: it is announced at startup, recorded in any crash report,
  and deliberately made impossible to mistake for a supported setting — the self-test fails outright
  if a run uses one, because the shipped defaults are the only tested configuration. See
  DIAGNOSTICS.md, "Moving a tunable for one run".
- **Fixed: a sharpened deep view was finished but never shown** (beta.103, user-reported —
  *"it does the computation but doesn't update the image, because it shows up almost immediately
  when you resize slightly smaller"*). With "prefer detail" on, a settled view too costly to draw in
  one pass keeps the last complete picture on screen while the sharper one is assembled underneath,
  and swaps to it when the assembly finishes. The test for "still assembling" was never satisfied
  the other way: a finished assembly goes on nominating its last piece every frame (harmless and
  deliberate — the graphics card skips repeated work), and that was being read as "still busy". So
  from the first assembly onward the view kept displaying the coarse placeholder it had started
  from, while the finished, sharp image sat unseen. Nudging the window revealed it instantly because
  interacting suspends the hold — the picture had been ready all along.
- **Fixed: a deep view could stay pixellated until you nudged the window** (beta.102,
  user-reported). Parked at 7.9e100× with an explicit 4,000,000 iterations, the settled picture was
  a grid of coarse blocks; resizing the window slightly re-rendered the same view in full detail,
  eleven times faster per frame. A settled view too costly to draw in one pass is drawn as a grid of
  small tiles instead, one per frame, and the number of tiles it was allowed to spend was switched
  on how large its measured frame budget happened to be. Because that budget converges to "whatever
  fits the safe time slice", the switch was really asking how FAST the view runs — and a view a
  few percent below the line was allowed sixteen tiles, which at four million iterations is 4% of
  the window, forever, while a view a few percent above got 512 and reached full resolution in a
  single frame. Nudging the window moved the view a fraction of a percent onto the fast side of that
  line. The allowance is now simply how many tiles the picture needs, so what a view looks like no
  longer depends on which side of an invisible threshold it lands on. Two related improvements come
  with it: a view still measuring itself keeps the small allowance (so it settles progressively
  rather than grinding through hundreds of tiny tiles), and finishing a better reference orbit no
  longer flashes a coarse frame over an already-sharp one.
- **Fixed: rendering a deep view with a high iteration count could crash the graphics device**
  (beta.101). Exporting at 7.9e100× with an explicit 4,000,000 iterations lost the device every
  time — a hard crash on default settings, since glitch correction is on by default for exports.
  The multi-reference correction passes ran with the iteration-skipping tree switched OFF: the base
  reference's tree cannot be reused for a different reference, and "cannot reuse" had been taken to
  mean "cannot have one". A correction pass was therefore a completely different renderer from the
  one that drew the picture — measured at **0.04 billion steps/second against the base pass's 174,
  a 4000× gap inside the same frame** — so the work-bounding arithmetic that keeps every dispatch
  inside the OS watchdog was pricing those passes with the wrong cost model, and a "bounded" tile of
  49 pixels ran for four and a half seconds. Each correction pass now builds its own tree. The
  render completes, and correction still resolves every glitch it finds (the previously-crashing
  render now changes 131 pixels out of 129,600 and leaves the rest identical).
- **Tours can script the dual-view split, and the orbit overlay stops following your mouse**
  (beta.100, both user-reported).
  - **`dual_split` keyframe field**: the fraction of the width the Mandelbrot panel takes — the same
    thing dragging the divider sets — interpolated between keyframes, so a tour can hand the Julia
    panel more room as it moves into it. The offline tour renderer honours it too; it previously
    hardcoded a 50/50 split, which meant a rendered tour could not match the playback it was
    rendering. Your own divider position is restored when the tour ends.
  - **The script now outranks the cursor during playback.** The orbit overlay picked the point under
    the mouse and fell back to the scripted point only when the pointer was off the canvas — so
    through any tour watched with the mouse resting over the fractal, the overlay followed the
    mouse instead of the presentation.
  - **The grand tour demonstrates both.** Its dual chapter opens even and slides to a two-thirds
    Julia panel as c travels along the boundary; its orbit chapter is now a three-stop story —
    inside the set the orbit closes into a 3-cycle, on the boundary it never cycles and never
    escapes, outside it spirals out and leaves the frame — with the point gliding between them so
    the path visibly reorganises as c crosses the boundary.
- **The Fraktaler-3 comparison corpus is reproducible again** (beta.99). Its regression check had
  been red since July on every location including the simplest one, and the reason was the harness,
  not the renderer: corpus renders patched a handful of settings into the developer's *live*
  session and inherited everything else, so the palette phase, distance-estimate shading, binary
  and duotone modes and half a dozen other image-affecting settings rode along from whatever the
  app happened to be doing last. No build could reproduce those baselines — including the July
  build that made them. Renders now run against a committed session file copied into a throwaway
  config directory: they depend on the program, the command line, and that file, and on nothing
  about the machine. Verified by rendering with a deliberately hostile live session and getting a
  pixel-identical result; baselines re-blessed deliberately (the escape values were never in
  question — the difference was a one-to-one recolour), and the check is 20/20 across depths to
  1.2e1008x.
  - The app now records which session it loaded, and says so when a session file exists but could
    not be read — previously indistinguishable from having no session at all, which is precisely
    how a harness ends up silently rendering with defaults. The corpus generator now insists on
    seeing that its staged session was loaded.
  - ⚠Note for anyone comparing renders: image files carry embedded metadata, so two identical
    images have different checksums. Compare decoded pixels.
- **A deep view no longer throws away the reference it is rendering with** (beta.98). Zooming or
  panning at extreme depth could leave the picture blank: a reference rebuild that happened while
  the view was moving was capped to a short "keep motion cheap" length, and once that cap sat below
  the reference already on screen, any rebuild that could not simply extend the cached orbit came
  back **shorter** — measured at 2e82×, a 1,208,193-sample reference replaced by a 256,001-sample
  one, and the view that arrived a moment later rendered **100% black**. A moving view may now
  decline to lengthen its reference, but never shortens it; growth still waits for the view to
  settle, so a dive costs no more than before. This also lets the expensive precision re-anchor
  (every ~128 octaves of zoom, the cached orbit's accuracy margin runs out and the reference must be
  rebuilt from scratch) happen during the descent, where there is time for it, instead of at the
  moment the camera stops. Same 6-minute deep-zoom validation script, reference pipeline only: five
  deep stops that previously landed on a stale or truncated reference now each render their own,
  and the run finishes 50 seconds sooner.
  - Supersedes beta.97's fix below, which is reverted with it: skipping those "futile" rebuilds was
    treating a symptom of the truncation, and it cost the descent its up-to-date approximation
    tables (frozen, reprojected frames all the way down).
  - **A finished reference no longer overwrites a better one that landed while it was building.**
    Four separate builders feed one cache, and a deep build runs for minutes: measured at 6.5e94×,
    a reference that took 190 seconds installed and was thrown away 97 milliseconds later by an
    older, half-as-long build that had been in flight the whole time. Results now carry the
    reference generation they were spawned against, and a result that is both older and shorter
    than what is already installed is dropped.
  - New `FRACTADYNE_NO_PREFETCH=1` diagnostic: play a tour with its reference lookahead disabled, so
    the live path is exercised the way an interactive viewer exercises it. The failure above was
    invisible for three attempts because the lookahead kept papering over it.
- **A deep hold no longer burns ~70 seconds rebuilding a table that cannot change** (beta.97). At a
  deep view whose reference came back *partial*, the renderer would notice its bilinear-approximation
  table had drifted out of range, rebuild it — and rebuild the identical table, because the fix it
  actually needs is a longer reference orbit, which is capped while the view is treated as moving.
  The trigger then fired again on the next frame. Measured at 2.6e72×: **91 rebuild cycles in one
  run, each spending ~800 ms on a four-million-node table**, with the orbit extension itself taking
  1 ms and changing nothing. That burn was self-defeating — it delayed the view settling, and
  settling is what lifts the cap that blocks the growth. The futile case is now skipped, and only
  that case: a reference that has fully escaped is long enough already, so its rebuild genuinely can
  help and still runs, which preserves the zoom-out tiling fix that trigger was added for.
- **Log-scaled palette mapping** (beta.96) — *Color → Log color scale*, or `--log-palette`.
  Escape values crowd towards the high end at depth: most of a deep frame's pixels sit in the last
  few percent of the range, so mapping the palette linearly spends nearly all of it on a thin
  shell hugging the boundary and flattens everything outside into near-darkness. Compressing with
  `log(v − lo + 1)` first spreads the palette across the range as the eye actually reads it. On a
  1e6× view the difference is stark — the linear render keeps its bright end in a few tight rings
  while the log render opens the whole field up — and it is what keeps colour stable through a
  zoom video instead of washing out as the measured range grows. Applies wherever normalization is
  active (the live "Normalize deep colors" path and `--normalize`); the classic linear mapping
  remains the default and is bit-identical to before.
- **Dithered 8-bit export — banding is gone from smooth gradients** (beta.95). Fractal exteriors
  are enormous, very shallow ramps, which is the worst case for 8-bit output: rounding maps a wide
  span of colour onto one byte value and leaves a visible contour where it steps. That is the
  complaint newcomers raise first. Every PNG now goes through an ordered (Bayer 8×8) dither that
  nudges the rounding threshold by up to half a level based on pixel position. Measured on the
  home view, the mean run of identical horizontal pixels — the thing you actually see as a band —
  falls from **4.89 px to 1.25 px**, and the image uses all 256 levels instead of 252.
  The dither is **ordered, not random, and that is load-bearing**: a random or error-diffused
  dither would make every render differ from the last, breaking the golden images and the corpus,
  and would crawl as static noise over a zoom video — considerably worse than the banding it
  replaces. Bayer is a pure function of `(x, y)`, so renders stay bit-identical run to run and the
  pattern stays fixed to the image rather than swimming through it. Alpha is never dithered (stray
  254s would make an apparently fine image no longer opaque). Verified as a pure ±1-level change:
  before re-blessing, every golden differed by `maxΔ 1, meanΔ ~0.17` and still passed the existing
  tolerance unaided.
- **The path-signature tripwire stops crying wolf on other people's GPUs** (beta.94). The first
  real cross-vendor validation run — an RX 6800 XT, hours after beta.93 shipped — reported seven
  bench-matrix segments as "ALGORITHMIC DRIFT" and twelve self-test checks as DRIFT on a
  perfectly healthy card. The signatures (mode, sa-skip, orbit length, effective iterations, GPU
  event counters) are deterministic for a given build **on a given GPU**, but they were assumed
  machine-*independent*, and they aren't: that card's compiler preserves the df32 error-free
  transforms NVIDIA folds, so escape decisions move by a pixel here and there and the
  rebase/skip counts follow. Differences against a baseline recorded on another GPU are now
  reported rather than failed, and `--bench-matrix` no longer exits 2 for them. On the card that
  blessed the baseline it remains exactly the tripwire it was built to be.
- **Paste a palette into the gradient editor** (beta.94). A *Paste…* box that accepts what people
  actually have: hex with or without `#`, 3- or 6-digit, and 0–255 RGB triples — the Fractint/KF
  `.map` line shape — separated by commas, spaces or new lines, with `;` and `//` comments
  ignored. Longer lists are sampled evenly down to the eight stops the gradient carries (keeping
  both ends, so a 256-entry `.map` keeps its shape rather than importing only its dark end), and
  sRGB is converted to linear on the way in. Deliberately format-tolerant rather than
  format-specific: "I found a palette on the web" is defeated by a format war.
- **Help → Diagnostics…: run the tests from the UI, and attach the result to an issue report**
  (beta.93). Two buttons — *Run self-test* (does the maths hold on this GPU?) and *Run UI test*
  (does it draw and lay out correctly?) — with live progress, the test's own verdict line, and
  **Open results**. A finished run can be attached to *Report an issue…*, upgrading a report from
  "here is my crash log" to "here is my crash log **and** a machine-validated test result from my
  hardware". The dialog is always available: the people it helps most are exactly the ones who
  will never pass a command-line flag, and that is what makes validation on GPUs we don't own
  something a stranger can contribute. Tests run as child processes, so a device loss during one
  kills the test rather than the session you were about to file a report from. The developer
  harnesses (`--livetest`, `--bench-matrix`, `--divetest`, `--juliadive`) stay deliberately
  CLI-only — buttons for those generate confused bug reports, not information. A pass/fail is
  taken from the test's own summary rather than its exit code, because the self-test exits
  non-zero on golden mismatches that are *expected* off the reference GPU, and a red banner there
  would teach testers to ignore it.
- **`scripts/gpu-validate.ps1` / `.sh`: one command to validate a machine, one bundle back**
  (beta.93). The power-user counterpart to the dialog: the same six steps in the same order on
  Windows and Linux, writing the same file names so two machines' bundles diff directly. Runs
  hermetically against a private config directory, so the tester's own settings are untouched and
  every machine renders identically. `summary.txt` leads with the results table and then explains
  which failures are *expected* on hardware other than the reference card.
- **`--gputest` can now say *which line* a compiler folded** (beta.92). Testing primitives in
  isolation had run out of explanatory power: on AMD every one of them — `two_sum`, `two_prod`,
  the armored variant, and `quick_two_sum` — comes back exact, yet `df_mul` still only reaches
  f32 accuracy, so the precision is lost somewhere in the composition. The new row re-implements
  `df_mul` inline and emits its three intermediates (`two_prod`'s residual, the cross-term sum,
  and the final renormalize's residual); whichever one comes back zero names the folded step,
  and the reported error is exactly what any one of them being zero would produce. It paid for
  itself immediately on the reference machine: NVIDIA preserves the first two and zeroes the
  final `quick_two_sum` residual in all 256 cases, which is a complete account of why `df_mul`
  degrades there. On AMD the isolated `quick_two_sum` passes, so if the inlined one is folded,
  inlining itself is the trigger.
- **`--gputest` gains a `quick_two_sum` probe, and stops feeding itself invalid inputs**
  (beta.91). The AMD RX 6800 XT result (Vulkan/OpenGL preserve the error-free transforms exactly,
  proving the shader arithmetic is correct and NVIDIA's compiler is the outlier) left one thing
  unexplained: on that same stack `df_mul`, `df_div`, `c_sqr` and `fe_mul` were still only
  f32-accurate. `quick_two_sum` — the *other* error-free transform, which all of those end in —
  was never tested in isolation, and its shape `e = b - ((a+b) - a)` is even easier to fold than
  `two_sum`'s. It now has its own row, with inputs ordered by magnitude since the algorithm
  requires `|a| >= |b|`. Separately, the harness built its double-float inputs with a
  fixed-band low limb, producing pairs up to **52.8× outside** the `|lo| <= ulp(hi)/2` invariant
  that `df_mul`'s error analysis assumes — enough to fail a *correct* implementation (measured on
  a CPU mirror: 8.64e-13 against a 2.3e-13 tolerance). Inputs are now normalized by construction,
  using integer/bit math rather than an error-free transform, so a machine that folds EFTs can't
  end up measured on different inputs than the CPU oracle uses.
- **The "hardware-dependent deep view" was largely the test harness racing itself** (beta.90).
  `--uitest` screenshots a live band once the reference-orbit length holds steady for 700 ms. But
  the reference build is progressive: it installs an iteration-capped coarse preview first and
  then computes the full orbit in a worker, and at 1e30× that full build runs for many seconds
  while the length sits at exactly 16,385. The gate fired on the preview, and an
  iteration-capped preview of a deep interior field is solid black — so whether a machine
  captured detail or blackness came down to a race against a fixed timer, which is exactly what
  "same view, different result on different GPUs" looked like. The gate now also requires that no
  reference build is in flight, the same lesson as beta.88's pacer fix: an in-flight worker is
  progress that a quiet-period timer cannot see.
- **A shared tour script could kill the app before it drew a frame — fixed** (beta.89). Tour
  `zoom` is a string so a tour can go deeper than `f64`, but that value sizes the bignum
  precision every centre in the script is parsed at, and is re-derived per frame while playing.
  The exponent itself was read as an `f64`, so `zoom = "1e1e999"` produced an infinite log₁₀, the
  octave count saturated on its cast, and the loader asked for a `usize::MAX`-bit number: the
  process died allocating, on nothing worse than opening someone's file. A merely absurd finite
  value like `1e999999999999` did the same thing more slowly (~415 GB per centre). Zoom is now
  bounded at the parse boundary, which covers both the load-time and per-frame paths, with a
  clean error instead of an abort. The ceiling (1e1000000×) is far past anything reachable — the
  deepest verified corpus location is ~4.6e1105× and the extreme-zoom battery runs 1e21000× — and
  all nine shipped tours load unchanged. The `.fdn` loader already clamped depth for exactly this
  reason; the tour path never got the same guard. Pinned by regression tests for each hostile
  shape and by a new tour-TOML fuzz (random grammar tokens, plus byte mutation of a *valid*
  script so it reaches cross-reference resolution).
- **`--gputest` sweeps every graphics backend, and the answer is the same on all of them**
  (beta.89). The harness now runs headless (no window, works over SSH) and grades DX12, Vulkan
  and OpenGL in one pass, because the arithmetic is compiled by the *backend's* shader compiler,
  so "is extended precision real here" can have a different answer per backend. Getting the DX12
  answer at all required compiling that backend in: eframe asks wgpu for only `metal` and
  `webgpu`, so this binary could reach Vulkan and OpenGL but never DX12, which had looked like a
  missing adapter on a card that plainly has one. The report now names the backends compiled into
  the binary and distinguishes "not compiled in" from "no adapter", and every log and crash
  report now names the running adapter and backend. Result on the reference machine: all three
  backends fold the error-free transforms bit-identically — same counts, same worst cases — while
  naga's translated HLSL and GLSL both preserve the arithmetic verbatim, which points at the one
  component the three share. Because the DX12 backend is now in the build, `NativeOptions` **pins
  the app's backend set** to the one this build is validated on: the TDR budgets, dispatch caps,
  goldens and livetest baselines were all measured on it, and a routine rebuild must not silently
  move users to a different shader compiler. `WGPU_BACKEND` still overrides.
- **`--gputest`: per-machine verification of the shader's arithmetic primitives** (beta.88).
  Runs the renderer's own df32/floatexp helpers (error-free transforms, df add/mul/div, complex
  square, floatexp mul/add, 64-step Mandelbrot- and Julia-form accumulation) on hash-derived
  bit-exact inputs and grades every op family against CPU oracles — exact-EFT checks are the
  fused-fma canary. Deep rendering was verified only by goldens blessed on one RTX 3080; on a
  GPU/driver with fast-math contraction or flushed denormals this prints *which primitive* is
  wrong instead of leaving an unexplained black frame. Exit 1 on any failing family; the table
  is designed to be pasted into bug reports. Its very first run found something real: on the
  reference machine itself (NVIDIA, Vulkan backend) the compiler folds the error-free
  transforms — `two_sum`'s residual comes back exactly 0, even bitcast-armored — so df32 has
  effectively been running at f32 precision all along, which retroactively explains the Julia
  tessellation depth and the direct-mode switch point sitting at the f32 cliff. Not a
  regression (status quo of every build shipped); the full account and follow-ups are in
  TODO.md "Open bugs". The report header now names the backend (`… · Vulkan`) since the
  shader-compiler stack is what makes a verdict interpretable.
- **The Julia "tessellation" is fixed: Julia views cross into perturbation at 100× instead of
  10,000×** (beta.88, user-confirmed at J 4,362×). A Julia pixel's identity lives only in its
  starting point z₀ — unlike Mandelbrot, which re-injects its per-pixel df32 c every iteration —
  so direct-mode f32 rounding noise swamps the per-pixel difference from a few hundred × on:
  speckle by ~530×, pretty-but-wrong iteration-plateau tessellation patches by ~1000–4000×,
  healthy again at 10,000× where perturbation used to take over. Perturbation is exact at any
  depth and the Julia reference machinery is the same one deep views already use, so the
  crossover simply moves to 100× for Julia views (`PERT_JULIA_THRESHOLD`). Verified with
  `--juliadive` from 8× to 32,818×: crisp in motion and settled, no seam at either threshold.
- **The dual-view Julia no longer freezes onto a stale texture** (beta.87). Reported as
  "blockiness and an odd artifact at the center while zooming the Julia": the missing-reference
  test in the live path forced reprojection unconditionally — but a Direct-mode view never
  builds a reference, so the Julia panel (which renders Direct at these depths) was
  `will_reproject` from its very first frame, never re-iterated after its first settle, and
  every zoom just magnified that one frozen texture into giant blocks with a bilinear-smeared
  bullseye at the anchor. The test now applies only to modes that need a reference. Reproduced
  and verified with a new dev harness (`--juliadive`: boots dual view and zooms the Julia
  in-app with per-octave screenshots — synthetic OS input proved unreliable). Remaining,
  now-unmasked and filed separately: Julia direct-mode precision degrades from ~300×
  (speckle → patches) — the freeze had been hiding it behind stale-but-clean frames.
- **Share location joins Navigate; menus no longer drive the dual-view Julia** (beta.86).
  "Share location…" moved from File to Navigate — it shares *where you are*, kin to Import .kfr
  and Go to location. And a real bug from the reorg review pass: in dual view, moving the cursor
  through an open menu re-rendered the Julia panel underneath — the live cursor-follows-c update
  was driven by raw pointer geometry; it now requires the Mandelbrot panel to be genuinely
  hovered (layer-occlusion-aware), so menus, popups, and dialogs over the view no longer steer c.
- **Menu reorganization** (beta.85, from an external UI review). **Navigate** replaces the
  Bookmarks/Locations split ("where can I go?" is one question — famous points, random, .kfr
  import, and a Bookmarks submenu together); **Color** is new (method, palette, gradient editor,
  normalize — coloring is what users do most and the menu bar said nothing about it); **Tools**
  now leads with the mathematical differentiators (find-minibrot, the Newton/Misiurewicz solver
  — previously buried in a dialog tooltip) and groups the tour actions, including a new "Render
  tour…" entry; **Fractal** gains Dual view (it selects *what* renders, like Julia mode); the
  **View** menu is decluttered to view/panel concerns, with the orbit overlay's options moved to
  a new control-panel "Overlays" section. Language: "script" → "tour" throughout the UI
  (Play tour, Tour from current view, Render tour), matching the docs. Also: **Ctrl+Z /
  Ctrl+Shift+Z now undo/redo the view** alongside the KF-style Backspace bindings, "Home view" /
  "Reset view" are renamed to say what they do ("Zoom out to full view" / "Reset to default
  view"), and shortcut hints are shown consistently.
- **Progressive (preview-first) render order** (beta.84). `--order progressive` (and "Render
  order: Progressive" in the Render Script dialog) renders the keyframes first, then repeatedly
  bisects the largest temporal gap — a coarse flip-book of the whole tour exists within minutes
  and refines toward full frame rate, so a mis-framed deep keyframe or palette drift is caught
  before hours of GPU are committed. Every frame still lands at its correct `frame_%05d` index,
  so Resume and mp4 assembly are order-blind; stop early, eyeball the arc, resume in either
  order. One caveat, warned at render time: a normalized render's palette smoothing follows
  render order, so prefer sequential for final normalized delivery.
- **Settings moved from View to File** (beta.83) — File → ⚙ Settings, above "Reset application
  state". View keeps only display toggles (panels, minimap, orbits, dual view), which is what a
  View menu is for; app-wide preferences (frame-rate cap, UI scale, theme, update track) belong
  where users look for them first.
- **The Render Script dialog has a real progress bar** (beta.82), parsed from the child's
  `frame K/N` lines, with the elapsed/remaining/fps detail line kept underneath; non-progress
  lines (resume notes, the ffmpeg step) leave the bar in place instead of blanking a long
  render's only visual anchor. Also fixed: a new run no longer inherits the previous run's
  failure reason in its status. This closes the 4K-silent-death item: the hardening it asked for
  (pipe-safe progress prints, the on-disk `render-status.txt` marker with `running` + pid →
  `complete`/`canceled`/`failed: why`) already shipped, so a future silent death is diagnosable
  from the frames folder alone.
- **Deep holds get their reference built BEFORE the camera arrives — the grand-tour livetest is
  clean for the first time ever (22 ok / 0 warn / 0 FAIL)** (beta.81). A hold keyframe with a
  large explicit `max_iter` needs a reference extension costing 25–90 s of bignum — and it could
  not even START until arrival: the lookahead deliberately builds with the short motion cap, and
  the reactive path's settled extension only fires once `interacting` drops at the hold, so the
  hold spent its whole window clamped at the previous keyframe's ask (hold-e82: a 20.6 s-stale
  reprojection; hold-e94: 100% capped-black at the stale 2M orbit while its reference was still
  building). A dedicated hold-prefetch slot now finds the next hold with an explicit ask past the
  motion cap, builds it at destination precision/ask during the glide (seeded from the cached
  orbit), and installs it inside the hold window. hold-e94 now renders BETTER than the old
  baseline (its higher-precision reference survives to the full 4M ask instead of a
  precision-cliff escape at 3.63M); baseline re-blessed. Also: every reference install is now
  traceable (`FRACTADYNE_TRACE=ref` "install v0:" lines).
- **The explicit budget respects latency floors — beta.79's deep-hold regression fixed**
  (beta.80). `--livetest` (its first run since beta.69) caught beta.79's 200 ms explicit target
  collapsing the grand tour's six deep holds from 480×270 to 16×16: a 16×16 mode-2 dispatch at a
  250k+ iteration ask measures ~250–330 ms **no matter how few pixels** (256 threads on
  quarter-million-step dependent chains are latency-bound — the codebase's oldest lesson, met
  from the shrink side), so every reading at the floor read "slow" and the controller cornered
  itself at the 4e6-step minimum with the pinned budget masquerading as converged. Two changes:
  the explicit target is now 400 ms (above every floor measured, still >2× under the ~0.9 s
  lethal band), and a latency-floor guard holds position — converged — when a small slow
  dispatch sits inside a 600 ms accept window, instead of shrinking into the corner. Floors past
  the accept window still shrink (a near-watchdog floor held in place is the beta.48 death loop;
  the honest fix for that regime is iteration chunking for floatexp, still open), and the
  auto-iter regime is untouched by construction. Also attempted and REVERTED same-day: pricing
  tour-glide frames at the depth cap — it rendered hard spar holds 100% black on `--livetest`
  (per-keyframe budgets exist precisely because the depth formula under-budgets hard fields).
  The livetest harness's budget walk is now traceable (`FRACTADYNE_TRACE=gpu` "lt" lines) — the
  regression was undiagnosable without it.
- **Explicit-count dispatches are now priced by MEASURED cost, not worst-case nominal**
  (beta.79). A scripted deep dive (5111×2158 window, ~1.29M explicit iterations at e216) rendered
  ~26-pixel blocks for most of the descent: the flat per-dispatch nominal cap (a device-loss
  guard) priced every dispatch at its zero-skip worst case, so cap-sized dispatches measuring
  54 ms real wasted 4× of safe headroom — while the frame budget sat frozen above the cap
  discarding every reading as undersized ("(settling)" forever). The budget controller now runs a
  separate explicit regime: it converges on 200 ms real (4.5× under the reproduced ~0.9 s lethal
  band), every growth step paced by a real measurement, ceilinged at 3× the old cap so even a
  total skip collapse between readings prices under ~600 ms. Same view: 3× the per-dispatch work,
  ~1.7× finer resolution, and the budget readout actually converges. Also: an unmeasured chunked
  opening dispatch no longer overshoots the bootstrap bound on large panels (the 256-iteration
  anti-thrash floor now applies only once a measurement exists).
- **"Prefer detail" no longer pixelates on a long dive** (beta.78). The stage-A cut froze every
  moving frame, which disabled the existing refresh-every-half-octave cadence — so a continuous
  zoom just magnified the frozen frame into ever-larger blocks (worse than the toggle off).
  Corrected semantics: the reuse-hold keeps reprojecting between refreshes exactly as before, and
  prefer-detail pins those periodic REFRESH frames to native resolution (instead of the adaptive
  motion resolution) — full-detail frames streamed at their real cost, with the hold never more
  than half an octave (~1.4×) past a sharp frame.
- **"Prefer detail while zooming" is complete — the settle is present-gated too (stage B)**
  (beta.77). With the toggle on, the view now never shows a partial composite: when rendering
  resumes after motion (or any refinement runs — settle tiles, chunked iteration, budget probes),
  the display keeps serving a snapshot of the last complete frame, geometrically tracked, while
  the new frame builds invisibly underneath — and reveals it whole when it finishes. Across the
  anti-alias ramp each revealed image is complete at its quality level. Interaction hands off to
  stage A's motion reprojection; everything stays within the bounded-dispatch machinery, so the
  wait is exactly as long as the render — never a risk.
- **"Prefer detail while zooming" (stage A)** (beta.76). A new Navigation toggle: while zooming or
  panning, the view keeps showing the last fully detailed frame — scaled and panned to follow the
  motion via the existing zoom-reprojection path — instead of re-rendering at reduced motion
  quality every frame (KF-style stepping). Rendering resumes in full when the motion pauses.
  Perturbation modes only (shallow direct views are cheap and sharp live either way); default off.
  The follow-up stage will present-gate the settle composite too (render the complete frame
  offscreen, swap it in whole).
- **Segment rendering for multiple machines: `--segments N --segment-index K`** (beta.75). Shard a
  tour's whole timeline into N contiguous, gap-free frame ranges and render only range K — run one
  shard per machine and the frames union to exactly the full video. Shard k covers
  `[⌊k·F/N⌋, ⌊(k+1)·F/N⌋)`, the formula that guarantees no missing or duplicated frames at the
  boundaries (pinned by a unit test across divisible, non-divisible, and more-shards-than-frames
  cases). Global frame numbering is kept, so the collected shards drop into one folder and the
  usual MP4 assembly works unchanged; combines with the chapter-level `--segment NAME` by
  intersection. `--dry-run` prints an invocation's exact frame range and exits without touching
  disk, so a farm script can verify its shards tile before committing hours of GPU.
- **Space no longer zooms while typing, and the player can switch scripts** (beta.74). The
  hold-Space continuous zoom read the key even while a dialog's text field owned the keyboard —
  typing a space into the "Script to current view" note nudged the zoom underneath; both Space
  readers now yield whenever any widget wants keyboard input, like the discrete hotkeys already
  did. And the playback transport gained a 📂 button: play a different script via the same picker
  as Tools → Play script, without closing the player first.
- **The Performance panel's FPS is now state-aware** (beta.73). The number was the raw repaint
  cadence — truthful about the UI loop, misleading about rendering: an idle settled view showed
  "1.0 FPS" (the heartbeat) while computing nothing, and a long capped refinement showed tile
  cadence as if it were completed frames. The line now says what the renderer is doing: `refining
  k/N (~m:ss)` during a settle grid (with progress and an ETA), `refining NN%` during a chunked
  progression, `building reference` while the orbit builds, `idle` when settled and quiet, and a
  real frames-per-second number only when frames genuinely render (interaction, palette/orbit
  animation, motion) — where the old number was already correct.
- **Spinner during long refinements, and the warning label moves as one unit** (beta.72). The
  "working" spinner previously showed only while a reference orbit was building — during the
  minutes-long capped settle/chunk refinement of an explicit-count view (reference done, tiles
  still landing) the view looked stalled; the spinner now also shows for those progressions
  (auto-iteration settles are sub-second and keep the old behavior). And the status bar's
  diagnostic label no longer breaks internally at the panel edge (the ⚠ stranded on one line,
  "iter capped" on the next) — it extends/moves as a single unit in both of its states.
- **The status-bar/settle feedback loop is fully closed** (beta.71). beta.70's reserved slot was
  not enough: an empty reserved allocation's height in a wrapped layout still differs from a
  rendered label's (measured: the bar oscillating 34↔41 px), and every bar-height change resizes
  the central panel — which the resize-detector rightly treats as an interaction, bumping the view
  generation and tearing down the settle grid. The result was a perpetual loop: label toggles → bar
  reflows → "interaction" → grid dies → re-render → counters flap → label toggles. The slot now
  renders the IDENTICAL monospace widget in both states — the label padded to the widest variant's
  length when a diagnostic binds, the widest variant drawn fully transparent when none does — so
  the bar's layout is invariant by construction. Soak at the affected view: zero panel resizes
  after startup, the settle grid completes to native and goes quiet. Two new gated diagnostics
  (`FRACTADYNE_TRACE=tile`: "active:" and "panel resize:" lines) make any future phantom
  interaction immediately visible.
- **The status bar's limit diagnostic no longer reflows the view** (beta.70). The "⚠ iter capped"
  (and siblings) label appears and disappears with live counters; at a window width where the bar
  just fit one line, its arrival wrapped the bar to two, shrank the view, and forced a re-render —
  whose counters then moved the label again (the same reflow family as the beta.58 cursor-readout
  fix). The diagnostic slot is now always present at a constant width, sized to the longest label
  via font metrics, blank when nothing binds.
- **Explicit-count dispatch cap — the third device-loss shape is closed, and deep explicit views
  render native via many safe tiles** (beta.69). A zooming crash on beta.68 revealed the mode-2
  (floatexp) sibling of the class: at a deep view with an explicit 4M count, the settle was
  composing the frame from budget-sized tiles that had converged to ~900 ms of unpreemptible GPU
  work each — the same intermittently-lethal regime proven twice on beta.67's climb soaks. The fix
  generalizes what those fixes learned: with auto-iteration OFF, **every single dispatch — tiles,
  chunks, and frames alike — is capped at ~2e10 nominal steps** (~60–200 ms real even with zero
  skip), replacing the narrower rebase-grind cap. Because the 900 ms convergence band is now
  unreachable by design, the tiled settle arms on a capped budget ("as converged as allowed") and
  its tile allowance grows to the full ceiling, so the native frame is composed from ~240 cap-sized
  tiles instead of resting at a coarse single-dispatch resolution. Verified at the exact crash
  view: native 1445×1134, six-minute soak, zero device loss. Auto-iteration views are untouched;
  the grand-tour livetest (tours run explicit counts) stays 22/22.
- **Iteration-range tiling extended to df32 perturbation — the pixellated spar view now renders
  at native resolution** (beta.68). beta.67 left a view with a huge explicit count over a short
  escaped reference resting at a safe but coarse resolution (79×62); the resumable chunked path now
  covers mode 0 as well as Direct, carrying δz, the floatexp derivative, and the reference position
  (rebasing across chunk boundaries) between bounded passes. The 197k× spar session at an explicit
  4,000,000 iterations now climbs to full 1445×1134 progressively — ~400 self-sized chunks
  (~12k iterations each, ~60 ms dispatches under the rebase-grind cap) — then rests quietly, with
  zero device loss over a 3-minute soak. Verified bit-identical to the single-pass render offline,
  including a deliberate 97-sample-reference rebase-storm case (5 `iter-chunk` checks, 0 texels
  differ). Also fixed two latent gates in the beta.65 direct chunking: non-holomorphic formulas
  (Tricorn/Ship/Celtic/Buffalo/Phoenix/Newton) and aux coloring methods (stripe/TIA/trap/
  decomposition) now correctly fall back to the single-pass path instead of chunking with wrong
  math. Floatexp (mode 2) chunking remains future work — its rebase-grind case stays safely capped.
- **The startup pixellation deadlock is fixed, and the rebase-grind regime is capped safely**
  (beta.67). Restarting into a view with a huge explicit iteration count left the screen at giant
  pixels forever: the frame budget can only climb when a measurement arrives, a measurement needs a
  dispatch, and a dispatch needs the frame to re-key — but at the 16×16 resolution floor one ×1.5
  budget step is too small to change the resolution, so the climb froze one step off the bootstrap
  (interaction used to mask this by re-keying every frame; before beta.65/66 this state usually
  crashed first). A paced budget-climb probe now forces re-measures on settled unconverged views,
  and the ratchet runs 16×16 → full budget-afforded resolution. Two climb soaks also proved that in
  the **rebase-grind regime** — an explicit count far beyond a short escaped reference, where BLA
  covers nothing and nominal cost is real cost — the controller's 900 ms dispatch target is
  reproducibly lethal on this hardware class, so that regime's single-dispatch allowance is capped
  at ~2e10 nominal (~60–90 ms): the view rests at a safe sub-resolution (79×62 at a 4M ask)
  instead of climbing into the device-loss band. Deep views (long references, BLA effective) and
  auto-iteration views are untouched. Native rendering of such an ask needs iteration-range
  chunking extended to perturbation modes — tracked.
- **Budget derate on reference-length collapse** (beta.66). A sibling of the beta.65 crash, hit
  while wheel-zooming on beta.65 itself: a multi-octave interactive jump re-picks the reference
  from millions of samples down to a short escaped one (90), per-step cost explodes (rebase every
  ≤90 steps, no BLA coverage), and the frame budget — measured at deep skip-effectiveness, sitting
  at its ceiling — mispriced the next dispatch into a device loss. The install derate now also
  fires on a length **collapse** (it previously only saw growth), dropping the budget to at most
  the bootstrap so the next frames re-measure from safe ground. Pinned by a pure-predicate unit
  test (the interactive jump has no scripted repro); a 60s scripted zoom-out at 4M iterations runs
  clean with no false trigger.
- **Iteration-range tiling: the zoom-out-from-deep device loss is fixed, and any explicit
  iteration count now renders safely** (beta.65). Zooming out from a deep multi-million-iteration
  view to a shallow one killed the GPU: the shallow view switches to Direct mode (no reference, no
  BLA skip), the huge count came with it, and one dispatch of `16×16 × 4M` real steps tripped the
  OS watchdog — the live cost-bound could shrink resolution and supersampling but never the
  iteration count, and capping the count would break the "explicit count honoured verbatim"
  guarantee. The fix honours the count **across frames** instead: a Direct frame whose cost exceeds
  the budget renders through a new resumable path — one bounded pass over an iteration range per
  frame at FULL resolution (no more 16×16 collapse), carrying per-pixel state (z, dz, iteration,
  status) in ping-pong textures, the cursor advancing while the view holds still and restarting on
  interaction. Escaped pixels appear progressively; a settled view runs to completion and is then
  exact. The offline `render_iter_chunked` variant is verified **bit-identical** to the single-pass
  render (new `iter-chunk` selftest group, 3 cases, 0 texels differ). Verified live at the exact
  crash configuration (shallow view, 4,000,000 explicit iterations): no device loss, responsive
  throughout, the progression completes in ~22 bounded chunks. Suite 111/111, goldens 17/17,
  livetest 22/22, A1 verbatim intact. Devices that can't grant the 48-byte color-attachment limit
  the state textures need fall back to a bounded-and-capped dispatch (safe, capped image).
- **Offline tour render: per-frame cost and lookahead memory are now bounded** (beta.64). Two
  latent ways a long render could die are closed. (1) A tour frame's GPU dispatch is now split into
  tiles capped at a TDR-safe work budget — a shallow, all-interior keyframe asking millions of
  iterations previously issued one multi-second dispatch and lost the device; it now renders as many
  short tiles instead. (`ExportRequest` gained an optional `work_budget`.) (2) The reference
  lookahead (which builds frame N+1 while N renders) now checks available system memory first — a
  new `sysinfo::available_memory()` — and builds synchronously when a second big bignum reference
  wouldn't fit, instead of OOM-killing the render (which previously happened at frame 221/233 of a
  deep 4K tour). Deep BLA-skipping holds render a little slower as a result (the fixed budget
  over-tiles them); a measurement-based tile budget is TODO'd.
- **`--uitest` deep-band capture is now reference-complete** (beta.63). The live bands wait to
  screenshot until the progressive reference orbit stops growing (build finished) rather than on a
  capped-fraction heuristic, so the capture point is machine-independent. This surfaced a real
  finding — the deep floatexp view's reference length and iteration budget vary sharply by hardware
  (3080/Windows builds a 30k reference / 22k-iter budget → dark; 3070/Linux a 2M reference / 44k
  budget → detailed, same point + precision) — now tracked in TODO's Open bugs.
- **`--uitest`: scripted UI + live-render validation bundle** (beta.62). A new dev mode walks
  every UI screen (menus, all dialogs, panels, minimap, palette editor) and the live-render path at
  each mode band (Direct / df32 / floatexp), screenshots each via egui's viewport-screenshot
  round-trip, and writes a review bundle — `NN-screen.png` + `report.md`/`report.json` + `log.txt`
  — with a verdict per step (screenshot captured, frame not blank, live RenderMode matches the
  depth, a real iterate ran). It also resizes the window wide/medium/narrow and checks the status
  bar's height is **stable** at a fixed width (catching the one-line/two-line waver) and reports
  where it wraps. Runs under the real eframe loop (wrap with `xvfb-run` when headless); defaults its
  output to the mounted share, else `logs/`. Wired into `scripts/linux-report.sh --uitest`.
- **File dialogs remember the last directory** (beta.61). Every open/save dialog now seeds from
  a shared, persisted last-used directory and updates it on each pick, so opening a script, saving
  a `.fdn`, importing a `.kfr`, saving a report/benchmark, or choosing a render folder all reopen
  where you last were instead of resetting to a fixed default. Category memories (image-export
  directory, the last script's own folder) still take precedence when set.
- **"Script to current view" produced an unloadable file past ~1e19×** (beta.60). The zoom was
  written with a bare `{magnification}`, and Rust's `f64` Display never uses exponent form — so
  1e87 became an 88-digit integer literal that TOML rejects as an i64 overflow ("zoom too large
  to fit in the target type"). It is now emitted as a quoted scientific string (`zoom = "5.4e87"`)
  at every depth. The generator round-trip self-check gained a mid-depth (~1e85×) case that
  exercises the finite-magnitude branch the old past-f64 case skipped. Also: a script that fails
  to load now opens a dialog titled **"Could not load script"** instead of borrowing the
  benchmark window's "Benchmark results" title.
- **The 2:58 device loss and the black deep holds — one root cause** (beta.48–50). The tour
  camera ROUNDED a pinned-centre glide's coordinates to the current depth's precision
  (`lerp(x, x)` is not the identity), and a rounded centre is a genuinely different point whose
  orbit spuriously escapes at a precision-set length — every mysterious reference length of the
  release cycle (626, 602,516) was `escape(round(centre, bits)) + 1`, and the deep holds only
  ever looked healthy when such a numerically-escaped orbit slipped past the freeze guard. Three
  coupled fixes: exact pinned-centre glides; a cliff rescue in reference selection (rescan at the
  build precision when no candidate survives — with a selftest pinning the exact bad pick); and
  the freeze guard now INSTALLS long partial references at a floor frame budget instead of
  refusing them, which retired the refused-extension rebuild loop and turned every known-black
  deep hold (1e61 / 1e63 / 1e82) green against the offline oracle. The frame-budget controller
  can also finally shrink below its own opening guess (its clamp used the bootstrap constant as a
  FLOOR — a safety valve that could not move toward safety), and the install derate no longer
  fires on ordinary partial-orbit growth, which had pinned the budget at its floor and rendered
  the whole deep chapter pixellated. Verified end to end: 0 device losses across the repro
  battery, the grand-tour live baseline all-green for the first time, and a 60-second
  `ultra-dive-e200` exercise tour (e30 → e200, five staged holds, all matching the offline
  oracle exactly) blessed as a standing gate.
- **An explicit iteration count is honoured verbatim** (beta.53). Auto-iter OFF now means the
  count reaches the GPU untouched — 10,000,000 in the Iterations box used to render as ~82,000
  (the zoom cap × the adaptive boost silently applied). One shared formula now feeds both the
  dispatch and the adaptive probe's staleness gate, so the two can never drift; tour keyframes
  get their scripted budgets immediately instead of boost-climbing toward them.
- **`--version`, and a tour render that survives its parent** (beta.51). `--version`/`-V` prints
  one parseable line. The tour renderer's progress printing is pipe-safe (a parent GUI exiting
  used to PANIC the child mid-render), and the output directory now carries `render-status.txt`
  (`running` with pid → `complete`/`canceled`/`failed: why`), so an interrupted render is
  diagnosable from the frames folder alone.
- **Tours: `minimap` keyframe field** (beta.52) — scripts can show the minimap overlay while
  panning between locations (the grand tour now does); the viewer's own minimap/orbit toggles
  are restored when the tour ends.

- **A refused reference extension is no longer forgotten** (beta.47). The freeze guard drops a
  reference that is still partial past `LIVE_REF_CAP`, and recorded why — but any later install
  cleared that record, and two of the three rebuild triggers never consulted it, so the same
  doomed build could be requested again. Both are closed: the record now survives a shorter
  install (only a longer one, or a re-anchor to a different point, supersedes it), and a rebuild
  that can only be refused again is suppressed at the spawn. Scoped to settled builds past the
  cap — suppressing ordinary rebuilds too froze a deep hold on a 31-second stale reprojection.
- **Live-path regression gates** (beta.47). Goldens are offline export renders, so none of them
  can see a live-path bug. Two additions close that: `--livetest` is now graded against a blessed
  baseline (`--bless`, mirroring `--bench-matrix`) and fails on CHANGE rather than on failure, so
  a tour whose deep holds fail for a known reason can still be a green gate; and a new
  `live-res` selftest group asserts both halves of the frame-budget invariant — an unmeasured
  budget bounds the first dispatch, and does not bind the settled resolution. Suite 102 → 104.
- **Frame-cost measurement: no GPU left rendering at a third of its resolution** (beta.47).
  A device without `TIMESTAMP_QUERY` never published an iterate timing, so the frame budget sat
  forever on the bootstrap constant — a value chosen as a safe FIRST dispatch on unknown hardware,
  never as a statement about what a view can afford — and since beta.40 that floor drove the
  resolution shrink in every mode. Measured: a 1445×1134 panel at 2,000 iterations rendered at
  **504×396**, permanently, with the tiled settle also gated off. Now:
  - **An unmeasured budget may bound the first dispatch but never binds resolution downward.**
    Tiling no longer requires a measured budget, and a never-measured one may arm a grid — every
    tile is independently budget-sized, so this means more small dispatches, not a bigger one.
  - **A wall-clock fallback** takes over when timings stop arriving, tripped by OBSERVATION rather
    than by the capability bit, so a sink that silently never delivers is covered too.
  - **Timings are paired with their own dispatch's cost at the producer.** The readback lands 2–3
    frames late and a tiled settle dispatches every frame, so the app's single "last steps" slot
    was always describing a later tile; every reading was discarded as undersized and the budget
    froze. A settled view now ratchets to native resolution instead of stalling part-way.
  - `FRACTADYNE_NO_TIMESTAMPS=1` makes the whole path reproducible on hardware that has the
    feature, and `render::controller_props` pins the controller's properties in `cargo test`.
- **Playback & render reliability** (beta.37–46):
  - Tour centres parse at the tour's deepest precision (beta.37), and the lookahead no longer
    spins re-spawning builds (a live device-loss class, beta.38).
  - **A playback player**: scrub bar, width-stable transport, outlives the tour (beta.39).
  - **Frame cost is bounded in every arithmetic mode**, not just floatexp (beta.40); the budget
    can bootstrap on a static view instead of leaving it coarse forever (beta.41); a tour hold
    can no longer stall the clock indefinitely when a reference is refused (beta.42).
  - **Silent deaths are visible** (beta.43): aborts and out-of-memory kills now leave a crash
    report (allocator hook + unclean-exit marker + `--oomtest`).
  - Export/render dialogs gained standard size presets incl. ultrawide/DCI (beta.44–45), renders
    report failure reasons and disk-cost estimates up front (beta.45), and tour renders are
    RESUMABLE — re-run with `--resume` and only missing frames render (beta.46).
- **Live deep-dive pipeline** — a scripted dive now stays smooth to extreme depth:
  - `best_reference` candidate scoring **parallelized across all cores** (result-identical;
    ~12–14× — 796→55 ms at 1e400×, 7.6 s→0.6 s at 1e1216×), removing the stall that blurred
    live dives past ~1e400× into a monocolor reprojection (beta.7).
  - **Reference lookahead** — during script playback a queue of workers pre-builds the
    references the dive is about to need (targets bisected onto the script's future path,
    0.5-octave spacing tuned so the active reference's lag never clips the pacer/freeze
    thresholds), installing each as the dive arrives (beta.8–12).
  - **Pipeline pacing** — the tour clock dilates (and interactive zoom-in damps) when the
    reference pipeline lags: the dive slows instead of blurring (beta.7).
  - **Time-floored detail refresh** — the reuse-hold now also expires on time (~150 ms), so
    slow deep dives stream real frames instead of one per 0.5 octaves of zoom (beta.13).
  - **Motion-resolution controller redesign** — adapts only on intervals following a real
    re-iterate frame (reprojection frames carry no iterate-cost signal), proportionally from
    the raw interval; divetest-verified: 54–201 hitches/12 s → 0–4 at 1e100–1e400× (beta.15).
- **Updates & release tracks** — in-app update check against GitHub Releases (Help → Check
  for updates, optional on-launch check, `--check-updates`), persisted **Stable/Beta** track
  where Beta always offers the newest of either channel, an "Update available" dialog with a
  direct download link, and `release.yml` publishing `-beta` tags as pre-releases (0.2.39,
  beta.2, beta.4). The repository went **public** to enable the Releases API.
- **Tours** — **Tools → "Script to current view…"**: one-click dive-tour generator (notation
  caption, depth-scaled duration; deep targets use the pan-shallow-then-dive structure that
  keeps every frame centered on the target) (beta.5–6).
- **Transport controls for script playback** (beta.36) — restart / back 10 s / pause / stop /
  forward 10 s / speed (0.5–4×) / loop, as media icons in the status bar, appearing only while a
  script runs. This required making the tour clock an ACCUMULATOR (`cur_t += dt · speed`) rather
  than deriving it from a wall-clock origin: pause, seek and speed cannot be expressed by moving an
  origin the pacer is already moving for its own reasons. A seek moves the clock only — the camera
  follows through the normal sampling path, so scrubbing needs no special case anywhere else. The
  status bar also WRAPS now: it was a fixed row that silently clipped, which put the whole
  transport off the right edge on any window narrower than ~1600 px.
- **Render a script from the GUI** (beta.36) — a 🎬 button on the playback transport opens a
  "Render script" dialog: output folder, frame prefix, size, fps, supersampling, chapter picker,
  mp4, overwrite — seeded from the script's own `[render]` block — with a live frame count and a
  progress readout. It renders as a CHILD PROCESS (`--render-tour`) rather than in-process, which
  is the point: a deep tour render is the heaviest thing this app does and the failure mode on
  record is a lost GPU device, so in a child that kills the render and leaves the session alive.
  It also reuses the code path everything else is measured through. "Copy command" yields the
  equivalent command line.
- **Playback transport** (beta.36) — restart / back 10 s / pause / stop / forward 10 s / speed
  (0.5–4×) / loop, in a floating bar over the top-centre of the view. It is not in the status bar
  because that bar is sized by its content: the centre coordinates gain and lose digits as the view
  moves, so controls placed there slide horizontally under the cursor. Seeking moves the tour clock
  only — the camera follows through the normal sampling path — which required converting playback
  from a wall-clock origin (`now - t0`) to an accumulating clock, since a derived time cannot
  express pause, seek or speed.
- **The tour no longer flies through black** (beta.36) — a keyframe that changes both centre and
  zoom interpolates them together, so leaving Seahorse Valley for Elephant Valley crossed the
  cardioid's interior while still at ~1e5×: every frame between them was solid black, and the
  status-bar screenshot that caught it showed an entirely black view. Landmarks are now approached
  and left the same way the deep dive already did it — zoom out in place, pan at the home zoom
  where the whole set is visible, then zoom in. Measured over the three affected chapters: frames
  more than 90% black went from 3 to 0. The general fix is a Van Wijk–Nuij path, which derives
  this in closed form rather than asking every author to remember it (TODO).
- **Script playback is visible in the status bar** (beta.36) — a spinner, `mm:ss / mm:ss (N%)`,
  and an explicit "waiting for detail" when the pacer has stopped the tour clock. A deep hold can
  legitimately sit on one frame for many seconds and the pacer deliberately freezes the clock while
  the renderer catches up, so a frozen percentage was expected behaviour that looked exactly like a
  hang — it was reported as "the script stopped".
- **The grand tour's deep chapter descends at half the old rate** (beta.36) — it ran at ~1 decade
  per second, which no deep view can resolve at (the reference rebuild alone costs seconds past
  1e60×), so the camera outran the renderer. The regression holds doubled to 6 s (10 s at the
  final one) and the tour is 316 s rather than 232 s.
- **Live tour playback lost the GPU device — FIXED beta.36.** A regression from beta.35's settling
  change, plus a latent bug it exposed. Once holds settled, the progressive-AA ramp ran during
  playback and climbed to **ss=8 — 64 samples per pixel** — and the crash report's new live
  manifest states the frame exactly: `1431x1134 ss=8 iter=12000 steps=1.246e12 budget=4.000e8`,
  a trillion-step dispatch against a 4e8 budget. Three fixes:
  - **No settle AA ramp during scripted playback.** A tour's camera moves again in seconds, so the
    ramp (which exists to sharpen a view an idle *user* stopped on) bought nothing and cost
    quadratically. Settling during a hold is for the iteration budget and the reference build.
  - **The watchdog supersampling cap now exists in every arithmetic mode.** It was gated on the
    floatexp path (`max_ss_tdr = if is_fe {…} else { u32::MAX }`) — no cap at all elsewhere. The
    crash was at a *shallow* mode-0 view, and the same frame is reachable interactively with a
    high manual iteration count and AA 8, so this was a latent crash independent of tours.
  - **Live playback is no longer classified as an offscreen one-shot.** It paints the window every
    frame, so it must obey the measured frame budget and may use the tiled settle — being lumped
    in with `--render` pinned it to the pre-measurement ceiling and disabled tiling.
  - **New: the live path records a manifest.** A live device loss used to write `manifest:` empty,
    because only the *export* request builder set one — so an on-screen crash said nothing about
    the frame that caused it. This is what turned the diagnosis from inference into one line.
  - **New `--play FILE`**: start the GUI with a tour already playing, the only way to exercise the
    on-screen playback path from a command line (no headless harness reaches present/watchdog).
  - `[playback] settle_timeout` on the shipped tours is **15 s, not 90 s** — measured: at 90 s a
    live tour stalled (a 320-second stretch with no reference build at all), at 15 s it played
    steadily start to finish, 286 s of wall clock against an authored 232 s.
- **The live view no longer goes black during a scripted tour** (beta.35), plus the harness that
  found it:
  - **`--livetest`** — a headless harness that plays a tour through the *live* pipeline
    (`advance_playback_core` + `build_params`, exactly as `--divetest` does) but keeps the pixels,
    and at every keyframe hold renders the same view through the offline path as an oracle. The
    contract it enforces: *the live view should show what an offline render of the same view at
    the same iteration budget shows.* Reports excess blackness and sRGB difference per checkpoint,
    dumps live/truth image pairs for failures, and exits non-zero so it can gate a release.
  - **Fix: a scripted tour marked every frame as "interaction."** Playback stamped the interaction
    timestamp on every tick, so the view was permanently on the cheap moving path — correct during
    a glide, wrong during a hold. Because the adaptive iteration budget only measures and adapts on
    *settled* frames, it could never climb during a tour: measured at the three-spar holds, the live
    view ran at the unboosted depth cap (49k–83k iterations) against script budgets of 250k–4M and
    rendered **100% black at 1e61×, 1e72× and 1e82× where the offline render of the same view is 0%
    black**. Holds now settle, so the budget climbs and the AA/tiled-settle ramps engage.
  - **`[playback] pace` script option** — `realtime` (never dilate; what a benchmark wants),
    `adaptive` (default: slow down while the reference pipeline lags), or `settled`: stop the clock
    at each hold until the view has actually resolved, bounded by `settle_timeout`. Settling makes
    the budget able to climb; `settled` gives it the time to, since at depth convergence needs more
    settled frames than a few seconds of hold provides. The four deep shipped tours set it.
- **Tour script format v2 — BREAKING** (beta.34). v1 scripts are rejected with a migration
  message rather than mis-played; the six shipped tours are migrated. What changed and why:
  - **Absolute keyframe times** (`t` = the second the camera arrives, plus `hold`) replace
    cumulative `secs`. Inserting or lengthening a keyframe used to silently desync every
    caption after it.
  - **Per-keyframe `max_iter`**, interpolated geometrically along each glide. One script-wide
    budget cannot serve both a 1.33× home view and a 1e94× dive: an exact count made shallow
    frames cost minutes each, and a depth-scaled base left the deepest holds capped. The grand
    tour now spends 2 000 iterations on the whole set and 8 000 000 at 6.5e94×.
  - **`[render]` block** (size, fps, ss, prefix, out, mp4, iteration budget, HUD) so
    `--render-tour x.toml` with no flags reproduces the intended output; CLI flags override.
  - **`[[location]]`** named coordinates — a 120-digit dive center is written once and
    referenced by keyframes and annotations.
  - **One `[[annotation]]` array** tagged by `kind` (caption / callout / spotlight), replacing
    three parallel arrays, plus stable `id`s on keyframes and annotations.
  - **`[[segment]]` chapters** and `--segment NAME`: render one chapter in isolation, keeping
    the global frame numbering, instead of re-rendering a thousand frames to fix ten seconds.
  - **`[[palette]]` definitions** with a per-keyframe reference, interpolated between
    keyframes — static palettes, morphs, and cycling through one mechanism.
  - `zoom = "6.5e94"` strings replace `mag` / `mag_log10` and their precedence rule.
  - New `script` selftest group (5 checks): every shipped tour resolves, absolute timing and
    the geometric budget ramp, deep zoom strings past f64 range, ten malformed scripts
    rejected, and `--segment` lookup.
- **The deep-view rendering arc** (beta.19–33) — one Misiurewicz location family exposed eight
  distinct depth bugs; fixing them produced the adaptive machinery:
  - **Adaptive live iteration budget** (beta.19): the view measures its own capped-pixel fraction
    on the GPU and raises the iteration budget until the image resolves — no other renderer
    self-corrects a starved view live. Applied on motion frames too (beta.22), taught to climb
    through plateaus (beta.26) and not to give up on views needing a large boost (beta.32).
  - **Live palette auto-normalization** (beta.20) and **WYSIWYG exports** (beta.21): deep views
    stopped aliasing into noise live, and a GUI export now matches what the screen shows.
  - **The iteration ceiling lifted 500k → 10,000,000** (beta.28) with references that follow the
    budget (beta.27), a **1 GiB orbit binding (7.4M samples)** (beta.30), and **status-bar
    diagnostics that say which limit binds** — depth wall, reference clamp, iteration cap —
    instead of leaving a black screen unexplained (beta.31).
  - **GPU device-loss recovery** (beta.29–30): a lost device writes a crash report and restarts
    the app with the session intact; in-app issue reporting points at GitHub Issues.
  - **The grand tour** (beta.33): a 5½-minute scripted demo whose final chapter holds at every
    depth that ever broke the renderer — the demo reel is the regression gauntlet.
- **Navigation & mathematics** (beta.23–25): exact rational/complex coordinate entry
  (`-3/4`, `(1+i)*(1-i)/2`); **Newton–Raphson zoom** — jump straight to a minibrot's own scale
  with the atom-size estimate; **multiplier λ at Misiurewicz points**.
- **Window resize correctness** (beta.16–18): the "squashed view" resize regression fixed, with
  `--resizetest` and a real-window smoke test to keep it fixed.
- **Dev tooling** — `--bench-matrix`: a 22-segment path-coverage perf + regression suite
  (deterministic GPU-counter signatures vs a blessed baseline; its 20 fast segments joined
  `--selftest`, growing it 63 → **83 checks**) (beta.3); `--divetest`: a headless live-dive
  perf harness playing real-time tour windows per depth band through the actual playback
  machinery (beta.14); `--frametest --center`; per-frame live perf capture
  (`FRACTADYNE_PERF=1`).
- **UI** — bookmark thumbnails inline in the Bookmarks menu (0.2.37); the Controls panel
  scrolls when it doesn't fit the window (0.2.38); `Min motion resolution` tooltip documents
  the sharpness-vs-smoothness trade.

## 0.2.36

Released 2026-08-04 — the first stable release since v0.1.11, rolling up **0.2.1 – 0.2.36**.
Per-version detail is in the git history.

- **Validation corpus fully resolved — all 20 locations genuinely match Fraktaler-3, to
  4.6e1105×.** The chain of deep fixes: extended-range orbit samples (dips below f32's
  flush-to-zero no longer break rebasing, 0.2.6); palette-cycle aliasing at dense deep fields
  diagnosed as coloring, not rendering — fixed generally by `--normalize` auto-normalized
  export coloring (0.2.17); a missed rebase check at BLA-skip landings on near-zero orbit
  dips (0.2.18); KF/F3 zoom-definition alignment (`REFERENCE_HEIGHT` 3→4 — our magnification
  now equals F3's zoom, 0.2.21); location 07 replaced with a true F3 save (me30). Corpus
  regression gate (`generate_corpus.py --check`) + per-location `.fdn` repro files.
- **Diagnostics shipped end-to-end** (design/diagnostics.md D1–D4): log file, crash reports
  with breadcrumbs/manifest/backtrace, a liveness watchdog, `FRACTADYNE_TRACE` categories,
  perf JSONL, GPU event counters (per-tile u64 sums), hermetic streaming selftest
  (61 → 63 checks + 17 goldens; config reset at entry), `--selftest-filter/-list`, CLI exit
  codes on failure, and the DIAGNOSTICS.md operator manual (0.2.7–0.2.11).
- **Extreme-zoom stability** — the ~1e2100× freeze-on-load fixed by capping the LIVE
  reference-orbit length (`LIVE_REF_CAP`, 0.2.15 → 0.2.23/0.2.25/0.2.26 — pixels iterate past
  the short reference by rebasing, so borders still resolve); tiled settle sharpens settled
  deep views to native (0.2.4); export `OrbitTooLarge` guard (0.2.22); canonical e21000
  diagnostic location. Export throughput measured **on par with Fraktaler-3** (the old "50×
  slower" figure refuted; the cold cost is `best_reference` scoring — later parallelized in
  0.2.40-beta).
- **Features** — exact **feature finder** (Newton-snap to parameterized Misiurewicz points and
  minibrot nuclei + curated POI list, 0.2.32); click-to-zoom tool (0.2.24); File → Open
  handles `.fdn`/`.kfr` (0.2.27); **Report an issue** (pre-filled email + Gmail compose, type
  picker, optional system info, 0.2.28–0.2.31); Re/Im axis naming (0.2.20); UI placement
  reorg by intent (0.2.33); `min_motion_res` floor slider (0.2.34); `zoom=inf` fixed in deep
  `.fdn`/bookmarks (0.2.35); help contents + corpus share files (0.2.19); the
  M(4,1) three-spar guided tour.
- **Zoom-reuse groundwork** — `--reusetest` measured reprojection staleness (perceptual sRGB
  metric; nearest beats bilinear — the filter isn't the lever), staging the future
  during-motion refine (design/xaos-reuse.md, 0.2.12–0.2.14).

## 0.2.0

Minor-version milestone rolling up **0.1.29 – 0.1.68**: the deep-zoom stability +
performance line, responsiveness/watchdog hardening, and the validation corpus. Validation
grew from **55/55 checks · 4/4 goldens** to **61/61 · 17/17**, and the 6-phase refactor
completed. Per-version detail is in the git history.

- **Deep-zoom performance.**
  - **Reference-orbit reuse** — a deeper rebuild now *extends* the cached bignum orbit
    (byte-identical) instead of recomputing it, cutting ~20× off dive-rebuilds (the orbit build
    was ~90% of a deep frame); truncated *and* escaped/complete references are reused so the view
    keeps one reference across rebuilds instead of re-picking and "jumping," and the last deep
    view's reference persists across sessions. (0.1.47, 0.1.57, 0.1.64, 0.1.65.)
  - **Adaptive deep-motion resolution (AIMD)** — moving-frame resolution follows the measured
    frame time: raise it while frames stay near vsync (the BLA is skipping), back off when they
    run long. (0.1.66.)
  - **aux⇄BLA coexistence** — orbit-trap (point/cross/circle), triangle-inequality, and stripe
    coloring ride the BLA via per-node aggregates folded O(1) on a skip: ~146–150× faster at
    depth, still exact. Series approximation is skipped where the BLA is active (it subsumes SA's
    early skip) — ~8× faster deep builds. (0.1.35, 0.1.37, 0.1.38, 0.1.49, 0.1.55, 0.1.56.)
  - **Reuse-first zoom for mode 0** — periodic-refresh scaled-frame reuse extended to the
    df32-perturbation path so a continuous zoom stays sharp. (0.1.53, 0.1.54.)
- **Deep-zoom correctness & smoothness.**
  - **Frozen-frame reprojection & hold** — the last good frame is scaled+panned to follow the
    zoom between real frames, with periodic refresh, fixing "goes blocky past ~1e28×," the
    >1e100× slide/jitter/jumping, and the reduced-resolution reproject "floating circle." (0.1.36,
    0.1.44, 0.1.58–0.1.63.)
  - **Deep-exterior tiling fix (SA-vs-BLA)** — at a deep exterior spot every candidate reference
    escapes early with an early-iteration perturbation glitch that series approximation masks; a
    BLA view had turned SA *off* ("BLA subsumes SA"), exposing it as distorted tiling. Fixed by
    keeping the BLA but forcing SA back on for short escaped references; plus a zoom-out BLA
    rebuild window and zoom-out reprojection scaling. `best_reference` also deep-ranks survivors
    to prefer a full-render-surviving reference. (0.1.65, 0.1.67, 0.1.68.)
- **Responsiveness & stability.**
  - **Async / progressive deep navigation** — the cold-start bignum reference builds off the
    render thread with a "working" spinner and a coarse-then-full progressive reference, so deep
    jumps stay responsive; fixed an async cold-start GPU device-loss crash. (0.1.39–0.1.43, 0.1.48.)
  - **GPU-watchdog (TDR) hardening** — a hard per-frame cost cap for deep floatexp frames plus
    adaptive wall-clock AA that lets measured-cheap frames extend the cap, preventing the deep
    floatexp freeze (with ss=1-floor hardening for large panels). (0.1.45, 0.1.46, 0.1.51, 0.1.52.)
  - **Resize / zoom-out fixes** — render coarse while resizing, aspect-fit (not stretch) the
    frozen frame on resize, fix the deep df32 zoom-out UI freeze, throttle the perf-overlay
    repaint, and close a menu when navigation starts. (0.1.31–0.1.33, 0.1.36, 0.1.50.)
- **UI / branding.** Dark + light themes, Spline Sans typography, and the wordmark. (0.1.30.)
- **Tooling / validation.** GPU timestamp queries split iterate-vs-color GPU time in the
  profiling harness (0.1.34); a **Kalles Fraktaler ↔ Fraktaler-3 cross-check corpus** under
  `validation/corpus/` (documenting that F3's extended-exponent kernels render blank past ~1e13×
  on this GPU); box-zoom (`zoom_to_rect`) test coverage.
- **Refactor complete.** The 6-phase refactor landed (core / GPU-export / app-module splits, an
  intra-crate `src/ui/` split, enum dispatch, structured `AppError`); the workspace slimmed
  **9 → 7 crates** as the empty `fractadyne-ui` / `fractadyne-fractals` stubs were retired
  (`fractadyne-render` remains the one reserved stub). Behavior-preserving throughout — goldens
  bit-identical at each step.

## 0.1.28

- **Refactor Phase 0 — guardrails & robustness** (behavior-preserving; see `REFACTOR-PLAN.md`). No
  change to rendered output — verified by the golden self-test (55/55 checks, 4/4 images identical).
  - **Lint gate:** added `[workspace.lints]` + per-crate `[lints] workspace = true` and a `rustfmt.toml`
    so clippy/format policy is shared and enforceable. `cargo clippy` is now warning-clean (0/0);
    applied all machine-applicable fixes.
  - **Panic hardening:** a missing wgpu backend now exits with a clear message instead of a Rust panic;
    the tour encoder thread pool recovers from mutex poisoning instead of cascade-crashing.
  - **Readability:** added a numeric-abbreviation glossary to the `fractadyne-core` module header and
    `SAFETY:` comments to the three Win32 FFI blocks.
  - Regenerated `TOURS.md` (its schema-drift guard test had been failing); rounded out workspace
    package metadata.

## 0.1.27

- **Export timing.** The Export dialog now shows a live **Elapsed:** readout while an export is in
  flight (including the off-thread reference build for a deep export), and the completion status
  line reports the **total time** — e.g. `Saved 3840×2160 → … (in 12.4s)`. The headless `--render`
  CLI prints the same total.

## 0.1.26

- **Fixed exported PNGs looking desaturated / washed out versus the live view.** The renderer is
  display-referred: `fs_color` writes palette colors straight into a **non-sRGB** framebuffer
  (egui-wgpu picks `Bgra8Unorm`/`Rgba8Unorm`), so the live view is already WYSIWYG. The PNG
  exporter was applying a second linear→sRGB transfer on top of those already-sRGB values, which
  lifted the shadows and drained saturation. PNG export now **quantizes the colors directly**, so
  the file matches the live framebuffer byte-for-byte. The OpenEXR master gets the inverse
  (sRGB→linear) so it remains a correct **linear** container that reproduces the same look. Golden
  images were re-blessed against the corrected output (all 55 numeric self-test checks unchanged).

## 0.1.25

- **Export target-directory picker.** The Export dialog now shows the target **Folder** with a
  **Choose…** button (a directory picker), and **Export** saves straight into it with an auto
  timestamped name — no save dialog to navigate each time. The folder persists across sessions. A
  **Save as…** button keeps the old flow (pick the file name + location).

## 0.1.24

- **Fixed the export stretching the fractal when the aspect ratio differs from the view window.**
  Choosing a fixed export aspect (0.1.20) overrode the image height but kept the window's complex
  span, so the GPU (which derives the per-texel step as span ÷ resolution per axis) mapped the
  window's vertical span across the new height — squashing/stretching the image. The vertical span is
  now set to keep the per-texel step isotropic (`span_y = span_x × height/width`), so a fixed aspect
  shows more/less vertical extent at the correct proportions, centered. No-op for "Match window".

## 0.1.23

- **Location HUD works in dual view** (was incorrectly greyed out). The HUD reflects the map /
  Mandelbrot view's zoom + center and lands on the image's top-left (the map panel of a side-by-side
  stitch).
- **Extended the deep-export freeze fix to dual view.** 0.1.22 moved the reference build off the UI
  thread only for single-view exports, so a deep *dual* export still froze. Now the (slow) map
  reference builds off-thread for dual too; the shallow Julia panel builds instantly when it lands.

## 0.1.22

- **Fixed the app freezing ("Not Responding") when exporting at extreme zoom.** A deep export built
  the bignum reference orbit (and ran glitch correction) *on the UI thread*, so at depths like
  1e1000×+ with hundreds of thousands of iterations the whole app locked up for minutes. Deep
  single-view exports now build the reference **off the main thread** (the dialog shows
  "Preparing — building reference…" and the UI stays live), then render on the background worker with
  progress + cancel — reusing the pre-built reference (no rebuild). Glitch correction is skipped at
  that depth (its multi-pass re-render would re-block the UI); a note and a **Glitch correction**
  checkbox were added to the Export dialog. Shallow / dual exports are unchanged.

## 0.1.21

- **Location HUD option in the Export dialog.** A new **"Location HUD"** checkbox burns the
  zoom-level + coordinate panel into the top-left of the exported image (single view; scales with the
  output). Previously the HUD was only reachable via the `--show-location` CLI flag. The setting
  persists. Implemented by pre-rasterizing the HUD into a premultiplied overlay on the main thread
  (the background export worker has no egui font context) — the live view / tour frames share the
  same rasterizer, so the look is identical.

## 0.1.20

- **Fixed the Export dialog showing "W × 1 px" at deep zoom**, and added an **aspect-ratio
  selector.** The dialog derived the preview height from `complex_span`, which saturates to 0 past
  ~1e308× → a bogus 1-px height (the *render* was always correct — it uses the pixel aspect). The
  height display now uses the same pixel-aspect math as the render. New **Aspect** dropdown: *Match
  window* (default) or a fixed ratio (16:9, 16:10, 3:2, 4:3, 1:1, 2:3, 9:16, 2:1); a fixed ratio
  renders that many rows centered on the same center. Persisted.

## 0.1.19

- **"Live render budget" slider (detail vs. speed at deep zoom).** The work budget that decides when
  the live view drops to a box-filtered reduced-resolution upscale (the "soft" look at extreme depth
  on large windows) is now a tunable multiplier (side panel → Navigation, 0.25×–8×, persisted).
  Higher renders the live deep-zoom view at fuller resolution (crisper) at the cost of frame-rate and
  GPU-watchdog margin; exports are always full resolution regardless.

## 0.1.18

- **Esc reliably stops auto-zoom.** Escape is now one of the auto-zoom loop's own interrupt inputs
  (alongside click / scroll / Space), so it stops the dive directly — including deep in a stepped
  dive — rather than relying only on the top-level key handler.
- **`--selftest` golden images are now deterministic and robustly located.** The golden-regression
  renders pin *every* render-affecting field explicitly — in particular the palette animation (a
  saved "Random gradients" mode would otherwise make `active_stops()` return a random palette and
  fail the goldens on a loaded session). The goldens are also always read from the canonical
  `validation/golden/` regardless of `--out` (previously `--selftest --out <elsewhere>` looked for a
  `golden/` next to the report and silently reported a fake `maxΔ 255` failure), and a missing /
  wrong-size / errored golden now reports a distinct reason instead of masquerading as a pixel diff.
  Goldens remain bit-identical (4/4, maxΔ 0).

## 0.1.17

- **Auto-zoom toolbar button.** A 🛸 button in the toolbar's navigation group toggles auto-zoom and
  stays **highlighted while it's running** (click it to stop) — so you can see at a glance that the
  hands-free dive is active without watching the View menu. Disabled in dual view (single-view only).
- **Deep-zoom sample location + updated sample render.** Added `scripts/deep-sample.fdn`, a
  Mandelbrot location at **~1e1108×** with a ~1138-digit center (loadable via File ▸ Open view /
  Share location). `scripts/render-deepest.ps1` now renders *this* location instead of the tour
  endpoint — reading the center + scale from the `.fdn` and converting `upp_log2` to the renderer's
  `--zoom-log2` (`L = log2(3/height) − upp_log2`). Verified it reproduces (real structure, not a
  blank) at the location's 500k-iteration budget.

## 0.1.16

- **Fixed the deep auto-zoom showing a blank colored square.** In the stepped deep dive, every
  frame was flagged as "moving," which triggered the smooth-motion freeze that never renders a real
  floatexp frame — so the held frame kept scaling down until only the uniform fill was left. The
  stepped dive now renders a **real full frame between jumps and holds it on screen while the next
  one computes** (it waits for a depth-matched reference before each jump, so no blanks and no
  spin). You watch a slideshow of real deep frames instead of a colored square.

## 0.1.15

- **Auto-zoom: adjustable dive limit + a stepped deep-dive mode.** The hard ~1e271× cap is gone.
  A new **"Auto-zoom dive limit"** slider (side panel → Navigation, persisted) sets where the
  hands-free dive (A key) stops, from 1e30× to 1e5000×. Up to ~1e271× it glides smoothly as
  before; past that — where each frame is too slow to animate — it switches to a **stepped dive**:
  pick the detail-richest target, jump the zoom ×4, render, repeat. Choppy, but it reaches extreme
  depth quickly. Target re-evaluation is now **adaptive**, spacing out as frames slow with depth
  (≈ once per rendered frame when deep) instead of a fixed 0.35 s, so it doesn't pile up work it
  can't keep up with.

## 0.1.14

- **`scripts/render-deepest.ps1`** — reproduces the deepest zoom the project documents: the
  Misiurewicz spiral at the endpoint of `tours/deep-spiral-dive.toml`, ~**1e420×** with a
  ~464-digit center. It reads the coordinate + depth straight from the tour (so it stays in sync),
  renders one still with an iteration budget matched to the depth (~220/octave), and reports timing
  — a genuine stress test of the deep-zoom pipeline. Verified end-to-end.
- **Softened "unlimited" zoom claims to concrete, verifiable ones.** The depth is bounded by
  coordinate precision and the iteration budget, not a fixed wall — so the docs, in-app **Help**,
  and code now say **"extreme deep zoom"** with real numbers (cross-checked against Fraktaler-3 past
  1e300×; the bundled tour reaches ~1e420×) instead of "unlimited." Technical accuracy over
  marketing.

## 0.1.13

- **Restartable tour renders** — a long `--render-tour` that gets interrupted can now be resumed.
  The new **`--resume`** flag keeps the frames already on disk and renders only the missing ones
  (no per-frame prompt), so you pick up where it stopped instead of starting over.
- **`scripts/render-spiral-dive.ps1`** uses this: if the target folder already has frames it checks
  the last frame's resolution and a saved `render.manifest.txt` against the current settings, warns
  about any mismatch (resolution / fps / supersampling / HUD), and offers to **Resume** or start
  **Over**. On resume it re-renders the final frame in case it was cut off mid-write. The script
  also now burns the location **HUD** into frames by default (disable with `-Hud:$false`).

## 0.1.12

- **Reset application state** — a way to wipe all saved data (session, bookmarks, thumbnails)
  and start fresh, from both the UI and the command line:
  - **File ▸ Reset application state…** opens a confirmation dialog that spells out exactly what
    will be deleted and where, and doesn't touch anything until you confirm. After a reset the
    current session isn't re-saved on exit, so defaults load next launch.
  - **`fractadyne --reset-state`** does the same headlessly, printing a warning and requiring you
    to type `reset` to confirm (`--yes`/`-y` skips the prompt for scripting).
- **Versioned session file + newer-version warning** — the saved session now carries a
  `state_version`. If a session was written by a newer Fractadyne than the running build can fully
  account for, it's loaded best-effort and the app warns (a toast) rather than silently
  misinterpreting it. Legacy files (no version) still load as before.
- **`FRACTADYNE_CONFIG_DIR`** environment variable overrides where state is stored — useful for
  sandboxing/portable installs, and so the destructive reset can be exercised against a throwaway
  directory. The **Help ▸ About** section now shows where your state lives and how to reset it.

## 0.1.11

- **Third-party license notices** — the bundled dependencies' licenses are now reproduced in
  `THIRD-PARTY-NOTICES.md` (generated with `cargo-about`; shipped with the download) and are
  viewable in-app under **Help → Licenses** (with a "Copy all notices" button). This satisfies the
  MIT/BSD/Apache/Zlib/Unicode/font notice requirements for the statically-linked binary, and calls
  out the one MPL-2.0 dependency (`option-ext`) and its source. The existing **Help → Acknowledgments**
  already credits the deep-zoom algorithms and libraries.

## 0.1.10

- **Fixed a deep-zoom "Not Responding" hang** — a fast dive (a tour, or holding zoom) crossing into
  the floatexp range (past ~1e28×) could freeze the app for seconds per frame. Root cause: the
  floatexp iterate shader spins when its reference/BLA fall behind a fast dive, and since GPU pixels
  run in parallel the slowest pixel stalls the whole frame — blocking the UI thread. Deep floatexp
  *motion* now reprojects the last good frame (smooth + responsive) and snaps to full detail when a
  fresh reference lands or the view settles; the offline `--render-tour` export is unaffected (full
  detail per frame). *(Live preview of a very deep dive is still soft while moving — an inherent cost
  of high-precision rendering; see `design/multiref-live.md`.)*
- **Faster floatexp normalization** — the extended-range `fe_norm`/`sf_norm` now use `frexp`/`ldexp`
  (ALU bit-ops) instead of `log2`/`exp2` (GPU special-function unit). Bit-identical output; a small,
  never-worse win in the deep-zoom loop.
- **Dev:** `--refdiag` samples reference-orbit lengths across a view (diagnoses deep-zoom cost).

## 0.1.9

- **Ultra-deep-zoom benchmark** — the standardized benchmark gains a **Depth** control: *Standard*
  (1 → 1e12×) or **Ultra deep** (1 → 1e28×, past f64's range, exercising the floatexp/BLA deep-zoom
  path far harder). CLI: `--depth standard|ultra` (or `--ultra`). The report records the depth.
- **Benchmark quality-of-life** — reports now include the **run date** (UTC); the standardized run
  shows a **spinner + per-pass progress bar** and advances one dive-frame per event-loop tick, so the
  window stays responsive and cancellable throughout (it no longer blocked on a whole pass); and the
  results window has a **"Run again…"** button that reopens the configuration dialog.
- **Guided-tour callouts no longer overlap** — landmark callout labels (and the title/caption text)
  now use collision-avoidance placement, so a tour's intro annotations stay legible instead of
  stacking on top of each other. Applies to live playback and exported tour frames.

## 0.1.7

- **Standardized benchmark + burn-in** — the benchmark now has two modes. **Current settings** plays
  the fixed deep-zoom tour into the live view exactly as before (measures your window/resolution and
  active settings). **Standardized** pins *every* render setting — Mandelbrot / smooth / Ember, 2× SS,
  depth-adaptive iterations, series-approx + BLA on, glitch off — and renders a fixed 60-frame dive to
  1e12× **offscreen** at a chosen resolution (**720p / 1080p / 4K / 5K×2K**), so 4K/5K work regardless
  of monitor size and the score means the same on every machine. The report now records the full
  settings block (resolution, SS, iteration policy, deep-zoom flags, dive) alongside the CPU/GPU/RAM
  facts. **Burn-in** repeats the standardized run N times and reports per-pass FPS, a stability
  std-dev, and a first-vs-last **throttle** delta — revealing thermal/clock decline under sustained
  load. In the GUI: *Tools → Benchmark…* (mode / resolution / burn-in dialog; runs a pass at a time so
  the window stays responsive and cancellable, then restores your live view untouched). Headless:
  `--benchmark-std [--res 720p|1080p|4k|5k] [--burnin N]` (`--out` saves the report).

## 0.1.6

- **Progressive settle anti-aliasing** — when the view settles it no longer jumps straight to full AA
  in one (potentially long) frame. Instead the AA ramps **1×→2×→4×→… up to your chosen level over
  consecutive frames**, each scheduling the next. So a heavy view (deep / high-iteration / dual /
  high AA) shows an instant coarse frame the moment you stop and refines to full quality, keeping
  interaction fluid instead of freezing on every stop. Per view (`settle_frame[2]`), reset while
  moving. *(The final full-AA frame on a very heavy view is still one expensive frame — its inherent
  cost — but you see near-final quality several fast frames earlier, and can move on before it.)*

## 0.1.5

- **Dual view: per-view settle** — driving the Julia `c` by moving the cursor over the Mandelbrot
  panel no longer forces the (unchanged) Mandelbrot panel to re-render at the coarse "moving"
  resolution. The interaction/settle timer is now tracked per view (`settle_t[2]`), so only the panel
  that actually changed drops quality while moving; the other stays sharp and its cached iteration
  texture is reused (just re-colored). This also lifts interaction FPS in dual view, since only one
  panel re-iterates during cursor movement. *(The settled frame rate on a heavy view — dual +
  Multibrot³ + 8× AA + animated relief lighting — is dominated by the anti-aliased color pass; lower
  the AA or turn off light rotation for a higher idle FPS.)*

## 0.1.4

- **Phoenix 3D relief lighting + distance glow** — these need the orbit derivative `dz/dc`, which the
  shader only tracked for the analytic families (formula ≤ 3), so Phoenix rendered flat. Phoenix is
  analytic, so it *can* have them: added its two-term derivative recurrence `D' = 2·z·D + [1] −
  0.5·D_{n-1}` (with a previous-derivative register) in all three render paths — direct, df32 (mode 0),
  and floatexp (mode 2) — and enabled the normal/DE output for it. Relief lighting and distance glow
  now work on Phoenix at any depth. (The abs families stay unlit — they're non-differentiable at the
  abs folds; only the analytic families + Phoenix qualify.) Verified visually + selftest 55/55.

## 0.1.3

- **Dual-view glitch correction** — glitch correction now applies to **dual exports** too (both the
  side-by-side and separate-files layouts, and the CLI `--render` + in-app export paths), correcting
  each panel through the same validated multi-reference loop. Refactored the export paths onto a
  shared `render_export_view` / `export_corrected_sync` helper. Verified: a dual Mandelbrot panel is
  byte-identical to the single-view corrected render; side-by-side stitches correctly; selftest
  55/55, goldens 4/4. Completes the "full glitch correction" export coverage (live-view correction
  remains, deferred as it would touch the fragile live pipeline).

## 0.1.2

- **Phoenix deep zoom** — the Phoenix family (`z' = z² + c − 0.5·z_{n-1}`) now perturbation-deep-zooms
  like the other families (previously direct-only, ~1e6×). Its two-term recurrence needed care: the
  GPU carries an extra `δz_{n-1}` register and, on rebasing, the previous term is rebased to the full
  value (rebase-to-0 works because the reference's `z_{-1} = 0`). Implemented in both the df32 (mode 0)
  and floatexp (mode 2) paths + the bignum reference. **Rigorously validated:** two new self-test
  checks show mode-0 perturbation matches the direct path to **mean Δ 0.007 iter** (0 pixels off by
  >2) and floatexp matches df32 exactly, on the smooth region at 1e5×. (Newton stays direct-only —
  it's convergence-based, with a nonlinear, coloring-incompatible perturbation.)

## 0.1.1

- **Glitch correction on by default** — multi-reference (Pauldelbrot) glitch correction, previously
  an opt-in export toggle, is now **on by default**, so shared images are glitch-free out of the
  box. The GPU flags glitched pixels (|z|² < 1e-4·|Z|²) and the CPU drops fresh references into the
  worst regions and re-renders until clean (up to 64 refs). Bounded to ~32 MP / the GPU texture
  limit and non-aux coloring; larger images and the live view fall back to the plain path (a VRAM
  cap was added so the default can't OOM big exports). New `--glitch` / `--no-glitch` CLI overrides.
  Verified: fixes real glitches at a deep seahorse spot (63 px changed vs uncorrected at 1e13×);
  selftest glitch checks pass (7 refs, 0 residual); goldens 4/4. *(Still to come: dual-view export
  correction and live-view correction.)*

## 0.1.0

Baseline for tracked versioning. Notable capabilities already present:

- **Deep zoom** — arbitrary-precision center, df64 reference orbit, df32 GPU
  perturbation with Zhuoran rebasing, hybrid direct/perturbation crossover; clean to
  ~10²²×. Generalized to Multibrot 3/4/5 and Tricorn in both Mandelbrot and Julia modes.
- **Fractal variety** — 10 escape-time families with per-fractal info panels.
- **Dual linked view** (Mandelbrot ↔ Julia) with per-view reference caches, Julia pin.
- **High-res export** — tiled PNG / OpenEXR with reloadable metadata, gallery browser,
  background rendering, progress + cancel.
- **UI** — combined menu/toolbar with icons, docked performance panel, animated
  zoom-home, fullscreen (Esc to exit), interactive orbit overlay (tapered gradient,
  racing-dot animation, normalized full-view mode), palette cycling animation.

### Added (post-baseline, this session)

- **Guided-tour captions + version tracking** — camera-tour scripts (`Tools → Play script…`,
  `--render-tour`) can now narrate: `[[caption]]` entries with `text` (multi-line), `at`/`secs`
  timing (independent of keyframes), `pos` (top/center/bottom), `fade`, and `size`. Captions ease
  in/out, wrap, and sit centred on a soft dark backing; they render live over the fractal **and**
  burn into exported tour frames. Scripts also declare a `format_version`; loading a newer script
  warns (schema is additive, so old scripts keep playing).

- **Guided-tour callouts** — `[[callout]]` entries point a labeled amber marker (ring + leader
  line) at a fractal coordinate (`center_x`/`center_y`), **anchored in fractal space** so it tracks
  the spot as the view pans/zooms (new exact-at-any-depth `Viewport::complex_to_pixel`). Timed like
  captions (`at`/`secs`/`fade`), rendered live and burned into exported tour frames; off-screen
  anchors are skipped.

- **Guided-tour spotlights** — `[[spotlight]]` entries dim everything outside a soft circle centred
  on a fractal coordinate to draw the eye, with `radius`/`softness`/`dim` and `at`/`secs`/`fade`.
  Applied in the color shader (aspect-corrected, so the circle is round; live and export identical)
  and anchored via `complex_to_pixel` so it tracks its point; the dimming eases with the window.

- **Guided-tour easing + holds** — keyframes take a per-segment `ease` (`smooth` default, `linear`,
  `smoother`, `in`, `out`) for the glide arriving at them, plus `hold` seconds to pause at a
  keyframe before the next glide. `Playback::sample` splits each segment into a hold + an eased move
  phase. This completes the guided-tour feature (captions, callouts, spotlights, easing/holds, and
  version tracking — all rendered live and into `--render-tour` movie frames).

- **Deep-zoom tours** — tour keyframes can now express zooms past f64's ~1e308 ceiling via
  `mag_log10` (e.g. `mag_log10 = 420` for 1e420×), and a script `palette` field sets a preset so a
  tour colours consistently regardless of the session palette (a binary palette rendered deep
  exterior-only views as one flat color). `--render-tour` also forces zoom-appropriate iteration
  counts so deep frames resolve. Ships example tours in `tours/`: a dive to a real minibrot
  (~1e30×) and a dive to an endless spiral (~1e420×).

- **Tour dual view / Julia sets / orbits** — keyframes gained `dual` (linked Mandelbrot + Julia
  side-by-side), `julia_re`/`julia_im` (pin the Julia parameter c), and `orbits` + `orbit_re`/
  `orbit_im` (overlay the escape-time orbit at a point). Applied live (with the c-marker on the
  Mandelbrot) and rendered into `--render-tour` frames (dual stitched side-by-side; orbit path
  rasterized). New example tour `tours/julia-and-mandelbrot.toml` teaches the Julia↔Mandelbrot
  relationship (connected vs. dust) and what the orbit means. The Julia parameter `c` (and the
  orbit point) interpolate smoothly between keyframes, so the Julia set *morphs* continuously as
  `c` glides along its path rather than jumping.

- **`--size` accepts `WIDTHxHEIGHT`** — previously `--size 5120x2160` silently fell back to the
  1280×720 default (only a bare width parsed); now both `--size 1920` and `--size 5120x2160` work,
  for `--render` and `--render-tour`. Explicit `--height` still overrides. Each dimension is clamped
  to 16–16384 px.

- **`fractadyne --help` / `-h`** — prints the full command-line reference to the terminal and exits.
  Both it and the in-app **Help → Command line** window now render from one shared `CLI_REFERENCE`
  table, so they can't drift out of sync. Also accepts Windows-style `/?`, `/h`, `/help` (and `-?`).
  An **unrecognized option** (e.g. a typo'd `--rendr`) now prints the reference to stderr and exits
  non-zero instead of silently launching the GUI — the known-flag set is derived from the same
  table, and only `--long` tokens are checked so negative-number values (`--center -0.5 …`) are
  never misread as flags. Docs (README, STATE) refreshed to match.

- **Location HUD on renders** — `--show-location` (alias `--hud`) burns a small overlay into the
  top-left of each rendered frame: zoom level (amber) + full-precision center coordinates (`re`/`im`).
  Works with `--render` and `--render-tour`; a tour can also set `show_location = true`. Uses the
  same deep-precision coordinate formatting as the live status bar.

- **Tour-script reference ([TOURS.md](TOURS.md))** — a complete field reference for the tour `.toml`
  schema (every table + field, types, defaults, a worked example), **auto-generated** from a schema
  table colocated with the serde structs. `fractadyne --dump-tour-schema` prints it (regenerate with
  `--dump-tour-schema > TOURS.md`); a test fails if the checked-in file drifts, so it can't rot.
  Refreshed `scripts/tour.example.toml`'s header and linked the reference from the README.

- **Response files (`@FILE` / `--args-file FILE`)** — read command-line arguments from a text file,
  spliced in place. Whitespace-separated tokens, `#` comments, and `"quotes"` for values with spaces;
  nestable (bounded). Keep a whole render/tour invocation in a file and run `fractadyne @render.args`.

- **Tour frame naming + overwrite guard** — `--render-tour` frames are now named
  `<prefix>_00000.png`, where `--prefix NAME` overrides the default (the tour script's file name;
  `frame` if none). The mp4 default follows suit (`<prefix>.mp4`). Before clobbering an existing
  frame it prompts on the terminal — **[y]es / [a]ll / [n]o / [q]uit** — with `--overwrite` (`-y`)
  to skip the prompt. When stdin isn't a terminal (automation) it errors with that hint instead of
  hanging.

- **Pipelined tour references** — `--render-tour` now computes frame N+1's arbitrary-precision
  reference (orbit + series approximation + BLA) on a worker thread while frame N renders on the
  GPU, so the deep-zoom bignum stall overlaps the render. The export reference path was unified onto
  the same `recompute_worker` the live view uses (one implementation, no divergence). Byte-identical
  output (verified); ~1.2× on a shallow mode-0 dive, more on deep large frames where the reference
  is a larger share of per-frame cost. Gated to single-view frames; falls back to synchronous
  (always correct) for dual/Julia-changing frames.

- **Pipelined tour encoding** — `--render-tour` now compresses PNGs on a small background thread
  pool while the next frame renders, instead of blocking on each `write_png`. Frame output is
  byte-identical (verified); the win grows with resolution (PNG deflate of a 4K/5K frame is tens to
  hundreds of ms that now overlap the render). A ~1 GB in-flight budget bounds memory (backpressure).

- **Tour → mp4 in one step** — `--render-tour … --mp4 [PATH]` assembles the rendered PNG sequence
  into an H.264 mp4 via ffmpeg (kept alongside the frames; PATH defaults to `<out-dir>/tour.mp4`).
  Rendering now prints live progress — frames done, elapsed time, ETA, and frames/sec — and reports
  total render + encode time when finished. Without ffmpeg on PATH the frames are still written and
  the exact assemble command is printed.

- **End-of-script message** — when a tour finishes playing, a toast reports "Script finished — …".

- **No freeze at the df32→floatexp crossover** — a fast live dive crossing ~1e28× could hang the
  app: the first floatexp frames run the full iteration count (BLA is still building off-thread),
  and at native resolution that single frame could trip the GPU watchdog. The interacting work
  budget is now shrunk for the costlier floatexp path, so resolution drops during motion instead
  of the frame stalling; settle frames (reference + BLA landed) keep full quality.

- **Zoom-reprojection (smooth deep dives)** — during the brief reference rebuild at extreme depth
  the held frame now scales + pans to follow the ongoing zoom instead of freezing, so the view keeps
  moving smoothly until the fresh reference snaps in. Generalizes the pan reprojection with a scale
  factor (a shader `uv_scale`); pure pan is unchanged.

- **Discreet "Fd" watermark** — a small brand mark in the lower-right of the live view and exported
  images, using the header wordmark's font: **F** in the light brand text color, **d** in the amber
  accent (matching "Fractadyne"). Sized ~2.6% of the frame (≈20–30 px on screen) with a soft dark
  halo so it stays legible on any background. Drawn with the egui painter live, and rasterized from
  the same font atlas + alpha-blended into exports (built once on the main thread; the export worker
  has no egui context). On by default; toggle in the control panel ("Fd watermark") or via
  `--no-watermark` / `--watermark`. Persisted; excluded from the self-test goldens (math only) and
  from the raw `--render-iter` data EXR.

- **Off-thread reference recompute (no more deep-zoom stalls)** — the slow arbitrary-precision
  recompute (reference orbit + series approximation + BLA tree) now runs on a worker thread; the
  live view keeps drawing with the cached reference and swaps in the fresh one when it's ready
  (only the first, cold-start reference is synchronous). Deep-zoom settle/motion no longer hitches:
  measured via the new `--frametest` harness on a 1e30× dive, per-frame recompute stalls dropped
  **27 → 1** and build-time p95 **91.8 → 0.1 ms**. New `scripts/frametest.ps1` automates the
  before/after comparison.

- **Faster deep-zoom recompute** — two exact (bit-identical) bignum optimizations on the deep-zoom
  settle/motion recompute: the reference orbit formed `2xy` with a full multiply-by-two → exact
  base-2 exponent shift (**−13–17%** reference compute at 1e6–1e20×); and the series-approximation
  coefficient loop multiplied by small-integer factors via shift-and-add and skips identity `Z^0`
  multiplies (**−7%** series setup at 1e30×).

- **Draggable dual-view splitter** — the Mandelbrot↔Julia divider is now draggable (grab the
  separator between panels; clamped 15–85%) and the position persists (`dual_split`), instead of a
  fixed 50/50 split.

- **BLA acceleration on by default** — deep floatexp Mandelbrot renders (≥1e28×) now use BLA out
  of the box: **~5× faster GPU render** (70→13 ms at 1e30×) with identical output. The tree is
  cached per reference (one-time build like the reference orbit), the cache is hardened against
  zoom-out, and it's validated at interior + boundary. Toggle it off in the View menu (or `--no-bla`
  headless) to compare or if an artifact ever appears.

- **BLA per-reference caching** — the acceleration tree is now cached per reference (rebuilt only
  when the reference orbit changes) instead of every frame, using a conservative view-diagonal
  `dc_max` that stays valid across pans. A settled deep view drops from ~35 ms/frame (build + render)
  to ~13.6 ms (render only) — the full ~5.4× — and the one-time tree build is now amortized like the
  reference orbit, removing the weak-CPU concern. Still opt-in (View menu) pending on-target
  verification before it's enabled by default.

- **BLA profiling tooling + measured verdict** — a `--bla` CLI flag forces BLA on for headless
  runs, the profiler now times the BLA tree build (`bla_build`) and labels runs with `use_bla`, and
  `scripts/profile-bla.ps1` runs BLA off-vs-on and breaks down the tradeoff (export / live /
  cached). Measured on an RTX 3080 / Ryzen 3950X: at 1e30× BLA cuts the GPU render **5.8×** (73→13
  ms) for a **~20 ms** tree build — **2.2× net even rebuilt every frame, 5.8× with caching**, and
  no cost where it doesn't apply. Verdict: enabling BLA by default is justified (per-reference
  caching is the remaining step).

- **BLA acceleration: user toggle + escape-path validation** — bilinear approximation (skip
  iterations throughout the orbit at extreme depth) is now a persisted **View-menu toggle**
  ("BLA acceleration (deep zoom)") instead of a hidden dev flag. Its GPU escape-overshoot revert
  is now validated: a new self-test renders a deep *boundary* view (48400 escaping pixels, 0
  mismatch vs BLA off) to complement the existing all-interior test — both code paths covered.
  Still off by default while the per-frame cost/benefit is measured (the acceleration tree is
  rebuilt each frame; per-reference caching is the next step before enabling by default).

- **Multi-reference glitch correction — now shipping for exports (phase 2c)** — a new
  **"Glitch correction (export)"** preference (View menu, persisted) makes single-view exports
  glitch-free: perturbation glitches are detected and those pixels re-rendered against extra
  references until clean. `color_iter_buffer` colors the merged buffer; `render_export_corrected`
  wires it into both the headless (`--render`) and interactive export paths. Applies to single-view
  exports up to the GPU texture limit with non-aux coloring; the live view is unaffected.
  (Follow-ups: tiling for larger exports, aux coloring methods, dual layouts.)

- **Multi-reference glitch correction — phase 2b (correction orchestration)** — `render_corrected_iter`
  renders the iteration buffer with detection on, then repeatedly places a fresh reference (bignum)
  at the largest glitched region and re-renders, adopting the newly-resolved pixels until nothing is
  glitched. Seeding at the exact pixel center guarantees convergence. A selftest resolves a
  seahorse-1e8× view's flagged glitches to **0 residual** with a handful of references. Next: color
  the corrected buffer and wire it into exports behind a preference.

- **Multi-reference glitch correction — phase 2a (GPU detection)** — the shader now detects
  Pauldelbrot glitches (`|z|² < tol·|Z|²`) in both perturbation paths (df32 + floatexp), flagging
  glitched pixels with a `-2` sentinel in the iteration texture (harmless when uncorrected — the
  color pass treats it as interior). Gated by a `glitch_on` uniform (off for live/normal render),
  plumbed via `ExportRequest`. Validated by a selftest that confirms detection fires and responds
  to reference quality. Next: the app-side multi-pass correction that consumes it.

- **Multi-reference glitch correction — phase 1 (core algorithm)** — the last real deep-zoom
  correctness gap. `fractadyne-core` gains the validated CPU algorithm: single-reference
  perturbation with Zhuoran rebasing **and Pauldelbrot glitch detection** (`perturb_pixel_mandel`,
  δz in f32 to mirror the GPU's high-precision-reference / low-precision-δz gap), plus
  `render_multiref_mandel`, which detects glitched pixels, places a new reference inside each
  glitched region, re-renders and merges, and repeats to convergence. Validated against a bignum
  per-pixel oracle at a real period-3 minibrot (induces glitches, converges with multiple
  references, matches ground truth). Follows the BLA playbook (correct core first); the GPU/export
  port is the next phase.

- **Pan reprojection (retain detail while dragging)** — dragging to pan no longer drops to the
  coarse moving preview (which shows no detail at deep zoom, so you couldn't see where you were
  going). Instead the last detailed frame is frozen and translated with the cursor in the color
  pass — no bignum recompute, no re-iterate — so the real image slides under the pointer. Only
  the newly-exposed edge is blank until you stop, at which point the view settles and re-renders
  at full detail. Applies to single and dual (left panel) views at deep zoom; the shallow direct
  path is already detailed so it renders normally.

- **Progressive iteration refinement (sharpen on settle)** — deep views no longer look
  permanently smooth. The Iterations slider now goes to 500,000 (was 50,000) and auto-scale's
  appetite climbs past 50k with depth. While you're moving, the preview caps iterations at
  50,000 with a tight work budget so motion stays responsive; the moment the view settles it
  re-renders at the full zoom-appropriate count (up to ~200k+ deep) with a ~6× larger budget —
  still well under the GPU watchdog — so the finest boundary filaments resolve on screen,
  matching an export. The live reference orbit is built only to the count actually rendered, so
  navigation speed is unchanged. A note appears only in the rare case a settled view is still
  resolution-limited (huge window at extreme depth), pointing to export for full resolution.

- **Recommended-hardware Help section** — GPU/CPU/memory guidance (what matters and why: the GPU
  drives per-pixel iteration + frame rate; the CPU's single-core speed drives the deep-zoom
  reference orbit) with minimum/recommended tiers.

- **Acknowledgments & citations (Help)** — a new Help section crediting the prior art Fractadyne
  builds on, each verified against its source: perturbation & series approximation (K. I.
  Martin), BLA + rebasing (Zhuoran), glitch detection (Pauldelbrot), non-analytic/Burning-Ship
  perturbation (laser blaster), reference implementations & cross-checks (Fraktaler-3 / Kalles
  Fraktaler 2+ by Claude Heiland-Allen, orig. Karl Runmo), smooth + stripe coloring (Jussi
  Härkönen), triangle-inequality average (Kerry Mitchell), the Mandelbrot set (B. Mandelbrot),
  and the libraries used. Includes a **dedication to the Stone Soup Group of Fractint**.

- **Bookmark thumbnails** — each saved bookmark now shows a small preview image in the
  Bookmarks (Manage) dialog. The thumbnail is rendered from the exact view at save time
  (small offscreen render) and stored as a PNG under `bookmark_thumbs/`; it's lazily loaded
  for display and cleaned up when the bookmark is deleted. The dialog now lists each bookmark
  as thumbnail + name + zoom + Go/Delete.

- **Minimap shown in dual view** — the "you are here" overview was hidden in dual view; it's
  now shown (it maps the left/Mandelbrot panel). Only a single Julia view still hides it,
  where a Mandelbrot overview wouldn't correspond to the shown set.

- **Zoom box (Shift+drag)** — hold Shift and drag a rectangle to zoom so it fills the view.
  The box is constrained to the panel's aspect ratio (fills exactly, no distortion), drawn as
  a live amber rubber-band overlay, and applied deep-zoom-correctly (recenter + scale via the
  arbitrary-precision viewport, so it's exact at any depth). Works in single and dual views; a
  tiny drag is ignored (treated as a click). (Replaces a "Right-drag box zoom" that Help
  documented but was never implemented.)

- **Duotone & binary palette modes** — Coloring → Palette gains two two-color modes sharing
  a pair of color pickers: **Duotone** maps the coloring value to a smooth Shadow→Highlight
  ramp; **Binary (set)** is a flat two-color view (one solid color inside the set, another
  outside, no gradient) — the clearest way to see the set's shape. The in-set (interior)
  color is now selectable (a new shader uniform; defaults to the previous near-black, so
  existing renders are unchanged). Persisted.

- **Validation & self-test suite** — a layered correctness harness with no external data
  (exact mathematics + internal cross-checks), designed to be independently verifiable.
  `cargo test -p fractadyne-core` adds exact-ground-truth tests: hyperbolic-component
  nuclei & periods (recovered to 1e-9), Misiurewicz pre-periodicity, closed-form interior
  membership, and dwell symmetry. `fractadyne --selftest` runs a GPU validation suite
  (exit code 0/1) checking the perturbation path against an independent **CPU f64 dwell**
  and a **naive arbitrary-precision (bignum) dwell oracle** (no perturbation, no reference)
  comparing the **integer escape count** across a **depth battery (1e6× … 1e30×)** that
  exercises whichever render mode the depth selector actually uses (df32 perturbation and
  floatexp), excluding only ill-conditioned boundary samples — independent deep-zoom
  correctness, not just internal consistency. Plus a **reference-independence** check
  (renders one view with three distinct in-view references and a reference-override path;
  the auto reference must agree with the per-pixel majority across the smooth region — an
  oracle-free glitch detector that also seeds multi-reference correction), floatexp-vs-df32
  agreement, real-axis symmetry, interior/exterior presence, and finiteness (via a new
  `render_iter` that reads back the raw iteration texture). **Family symmetries** are
  verified exactly in `fractadyne-core` (Multibrot (d−1)-fold rotation, Tricorn 3-fold,
  Julia z→−z, Celtic real-axis; confirmed Burning Ship / Buffalo have *no* axis symmetry)
  and the **render pipeline** is checked for the non-Mandelbrot family shaders in
  `--selftest` (Multibrot-3 180°, Tricorn / Celtic real-axis). The exact-landmark catalog
  is extended (cardioid cusp c=¼, period-1↔2 neck c=−¾, period-2 disk, cardioid boundary
  parametrization). **Invariance/consistency** checks target the tier crossovers:
  resolution independence (N vs 3N — validates δc construction), max-iter monotonic
  stability, zoom-sequence consistency across the direct→df32 seam, pan consistency, and
  render determinism. **Derivative checks** validate the `dz/dc`-derived distance estimate
  independently of dwell: DE self-consistency (a boundary-adjacent pixel can't claim a far
  boundary) and the Koebe-¼ lower bound (a disk of radius DE/4 is boundary-free, verified
  against an independent CPU dwell). **External checkability:** a committed, human-readable
  **location catalog** (`validation/catalog.toml`) of full-precision coordinates with
  independently-known answers (period + nucleus, set membership) that `--selftest` verifies
  — doubling as published *challenge coordinates*; and a **Coverage & scope** section in the
  report stating exactly what each oracle checks and, importantly, where the deep regime is
  *not* independently oracle-checked. **Fuzz tests** (dependency-free, deterministic) hammer
  the untrusted-input parsers — the arbitrary-precision coordinate parser and the
  view-metadata parser chain — asserting they never panic on random/adversarial/oversized
  input. This hardened `parse_bf` to also reject non-finite (±∞/NaN) coordinates.
  **Comparison tooling:** `--render-iter` exports the raw iteration texture as an EXR
  (R=smooth iteration, G/B=slope normal, A=log₂ DE-in-pixels) with a documented layout, so
  a reviewer can diff iteration data directly (no coloring confound); and `--compare A B`
  reports max/mean per-pixel difference (channel-0 iteration data + finite all-channels) and
  writes a difference heatmap — for A/B against another build or imported renderer data.
  **Cross-renderer import:** Locations → "Import .kfr…" (and `--import-kfr FILE`) loads a
  Kalles Fraktaler location via a **hardened, fuzzed** parser (size/length-bounded, strict
  key allow-list, every field validated/clamped, no paths/code) — so the identical
  coordinate can be opened in a trusted third-party renderer for the strongest external
  cross-check. Verified bit-identical to a direct render of the same coordinates. **Golden-image regression**: `--selftest --bless` records
  reference PNGs under `validation/golden/`; subsequent runs diff against them with a pixel
  tolerance. Every run writes a **readable, verifiable Markdown report**
  (`validation/report.md`) with full provenance (version, GPU, CPU, OS), each check's
  parameters/result/threshold/verdict, golden checksums, and the exact `--render` command
  to reproduce each golden — so a third party can independently re-run and confirm.

- **Cross-renderer cross-check (Fraktaler-3)** — `--crosscheck-f3 raw.exr --center X Y
  --zoom-f3 Z [--iter K] [--er R]` validates against **Fraktaler-3** (Claude
  Heiland-Allen's independent GPU-perturbation renderer) at the *iteration* level. F3's raw
  EXR carries the integer escape count in a `UINT` channel `N` (exterior `n + 1024`,
  interior `0xFFFFFFFF`); we recover each pixel's exact `c` from F3's documented pixel
  mapping — including replicating its deterministic triangular sub-pixel **jitter**
  (`burtle_hash`/`triangle`, applied even at `subframes = 1`) and the vertical EXR flip —
  and compare F3's count to our independent arbitrary-precision **CPU bignum dwell oracle**
  (the same oracle `--selftest` checks our GPU pipeline against, so the results compose
  transitively into `our GPU ≈ Fraktaler-3`). Boundary/max-iteration-cliff pixels are
  excluded as genuinely ULP-ambiguous. Measured: **100%** interior/exterior membership and
  **100%** of exterior counts agree to within one iteration (≈79% exact; the residual ±1 is
  the `≥`-vs-`>` escape-test convention at band edges), holding undiminished at **10⁶×**
  zoom. New `fractadyne-export::read_exr_channel_f32` reads an arbitrary named EXR channel
  (UINT/F16/F32 → f32). Reproduction recipe + results table:
  [validation/crosscheck-fraktaler3.md](validation/crosscheck-fraktaler3.md). (Uses an
  external F3 EXR by design; kept entirely separate from `--selftest`, which uses no
  external data.)

- **Extreme-depth precision validation** — `--validate-deep [--out report.md]` validates the
  arbitrary-precision arithmetic core at magnifications far beyond `f64` range — **1e1000×,
  1e10000×, 1e100000×, and 1e1000000×** (≈3.3-million-bit precision). With no external
  corpus at this depth it uses the standard precision-doubling technique: iterate `z²+c`
  from a full-mantissa interior point (seeded by `√½` so the multiply exercises real carries
  across every limb) at precision `p` and again at `p+256`, and require the results agree to
  ≈`p` bits, plus a decimal `to_string → parse` coordinate round-trip. Feasible because
  `astro-float` switches to **FFT multiplication** above ~5400 limbs (measured ~32 ms per
  iteration at 3.3 M bits — near-linear, not quadratic) and the check is **single-point**
  (a per-pixel dwell oracle would take years that deep). New core API:
  `precision_for_octaves` (bypasses the `f64` magnification overflow), `deep_consistency_bits`,
  `deep_roundtrip_bits`; new `fractadyne-core` tests (`deep_precision_self_consistent_1e1000`,
  plus an `#[ignore]`d 1e100000× case). This surfaced that the renderer's **live** zoom is
  capped near **1e308×** by the viewport's `f64` `units_per_pixel`/`magnification` (the
  bignum *center* precision scales with depth; the *scale* underflows) — tracked in TODO as a floatexp /
  log-magnitude scale rework. Recipe + measured cost-scaling table:
  [validation/extreme-depth.md](validation/extreme-depth.md).

- **Lifted the ~1e308× live-zoom ceiling (extended-range scale)** — the viewport scale was
  an `f64` (`units_per_pixel`), so it underflowed (and `magnification()` overflowed) near
  **1e308×** — the real live-zoom wall, even though the arbitrary-precision *center* never
  ran out of digits. Introduced `FloatExp` (`m · 2^e`, `i32` exponent) and made
  `Viewport::units_per_pixel` use it, with `log2_magnification` + `precision_for_octaves`
  driving precision past f64 range, `complex_span_fe` / `gpu_scale` (an O(1) span mantissa +
  shared `delta_exp`) and `ref_offset_mantissa` feeding the GPU, `set_center_log2mag` +
  `--render --zoom-log2 L` for deep jumps, an extra session field so deep locations persist,
  and `fmt_zoom_log2` for the readout. **The WGSL shader was already exponent-aware (it
  consumes mantissas + `delta_exp`), so it needed no change** — the fix was entirely
  CPU-side. Verified: **bit-identical** to the previous build through 1e30× (selftest
  goldens, maxΔ 0), the GPU renders correctly at **1e331×** (interior/exterior classified
  exactly), 28 `fractadyne-core` tests pass (incl. a new past-1e308 scale test), and
  `--selftest` stays 29/29 + 4/4. (Follow-ups: the goto dialog and exported-image metadata
  still take `f64` zoom — fine to ~1e308×.)

- **Deep zoom save/restore/goto past 1e308×** — completes the ceiling lift so the deepest
  views are fully round-trippable. The "Go to location" dialog now parses and displays zoom
  via `log2(magnification)` (`parse_zoom_to_log2` / `fmt_zoom_field`: accepts plain or
  scientific input like `1.5e400`, grouping-tolerant, clamped to a sane octave bound — no
  more `inf` readout or f64 truncation). The reloadable image metadata carries an
  extended-range `upp_log2` (reconstructed on load; the f64 `upp` stays for back-compat and
  readability), so **exported PNG/EXR images and bookmarks restore views deeper than 1e308×
  exactly**. Round-trip unit-tested (shallow through 1e30000×).

- **Auto-zoom autopilot** — hands-free continuous deep zoom that re-steers toward detail
  (XaoS-style), via **View → "Auto-zoom (autopilot)"** or the **A** key; **Esc** or any
  navigation input stops it. Every ~0.35 s it renders a small (56×56) iteration field of the
  current view through the live perturbation pipeline (so it works at any depth) and scores
  each cell by **boundary adjacency + escape-time gradient**, center-biased for a stable
  dive. The zoom pivot **eases toward the evaluated goal every frame** (time-constant
  smoothing) rather than snapping at each re-evaluation, so the pan direction changes
  smoothly instead of jerking; it zooms toward that gliding pivot each frame (reusing
  `zoom_at` + the continuous-zoom rate), treating the dive as interaction (AA off, throttled
  reference refresh). Stops on a dead end (no boundary detail in view) or at a depth cap
  (~1e271×).

- **Shareable `.fdn` locations** — **File → "Share location…"** opens a dialog showing the
  current view as a self-contained text blob (fractal, full-precision center, the
  extended-range `upp_log2` so depths past 1e308× round-trip, zoom, coloring): **Copy** it to
  the clipboard, **Apply** a pasted/edited one, or **Save .fdn… / Load .fdn…**. So an exact
  location/look is shareable as a short text snippet or a tiny file. Untrusted input is
  handled safely — size-bounded (a 256 KB cap plus a file-size check) and parsed through the
  existing **hardened, fuzzed** `load_view_metadata`/`meta_get` chain (key=value allow-list,
  every field validated/clamped, unknown keys ignored, no paths or code execution).

- **Series approximation (iteration-skipping)** — deep Mandelbrot renders (mode 2, ≥1e28×)
  now skip the early perturbation iterations by seeding `δz` from an order-3 polynomial
  `δz ≈ A·δc + B·δc² + C·δc³`. The coefficients are iterated in arbitrary precision alongside
  the reference orbit (`A'=2ZA+1, B'=2ZB+A², C'=2ZC+2AB`); the skip is the largest count where
  the cubic term stays `≤2⁻¹⁶` of the linear term for the worst-case corner `|δc|` (which also
  guarantees no pixel escapes before the skip), cached per reference. The GPU evaluates the
  polynomial in floatexp to seed `δz` and the derivative `D`, then iterates from `skip`.
  Disabled for Julia, non-Mandelbrot formulas, and aux-accumulating coloring methods
  (stripe/TIA/orbit-trap/decomposition need every iterate). **Validated:** the seeded render
  matches full iteration (`maxΔ 0`) and the independent bignum oracle (0 mismatches) at 1e30×
  — a new `--selftest` check confirms both engagement and equivalence. Default on (toggle in
  View → "Series approximation"); the perf panel shows the skip count. Mode-0 (1e4–1e28×) and
  other formulas are follow-ups (see TODO).

- **Development profiling harness** — `--profile [--reps N] [--regions FILE] [--out PATH]`
  renders a set of benchmark **regions** (built-in defaults spanning the regimes: direct,
  df32 perturbation, floatexp + series approximation, plus a stripe variant that disables SA)
  and times the costly stages separately — bignum **reference orbit**, **series-approximation**
  setup, and the **GPU iterate / full render** passes — then writes a structured **JSON log**
  to `logs/` with full run context (version, GPU, CPU, OS, settings) plus per-region
  min/median/mean/max. Surfaces bottlenecks at a glance (e.g. at 1e30× the smooth render is
  ~3× faster than the SA-disabled stripe one, while the series-skip setup is itself a
  measurable cost). `scripts/profile.ps1` runs it; `scripts/profile-compare.ps1` diffs a
  before/after pair (per-stage % change, flags regressions) to validate optimizations;
  `scripts/regions.example.toml` is an editable region set. Logic lives in a new
  `profile` module (keeping `main.rs` glue lean); opt-in, so zero overhead in normal use.

- **In-app Help & reference** — Help → "Help & reference…" (or F1 / ?) opens a multi-section
  window with a table of contents: Overview, Navigation, Coloring & options, Fractals
  (mathematically accurate per-family formulas + descriptions, Julia mode, deep-zoom
  support), How it works (escape-time, arbitrary precision, perturbation, floatexp,
  distance estimation), Command line, Shortcuts, and About. Written for newcomers.

- **Famous-locations tour, random location & help overlay** — a **Locations** menu with
  curated named Mandelbrot spots (Seahorse / Elephant Valley, spirals, a mini-Mandelbrot,
  a deep seahorse) that jump at full precision, plus **"🎲 Random location"** which finds
  a detail-rich boundary point (bisecting between an interior anchor and a random exterior
  direction) and zooms in a random amount. A **Keyboard & controls** overlay (Help menu /
  **F1** / **?**) documents every shortcut and the new coloring/minimap/minibrot features.

- **Custom gradient / palette editor** — Coloring → "Edit gradient…" (or the "Custom"
  palette entry) opens an editor with a live gradient preview, a color picker and
  position slider per stop, add/remove (up to 8 stops), and "Copy preset…" to seed from a
  built-in. The custom gradient is persisted and used everywhere the palette is (live
  view, export, minimap thumbnail).

- **Minimap overview ("you are here")** — View → "Minimap overview" shows a small static
  home-view thumbnail (rendered via the export pipeline, cached per fractal/palette/
  method) in the bottom-left, with a marker for the current location (the view rectangle
  when shallow, a crosshair when the view is sub-pixel deep) and the live zoom-depth
  label. Click it to jump to a region at home zoom. Persisted; shown in single
  Mandelbrot-mode (hidden in dual / Julia).

- **Period / minibrot finder ("zoom to center")** — View → "Find minibrot center" (or
  press **M**) snaps the view center to the nearby minibrot's exact nucleus and reports
  its period in a transient toast. Detects the atom-domain period (global argmin of |Zₙ|
  on the critical orbit), then Newton-refines `c` in arbitrary precision until the orbit
  closes (`Z_period(c) = 0`), recovering the true smallest period and rejecting runaway
  Newton / non-nuclei. Holomorphic families (Mandelbrot / Multibrot). Verified deep
  (period-998 at 2e7×). Headless `--find-minibrot --center X Y [--zoom M] [--fractal N]`.

- **More coloring methods** — a Coloring → "Method" picker beyond smooth iteration:
  **stripe average** (flowing sinusoidal orbit bands, with a density slider),
  **triangle-inequality average**, **orbit trap** (point / cross / circle shapes, colors
  the interior too), **distance estimate** (shades by proximity to the boundary), and
  **decomposition** (binary external-angle cells). Orbit statistics are accumulated on
  the GPU into a second render target (added only when a method needs it, so smooth/
  distance keep full speed) and work at any zoom depth (direct + both perturbation
  paths). Persisted; CLI `--method NAME [--stripe-freq N] [--trap point|cross|circle]`.

- **Go-to location dialog + navigation undo/redo** — View → "Go to location…" to
  view/edit/paste/copy the exact center (full precision) + zoom; navigation history
  records each settled location and discrete jumps, with Backspace = undo view,
  Shift+Backspace / Ctrl+Y = redo (and View-menu items). Keys are ignored while typing.
- **Fractadyne branding & theme** — a charcoal dark UI theme with amber (#E0A030)
  accents (selection, links, hovered/active widgets), the two-color "Fracta·dyne"
  logotype in the top bar, and a procedural amber-ring window/taskbar icon — matching
  the `design/Fractadyne.dc.html` mockup.
- **Animated 3D relief lighting** — "Rotate light" spins the relief light direction over
  time (shares the Speed slider), alongside the animated distance glow and palette cycling.

- **Auto-incrementing build versioning** + this changelog; version shown in the title
  bar, Help menu, and export metadata.
- **Randomized palette mode** — palette animation can synthesize and continuously morph
  random gradients instead of cycling a fixed preset. Gradients are **harmonious** (one
  base hue + gentle analogous excursion + smooth dark→bright→dark arc), not clashing.
- **Scripting** — keyframe camera tours (center + zoom over time, eased), loadable from
  TOML, with a built-in demo/benchmark tour.
- **Benchmark** — runs a fixed scripted tour while sampling FPS, CPU ms, GPU ms, and
  peak RAM, then reports aggregates for comparing builds and machines. Report includes
  host **system info** (CPU brand/cores/cache, GPU, VRAM). Runnable headless via
  `fractadyne --benchmark [--out PATH]` for automated evaluation.
- **Headless render** — `fractadyne --render --out IMG [view options]` renders a single
  fractal image (fractal/center/zoom/size/ss/iter/julia/palette) and exits, for
  debugging and automated golden-image checks.
- **Release build** — `[profile.release]` tuned to build under this machine's memory
  limit (no debuginfo, no LTO). Optimized numerics: bignum reference recompute ~8×
  faster (374 ms → 45 ms), avg CPU ~7.6× faster. Build counter is now shared across
  debug/release profiles (`.build_seq` at the workspace root) so it stays monotonic.

- **Extreme deep zoom (floatexp δ)** — the GPU perturbation delta now uses a floatexp
  representation (df32 mantissa + i32 exponent) past ~1e28×, removing the f32 exponent
  wall that broke rendering around 1e31–1e32×. Hybrid by depth (direct → df32
  perturbation → floatexp perturbation) so the common range keeps full speed; floatexp
  (~1.7× per-iteration cost) engages only when needed. Depth is now limited by the
  center-coordinate precision (which grows as you zoom) and the iteration budget.

- **Bookmarks / presets** — save the current view (full precision) and jump back to it
  instantly. Bookmarks menu + ★ toolbar button + a Manage… window (add/name/list/delete);
  persisted to `bookmarks.toml` in the config dir.

- **Distance-estimate relief lighting** — optional 3D/embossed shading from the
  fractal's derivative (`dz/dc`), tracked in floatexp so it works at any zoom depth
  (direct + perturbation paths; holomorphic families). Coloring-panel toggle + light
  angle/relief sliders; `--light` CLI flag; persisted. Iteration texture is now
  RGBA32F (the slope normal rides alongside the iteration value).
- **Distance-estimate glow (+ animation)** — bright distance-contour bands that densify
  into glowing filaments near the boundary, from the derivative magnitude (distance
  estimate). "Distance glow" toggle + Glow/Band-width sliders + "Animate glow" (flows
  the bands). `--de` CLI flag; persisted. Works at any depth (verified at 1e8×).

- **Deep zoom for the abs families (Burning Ship / Celtic / Buffalo)** — the non-analytic
  families now perturbation-deep-zoom like the analytic ones (previously direct only,
  ~10⁶×). Because they take absolute values, the abs fold on a z² component is handled with
  the Kalles-Fraktaler **`diffabs`** identity `|c+d|−|c|`, evaluated branch-wise against the
  reference z² so it never suffers catastrophic cancellation: exactly `±d` when the
  reference component and its perturbation share a sign, `±(2c+d)` across a sign flip. Both
  render paths fold: `df_diffabs` in the df32 loop (mode 0, ~10⁴×…~10²⁸×) and a new
  **scalar-floatexp** `Sf` type with `sf_diffabs`/`fe_from_sf` in the floatexp loop (mode 2,
  past 10²⁸×) — needed because the complex `Fe` shares one exponent across re/im while the
  fold is per-component. Core `step_bf` gained the bignum reference iterations. `--selftest`
  verifies perturbation == direct at 1e5×, floatexp == df32 at 1e10×, and finiteness at
  1e35×. Lighting/DE stay off (these maps are non-holomorphic). Residual fold speckle awaits
  multi-reference glitch correction.

- **View-file format versioning + hardened loader** — the reloadable view metadata (exports
  / `.fdn` / bookmarks) gained a single source-of-truth `VIEW_FORMAT_VERSION`; loading now
  returns a report and **surfaces anything noteworthy** instead of loading silently. Opening
  a file from a *newer* Fractadyne warns "some settings may not apply — consider updating"
  (best-effort load; the format is additive key=value so core fields still parse); the
  loader also reports **clamped** out-of-range fields and **ignored unknown** keys. The
  untrusted parser is hardened: `max_iter` ≤ 10⁷, anti-aliasing 1..16, zoom depth ≤ 3.4e7
  octaves (a hostile `upp_log2` can't balloon bignum precision into a memory DoS), and
  `cycle`/`offset` rejected when non-finite. A file with no `format_version` is treated as
  v1 (legacy files still load). `--selftest` covers round-trip, newer-version detection, and
  clamp/report.

- **Depth-aware status-bar readouts** — the center coordinate now shows full arbitrary
  precision with the changing **frontier** digits visible: at deep zoom the middle is elided
  (`-0.74364 38870 … 06114 7740`) so the deepest digits no longer freeze at `f64`'s ~15
  (they used to look static while panning deep). The magnification's scientific-notation
  mantissa is space-grouped in 5s (`3.38050 02722 7e15`) to match the coordinate readout.

- **Series approximation on the df32 path (mode 0)** — the iteration-skipping seed, previously
  floatexp-only (mode 2, ≥1e28×), now also accelerates the common df32 perturbation range
  (1e4–1e28×). The order-3 polynomial seed is evaluated in floatexp (the coefficients overflow
  f32) then collapsed to the absolute df32 δ that path carries (`fe_to_cdf`); coefficients are
  mode-independent (computed once in bignum). Validated to reproduce full iteration exactly
  (max Δ 0) at 1e20× — skipping 19007 of 19008 iterations at a deep minibrot.

- **Series approximation for the Multibrot families** — SA now also accelerates Multibrot
  3/4/5, not just Mandelbrot. The order-3 coefficient recurrence is generalized to `z^d+c`
  (`A'=d·Z^{d-1}·A+1`, etc., with binomial weights); the GPU seed is already formula-agnostic.
  Validated by a core test (series vs exact perturbation for z³, rel err <1e-3) and a GPU
  check that SA engages and matches an SA-off render for all three families. (Tricorn and the
  abs families have no such δc expansion.)

- **Zoom-movie / frame-sequence export** — `--render-tour FILE [--fps N] [--size W]
  [--height H] [--ss N] [--out DIR]` renders a keyframe-tour TOML to a numbered PNG frame
  sequence (`frame_00000.png …`) for assembly into a deep-zoom dive video (prints an ffmpeg
  one-liner; example in `scripts/tour.example.toml`, also loadable via Tools → Play script).
  Reuses the scripting keyframe interpolation (factored into `Playback::sample`, shared with
  live playback) and the offscreen export path; samples the timeline at a fixed fps and
  recomputes a fresh deep reference per frame. Deep-correct (`set_center_log2mag`,
  octave-based precision) so dives past 1e308× sample exactly — which also fixed live
  playback (it used `set_center_mag`, saturating at 1e308×).

- **Prebuilt binaries via GitHub Releases** — `.github/workflows/release.yml` builds the
  Windows x64 binary and, on a `v*` tag push, packages `fractadyne.exe` + README + licenses
  into a versioned zip with a SHA-256 sidecar and publishes a GitHub Release (auto-generated
  notes) via the `gh` CLI. A manual `workflow_dispatch` run uploads the zip as a test
  artifact instead of publishing. Users can now download and run without the Rust toolchain
  (README gained a **Download** section).

- **Continuous integration** — `.github/workflows/ci.yml` gates every push/PR with the
  exact-math core test suite (`cargo test -p fractadyne-core`, Linux) and a full
  `cargo build --workspace` (Windows) confirming the GPU/egui crates still compile. The GPU
  `--selftest` stays a local/manual gate (runners have no GPU).

### Fixed (post-baseline, this session)

- **Black moving frame past ~1e420×** — while zooming (e.g. holding space) very deep, the view
  went solid black until it settled. The interacting path hard-capped iterations at 50k for
  responsiveness, but past ~1e420× the fractal needs far more (~369k at 1e431×) to escape, so every
  pixel read as interior → black. Now motion keeps the same zoom-appropriate iteration count as the
  settled view and lets the smaller work budget reduce the iteration-texture *resolution* instead —
  blurry-but-correct while moving, sharpening on settle (full-iteration deep frames are cheap at
  reduced resolution: ~1 ms). Also part of this fix: the deep-zoom reference recompute is off-thread
  with a last-good-frame freeze when the reference drifts fully out of view, so a slow/continuous
  deep dive holds the previous frame instead of flashing.

- **Smoother deep dives (less "zoom, pause, zoom")** — the bignum reference was rebuilt every single
  octave (both precision and the iteration cap grow each octave), and at extreme depth the rebuild
  can't keep up with a continuous zoom → periodic stalls. Now one reference serves ~32 octaves while
  moving: its precision is allowed to lag within the 64 guard bits, and the orbit is built with
  ~32 octaves of iteration headroom — so rebuilds happen ~32× less often during a dive. The view
  still rebuilds at full precision the moment it settles, so the still frame is maximally sharp.

- **More Help polish** — the content now scrolls when it overflows (the window was growing to
  fit and pushing content off-screen; its height is now capped so the scroll area engages);
  the key column in shortcut/flag tables is left-aligned (was centered); and math glyphs that
  the default font lacked (→, ≪, super/subscripts) now render via a bundled fallback font
  instead of showing as tofu boxes.

- **Help window layout was broken** — the table-of-contents + content were hand-split in a
  horizontal layout with manual width math that egui didn't honor, so the content ran off
  sideways and paragraphs wrapped to one character per line. Rebuilt with the standard
  `SidePanel` + `CentralPanel` idiom so the content is width-bounded and wraps normally, with
  a proper vertical scroll.

- **Minimap "you are here" marker was invisible / missing** — the amber marker had almost no
  contrast against a warm-palette thumbnail (e.g. Ember), and it was only drawn when the view
  center fell inside the minimap's fixed region. Now it always shows (clamped to the thumbnail
  edge if the view is outside the region) and is drawn with a dark halo behind the bright
  marker so it reads on any palette — a view rectangle when shallow, a crosshair + centre dot
  when deep.

- **Frame-rate cap "Uncapped" wasn't persisted** — the cap was stored as an `Option<f64>`,
  and TOML omits `None`, so the *uncapped* choice was dropped and reloaded as the default 60
  every restart. Now stored as a plain `f64` (`0` = uncapped) that round-trips. While auditing
  persistence, also added the missing view/preference state to the saved session so restart
  fully restores where you were: **fractal family, Julia mode + parameter `c`, dual view, and
  the series-approximation toggle** (center/zoom/coloring/lighting/export settings already
  persisted). Round-trip + legacy-file tests added in `fractadyne-state`.

- **Speckle/noise across the exterior at deep zoom on a large window** — a very high
  iteration count (e.g. a base of 50,000) over-resolved the boundary's sub-pixel "dust"
  into per-pixel noise *and* consumed the entire GPU-watchdog budget (forcing low
  resolution and no anti-aliasing). Rendering now caps the iteration count at a
  **zoom-scaled** value (`~2000 + 256·octaves`) — generous enough that normal
  auto-iteration is never limited, but an inflated manual base is — applied to both the
  live view and exports so they match. Result: coherent structure instead of dust. When
  the budget still can't afford true supersampling, a color-pass box filter anti-aliases
  the settled view; at extreme depth it falls back to reducing resolution (box-filtered).
  The perf panel's "eff iter" now reports the count actually rendered.
- **Quick export froze the app / could crash on a deep view** — the export's reference
  orbit was built on the main thread (briefly freezing the UI), and `render_export` tiled
  by texture/buffer size only, so a single tile at a huge iteration count was an enormous
  GPU submission that monopolized the shared device (freezing the live UI) and could trip
  the OS GPU watchdog (TDR → device-lost). Exports now use the same zoom-appropriate
  iteration cap, and export tiles are additionally bounded by **iteration work** so each
  GPU submission stays short — the UI stays responsive and the watchdog never fires. (A
  3840×2160, 2× export of a deep view now finishes in a few seconds.)

- **Deep zoom lost on quit/restart (uniform screen after relaunch)** — the auto-saved
  session stored the center as `f64`, so restoring a deep view truncated the coordinate
  to ~16 digits → a wrong location → uniform. The session now persists the center as a
  full-precision decimal string (like bookmarks/exports) and restores via `parse_bf`,
  falling back to `f64` for old session files. Also fixed the autosave debounce, which
  reset its timer every frame so an animating palette offset blocked the idle save
  (it now saves ~every second). Plus **multi-scale reference search**: the perturbation
  reference picker sampled a single coarse grid over the full view, which on a wide
  window at deep zoom landed between the thin filaments → a useless reference → uniform;
  it now samples several scales concentrated toward the center.
- **Uniform render at extreme depth on a large window** — the GPU-watchdog budget
  (texels × iterations) was kept by *clamping the iteration count*; on a big window
  (>~1.2 Mpx, i.e. maximized or with Windows display scaling) at very deep zoom that
  capped iterations well below what the detail needs, so the whole view escaped
  "late"/never and rendered as flat interior. Now the budget is met by reducing the
  iteration-texture *resolution* (the color pass upscales it) while keeping the full
  iteration count — graceful softness instead of a blank. (`--render` was unaffected,
  which is why a bookmark looked detailed when exported but blank live.)
- **Uniform/blank render after a cold jump to deep zoom (bookmark reload, Open view,
  `--render`)** — `best_reference` ranked candidate reference points using `f64`
  coordinates, which all collapse to the same value at deep zoom, so the search
  defaulted to the view center; if that sat in a fast-escape gap it was a poor
  reference → glitchy/uniform. Gradual zoom hid this by carrying a good reference
  chosen at shallower depth. Reference candidates are now scored in **arbitrary
  precision** (`orbit_length_bf`, scan-capped), so cold jumps find a good reference at
  any depth. (Confirmed the bookmark *coordinate* itself round-trips to ~1e-79 — far
  sub-pixel — so it was never an imprecision; added round-trip tests as guards.)
- **Soft "impressionist" frames while zooming deep** — the high-precision reference
  orbit was only refreshed once the view *settled*, so during motion a stale /
  out-of-view / under-precise reference made the perturbation blotchy until you paused.
  It now also refreshes *during* motion when out of view or under-precise, throttled
  adaptively (~2.5× the last recompute's duration) so it stays sharp without stalling
  the frame-rate (smooth on the release build; throttle widens automatically on debug).
- **Julia deep-zoom rebasing** — rebasing reset `δz = z_full`, valid only when
  `reference[0] = 0` (Mandelbrot). Julia orbits start at `Z₀ = ref_point ≠ 0`, so every
  rebase offset the perturbation and corrupted deep Julia renders. Now rebases to
  `δz = z_full − reference[0]` (a no-op for Mandelbrot).
