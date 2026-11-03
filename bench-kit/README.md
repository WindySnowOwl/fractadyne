# Fractadyne relative-performance benchmark kit

A reproducible head-to-head of deep-zoom Mandelbrot renderers on **your** hardware:

| Renderer | Lane | Engine | Notes |
|---|---|---|---|
| **Fractadyne** | automated | GPU (wgpu: Vulkan/DX12/Metal/GL) | the app this kit ships with |
| **Fraktaler-3** | automated | GPU (OpenCL), plus the CPU unless `-F3GpuOnly` (BLA + rebasing) | binary + source included (AGPL-3.0) |
| **Imagina** | automated (`-ImaginaCliExe`) | CPU (MipLA) | upstream is GUI-only; a small headless `imagina-cli` fork enables the automated lane - see "Imagina" below |
| **FractalShark** | automated (GPU, 0.541+) | GPU (CUDA) | GPU lane works from 0.541 (adds sm_75; runs on RTX 20/30/40/50); earlier releases were CPU-only here - see below |
| **Fractal Zoomer** | automated (`-FractalZoomerJar`, `-FzsWriterPath`, a JDK) | CPU (Java; perturbation + BLA) | its own command line (1.0.9.6), one batch per round; a small GPL-3.0 writer, FzsWriter, makes its `.fzs` inputs, and the lane competes on Fractal Zoomer's OWN time - see "Fractal Zoomer" below |
| **Fracturing Fog** | automated (`-FracturingFogExe`) | CPU (.NET; perturbation + series approximation + BLA; double-, quad- or octuple-double reference by depth) | its own batch command line (0.7.1); renders to 1e100, so the deeper scenes are `NA-past-its-ceiling` - see "Fracturing Fog" below |

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

`59-iter-p951094-1e68` is the scene whose Fraktaler-3 limits matter: at the kit's default of 8192
most of its field stops early and comes out black, so its row in `scenes-ladder.csv` sets
`f3_limit` to 32768, the figure Fraktaler-3's author gave (65536 draws the same image). Until
2026-10-10 the kit's Fraktaler-3 ran clamped at 4096 and this scene came out entirely black; see
"Fraktaler-3's limits" below.

### Suggested locations (`-SceneSet suggested`)

