# Fractadyne Refactor Plan

**Status:** proposed · **Baseline:** v0.1.27 (`a681f06`) · **Author:** derived from the 2026 organization audit (see the audit summary in the project history)

This plan turns the audit findings into a **sequenced, low-risk, verifiable** refactor. It is ordered so that each phase installs the safety net or structural scaffolding the next phase depends on. Nothing here changes rendering behavior — the golden-image self-test is the invariant that proves it.

---

## Guiding goals (from the maintainer)

1. **Human understanding & maintainability** come first — favor readability wins (named operators, glossaries, single-source-of-truth tables, clear module boundaries) over merely satisfying the compiler.
2. **Make adding a new formula more straightforward** — reduce the number and spread of edit sites, and document the ones that remain. Weight work toward this.

## Progress log

- ✅ **Phase 0** (v0.1.28) — lint gate, panic hardening, readability, `TOURS.md` fix.
- ✅ **Phase 1a-i** — `FloatExp`/`CFloatExp` arithmetic migrated to `std::ops` operators (reads `a * b - c`).
- ✅ **Phase 1a — core split COMPLETE.** `core/lib.rs` decomposed from a 2830-line monolith into `floatexp` (219), `bignum` (258), `viewport` (244), `reference` (1103); `lib.rs` is now a thin facade (module decls + re-exports + formula ids + tests). All re-exported so callers are unchanged; each extraction gated (tests 35/35, selftest 55/55, goldens 17/17).
- ✅ **Phase 1c — GPU export de-dup COMPLETE.** The iterate/color bind-group layouts + shader module were recreated verbatim in 4 places; extracted `shader_module` / `iter_bind_group_layout` / `color_bind_group_layout` factories (~120 lines of copy-paste removed, layouts can no longer drift). Gated (selftest 55/55, goldens 17/17, `render_iter` sanity-checked).
- ✅ **Phase 1b — GPU export split COMPLETE.** Extracted `gpu::export` (`render_export`/`render_iter`/`color_iter_buffer` + `ExportRequest`/`Result`, 715 lines); `gpu/lib.rs` dropped 1420 → 718 (the live-view `Renderer`/`ViewResources`/`CallbackTrait` + shared scaffolding, now `pub(crate)`). Gated (selftest 55/55, goldens 17/17, `render_iter` sanity-checked). *(The remaining 718-line `lib.rs` is cohesive; further splitting it into `uniforms`/`pipeline`/`live` is optional polish.)*
- ✅ **ARCHITECTURE.md** brought current with the refactor.
- 🎉 **Phase 1 (library decomposition) is complete** — core split (4 modules), GPU export split, and the GPU export de-dup all landed.
- ✅ **Phase 2b — `update()` dialog pass COMPLETE.** All **10 modal dialogs** (goto, share, reset, toast, bookmarks, 3× benchmark, gallery, export) extracted from the ~2300-line `update()` into `draw_*_dialog` methods on a dedicated `impl FractadyneApp` block; `update()` now calls them as a flat sequence and is down to **~1500 lines** (~675 lines of inline `egui::Window` code moved out). Gated each step (selftest 55/55, goldens 17/17, clippy 0). *Remaining Phase 2: extract the **panels** (menu bar / right coloring panel) from `update()`, then the **field-grouping** (2a) of the ~165-field struct — which unblocks pulling the dialog/panel methods into a real `fractadyne-ui` crate.*
- ✅ **Formula-ease (goal 2), app side** — all per-family metadata consolidated into one `FractalKind::SPECS` table (one row per formula) + an authoritative "Adding a new formula" checklist + guard tests.
- ✅ **Formula-ease, core side** — canonical `core::formula::{…}` id constants (single source of truth for the numbering) + `is_valid_formula` + a core checklist mirroring the app one; **adopted the constants at every CPU dispatch site** (`step_bf`/`orbit_points`/`reference_orbit`/`series_skip`), so the arms read `formula::PHOENIX => …`.
- ✅ **Formula-ease, shader side** — id→family legend + add-a-formula note at the WGSL `formula` uniform, stating the ids must match `core::formula` / `FractalKind::formula_id`. All three dispatch sites (app table · core constants · shader legend) now cross-reference one checklist.
- ✅ **Golden coverage — direct path for every family + deep path for polynomials** (17 goldens, each visually verified):
  - A **direct-mode overview** per family (Multibrot 3/4/5, Tricorn, Burning Ship, Celtic, Buffalo, Phoenix, Newton) — guards each formula's direct shader dispatch (was Mandelbrot-only before).
  - A **deep mode-0 (df32 perturbation, 1e6×)** golden for each polynomial family (Mandelbrot, Multibrot 3/4/5) at a bisected boundary coordinate — guards the bignum reference orbit (`step_bf`) + series approximation + the mode-0 shader branch. Coordinates via core's `dump_deep_boundary_coords` utility.
  - **Deferred deep tiers** (investigated): (a) deep goldens for the **abs families** (Burning Ship/Celtic/Buffalo) show fold **glitch-speckle** at deep perturbation — enshrining that is undesirable; wait for multi-reference glitch correction. (b) A **mode-2 / floatexp (~1e30×)** tier needs a coordinate accurate to ~1e-30 (≈70k-iter bignum bisection) **and** ~70k render iters per golden (slows every selftest run) — a real cost; `find_nucleus` minibrots are the cheaper path for polynomials when this is tackled.
