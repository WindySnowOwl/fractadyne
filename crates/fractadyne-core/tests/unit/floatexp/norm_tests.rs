use super::*;

/// The normalization `FloatExp::norm` used before 2026-10-19 (estimate with `log2`, scale with
/// `powi`, adjust), kept here as the reference the fast path must match bit for bit. ⛔Only for
/// NORMAL inputs: below 2^-1023 its `powi(-k)` overflows to infinity and the halving loop never
/// ends (the first run of this test hung there).
fn old_norm(mut m: f64, mut e: i32) -> FloatExp {
    assert!(m == 0.0 || !m.is_finite() || m.abs() >= f64::MIN_POSITIVE, "old_norm never ends on {m:e}");
    if m == 0.0 || !m.is_finite() {
        return FloatExp { m, e: 0 };
    }
    let a = m.abs();
    if !(1.0..2.0).contains(&a) {
        let k = a.log2().floor() as i32;
        m *= 2f64.powi(-k);
        e += k;
        while m.abs() >= 2.0 {
            m *= 0.5;
            e += 1;
        }
        while m.abs() < 1.0 {
            m *= 2.0;
            e -= 1;
        }
    }
    FloatExp { m, e }
}

/// A small deterministic generator (xorshift64*), so the test needs no crate.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
}

fn same(a: FloatExp, b: FloatExp) -> bool {
    a.m.to_bits() == b.m.to_bits() && a.e == b.e
}

/// `v` normalized: `|m|` in `[1, 2)` with `m·2^e == v` exactly (as `f64`, `e` near 0).
fn is_exact_norm_of(n: FloatExp, v: f64) -> bool {
    (1.0..2.0).contains(&n.m.abs()) && ldexp_f64(n.m, n.e) == v
}

#[test]
fn the_fast_normalization_matches_the_old_one_bit_for_bit() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut normal, mut subnormal) = (0, 0);
    // Random bit patterns: every exponent field, subnormals and both signs included.
    for _ in 0..200_000 {
        let v = f64::from_bits(rng.next());
        if !v.is_finite() || v == 0.0 {
            continue;
        }
        if v.abs() >= f64::MIN_POSITIVE {
            let e = (rng.next() % 200_001) as i32 - 100_000;
            let (n, o) = (FloatExp::norm(v, e), old_norm(v, e));
            assert!(same(n, o), "norm({v:e}, {e}) = {n:?}, old {o:?}");
            normal += 1;
        } else {
            assert!(is_exact_norm_of(FloatExp::norm(v, 0), v), "norm({v:e}) = {:?}", FloatExp::norm(v, 0));
            subnormal += 1;
        }
    }
    // The edges: powers of two and their neighbours across the whole range, both signs.
    for k in -1074..=1023 {
        let p = if k >= -1022 { pow2_normal(k) } else { f64::from_bits(1u64 << (k + 1074)) };
        for v in [p, f64::from_bits(p.to_bits() + 1), f64::from_bits(p.to_bits().saturating_sub(1))] {
            for s in [v, -v] {
                if s == 0.0 || !s.is_finite() {
                    continue;
                }
                if s.abs() >= f64::MIN_POSITIVE {
                    assert!(same(FloatExp::norm(s, 7), old_norm(s, 7)), "norm({s:e})");
                    normal += 1;
                } else {
                    assert!(is_exact_norm_of(FloatExp::norm(s, 0), s), "norm({s:e})");
                    subnormal += 1;
                }
            }
        }
    }
    assert!(normal > 190_000 && subnormal > 100, "both paths exercised: {normal} normal, {subnormal} subnormal");
    // The largest finite value, whose 2^-1023 scale is itself subnormal.
    for s in [f64::MAX, -f64::MAX, 1.5 * pow2_normal(1023)] {
        assert!(same(FloatExp::norm(s, 3), old_norm(s, 3)), "norm({s:e})");
    }
}

#[test]
fn addition_aligns_by_the_same_exact_power_of_two() {
    let mut rng = Rng(42);
    for _ in 0..50_000 {
        let a = FloatExp::norm(f64::from_bits(rng.next() >> 2 | 0x3ff0_0000_0000_0000), 0);
        let de = (rng.next() % 121) as i32;
        let b = FloatExp::norm(f64::from_bits(rng.next() >> 2 | 0x3ff0_0000_0000_0000), -de);
        let want = old_norm(a.m + b.m * 2f64.powi(-de), a.e);
        assert!(same(a + b, want), "de {de}: {:?} vs {want:?}", a + b);
    }
}
