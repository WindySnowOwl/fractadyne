//! The arbitrary-precision **backend** the reference orbit is iterated in.
//!
//! [`crate::BigFloat`] (astro-float) is the crate's **carrier** type: the viewport centre, parsing,
//! decimal serialization and every public signature use it, and that does not change. This trait
//! exists so the *hot loop* — [`crate::reference::run_orbit`], which is `max_iter × step` in bignum
//! and dominates a deep frame — can be **monomorphized over a different arithmetic library**
//! without touching the carrier, the app, or any public API.
//!
//! Today there is exactly one implementation ([`BigFloat`] itself, where the conversions are
//! `clone`), so the generic loop is bit-identical to the hand-written one it replaced. A second
//! backend plugs in by implementing this trait; dispatch happens **once per orbit build**, never
//! per operation.
//!
//! # Contract for a second implementation
//!
//! A backend must reproduce astro-float's arithmetic **bit for bit**, or the F3 corpus goldens stop
//! gating it and every deep render needs its own blessed set. Measured against MPFR (validation
//! probe, 2026-08-26: 9 operand classes × 32 trials × 7 precisions from 64 to 3776 bits, plus
//! 20,000 iterations of the real `z²+c` recurrence — zero divergence), three conditions are each
//! load-bearing:
//!
//! 1. **Truncating arithmetic.** astro-float's [`RM`](crate::bignum::RM) is `RoundingMode::None`,
//!    documented in its source as "skip rounding operation". MPFR's equivalent is `Round::Zero`;
//!    its *default* nearest rounding does **not** match.
//! 2. **Word-granular precision.** astro-float rounds a requested precision up to whole 64-bit
//!    words (`bit_len_to_word_len(p) = (p + 63) / 64`). A backend honouring `p` exactly does not
//!    match — it must round up to `p.div_ceil(64) * 64`, which
//!    `word_width_matches_astro_floats_own_allocation` below pins against what astro-float
//!    actually allocates.
//! 3. **Truncating `f64` extraction.** [`to_f64_trunc`](RefBackend::to_f64_trunc) must agree with
//!    [`crate::to_f64`], which truncates (`ret |= m >> 12`). A round-to-nearest conversion shifts
//!    emitted samples by ~1 ulp *even when the bignum state is identical* — which is exactly how
//!    the validation probe's first run manufactured a divergence that did not exist.

use crate::fractal::Field;
use astro_float::BigFloat;
use std::sync::atomic::{AtomicU32, Ordering};

/// Every backend name, indexed by [`RefBackend::BIT`]. The list is feature-dependent: a name only
/// appears when the backend is actually compiled in, so it can never advertise something the
/// binary cannot run.
#[cfg(not(feature = "rug"))]
pub const BACKEND_NAMES: &[&str] = &["astro-float"];
#[cfg(feature = "rug")]
pub const BACKEND_NAMES: &[&str] = &["astro-float", "rug"];

/// Which backend the reference orbit should be iterated in.
///
/// `Rug` exists only when the `rug` feature is compiled in — so "asked for a backend this build
/// does not have" is a *parse* error at the CLI boundary rather than a value that has to be
/// checked (and could be forgotten) at every use.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackendChoice {
    /// astro-float — pure Rust, always available, the carrier type's own arithmetic.
    Astro,
    /// MPFR via `rug`. LGPL-3.0+, and unavailable on `x86_64-pc-windows-msvc`.
    #[cfg(feature = "rug")]
    Rug,
}

impl BackendChoice {
    /// The registry index this choice iterates under.
    pub fn bit(self) -> u32 {
        match self {
            BackendChoice::Astro => 0,
            #[cfg(feature = "rug")]
            BackendChoice::Rug => 1,
        }
    }
    pub fn name(self) -> &'static str {
        BACKEND_NAMES[self.bit() as usize]
    }
}

