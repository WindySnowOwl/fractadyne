//! Faithful mode-2 `Fe` (floatexp) simulator — mirrors mandelbrot.wgsl's df/Cdf/Fe arithmetic
//! (Dekker/Knuth error-free transforms), generic over the base float so the SAME machinery
//! (shared exponent + fe_add de>60 cutoff + Zhuoran rebasing) runs with a df32 mantissa (= the GPU,
//! F=f32) or a df64 mantissa (= wide, F=f64). If df32 reproduces the GPU speckle and df64 is smooth,
//! the fix is a wider delta-z mantissa and the shared-exponent machinery is sound. Env-gated:
//!   PROBE_FE="label|cx|cy|mag_log10|max_iter|prec|npix|height_px"

trait Flt:
    Copy
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Neg<Output = Self>
    + PartialOrd
{
    fn cv(x: f64) -> Self;
    fn f64v(self) -> f64;
    fn fma(self, b: Self, c: Self) -> Self; // self*b + c (fused)
    fn absv(self) -> Self;
    fn maxv(self, o: Self) -> Self;
    fn ldx(self, e: i32) -> Self; // self * 2^e (exact power of two)
    fn flog2v(self) -> i32; // floor(log2(|self|)); very-negative for 0
    fn sqrtv(self) -> Self; // sqrt at this precision (mirrors the shader's f32 sqrt)
}
impl Flt for f32 {
    fn cv(x: f64) -> f32 { x as f32 }
    fn f64v(self) -> f64 { self as f64 }
    fn fma(self, b: f32, c: f32) -> f32 { self.mul_add(b, c) }
    fn absv(self) -> f32 { self.abs() }
    fn maxv(self, o: f32) -> f32 { self.max(o) }
    fn ldx(self, e: i32) -> f32 { self * 2f32.powi(e) }
    fn sqrtv(self) -> f32 { self.sqrt() }
    fn flog2v(self) -> i32 {
        let m = self.abs();
        if m == 0.0 || !m.is_finite() { return -1_000_000; }
        let b = m.to_bits();
        let ef = ((b >> 23) & 0xff) as i32;
        if ef == 0 { return (m * 16_777_216.0).flog2v() - 24; } // subnormal -> scale by 2^24
        ef - 127
    }
}
impl Flt for f64 {
    fn cv(x: f64) -> f64 { x }
    fn f64v(self) -> f64 { self }
    fn fma(self, b: f64, c: f64) -> f64 { self.mul_add(b, c) }
    fn absv(self) -> f64 { self.abs() }
    fn maxv(self, o: f64) -> f64 { self.max(o) }
    fn ldx(self, e: i32) -> f64 { self * 2f64.powi(e) }
    fn flog2v(self) -> i32 {
        let m = self.abs();
        if m == 0.0 || !m.is_finite() { return -1_000_000; }
        let b = m.to_bits();
        let ef = ((b >> 52) & 0x7ff) as i32;
        if ef == 0 { return (m * 9_007_199_254_740_992.0).flog2v() - 53; } // subnormal -> *2^53
        ef - 1023
    }
    fn sqrtv(self) -> f64 { self.sqrt() }
}

