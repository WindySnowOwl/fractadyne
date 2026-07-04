# Fractadyne — a Rust/wgpu deep-zoom explorer: status, validation, and a request for feedback

Hi all. I've been building **Fractadyne**, a native fractal explorer (Rust + wgpu/egui), with
ultra-deep zoom and correctness as the two priorities. It's at a point where I'd value critical
feedback from this community — especially on **validation** and on one **deep-zoom performance
wall** I've characterized but not solved. This post summarizes what it does, how I've been
validating it (including cross-checks against **Fraktaler-3**), the known gaps, and specific
questions I'd love input on. Everything below is reproducible; commands and data are included.

---

## 1. What it is

- **Extreme deep zoom.** Arbitrary-precision center (`astro-float`), a bignum reference orbit
  on the CPU, and a GPU perturbation pipeline that switches by depth:
  **direct df32 → df32-δ perturbation → floatexp-δ perturbation** (df32 mantissa + `i32`
  exponent), so the deviation δz never runs out of `f32` exponent range. Zhuoran rebasing;
  depth bounded by coordinate precision + iteration budget, not a fixed wall — cross-checked against
  Fraktaler-3 to 1e300×, renders correctly far deeper (a ~1e1108× sample location, `scripts/deep-sample.fdn`),
  and extreme-depth arithmetic self-consistency validated to 1e1000000×.
- **Acceleration.** Order-3 **series approximation** (skips early iterations, validated to
  reproduce full iteration exactly) and **BLA** (bilinear approximation, Zhuoran/KF-style),
  both on the GPU; the slow bignum reference/SA/BLA build runs off the render thread.
- **Families.** Mandelbrot, Multibrot 3/4/5, Tricorn, Burning Ship, Celtic, Buffalo, Phoenix,
  Newton; Julia mode for any; dual linked Mandelbrot↔Julia view.
- **Rendering.** DE/slope relief lighting, distance-glow, several coloring methods
  (smooth, stripe/TIA, orbit trap, decomposition), custom gradient editor, tiled PNG/EXR export
  with reloadable metadata, multi-reference glitch correction on the **export** path.
- **Tooling.** Guided keyframe tours + headless movie export (restartable with `--resume`),
  `.fdn` shareable locations (hardened/fuzzed parser), an auto-zoom autopilot (hands-free dive with
  an adjustable depth limit; switches to a stepped dive at extreme depth), a standardized benchmark,
  and a layered validation harness (below).

Stack: Rust, `wgpu`/`egui`/`eframe` 0.31, `astro-float` (bignum), 9-crate workspace. Windows.

---

## 2. Validation methodology — the part I most want critiqued

The design goal is **correctness that a third party can check without trusting my code.** The
suite uses *no external data* — only exact mathematics or internal cross-checks — with one
separate, opt-in cross-*renderer* comparison against Fraktaler-3.

**Layers:**

1. **Exact-math core tests** (`cargo test -p fractadyne-core`) — perturbation/SA/BLA reproduce
   the exact bignum recurrence; minibrot nuclei Newton-solve to known constants; coordinate
   round-trips; etc.
2. **GPU ↔ bignum-oracle self-test** (`--selftest`) — the GPU pipeline is compared, pixel for
   pixel, against an independent arbitrary-precision **CPU dwell oracle** (`z→z²+c` in
   `astro-float`, sharing *nothing* with the GPU perturbation path), plus golden images. Result:
   **55/55 checks, 4/4 goldens.** (`results/selftest.txt`)
3. **Cross-renderer vs Fraktaler-3** (`--crosscheck-f3`) — see §3. F3's exact integer escape
   counts (raw EXR `N` channel) vs the same bignum oracle. Because the oracle is *also* what
   `--selftest` checks the GPU against, the results compose transitively:
   `our GPU ≈ oracle` (selftest) and `F3 ≈ oracle` (crosscheck) ⇒ `our GPU ≈ F3`, via a shared
   ground truth neither renderer can bias.
4. **Extreme-depth self-consistency** (`--validate-deep`) — a precision battery from
   **1e1000× to 1e1000000×**: independent recomputation, precision-headroom checks, and
   octave-scaling invariants where no external oracle can run. Result: **PASS** to 1e1000000×.
   (`results/validate-deep.md`)
5. **Externally-verifiable challenge coordinates** (`validation/catalog.toml`, reformatted in
   `catalog.md`) — every entry has a known answer (a hyperbolic-component center's
   exact period+nucleus, or a yes/no set membership) that anyone can confirm with any trusted
   renderer or by hand.

I'd genuinely like to know: **where is this methodology weakest?** My own honest answer is in §4.

---

## 3. Cross-check against Fraktaler-3 (deep)

I compare **Fraktaler-3 3.1** (batch mode, `N`-channel EXR) against my independent bignum oracle,
recovering each pixel's exact `c` from F3's documented mapping (including its deterministic
triangular sub-pixel jitter) and iterating `z→z²+c` at full precision. Boundary/cliff pixels
(where a 4-neighbour flips interior/exterior, or the count jumps >2) are excluded — there the
classification is genuinely ambiguous to the last ULP and the two engines legitimately sample
differently.

