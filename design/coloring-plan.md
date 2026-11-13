# Coloring methods: implementation plan

Status: **plan (2026-11-08); phase A done (A1–A10), B1 done (the coloring matrix; its live tour is B1b), B2 done (what a frame accumulates is a `StatSet`, apart from the method shown), B3 done (in floatexp a stripe, triangle or trap frame carries the whole G-buffer), see §10.** Line references are to `main` at `9ab75357`; check
them before editing, they drift. The research behind it (methods, prototypes, measured results,
prior art) is the coloring-methods doc; its local copy is
`local/research_notes/Fractal coloring methods/` (PDF, HTML, Markdown, prototype scripts).

The order follows one rule: **simple and independent first, then the plumbing everything else
needs, then the methods from the best defined to the least proven.** Each step is one branch and
one reviewable change. A step lists what it depends on, so steps without a dependency on each
other can be done in any order.

---

## 0. How to use this plan

### 0.1 Step template

Every step below has:

- **Goal**: what the user can do afterwards, in one or two sentences.
- **Depends on**: step ids that must be merged first.
- **Size**: S (one session), M (two to four), L (more).
- **Touches**: files and functions, with line numbers at `9ab75357`.
- **Work**: the change, in order.
- **Tests**: what to add and what to run.
- **Done when**: the acceptance checks.
- **Traps**: the known ways this goes wrong.

### 0.2 Branch, commit and merge

- Branch per step: `feat/color-<id>-<slug>`, `fix/color-<id>-<slug>`, `chore/...`.
- Commit messages through the Write tool and `git commit -F`; end with the Co-Authored-By line.
- Before every commit, chained: `python scripts/check_text_hygiene.py --staged &&
  python scripts/check_test_layout.py && <redaction grep> && git commit -F msg.txt`.
- Merge and push only when the user says so. After each merge and push: run
  `scripts/build-accelerated.ps1` on `main` and check `installed.txt`.
- User-visible changes get a CHANGELOG entry under the unreleased version, written with the
  change, not at release time.

### 0.3 Standard commands

Run every exe test with a scratch config and no sound:
`FRACTADYNE_CONFIG_DIR=<scratch> FRACTADYNE_NO_SOUND=1`. Never run against the real config dir,
never kill the user's running app, and put windowed runs on the right monitor.

| What | Command |
|---|---|
| Build | `CARGO_BUILD_JOBS=1 cargo build --release -p fractadyne-app` |
| Unit tests | `CARGO_BUILD_JOBS=1 cargo test --release -p fractadyne-app -p fractadyne-gpu -p fractadyne-core -p fractadyne-color -p fractadyne-state -p fractadyne-farm` |
| One selftest group | `target/release/fractadyne.exe --selftest --selftest-filter <tag> --out <scratch>/report.md` |
| Full selftest | `target/release/fractadyne.exe --selftest --out <scratch>/report.md` (about 3.5 min on the RTX 3080); if it ran with the default `--out`, `git restore validation/report.md` |
| Bless one golden | `--selftest --bless --selftest-filter <golden-name>` (only on the RTX 3080, the reference card) |
| One render | `--render --out F.png --fractal "NAME" --center X Y --zoom M --size WxH --iter N --ss 1 --method KEY --palette N --no-watermark` |
| Compare two renders | `--compare A.png B.png --out heat.png` (report only) |
| GPU timings | `--profile --regions F.toml --reps 3` (iterate and color-pass ms from timestamp queries) |
| Bench matrix | `--bench-matrix --reps 5`; timing comparisons between builds: interleave ABABAB, median of 3 |
| UI screens | `python scripts/on_monitor.py right -- target/release/fractadyne.exe --uitest <dir>`; then LOOK at the screenshots (physical pixels) |
| Text layout | the uitest text-layout check, also with `--language en-XA` |
| Second machine | `scripts/field-request.ps1 -Action harness -Run "..." -Build v...dev.N` (PLUTO: RTX 3070 + RX 6800 XT) |

### 0.4 Standing rules for every coloring change

These come from past bugs; each one has bitten before.

1. **A method or trap is a row of one table** (since A10): `fractadyne_core::coloring`
   (`ColorMethod`, `TrapType`, `MethodSpec`, `TrapSpec`). A row gives the persisted key, the
   shader constant's name and what the method needs (`needs_aux`, `blocks_iter_skip`,
   `bounded_value`, `needs_distance_estimate`, `bla_aggregate`). The GPU number is the row's
   position, `gpu_id()`, for one build only: never store it, never write it as a literal. The
   shader compares `color_method` and `trap_type` only with the generated `METHOD_*` / `TRAP_*`
   constants, which `fractadyne_gpu::renderer_wgsl()` appends to `mandelbrot.wgsl` (the file alone
   does not compile; every module starts from that function). Files, the farm, the CLI and the
   harness outputs (bench matrix, profile JSON, goldens) use keys or the typed value. A new method
   is a new row, a new variant, a label in `ColoringLabel` (`main.rs`) and the shader branch;
   `coloring_ids_tests` (GPU crate) fails on a numeric comparison in the shader. Palette presets are
   still append-only (their indices are saved).
2. **The method sets live in the table.** Since B2 a method row says which orbit statistic it
   reads (`reads`, a `Stat`), and the iteration's rules ask a `StatSet`, the statistics a frame
   accumulates (`ColoringConfig::orbit_stats`; the GPU crate's `iter_stats` adds what the method
   reads, so a frame never shows a statistic it did not accumulate). A rule about the ITERATION
   (series, BLA, the walk, the band's BLA, the corrector, the normalized export, the iteration
   keys, the reference's aggregates, the twin's readback) asks the set; a rule about the PICTURE
   (the color pass, strips, zoom video, the fit range, the menus) asks the method (`reads_aux`).
   A missing trap entry gave flat trap frames on 2026-11-06 (fixed in `028d7204`), and core's unit
   test pins every set as established. Outside the table:
   `WindowTexels::for_colour` (what a window keeps per method, by name) and `COLOR_METHODS` in
   `scripts/release_checklist.py:36` (the English labels, for the manual checklist).
3. **Anything an iteration depends on goes in both keys**: the GPU `IterKey`
   (`gpu/src/lib.rs:422-457`, built at 3872-3893) and the app's `settings_hash`
   (`render.rs:8585-8621`; its coloring part is `iter_coloring_hash`). Since B2 both key the
   statistics set and a statistic's parameters only while it is accumulated, never the method on
   screen. Anything that colors a frame goes in `accum_color_sig` (`main.rs:16163-16219`), or the
   progressive average and the palette browser go stale.
4. **ColorU and ColorUniforms change together.** `ColorU` (`mandelbrot.wgsl:3601-3646`) and
   `ColorUniforms` (`gpu/src/lib.rs:340-387`, two pad words free at 385) are pinned by
   `gpu/tests/unit/uniform_layout_tests.rs` (size 69-90, offsets 112-125, naga 144-151); give
   every new field an offset entry.
5. **i18n**: every label, tooltip and message is a key in `i18n/en/fractadyne.ftl` used through
   `tr!` (a missing key fails the build). `scripts/check_ui_text.py` rejects literal widget text;
   `scripts/check_i18n.py` checks the other 14 languages. New English keys need placeholder
   entries or the language check's allowance. US English.
6. **UI**: explicit ids on every `Window`, `CollapsingHeader` and `ComboBox`; icons only from
   `icons.rs`, generated by `scripts/subset_lucide.py` (edit its `ICONS` table, never `icons.rs`);
   every dialog has a named way out (`dlg-close`); action rows via `theme::action_row`.
7. **Persistence travels to four places**: the session (`fractadyne-state`, additive
   `#[serde(default)]`), the `.fdn` view (writer `export.rs:847-933`, reader 1052-1225, gates
   below), the farm (`fractadyne-farm/src/settings.rs` `RenderSettings`, `deny_unknown_fields`:
   a field missing here makes farm renders silently use the default) and the CLI (`help.rs`
   `CLI_REFERENCE`; a flag missing there is rejected). Tours and scripts are a fifth when the
   setting should be keyframed.
8. **`.fdn` gates**: the writer must stay a subset of the reader. Selftest "every emitted key is
   a known key" (`selftest.rs:9965-9988`), `the_sample_location_covers_every_view_key`
   (`app/tests/unit/tests.rs:315-383`, with `SAMPLE_OPTIONAL_KEYS`), `tests/unit/export/view_text.rs`
   and `shipped_files.rs`. Emit only non-default values so ordinary files stay byte-identical.
9. **Selftest checks are hermetic**: reset and restore every field they touch, like `aux-bla`
   (`selftest.rs:6936-7019`), and add new fields to the reset at `selftest.rs:714-748`. Add every
   new `want` tag to `GROUPS` (`selftest.rs:757-763`).
10. **Never compare renders by file hash.** Compare decoded channels with a tolerance
    (`img_diff`, `--compare`, the corpus script). And LOOK at the image before calling a step
    done: a number that looks good on a broken picture is a bug report.
11. **A change that turns on a new GPU regime runs untested code.** The first run of a new
    per-iteration statistic, a deeper loop or a new BLA lane is a device-loss risk: say so first,
    run it capped, and run it on PLUTO before the user's machine.
12. **Test layout**: unit tests under `crates/<crate>/tests/unit/...` via `#[cfg(test)] #[path]`;
    harness code in `src/harness/`; `scripts/check_test_layout.py` before every commit. A new call
    from `main.rs`, `cli.rs` or `render.rs` into a harness raises `HARNESS_HOOKS`
    (`check_test_layout.py:47-58`) on purpose. New harness flags go in `help.rs` and `TASK_FLAGS`
    (`main.rs:802-811`).
13. **DE units**: the stored DE is per iteration sample, not per pixel: `de_log2` subtracts the
    texel step (`mandelbrot.wgsl:646-657`, step per texel at `gpu/src/lib.rs:3929-3937`) and the
    texture is screen × supersampling. Anything with a width in pixels must correct by log2 of the
    supersampling factor, and also for reduced-resolution motion frames.
14. **Depth bands**: direct below 1e4× (1e2× for Julia), df32 from 1e4× to 1e28×
    (`PERT_FE_THRESHOLD`, `tunables.rs:2308`), floatexp above. The live view renders the df32 band
    as floatexp with BLA (`BAND_FLOATEXP_DEFAULT`, `tunables.rs:1584-1591`), except for aux
    methods (`render.rs:6752-6762`); exports keep df32. Aux methods get BLA only past 1e28×. Test
    every method in every band it can reach.

### 0.5 Definition of done (every step)

- Unit tests green; the step's selftest group green; the full selftest green before merge for any
  GPU change.