- ⏭️ **Next for formula-ease:** a `Fractal`-trait PoC (Phase 5 pulled forward) unifying the CPU step functions so a family's math is defined once — now guarded by the per-family goldens above.

## Guiding principles

1. **Behavior-preserving.** Every phase is a pure reorganization or a mechanical, semantics-preserving change. The observable output (rendered pixels, exported files, session round-trips) must not change.
2. **Verified after every step**, not just at the end. The acceptance gate for *every* commit is:
   - `cargo build -j1 --workspace` clean,
   - `cargo clippy -j1 --workspace --all-targets` clean (once Phase 0 lands),
   - `cargo test -j1 --workspace` green,
   - `fractadyne --selftest` → **55/55 checks + 4/4 golden images bit-identical**.
   The goldens (`validation/golden/*.png`) are the tripwire: any accidental behavior change fails them.
3. **Branch per phase.** Each phase is a short-lived branch merged to `main` only when the gate is green. Small commits within a phase.
4. **Re-exports keep callers stable.** Library decomposition moves code into submodules but re-exports the public API from `lib.rs`, so downstream crates need zero changes.
5. **Version discipline.** Bump `Cargo.toml` patch per functional milestone (per project policy). Refactor-only commits can share a version; note them in `CHANGELOG.md`.

> **Build constraints (unchanged):** `-j1`, no debuginfo (page-file limit), release exe lock released via PowerShell `Stop-Process -Name fractadyne -Force` before a release build. No AV exclusions.

---

## Target architecture (before → after)

**Crate graph** stays acyclic and downward-only; the change is *where the mass lives*.

```
BEFORE (mass concentrated in the binary)          AFTER (mass pushed down into libraries)
------------------------------------------        ---------------------------------------
fractadyne-app  ~12k lines (fat binary)           fractadyne-app   ~2–3k lines (shell + glue)
  main.rs 5986  (god struct + update())              main.rs        (App wiring, event loop)
  scripting/render/export/selftest/... inline        ui/            (menus, panels, dialogs)   ← was inline
fractadyne-core  2748 (one flat file)             fractadyne-ui    (egui panels/dialogs)        ← was empty stub
fractadyne-gpu   1495 (one flat file)             fractadyne-render/scripting (tours/overlays)  ← was empty stub
fractadyne-render/-ui/-fractals = empty stubs     fractadyne-core  core::{floatexp,viewport,reference,nucleus,multiref}
                                                  fractadyne-gpu   gpu::{uniforms,pipeline,live,export}
```

