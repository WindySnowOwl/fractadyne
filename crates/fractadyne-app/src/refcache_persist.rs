//! Persist the deep-zoom reference (orbit + SA + BLA) so the last view loads instantly next launch.
//!
//! At extreme depth the slow part of resuming a session is the *reference build* (bignum orbit +
//! series-approximation + BLA — up to ~10 s), not the GPU render (~ms). Saving that reference lets
//! the restored view re-render immediately: a live, navigable frame, not a static screenshot.
//!
//! We do NOT serialize the GPU-rendered pixels, and we do NOT recompute `best_reference` on load
//! (which could drift if its heuristic changed between builds). Instead we store the chosen
//! reference *point* as full-precision decimal strings (the same representation the session uses for
//! the view centre) alongside the already-computed df32 orbit/BLA/SA. On load we validate the saved
//! view-key against the restored session view (exact centre + zoom exponent + formula + Julia) and,
//! if it matches, install the reference directly — the render then treats it as an already-built
//! cache (no cold-start rebuild). Any mismatch (view changed, format bumped, corrupt file) simply
//! falls through to the normal rebuild, so this is a pure best-effort accelerator.
//!
//! Format: a small self-describing little-endian binary blob (`last_reference.bin`). Bump
//! `FORMAT_VERSION` whenever the orbit/BLA/SA byte layout or the reference-point meaning changes, so
//! a stale cache is discarded rather than rendered wrong.

use std::path::PathBuf;
use std::sync::Arc;

/// Bump when the on-disk layout OR the meaning of the stored reference changes (orbit/BLA/SA packing,
/// df32 mantissa convention, reference-point semantics). A mismatch discards the cache → safe rebuild.
const FORMAT_VERSION: u32 = 1;
const MAGIC: &[u8; 8] = b"FDNREFB\x01";

/// A reference cache snapshot for one view, with the view-key needed to validate it on load.
pub(crate) struct SavedRef {
    // View-key (must match the restored session view for the reference to be valid).
    pub center_x_str: String,
    pub center_y_str: String,
    pub upp_e: i32,
    pub formula_id: u32,
    pub julia: bool,
    pub julia_c: (f64, f64),
    // Reference data.
    pub rp_x_str: String,
    pub rp_y_str: String,
    pub orbit: Arc<Vec<[f32; 4]>>,
    pub orbit_len: u32,
    pub orbit_iter: u32,
    pub orbit_prec: u64,
    pub partial: bool,
    pub sa: fractadyne_core::SeriesSkip,
    pub bla: Arc<Vec<[f32; 4]>>,
    pub bla_dc_max_log2: f64,
}

/// `<config_dir>/last_reference.bin`, or `None` if no config dir is available.
pub(crate) fn path() -> Option<PathBuf> {
    fractadyne_state::config_dir().map(|d| d.join("last_reference.bin"))
}

// --- little-endian writers (append to a byte buffer) ---
fn w_u32(b: &mut Vec<u8>, v: u32) {
    b.extend_from_slice(&v.to_le_bytes());
}
fn w_u64(b: &mut Vec<u8>, v: u64) {
    b.extend_from_slice(&v.to_le_bytes());
}
fn w_i32(b: &mut Vec<u8>, v: i32) {
    b.extend_from_slice(&v.to_le_bytes());
}
fn w_f64(b: &mut Vec<u8>, v: f64) {
    b.extend_from_slice(&v.to_le_bytes());
}
fn w_f32x4(b: &mut Vec<u8>, v: &[f32; 4]) {
    for f in v {
        b.extend_from_slice(&f.to_le_bytes());
    }
}
fn w_str(b: &mut Vec<u8>, s: &str) {
    w_u32(b, s.len() as u32);
    b.extend_from_slice(s.as_bytes());
}
fn w_vec4(b: &mut Vec<u8>, v: &[[f32; 4]]) {
    w_u32(b, v.len() as u32);
    for e in v {
        w_f32x4(b, e);
    }
}

