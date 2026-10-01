# Built-in power and fold families

Status: phases 1–4 built, phase 5 run on the RX 6800 XT for 1–3, 2026-10-01 (§5.1); phase 4 is
not yet run on it. Step 3 of "ship a formula collection, read Fractint's formulas, add the
families deep zoomers use" (the user: "proceed in order 1-3 with appropriate tests").

## 1. Goal

Built-in families, each with the hand-written deep-zoom path the existing ones have (CPU step,
bignum reference, GPU direct and perturbation modes, distance estimate), and the accelerations the
Mandelbrot set has where the mathematics allows them (series approximation, bilinear approximation):

| Family | Powers | Fold | Holomorphic |
|---|---|---|---|
| Multibrot | 6, 7, 8 (3–5 exist) | none | yes |
| Burning Ship | 3, 4, 5 (2 exists) | `\|x\| + i\|y\|` BEFORE the power | no |
| Tricorn | 3, 4, 5 (2 exists) | `conj` before the power | anti |
| Celtic | 3, 4, 5 (2 exists) | `\|Re\|` AFTER the power | no |
| Buffalo | 3, 4, 5 (2 exists) | `\|Re\| + i\|Im\|` after the power | no |

Fifteen families. Each is written today as a custom formula (the collection ships several) and
deep-zooms through the generated path, but without series approximation, BLA, distance estimate or
the nucleus finder, and with a module compile on first use.

### Non-goals

- A free power parameter. Nothing in the pipeline has a power slot: the uniform has no spare row,
  and every cache key, view file and session keys on the family id or its name. One id per (shape,
  power) needs none of that, and the shader reads the shape and power off the id.
- Powers past 8 (single-precision range, §4.4).
- Phoenix or Newton at other powers.

## 2. Names, ids, menus

Ids 10–24, before `Custom` (1000), `formula_id == row == discriminant` as now. Names are the stable
tokens views, sessions, the CLI and tours use: "Multibrot 6" … "Multibrot 8", "Burning Ship 3" …
"Buffalo 5" (the existing "Burning Ship" is power 2 and keeps its name). The Fractal menu groups
the powers of a family into a submenu (25 rows flat would scroll).

An older build opening a view of a new family ignores the unknown name and shows the family it
had: the view writes `format_version = 2`, as a custom view does, so that build warns instead.

## 3. Assumptions to validate

