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

/// Every backend name, indexed by [`RefBackend::BIT`]. One entry today; a second backend adds its
/// name here and takes the next bit.
pub const BACKEND_NAMES: &[&str] = &["astro-float"];

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
}