/// Serialize a snapshot into a byte buffer.
fn encode(r: &SavedRef) -> Vec<u8> {
    let mut b = Vec::with_capacity(16 + (r.orbit.len() + r.bla.len()) * 16);
    b.extend_from_slice(MAGIC);
    w_u32(&mut b, FORMAT_VERSION);
    w_str(&mut b, &r.center_x_str);
    w_str(&mut b, &r.center_y_str);
    w_i32(&mut b, r.upp_e);
    w_u32(&mut b, r.formula_id);
    b.push(r.julia as u8);
    w_f64(&mut b, r.julia_c.0);
    w_f64(&mut b, r.julia_c.1);
    w_str(&mut b, &r.rp_x_str);
    w_str(&mut b, &r.rp_y_str);
    w_u32(&mut b, r.orbit_len);
    w_u32(&mut b, r.orbit_iter);
    w_u64(&mut b, r.orbit_prec);
    b.push(r.partial as u8);
    w_u32(&mut b, r.sa.skip);
    w_f32x4(&mut b, &r.sa.a);
    w_i32(&mut b, r.sa.a_exp);
    w_f32x4(&mut b, &r.sa.b);
    w_i32(&mut b, r.sa.b_exp);
    w_f32x4(&mut b, &r.sa.c);
    w_i32(&mut b, r.sa.c_exp);
    w_f64(&mut b, r.bla_dc_max_log2);
    w_vec4(&mut b, &r.orbit);
    w_vec4(&mut b, &r.bla);
    b
}

