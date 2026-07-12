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

Net: clean, arm-for-arm F3 matches at **18 of the 20 locations** — from 1× to **4.60e1105×**, over a
thousand orders of magnitude of zoom. The former gap at **07** (1e30×) is now closed: its old shared
34-digit seahorse center was too coarse for a reproducible match there — F3 rounded it below
Fractadyne's bignum reference onto different sub-structure — so it was **replaced by a user Fraktaler-3
save** (`me30.exr`, center −1.1788…, 43-digit) that matches arm-for-arm (both apps render the same
diagonal dendrite filament with a central pinch and spiral-armed clusters). The center-precision lesson
still stands — the plainest proof being that 08 (83-digit center), 09 (526-digit), and 10 (1141-digit)
all render and match *far* deeper. The `.f3.toml` / `.kfr` params are written for all twenty, and
Fractadyne renders all twenty.

**Two exceptions — 14 (1.2e148×) and 15 (3.7e163×) are GLITCH-LIMITED.** Both are rendered with
multi-reference glitch correction **disabled**, because it goes pathologically slow (>1 h,
uninterruptible GPU dispatch) at their deep-interior "dark dendrite core" pixels (TODO.md → Open bugs).
Without it, the reference-orbit dips that these dip-carrying dives pass through (~1e-71 every 4383
iterations) leave a large fraction of interior pixels as **uncorrected perturbation glitches — speckle
noise**. Their overall structure and boundary placement match F3, but the interior *detail* does not:
these are structure-placement matches, not clean detail matches, pending the glitch-correction fix.
(v0.2.6's extended-range orbit fix turned these from a uniform-interior *blank* into visible-but-glitchy
structure — an improvement, not a clean render.)

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