---

## Phase 0 — Guardrails & mechanical cleanup (do first)

**Why first:** installs the lint gate that guides every later phase, and clears the cheap noise so real regressions stand out. Zero-to-tiny code change; unblocks everything.

**Steps**
1. Add a root `[workspace.lints.clippy]` table and have each crate opt in with `[lints] workspace = true`. Start permissive, then tighten:
   ```toml
   [workspace.lints.clippy]
   too_many_arguments = "warn"
   needless_range_loop = "warn"
   # ... plus a policy decision on `unwrap_used` (allow in tests, warn elsewhere)
   ```
2. Add a `rustfmt.toml` (even if empty/defaults) to pin formatting.
3. Run `cargo clippy --fix` to auto-clear the ~12 mechanical warnings (`needless_range_loop`, `manual_clamp`, `manual_div_ceil`, `unnecessary_map_or`, `useless_format`, `needless_borrow`, `unnecessary_sort_by`, `is_multiple_of`, `bool_assert_comparison`, `field_reassign_with_default`, `unnecessary_cast`, `empty_line_after_doc_comments`, `print_literal`).
4. Fix the two **shipped-path panic risks**:
   - `main.rs:1294` — replace `.expect()` on `wgpu_render_state` with a graceful error (dialog/log + clean exit) for the "no GPU backend" case.
   - `scripting.rs` encoder pool — replace `lock().unwrap()` with poison-recovering access (`lock().unwrap_or_else(|e| e.into_inner())`) so a worker panic doesn't cascade-crash the tour render.
5. Add `SAFETY:` comments to the three `sysinfo.rs` FFI blocks; annotate the safe-by-construction `unwrap`s.
6. Round out `[workspace.package]` metadata (`homepage`/`documentation`/`keywords`/`categories`).
7. Add a `core` module-header **abbreviations glossary** (`p`, `RM`, `df64`, `zx/zy/dzx/dzy`, …) — a pure readability win the audit called for.
8. Regenerate `TOURS.md` from the schema so the `tour_schema_doc_current` guard test passes (it had drifted).

**Deliberately deferred out of Phase 0** (documented `#[allow(...)]` with a plan reference at each site, so the lints stay enforceable for new code):
- **`FloatExp`/`CFloatExp` → `std::ops` operators** (clippy `should_implement_trait`, 5 sites). This is a real readability win but the migration touches **~100 precedence-sensitive call sites** (`a.add(b).mul(c)` ≠ `a + b * c`), so it moves to **Phase 1**, done carefully alongside the `floatexp` module extraction with the goldens as the gate.
- **The three core param-struct conversions** (`orbit_length_bf`/`best_reference`/`render_multiref_mandel`) → **Phase 1** (they belong with the core decomposition; `best_reference` also crosses into `app`, so its param struct becomes part of the tidied public API). These stay `#[allow]`'d meanwhile.
- **The ten `app` `too_many_arguments` sites** → **Phase 2c/3** (rasterization primitives fold into the overlay module; `render_tour_to_dir` extracts with the scripting crate).

**Status:** ✅ **completed** at v0.1.28 — build + clippy (0/0) + 46 unit tests + `--selftest` (55/55, goldens 4/4) all green.

**Risk:** very low. **Effort:** Small (S). **Gate:** clippy clean is now enforceable.

---

## Phase 1 — Library decomposition (mechanical, high value)

**Why now:** purely moving code into submodules with re-exports. Low risk (compiler + goldens catch everything), and it makes Phases 4–5 tractable by giving each concern a home.

### 1a. Split `fractadyne-core/src/lib.rs` (2748 → ~6 modules)
Introduce submodules, keep `lib.rs` as a thin facade that `pub use`s the entry points so **no external caller changes**:

| New module | Moves (current lines) | Contents |
|---|---|---|
| `core::bignum` | 1–266 | precision scaling, `BigFloat` helpers, KFR import, `to_f64`/`parse_bf`/`to_decimal_string` |
| `core::floatexp` | 267–421, 1063–1076 | `FloatExp`, `CFloatExp` (+ the new ops-trait impls from Phase 0) |
| `core::viewport` | 422–658 | `Viewport` + coordinate math |
| `core::reference` | 659–1173, 1174–1306 | reference orbit, perturbation, series approximation, BLA tree |
| `core::nucleus` | 1307–1579 | period detection, nucleus finding |
| `core::multiref` | 1580–1796 | Pauldelbrot glitch detection + multi-reference correction |

Move each module's tests into a `#[cfg(test)] mod tests` inside that module (splitting the 950-line test block by domain). Document the abbreviation glossary (`p`, `RM`, `df64`, `zx/zy/dzx/dzy`) in the `core` crate `//!` header.

### 1b. Split `fractadyne-gpu/src/lib.rs` (1495 → ~4 modules)

| New module | Contents |
|---|---|
| `gpu::uniforms` | `IterUniforms`, `ColorUniforms`, `IterKey`, `MandelbrotParams`, `ExportRequest`, constants (formats, thresholds) |
| `gpu::pipeline` | `fullscreen_pipeline`, `make_iter_texture/orbit_buffer/*_bg`, and a new `RenderPipelineSet` factory (see 1c) |
| `gpu::live` | `Renderer`, `ViewResources`, `install_renderer`, `add_mandelbrot` |
| `gpu::export` | `render_export`, `render_iter`, `color_iter_buffer`, `ExportResult` |

### 1c. De-duplicate the GPU export path *(the one real code change in Phase 1)*
- **Compile `mandelbrot.wgsl` once.** Today it is `include_str!`'d and recompiled at 4 sites (`lib.rs:252/798/1099/1305`). Store one `ShaderModule` (in the crate init / a `OnceCell`, or pass it in) and reuse.
- **`RenderPipelineSet` factory.** `render_export` / `render_iter` / `color_iter_buffer` each rebuild ~80% identical bind-group + pipeline layouts. Extract `fn make_render_pipelines(device, shader) -> RenderPipelineSet`.
- **Shared render-and-readback** routine parameterized by a pass-builder closure, so the three export fns keep only their genuine differences (target format, tiling, which passes run).
- *Verification is critical here:* `render_export` feeds the goldens and `render_iter` feeds `--validate-deep`; both must stay bit-identical.

**Risk:** low (1a/1b), medium (1c — behavior-bearing). **Effort:** Medium (M). **Gate:** goldens + `--validate-deep` + `--selftest`.

---

## Phase 2 — App decomposition (the big one, incremental)

**Why now:** with libraries tidy, attack the god object. Do it **field-group by field-group** so it never becomes a big-bang rewrite. This is the highest-impact maintainability work.

### 2a. Group `FractadyneApp`'s ~165 flat fields into sub-structs
Introduce nested structs and migrate one group per commit (update all `self.x` → `self.group.x` for that group, compile, gate, commit):

| Sub-struct | Fields absorbed |
|---|---|
| `ViewState` | `viewport`, `julia_viewport`, `fractal`, `julia_mode`, `julia_c`, dual-view flags |
| `AnimationState` | zoom/pan velocity, `orbit_anim*`, `palette_anim*`, home-lerp |
| `AutopilotState` | `autopilot_*`, dive limit, stepping |
| `ColoringConfig` | `palette_idx`, custom/duotone/binary palette, `color_method`, `stripe_freq`, `trap_type`, `cycle`, `offset`, light/DE settings |
| `RenderConfig` | `max_iter`, `auto_iter`, `aa`, `work_budget_scale`, `glitch_correct` |
| `ExportUiState` | `export_format/width/aspect/notes/last_dir/started/...` + task/prep/progress |
| `DialogState` | `*_open` flags (or an `ActiveModal` enum — one modal at a time) |
| `NavHistory` | nav stack + position |
| `CliMode` (enum) | the 13 `auto_*` automation flags collapsed into one state enum |

