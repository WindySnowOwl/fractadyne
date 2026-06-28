// Fractadyne — two-pass: iterate (→ R32Float) then color (→ screen).

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Shared full-screen triangle. uv is 0..1 across the view; (0,0) = top-left.
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var clip = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    var uv = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );
    var out: VsOut;
    out.pos = vec4<f32>(clip[vi], 0.0, 1.0);
    out.uv = uv[vi];
    return out;
}

// ---------------- double-single (df32) arithmetic --------------------------------
// A value is a vec2<f32> (hi, lo) with hi the rounded result and lo the error term,
// giving ~14 decimal digits (vs ~7 for plain f32). Error-free transforms below
// assume round-to-nearest and a *fused* fma (true on the target GPUs).

fn quick_two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = a + b;
    let e = b - (s - a);
    return vec2<f32>(s, e);
}
fn two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    return vec2<f32>(s, e);
}
fn two_prod(a: f32, b: f32) -> vec2<f32> {
    let p = a * b;
    let e = fma(a, b, -p);
    return vec2<f32>(p, e);
}
fn df_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let s = two_sum(a.x, b.x);
    let t = two_sum(a.y, b.y);
    let c = s.y + t.x;
    let v = quick_two_sum(s.x, c);
    let w = t.y + v.y;
    return quick_two_sum(v.x, w);
}
fn df_sub(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return df_add(a, vec2<f32>(-b.x, -b.y));
}
fn df_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let p = two_prod(a.x, b.x);
    let e = p.y + (a.x * b.y + a.y * b.x);
    return quick_two_sum(p.x, e);
}
fn df_mul_f32(a: vec2<f32>, b: f32) -> vec2<f32> {
    let p = two_prod(a.x, b);
    let e = p.y + a.y * b;
    return quick_two_sum(p.x, e);
}
// hi*2 is exact in binary floating point, so scale both limbs directly.
fn df_two(a: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(2.0 * a.x, 2.0 * a.y);
}
fn df_abs(a: vec2<f32>) -> vec2<f32> {
    if (a.x < 0.0) { return vec2<f32>(-a.x, -a.y); }
    return a;
}
// df32 division a/b (Newton needs it). Standard long-division refinement.
fn df_div(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let q1 = a.x / b.x;
    let p = two_prod(q1, b.x);
    let r = (((a.x - p.x) - p.y) + a.y) - q1 * b.y;
    let q2 = r / b.x;
    return quick_two_sum(q1, q2);
}

// Complex number in df32: real and imaginary parts each a (hi, lo) pair.
struct Cdf {
    re: vec2<f32>,
    im: vec2<f32>,
};
fn cset(r: vec2<f32>, i: vec2<f32>) -> Cdf {
    var c: Cdf;
    c.re = r;
    c.im = i;
    return c;
}
fn c_add(a: Cdf, b: Cdf) -> Cdf {
    return cset(df_add(a.re, b.re), df_add(a.im, b.im));
}
fn c_sub(a: Cdf, b: Cdf) -> Cdf {
    return cset(df_sub(a.re, b.re), df_sub(a.im, b.im));
}
fn c_mul(a: Cdf, b: Cdf) -> Cdf {
    return cset(
        df_sub(df_mul(a.re, b.re), df_mul(a.im, b.im)),
        df_add(df_mul(a.re, b.im), df_mul(a.im, b.re)),
    );
}
fn c_sqr(a: Cdf) -> Cdf {
    return c_mul(a, a);
}
fn c_scale(a: Cdf, s: f32) -> Cdf {
    return cset(df_mul_f32(a.re, s), df_mul_f32(a.im, s));
}
fn c_two(a: Cdf) -> Cdf {
    return cset(df_two(a.re), df_two(a.im));
}
fn c_conj(a: Cdf) -> Cdf {
    return cset(a.re, vec2<f32>(-a.im.x, -a.im.y));
}
fn c_div(a: Cdf, b: Cdf) -> Cdf {
    let d = df_add(df_mul(b.re, b.re), df_mul(b.im, b.im));
    let nr = df_add(df_mul(a.re, b.re), df_mul(a.im, b.im));
    let ni = df_sub(df_mul(a.im, b.re), df_mul(a.re, b.im));
    return cset(df_div(nr, d), df_div(ni, d));
}
fn c_mag2(a: Cdf) -> f32 {
    return a.re.x * a.re.x + a.im.x * a.im.x;
}

