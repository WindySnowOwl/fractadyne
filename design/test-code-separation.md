# Separating test code from functional code

Status (2026-10-16): Phases 0–2 done on `refactor/test-separation` — no test code under any
`src/`, every test name kept (one declared rename), each move checked pure. Phases 3–5 (the
harnesses) not started.

## 1. Goal

Make it obvious from the file tree alone which source files are the product and which exist to
test it, so the functional code paths are easy to find, read and trace.

Nothing is compiled out. The in-binary harnesses still ship (Help ▸ Diagnostics runs
`--selftest`, `--uitest` and `--gputest` as child processes of the installed exe), and every unit
test keeps its private access to the code it tests.

The end state is one rule per folder:

| Folder | Holds | Compiled into |
|---|---|---|
| `crates/<crate>/src/` except `src/harness/` | functional code only | the product |
| `crates/<crate>/src/harness/` | test harnesses and dev tools that ship in the exe | the product |
| `crates/<crate>/tests/unit/` | unit tests, mirroring the `src/` tree | `cargo test` only |
| `crates/<crate>/tests/*.rs` | integration tests (public API), unchanged | `cargo test` only |

## 2. Where things stand

- **Unit-test files outnumber functional files.** 186 of the 337 `.rs` files under the eight
  crates' `src/` are unit-test modules (32,133 lines). In the app it is 120 of 210 files. The app's
  `src/` root lists 81 files: 32 functional, 31 unit-test files and 18 harnesses.
- **Test code still inside functional files:**
  - 31 inline `#[cfg(test)] mod … { }` blocks (3,413 lines, 151 tests). The largest is 1,058 lines
    at the bottom of `render/decide.rs`.
  - 15 test-only helper items: `#[cfg(test)]` functions and constants, plus
    `cfg_attr(not(test), allow(dead_code))` on functions only tests call.
- **File names don't tell you either.**
  - 49 test files have no "test" in their name. For example, `render/pin_policy.rs` is a test file
    that sits beside the functional `render/pricing.rs`.
  - 19 test files are `#[path]` siblings (`calibration_tests.rs` beside `calibration.rs`); the rest
    are nested files.
- **Harnesses: about 25,500 lines in 19 dedicated app modules** (selftest alone is 10,650), mixed
  in with the functional files at the `src/` root.
- **About 3,900 more harness lines are embedded in functional files:**

  | File | Harness code | Lines |
  |---|---|---|
  | `cli.rs` | dev modes (1,140 of its 1,800 lines) | 1,140 |
  | `main.rs` | flag parsing, banner, `--oomtest` | 620 |
  | `render/decide.rs` | `--replay` + the decision recorder | 550 |
  | `render.rs` | selfcheck methods + shims | 470 |
  | `autopilot.rs` | `--autodive` | 465 |
  | `ui/` | uitest data seeders | 200 |

- **Library crates:**
  - gpu: about 950 lines (test-only render entry points, `life/check.rs`, `lsystem/check.rs`,
    `timing.rs`), plus 236 lines of `fs_gputest` WGSL inside the shipping `mandelbrot.wgsl`.
  - core: about 100 lines (the dwell oracles and the deep-bits checks).
- **Why the 2026-09-02 convention eroded.** It allowed exceptions (blocks under 100 lines could
  stay inline) and nothing could fail when it was broken. Since then 31 inline blocks have appeared
  or grown, four of them over the limit. This plan replaces the exceptions with a rule simple
  enough to check mechanically, and adds the check.

## 3. The mechanism (verified 2026-10-16)

A unit test must be a child module of the code it tests to see private items. `#[path]` lets the
child's *file* live anywhere while its *module* stays where it is:

```rust
// crates/fractadyne-app/src/render/pricing.rs, at the bottom
#[cfg(test)]
#[path = "../../tests/unit/render/pricing/admission_tests.rs"]
mod admission_tests;
```

A scratch crate confirmed five things:

- The test still sees private functions and private struct fields.
- `module_path!()` is unchanged, so every test keeps its name
  (`render::pricing::admission_tests::…`), and existing filters and checklist entries still match.
