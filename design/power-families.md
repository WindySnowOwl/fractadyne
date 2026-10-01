# Built-in power and fold families

Status: design, 2026-10-01. Step 3 of "ship a formula collection, read Fractint's formulas, add the
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

## 6. Risks

- The silent fallbacks the inventory found (an unknown id renders as z⁵ in the chunk pass, as
  Mandelbrot in the chunk δ-step and in `step_gen`): every new path is entered by an explicit id test,
  and a test asserts each new id reaches its own arm (a family rendered as Mandelbrot would pass a
  "there is a picture" check).
- Fifteen goldens are large: one overview per family at the standard size, deep rows for the
  Multibrot powers only (as now).
- Calibration rows are measured, not guessed (a missing row prices as Mandelbrot).
