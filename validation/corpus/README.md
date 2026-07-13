# Fractadyne ↔ Kalles Fraktaler comparison corpus

Twenty reference locations — from the full-set overview down to **1.2e1008×** (deepest matched
pair **4.6e1105×**) — rendered by both apps from the **exact same center, magnification, and
iteration count**, for side-by-side structural comparison. Locations 11–20 are user-saved
Fraktaler-3 finds (1.7e124× → 1.2e1008×), imported verbatim from their `.exr` headers. Open **[catalog.html](catalog.html)** to view the pairs with each
location's full compute details.

## Layout

| Path | What |
|---|---|
| `locations.toml` | Canonical location list (full-precision centers, log10 magnification, iterations) — the single source of truth everything else derives from. |
| `locations/*.kfr` | Kalles Fraktaler 2 location files (fraktaler-3 reads them too). |
| `locations/*.fdn` | Fractadyne "Share location" files — load one to reproduce that exact view in the app (fractal, full-precision center, zoom, iteration count). Extracted from each render's embedded reload metadata; the catalog has a per-location download + "Copy .fdn" button. |
| `renders/<slug>-fractadyne.png` | Fractadyne renders (1280×720, 2× supersampling, Ember palette, smooth iteration). |
| `renders/<slug>-fraktaler.png` | Fraktaler-3 renders (1280×720; produced by `generate_fraktaler.py` where F3 works — see the limitation below). |
| `locations/*.f3.toml` | Fraktaler-3 batch parameter files (one per location), written for **every** location. |
| `generate_corpus.py` | Regenerates the `.kfr` files, the Fractadyne renders, and `catalog.html` from `locations.toml`. |
| `generate_fraktaler.py` | Runs the vendored Fraktaler-3 (`diag/fraktaler/`) in batch mode to produce the F3 renders + `.f3.toml` params. |

## Reproduce a location in Fractadyne

Every location ships as a `locations/<slug>.fdn` — Fractadyne's native "Share location" file. To open
one in the app:

- **File ▸ Share location ▸ Load .fdn…** and pick the file, or
- copy its text (the catalog's **Copy .fdn** button, or the `.fdn` file) and paste it into the same
  Share-location dialog.

It carries the fractal, the full-precision center, the per-pixel scale (extended-range, exact past
1e308×), and the iteration count, so the location loads in one step — no retyping a 2000-digit center.
The scale is stored per pixel and referenced to the corpus's 720px-tall framing, so a much taller window
frames the same center a little wider — the **center is exact** regardless, and a scroll of the wheel
trims the zoom. The `.fdn` is extracted from the reload metadata that `--render` embeds in every PNG
(`Fractadyne` tEXt chunk), with the save timestamp / build stripped so it stays stable across re-renders. Coloring loads as the corpus
contract (Ember / smooth); the deep dendrite/minibrot views used `--normalize` on export (a transient
export setting), so the live view shows the standard palette cycle at those depths.

## Regenerating the Fractadyne side

```
cargo build --release -p fractadyne-app
python validation/corpus/generate_corpus.py            # everything
python validation/corpus/generate_corpus.py --skip-renders   # just .kfr + catalog
```

## Checking for regressions (`--check`)

Run this after any change that could affect the renderer (a major update, a shader edit) to confirm
the corpus still matches the committed, F3-confirmed renders:

```
python validation/corpus/generate_corpus.py --check            # all 20 locations
python validation/corpus/generate_corpus.py --check --only 14,15
```

It re-renders each location to a temp file and compares it **pixel-for-pixel** against the committed
`renders/<slug>-fractadyne.png`. The renderer is deterministic (same pipeline ⇒ byte-identical pixels,
like the `--selftest` goldens), so an unchanged renderer prints `20/20 MATCH`; any `CHANGED` location
must be eyeballed against its F3 reference (regenerate, then open `catalog.html`) before it is
re-committed. The check exits non-zero on any change and never modifies the committed renders or
catalog. It is intentionally **not** part of the fast `--selftest` — a full 20-location re-render takes
minutes (the deep locations dominate), so it is an on-demand gate.

Renders go through `--render` with a fully staged session per location — exact center strings +
magnification, the location's verbatim iteration count (auto-scale off; `--render` honors an
explicit count, unlike the tour renderer, which forces auto-iteration and silently re-caps it),
and pinned coloring (Ember, smooth, no DE/lighting/HUD). Your session file is restored
afterwards. Staging everything the render reads is what makes the corpus reproducible — a
partial staging inherits the live session's coloring (the same hermeticity lesson as
`--selftest`).

## Producing the Fraktaler side

Automated, using the Fraktaler-3 3.1 binary vendored at `diag/fraktaler/`:

```
python validation/corpus/generate_fraktaler.py               # render everything F3 can
python validation/corpus/generate_fraktaler.py --max-log10 8 # only run F3 up to ~1e8x
```

It writes an F3 batch param (`locations/<slug>.f3.toml`) for **every** location and renders each
to `renders/<slug>-fraktaler.png` (1280×720, matched framing via `f3_zoom = mag × 4/3`), detecting
and skipping blank output.

