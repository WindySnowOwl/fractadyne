//! `--torture` — the escalating failure-hunting suite. Design: `design/torture-suite.md`.
//!
//! This module is the SUPERVISOR half. It owns the ladder (an ordered set of rungs), resolves
//! selectors, launches each rung as a CHILD PROCESS under a deadline, classifies the outcome, and
//! writes a failure artifact shaped like a crash report.
//!
//! ⚠**Why a child process per rung, and not an in-process loop.** The failures this suite exists to
//! find either kill the process or wedge it: a lost device calls `crate::exit(2)` from a wgpu
//! callback (that IS the recovery path — see the device-loss handler in `main.rs`), and the
//! historical "present wedge" blocks the main thread inside a wgpu wait forever. An in-process
//! runner dies or hangs at the first interesting result and reports nothing about the rest. A
//! supervisor survives both, and gets per-rung isolation, deadlines and resume for free.
//!
//! ⚠**A deadline expiry is a FAILURE, not a skip.** The oldest banked lesson about soak testing
//! here is that a harness which greps only for crashes passes a hung app.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Which broad harness a rung drives. Lanes run independently: a live failure never blocks an
/// offline rung, because they share no prerequisite.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Lane {
    /// Interactive/live rendering — the scripted-tour and dive harnesses.
    Live,
    /// Whole-tour playback and per-chapter checks.
    Tour,
    /// Headless rendering, corpus, writers, resume. Deterministic and CI-safe.
    Offline,
}

impl Lane {
    fn as_str(self) -> &'static str {
        match self {
            Lane::Live => "live",
            Lane::Tour => "tour",
            Lane::Offline => "offline",
        }
    }
}

/// What a rung actually executes.
///
/// ⚠`External` exists because not every gate is a subcommand: the 20-location Fraktaler-3 corpus
/// check is `validation/corpus/generate_corpus.py --check`. The first draft of that rung guessed
/// `--crosscheck-f3 --check`, which is a different tool entirely (it compares ONE supplied F3 EXR
/// against the CPU oracle). A missing interpreter is reported `skip-unsupported`, never a pass and
/// never a failure — a gate that silently vanishes is how a suite starts lying.
pub(crate) enum Cmd {
    /// Re-invoke this same binary with these arguments.
    SelfExe(&'static [&'static str]),
    /// Run another program (program, args).
    External(&'static str, &'static [&'static str]),
}

impl Cmd {
    fn display(&self) -> String {
        match self {
            Cmd::SelfExe(a) => format!("fractadyne {}", a.join(" ")),
            Cmd::External(p, a) => format!("{p} {}", a.join(" ")),
        }
    }
}

/// One rung of the ladder. `id` is `lane/family/name` and is the unit of targeting.
///
/// ⚠`requires` is what keeps "continue after a failure" useful rather than noisy. Without it, one
/// crossover device loss makes every deeper live rung fail for the same reason and buries whatever
/// novel failure the run was supposed to surface. With it, the dependent spine reports `blocked-by`
/// once and the independent families keep running. See design §P3a.
pub(crate) struct Rung {
    pub id: &'static str,
    pub lane: Lane,
    /// Why this rung exists — the incident or invariant it guards. Printed in the failure artifact,
    /// because a rung nobody can motivate is a rung nobody will maintain.
    pub motivation: &'static str,
    /// What the child runs.
    pub cmd: Cmd,
    /// Wall-clock budget. Sized from the blessed duration with headroom; expiry is a failure.
    pub deadline: Duration,
    /// Rung IDs that must PASS for this one to be meaningful.
    pub requires: &'static [&'static str],
    /// Bytes this rung needs free on the output volume before it is allowed to start.
    ///
    /// ⚠A render that fills the disk does not fail cleanly: it writes a truncated PNG, and the
    /// resume vetter's own tests record that the decoder ACCEPTS a file truncated by one byte
    /// (IEND's CRC is never verified). So an out-of-space render can produce a file that later
    /// reads as a finished frame. Refusing up front is the only honest handling; the alternative is
    /// a corrupt artifact that looks valid.
    pub needs_disk: u64,
}

const MB: u64 = 1024 * 1024;
const GB: u64 = 1024 * MB;

/// How a rung ended.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Outcome {
    Pass,
    /// Ran to completion but the harness reported a failure (drift, Δ, verdict).
    FailAssert,
    /// Exceeded its deadline and was killed.
    FailDeadline,
    /// Lost the GPU device.
    FailDeviceLost,
    /// Panicked, aborted, or exited non-zero for another reason.
    FailCrash,
    /// The in-app watchdog reported a possible hang.
    FailHang,
    /// A prerequisite failed, so this rung was not run.
    Blocked(String),
    /// The rung cannot run here (e.g. no Python for the corpus gate). Recorded, never silent.
    SkipUnsupported(String),
}

impl Outcome {
    pub(crate) fn is_pass(&self) -> bool {
        matches!(self, Outcome::Pass)
    }
    /// Did this rung actually FAIL (as opposed to being blocked by something that did)? Only a real
    /// failure propagates to dependents — a blocked rung must not cascade a second time, or one
    /// root cause reports as a chain of distinct failures.
    pub(crate) fn is_failure(&self) -> bool {
        !matches!(
            self,
            Outcome::Pass | Outcome::Blocked(_) | Outcome::SkipUnsupported(_)
        )
    }
    fn as_str(&self) -> &str {
        match self {
            Outcome::Pass => "pass",
            Outcome::FailAssert => "fail-assert",
            Outcome::FailDeadline => "fail-deadline",
            Outcome::FailDeviceLost => "fail-device-lost",
            Outcome::FailCrash => "fail-crash",
            Outcome::FailHang => "fail-hang-watchdog",
            Outcome::Blocked(_) => "blocked",
            Outcome::SkipUnsupported(_) => "skip-unsupported",
        }
    }
}

/// Peak/typical resource use observed while a rung ran.
///
/// ⚠These are not decoration. Today's contention incident (design §P8) produced a run that looked
/// exactly like a hang and was really a loaded machine; without a load figure recorded alongside the
/// duration there is no way to tell those apart after the fact. A `fail-deadline` with "CPU was at
/// 98% and it wasn't us" is a different bug report from one at 12%.
#[derive(Clone, Copy, Default)]
pub(crate) struct ResourceStats {
    pub samples: u32,
    pub cpu_peak: f64,
    pub cpu_sum: f64,
    pub ram_used_peak: u64,
    pub ram_total: u64,
    pub gpu_peak: f64,
    pub vram_peak: u64,
    pub vram_total: u64,
    /// Free bytes on the output volume, lowest seen.
    pub disk_free_min: u64,
    /// Did we ever get a GPU reading? Distinguishes "0%" from "no idea".
    pub gpu_known: bool,
}