- Once B1 exists: `--coloring-matrix --quick` shows no new red cell, and the step's contact sheet
  has been looked at.
- UI steps: uitest screens added and looked at, also in `en-XA`.
- CHANGELOG entry for user-visible changes; the TODO.md item checked or updated; this plan's step
  marked done with its commit.

---

## 1. Where coloring lives today

### 1.1 Data per pixel

Two `Rgba32Float` targets per iteration sample (`mandelbrot.wgsl:1328-1335`):

| Target | Lanes | Written when |
|---|---|---|
| `main` | smooth count `smit`, normal x, normal y, log2 DE | always; interior `(-1, 0, 0, 1e30)`, Newton `(iter, 0, 0, 1e30)`, glitch −2; normal and DE only for formulas 0–3, Phoenix and Multibrot 6–8 (`wgsl:1653-1658`), 1e30 otherwise |
| `aux` | stripe, TIA, trap minimum, final angle | only under an aux method (`aux_on` bit 0); `aux_step` fills only the selected statistic (`wgsl:1374-1414`), `aux_pack` writes the angle for every aux method (`wgsl:1417-1435`); otherwise `AUX_NONE` = (0, 0, 1e30, 0) |

The color pass (`fs_color`, `wgsl:3835-4001`) reads both targets at every tap. `coord_under`
(`wgsl:3731-3761`) picks the value by `cu.color_method`: 1 `a.r`, 2 `a.g`, 3 `−log2(a.b)`
(colors the interior too), 4 `m.a`, 5 `a.a`, else `m.r`; then the optional log map, then
`× cycle + offset`. Then palette lookup (`palette()`, 3693-3708, with the prefix-sum box filter
3767-3793 behind `FRACTADYNE_PALETTE_AA`), relief lighting (3937-3980), glow (3984-3991) and
vignette (3994-3999).

### 1.2 The method enum and what it switches

`ColorMethod` (`main.rs:3036-3131`): Smooth 0, Stripe 1, TriangleIneq 2, OrbitTrap 3, Distance 4,
Decomposition 5. One value drives both sides:

- **Iteration side**: `IterUniforms.color_method` (`gpu/src/lib.rs:69`), read by the shader only at
  `wgsl:1012` (trap histogram), 1358 (stripe tail) and 1381 (`aux_step`); `aux_on_word`
  (`gpu/src/lib.rs:3440-3446`: bit 0 aux, bit 1 tail, bits 2.. tail length).
- **What an aux method turns off**: series approximation (`blocks_iter_skip`, `main.rs:3122-3130`),
  the chunked walk (`frame_plan.rs:396-399`), band BLA (`render.rs:6752-6762`), the glitch
  corrector and the normalized export (`render.rs:4858, 4967`), zoom video
  (`ui/zoom_video.rs:263-264`, `assemble.rs:735-739`), multi-GPU export splitting
  (`gpu/src/export.rs:1524`), and the renormalized step (`wgsl:1077-1080`). BLA is kept only for
  stripe, TIA and trap (`aux_bla_ok`, `render.rs:3693-3695`), whose aggregates sit in the BLA
  node's fourth vec4 `[span, trap, ΣTIA, Σstripe]` (`wgsl:1851-1870`; `reference.rs:2177-2273`).
- **Color side**: `ColorUniforms.color_method` (`gpu/src/lib.rs:355`), `coord_under`, and the
  palette rate `color_cycle` (`main.rs:11438-11444`).

### 1.3 Where coloring settings travel

| Place | Carries today | Gap |
|---|---|---|
| Session (`fractadyne-state/src/lib.rs:291-361`) | everything | none |
| `.fdn` view, bookmarks, Share location, PNG/EXR text | palette, cycle, iter_div, offset, aa, custom gradient; since A1 also method, stripe, trap, effects, duotone/binary, normalize, log scale | none since A1 (interior color when C1 adds it) |
| Farm `RenderSettings` | method, stripe, trap, light, glow, log, cycle, offset, iter_div, palette, custom | `palette_id`, normalize live/fit |
| CLI (`main.rs:8452-8520`) | method, stripe, trap, binary, light, light angle, de, iter_div | cycle, offset, light height, glow strength and width, normalize live/fit, interior color |
| Tours (`scripting.rs:576-688`) | palette only | method, cycle, offset, effects |
| GPU export (`gpu/src/export.rs:278-290, 1710, 1789-1796`) | all | aux methods export on one device, without glitch correction |
| Zoom video | Smooth and Distance only | every aux method refused |
| Multi-GPU live twin (`motion_worker.rs:312`) | with aux when needed | none |

### 1.4 Tests that exist

- Selftest `appearance` (`selftest.rs:10069-10235`): all six methods at the seahorse, 2e3×, direct;
  coherent and mutually different; toggles for cycle, offset, divide, binary, duotone, relief, glow.
- Selftest `aux-bla` (`selftest.rs:6928-7020`): BLA on vs off at 1e30× for traps, TIA, stripe.
- Selftest `bla` (6636-6925): per-family BLA A/B, Smooth only. `cycle-check`, `colour-box`,
  `palette-fade`.
- Goldens (31): one stripe (`seahorse-stripe-1e6`) and one relief golden; the rest Smooth.
- Bench matrix: color segments smooth, stripe, trap, decomposition at 1e20× (df32, no BLA).
- Unit: the needs_aux sync, uniform layout, trap and norm histograms, the core aux-BLA oracle
  (`core/tests/unit/reference/aux_bla_oracle.rs`).

**Gaps that matter for this plan**: no live-path coloring harness (`--livetest` pins Smooth,
`livetest.rs:362`; windowed harnesses ignore `--method`, `main.rs:8155-8157`); no aux A/B for the
fold or Multibrot BLA trees; no TIA or Distance bench segment; no stored non-Mandelbrot or Julia
view files (deep points for other families exist only as `FOLD_DEEP` and `MULTIBROT_DEEP`
constants, `selftest.rs:517-577`); `GROUPS` is not checked against the `want` tags.

---

## 2. Overview

| Phase | Steps | Needs | Gives |
|---|---|---|---|
| A. Independent fixes | A1–A10 | nothing | correct baseline, persistence, reproducible CLI |
| B. Plumbing | B1–B8 | A1, A6 | test matrix, accumulate mask, full G-buffer, settings model, method picker and preview sheet, second LUT and interior modes, stored angle |
| C. Color-pass methods | C1–C11 | B4 (B5 for C4) | interior color, transfers, DE shading, lighting, histogram, OKLab, palette AA, fringe, grid, linked Julia palettes |
| D. Escape-data methods | D1–D3 | B8 | angle colorings, exact count past 8.4M, interior period |
| E. Statistic combinations | E1–E5 | B3, B5 | warped bands, statistic near filaments, two-layer blend, bump, specular by statistic |
| F. New per-iteration statistics | F1–F6 | B2, F2 | exponential smoothing, second aux target, curvature, parametric traps, atom domains, pixel-width traps |
| G. Research methods | G1–G8 | as listed | interior DE, interior coordinates, second derivative, fold DE, two-plane, multi-degree, texture effects, user scripts |

Critical path: **A6 → B1 → B2 → B3 → B7**, with **A1 → B4 → B6 → B7** beside it. Everything in
C needs B4 (C4 also B5); E needs B3 and B5; F needs B2. Phase A steps are independent of each
other.

---

## 3. Phase A: independent fixes

### A1. Coloring travels with the view

- **Goal**: a `.fdn` file, a bookmark, a Share link and a PNG's embedded view reopen in the method
  and effects they were saved with.
- **Depends on**: none. **Size**: M.
- **Touches**: `export.rs` `view_metadata` (847-933), `load_view_metadata` (1052-1225),
  `KNOWN_VIEW_KEYS` (300-324), `TYPED_VIEW_KEYS` (357-382); `tests/unit/tests.rs` `SAMPLE_LOCATION`
  and `SAMPLE_OPTIONAL_KEYS` (315-383); `tests/unit/export/view_text.rs`.
- **Work**:
  1. Keys, all optional and written only when they differ from the default: `method`,
     `stripe_freq`, `stripe_tail`, `stripe_tail_len`, `trap`, `light`, `light_angle`,
     `light_height`, `glow`, `glow_strength`, `glow_width`, `duotone`/`binary` with their two
     colors, `normalize_live`, `normalize_fit`, `log_scale`.
  2. Reader: follow the `iter_div` precedent (1186-1204): a file that states `cycle=` but omits a
     coloring key was written with the default, so reset that setting; a file stating neither
     leaves the session alone. Parse method and trap with a round-trip check and report an
     unknown value as a warning, never a silent Smooth (`from_key` falls back silently,
     `main.rs:3082-3087`).
  3. No format bump: unknown keys are already reported, not fatal (rule at `export.rs:195-209`).
- **Tests**: round-trip unit test for every key and its default; a view with every key set reloads
  to the same `ColoringConfig`; the existing gates stay green; selftest render of a saved trap view
  equals the live settings.
- **Done when**: a stripe view saved, closed and reopened renders identically (channels), and an
  older file without the keys opens unchanged.
- **Traps**: the checksum covers every field (506-527); keep notes Latin-1; `MAX_EMBEDDED_SEGMENTS`
  for custom gradients is unchanged.
- **Done 2026-11-08** (branch `feat/color-a1-view-coloring`): a new module `view_coloring.rs` holds
  the keys, the writer (`ViewColoring::lines`) and the reader (`ViewColoring::apply`) apart from the
  app, so they are unit-tested on their own; `export.rs` maps the app's fields to and from it
  (`view_coloring`, `set_view_coloring`). Keys as listed, except that Duotone and Binary are one
  key, `palette_mode=duotone|binary`, with `palette_lo` and `palette_hi`, since the two are one
  choice beside the file's other `palette` keys. One rule beyond the list: **a key is written only
  when the picture depends on it**, and only then reset when absent (the stripe density only under
  Stripe, the light's angle only with the light on, the two colors only under Duotone or Binary,
  "Normalize deep colors" only under Smooth, "Always fit range" and the log scale under Smooth and
  the orbit trap), so opening a view does not undo what was tuned for another method. Also: a view
  that states its coloring with no gradient and no two-color palette turns the custom gradient off
  (it was saved on its preset; before, a custom gradient stayed on over it), and Life and L-system
  views neither write nor read these keys. An unknown method, trap or palette mode is a positional
  problem naming the choices, and keeps the session's setting. Tests: 9 unit tests
  (`tests/unit/view_coloring/round_trip.rs`); selftest group `view-coloring` (5 colorings through the
  app's writer and reader onto a different session, a default view carries no key, an older view
  opens in the defaults, a bare location keeps the session's; a trap + relief + glow + Duotone view
  renders meanΔ 0.000 after a reload, 73.8 from the default coloring). Both checks go red with the
  writer's lines removed. ⚠An older view (no keys, with `cycle=`) now opens in Smooth with the
  effects off rather than in the session's method, as `iter_div` already resets.

