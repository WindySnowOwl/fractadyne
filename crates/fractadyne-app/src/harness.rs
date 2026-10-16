//! The test harnesses and developer tools that ship in the binary: everything under `harness/`
//! exists to test, measure, audit or reproduce the product, never to be it.
//!
//! They ship on purpose. Help ▸ Diagnostics runs `--selftest`, `--uitest` and `--gputest` from the
//! installed exe, and the field battery (`scripts/gpu-validate.ps1` / `.sh`) runs the rest on a
//! tester's machine. The unit tests are elsewhere, in each crate's `tests/unit/`; this is the
//! in-binary half (design/test-code-separation.md).
//!
//! | Flag | Module | What it does |
//! |---|---|---|
//! | `--selftest` | [`selftest`] | GPU validation: the render paths against each other, CPU and bignum oracles, and golden images |
//! | `--uitest`, `--juliadive`, `--dualsettle` | [`uitest`] | a scripted walk through the UI and the live render, a screenshot per screen |
//! | `--gputest` | [`gputest`] | the shader's df32/floatexp primitives against CPU oracles, on every backend |
//! | `--livetest` | [`livetest`] | what the live view shows during a tour, against offline renders of the same views |
//! | `--profile`, `--divetest`, `--resizetest`, `--frametest` | [`profile`] | stage timings and live-dive frame costs |
//! | `--motiontest` | [`motiontest`] | the motion-presentation gate for chunked deep views |
//! | `--zoomtest` | [`zoomtest`] | the on-screen update latency of live zooms |
//! | `--recordtest` | [`recordtest`] | the frame record records |
//! | `--soak` | [`soak`] | a deep view left running is still alive (and the regime rungs) |
//! | `--chunk-sweep` | [`chunksweep`] | a chunked iterate's per-window cost |
//! | `--deviceloss-repro` | [`deviceloss_repro`] | a safe reproduction of the deep-interior device loss |
//! | `--bench-matrix` | [`bench_matrix`] | the path-coverage performance and regression suite |
//! | `--reusetest` | [`reusetest`] | reprojection staleness against from-scratch frames |
//! | `--glitch-audit`, `--tail-audit`, `--renorm-audit` (with `--render`) | [`glitchaudit`] | the pixels a render path changes, against the bignum oracle |
//! | `--torture` | [`torture`] | the escalating failure-hunting ladder, each rung a child process |
//! | `--logcheck` | [`logcheck`] | a session's log against `validation/logcheck-rules.toml` |
//! | `--shot` | [`shot`] | the site screenshot, regenerated from a saved location |
//! | `--farmtest` | `crate::farm::farmtest` (file `harness/farm/farmtest.rs`) | the render farm end to end on one machine |
//!
//! [`verdict`] is the exit-code convention the gating harnesses share: 0 PASS, 2 RED, 3 VACUOUS
//! (the experiment did not run), 4 WATCHDOG. A run whose own log breaks a `--logcheck` rule turns
//! 0 into 1.
//!
//! Still in functional files until phase 4 moves them: `--autodive` (`autopilot.rs`), `--replay`
//! and the decision recorder (`render/decide.rs`), and `--pickcheck`, `--refdiag`,
//! `--crosscheck-f3`, `--bench-bignum`, `--validate-deep` and `--compare` (`cli.rs`).
//!
//! **Where the product calls in.** `scripts/check_test_layout.py` holds each functional file to the
//! number of times it names `harness::` (HARNESS_HOOKS), so a new call into a harness is a decision,
//! not an accident:
//! - `main()` logs the run's start ([`logcheck::harness_begin`]), and every exit checks the log
//!   ([`logcheck::at_exit`], from `crate::exit`);
//! - `cli::run_headless` starts the harnesses that run before any window (`--torture`,
//!   `--logcheck`, `--gputest`);
//! - `FractadyneApp::new_state` parses the windowed harnesses' flags into `cli::HarnessModes`, and
//!   `update()` drives them each frame (`harness_frame_hooks`, `cli_mode_ladder`);
//! - the export-request builder (`render.rs`) records the profiler's setup timings
//!   ([`profile::ProfSetup`], `FractadyneApp::prof`).

pub(crate) mod bench_matrix;
pub(crate) mod chunksweep;
pub(crate) mod deviceloss_repro;
pub(crate) mod glitchaudit;
pub(crate) mod gputest;
pub(crate) mod livetest;
pub(crate) mod logcheck;
pub(crate) mod motiontest;
pub(crate) mod profile;
pub(crate) mod recordtest;
pub(crate) mod reusetest;
pub(crate) mod selftest;
pub(crate) mod shot;
pub(crate) mod soak;
pub(crate) mod torture;
pub(crate) mod uitest;
pub(crate) mod verdict;
pub(crate) mod zoomtest;