- Cargo does not treat `tests/unit/` as an integration-test target. It only builds `tests/*.rs` and
  `tests/*/main.rs`.
- A `mod x;` inside a path-loaded file resolves beside that file. No test file declares a
  submodule today.
- The non-test build skips `#[cfg(test)]` items entirely, so moving unit tests cannot change the
  shipped exe.

The same mechanism lets harness code that needs a functional module's private items live under
`src/harness/` while staying attached to that module. Examples are `render.rs`'s selfcheck methods
and the uitest seeders in `ui/`.

## 4. The layout rules

1. **Unit tests mirror the file tree.** The tests for `src/<path>.rs` live in
   `tests/unit/<path>/`, one file per test module, named after the module. For example,
   `mod admission_tests` in `src/render/pricing.rs` lives at
   `tests/unit/render/pricing/admission_tests.rs`.
   - A crate root's (`main.rs`, `lib.rs`) test modules sit at the top of `tests/unit/`.
   - The declaration is always the same three lines at the bottom of the functional file, so its
     `#[path]` is also the pointer from the code to its tests.
2. **No exceptions.** No `#[test]` and no inline test module anywhere under `src/`.
   - Test-only helper items move into the test file that uses them.
   - The few that must stay, because they need private items of a module that isn't the test's
     parent, are marked and listed in an allowlist the check reads.
3. **Harnesses live in `src/harness/`.** That means code whose only job is to test, measure, audit
   or reproduce: verdicts, benchmarks, repros, audits, documentation screenshots.
   - One file per mode, named after its flag.
   - `src/harness.rs` is the index: flag, file, what it checks, and the list of hooks it has into
     functional code.
4. **Dependencies run one way.** Functional code calls into the harness only at named hooks: the
   dispatch in `main()` and `update()`, the exit check, and a few gates. Any functional code a
   harness file holds today moves to a functional home first.
5. **Fixtures move with their tests.** For example, `core/src/ir/parse/snapshot.txt` becomes
   `tests/unit/ir/parse/snapshot.txt`.

What a reader gets:

- `src/` shows only the product. The app's root goes from 81 files to 33.
- The tests for any file are at the same path under `tests/unit/`.
- In VS Code, setting search "files to exclude" to `**/tests/unit/**, **/src/harness/**` searches
  only functional code.

## 5. Phases

Each phase lands as small commits, and each commit passes its gates before the next starts.

- Moves are pure: the moved text is not edited.
- Pure-move commits go into `.git-blame-ignore-revs`.
- Git follows the renames in `log --follow` and blame.
- Work from the tip that carries W8 and W9 (`feat/w8-gates`), or after it merges. Moving 200+ files
  on a branch that diverges from active work guarantees conflicts. Other unpushed branches rebase
  through git's rename detection.

### Phase 0: baseline and the checks (no moves)

**Record the baseline with `scripts/test_names.py`.** It dumps every test's name per test target
(`cargo test -- --list`, ignored tests included) and compares two dumps, optionally mapping a
renamed module prefix. A count can stay the same while one test is lost and another duplicated; a
list of names cannot.
- The workspace: 1,579 tests in 16 targets at `30128530`.
- Core with `--features rug`. MPFR does not build for the MSVC target, so on Windows this list
  comes from the MSYS2 GNU toolchain (as `scripts/build-accelerated.ps1` builds), read back with
  `test_names.py parse`.

**Write `scripts/check_test_layout.py`, report-only at first.** It fails on:
- a `#[test]` or an inline test module under `src/`;
- a `#[cfg(test)]` or `cfg_attr(not(test), ...)` item under `src/` that is neither a canonical
  `#[path]` declaration nor allowlisted;
- a `#[path]` that points anywhere but the canonical location, or at a file that does not exist;
- any `.rs` under `tests/unit/` that no declaration reaches (rustc ignores orphan files silently,
  so a dead test could hide there);
- `tests/unit/main.rs` (Cargo would build it as its own target).

