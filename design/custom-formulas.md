# Custom formulas — design

Status: **in progress** (2026-09-29): phases 0 and 1 done, results in §5.1. Supersedes the unbuilt
M6 sketch in `DESIGN.md` §8 where they differ. Evidence behind every claim is in §3 (measured on
this codebase) or cited.

## 1. Goal

Let a user render formulas that are not among the ten built-in families, without giving up what the
program is for: deep zoom. Concretely:

- **Any formula renders.** A user formula always renders at shallow depth, however exotic.
- **Deep zoom is automatic where the maths allows it**, and the program says plainly where it does not.
- **Existing formulas and files keep working**, and the ten built-ins render exactly as today.
- **A custom formula cannot take the machine down**: it gets the same dispatch ceiling, dead-man and
  watchdog protection as a built-in, calibrated to its own cost.

### Non-goals

- Adopting Ultra Fractal's full language (classes, arrays, `global:`, plug-ins, layers).
- User-written colouring algorithms (a separate project; the IR below does not preclude it).
- Pixel-perfect reproduction of every Fractint quirk (`rand` order, `whitesq`, `LastSqr`).
- Porting code from Kalles Fraktaler, Fraktaler-3, `et`, Imagina or any GPL/AGPL project (§10).

## 2. What exists today

- **Ten hand-written families.** Each family's step is written six times, once per numeric form: the
  bignum reference (`core::reference::step_bf`), the f64 overlay (`orbit_points`), three WGSL paths
  (direct df32, df32 perturbation, floatexp perturbation) and the SA recurrence (`ARCHITECTURE.md`).
- **One fixed shader.** `mandelbrot.wgsl` (3,037 lines) is compiled once from `include_str!`; the
  formula is a runtime uniform tested in 74 places.
- **Formula identity is a `u32`.** `FractalKind` is converted with `formula_id()` and never matched;
  capabilities are ~20 hard-coded id gates (`formula_id() <= 3` for SA/finders/resumable passes,
  `== 0` for BLA, `formula <= 3` in the export paths).
- **No formula parser.** The coordinate-expression parser in `core/src/bignum.rs` (complex, bignum,
  trig/log/exp tables, recursive descent) evaluates constants only.
- **KFR import ignores the formula.** `parse_kfr` reads Re/Im/Zoom/Iterations and the app forces
  Mandelbrot (`main.rs:9573`).
- **Per-formula cost factors** for the dispatch ceiling are measured on the RTX 3080 at all-interior
  views and compiled in (`validation/calibration/ceilings.toml`).

## 3. Assumptions, validated