impl ResourceStats {
    fn cpu_mean(&self) -> f64 {
        if self.samples == 0 {
            0.0
        } else {
            self.cpu_sum / self.samples as f64
        }
    }
    /// One line for the summary and the failure artifact.
    pub(crate) fn line(&self) -> String {
        let gb = |b: u64| b as f64 / (1024.0 * 1024.0 * 1024.0);
        let gpu = if self.gpu_known {
            format!(
                "gpu {:.0}% peak, vram {:.1}/{:.1} GB",
                self.gpu_peak,
                gb(self.vram_peak),
                gb(self.vram_total)
            )
        } else {
            "gpu n/a".to_string()
        };
        format!(
            "cpu {:.0}% mean / {:.0}% peak · ram {:.1}/{:.1} GB peak · {gpu} · disk free {:.1} GB",
            self.cpu_mean(),
            self.cpu_peak,
            gb(self.ram_used_peak),
            gb(self.ram_total),
            gb(self.disk_free_min),
        )
    }
}

/// What a finished rung produced.
pub(crate) struct RunRecord {
    pub id: String,
    pub lane: Lane,
    pub outcome: Outcome,
    pub duration: Duration,
    pub exit_code: Option<i32>,
    pub output: String,
    pub res: ResourceStats,
}

const MIN: u64 = 60;