/// Parse `auto` | `astro` | `rug`.
///
/// ⚠**`rug` on a build without the feature is an ERROR, never a quiet fall back to astro-float.**
/// A silent downgrade would let a benchmark or a gate report numbers for a backend it was never
/// running — the silent-CLI-default class this project already enumerated and closed once.
pub fn parse_choice(s: &str) -> Result<BackendChoice, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        // `auto` = the fastest backend this build actually has.
        //
        // ⭐There is NO crossover precision to tune, and that is a measured result rather than a
        // convenient assumption: MPFR is faster at *every* precision tried, 2.47×–7.04× from 1 to
        // 129 limbs through the real engine (`--bench-bignum` reproduces the table). A threshold
        // here would be a constant nobody validated, guarding a comparison that never changes
        // sign. If a future backend does trade places with astro-float somewhere, this is where
        // that decision goes — and it must come with the measurement that found the crossing.
        //
        // Switching backend cannot change what is rendered: the two are byte-identical, which the
        // cross-backend matrix and `--bench-bignum`'s own per-row check both enforce.
        #[cfg(feature = "rug")]
        "auto" => Ok(BackendChoice::Rug),
        #[cfg(not(feature = "rug"))]
        "auto" => Ok(BackendChoice::Astro),
        "astro" | "astro-float" => Ok(BackendChoice::Astro),
        #[cfg(feature = "rug")]
        "rug" | "mpfr" => Ok(BackendChoice::Rug),
        #[cfg(not(feature = "rug"))]
        "rug" | "mpfr" => Err(
            "this build has no `rug` backend (it is an off-by-default cargo feature, and it does \
             not build on x86_64-pc-windows-msvc). Rebuild fractadyne-core with `--features rug` \
             using the GNU toolchain, or pass --bignum=astro."
                .to_string(),
        ),
        other => Err(format!(
            "unknown bignum backend {other:?} — expected one of: auto, astro, rug"
        )),
    }
}

/// Every backend this build can actually run, in registry order.
///
/// Derived from the same `cfg` as [`BackendChoice`] itself, so it cannot drift out of step with
/// what is compiled in — a list maintained by hand would eventually advertise a backend the binary
/// does not have, which is the failure this module exists to make impossible.
pub fn available_backends() -> Vec<BackendChoice> {
    vec![
        BackendChoice::Astro,
        #[cfg(feature = "rug")]
        BackendChoice::Rug,
    ]
}

/// The chosen backend, defaulting to astro-float.
static SELECTED: std::sync::OnceLock<BackendChoice> = std::sync::OnceLock::new();

/// Choose the backend. Call once, before anything renders; a second differing call is an error
/// rather than a silent no-op, because half a session in each backend is not a configuration any
/// gate describes.
pub fn select(choice: BackendChoice) -> Result<(), String> {
    match SELECTED.set(choice) {
        Ok(()) => Ok(()),
        Err(_) if SELECTED.get() == Some(&choice) => Ok(()),
        Err(_) => Err(format!(
            "the bignum backend is already set to {} and cannot be changed mid-session",
            SELECTED.get().map(|c| c.name()).unwrap_or("?")
        )),
    }
}

/// What `selected()` answers when nothing has been chosen: the fastest backend this build has.
///
/// ⭐⭐**A compiled-in backend that nothing selects is a feature that never runs** — and this
/// project has already shipped one of those (`reference pipelining`, guarded by a condition that
/// was unsatisfiable by construction, silently doing nothing for weeks). If someone takes the
/// trouble to build with `--features rug`, the default must be the thing they built. Safe because
/// the backends are byte-identical: the default changes speed, never output.
const fn default_choice() -> BackendChoice {
    #[cfg(feature = "rug")]
    {
        BackendChoice::Rug
    }
    #[cfg(not(feature = "rug"))]
    {
        BackendChoice::Astro
    }
}

/// The backend orbits will be built in.
pub fn selected() -> BackendChoice {
    *SELECTED.get().unwrap_or(&default_choice())
}

/// Is the MPFR runtime actually loadable in this process?
///
/// ⭐**Only meaningful for the accelerated Windows build**, where libmpfr-6.dll / libgmp-10.dll are
/// linked with DELAY loading (see `scripts/build-accelerated.ps1`): the process starts even when
/// they are missing, and this probe — a plain `LoadLibrary`, which never triggers the delay-load
/// helper — decides whether to use MPFR or fall back to astro-float. Everywhere else (non-Windows,
/// or a build without the `rug` feature) MPFR is either statically present or not compiled in, so
/// this is trivially `true` and nothing calls it in anger.
#[cfg(all(feature = "rug", target_os = "windows"))]
pub fn mpfr_runtime_available() -> bool {
    fn loadable(dll: &str) -> bool {
        use std::os::windows::ffi::OsStrExt;
        // Minimal Win32 FFI (the crate already talks to Win32 this way elsewhere; no new dep).
        extern "system" {
            fn LoadLibraryW(name: *const u16) -> *mut core::ffi::c_void;
            fn FreeLibrary(handle: *mut core::ffi::c_void) -> i32;
        }
        let wide: Vec<u16> =
            std::ffi::OsStr::new(dll).encode_wide().chain(std::iter::once(0)).collect();
        // SAFETY: `wide` is a NUL-terminated UTF-16 string that outlives the call. `LoadLibraryW`
        // returns a module handle or null; on success we release it immediately with `FreeLibrary`
        // (this is a presence probe, not a load — the real use goes through the delay-load stubs).
        unsafe {
            let h = LoadLibraryW(wide.as_ptr());
            if h.is_null() {
                false
            } else {
                FreeLibrary(h);
                true
            }
        }
    }
    loadable("libmpfr-6.dll") && loadable("libgmp-10.dll")
}

