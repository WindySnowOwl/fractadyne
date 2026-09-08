//! The blob codec's own guarantees. ⚠These pin the ENCODING; the claim that actually matters —
//! that a decoded orbit renders bit-identically to a freshly built one — needs a GPU and lives in
//! `--selftest` (`orbit-cache`), because only a render can prove it.

use super::*;
use fractadyne_core::{parse_bf_prec, OrbitTail};

pub(crate) fn key() -> OrbitKey {
    OrbitKey { formula_id: 0, julia: false, julia_c: [-0.8, 0.156], backend: 7 }
}

const PX: &str = "-0.743643887037158704752191506114774";
const PY: &str = "0.131825904205311970493132056385139";

pub(crate) fn point(prec: usize) -> [BigFloat; 2] {
    [parse_bf_prec(PX, prec).unwrap(), parse_bf_prec(PY, prec).unwrap()]
}

/// Recompute both digests after a deliberate edit, so a test fails on the field it changed and not
/// on the checksum — otherwise it would pass for the wrong reason and keep passing if the check it
/// targets were deleted.
fn reseal(b: &mut [u8]) {
    let header_len = u32::from_le_bytes(b[10..14].try_into().unwrap()) as usize;
    let hend = PRELUDE + header_len;
    let hd = digest(&b[..hend]);
    b[hend..hend + 8].copy_from_slice(&hd.to_le_bytes());
    let body = b.len() - 8;
    let d = digest(&b[..body]);
    b[body..].copy_from_slice(&d.to_le_bytes());
}

/// A `RecomputeResult` cannot be built here (its fields are produced by the worker), so the codec is
/// exercised through a hand-made blob with the same layout. ⚠That means this file pins the
/// DECODER against a known-good byte stream; `encode` is covered by the selftest round trip.
pub(crate) fn blob(orbit: &[[f32; 4]], prec: usize, iter: u32, k: OrbitKey) -> Vec<u8> {
    blob_at(orbit, prec, iter, k, point(prec))
}