Locations other people proposed for what they stress, in `suggested\` and `scenes-suggested.csv`.
Like the ladder, they are benchmark scenes, not cross-checked correctness fixtures. Run them with
`-SceneSet suggested` (or `all`; `--scene-set suggested` on Linux).

| Scene | Magnification | Iterations | What it stresses | Source |
|---|---|---|---|---|
| 61-flake-7.8e156 | 7.8e156 | 60,000 | perturbation without rebasing needs many reference points to resolve it | Dinkydau's "Flake", suggested by kaludix |
| 62-fraqcel-2.3e31 | 2.3e31 | 2,000 | without rebasing, glitch correction is likely to recurse into the regions it corrected | the Fraqcel test location, suggested by kaludix |
| 63-cerebral-spin-6.7e152 | 6.7e152 | 2,000,000 | BLA far ahead of series approximation (36x in Very Plotter) | a deeper "Cerebral Spin", from [Phil Thompson's BLA article](https://philthompson.me/2023/Faster-Mandelbrot-Set-Rendering-with-BLA-Bivariate-Linear-Approximation.html) |
| 64-ssss-1.3e694 | 1.3e694 | 350,000 | series approximation ahead of BLA (1.66x in Very Plotter) | Dinkydau's "SSSSSSSSSSSSSSSS", from the same article |

kaludix suggested these in the fractalforums.org thread "Benchmarking deep zoom renderers": the
first two directly, and one location each from Phil Thompson's article where BLA or series
approximation wins. The centres and iteration caps are copied from the post and from the article's
own Very Plotter links. Magnifications are converted to the kit's zoom, where the view is 4 / zoom
high:

- kaludix's "Plot Scale" is read as the view's half-height (zoom = 2 / scale). That reading frames
  both of their views as a whole structure; read as the full height, the Fraqcel location runs off
  the frame.
- Very Plotter's `mag` makes the shorter side of the view 3 / mag (`magnificationFactor` 3 for the
  Mandelbrot set), so zoom = 4 × mag / 3.

Scene 63 gives Fractadyne `normalize` = `log`, as the ladder's minibrots do: at 2 million iterations
its field is otherwise palette noise.

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

**Fraktaler-3's limits, and why the kit gives it a profile of its own.** Fraktaler-3 3.1 renders a
batch PNG through its GUI program, which first loads the GUI's preferences. With Preferences ▸
Limit Iterations on, which is its default, that clamps the scene file's `maximum_perturb_iterations`
and `maximum_bla_steps` to 4096 each and its iteration cap to 2^27, without a word. So every F3 call
runs with a profile of the kit's own (`f3-profile\` in the run folder, Limit Iterations off; on
Linux through `XDG_DATA_HOME`) and never reads your own F3 preferences. Both limits are written into
each per-run scene file: **`-F3Limit`** (default 8192; F3's manual advises "a few 1000"), or the
scene's own `f3_limit` column in its scenes CSV where it needs more. A pixel that reaches either
limit stops and is coloured as interior. **Every Fraktaler-3 number this kit published before
2026-10-10 ran at 4096/4096**, whatever the scene files said. A survey that day (RTX 3080,
1280x720, Limit Iterations off) rendered all 31 scenes at 4096, 8192 and 32768: 29 drew identical
images at all three, in the same time; `58-iter-p121984-1e66` changed 4 pixels between 4096 and 8192
and none after; `59-iter-p951094-1e68` was black at 4096, 85% black at 8192 and complete at 32768.

**Fraktaler-3's view check reads its own iteration counts, not its image.** Until 2026-10-31 the
lane was not checked at all: `verify-views.py` looked for `f3-<scene>.png` and Fraktaler-3 writes
`<scene>-f3.png`. Run by hand, the check then called one correct render the wrong view. It judges
magnification by the share of pixels that never escaped, read off an image as its black share,
and Fraktaler-3's default colouring is a distance estimate: everything close to the boundary is
black too, escaped or not. `59-iter-p951094-1e68` is 62% black at 4K while 6.6% of its pixels never
escaped; the rest of its black escaped after 4 to 10 million of the scene's 28.5 million
iterations. Nothing else in such an image can stand in. Its black covers the right scene's
interior and the wrong scenes' alike, and its grainy shading leaves edge correlation negative at
every blur tried. So after the timed rounds the lane renders each scene once more, untimed, from
the same per-run scene file at the check's own 640x360, with `save_exr = true` and
`exr_channels = ["N0", "N1"]` in place of the PNG (`f3-raw\`). That costs time outside every row:
458 s for all 31 scenes on the test machine, 278 s of it the reference orbit of
`55-path-1e30000`, which no smaller image avoids. Fraktaler-3 marks a pixel that
never escaped with the count 0xFFFFFFFF, and a pixel stopped by either limit counts as one, so a
limit set too low still shows. `tools/exr-interior.py` writes those shares to `f3-interior.json`,
and `verify-views.py --interior-json` uses them in place of the black. For a scene without one,
the black stands as an upper bound, and only a shortfall counts against the render. As for
Imagina and Fractal Zoomer, the verdict (`f3-view-check.json`) goes to the report and does not
change a row. The lane now also refuses a missing or uniform image as every other lane does
(`DNF-no-output`, `DNF-blank`). A Fraktaler-3 whose wisdom file enables the wrong devices can write
a black frame in a second, and this lane used to record a time for it.

**The view check's magnification test is strict since 2026-11-01.** It compared each render's
black share with the Fractadyne reference picture's, within 0.06, and that was loose in two ways.
Palette 0 draws escaped pixels dark as well, so the reference overstated the interior: scene 59's
picture is 8.9% black where 5.5% of it is interior, and on every deep minibrot scene the other
four renderers agreed with each other and not with it. A 2.14x under-zoom of scene 57 passed. And
most scenes have no interior at all, where the test could say nothing, yet the check reported
"magnification agrees": re-run on the 2026-09-21 run whose zooms were truncated, it caught 1 of
the 10 wrong renders. Now:

- The Fractadyne lane renders each scene once more, untimed, through an all-white palette
  (`fd-interior\`), so the reference's black is its interior and nothing else. On a scene with 1%
  interior or more, a render may fall at most 20% short of it and exceed it at most 2x (dark
  palettes add black; FractalShark reached 1.44x), each bound no looser than the old 0.06. At least
  75% of the scene's interior pixels must be black in it, which palette darkness cannot fake: it
  only adds black. There an interior contradiction stands against a decisive picture match, which
  identifies the scene but not its magnification.
- On a scene with less, a scale search: edge correlation at magnifications from 0.1x to 10x. A
  render whose best match is a quarter-octave or more away, scores 0.45 there and beats the asked
  magnification by 0.25 fails, and the check names the factor. It is not used on the minibrot
  scenes, where correct renders also matched at other scales.
- Where nothing can check the magnification, the verdict says "magnification unchecked", not
  "agrees".

Measured: Imagina at 1.26x, 1.41x and 2x too deep and too shallow, 174 renders: 144 fail (the old
check: 35), including every under-zoom of a scene with interior. The truncated run: 9 of 10, each
with its factor (scene 04: 3.78x found, 3.90x true). 139 correct renders by five renderers: none
fail. Still passing: an over-zoom of a minibrot scene under 1.41x, and scenes 17 and 35 where a
renderer's palette matches ours at no scale (reported as unchecked). Results without the masks
get the old test, with the old verdicts. Details and every threshold's evidence are in
`tools/verify-views.py`.

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

## The inspection sheet (`-Inspect`)

Each renderer draws with its own palette, so two correct renders of one scene can look nothing
alike, and a real difference hides among the colouring differences. `-Inspect` adds an untimed
pass after the lanes that writes `inspect-sheet.jpg`: the renderers that allow it on ONE stepped
palette, side by side, and a map of where Fraktaler-3's iteration counts disagree with
Fractadyne's. It is for looking at, not for publishing; the published images keep each
renderer's own look.

- **The palette.** Four colours in a fixed order, light and dark alternating (white, blue,
  yellow, vermillion), never blended, interior black: a pixel's band is floor(n / k) mod 4 for its
  whole iteration count n. `k` is a power of two chosen per scene from Fractadyne's counts, so
  bands are about 10 pixels wide at 640x360. Hard edges are the point: a count off by a few
  iterations moves an edge, which a smooth ramp (grayscale included) would hide.
- **Who gets it.** Fractadyne (`--render-iter`) and Fraktaler-3 (its view check's raw EXR) write
  raw counts, which `tools/inspect-sheet.py` colours itself, so their palette is identical by
  construction. FractalShark 0.545 takes `--palette-map` and `--palette-depth`: it resamples the
  file to 2^depth entries and advances one entry per iteration (measured on scene 03), so the
  tool writes each scene a file of exactly 2^depth entries, each colour repeated k times
  (`inspect\<scene>.fs.map`). Imagina, Fractal Zoomer and Fracturing Fog appear in their own
  palettes for now. Fraktaler-3 3.1 has no colour option, which is why its raw counts are used.
- **The deviation map.** Fraktaler-3's counts, less their median offset from Fractadyne's (1024
  from its EXR's iteration bias plus about 3 from its escape radius), are compared with the RANGE
  of Fractadyne's 3x3 neighbourhood, since a renderer samples a pixel somewhere inside it.
  Outside that range by more than 2 iterations is a deviation, counted only in connected patches
  of 12 pixels or more. Measured on scenes 61 to 64: in smooth areas the two agree to one
  iteration everywhere, and they disagree only in isolated pixels where the field has structure
  finer than a pixel (drawn dull purple and reported as speckle). Magenta is a deviating patch,
  red escaped in one renderer only, grey chaotic (the neighbourhood spans more than a band).
- **It can go red.** Faults planted in Fraktaler-3's counts were found: a 30x30 block 50
  iterations out, a 20x20 blank patch and a 4x4 glitch, one patch each; a block shifted by half
  a band was found where it crossed smooth ground, which is the limit of a test done pixel by
  pixel. A 1-iteration shift of the whole frame was not flagged, as it should not be.

`run-all.ps1 -Inspect` needs the Fractadyne lane and Python with numpy and Pillow. By hand, with
`fd-raw\<scene>.exr` in a results folder: `python tools/inspect-sheet.py prepare results\<run>`,
then `python tools/inspect-sheet.py sheet results\<run>`. Per-scene numbers are in
`inspect\inspect.json`.

## One command: fetch the latest apps and run everything

`bench-latest` downloads the newest release of every renderer, verifies the target drive has
enough free space first, runs the benchmark sequentially, and writes the summary report with
the exact app versions stamped into it:

- **Windows**: `powershell -ExecutionPolicy Bypass -File bench-latest.ps1`
  (options: `-AppsDir <folder>` where the apps land, `-RequiredGB 3` free-space floor,
  `-Reps`, `-Skip`, `-Scenes`, `-TimeoutS`, `-SkipDownload` for offline reuse,
  `-FractadyneExe <path>` to benchmark a local build instead of the release).
- **Linux**: `./bench-latest.sh` (same options in `--flag` form; `--fraktaler3 <path>` points
  at a locally built Fraktaler-3, since mathr publishes no Linux binary — the script tells
  you where the source lives).

Sources: Fractadyne and Imagina from their GitHub releases (prereleases included — that is
where the current builds live), Fraktaler-3 from `fraktaler.mathr.co.uk/download/latest`,
FractalShark from its GitHub releases (downloaded only on NVIDIA machines, where its lane can
actually run), Fracturing Fog's Windows zip from its GitHub releases (Windows only; about 200 MB,
470 MB unpacked). Published sha256 side-files are verified, including a `SHA256SUMS` that lists
several files. Each result folder gains an
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
   - Fractal Zoomer: download `FZMinimalRenderer.jar` 1.0.9.6 or later from
     https://github.com/hrkalona/Fractal-Zoomer/releases (GPL-3.0) and pass
     `-FractalZoomerJar <path> -FzsWriterPath <path> -JavaExe <a JDK's java.exe>`; see
     "Fractal Zoomer". `bench-latest` does not run this lane, because the writer is not
     downloadable.
   - Fracturing Fog: download `FracturingFog-win-x64.zip` from
     https://github.com/AloneButUnsober/FracturingFog/releases (AGPL-3.0), unzip, and pass
     `-FracturingFogExe <FracturingFog.App.exe>`; see "Fracturing Fog".
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
  a colouring decision (read off an image it is the black share, which one renderer here breaks;
  see Fraktaler-3's view check); when it disagrees, the render is of somewhere else and the row becomes
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

## Fractal Zoomer

Fractal Zoomer renders from its own command line from 1.0.9.6 (`FZMinimalRenderer.jar`, from
https://github.com/hrkalona/Fractal-Zoomer/releases). Its inputs are `.fzs` files,
which are Java object serialization, so only Fractal Zoomer's own classes can write one: a small
writer, FzsWriter, does. FzsWriter calls into Fractal Zoomer, so it is GPL-3.0 too, kept in its own
folder and never part of this kit; ask us for it. Build it with
`javac -cp FZMinimalRenderer.jar -d classes FzsWriter.java`, then pass
`-FractalZoomerJar <FZMinimalRenderer.jar> -FzsWriterPath <classes folder> -JavaExe <a JDK's java.exe>`.

⚠**A JDK, not a JRE.** Fractal Zoomer compiles its user code at every start, and without a Java
compiler it opens a modal "Java Compiler was not found" dialog that a command-line batch waits on
forever. Its author's main target is Java 1.8, with `-XX:+UseG1GC` (the lane passes it by default,
`-FractalZoomerJavaArgs`; it is Java 11's default collector anyway).

What the lane does:

- writes every scene's `.fzs` once per run (`fz-fzs\`), with Fractal Zoomer's own
  `Settings.save`: the `.kfr` centre, the iteration cap from the scene table, and the
  magnification as `size = 4 / Zoom` (Fractal Zoomer's `size` spans the shorter side of the image,
  the height here), at the precision Fractal Zoomer itself derives from those strings;
- renders each scene of each round in a FRESH Java process of its own, through the command line
  (`--cli_batch_dir` holding just that scene's `.fzs`, `--cli_width`/`--cli_height` from `-Size`,
  `--cli_output_dir`), in a fresh working folder (`fz-batch\r<rep>\<scene>\`). The command line
  reads no preferences file; everything comes from the `.fzs` and the arguments.

  ⚠**Why a fresh process per scene, with no warm-up.** Fractal Zoomer's pixel loop runs at a speed
  that depends on what the same Java process rendered before it (measured 2026-10-13 at 4K, on
  Java 8 and 11, with the heap size making no difference). Rendered in one batch after scene 14,
  scenes 17 and 35 took 2-3x as long (17: 9.3-10.9 s against 4.3 s in a fresh process). After a
  cheap warm-up image, scenes 58 and 59 took 3.4x as long (59: 250 s against 73.5 s fresh, every
  round within 1%), so the warm-up meant to help hurt. A fresh process times each scene by itself,
  as a user opening that location sees it. On the shortest scenes the Java compiler's first-run
  cost, a few hundred milliseconds, is part of that. `-FractalZoomerSharedBatch` (one batch per
  round) and `-FractalZoomerWarmup` (an untimed warm-up image at a view outside the set first)
  reproduce the effects;
- reads each scene's time from the `<scene>.stats` file the batch writes beside the image, checks
  the image is complete (`Test-PngComplete`) and has structure, and keeps every `.stats` and the
  batch's output. The command line exits 0 even when a file fails, so a missing image or `.stats`,
  or a "Failed to render" line, is that scene's `DNF`.

The settings are the ones Fractal Zoomer's author advised for a benchmark: the defaults, with
perturbation on (Fractal Zoomer then picks its arbitrary-precision library and uses Zhuoran's
BLA by default) and the rendering method Successive Refinement. The default method, Patterned
Successive Refinement, is meant for the main window's rendering effects. Successive Refinement
guesses flat regions instead of iterating every pixel, so on a scene with large flat areas it
does less work for the same picture. Fractadyne iterates every pixel.

**The lane competes on Fractal Zoomer's own time, not the wall.** Its Total Elapsed Time is the
reference orbit plus the pixels, measured inside the program, without saving the image; the
reference alone is in each row's note. Its `wall_s` in `results.csv` is its process, start to exit:
starting Java, compiling its user code, rendering and saving, about 10 s more than its own time on
the test machine, which a user pays once per session. So the report never ranks Fractal Zoomer
against a wall-clock column (see "Fairness protocol"); it puts it beside Fractadyne's own
figure instead, which is the render and the PNG save without the process start. Fractal Zoomer does
not reuse a reference orbit between loads, and every scene of every round is a new process, so the
reps of a scene are independent. The view check (`fz-view-check.json`) compares each image with Fractadyne's
render of the same scene, as for FractalShark, Imagina and Fraktaler-3.

Before the command line existed (Fractal Zoomer 1.0.9.5), the lane drove the Minimal Renderer
through a driver, FzBench, one Java process per image. Fractal Zoomer writes the `.stats` file
BEFORE it saves the image, and the first 4K run's driver exited on the stats file, which cut every
image off part-way (its times were right); the kit's structure guard could not see it, because its
decoder fills the missing rows. That is why every image is now checked for its PNG end marker.
Fractal Zoomer's own command line exits only after the image is written.

## Fracturing Fog

Fracturing Fog (from 0.7.1; .NET, AGPL-3.0) renders a view headlessly:

    FracturingFog.App.exe --batch --x "<limbs>" --y "<limbs>" --zoom <z> --iter <cap>
        --width 3840 --height 2160 --quality Standard --no-taa --no-watermark --out ff-<scene>.png

The lane runs it once per image, a fresh process each, like every other lane. `tools/ff-view.py`
converts each scene's view once, before any timing. Each conversion was measured against the
Fractadyne render of the same scene:

- **Its zoom fixes the image's width.** Its pixel size is `3.5 / (max(width, height) · zoom)`, so
  the width spans 3.5/zoom; the `.kfr` zoom sets the height to 4/zoom. The lane passes the scene's
  zoom × 3.5 × (height / max(width, height)) / 4, which is 0.4921875 at 3840x2160. Scene 03 then
  has 42.1% interior against Fractadyne's 42.7%.
- **Its imaginary axis points down the image.** Given the scene's own centre, scene 03 came out
  flipped top-to-bottom at the right magnification. The Mandelbrot set is symmetric about the
  real axis, so the lane passes −Im: the conjugate centre drawn downward is the scene drawn upward.
- **A deep centre goes in as limbs.** `--x` and `--y` take up to eight doubles, `hi|lo|...`, whose
  sum is the centre (about 124 digits). The `.kfr` decimal is split exactly, with rational
  arithmetic, never through a float.

**One sample per pixel.** Its quality presets set the sampling: Draft and Standard take one
sample per pixel, High 4, Ultra and Extreme 16, plus temporal anti-aliasing on top. The lane
passes `--quality Standard --no-taa`. In batch mode the preset does not rise with depth, and the
arithmetic (double-double, quad-double, octuple-double) follows the zoom alone, so Standard
renders deep views at full precision. `--no-watermark` leaves its caption off the image.

**It runs on the CPU.** Its batch renderer attaches no GPU kernel, whatever the interactive app
uses, so this is a CPU lane like Imagina's.

**It renders to 1e100.** That is its Extreme preset's limit, octuple-double. Version 0.7.1 still
exits 0 past it, with one flat colour in a fraction of a second, measured at 1.6e148, 5.5e275 and
2.3e296. A flat frame of a scene with almost no interior gives the view check's magnification
test nothing to contradict. So the lane refuses those scenes up front (`NA-past-its-ceiling`), and the
structure guard stands behind it. Of the kit's 31 scenes, 17 are in range.

**Two clocks, as for every lane.** `wall_s` is its process, from start to exit after writing the
PNG, and is the column it competes in. `reported_s` is the time it prints, "saved ...
(3840x2160, N ms)": the render, before the PNG is written. On the test machine the gap was about
1.9 s per image, which is .NET start-up plus the PNG.

**Its view check runs on its default theme**, HSV, which cycles fast. Around a deep minibrot it
aliases into grey noise at 4K, so the image test cannot confirm those scenes, while their
magnification agrees; the shapes match when you look. Its interior is black, so the
magnification test applies as it stands.

Batch mode left Fracturing Fog's own settings folder (`%APPDATA%\FracturingFog`) unchanged on
the test machine.

## Licenses

This kit redistributes Fraktaler-3 (AGPL-3.0) with its corresponding source in
`fraktaler3\source\`. Imagina (AGPL-3.0), FractalShark (GPL-3.0), Fractal Zoomer (GPL-3.0) and
Fracturing Fog (AGPL-3.0) are NOT bundled — download them from their own release pages, linked
above. The kit only runs Fracturing Fog as a separate program. The
Fractal Zoomer writer, FzsWriter (and the earlier driver, FzBench), is GPL-3.0 and kept outside
this kit. Scene files are original to the
Fractadyne project.