/// Non-Windows or non-rug: MPFR is either linked-and-present or not built in, never a
/// findable-at-runtime question, so the answer is always yes.
#[cfg(not(all(feature = "rug", target_os = "windows")))]
pub fn mpfr_runtime_available() -> bool {
    true
}

/// The user-facing message shown when the accelerated build cannot find its MPFR libraries and
/// falls back to astro-float. States what happened, WHERE the DLLs need to go (the folder the
/// program started from, so the fix is unambiguous), that images are unaffected, and — each on its
/// own line so they copy cleanly — where to get the libraries or the no-DLL standard download.
///
/// `exe_dir` is the directory Fractadyne is running from; `None` (e.g. it could not be determined)
/// falls back to naming the executable instead.
pub fn mpfr_missing_message(exe_dir: Option<&std::path::Path>) -> String {
    let where_to = match exe_dir {
        Some(d) => format!(
            "They must sit in the folder Fractadyne is running from — copy them here:\n\
             \x20   {}",
            d.display()
        ),
        None => "They must sit in the same folder as fractadyne.exe.".to_string(),
    };
    format!(
        "This is the accelerated (MPFR) build, but its math libraries could not be found.\n\n\
         libmpfr-6.dll and libgmp-10.dll (and libgcc_s_seh-1.dll, libwinpthread-1.dll) are missing.\n\
         {where_to}\n\n\
         Fractadyne has fallen back to its built-in arithmetic (astro-float). Your images are \
         unaffected — the two are byte-identical. Only the pause while a deep view's reference \
         orbit is built is slower.\n\n\
         To restore the faster path, keep every file from the download's .zip together (or \
         re-extract it). The libraries also come from:\n\
         https://www.msys2.org/  (packages mingw-w64-x86_64-gmp and mingw-w64-x86_64-mpfr)\n\
         https://gmplib.org/\n\
         https://www.mpfr.org/\n\n\
         Or use the standard download, which needs no DLLs:\n\
         https://github.com/WindySnowOwl/fractadyne/releases"
    )
}

/// The message shown if the MPFR libraries APPEAR during a session after a fallback (the user
/// dropped them in). The selected backend is fixed for the process — the design forbids running
/// half a session in each — so they take effect on the next launch, which this says plainly.
pub fn mpfr_found_message() -> String {
    "The MPFR math libraries are now present — Fractadyne will use the faster accelerated \
     arithmetic the next time you start it. This session continues with the built-in one."
        .to_string()
}

/// Decide the startup backend, falling back instead of crashing when the accelerated build cannot
/// find MPFR. Returns the chosen backend and whether it FELL BACK from MPFR to astro-float because
/// the libraries were missing; it does NOT call [`select`] (the caller does, so the one selection
/// point and its logging stay in one place) and does NOT format the message (the caller builds it
/// with the running directory).
///
/// `requested` is the explicit `--bignum` / `FRACTADYNE_BIGNUM` choice, or `None` to take the
/// build default. On a Windows accelerated build that wants MPFR but cannot load it, this returns
/// `(Astro, true)` — the load-bearing half of the delay-load design: without it a missing DLL is a
/// hard `0xC0000135` at process start (nothing can warn), and with delay loading alone the first
/// MPFR call would raise a delay-load exception. Choosing astro up front avoids both.
pub fn resolve_startup_backend(requested: Option<BackendChoice>) -> (BackendChoice, bool) {
    let want = requested.unwrap_or_else(default_choice);
    #[cfg(all(feature = "rug", target_os = "windows"))]
    {
        if want == BackendChoice::Rug && !mpfr_runtime_available() {
            return (BackendChoice::Astro, true);
        }
    }
    (want, false)
}