// GPU f32 with SUBNORMAL FLUSH (FTZ): most GPUs flush |x| < 2^-126 to zero on every op. IEEE
// Rust f32 does not, so this isolates whether FTZ degrades the df32 error-free transforms.
fn ftz(x: f32) -> f32 { if x != 0.0 && x.abs() < f32::MIN_POSITIVE { 0.0 } else { x } }
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Ftz(f32);
impl std::ops::Add for Ftz { type Output = Ftz; fn add(self, o: Ftz) -> Ftz { Ftz(ftz(self.0 + o.0)) } }
impl std::ops::Sub for Ftz { type Output = Ftz; fn sub(self, o: Ftz) -> Ftz { Ftz(ftz(self.0 - o.0)) } }
impl std::ops::Mul for Ftz { type Output = Ftz; fn mul(self, o: Ftz) -> Ftz { Ftz(ftz(self.0 * o.0)) } }
impl std::ops::Neg for Ftz { type Output = Ftz; fn neg(self) -> Ftz { Ftz(-self.0) } }
impl Flt for Ftz {
    fn cv(x: f64) -> Ftz { Ftz(ftz(x as f32)) }
    fn f64v(self) -> f64 { self.0 as f64 }
    fn fma(self, b: Ftz, c: Ftz) -> Ftz { Ftz(ftz(self.0.mul_add(b.0, c.0))) }
    fn absv(self) -> Ftz { Ftz(self.0.abs()) }
    fn maxv(self, o: Ftz) -> Ftz { Ftz(self.0.max(o.0)) }
    fn ldx(self, e: i32) -> Ftz { Ftz(ftz(self.0 * 2f32.powi(e))) }
    fn flog2v(self) -> i32 { self.0.flog2v() }
    fn sqrtv(self) -> Ftz { Ftz(ftz(self.0.sqrt())) }
}
// GPU f32 with a NON-FUSED FMA: if the driver/compiler does not fuse `a*b+c` (or contracts it
// wrong), two_prod's error term collapses to 0 and the df32 mul loses ~24 bits. Everything else
// is exact IEEE f32.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Nf(f32);
impl std::ops::Add for Nf { type Output = Nf; fn add(self, o: Nf) -> Nf { Nf(self.0 + o.0) } }
impl std::ops::Sub for Nf { type Output = Nf; fn sub(self, o: Nf) -> Nf { Nf(self.0 - o.0) } }
impl std::ops::Mul for Nf { type Output = Nf; fn mul(self, o: Nf) -> Nf { Nf(self.0 * o.0) } }
impl std::ops::Neg for Nf { type Output = Nf; fn neg(self) -> Nf { Nf(-self.0) } }
impl Flt for Nf {
    fn cv(x: f64) -> Nf { Nf(x as f32) }
    fn f64v(self) -> f64 { self.0 as f64 }
    fn fma(self, b: Nf, c: Nf) -> Nf { Nf(self.0 * b.0 + c.0) } // NON-fused
    fn absv(self) -> Nf { Nf(self.0.abs()) }
    fn maxv(self, o: Nf) -> Nf { Nf(self.0.max(o.0)) }
    fn ldx(self, e: i32) -> Nf { Nf(self.0 * 2f32.powi(e)) }
    fn flog2v(self) -> i32 { self.0.flog2v() }
    fn sqrtv(self) -> Nf { Nf(self.0.sqrt()) }
}
#[derive(Clone, Copy)]
struct Df<F: Flt>(F, F);
#[derive(Clone, Copy)]
struct Cd<F: Flt> { re: Df<F>, im: Df<F> }
#[derive(Clone, Copy)]
struct Fe<F: Flt> { m: Cd<F>, e: i32 }

fn qsum<F: Flt>(a: F, b: F) -> Df<F> { let s = a + b; let e = b - (s - a); Df(s, e) }
fn tsum<F: Flt>(a: F, b: F) -> Df<F> { let s = a + b; let v = s - a; let e = (a - (s - v)) + (b - v); Df(s, e) }
fn tprod<F: Flt>(a: F, b: F) -> Df<F> { let p = a * b; let e = a.fma(b, -p); Df(p, e) }
fn dadd<F: Flt>(a: Df<F>, b: Df<F>) -> Df<F> {
    let s = tsum(a.0, b.0);
    let t = tsum(a.1, b.1);
    let c = s.1 + t.0;
    let v = qsum(s.0, c);
    let w = t.1 + v.1;
    qsum(v.0, w)
}
fn dneg<F: Flt>(a: Df<F>) -> Df<F> { Df(-a.0, -a.1) }
fn dsub<F: Flt>(a: Df<F>, b: Df<F>) -> Df<F> { dadd(a, dneg(b)) }
fn dmul<F: Flt>(a: Df<F>, b: Df<F>) -> Df<F> {
    let p = tprod(a.0, b.0);
    let e = p.1 + (a.0 * b.1 + a.1 * b.0);
    qsum(p.0, e)
}
fn cadd<F: Flt>(a: Cd<F>, b: Cd<F>) -> Cd<F> { Cd { re: dadd(a.re, b.re), im: dadd(a.im, b.im) } }
fn cmul<F: Flt>(a: Cd<F>, b: Cd<F>) -> Cd<F> {
    Cd { re: dsub(dmul(a.re, b.re), dmul(a.im, b.im)), im: dadd(dmul(a.re, b.im), dmul(a.im, b.re)) }
}
fn csqr<F: Flt>(a: Cd<F>) -> Cd<F> {
    let ri = dmul(a.re, a.im);
    Cd { re: dsub(dmul(a.re, a.re), dmul(a.im, a.im)), im: dadd(ri, ri) }
}