// ---------------- floatexp complex (df32 mantissa + shared i32 exponent) ----------
// value = m · 2^e, with the df32 complex mantissa `m` normalized so its larger hi
// word is ~[1,2). This extends df32's f32 *exponent* range to i32, so the
// perturbation δz no longer underflows f32 (~1e-38) at extreme depth → effectively
// unlimited zoom (bounded only by the reference orbit / iteration budget).
struct Fe {
    m: Cdf,
    e: i32,
};
const FE_ZERO_E: i32 = -1000000000; // exponent for a true zero (never dominates a sum)

fn fe_make(m: Cdf, e: i32) -> Fe {
    var f: Fe;
    f.m = m;
    f.e = e;
    return f;
}
fn fe_norm(m: Cdf, e: i32) -> Fe {
    let mag = max(abs(m.re.x), abs(m.im.x));
    if (mag == 0.0) {
        return fe_make(m, FE_ZERO_E);
    }
    let shift = i32(floor(log2(mag)));
    let s = exp2(f32(-shift)); // shift is tiny in steady iteration → s ~ 1, exact
    let m2 = cset(
        vec2<f32>(m.re.x * s, m.re.y * s),
        vec2<f32>(m.im.x * s, m.im.y * s),
    );
    return fe_make(m2, e + shift);
}
fn fe_zero() -> Fe {
    return fe_make(cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0)), FE_ZERO_E);
}
fn fe_add(a: Fe, b: Fe) -> Fe {
    var hi = a;
    var lo = b;
    if (b.e > a.e) { hi = b; lo = a; }
    let de = hi.e - lo.e; // >= 0
    if (de > 60) { return hi; } // lo below hi's df32 precision (~48 bits)
    let s = exp2(f32(-de));
    let lom = cset(
        vec2<f32>(lo.m.re.x * s, lo.m.re.y * s),
        vec2<f32>(lo.m.im.x * s, lo.m.im.y * s),
    );
    return fe_norm(c_add(hi.m, lom), hi.e);
}
fn fe_neg(a: Fe) -> Fe {
    return fe_make(cset(vec2<f32>(-a.m.re.x, -a.m.re.y), vec2<f32>(-a.m.im.x, -a.m.im.y)), a.e);
}
fn fe_sub(a: Fe, b: Fe) -> Fe {
    return fe_add(a, fe_neg(b));
}
fn fe_mul_cdf(a: Fe, z: Cdf) -> Fe {
    return fe_norm(c_mul(a.m, z), a.e);
}
fn fe_mul(a: Fe, b: Fe) -> Fe {
    return fe_norm(c_mul(a.m, b.m), a.e + b.e);
}
fn fe_sqr(a: Fe) -> Fe {
    return fe_norm(c_sqr(a.m), a.e + a.e);
}
fn fe_scale(a: Fe, s: f32) -> Fe {
    return fe_norm(c_scale(a.m, s), a.e);
}
fn fe_two(a: Fe) -> Fe {
    return fe_make(a.m, a.e + 1);
}
fn fe_conj(a: Fe) -> Fe {
    return fe_make(cset(a.m.re, vec2<f32>(-a.m.im.x, -a.m.im.y)), a.e);
}
fn fe_from_cdf(z: Cdf) -> Fe {
    return fe_norm(z, 0);
}
// Plain-f32 hi value of m·2^e (for bailout / full-orbit value); → 0 when e ≪ 0.
fn fe_lo_f32(a: Fe) -> vec2<f32> {
    let s = exp2(clamp(f32(a.e), -127.0, 127.0));
    return vec2<f32>(a.m.re.x * s, a.m.im.x * s);
}
fn fe_mag2(a: Fe) -> f32 {
    let s = exp2(clamp(f32(a.e) * 2.0, -250.0, 250.0));
    return (a.m.re.x * a.m.re.x + a.m.im.x * a.m.im.x) * s;
}

