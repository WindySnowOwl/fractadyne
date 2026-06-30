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
// |c+d| − |c| computed without catastrophic cancellation (KF "diffabs"): the
// abs-fold contribution for perturbing non-analytic families. `c` is the
// high-precision reference component, `d` the small perturbation. When c and
// c+d share a sign the result is exactly ±d (no cancellation); only a sign flip
// (near the fold) uses ±(2c+d), where a wrong branch shows up as a glitch.
fn df_diffabs(c: vec2<f32>, d: vec2<f32>) -> vec2<f32> {
    let cd = df_add(c, d);
    if (c.x >= 0.0) {
        if (cd.x >= 0.0) { return d; }
        let t = df_add(df_two(c), d);
        return vec2<f32>(-t.x, -t.y);
    }
    if (cd.x > 0.0) { return df_add(df_two(c), d); }
    return vec2<f32>(-d.x, -d.y);
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

// ---------------- scalar floatexp (one df32 mantissa + i32 exponent) ---------------
// value = m · 2^e. The complex `Fe` shares one exponent across its re+im parts, but
// `diffabs` (the abs fold for non-analytic families) is inherently per-component, so
// the abs families fold each z² component as a *scalar* floatexp — keeping it in
// extended range — before recombining into an `Fe`.
struct Sf {
    m: vec2<f32>,
    e: i32,
};
fn sf_make(m: vec2<f32>, e: i32) -> Sf {
    var s: Sf;
    s.m = m;
    s.e = e;
    return s;
}
fn sf_norm(m: vec2<f32>, e: i32) -> Sf {
    let mag = abs(m.x);
    if (mag == 0.0) { return sf_make(m, FE_ZERO_E); }
    let shift = i32(floor(log2(mag)));
    let s = exp2(f32(-shift));
    return sf_make(vec2<f32>(m.x * s, m.y * s), e + shift);
}
fn sf_re(f: Fe) -> Sf { return sf_make(f.m.re, f.e); }
fn sf_im(f: Fe) -> Sf { return sf_make(f.m.im, f.e); }
fn sf_from_df(d: vec2<f32>) -> Sf { return sf_norm(d, 0); }
fn sf_add(a: Sf, b: Sf) -> Sf {
    var hi = a;
    var lo = b;
    if (b.e > a.e) { hi = b; lo = a; }
    let de = hi.e - lo.e; // >= 0
    if (de > 60) { return hi; } // lo below hi's df32 precision (~48 bits)
    let s = exp2(f32(-de));
    return sf_norm(df_add(hi.m, vec2<f32>(lo.m.x * s, lo.m.y * s)), hi.e);
}
fn sf_neg(a: Sf) -> Sf { return sf_make(vec2<f32>(-a.m.x, -a.m.y), a.e); }
fn sf_two(a: Sf) -> Sf { return sf_make(a.m, a.e + 1); }
// |c+d| − |c| in scalar floatexp (KF "diffabs"), branch-wise to avoid catastrophic
// cancellation — the extended-range twin of `df_diffabs`.
fn sf_diffabs(c: Sf, d: Sf) -> Sf {
    let cd = sf_add(c, d);
    if (c.m.x >= 0.0) {
        if (cd.m.x >= 0.0) { return d; }
        return sf_neg(sf_add(sf_two(c), d));
    }
    if (cd.m.x > 0.0) { return sf_add(sf_two(c), d); }
    return sf_neg(d);
}
// Recombine two scalar-floatexp components into a complex `Fe` (shared exponent).
fn fe_from_sf(re: Sf, im: Sf) -> Fe {
    let e = max(re.e, im.e);
    let sr = exp2(f32(clamp(re.e - e, -127, 0)));
    let si = exp2(f32(clamp(im.e - e, -127, 0)));
    let m = cset(
        vec2<f32>(re.m.x * sr, re.m.y * sr),
        vec2<f32>(im.m.x * si, im.m.y * si),
    );
    return fe_norm(m, e);
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

// Slope normal for distance-estimate lighting: direction of u = z / (dz/dc).
// Only the *direction* matters (the magnitude is normalized away), so any positive
// real scale on the derivative — e.g. a floatexp exponent — cancels and we can pass
// just the derivative's mantissa. Returns (0,0) when the derivative is unavailable
// (→ the color pass leaves the pixel unlit).
fn slope_normal(z: vec2<f32>, d: vec2<f32>) -> vec2<f32> {
    // u = z · conj(d)  (same direction as z/d)
    let ur = z.x * d.x + z.y * d.y;
    let ui = z.y * d.x - z.x * d.y;
    let m = sqrt(ur * ur + ui * ui);
    if (m > 1.0e-30) {
        return vec2<f32>(ur / m, ui / m);
    }
    return vec2<f32>(0.0, 0.0);
}

// floatexp constant 1, and floatexp × (plain f32 complex) — for the perturbation
// derivative dz/dc, which grows past f32 range at depth (so it lives in floatexp).
fn fe_one() -> Fe {
    return fe_make(cset(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 0.0)), 0);
}
fn fe_mul_c(a: Fe, zr: f32, zi: f32) -> Fe {
    let re = df_sub(df_mul_f32(a.m.re, zr), df_mul_f32(a.m.im, zi));
    let im = df_add(df_mul_f32(a.m.re, zi), df_mul_f32(a.m.im, zr));
    return fe_norm(cset(re, im), a.e);
}
// log2 of the distance estimate in *pixels*: de = |z|·ln|z| / |dz/dc|, then / step.
//   |D| = sqrt(dmag2)·2^dexp ;  pixel step = |iu.step.x|·2^iu.delta_exp.
// Returned as a log so the color pass can build animated distance contours cheaply.
fn de_log2(z2: f32, dmag2: f32, dexp: f32) -> f32 {
    let zm = sqrt(max(z2, 1.0));
    let num = zm * log(zm); // |z|·ln|z|
    let dlog2 = 0.5 * log2(max(dmag2, 1.0e-30)) + dexp;
    let steplog = log2(max(abs(iu.step.x), 1.0e-30)) + f32(iu.delta_exp);
    return log2(max(num, 1.0e-30)) - dlog2 - steplog;
}

// Derivative factor f'(z) = p·z^(p-1) for the holomorphic families (0..3), plain f32.
fn deriv_factor(formula: u32, z: vec2<f32>) -> vec2<f32> {
    if (formula == 0u) { return 2.0 * z; }
    let z2 = vec2<f32>(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y);
    if (formula == 1u) { return 3.0 * z2; }
    let z3 = vec2<f32>(z2.x * z.x - z2.y * z.y, z2.x * z.y + z2.y * z.x);
    if (formula == 2u) { return 4.0 * z3; }
    let z4 = vec2<f32>(z2.x * z2.x - z2.y * z2.y, 2.0 * z2.x * z2.y);
    return 5.0 * z4; // formula 3
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
    color_method: u32,     // selected coloring method (drives whether aux is accumulated)
    stripe_freq: f32,      // stripe-average angular frequency
    trap_type: u32,        // 0 = point, 1 = cross, 2 = unit circle
    aux_on: u32,           // 1 = accumulate orbit statistics into the aux target
    sa_skip: u32,          // series-approximation skip (0 = none): seed δz at this iteration
    _pad0: u32,            // pad sa_a to 16-byte alignment
    sa_a: vec4<f32>,       // order-3 series coeffs (complex df32 mantissa): δz ≈ A·δc + B·δc² + C·δc³
    sa_b: vec4<f32>,
    sa_c: vec4<f32>,
    sa_a_exp: i32,         // per-coefficient base-2 exponents (floatexp)
    sa_b_exp: i32,
    sa_c_exp: i32,
    _pad1: u32,
};
@group(0) @binding(0) var<uniform> iu: IterU;
// Reference orbit as double-single: each Z_n = (re.hi, im.hi, re.lo, im.lo).
@group(0) @binding(1) var<storage, read> reference: array<vec4<f32>>;

// Two render targets: `main` = (smooth iter, normal.x, normal.y, DE log2);
// `aux` = (stripe average, triangle-inequality average, orbit-trap distance,
// decomposition angle) — orbit statistics for the extra coloring methods.
struct FragOut {
    @location(0) main: vec4<f32>,
    @location(1) aux: vec4<f32>,
};

// Running orbit statistics, accumulated per iteration when `aux_on`.
struct Aux {
    sac_sum: f32,  // Σ stripe terms
    sac_prev: f32, // Σ stripe terms excluding the last (for smooth blending)
    tia_sum: f32,  // Σ triangle-inequality terms
    tia_prev: f32,
    n: f32,        // term count
    prev_abs: f32, // |z| of the previous iteration (TIA needs it)
    trap: f32,     // min orbit-trap distance so far
};
fn aux_init(z0: vec2<f32>) -> Aux {
    var a: Aux;
    a.sac_sum = 0.0; a.sac_prev = 0.0;
    a.tia_sum = 0.0; a.tia_prev = 0.0;
    a.n = 0.0; a.prev_abs = length(z0); a.trap = 1.0e30;
    return a;
}
// Fold one orbit point z into the running statistics.
fn aux_step(a: ptr<function, Aux>, zf: vec2<f32>, cmag: f32, power_f: f32) {
    let cur_abs = length(zf);
    // Orbit trap: nearest approach to a shape (point / axes-cross / unit circle).
    var d: f32;
    if (iu.trap_type == 1u) { d = min(abs(zf.x), abs(zf.y)); }
    else if (iu.trap_type == 2u) { d = abs(cur_abs - 1.0); }
    else { d = cur_abs; }
    (*a).trap = min((*a).trap, d);
    // Stripe average: smooth orbit average of a sinusoid of the argument.
    let term = 0.5 + 0.5 * sin(iu.stripe_freq * atan2(zf.y, zf.x));
    (*a).sac_prev = (*a).sac_sum;
    (*a).sac_sum = (*a).sac_sum + term;
    // Triangle-inequality average: where |z_{n+1}| sits between ||z_n|^p − |c|| and
    // |z_n|^p + |c|. Needs a valid previous |z|.
    if ((*a).n >= 1.0) {
        let m = pow(max((*a).prev_abs, 1.0e-12), power_f);
        let lower = abs(m - cmag);
        let upper = m + cmag;
        let tt = clamp((cur_abs - lower) / max(upper - lower, 1.0e-9), 0.0, 1.0);
        (*a).tia_prev = (*a).tia_sum;
        (*a).tia_sum = (*a).tia_sum + tt;
    }
    (*a).prev_abs = cur_abs;
    (*a).n = (*a).n + 1.0;
}
// Pack the accumulated statistics into the aux target. `frac` is the fractional part
// of the smooth iteration count (blends the last two averages for a continuous result).
fn aux_pack(a: Aux, frac: f32, zf: vec2<f32>) -> vec4<f32> {
    let sac_avg = a.sac_sum / max(a.n, 1.0);
    let sac_prev = a.sac_prev / max(a.n - 1.0, 1.0);
    let stripe = mix(sac_prev, sac_avg, frac);
    let tn = max(a.n - 1.0, 1.0);
    let tia_avg = a.tia_sum / tn;
    let tia_prev = a.tia_prev / max(a.n - 2.0, 1.0);
    let tia = mix(tia_prev, tia_avg, frac);
    let decomp = atan2(zf.y, zf.x) * 0.15915494 + 0.5; // angle → 0..1
    return vec4<f32>(stripe, tia, a.trap, decomp);
}
const AUX_NONE: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0e30, 0.0);