const FZE: i32 = -1_000_000_000;
fn fnorm<F: Flt>(m: Cd<F>, e: i32) -> Fe<F> {
    let mag = m.re.0.absv().maxv(m.im.0.absv());
    let sh = mag.flog2v();
    if sh <= -1_000_000 { return Fe { m, e: FZE }; }
    let m2 = Cd {
        re: Df(m.re.0.ldx(-sh), m.re.1.ldx(-sh)),
        im: Df(m.im.0.ldx(-sh), m.im.1.ldx(-sh)),
    };
    Fe { m: m2, e: e + sh }
}
fn fzero<F: Flt>() -> Fe<F> {
    Fe { m: Cd { re: Df(F::cv(0.0), F::cv(0.0)), im: Df(F::cv(0.0), F::cv(0.0)) }, e: FZE }
}
fn fadd<F: Flt>(a: Fe<F>, b: Fe<F>) -> Fe<F> {
    let (hi, lo) = if b.e > a.e { (b, a) } else { (a, b) };
    let de = hi.e - lo.e;
    if de > 60 { return hi; }
    let lom = Cd {
        re: Df(lo.m.re.0.ldx(-de), lo.m.re.1.ldx(-de)),
        im: Df(lo.m.im.0.ldx(-de), lo.m.im.1.ldx(-de)),
    };
    fnorm(cadd(hi.m, lom), hi.e)
}
fn fmul<F: Flt>(a: Fe<F>, b: Fe<F>) -> Fe<F> { fnorm(cmul(a.m, b.m), a.e + b.e) }
fn fsqr<F: Flt>(a: Fe<F>) -> Fe<F> { fnorm(csqr(a.m), a.e + a.e) }
fn fmulcdf<F: Flt>(a: Fe<F>, z: Cd<F>) -> Fe<F> { fnorm(cmul(a.m, z), a.e) }
fn ftwo<F: Flt>(a: Fe<F>) -> Fe<F> { Fe { m: a.m, e: a.e + 1 } }
fn flog2_fe<F: Flt>(a: Fe<F>) -> f64 {
    let mag = a.m.re.0.f64v().hypot(a.m.im.0.f64v());
    if mag == 0.0 { return f64::NEG_INFINITY; }
    mag.log2() + a.e as f64
}
// Shader-faithful |a| (mirrors `fe_abs_sf`): sqrt(re.hi² + im.hi²) as a SINGLE F (only ~24-bit
// precision for f32), normalized to a (mantissa∈[1,2), exp) scalar floatexp. This is the crux of the
// GPU/sim divergence — the sim's `flog2_fe` uses f64 (~52-bit), so it resolves `|zfull| < |dz|` at the
// near-zero orbit dips where the 24-bit f32 magnitudes tie → the shader misses the rebase (corpus 15).
fn abs_sf<F: Flt>(a: Fe<F>) -> (F, i32) {
    let mag = (a.m.re.0 * a.m.re.0 + a.m.im.0 * a.m.im.0).sqrtv();
    let sh = mag.flog2v();
    if sh <= -1_000_000 { return (mag, FZE); }
    (mag.ldx(-sh), a.e + sh)
}
// `sf_lt` for two normalized magnitudes: compare exponent, then mantissa at F precision (equivalent to
// the shader's `sf_add(a,-b).m.x < 0` for mantissas already in [1,2)).
fn sf_lt2<F: Flt>(a: (F, i32), b: (F, i32)) -> bool {
    if a.1 != b.1 { return a.1 < b.1; }
    a.0 < b.0
}
fn flo<F: Flt>(a: Fe<F>) -> (f64, f64) {
    let s = 2f64.powi(a.e.clamp(-127, 127));
    (a.m.re.0.f64v() * s, a.m.im.0.f64v() * s)
}
fn df_from<F: Flt>(v: f64) -> Df<F> {
    let hi = F::cv(v);
    let lo = F::cv(v - hi.f64v());
    Df(hi, lo)
}
fn fe_from<F: Flt>(vr: f64, vi: f64) -> Fe<F> {
    let mag = vr.abs().max(vi.abs());
    if mag == 0.0 { return fzero(); }
    let e = mag.log2().floor() as i32;
    let s = 2f64.powi(-e);
    fnorm(Cd { re: df_from(vr * s), im: df_from(vi * s) }, e)
}

