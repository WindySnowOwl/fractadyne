# Fractadyne relative-performance benchmark kit

A reproducible head-to-head of deep-zoom Mandelbrot renderers on **your** hardware:

| Renderer | Lane | Engine | Notes |
|---|---|---|---|
| **Fractadyne** | automated | GPU (wgpu: Vulkan/DX12/Metal/GL) | the app this kit ships with |
| **Fraktaler-3** | automated | GPU (OpenCL), plus the CPU unless `-F3GpuOnly` (BLA + rebasing) | binary + source included (AGPL-3.0) |
| **Imagina** | automated (`-ImaginaCliExe`) | CPU (MipLA) | upstream is GUI-only; a small headless `imagina-cli` fork enables the automated lane - see "Imagina" below |
| **FractalShark** | automated (GPU, 0.541+) | GPU (CUDA) | GPU lane works from 0.541 (adds sm_75; runs on RTX 20/30/40/50); earlier releases were CPU-only here - see below |

Ten single-frame scenes from Fractadyne's cross-validated corpus (each verified pixel-for-pixel
against Fraktaler-3), spanning 1e6× to 4.6e1105× magnification, plus a **zoom sequence** lane
(below) that measures what no single frame can:

| Scene | Magnification | Iterations | Regime |
|---|---|---|---|
| 03-seahorse-1e6 | 1.3e6 | 3,000 | shallow (direct/df32) |
| 04-seahorse-1e12 | 3.9e12 | 60,000 | perturbation |
| 08-deep-6.6e43 | 8.9e43 | 60,000 | deep floatexp |
| 14-deep-1.2e148 | 1.6e148 | 800,000 | deep, dense field |
| 17-deep-4.2e275 | 5.5e275 | 600,008 | very deep |
| 10-deep-4.6e1105 | 6.1e1105 | 250,000 | extreme |
| 21-m43-spar-1e27.7 | 5.1e27 | 30,000 | Misiurewicz spar |
| 23-nucleus-p145-1e27.7 | 5.1e27 | 30,000 | period-145 nucleus (setup-dominated) |
| 24-nucleus-p148-1e28.2 | 1.8e28 | 30,000 | period-148 nucleus |
| 35-field-dive-1p47e77 | 1.5e77 | 300,000 | dense field, normalized |

Every scene ships in three formats with the **same** center, magnification and iteration cap:
`.kfr` (Kalles Fraktaler text format — Imagina and FractalShark import it), `.f3.toml`
(Fraktaler-3), `.fdn` (Fractadyne). All renders are **3840x2160** by default (`-Size WxH` to
change it, applied identically to every lane) at **one sample per pixel** — supersampling
semantics differ per renderer and would silently benchmark different work. The scene files are
correctness fixtures first, so they carry the corpus's own resolution and sample count
(Fraktaler-3's `subframes = 4`, paired there with Fractadyne's `--ss 2`); `run-all.ps1` rewrites
both into a per-run copy, and the corpus originals are never touched.

### The ladder: depth to 1e30000x, iteration caps to 28.5 million (`-SceneSet ladder`)

The ten scenes above stop at 6.1e1105x and 800,000 iterations. `scenes-ladder.csv` adds 21 that
reach the deep and the long, in three rungs:

- **Minibrots** from 1.3e13x to 1.5e30000x, each centred on its nucleus with an iteration cap of
  30 times its period (a minibrot's own picture keeps changing up to about there; 30 against 300
  periods still differed in 1.1% of pixels, so the cap is a choice, recorded here).
- **Views along one spiral path** from 1e13x to 1e30000x (the path to `validation/spiral-9.98e60205.fdn`,
  the deepest location in the repository), each capped at three times the period of the minibrot it leads to; on these the
  picture no longer changes at 10 or 30 times.
- **An iteration ladder**: four minibrots between 1.3e53x and 2.1e68x whose periods run from 953 to
  951,094, so the cap climbs to 28.5 million at nearly the same depth.