`--selfcheck` plants one violation of each kind in a scratch workspace and fails unless each is
reported. At `30128530` the report reads: 1,595 inline (1,564 `#[test]` + 31 blocks), 160
declarations without `#[path]`, 26 non-canonical, 15 test-only items. Phase 3 adds the direction
rule (a `crate::harness::` use in functional code outside the hook list) once `src/harness/`
exists.

**Write `scripts/check_move_only.py`.** It reads a commit's diff with rename detection off, so a
moved file is a whole-file removal plus a whole-file addition. It passes only if:
- the removed and added lines are the same multiset, ignoring indentation and the lines a move
  legitimately touches (blank lines, `#[cfg(test)]`, `#[path]`, `mod x;`, `mod x {`, a lone `}`);
- every run of removed lines reappears in order inside one run of added lines, so nothing is
  reordered or spliced;
- binary files move byte-identical.

`--rewrite OLD NEW` declares a mechanical substitution that comes with a move (a module path).
`--selfcheck` proves that an edit, a reorder, an added line and a dropped line hidden in a move
all fail. This makes "pure move" something checked, not claimed.

**Make every relative include location-independent.** Every relative `include_str!` /
`include_bytes!` in a file that will move switches to `concat!(env!("CARGO_MANIFEST_DIR"), "/…")`,
in one content commit before any move: 49 sites.
- app: the crate-root `tests.rs` (16), `icons_coverage.rs` (14), `frame_record_tests.rs` (2),
  `schema_tests.rs`, the `frame_plan.rs` test block;
- app harnesses, which move in Phase 3: `selftest.rs` (7 tour files and `opt_in_tests`'s read of
  itself), `logcheck.rs` (its rules file);
- core: `reference/sched_tests.rs`;
- gpu: `grad_hist_tests.rs` (2), `uniform_layout_tests.rs` (3), which read `mandelbrot.wgsl`.

The included bytes are unchanged, so the exe is too. Two more sites stay as they are, with the move
checker told about each:
- `ir/parse/snapshot.rs` reads `snapshot.txt` beside it, and the fixture moves with it;
- its re-record path (`src/ir/parse/snapshot.txt`) changes in the commit that moves them, declared
  with `--rewrite`.

**Teach the `frame_plan.rs` source scan a `#[path]` declaration.** It recognized a test module
only when `mod NAME;` sat on the line right after `#[cfg(test)]`, so it missed all 26 path
declarations, and after Phase 1 it would miss every one.

### Phase 1: move the unit-test files (pure renames)

- Move the 186 files to their canonical `tests/unit/` location with a script that runs `git mv`
  and rewrites the declaration.
  - One commit per crate, small crates first: text, state, export, color, farm, gpu, core, then the
    app, split by top-level module if large.
  - Test files of harness modules go to `tests/unit/selftest/…` etc. for now, and move again in
    Phase 3.
- The 19 `foo_tests.rs` siblings become `tests/unit/foo/tests.rs`, so the file is named after its
  module. `diag/console.rs` and `diag/device_loss_hint.rs`, which are test files despite their
  names, become `tests/unit/diag/console_tests.rs` and `tests/unit/diag/device_loss_hint_tests.rs`.
  File contents are unchanged.
- **Gates:**
  - `cargo build` and `cargo clippy --all-targets` are clean (`-j 1`).
  - The sorted test-name lists are byte-identical to the baseline, per crate and with `rug`.
  - `check_move_only.py` passes.
  - The exe cannot change, since only `#[cfg(test)]` lines are touched.

### Phase 2: no test code left under `src/`

- **Extract the 31 inline blocks to files.** The largest are:

  | Block | Lines |
  |---|---|
  | `decide.rs` | 1,058 |
  | `renorm.rs` | 323 |
  | `frame_plan.rs` | 281 |
  | `diag.rs` (five blocks) | 253 |
  | `backend_rug.rs` | 248 |
  | `update.rs` | 109 |

  Each body moves verbatim and is dedented one level. The dedent is whitespace only and goes in
  the blame-ignore list.
- **Move the 15 test-only helper items** into their test files. Delete the stale
  `cfg_attr(not(test), allow(dead_code))` on `ui/textbook/model.rs`'s `math_row`, which production
  now calls.