// Reference-sample decode, mirroring orbit_is_ext / orbit_fe / orbit_cdf.
fn is_ext(r: &[f32; 4]) -> bool { r[0] == 0.0 && r[2].abs() >= 2.0 }
fn orbit_fe_sim<F: Flt>(r: &[f32; 4]) -> Fe<F> {
    if is_ext(r) {
        Fe {
            m: Cd { re: Df(F::cv((r[2] - 4.0) as f64), F::cv(0.0)), im: Df(F::cv(r[3] as f64), F::cv(0.0)) },
            e: r[1] as i32,
        }
    } else {
        fnorm(Cd { re: Df(F::cv(r[0] as f64), F::cv(r[2] as f64)), im: Df(F::cv(r[1] as f64), F::cv(r[3] as f64)) }, 0)
    }
}
fn orbit_cdf_sim<F: Flt>(r: &[f32; 4]) -> Cd<F> {
    if is_ext(r) {
        Cd { re: Df(F::cv(0.0), F::cv(0.0)), im: Df(F::cv(0.0), F::cv(0.0)) }
    } else {
        Cd { re: Df(F::cv(r[0] as f64), F::cv(r[2] as f64)), im: Df(F::cv(r[1] as f64), F::cv(r[3] as f64)) }
    }
}

// One pixel's escape iteration via the faithful Fe kernel (Mandelbrot, no BLA/SA — BLA-off is
// equally noisy, so plain perturbation reproduces it). `orbit` is the raw df32/ext samples.
// `gpu_sf`: rebase decision as the SHADER does it (24-bit `abs_sf`/`sf_lt2`) vs the sim's f64 `flog2_fe`.
fn fe_escape<F: Flt>(orbit: &[[f32; 4]], len: usize, dcx: f64, dcy: f64, max_iter: u32, gpu_sf: bool) -> i64 {
    let dc = fe_from::<F>(dcx, dcy);
    let mut dz = fzero::<F>();
    let mut ref_n = 0usize;
    for it in 0..max_iter as usize {
        let r = &orbit[ref_n];
        let t2z = if is_ext(r) {
            ftwo(fmul(dz, orbit_fe_sim::<F>(r)))
        } else {
            ftwo(fmulcdf(dz, orbit_cdf_sim::<F>(r)))
        };
        let t = fadd(t2z, fsqr(dz));
        dz = fadd(t, dc);
        ref_n += 1;
        let rn = &orbit[ref_n.min(len - 1)];
        let zfull = fadd(orbit_fe_sim::<F>(rn), dz);
        let (zfx, zfy) = flo(zfull);
        if zfx * zfx + zfy * zfy > 256.0 * 256.0 {
            return it as i64 + 1;
        }
        let rebase = if gpu_sf {
            sf_lt2(abs_sf(zfull), abs_sf(dz))
        } else {
            flog2_fe(zfull) < flog2_fe(dz)
        };
        if rebase || ref_n + 1 >= len {
            dz = zfull; // fe_sub(zfull, Z0=0) for Mandelbrot
            ref_n = 0;
        }
    }
    -1
}

