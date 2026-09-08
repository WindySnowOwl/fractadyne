//! The blob codec's own guarantees. ⚠These pin the ENCODING; the claim that actually matters —
//! that a decoded orbit renders bit-identically to a freshly built one — needs a GPU and lives in
//! `--selftest` (`orbit-cache`), because only a render can prove it.

use super::*;
use fractadyne_core::{parse_bf_prec, OrbitTail};

fn key() -> OrbitKey {
    OrbitKey { formula_id: 0, julia: false, julia_c: [-0.8, 0.156], backend: 7 }
}

/// A `RecomputeResult` cannot be built here (its fields are produced by the worker), so the codec is
/// exercised through a hand-made blob with the same layout. ⚠That means this file pins the
/// DECODER against a known-good byte stream; `encode` is covered by the selftest round trip.
fn blob(orbit: &[[f32; 4]], prec: usize, iter: u32, k: OrbitKey) -> Vec<u8> {
    let rp = [
        parse_bf_prec("-0.743643887037158704752191506114774", prec).unwrap(),
        parse_bf_prec("0.131825904205311970493132056385139", prec).unwrap(),
    ];
    let tail = OrbitTail {
        zx: parse_bf_prec("0.25", prec).unwrap(),
        zy: parse_bf_prec("-0.5", prec).unwrap(),
        zpx: parse_bf_prec("0", prec).unwrap(),
        zpy: parse_bf_prec("0", prec).unwrap(),
        escaped: false,
        backend: k.backend,
    };
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&k.formula_id.to_le_bytes());
    out.push(u8::from(k.julia));
    out.extend_from_slice(&k.julia_c[0].to_le_bytes());
    out.extend_from_slice(&k.julia_c[1].to_le_bytes());
    out.extend_from_slice(&k.backend.to_le_bytes());
    out.extend_from_slice(&(prec as u64).to_le_bytes());
    out.extend_from_slice(&(prec as u64).to_le_bytes());
    out.extend_from_slice(&iter.to_le_bytes());
    out.extend_from_slice(&(orbit.len() as u32).to_le_bytes());
    out.push(1u8); // partial
    write_bf(&rp[0], &mut out);
    write_bf(&rp[1], &mut out);
    write_bf(&tail.zx, &mut out);
    write_bf(&tail.zy, &mut out);
    write_bf(&tail.zpx, &mut out);
    write_bf(&tail.zpy, &mut out);
    out.push(u8::from(tail.escaped));
    out.extend_from_slice(&tail.backend.to_le_bytes());
    out.extend_from_slice(&(orbit.len() as u32).to_le_bytes());
    for p in orbit {
        for v in p {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    let d = digest(&out);
    out.extend_from_slice(&d.to_le_bytes());
    out
}

fn sample_orbit(n: usize) -> Vec<[f32; 4]> {
    (0..n)
        .map(|i| {
            let t = i as f32 * 0.001;
            [t.sin(), t.cos(), t * 0.5, -t]
        })
        .collect()
}

#[test]
fn a_well_formed_blob_decodes_to_what_it_carried() {
    let orbit = sample_orbit(1000);
    let d = decode(&blob(&orbit, 1024, 60_000, key())).expect("decode");
    assert_eq!(d.key, key());
    assert_eq!(d.iter, 60_000);
    assert_eq!(d.orbit_len, orbit.len() as u32);
    assert!(d.partial);
    assert_eq!(d.reuse.prec, 1024);
    // ⭐Bit-exact on the orbit: an f32 that came back merely close would be a different orbit.
    assert_eq!(d.reuse.prefix.len(), orbit.len());
    for (i, (a, b)) in orbit.iter().zip(d.reuse.prefix.iter()).enumerate() {
        for c in 0..4 {
            assert_eq!(a[c].to_bits(), b[c].to_bits(), "orbit[{i}][{c}]");
        }
    }
    // And the reference point, which is the value a decimal round trip could have blunted.
    let want = parse_bf_prec("-0.743643887037158704752191506114774", 1024).unwrap();
    assert_eq!(d.reuse.point[0].mantissa_digits(), want.mantissa_digits());
    assert_eq!(d.reuse.point[0].exponent(), want.exponent());
}

/// ⛔⭐⭐**Every corruption must be REFUSED.** A damaged orbit has no salvageable reading: accepting
/// one renders a plausible picture of the wrong place, quickly, with nothing downstream to notice.
#[test]
fn every_single_byte_corruption_is_refused() {
    let good = blob(&sample_orbit(64), 512, 1000, key());
    assert!(decode(&good).is_some(), "the guard: the undamaged blob must decode");

    // Flip one bit in every byte in turn. The digest covers all of it, header and orbit alike.
    let mut accepted = Vec::new();
    for i in 0..good.len() {
        let mut bad = good.clone();
        bad[i] ^= 0x01;
        if decode(&bad).is_some() {
            accepted.push(i);
        }
    }
    assert!(accepted.is_empty(), "corruption accepted at byte offsets {accepted:?}");
}

#[test]
fn truncation_and_trailing_bytes_are_refused() {
    let good = blob(&sample_orbit(32), 512, 1000, key());
    for cut in 0..good.len() {
        assert!(decode(&good[..cut]).is_none(), "a {cut}-byte prefix decoded");
    }
    let mut extra = good.clone();
    extra.push(0);
    assert!(decode(&extra).is_none(), "trailing bytes must not be tolerated");
}

/// ⛔A version bump makes older entries unreadable rather than misread — the point of having one.
#[test]
fn a_foreign_version_is_refused() {
    let mut b = blob(&sample_orbit(16), 512, 1000, key());
    b[8..10].copy_from_slice(&VERSION.wrapping_add(1).to_le_bytes());
    // Re-digest so it fails on the VERSION, not on the checksum — otherwise this would pass
    // for the wrong reason and would keep passing if the version check were deleted.
    let body = b.len() - 8;
    let d = digest(&b[..body]);
    b[body..].copy_from_slice(&d.to_le_bytes());
    assert!(decode(&b).is_none());
}

#[test]
fn a_foreign_magic_is_refused() {
    let mut b = blob(&sample_orbit(16), 512, 1000, key());
    b[0] = b'X';
    assert!(decode(&b).is_none());
}

/// ⚠A corrupt orbit length must not become a huge allocation before it is rejected.
#[test]
fn an_absurd_orbit_length_is_refused_without_allocating() {
    let mut b = blob(&sample_orbit(16), 512, 1000, key());
    let body = b.len() - 8;
    // The orbit count is the last u32 before the payload.
    let count_at = body - 16 * 16 - 4;
    b[count_at..count_at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    let d = digest(&b[..body]);
    b[body..].copy_from_slice(&d.to_le_bytes());
    assert!(decode(&b).is_none());
}

/// ⭐The size claim the whole feature rests on: 16 bytes per iteration, so the live cap is ~4 MB.
#[test]
fn the_orbit_costs_sixteen_bytes_per_iteration() {
    let small = blob(&sample_orbit(1000), 512, 1000, key());
    let big = blob(&sample_orbit(2000), 512, 1000, key());
    assert_eq!(big.len() - small.len(), 1000 * 16, "16 bytes per orbit point");
    // At LIVE_REF_CAP the payload is ~4 MB, which is the number that makes caching worth doing.
    assert_eq!(crate::tunables::LIVE_REF_CAP as usize * 16, 4_096_000);
}