@fragment
fn fs_iterate(in: VsOut) -> FragOut {
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
        // Derivative dz/dc (Mandelbrot mode) or dz/dz0 (Julia mode), for DE lighting.
        // Tracked for the holomorphic families (0..3); others stay 0 → unlit.
        let one = cset(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 0.0));
        var dz: Cdf;
        if (iu.julia == 1u) { dz = one; } else { dz = cset(zero, zero); }
        let cmag = length(vec2<f32>(c.re.x, c.im.x));
        var aux = aux_init(vec2<f32>(z.re.x, z.im.x));
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
                // Derivative update D ← f'(z)·D (+1 in Mandelbrot mode), using current z.
                if (iu.formula <= 3u) {
                    var fp: Cdf;
                    if (iu.formula == 0u) {
                        fp = c_two(z);
                    } else if (iu.formula == 1u) {
                        fp = c_scale(c_sqr(z), 3.0);
                    } else if (iu.formula == 2u) {
                        fp = c_scale(c_mul(c_sqr(z), z), 4.0);
                    } else {
                        fp = c_scale(c_sqr(c_sqr(z)), 5.0);
                    }
                    dz = c_mul(fp, dz);
                    if (iu.julia == 0u) { dz = c_add(dz, one); }
                }
                zn = c_add(zn, c);
                zprev = z;
                z = zn;
                iter = iter + 1u;
                zf = vec2<f32>(z.re.x, z.im.x);
                if (iu.aux_on == 1u) { aux_step(&aux, zf, cmag, power_f); }
                if (dot(zf, zf) > bail2) { escaped = true; break; }
            }
        }
        if (!escaped) {
            let aux_out = select(AUX_NONE, aux_pack(aux, 0.0, zf), iu.aux_on == 1u);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        if (newton) {
            // Color by convergence speed (iteration count).
            return FragOut(vec4<f32>(f32(iter), 0.0, 0.0, 1.0e30), AUX_NONE);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u) {
            nrm = slope_normal(zf, vec2<f32>(dz.re.x, dz.im.x));
            de = de_log2(mag2, dz.re.x * dz.re.x + dz.im.x * dz.im.x, 0.0);
        }
        let aux_out = select(AUX_NONE, aux_pack(aux, fract(smit), zf), iu.aux_on == 1u);
        return FragOut(vec4<f32>(smit, nrm.x, nrm.y, de), aux_out);
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
        // Derivative dz/dc (Mandelbrot) or dz/dz0 (Julia) in floatexp, for DE lighting.
        var D: Fe;
        if (iu.julia == 1u) { D = fe_one(); } else { D = fe_zero(); }
        let cmag = select(
            length(vec2<f32>(iu.center.x, iu.center.y)),
            length(vec2<f32>(iu.julia_c.x, iu.julia_c.y)),
            iu.julia == 1u,
        );
        let r0i = reference[0];
        var aux = aux_init(vec2<f32>(r0i.x, r0i.y));
        var ref_n: u32 = 0u;
        var power_f = 2.0;
        // Series approximation: seed δz (and the derivative D) from the order-3 polynomial
        // δz ≈ A·δc + B·δc² + C·δc³ and start at iteration `sa_skip`, skipping that many
        // perturbation steps. Mandelbrot only (the CPU gates this: formula 0, no Julia, no
        // aux-accumulating coloring method).
        if (iu.sa_skip > 0u && iu.julia == 0u) {
            let A = fe_norm(cset(iu.sa_a.xy, iu.sa_a.zw), iu.sa_a_exp);
            let B = fe_norm(cset(iu.sa_b.xy, iu.sa_b.zw), iu.sa_b_exp);
            let C = fe_norm(cset(iu.sa_c.xy, iu.sa_c.zw), iu.sa_c_exp);
            let dc2 = fe_sqr(dc);
            let dc3 = fe_mul(dc2, dc);
            dz = fe_add(fe_add(fe_mul(A, dc), fe_mul(B, dc2)), fe_mul(C, dc3));
            // D = d(δz)/d(δc) = A + 2·B·δc + 3·C·δc²
            D = fe_add(fe_add(A, fe_scale(fe_mul(B, dc), 2.0)), fe_scale(fe_mul(C, dc2), 3.0));
            iter = iu.sa_skip;
            ref_n = iu.sa_skip;
        }
        loop {
            if (iter >= iu.max_iter) { break; }
            let r = reference[ref_n];
            let Z = cset(vec2<f32>(r.x, r.z), vec2<f32>(r.y, r.w)); // reference Z_n (df32)

            // Derivative update D ← f'(z_n)·D (+1 Mandelbrot) using full z_n = Z_n + δz_n.
            if (iu.formula <= 3u) {
                let dzc = fe_lo_f32(dz);
                let zfn = vec2<f32>(r.x + dzc.x, r.y + dzc.y);
                let fp = deriv_factor(iu.formula, zfn);
                D = fe_mul_c(D, fp.x, fp.y);
                if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            }

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
            } else if (iu.formula >= 5u && iu.formula <= 7u) {
                // Abs families (floatexp). δ(z²) = 2Zδz + δz²; the abs fold on a z²
                // component is a per-component scalar diffabs against the reference z²
                // = c_sqr(Z). Recombine the folded components into the new δz.
                let dw = fe_add(fe_two(fe_mul_cdf(dz, Z)), fe_sqr(dz));
                let W = c_sqr(Z);
                let Wre = sf_from_df(W.re);
                let Wim = sf_from_df(W.im);
                if (iu.formula == 5u) {
                    // Burning Ship: real = Re(z²)+cx, imag = |Im(z²)|+cy
                    dz = fe_from_sf(
                        sf_add(sf_re(dw), sf_re(dc)),
                        sf_add(sf_diffabs(Wim, sf_im(dw)), sf_im(dc)),
                    );
                } else if (iu.formula == 6u) {
                    // Celtic: real = |Re(z²)|+cx, imag = Im(z²)+cy
                    dz = fe_from_sf(
                        sf_add(sf_diffabs(Wre, sf_re(dw)), sf_re(dc)),
                        sf_add(sf_im(dw), sf_im(dc)),
                    );
                } else {
                    // Buffalo: real = |Re(z²)|+cx, imag = |Im(z²)|+cy
                    dz = fe_from_sf(
                        sf_add(sf_diffabs(Wre, sf_re(dw)), sf_re(dc)),
                        sf_add(sf_diffabs(Wim, sf_im(dw)), sf_im(dc)),
                    );
                }
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
            if (iu.aux_on == 1u) { aux_step(&aux, zf, cmag, power_f); }
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
            let aux_out = select(AUX_NONE, aux_pack(aux, 0.0, zf), iu.aux_on == 1u);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u) {
            nrm = slope_normal(zf, vec2<f32>(D.m.re.x, D.m.im.x));
            de = de_log2(mag2, D.m.re.x * D.m.re.x + D.m.im.x * D.m.im.x, f32(D.e));
        }
        let aux_out = select(AUX_NONE, aux_pack(aux, fract(smit), zf), iu.aux_on == 1u);
        return FragOut(vec4<f32>(smit, nrm.x, nrm.y, de), aux_out);
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
        // Derivative in floatexp (grows past f32 range at depth), for DE lighting.
        var D: Fe;
        if (iu.julia == 1u) { D = fe_one(); } else { D = fe_zero(); }
        let cmag = select(
            length(vec2<f32>(iu.center.x, iu.center.y)),
            length(vec2<f32>(iu.julia_c.x, iu.julia_c.y)),
            iu.julia == 1u,
        );
        let r0i = reference[0];
        var aux = aux_init(vec2<f32>(r0i.x, r0i.y));
        var ref_n: u32 = 0u;
        var power_f = 2.0;
        loop {
            if (iter >= iu.max_iter) { break; }
            let r = reference[ref_n];
            let z = cset(vec2<f32>(r.x, r.z), vec2<f32>(r.y, r.w));
            if (iu.formula <= 3u) {
                let zfn = vec2<f32>(r.x + dz.re.x, r.y + dz.im.x); // full z_n
                let fp = deriv_factor(iu.formula, zfn);
                D = fe_mul_c(D, fp.x, fp.y);
                if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            }
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
            } else if (iu.formula >= 5u && iu.formula <= 7u) {
                // Abs families. δ(z²) = 2Z·δz + δz²; the abs fold on a component
                // becomes diffabs(reference z² component, its δ). Reference z² = c_sqr(Z).
                let W = c_sqr(z);
                let dw = c_add(c_two(c_mul(z, dz)), c_sqr(dz));
                if (iu.formula == 5u) {
                    // Burning Ship: real = x²−y²+cx, imag = |2xy|+cy
                    dz = cset(df_add(dw.re, dc.re),
                              df_add(df_diffabs(W.im, dw.im), dc.im));
                } else if (iu.formula == 6u) {
                    // Celtic: real = |x²−y²|+cx, imag = 2xy+cy
                    dz = cset(df_add(df_diffabs(W.re, dw.re), dc.re),
                              df_add(dw.im, dc.im));
                } else {
                    // Buffalo: real = |x²−y²|+cx, imag = |2xy|+cy
                    dz = cset(df_add(df_diffabs(W.re, dw.re), dc.re),
                              df_add(df_diffabs(W.im, dw.im), dc.im));
                }
            } else {
                dz = c_add(c_add(c_two(c_mul(z, dz)), c_sqr(dz)), dc);
            }
            ref_n = ref_n + 1u;
            iter = iter + 1u;
            let rn = reference[ref_n];
            let zr_full = df_add(vec2<f32>(rn.x, rn.z), dz.re);
            let zi_full = df_add(vec2<f32>(rn.y, rn.w), dz.im);
            zf = vec2<f32>(zr_full.x, zi_full.x);
            if (iu.aux_on == 1u) { aux_step(&aux, zf, cmag, power_f); }
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
            let aux_out = select(AUX_NONE, aux_pack(aux, 0.0, zf), iu.aux_on == 1u);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u) {
            nrm = slope_normal(zf, vec2<f32>(D.m.re.x, D.m.im.x));
            de = de_log2(mag2, D.m.re.x * D.m.re.x + D.m.im.x * D.m.im.x, f32(D.e));
        }
        let aux_out = select(AUX_NONE, aux_pack(aux, fract(smit), zf), iu.aux_on == 1u);
        return FragOut(vec4<f32>(smit, nrm.x, nrm.y, de), aux_out);
    }
}

