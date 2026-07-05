# Fractadyne Refactor Plan

**Status:** proposed · **Baseline:** v0.1.27 (`a681f06`) · **Author:** derived from the 2026 organization audit (see the audit summary in the project history)

This plan turns the audit findings into a **sequenced, low-risk, verifiable** refactor. It is ordered so that each phase installs the safety net or structural scaffolding the next phase depends on. Nothing here changes rendering behavior — the golden-image self-test is the invariant that proves it.

---

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
5. Convert the three `#[allow(clippy::too_many_arguments)]` core fns to parameter structs (`orbit_length_bf`, `best_reference`, `render_multiref_mandel` at core `lib.rs:1271/1314/1708`) — removes the suppressions.
6. Add `SAFETY:` comments to the three `sysinfo.rs` FFI blocks; annotate the safe-by-construction `unwrap`s in `to_f64`/`build_bla_mandel`.
7. Round out `[workspace.package]` metadata (`homepage`/`documentation`/`keywords`/`categories`) and comment the machine-specific `debug=false` profile.

**Decouple the `FloatExp` operator naming** (clippy `should_implement_trait`, 5 sites): implement `std::ops::{Add,Sub,Mul}` for `FloatExp`/`CFloatExp` **or** rename the inherent methods (e.g. `mul` → `times`). Prefer implementing the ops traits — it is both idiomatic and removes call-site noise. *(This one touches many call sites in core; keep it a separate commit within Phase 0 and lean on the goldens.)*

**Risk:** very low. **Effort:** Small (S). **Gate:** clippy clean becomes enforceable from here on.

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