/// ⭐THE LADDER. Ordered within each lane from easiest to hardest.
///
/// This first increment deliberately composes rungs ONLY from flags that already exist, so the
/// supervisor can be landed and trusted before any engine surface changes. The regimes nothing can
/// currently reach — above all a forced short-escaped reference (`bla_skip=0`), which is what lost
/// the device on the RX 6800 XT on 2026-08-15 — are increment 2 in the design doc and are absent
/// here ON PURPOSE. A green run of this ladder does NOT mean that regime is covered.
pub(crate) const LADDER: &[Rung] = &[
    // ==================== OFFLINE — deterministic, no GPU timing, cheapest ====================
    // Ordered so a broken build fails in seconds rather than after a 40-minute live rung.
    //
    // WARNING FROM EXPERIENCE: the first draft of the opening rung ran `--selftest-filter writer`,
    // which matched NOTHING — the beta.104 writer round-trip tests are `cargo test` unit tests, not
    // selftest groups. The suite caught it on its own first run. Every filter below is a real tag
    // from `--selftest-list`, checked against that output.
    Rung {
        id: "offline/smoke/metadata",
        lane: Lane::Offline,
        motivation: "PNG/EXR metadata embed + read-back. Fast first rung: if this fails the build \
                     is broken in a way that makes every later verdict meaningless.",
        cmd: Cmd::SelfExe(&["--selftest", "--selftest-filter", "metadata"]),
        deadline: Duration::from_secs(3 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "offline/smoke/coords",
        lane: Lane::Offline,
        motivation: "Coordinate parse/format round-trip — the layer every deeper rung's centre \
                     string passes through before it reaches the shader.",
        cmd: Cmd::SelfExe(&["--selftest", "--selftest-filter", "coords"]),
        deadline: Duration::from_secs(3 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "offline/smoke/live-res",
        lane: Lane::Offline,
        motivation: "The two beta.102/103 settled-resolution invariants — the deterministic gates \
                     that replaced a repro script which was really a regime lottery.",
        cmd: Cmd::SelfExe(&["--selftest", "--selftest-filter", "live-res"]),
        deadline: Duration::from_secs(5 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "offline/math/numeric",
        lane: Lane::Offline,
        motivation: "Core numeric + symmetry invariants. A failure here invalidates every image \
                     comparison below, so it gates the depth ladder.",
        cmd: Cmd::SelfExe(&["--selftest", "--selftest-filter", "numeric"]),
        deadline: Duration::from_secs(5 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "offline/math/bla",
        lane: Lane::Offline,
        motivation: "Bilinear approximation correctness. BLA is the difference between ~1 s and \
                     ~10 ms per deep frame, and the regime where it does NOT engage is where every \
                     device loss so far has landed.",
        cmd: Cmd::SelfExe(&["--selftest", "--selftest-filter", "bla"]),
        deadline: Duration::from_secs(5 * MIN),
        requires: &["offline/math/numeric"],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "offline/math/iter-chunk",
        lane: Lane::Offline,
        motivation: "Iteration-range chunking must be honoured ACROSS frames (beta.64-69): chunk \
                     state lives in ping-pong textures and must stay bit-identical.",
        cmd: Cmd::SelfExe(&["--selftest", "--selftest-filter", "iter-chunk"]),
        deadline: Duration::from_secs(5 * MIN),
        requires: &["offline/math/numeric"],
        needs_disk: 128 * MB,
    },
    // ---- the offline DEPTH ladder. Coordinates are the corpus seahorse centre (34 significant
    // digits, so it stays meaningful past e28). These rungs assert that the engine SURVIVES and
    // exits clean at each depth; correctness at depth is the corpus gate's job. Survival is exactly
    // the failure class that matters here — device loss and hangs.
    Rung {
        id: "offline/depth/e00-home",
        lane: Lane::Offline,
        motivation: "Framing/palette anchor at magnification 1.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.75", "0", "--zoom", "1", "--size", "640x400",
            "--iter", "512", "-o", "{OUT}/render-e00.png",
        ]),
        deadline: Duration::from_secs(2 * MIN),
        requires: &["offline/math/numeric"],
        needs_disk: 64 * MB,
    },
    Rung {
        id: "offline/depth/e06-double-exhausted",
        lane: Lane::Offline,
        motivation: "Classic KF benchmark depth — double precision exhausted, perturbation on.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139",
            "--zoom", "1e6", "--size", "640x400", "--iter", "3000",
            "-o", "{OUT}/render-e06.png",
        ]),
        deadline: Duration::from_secs(3 * MIN),
        requires: &["offline/depth/e00-home"],
        needs_disk: 64 * MB,
    },
    Rung {
        id: "offline/depth/e13-f32-cliff",
        lane: Lane::Offline,
        motivation: "The f32 cliff / direct-to-perturbation switch (~2^13.7), named in the GPU \
                     arithmetic work as a real regime boundary. Nothing sampled it before.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139",
            "--zoom", "5e13", "--size", "640x400", "--iter", "20000",
            "-o", "{OUT}/render-e13.png",
        ]),
        deadline: Duration::from_secs(5 * MIN),
        requires: &["offline/depth/e06-double-exhausted"],
        needs_disk: 64 * MB,
    },
    Rung {
        id: "offline/depth/e24-deep-df32",
        lane: Lane::Offline,
        motivation: "Deep df32, still arithmetic mode 0 — the last rung before the crossover.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139",
            "--zoom", "1e24", "--size", "640x400", "--iter", "20000",
            "-o", "{OUT}/render-e24.png",
        ]),
        deadline: Duration::from_secs(6 * MIN),
        requires: &["offline/depth/e13-f32-cliff"],
        needs_disk: 64 * MB,
    },
    Rung {
        id: "offline/depth/e28-crossover",
        lane: Lane::Offline,
        motivation: "PERT_FE_THRESHOLD, arithmetic mode 0 to 2 — the 2:58 device-loss class and \
                     the 2026-08-15 RX 6800 XT loss. Verified to reach mode=2. The first test \
                     point this project has ever had at the crossover. NOTE this is the BLA-live \
                     variant; the no-BLA regime that actually kills is design increment 2.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139",
            "--zoom", "1e28", "--size", "640x400", "--iter", "20000",
            "-o", "{OUT}/render-e28.png",
        ]),
        deadline: Duration::from_secs(8 * MIN),
        requires: &["offline/depth/e24-deep-df32"],
        needs_disk: 64 * MB,
    },
    Rung {
        id: "offline/depth/e28-explicit-10m",
        lane: Lane::Offline,
        motivation: "The crossover with a huge EXPLICIT count (auto-iter off) — the regime where \
                     skip effectiveness is least predictable, which is the stated reason \
                     TDR_EXPLICIT_BUDGET_MS was cut to 400 ms after three Event-153 losses.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139",
            "--zoom", "1e28", "--size", "640x400", "--iter", "10000000",
            "-o", "{OUT}/render-e28x.png",
        ]),
        deadline: Duration::from_secs(15 * MIN),
        requires: &["offline/depth/e28-crossover"],
        needs_disk: 64 * MB,
    },
    // ---- resolution sweep at an already-proven depth, so a failure isolates RESOLUTION.
    Rung {
        id: "offline/res/1080p",
        lane: Lane::Offline,
        motivation: "Full-HD offline render at the crossover depth.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139",
            "--zoom", "1e28", "--size", "1920x1080", "--iter", "20000",
            "-o", "{OUT}/render-1080p.png",
        ]),
        deadline: Duration::from_secs(10 * MIN),
        requires: &["offline/depth/e28-crossover"],
        needs_disk: 256 * MB,
    },
    Rung {
        id: "offline/res/4k",
        lane: Lane::Offline,
        motivation: "4K offline render — the largest allocation an ordinary export makes, and the \
                     one most likely to meet the GPU texture-size ceiling.",
        cmd: Cmd::SelfExe(&[
            "--render", "--center", "-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139",
            "--zoom", "1e28", "--size", "3840x2160", "--iter", "20000",
            "-o", "{OUT}/render-4k.png",
        ]),
        deadline: Duration::from_secs(20 * MIN),
        requires: &["offline/res/1080p"],
        needs_disk: 1 * GB,
    },
    // ---- the heavy offline gates last.
    Rung {
        id: "offline/gate/selftest-full",
        lane: Lane::Offline,
        motivation: "The 116-check + 17-golden release gate, run whole. Group state is shared, so \
                     the filtered rungs above are smoke tests and THIS is the verdict.",
        cmd: Cmd::SelfExe(&["--selftest"]),
        deadline: Duration::from_secs(25 * MIN),
        requires: &[],
        needs_disk: 512 * MB,
    },
    Rung {
        id: "offline/gate/corpus-f3",
        lane: Lane::Offline,
        motivation: "Fraktaler-3 cross-implementation oracle, 20 locations to 1.2e1008x — the only \
                     independent evidence that deep arithmetic is CORRECT and not merely stable.",
        cmd: Cmd::External("python", &["validation/corpus/generate_corpus.py", "--check"]),
        deadline: Duration::from_secs(40 * MIN),
        requires: &["offline/gate/selftest-full"],
        needs_disk: 4 * GB,
    },
    // ==================== TOUR — chapter by chapter, then the whole thing ====================
    // Chapters are individually targetable so a chapter regression does not cost a 40-minute run.
    // They CHAIN: if the shallowest chapter is broken, a deeper chapter's verdict is not meaningful.
    // Verified 2026-08-15 that a --segment subset grades cleanly against the FULL blessed baseline
    // ("1 checkpoint - 0 drifted - 0 new - 23 in baseline not reached").
    //
    // WARNING: 480x270 ONLY. Baselines are per (tour, resolution); an unblessed size falls back to
    // raw-FAIL grading, which the harness itself warns "stays red while the tour's known-failing
    // holds are unfixed". A resolution sweep here needs blessing first — design increment 3.
    Rung {
        id: "tour/grand/whole-set",
        lane: Lane::Tour,
        motivation: "Chapter 1 — the whole set at magnification 1. The shallowest live path.",
        cmd: Cmd::SelfExe(&[
            "--livetest", "tours/grand-tour.toml", "--segment", "whole-set", "--size", "480x270",
        ]),
        deadline: Duration::from_secs(5 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "tour/grand/landmarks",
        lane: Lane::Tour,
        motivation: "Chapter 2 — seahorse, elephant, triple spiral. Shallow perturbation.",
        cmd: Cmd::SelfExe(&[
            "--livetest", "tours/grand-tour.toml", "--segment", "landmarks", "--size", "480x270",
        ]),
        deadline: Duration::from_secs(12 * MIN),
        requires: &["tour/grand/whole-set"],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "tour/grand/exact-points",
        lane: Lane::Tour,
        motivation: "Chapter 3 — points known exactly (c = i, antenna tip): answers checkable \
                     against the catalog rather than against ourselves.",
        cmd: Cmd::SelfExe(&[
            "--livetest", "tours/grand-tour.toml", "--segment", "exact-points", "--size", "480x270",
        ]),
        deadline: Duration::from_secs(12 * MIN),
        requires: &["tour/grand/landmarks"],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "tour/grand/features",
        lane: Lane::Tour,
        motivation: "Chapter 4 — dual Julia, orbit overlay, fractal variety. The beta.100 orbit \
                     narrative and the dual-view path live here.",
        cmd: Cmd::SelfExe(&[
            "--livetest", "tours/grand-tour.toml", "--segment", "features", "--size", "480x270",
        ]),
        deadline: Duration::from_secs(15 * MIN),
        requires: &["tour/grand/exact-points"],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "tour/grand/gauntlet",
        lane: Lane::Tour,
        motivation: "Chapter 5 — the deep gauntlet: holds at e55/e61/e63/e72/e82/e94. The \
                     motion-cap truncation family (beta.98) and the reference-extension refusal \
                     band both regress here first. The single most valuable chapter.",
        cmd: Cmd::SelfExe(&[
            "--livetest", "tours/grand-tour.toml", "--segment", "gauntlet", "--size", "480x270",
        ]),
        deadline: Duration::from_secs(40 * MIN),
        requires: &["tour/grand/features"],
        needs_disk: 128 * MB,
    },
    // ---- FULL-RESOLUTION live coverage (design G3). Blessed 2026-08-16 at
    // benchmarks/livetest-grand-tour-1920x1080.json, verified reproducible (0 drifted on a re-run).
    //
    // ⭐Why a second resolution earns its keep, stated precisely because the obvious reason is
    // WRONG. It is not that "the budget never binds at 480×270" — it does: e63/e72/e82/e94 settle
    // at 461×259 / 298×167 / 230×129 / 163×91 there, and at 1920×1080 they settle at exactly the
    // same sizes, because the budget is denominated in steps and the resolution that fits it does
    // not care about the window. What the large window actually adds is:
    //   - e55 and e61 become budget-bound too (653×367, 516×290) where the small window capped
    //     them first, so two more holds exercise the controller instead of the window;
    //   - the shallow checkpoints render at TRUE full resolution, which is the only way to reach
    //     the TILED SETTLE at full size — and both 2026 field device losses were on that path
    //     (the 08-16 one at 2247×1485, settled=true, tile=true).
    // A 480×270 window is simply too small to need tiling, so no gate had ever run it.
    Rung {
        id: "tour/grand/gauntlet-1080p",
        lane: Lane::Tour,
        motivation: "The deep gauntlet at 1920x1080: e55/e61 become budget-bound here, and the \
                     full-resolution tiled settle runs for the first time in any gate.",
        cmd: Cmd::SelfExe(&[
            "--livetest", "tours/grand-tour.toml", "--segment", "gauntlet", "--size", "1920x1080",
        ]),
        deadline: Duration::from_secs(30 * MIN),
        requires: &["tour/grand/gauntlet"],
        needs_disk: 256 * MB,
    },
    Rung {
        id: "tour/grand/full-1080p",
        lane: Lane::Tour,
        motivation: "The whole grand tour at 1920x1080 - the widest live coverage there is, and \
                     the only gate that exercises the frame-cost controller at a window size a \
                     person would actually use.",
        cmd: Cmd::SelfExe(&["--livetest", "tours/grand-tour.toml", "--size", "1920x1080"]),
        deadline: Duration::from_secs(45 * MIN),
        requires: &["tour/grand/gauntlet-1080p"],
        needs_disk: 512 * MB,
    },
    Rung {
        id: "tour/bench/matrix",
        lane: Lane::Tour,
        motivation: "22-segment path-coverage perf + determinism vs a blessed baseline. The \
                     pick-determinism tripwire; exit 2 means algorithmic drift.",
        cmd: Cmd::SelfExe(&["--bench-matrix"]),
        deadline: Duration::from_secs(45 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    // ==================== LIVE — where every device loss has actually happened ====================
    Rung {
        id: "live/ui/walk",
        lane: Lane::Live,
        motivation: "Scripted walk through every UI screen and the three live-render bands \
                     (Direct/df32/floatexp). Has caught recent UI regressions faster than \
                     widget-level tests would have.",
        cmd: Cmd::SelfExe(&["--uitest"]),
        deadline: Duration::from_secs(20 * MIN),
        requires: &[],
        needs_disk: 512 * MB,
    },
    Rung {
        id: "live/home/glide-from-depth",
        lane: Lane::Live,
        motivation: "ZOOM HOME FROM DEPTH — the only automated reproduction of the device-loss regime,                      and the button that actually lost a device (2026-08-18, crash-1787014795-0.txt).                      One continuous sweep from depth to 1x crosses every mode boundary while the budget                      is still sized for the deep regime, which is the shape of both field losses: a                      frame ALREADY SIZED by the old regime going out at 405x over budget (3040 ms)                      after a mode switch reset it, and bla_skip collapsing 4,457,481 -> 9,343 so                      per-step cost jumped ~100x under a stale budget. Measured 3/3 glides from 1e22                      each produced a lethal reading (peaks 1025/1042/1605 ms against a 900 ms band);                      the emergency retreat survived all three. A MONOTONIC DIVE CANNOT DO THIS - it                      peaked at 36 ms on the RX 6800 XT at 1e150 because the controller was starved,                      not stressed. --autodive exits 3 if no lethal reading occurs, so this rung fails                      loudly when it stops reproducing rather than passing vacuously.",
        cmd: Cmd::SelfExe(&["--autodive", "22", "--autodive-timeout", "420", "--autodive-home", "3"]),
        deadline: Duration::from_secs(10 * MIN),
        requires: &[],
        needs_disk: 256 * MB,
    },
    Rung {
        id: "live/julia/dive",
        lane: Lane::Live,
        motivation: "Dual-Julia motion harness through PERT_JULIA_THRESHOLD (100x, beta.88), where \
                     Julia starts perturbing and the tessellation artefacts appeared.",
        cmd: Cmd::SelfExe(&["--juliadive"]),
        deadline: Duration::from_secs(20 * MIN),
        requires: &[],
        needs_disk: 512 * MB,
    },
    Rung {
        id: "live/resize/settle",
        lane: Lane::Live,
        motivation: "Resize during settle — the beta.102/103 pixellation pair (a tile allowance \
                     that was secretly a rate test; a finished sharp frame never presented).",
        cmd: Cmd::SelfExe(&["--resizetest"]),
        deadline: Duration::from_secs(20 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "live/reuse/stage0",
        lane: Lane::Live,
        motivation: "XaoS-style zoom reuse, stage 0 reprojection — guards the during-motion path a \
                     future stage-2 refine will build on.",
        cmd: Cmd::SelfExe(&["--reusetest"]),
        deadline: Duration::from_secs(15 * MIN),
        requires: &[],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "live/tour/grand-full",
        lane: Lane::Live,
        motivation: "The whole grand tour end to end, all 24 checkpoints, against the blessed \
                     baseline. The release gate for the live path.",
        cmd: Cmd::SelfExe(&["--livetest", "tours/grand-tour.toml", "--size", "480x270"]),
        deadline: Duration::from_secs(75 * MIN),
        requires: &["tour/grand/gauntlet"],
        needs_disk: 128 * MB,
    },
    Rung {
        id: "live/dive/ultra-e200",
        lane: Lane::Live,
        motivation: "e30/e60/e100/e150/e200 holds — a SHORTER-reference regime than the grand tour \
                     (orbit_len 5k-24k vs 258k-4M), so it exercises different rebase/BLA behaviour \
                     at comparable depths. WARNING: the first draft of this rung passed \
                     `--livetest --ultra`; `--ultra` is a BENCHMARK depth flag, not a livetest \
                     modifier, so it silently ran the default tour instead.",
        cmd: Cmd::SelfExe(&["--livetest", "tours/ultra-dive-e200.toml", "--size", "480x270"]),
        deadline: Duration::from_secs(60 * MIN),
        // Same harness, deeper (e200 vs e94): a broken grand tour makes this verdict meaningless.
        requires: &["live/tour/grand-full"],
        needs_disk: 128 * MB,
    },
];

/// Resolve selectors to rungs, preserving ladder order. An empty selector list means everything.
/// Matching is by exact ID or by `/`-delimited prefix, so `live`, `live/tour` and a full ID all work.
pub(crate) fn select(selectors: &[String]) -> Vec<&'static Rung> {
    LADDER
        .iter()
        .filter(|r| {
            selectors.is_empty()
                || selectors.iter().any(|s| {
                    let s = s.trim_end_matches('/');
                    r.id == s || r.id.starts_with(&format!("{s}/"))
                })
        })
        .collect()
}

/// Classify a finished child. Order matters: a device loss also exits non-zero, and the watchdog
/// can fire on a run that then crashes, so the most specific cause wins.
pub(crate) fn classify(timed_out: bool, exit_code: Option<i32>, output: &str) -> Outcome {
    if timed_out {
        return Outcome::FailDeadline;
    }
    if output.contains("DEVICE LOST") || output.contains("device lost — restarting") {
        return Outcome::FailDeviceLost;
    }
    if output.contains("[fd-watch] possible hang") {
        return Outcome::FailHang;
    }
    match exit_code {
        Some(0) => Outcome::Pass,
        // The harnesses use a distinct code for "ran fine, result is wrong" where they can
        // (`--bench-matrix` exits 2 on algorithmic drift); everything else non-zero is a crash.
        Some(2) => Outcome::FailAssert,
        Some(_) | None => Outcome::FailCrash,
    }
}

/// Entry point for `--torture`. Returns the process exit code.
pub(crate) fn run(args: &[String]) -> i32 {
    let selectors: Vec<String> = args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .collect();
    let stop_at_first = args.iter().any(|a| a == "--stop-at-first");
    let list_only = args.iter().any(|a| a == "--list");

    let rungs = select(&selectors);
    if rungs.is_empty() {
        eprintln!("--torture: no rung matches {selectors:?}");
        eprintln!("try `--torture --list` for the ladder");
        return 2;
    }
    if list_only {
        print_ladder(&rungs);
        return 0;
    }

    let out_dir = out_dir(args);
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("--torture: cannot create {}: {e}", out_dir.display());
        return 2;
    }
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("--torture: cannot find own executable: {e}");
            return 2;
        }
    };

    println!("== torture: {} rung(s) ==", rungs.len());
    if !stop_at_first {
        println!("continuing past failures; dependents of a failed rung report as blocked");
    }
    // ⭐STARTING LOAD. Recorded because a busy machine changes RESULTS here, not merely durations:
    // checkpoint resolution derives from the measured frame-cost budget, so a livetest run made
    // alongside a compile spent 1400 s in a hold the same build cleared in ~200 s and looked
    // exactly like a hang (design §P8). Warning up front is cheaper than diagnosing it afterwards.
    {
        let mut cpu = crate::sysinfo::CpuSampler::default();
        let _ = cpu.sample(); // establish the baseline; the first reading is always None
        std::thread::sleep(Duration::from_millis(400));
        let busy = cpu.sample().unwrap_or(0.0);
        let free = crate::sysinfo::free_disk_bytes(&out_dir).unwrap_or(0);
        println!(
            "machine at start: cpu {busy:.0}% · disk free {:.1} GB on {}",
            free as f64 / GB as f64,
            out_dir.display()
        );
        if busy > 25.0 {
            println!(
                "  ⚠ CPU is already {busy:.0}% busy. Timing-derived rungs (every livetest \
                 checkpoint) can report DIFFERENT RESULTS on a loaded machine, not just slower \
                 ones. Consider waiting until it is idle."
            );
        }
    }

    let mut records: Vec<RunRecord> = Vec::new();
    for rung in &rungs {
        // P3a: only a REAL failure blocks dependents. A blocked rung must not cascade again, or one
        // root cause reports as a chain of unrelated-looking failures.
        if let Some(blocker) = rung.requires.iter().find(|need| {
            records
                .iter()
                .any(|r| r.id == **need && r.outcome.is_failure())
        }) {
            println!("  [BLOCKED] {} — needs {}", rung.id, blocker);
            records.push(RunRecord {
                id: rung.id.to_string(),
                lane: rung.lane,
                outcome: Outcome::Blocked((*blocker).to_string()),
                duration: Duration::ZERO,
                exit_code: None,
                output: String::new(),
                res: ResourceStats::default(),
            });
            continue;
        }

        // DISK PRECHECK. Refusing up front beats discovering it mid-render: an out-of-space write
        // leaves a truncated PNG, and this project's own writer tests record that the decoder
        // accepts a file short by one byte (IEND's CRC is never verified). A rung that "passed"
        // having written a corrupt frame is worse than one that declined to start.
        if let Some(free) = crate::sysinfo::free_disk_bytes(&out_dir) {
            if free < rung.needs_disk {
                let msg = format!(
                    "needs {:.1} GB free, have {:.1} GB on {}",
                    rung.needs_disk as f64 / GB as f64,
                    free as f64 / GB as f64,
                    out_dir.display()
                );
                println!("  [SKIP]    {} — {msg}", rung.id);
                records.push(RunRecord {
                    id: rung.id.to_string(),
                    lane: rung.lane,
                    outcome: Outcome::SkipUnsupported(msg),
                    duration: Duration::ZERO,
                    exit_code: None,
                    output: String::new(),
                    res: ResourceStats::default(),
                });
                continue;
            }
        }

        print!("  [run]     {} … ", rung.id);
        use std::io::Write;
        let _ = std::io::stdout().flush();

        let rec = run_rung(&exe, rung, &out_dir);
        println!(
            "{} ({:.1}s)",
            rec.outcome.as_str().to_uppercase(),
            rec.duration.as_secs_f64()
        );
        if rec.outcome.is_failure() {
            match write_artifact(&out_dir, rung, &rec) {
                Ok(p) => println!("            report → {}", p.display()),
                Err(e) => eprintln!("            (could not write report: {e})"),
            }
        }
        let failed = rec.outcome.is_failure();
        records.push(rec);
        if failed && stop_at_first {
            println!("  stopping: --stop-at-first");
            break;
        }
    }

    summarize(&records, &out_dir)
}