| # | Assumption | How it was checked | Result |
|---|---|---|---|
| A1 | Fractint `.frm` is the de facto interchange format with the largest corpus | Survey of readers (Ultra Fractal, ChaosPro, Gnofract 4D, Iterated Dynamics) and corpora | Holds. Orgform: 29,348 entries, **20,181 distinct formula bodies** (counted) |
| A2 | Nothing in `.frm` or the UF corpus supplies deep-zoom forms | Survey | Holds. UF6 perturbation sections appear in 2 shipped and 3 public UF formulas; `.frm` has none |
| A3 | The Fraktaler-3 / Kalles Fraktaler hybrid opcode model covers the deep-zoom community's formulas and 8 of our 10 built-ins | `crates/fractadyne-core/tests/opcode_equivalence.rs`: the opcode program applied at every point of the core's own orbits (~120,000 steps) | **Bit for bit** for Mandelbrot, Multibrot 3/4/5, Tricorn, Burning Ship, Celtic, Buffalo. Phoenix and Newton are not expressible |
| A4 | The opcode perturbation rules (reimplemented from the published maths, incl. `diffabs`) are cancellation-free | Exact rational arithmetic vs the f64 perturbed update, 3,000 cases per program, deltas 1e-9…1e-14, including deltas straddling a fold | Worst error **1.0e-15–2.7e-15 relative** for all eight programs; naive f64 differencing: up to **178%** |
| A5 | A first `.frm` subset without the screen/random features covers most of the corpus | Scan of Orgform (excluding `whitesq scrnpix scrnmax rand lastsqr cosxx`) | **88.5%** of distinct bodies; adding `cosxx` (4.8%, = conj(cos)) makes it ~93%. `fn1..fn4` in 34.5%, `p1..p5` in 70%, `if` in 8.5% |
| A6 | A generated per-formula shader compiles fast enough to use | `crates/fractadyne-gpu/examples/shader_compile_bench.rs`, RTX 3080, Vulkan, cold driver cache | naga parse+validate ~10 ms. Today's all-formula module: iterate **~4.0–4.2 s**, resumable 0.32–0.36 s, resumable-floatexp 1.4–1.7 s. Specialised to one formula: iterate **0.9–1.6 s**, 0.29–0.32 s, 0.58–1.1 s (1.8–3.0 s for the three). **Too slow per keystroke; fine per "apply", cached after** |
| A7 | Real deep-zoom parameter files carry their formulas | Kalles Fraktaler's 88 example `.kfr` files | 74 carry `HybridFormula` strings (e.g. `0/1\|0,0,0,0,2,1,0;0,0,0,0,0,0,0;0`) |
| A8 | Automatic deep zoom is feasible only for a class of formulas | Published theory (§11) and the corpus classification | Orgform bodies: polynomial 10.3%, +abs/conj 5.7%, rational 15.9%, non-integer powers 7.7%, transcendental/`fn` 40.4%, branching 17.4% |
| A9 | Our licence forbids porting the reference implementations | `Cargo.toml`: `MIT OR Apache-2.0`; KF/F3/`et`/Imagina AGPL, Iterated Dynamics / formula-compiler GPL | Holds: reimplement from the published maths |

Not yet measured: compile times on the RX 6800 XT; UF6 `#z` semantics inside `perturbloop` (inferred
from shipped code, not documented).

## 4. Design

### 4.1 One intermediate representation

`fractadyne-core::ir` holds an **expression IR**: instructions in SSA form, each computing one complex
value from earlier ones. That makes it a linearised expression DAG, so a shared subexpression is computed once. Its values are
`z`, `c` (the pixel), the previous iterate (for Phoenix-like formulas), parameters, constants,
`+ − × ÷`, integer and complex powers, `sqr`, real scaling, `conj`, negation, `abs` per component,
real/imag parts, `|z|²`, and the elementary functions (`exp log sqrt sin cos tan` and hyperbolics;
inverses to follow with the `.frm` reader). A formula is one step program per **phase** (hybrid
lines interleave, iteration `n` runs phase `n mod len`). Still to come with the front ends:

- `init`: statements setting `z` (default `z = 0` for parameter-plane formulas);
- `bailout`: a predicate (default `|z|² ≤ R²`), or a convergence test for Newton-type formulas;
- parameter declarations with defaults and UI metadata (values are already an evaluation input).

Division and the elementary functions are `f64`-only for now: a reference orbit may depend on them only
once their bignum forms meet the same astro-float/MPFR bit-identity contract as the ring operations
(phase 5). `Program::bignum_evaluable` says which programs qualify.

Everything else is **generated from the IR**:

| Artefact | How |
|---|---|
| Bignum reference step | tree-walking interpreter over astro-float (and rug when enabled) |
| f64 overlay step, CPU oracle | the same interpreter over f64 |
| WGSL direct step (f32 / df32) | code generation, one expression per statement |
| WGSL perturbed step (df32, floatexp) | rule-table derivation (§4.4), where the IR is in the deep-zoom class |
| Derivatives (distance estimate, Newton zoom, BLA) | forward-mode duals over the IR |
| Capabilities | computed from the IR (§4.3) |

### 4.2 Front ends

