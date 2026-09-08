//! Exact, compact serialization of a [`BigFloat`] — sign, exponent and mantissa words, verbatim.
//!
//! ⭐⭐**Why not decimal.** A `.fdn` stores coordinates as decimal because a human reads and pastes
//! them, and "correct to the view's precision" is all that a location needs. A cached reference
//! ORBIT is a different contract: the stored `f32` orbit is the orbit *of a specific point*, and the
//! per-pixel delta `c - c_ref` is computed at full precision against it. A point that comes back
//! differing in its last bits produces an orbit that is subtly not the one the pixels were rendered
//! against — a wrong picture, arrived at quickly, which is worse than a slow correct one.
//!
//! ⚠**And decimal round-tripping is not obviously exact here**: `astro-float`'s `FromStr` is lenient
//! (see `parse_bf`'s shape-validation), so "format then parse" would be trusting a property nothing
//! pins. Words in, words out has no such question — `mantissa_digits()` and `from_words` are inverse
//! by construction.
//!
//! ⚠A 200,000-bit number is ~25 KB here against ~60,000 decimal digits, which also happens to be
//! the smaller of the two.

use astro_float::{BigFloat, Sign};

/// Serialization format tag. ⛔Bump on any layout change: a cache entry written by an older build
/// must be REJECTED, not misread — the whole point of this codec is that a wrong point is a wrong
/// picture.
pub const BF_FORMAT: u8 = 1;

/// Append `v` to `out` as `[tag, sign, exponent i32, word count u32, words…]`, little-endian.
///
/// ⚠A zero/NaN/inf `BigFloat` has no mantissa WORDS; it takes a short form carrying only its
/// `f64` shadow, which is the same treatment the backends give it.
pub fn write_bf(v: &BigFloat, out: &mut Vec<u8>) {
    out.push(BF_FORMAT);
    // ⚠⚠**Zero is `exponent = Some(0)` with an EMPTY mantissa**, not a missing exponent — so
    // "has an exponent and digits" is not the same question as "has any digits". Writing it down
    // the normal path emitted a zero word count, which the reader (rightly) refuses as corrupt.
    let (Some(e), Some(d)) = (v.exponent(), v.mantissa_digits().filter(|d| !d.is_empty())) else {
        // Not a finite, normalized number: record the f64 shadow and nothing else.
        out.push(2u8); // sign slot doubles as "special"
        out.extend_from_slice(&crate::to_f64(v).to_le_bytes());
        return;
    };
    out.push(u8::from(v.sign() == Some(Sign::Neg)));
    out.extend_from_slice(&e.to_le_bytes());
    out.extend_from_slice(&(d.len() as u32).to_le_bytes());
    for w in d {
        out.extend_from_slice(&w.to_le_bytes());
    }
}

/// Read a `BigFloat` written by [`write_bf`], advancing `at`. `None` on any malformed input —
/// ⚠never a partially-read value, because a half-read coordinate is exactly the silent corruption
/// this module exists to prevent.
pub fn read_bf(buf: &[u8], at: &mut usize) -> Option<BigFloat> {
    let tag = *buf.get(*at)?;
    *at += 1;
    if tag != BF_FORMAT {
        return None;
    }
    let sign = *buf.get(*at)?;
    *at += 1;
    if sign == 2 {
        let b: [u8; 8] = buf.get(*at..*at + 8)?.try_into().ok()?;
        *at += 8;
        return Some(BigFloat::from_f64(f64::from_le_bytes(b), 64));
    }
    let e = i32::from_le_bytes(buf.get(*at..*at + 4)?.try_into().ok()?);
    *at += 4;
    let n = u32::from_le_bytes(buf.get(*at..*at + 4)?.try_into().ok()?) as usize;
    *at += 4;
    // ⚠Bound the allocation from the buffer we actually have, not from the claimed count: a
    // corrupt length must fail here rather than ask for gigabytes.
    if n == 0 || n > buf.len() / 8 + 1 {
        return None;
    }
    let mut words = Vec::with_capacity(n);
    for _ in 0..n {
        let b: [u8; 8] = buf.get(*at..*at + 8)?.try_into().ok()?;
        *at += 8;
        words.push(u64::from_le_bytes(b));
    }
    Some(BigFloat::from_words(
        &words,
        if sign == 1 { Sign::Neg } else { Sign::Pos },
        e,
    ))
}

/// Bytes `v` occupies, without building them — for sizing a buffer up front.
/// ⚠⚠**The condition must match [`write_bf`]'s exactly.** These disagreed for ZERO, which has an
/// exponent and an EMPTY digit slice: this reported the long form for a value the writer emitted in
/// the short one. Caught by the round-trip test's own `assert_eq!(buf.len(), bf_len(v))`, which is
/// there for exactly this.
pub fn bf_len(v: &BigFloat) -> usize {
    match (v.exponent(), v.mantissa_digits().filter(|d| !d.is_empty())) {
        (Some(_), Some(d)) => 1 + 1 + 4 + 4 + d.len() * 8,
        _ => 1 + 1 + 8,
    }
}

#[cfg(test)]
mod tests;