/// Every backend compiled into this build, with the versions that can be queried at runtime.
/// For MPFR/GMP those are the C libraries linked into this binary — the versions that actually
/// did the arithmetic, not ones named in a manifest.
pub fn built_in_backends() -> String {
    #[cfg(not(feature = "rug"))]
    {
        "astro-float".to_string()
    }
    #[cfg(feature = "rug")]
    {
        format!("astro-float, {}", crate::backend_rug::linked_versions())
    }
}

/// Bits for backends that have **actually completed a reference-orbit build** in this process.
///
/// ⭐Deliberately recorded at the point of execution rather than read back from configuration. A
/// stamp sourced from "which backend was requested" can disagree with what ran — a feature guarded
/// by an impossible condition, a silent fallback, a flag that never reached a child process — and
/// then every gate quoted against it is quoting the wrong build. This one cannot: nothing sets a
/// bit except an orbit that finished.
static OBSERVED: AtomicU32 = AtomicU32::new(0);

/// Names of the backends that have actually run a reference orbit, in registry order.
pub fn observed_backends() -> Vec<&'static str> {
    let m = OBSERVED.load(Ordering::Relaxed);
    BACKEND_NAMES
        .iter()
        .enumerate()
        .filter(|(i, _)| m & (1 << i) != 0)
        .map(|(_, n)| *n)
        .collect()
}

/// One line for logs, crash reports and gate output. `none` is printed rather than omitted when no
/// orbit has run — a report silent about the backend cannot be told apart from one written by a
/// build that predates the mechanism (same reasoning as `tunables::status_line`).
pub fn status_line() -> String {
    match observed_backends()[..] {
        [] => "none (no reference orbit built yet)".to_string(),
        [one] => one.to_string(),
        ref many => format!("MIXED — {}", many.join(" + ")),
    }
}

/// A number type the reference orbit can be iterated in.
///
/// [`Field`] supplies the arithmetic (`fmul`/`fadd`/`fsub`/`fabs`/`fdouble`) that the per-family
/// steps in [`crate::fractal`] are already generic over, so a backend does **not** re-author any
/// formula. This trait adds only what the orbit loop needs around that: conversion to and from the
/// carrier type, a constant, and the sample extraction.
pub(crate) trait RefBackend: Field {
    /// This backend's index into [`BACKEND_NAMES`], and its bit in the observation mask.
    const BIT: u32;

    /// Build the per-call arithmetic context from `p` mantissa bits (for `BigFloat`, `p` itself).
    fn ctx_for(p: usize) -> Self::Ctx;

    /// Convert **exactly** from the carrier type. Must not round: the orbit's whole determinism
    /// contract starts here.
    fn from_carrier(v: &BigFloat, ctx: Self::Ctx) -> Self;

    /// Convert **exactly** back to the carrier type, so an [`crate::OrbitTail`] stays backend-
    /// agnostic and a later `extend` can resume from it.
    fn to_carrier(&self, ctx: Self::Ctx) -> BigFloat;

    /// A small exact constant (the Phoenix step's `0.5`).
    fn from_f64(v: f64, ctx: Self::Ctx) -> Self;

    /// The `f64` value **by truncation**, matching [`crate::to_f64`] bit for bit. See condition 3
    /// in the module docs — this is not interchangeable with a round-to-nearest conversion.
    fn to_f64_trunc(&self) -> f64;

    /// The value as an extended-range [`crate::FloatExp`]: the top 64 mantissa bits **by
    /// truncation**, rounded once 64→53 by the `u64 → f64` conversion, with the full binary
    /// exponent. Every backend must match the `BigFloat` recipe bit for bit — the pick's
    /// perturbation scorer consumes orbits recorded through this, so a divergent conversion
    /// would make the reference pick backend-dependent. Never under/overflows: a 1e-71
    /// near-nucleus orbit dip keeps its true magnitude.
    fn to_floatexp(&self) -> crate::floatexp::FloatExp;
}

/// Record that `B` actually built a reference orbit. Called from the one place an orbit is built,
/// so the stamp is evidence rather than intent.
pub(crate) fn note_observed<B: RefBackend>() {
    OBSERVED.fetch_or(1 << B::BIT, Ordering::Relaxed);
}