fn out_dir(args: &[String]) -> PathBuf {
    if let Some(p) = args
        .iter()
        .position(|a| a == "--torture-out")
        .and_then(|i| args.get(i + 1))
    {
        return PathBuf::from(p);
    }
    PathBuf::from("validation/torture")
}

/// Launch one rung and wait for it, killing it at the deadline.
///
/// ⚠**Hermetic**: each rung gets its own `FRACTADYNE_CONFIG_DIR`, so a rung can neither read the
/// developer's live session nor leave state for the next rung. The corpus gate spent a week red for
/// exactly this reason before it was made hermetic.
fn run_rung(exe: &Path, rung: &Rung, out_dir: &Path) -> RunRecord {
    // ⚠WIPED, not merely created. A per-rung directory that PERSISTS between invocations is only
    // half-hermetic: the app writes a session on exit, so the second run of a rung starts from
    // wherever the first one finished. That is not hypothetical — `--livetest` boots into the saved
    // view, and on 2026-08-15 a session holding a 1e102 view (2.4M-sample reference at ~1 fps) kept
    // the tour from starting AT ALL for 11.5 minutes, after which the run was abandoned. The gate
    // must begin from defaults every single time or its duration and its results both wander.
    let cfg = out_dir.join(format!("cfg-{}", rung.id.replace('/', "_")));
    let _ = std::fs::remove_dir_all(&cfg);
    let _ = std::fs::create_dir_all(&cfg);
    let log = out_dir.join(format!("{}.log", rung.id.replace('/', "_")));

    // ⚠`{OUT}` is substituted with the run's output directory, which the supervisor has created.
    // The first version of the render rungs hard-coded `validation/torture/render-*.png`, a
    // relative path into a GITIGNORED directory: it worked on the dev box only because that
    // directory happened to exist there by hand, and on the RX 6800 XT's fresh clone every render
    // rung died instantly with "The system cannot find the path specified. (os error 3)". A rung
    // must not depend on the working directory it is launched from, nor on untracked state.
    let subst = |a: &&'static str| -> String { a.replace("{OUT}", &out_dir.to_string_lossy()) };
    let started = Instant::now();
    let mut builder = match &rung.cmd {
        Cmd::SelfExe(a) => {
            let mut c = Command::new(exe);
            c.args(a.iter().map(subst));
            c
        }
        Cmd::External(prog, a) => {
            let mut c = Command::new(prog);
            c.args(a.iter().map(subst));
            c
        }
    };
    let child = builder
        .env("FRACTADYNE_CONFIG_DIR", &cfg)
        // Announce the context so a rung's own log says which rung wrote it.
        .env("FRACTADYNE_TORTURE_RUNG", rung.id)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            // An EXTERNAL program that is simply absent (no Python) is not a product failure — it
            // is a gate this machine cannot run, and saying so is the honest report. Our own
            // executable failing to spawn IS a failure.
            let outcome = match &rung.cmd {
                Cmd::External(prog, _) if e.kind() == std::io::ErrorKind::NotFound => {
                    Outcome::SkipUnsupported(format!("{prog} not found on PATH"))
                }
                _ => Outcome::FailCrash,
            };
            return RunRecord {
                id: rung.id.to_string(),
                lane: rung.lane,
                outcome,
                duration: started.elapsed(),
                exit_code: None,
                output: format!("could not spawn {}: {e}", rung.cmd.display()),
                res: ResourceStats::default(),
            };
        }
    };

    // ⚠**Drain the pipes on their own threads, concurrently with the wait.** Reading them after the
    // child exits deadlocks: a pipe buffer is ~64 KB, `--livetest` emits several MB of frame
    // diagnostics, and a child blocked writing into a full pipe never exits. The supervisor would
    // then hit its own deadline and file a FABRICATED `fail-deadline` against a perfectly healthy
    // rung — the worst failure mode a test harness can have, since it manufactures bugs instead of
    // finding them. (Reader threads also mean a killed child's partial output still reaches the
    // artifact, which is exactly the output a hang report needs.)
    let drain = |pipe: Option<std::process::ChildStdout>| {
        pipe.map(|p| {
            std::thread::spawn(move || {
                use std::io::Read;
                let mut s = String::new();
                let mut p = p;
                let _ = p.read_to_string(&mut s);
                s
            })
        })
    };
    let out_h = drain(child.stdout.take());
    let err_h = child.stderr.take().map(|p| {
        std::thread::spawn(move || {
            use std::io::Read;
            let mut s = String::new();
            let mut p = p;
            let _ = p.read_to_string(&mut s);
            s
        })
    });

    // Poll rather than `wait()`: the deadline is the whole point, and a wedged child never returns.
    // The same loop samples system load and repaints the live status line, so a long rung shows
    // progress instead of looking like the terminal died — which, on a 40-minute rung, is the
    // difference between "it's working" and "I killed a healthy run because I couldn't tell".
    let mut timed_out = false;
    let mut cpu = crate::sysinfo::CpuSampler::default();
    let mut res = ResourceStats {
        ram_total: crate::sysinfo::total_memory().unwrap_or(0),
        disk_free_min: u64::MAX,
        ..Default::default()
    };
    let mut last_paint = Instant::now();
    let mut last_gpu = Instant::now() - Duration::from_secs(60);
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) => {
                if started.elapsed() > rung.deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break None;
                }
                if let Some(pct) = cpu.sample() {
                    res.samples += 1;
                    res.cpu_sum += pct;
                    res.cpu_peak = res.cpu_peak.max(pct);
                }
                if let (Some(avail), true) = (crate::sysinfo::available_memory(), res.ram_total > 0)
                {
                    res.ram_used_peak = res.ram_used_peak.max(res.ram_total.saturating_sub(avail));
                }
                if let Some(free) = crate::sysinfo::free_disk_bytes(out_dir) {
                    res.disk_free_min = res.disk_free_min.min(free);
                }
                // GPU costs a process spawn, so sample it rarely. `None` stays "unknown", never 0.
                if last_gpu.elapsed() >= Duration::from_secs(10) {
                    last_gpu = Instant::now();
                    if let Some((util, used, total)) = crate::sysinfo::gpu_usage() {
                        res.gpu_known = true;
                        res.gpu_peak = res.gpu_peak.max(util);
                        res.vram_peak = res.vram_peak.max(used);
                        res.vram_total = total;
                    }
                }
                if last_paint.elapsed() >= Duration::from_secs(2) {
                    last_paint = Instant::now();
                    paint_progress(rung, started.elapsed(), &res);
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(_) => break None,
        }
    };
    if res.disk_free_min == u64::MAX {
        res.disk_free_min = 0;
    }
    clear_progress();

    // Both readers hit EOF once the child is gone (killed or exited), so these joins terminate.
    let mut output = out_h.and_then(|h| h.join().ok()).unwrap_or_default();
    if let Some(e) = err_h.and_then(|h| h.join().ok()) {
        output.push_str(&e);
    }
    let _ = std::fs::write(&log, &output);

    let exit_code = status.and_then(|s| s.code());
    RunRecord {
        id: rung.id.to_string(),
        lane: rung.lane,
        outcome: classify(timed_out, exit_code, &output),
        duration: started.elapsed(),
        exit_code,
        output,
        res,
    }
}