1. **Hybrid / opcode formulas** (deep-zoom first class). Reads the Fraktaler-3 `[[formula]]` blocks
   (`abs_x abs_y neg_x neg_y power`, or `opcodes = "store sqr mul absx absy negx negy rot{deg} add"`)
   and Kalles Fraktaler `HybridFormula` strings; several lines interleave one per iteration (phases).
   Lowered to the IR. Also a simple in-app editor for these (the KF/F3 row-of-checkboxes UI).
2. **Fractint `.frm` subset.** `Name(sym) { init : step , bailout }`; complex-only variables;
   `p1..p5` → parameters; `fn1..fn4` → a user choice from Fractint's list, fixed at compile time;
   `if/elseif/else/endif`; real-part comparisons and non-short-circuit `&& ||` as in Fractint; `|x|` as
   the squared modulus; `cosxx`. Out of the first subset: `whitesq scrnpix scrnmax rand LastSqr`
   (reported as unsupported, not silently mis-rendered). Symmetry annotations are ignored (we do not
   force symmetry).
3. **Optional author-supplied perturbation**, in the model of Ultra Fractal 6: `perturbinit:` /
   `perturbloop:` statements updating `#dz` from the reference `#z`, `#dpixel` and parameters, with a
   `perturb` guard. Used when automatic derivation (§4.4) does not apply.
4. **Raw WGSL** (later, optional): a fixed hook signature validated by naga, shallow only unless the
   author also supplies the perturbed step.

### 4.3 Capabilities, not id ranges

Every formula — built-in or custom — carries a `FormulaCaps`:

| Capability | Built-ins today | Custom |
|---|---|---|
| `julia` | spec table | yes unless convergent |
| `perturbation` (df32 / floatexp) | all but Newton | IR in the deep-zoom class, or author-supplied `perturbloop` |
| `series_approximation` | 0–3 | integer-power holomorphic polynomials |
| `bla` | 0 | opcode class (2×2); polynomial holomorphic later |
| `resumable_passes` (chunked walk / export tiles) | 0–3 | any generated formula whose state fits the attachments |
| `distance_estimate` | 0–3, Phoenix | any formula with a derivative |
| `nucleus_finder` | 0–3 | integer-power polynomials |
| `feature_solvers` (Misiurewicz explorer, feature go-to, snap, autopilot target) | 0 | quadratic polynomials first |
| `export_glitch_correction` policy | Julia or id > 3 | as for abs-family built-ins |
| `convergent` | Newton | from the bailout kind |
| cost factors (direct, df32) | `ceilings.toml` | measured at compile time (§4.6) |

**Phase 0 replaced the id gates with these flags, byte-neutral** (`formula::caps`; the gates that
existed as id ranges: series approximation, BLA, resumable passes, the two finder rows, glitch
policy, convergent). `julia` and `perturbation` are still the app's `FractalSpec` flags, and
`distance_estimate` is still per-method; they move when custom formulas need them. The same
structure is what the live-render plan's W7 ("typed capabilities") needs; this design supplies its
formula half.

### 4.4 Deep zoom: what is derived automatically

Perturbation replaces `z` by reference `Z` plus small `δ` and iterates `δ` in low precision. The perturbed
step must avoid cancellation. For the IR it is derived by a **fixed rule table** over the tree (not
general symbolic algebra), with `W = Z + δ` (widened) and `B = Z` (reference):

- `P(f + g) = P(f) + P(g)`; `P(f·g) = P(f)·W(g) + B(f)·P(g)`; `P(fⁿ)` by the difference-of-powers
  sum; `P(sqr f) = (2·B(f) + P(f))·P(f)`;
- `P(|f|ₓ) = diffabs(B(fₓ), P(fₓ))`, and likewise for the imaginary part; negation and conjugation
  pass through;
- `P(1/f) = −P(f) / (B(f)·W(f))`; `P(exp f) = exp(B(f))·expm1(P(f))`; `P(log f) = log1p(P(f)/B(f))`;
  and so on for the elementary functions.

