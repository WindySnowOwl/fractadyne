//! The extreme-depth precision self-consistency probes (`--validate-deep`, `--selftest`).
//! Attached to `bignum` (`#[path]`) and re-exported. Moved from `bignum.rs`.

use super::*;

/// Extreme-depth **precision self-consistency** probe (needs no external oracle): iterate
/// `z²+c` for `k` steps from a full-mantissa interior point at precision `p`, and again at
/// `p + guard` bits, then return how many leading base-2 bits of the result agree.
///
/// This is the standard precision-doubling validation technique: if the `p`-bit answer is
/// stable under a precision increase it is almost certainly correct. Sound `p`-bit
/// arithmetic gives agreement ≈ `p − log₂(k)`; a precision-propagation or arithmetic bug at
/// that bit-width collapses it. Feasible to any depth (cost ∝ `k · M(p)` with FFT multiply),
/// unlike a per-pixel dwell oracle.
pub fn deep_consistency_bits(p: usize, guard: usize, k: u32) -> i64 {
    let pg = p + guard;
    let (cx, cy) = deep_test_point(pg);
    let lo = iter_zsq_c(&cx, &cy, k, p);
    let hi = iter_zsq_c(&cx, &cy, k, pg);
    agree_bits(&lo.0, &hi.0, pg).min(agree_bits(&lo.1, &hi.1, pg))
}

/// Round-trip a full-mantissa coordinate through decimal at precision `p`
/// (`to_decimal_string` → `parse_bf`) and return the bits of agreement — validates the
/// persisted-coordinate I/O path (the deep-zoom save/restore format) at extreme precision.
pub fn deep_roundtrip_bits(p: usize) -> i64 {
    let (cx, _) = deep_test_point(p);
    match parse_bf(&to_decimal_string(&cx)) {
        Some(back) => agree_bits(&cx, &back, p),
        None => -1,
    }
}