Every minibrot was located with Fractadyne's nucleus finder (`--find-minibrot --period-search ball`)
and checked by Newton's method; the iteration ladder's are tuned copies inside the period-953
minibrot (`c_A + s_A·c_B`, period `p_A·p_B`). The files live in `ladder\`, not the corpus: they are benchmark scenes, not cross-checked
correctness fixtures. Run them with `-SceneSet ladder` (or `all`); expect hours, not minutes, of
which the two 1e30000x scenes take several minutes per render in every lane.

| Scene | Magnification | Iterations | Rung |
|---|---|---|---|
| 40-mini-p128-1e13 | 1.3e13 | 3,840 | minibrot, period 128 |
| 41-mini-p448-1e34 | 5.2e33 | 13,440 | minibrot, period 448 |
| 42-mini-p2066-1e97 | 3.4e96 | 61,980 | minibrot, period 2,066 |
| 43-mini-p4796-1e296 | 2.3e296 | 143,880 | minibrot, period 4,796 |
| 44-mini-p14416-1e1001 | 3.4e1000 | 432,480 | minibrot, period 14,416 |
| 45-mini-p41722-1e3000 | 1.0e3000 | 1,251,660 | minibrot, period 41,722 |
| 46-mini-p137396-1e10003 | 3.4e10002 | 4,121,880 | minibrot, period 137,396 |
| 47-path-1e13 | 1.0e13 | 2,859 | spiral-path view |
| 48-path-1e33 | 1.0e33 | 2,859 | spiral-path view |
| 49-path-1e100 | 1.0e100 | 10,293 | spiral-path view |
| 50-path-1e300 | 1.0e300 | 26,673 | spiral-path view |
| 51-path-1e320 | 1.0e320 | 28,311 | spiral-path view |
| 52-path-1e1000 | 1.0e1000 | 84,003 | spiral-path view |
| 53-path-1e3000 | 1.0e3000 | 247,803 | spiral-path view |
| 54-path-1e10000 | 1.0e10000 | 821,103 | spiral-path view |
| 55-path-1e30000 | 1.0e30000 | 2,459,103 | spiral-path view |
| 56-iter-p953-1e53 | 1.3e53 | 28,590 | minibrot, period 953 (iteration ladder) |
| 57-iter-p15248-1e57 | 2.1e57 | 457,440 | minibrot, period 15,248 (iteration ladder) |
| 58-iter-p121984-1e66 | 1.7e66 | 3,659,520 | minibrot, period 121,984 (iteration ladder) |
| 59-iter-p951094-1e68 | 2.1e68 | 28,532,820 | minibrot, period 951,094 (iteration ladder) |
| 60-mini-p410594-1e30000 | 1.5e30000 | 12,317,820 | minibrot, period 410,594 |

The twelve minibrot scenes give Fractadyne `--normalize --log-palette` (`normalize` = `log` in
`scenes-ladder.csv`; `1` is `--normalize` alone). Without it their field is palette noise: escape
counts in the hundreds of thousands put neighbouring pixels far apart on the cyclic palette. On an
RTX 3080 at 4K it costs Fractadyne 0.1 to 0.4 s a render, so it does not flatter it; the other
renderers colour as they always do.

Two scenes need their centre in a file, because a 1e30000x centre is 30,000 digits per coordinate
and a Windows command line holds 32,767 characters. Each lane checks its line first: Fractadyne
reads the same arguments from an `@file` (kept in `fd-args\`), FractalShark reads a locations file
written by `tools\make-fsloc.py` (kept in `fs-loc\`; it needs Python), Fraktaler-3 already reads
its scene file, and imagina-cli, which takes the centre only on its command line, records
`NA-command-line` for that scene. On Linux (`bench-latest.sh --scene-set ladder`) an argument may
be 128 KiB, so no lane needs a file.

Known result, so nobody chases it: Fraktaler-3 3.1 reports every pixel of
`59-iter-p951094-1e68` as never escaping and writes a black image, also with
`maximum_bla_steps` raised to the cap; the other three render the minibrot.

## Prerequisites & quick start

Prerequisites: **PowerShell** (Windows) or **bash** (Linux), and **Python 3 with Pillow**
(`pip install pillow`) — Python drives the zoom-sequence lane, the cutoff-crossing lane, and the
blank-frame structure checks. Every renderer is optional; a lane you don't supply is simply skipped.

- **Fastest path** — `bench-latest` fetches each renderer's current release and runs the whole
  single-frame + zoom-sequence benchmark (see "One command" below).
- **Local build or a subset** — point `run-all.ps1` at the exes you have (see "Running by hand").
- **Cutoff-crossing lane** — a separate tool for the arithmetic-mode transitions (see that section).

Every run writes `results/<host>-<timestamp>/` with `sysinfo.txt`, `results.csv`, `summary.md`, and
an `apps-manifest.txt` stamping each renderer's version and binary hash — so a result says exactly
what produced it and can be reproduced or challenged.

**Start with `report.html`.** Every run also writes a single self-contained page that puts the
three things a reader needs in one place, per scene: the image each renderer produced, the exact
command line and inputs that produced it, and the time. It opens with findings computed from the
data rather than asserted — fastest per scene, the largest margin, every DNF with its reason, the
run-to-run spread so you know what counts as noise on that machine, and the view-check verdict —
then the results table, then the per-scene cards, then every image the run made, then every
command it executed in order including one-off setup such as Fraktaler-3's hardware tuning and
the FractalShark server.

Underneath it is `run-manifest.json`, which is the primary record and is **always** written: per
render the executable, the full argument line, the working directory, the parsed inputs, the
output file and the outcome, plus sha256 and size for every binary involved. The HTML is a
rendering of that file and needs Python with Pillow; if either is missing the page is skipped and
the manifest is not, because a missing dependency should cost you the readable view and never the
evidence. Rebuild or refresh a page at any time with
`python tools/make-report.py results\<host>-<stamp>`, including for folders written before the
manifest existed.

**Where Fractadyne's time went.** A wall time says how long, not where. The Fractadyne lane keeps
every render's own log (`fd-logs/<scene>-r<rep>.log`) and parses it into `fd-phases.csv`, one row
per rep: startup, the wait for the reference (and, under the `FRACTADYNE_TRACE=ref` the lane sets,
that build's pick / orbit / SA / BLA times), GPU iterate and color time from timestamp queries,
host work (readback, normalization), PNG encode + write, exit, and what falls outside the log
(process launch, DLL load, teardown), plus the step counters: executed steps, iterations per step
(BLA skipping), df32 share, rebases. `summary.md` and the report show the median per scene, as a
table and as a stacked bar. The trace is a handful of log lines per render. In two interleaved
A/B runs with the order swapped, whichever arm ran first was slower, by about 1% in the quieter
run, and the trace made no difference that survived the swap. `-NoPhaseTrace` runs without it. `bench-lib.ps1` `Read-FdPhases` defines every
column. Builds before beta.149 report no GPU iterate time on the normalized scenes and no PNG
write time. The logs carry full paths, so redact them with the rest before publishing.

**Every rep, not only the fastest.** The tables lead with the fastest rep, which is the protocol.
With `-Reps 2` or more, `summary.md` adds each renderer's fastest, median and slowest per scene
and the spread, (slowest - fastest) / median, and the report prints the median and spread under
each time. Two renderers are only different by more than their spread.

**Publishing a report.** `--self-contained` inlines the thumbnails and writes a single
`report-standalone.html` that travels on its own. Before a report leaves the machine, add
`--redact-home`: a Windows profile directory is an account name, often a real name, and the
command lines repeat it hundreds of times — 313 in one page here, 213 in one manifest. Redaction
leaves the arguments complete and the paths readable. It only covers the HTML the tool generates,
so run `python tools/redact-home.py <files>` over anything else you publish beside it, and
`--check` to verify before you push. For a page shared by URL rather than linked, `--noindex`
adds a robots meta and a no-referrer policy: unlinked is not unindexed once a URL has been
shared, and a `robots.txt` rule would be worse than nothing because it is world-readable and
would publish the very path you are trying not to advertise. `--note "<html>"` puts a line under
the title, which is where a published page says what run it is and links a companion run; a page
someone reaches by a bare URL has no other context.

This exists because of a specific failure. The FractalShark lane spent its whole life sending a
zoom the renderer silently truncated, so it benchmarked the wrong view at every depth, and no
artifact on disk recorded the argument responsible. The numbers were checkable and the thing that
made them wrong was not.

Note on scene files: the distributed kit zip already contains the ten scenes in all three formats.
From a **git clone** they are generated, not committed (only the four cutoff-crossing `.kfr` are) —
run `powershell -File package.ps1` once to stage `scenes/` before `run-all.ps1`. The cutoff-crossing
tours in `tours/crossing/` are committed and need no staging.

## The zoom-sequence lane

Every scene above is ONE frame, and one frame is blind to the optimisation that matters most
for zoom video: a dive toward a fixed centre keeps the same reference orbit valid across many
frames, so the expensive setup can be amortised instead of paid per frame. A single-frame
benchmark scores that work as exactly zero.

The lane descends a Misiurewicz λ-ladder — every frame is the **same picture at a different
scale**, so per-frame cost *should* be flat and any ramp is the renderer failing to reuse its
setup rather than the scene getting harder. The metric is each app measured against **itself**:

    amortisation = frames_rendered × single_frame_wall / sequence_wall

1.0 means everything is rebuilt every frame; higher means reuse. Because it is a self-ratio it
needs no cross-app calibration.

It runs automatically when Python 3 is on PATH (the ladder needs 400-digit decimal arithmetic
to place its rungs); `-ZoomSeqFrames 0` or `-Skip zoomseq` turns it off, and without Python the
run says so and continues. Fractadyne renders the sequence in ONE process (`--render-tour`),
which is the mode that owns its reference prefetch.

**Read Fraktaler-3's figure carefully.** Its 3.1 batch CLI renders one image per invocation, so
its "sequence" is N processes and its amortisation is 1.0 *by construction*. That is a property
of the command-line interface, not of its engine — F3 has zoom-sequence and exponential-map
machinery this lane cannot reach. Do not quote it as an engine ceiling.

**Fraktaler-3's devices and tiles are set by YOU, and its defaults are conservative.** Its wisdom
(generated once per machine with `-W`, then benchmarked with `-B`) enables every device it finds,
the CPU included, and F3 spreads tiles across all of them — so a CPU tile can still be running long
after the GPU has finished. Pass **`-F3GpuOnly`** when the wisdom is created to disable the CPU
(platform -1), as F3's author does. Measured 2026-09-24 on the RTX 3080, alternating old and new
wisdom in one session: the 6.1e1105 scene went from 68.0 s to 12.9 s, the rest up to 26% faster.
**`-F3Tile WxH`** sets `opencl.tile_width`/`tile_height`; left empty, F3 uses 128x128, which its
manual calls conservative. Pick a size that divides the frame exactly (960x540 at 3840x2160) and
that stays under the OS GPU watchdog — a tile that overruns it resets the display driver, which on
the machine you are sitting at is a device loss. Every F3 call passes `-P`, so the lane neither
reads nor rewrites your own F3 persistence file.

## The cutoff-crossing lane

Fractadyne switches arithmetic at fixed magnifications — direct→df32 at 1e4, df32→floatexp at 1e28,
and there is an f64 magnitude edge near 1e308 — and a switch mid-dive is where the reference-reuse,
prefetch and mode-transition code is most likely to break. `tools/gen-crossing-tours.py` generates
N-frame tours whose zoom sweeps a band straddling one cutoff at a fixed hard center (the committed
set lives in `tours/crossing/`: seahorse across 1e4, the Misiurewicz spar and a period-148 nucleus
across 1e28, a deep field across 1e308). Two harnesses consume them:

- `tools/cutoff-validate.py TOUR` — a Fractadyne correctness gate. Renders the tour twice through the
  identical `--render-tour` path, reuse OFF (cold oracle) vs reuse ON (warm/sequenced), and diffs
  every frame: they must be bit-identical, or a caching optimization changed the picture. Then runs
  `--livetest` for the live/multithreaded path.
- `tools/crossing-bench.py` — the cross-renderer lane. Drives each renderer through the SAME per-frame
  views at one sample per pixel: Fractadyne as a sequence (`--render-tour`, plus singles for the
  amortisation denominator), and Fraktaler-3, FractalShark and Imagina one process per frame. Every
  frame is structure-checked, so a blank render is `DNF-blank`, never a time. FractalShark is driven
  through its `--locations` file (not `--center-x`) to dodge a 0.541 CLI parse bug that otherwise
  blanks the spar and nuclei; the Imagina lane uses the headless `imagina-cli` (see "Imagina"); Julia
  tours record `NA-julia` for the F3/FractalShark/Imagina lanes (the Julia parameter is not passed to
  them). Run:
  `python tools/crossing-bench.py --fractadyne <exe> [--fraktaler3 <exe> --f3-wisdom <toml>]
  [--fractalshark-cli <exe>] [--imagina-cli <exe>] [--size 3840x2160]`.

## Fairness protocol

- Plug in, high-performance power plan, close other GPU/CPU-heavy apps.
- Each lane runs the same scenes at the same size and iteration caps.
- **Two timing columns, deliberately:**
  - `wall_s` — process start to exit, measured by the script. Only meaningful for the two
    automated lanes; it includes reference building and encode for both, so it is the honest
    end-to-end comparison between them. **FractalShark in server mode** has no process per image,
    so its `wall_s` is the client call until **that image's PNG is complete on disk**, and the next
    image starts only after it: the server encodes PNGs in the background and the next render
    waits for the encode, so client calls timed back to back charged every image with the
    PREVIOUS image's encode. Every FractalShark time this kit published up to 2026-10-03 has that
    flaw (measured 2026-10-04: the spar scene's 69 ms render took 3.5 s, waiting for scene 17's
    12.7 MB PNG). `fs-phases.csv` splits each wall into the client call and the encode, beside
    FractalShark's own report (reference orbit, LA tables, per-pixel), printed because the lane no
    longer passes `--quiet`. **Each image gets a fresh server**, warmed by one untimed frame
    entirely outside the set: a server KEEPS each scene's reference orbit, so with one server for
    the whole lane, rounds 2 and 3 of a scene reused the orbit round 1 built, and "fastest of N"
    picked a frame that skipped it (measured 2026-10-05: the 1e10000x path view took 17.1 s in
    round 1, 2.4 s and 2.1 s after it). Every run of the other three lanes builds its own orbit.
    The warm-up absorbs a server's one-off first-render setup (scene 03 at 4K: 1.2-1.3 s as a
    server's first image, 0.36 s after the warm-up) and its orbit is one step long, so nothing in
    it can be reused. Startup and warm-up stay outside every image's time.
  - `reported_s` — the renderer's own render-time figure. Fractadyne prints one, and FractalShark's
    client prints one that the lane records only for a row that already passed the exit code and
    the structure guard (a *refused* render prints one too). Self-reported figures exclude
    different amounts of startup and encode per renderer: compare them across ALL lanes, but treat
    small differences as noise. Never mix the two columns. With `imagina-cli` automated, all four
    renderers now have a `wall_s`, which is the column to compare.
- Run everything at least twice if you can (`-Reps 2`); the summary uses the fastest run
  (cold-start effects, driver shader caches, and OS file cache all favor later runs — the
  fastest run is the closest to "the renderer's actual speed on this machine").
- A scene a renderer cannot complete (crash, >2 h timeout, unsupported hardware) is recorded
  as `DNF` — a result, not a gap.

## One command: fetch the latest apps and run everything

`bench-latest` downloads the newest release of every renderer, verifies the target drive has
enough free space first, runs the benchmark sequentially, and writes the summary report with
the exact app versions stamped into it:

- **Windows**: `powershell -ExecutionPolicy Bypass -File bench-latest.ps1`
  (options: `-AppsDir <folder>` where the apps land, `-RequiredGB 2` free-space floor,
  `-Reps`, `-Skip`, `-Scenes`, `-TimeoutS`, `-SkipDownload` for offline reuse,
  `-FractadyneExe <path>` to benchmark a local build instead of the release).
- **Linux**: `./bench-latest.sh` (same options in `--flag` form; `--fraktaler3 <path>` points
  at a locally built Fraktaler-3, since mathr publishes no Linux binary — the script tells
  you where the source lives).

Sources: Fractadyne and Imagina from their GitHub releases (prereleases included — that is
where the current builds live), Fraktaler-3 from `fraktaler.mathr.co.uk/download/latest`,
FractalShark from its GitHub releases (downloaded only on NVIDIA machines, where its lane can
actually run). Published sha256 side-files are verified. Each result folder gains an
`apps-manifest.txt` recording version, source URL, and binary hash per app — a "latest"
benchmark that doesn't say which latest it measured is not reproducible.

## Running by hand

1. Unzip anywhere writable. Install what you want to compare:
   - Fractadyne: place `fractadyne.exe` in `bin\` (or pass `-FractadyneExe <path>`).
   - Fraktaler-3: included in `fraktaler3\` (with its source, per AGPL-3.0).
   - Imagina: download from https://github.com/5E-324/Imagina/releases (AGPL-3.0),
     pass `-ImaginaCliExe <path>` for the automated headless lane (see "Imagina"), or
     `-ImaginaExe <path>` to fall back to the assisted GUI lane.
   - FractalShark: download from https://github.com/mattsaccount364/FractalShark/releases
     (GPL-3.0), pass `-FractalSharkExe <path>`. The lane finds `FractalSharkCli.exe` beside
     it and runs automatically (`-FractalSharkCliExe <path>` to point elsewhere); see
     "FractalShark, honestly" for what it can and cannot render headlessly.
2. `powershell -ExecutionPolicy Bypass -File run-all.ps1` (add `-Reps 2` for repeats; skip
   lanes with `-Skip imagina,fractalshark`).
3. Results land in `results\<hostname>-<timestamp>\`: open **`report.html`** first; beside it are
   `run-manifest.json`, `sysinfo.txt`, `results.csv`,
   `summary.md`, `fd-phases.csv` and `fd-logs\` for the Fractadyne lane, and `zoomseq\` when the
   sequence lane ran. Send the whole folder (or its zip) to feedback@fractadyne.org, or attach it
   to a GitHub issue on WindySnowOwl/fractadyne.

## FractalShark, honestly

FractalShark ships `FractalSharkCli.exe` beside the GUI, so this lane is automated. Through 0.54
its GPU path could not run on this RTX 3080 at all; **0.541 (2026-09-12) fixed that**, and the lane
now measures real GPU renders. The history and the current caveats, because the kit says so rather
than papering over it:

- **0.541 added `sm_75` code, so the GPU lane runs on RTX 20/30/40/50.** Through 0.54 the release
  binaries embedded PTX and SASS for `sm_89` and `sm_120` only (`tools/cuda-arch-inventory.py`), so
  on an RTX 3080 (`sm_86`) — or anything below Ada — no kernel could load: `tools/cuda-load-check.py`
  handed every fat binary to `cuModuleLoadData` and all 33 came back `CUDA_ERROR_NO_BINARY_FOR_GPU`
  (209), and every GPU algorithm returned a flat image at exit 0. 0.541 embeds `sm_75` + `sm_89` +
  `sm_120` (PTX and SASS); the added `sm_75` PTX JIT-compiles onto `sm_86`, all 10 fat binaries now
  load `CUDA_SUCCESS`, and the GPU renders are real pictures. Pass a GPU algorithm explicitly
  (`-FractalSharkAlgo GpuHDRx32PerturbedLAv2`); `AutoSelect` picks a non-HDR GPU algorithm that goes
  flat at deep zoom (blank at 6.6e43 here), so it is the wrong lane default.
- **The old OpenGL-context warning is now cosmetic.** The CLI still prints "OpenGlContext: null HWND
  / OpenGL context creation FAILED, no rendering will occur" on stderr, but on 0.541 the GPU output
  reaches the PNG regardless — the blank-headless-GPU symptom is gone. (Through 0.54 that path, on
  top of the missing kernels, is why headless GPU renders came back blank.)
- **The zoom it is given is not always the zoom it renders, and this invalidated every
  FractalShark number this kit published before 2026-09-21.** The lane used to send
  `--zoom 1e<mag_log10>`, e.g. `1e6.1249387366083`. `FractalSharkCli` accepts that and **silently
  truncates the fractional exponent**: on 0.543 that render is byte-identical to `--zoom 1e6`,
  while the correct `--zoom 1.333333E6` produces a different and correct image. Every scene came
  out under-zoomed by up to 10x, exited 0, and passed the structure check as a real picture of the
  wrong place. **An under-zoomed frame is a cheaper frame**, so the fault presented as FractalShark
  being 20-50x faster than both other renderers at 4K, including 0.3 s for a 1e1105 frame that
  takes Fraktaler-3 62.8 s. The lane now hands over the `.kfr`'s own `Zoom` string verbatim, which
  is the Kalles Fraktaler convention every other lane already uses. If you have older results from
  this kit, the FractalShark column in them is void; the other lanes are unaffected.
- **Imagina renders in Fractadyne's own palette (`-ImaginaPaletteMap`, on by default).**
  Comparing two renderers by image only works while their palettes are comparable, and by default
  they are not at all. Rather than move both onto a neutral palette, the lane moves only the one
  we control: Fractadyne renders exactly as it ships, so every reference image and historical
  number stays comparable, and `imagina-cli` is calibrated to meet it.
  `palettes/ember.map` is Fractadyne's built-in **Ember** (its default preset) baked to 1024
  entries the way its own gradient baker does. **Position matters as much as colour**: Fractadyne
  maps `position = smooth_iteration * cycle + offset` and wraps, at `cycle = 0.0202` and
  `offset = 0.1` by default, while Imagina's smooth value advances at twice that rate, so the
  matching cycle is exactly half — `0.0101` at offset `0`. That factor of two came out of a
  two-dimensional sweep over two scenes at opposite ends of the corpus, both peaking at the same
  point, so it is a convention difference rather than a per-scene fudge.
  Measured on the period-148 nucleus, correlation against the Fractadyne render:

  | palette | RGB correlation |
  |---|---|
  | each renderer's own | 0.067 |
  | a neutral shared `.map` | 0.496 |
  | Ember, matched cycle and offset | **0.949** |

  `-SharedPalette <file>` is the other option: it puts *both* Fractadyne and Imagina on one file.
  It is off by default, because switching Fractadyne off its preset changes every reference image.
  Either way **only those two lanes can join** — Fraktaler-3 3.1 exposes no colour options in its
  batch toml or its CLI, and `FractalSharkCli` has none at all — so those two keep their own
  colours and the report says so rather than implying the column is like-for-like.
  What remains after matching is the finer detail Fractadyne resolves through its analytic
  palette anti-aliasing, which Imagina point-samples; the shallow scene sits at 0.76 for that
  reason.
- **A shared palette also removes a harness cost that was being counted as engine time.** Our
  `imagina-cli` fork coloured every pixel with three `cos()` calls; at 4K that is 25 million
  cosines, and it measured **1.83 s against 0.93 s** for the same scene with a `.map` lookup,
  three reps each. Roughly half of every Imagina figure this kit ever produced was our own
  colouring code. Colouring cost cannot be equalised across renderers we do not control, so the
  honest statement is that each lane pays whatever its own output path costs, and the two lanes
  we do control now pay a table lookup.
- **Mirrored output is its own verdict.** `verify-views.py` also scores each render against the
  reference flipped and rotated, because nothing else here can see a mirror: interior fraction is
  exactly flip-invariant, and a large palette difference flattens the image test. This is not
  hypothetical - our own Imagina fork wrote its PPM rows top-down out of a bottom-up buffer, so
  every Imagina image in this kit was upside down and the lane passed all ten, since
  "magnification agrees" is true of a mirror image too. A mirror now fails the check and names
  the axis.
- **A time is only meaningful for the work that was asked for, so the lane now checks the view.**
  The structure check asks "is this a picture"; it has never asked "is this the *right* picture",
  and all ten wrong renders passed it — one of them a near-flat field. `tools/verify-views.py`
  compares each render against the Fractadyne render of the same scene on two signals: interior
  (never-escaped) fraction, which moves fast with magnification, and gradient-magnitude
  correlation, which survives the palette inversion that defeats a plain grayscale correlation.
  Each scene is also scored against every other scene as a control.
  **The two signals carry different authority, and the difference matters.** Interior fraction
  tracks magnification and no palette choice can move it, because whether a pixel escaped is not
  a colouring decision; when it disagrees, the render is of somewhere else and the row becomes
  `DNF-not-the-scene`. Edge correlation compares *pictures*, and two renderers can draw the same
  iteration field so differently that it collapses — see the 1.6e148 note below — so on its own
  it only downgrades a row to `view UNCONFIRMED`, keeping the time. Treating that as a failure
  once made this kit state that another project's renderer was broken when it was not. Where the
  check cannot run (no Python, or the Fractadyne lane skipped) rows say `view NOT verified`
  rather than passing quietly. Run it by hand with
  `python tools/verify-views.py results\<host>-<stamp>`.
- **`GpuHDRx32PerturbedLAv2` renders almost the whole corpus on 0.543; what it misses, it misses by
  location, not depth.** Re-measured at 4K on 2026-09-21 with the corrected zoom and the view
  check: **all ten scenes render the right picture**, from 1e6 through the extreme
  4.6e1105 — including the Misiurewicz spar, both period nuclei and 4.2e275, all four of which
  older notes here recorded as `DNF-blank`. **That older list is superseded**; it was written
  against 0.54x, and the four are no longer blank.
  **All ten render.** `14-deep-1.2e148` was briefly recorded here as returning noise; that was
  wrong, and the correction is worth keeping because it is a trap anyone comparing two renderers
  will hit. Downsampled, that frame looks like salt-and-pepper static where the dendrite belongs.
  At 1:1 it is smooth, continuous iteration banding: FractalShark advances its palette once per
  iteration, the scene caps at 800,000 iterations, and where the field moves fastest adjacent
  pixels land on different colours, so the bands alias into apparent static the moment anything
  resamples them — while our render of that same scene is the one we apply `--normalize` to,
  mapping the identical field onto one slow gradient. Same view, two images with nothing visually
  in common, and no CLI option on either side to bring the palettes closer.
  Its CPU algorithms (e.g. `Cpu64PerturbedBLAV2HDR`) still go blank past ~1e27, so for depth use a
  GPU algorithm.
- Because of that, **no FractalShark row records a time without a picture**: every render is checked
  for structure first, and a flat image becomes `DNF-blank`. This kit once published "144x faster
  than Fraktaler-3" for a frame that was entirely empty; never again. (The structure check itself
  was tightened in 2026-09: the old "any two sampled pixels differ" test passed a near-flat blank
  with a few stray edge pixels; it now requires many distinct colours and no single colour over 98%,
  matching `tools/image-structure.py`.)
- **The GUI can also be driven without a mouse** — `tools/fractalshark-gui-scene.ps1` is a working
  prototype (window sized, Enter Location dialog filled, algorithm and antialiasing set by posted
  menu commands, "Benchmark (5x, full recalc)" writing `BenchmarkResults.txt`, bitmap saved through
  Save As). Use it for the GUI's own reported figure; the automated CLI lane is the default now that
  it produces real GPU numbers.

FractalShark is a CUDA renderer, and the automated lane now measures its GPU path. Say which path
produced a number when you compare — a GPU time against another renderer's GPU time is the
like-for-like one.

## Imagina

Upstream Imagina ships a GUI only — no headless render mode — so it has two lanes:

- **Automated (`imagina-cli`)** — a small headless fork of Imagina drives the engine directly
  (`SetLocation` → render loop → read the pixel buffer → PPM), with no GUI, no GL context, and no
  libpng. It renders the whole corpus (1e6 through 4.6e1105). **`run-all.ps1` now drives it too**:
  pass `-ImaginaCliExe <exe>` and the lane automates like the others, which supersedes the assisted
  lane whenever the CLI is present. Because it is a derivative of Imagina it is **AGPL-3.0**, kept
  in a separate repo and never linked into Fractadyne. Building it needs msys2 mingw-clang and GMP;
  the changes from upstream are a headless entry point, an MPIR→GMP switch, and a Windows LLP64 fix
  to `FloatExp`'s `mpf` exponent read. Ask us for the fork, or reproduce it from upstream with those
  notes.
  Two things the lane has to get right, both of which have bitten this kit before:
  **it needs the msys2 toolchain DLLs on `PATH`** or it exits `0xC0000135` with no message at all,
  which reads exactly like a crash (`-ImaginaRuntimeDir`, autodetected from `C:\msys64`); and
  **the zoom is the `.kfr` `Zoom` string with a lowercase `e`**, because `mpf_set_str` takes a
  mantissa with an *integer* exponent and silently will not take the fractional-exponent spelling
  — the same class of trap that had the FractalShark lane rendering the wrong view for months.
  Output is PPM, converted once via `tools/ppm-to-png.py` so the structure guard and the report do
  not each need a second format.
- **Assisted (the GUI)** — used only when no `imagina-cli` is supplied. `run-all.ps1` launches the
  app per scene and prompts you for the render time it displays. That is transcription, not
  automation — type what the app shows, don't estimate. It imports `.kfr` (format drift happens;
  record DNF and say why if a scene won't load).

## Licenses

This kit redistributes Fraktaler-3 (AGPL-3.0) with its corresponding source in
`fraktaler3\source\`. Imagina (AGPL-3.0) and FractalShark (GPL-3.0) are NOT bundled —
download them from their own release pages, linked above. Scene files are original to the
Fractadyne project.
