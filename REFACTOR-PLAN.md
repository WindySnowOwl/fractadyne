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
- 🎉 **Phase 2b — `update()` split COMPLETE.** Extracted **all 10 modal dialogs** + the **menu bar**, **right control panel**, **status bar**, and **central fractal area** from the ~2300-line `update()` into methods on a dedicated `impl FractadyneApp` block. **`update()` is now ~426 lines** (under the plan's <500 target): setup + background polling + a flat sequence of `self.draw_*()` calls + the perf/nav tail. Large blocks were moved *mechanically* (verbatim `sed` moves — no transcription risk); every step gated (selftest 55/55, goldens 17/17, clippy 0).
- 🎉 **Phase 2a — field grouping COMPLETE (13 sub-structs).** The ~165-field `FractadyneApp` god-struct is decomposed one cohesive group per gated commit. Earlier: `HomeAnim`, `GotoDialog`, `ShareDialog`, `BenchConfig`, `GalleryState`, `NavHistory`, `AutopilotState`, **`ExportState`** (14 fields, 5 files — the churniest), **`EffectsConfig`**. Final pass (this session): **`PointerState`** (9 transient zoom/pan/settle), **`RenderConfig`** (8 persisted compute knobs; handled the `app.use_bla`/`app.glitch_correct` CLI-rebindings), **`ColoringConfig`** (14 palette/duotone/method fields; hand-fixed the `duo_hash` multiline chain), **`AnimationState`** (11 orbit/palette-anim; `\b`-anchored the `orbit_anim`/`_speed` prefix pairs), **`DialogState`** (8 open-flags; `\b` protected the `minimap` toggle from the `minimap_tex`/`_key` caches). A collision-mapping sub-agent pass drove the final five cleanly (per-field method-collision / `new()`-rebinding / prefix-ordering / multiline analysis → each compiled first try). Pattern proven across every variation; every group gated (clippy 0/0, selftest 55/55, goldens 17/17). Core view state (`viewport`/`fractal`/`julia_*`/`dual*`), the CLI-headless one-shots, and the caches (`orbit_cache`/`ref_cache`/`thumb_cache`/`minimap_*`) are deliberately left flat.
- 🎉 **Phase 3 COMPLETE** (a feasibility analysis *overrode the original plan* — see the revised Phase 3 section): (1) **UI decomposition via an *intra-crate* `src/ui/` split, not a `fractadyne-ui` crate** — the 18 `draw_*` methods have a maximally-wide API surface (~40 fields incl. the flat `viewport`/`fractal`/`julia`, ~50 cross-subsystem calls), so no clean crate boundary exists; a crate would force pub-widening all app state / a crate cycle / an `app→ui` inversion. Moved them verbatim into `fractadyne-app/src/ui/{dialogs 824, central 757, menus 521, panels 321}.rs` as `impl FractadyneApp` submodule blocks (only edit: `pub(crate)` + `use crate::*;`). **main.rs 6305 → 3915 lines (−38%).** (2) **Retired the `-ui` and `-fractals` stub crates** (empty, zero dependents; `-fractals` superseded by the Phase-5 trait); kept `-render` as a `publish=false` + README marker; `-color` stays. Workspace 9 → 7 crates. (3) **Reclassified the "mpsc leak"** as intentional bounded fire-and-forget (not a leak) with intent comments at the spawn sites. Every slice gated (clippy 0/0, selftest 55/55, goldens 17/17).
- 🔬 **Phase 5 — narrow `Field`-generic-step PoC DONE** (a design analysis again *scoped down the plan*: a full four-method `Fractal` trait is mostly ceremony — metadata is already SPECS-unified, `series_skip` is already degree-generic, `formula_power` is 5 lines, Newton can't join, and even a perfect trait only reduces ~7 edit-sites to ~3 since the 3 WGSL branches + SPECS never join a Rust trait. The **one** part that genuinely pays is a `Field`-generic `step<F>` that de-duplicates the *three hand-copied* recurrences — f64 `orbit_points`, `BigFloat` `step_bf`, and the perturbation delta — into one text). New `core/src/fractal.rs`: a `Field` trait (`f64` + `BigFloat`), a generic `cmul`, and `Fractal::step<F>` for **Mandelbrot + Multibrot 3/4/5 + Burning Ship**, dispatched by `trait_step` and wired into `step_bf` + `orbit_points` behind a `if let Some(r) = trait_step` shim (their inline arms deleted; Tricorn/Celtic/Buffalo/Phoenix/Newton stay on the enum path). **Bit-identical** — proven by the exact SA cross-check unit tests, a new f64 bit-for-bit test over all 5 families, the selftest's Multibrot SA-match checks, and the deep polynomial goldens (clippy 0/0, core 36/36, goldens 17/17). The abstraction reproduced the exact op sequence with no contortion → per its stop-condition, it greenlights migrating the remaining escape families (Tricorn/Celtic/Buffalo next; Phoenix needs prev-threading; Newton stays out). *The refactor's six phases are now all landed (Phase 5 as a validated PoC).*
- ✅ **Formula-ease (goal 2), app side** — all per-family metadata consolidated into one `FractalKind::SPECS` table (one row per formula) + an authoritative "Adding a new formula" checklist + guard tests.
- ✅ **Formula-ease, core side** — canonical `core::formula::{…}` id constants (single source of truth for the numbering) + `is_valid_formula` + a core checklist mirroring the app one; **adopted the constants at every CPU dispatch site** (`step_bf`/`orbit_points`/`reference_orbit`/`series_skip`), so the arms read `formula::PHOENIX => …`.
- ✅ **Formula-ease, shader side** — id→family legend + add-a-formula note at the WGSL `formula` uniform, stating the ids must match `core::formula` / `FractalKind::formula_id`. All three dispatch sites (app table · core constants · shader legend) now cross-reference one checklist.
- ✅ **Golden coverage — direct path for every family + deep path for polynomials** (17 goldens, each visually verified):
  - A **direct-mode overview** per family (Multibrot 3/4/5, Tricorn, Burning Ship, Celtic, Buffalo, Phoenix, Newton) — guards each formula's direct shader dispatch (was Mandelbrot-only before).
  - A **deep mode-0 (df32 perturbation, 1e6×)** golden for each polynomial family (Mandelbrot, Multibrot 3/4/5) at a bisected boundary coordinate — guards the bignum reference orbit (`step_bf`) + series approximation + the mode-0 shader branch. Coordinates via core's `dump_deep_boundary_coords` utility.
  - **Deferred deep tiers** (investigated): (a) deep goldens for the **abs families** (Burning Ship/Celtic/Buffalo) show fold **glitch-speckle** at deep perturbation — enshrining that is undesirable; wait for multi-reference glitch correction. (b) A **mode-2 / floatexp (~1e30×)** tier needs a coordinate accurate to ~1e-30 (≈70k-iter bignum bisection) **and** ~70k render iters per golden (slows every selftest run) — a real cost; `find_nucleus` minibrots are the cheaper path for polynomials when this is tackled.
- ⏭️ **Next for formula-ease:** a `Fractal`-trait PoC (Phase 5 pulled forward) unifying the CPU step functions so a family's math is defined once — now guarded by the per-family goldens above.
- 🔨 **Phase 4 (type safety) — enum dispatch, in progress** (behavior-preserving; each increment gated: clippy 0/0, app+state tests, selftest 55/55, goldens 17/17):
  - ✅ **Coloring dispatch typed** — the `color_method`/`trap_type` `u32` fields became `ColorMethod`/`TrapType` enums, deleting the `COLOR_METHODS`/`TRAP_TYPES` string tables + four `*_from_str`/`*_to_str` converters. The enum owns `key` (persisted) / `label` (UI) / `to_u32` / `from_u32` / `from_key` / `ALL` / `needs_aux`; `u32` is produced only at the uniform-write / snapshot boundary. Magic-number compares (`== 1`, `matches!(…, 1|2|5)`) read as `== ColorMethod::Stripe`, `.needs_aux()`.
  - ✅ **Render-mode dispatch typed** — the depth-selected `mode` `u32` became `RenderMode` {`Direct`, `Df32Pert`, `Floatexp`}. Its selection logic, previously **duplicated in three places** (`if !supports_pert || mag < 1e4 {1} else if mag >= FE {2} else {0}`), collapses to one `RenderMode::select(supports_pert, mag)` — the single place the representation is decided. Compares (`mode == 2`, `mode != 1`) read as `.is_floatexp()` / `!.is_direct()`; `u32` appears only at the GPU-uniform / `ExportRequest` boundary (`to_u32` / `from_u32`).
  - ✅ **Packed `ref_offset` newtyped** — the `[f32; 4]` df32 reference-offset (packed `[re_hi, im_hi, re_lo, im_lo]`, read by the shader as real `(.x,.z)` / imag `(.y,.w)`) became a `RefOffset` struct with **named limbs**. The hi/lo split — previously the byte-identical `[dxh, dyh, (dx-dxh) as f32, (dy-dyh) as f32]` **copy-pasted at 4 sites** (+ two `[0.0;4]` zero-inits) — now lives once in `RefOffset::from_df32(re, im)` (+ `ZERO`); the packed layout exists only inside `to_array()`, called at the 3 `IterUniforms` (the sole `#[repr(C)] Pod` struct) constructions. A limb transposition is now unrepresentable in app code. Verified with an adversarial 3-way review (completeness / bit-for-bit equivalence / no collateral to the sibling `step` array, which uses the *different* `[x_hi, x_lo, y_hi, y_lo]` order).
  - ⏭️ **`SpanMantissa` deferred (deliberate):** `span_mantissa: [f64;2]` never crosses a Pod/WGSL boundary (it's lowered CPU-side into the `step` uniform), so it has none of the shader-transposition payoff — while its blast radius is wider: a `pub` field on core `GpuScale` + gpu `MandelbrotParams`/`ExportRequest` (3 crates), ~14 indexed reads, and 3 in-place `span_mantissa[1] = span_mantissa[0] * aspect` mutations (needing `IndexMut`/`set_aspect`). Best co-designed with any future `step`/`center`/`julia_c` df32 newtypes (they share the `build_params`/`gpu_scale` call sites).
  - ✅ **Structured errors — `ExportError` (the whole `fractadyne-export` crate, slices 1–4 of 6):** wired `thiserror = "2"` into the workspace and defined `ExportError` (library variants `Io`/`PngDecode`/`PngEncode`/`Exr` as transparent `#[from]` sources; hand-written `ChannelNotFound`/`UnsupportedColorType`/`UnsupportedFormat`/`EmptyImage`/`SizeMismatch{expected,got}`). The two `write_*` went `Result<_,String> → Result<_,ExportError>` (buffer guards → `SizeMismatch`, every `.map_err(to_string)?` → bare `?`); **all seven `read_*`/thumbnail functions went `Option → Result<_,ExportError>`**, so a failed load distinguishes not-found / corrupt / channel-missing / size-mismatch instead of collapsing to `None`. Metadata readers → `Result<Option<String>,_>` (Ok(None) = untagged vs Err = unreadable). Callers stay behavior-preserving via `.ok()`/`.ok().flatten()` where they already ignored failure; the one UX gain is `open_view` reporting "Couldn't read {path}: {e}" on a corrupt image instead of the misleading "no metadata". Each slice gated (clippy 0/0, selftest 55/55, goldens 17/17 — the golden compare decodes through the migrated `read_png_rgba8`).
  - ✅ **Structured errors — `GpuError` + `AppError` (slices 5a–5b of 6):** `GpuError { Canceled, Readback }` in `fractadyne-gpu` — `render_export`/`render_iter`/`color_iter_buffer` return `Result<_,GpuError>`; the `~15` `.ok()`/`let _`/`let Ok…else` callers are untouched. `AppError { Gpu, Export }` (`#[from]`, transparent) in the app unifies the two crate errors so the export fns `?`-thread both, **removing all seven temporary `.map_err(to_string)` boundaries** from slices 1/5a. The payoff: the load-bearing cross-thread worker cancel — previously the fragile `Err(e) if e == "canceled"` **string compare** — is now the typed `Err(AppError::Gpu(GpuError::Canceled))`. Behavior-identical (every message Displays the same via `#[error("canceled")]` / transparent variants); adversarially reviewed for cancel-semantics preservation since the selftest doesn't exercise the worker/cancel path. `AppError` intentionally omits `Io`/`Parse`/`Message` for now (a `pub(crate)` enum warns on unconstructed variants) — a later `.kfr`/settings slice adds them.
  - ✅ **Structured errors — surface silent saves (slice 6 of 6, v0.1.29):** the two user-action saves that silently `let _ = fs::write(…)` now report status. Benchmark **Save** captures the write outcome in a local (the egui closure borrows `self`) and toasts "Benchmark saved." / "Save failed: {e}" after. **`save_bookmarks`** (durable data — silently lost on failure before) became `&mut self` and queues a `pending_toast: Option<String>` on a serialize/write error, drained into a toast early in `update()` (mirrors the existing `pending_state_warning` idiom) since its callers (`add_bookmark`/`process_pending_thumb`/delete) have no `egui::Context`. The 8 genuinely best-effort `let _ = fs::*` sites (dir-creation ahead of a reported write, thumbnail cache, stale-file cleanup, build-script codegen, selftest scratch) are deliberately left. Gated (clippy 0/0, app 5/5, selftest 55/55, goldens 17/17).
  - 🎉 **Phase 4 structured-errors migration complete** — `ExportError` (whole export crate), `GpuError` (gpu render/readback + typed cross-thread cancel), `AppError` (app export path), and the silent-save surfacing. Optional future polish: adopt `AppError` in `load_kfr_file` / `load_regions` (adding `Parse`/`Io` variants) to retire the last ~5 app `Result<_,String>` / `map_err(to_string)` sites in the `.kfr`/`--profile` paths.

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

**Steps** *(revised after a Phase-3 feasibility analysis — see the Progress log; the original "extract a `fractadyne-ui` crate" target was found net-negative and replaced with an intra-crate split)*
1. **Stub crates — DONE.** Retired `-ui` and `-fractals` from the workspace (empty, zero dependents; `-ui` should never be a crate given the coupling below, `-fractals` is superseded by the Phase-5 trait). Kept `-render` as a documented marker (`publish = false` + README). `-color` stays its own crate (a real layering boundary).
2. **UI decomposition — DONE, via an *intra-crate* `src/ui/` split, NOT a crate.** The 18 `draw_*` methods have a maximally-wide API surface (touch ~40 struct fields incl. the deliberately-flat `viewport`/`fractal`/`julia`, call ~50 cross-subsystem methods), so no clean crate boundary exists — a `fractadyne-ui` crate would force pub-widening the whole app state, a crate cycle, or an `app→ui` inversion. Instead the methods moved verbatim into `fractadyne-app/src/ui/{dialogs,central,menus,panels}.rs` as `impl FractadyneApp` submodule blocks (Rust splits one inherent impl across files): **main.rs 6305 → 3915 lines (−38%)**, zero pub-field widening, zero borrow re-choreography. Gated per file (goldens 17/17).
3. **The "`mpsc::Receiver` leak" — reclassified, no fix needed.** The `ExportPrep`/recompute channels are intentional fire-and-forget: a dropped receiver leaves a worker that runs its bignum orbit to completion then exits — bounded, self-terminating CPU on a superseded reference, **not a leak**. Documented with intent comments at the two spawn sites (`render.rs`). A cooperative-cancel `AtomicBool` is deferred until profiling shows it matters. *(Distinct latent item, noted in code: if `-render` is ever populated, wrap the `pub ExportPrep.rx` / raw `Receiver` behind a method API first.)*

**Deferred to a genuine future extraction** (better crate candidates than egui panels): the scripting/tour renderer + the overlay/annotation rasterization (Phase 2c) — those have a real headless, goldens-testable input→output contract. Populating `-render` waits on that.

**Risk:** ~~medium~~ **low** as executed (mechanical intra-crate moves). **Gate:** full suite + goldens.

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
