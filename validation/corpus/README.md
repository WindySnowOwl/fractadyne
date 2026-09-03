# Fractadyne ↔ Kalles Fraktaler comparison corpus

Thirty-nine reference locations — from the full-set overview down to the deepest matched
pair at **6.13e1105×**, plus one EXTREME-tier row at **5.63e18003×** — rendered from the
**exact same center, magnification, and iteration count**, for side-by-side structural
comparison. Locations 11–20 are user-saved Fraktaler-3 finds (2.27e124× → 1.58e1008×), imported
verbatim from their `.exr` headers; later additions (21–38) extend the corpus into the
extreme-depth regime, and 39 (a user location reached with the log-space Misiurewicz solver)
pins the e18000-class regime. Rows marked `extreme = true` in `locations.toml` take a long time to build a reference
(row 39: ~24 min accelerated, ~2 h standard), so `--check` and full regeneration skip them
unless asked (`--extreme`, or name them with `--only`). Row 39 has a Fraktaler-3 cross-render
(`generate_fraktaler.py --only 39`, 592 s) and matches arm-for-arm. Open **[catalog.html](catalog.html)** to view the pairs with each
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
python validation/corpus/generate_corpus.py --check            # every non-extreme location
python validation/corpus/generate_corpus.py --check --extreme  # include EXTREME-tier rows
python validation/corpus/generate_corpus.py --only 39          # (re)generate one row only
python validation/corpus/generate_corpus.py --check --only 14,15
```

It re-renders each location to a temp file and compares it **pixel-for-pixel** against the committed
`renders/<slug>-fractadyne.png`. The renderer is deterministic (same pipeline ⇒ byte-identical pixels,
like the `--selftest` goldens), so an unchanged renderer prints `20/20 MATCH`; any `CHANGED` location
must be eyeballed against its F3 reference (regenerate, then open `catalog.html`) before it is
re-committed. The check exits non-zero on any change and never modifies the committed renders or
catalog. It is intentionally **not** part of the fast `--selftest` — a full 20-location re-render takes
minutes (the deep locations dominate), so it is an on-demand gate.

> ### ✅ Fixed 2026-08-14: this gate was RED because the renders were not hermetic
>
> `--check` reported `0/3 MATCH` on the first three locations (maxΔ 169, meanΔ 27–47), **including
> `01-home`** — the shallowest, simplest view in the set. It was never a rendering regression:
>
> - **The escape field was unchanged.** Bucketing every pixel by its old colour showed the new
>   colours clustering to a spread of ~1/255 with an identical interior fraction (0.0933 vs
>   0.0933) — a one-to-one recolour, not different mathematics.
> - **It was not a code change.** The `--selftest` goldens were blessed *earlier* (2026-07-04) than
>   those renders (07-12) and still pass; and the baseline-era commit, rebuilt, reproduced the same
>   ~28 mean delta against its own baseline. The July binary could not reproduce the July render,
>   so the variable was the environment.
> - **The cause: corpus renders inherited the developer's live session.** The old `stage_session`
>   pinned ~17 keys and let every other one ride along — `cycle`/`offset` (palette phase!),
>   `log_palette`, `normalize_live`, `use_binary`, `use_duotone`, `de_strength`, `stripe_freq`.
>   Exactly the trap the generator's own docstring warned about for colouring, only half-fixed.
>
> **Now hermetic by construction**: renders run against the committed `session-template.toml`,
> copied into a throwaway `FRACTADYNE_CONFIG_DIR`. Your own session is never read or written.
> Verified by rendering `01-home` with a deliberately hostile live session (`cycle 3.5`,
> `offset 0.77`, `de = true`, `use_binary = true`, `stripe`) — **maxΔ 0**. The app also logs which
> session it used (`[fd-start] session: <path> — loaded`), and the generator fails loudly if the
> staged one was not loaded, so a template gone stale against the schema cannot silently render the
> corpus with defaults. The baselines were then re-blessed deliberately and `--check` is **20/20**.
>
> ⚠**Compare pixels, never file bytes.** `--render` embeds metadata, so identical images produce
> different sha256s — four runs, four hashes, maxΔ 0. `--check` decodes and compares RGB; any new
> determinism probe must do the same.

## The F3 references are for *visual* comparison, not pixel scoring

Worth stating because it is an easy and expensive mistake: `renders/<slug>-fraktaler.png` carries
**Fraktaler-3's own colouring**, so diffing it against ours measures palette, not precision. Measured
mean delta is **189/255 on `01-home`**, where precision cannot matter at all, and 125 averaged over
all twenty. The pairs are for structural, side-by-side inspection in `catalog.html` — which is what
the corpus was built for and what "matches F3" refers to throughout the docs.

A genuinely numeric F3 comparison needs raw escape data rather than images. Only three locations have
a non-empty F3 `.exr` (`diag/fraktaler/locations/`: `me30`, `me141`, `me1007`); the other eight of
locations 11–20 are header-only, which is how their parameters were imported. Re-colouring those
three through our palette would give a real numeric cross-check at three depths.

Renders go through `--render` with the location on the command line — exact center strings +
magnification and its verbatim iteration count (auto-scale off; `--render` honors an explicit
count, unlike the tour renderer, which forces auto-iteration and silently re-caps it) — and
everything else from the committed `session-template.toml`, copied into a throwaway
`FRACTADYNE_CONFIG_DIR`. That file pins every image-affecting setting explicitly (Ember palette
with its exact cycle/offset, smooth iteration, no DE/lighting/HUD, SA and glitch correction off),
so a render depends on the exe, the command line, and that file — and on nothing about the machine
it runs on. Your own session is neither read nor written.

## Producing the Fraktaler side

Automated, using the Fraktaler-3 3.1 binary vendored at `diag/fraktaler/`:

```
python validation/corpus/generate_fraktaler.py               # render everything F3 can
python validation/corpus/generate_fraktaler.py --max-log10 8 # only run F3 up to ~1e8x
```

It writes an F3 batch param (`locations/<slug>.f3.toml`) for **every** location and renders each
to `renders/<slug>-fraktaler.png` (1280×720; our magnification equals F3's zoom as of v0.2.21, so
`f3_zoom = 10^mag_log10`), detecting and skipping blank output.

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
arm-for-arm matches verified all the way to **6.13e1105×** (location 10, a 1141-digit center; ~4 min
to render). Two further things have to hold for a clean cross-app match: the iteration cap must be
high enough for the depth (F3's reference otherwise truncates and blanks), and the **center must carry
enough digits for the depth *plus* margin** for F3's internal reference rounding, which is coarser
than Fractadyne's full-precision bignum reference.

Net: clean, arm-for-arm F3 matches at **all 20 locations** — from 1× to **6.13e1105×**, over a
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
- **Zoom:** Fractadyne's magnification now **equals** the Kalles Fraktaler / Fraktaler-3 zoom (both
  reference a 4-unit vertical extent at value 1; aligned in v0.2.21, so the same number frames the
  same view in either app) — a single zoom scale across the two tools.
- **Iterations are pinned** to the same value in both apps (auto-scaling off), so interior/exterior
  classification near the cap is comparable.
- Locations 07–10 exercise Fractadyne's floatexp + BLA pipeline (KF covers the same range with
  its series approximation + long-double/floatexp stages) — these are the deep-zoom
  correctness anchors; 09 and 10 are far beyond double-exponent range in both apps.
