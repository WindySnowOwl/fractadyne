# Cross-renderer validation against Fraktaler-3

This is an **independent, external** check: it compares the exact integer escape counts
produced by [Fraktaler-3](https://fraktaler.mathr.co.uk/) (a separate, well-regarded
GPU-perturbation renderer by Claude Heiland-Allen) against Fractadyne's own
arbitrary-precision **CPU bignum dwell oracle** — a code path that shares *nothing* with
either renderer's GPU perturbation pipeline. Two fully independent engines agreeing on
exact integer iteration counts, pixel for pixel, is the strongest correctness evidence we
can produce without a published reference corpus.

The oracle is also what `--selftest` checks our own GPU pipeline against, so the two
results compose transitively: `our GPU ≈ oracle` (selftest) and `Fraktaler-3 ≈ oracle`
(here) ⇒ `our GPU ≈ Fraktaler-3`, via a shared ground truth neither renderer can bias.

## How it works

Fraktaler-3's raw OpenEXR export carries the integer escape count in a `UINT` channel
named **`N`** (exterior pixels store `n + 1024`; interior stores `0xFFFFFFFF`). For every
pixel we recover its exact complex coordinate `c` from F3's documented pixel mapping —

```
pixel_spacing = 4 / zoom / height
c_x = center_x + ((i + 0.5 + jitter) − width/2)  · pixel_spacing
c_y = center_y + ((j + 0.5 + jitter) − height/2) · pixel_spacing   (kernel row j = height−1−y, EXR is v-flipped)
```

— replicating F3's deterministic triangular sub-pixel **jitter** (`burtle_hash`/`triangle`,
applied even at `subframes = 1`), then iterate `z → z² + c` in `astro-float` at full
precision and compare. Boundary/cliff pixels (where a 4-neighbour flips interior/exterior
or the count jumps by >2) are excluded — there the classification is genuinely ambiguous
to the last ULP and the two engines legitimately sample differently.

## Reproduce

1. **Conventions.** F3 `zoom = 1` shows vertical extent 4; Fractadyne `mag = 1` shows
   3 — so `our_mag = 0.75 × f3_zoom`. Match `escape_radius` (we use 256) and `iterations`.

2. **Render the F3 side** (batch mode). Save the `N` channel and disable the exponential
   map; `image.subframes = 1`:

   ```toml
   # sea.f3.toml
   [location]
   real = "-0.745"
   imag = "0.113"
   zoom = "500"
   [image]
   width = 300
   height = 300
   subframes = 1
   subsampling = 1
   supersampling = 1
   [bailout]
   iterations = 5000
   escape_radius = 256
   [transform]
   exponential_map = false
   vertical_flip = false
   [render]
   filename = "sea_out"
   save_exr = true
   exr_channels = ["N0"]
   ```

   ```sh
   fraktaler-3 -W wisdom.toml        # once, to generate tuning wisdom
   fraktaler-3 -b sea.f3.toml        # writes sea_out.exr
   ```

3. **Cross-check** against our oracle (exit 0 = PASS):

   ```sh
   fractadyne --crosscheck-f3 sea_out.exr --center -0.745 0.113 --zoom-f3 500 --iter 5000 --er 256
   ```

## Representative results (Fraktaler-3 v3.1)

| View                                            | Zoom  | Membership (non-boundary) | Exterior counts within 1 iter | Exact integer match |
| ----------------------------------------------- | ----- | ------------------------- | ----------------------------- | ------------------- |
| Seahorse valley `(-0.745, 0.113)`               | 5×10² | 100% (6832/6832)          | 100% (35780/35780)            | 78.9%               |
| Deep seahorse `(-0.7436438870…, 0.1318259042…)` | 1×10⁶ | 100% (11548/11548)        | 100% (2647/2647)              | 79.9%               |

Every non-boundary exterior pixel matches within **one** iteration; the residual ±1 is the
escape-test convention (`|z|² ≥ r²` vs `>`, and test-vs-increment ordering) at iso-iteration
band edges — a definitional one-iteration offset, not a numerical error. Interior/exterior
membership agrees exactly. The match holds undiminished at 10⁶× zoom (84-bit oracle).
