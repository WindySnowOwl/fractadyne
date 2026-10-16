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
//! | `--autodive` | [`autodive`] | the autopilot's dive driven unpaced, to hammer the frame-cost controller |
//! | `--replay` | [`replay`] | every decision a `FRACTADYNE_RECORD_DECIDE` recording holds, recomputed and compared |
//! | `--farmtest` | `crate::farm::farmtest` (file `harness/farm/farmtest.rs`) | the render farm end to end on one machine |
//! | `--oomtest` | [`oomtest`] | a real allocation failure, to prove the out-of-memory path writes a crash report |
//!
//! [`banner`] is the LIVE TEST banner a harness-driven window wears (`--test-banner`,
//! `--no-test-banner`).
//!
//! [`verdict`] is the exit-code convention the gating harnesses share: 0 PASS, 2 RED, 3 VACUOUS
//! (the experiment did not run), 4 WATCHDOG. A run whose own log breaks a `--logcheck` rule turns
//! 0 into 1.
//!
//! Developer modes that run before any window, from `cli::run_headless`, one module each:
//! `--refdiag` ([`refdiag`]), `--pickcheck` ([`pickcheck`]), `--compare` ([`compare`]),
//! `--crosscheck-f3` ([`crosscheck_f3`]), `--bench-bignum` ([`bench_bignum`]) and
//! `--validate-deep` ([`validate_deep`]).
//!
//! [`modes`] holds the windowed harnesses' state, the per-frame hooks `update()` runs for them
//! (`harness_frame_hooks`), and the mode ladder's harness branches (`harness_ladder`).
//!
//! The decision recorder `--replay` reads (`FRACTADYNE_RECORD_DECIDE`, in `render/decide.rs`)
//! stays with the product: the live path writes it, and a field session is diagnosed with it.
//!
//! Files under `harness/<path>/` stay modules of the functional file `src/<path>.rs` whose private
//! parts they need, attached with `#[path]` from it: `harness/render/selfcheck.rs` is
//! `crate::render::selfcheck` (the self-test's and the UI test's ways into the live reference
//! pipeline), `harness/farm/farmtest.rs` is `crate::farm::farmtest`, and
//! `harness/ui/{farm_client,farm_controller,tour_render}/uitest_seed.rs` fill those windows with
//! the state `--uitest` photographs.
//!
//! **Where the product calls in.** `scripts/check_test_layout.py` holds each functional file to the
//! number of times it names `harness::` (HARNESS_HOOKS), so a new call into a harness is a decision,
//! not an accident:
//! - `main()` logs the run's start ([`logcheck::harness_begin`]), and every exit checks the log
//!   ([`logcheck::at_exit`], from `crate::exit`);
//! - `cli::run_headless` starts the harnesses that run before any window (`--torture`,
//!   `--logcheck`, `--gputest` and the developer modes above);
//! - `FractadyneApp::new_state` parses the windowed harnesses' flags into [`modes::HarnessModes`],
//!   and `update()` drives them (`harness_frame_hooks`, and `cli_mode_ladder` → `harness_ladder`);
//! - the export-request builder (`render.rs`) records the profiler's setup timings
//!   ([`profile::ProfSetup`], `FractadyneApp::prof`).

pub(crate) mod autodive;
pub(crate) mod banner;
pub(crate) mod bench_bignum;
pub(crate) mod bench_matrix;
pub(crate) mod chunksweep;
pub(crate) mod compare;
pub(crate) mod crosscheck_f3;
pub(crate) mod deviceloss_repro;
pub(crate) mod glitchaudit;
pub(crate) mod gputest;
pub(crate) mod livetest;
pub(crate) mod logcheck;
pub(crate) mod modes;
pub(crate) mod motiontest;
pub(crate) mod oomtest;
pub(crate) mod pickcheck;
pub(crate) mod profile;
pub(crate) mod recordtest;
pub(crate) mod refdiag;
pub(crate) mod replay;
pub(crate) mod reusetest;
pub(crate) mod selftest;
pub(crate) mod shot;
pub(crate) mod soak;
pub(crate) mod torture;
pub(crate) mod uitest;
pub(crate) mod validate_deep;
pub(crate) mod verdict;
pub(crate) mod zoomtest;