Target: the top-level struct drops from ~165 fields to ~10 grouped members + a handful of caches.

### 2b. Split `update()` (2300 lines → orchestrator < 500 lines)
Extract cohesive methods (and/or a `ui/` submodule tree), leaving `update()` as a thin dispatcher:
- `handle_cli_modes(ctx, gpu) -> ControlFlow` (early-return automation path) → into `cli.rs`
- `advance_animations(ctx)` and `poll_background_tasks(ctx, gpu)` (export task/prep, tour playback)
- `ui_menu_bar` / `ui_status_bar` / `ui_right_panel` / `ui_central` → new `ui/menus.rs`, `ui/panels.rs`, `ui/central.rs`
- `ui_dialogs` or a `DialogManager` → `ui/dialogs.rs` (~1200 lines of dialog code)

Do menus, then panels, then dialogs — one extraction per commit.

### 2c. Consolidate overlay/annotation rasterization
Today HUD/caption/callout/orbit live in `scripting.rs` while watermark lives in `export.rs`, and `export.rs` reaches into `scripting.rs` internals. Merge into one `overlay` module with a unified `Overlay`/`AnnotationSet` type and a single `stamp()` entry point. This also **unblocks Phase 3** (extracting a scripting/tour crate).

**Risk:** medium — mechanical but touches the most code. The goldens + a manual GUI smoke test per UI extraction are the safety net. **Effort:** Large (L), but fully incremental. **Gate:** goldens + `--selftest` + manual GUI pass (menus/panels/dialogs still work).

---

## Phase 3 — Populate (or retire) the stub crates

**Why now:** the app is grouped and the overlay code is consolidated, so extraction is finally mechanical rather than surgical.

**Steps**
1. **Decide the stubs' fate up front.** Either:
   - **(a)** delete `fractadyne-render`/`-ui`/`-fractals` from workspace members and recreate when work begins, **or**
   - **(b)** keep them, add `publish = false` + a `README` mapping each to its current home.
   Recommended: **(a)** for `-fractals` (superseded by the Phase-5 trait), **populate** `-ui` and a `-scripting`/`-render` crate.
2. **Fix the internal leak.** `ExportPrep`/`RecomputeResult` currently expose `std::sync::mpsc::Receiver` in the shared surface (`render.rs:18`). Wrap the channel behind a method-based API before moving code across a crate boundary.
3. **Extract `fractadyne-ui`.** Move the `ui/` submodule tree (menus/panels/dialogs) into the crate. `App` passes a borrowed context/state view; UI functions take `&mut` slices of the grouped sub-structs rather than the whole god object — now possible thanks to Phase 2a.
4. **Extract scripting/tours** into `fractadyne-render` (or a new `fractadyne-scripting`): `Playback`, tour rendering, overlay module, MP4 orchestration.

**Risk:** medium — crate boundaries force honest API design (a feature, not a bug). **Effort:** Large (L). **Gate:** full suite + GUI smoke test.

---

## Phase 4 — Type safety & structured errors

**Why now:** with code in its final homes, tighten the contracts.

**Steps**
1. **Replace bare `u32` dispatch with enums.** `mode`, `formula`, `julia`, `color_method`, `trap_type` become real enums in `core`/`gpu`, serialized to `u32` only at uniform-write time. Add `is_valid_formula` + a documented **formula-support matrix** (which families support perturbation vs series approximation).
2. **Newtypes for packed arrays.** `ref_offset: [f32;4]` (df32 pair), `span_mantissa: [f64;2]` (x/y), palette-stop arrays — wrap so transposition is a type error, not a silent bug.
3. **Structured errors.** Add `thiserror`; define `GpuError`, `ExportError`, `AppError` replacing `Result<T, String>`. Make export `read_*` return `Result<_, ExportError>` with variant-specific detail (not-found / bad-format / size-mismatch) instead of collapsing to `None`.
4. Stop silently swallowing I/O: replace `let _ = fs::write(...)` (e.g. `main.rs:1949`) with surfaced status.

