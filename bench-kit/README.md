# Fractadyne relative-performance benchmark kit

A reproducible head-to-head of deep-zoom Mandelbrot renderers on **your** hardware:

| Renderer | Lane | Engine | Notes |
|---|---|---|---|
| **Fractadyne** | automated | GPU (wgpu: Vulkan/DX12/Metal/GL) | the app this kit ships with |
| **Fraktaler-3** | automated | CPU (OpenMP, BLA + rebasing) | binary + source included (AGPL-3.0) |
| **Imagina** | operator-assisted | CPU (MipLA) | no headless mode; you transcribe its reported time |
| **FractalShark** | automated (CPU path) | GPU (CUDA) | headless CLI renders only its CPU algorithms - see below |

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
| 35-vger-dive-1p47e77 | 1.5e77 | 300,000 | dense field, normalized |

Every scene ships in three formats with the **same** center, magnification and iteration cap:
`.kfr` (Kalles Fraktaler text format — Imagina and FractalShark import it), `.f3.toml`
(Fraktaler-3), `.fdn` (Fractadyne). All renders are **3840x2160** by default (`-Size WxH` to
change it, applied identically to every lane) at **one sample per pixel** — supersampling
semantics differ per renderer and would silently benchmark different work. The scene files are
correctness fixtures first, so they carry the corpus's own resolution and sample count
(Fraktaler-3's `subframes = 4`, paired there with Fractadyne's `--ss 2`); `run-all.ps1` rewrites
both into a per-run copy, and the corpus originals are never touched.

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

## Fairness protocol

- Plug in, high-performance power plan, close other GPU/CPU-heavy apps.
- Each lane runs the same scenes at the same size and iteration caps.
- **Two timing columns, deliberately:**
  - `wall_s` — process start to exit, measured by the script. Only meaningful for the two
    automated lanes; it includes reference building and encode for both, so it is the honest
    end-to-end comparison between them.
  - `reported_s` — the renderer's own render-time figure (Fractadyne prints one; Imagina and
    FractalShark display one that you transcribe). Self-reported figures exclude different
    amounts of startup/encode per renderer: compare them across ALL lanes, but treat small
    differences as noise. Never mix the two columns.
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
     pass `-ImaginaExe <path>`.
   - FractalShark: download from https://github.com/mattsaccount364/FractalShark/releases
     (GPL-3.0), pass `-FractalSharkExe <path>`. The lane finds `FractalSharkCli.exe` beside
     it and runs automatically (`-FractalSharkCliExe <path>` to point elsewhere); see
     "FractalShark, honestly" for what it can and cannot render headlessly.
2. `powershell -ExecutionPolicy Bypass -File run-all.ps1` (add `-Reps 2` for repeats; skip
   lanes with `-Skip imagina,fractalshark`).
3. Results land in `results\<hostname>-<timestamp>\`: `sysinfo.txt`, `results.csv`,
   `summary.md`, and `zoomseq\` when the sequence lane ran. Send the whole folder (or its zip) to feedback@fractadyne.org, or attach it
   to a GitHub issue on WindySnowOwl/fractadyne.

## FractalShark, honestly

FractalShark ships `FractalSharkCli.exe` beside the GUI, so this lane is automated — but what it
can measure is narrower than it looks, and the kit says so rather than papering over it:

- **Every GPU algorithm renders blank headlessly** (0.532). Each pixel comes back with iteration
  count 1, the PNG is one flat colour, and the exit status is **0**. The CLI admits it on the
  `--console` path ("all exterior pixels have the same iteration count 1") and its stderr says
  "OpenGL context creation FAILED, no rendering will occur". `AutoSelect` picks a GPU algorithm,
  so the obvious invocation is silently broken. Upstream CI smoke-tests `Cpu64` only.
- **The CPU algorithms work at shallow and mid depth** — verified 1e6 through 1e27 — and come back
  blank on the deeper corpus locations, regardless of how many digits of centre they are given
  (40, 60, 100 and 196 all blank). Expect real numbers for the shallow scenes, `DNF-blank` for the
  rest.
- Because of that, **no FractalShark row records a time without a picture**: every render is
  checked for structure first, and a flat image becomes `DNF-blank`. This kit once published
  "144x faster than Fraktaler-3" for a frame that was entirely empty; never again.

Comparing FractalShark's CPU path against another renderer's GPU path is not a like-for-like
statement about the app, which is a CUDA renderer. Say which path produced the number.

For its GPU figures, run the GUI by hand and transcribe them: pass `-FractalSharkExe` with no CLI
beside it and the kit falls back to the assisted prompt.

## The assisted lane, honestly

Imagina has no headless render mode, so its lane launches the app per scene and prompts you for
the render time it displays. That is transcription, not automation — type what the app shows,
don't estimate. If a scene doesn't import cleanly (it imports `.kfr`, but format drift happens),
record DNF and note why in the prompt.

## Licenses

This kit redistributes Fraktaler-3 (AGPL-3.0) with its corresponding source in
`fraktaler3\source\`. Imagina (AGPL-3.0) and FractalShark (GPL-3.0) are NOT bundled —
download them from their own release pages, linked above. Scene files are original to the
Fractadyne project.
