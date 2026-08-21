# Fractadyne relative-performance benchmark kit

A reproducible head-to-head of deep-zoom Mandelbrot renderers on **your** hardware:

| Renderer | Lane | Engine | Notes |
|---|---|---|---|
| **Fractadyne** | automated | GPU (wgpu: Vulkan/DX12/Metal/GL) | the app this kit ships with |
| **Fraktaler-3** | automated | CPU (OpenMP, BLA + rebasing) | binary + source included (AGPL-3.0) |
| **Imagina** | operator-assisted | CPU (MipLA) | no headless mode; you transcribe its reported time |
| **FractalShark** | operator-assisted | GPU (CUDA) | **NVIDIA only**; recorded N/A elsewhere |

Six scenes from Fractadyne's cross-validated corpus (each verified pixel-for-pixel against
Fraktaler-3), spanning 1e6× to 4.6e1105× magnification:

| Scene | Magnification | Iterations | Regime |
|---|---|---|---|
| 03-seahorse-1e6 | 1.3e6 | 3,000 | shallow (direct/df32) |
| 04-seahorse-1e12 | 3.9e12 | 60,000 | perturbation |
| 08-deep-6.6e43 | 8.9e43 | 60,000 | deep floatexp |
| 14-deep-1.2e148 | 1.6e148 | 800,000 | deep, dense field |
| 17-deep-4.2e275 | 5.5e275 | 600,008 | very deep |
| 10-deep-4.6e1105 | 6.1e1105 | 250,000 | extreme |

Every scene ships in three formats with the **same** center, magnification and iteration cap:
`.kfr` (Kalles Fraktaler text format — Imagina and FractalShark import it), `.f3.toml`
(Fraktaler-3), `.fdn` (Fractadyne). All renders are 1920x1080, supersampling **off** —
supersampling semantics differ per renderer and would silently benchmark different work.

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

## Running

1. Unzip anywhere writable. Install what you want to compare:
   - Fractadyne: place `fractadyne.exe` in `bin\` (or pass `-FractadyneExe <path>`).
   - Fraktaler-3: included in `fraktaler3\` (with its source, per AGPL-3.0).
   - Imagina: download from https://github.com/5E-324/Imagina/releases (AGPL-3.0),
     pass `-ImaginaExe <path>`.
   - FractalShark: download from https://github.com/mattsaccount364/FractalShark/releases
     (GPL-3.0), pass `-FractalSharkExe <path>`. Requires an NVIDIA GPU.
2. `powershell -ExecutionPolicy Bypass -File run-all.ps1` (add `-Reps 2` for repeats; skip
   lanes with `-Skip imagina,fractalshark`).
3. Results land in `results\<hostname>-<timestamp>\`: `sysinfo.txt`, `results.csv`,
   `summary.md`. Send the whole folder (or its zip) to feedback@fractadyne.org, or attach it
   to a GitHub issue on WindySnowOwl/fractadyne.

## The assisted lanes, honestly

Imagina and FractalShark have no headless render mode, so their lanes launch the app per
scene and prompt you for the render time each one displays. That is transcription, not
automation — type what the app shows, don't estimate. If a scene doesn't import cleanly
(both import `.kfr`, but format drift happens), record DNF and note why in the prompt.

## Licenses

This kit redistributes Fraktaler-3 (AGPL-3.0) with its corresponding source in
`fraktaler3\source\`. Imagina (AGPL-3.0) and FractalShark (GPL-3.0) are NOT bundled —
download them from their own release pages, linked above. Scene files are original to the
Fractadyne project.