/// Run the faithful Fe kernel at df32 (GPU) and df64 (wide) mantissa over a dense pixel row.
#[test]
fn probe_row_fe() {
    let Ok(spec) = std::env::var("PROBE_FE") else { return };
    let p: Vec<&str> = spec.split('|').collect();
    let (label, cx, cy) = (p[0], p[1], p[2]);
    let mag_log10: f64 = p[3].parse().unwrap();
    let max_iter: u32 = p[4].parse().unwrap();
    let prec: usize = p[5].parse().unwrap();
    let npix: usize = p.get(6).and_then(|s| s.parse().ok()).unwrap_or(48);
    let height_px: f64 = p.get(7).and_then(|s| s.parse().ok()).unwrap_or(180.0);
    let cx = fractadyne_core::parse_bf(cx).unwrap();
    let cy = fractadyne_core::parse_bf(cy).unwrap();
    let center = [cx.clone(), cy.clone()];
    let zero = fractadyne_core::BigFloat::from_f64(0.0, prec);
    let half = 1.5f64 * 10f64.powf(-mag_log10);
    let step = 2.0 * half / height_px;
    // Use the SAME reference the GPU picks (best_reference), not the center — the escape times are
    // reference-independent in exact arithmetic but NOT in df32, so this is the faithful comparison.
    let mag_log2 = mag_log10 * std::f64::consts::LN_10 / std::f64::consts::LN_2;
    let span = [
        fractadyne_core::FloatExp::from_f64(3.0 * 16.0 / 9.0).mul_pow2(-mag_log2),
        fractadyne_core::FloatExp::from_f64(3.0).mul_pow2(-mag_log2),
    ];
    let refpt = fractadyne_core::best_reference(&center, span, 0, false, [0.0, 0.0], max_iter, prec);
    let (orbit, len) = fractadyne_core::reference_orbit(&zero, &zero, &refpt[0], &refpt[1], 0, max_iter, prec);
    let len = len as usize;
    // Reference offset from the view centre (complex units), so each row pixel's dc = (pixel_c - ref).
    let rox = fractadyne_core::sub_f64(&refpt[0], &center[0], prec);
    let roy = fractadyne_core::sub_f64(&refpt[1], &center[1], prec);
    // df32 with the SHADER's 24-bit sf-rebase (= the GPU) vs df32 with the sim's f64-rebase vs df64 truth.
    let (mut r32g, mut r32f, mut r64) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..npix {
        let off = i as f64 - (npix as f64 - 1.0) / 2.0;
        let dcx = step * off - rox;
        let dcy = -roy;
        r32g.push(fe_escape::<f32>(&orbit, len, dcx, dcy, max_iter, true));
        r32f.push(fe_escape::<f32>(&orbit, len, dcx, dcy, max_iter, false));
        r64.push(fe_escape::<f64>(&orbit, len, dcx, dcy, max_iter, false));
    }
    // Jump count + max escape among ESCAPED px only (interior = -1), matching FEDUMP.
    let stats = |r: &[i64]| -> (usize, i64) {
        let e: Vec<i64> = r.iter().copied().filter(|&v| v >= 0).collect();
        let jumps = e.windows(2).filter(|w| (w[0] - w[1]).abs() > 2000).count();
        (jumps, e.iter().copied().max().unwrap_or(-1))
    };
    let (jg, mg) = stats(&r32g);
    let (jf, mf) = stats(&r32f);
    let (j6, m6) = stats(&r64);
    let stride: Vec<i64> = r32g.iter().step_by(8).copied().collect();
    println!("[fe] {label}: reference len={len}");
    println!("[fe] {label}: df32-GPUsf every-8th: {stride:?}");
    println!(
        "[fe] {label}: jumps>2000 / max-escape -- df32-GPUsf(24bit)={jg}/{mg}  df32-f64rebase={jf}/{mf}  df64={j6}/{m6}"
    );
    println!("[fe] {label}: EXPECT if the 24-bit rebase is the bug: GPUsf max-escape caps near the ref len ({len}) while f64-rebase reaches ~max_iter.");
}