Tiers, by what the rest of the pipeline also needs:

| Class | Perturbation | Rebase / critical point | BLA | SA |
|---|---|---|---|---|
| **Opcode programs** | rule table | critical point 0, one reference per phase | 2×2 Jacobian from duals; radius from per-op bounds, **validated by us** | no |
| Holomorphic integer-power polynomials | rule table | critical points = roots of f′, found once in bignum | A, B from duals; radius from the second derivative bound | yes (general coefficient recurrence) |
| Rational / transcendental | rule table (or author `perturbloop`) | must be found; may need the formula conjugated so a critical point sits at 0 (the "perturbing Nova" lesson) | none | none |
| Branching bodies, non-integer powers | none (the reference and the pixel may take different branches) | — | — | — |

A formula outside the automatic tiers still renders; the status bar states the depth limit ("this
formula renders to 1e4× — no deep-zoom form"), as the program already does for other limits.

### 4.5 GPU code: a template with generated slots

Today's module compiles all ten formulas into every entry point (A6: ~4 s cold for `fs_iterate`).
The shader is restructured so every formula-specific piece sits behind **named functions** —
`step_direct`, `deriv_direct`, `step_pert_df32`, `step_pert_fe`, `deriv_pert`, `bla_step`, `power_f`,
`bailout`, `init_z` — and the dispatch over `iu.formula` happens only inside them. Then:

- **Built-ins** keep the fixed module exactly as compiled today (the functions dispatch on the id).
- **A custom formula** gets a generated module: the same template with those functions replaced by
  generated bodies and no built-in code at all. Measured specialisation cost: 0.9–1.6 s for the
  iterate pipeline, 1.8–3.0 s for all three iterate entry points (A6).
- **Compile policy:** a small direct-only preview pipeline while editing (one entry point); the full
  set compiled in the background on "apply"; a pipeline cache keyed by the IR's hash, persisted with
  wgpu's `PipelineCache` where the backend supports it, and the driver's own cache otherwise.
  **Measured after building it (phase 2):** the module `custom::build` generates (direct path only,
  perturbation paths cut) compiles its `fs_iterate` pipeline in **74–119 ms cold** on the RTX 3080,
  against 3.98–4.15 s for today's all-formula module in the same run. That is fast enough to recompile
  on every edit, so the preview needs no separate pipeline. The perturbation paths are the cost.
- The restructuring of the fixed module is its own **byte-neutral** step, gated by the goldens, the
  self-test's bit-exact checks and the F3 corpus, before any generated code exists.

### 4.6 Cost and safety

A custom formula's per-step cost is unknown, and the dispatch ceiling depends on it. On compile, the
program runs a short **no-escape calibration pass** (escape test disabled, fixed pixel count, wall-clock
timed after a warm-up pass — the method the `live-split` self-test uses) for the formula and for
Mandelbrot on the same card, and uses the ratio as the formula's factor (clamped ≥ 1, as today).
Until it has run, the formula uses a conservative default factor. The dead-man, the lethal-frame latch
and the fixed ceiling apply unchanged. Loops inside a formula's step are not allowed in the `.frm`
subset (Fractint has none), so a step's cost is bounded by its size.

### 4.7 Persistence and identity

- `.fdn`, sessions and PNG metadata store `fractal=Custom` plus the formula **source text** and its
  front end (`hybrid`, `frm`), parameters, and a hash of the normalised IR. Files written before this
  stay valid (`writer ⊆ reader`).
- The on-disk orbit cache keys on the IR hash instead of the numeric id for custom formulas.
- KFR import maps `FractalType`/`Power` to a built-in where one matches, and otherwise
  `HybridFormula` to an opcode program — instead of forcing Mandelbrot. F3 `.f3.toml` import reads the
  `[[formula]]` blocks.

### 4.8 User interface

First: a formula dialog with a text field, the front-end choice, parameter widgets from the formula's
declarations, compile errors mapped to lines (naga spans for generated WGSL are ours, not the user's —
errors are reported against the source text), and a capability line ("deep zoom: yes / to 1e4× only,
because …"). The hybrid editor is a grid of per-line checkboxes and a power, as in KF/F3. The fuller
editor of mockup `07` comes later.

## 5. Phases and gates

| Phase | Content | Gate (must be green before the next phase) |
|---|---|---|
| **0** | `FormulaCaps`; the ~20 id gates replaced | Byte-neutral: self-test all checks, goldens 19/19, F3 corpus, `cargo test` count never decreases |
| **1** | Expression IR; f64 and bignum interpreters; opcode programs lowered to IR | The eight opcode programs reproduce the built-ins through the IR bit for bit (A3 generalised); bignum IR orbit = `step_bf` orbit bit for bit |
| **2** | Shader restructured into formula functions (byte-neutral); generated direct (shallow) WGSL; pipeline cache; compile-time cost calibration; minimal formula dialog; `.fdn` persistence | Fixed module byte-neutral as in phase 0; a generated Mandelbrot/Burning Ship renders equal the built-in at shallow depth (bit for bit or within a stated tolerance, measured); GPU vs CPU interpreter equivalence over a formula set |
| **3** | `.frm` subset reader | Parses ≥ 88% of Orgform's distinct bodies (A5); rendered spot checks against the CPU interpreter; unsupported features reported, never mis-rendered |
| **4** | Deep zoom for opcode programs: generated df32/floatexp perturbation, per-phase references, rebasing, 2×2 BLA with our own radius validation; KFR/F3 formula import | Generated perturbation renders of the eight built-in programs vs the hand-written built-ins at the corpus depths (difference measured and bounded); perturbed vs CPU bignum per-pixel oracle at 1e30, 1e100, 1e300; BLA on vs off identical within the corpus tolerance |
| **5** | Polynomial / rational deep zoom (rule table beyond opcodes, critical points, SA for polynomials); UF6-style author `perturbloop` | Oracle checks as in phase 4 on a formula set with known critical points; Nova-type formula handled or refused explicitly |
| **6** | Raw WGSL tier (optional) | naga validation, shallow equivalence |

Built-ins stay on their hand-written paths throughout. Switching a built-in to generated code is a
separate, later decision, and only if phase 4 shows the generated render is identical.

### 5.1 Results

- **Phase 0** (`1bcf16e`). `formula::caps` answers exactly as the replaced id expressions for ids
  0..1000, out-of-range ids included (unit test). `cargo test` 923 passed, 0 failed; self-test
  203/203, goldens 19/19.
- **Phase 1**. Gates in `crates/fractadyne-core/src/ir/tests.rs`, each asserting a count floor:
  - `f64`, 9 escape-time built-ins (eight opcode programs + Phoenix) vs `orbit_points`: 2,000 orbits
    each, 28,565–78,512 points per family, **all bit-identical**. Newton's step: 15,886 steps, bit-identical.
  - bignum vs `reference_orbit_t_in` (every sample, the length, and the full-precision tail): 9 families ×
    p = 64, 128, 320, 1088 × 17 cases, 1,000 iterations, **bit-identical in astro-float and in MPFR**
    (GNU toolchain, `--features rug`). Plus 5,000-step orbits: the real axis (chaotic) and a complex orbit
    spiralling into a fixed point with multiplier 0.995.
  - `PowI` reproduces the Multibrot 3/4/5 chains bit for bit; hybrid phases rotate per iteration
    (checked against the hand-written steps, `f64` and bignum).
  - **Controls.** Computing `Re z²` as `(x+y)(x−y)` turned 7 of the 11 tests then present red. The
    long-orbit test stayed green because on the real axis `y = 0` makes the two forms identical, so
    the complex long orbit was added; under the same mutation it goes red. Removing the sign fold (computing `(−a) + b` literally)
    keeps every gate green: astro-float's `(−a) + b` and `b − a` agree on these cases, so the fold is
    kept for cost (negation is free), not because identity needs it.
  - `cargo test` 935 passed, 0 failed (+12).
- **Phase 2, slice 1 (generation).** `fractadyne_gpu::custom::build` turns a formula into WGSL using the shader's own df32
  helpers and splices it into the fixed module at two marker comments (`@@CUSTOM_STEP`, `@@CUSTOM_CUT`).
  The fixed module gained comments only: self-test 203/203, goldens 19/19. All ten built-in steps,
  hybrids, parameters, every elementary function and the whole op set generate modules that naga
  validates; generated built-in steps are exactly the built-in helper calls (Mandelbrot `c_sqr` then
  `c_add`). Elementary functions run at `f32` precision for now (`Precision::F32`). The
  `escape_degree` of the IR sets smooth colouring's power, with 2 where no power law holds.
  `cargo test` 942 passed.
- **Phase 2, slice 2 (GPU equivalence).** `ExportRequest::custom` carries a generated module through
  the export paths. The self-test group `custom-formula` (8 checks; full run 211/211, goldens 19/19):
  - Generated Mandelbrot, Multibrot 3, Tricorn and Burning Ship vs the built-ins, direct mode:
    **0 pixels differ** (31,835–45,478 escaped pixels each). The first attempt differed by 1–2 ulps of the
    smooth value on 1,292–15,848 pixels. The cause was isolated by experiment: cutting the perturbation
    paths changes nothing (0 px), while the generated step differs. The difference was `log(power_f)` folded
    at compile time, where the fixed module computes it on the GPU. The generated power is therefore kept opaque
    to the compiler. Two further red gates showed what that takes. A condition the loop guard decides
    (`max_iter == 0`) is folded. So are equal arms (`select(2.0, 2.0, …)` for Mandelbrot).
  - GPU vs the CPU interpreter, at the shader's own pixel centres: a two-phase hybrid 0.76% and a
    parameterised quadratic 0.36% of pixels disagree (boundary chaos). None of the ~42,000 pixels that
    escape within 20 iterations disagree. Each f32-tier function evaluated once per pixel
    (`z + 8·(f(c) + 1 + 2i)`): 1 of 371,940 informative pixels. A sign error planted in `cos`
    turned that check red (75,838 px), as it should.
  - **Two rendering bugs found and fixed in the generated module** (the built-ins cannot reach either
    from their views): (1) an escaping step of a steep formula overflows f32 (`sin z + c` jumps to
    |z|² ≈ 1e50–1e158), and the smooth value of an infinite `z` is −∞, which the colour pass paints as
    interior. 3,044 pixels were affected; `custom_tame` now clamps a non-finite or ≥1e15 component
    (bit tests, since a compiler may assume floats are finite). (2) The smooth value
    `n + 1 − log(log₂|z|)/log d` is negative for an escape at n ≤ 2 or far past the bailout, which
    also reads as interior. It is now clamped at 0 through a third marker pair; every value ≥ 0 keeps its bits.
  - `sin z + c` iterated disagrees on 3.0% of pixels and cannot do better. The family expands by
    |cos z| ≈ cosh(Im z) per step, so f32 rounding reaches O(1) within about 10 iterations. "No
    disagreement among early escapes" is therefore not a bug detector for expansive formulas. That
    check's gate is 0 non-finite smooth values (the overflow guard).
- **Phase 3 core, brought forward** (`defa724`). A custom formula needs a text form before it can be typed
  or saved, so the expression core of the `.frm` reader came first: `ir::parse`, Fractint-style
  statements (`z = z^3 - p1*z + c`, temporaries, `;` comments, `|z|` as the squared modulus,
  p1–p5, 20 named functions). Typed built-ins reproduce the built-ins bit for bit in f64 and bignum.
  `init:`/`bailout:` sections, `if` and `fn1`–`fn4` are errors that name the feature. They remain
  phase 3, together with the corpus gate.
- **Phase 2, slice 3 (the app).** `FractalKind::Custom` (the last variant, id 1000, outside
  `ALL`, `supports_perturbation: false`, so direct mode comes from the existing mode selection). The
  formula lives on the app as `CustomFormula` (text, parameters, IR, shader). The live renderer caches
  the custom `fs_iterate` pair per shader key, and the shader key joins every cache keyed on the formula
  id: the iterate key, the settings hash, `view_key`, the minimap and the orbit overlay (which iterates
  the IR). Persistence: `.fdn`, exported images and the crash view carry `formula=` (one escaped line)
  and `formula_params=`, written only for Custom views, so built-in views are byte-identical. Sessions
  keep the formula. A view whose formula does not compile is reported, and the view is not switched.
  Tours refuse Custom for now. The dispatch ceiling prices a custom formula by
  `custom::cost_factor` (a static estimate that over-prices every measured built-in, tested), or by 3×
  before one is applied. UI: Fractal > Custom formula… and the toolbar picker open a dialog (live syntax
  check, parameters, examples, depth note), plus `--formula` / `--formula-params` on the command line.
  Checked by looking: a `--render` export, a `--shot` live window loading a Custom `.fdn` (the dual
  view's Julia panel included), and two new `--uitest` screens (47 steps, 0 fail; the one WARN,
  `live-floatexp-1e30`, predates this work). `cargo test` 953, self-test 212/212, goldens 19/19.
- **Still open in phase 2:** the compile-time cost calibration (§4.6) to replace the estimate; a
  persisted pipeline cache (compiles take ~0.1 s, so this can wait); distance estimation for custom
  formulas (forward-mode duals). Compile times on the RX 6800 XT are not yet measured.
- **The direct path's real depth limit (field report, measured 2026-09-30).** A custom formula
  bricked past ~2×10⁴× (`sin z + cos z·cos z + c`: 21×5-px bricks at 549,309×). The first
  explanation, single-precision elementary functions, was WRONG as the whole story. On the RTX 3080
  (Vulkan, NVIDIA 616.92) `--gputest` still shows `two_sum`/`quick_two_sum` folded by the compiler
  (df_add error 8.1e-8). The pixel's `c` is therefore single precision for EVERY formula. A linear
  formula with no functions, `z + 5e7·(c − c₀) + 40`, showed the same 17×4-px bricks at the same
  view: 338 distinct values in 48,400 pixels, against 17.5×4.4 predicted from one f32 step of `c`
  at −1.64+0.36i. Double-single elementary functions therefore cannot help on this hardware, and are
  dropped; AMD folds the multiply family, so it would not help there either. A precision check that
  would have gated them (`z + M·(f(c) − f(c₀))` over a 1e-6 view) failed 98.6% of 482,520 pixels
  on the f32 functions, as a control should. It was not kept, because no current GPU can pass it.
  The limit is where one f32 step of `c` spans a pixel: ~1e4–1e5× for any custom formula. Past it,
  only a perturbed step helps, which is how the built-ins get past 1e4× on the same card (phase 4).
  The dialog note and Help now state the measured limit.

## 6. Validation plan

- **Conformance corpora, never bundled:** a script fetches Orgform (Wayback copy), id-libraries and
  fract4d/formulas into `local/` for parser and render conformance runs. The MIT-licensed
  fract4d/formulas set (outside its `orgform/` folder) may be vendored in small samples.
- **Oracles:** the IR's own bignum interpreter per pixel (slow, exact) for deep checks; the f64
  interpreter for shallow checks; the built-ins' hand-written paths for the eight opcode programs.
- **Device safety:** the calibration pass and ceiling are exercised on the RX 6800 XT through the
  field agent before custom formulas ship; a deliberately dear formula must be ceiling-bounded.
- **Every gate names its counts** (formulas parsed, pixels differing, tests run), per the project's
  rule that a check which cannot go red is not a gate.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Restructuring the fixed shader changes built-in output | Its own byte-neutral phase, gated by goldens and bit-exact self-tests |
| Compile times on other GPUs (only the RTX 3080 measured) | Measure on the RX 6800 XT early in phase 2; preview pipeline stays small |
| BLA radii for opcode formulas are heuristic (F3 marks them unverified) | Our own validation (BLA on vs off at depth, per formula class) before enabling |
| Fractint semantics are underspecified | Document our choices; test against the corpus with a CPU oracle; report unsupported features |
| Licensing of corpora | Fetch at test time; no bundling; cite |
| Scope creep toward UF's language | Non-goal; `perturbloop` is the only UF import |

## 8. Relation to other work

- **Live-render plan (`design/live-render-robustness.md`).** Independent layers: W4–W6 decide what to
  dispatch and never look inside a formula. The coupling is `FormulaCaps` (W7's formula half) and the
  per-formula cost factor (W2/W3's price). Phase 0 here can go before or with W7.
- **Imagina-strategy work** (`local/…`): its linear-approximation scope order (Mandelbrot → Julia →
  Multibrot → abs families as 2×2) matches phase 4's BLA.

## 9. Open questions

1. How much of the ~4 s all-formula compile is a startup cost users pay today on a driver update? (A6
   suggests specialising the built-in module too could cut first-launch time; out of scope here.)
2. Should the `.frm` reader accept `whitesq` (6.5% of the corpus) as a per-pixel checkerboard value,
   accepting that supersampling changes its meaning?
3. Newton-type (convergent) custom formulas: a separate bailout kind and colouring; phase 3 or later?
4. Phoenix-like formulas (previous iterate): in the IR from phase 1, deep zoom only with an author
   `perturbloop` (as UF does) unless the rule table is extended to two-term state.

## 10. Licensing

Fractadyne is `MIT OR Apache-2.0`. Kalles Fraktaler 2+, Fraktaler-3, `et` and Imagina are AGPL-3;
Iterated Dynamics and its formula-compiler, Fractal Zoomer and Mandelbulber are GPL-3. None of their code
is copied or ported: the algorithms come from the published papers and blog posts (§11), and their
**file formats** (`.f3.toml`, `.kfr`, `.frm`) are read as data. Formula corpora carry no clear
redistribution terms and are not bundled.

## 11. References

- Fractint 20.04 documentation §2.35 "Formula"; B. Beacham, *FRMTUTOR* (1995); Fractint parser source
  (github.com/LegalizeAdulthood/fractint-legacy).
- Ultra Fractal manual — formula sections and perturbation equations
  (ultrafractal.com/help/writing/formulas/fractalformulas.html, …/perturbationequations.html,
  …/writing/tips/compatibility.html).
- Fraktaler-3 3.1 manual (formula window, formula parameters) and fraktaler.mathr.co.uk.
- Kalles Fraktaler 2+ manual (mathr.co.uk/kf/manual.html) — hybrid formula designer, `.kfr` keys.
- C. Heiland-Allen: "Perturbation algebra" (mathr.co.uk/blog/2018-03-12), "Deep zoom theory and
  practice" (2021-05-14) and "… again" (2022-02-21), "Generalized series approximation" (2021-09-08),
  "Perturbing Nova" (2021-09-27), mathr.co.uk/web/deep-zoom.html.
- Corpora: Orgform (Wayback copy of nahee.com/Fractals/ORGFORM.ZIP), github.com/LegalizeAdulthood/id-libraries,
  github.com/fract4d/formulas, the Ultra Fractal public formula database (ultrafractal.com/formulas).
- The survey behind this document, with verbatim samples and per-source notes, is kept outside the
  repository (it quotes third-party formulas).