- **Rewrite the two tests that decide "is this a test file?" from its name.**
  - `diag.rs`'s `every_log_category_is_documented` keys on a `_tests.rs` suffix plus three
    hard-coded names. Today it scans `tests.rs` and `pin_policy.rs` as production.
  - The `frame_plan.rs` scan misses every `#[path]` declaration.

  Both switch to the new rule: under `src/` is production, under `tests/unit/` is test.
- **Make `check_test_layout.py` fail for real.** Run it in CI next to `cargo test` and in the
  pre-commit routine.
- **Update the docs:** CONTRIBUTING.md's test-layout section and ARCHITECTURE.md's module map.
- **Gates:** as Phase 1.
- **Milestone: no test code under any `src/`.**

### Phase 3: the app's harness modules move to `src/harness/`

- **First, move functional code out of harness files**, in its own behavior-neutral commits:

  | Item | Used by |
  |---|---|
  | `profile::params_to_request(_exact)` | the benchmark and `main.rs` |
  | `glitchaudit::correction_applies` | `export.rs` |
  | `gputest::block_on` | `gpu_worker.rs` |

- **Then move the 19 modules:** selftest, uitest (with `--juliadive` and `--dualsettle`), livetest,
  profile, motiontest, zoomtest, recordtest, soak, chunksweep, deviceloss_repro, bench_matrix,
  reusetest, glitchaudit, gputest, torture, `farm/farmtest`, logcheck, shot, verdict.
  - `crate::selftest` becomes `crate::harness::selftest`, and so on. Call sites are updated
    mechanically.
  - The modules stay descendants of the crate root, so their access to `FractadyneApp`'s private
    fields and methods is unchanged.
  - Their unit tests move to `tests/unit/harness/…`. Those test names gain a `harness::` prefix, so
    the baseline comparison maps the old prefix to the new one.
- **Write `src/harness.rs`:** every flag, its file, its verdict codes, and the hooks into
  functional code.
- **Add the direction rule to `check_test_layout.py`:** a `crate::harness::` use in functional
  code outside the hook list fails, with a planted case in `--selfcheck`.
- **Update `scripts/checklist_coverage.py`** (it reads `src/selftest.rs`, `src/uitest.rs` and
  `src/main.rs` by path) and ARCHITECTURE.md.
- **Gates:**
  - build, clippy and tests (with the mapped identity);
  - `checklist_coverage.py --cargo` resolves every step;
  - a full `--selftest` with goldens byte-identical on the RTX 3080;
  - `--gputest`;
  - `--uitest`, and look at the screenshots;
  - the validation battery on the RX 6800 XT through the field agent;
  - Help ▸ Diagnostics launches all three harnesses.

### Phase 4: embedded harness code leaves functional files

One commit per row. Each moves whole functions or impl blocks.

| From | What | Lines | To |
|---|---|---|---|
| `cli.rs` | `--pickcheck`, `--refdiag`, `--crosscheck-f3`, `--bench-bignum`, `--validate-deep`, `--compare` | ≈830 | `harness/<flag>.rs` each |
| `cli.rs` | `HarnessModes`, `harness_frame_hooks`, the harness branches of `cli_mode_ladder` | ≈320 | `harness.rs` |
| `main.rs` | harness flag parsing in `new_state` | ≈300 | `HarnessModes::from_args` |
| `main.rs`, `ui/menus.rs` | `--oomtest`, test-banner mode name and `draw_test_banner` | ≈115 | `harness/` |
| `autopilot.rs` | `--autodive` (`AutoDive`, `DiveTally`, verdict, frame hook) | 465 | `harness/autodive.rs` |
| `render/decide.rs` | `--replay` and the `FRACTADYNE_RECORD_DECIDE` recorder | ≈550 | `harness/replay.rs`, `harness/decide_record.rs` |
| `render.rs` | the six `selfcheck_*` methods and the `*_for_selftest` / `*_for_uitest` shims | ≈470 | `harness/render/selfcheck.rs`, attached to `render` (needs its private items) |
| `ui/farm_client.rs`, `ui/farm_controller.rs`, `ui/tour_render.rs` | uitest data seeders | ≈180 | `harness/ui/…`, attached |