// ---------------- coloring pass (samples the iteration texture) ----------------
struct ColorU {
    stop_count: u32,
    cycle: f32,
    offset: f32,
    ss: u32, // supersampling factor (iteration texture is screen × ss)
    light: u32,        // 0 = off, 1 = slope/relief lighting
    light_angle: f32,  // radians
    light_height: f32, // relief strength
    de_on: u32,        // 0 = off, 1 = distance-estimate glow
    de_strength: f32,  // glow blend amount (0..1)
    de_width: f32,     // distance-contour spacing (octaves per band)
    de_phase: f32,     // animated phase (cycles the glow bands)
    color_method: u32, // 0 smooth, 1 stripe, 2 triangle-ineq, 3 orbit-trap, 4 distance, 5 decomposition
    aa_filter: u32,    // color-pass box-filter taps per axis (≥1); >1 anti-aliases an upscaled iter texture
    _capad0: u32,
    _capad1: u32,
    _capad2: u32,
    interior_col: vec4<f32>, // color for in-set (non-escaping) pixels; rgb in xyz
    stops: array<vec4<f32>, 8>, // rgb + position
};
@group(0) @binding(0) var<uniform> cu: ColorU;
@group(0) @binding(1) var iter_tex: texture_2d<f32>;
@group(0) @binding(2) var aux_tex: texture_2d<f32>;

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