**Risk:** medium (wide but mechanical churn). **Effort:** Medium (M). **Gate:** full suite.

---

## Phase 5 — The `Fractal` trait (largest, last)

**Why last:** highest design leverage but also highest risk; it wants every prior phase (enums, tidy core modules, structured errors) in place first. It touches core + gpu + app together.

**Steps**
1. Define `trait Fractal { fn id/name/info; fn supports_perturbation/series; fn step_bignum(...); fn step_f64(...); ... }` and, where the GPU is involved, a way to select the shader branch. Model the shape on the DESIGN.md §4 intent.
2. **Migrate Mandelbrot first** as a proof of concept behind the trait, keeping the enum path alive for the other families.
3. Migrate the remaining families one at a time; delete each family's scattered `match` arm as it moves. End state: adding a formula is one `impl Fractal`, not coordinated edits across `step_bf`, `orbit_points`, `series_skip`, three WGSL modes, help text, and info metadata.
4. Interim mitigation *(if Phase 5 is deferred):* add a **"adding a formula" checklist** comment in `core` listing all ~6 sites that must change in lockstep.

**Risk:** high. **Effort:** Large (L). **Gate:** goldens across *all* fractal families + `--validate-deep`.

---

## Cross-cutting: test & doc gaps (fold into each phase)

- Add round-trip/reference-value unit tests to **`fractadyne-export`** (PNG/EXR/sRGB transfer, thumbnail edge cases) and **`fractadyne-color`** (`Palette::packed`) — both currently have **zero** tests. Do this in the phase that touches each crate.
- Add pure-function tests for scripting `Playback::sample` and easing (Phase 3).
- Add doc comments to public `Viewport`/`FloatExp` methods, `SessionState`/`FractalInfo` fields, and `gpu` `pub` fns (Phases 1–2).
- Reconsider whether **`fractadyne-color`** (73 lines) earns its own crate or folds into `core`/`gpu` (decide in Phase 3).

---

## Sequencing summary

| Phase | Theme addressed | Risk | Effort | Depends on |
|---|---|---|---|---|
| 0 | Lint gate, panic fixes, mechanical cleanup | very low | S | — |
| 1 | Split core & gpu into modules; de-dup GPU export | low–med | M | 0 |
| 2 | Group app struct; split `update()`; overlay module | medium | L | 1 |
| 3 | Populate/retire stub crates; fix channel leak | medium | L | 2 |
| 4 | Enum dispatch, newtypes, `thiserror` | medium | M | 1, 3 |
| 5 | `Fractal` trait | high | L | 4 |

**Recommended cadence:** land Phase 0 immediately (a day or less of work; it pays for itself by making every later diff clippy-verified). Phases 1 and the field-grouping half of Phase 2 are the sweet spot — mechanical, compiler-checked, and they remove most of the day-to-day friction. Phases 3–5 are larger investments to schedule against feature roadmap milestones (they map to the DESIGN.md M4+ line).

## Safety net (applies to all phases)

- **Goldens** (`validation/golden/*.png`) + `--selftest` (55 numeric checks) after every commit — the primary behavioral tripwire. Re-bless goldens **only** for an intentional visual change, never to paper over a refactor.
- **`--validate-deep`** for anything touching `core::reference`/`gpu::export`.
- **`cargo clippy` clean** enforced from Phase 0 onward.
- **Branch per phase**, small commits, GUI smoke test after any `update()`/UI extraction (not headless-testable).