// ---------------- iteration pass (perturbation; writes smooth escape value) -----
// Per pixel: c = c0 + δc, z_n = Z_n + δz_n where Z_n is the reference orbit.
//   δz_{n+1} = 2·Z_n·δz_n + δz_n² + δc      (δz, δc carried in df32 → deep zoom)
struct IterU {
    step: vec4<f32>,       // per-texel step *mantissa* (scaled by 2^-delta_exp), df32
    ref_offset: vec4<f32>, // (center − reference) *mantissa* (scaled by 2^-delta_exp), df32
    center: vec4<f32>,     // view center df32: (re_hi, im_hi, re_lo, im_lo) — direct mode
    julia_c: vec4<f32>,    // Julia parameter df32: (re_hi, im_hi, re_lo, im_lo)
    res: vec2<f32>,        // FULL iteration resolution in texels (across all tiles)
    px_offset: vec2<f32>,  // this tile's top-left texel in the full grid (0 for live)
    max_iter: u32,
    orbit_len: u32,        // number of valid reference samples
    mode: u32,             // 0 = perturbation (deep), 1 = direct df32 (shallow)
    formula: u32,          // escape-time formula id (see fs_iterate)
    julia: u32,            // 0 = Mandelbrot mode (z0=0, c=pixel), 1 = Julia (z0=pixel, c=const)
    delta_exp: i32,        // shared base-2 exponent of the δ mantissas (step / ref_offset)
};
@group(0) @binding(0) var<uniform> iu: IterU;
// Reference orbit as double-single: each Z_n = (re.hi, im.hi, re.lo, im.lo).
@group(0) @binding(1) var<storage, read> reference: array<vec4<f32>>;