- **After Phases 2–4:**

  | File | Before (lines) | After (lines) |
  |---|---|---|
  | `cli.rs` | 1,800 | ≈660 |
  | `autopilot.rs` | 1,658 | ≈1,190 |
  | `render/decide.rs` | 3,789 | ≈2,180 |

- **Hooks stay in functional code.** These are one-line calls and gates:
  - `accumulation_allowed`'s harness terms;
  - the finish-sound and open-after-export gates;
  - the `uitest_*` fields on UI structs;
  - the instrument env-var reads in `render.rs`.

  `harness.rs` lists them all. Removing them is a different refactor (section 6).
- **Gates:** Phase 3's, plus `--bench-matrix` interleaved ABABAB against the pre-phase build.
  Release builds use `lto = false` and `codegen-units = 16`, so shrinking `render.rs` and `main.rs`
  can re-partition codegen units and shift inlining.
- **Risk:** extracting the `new_state` flag parsing is the one step that is not a plain move,
  because it splits a function body. Do it last and alone.

### Phase 5: the library crates

- **gpu → `gpu/src/harness/`.** Move `render_iter_chunked_timed` (271 lines), `render_iter_chunked`,
  `render_iter_split`, `time_iter_pass`, `render_export_unchunked`, `render_export_multi_with`,
  `gputest`, `life/check.rs`, `lsystem/check.rs` and `timing.rs`. They stay attached with `#[path]`,
  so the public paths (`fractadyne_gpu::…`) and the app's call sites don't change.
- **core → `core/src/harness/`.** Move `naive_dwell_bf` and `formula_dwell` (the oracles), and
  `deep_consistency_bits` and `deep_roundtrip_bits`.
- **Optional, highest risk: the `fs_gputest` shader code.** It means moving `fs_gputest` (236 lines
  of WGSL) out of `mandelbrot.wgsl` into `gpu/src/harness/gputest.wgsl`, concatenated at load. This
  changes the text of the shipping shader, so it needs:
  - `--gputest` and the goldens on both cards;
  - the Fraktaler-3 corpus at 38/38.

  It is only worth doing if the shader's length gets in the way of reading it.
- **Gates:** as Phase 4.

### Size

- **Phases 0–2:** mechanical and script-driven, and they cannot change the exe.
- **Phase 3:** mostly waiting on the selftest, gputest and uitest runs and the field battery.
- **Phases 4–5:** about twenty independent commits.

Each phase stands alone. Stopping after Phase 2 already gives "no test code under `src/`".

## 6. Not in this plan (could follow)

- **Splitting `selftest.rs`** into one file per check group. Today it is one 10,050-line function
  over 40 groups.
- **Splitting the crate-root catch-all test files into the modules they test.** These are the app's
  `tests.rs` (1,464 lines) and core's `tests.rs` (2,276, mostly `reference.rs` internals). About 15
  items could then go back from `pub(crate)` to private. This changes test names.
- **Untangling the hooks from functional code:**
  - one typed instruments registry (6 env vars are registered; about 14 more change behavior
    without being registered);
  - one run policy in place of the three task-flag lists that disagree;
  - a `perf.harness` sub-struct for the harness-only counters.

## 7. Found on the way (separate fixes, never mixed into a move)

1. **`TASK_FLAGS` is stale.** At `main.rs:763` it lists `--play-tour`; the real flag is `--play`.
   It also omits `--replay`, `--logcheck`, `--validate-deep` and `--compare`, and the two other
   task-flag lists (`launched_for_a_task` and the autosave gate) disagree with it. The visible
   effect is not yet reproduced, so reproduce it before fixing.
2. **`--selftest-list` hides four groups that `--selftest` runs:** `export-contain`, `retreat`,
   `view-thumb` and `width-independence` are missing from `GROUPS`.
3. **`--replay` is missing from Help's CLI reference.**
4. **Two core probe files are mislabelled.** `core/tests/probe_escape.rs` and `probe_orbit.rs` say
   "TEMPORARY diagnostic (not committed)" but are tracked. Keep them (and fix the header) or delete
   them.
