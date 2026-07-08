# Fractadyne ↔ Kalles Fraktaler comparison corpus

Ten reference locations — from the full-set overview down to **4.6e1105×** — rendered by both
apps from the **exact same center, magnification, and iteration cap**, for side-by-side
structural comparison. Open **[catalog.html](catalog.html)** to view the pairs with each
location's full compute details.

## Layout

| Path | What |
|---|---|
| `locations.toml` | Canonical location list (full-precision centers, log10 magnification, iterations) — the single source of truth everything else derives from. |
| `locations/*.kfr` | Kalles Fraktaler 2 location files (fraktaler-3 reads them too). |
| `renders/<slug>-fractadyne.png` | Fractadyne renders (1280×720, 2× supersampling, Ember palette, smooth iteration). |
| `renders/<slug>-fraktaler.png` | Fraktaler-3 renders (1280×720; produced by `generate_fraktaler.py` where F3 works — see the limitation below). |
| `locations/*.f3.toml` | Fraktaler-3 batch parameter files (one per location), written for **every** location. |
| `generate_corpus.py` | Regenerates the `.kfr` files, the Fractadyne renders, and `catalog.html` from `locations.toml`. |
| `generate_fraktaler.py` | Runs the vendored Fraktaler-3 (`diag/fraktaler/`) in batch mode to produce the F3 renders + `.f3.toml` params. |

## Regenerating the Fractadyne side

```
cargo build --release -p fractadyne-app
python validation/corpus/generate_corpus.py            # everything
python validation/corpus/generate_corpus.py --skip-renders   # just .kfr + catalog
```

Renders go through the tour renderer (`--render-tour`) with a one-keyframe script per location,
so the framing comes from the exact `center + mag_log10` — independent of window geometry. The
session's iteration cap is staged per location (fixed iterations, auto-scale off, HUD off) and
your session file is restored afterwards.

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

### ⚠ Deep-render limitation on this machine

Fraktaler-3 renders correctly only in the **double-precision regime (through ~1e13×** — verified:
1e6, 1e10, 1e13 all produce full detail). Past that it switches to its **extended-exponent number
types** (softfloat / floatexp / doubleexp), which render **blank (all-interior)** here — confirmed on
both the GPU and forced-CPU paths, independent of iteration count, and **identically in F3 3.0 and
3.1** (so it is not a version regression) and on the 3080 alone (so it is not the second GPU). The
cause is F3's extended-type CUDA kernels not working on this machine's NVIDIA driver — an F3/driver
issue, not a Fractadyne one.

Net: of the corpus, **01–03** (≤ 1e6×) have real F3 renders; **04–10** (≥ 1e12×) blank and await a
working F3 setup — most likely an **NVIDIA driver update**, else a different machine or a
source build. The `.f3.toml` / `.kfr` params are written for all ten, so producing the deep side is
one command (`python validation/corpus/generate_fraktaler.py`) once F3 works there. Fractadyne
renders all ten; the shallow pairs confirm the coordinate + framing conventions match exactly.

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