### A2. Small corrections

- **Goal**: help text and comments say what the code does.
- **Depends on**: none. **Size**: S.
- **Work**:
  - `--binary` help says "Binary-decomposition coloring" (`help.rs:343`) but selects the Binary
    palette (`main.rs:8452-8456`): reword.
  - The stale doc comment of `aux_agg_from_orbit` (`render.rs:490-494`) says stripe "isn't folded
    yet" and "BLA is Mandelbrot-only": correct both.
  - The stale comment at `wgsl:949-950` about the BLA node layout.
  - `GROUPS` (`selftest.rs:757-763`): add the eight missing tags (`colour-box`, `cycle-check`,
    `export-contain`, `honest-step`, `palette-fade`, `retreat`, `view-thumb`,
    `width-independence`) and a unit test that every `want("...")` tag in `selftest.rs` appears in
    `GROUPS` (parse the source, like the layout check).
  - `design/multilingual.md` §9 still says eight languages; there are 15.
- **Done when**: the new GROUPS test fails if a tag is removed from the list.
- **Done 2026-11-08** (branch `fix/color-a2-small-corrections`, stacked on A6): all five. `GROUPS`
  moved to module level beside `OPT_IN_GROUPS`; `every_want_group_is_listed` (in
  `opt_in_tests.rs`) reads the source's `want("…")` calls both ways, like the opt-in check.

### A3. Hide what a formula cannot do

- **Goal**: Distance, relief lighting and glow are not offered where they cannot work.
- **Depends on**: none. **Size**: S.
- **Touches**: `ui/panels.rs` method combo (167-177) and Effects (410, 477-513); `ui/menus.rs`
  586-596; `FormulaCaps` (`core/src/lib.rs:192-240`).
- **Work**: a helper `formula_has_derivative(fractal)` from `FormulaCaps` (true for formulas 0–3,
  Phoenix, Multibrot 6–8; false for the folds, Newton and custom). Gray out Distance in the combo
  and menu with a tooltip saying why; gray the relief and glow rows the same way. If the session
  already holds Distance for such a formula, keep it but show the reason under the combo.
- **Tests**: uitest screen with Burning Ship selected (looked at); unit test of the helper against
  the shader's list (`wgsl:1655`).
- **Traps**: Newton stores 1e30 in the DE lane (`wgsl:1641-1647`); custom modules hard-code
  `de = 1.0e30` (`gpu/src/custom.rs:999`).
- **Done 2026-11-08** (branch `feat/color-a3-derivative-gate`): the helper is a capability,
  `FormulaCaps::distance_estimate` (Mandelbrot, Multibrot 3–8, Phoenix), so every caller asks it
  as for the other capabilities; `ColorMethod::needs_distance_estimate` names the methods that read
  it (Distance). Measured first, at Burning Ship ×30: Distance renders 3 colors (all black), relief
  and glow change no pixel (meanΔ 0.0); Phoenix with Distance renders 10,536 colors. Panel and Color
  menu gray out Distance, Effects grays out relief and glow, each with the reason as its disabled
  tooltip; a setting already on stays on, with a note under the Method box or at the top of Effects.
  `--method distance`, `--light`, `--de` warn on such a family. The relief and glow tooltips now
  name Phoenix too. Tests: `fractadyne-gpu` `de_gate_tests` reads the shader's three distance
  writes and `fam_holo`, and checks the capability for every id (red with Phoenix dropped); the
  core caps test; uitest screen `no-derivative` (Burning Ship, Distance, relief and glow on,
  Effects held open by `Dialogs::uitest_effects_open`), whose check holds the canvas to one color
  as the note says, looked at in English and en-XA.

### A4. The stripe-tail iterate key