| # | Assumption | How | Gate |
|---|---|---|---|
| B1 | A generic square-and-multiply direct step (the IR's `PowI` chain, MSB first, with the folds as the IR orders them) renders bit for bit as the generated module of the same IR program | self-test "generated X = built-in", extended to the new ids | 0 px differ |
| B2 | A generic binomial (Horner in δ) perturbation in df32 and floatexp follows the CPU interpreter as closely as the explicit Multibrot 3–5 arms do | "generated X perturbation = built-in" twins + per-pixel CPU oracle | within the existing twins' bounds |
| B3 | The folds of power ≥ 3 perturb as the IR rule table says (diffabs on Z and δ before the power for Burning Ship; on Z^d and δ(z^d) after it for Celtic and Buffalo) | abs-family cases: perturbation vs direct at 1e5×, floatexp vs df32 at 1e10×, finite at 1e35× | as the power-2 families |
| B4 | Powers up to 8 stay inside f32 range at escape with a per-family bailout and reference threshold | escape-step values measured on the CPU at the radii chosen; GPU smooth values finite | 0 non-finite |
| B5 | The core series approximation is generic in d (its binomials come from `cpow_bf`) | "multibrot-sa" extended to Multibrot 6–8 at 1e7×: SA on vs off | within that check's bound |
| B6 | BLA for z^d + c: A = d·Z^(d−1), B = 1, radius from the second-order term | BLA on vs off, per family, at 1e30× and 1e100× | within the Mandelbrot BLA gate |
| B7 | BLA for the folds as 2×2 real Jacobians, radius from the distance to the fold lines (`min(\|X\|, \|Y\|)`) and the operator norm | as B6, on the fold families old and new | as B6 |

## 4. Design

### 4.1 One descriptor

`fractadyne_core::family(id) -> Option<(Shape, u32)>` for ids 10–24 (`Shape` = Multibrot,
BurningShip, Tricorn, Celtic, Buffalo), and its WGSL twin `fam_shape(f)` / `fam_power(f)`. Every new
code path is written once, generic in (shape, d); the existing families keep their own arms
untouched, so their output does not move by a bit.

### 4.2 CPU

- `fractal.rs`: one `PowerFamily { shape, d }` implementing the `Field`-generic step, the power by
  the IR interpreter's exact chain (so f64, bignum and the IR agree bit for bit).
- `ir::builtin_step` for the new ids (fold ops + `PowI` + `C`), so every "generated = built-in" and
  "same as built-in" test covers them.
- Series approximation: one arm per Multibrot power (the recurrence is generic).
- `formula_power` → d for Multibrot 6–8 (nucleus and Misiurewicz solvers are generic in d);
  `perturb_orbit_length`'s binomial table extended to 8.

### 4.3 GPU

Generic blocks, entered only for ids ≥ 10:

- direct: fold, `z^d` by square-and-multiply (`c_sqr`/`c_mul`, the generated module's helpers), fold,
  `+ c`; derivative `d·z^(d−1)` for Multibrot (distance estimate);
- perturbation, df32 (mode 0) and floatexp (mode 2): `δ' = Σ C(d,k)·Z^(d−k)·δ^k` in Horner form, with
  the reference powers `Z^0..Z^(d−1)` formed once per step; folds by `diffabs` as in §3 B3;
- the resumable chunk passes for Multibrot 6–8 (holomorphic, like 3–5); the fold families stay
  out of chunk scope, as the power-2 ones are.

### 4.4 Range

At escape a pixel's `|z|² ≤ (R^d + |c|)²` must stay below f32's 3.4e38: with R = 256 that holds to
d = 7, so Multibrot 8 escapes at R = 128. The reference keeps its first sample past `|Z|² > 1e12`,
up to `(1e6)^d`, which overflows f32 from d = 7: the high powers stop the reference at `|Z|² > 1e9`.
Both are per-family helpers that return today's values for every existing id.

### 4.5 Bilinear approximation

Today Mandelbrot only (`bla_level0_node`: A = 2Z, B = 1). Holomorphic d: A = d·Z^(d−1), radius from
`|C(d,2)·Z^(d−2)|` against `|A|` (the quadratic term's share). Folds: a 2×2 real A per step, its
operator norm in the merge rule, the radius shrunk to the distance from the fold lines. The table
layout grows for 2×2 (four reals for A, B stays the identity for the `+ c` families). Built as its
own phase: it changes the existing fold families' rendering (within tolerance), so it gets its own
gates and goldens review.

## 5. Phases and gates

| Phase | Content | Gate |
|---|---|---|
| **1** | Ids, descriptor, CPU step, IR steps, app rows (grouped menu, calibration, help), GPU direct + perturbation (modes 0, 2) + distance estimate; range helpers | B1, B2, B3, B4; existing goldens 26/26 unchanged; new overview goldens; app/core/gpu tests; uitest |
| **2** | Series approximation and resumable chunk passes for Multibrot 6–8 | B5; chunked = single pass, bit for bit |
| **3** | BLA for Multibrot d (and 3–5) | B6 |
| **4** | 2×2 BLA for the fold families | B7 |
| **5** | The RX 6800 XT (field agent; the user's go-ahead first) | self-test and goldens as on the 3080 |

### 5.1 Results

- **Phase 1** (RTX 3080).
  - *B1, CPU.* `fractal.rs` `PowerFamily` is each family's IR program bit for bit: f64 orbits
    (51,000–101,000 points a family, parameter plane and Julia starts) and bignum references at
    128 and 320 bits; and the parser's spellings (`abs(z)^3 + c`, `conj(z)^4 + c`,
    `abs(real(z^5)) + flip(imag(z^5)) + c`, `abs(z^3) + c`) are the built-ins bit for bit. Tricorn
    conjugates AFTER the power on the CPU and before it in the IR: the same bits, as the IR's signs
    are flags beside magnitudes. Found on the way: `Formula::escape_degree` returned
    `exp(ln 7)` = 6.999999999999999 for one phase of degree 7 — exact now for phases of one degree.
  - *B1, GPU.* The direct render is the generated module's bit for bit for every power to 6. From 7
    the two part by POLICY (the module tames an escape past 1e15; Multibrot 8 escapes at 128), and
    leaving those pixels out by a CPU orbit failed on chaotic ones; Multibrot 7 and 8 are judged
    against the CPU with their own policy instead: 0.4% and 0.3% of pixels disagree, none of the
    escapes within 20 steps that are stable under c ± 1e-5 (power 8 grows an f32 error ~1,000× a
    step, so an orbit landing by the bailout escapes a step apart).
  - *B2.* The perturbation was first a binomial in Horner form with a table of Z's powers built by
    d − 2 products: 1.6–2.3× further from the CPU than the generated module for Multibrot 6–8. It
    now follows the power's own chain, one rule per link, as `ir::perturb` derives it — equal
    medians (Multibrot 6: 0.00089 against 0.00087). The twins check's question is reversed for these
    families (the built-in under test, "no worse than the generated module"), and each side is judged
    against the CPU with ITS policy: by the module's, the more exact built-in looked 1.6–3.6× worse.
  - *B3.* Every family at 1e5×: the perturbation against the CPU's f64 orbit (mean |Δ| under 0.0002
    iterations, thousands of pixels each), floatexp against df32 at 1e10×, finite at 1e35×. The
    direct path was the first comparison and is not usable here: its c is f32-quantised (the
    compiler folds df32; two ulps a pixel at |c| ≈ 1), which moved Multibrot 8's mean to 0.89.
    Views come from `family_view`: rays from 0 at 3.75° past multiples of 7.5° (never a symmetry
    axis: Tricorn 3's real and imaginary axes end in parabolic points, every pixel steep), the first
    with 15% smooth escaping samples one check pixel apart. Mutation: the Celtic fold dropped from
    the df32 step → mean |Δ| 48 / 6.9 / 5.1 against the CPU, twins 2 right against 32,010.
  - *B4.* Multibrot 8's escape at 128 and the d ≥ 7 reference stop at 1e9 (`ref_escape2`, shared by
    every walk that measures a reference, and by the custom formulas' reference by escape degree).
  - *Gates held.* Self-test 386/386 (296 + 90), goldens 26/26 unchanged on the blessed card, plus
    five new overviews (one per shape; all 31 pass).
  - *Cost factors* (`ceilings.toml`, method recorded there). The all-interior zoomtest the existing
    rows came from gave one or two readings a family in direct mode and none in df32, so: one
    timed 1080p render per family, three times, every pixel interior — direct at `--ss 1` (a
    glitch-corrected export never supersamples, so an unpinned Mandelbrot baseline ran at 2× and
    flattered itself 1.28×), df32 in Julia mode (no series approximation for any family). Controls
    against the existing rows: direct 1.14–1.21× low, df32 high, hence margins ×1.25 / ×1.1.
  - *A finding beyond this step.* Per step, a generated custom module outruns the fixed shader for
    EVERY family, old ones included (all-interior Julia views, no SA/BLA): df32 at 1e9× Mandelbrot
    57 vs `z² + c` 110 Gsteps/s, Multibrot 5 35 vs 67, Burning Ship 3 50 vs 68; floatexp at 1e40×
    17.7 vs 38, 8.5 vs 27, 11.7 vs 20.5; direct `z⁶ + c` 145 vs the built-in 72. Part is what the
    lean modules leave out (the derivative for distance estimation, glitch detection, aux
    colouring); part is presumably one fragment function carrying every family's and mode's live
    state. The built-in power families are therefore correct but no faster per step than typing them
    as custom formulas; their case rests on the fixed path's extras and on phases 2–4. Specialising
    the fixed shader per family (a module per family, as the custom path builds) looks like a
    broad per-step win for the existing families too — not started, a decision of its own.
- **Phase 2** (RTX 3080). `caps` counts Multibrot 6–8 as `z^d + c`: series approximation,
  resumable passes, the nucleus finder, exports without glitch correction.
  - *B5.* The walk was already generic in d; only its degree table stopped at 5. Its coefficients
    are now checked against the map itself, not only through δz at one δc: the exact δz at
    δc = h·iᵏ from bignum orbits, Fourier-summed, gives A, B and C to 1e-9 for d = 2..8
    (`series_coefficients_are_the_maps_taylor_coefficients`). That test exists because a walk that
    dropped the C(d,3)·A³ term passed everything else — SA's own criterion holds the cubic term
    under 2^-16 of the linear one, so a wrong C only lengthens skips (Multibrot 4: 49 → 67).
  - *The old SA check was vacuous.* "Multibrot 3–5 SA matches SA-off at 1e7×" ran at (0.2, 0.1),
    every pixel interior. At boundary views SA on and off part on thousands of pixels for every
    power, old ones included (4,555 for Multibrot 3), nearly all steep; against the CPU's f64 orbit,
    at the pixels its own neighbours agree on, they are equally right (SA on / off wrong at 182 /
    190, 81 / 76, 242 / 226, 31 / 32, 0 / 0, 3 / 3 for d = 3..8). The boundary rows now ask that
    question; the interior rows stay.
  - *A seed past the bailout — a black frame in a released family.* The walk stops at the
    reference's escape (|Z|² > 1e12), so a skip could land two samples before the end with |Z| far
    past 256; the shader first tests the seeded pixel a step later, where |z|² ≈ |Z|^(2d)
    overflows f32 from d = 4: −∞ everywhere, all "interior". Multibrot 5 at 1e40× (skip 4,466 of a
    4,468-sample reference) and Multibrot 6 at 1e45× rendered black. Seeds are now bounded by
    |Z_n|² ≤ 2^(120/d) (`sa_seed_max2`), which for d ≤ 3 sits above the reference's own stop, so
    Mandelbrot and Multibrot 3 skip exactly as before (the MPFR twin mirrors it;
    `the_sa_walk_is_backend_identical` green on the GNU build).
  - *Chunked = single pass, bit for bit,* in direct, df32 (incl. the rebase storm) and floatexp at
    1e30× on deep points bisected in bignum: 48,390 pixels escaping over 7,474 distinct counts,
    SA seeding at 1,920 (Multibrot 6). ⚠The first bisected points left the set through the main
    body, whose boundary is neutral: every neighbour escaped on step 1999 or 2000 — a view that
    agrees trivially. The fixtures are the first rays whose 1e-30 neighbours spread ≥ 300 steps.
  - *Glitch correction* (the audit, 800×450 at 1e30×): it changed 0.016–0.027% of pixels and, of
    24 sampled per family, made none right that was wrong (22–24 sub-pixel sensitive, the rest
    right either way) — Multibrot 3–5's finding. Off for 6–8, as for them.
  - *Live:* a 40-octave zoomtest 2^60 → 2^100 on Multibrot 6, 55.5 fps, worst frame 27.7 ms, every
    dispatch chunked.
  - *Gates.* Core 170, app 679, self-test 413/413 (27 new), goldens 31/31 unchanged; the MPFR
    identity matrix now covers ids 0–24.
- **Phase 3** (RTX 3080). `caps.bla` for every `z^d + c` family (Multibrot 3–8 with Mandelbrot);
  `build_bla(…, d)`: level 0 is A = d·Z^(d−1), r = 2·eps·|Z|/(d−1) (d = 2 keeps its exact bytes);
  merging and the shader's traversal were already power-agnostic. The triangle-inequality BLA
  aggregate takes the degree too.
  - *B6.* Core: at an interior reference the skips reproduce the exact perturbation for d = 3..8
    (rel. err < 1e-3 while skipping ≥ 3/4 of the steps); at a deep chaotic point per power,
    `bla_iterate_power` matches plain perturbation where plain perturbation matches the bignum
    count. ⚠Unjudged elsewhere: one reference and no rebasing glitches (Multibrot 8, a δc of
    0.85e-30: truth 2271.8, plain 2299.4, BLA 2156.4). Mutation (A = d·Z^(d−2)) → both red.
    Self-test: BLA = no BLA at every smooth pixel of each power's deep chaotic view at 1e30×
    (7,714–29,203 escapers, 55,608–1,456,655 skips); split frames with BLA bit-identical.
  - *Cost* (end to end, 960×540, ss 2): Multibrot 6 inside the period-1590 minibrot at 1e30×,
    200k iterations — 1.27 s with BLA, 4.48 s without (3.47 s of it the SA walk, d = 6 costs it
    3 products a step). The deep chaotic view (reference escaping at ~2,560) — 4.35 s with, 2.87 s
    without: BLA saved 1.7% of the steps and its level search costs every step. Mandelbrot pays the
    same at its short escapers (corpus 07 at 1e30×: 64 vs 29 ms GPU), the trade-off its policy
    keeps for a nearby minibrot's interior; so does Multibrot now.
  - *Not a finding:* BLA forced onto floatexp at 1e10–1e17 (a mode the app never selects there) is
    wrong at sensitive pixels for Mandelbrot and Multibrot 6 alike.
  - *Gates.* Core 172, gpu 53, app 679, self-test 431/431, goldens 31/31, uitest 65/65; live
    zoomtest 2^60 → 2^100 on Multibrot 6: 55.6 fps, worst frame 30.8 ms.
- **Phase 4** (RTX 3080). The folds at every power (ids 4–7, 13–24) take a real 2×2 tree
  (`reference/bla_fold.rs`): level 0 is the step's linear map — Burning Ship `M(d·F^(d−1))·diag(sgn
  X, sgn Y)` (F = |X| + i|Y|; the power-2 ship folds |Im z²| after the square, the same map),
  Tricorn `M(d·Z̄^(d−1))·diag(1, −1)`, Celtic/Buffalo `diag(sgn Re W, [sgn Im W])·M(d·Z^(d−1))` —
  and its radius the lesser of the power's `2·eps·|Z|/(d−1)` and the fold's (`min(|X|, |Y|)`;
  `|Re W|` [and `|Im W|`] over `(1 + eps)·|d·Z^(d−1)|`). Merging divides by EXACT spectral norms (a
  Frobenius bound is √2 high on a rotation and compounds over levels). A node is the complex one's
  16 floats (matrices as f32 mantissas under one exponent), so the shader's traversal and the aux
  patching are shared; the shader applies `fe_mat` for a fold. The chunk pass never sees a fold
  tree (folds are not resumable). ⚠Three places assumed "a BLA view has a series to fall back on"
  (the walk started beside the build, the short-escaper rule, its live mirror) — true until the
  folds; each now asks `series_approximation`.
  - *B7, core.* Level 0 is each fold's linearisation to 1.5·eps in every direction at reference
    values in all four quadrants, for all 16; the fold radius binds next to a fold (y ≈ 3e-8; arg z
    = π/(2d) + 1e-7), holds inside, fails twice past it — ⚠a mutation that dropped the fold radius
    passed every other test, since away from an axis the power's radius is ~1e-6 and the smaller;
    the spectral norm is exact against brute force; at an ATTRACTING interior reference per family
    the skips reproduce the exact perturbation (< 1e-3 while skipping ≥ 3/4). ⚠"Bounded" is not
    interior: c = −1 − i is a repelling Burning Ship fixed point (every perturbation overflows), and
    −1 − 0.4i a chaotic bounded orbit.
  - *Deep views — what can be judged.* No finite precision is truth at a fold's chaotic boundary: a
    pixel wanders for hundreds of steps after its offset reaches O(1) (Burning Ship: f64
    perturbation followed the 192-bit difference to 1e-14 until step 300, then parted ×1.3 a step).
    And a pixel's fate can turn on the DIRECTION of an error far below a pixel: at Celtic's fixture
    BLA's state at step 97 was 4e-12 from plain perturbation's, a δc nudge moving plain's 7e-11
    changed nothing, and BLA's pixel escaped at 162 where plain's never did. So the core judges
    WHERE EACH SKIP LANDS — within 1e-4 of plain perturbation (the same walk, rebasing, no skips) at
    that iteration, up to the first rebase, over 25 δc per family — and the final counts only at 90%
    of the pixels stable under nudges; the self-test holds each fold row to its CHAOS FLOOR, the
    mismatches of the render without BLA shifted 0.001 px: BLA 95 / 1 / 2 / 1 against the shift's
    178 / 1 / 6 / 4 (Celtic, Burning Ship 3, 4, Celtic 5), 0 wherever the shift gave 0. Mutations:
    no fold radius → a landing 53% off; the shader's matrix transposed → thousands of mismatches in
    every fold row, the interior rows unmoved.
  - *Deep fixtures.* The first ones (bisection + a 300-step spread, as for the powers) were noise for
    ten families — not one of 25 pixels stable, which the app draws as noise too; the kept ones are
    the first rays with ≥ 15 of 25 stable (≥ 10 for Celtic, Celtic 3, Buffalo 3 and 5). Two test bugs
    on the way: a lane-summed orbit sample reads an extended-range dip as |z| ≈ 300 (`sample_xy`);
    `parse_bf_prec` may keep more bits than asked, which moved an escape by 665 steps between two
    "identical" inputs.
  - *Cost.* Inside Burning Ship's main body at 1e30×, 30,000 iterations (960×540): GPU 4 ms with the
    tree, 5,118 ms without; 0.97 s against 6.15 s end to end. At Tricorn 3's chaotic fixture (1e30×):
    2.74 s against 2.42 s — the short-escaper cost the `z^d + c` families pay too.
  - *Gates.* Core 178, gpu 53, app 679, self-test 463/463 (32 new), goldens 31/31, uitest 65/65;
    live zoomtest 2^60 → 2^100 on the Burning Ship fixture: 56.0 fps, worst frame 57 ms.
- **Phase 5** (RX 6800 XT, PLUTO; field battery `20261001-145757-j02k`, `0.3.0-beta.13` = `34d4ca9`).
  Self-test 430/431: the one failure is the known Windows/Radeon "a wider canvas contains the
  narrower one" (10 of 20,480 texels at 1e2×); every power-family check passes — SA no worse than
  SA-off (178/153, 68/65, 221/242, 32/33, 0/0, 2/1 wrong vs the CPU for d = 3..8), BLA = no BLA
  (0 mismatch, each power), seeds in range, the period-2 nuclei. Goldens 31/31 (cross-GPU
  tolerance, the five new included). Live-res, livetest, uitest (the dropdown whole at 1.0×),
  recordtest and screen pass; gputest's exit 1 is the expected compiler-folding report.
  - *Bench matrix:* no algorithmic drift; 15 new segments. Its logcheck FAILED: 25 direct-mode
    fractal segments, which build no reference, ran ~12 s without a liveness stamp and the
    watchdog called it a possible hang (the 3080 finishes the whole matrix in 16.6 s). Fixed by a
    breadcrumb per segment (`dd955b0`), verified on the 3080; not yet re-run on PLUTO.

## 6. Risks

- The silent fallbacks the inventory found (an unknown id renders as z⁵ in the chunk pass, as
  Mandelbrot in the chunk δ-step and in `step_gen`): every new path is entered by an explicit id test,
  and a test asserts each new id reaches its own arm (a family rendered as Mandelbrot would pass a
  "there is a picture" check).
- Fifteen goldens are large: one overview per family at the standard size, deep rows for the
  Multibrot powers only (as now).
- Calibration rows are measured, not guessed (a missing row prices as Mandelbrot).
