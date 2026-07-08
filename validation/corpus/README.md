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
| `renders/<slug>-fraktaler.png` | **You produce these** — the catalog shows a placeholder until each exists. |
| `generate_corpus.py` | Regenerates the `.kfr` files, the Fractadyne renders, and `catalog.html` from `locations.toml`. |

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

## Producing the Kalles Fraktaler side

1. In **Kalles Fraktaler 2**: *File → Open* the location's `.kfr`.
2. Set the image size to **1280×720** (Image → Image size). Iterations are already in the file.
3. Render, then *File → Save PNG* as `renders/<slug>-fraktaler.png` (same slug as the `.kfr`).
4. Re-open `catalog.html` — the pair appears side by side.

**fraktaler-3** also opens `.kfr` files directly; render at the same size and save under the
same name.

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
