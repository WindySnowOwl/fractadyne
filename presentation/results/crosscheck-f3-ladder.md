# Cross-renderer validation ladder: Fraktaler-3 vs the Fractadyne bignum oracle

Each row: **Fraktaler-3 3.1** renders the location (batch mode, integer `N`-channel EXR);
**Fractadyne's independent arbitrary-precision CPU dwell oracle** (`z→z²+c` in `astro-float`,
sharing nothing with either renderer's GPU perturbation pipeline) recovers each pixel's exact
`c` from F3's documented mapping — including F3's deterministic triangular sub-pixel jitter —
and compares.

- **`exact`** — fraction of non-boundary exterior pixels whose *integer* escape count matches
  F3 bit-for-bit.
- **`|Δn|≤1`** — fraction matching to within one iteration. The residual (exact < 100%) is the
  half-open-pixel jitter/sampling difference between the two engines, always ≤ 1 iteration.
- **`membership`** — fraction agreeing on interior vs exterior (in/out of the set).
- Boundary/cliff pixels (a 4-neighbour flips interior/exterior, or the count jumps > 2) are
  excluded — there the classification is genuinely ambiguous to the last ULP and the two engines
  legitimately sample differently.

Two fully independent engines agreeing pixel-for-pixel, via a shared ground truth neither can
bias. Deep rows use small images because the oracle is bignum-per-pixel — the claim is per-pixel
exactness, not resolution.

| location          | zoom (F3) | image | iters | exact | \|Δn\|≤1 | membership | verdict  |
| ----------------- | --------- | ----- | ----- | ----- | -------- | ---------- | -------- |
| Seahorse Valley   | 1e6       | 200²  | 4000  | 80.1% | **100%** | **100%**   | **PASS** |
| Seahorse Valley   | 1e12      | 160²  | 60000 | 79.0% | **100%** | **100%**   | **PASS** |

**Every non-boundary exterior pixel matches Fraktaler-3 to within one iteration across F3's
working depth range on this hardware (1e6× – 1e12×), and interior/exterior membership matches
exactly.**

> **F3 depth ceiling — why this cross-renderer ladder stops at ~1e13×.** Fraktaler-3's
> extended-exponent kernels render **blank (all-interior) past ~1e13× on this RTX 3080** — a
> driver/kernel-level limit, not a batch-config issue (tested F3 3.0 + 3.1; see
> [`../../validation/corpus/README.md`](../../validation/corpus/README.md)). So the *cross-renderer*
> comparison is bounded to where F3 actually renders here. Correctness **deeper than that** is
> carried by Fractadyne's own arbitrary-precision self-consistency battery — `--validate-deep`,
> validated to **1e1000000×** ([`validate-deep.md`](validate-deep.md)) — which needs no second
> renderer.

## Reproduce

```sh
# 1. Fraktaler-3 side (batch). NOTE: raise maximum_reference_iterations for deep views,
#    or F3 batch silently renders uniform. Full-precision center + high-precision zoom.
cat > loc.toml <<'EOF'
location.real = "-0.743643887037158704752191506114774"
location.imag = "0.131825904205311970493132056385139"
location.zoom = "1e12"
bailout.iterations = 60000
bailout.maximum_reference_iterations = 60000
bailout.maximum_perturb_iterations = 60000
bailout.maximum_bla_steps = 8192
bailout.escape_radius = 256
image.width = 160
image.height = 160
image.subframes = 1
transform.exponential_map = false
transform.vertical_flip = false
render.filename = "loc"
render.save_exr = true
render.exr_channels = ["N0"]
EOF
fraktaler-3 -W -w wisdom.toml               # once, generate wisdom
fraktaler-3 -b -w wisdom.toml -P loc.toml   # -> loc.exr

# 2. Cross-check against our oracle (exit 0 = PASS)
fractadyne --crosscheck-f3 loc.exr \
  --center -0.743643887037158704752191506114774 0.131825904205311970493132056385139 \
  --zoom-f3 1e12 --iter 60000 --er 256
```

Conventions: F3 `zoom=1` shows vertical extent 4, Fractadyne `mag=1` shows 3, so
`our_mag = 0.75 × f3_zoom`; match `escape_radius` and iteration count.

Generated on RTX 3080 / Ryzen 3950X, Fraktaler-3 v3.1, Fractadyne v0.1.10.