**Ladder** (full table + counts in `results/crosscheck-f3-ladder.md`):

| location          | zoom (F3) | image | iters | exact | \|Δn\|≤1 | membership | verdict  |
| ----------------- | --------- | ----- | ----- | ----- | -------- | ---------- | -------- |
| Seahorse Valley   | 1e6       | 200²  | 4000  | 80.1% | **100%** | **100%**   | **PASS** |
| Seahorse Valley   | 1e12      | 160²  | 60000 | 79.0% | **100%** | **100%**   | **PASS** |
| Seahorse Valley   | 1e28      | 120²  | 60000 | 77.8% | **100%** | **100%**   | **PASS** |
| Elephant / spiral | 1e100     | 96²   | 60000 | 79.6% | **100%** | **100%**   | **PASS** |
| Elephant / spiral | 1e300     | 56²   | 60000 | 80.0% | **100%** | **100%**   | **PASS** |

**Every non-boundary exterior pixel matches Fraktaler-3 to within one iteration at every depth
from 1e6× to 1e300×; interior/exterior membership matches exactly.** `exact` is bit-exact escape
count; the ~20% residual is the sub-pixel jitter/sampling difference between the two engines,
always ≤ 1 iteration. Deep rows use small images because the oracle is bignum-per-pixel — the
claim is per-pixel exactness, not resolution.

**Visual side-by-sides** (`images/cmp_*`): the same location in both renderers —
F3's default grayscale (exterior filaments) next to Fractadyne's Ember palette (interior +
relief lighting). Different coloring, **identical structure**, at 1e12×, 1e100×, and 1e300×.

> Note on conventions (for reproduction): F3 `zoom=1` shows vertical extent 4, Fractadyne
> `mag=1` shows 3, so `our_mag = 0.75 × f3_zoom`; match `escape_radius` and iterations. **A
> gotcha I hit:** F3 batch sets the center's MPFR precision from `location.zoom`, and its default
> `bailout.maximum_reference_iterations` is too low for deep views — without raising it, deep
> batch renders silently come out uniform. (Is there a cleaner way to drive deep F3 batch? — see §5.)

---

## 4. Known gaps & open problems (candid)

- **Deep floatexp *live* rendering is slow (~1–5 s/frame past ~1e28×).** This is the big one.
  I profiled it hard: the mode-2 (floatexp) iterate shader spends most of its time on **full
  floatexp steps that BLA can't skip** in filament/Misiurewicz fields. A `--refdiag` tool that
  samples reference orbit lengths across a view shows that in these deep views **every candidate
  reference escapes at ~6000–6500 iterations — there is no long/interior reference to pick or
  rebase onto** (`results/refdiag.txt`). Things I tried that did **not** help:
  reducing motion resolution (it's per-pixel spin, not total work); rebuilding the BLA more
  aggressively; longer/predictive reference selection; replacing the `log2`/`exp2` in the
  floatexp normalization with `frexp`/`ldexp` (bit-identical, only ~5%). Multi-reference (KF-style)
  I validated as *inapplicable to these particular views* — with no longer reference to rebase
  onto, it can't help the filament case. Offline export renders full detail fine (as slow as it
  needs); it's the interactive preview that suffers. **I'd love to know how KF/F3 keep deep
  filament/embedded-Julia views interactive — is the answer just "more/cheaper references," better
  BLA coverage in the rebased regime, occupancy tuning, or something I'm missing?**
- **Abs-family fold speckle.** Burning Ship / Celtic / Buffalo deep-zoom via a per-component
  `diffabs`; residual speckle at the folds needs multi-reference glitch correction in the live
  path (currently export-only).
- **Aux coloring at depth.** Stripe/TIA/orbit-trap need every iteration's `z`, so they can't use
  BLA → ~10× slower at depth. Inherent; wants a cheaper aux accumulation.
- **F3 cross-check depth.** Rigorous to the depths in the ladder; the oracle's bignum-per-pixel
  cost is the limiter for going deeper on larger images (not a correctness limit).
- **Missing features.** Histogram/equalized coloring (needs new GPU compute), layers/compositing,
  a formula DSL, 3D (Mandelbulb/box), L-systems/CA — all on the roadmap, none built.

---

## 5. Specific things I'd value feedback on

1. **Deep interactive performance** (§4, first bullet) — the filament/Misiurewicz case where no
   good reference exists. What's the state-of-the-art here?
2. **Validation methodology** (§2) — where is the oracle-transitive argument weakest? What checks
   would you trust that I'm not doing? Any coordinates you *know* the answer to that you'd like me
   to run through the catalog?
3. **Driving Fraktaler-3 batch for deep cross-checks** (§3 note) — is there a canonical way to set
   deep-batch precision/reference limits, so a third party can reproduce the ladder cleanly?
4. **Anything that looks wrong** in the images or numbers. Please try to break it.

Thanks for reading — I know this crowd sets a high bar, which is exactly why I'm posting. Happy to
share more detail, coordinates, or raw data on request.

*(Renders + data generated on an RTX 3080 / Ryzen 3950X. Fraktaler-3 v3.1.)*