You can also render any location by hand: **Kalles Fraktaler 2** or **fraktaler-3** opens the
`.kfr` (or the `.f3.toml`); set the image size to 1280×720 and save the PNG as
`renders/<slug>-fraktaler.png`.

### F3 deep renders and the `maximum_reference_iterations` gotcha

Fraktaler-3's **batch** mode defaults `maximum_reference_iterations` far too low for a zoomed view,
so without it F3 silently renders a **uniform / blank** image **at any depth** — no error, just a
flat result. For a long time this looked like a hard *"F3 blanks past ~1e13×"* ceiling (its
extended-exponent kernels vs this GPU): every deep test shared the same missing setting, so they all
blanked identically and consistently — across GPU and forced-CPU, F3 3.0 and 3.1, and a full NVIDIA
driver update + reboot. That reproducibility read as a hardware wall. It was a **config gap**.

`generate_fraktaler.py` now writes `maximum_reference_iterations`, `maximum_perturb_iterations`, and
`maximum_bla_steps` into every `.f3.toml`. With those set, **F3 renders deep correctly here** — real,
arm-for-arm matches verified all the way to **4.60e1105×** (location 10, a 1141-digit center; ~4 min
to render). Two further things have to hold for a clean cross-app match: the iteration cap must be
high enough for the depth (F3's reference otherwise truncates and blanks), and the **center must carry
enough digits for the depth *plus* margin** for F3's internal reference rounding, which is coarser
than Fractadyne's full-precision bignum reference.

Net: clean, arm-for-arm F3 matches at **all 20 locations** — from 1× to **4.60e1105×**, over a
thousand orders of magnitude of zoom. The former gap at **07** (1e30×) is now closed: its old shared
34-digit seahorse center was too coarse for a reproducible match there — F3 rounded it below
Fractadyne's bignum reference onto different sub-structure — so it was **replaced by a user Fraktaler-3
save** (`me30.exr`, center −1.1788…, 43-digit) that matches arm-for-arm (both apps render the same
diagonal dendrite filament with a central pinch and spiral-armed clusters). The center-precision lesson
still stands — the plainest proof being that 08 (83-digit center), 09 (526-digit), and 10 (1141-digit)
all render and match *far* deeper. The `.f3.toml` / `.kfr` params are written for all twenty, and
Fractadyne renders all twenty.

**Deep dendrite / minibrot views (13, 14, 15, 16–20) — a coloring note, and one render fix.** For a
long time 14/15 rendered as per-pixel **speckle** and were wrongly diagnosed (in turn) as glitches, then
as a perturbation-accuracy bug. Both were wrong. The escape **values are correct** — verified by a
faithful CPU transcription of the exact mode-2 shader kernel (df/Cdf/Fe Dekker–Knuth arithmetic, shared
exponent, Zhuoran rebasing; `fractadyne-core/tests/probe_fe.rs`), which reproduces the GPU's smooth-iter
values (df32 == df64 == GPU). The problem is **coloring**: at these depths the smooth-iter counts are
huge (~3e5–1.6e6) and, at dense dendrite/minibrot fields, vary steeply pixel-to-pixel, so the fixed
palette `cycle` (0.27) turns a correct escape field into aliased speckle. All the deep views with a
rapidly-varying exterior (**13, 14, 15, 16–20**) therefore use **auto-normalized coloring**
(`--normalize` — the frame's escape range mapped to the palette) and then match F3 arm-for-arm; the
spiral views **09–12** have slowly-varying exteriors that don't alias, so they keep the standard cycle.

**15 additionally needed a real render fix (v0.2.18).** Its right-side dendrites escape at 928k–1.6M —
*past* the reference orbit (~918k) — so they must rebase at the reference's near-zero orbit dips. The
GPU's BLA iteration-skip path was skipping those dips *without* a Zhuoran rebase check (it assumed δz
stays small in the BLA regime, which fails where |Z|≈0 ⇒ |z|≈|δz|), marching the deep pixels to the
reference end and capping them at ~919k so the dendrites vanished. A rebase check at the BLA landing
restored the full range (and sharpened the deep-center detail of 11–13); `--selftest` goldens stayed
byte-identical. Broader lesson: the app's fixed-cycle smooth coloring aliases at extreme depth; an
auto-normalize / adaptive-cycle coloring mode for the live view is the general fix (TODO.md).

## Reading the comparison

- **Compare structure, not color.** Palettes and smooth-coloring transfer curves differ by
  design; what must match is feature placement — spiral arm counts and chirality, escape-boundary
  shape, minibrot positions, filament topology.
- **Framing:** Fractadyne's magnification is referenced to a 3-unit-high view; KF's zoom is
  referenced to its own (wider) home frame. The same zoom value may therefore frame slightly
  wider in KF — the centered feature and its structure must still correspond exactly.
- **Iterations are pinned** to the same value in both apps (auto-scaling off), so interior/exterior
  classification near the cap is comparable.
- Locations 07–10 exercise Fractadyne's floatexp + BLA pipeline (KF covers the same range with
  its series approximation + long-double/floatexp stages) — these are the deep-zoom
  correctness anchors; 09 and 10 are far beyond double-exponent range in both apps.