- **Goal**: toggling "Tail only (deep zoom)" or changing its length always re-iterates.
- **Depends on**: none. **Size**: S.
- **Work**: first reproduce: a settled, untiled stripe view, toggle the tail, check whether the
  frame changes (`IterKey` has no `stripe_tail`/`stripe_tail_len`, `gpu/src/lib.rs:430-457`; the
  app's `settings_hash` has them, `render.rs:8610-8611`). If it does not, add both to `IterKey`.
- **Tests**: unit test that two `IterKey`s differing only in the tail are unequal; a selftest that
  toggles the tail on a fixed view and requires a channel difference.
- **Done 2026-11-08** (branch `fix/color-a4-stripe-tail-key`, stacked on A2). **Reproduced
  first**: a uitest, not a selftest, because the bug lives in the live paint callback the
  selftest cannot reach. Four screens (`stripe-tail-off`, `-toggle`, `-away`, `-on`): settle a
  stripe view with the tail off, flip it as the checkbox does, pan away and back for an honest
  re-render. Before the fix the toggled frame matched the tail-off frame pixel for pixel (fingerprint
  distance to the honest render 72.7, two runs); after, 0.0. ⚠It was stale WITH a settle grid in
  force too: the app's `settings_hash` re-arming the grid did not cover it, so the tail could not
  be left to the app's key. Fix: `stripe_tail` and `stripe_tail_len` in `IterKey`, whose
  construction moved to `MandelbrotParams::iter_key` so `gpu/tests/unit/iter_key_tests.rs` can
  check the tail changes it. ⚠A `view_gen` bump does not force a GPU re-iterate (the minimap check
  uses one): only a key change, a resize, an adopted frame or a new orbit does.

### A5. The two open coloring bugs

Both are in TODO.md (top of "Open bugs", branch `chore/todo-coloring-bugs`). Neither is
reproduced. Reproduce before fixing; never close on the mechanism argument alone.

**A5a. Short-escaper series drops statistics.**

- **Depends on**: none. **Size**: M.
- **Work**: `sa_short_escaper` (`render.rs:1562-1567`) and `live_series` (3721-3727) apply the
  series for a BLA view whose reference escapes early, without checking the method; the shader
  seeds δz at `sa_skip` with no aggregate (`wgsl:1720-1735`). Repro: a stripe and a point-trap
  view past 1e28× in the deep exterior, export with the series as is and with it forced off,
  compare aux channels. Fix by seeding the aggregate from the reference's first `sa_skip` iterates
  (the same approximation the BLA fold makes), computed once per reference on the CPU and passed as
  a uniform; gating the rule off for aux methods would bring back the slow minibrot pixels it was
  added for (its doc comment).
- **Tests**: extend `aux-bla` with a short-escaper view: SA forced on vs off within the `aux-bla`
  tolerance.
- **Traps**: the aggregate depends on the trap type and stripe frequency like the BLA lanes do.
- **Done 2026-11-08** (branch `fix/color-a5a-series-aux`). **Reproduced first** by the new selftest
  `aux-series` (the method set before the request is built, as an export does): the seahorse at
  1e30× needs a 20,000 budget to be a short escaper (its reference escapes at 3,090; at 5,000 it is
  not, and `sa_skip` is 0); then `sa_skip` 3,088, and stripe / TIA / point and cross traps differed
  from the view iterated in full (no series, no BLA) by maxΔ 0.22 / 0.72 / 0.25 / 0.27 on 75% of the
  channels. **Fix, simpler than planned**: no CPU uniform. The shader folds [0, sa_skip) out of the
  BLA tree at the mode-2 seed (binary decomposition into aligned nodes, highest level first) with
  `aux_fold_run`, now shared with the BLA skip; the aggregates follow the trap type and stripe
  frequency because the BLA lanes already do. Result against every iteration: 0.0010 / 0.0004 /
  0.0000 / 0.0000. **The stripe tail stays approximate** (0.029; 0.70 with the prefix dropped): its
  window weighs the last iterations before the seed, where each pixel has drifted from the
  reference (folding the prefix term by term made it 0.17, so the mean fold stays). BLA alone is
  0.23 off in tail mode, a separate TODO item. Mode 1 and the chunk pass have no aux + series case
  (a statistics method gets the series only through the BLA rule, which is mode 2 only).

**A5b. Stripe and TIA band-edge seams on Multibrot 3–7.**

- **Depends on**: none. **Size**: S.
- **Work**: `aux_pack` blends with `fract(smit)` (`wgsl:1660, 2132, 2361`), which is the position
  inside the band only for powers 2 and 8. Repro: Stripe and TIA on Multibrot 3 with "Always fit
  range", LOOK for seams along band edges. Fix: pass the in-band position
  `clamp(1 − log_p(log2|z_N| / log2 R), 0, 1)` (R = the family's bailout: 256, or 128 for
  Multibrot 8), checking its direction against `mix(prev, avg, ·)`.
- **Tests**: a CPU unit test of the weight at both band edges for p = 2…8; a selftest measuring the
  largest jump across a band edge on a Multibrot 3 stripe render before and after.
- **Traps**: the degree-2 picture changes slightly at some edges, so `seahorse-stripe-1e6` needs a
  re-bless on the RTX 3080; say so in the commit.
- **Done 2026-11-08** (branch `fix/color-a5b-stripe-seams`). **Reproduced first**: at the overview
  (700 px, iter 500) stripe and TIA on Multibrot 3 drew a ring along every band, Multibrot 4,
  Multibrot 8 and Burning Ship 3 likewise, Mandelbrot none. Multibrot 8 is affected as well (R = 128,
  so log_8 7 is not whole; the plan's "only 2 and 8" was wrong). Fix: `band_pos(mag2, power_f, bail2)`
  = clamp(1 − log(log mag2 / log bail2) / log p, 0, 1) at the three escape sites and in the custom
  module (`custom.rs`). Direction as `mix(prev, avg, ·)`: 1 at |z_N| = R (all N terms), 0 at R^p.
  **The degree-2 trap did not happen**: Mandelbrot stripe and TIA are unchanged (mean 0.000, max 1
  level of 255), and the goldens stayed within tolerance (no re-bless). Tests: `fractadyne-gpu`
  `band_pos_tests` (the shader's expression word for word, weight 1 and 0 at both edges for p = 2–7
  at R = 256 and p = 8 at 128, monotone, clamped past R^p, `fract(smit)` agrees only at p = 2, and no
  `fract(smit)` blend left); selftest `band-seams`, which reads the statistic back exactly (a gray
  ramp palette at cycle 1) beside the smooth count and scores pixel pairs across the lines where the
  old weight jumped against Mandelbrot as the control: Multibrot 3 and 8 at 0.84–1.19× the control
  with the fix, 3.5–15× (0.57–0.74) with `fract(smit)`.

### A6. CLI flags for existing settings

- **Goal**: every coloring setting can be set from the command line, so tests are reproducible.
- **Depends on**: none. **Size**: S.
- **Work**: add `--cycle`, `--offset`, `--light-height`, `--glow-strength`, `--glow-width`,
  `--normalize-live on|off`, `--normalize-fit on|off`, and later `--interior-color` (C1), to
  `help.rs` and `apply_cli_render` (`main.rs:8452-8520`). Note that `apply_cli_render` runs only
  with `--render` (`main.rs:8155-8157`); also give `--profile` regions these keys
  (`profile.rs:102-166`, which today turns an unknown method into Smooth silently: make that an
  error).
- **Tests**: unit tests of the parsing; the flag list test.
- **Done 2026-11-08** (branch `feat/color-a6-cli-flags`): `--cycle`, `--offset`, `--light-height`,
  `--glow-strength`, `--glow-width`, each limited to its slider's range from one table
  (`slider_range` in `main.rs`, which the panel's sliders now read too; out of range is fatal,
  and a relief or glow setting without `--light`/`--de` warns, as does `--light-angle`). A unit
  test pins each flag's help entry to its range. `--profile` regions: `deny_unknown_fields`,
  unknown method, trap or fractal is an error, new keys `trap` and `stripe_freq` (set for every
  region, so none carries into the next). **Left out, on purpose**: `--normalize-live` and
  `--normalize-fit`. Both act only on the live view (`live_norm_window` reads the live range
  readings; a headless render has none, `render.rs` at the export request's `cycle`), so on
  `--render` they would change no pixel; `--assemble` already normalizes with `--normalize`.
  They belong with B1's live path (`--livetest` method tours), where a live run reads them. The
  cycle, offset and effect sliders are not region keys: they do not change what a region costs.

### A7. Field agent can run coloring tests

- **Goal**: PLUTO can render any method, fractal and depth.
- **Depends on**: none. **Size**: S.
- **Work**: extend `$Allowed` in `field-agent.ps1:275-327` with `--method`, `--fractal`,
  `--profile`, `--stripe-freq`, `--trap` and the A6 flags; bump the agent version; deploy per
  `scripts/field-request.ps1`.
- **Done 2026-11-11** (agent v40, branch `feat/color-a7-field-agent`): for the modes that render,
  `--method`, `--trap`, `--stripe-freq`, `--stripe-tail`, `--palette`, `--binary`, `--cycle`,
  `--offset`, `--iter-div`, `--normalize`, `--log-palette`, `--light`, `--light-angle`,
  `--light-height`, `--de`, `--glow-strength`, `--glow-width`, `--fractal`, `--julia`,
  `--julia-c`; and `--profile` as a mode with `--reps` and `--regions` (a package TOML or
  `field\locations\NAME.toml` in the share; its `profile-*.json` comes back with the logs). A
  request's flags arrive split on whitespace, so a family name with a space is written with `_`
  (`Burning_Ship`, `Multibrot_3`); the agent turns it back and quotes it. The agent checks only a
  value's shape; the app refuses an unknown method, trap, palette or family. Windowed harnesses
  still read the session's coloring, not these flags. Checked on PLUTO's RX 6800 XT with
  `v0.3.0-beta.26.dev.19`: a Burning Ship stripe `--render` (the same picture as the RTX 3080's)
  and `--profile` over a regions file in the share (Burning Ship stripe, Multibrot 3 cross trap).

### A8. Palette browser: a named way out

- **Goal**: the browser follows the dialog rule and becomes the template for the method sheet.
- **Depends on**: none. **Size**: S.
- **Work**: a Close button (`dlg-close`) in an action row (`ui/palette_browser.rs:273-334`);
  optionally a keyboard shortcut for both browsers.
- **Done 2026-11-08** (branch `feat/color-a8-browser-close`, stacked on A4): the Close row under
  the details (`theme::action_row` + `cancel_button`, the dialogs' pattern), with the grid's
  reserve below it raised from 120 to 156 px. uitest palette-browser screens 3/3 in English; in
  en-XA the same two shortened filter headings (Temperature, Brightness) as on `main`, so not
  this change. No keyboard shortcut: no dialog closes on a key (Esc stops the auto-zoom, a tour
  or full screen, `main.rs`), so one here would be the only one; decide for all dialogs at once.

### A9. DE in pixels

- **Goal**: Distance colors and glow bands look the same at every supersampling factor.
- **Depends on**: none. **Size**: S. **User decision**: this changes the default look at the
  default 2× supersampling (Distance colors shift by one octave, glow bands rescale), so ask first.
- **Work**: pass `log2(ss)` (and the motion-frame resolution factor) in `ColorU` and subtract it
  where the color pass reads `m.a` (`coord_under`, glow at `wgsl:3983-3990`).
- **Tests**: render the same view at ss 1, 2 and 3; the Distance coloring agrees within the
  tolerance.
- **Done 2026-11-08** (branch `fix/de-in-pixels`): `ColorU.display_w` (the panel width, from the
  new `MandelbrotParams.display_res`; 0 = the texture's base) and a `de_px_shift` the color pass
  computes from the texture's real width, so it also covers reduced motion frames, held frames and
  a reprojection's zoom. Selftest group `de-pixels`: ss 1 vs ss 3 median pixel difference 0.0001
  (Distance) and 0.0000 (glow) fixed, 0.20 and 0.12 with the shift disabled.

### A10. One method table: string identity, generated GPU numbers

- **Goal**: a method is identified by its key string everywhere outside the GPU; the GPU number is
  generated from one Rust table, so method ids no longer need to be append-only.
- **Depends on**: none (easier before B2, which adds the accumulate bits). **Size**: M.
- **Work**:
  1. One table in Rust: key, label key, GPU id, predicates (needs aux, bounded, blocks skips),
     capability row (B4 later extends it). `ColorMethod`'s helpers read the table.
  2. Inject the ids into the shader as constants (`const METHOD_STRIPE: u32 = …;`), the way the
     shader already takes generated text at `@@` markers, and replace the numeric literals in
     `coord_under`, `aux_step`, the trap histogram and the stripe-tail gate.
  3. Replace the Rust literals: `method_needs_aux` (`gpu/src/lib.rs:3453`),
     `WindowTexels::for_colour` (`gpu/src/assemble.rs:77`), `render.rs:3645-3648, 3693-3695,
     9947-9950`. The GPU crate cannot see `ColorMethod` (it lives in the app), so the table belongs
     in a crate both use (`fractadyne-core` or `fractadyne-color`).
  4. Harness outputs and baselines store the key, not the number: `bench_matrix.rs:208` and its
     baseline JSON (migrate on read), `profile.rs:38, 314`, the golden spec
     (`selftest.rs:11316`), `BenchSnapshot` (`scripting.rs:4890`, `from_u32` at 5051).
  5. The same for `TrapType` (numbers also in `core/src/reference.rs:1946-1950`).
- **Tests**: a unit test that every table row's shader constant matches the injected text; that no
  numeric method literal remains in the shader (grep the source); the needs_aux sync test reads the
  table; old bench baselines still load.
- **Done 2026-11-11** (branch `feat/color-a10-method-table`, stacked on A7):
  `fractadyne_core::coloring` holds both tables and `wgsl_ids()`; the app re-exports `ColorMethod`
  and `TrapType` and keeps only the words (`ColoringLabel`); `fractadyne_gpu::renderer_wgsl()`
  (the file plus the ids, appended so compiler line numbers stay the file's) replaces every
  `include_str!` of the shader (the live and export modules, the split iterate modules, strip
  modules, custom formulas, the GL link probe, the compile bench). The shader's 16 numeric
  comparisons now name `METHOD_*` / `TRAP_*`; `method_needs_aux` and `for_colour` ask the table;
  `render.rs`'s BLA checks name the method (`bla_aggregate`) and keep the trap shape typed (the
  cache's `bla_trap_type` is `Option<TrapType>`, `None` = no tree yet); core's `AuxAggParams` takes a
  `TrapType`. The bench matrix, profile regions, the benchmark snapshot and the golden specs hold
  `ColorMethod`; the profile JSON writes `"method": "stripe"` (it wrote the number; nothing read
  it). Bench baselines never stored a method (segments are named), so nothing to migrate. Tests:
  core `coloring` (keys pinned, every set pinned, the generated text), GPU `coloring_ids_tests`
  (no numeric comparison in the shader, sabotage red; the ids appended once; `method_needs_aux` =
  the table), and every naga test now validates the built source.

---

## 4. Phase B: plumbing

### B1. The coloring test matrix

- **Goal**: one command renders every method across fractal families and depth bands, writes
  contact sheets to look at and metrics to compare, and flags regressions.
- **Depends on**: A6 (flags for reproducible settings). **Size**: L.
- **Touches**: new `crates/fractadyne-app/src/harness/coloring_matrix.rs`; `harness.rs` module
  list; `help.rs` (`--coloring-matrix`, `--quick`, `--only`, `--bless`); `TASK_FLAGS`; new
  `validation/coloring/views.toml`; baseline `benchmarks/coloring-matrix-baseline-<card>.json`.
- **View set** (`validation/coloring/views.toml`), each with family, center, zoom, iterations and
  the bands it covers:

  | Family | Views |
  |---|---|
  | Mandelbrot | overview; seahorse 2e3× (direct); seahorse 1e6× and 1e20× (df32); seahorse 1e30× (floatexp); minibrot interior `minibrot-auto-iter-9.3e78`; Misiurewicz `misiurewicz-4-1` 7.4e143×; extreme opt-in `spiral-9.98e60205` |
  | Multibrot 3–8 | overview and the `MULTIBROT_DEEP` points (`selftest.rs:540-577`) at 1e6× and 1e30× |
  | Folds (Burning Ship, Tricorn, Celtic, Buffalo, and powers 3–5) | overview and `FOLD_DEEP` (`selftest.rs:517-538`) at 1e6× and 1e28× (accurate to about 1e30×) |
  | Phoenix | overview and 1e3× |
  | Newton | overview |
  | Julia | Mandelbrot Julia at c = −0.75 + 0.1i and the triple point; overview and 1e10× |
  | Custom | `z^2.5 + c` overview and 1e20× |

  Copy the constants into the TOML so they are no longer selftest-only.
- **Cells**: every view × every method (and each method's parameter variants, e.g. the three trap
  shapes) that the method claims to support; unsupported cells are recorded as "n/a", not skipped
  silently.
- **Per cell, measured**:
  - picture: standard deviation and bucket count (`frame::coherent`), count of NaN, sentinel and
    pure-black pixels, mean difference from the Smooth cell (a method that equals Smooth is a bug);
  - correctness A/Bs: BLA on vs off and SA on vs off where the method claims to be skip-safe
    (tolerance as `aux-bla`: max channel Δ < 0.1 and < 1% of channels above 0.02); render twice
    and require identical channels (determinism); ss 1 vs 2 for pixel-width features;
  - timing: GPU iterate ms and color-pass ms from timestamp queries (as `--profile`,
    `profile.rs:270-330`), and the ratio to the Smooth cell of the same view.
- **Outputs**: one contact sheet per view (methods as cells, labeled) and one per method (views as
  cells), PNG, for looking at; `coloring-matrix-<stamp>.json` with every metric; a Markdown summary
  of red cells.
- **Gates**: picture checks and A/Bs are pass or fail on every card; timing is compared to the
  baseline of the same card only, warning above +35% and 1 ms (the bench-matrix rule), never
  failing a run on timing alone. `--quick`: the overview, one df32 and one floatexp view of
  Mandelbrot, Multibrot 3 and Burning Ship; a selftest group `coloring-matrix-quick` runs it.
- **Live path**: add a `method` key to the livetest/tour `[render]` table (`scripting.rs:158-163`)
  so `--livetest` can run a method tour; a short `tours/coloring-methods.toml` dives through the
  depth bands once per method.
- **Done when**: the full matrix runs on the RTX 3080 and on PLUTO, the current six methods give
  no red cell except known ones (recorded with a reason), and the contact sheets have been looked
  at.
- **Traps**: render with a scratch config; the `make()` span caveat (`selftest.rs:895-900`); keep
  timing runs one at a time.
- **Done 2026-11-11** (branch `feat/color-b1-coloring-matrix`): `harness/coloring_matrix.rs`,
  `validation/coloring/views.toml` (83 views, the spiral opt-in: the table above, the self-test's `FOLD_DEEP` and
  `MULTIBROT_DEEP` copied in; Phoenix, the two Julia sets and `z^2.5 + c` at points found by
  bisection and inverse iteration), the self-test group `coloring-matrix-quick` (9 views, 81 cells,
  about 30 s), field agent v41 (`--coloring-matrix`, `--quick`, `--only`, `--views`; the agent names
  the folder and delivers it), and every package ships the view set and its `.fdn` views. Cells are
  `ColorMethod::ALL` with each trap shape and the stripe tail (9 a view), offered by
  `ColorMethod::offered_for` (the rule the method box and the Color menu now share). How it judges,
  where it differs from the text above:
  - **Accelerators against the plain render** (no BLA, no series), not one toggled at a time: with
    the series on in both, stripe with BLA on and off differed on 96% of the pixels at corpus 07 at
    1e30x, and toggling cannot say which one is wrong. The check is named for what the request used
    (`bla`, `series`, `bla+series`); a failing `bla+series` also measures BLA alone. (Not the series
    alone: the app never runs it without BLA, and without the tree the series' prefix folds no
    statistics.)
  - **A chaotic picture differs between two correct renders**, so a difference is judged where the
    picture is smooth and stable (not steep by count for a method colored by the count, the
    self-test oracle's rule; smooth in the plain render; unchanged by a nudge of 1/1000 px), and by
    rate: a render passes when it changes no more pixels there than the nudge itself changes.
    aux-bla's channel tolerance alone failed Smooth at the seahorse at 1e6x (4.77% of channels, all
    speckle). Sabotage: halving the BLA trap aggregate and taking 10% off the stripe sum turned all
    12 BLA trap and stripe cells of the quick set red (49-100% of the judged pixels, against nudge
    floors of 0-5%); triangle, untouched, stayed green.
  - Distance at ss 1 and 2 is compared on the smooth pixels (`de-pixels`' median; speckle changes
    with supersampling).
  - A failed comparison writes its evidence (A, B, and where they differ) to `ab/`.
  - `[[known]]` takes `*` patterns over `view/variant`.
  - **Results**, RTX 3080, full set: 738 cells, 51 n/a (Distance without a distance estimate), 0
    new red, 48 known, about 5.5 min. The known: Distance draws NaN where the derivative overflows
    f32 (BUG; fixed on `fix/color-de-overflow`, the square in `de_log2`); Newton drew every
    statistics method as one color (fixed on `fix/color-newton-statistics`:
    `FormulaCaps::orbit_statistics`, the menus no longer offer them, the matrix records n/a); the
    stripe tail
    through BLA and the series (the existing TODO); statistics flat at depth (the shared orbit
    prefix); and, open, near the minibrot at 9.3e78x the point and cross traps through BLA and the
    plain render disagree on every pixel (TODO.md; which is right needs the oracle). PLUTO (dev.20,
    field agent v41): the RX 6800 XT in 288 s and the RTX 3070 in 859 s, each 0 new red and the same
    48 known. Contact sheets looked at: every quick view, each failure's evidence, and the Smooth,
    Distance, stripe and cross-trap sheets of the full set (the stripe one from the RX 6800 XT).
  - **B1b, not done**: the live path (a `method` key in the tour `[render]` table, `--livetest` with
    a method, `tours/coloring-methods.toml`). It drives the live pipeline, a separate step.

### B2. Decouple what is accumulated from how it is shown

- **Goal**: the iteration accumulates a set of statistics (an accumulate mask) independent of the
  displayed method, so switching between methods whose data is present only recolors.
- **Depends on**: B1 (to prove no change). **Size**: L.
- **Work** (the full table is in §8):
  1. `AccumMask` bits: stripe, TIA, trap, angle. Today's behavior is `mask = default_mask(method)`.
  2. Repurpose `IterUniforms.color_method` (`gpu/src/lib.rs:69`; read only at `wgsl:1012, 1358,
     1381`) as the mask; `aux_step` dispatches per bit; the stripe-tail gate (`wgsl:1357-1359`)
     reads the stripe bit; mask the tail-length shift (`wgsl:1361`) with `& 0x1FFFFF` if bits are
     packed above it.
  3. Split `method_needs_aux` into "iteration accumulates" (mask ≠ 0: chunked walk, band BLA, SA,
     corrector, normalized export, renormalized step) and "display reads aux" (strips, assembler,
     `WindowTexels`).
  4. `IterKey` and `settings_hash` key on the mask plus only the enabled bits' parameters;
     `accum_color_sig` keeps the displayed method and adds the mask.
  5. `aux_bla_ok` becomes a per-bit table; the aux staleness patch (`render.rs:9939-9988`) watches
     the stripe frequency when the stripe bit is on and the trap type when the trap bit is on.
  6. `RefCache.bla_stripe_freq`/`bla_trap_type` and `same_reference` (`render.rs:10279-10287`)
     compare only enabled bits.
- **Tests**: with the default mask, the B1 matrix and all goldens are channel-identical to `main`;
  a unit test per predicate; the needs_aux sync test becomes a mask test.
- **Traps**: the trap histogram commit (`wgsl:1011-1017`) needs the trap bit and display = trap;
  the custom-formula code path (`gpu/src/custom.rs:1097-1110`) has its own `aux_step` copy.
- **Done 2026-11-11** (branch `feat/color-b2-accum-mask`). Named for what it holds, since `accum`
  already means the progressive average here (`AccumState`, `accum_color_sig`, `ACCUM_*`):
  - `fractadyne_core::coloring`: a statistics table (`Stat`: stripe, triangle, trap, angle; each
    with `blocks_iter_skip` and `bla_aggregate`) and `StatSet`, a set of them. A method row says
    which one it `reads`; `ColorMethod::orbit_stats` is that one as a set. `needs_aux`,
    `blocks_iter_skip` and `bla_aggregate` left the method rows, so the compiler found every caller
    and each was decided: iteration rules ask the set, picture rules ask `reads_aux`.
  - The shader: `IterU.color_method` became `orbit_stats` (`STAT_*` bits, generated like
    `METHOD_*`); `aux_step` and `aux_fold_run` work per statistic (disjoint fields, so a lane's
    value does not depend on what else is accumulated); the stripe tail tests the stripe bit. The
    tail length kept its own field, so no shift needed masking. Custom formulas call the same
    `aux_step` (the plan's "own copy" was out of date).
  - ⭐**A floor, not in the plan**: the GPU crate's `iter_stats` (the uniform, `aux_on`, `IterKey`,
    the export's walk, the twin's readback) takes the set asked for AND what `color_method` reads.
    Two selftest checks set only `req.color_method` on a built request; with the floor no caller
    can show a statistic it did not accumulate.
  - The trap histogram counts whenever the trap is accumulated, not "and shown": the app drains it
    only while the trap is shown, and a switch to the trap that only recolors (B3) finds it counted.
  - `IterKey` and `settings_hash` (`iter_coloring_hash`) key the set, and the stripe frequency and
    tail and the trap shape only while accumulated; the reference's lanes (`same_reference`, the
    overtaken check, `aux_lanes_stale` for both staleness checks) compare only accumulated
    statistics; `accum_color_sig` adds the set.
  - **What it changes now**: Smooth and Distance accumulate nothing, so a switch between them only
    recolors (uitest `distance-recolors`: settle Smooth, switch, pan away and back; the switched
    frame must match the honest Distance render).
  - **Proof of no change**: the full matrix on the RTX 3080 against main's exe (build 6537),
    twice: 738 cells, every picture number equal (A/B shares, determinism, NaN counts, spreads)
    and all 738 cell images equal pixel for pixel. Full self-test 563/563, goldens 31/31; uitest
    `stripe-tail-redraws`, `distance-recolors` (meanD 0.0 against the honest render, Smooth 128.9
    away), `no-derivative` and `no-statistics` pass, screenshots looked at.
  - **Timing**, interleaved main/branch x3 on the eight views that looked slower against one main
    run (the custom formula, Julia 1e10, Multibrot 3, the folds at 1e6): runs 12.9 s against
    13.1 s, median cell ratio 1.014, per variant 1.00-1.05; the outliers (Tricorn 3 at 1.22) swing
    between ~21 and ~26 ms in both builds. ⛔Compare like with like: a plain `cargo build` has no
    MPFR, and the custom view's reference took 5 s more there than in the installed accelerated
    exe; build the branch with `build-accelerated.ps1 -NoInstall -SkipVerify -OutDir <scratch>`.

### B3. The full G-buffer

- **Goal**: an option (and, where cheap, the default) to accumulate stripe, TIA, trap and the angle
  in one pass, so every recolor-capable method is available without re-iterating.
- **Depends on**: B2. **Size**: M.
- **Work**:
  1. Mask = all bits. Per iteration this adds about 5–6 transcendental and 15 plain operations.
  2. Measure with B1 timing and `--profile` in all three bands, on the RTX 3080 and both PLUTO
     cards, interleaving builds. The only measurement so far is floatexp at 1e30×: perf-neutral
     when the per-method gate was added (commit `1de2b888`, 298.2 → 297.0 ms). The direct and df32
     paths were never measured.
  3. Choose: full G-buffer always in floatexp (where measured free); on demand in direct and df32
     (when the method sheet or a combination asks for it), unless the measurement says it is free
     there too.
  4. The trap shape and stripe frequency remain iteration parameters; the trap lane holds the
     selected shape.
- **Tests**: B1 with the full mask: every method's cell equals its own-mask cell (channels); BLA
  A/B for all lanes on Mandelbrot, a fold and Multibrot 5.
- **Traps**: an all-lanes view still pays every aux restriction (no SA, no walk, no band BLA): the
  cost is not just the operations; report both.
- **Done 2026-11-11** (branch `feat/color-b3-gbuffer`):
  - `StatSet::GBUFFER`: every statistic BLA folds (stripe, triangle, trap), derived from the table.
    Each also blocks the series, so adding it to a frame that accumulates anything changes none of
    that frame's rules (unit test over every non-empty set): it costs only its operations. The angle
    stays out: it has no BLA aggregate and would turn BLA off for the others (B8).
  - **The check**: `--coloring-matrix` renders every cell again with the G-buffer added (a cell that
    already carries it, with its method's own statistic alone), built through
    `current_export_request_for` with `ColoringConfig::orbit_stats_override`, so the series, BLA and
    the walk follow the app's rules. Both accumulate something: the channels must be identical
    (check `gbuffer`). The cell accumulates nothing (Smooth, Distance): judged against the plain
    render. `summary.md` gains a cost table per variant and band.
  - ⛔**Timing order**: cell, cell, G-buffer, G-buffer, each pair ADJACENT (a request rendered
    again, as a live frame is). Timed after the cell's plain and nudged renders (dev.21), the
    G-buffer read 12-15% FASTER than the cell on the RX 6800 XT: the card had clocked up.
    Alternated with the cell's (dev.22), the cell's second render lost what its first left warm:
    the df32 cells of the statistics methods read 14-39% over the RTX 3080's baseline (146 cells).
    The table below is dev.22's alternated order; dev.23 has the adjacent one.
  - **Results**, 738 cells on each card, 0 red, no `gbuffer` or determinism failure: RTX 3080 (here),
    RTX 3070 and RX 6800 XT (PLUTO, dev.21 and dev.22). Iterate time with the G-buffer over without,
    median over the band's views, for stripe, triangle and the traps:

    | card | direct | df32 | floatexp |
    |---|---|---|---|
    | RTX 3080 | +15-34% | +10-21% | +2-4% |
    | RTX 3070 | +15-33% | +9-21% | +2-4% |
    | RX 6800 XT | (not priced) | (not priced) | +2-7% |

    ⛔On the RX 6800 XT the paired renders could not price it: whichever request rendered second
    read 10-36% slower (dev.22, the G-buffer second: +18-34% in floatexp; dev.23, the method's own
    second: the reverse by as much). Its floatexp figure is from `--set GBUFFER_DEEP=0` (dev.24),
    four whole runs alternated on, off, on, off, each cell's median per setting: stripe +1.6%,
    stripe tail +1.9%, triangle +4.3%, point, cross and circle traps +7.0%, +7.4%, +6.4%; every cell
    the switch does not touch (direct, df32, and Smooth, Distance and decomposition in floatexp)
    within -0.7% and +1.8%. The NVIDIA figures read the same in both pair orders.
    For Smooth and Distance (losing the series and the walk) the ratios scatter from 0.2 to 3.6 and
    are no case for carrying it always. Decomposition with it, floatexp without BLA: +5-10% median,
    +48% on the RTX 3070's heaviest view.
  - **The choice (user, 2026-11-11: "Always in deep views")**: `ColorMethod::orbit_stats_at`: in
    floatexp a frame colored by stripe, triangle or a trap carries the whole G-buffer, so a switch
    among them recolors; elsewhere, and for every other method, what the method reads. Asked
    because the price depends on the card; the user was given dev.22's +18-34% for the RX 6800 XT,
    and the whole-run measurement found 2-7%. `--set GBUFFER_DEEP=0` turns it off. `ColoringConfig::orbit_stats` now takes the
    frame's band, and every key and every check of the reference's BLA lanes asks with it: the BLA
    lane checks (`same_reference`, `aux_lanes_stale`, the overtaken check) ask for floatexp, the
    only band that builds a tree; the rules that ask only whether anything is accumulated use
    `accumulates_any`, the same in every band (and so no loop with the live band's floatexp-for-df32
    choice, which asks it). `accum_color_sig` hashes the method's own set: a band change is a new
    view anyway.
  - uitest `trap-recolors` (corpus 07 at 1e30, floatexp): settle Stripe with the point trap, set the
    cross trap under Stripe, switch to the trap, pan away and back, set the point trap. The switched
    frame equals the honest cross render (meanD 0.0); Stripe is 79.0 away and the point trap, the
    control a stale lane would look like, 39.6. Looked at.
  - Found on the way (TODO.md): the RTX 3070 takes 27.7 s for the minibrot's decomposition cell
    (RTX 3080 1.4 s, RX 6800 XT 0.45 s); a direct export of Smooth or Distance at 320x200 is up to
    5x slower than the same request with statistics (the walk against one dispatch).

### B4. One coloring settings model

- **Goal**: adding a coloring parameter means editing one struct, and it then persists everywhere.
- **Depends on**: A1. **Size**: M.
- **Work**: a `ColoringSpec` struct (method, method parameters, transfer, wrap, interior mode and
  color, layer settings, lighting and glow, palette references) with serde, used by the session,
  the `.fdn` reader and writer, farm `RenderSettings`, the CLI and the script schema. A capability
  table per method: live, export, normalized export, corrector, zoom video, farm, multi-GPU split,
  chunked walk, SA, BLA, which families, which depth bands. The UI and the export dialogs read
  this table to gray out or explain.
- **Tests**: one round-trip test per destination; the farm settings tests
  (`farm/tests/unit/settings/tests.rs:26-102`); `RenderSettings` stays `deny_unknown_fields`, so a
  missing field fails its test, not a farm render.
- **Traps**: `ColorU` has a size budget; group new fields into vec4s and update the layout tests.
- **Done 2026-11-12** (branch `feat/color-b4-coloring-spec`):
  - **Step 1, the spec** (`2956066a`): `fractadyne_state::ColoringSpec` holds the session's 27
    coloring fields (method by key, stripe, trap, light, glow, two-color, palette by id and number,
    cycle, offset, divisor, normalization, custom gradient). The session flattens it, so the file's
    keys did not move: a session written by main (542a5264) and by the branch parse to the same
    table, each build reads the other's, and the older file is a fixture with a test. The farm's
    `RenderSettings` nests it (protocol 9), so a job also carries the palette's id and the
    normalization; serde cannot combine flatten with `deny_unknown_fields`, so the nested table is
    read by `strict_coloring`, which refuses unknown keys. `App::coloring_spec()` is the app's one
    way out.
  - **Step 2, the view keys** (`073e8107`): A1's 17 keys are rows of `coloring_fields::FIELDS`
    (key, kind and range, parents, when the picture depends on it); `view_lines` and `apply_view`
    walk it on the spec with A1's rules unchanged (A1's nine round-trip tests pass through it).
    `App::apply_coloring_spec()` is the one way in at run time (start-up still reads the session in
    `new`, where `--log-palette` joins). A test fails when a spec field has neither a row nor a place
    on the list of older keys `export.rs` carries (palette, cycle, offset, divisor, gradient).
    `ViewColoring` remains a typed front for the harnesses.
  - **Step 3, tours** (`740d2844`): a `[render.coloring]` table by the field table's keys, each in
    its TOML shape, checked when the script is read (an unknown key, a wrong shape, an unreadable or
    out-of-range value is an error naming the key); `begin_playback` keeps the viewer's settings
    and puts it on, and `--render-tour` puts it on too. `PlaybackRestore` holds the viewer's whole
    `ColoringSpec`. `TOURS.md`'s `[render.coloring]` section is generated from the table. Found on
    the way: a tour's palette morph set the stop list and left the viewer's segments, which win
    (`custom_gradient`), so it drew the viewer's own gradient instead of the morph, live and
    rendered; it now clears them (and the band flag), and they come back with the coloring.
  - **Step 4, the capability table** (`8bbbbc0b`): `Capability` (glitch correction, normalized export, strips,
    the live walk, series, BLA) and `ColorMethod::can`, read off the method's statistics and
    `reads_aux`, so it cannot disagree with the renderer; the zoom-video dialog and the assembler
    ask it. The rules themselves still ask the frame's `StatSet` (B2). B6's picker will read it.
  - **Not done, by choice**: the command line stays hand-written (its ranges are already
    `slider_range`'s, and several flags mean more than one row: `--stripe-tail N` turns the tail
    on, `--binary` turns the gradient off).
  - **Windowed checks, 2026-11-12**: "a tour's palette morph shows over a custom gradient" fails on
    a build without the fix and passes with it; the self-test 565/565 (with "the whole coloring
    goes in and comes back out", which now also round-trips a Binary spec, `a5d7d74b`: its own
    audit found `use_binary` untested); `--farmtest` passes on protocol 9; a tour rendered with and
    without `[render.coloring]` draws each coloring; the coloring matrix has no failures and 101
    timing warnings, not attributed (two runs of the same code gave 14 and 101).

### B5. Second LUT and interior modes

- **Goal**: the color pass can use a second gradient (for the interior or a second layer) and an
  interior mode other than one flat color.
- **Depends on**: B4. **Size**: M.
- **Work**: grow the LUT buffer to two gradients (`make_lut_buffer`, `write_lut`,
  `gpu/src/lib.rs:1910-1949`; each with its prefix sum) with an offset in `ColorU`; an
  `InteriorMode` (solid, trap, period, later others) and an interior palette reference; generalize
  the interior branch of `coord_under`.
- **Tests**: layout tests; a selftest rendering an interior with the second gradient.

### B6. Method picker

- **Goal**: choosing a method shows what it does and what it costs, grouped sensibly.
- **Depends on**: B4. **Size**: M. Details in §9.
- **Work**: replace the flat combo (`ui/panels.rs:167-177`) with a grouped list (Escape count,
  Distance, Orbit statistics, Angle, Layers), an Inside section (interior mode), per-method
  settings below, cost badges from the B4 capability table, and a "Preview methods…" button for
  B7; mirror in the Color menu (`ui/menus.rs:586-596`).
- **Tests**: uitest screens for each group and a disabled method; text layout in `en-XA`.
- **In progress 2026-11-12** (branch `feat/color-b6-method-picker`):
  - `MethodGroup` is a column of the method table (`fractadyne_core::coloring`), so a new method
    names its heading where it names everything else; a core test pins the grouping and that no
    heading is empty. Layers waits for its first method (E).
  - `ui::method_list` draws the one list the panel's Method box and Color ▸ Method share: the
    headings, each method's tooltip (what it shows; what switching to it does now; what it gives
    up here), and the family's reason for a grayed-out method as before.
  - The switch's cost is `ColoringConfig::orbit_stats_of` under each view's current mode: the GPU
    iterates again exactly when the statistics set changes (B2's key), and asking under the
    current mode is exact (the mode depends on the method only through `band_bla`, which needs an
    empty set). The uitest's `method-switch-cost` check holds the claim against the GPU's own
    record of the passes it ran (`gpu_ran_frames`): deep Stripe to the trap recolors, the trap to
    Smooth calculates again, Smooth to Distance recolors, and in df32 Stripe to the trap
    calculates again.
  - The notes come from `ColorMethod::can` (each `MethodNote` reads its capabilities, and a test
    fails on a capability no note reads unless it is left to another control: the normalized
    export, which "Normalize deep colors" explains) and appear only where the family, the view and
    the settings would give the method that speed-up or output (`Offered`): no skip notes on a
    Julia view, no glitch-correction note on Mandelbrot, no zoom-video note on Burning Ship.
  - Not in this step: the Inside section (B5's interior modes) and "Preview methods…" (B7).
  - Found on the way: core's `every_variant_has_its_own_row_and_number` and
    `every_statistic_has_its_own_row_and_bit` compared row addresses of a `const` table, which
    holds only when the optimizer merges the copies: they failed in a debug test build (on main
    too). They compare the rows' contents now.

### B7. Method preview sheet

- **Goal**: a window that shows the current view in every method, like the palette browser;
  hover previews on the main view, click applies.
- **Depends on**: B3, B6 (and A8 for the window pattern). **Size**: L. Details in §9.
- **Work**:
  1. A sibling of `atlas::render_palette_atlas` (`gpu/src/atlas.rs:67-210`) that takes per-cell
     `ColorU` overrides (method, cycle from `color_cycle()` per method, no normalization, trap and
     stripe fields), exposed through a public struct since `ColorUniforms` is `pub(crate)`.
  2. Cells whose data is in the G-buffer are recolors (instant). Cells that need other iteration
     parameters (another trap shape, another stripe frequency, a method not accumulated) are small
     re-renders in the background, one per frame, like `render_misi_thumb`
     (`ui/misiurewicz_explorer.rs:395-447`), with a time estimate; at depth, a cell over the budget
     shows the estimate and a "Render" button instead of rendering.
  3. Hover previews the method on the live view through the preview rules of
     `palette_preview.rs` ("a preview is never a choice", 0.3 s delay); click applies; Close button.
- **Tests**: uitest screens (shallow, 1e30× with recolor cells, a re-render cell in progress);
  unit tests of the cell plan (which cells recolor, which re-render) against the capability table;
  a selftest that the sheet's recolor cell equals a full render of that method (channels).
- **Traps**: the atlas uses view 0 only; `last_cu`'s cycle belongs to the current method, so each
  cell needs its own; invalidate on `accum_color_sig`, view writes and the mask.

### B8. The final angle without aux

- **Goal**: the final angle is stored for every method, and we know when it survives SA and BLA.
- **Depends on**: B2. **Size**: M.
- **Work**: an angle-only mask bit that writes `aux.a` at escape without running `aux_step`;
  the chunked resolve (`fs_resolve`, `wgsl:3596`) must output it too, or the walk stays off for
  it. Then measure: the angle with SA on vs off and BLA on vs off (Decomposition was gated off for
  "~15% of pixels change with SA", `main.rs:3114-3121`; BLA was never measured). Inside a valid
  skip a bit can flip only where the reference passes within the BLA tolerance of the real axis
  (derived), so BLA may be safe.
- **Tests**: B1 A/Bs for Decomposition with BLA on, at 1e30× and 1e60×, three families.
- **Done when**: a measured verdict on BLA and SA for angle colorings, recorded here.

---

## 5. Phase C: color-pass methods

All need B4 for persistence; their data already exists. Each: shader branch or `ColorU` fields,
UI rows, `.fdn`/farm/CLI through B4, B1 cells, a selftest appearance check, uitest screen.

| Id | Method | Notes | Size |
|---|---|---|---|
| C1 | Interior color for every palette | `interior_col` exists in `ColorU`; Binary/Duotone already have a picker (`ui/panels.rs:313-330`) | S |
| C2 | Transfer functions, wrap modes, banding step | sqrt, cbrt, fourth root, square, log, log-log, atan; repeat, clamp, mirror; step count. Match KF/Mandel Machine names. Live normalization and fit range must apply before the transfer | M |
| C3 | DE darkening, DE transfer curves, edge anti-aliasing | Chéritat's blend `t = d/(thickness·px)`; needs A9 for pixel units | M |
| C4 | Smooth outside, trap inside | under the trap method color escaped pixels by μ; interior palette from B5; prototype worked | S |
| C5 | Lighting: specular, light color, strength, DE-scaled steepness | Relief is already the elevation; add Blinn-Phong on the stored normal with a steepness from the DE | M |
| C6 | Histogram and rank-order coloring with a held CDF | the live fine histogram exists (`wgsl:994-999`); the offline `--normalize` path and tours still use min/max | M |
| C7 | OKLab blending | `tags.rs` has only sRGB→OKLab (362-381): add the inverse; a new blend-space id outside GIMP's `.ggr` set, so `.ggr` export must convert or refuse | M |
| C8 | Palette anti-aliasing in the UI, DE-sized | exists behind `FRACTADYNE_PALETTE_AA`; prototype 2.9× less aliasing at the default density | S |
| C9 | Rainbow fringe and DE-lightness OKLCh mapping | hue from the normal's azimuth, lightness from the DE; prior art zoomasm | S |
| C10 | Level-of-detail equipotential grid | equipotentials only (rays wait for D1); width from \|∇μ\| = 1/(d·ln p); needs A9; prototype held 1.1–1.5 px at eight depths | M |
| C11 | Linked dual-view palettes | the Julia count at z = c is the Mandelbrot count minus 1: a +1 offset and a shared normalization window | S |

Order: C1, C4, C8, C9, C11 (smallest), then C2, C3, C10, then C5, C6, C7.

**Tests specific to this phase**: CPU oracles for C2 transfers and C7 OKLab round trip
(sRGB → OKLab → sRGB within 1e-5); C3 and C10 line widths measured as 2·area/perimeter in B1 at
ss 1 and 2; C6 temporal stability over a zoom tour (frame-to-frame palette shift).

---

## 6. Phases D to G

### D. Escape-data methods (tier 2)

| Id | Method | Needs | Size |
|---|---|---|---|
| D1 | Angle colorings for every method: phase strength (KF), one-step hard binary decomposition, rays for the C10 grid, distance + dwell + decomposition | B8 verdict; hard decomposition today is reachable only with Cycle 0.125, Offset 0 and a two-color "Hard bands" gradient (derived, not run) | M |
| D2 | Exact iteration count past 8.4M | store N and the fraction separately in `main` as the zoom-video strips already do (`strip.wgsl:66-75`); the f32 count loses its fraction at 2^23 | M |
| D3 | Interior period tint, optionally trap-shaded | the exact-repeat check's cycle length into the free interior lanes of `main`; direct and df32 loops only; a Newton pass for deep perturbed interiors later | M |

### E. Statistic combinations (tier 1 after B3)

| Id | Combination | Notes | Size |
|---|---|---|---|
| E1 | Statistic-warped bands | position = rate·μ + offset + λ·(s − center); center fixed at first (no stripe/TIA histogram exists) | S |
| E2 | Statistic near filaments | weight exp(−d_px/k); needs A9 | S |
| E3 | Two-layer blend | statistic over smooth with multiply, screen, overlay, hard light and soft light; second LUT from B5 | M |
| E4 | Statistic bump map | screen-space differences of the statistic (analytic gradients measured as noise), reusing the palette-AA neighbor fetch and the relief low-pass | M |
| E5 | Specular strength from a statistic | after C5 | S |

Every E step shows "slower at depth" in the picker: it needs the statistic, so it pays the aux
restrictions under Smooth.

### F. New per-iteration statistics (tier 3)

| Id | Statistic | Notes | Size |
|---|---|---|---|
| F1 | Exponential smoothing for Newton and convergent formulas | direct loop only, no BLA; can borrow an unused aux lane under Newton | S |
| F2 | Second aux target and a fifth BLA vec4 | all four aux lanes and the node's fourth vec4 are full; touches the seed pass (`wgsl:3668-3681`), the color binding (3649), the G-buffer, strips, bytes per sample, `reference.rs` node stride and `bla_fold.rs` | L |
| F3 | Curvature average | after F2; BLA aggregate like TIA's | M |
| F4 | Parametric traps, new shapes, tail window | center, rotation, scale; line, ring with radius, stalks; tail window (Fractal Zoomer and Fracturing Fog have one); lanes rebuilt per slider change | M |
| F5 | Atom domains and a gated period tint for the exterior | record minimum index and z_p; the hard tint failed on seams, so only the near-filament version | L |
| F6 | Pixel-width traps | save \|dz/dc\| at the trap's best iteration; no derivative inside BLA skips | M |

Every F step is a new GPU regime: rule 11 (capped first run, PLUTO first).

### G. Research methods (tier 4, unproven)

| Id | Method | Notes |
|---|---|---|
| G1 | Interior DE and two-sided glow | read the TODO.md entry first: the math worked and was reverted because the chunked direct render stopped being bit-identical; carry c through the chunk→resolve handoff before re-applying. Also fix the glow's mixed-tap DE average |
| G2 | Interior coordinates (multiplier rings and rays), Lyapunov interior | after G1's period and cycle machinery |
| G3 | Second-derivative shading | under BLA z'' needs A·z'' + C·z'² + E·z' + F, three more coefficients per node (derived, checked numerically) |
| G4 | DE for fold families | Jacobian (2×2) derivatives, as KF did in 2.13.2 |
| G5 | Mandelbrot + Julia ideas | Tan Lei texture transfer (median defect 0.01–0.06 iterations, about 10% of pixels off by more than one), derivative-ratio deviation, post-critical trap for the Julia view |
| G6 | Multi-degree ideas | auto light angle at arg σ + 180°, crossing-point navigator, degree channels |
| G7 | Texture effects | supersample variance, LIC, thin film, material shading, zoom-anchored phase, rebase-event sampling |
| G8 | User-scriptable coloring | a WGSL snippet over the G-buffer; shares the custom-formula codegen |

Each G item starts with a prototype on the B1 view set and a written verdict before any shader
work. The period-locked palette is not listed: its prototype failed.

---

## 7. Testing in detail

### 7.1 Layers

| Layer | What | Where |
|---|---|---|
| CPU oracles | transfers, OKLab, blend weights, LOD level choice, line-width math, period detection | `crates/*/tests/unit/` |
| Layout | every `ColorU`, `IterUniforms` and BLA node change | `gpu/tests/unit/uniform_layout_tests.rs` |
| Selftest | per method: coherent, differs from Smooth, BLA and SA A/B where claimed, determinism, `.fdn` round trip; one golden per method at 1e6× once stable | `harness/selftest.rs`, new tags in `GROUPS` |
| Coloring matrix | families × depth bands × methods: pictures, A/Bs, timings | `--coloring-matrix` (B1) |
| Live | a method tour through the bands; motion and settle with each method | `--livetest tours/coloring-methods.toml` |
| UI | picker, preview sheet, disabled states, `en-XA` | `--uitest` |
| Second machine | the matrix and the live tour on the RTX 3070 and RX 6800 XT | field agent |

### 7.2 Performance budget

Per method and band, from B1 (iterate ms relative to Smooth on the same view):

- color-pass methods (C, E without a new statistic): iterate time unchanged; color-pass time
  under 2 ms at 1080p on the RTX 3080;
- methods that need the aux statistic: report the ratio in every band; the picker's "slower at
  depth" badge comes from these numbers, not from guesses;
- a regression above +35% (and 1 ms) against the card's own baseline is a warning to investigate
  with an interleaved A/B, not a fail.

### 7.3 Looking at results

Every step that changes a picture ends with its contact sheet looked at, at 1:1, before merge;
the step's PR description names the sheet and what was checked. Metrics catch regressions; they
do not prove a picture is right.

---

## 8. The accumulate mask: every switch

| Switch | Location | Today | With the mask |
|---|---|---|---|
| `method_needs_aux` | `gpu/src/lib.rs:3448-3454` | method ∈ {1,2,3,5} | two predicates: accumulates (mask ≠ 0) and display reads aux |
| `aux_on_word` | `gpu/src/lib.rs:3440-3446`; `wgsl:863-866` | bit 0 aux, bit 1 tail, bits 2.. length | bit 0 from the mask; tail gated on the stripe bit (`wgsl:1357-1359`); mask the shift at 1361 |
| `IterUniforms.color_method` | `gpu/src/lib.rs:69`; `wgsl:860` | read at 1012, 1358, 1381 | becomes the mask; setters `lib.rs:3969-3976`, `export.rs:1789-1796` and the three aux-off paths (`export.rs:2728-2731, 3288-3291, 3581-3584`) |
| `aux_step` / `aux_pack` | `wgsl:1374-1436`; `gpu/src/custom.rs:1097-1110` | selected statistic only | per bit |
| `IterKey` | `gpu/src/lib.rs:422-457` | method, stripe freq, trap type | mask plus enabled parameters (plus A4's tail fields) |
| `settings_hash` | `render.rs:8585-8621`; walk signature `render/decide.rs:1956-1977` | method and parameters | mask plus enabled parameters |
| `bla_eligible` / `aux_bla_ok` | `render.rs:3683-3701` | method ∈ {1,2,3} | per-bit fold table |
| SA (`do_sa`) | `render.rs:3177-3181, 3452-3456, 3873-3878, 9286-9292` | `!blocks_iter_skip` | from the mask |
| Aux staleness | `render.rs:9939-9988`; `live_ref_fields` 3644-3653 | method 1 + freq, method 3 + trap | stripe bit + freq, trap bit + trap |
| Chunked walk | `render/frame_plan.rs:396-399`; `build_params` 8582-8584, 8757-8760 | `!needs_aux` | mask = 0 (or angle-only after B8) |
| Band BLA | `render.rs:6752-6762` | `!needs_aux` | from the mask |
| Corrector, normalized export | `render.rs:4858, 4967` | refuse aux methods | refuse mask ≠ 0 |
| Renormalized step | `wgsl:1077-1080` | off when aux | off when mask ≠ 0 |
| Trap histogram / fit | `wgsl:1011-1017`; `render.rs:2202-2212, 2279-2287` | method 3 | trap bit and display = trap |
| Multi-GPU twin | `motion_worker.rs:312` | `method_needs_aux` | accumulates predicate |
| Strips and assembler | `assemble.rs:735-739`; `gpu/src/assemble.rs:60-89` | refuse aux | display predicate |
| Reference cache | `main.rs:5728-5735`; `render.rs:434-441, 10279-10287, 9120` | stripe freq and trap type always compared | only enabled bits |
| Accumulation signature | `main.rs:16163-16219` | method and parameters | add the mask |

---

## 9. UI design

### 9.1 Choosing a method

The Coloring section keeps its place (`ui/panels.rs:166-407`). Top to bottom:

1. **Method**: a combo whose list is grouped under small headings (`menu_section` style):
   - Escape count: Smooth (default), and with C2 the transfer appears as a setting, not a method;
   - Distance: Distance estimate, Rainbow fringe (C9);
   - Orbit statistics: Stripe average, Triangle inequality, Curvature (F3), Orbit trap;
   - Angle: Decomposition, Phase (D1);
   - Layers (E): Warped bands, Statistic near filaments, Two-layer blend.
   Each item has a one-line tooltip: what it shows, then its cost from the B4 table, for example
   "Recolors instantly", "Re-renders the view", "Slower past 1e4× (no series approximation)".
   Disabled items say why ("Needs a derivative; Burning Ship has none").
2. **Method settings**: only the chosen method's rows (stripe density and tail, trap shape and its
   parameters, layer weight).
3. **Inside** (B5): a small combo, Solid color (picker), Trap, Period (D3), later Distance (G1),
   with the interior palette below it when it is not solid.
4. **Palette** and its rows, unchanged.
5. **Preview methods…** button with the `GALLERY` icon, beside the method combo; also Color ▸
   Preview methods… in the menu.

The Effects section gains specular and light color (C5), DE darkening (C3) and the grid (C10).

### 9.2 The method preview sheet

- An `egui::Window` with id `fractadyne.method-preview`, laid out like the palette browser: a grid
  of cells, each the current view (view 0) in one method, labeled with the method name and a cost
  badge; a row per group; the current method outlined.
- Cells that recolor from the G-buffer appear at once. Cells that must re-render show a progress
  spinner and fill in one per frame; at depth, a cell whose estimate exceeds a budget (for
  example 2 s) shows the estimate and a Render button instead.
- Hover previews the method on the main view after 0.3 s (the palette preview rules); moving off
  restores; click applies and keeps the sheet open; Close closes.
- Parameter variants appear as extra cells where they matter: the three trap shapes, stripe
  density 3 and 6.
- Invalidation: the view's writes, `accum_color_sig`, the mask. While the user zooms or pans, the
  sheet pauses re-renders and redraws recolor cells at most every 400 ms, like the palette atlas
  (`REDRAW_PACE`).
- Dual view: the sheet previews view 0; a later option can switch it to the Julia panel.

### 9.3 View rotation (planned separately; where it meets this work)

Rotation (TODO.md, "Rotation for views", user 2026-11-01) is its own project, but it touches
coloring in these places, so sequence the two with care:

- **The distance estimate's pixel size.** `de_log2` takes the texel size from `abs(iu.step.x)`
  (`mandelbrot.wgsl:646-657`). If rotation turns the per-texel step into a rotated complex step,
  that must become the step's LENGTH, or every distance is wrong by `|cos θ|` (a 45° view reads
  distances half an octave off). A distance itself is rotation-invariant, so A9's per-pixel
  correction needs no change.
- **Normals and the light.** The stored slope normal is in plane coordinates. Decide whether the
  light stays fixed on screen (rotate normals by the view angle before lighting) or turns with the
  image; the same choice applies to specular (C5), bump maps (E4) and the auto light angle (G6,
  arg σ is a plane angle).
- **Angle colorings** (Decomposition, D1, the grid's rays in C10) read arg z, which does not
  change with the view; they turn with the image, which is right.
- **The `.fdn` file.** Rotation adds a view key next to A1's coloring keys: do them one after the
  other, with one writer-⊆-reader update each.
- **Reprojection** (the frozen-frame uv transform, `ColorU.uv_off`/`uv_scale`) gains a rotation;
  A9's zoom factor is unaffected because rotation does not change texel size. The method preview
  sheet (B7) and the palette atlas reuse the live texture and are unaffected.
- **Screen-space differences** (palette anti-aliasing's +x/+y footprint, the relief's 3×3
  low-pass, E4) stay in screen pixels and need no change.
- **Tests.** B1's view set gets one rotated view per family (30° and 45°); the angle colorings and
  lighting are checked there.
- **Importers.** `.kfr` carries `RotateAngle`; the importer reads only center, zoom and iterations
  today.

### 9.4 Consistency

- Every method has a key, a label, a tooltip, a capability row and a selftest tag; adding a method
  without all five fails a unit test that walks `ColorMethod::ALL`.
- Labels fit the 104 px label column (`ui/panels.rs:5-32`); check in `en-XA`.

---

## 10. Status

| Step | State | Commit |
|---|---|---|
| A9 | done, merged | `33c6527e` (merge `3d202f30`) |
| A6 | done, merged | `2c898852` (merge `38d93def`) |
| A2 | done, merged | `3e5aa578` (merge `2a14e005`) |
| A4 | done, merged | `475e2190` (merge `b9812c74`) |
| A8 | done, merged | `095a7131` (merge `1db50a24`) |
| A1 | done, merged | `c15bd9c7` (merge `aa5b22e7`) |
| A3 | done, merged | `3ee690d7` (merge `3c8a59e7`) |
| A5b | done, merged | `fddc6ada` (merge `e11d1890`) |
| A5a | done, merged | `05306f71` (merge `45c2e84d`) |
| A7 | done, merged | `603275eb` (merge `1bfdc833`) |
| A10 | done, merged | `aaffca7f` (merge `6ec2fc76`) |
| B1 | done, merged (agent v41, deployed) | `312c54a8` (merge `4efdbbb3`) |
| B1 fixes | Distance NaN; Newton's statistics methods | merges `91fa8f91`, `7722b660` |
| B1b | not started: the live method tour | |
| B2 | done, merged | `c017a577` (merge `dcfd776b`) |
| B3 | done, merged | `412fafc6` (merge `542a5264`) |
| B4 | done, merged | `a5d7d74b` (merge `ba452e9c`) |
| B6 | in progress, branch `feat/color-b6-method-picker` | |
| B5, B7, B8 | not started | |
| C1–C11 | not started | |
| D, E, F, G | not started | |
