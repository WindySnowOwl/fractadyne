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
// giving ~14 decimal digits (vs ~7 for plain f32). The `two_sum` / `two_prod` /
// `quick_two_sum` primitives below are the standard error-free transforms (Dekker 1971,
// Knuth) — implemented from the math, canonical names — and assume round-to-nearest with
// a *fused* fma (true on the target GPUs).

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
    // z² in 3 df32 multiplies instead of c_mul(a,a)'s 4: re·im and im·re are the *same* product
    // (df_mul commutes bit-for-bit — IEEE mul/fma/add all commute), so the imaginary part is
    // df_add(ri, ri) with ri computed once. Bit-identical to c_mul(a, a); shaves one multiply off
    // every Mandelbrot iteration, each Multibrot power, and (via fe_sqr) the whole floatexp loop.
    let ri = df_mul(a.re, a.im);
    return cset(df_sub(df_mul(a.re, a.re), df_mul(a.im, a.im)), df_add(ri, ri));
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
// perturbation δz no longer underflows f32 (~1e-38) at extreme depth → very deep
// zoom (bounded by the reference orbit / iteration budget, not an f32 wall).
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
    // `shift = floor(log2(mag))` via frexp (bit-manip, not the SFU) instead of `log2`; scale by
    // `2^-shift` via ldexp instead of `exp2`+mul. Both are exact powers of two, so the (m,e) pair
    // is bit-identical to the old path — but this drops 2 transcendentals/call, and fe_norm runs
    // several times per iteration, so it was the dominant cost in the deep floatexp loop.
    let shift = frexp(mag).exp - 1;
    let m2 = cset(
        vec2<f32>(ldexp(m.re.x, -shift), ldexp(m.re.y, -shift)),
        vec2<f32>(ldexp(m.im.x, -shift), ldexp(m.im.y, -shift)),
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
    let lom = cset(
        vec2<f32>(ldexp(lo.m.re.x, -de), ldexp(lo.m.re.y, -de)),
        vec2<f32>(ldexp(lo.m.im.x, -de), ldexp(lo.m.im.y, -de)),
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
// floatexp → absolute df32 (inverse of fe_from_cdf): value = m · 2^e collapsed into a Cdf.
// Safe only when the value is within f32's exponent range (the df32 perturbation path uses
// this for the series-approximation seed, valid until ~1e30× where mode 0 itself gives way).
fn fe_to_cdf(a: Fe) -> Cdf {
    let s = exp2(f32(clamp(a.e, -127, 127)));
    return cset(
        vec2<f32>(a.m.re.x * s, a.m.re.y * s),
        vec2<f32>(a.m.im.x * s, a.m.im.y * s),
    );
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
    let shift = frexp(mag).exp - 1; // floor(log2(mag)) without the SFU (see fe_norm)
    return sf_make(vec2<f32>(ldexp(m.x, -shift), ldexp(m.y, -shift)), e + shift);
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
    return sf_norm(df_add(hi.m, vec2<f32>(ldexp(lo.m.x, -de), ldexp(lo.m.y, -de))), hi.e);
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
// ---------------- extended-range orbit samples --------------------------------------
// The CPU stores a reference sample whose |Z| would degrade or flush in the plain df32
// lanes (below ~1e-36: f32 min normal is ~1.2e-38 and GPU arithmetic flushes subnormals)
// as [NaN marker, exponent, m_re, m_im], mantissas scaled to put the leading one in
// [1,2) — see `pack_sample` in fractadyne-core. A deep minibrot-family reference passes
// through such dips periodically; reading them as zero drops the 2Z·δz recurrence term
// at exactly those iterations, and past ~(dip magnitude ÷ per-period growth) zoom depth
// that annihilates every pixel's accumulated δz each period — the whole frame renders
// interior (validation corpus 14/15 at ≳1e142×). NaN never occurs in a normal sample,
// so the marker is unambiguous (NaN != NaN).
// Finite marker [0.0, k, m_re+4.0, m_im]: a legit df32 sample can never have hi == 0.0
// with |lo| ≥ 2 (hi == 0 forces lo == 0). Deliberately NOT NaN-based — WGSL gives no NaN
// guarantees and compilers may fold `x != x` to false (which silently disabled the first
// version of this marker on the GPU while the identical CPU code worked).
fn orbit_is_ext(r: vec4<f32>) -> bool { return r.x == 0.0 && abs(r.z) >= 2.0; }
// Full extended-range decode → Fe. Normal samples go through fe_from_cdf unchanged.
fn orbit_fe(r: vec4<f32>) -> Fe {
    if (orbit_is_ext(r)) {
        return fe_make(cset(vec2<f32>(r.z - 4.0, 0.0), vec2<f32>(r.w, 0.0)), i32(r.y));
    }
    return fe_from_cdf(cset(vec2<f32>(r.x, r.z), vec2<f32>(r.y, r.w)));
}
// Plain-df32 view for paths that cannot carry the extended range (mode 0, aux probes,
// derivative factors): an extended dip reads as (0,0) — the pre-encoding behavior at
// those magnitudes, which is adequate wherever plain df32 was adequate before.
fn orbit_cdf(r: vec4<f32>) -> Cdf {
    if (orbit_is_ext(r)) { return cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0)); }
    return cset(vec2<f32>(r.x, r.z), vec2<f32>(r.y, r.w));
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
// |a| as a scalar floatexp (extended range) — for BLA radius comparison at any depth.
fn fe_abs_sf(a: Fe) -> Sf {
    let mag = sqrt(a.m.re.x * a.m.re.x + a.m.im.x * a.m.im.x);
    return sf_norm(vec2<f32>(mag, 0.0), a.e);
}
// Scalar floatexp comparison a < b (via the sign of a−b; both used as magnitudes here).
fn sf_lt(a: Sf, b: Sf) -> bool {
    return sf_add(a, sf_neg(b)).m.x < 0.0;
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
    // Escape-time formula id. These MUST match Rust: `core::formula::*` and
    // `FractalKind::formula_id` (fractadyne-app/src/fractal.rs). Legend:
    //   0 Mandelbrot   1 Multibrot3   2 Multibrot4   3 Multibrot5   4 Tricorn
    //   5 BurningShip  6 Celtic       7 Buffalo      8 Phoenix      9 Newton
    // Adding a formula: add a branch below in every mode it supports (direct / df32
    // perturbation / floatexp), keyed on this id — see the checklist in fractal.rs.
    formula: u32,
    julia: u32,            // 0 = Mandelbrot mode (z0=0, c=pixel), 1 = Julia (z0=pixel, c=const)
    delta_exp: i32,        // shared base-2 exponent of the δ mantissas (step / ref_offset)
    color_method: u32,     // selected coloring method (drives whether aux is accumulated)
    stripe_freq: f32,      // stripe-average angular frequency
    trap_type: u32,        // 0 = point, 1 = cross, 2 = unit circle
    aux_on: u32,           // 1 = accumulate orbit statistics into the aux target
    sa_skip: u32,          // series-approximation skip (0 = none): seed δz at this iteration
    glitch_on: u32,        // 1 = flag Pauldelbrot-glitched pixels (multi-reference correction)
    sa_a: vec4<f32>,       // order-3 series coeffs (complex df32 mantissa): δz ≈ A·δc + B·δc² + C·δc³
    sa_b: vec4<f32>,
    sa_c: vec4<f32>,
    sa_a_exp: i32,         // per-coefficient base-2 exponents (floatexp)
    sa_b_exp: i32,
    sa_c_exp: i32,
    bla_on: u32,           // 1 = use the BLA tree (appended after the orbit at index orbit_len)
};
@group(0) @binding(0) var<uniform> iu: IterU;
// Reference orbit as double-single: each Z_n = (re.hi, im.hi, re.lo, im.lo). When BLA is on,
// the BLA tree is appended right after (starting at index orbit_len): 4 vec4 per node —
// [A mantissa], [B mantissa], [a_exp, b_exp, r_exp, r_mant], [span, -, -, -].
@group(0) @binding(1) var<storage, read> reference: array<vec4<f32>>;
// Event counters (diagnostics D3.3): per-fragment tallies are kept in registers and
// committed as one atomicAdd per nonzero slot at fragment end - a few atomics per pixel,
// negligible next to the iteration loop, and pixel output is untouched. Execution proof
// for the deep-zoom code paths: a "fix" whose counter stays zero did not run (the WGSL
// NaN-marker lesson). Slot order matches COUNTER_SLOTS / CTR_* in lib.rs.
@group(0) @binding(2) var<storage, read_write> counters: array<atomic<u32>>;
const CTR_REBASE: u32 = 0u;      // Zhuoran rebases taken
const CTR_EXT_SAMPLE: u32 = 1u;  // extended-range orbit samples decoded (recurrence)
const CTR_GLITCH: u32 = 2u;      // Pauldelbrot glitch flags raised (glitch_on = 1)
const CTR_BLA_SKIP: u32 = 3u;    // BLA multi-step skips taken
const CTR_MAXITER: u32 = 4u;     // fragments that exhausted max_iter
const CTR_ESC_MIN: u32 = 5u;     // min escaped smooth-iter (f32 bits; positive floats sort as u32)
const CTR_ESC_MAX: u32 = 6u;     // max escaped smooth-iter (f32 bits)

fn ctr_commit(n_rebase: u32, n_ext: u32, n_bla: u32) {
    if (n_rebase > 0u) { atomicAdd(&counters[CTR_REBASE], n_rebase); }
    if (n_ext > 0u) { atomicAdd(&counters[CTR_EXT_SAMPLE], n_ext); }
    if (n_bla > 0u) { atomicAdd(&counters[CTR_BLA_SKIP], n_bla); }
}

// Track the frame's escaped smooth-iteration RANGE (min/max) — positive IEEE f32s compare
// identically as unsigned ints, so plain u32 atomics work on the bit patterns. Feeds the app's
// live palette auto-normalization (a dense deep field spans huge escape ranges that alias a
// fixed palette cycle into speckle). The min slot is seeded to 0xFFFFFFFF host-side after the
// per-frame clear. Two atomics per escaped fragment — same cost class as ctr_commit.
fn esc_range_commit(sm: f32) {
    if (sm >= 0.0) {
        let b = bitcast<u32>(sm);
        atomicMin(&counters[CTR_ESC_MIN], b);
        atomicMax(&counters[CTR_ESC_MAX], b);
    }
}

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
// Fold one orbit point z into the running statistics — but only the sub-statistic the *selected*
// color method actually samples (stripe→aux.x, TIA→aux.y, trap→aux.z; decomposition uses only the
// final angle in aux_pack, so it accumulates nothing). The un-gated version computed all three every
// iteration — atan2+sin AND pow AND a trap distance — even though a frame reads exactly one; the two
// dead ones cost ~10-17× at depth (e.g. stripe at 1e30× carried a pointless per-iteration `pow`).
// Each branch preserves its own method's exact accumulation order, so per-method output is bit-identical.
fn aux_step(a: ptr<function, Aux>, zf: vec2<f32>, cmag: f32, power_f: f32) {
    let method = iu.color_method;
    if (method == 3u) {
        // Orbit trap: nearest approach to a shape (point / axes-cross / unit circle).
        var d: f32;
        if (iu.trap_type == 1u) { d = min(abs(zf.x), abs(zf.y)); }
        else if (iu.trap_type == 2u) { d = abs(length(zf) - 1.0); }
        else { d = length(zf); }
        (*a).trap = min((*a).trap, d);
    } else if (method == 1u) {
        // Stripe average: smooth orbit average of a sinusoid of the argument.
        let term = 0.5 + 0.5 * sin(iu.stripe_freq * atan2(zf.y, zf.x));
        (*a).sac_prev = (*a).sac_sum;
        (*a).sac_sum = (*a).sac_sum + term;
    } else if (method == 2u) {
        // Triangle-inequality average: where |z_{n+1}| sits between ||z_n|^p − |c|| and
        // |z_n|^p + |c|. Needs a valid previous |z|.
        let cur_abs = length(zf);
        if ((*a).n >= 1.0) {
            let m = pow(max((*a).prev_abs, 1.0e-12), power_f);
            let lower = abs(m - cmag);
            let upper = m + cmag;
            let tt = clamp((cur_abs - lower) / max(upper - lower, 1.0e-9), 0.0, 1.0);
            (*a).tia_prev = (*a).tia_sum;
            (*a).tia_sum = (*a).tia_sum + tt;
        }
        (*a).prev_abs = cur_abs;
    }
    // method == 5u (decomposition): nothing to accumulate — aux_pack reads only the final angle.
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

// Pauldelbrot glitch: a perturbed pixel is unreliable when its full value dips far below the
// reference (|z_n|² < GLITCH_TOL2·|Z_n|²) — the low-precision δz can't hold the cancellation.
// Flagged (only when iu.glitch_on) with a distinct sentinel in main.r for multi-reference
// correction: r = -1 interior, r ≥ 0 escaped, r = -2 glitched.
const GLITCH_TOL2: f32 = 1.0e-4;
const GLITCH_SENTINEL: vec4<f32> = vec4<f32>(-2.0, 0.0, 0.0, 1.0e30);

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
    // Event-counter tallies (committed once per fragment via ctr_commit).
    var n_rebase: u32 = 0u;
    var n_ext: u32 = 0u;
    var n_bla: u32 = 0u;

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
        var dprev = cset(zero, zero); // Phoenix derivative D_{n-1} (two-term)
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
                } else if (iu.formula == 8u) {
                    // Phoenix derivative (analytic): D' = 2·z·D + [1 if Mandelbrot] − 0.5·D_{n-1}.
                    var dn = c_mul(c_two(z), dz);
                    if (iu.julia == 0u) { dn = c_add(dn, one); }
                    dn = c_sub(dn, c_scale(dprev, 0.5));
                    dprev = dz;
                    dz = dn;
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
            atomicAdd(&counters[CTR_MAXITER], 1u);
            let aux_out = select(AUX_NONE, aux_pack(aux, 0.0, zf), iu.aux_on == 1u);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        if (newton) {
            // Color by convergence speed (iteration count).
            esc_range_commit(f32(iter));
            return FragOut(vec4<f32>(f32(iter), 0.0, 0.0, 1.0e30), AUX_NONE);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u || iu.formula == 8u) {
            nrm = slope_normal(zf, vec2<f32>(dz.re.x, dz.im.x));
            de = de_log2(mag2, dz.re.x * dz.re.x + dz.im.x * dz.im.x, 0.0);
        }
        let aux_out = select(AUX_NONE, aux_pack(aux, fract(smit), zf), iu.aux_on == 1u);
        esc_range_commit(smit);
        return FragOut(vec4<f32>(smit, nrm.x, nrm.y, de), aux_out);
    } else if (iu.mode == 2u) {
        // Floatexp perturbation (mode 2): δz/δc carried as floatexp (df32 mantissa +
        // i32 exponent), so the deviation never underflows f32 → extreme depth. ~1.7×
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
        var dz_prev: Fe = fe_zero(); // Phoenix: δz_{n-1} (unused by other formulas)
        // Derivative dz/dc (Mandelbrot) or dz/dz0 (Julia) in floatexp, for DE lighting.
        var D: Fe;
        if (iu.julia == 1u) { D = fe_one(); } else { D = fe_zero(); }
        var Dprev: Fe = fe_zero(); // Phoenix derivative D_{n-1} (two-term)
        let cmag = select(
            length(vec2<f32>(iu.center.x, iu.center.y)),
            length(vec2<f32>(iu.julia_c.x, iu.julia_c.y)),
            iu.julia == 1u,
        );
        let r0i = reference[0];
        var aux = aux_init(vec2<f32>(r0i.x, r0i.y));
        var ref_n: u32 = 0u;
        var power_f = 2.0;
        // BLA level layout: offsets/lengths per level, reconstructed from the reference length
        // (level 0 has one node per step; each higher level halves). Matches the CPU packing.
        var bla_off: array<u32, 32>;
        var bla_len: array<u32, 32>;
        var bla_levels = 0u;
        if (iu.bla_on == 1u && iu.formula == 0u && iu.orbit_len > 1u) {
            var blen = iu.orbit_len - 1u;
            var boff = 0u;
            loop {
                bla_off[bla_levels] = boff;
                bla_len[bla_levels] = blen;
                boff = boff + blen;
                bla_levels = bla_levels + 1u;
                if (blen <= 1u || bla_levels >= 32u) { break; }
                blen = (blen + 1u) / 2u;
            }
        }
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
            // BLA: skip 2^l reference steps at once while |δz| is within the merged validity
            // radius; revert to a lower level (ultimately a full step) on escape overshoot.
            // δz stays small in the BLA regime, so rebasing never triggers here.
            if (bla_levels > 0u) {
                let dzmag = fe_abs_sf(dz);
                var applied = false;
                var l = bla_levels;
                loop {
                    if (l == 0u) { break; }
                    l = l - 1u;
                    let stepn = 1u << l;
                    if ((ref_n & (stepn - 1u)) != 0u) { continue; } // ref_n not aligned to 2^l
                    let j = ref_n >> l;
                    if (j >= bla_len[l]) { continue; }
                    let node = iu.orbit_len + (bla_off[l] + j) * 4u;
                    let v2 = reference[node + 2u];
                    let span = u32(reference[node + 3u].x);
                    if (ref_n + span >= iu.orbit_len) { continue; } // keep reference[nref] valid
                    if (!sf_lt(dzmag, sf_norm(vec2<f32>(v2.w, 0.0), i32(v2.z)))) { continue; }
                    let v0 = reference[node];
                    let v1 = reference[node + 1u];
                    let A = fe_norm(cset(v0.xy, v0.zw), i32(v2.x));
                    let B = fe_norm(cset(v1.xy, v1.zw), i32(v2.y));
                    let ndz = fe_add(fe_mul(A, dz), fe_mul(B, dc));
                    let nref = ref_n + span;
                    // orbit_cdf: the landing sample may be an extended-range dip (NaN-marked).
                    let rn = orbit_cdf(reference[nref]);
                    let ndzf = fe_lo_f32(ndz);
                    let zx = rn.re.x + ndzf.x;
                    let zy = rn.im.x + ndzf.y;
                    if (zx * zx + zy * zy > bail2) { continue; } // overshoot → drop a level
                    if (iu.formula <= 3u) { D = fe_add(fe_mul(A, D), B); }
                    dz = ndz;
                    ref_n = nref;
                    iter = iter + span;
                    zf = vec2<f32>(zx, zy);
                    // Fold this node's precomputed aux aggregate over the `span` skipped iterates, so
                    // stripe/TIA/trap coloring stays correct across the skip instead of dropping the
                    // skipped run. `zf` is the actual landing value → prev_abs restores exactly.
                    if (iu.aux_on == 1u) {
                        let agg = reference[node + 3u]; // [span, trap_min, ΣTIA, Σstripe]
                        aux.trap = min(aux.trap, agg.y);
                        aux.tia_sum = aux.tia_sum + agg.z;
                        aux.sac_sum = aux.sac_sum + agg.w;
                        aux.n = aux.n + f32(span);
                        aux.prev_abs = length(zf);
                    }
                    n_bla = n_bla + 1u;
                    // Rebase at the BLA landing. The "δz stays small in the BLA regime ⇒ rebasing
                    // never triggers here" assumption (above) FAILS when the landing sample is a
                    // near-zero orbit dip: |Z_nref| ≈ 0 ⇒ |zfull| ≈ |δz|, so the Zhuoran condition
                    // |zfull| < |δz| can hold. Corpus 15's deep dendrite pixels must rebase exactly
                    // at these dips; without the check a valid skip lands past them and the pixel
                    // marches to the reference END, escaping prematurely there (~orbit_len) instead
                    // of deep — the dendrites vanish. Mirrors the full-step rebase below.
                    let zfe = fe_add(orbit_fe(reference[nref]), dz);
                    if (sf_lt(fe_abs_sf(zfe), fe_abs_sf(dz))) {
                        n_rebase = n_rebase + 1u;
                        dz = fe_sub(zfe, orbit_fe(reference[0]));
                        ref_n = 0u;
                    }
                    applied = true;
                    break;
                }
                if (applied) { continue; }
            }
            let r = reference[ref_n];
            let Z = orbit_cdf(r); // reference Z_n (df32; an extended dip reads (0,0) here)

            // Derivative update D ← f'(z_n)·D (+1 Mandelbrot) using full z_n = Z_n + δz_n.
            if (iu.formula <= 3u) {
                let dzc = fe_lo_f32(dz);
                let zfn = vec2<f32>(Z.re.x + dzc.x, Z.im.x + dzc.y);
                let fp = deriv_factor(iu.formula, zfn);
                D = fe_mul_c(D, fp.x, fp.y);
                if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            } else if (iu.formula == 8u) {
                // Phoenix derivative: D' = 2·z_n·D + [1 if Mandelbrot] − 0.5·D_{n-1}.
                let dzc = fe_lo_f32(dz);
                let zfn = vec2<f32>(Z.re.x + dzc.x, Z.im.x + dzc.y);
                var dn = fe_mul_c(D, 2.0 * zfn.x, 2.0 * zfn.y);
                if (iu.julia == 0u) { dn = fe_add(dn, fe_one()); }
                dn = fe_sub(dn, fe_scale(Dprev, 0.5));
                Dprev = D;
                D = dn;
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
            } else if (iu.formula == 8u) {
                // Phoenix: δz' = 2Z·δz + δz² + δc − 0.5·δz_{n-1}
                var t: Fe;
                if (orbit_is_ext(r)) {
                    n_ext = n_ext + 1u;
                    t = fe_two(fe_mul(dz, orbit_fe(r)));
                } else {
                    t = fe_two(fe_mul_cdf(dz, Z));
                }
                t = fe_add(t, fe_sqr(dz));
                t = fe_add(t, dc);
                let dz_new = fe_sub(t, fe_scale(dz_prev, 0.5));
                dz_prev = dz;
                dz = dz_new;
            } else {
                // Mandelbrot: δz' = 2Z·δz + δz² + δc. THE LOAD-BEARING dip handling: at an
                // extended-range sample (true |Z| ~ 1e-71, unrepresentable in the df32 lanes)
                // the 2Z·δz term must be computed in extended range — dropping it is what
                // re-glued every pixel to the reference each dip period and rendered whole
                // deep views interior (corpus 14/15). Normal samples keep the exact original
                // code path, so existing content is bit-identical.
                var t: Fe;
                if (orbit_is_ext(r)) {
                    n_ext = n_ext + 1u;
                    t = fe_two(fe_mul(dz, orbit_fe(r)));
                } else {
                    t = fe_two(fe_mul_cdf(dz, Z));
                }
                t = fe_add(t, fe_sqr(dz));
                dz = fe_add(t, dc);
            }

            ref_n = ref_n + 1u;
            iter = iter + 1u;

            // Full value z = Z_{n+1} + δz — carried in EXTENDED range. The old f32 shortcut
            // (zf = rn.hi + fe_lo_f32(dz); rebase if dot(zf,zf) < fe_mag2(dz)) underflows on
            // BOTH sides once |z| and |δz| drop below f32 range (fe_mag2's exp2 clamp is 0.0
            // for dz.e ≤ −125; FTZ hardware zeroes it from ~−75), so the test read 0 < 0 and
            // Zhuoran rebasing was silently DISABLED at exactly the moment it exists for
            // (|Z+δz| < |δz| at a deep near-zero orbit pass). A missed rebase forces δz to
            // carry a value orders below its own magnitude through fe_add's 60-bit window —
            // per-pixel state is destroyed and the whole view collapses to a uniform early
            // escape (validation corpus 14/15: a minibrot-family dive path breaks past
            // ~1e142× while the same reference renders fine at 1e141×, because |δz| scales
            // with |δc| and slid under the underflow floor). Compare in scalar floatexp
            // instead — the exact machinery the BLA radius test above already uses.
            let rn = reference[ref_n];
            // orbit_fe: full extended-range decode — Zn may be a NaN-marked dip sample.
            let Znfe = orbit_fe(rn);
            let zfull = fe_add(Znfe, dz);
            zf = fe_lo_f32(zfull);
            if (iu.aux_on == 1u) { aux_step(&aux, zf, cmag, power_f); }
            let z2 = dot(zf, zf);
            if (iu.glitch_on == 1u) {
                // |z| < 1e-2·|Z_n| (≡ the old z2 < 1e-4·zr2), in extended range: the f32
                // form underflowed to `0 < 0` at flushed/near-zero samples, so Pauldelbrot
                // glitches at exactly the sensitive orbit indices were never flagged.
                let zr = fe_abs_sf(Znfe);
                let ztol = sf_norm(vec2<f32>(zr.m.x * 1.0e-2, zr.m.y * 1.0e-2), zr.e);
                if (sf_lt(fe_abs_sf(zfull), ztol)) {
                    atomicAdd(&counters[CTR_GLITCH], 1u);
                    ctr_commit(n_rebase, n_ext, n_bla);
                    return FragOut(GLITCH_SENTINEL, AUX_NONE);
                }
            }
            if (z2 > bail2) { escaped = true; break; }

            if (sf_lt(fe_abs_sf(zfull), fe_abs_sf(dz)) || ref_n + 1u >= iu.orbit_len) {
                n_rebase = n_rebase + 1u;
                // Rebase onto reference index 0: δz = (Z_{n+1} + δz) − Z₀. Z₀ = 0 for
                // Mandelbrot, the reference point for Julia (subtraction required).
                dz = fe_sub(zfull, orbit_fe(reference[0]));
                // Phoenix (two-term): also rebase δz_{n-1}. After rebasing to index 0 the "previous"
                // reference is Z_{-1}=0 (the orbit's initial z_prev), so δz_{n-1} → the full previous
                // value z_{n-1} = Z_{ref_n-1} + δz_{n-1}. Uses reference[ref_n-1] before ref_n resets.
                if (iu.formula == 8u) {
                    dz_prev = fe_add(orbit_fe(reference[ref_n - 1u]), dz_prev);
                }
                ref_n = 0u;
            }
        }
        ctr_commit(n_rebase, n_ext, n_bla);
        if (!escaped) {
            atomicAdd(&counters[CTR_MAXITER], 1u);
            let aux_out = select(AUX_NONE, aux_pack(aux, 0.0, zf), iu.aux_on == 1u);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u || iu.formula == 8u) {
            nrm = slope_normal(zf, vec2<f32>(D.m.re.x, D.m.im.x));
            de = de_log2(mag2, D.m.re.x * D.m.re.x + D.m.im.x * D.m.im.x, f32(D.e));
        }
        let aux_out = select(AUX_NONE, aux_pack(aux, fract(smit), zf), iu.aux_on == 1u);
        esc_range_commit(smit);
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
        var dz_prev: Cdf = zero; // Phoenix: δz_{n-1} (unused by other formulas)
        // Derivative in floatexp (grows past f32 range at depth), for DE lighting.
        var D: Fe;
        if (iu.julia == 1u) { D = fe_one(); } else { D = fe_zero(); }
        var Dprev: Fe = fe_zero(); // Phoenix derivative D_{n-1} (two-term)
        let cmag = select(
            length(vec2<f32>(iu.center.x, iu.center.y)),
            length(vec2<f32>(iu.julia_c.x, iu.julia_c.y)),
            iu.julia == 1u,
        );
        let r0i = reference[0];
        var aux = aux_init(vec2<f32>(r0i.x, r0i.y));
        var ref_n: u32 = 0u;
        var power_f = 2.0;
        // Series approximation: seed δz + derivative D from the order-3 polynomial and start
        // at iteration `sa_skip`, skipping that many steps. Same coefficients as mode 2, but
        // evaluated in floatexp (the coeffs overflow f32) then collapsed to the absolute df32
        // δ this path carries. Mandelbrot only (the CPU gates formula 0 / no Julia / no aux).
        if (iu.sa_skip > 0u && iu.julia == 0u) {
            let A = fe_norm(cset(iu.sa_a.xy, iu.sa_a.zw), iu.sa_a_exp);
            let B = fe_norm(cset(iu.sa_b.xy, iu.sa_b.zw), iu.sa_b_exp);
            let C = fe_norm(cset(iu.sa_c.xy, iu.sa_c.zw), iu.sa_c_exp);
            let dcf = fe_from_cdf(dc);
            let dc2 = fe_sqr(dcf);
            let dc3 = fe_mul(dc2, dcf);
            dz = fe_to_cdf(fe_add(fe_add(fe_mul(A, dcf), fe_mul(B, dc2)), fe_mul(C, dc3)));
            // D = d(δz)/d(δc) = A + 2·B·δc + 3·C·δc²
            D = fe_add(fe_add(A, fe_scale(fe_mul(B, dcf), 2.0)), fe_scale(fe_mul(C, dc2), 3.0));
            iter = iu.sa_skip;
            ref_n = iu.sa_skip;
        }
        loop {
            if (iter >= iu.max_iter) { break; }
            // orbit_cdf: an extended-range dip sample (NaN-marked) reads as (0,0) here —
            // mode 0's plain df32 math cannot carry it, matching pre-encoding behavior.
            let z = orbit_cdf(reference[ref_n]);
            if (iu.formula <= 3u) {
                let zfn = vec2<f32>(z.re.x + dz.re.x, z.im.x + dz.im.x); // full z_n
                let fp = deriv_factor(iu.formula, zfn);
                D = fe_mul_c(D, fp.x, fp.y);
                if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            } else if (iu.formula == 8u) {
                // Phoenix derivative: D' = 2·z_n·D + [1 if Mandelbrot] − 0.5·D_{n-1}.
                let zfn = vec2<f32>(z.re.x + dz.re.x, z.im.x + dz.im.x); // full z_n
                var dn = fe_mul_c(D, 2.0 * zfn.x, 2.0 * zfn.y);
                if (iu.julia == 0u) { dn = fe_add(dn, fe_one()); }
                dn = fe_sub(dn, fe_scale(Dprev, 0.5));
                Dprev = D;
                D = dn;
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
            } else if (iu.formula == 8u) {
                // Phoenix: δz' = 2Z·δz + δz² + δc − 0.5·δz_{n-1}
                let base = c_add(c_add(c_two(c_mul(z, dz)), c_sqr(dz)), dc);
                let dz_new = c_sub(base, c_scale(dz_prev, 0.5));
                dz_prev = dz;
                dz = dz_new;
            } else {
                dz = c_add(c_add(c_two(c_mul(z, dz)), c_sqr(dz)), dc);
            }
            ref_n = ref_n + 1u;
            iter = iter + 1u;
            // orbit_cdf: an extended-range dip sample (NaN-marked) reads as (0,0) here.
            let rn = orbit_cdf(reference[ref_n]);
            let zr_full = df_add(rn.re, dz.re);
            let zi_full = df_add(rn.im, dz.im);
            zf = vec2<f32>(zr_full.x, zi_full.x);
            if (iu.aux_on == 1u) { aux_step(&aux, zf, cmag, power_f); }
            let z2 = dot(zf, zf);
            if (iu.glitch_on == 1u) {
                let zr2 = rn.re.x * rn.re.x + rn.im.x * rn.im.x;
                if (z2 < GLITCH_TOL2 * zr2) {
                    atomicAdd(&counters[CTR_GLITCH], 1u);
                    ctr_commit(n_rebase, n_ext, n_bla);
                    return FragOut(GLITCH_SENTINEL, AUX_NONE);
                }
            }
            if (z2 > bail2) { escaped = true; break; }
            let dzmag2 = dz.re.x * dz.re.x + dz.im.x * dz.im.x;
            if (z2 < dzmag2 || ref_n + 1u >= iu.orbit_len) {
                n_rebase = n_rebase + 1u;
                let r0 = orbit_cdf(reference[0]);
                // Phoenix (two-term): rebase δz_{n-1} to Z_{-1}=0 → the full previous value
                // z_{n-1} = Z_{ref_n-1} + δz_{n-1} (before ref_n resets to 0).
                if (iu.formula == 8u) {
                    let rp = orbit_cdf(reference[ref_n - 1u]);
                    dz_prev = cset(
                        df_add(rp.re, dz_prev.re),
                        df_add(rp.im, dz_prev.im),
                    );
                }
                dz = cset(
                    df_sub(zr_full, r0.re),
                    df_sub(zi_full, r0.im),
                );
                ref_n = 0u;
            }
        }
        ctr_commit(n_rebase, n_ext, n_bla);
        if (!escaped) {
            atomicAdd(&counters[CTR_MAXITER], 1u);
            let aux_out = select(AUX_NONE, aux_pack(aux, 0.0, zf), iu.aux_on == 1u);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u || iu.formula == 8u) {
            nrm = slope_normal(zf, vec2<f32>(D.m.re.x, D.m.im.x));
            de = de_log2(mag2, D.m.re.x * D.m.re.x + D.m.im.x * D.m.im.x, f32(D.e));
        }
        let aux_out = select(AUX_NONE, aux_pack(aux, fract(smit), zf), iu.aux_on == 1u);
        esc_range_commit(smit);
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
    reproject: u32,    // 1 = reprojection: sample the frozen texture scaled+translated (no re-iterate)
    uv_off: vec2<f32>, // uv translation for the reprojection (fraction of the screen panned)
    uv_scale: f32,     // uv scale about centre (1 = pan only; <1 = zoomed in since the frozen frame)
    vig_on: u32,       // 1 = spotlight vignette: dim everything outside a soft circle
    vig_dim: f32,      // how dark outside the spotlight (0..1)
    vig_soft: f32,     // soft-edge width (uv height fraction)
    vig_center: vec2<f32>, // spotlight centre in screen uv (0..1)
    vig_radius: f32,   // spotlight radius (uv height fraction)
    _pad_vig: f32,
    interior_col: vec4<f32>, // color for in-set (non-escaping) pixels; rgb in xyz
    stops: array<vec4<f32>, 8>, // rgb + position
    out_res: vec2<f32>, // output rect size (px); with reproject, aspect-fits a frozen (old-size) frame
    _pad_out: vec2<f32>,
};
@group(0) @binding(0) var<uniform> cu: ColorU;
@group(0) @binding(1) var iter_tex: texture_2d<f32>;
@group(0) @binding(2) var aux_tex: texture_2d<f32>;

// ---------------------------------------------------------------------------
// Seed pass (tiled settle): nearest-neighbour upscale of the previous
// iteration + aux textures into a freshly resized pair, so a tiled settle
// starts from the coarse frame it is refining instead of from black. Raw
// iteration data upscales losslessly-in-meaning (same values, coarser grid),
// so the display before the first tile lands is exactly the blocky image the
// user was already seeing. Reuses the color pass's bind group layout (the
// uniform at binding 0 is bound but unread). textureLoad, not a sampler:
// ITER_FORMAT is rgba32float, which is not filterable without an optional
// device feature.

struct SeedOut {
    @location(0) tex: vec4<f32>,
    @location(1) aux: vec4<f32>,
}

@fragment
fn fs_seed(in: VsOut) -> SeedOut {
    let dims = vec2<f32>(textureDimensions(iter_tex));
    let p = vec2<i32>(clamp(in.uv * dims, vec2<f32>(0.0), dims - vec2<f32>(1.0)));
    var out: SeedOut;
    out.tex = textureLoad(iter_tex, p, 0);
    out.aux = textureLoad(aux_tex, p, 0);
    return out;
}

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

// Average color of the frozen frame, from a coarse grid over the iteration texture. Used to
// fill the edge a pan reveals before it's rendered — a color-matched background reads far nicer
// than black. Only the (thin) revealed strip runs this, so the per-fragment cost is negligible.
fn view_average() -> vec3<f32> {
    let dim = textureDimensions(iter_tex);
    let maxc = vec2<i32>(dim) - vec2<i32>(1, 1);
    let n: u32 = 6u;
    var acc = vec3<f32>(0.0);
    for (var j: u32 = 0u; j < n; j = j + 1u) {
        for (var i: u32 = 0u; i < n; i = i + 1u) {
            let g = (vec2<f32>(f32(i), f32(j)) + 0.5) / f32(n);
            let texel = clamp(vec2<i32>(g * vec2<f32>(dim)), vec2<i32>(0, 0), maxc);
            acc = acc + shade(textureLoad(iter_tex, texel, 0), textureLoad(aux_tex, texel, 0));
        }
    }
    return acc / f32(n * n);
}

@fragment
fn fs_color(in: VsOut) -> @location(0) vec4<f32> {
    let tex_dim = textureDimensions(iter_tex); // = screen × ss
    let ss = max(cu.ss, 1u);
    let screen_dim = tex_dim / ss;
    // Pan reprojection: sample the frozen texture shifted by the panned offset, so the detailed
    // image slides with the cursor. Anything the pan drags in from outside the frozen frame
    // isn't rendered yet — fill it with the frame's average color (nicer than black) until the
    // view settles and re-iterates.
    var suv = in.uv;
    if (cu.reproject == 1u) {
        // Aspect-fit first: display the frozen (possibly old-size) iteration texture at NATIVE
        // scale, centred in the output rect, so a resize to a new aspect ratio doesn't stretch it —
        // the center stays centred and the revealed border fills with the average color below. When
        // the frozen frame matches the current size (pan / zoom reprojection) this is the identity.
        let fit = cu.out_res / max(vec2<f32>(screen_dim), vec2<f32>(1.0));
        suv = (in.uv - vec2<f32>(0.5)) * fit + vec2<f32>(0.5);
        // Then scale about the centre + translate: follows both the zoom and pan since the frozen
        // frame was rendered (uv_scale == 1 → pure pan, the original behaviour).
        suv = (suv - vec2<f32>(0.5)) * cu.uv_scale + vec2<f32>(0.5) - cu.uv_off;
        if (suv.x < 0.0 || suv.x > 1.0 || suv.y < 0.0 || suv.y > 1.0) {
            return vec4<f32>(view_average(), 1.0);
        }
    }
    let uv = clamp(suv, vec2<f32>(0.0), vec2<f32>(1.0));
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
    // Spotlight vignette: darken outside a soft circle (anchored in screen uv; aspect-corrected so
    // it stays round). Used by guided tours to draw the eye to a region.
    if (cu.vig_on == 1u) {
        let aspect = f32(screen_dim.x) / f32(max(screen_dim.y, 1u));
        let d = length((in.uv - cu.vig_center) * vec2<f32>(aspect, 1.0));
        let e = smoothstep(cu.vig_radius, cu.vig_radius + max(cu.vig_soft, 1.0e-4), d);
        col = col * (1.0 - e * cu.vig_dim);
    }
    return vec4<f32>(col, 1.0);
}