// Map one texel (main + aux statistics) to a color, per the selected method.
// `m` = (smooth iter, normal.x, normal.y, DE log2); `a` = (stripe, TIA, trap, decomp).
fn shade(m: vec4<f32>, a: vec4<f32>) -> vec3<f32> {
    var pv: f32;
    var interior: bool = (m.r < 0.0);
    if (cu.color_method == 1u) {
        pv = a.r;                                  // stripe average (0..1)
    } else if (cu.color_method == 2u) {
        pv = a.g;                                  // triangle-inequality average (0..1)
    } else if (cu.color_method == 3u) {
        interior = false;                          // orbit trap colors interior too
        pv = -log2(max(a.b, 1.0e-20));             // nearer approach → larger value
    } else if (cu.color_method == 4u) {
        interior = (m.a > 1.0e20);                 // DE unavailable → interior
        pv = m.a;                                  // distance estimate (log2 pixels)
    } else if (cu.color_method == 5u) {
        pv = a.a;                                  // decomposition angle (0..1)
    } else {
        pv = m.r;                                  // smooth iteration count
    }
    if (interior) { return cu.interior_col.xyz; }
    return palette(pv * cu.cycle + cu.offset);
}

@fragment
fn fs_color(in: VsOut) -> @location(0) vec4<f32> {
    let tex_dim = textureDimensions(iter_tex); // = screen × ss
    let ss = max(cu.ss, 1u);
    let screen_dim = tex_dim / ss;
    let uv = clamp(in.uv, vec2<f32>(0.0), vec2<f32>(1.0));
    let pix = vec2<i32>(uv * vec2<f32>(screen_dim)); // screen pixel index
    let maxc = vec2<i32>(tex_dim) - vec2<i32>(1, 1);

    // Average a `taps×taps` block of iteration samples covering this pixel. With
    // supersampling, taps = ss (true SSAA). When the iteration texture was rendered
    // below display resolution (work-budget at deep zoom on a big window), `aa_filter`
    // widens the box so the upscaled, finely-banded texture is anti-aliased instead of
    // point-sampled into speckle. Also accumulates the slope normal (relief lighting).
    let taps = max(ss, max(cu.aa_filter, 1u));
    var acc = vec3<f32>(0.0);
    var nacc = vec2<f32>(0.0);
    var dacc = 0.0;
    var count = 0.0;
    for (var dj: u32 = 0u; dj < taps; dj = dj + 1u) {
        for (var di: u32 = 0u; di < taps; di = di + 1u) {
            let texel = clamp(
                pix * i32(ss) + vec2<i32>(i32(di), i32(dj)),
                vec2<i32>(0, 0),
                maxc,
            );
            let t = textureLoad(iter_tex, texel, 0);
            let ta = textureLoad(aux_tex, texel, 0);
            acc = acc + shade(t, ta);
            nacc = nacc + vec2<f32>(t.g, t.b);
            dacc = dacc + t.a;
            count = count + 1.0;
        }
    }
    var col = acc / count;
    // Distance-estimate relief lighting: light the surface whose normal is the
    // averaged slope. `light_height` raises the ambient floor (smaller = sharper).
    if (cu.light == 1u) {
        let n = nacc / count;
        if (dot(n, n) > 1.0e-12) {
            let ld = vec2<f32>(cos(cu.light_angle), sin(cu.light_angle));
            let diff = dot(normalize(n), ld);
            let lam = clamp((diff + cu.light_height) / (1.0 + cu.light_height), 0.0, 1.0);
            col = col * lam;
        }
    }
    // Distance-estimate glow: bright contour bands at log-distance intervals from the
    // boundary; they densify near the filaments (→ glow) and flow when `de_phase`
    // animates. `de` channel carries log2(distance in pixels); ≥20 means "far"/none.
    if (cu.de_on == 1u) {
        let dl = dacc / count;
        if (dl < 20.0) {
            let phase = dl / max(cu.de_width, 0.05) - cu.de_phase;
            let band = pow(0.5 + 0.5 * cos(phase * 6.2831853), 3.0);
            col = mix(col, vec3<f32>(1.0), clamp(band * cu.de_strength, 0.0, 1.0));
        }
    }
    return vec4<f32>(col, 1.0);
}