/// Repaint the in-place status line. Carriage return, no newline: the line is overwritten in place
/// so a 40-minute rung produces one live line rather than a thousand scrolled ones.
fn paint_progress(rung: &Rung, elapsed: Duration, res: &ResourceStats) {
    use std::io::{IsTerminal, Write};
    // ⚠Only to a terminal. Carriage-return repainting into a REDIRECTED stream writes every frame
    // of the animation into the file, which is what turned the RX 6800 XT's captured torture log
    // into an unreadable smear of half-overwritten status lines. A progress display exists for a
    // human watching; a log exists for whoever reads it afterwards, and they want different bytes.
    if !std::io::stdout().is_terminal() {
        return;
    }
    let gb = |b: u64| b as f64 / (1024.0 * 1024.0 * 1024.0);
    let pct = (elapsed.as_secs_f64() / rung.deadline.as_secs_f64() * 100.0).min(999.0);
    let gpu = if res.gpu_known {
        format!(" gpu {:>3.0}%", res.gpu_peak)
    } else {
        String::new()
    };
    print!(
        "\r      {:<30} {:>5.0}s/{:.0}s ({:>3.0}%)  cpu {:>3.0}%  ram {:>4.1}G{}  disk {:>5.1}G   ",
        elide(rung.id, 30),
        elapsed.as_secs_f64(),
        rung.deadline.as_secs_f64(),
        pct,
        res.cpu_peak,
        gb(res.ram_used_peak),
        gpu,
        gb(res.disk_free_min),
    );
    let _ = std::io::stdout().flush();
}