// --- little-endian readers (advance an offset; any short read → None) ---
struct Cur<'a> {
    b: &'a [u8],
    off: usize,
}
impl<'a> Cur<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.b.get(self.off..self.off + n)?;
        self.off += n;
        Some(s)
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn f64(&mut self) -> Option<f64> {
        Some(f64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn f32x4(&mut self) -> Option<[f32; 4]> {
        Some([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn string(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        // Guard against a corrupt length demanding a huge allocation.
        if n > 1 << 20 {
            return None;
        }
        String::from_utf8(self.take(n)?.to_vec()).ok()
    }
    fn vec4(&mut self) -> Option<Vec<[f32; 4]>> {
        let n = self.u32()? as usize;
        // ~16 bytes/elem; cap at a sane maximum so a corrupt count can't OOM.
        if n > 64 << 20 {
            return None;
        }
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.f32x4()?);
        }
        Some(v)
    }
}

/// Deserialize a snapshot; `None` on any magic/version/format mismatch or short/corrupt read.
fn decode(bytes: &[u8]) -> Option<SavedRef> {
    let mut c = Cur { b: bytes, off: 0 };
    if c.take(8)? != MAGIC || c.u32()? != FORMAT_VERSION {
        return None;
    }
    let center_x_str = c.string()?;
    let center_y_str = c.string()?;
    let upp_e = c.i32()?;
    let formula_id = c.u32()?;
    let julia = c.u8()? != 0;
    let julia_c = (c.f64()?, c.f64()?);
    let rp_x_str = c.string()?;
    let rp_y_str = c.string()?;
    let orbit_len = c.u32()?;
    let orbit_iter = c.u32()?;
    let orbit_prec = c.u64()?;
    let partial = c.u8()? != 0;
    let sa = fractadyne_core::SeriesSkip {
        skip: c.u32()?,
        a: c.f32x4()?,
        a_exp: c.i32()?,
        b: c.f32x4()?,
        b_exp: c.i32()?,
        c: c.f32x4()?,
        c_exp: c.i32()?,
    };
    let bla_dc_max_log2 = c.f64()?;
    let orbit = Arc::new(c.vec4()?);
    let bla = Arc::new(c.vec4()?);
    Some(SavedRef {
        center_x_str,
        center_y_str,
        upp_e,
        formula_id,
        julia,
        julia_c,
        rp_x_str,
        rp_y_str,
        orbit,
        orbit_len,
        orbit_iter,
        orbit_prec,
        partial,
        sa,
        bla,
        bla_dc_max_log2,
    })
}

/// Write a snapshot to `path()`, atomically (temp file + rename). Best-effort: errors are ignored by
/// the caller — a failed save just means the next launch rebuilds the reference.
pub(crate) fn save(r: &SavedRef) -> std::io::Result<()> {
    let Some(p) = path() else {
        return Ok(());
    };
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = p.with_extension("bin.tmp");
    std::fs::write(&tmp, encode(r))?;
    std::fs::rename(&tmp, &p)
}

/// Load and decode `last_reference.bin`, or `None` if absent/corrupt. (The state-reset path removes
/// the whole config dir, so there's no separate delete here — a reset wipes this file with it.)
pub(crate) fn load() -> Option<SavedRef> {
    let bytes = std::fs::read(path()?).ok()?;
    decode(&bytes)
}

// The view-validity check (same centre/zoom/formula/Julia) lives at the call site in `main.rs`,
// where the viewport + bignum helpers are available — the centre must be compared numerically, not
// by decimal string (astro-float's to_string()/parse round-trip is not bit-stable at this precision).

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SavedRef {
        SavedRef {
            center_x_str: "-0.7436438870371587".to_string(),
            center_y_str: "0.13182590420531196".to_string(),
            upp_e: -125,
            formula_id: 0,
            julia: false,
            julia_c: (0.0, 0.0),
            rp_x_str: "-0.74364388703".to_string(),
            rp_y_str: "0.13182590420".to_string(),
            orbit: Arc::new(vec![[1.0, 2.0, 3.0, 4.0], [-0.5, 0.25, 1e30, -1e-30]]),
            orbit_len: 2,
            orbit_iter: 39928,
            orbit_prec: 448,
            partial: false,
            sa: fractadyne_core::SeriesSkip {
                skip: 7,
                a: [1.0, 0.0, 2.0, 0.0],
                a_exp: -3,
                b: [3.0, 0.0, 4.0, 0.0],
                b_exp: 5,
                c: [5.0, 0.0, 6.0, 0.0],
                c_exp: -7,
            },
            bla: Arc::new(vec![[9.0, 8.0, 7.0, 6.0]]),
            bla_dc_max_log2: -400.5,
        }
    }

    #[test]
    fn round_trips() {
        let r = sample();
        let d = decode(&encode(&r)).expect("decode");
        assert_eq!(d.center_x_str, r.center_x_str);
        assert_eq!(d.center_y_str, r.center_y_str);
        assert_eq!(d.upp_e, r.upp_e);
        assert_eq!(d.formula_id, r.formula_id);
        assert_eq!(d.rp_x_str, r.rp_x_str);
        assert_eq!(d.orbit_len, r.orbit_len);
        assert_eq!(d.orbit_iter, r.orbit_iter);
        assert_eq!(d.orbit_prec, r.orbit_prec);
        assert_eq!(d.partial, r.partial);
        assert_eq!(d.sa.skip, r.sa.skip);
        assert_eq!(d.sa.a, r.sa.a);
        assert_eq!(d.sa.c_exp, r.sa.c_exp);
        assert_eq!(d.bla_dc_max_log2, r.bla_dc_max_log2);
        assert_eq!(*d.orbit, *r.orbit);
        assert_eq!(*d.bla, *r.bla);
    }

    #[test]
    fn rejects_bad_magic_and_truncation() {
        assert!(decode(b"nope").is_none());
        let good = encode(&sample());
        assert!(decode(&good[..good.len() - 4]).is_none()); // truncated orbit/bla
        let mut bad = good.clone();
        bad[0] = b'X'; // corrupt magic
        assert!(decode(&bad).is_none());
    }
}
