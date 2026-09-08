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
//! decoded orbit render BIT-IDENTICALLY to a freshly built one".

use super::{RecomputeResult, ReuseRef};
use fractadyne_core::bfbytes::{bf_len, read_bf, write_bf};

const MAGIC: &[u8; 8] = b"FDNORBIT";

/// ⛔Bump on ANY layout or semantic change. An entry written by a different build must be refused,
/// never reinterpreted — see the module note on wrong pictures.
const VERSION: u16 = 1;

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

/// A decoded orbit, ready to be handed to the recompute path as a [`ReuseRef`].
pub(crate) struct DecodedOrbit {
    pub(crate) key: OrbitKey,
    pub(crate) reuse: ReuseRef,
    pub(crate) iter: u32,
    pub(crate) orbit_len: u32,
    pub(crate) partial: bool,
}

fn digest(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Serialize `res` and the view identity that produced it.
///
/// ⚠**Only the orbit is stored — the series approximation and the BLA are NOT.** They are derived,
/// they are cheap next to the orbit, and the BLA in particular depends on live colouring settings
/// (`bla_stripe_freq`, `bla_trap_type`), so caching it would drag those into the identity for no
/// gain. Rebuilding them on load is both simpler and more correct.
pub(crate) fn encode(res: &RecomputeResult, key: OrbitKey) -> Option<Vec<u8>> {
    let tail = res.orbit_tail.as_ref()?;
    let mut out = Vec::with_capacity(
        128 + res.orbit.len() * 16
            + bf_len(&res.rp[0])
            + bf_len(&res.rp[1])
            + bf_len(&tail.zx)
            + bf_len(&tail.zy)
            + bf_len(&tail.zpx)
            + bf_len(&tail.zpy),
    );
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&key.formula_id.to_le_bytes());
    out.push(u8::from(key.julia));
    out.extend_from_slice(&key.julia_c[0].to_le_bytes());
    out.extend_from_slice(&key.julia_c[1].to_le_bytes());
    out.extend_from_slice(&key.backend.to_le_bytes());
    out.extend_from_slice(&(res.prec as u64).to_le_bytes());
    out.extend_from_slice(&(res.req_prec as u64).to_le_bytes());
    out.extend_from_slice(&res.iter.to_le_bytes());
    out.extend_from_slice(&res.orbit_len.to_le_bytes());
    out.push(u8::from(res.partial));
    write_bf(&res.rp[0], &mut out);
    write_bf(&res.rp[1], &mut out);
    write_bf(&tail.zx, &mut out);
    write_bf(&tail.zy, &mut out);
    write_bf(&tail.zpx, &mut out);
    write_bf(&tail.zpy, &mut out);
    out.push(u8::from(tail.escaped));
    out.extend_from_slice(&tail.backend.to_le_bytes());
    out.extend_from_slice(&(res.orbit.len() as u32).to_le_bytes());
    for p in res.orbit.iter() {
        for v in p {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    let d = digest(&out);
    out.extend_from_slice(&d.to_le_bytes());
    Some(out)
}

/// Decode a blob. `None` on anything that is not exactly what [`encode`] wrote.
///
/// ⚠⚠**Refuse, never repair.** Unlike a pasted location — where a mangled hyphen has an obvious
/// intent and fixing it helps — a damaged orbit has no salvageable reading. The cost of refusing is
/// rebuilding it; the cost of accepting is a plausible picture of the wrong place.
pub(crate) fn decode(buf: &[u8]) -> Option<DecodedOrbit> {
    if buf.len() < MAGIC.len() + 2 + 8 || &buf[..8] != MAGIC {
        return None;
    }
    // ⭐The digest covers everything before itself, so a truncated or flipped byte anywhere —
    // header or orbit — fails here rather than downstream.
    let body = buf.len().checked_sub(8)?;
    let want = u64::from_le_bytes(buf[body..].try_into().ok()?);
    if digest(&buf[..body]) != want {
        return None;
    }
    let mut at = 8usize;
    let u16le = |at: &mut usize| -> Option<u16> {
        let v = u16::from_le_bytes(buf.get(*at..*at + 2)?.try_into().ok()?);
        *at += 2;
        Some(v)
    };
    if u16le(&mut at)? != VERSION {
        return None;
    }
    let u32le = |at: &mut usize| -> Option<u32> {
        let v = u32::from_le_bytes(buf.get(*at..*at + 4)?.try_into().ok()?);
        *at += 4;
        Some(v)
    };
    let formula_id = u32le(&mut at)?;
    let julia = *buf.get(at)? != 0;
    at += 1;
    let f64le = |at: &mut usize| -> Option<f64> {
        let v = f64::from_le_bytes(buf.get(*at..*at + 8)?.try_into().ok()?);
        *at += 8;
        Some(v)
    };
    let julia_c = [f64le(&mut at)?, f64le(&mut at)?];
    let backend = u32le(&mut at)?;
    let u64le = |at: &mut usize| -> Option<u64> {
        let v = u64::from_le_bytes(buf.get(*at..*at + 8)?.try_into().ok()?);
        *at += 8;
        Some(v)
    };
    let prec = u64le(&mut at)? as usize;
    let _req_prec = u64le(&mut at)? as usize;
    let iter = u32le(&mut at)?;
    let orbit_len = u32le(&mut at)?;
    let partial = *buf.get(at)? != 0;
    at += 1;
    let rp = [read_bf(buf, &mut at)?, read_bf(buf, &mut at)?];
    let (zx, zy) = (read_bf(buf, &mut at)?, read_bf(buf, &mut at)?);
    let (zpx, zpy) = (read_bf(buf, &mut at)?, read_bf(buf, &mut at)?);
    let escaped = *buf.get(at)? != 0;
    at += 1;
    let tail_backend = u32le(&mut at)?;
    let n = u32le(&mut at)? as usize;
    // ⚠Bound against the bytes actually present before reserving anything.
    if n.checked_mul(16)? > body.saturating_sub(at) {
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
    if at != body {
        return None; // trailing bytes: not what we wrote
    }
    Some(DecodedOrbit {
        key: OrbitKey { formula_id, julia, julia_c, backend },
        reuse: ReuseRef {
            point: rp,
            prefix: std::sync::Arc::new(orbit),
            tail: fractadyne_core::OrbitTail { zx, zy, zpx, zpy, escaped, backend: tail_backend },
            prec,
        },
        iter,
        orbit_len,
        partial,
    })
}

#[cfg(test)]
mod tests;