fn clear_progress() {
    use std::io::{IsTerminal, Write};
    if !std::io::stdout().is_terminal() {
        return;
    }
    print!("\r{:100}\r", " ");
    let _ = std::io::stdout().flush();
}

fn elide(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("…{}", &s[s.len() - (n - 1)..])
    }
}

/// The failure artifact, deliberately shaped like a crash report: whoever reads it should not have
/// to ask a follow-up question to reproduce.
fn write_artifact(out_dir: &Path, rung: &Rung, rec: &RunRecord) -> std::io::Result<PathBuf> {
    let path = out_dir.join(format!("{}.txt", rung.id.replace('/', "_")));
    let tail: Vec<&str> = rec.output.lines().rev().take(60).collect();
    let tail: Vec<&str> = tail.into_iter().rev().collect();
    let body = format!(
        "fractadyne torture failure\n\
         rung      : {}\n\
         lane      : {}\n\
         outcome   : {}\n\
         duration  : {:.1}s (deadline {:.0}s)\n\
         exit code : {}\n\
         version   : {}\n\
         tunables  : {}\n\
         repro     : fractadyne --torture {}\n\
         child cmd : {}\n\
         motivation: {}\n\
         resources : {}\n\
         \n== output tail ==\n{}\n",
        rung.id,
        rung.lane.as_str(),
        rec.outcome.as_str(),
        rec.duration.as_secs_f64(),
        rung.deadline.as_secs_f64(),
        rec.exit_code.map_or("none (killed or signalled)".to_string(), |c| c.to_string()),
        crate::version_string(),
        crate::tunables::status_line(),
        rung.id,
        rung.cmd.display(),
        rung.motivation,
        rec.res.line(),
        tail.join("\n"),
    );
    std::fs::write(&path, body)?;
    Ok(path)
}