impl RefBackend for BigFloat {
    const BIT: u32 = 0;

    #[inline]
    fn ctx_for(p: usize) -> usize {
        p
    }
    #[inline]
    fn from_carrier(v: &BigFloat, _ctx: usize) -> Self {
        v.clone()
    }
    #[inline]
    fn to_carrier(&self, _ctx: usize) -> BigFloat {
        self.clone()
    }
    #[inline]
    fn from_f64(v: f64, ctx: usize) -> Self {
        BigFloat::from_f64(v, ctx)
    }
    #[inline]
    fn to_f64_trunc(&self) -> f64 {
        crate::bignum::to_f64(self)
    }

    #[inline]
    fn to_floatexp(&self) -> crate::floatexp::FloatExp {
        crate::bignum::bf_to_floatexp(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_width_matches_astro_floats_own_allocation() {
        // The rule a second backend has to match. Verified against what astro-float actually
        // allocates rather than against the formula restated.
        for p in [1usize, 63, 64, 65, 127, 128, 200, 576, 1088, 3776] {
            let v = BigFloat::from_f64(0.7071067811865476, p);
            let got = v.mantissa_digits().map(|d| d.len()).unwrap_or(0);
            assert_eq!(got, p.div_ceil(64), "astro-float allocated {got} words for p={p}");
        }
    }

    #[test]
    fn the_name_registry_agrees_with_the_trait() {
        // A backend whose BIT does not index its own name would mis-stamp every gate.
        assert_eq!(
            BACKEND_NAMES.get(<BigFloat as RefBackend>::BIT as usize),
            Some(&"astro-float")
        );
        assert!(BACKEND_NAMES.len() <= 32, "the observation mask is a u32");
    }

    #[test]
    fn the_carrier_backend_round_trips_identically() {
        let p = 576;
        let ctx = <BigFloat as RefBackend>::ctx_for(p);
        let v = crate::parse_bf_prec("-0.743643887037158704752191506114774", p).unwrap();
        let back = <BigFloat as RefBackend>::from_carrier(&v, ctx).to_carrier(ctx);
        assert_eq!(crate::to_decimal_string(&v), crate::to_decimal_string(&back));
        assert_eq!(v.to_f64_trunc(), crate::to_f64(&v));
    }

    /// Startup resolution never returns a backend this build cannot run, and only warns when it had
    /// to downgrade. On a build without `rug` the default is astro and there is nothing to probe, so
    /// there is never a warning. (The rug fallback path is compiled only into the accelerated build;
    /// it is exercised end-to-end by that package's clean-room verify, which runs the binary with
    /// its MPFR DLLs removed and asserts it falls back.)
    #[test]
    fn resolve_startup_backend_stays_within_this_build_and_warns_only_on_downgrade() {
        // The probe is trivially true when MPFR is statically present or not compiled in.
        assert!(mpfr_runtime_available() || cfg!(all(feature = "rug", target_os = "windows")));

        let (choice, fell_back) = resolve_startup_backend(None);
        assert!(available_backends().contains(&choice), "default is a backend this build lacks");

        let (c2, fb2) = resolve_startup_backend(Some(BackendChoice::Astro));
        assert_eq!(c2, BackendChoice::Astro);
        assert!(!fb2, "asking for astro never falls back");

        #[cfg(not(all(feature = "rug", target_os = "windows")))]
        {
            // No delay-load probe here, so resolution is a pure pass-through: never a fallback.
            assert!(!fell_back);
            assert_eq!(choice, default_choice());
        }
        let _ = fell_back;

        // The fallback message names the fix and the links, and the running directory when known.
        let m = mpfr_missing_message(Some(std::path::Path::new("C:\\Fractadyne\\accel")));
        assert!(m.contains("astro-float") && m.contains("releases") && m.contains(".dll"));
        assert!(m.contains("C:\\Fractadyne\\accel"), "the running directory is named");
        assert!(m.contains("https://gmplib.org/") && m.contains("https://www.mpfr.org/"));
        // Each URL on its own line (for clean copy-paste): no two https on the same line.
        assert!(m.lines().all(|l| l.matches("https://").count() <= 1), "two URLs share a line");
        // The no-dir form still names the executable.
        assert!(mpfr_missing_message(None).contains("fractadyne.exe"));
        // The recovery message says the next launch, not this session.
        assert!(mpfr_found_message().contains("next time you start"));
    }
}
