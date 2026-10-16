//! The property the orbit cache rests on: a point that goes to disk comes back BIT-IDENTICAL.

use super::*;
use crate::parse_bf_prec;

/// Two `BigFloat`s are the same number iff sign, exponent and every mantissa word agree.
/// ⭐Compared this way rather than with `==`, which is a numeric comparison and would happily call
/// two differently-rounded values equal — the exact thing under test here.
fn identical(a: &BigFloat, b: &BigFloat) -> bool {
    a.sign() == b.sign()
        && a.exponent() == b.exponent()
        && a.mantissa_digits() == b.mantissa_digits()
}

fn roundtrip(v: &BigFloat) -> Option<BigFloat> {
    let mut buf = Vec::new();
    write_bf(v, &mut buf);
    assert_eq!(buf.len(), bf_len(v), "bf_len disagreed with what was written");
    let mut at = 0usize;
    let out = read_bf(&buf, &mut at)?;
    assert_eq!(at, buf.len(), "the reader did not consume exactly what was written");
    Some(out)
}

/// ⭐⭐**The deep case is the point.** A ~200,000-bit coordinate is what a 9.98e60205× location
/// carries, and it is the one where "close enough" silently produces the orbit of a different point.
#[test]
fn a_deep_coordinate_round_trips_bit_for_bit() {
    // ⚠`precision_for_magnification` cannot express this depth at all — 9.98e60205 is far past
    // `f64::MAX`, which is exactly why the octave-based entry point exists.
    let prec = crate::precision_for_octaves(200_000);
    // The centre of the deepest location we track, at the precision that view needs.
    let src = "-0.28041054305504546698407770028983979273643258419006230007410381499044388400475";
    let v = parse_bf_prec(src, prec.max(4096)).expect("parse");
    let back = roundtrip(&v).expect("decode");
    assert!(identical(&v, &back), "a deep coordinate did not survive the round trip");
    assert!(
        v.mantissa_digits().is_some_and(|d| d.len() >= 64),
        "the guard: this must actually be a wide number, or it proves nothing about deep points"
    );
}

#[test]
fn signs_zero_and_small_values_round_trip() {
    for (name, s, prec) in [
        ("negative", "-0.7436438870371587047521915061147", 512usize),
        ("positive", "0.1318259042053119704931320563851", 512),
        ("tiny", "1.0e-300", 512),
        ("huge", "1.0e300", 512),
        ("one", "1", 128),
    ] {
        let v = parse_bf_prec(s, prec).unwrap_or_else(|| panic!("{name} parse"));
        let back = roundtrip(&v).unwrap_or_else(|| panic!("{name} decode"));
        assert!(identical(&v, &back), "{name} changed across the round trip");
    }
    // Zero has no mantissa and takes the special path.
    let z = BigFloat::from_f64(0.0, 128);
    let back = roundtrip(&z).expect("zero decodes");
    assert_eq!(crate::to_f64(&back), 0.0);
}

/// ⚠Every truncation must be refused, not partially read. A cache file cut short by a full disk or
/// an interrupted write is the realistic corruption, and a half-read coordinate is a wrong picture.
#[test]
fn truncated_input_is_refused() {
    let v = parse_bf_prec("-0.743643887037158704752191506114774", 1024).unwrap();
    let mut buf = Vec::new();
    write_bf(&v, &mut buf);
    for cut in 0..buf.len() {
        let mut at = 0usize;
        assert!(
            read_bf(&buf[..cut], &mut at).is_none(),
            "a {cut}-byte prefix of a {}-byte value decoded to something",
            buf.len()
        );
    }
    // And the whole thing still works, so the loop above was not vacuous.
    let mut at = 0usize;
    assert!(read_bf(&buf, &mut at).is_some());
}

/// ⛔A format bump must make older entries UNREADABLE rather than misread.
#[test]
fn a_foreign_format_tag_is_refused() {
    let v = parse_bf_prec("-0.5", 128).unwrap();
    let mut buf = Vec::new();
    write_bf(&v, &mut buf);
    buf[0] = BF_FORMAT.wrapping_add(1);
    let mut at = 0usize;
    assert!(read_bf(&buf, &mut at).is_none());
}

/// ⚠A corrupt word count must not become an allocation request.
#[test]
fn an_absurd_word_count_is_refused() {
    let v = parse_bf_prec("-0.5", 128).unwrap();
    let mut buf = Vec::new();
    write_bf(&v, &mut buf);
    buf[6..10].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut at = 0usize;
    assert!(read_bf(&buf, &mut at).is_none());
}

/// ⭐A deep coordinate is SMALLER in words than in decimal digits — the reason this codec is also
/// the cheaper one, not merely the exact one.
#[test]
fn words_are_more_compact_than_decimal() {
    let prec = 200_000usize;
    let v = parse_bf_prec("-0.28041054305504546698407770028983979273643258419", prec).unwrap();
    let bytes = bf_len(&v);
    // 200,000 bits is ~60,200 decimal digits; the word form is ~25 KB.
    assert!(bytes < 40_000, "expected well under 40 KB for 200k bits, got {bytes}");
    assert!(bytes > 20_000, "the guard: this must really be a 200k-bit value, got {bytes}");
}
