//! A reference orbit, serialized — the artifact that makes returning to an extreme location cheap.
//!
//! ⭐⭐**What actually costs an hour.** At 9.98e60205× the reference orbit is ~200,000-bit arithmetic
//! run for up to two million iterations. What the GPU consumes from it is `[f32; 4]` per iteration —
//! **16 bytes**. The full-precision numbers are scaffolding, thrown away. So the thing that took an
//! hour is 4.1 MB at the live cap (`LIVE_REF_CAP` = 256,000) and 32 MB at full quality: small enough
//! to keep, which is the whole opportunity.
//!
//! ⭐**One orbit serves a NEIGHBOURHOOD, not a point.** Perturbation renders every pixel as a delta
//! from the reference, so a saved orbit covers every nearby view too — which is exactly the case
//! that motivated this: come back, then zoom somewhere else nearby.
//!
//! ⛔⭐⭐**The failure mode here is a WRONG PICTURE, not a slow one.** A blob that decodes into a
//! subtly different reference renders quickly and incorrectly, and nothing downstream would notice.
//! That is why the point is stored as exact words (`fractadyne_core::bfbytes`) rather than decimal,
//! why every field that could change the orbit is written into the header, and why the whole thing
//! carries a digest. ⚠It is also why the gate that matters is not "does it round-trip" but "does a
//! decoded orbit render BIT-IDENTICALLY to a freshly built one" — `--selftest`'s `orbit-cache`.
//!
//! **Layout (v2).** Two sections, each under its own digest, so the cache index can read what an
//! entry IS without reading the orbit it carries:
//!
//! ```text
//! prelude   MAGIC(8) VERSION(2) HEADER_LEN(4)
//! header    formula, julia, c, backend, prec, req_prec, iter, orbit_len, partial, file_len,
//!           point[2]
//!           HEADER_DIGEST(8)   — over prelude + header
//! tail      zx zy zpx zpy escaped backend      (only a decode needs it)
//! orbit     orbit_len × [f32; 4]
//!           FILE_DIGEST(8)     — over everything before it
//! ```
//!
//! ⭐The header is what the cache keys and ranks on (`refcache_persist`): the point is there so a
//! lookup can test "is this reference inside the view I am about to render" against every entry
//! with ~50 KB read per entry at extreme depth, instead of 4 MB.

use super::{RecomputeResult, ReuseRef};
use fractadyne_core::bfbytes::{bf_len, read_bf, write_bf};
use fractadyne_core::BigFloat;

const MAGIC: &[u8; 8] = b"FDNORBIT";

/// ⛔Bump on ANY layout or semantic change. An entry written by a different build must be refused,
/// never reinterpreted — see the module note on wrong pictures. (v1 never shipped in a release.)
const VERSION: u16 = 2;

/// Bytes before the header body: magic, version, header length.
const PRELUDE: usize = 8 + 2 + 4;

/// ⚠A header length past this is refused before anything is allocated. A ~200,000-bit point is
/// ~25 KB, so this leaves room for coordinates ~300× deeper than the deepest location we track.
const MAX_HEADER_LEN: usize = 16 << 20;

/// Everything about the VIEW that changes what the orbit is. ⚠Anything affecting the orbit and
/// missing from here is a cache key that can collide — the defect would be a correct-looking render
/// of the wrong place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct OrbitKey {
    pub(crate) formula_id: u32,
    pub(crate) julia: bool,
    pub(crate) julia_c: [f64; 2],
    /// Which `BackendChoice` built it. ⭐`OrbitTail::backend` exists because an extension must
    /// resume in the backend that started it; the same reasoning makes it part of the identity.
    pub(crate) backend: u32,
}

/// What the cache can know about an entry WITHOUT reading its orbit: the identity, the point, and
/// how much of an orbit it holds. Read by [`read_header`] from a file prefix.
#[derive(Clone)]
pub(crate) struct Header {
    pub(crate) key: OrbitKey,
    /// Precision the orbit was BUILT at (request + headroom): what a reuse must not exceed.
    pub(crate) prec: usize,
    pub(crate) req_prec: usize,
    pub(crate) iter: u32,
    pub(crate) orbit_len: u32,
    pub(crate) partial: bool,
    /// The whole file's length. ⭐So an index reading only headers can still refuse a truncated
    /// or padded file from its metadata — a body it never reads cannot be verified any other way,
    /// and an entry that only fails at load time would count against the budget until then.
    pub(crate) file_len: u64,
    pub(crate) point: [BigFloat; 2],
}

impl Header {
    /// The cache's file identity: everything that makes two orbits the SAME orbit (a longer build
    /// of the same point at the same precision replaces a shorter one under this id).
    pub(crate) fn key_id(&self) -> u64 {
        key_id(&self.key, self.prec, &self.point)
    }
}