fn print_ladder(rungs: &[&Rung]) {
    println!("{:<34} {:<8} {:>7}  {}", "RUNG", "LANE", "DEADLN", "MOTIVATION");
    for r in rungs {
        let m: String = r.motivation.split_whitespace().collect::<Vec<_>>().join(" ");
        let m: String = m.chars().take(70).collect();
        println!(
            "{:<34} {:<8} {:>6.0}s  {}",
            r.id,
            r.lane.as_str(),
            r.deadline.as_secs_f64(),
            m
        );
    }
}

/// Print the ladder summary and return the process exit code.
fn summarize(records: &[RunRecord], out_dir: &Path) -> i32 {
    println!("\n== summary ==");
    for lane in [Lane::Offline, Lane::Live, Lane::Tour] {
        let in_lane: Vec<&RunRecord> = records.iter().filter(|r| r.lane == lane).collect();
        if in_lane.is_empty() {
            continue;
        }
        // "Highest rung passed" = the last CONTIGUOUS pass, which is the honest reading of an
        // ordered ladder: a pass after a failure says nothing about the rungs in between.
        let highest = in_lane
            .iter()
            .take_while(|r| r.outcome.is_pass())
            .last()
            .map(|r| r.id.as_str())
            .unwrap_or("(none)");
        let failed: Vec<&&RunRecord> = in_lane.iter().filter(|r| r.outcome.is_failure()).collect();
        let blocked = in_lane
            .iter()
            .filter(|r| matches!(r.outcome, Outcome::Blocked(_)))
            .count();
        println!(
            "  {:<8} {}/{} passed, highest contiguous: {}",
            lane.as_str(),
            in_lane.iter().filter(|r| r.outcome.is_pass()).count(),
            in_lane.len(),
            highest
        );
        for f in &failed {
            println!("           FAILED {} — {}", f.id, f.outcome.as_str());
            // The resource line next to the failure is what tells a `fail-deadline` caused by a
            // loaded machine apart from one caused by the product. See design §P8.
            println!("             {}", f.res.line());
        }
        for s in in_lane
            .iter()
            .filter_map(|r| match &r.outcome {
                Outcome::SkipUnsupported(why) => Some((r.id.as_str(), why.as_str())),
                _ => None,
            })
        {
            println!("           skipped {} — {}", s.0, s.1);
        }
        if blocked > 0 {
            println!("           {blocked} blocked by an earlier failure");
        }
    }
    let failures = records.iter().filter(|r| r.outcome.is_failure()).count();
    if failures == 0 {
        println!("\nall {} rung(s) passed", records.len());
        0
    } else {
        println!("\n{failures} rung(s) FAILED — reports in {}", out_dir.display());
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rung_id_is_unique_and_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for r in LADDER {
            assert!(seen.insert(r.id), "duplicate rung id {}", r.id);
            let parts: Vec<&str> = r.id.split('/').collect();
            assert_eq!(parts.len(), 3, "{} must be lane/family/name", r.id);
            assert_eq!(parts[0], r.lane.as_str(), "{} lane prefix mismatch", r.id);
            assert!(!r.motivation.is_empty(), "{} needs a motivation", r.id);
            match &r.cmd {
                Cmd::SelfExe(a) => assert!(!a.is_empty(), "{} needs args", r.id),
                Cmd::External(p, _) => assert!(!p.is_empty(), "{} needs a program", r.id),
            }
        }
    }

    #[test]
    fn every_prerequisite_exists_and_comes_earlier() {
        // A forward reference would silently never block, which is worse than a cycle: the suite
        // would look like it had dependency awareness while having none.
        for (i, r) in LADDER.iter().enumerate() {
            for need in r.requires {
                let at = LADDER.iter().position(|o| o.id == *need);
                let at = at.unwrap_or_else(|| panic!("{} requires unknown rung {need}", r.id));
                assert!(at < i, "{} requires {need}, which comes later in the ladder", r.id);
            }
        }
    }

    #[test]
    fn the_ladder_actually_escalates_within_each_lane() {
        // A rung with prerequisites must come after them (checked elsewhere); here we assert the
        // ladder is not accidentally flat — each lane needs a dependency chain, or "highest rung
        // passed" is meaningless and a failure blocks nothing.
        for lane in [Lane::Offline, Lane::Tour, Lane::Live] {
            let chained = LADDER
                .iter()
                .filter(|r| r.lane == lane && !r.requires.is_empty())
                .count();
            assert!(chained >= 2, "{:?} lane has no escalation chain", lane);
        }
    }

    #[test]
    fn the_crossover_is_covered_and_reaches_the_regime_that_matters() {
        // G1 in the design doc: PERT_FE_THRESHOLD had no test point anywhere in the project. The
        // CORRECTNESS half is now covered by --selftest (oracle entries at 9.3e27×/1.3e28× plus a
        // bracket check on the selector); this rung owns the half that still is not — HOLDING at
        // the crossover under a frame budget, which is the regime every device loss has landed in.
        // If someone deletes it, the most dangerous depth in the app goes unsampled under load.
        let x = LADDER
            .iter()
            .find(|r| r.id == "offline/depth/e28-crossover")
            .expect("the e28 crossover rung must exist — see design G1");
        match &x.cmd {
            Cmd::SelfExe(a) => assert!(
                a.windows(2).any(|w| w[0] == "--zoom" && w[1] == "1e28"),
                "the crossover rung must actually sit at 1e28"
            ),
            _ => panic!("crossover rung must run our own binary"),
        }
    }

    #[test]
    fn an_absent_external_tool_is_a_skip_not_a_pass_and_not_a_failure() {
        // The corpus gate needs Python. If it is missing the run must say so out loud: counting it
        // as a pass would let a machine report a green suite while never checking deep arithmetic.
        let skip = Outcome::SkipUnsupported("python not found".into());
        assert!(!skip.is_pass(), "a skipped gate is not a pass");
        assert!(!skip.is_failure(), "a machine without python is not a product bug");
    }

    #[test]
    fn selectors_match_by_exact_id_and_by_prefix() {
        assert_eq!(select(&[]).len(), LADDER.len(), "no selector means everything");
        let live = select(&["live".to_string()]);
        assert!(!live.is_empty());
        assert!(live.iter().all(|r| r.lane == Lane::Live));
        let one = select(&["live/tour/grand-full".to_string()]);
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].id, "live/tour/grand-full");
        // A prefix must not match a longer sibling name by accident.
        assert!(select(&["live/tou".to_string()]).is_empty());
        assert!(select(&["nope".to_string()]).is_empty());
    }

    #[test]
    fn selection_preserves_ladder_order() {
        let all = select(&[]);
        let ids: Vec<&str> = all.iter().map(|r| r.id).collect();
        let ladder_ids: Vec<&str> = LADDER.iter().map(|r| r.id).collect();
        assert_eq!(ids, ladder_ids);
    }

    #[test]
    fn a_deadline_expiry_outranks_the_exit_code() {
        // A killed child also reports a non-zero/absent code; "timed out" must win, or every hang
        // is filed as a crash and the real signal is lost.
        assert_eq!(classify(true, None, ""), Outcome::FailDeadline);
        assert_eq!(classify(true, Some(0), ""), Outcome::FailDeadline);
    }

    #[test]
    fn device_loss_is_recognised_even_though_it_exits_nonzero() {
        let log = "[fd-wgpu] DEVICE LOST (Unknown): Device is lost";
        assert_eq!(classify(false, Some(2), log), Outcome::FailDeviceLost);
        assert_eq!(classify(false, None, log), Outcome::FailDeviceLost);
    }

    #[test]
    fn a_watchdog_hang_is_a_failure_not_a_pass() {
        // The banked lesson: a soak that greps only for crashes passes a hung app. Exit 0 with a
        // watchdog line in the log is exactly that shape.
        let log = "[fd-watch] possible hang: no activity for 71s";
        assert_eq!(classify(false, Some(0), log), Outcome::FailHang);
    }

    #[test]
    fn clean_and_drifted_runs_are_told_apart() {
        assert_eq!(classify(false, Some(0), "checks 116/116 — OK"), Outcome::Pass);
        assert_eq!(classify(false, Some(2), "algorithmic drift"), Outcome::FailAssert);
        assert_eq!(classify(false, Some(101), "panicked"), Outcome::FailCrash);
    }

    #[test]
    fn only_real_failures_block_dependents() {
        // Blocked must not propagate: if it did, one root cause would report as a chain of
        // apparently distinct failures and the "multiple failures per run" goal would be defeated.
        assert!(Outcome::FailDeviceLost.is_failure());
        assert!(Outcome::FailDeadline.is_failure());
        assert!(!Outcome::Blocked("x".into()).is_failure());
        assert!(!Outcome::Pass.is_failure());
        assert!(!Outcome::Blocked("x".into()).is_pass());
    }
}