/// As [`blob`], at an explicit reference point (the store's tests need distinct identities).
pub(crate) fn blob_at(orbit: &[[f32; 4]], prec: usize, iter: u32, k: OrbitKey, rp: [BigFloat; 2]) -> Vec<u8> {
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
    out.extend_from_slice(&0u32.to_le_bytes());
    write_key(&mut out, &k);
    out.extend_from_slice(&(prec as u64).to_le_bytes());
    out.extend_from_slice(&(prec as u64).to_le_bytes());
    out.extend_from_slice(&iter.to_le_bytes());
    out.extend_from_slice(&(orbit.len() as u32).to_le_bytes());
    out.push(1u8); // partial
    let file_len_at = out.len();
    out.extend_from_slice(&0u64.to_le_bytes()); // file length, patched below
    write_bf(&rp[0], &mut out);
    write_bf(&rp[1], &mut out);
    let header_len = (out.len() - PRELUDE) as u32;
    out[10..14].copy_from_slice(&header_len.to_le_bytes());
    out.extend_from_slice(&[0u8; 8]); // header digest, sealed below
    write_bf(&tail.zx, &mut out);
    write_bf(&tail.zy, &mut out);
    write_bf(&tail.zpx, &mut out);
    write_bf(&tail.zpy, &mut out);
    out.push(u8::from(tail.escaped));
    out.extend_from_slice(&tail.backend.to_le_bytes());
    for p in orbit {
        for v in p {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out.extend_from_slice(&[0u8; 8]); // file digest, sealed below
    let file_len = out.len() as u64;
    out[file_len_at..file_len_at + 8].copy_from_slice(&file_len.to_le_bytes());
    reseal(&mut out);
    out
}

pub(crate) fn sample_orbit(n: usize) -> Vec<[f32; 4]> {
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
    assert_eq!(d.header.key, key());
    assert_eq!(d.header.iter, 60_000);
    assert_eq!(d.header.orbit_len, orbit.len() as u32);
    assert!(d.header.partial);
    assert_eq!(d.reuse.prec, 1024);
    assert_eq!(d.reuse.tail.backend, key().backend);
    // ⭐Bit-exact on the orbit: an f32 that came back merely close would be a different orbit.
    assert_eq!(d.reuse.prefix.len(), orbit.len());
    for (i, (a, b)) in orbit.iter().zip(d.reuse.prefix.iter()).enumerate() {
        for c in 0..4 {
            assert_eq!(a[c].to_bits(), b[c].to_bits(), "orbit[{i}][{c}]");
        }
    }
    // And the reference point, which is the value a decimal round trip could have blunted.
    let want = parse_bf_prec(PX, 1024).unwrap();
    assert_eq!(d.reuse.point[0].mantissa_digits(), want.mantissa_digits());
    assert_eq!(d.reuse.point[0].exponent(), want.exponent());
}

/// ⭐The property the cache index rests on: the header can be read from a file PREFIX — exactly
/// `header_span` bytes — and says the same thing the full decode says.
#[test]
fn the_header_reads_from_a_prefix_and_agrees_with_decode() {
    let b = blob(&sample_orbit(500), 768, 9000, key());
    let span = header_span(&b[..PRELUDE]).expect("our prelude");
    assert!(span < b.len() / 4, "the header must be a small fraction of the file ({span} of {})", b.len());
    let (h, tail_at) = read_header(&b[..span]).expect("header from a prefix");
    assert_eq!(tail_at, span);
    let d = decode(&b).expect("decode");
    assert_eq!(h.key, d.header.key);
    assert_eq!((h.prec, h.req_prec, h.iter, h.orbit_len, h.partial), (768, 768, 9000, 500, true));
    assert_eq!(h.file_len, b.len() as u64, "the header knows the whole file's length");
    assert_eq!(h.key_id(), d.header.key_id());
    // One byte short of the span is not a header.
    assert!(read_header(&b[..span - 1]).is_none(), "a prefix short of the header digest decoded");
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

    // And the header on its own, read the way the index reads it: every byte of the header
    // section is covered by the header digest, so a flip anywhere in it is refused there too.
    let span = header_span(&good[..PRELUDE]).unwrap();
    let mut accepted = Vec::new();
    for i in 0..span {
        let mut bad = good[..span].to_vec();
        bad[i] ^= 0x01;
        if read_header(&bad).is_some() {
            accepted.push(i);
        }
    }
    assert!(accepted.is_empty(), "header corruption accepted at byte offsets {accepted:?}");
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
    reseal(&mut b);
    assert!(decode(&b).is_none());
    assert!(header_span(&b[..PRELUDE]).is_none(), "the index must not even size a foreign version");
}

#[test]
fn a_foreign_magic_is_refused() {
    let mut b = blob(&sample_orbit(16), 512, 1000, key());
    b[0] = b'X';
    assert!(decode(&b).is_none());
    assert!(header_span(&b[..PRELUDE]).is_none());
}

/// ⚠A corrupt orbit length must not become a huge allocation before it is rejected.
#[test]
fn an_absurd_orbit_length_is_refused_without_allocating() {
    let mut b = blob(&sample_orbit(16), 512, 1000, key());
    // `orbit_len` sits in the header after formula(4) julia(1) c(16) backend(4) prec(8) req(8) iter(4).
    let at = PRELUDE + 4 + 1 + 16 + 4 + 8 + 8 + 4;
    b[at..at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    reseal(&mut b);
    assert!(decode(&b).is_none());
    // The same for a header that claims to be enormous: refused from the prelude alone.
    let mut huge = blob(&sample_orbit(16), 512, 1000, key());
    huge[10..14].copy_from_slice(&(u32::MAX).to_le_bytes());
    assert!(header_span(&huge[..PRELUDE]).is_none());
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

/// ⭐⭐**Key stability, the reason the point is stored as words.** The same point names the same
/// file every time; a point that differs in its LAST mantissa word names a different one. A
/// decimal round trip that wobbled a trailing digit would break the first property silently.
#[test]
fn the_key_is_stable_and_last_word_sensitive() {
    let p = point(1024);
    let a = key_id(&key(), 1024, &p);
    assert_eq!(a, key_id(&key(), 1024, &point(1024)), "the same point must name the same entry");
    // Nudge the last word of x by one unit.
    let words: Vec<u64> = p[0].mantissa_digits().unwrap().to_vec();
    let mut nudged = words.clone();
    nudged[0] ^= 1;
    let q = BigFloat::from_words(&nudged, p[0].sign().unwrap(), p[0].exponent().unwrap());
    assert_ne!(a, key_id(&key(), 1024, &[q, p[1].clone()]), "a last-word change must be a different key");
    // Precision and backend are part of the identity too.
    assert_ne!(a, key_id(&key(), 1088, &p));
    assert_ne!(a, key_id(&OrbitKey { backend: 8, ..key() }, 1024, &p));
}