/// A decoded entry, ready to be handed to the recompute path as a [`ReuseRef`].
pub(crate) struct DecodedOrbit {
    pub(crate) header: Header,
    pub(crate) reuse: ReuseRef,
}

fn digest(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn write_key(out: &mut Vec<u8>, key: &OrbitKey) {
    out.extend_from_slice(&key.formula_id.to_le_bytes());
    out.push(u8::from(key.julia));
    out.extend_from_slice(&key.julia_c[0].to_le_bytes());
    out.extend_from_slice(&key.julia_c[1].to_le_bytes());
    out.extend_from_slice(&key.backend.to_le_bytes());
}

/// The identity a cache file is named by. ⭐Over the exact point WORDS, which is the whole reason
/// `bfbytes` exists: a decimal round trip that moved a last bit would name a different file.
pub(crate) fn key_id(key: &OrbitKey, prec: usize, point: &[BigFloat; 2]) -> u64 {
    let mut b = Vec::with_capacity(64 + bf_len(&point[0]) + bf_len(&point[1]));
    write_key(&mut b, key);
    b.extend_from_slice(&(prec as u64).to_le_bytes());
    write_bf(&point[0], &mut b);
    write_bf(&point[1], &mut b);
    digest(&b)
}

/// Serialize `res` and the view identity that produced it.
///
/// ⚠**Only the orbit is stored — the series approximation and the BLA are NOT.** They are derived,
/// they are cheap next to the orbit (measured at 2.37e4000×: BLA 0.4 s against a 405 s build, and
/// SA is not computed at all when a BLA tree will exist), and the BLA depends on live colouring
/// settings (`bla_stripe_freq`, `bla_trap_type`), so caching it would drag those into the identity
/// for no gain. Rebuilding them on load is both simpler and more correct.
pub(crate) fn encode(res: &RecomputeResult, key: OrbitKey) -> Option<Vec<u8>> {
    let tail = res.orbit_tail.as_ref()?;
    // ⚠`orbit_len` is what the header promises and what decode reads back; it must be the
    // sample count actually written, or a decode would refuse its own writer's output.
    if res.orbit.is_empty() || res.orbit.len() != res.orbit_len as usize {
        return None;
    }
    let mut out = Vec::with_capacity(
        160 + res.orbit.len() * 16
            + bf_len(&res.rp[0])
            + bf_len(&res.rp[1])
            + bf_len(&tail.zx)
            + bf_len(&tail.zy)
            + bf_len(&tail.zpx)
            + bf_len(&tail.zpy),
    );
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // header length, patched below
    write_key(&mut out, &key);
    out.extend_from_slice(&(res.prec as u64).to_le_bytes());
    out.extend_from_slice(&(res.req_prec as u64).to_le_bytes());
    out.extend_from_slice(&res.iter.to_le_bytes());
    out.extend_from_slice(&res.orbit_len.to_le_bytes());
    out.push(u8::from(res.partial));
    let file_len_at = out.len();
    out.extend_from_slice(&0u64.to_le_bytes()); // file length, patched below
    write_bf(&res.rp[0], &mut out);
    write_bf(&res.rp[1], &mut out);
    let header_len = (out.len() - PRELUDE) as u32;
    out[10..14].copy_from_slice(&header_len.to_le_bytes());
    let hd_at = out.len();
    out.extend_from_slice(&[0u8; 8]); // header digest, sealed below
    write_bf(&tail.zx, &mut out);
    write_bf(&tail.zy, &mut out);
    write_bf(&tail.zpx, &mut out);
    write_bf(&tail.zpy, &mut out);
    out.push(u8::from(tail.escaped));
    out.extend_from_slice(&tail.backend.to_le_bytes());
    for p in res.orbit.iter() {
        for v in p {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    let file_len = (out.len() + 8) as u64;
    out[file_len_at..file_len_at + 8].copy_from_slice(&file_len.to_le_bytes());
    let hd = digest(&out[..hd_at]);
    out[hd_at..hd_at + 8].copy_from_slice(&hd.to_le_bytes());
    let d = digest(&out);
    out.extend_from_slice(&d.to_le_bytes());
    Some(out)
}

/// How many bytes from the start of a file [`read_header`] needs, given its first [`PRELUDE`]
/// bytes — so an index can read exactly the header and not the orbit behind it. `None` when the
/// prelude is not ours.
pub(crate) fn header_span(prelude: &[u8]) -> Option<usize> {
    if prelude.len() < PRELUDE || &prelude[..8] != MAGIC {
        return None;
    }
    if u16::from_le_bytes(prelude[8..10].try_into().ok()?) != VERSION {
        return None;
    }
    let header_len = u32::from_le_bytes(prelude[10..14].try_into().ok()?) as usize;
    if header_len > MAX_HEADER_LEN {
        return None;
    }
    Some(PRELUDE + header_len + 8)
}

/// Parse and verify the header alone. `buf` may be just a prefix of the file (at least
/// [`header_span`] bytes). Returns the header and the offset at which the tail begins.
///
/// ⚠⚠**Refuse, never repair** (as for [`decode`]): a header that does not verify is not an entry.
pub(crate) fn read_header(buf: &[u8]) -> Option<(Header, usize)> {
    let span = header_span(buf.get(..PRELUDE)?)?;
    let end = span - 8;
    let want = u64::from_le_bytes(buf.get(end..span)?.try_into().ok()?);
    if digest(&buf[..end]) != want {
        return None;
    }
    // Parse inside the verified bytes only, so a field can never read past its own section.
    let hb = &buf[..end];
    let mut at = PRELUDE;
    let u32le = |at: &mut usize| -> Option<u32> {
        let v = u32::from_le_bytes(hb.get(*at..*at + 4)?.try_into().ok()?);
        *at += 4;
        Some(v)
    };
    let f64le = |at: &mut usize| -> Option<f64> {
        let v = f64::from_le_bytes(hb.get(*at..*at + 8)?.try_into().ok()?);
        *at += 8;
        Some(v)
    };
    let u64le = |at: &mut usize| -> Option<u64> {
        let v = u64::from_le_bytes(hb.get(*at..*at + 8)?.try_into().ok()?);
        *at += 8;
        Some(v)
    };
    let formula_id = u32le(&mut at)?;
    let julia = *hb.get(at)? != 0;
    at += 1;
    let julia_c = [f64le(&mut at)?, f64le(&mut at)?];
    let backend = u32le(&mut at)?;
    let prec = usize::try_from(u64le(&mut at)?).ok()?;
    let req_prec = usize::try_from(u64le(&mut at)?).ok()?;
    let iter = u32le(&mut at)?;
    let orbit_len = u32le(&mut at)?;
    let partial = *hb.get(at)? != 0;
    at += 1;
    let file_len = u64le(&mut at)?;
    let point = [read_bf(hb, &mut at)?, read_bf(hb, &mut at)?];
    if at != end {
        return None; // the header carried bytes its writer would not have written
    }
    Some((
        Header {
            key: OrbitKey { formula_id, julia, julia_c, backend },
            prec,
            req_prec,
            iter,
            orbit_len,
            partial,
            file_len,
            point,
        },
        span,
    ))
}

/// Decode a whole blob. `None` on anything that is not exactly what [`encode`] wrote.
///
/// ⚠⚠**Refuse, never repair.** Unlike a pasted location — where a mangled hyphen has an obvious
/// intent and fixing it helps — a damaged orbit has no salvageable reading. The cost of refusing is
/// rebuilding it; the cost of accepting is a plausible picture of the wrong place.
pub(crate) fn decode(buf: &[u8]) -> Option<DecodedOrbit> {
    // ⭐The file digest covers everything before itself, so a truncated or flipped byte anywhere —
    // header, tail or orbit — fails here rather than downstream.
    let body = buf.len().checked_sub(8)?;
    if body < PRELUDE + 8 {
        return None;
    }
    let want = u64::from_le_bytes(buf[body..].try_into().ok()?);
    if digest(&buf[..body]) != want {
        return None;
    }
    let (header, mut at) = read_header(&buf[..body])?;
    if header.file_len != buf.len() as u64 {
        return None;
    }
    let (zx, zy) = (read_bf(buf, &mut at)?, read_bf(buf, &mut at)?);
    let (zpx, zpy) = (read_bf(buf, &mut at)?, read_bf(buf, &mut at)?);
    let escaped = *buf.get(at)? != 0;
    at += 1;
    let tail_backend = u32::from_le_bytes(buf.get(at..at + 4)?.try_into().ok()?);
    at += 4;
    let n = header.orbit_len as usize;
    // ⚠Exactly the promised samples must remain: fewer is truncation, more is trailing bytes,
    // and the count is bounded by the bytes actually present before anything is reserved.
    if n == 0 || n.checked_mul(16)? != body.checked_sub(at)? {
        return None;
    }
    let mut orbit = Vec::with_capacity(n);
    for _ in 0..n {
        let mut p = [0f32; 4];
        for slot in &mut p {
            *slot = f32::from_le_bytes(buf.get(at..at + 4)?.try_into().ok()?);
            at += 4;
        }
        orbit.push(p);
    }
    let reuse = ReuseRef {
        point: header.point.clone(),
        prefix: std::sync::Arc::new(orbit),
        tail: fractadyne_core::OrbitTail { zx, zy, zpx, zpy, escaped, backend: tail_backend },
        prec: header.prec,
    };
    Some(DecodedOrbit { header, reuse })
}

/// `pub(crate)` so the store's tests (`refcache_persist`) can build blobs the same way.
#[cfg(test)]
pub(crate) mod tests;