@fragment
fn fs_iterate(in: VsOut) -> @location(0) vec4<f32> {
    // Pixel offset from the view center, from the *exact integer* texel coordinate
    // (in.pos is the texel center = integer + 0.5, exact in f32 up to 2^24) times
    // the df32 per-texel step.
    let step_re = iu.step.xy;
    let step_im = iu.step.zw;
    // Global texel coordinate = this tile's offset + local fragment position.
    let gx = iu.px_offset.x + in.pos.x;
    let gy = iu.px_offset.y + in.pos.y;
    let coord_re = gx - iu.res.x * 0.5;
    let coord_im = iu.res.y * 0.5 - gy;
    let off_re = df_mul_f32(step_re, coord_re);
    let off_im = df_mul_f32(step_im, coord_im);

    let bail2 = 256.0 * 256.0;
    var iter: u32 = 0u;  // true iteration count
    var zf = vec2<f32>(0.0, 0.0);
    var escaped = false;

    if (iu.mode == 1u) {
        // Direct df32 (no reference). Glitch-free; used while depth is within df32's
        // reach (~1e6×). Mandelbrot beyond that uses perturbation; the other formulas
        // always use this path. Formula ids:
        //   0 Mandelbrot (z²+c)   1 Multibrot³ (z³+c)   2 Multibrot⁴   3 Multibrot⁵
        //   4 Tricorn (z̄²+c)      5 Burning Ship        6 Celtic       7 Buffalo
        //   8 Phoenix             9 Newton (z³−1)
        let zero = vec2<f32>(0.0, 0.0);
        // `off_*` are scaled by 2^-delta_exp (shared with perturbation); restore the
        // true offset for the direct path. delta_exp is small at shallow depth.
        let dsc = exp2(f32(iu.delta_exp));
        let pr = df_add(vec2<f32>(iu.center.x, iu.center.z), df_mul_f32(off_re, dsc));
        let pi = df_add(vec2<f32>(iu.center.y, iu.center.w), df_mul_f32(off_im, dsc));
        let newton = iu.formula == 9u;
        var z: Cdf;
        var c: Cdf;
        if (iu.julia == 1u || newton) {
            // Julia / Newton: z0 = point, c = fixed parameter (unused for Newton).
            z = cset(pr, pi);
            c = cset(vec2<f32>(iu.julia_c.x, iu.julia_c.z), vec2<f32>(iu.julia_c.y, iu.julia_c.w));
        } else {
            // Mandelbrot mode: z0 = 0, c = point under the texel.
            z = cset(zero, zero);
            c = cset(pr, pi);
        }
        var zprev = cset(zero, zero); // Phoenix needs z_{n-1}
        var power_f = 2.0;
        loop {
            if (iter >= iu.max_iter) { break; }
            if (newton) {
                // z ← z − (z³ − 1)/(3z²); converges to a cube root of unity.
                let z2 = c_sqr(z);
                let z3 = c_mul(z2, z);
                let f = cset(df_sub(z3.re, vec2<f32>(1.0, 0.0)), z3.im);
                z = c_sub(z, c_div(f, c_scale(z2, 3.0)));
                iter = iter + 1u;
                zf = vec2<f32>(z.re.x, z.im.x);
                if (c_mag2(f) < 1.0e-12) { escaped = true; break; }
            } else {
                var zn: Cdf;
                if (iu.formula == 0u) {
                    zn = c_sqr(z);
                } else if (iu.formula == 1u) {
                    zn = c_mul(c_sqr(z), z);
                    power_f = 3.0;
                } else if (iu.formula == 2u) {
                    zn = c_sqr(c_sqr(z));
                    power_f = 4.0;
                } else if (iu.formula == 3u) {
                    zn = c_mul(c_sqr(c_sqr(z)), z);
                    power_f = 5.0;
                } else if (iu.formula == 4u) {
                    // Tricorn / Mandelbar: conj(z)².
                    zn = c_sqr(cset(z.re, vec2<f32>(-z.im.x, -z.im.y)));
                } else if (iu.formula == 5u) {
                    // Burning Ship: (x²−y², 2|xy|).
                    let xx = df_mul(z.re, z.re);
                    let yy = df_mul(z.im, z.im);
                    zn = cset(df_sub(xx, yy), df_two(df_abs(df_mul(z.re, z.im))));
                } else if (iu.formula == 6u) {
                    // Celtic: (|x²−y²|, 2xy).
                    let xx = df_mul(z.re, z.re);
                    let yy = df_mul(z.im, z.im);
                    zn = cset(df_abs(df_sub(xx, yy)), df_two(df_mul(z.re, z.im)));
                } else if (iu.formula == 7u) {
                    // Buffalo: (|x²−y²|, |2xy|).
                    let xx = df_mul(z.re, z.re);
                    let yy = df_mul(z.im, z.im);
                    zn = cset(df_abs(df_sub(xx, yy)), df_two(df_abs(df_mul(z.re, z.im))));
                } else if (iu.formula == 8u) {
                    // Phoenix: z² + c + p·z_{n-1}, with p = −0.5.
                    let s = c_sqr(z);
                    zn = cset(
                        df_add(s.re, df_mul_f32(zprev.re, -0.5)),
                        df_add(s.im, df_mul_f32(zprev.im, -0.5)),
                    );
                } else {
                    zn = c_sqr(z);
                }
                zn = c_add(zn, c);
                zprev = z;
                z = zn;
                iter = iter + 1u;
                zf = vec2<f32>(z.re.x, z.im.x);
                if (dot(zf, zf) > bail2) { escaped = true; break; }
            }
        }
        if (!escaped) {
            return vec4<f32>(-1.0, 0.0, 0.0, 1.0);
        }
        if (newton) {
            // Color by convergence speed (iteration count).
            return vec4<f32>(f32(iter), 0.0, 0.0, 1.0);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        return vec4<f32>(f32(iter) + 1.0 - nu, 0.0, 0.0, 1.0);
    } else if (iu.mode == 2u) {
        // Floatexp perturbation (mode 2): δz/δc carried as floatexp (df32 mantissa +
        // i32 exponent), so the deviation never underflows f32 → unlimited depth. ~1.7×
        // costlier per iteration than mode 0, so it's used only past df32's reach. In
        // Mandelbrot mode the pixel deviation is δc (δz₀ = 0); in Julia mode it is δz₀
        // (δc = 0). `off_*` and `ref_offset` are mantissas sharing the exponent.
        let pert_m = cset(
            df_add(off_re, vec2<f32>(iu.ref_offset.x, iu.ref_offset.z)),
            df_add(off_im, vec2<f32>(iu.ref_offset.y, iu.ref_offset.w)),
        );
        let pert = fe_norm(pert_m, iu.delta_exp);
        var dz: Fe;
        var dc: Fe;
        if (iu.julia == 1u) {
            dz = pert;
            dc = fe_zero();
        } else {
            dz = fe_zero();
            dc = pert;
        }
        var ref_n: u32 = 0u;
        var power_f = 2.0;
        loop {
            if (iter >= iu.max_iter) { break; }
            let r = reference[ref_n];
            let Z = cset(vec2<f32>(r.x, r.z), vec2<f32>(r.y, r.w)); // reference Z_n (df32)

            if (iu.formula == 1u) {
                // z³: δz' = 3Z²δz + 3Z δz² + δz³ + δc
                power_f = 3.0;
                let Z2 = c_sqr(Z);
                let dz2 = fe_sqr(dz);
                var t = fe_scale(fe_mul_cdf(dz, Z2), 3.0);
                t = fe_add(t, fe_scale(fe_mul_cdf(dz2, Z), 3.0));
                t = fe_add(t, fe_mul(dz2, dz));
                dz = fe_add(t, dc);
            } else if (iu.formula == 2u) {
                // z⁴: δz' = 4Z³δz + 6Z²δz² + 4Z δz³ + δz⁴ + δc
                power_f = 4.0;
                let Z2 = c_sqr(Z);
                let Z3 = c_mul(Z2, Z);
                let dz2 = fe_sqr(dz);
                let dz3 = fe_mul(dz2, dz);
                var t = fe_scale(fe_mul_cdf(dz, Z3), 4.0);
                t = fe_add(t, fe_scale(fe_mul_cdf(dz2, Z2), 6.0));
                t = fe_add(t, fe_scale(fe_mul_cdf(dz3, Z), 4.0));
                t = fe_add(t, fe_sqr(dz2));
                dz = fe_add(t, dc);
            } else if (iu.formula == 3u) {
                // z⁵: δz' = 5Z⁴δz + 10Z³δz² + 10Z²δz³ + 5Z δz⁴ + δz⁵ + δc
                power_f = 5.0;
                let Z2 = c_sqr(Z);
                let Z3 = c_mul(Z2, Z);
                let Z4 = c_sqr(Z2);
                let dz2 = fe_sqr(dz);
                let dz3 = fe_mul(dz2, dz);
                let dz4 = fe_sqr(dz2);
                var t = fe_scale(fe_mul_cdf(dz, Z4), 5.0);
                t = fe_add(t, fe_scale(fe_mul_cdf(dz2, Z3), 10.0));
                t = fe_add(t, fe_scale(fe_mul_cdf(dz3, Z2), 10.0));
                t = fe_add(t, fe_scale(fe_mul_cdf(dz4, Z), 5.0));
                t = fe_add(t, fe_mul(dz4, dz));
                dz = fe_add(t, dc);
            } else if (iu.formula == 4u) {
                // Tricorn: δz' = 2·conj(Z)·conj(δz) + conj(δz)² + δc
                let cz = c_conj(Z);
                let cd = fe_conj(dz);
                var t = fe_two(fe_mul_cdf(cd, cz));
                t = fe_add(t, fe_sqr(cd));
                dz = fe_add(t, dc);
            } else {
                // Mandelbrot: δz' = 2Z·δz + δz² + δc
                var t = fe_two(fe_mul_cdf(dz, Z));
                t = fe_add(t, fe_sqr(dz));
                dz = fe_add(t, dc);
            }

            ref_n = ref_n + 1u;
            iter = iter + 1u;

            // Full value z = Z_{n+1} + δz, used for bailout and rebasing.
            let rn = reference[ref_n];
            let dzf = fe_lo_f32(dz);
            zf = vec2<f32>(rn.x + dzf.x, rn.y + dzf.y);
            let z2 = dot(zf, zf);
            if (z2 > bail2) { escaped = true; break; }

            let dzmag2 = fe_mag2(dz);
            if (z2 < dzmag2 || ref_n + 1u >= iu.orbit_len) {
                // Rebase onto reference index 0: δz = (Z_{n+1} + δz) − Z₀. Z₀ = 0 for
                // Mandelbrot, the reference point for Julia (subtraction required).
                let r0 = reference[0];
                let Zn = cset(vec2<f32>(rn.x, rn.z), vec2<f32>(rn.y, rn.w));
                let Z0 = cset(vec2<f32>(r0.x, r0.z), vec2<f32>(r0.y, r0.w));
                let zfull = fe_add(fe_from_cdf(Zn), dz);
                dz = fe_sub(zfull, fe_from_cdf(Z0));
                ref_n = 0u;
            }
        }
        if (!escaped) {
            return vec4<f32>(-1.0, 0.0, 0.0, 1.0);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        return vec4<f32>(f32(iter) + 1.0 - nu, 0.0, 0.0, 1.0);
    } else {
        // df32 perturbation (mode 0): the fast path for the common deep range. Valid
        // until the df32 δ's f32 exponent underflows (~1e30×); deeper zoom uses mode 2.
        // `off_*`/`ref_offset` are mantissas scaled by 2^-delta_exp → restore the real δ.
        let dsc = exp2(f32(iu.delta_exp));
        let pert = cset(
            df_mul_f32(df_add(off_re, vec2<f32>(iu.ref_offset.x, iu.ref_offset.z)), dsc),
            df_mul_f32(df_add(off_im, vec2<f32>(iu.ref_offset.y, iu.ref_offset.w)), dsc),
        );
        let zero = cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0));
        var dz: Cdf;
        var dc: Cdf;
        if (iu.julia == 1u) { dz = pert; dc = zero; } else { dz = zero; dc = pert; }
        var ref_n: u32 = 0u;
        var power_f = 2.0;
        loop {
            if (iter >= iu.max_iter) { break; }
            let r = reference[ref_n];
            let z = cset(vec2<f32>(r.x, r.z), vec2<f32>(r.y, r.w));
            if (iu.formula == 1u) {
                power_f = 3.0;
                let z2 = c_sqr(z); let dz2 = c_sqr(dz); let dz3 = c_mul(dz2, dz);
                var t = c_add(c_scale(c_mul(z2, dz), 3.0), c_scale(c_mul(z, dz2), 3.0));
                t = c_add(t, dz3); dz = c_add(t, dc);
            } else if (iu.formula == 2u) {
                power_f = 4.0;
                let z2 = c_sqr(z); let z3 = c_mul(z2, z);
                let dz2 = c_sqr(dz); let dz3 = c_mul(dz2, dz); let dz4 = c_sqr(dz2);
                var t = c_scale(c_mul(z3, dz), 4.0);
                t = c_add(t, c_scale(c_mul(z2, dz2), 6.0));
                t = c_add(t, c_scale(c_mul(z, dz3), 4.0));
                t = c_add(t, dz4); dz = c_add(t, dc);
            } else if (iu.formula == 3u) {
                power_f = 5.0;
                let z2 = c_sqr(z); let z3 = c_mul(z2, z); let z4 = c_sqr(z2);
                let dz2 = c_sqr(dz); let dz3 = c_mul(dz2, dz); let dz4 = c_sqr(dz2); let dz5 = c_mul(dz4, dz);
                var t = c_scale(c_mul(z4, dz), 5.0);
                t = c_add(t, c_scale(c_mul(z3, dz2), 10.0));
                t = c_add(t, c_scale(c_mul(z2, dz3), 10.0));
                t = c_add(t, c_scale(c_mul(z, dz4), 5.0));
                t = c_add(t, dz5); dz = c_add(t, dc);
            } else if (iu.formula == 4u) {
                let cz = c_conj(z); let cd = c_conj(dz);
                dz = c_add(c_add(c_two(c_mul(cz, cd)), c_sqr(cd)), dc);
            } else {
                dz = c_add(c_add(c_two(c_mul(z, dz)), c_sqr(dz)), dc);
            }
            ref_n = ref_n + 1u;
            iter = iter + 1u;
            let rn = reference[ref_n];
            let zr_full = df_add(vec2<f32>(rn.x, rn.z), dz.re);
            let zi_full = df_add(vec2<f32>(rn.y, rn.w), dz.im);
            zf = vec2<f32>(zr_full.x, zi_full.x);
            let z2 = dot(zf, zf);
            if (z2 > bail2) { escaped = true; break; }
            let dzmag2 = dz.re.x * dz.re.x + dz.im.x * dz.im.x;
            if (z2 < dzmag2 || ref_n + 1u >= iu.orbit_len) {
                let r0 = reference[0];
                dz = cset(
                    df_sub(zr_full, vec2<f32>(r0.x, r0.z)),
                    df_sub(zi_full, vec2<f32>(r0.y, r0.w)),
                );
                ref_n = 0u;
            }
        }
        if (!escaped) {
            return vec4<f32>(-1.0, 0.0, 0.0, 1.0);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        return vec4<f32>(f32(iter) + 1.0 - nu, 0.0, 0.0, 1.0);
    }
}

// ---------------- coloring pass (samples the iteration texture) ----------------
struct ColorU {
    stop_count: u32,
    cycle: f32,
    offset: f32,
    ss: u32, // supersampling factor (iteration texture is screen × ss)
    stops: array<vec4<f32>, 8>, // rgb + position
};
@group(0) @binding(0) var<uniform> cu: ColorU;
@group(0) @binding(1) var iter_tex: texture_2d<f32>;

fn palette(t_in: f32) -> vec3<f32> {
    let t = fract(t_in);
    var col = cu.stops[0].xyz;
    for (var i: u32 = 0u; i + 1u < cu.stop_count; i = i + 1u) {
        let a = cu.stops[i];
        let b = cu.stops[i + 1u];
        if (t >= a.w && t <= b.w) {
            let f = (t - a.w) / max(b.w - a.w, 1e-6);
            col = mix(a.xyz, b.xyz, f);
            break;
        }
    }
    return col;
}

fn shade(v: f32) -> vec3<f32> {
    if (v < 0.0) {
        return vec3<f32>(0.02, 0.02, 0.03); // interior
    }
    return palette(v * cu.cycle + cu.offset);
}

@fragment
fn fs_color(in: VsOut) -> @location(0) vec4<f32> {
    let tex_dim = textureDimensions(iter_tex); // = screen × ss
    let ss = max(cu.ss, 1u);
    let screen_dim = tex_dim / ss;
    let uv = clamp(in.uv, vec2<f32>(0.0), vec2<f32>(1.0));
    let pix = vec2<i32>(uv * vec2<f32>(screen_dim)); // screen pixel index
    let maxc = vec2<i32>(tex_dim) - vec2<i32>(1, 1);

    // Average the ss×ss block of iteration samples covering this pixel (SSAA).
    var acc = vec3<f32>(0.0);
    var count = 0.0;
    for (var dj: u32 = 0u; dj < ss; dj = dj + 1u) {
        for (var di: u32 = 0u; di < ss; di = di + 1u) {
            let texel = clamp(
                pix * i32(ss) + vec2<i32>(i32(di), i32(dj)),
                vec2<i32>(0, 0),
                maxc,
            );
            acc = acc + shade(textureLoad(iter_tex, texel, 0).r);
            count = count + 1.0;
        }
    }
    return vec4<f32>(acc / count, 1.0);
}
