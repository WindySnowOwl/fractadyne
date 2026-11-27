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

//
// ⭐THE ADD RESIDUALS ARE DROPPED (TODO PR-C2, 2026-11-02): `two_sum` and `quick_two_sum` return a
// zero error term unless a pipeline sets `EFT_ADD_RESIDUALS`. That is exactly what NVIDIA's shader
// compiler has always made of them (it reassociates the residual to 0 — `fs_gputest` rows 0 and 11),
// so every render, golden and corpus check was made at this precision: on the RTX 3080 dropping
// them changes no pixel and no time. AMD's compilers keep the add residuals, and computing them made
// the RX 6800 XT 1.7–2.9× slower for precision nothing used. `two_prod` keeps its fma residual (NVIDIA
// keeps it too). Only `--gputest` sets the constant, so it still tests the exact transforms; making
// them exact for real takes a uniform-derived barrier as well (rows 19–23), which on NVIDIA costs
// 1.4–3.4× the GPU time (PR-A1, not adopted).
override EFT_ADD_RESIDUALS: bool = false;

fn quick_two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = a + b;
    if (!EFT_ADD_RESIDUALS) {
        return vec2<f32>(s, 0.0);
    }
    let e = b - (s - a);
    return vec2<f32>(s, e);
}
fn two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = a + b;
    if (!EFT_ADD_RESIDUALS) {
        return vec2<f32>(s, 0.0);
    }
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    return vec2<f32>(s, e);
}
// ⭐THE PRODUCT IS KEPT OPAQUE (TODO PR-C2 probe, 2026-11-02). The `--gputest` probe found AMD's
// Vulkan/GL compiler FUSING `p` into the add that consumes it — df_mul's `p.x + e` became
// `fma(a.x, b.x, e)`, counting the product's own error twice (256/256 inputs; NVIDIA evaluates it as
// written) — and doing so in some shaders and not others. `ob` ORs in `ob_zero`, 0 at run time but
// unknowable at compile time wherever an entry point has armed it from a uniform, so `p` is no
// longer a product to fuse. Unarmed entry points fold it away and compile as before.
// The barrier is compiled in only where this device's compiler was found to fuse
// (`PRODUCT_BARRIER`, set per device by `fusion::detect_product_fusion` through the iterate
// pipelines' constants): off, `ob` is the identity and the code is what it was before the barrier.
override PRODUCT_BARRIER: bool = false;
var<private> ob_zero: u32;
fn ob(x: f32) -> f32 {
    if (!PRODUCT_BARRIER) {
        return x;
    }
    return bitcast<f32>(bitcast<u32>(x) | ob_zero);
}
// `iu.res` is the iteration grid's size: never negative, so this stores 0 — unprovably.
fn ob_arm() {
    ob_zero = select(0u, 1u, iu.res.x < 0.0);
}
fn two_prod(a: f32, b: f32) -> vec2<f32> {
    let p = ob(a * b);
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
// ⭐B + P for a REFERENCE value B (its low word intact) and a perturbation P, as a product's operand
// (GitHub #5, 2026-11-16). `df_add` rounds to f32 here (it drops its residual,
// `EFT_ADD_RESIDUALS`), so the full-size W = 2Z + δz of a perturbed step lost Z's low word before it
// multiplied δz; along a near-periodic reference that rounding repeats every cycle and the error
// adds up (a custom z²+c at a 4.9e141× minibrot: 2.5e-8 of δz per step, linearly; its escape counts
// a median 13 iterations off). While P is inside half an ulp of B's high word, which is every deep
// view's case, P goes into the LOW word: B's digits kept at the cost of one f32 add, and the pair
// still (nearly) normalized. A larger P takes the plain `df_add`, whose rounding then depends on P,
// so it walks instead of adding up. ⛔Folding a larger P too left the low word far above an ulp of
// the high one, and the chunked and single passes of a custom hybrid at 1e6× then differed on 894
// texels: the two entry points' products contract to fma differently once that term is significant.
fn df_add_lo(b: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    if (abs(p.x) <= abs(b.x) * 5.9604645e-8) { return vec2<f32>(b.x, b.y + p.x); }
    return df_add(b, p);
}
fn c_add_lo(b: Cdf, p: Cdf) -> Cdf {
    return cset(df_add_lo(b.re, p.re), df_add_lo(b.im, p.im));
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

// ---------------- HONEST df32: the direct Multibrot step past the critical point ----------
// ⭐⭐WHY. The df32 ops above run as plain f32 on the shipping stacks (`EFT_ADD_RESIDUALS` off,
// and NVIDIA's compiler folds the add EFTs anyway), which is right almost everywhere and is the
// renderer's speed. It fails at ONE place: the step just after an orbit passes near the critical
// point 0. There z^d is tiny against c, z^d + c rounds to c, and every orbit that passed within
// that radius continues as c's own (0.126 of 0 for d = 8). Measured 2026-11-06 on a Multibrot 8
// Julia set at c ≈ 0.32295 + 0.98330i: 380 of 408,240 pixels collapsed onto c's orbit, which repeats
// in f32, and drew as interior though every one escapes in 20–491 steps in double precision.
// So a pixel switches to these ops when z^d falls below 2^-16 of |c| (`honest_r2`), and STAYS on them
// while its z carries a low word: the plain ops always leave it exactly 0 and these almost never do,
// so the stored z says which arithmetic a pixel is in, and a chunked walk resumes it on the same one
// as the single pass. Emulated on the CPU (f32 steps, double after the switch, against double
// throughout): 2^-24 left 12 pixels stuck, 2^-20 none, 2^-16 none with 4% of the pixels switched.
// ⭐HOW. `hb` passes every rounding step through `ob_zero`, 0 at run time and unknowable at compile
// time in the armed entry points (`ob_arm`), so no compiler can fold the error-free transforms: the
// uniform-derived barrier that restored exact df32 on all six stacks (`--gputest` rows 19–23).
fn hb(x: f32) -> f32 {
    return bitcast<f32>(bitcast<u32>(x) | ob_zero);
}
fn two_sum_h(a: f32, b: f32) -> vec2<f32> {
    let s = hb(a + b);
    let v = hb(s - a);
    let e = hb(a - hb(s - v)) + hb(b - v);
    return vec2<f32>(s, hb(e));
}
fn quick_two_sum_h(a: f32, b: f32) -> vec2<f32> {
    let s = hb(a + b);
    return vec2<f32>(s, hb(b - hb(s - a)));
}
fn df_add_h(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let s = two_sum_h(a.x, b.x);
    let t = two_sum_h(a.y, b.y);
    let v = quick_two_sum_h(s.x, hb(s.y + t.x));
    return quick_two_sum_h(v.x, hb(t.y + v.y));
}
fn df_mul_h(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let p = hb(a.x * b.x);
    let pe = hb(fma(a.x, b.x, -p));
    let e = hb(pe + hb(hb(a.x * b.y) + hb(a.y * b.x)));
    return quick_two_sum_h(p, e);
}
fn c_sqr_h(a: Cdf) -> Cdf {
    let ri = df_mul_h(a.re, a.im);
    let rr = df_mul_h(a.re, a.re);
    let ii = df_mul_h(a.im, a.im);
    return cset(df_add_h(rr, vec2<f32>(-ii.x, -ii.y)), df_add_h(ri, ri));
}
fn c_mul_h(a: Cdf, b: Cdf) -> Cdf {
    let rr = df_mul_h(a.re, b.re);
    let ii = df_mul_h(a.im, b.im);
    return cset(df_add_h(rr, vec2<f32>(-ii.x, -ii.y)), df_add_h(df_mul_h(a.re, b.im), df_mul_h(a.im, b.re)));
}
// z^d + c in honest df32, the power chained as `fam_pow_df` chains it (d = 3..8).
fn mb_step_h(z: Cdf, d: u32, c: Cdf) -> Cdf {
    var w: Cdf;
    switch (d) {
        case 3u: { w = c_mul_h(c_sqr_h(z), z); }
        case 4u: { w = c_sqr_h(c_sqr_h(z)); }
        case 5u: { w = c_mul_h(c_sqr_h(c_sqr_h(z)), z); }
        case 6u: { w = c_sqr_h(c_mul_h(c_sqr_h(z), z)); }
        case 7u: { w = c_mul_h(c_sqr_h(c_mul_h(c_sqr_h(z), z)), z); }
        default: { w = c_sqr_h(c_sqr_h(c_sqr_h(z))); }
    }
    return cset(df_add_h(w.re, c.re), df_add_h(w.im, c.im));
}
// The Multibrot power whose direct step switches to the honest ops (Multibrot 3–5 are formulas 1–3,
// 6–8 the families 10–12) in a Julia set; 0 = never (the parameter plane, Mandelbrot, whose radius
// is 2^-8·|c|^½ and which was not seen to collapse, and every non-holomorphic or custom formula).
fn honest_power(f: u32) -> u32 {
    if ((iu.tail_on & 4u) != 0u) { return 0u; } // `HONEST_STEP` off
    // Julia sets only. There c is fixed, so every orbit that passes near 0 lands on the SAME orbit,
    // c's, and whole regions stall at once (380 of 408,240 pixels above). In the parameter plane the
    // collapse lands on the pixel's own orbit, mostly inside a component anyway: measured at
    // 0.32295 + 0.98330i ×1000, 0.28% of the frame false interior against 0.04% with the honest step,
    // for 3.5× the time (interior pixels converge to an exact repeat more slowly in df32).
    if (iu.julia != 1u) { return 0u; }
    if (f >= 1u && f <= 3u) { return f + 2u; }
    if (fam_holo(f)) { return fam_power(f); }
    return 0u;
}
// |z|² below which z^d < 2^-16·|c|: (2^-16·|c|)^(2/d).
fn honest_r2(d: u32, c: Cdf) -> f32 {
    let cm = max(length(vec2<f32>(c.re.x, c.im.x)), 1.0e-30);
    return exp2((-16.0 + log2(cm)) * 2.0 / f32(max(d, 1u)));
}
// Does this step run on the honest ops: z near the critical point, or already carrying a low word.
fn honest_step(z: Cdf, r2: f32) -> bool {
    return z.re.y != 0.0 || z.im.y != 0.0 || (z.re.x * z.re.x + z.im.x * z.im.x) < r2;
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

// ⭐⭐THE df32 TAIL PHASE of the mode-2 loops (beta.141). Floatexp exists only for a δz too small
// for df32's f32 exponent (~1e-38). Once |δz| is large enough, a full step can run in plain
// df32 — about 383 lane-instructions against floatexp's 955 per iteration (RTX 3080, TODO.md).
// The step counters (beta.137) found 84–100% of mode-2 full steps had |δz| >= 2^-100 at every deep
// corpus scene, and full steps were 33–99% of all executed steps.
// ⭐MEMORYLESS: a step runs in df32 exactly when |δz| >= 2^-50 and Z is 0 or |Z| >= 2^-50 (see
// `tail_step_ok`), whatever the previous step did, so a chunked walk cannot depend on where a frame
// split. Every conversion
// between the two forms is an exact power-of-two scaling at those magnitudes. BLA still works in
// floatexp: the δz is handed back before each attempt, so skip decisions are unchanged. Mandelbrot
// only (the scope the corpus measured), and never while glitch detection runs, so correction
// passes stay bit-identical.
// ⭐⭐BIT-IDENTICAL TO THE ALL-FLOATEXP LOOP, and it must stay so (2026-10-15). As first shipped
// (|δz| >= 2^-60, its own rebase test, any Z) it changed 10 of the 38 corpus rows, and the
// exact-arithmetic audit found 9 of 462 changed pixels WRONG (one 20.3 iterations late, corpus row
// 13), against 0 the other way: floatexp keeps its mantissa near 1 under a shared exponent, and
// plain df32 does not. Three causes, each found by bisecting on the corpus (`--tail-audit` with
// FRACTADYNE_AUDIT_PIXELS on the row-13 point):
//  - δz from 2^-60: a term of δz² or 2Z·δz falls under f32's normal range (2^-126) and flushes.
//    Differing pixels at row 13: 1,556 at 2^-60, 25 at 2^-56, 0 at 2^-52. 2^-50 keeps even the
//    low limbs of δz² (>= 2^-100) normal on hardware that computes them exactly (the RX 6800 XT).
//  - Z near the reference orbit's close approaches (above the 1e-36 dip marker, below ~2^-79):
//    2Z·δz flushes there whatever δz is. TAIL_Z_MIN sends those rare steps to floatexp (rows 35,
//    37 matched only with it) — but NOT Z = 0: guarding Z₀ = 0 too sent the step after every
//    rebase to floatexp and cost a third of the speed-up, for nothing (`tail_step_ok`).
//  - The rebase test: comparing f32 SQUARES split the |z| ~ |δz| ties the other way; it takes the
//    square roots floatexp's comparison takes now (`tail_rebase`; rows 11, 12, 34, 36 matched only
//    with it). Calling floatexp's own `fe_abs_sf`/`sf_lt` instead also matched, at half the gain.
// With all three the corpus is 38/38 against the pre-tail baselines, and the speed-up holds (RTX
// 3080, 4K: 6.6e43 167 -> 60 ms, 1.2e148 2,359 -> 1,670, 4.2e275 741 -> 297, 4.6e1105 343 -> 107;
// the same as at 2^-60). A change here is checked by `generate_corpus.py --check` and `--tail-audit`.
const TAIL_DF32_MIN: f32 = 8.881784197001252e-16; // 2^-50
const TAIL_DF32_MIN_E: i32 = -50;
const TAIL_Z_MIN: f32 = 8.881784197001252e-16; // 2^-50: the reference value, as above
// δc below 2^-120 is dropped in the tail: against |δz| >= 2^-50 it is under 2^-70 relative, below
// df32 precision, and f32 could not hold it anyway.
const TAIL_DC_MIN_E: i32 = -120;
fn tail_df32_ok() -> bool {
    return (iu.tail_on & 1u) == 1u && iu.formula == 0u && iu.glitch_on == 0u;
}
fn tail_dc(dc: Fe) -> Cdf {
    if (dc.e < TAIL_DC_MIN_E) {
        return cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0));
    }
    return fe_to_cdf(dc);
}
fn df_max_abs(z: Cdf) -> f32 {
    return max(abs(z.re.x), abs(z.im.x));
}
// Zhuoran's rebase test |z| < |δz| — the decision floatexp's own comparison makes (`fe_abs_sf` +
// `sf_lt`: f32 square roots of the hi limbs' squared sums, compared exactly). Rounding commutes
// with the power-of-two scaling between the two forms while no square underflows, so the same
// square roots on the df32 hi limbs decide alike (a |z| whose square flushes reads 0 < |δz|, as
// floatexp's tiny |z| does). As first shipped this compared the SQUARES, which split the ties where
// two square roots round to one f32 the other way: an equally valid trajectory, but not floatexp's.
fn tail_rebase(zfull: Cdf, dz: Cdf) -> bool {
    let a = vec2<f32>(zfull.re.x, zfull.im.x);
    let b = vec2<f32>(dz.re.x, dz.im.x);
    return sqrt(dot(a, a)) < sqrt(dot(b, b));
}
// Zhuoran's rebase test |z| < |δz| on the df32 (mode 0) paths: ONE function for fs_iterate and the
// resumable pass, which must decide alike, or a view's pixels would depend on whether a frame was
// split (the chunk self-tests assert bit-identical output across a pass boundary). `z2` is
// dot(zf, zf), which the caller forms for its escape test. `--gputest` grades this decision per
// backend (row 13): the rebase counter is shader arithmetic, and a compiler that fuses these
// sums differently decides a near-tie differently (U26).
//
// This is the SAME underflow that disabled Zhuoran rebasing in mode 2 (see the long note at the
// mode-2 trigger), and it was live on this path too: the test squares plain f32, so once |δz| drops
// below ~1.1e-19 the square falls under f32's 1.18e-38 normal floor and flushes to zero. With |z|
// near zero the other side flushes too, the test reads `0 < 0`, and rebasing is silently off at
// exactly the near-zero orbit passes it exists for. Mode 0 reaches 1e28×, where pixel δ is ~1e-31 —
// squared, thirty decades under the floor — so the upper part of this mode's range ran with the
// trigger dead, and the live path has no Pauldelbrot detection to catch the resulting glitch
// (single reference + rebasing, by design).
//
// The repair is deliberately scoped to the degenerate case ONLY. Both squares reading exactly zero
// is the one situation the original form cannot answer; every other combination it already gets
// right (one side flushed while the other did not still orders correctly, because the flushed side
// is genuinely the smaller magnitude). So the original comparison is kept verbatim and consulted
// first, and the fallback runs just for `0 < 0`. Rewriting the test unconditionally is what NOT to
// do here: an earlier attempt normalized by max-component, which is algebraically identical but
// multiplies by an inexact 1/s, and it drifted three determinism checks by ~99 iterations in 1.2M
// as near-ties rounded the other way.
//
// In the fallback both magnitudes are known to be under ~1.1e-19, so scaling by 2^64 cannot
// overflow, and being a power of two the scaling is EXACT — this stays a true 2-norm comparison
// rather than switching to an ∞-norm, which would shift the trigger by up to sqrt(2). Post-scale
// the floor sits near 1e-38 * 2^64, leaving about seven decades of headroom below mode 0's deepest δ.
fn rebase_df32(z2: f32, zf: vec2<f32>, dz: Cdf) -> bool {
    let dzmag2 = dz.re.x * dz.re.x + dz.im.x * dz.im.x;
    var rebase_now = z2 < dzmag2;
    if (z2 == 0.0 && dzmag2 == 0.0) {
        const LIFT: f32 = 18446744073709551616.0; // 2^64, exact
        let zl = zf * LIFT;
        let dl = vec2<f32>(dz.re.x * LIFT, dz.im.x * LIFT);
        rebase_now = dot(zl, zl) < dot(dl, dl);
    }
    return rebase_now;
}

// ---------------- THE CRITICAL ORBIT (Julia views, 2026-11-19) ----------------------------
// ⭐⭐WHY. Zhuoran's rebase folds a pixel back onto reference index 0. For Mandelbrot Z_0 = 0, so the
// new δz is the full value z, small (|z| < |δz| is what triggered it), and keeps its relative
// precision. A Julia reference starts at its own pixel, Z_0 ≠ 0, so the new δz = z − Z_0 is about
// |Z_0| — and from there every step adds ~2^-24 of it in absolute error (the df32 adds run as plain
// f32 on the shipping stacks; floatexp's mantissa is the same df32), more than the pixel spacing at
// the rebase. Neighbors merge, and the pixels one near-zero pass sends back — a half-plane, since
// |Z + δz| < |δz| is linear in δz — go smooth behind a straight edge (the user's dual-view Julia
// panel at 2^33.9, b6933: in an f32 emulation of this loop 95.8% of that half-plane was wrong).
// So a Julia view carries a second orbit of the same map, the CRITICAL orbit W_0 = 0,
// W_{k+1} = W_k² + c, at `crit_at`, and a rebase lands on whichever start leaves the smaller δz:
// W_0 (δz = z, as in Mandelbrot mode) or Z_0 (δz = z − Z_0). In the emulation the half-plane fell
// to 2.4% wrong, the single-pixel filament noise the rest of the view has (1.7%).
// A segment ends at its own last sample: an orbit-end rebase on the critical orbit is tested
// against its length, not the reference's. `crit_len` 0 = the rule before (Mandelbrot views, custom
// formulas, Phoenix — whose rebased δz_{n-1} is the full previous value either way).
fn crit_on() -> bool {
    return iu.julia == 1u && iu.crit_len > 0u;
}
// One past the last sample of the segment `ref_n` is on.
fn seg_end(ref_n: u32) -> u32 {
    if (crit_on() && ref_n >= iu.crit_at) { return iu.crit_at + iu.crit_len; }
    return iu.orbit_len;
}
// df32 paths: does the rebase of full value `z` land on the critical orbit, |z| <= |z − Z_0| (`d0`)?
// Compared as `tail_rebase` compares (square roots of the hi limbs' squared sums), so mode 2's tail
// and its floatexp steps (`rebase_crit_fe`) decide alike.
fn rebase_crit_df(z: Cdf, d0: Cdf) -> bool {
    if (!crit_on()) { return false; }
    let a = vec2<f32>(z.re.x, z.im.x);
    let b = vec2<f32>(d0.re.x, d0.im.x);
    return sqrt(dot(a, a)) <= sqrt(dot(b, b));
}
// floatexp paths: the same decision in scalar floatexp, as the rebase test itself is made there.
fn rebase_crit_fe(z: Fe, d0: Fe) -> bool {
    return crit_on() && !sf_lt(fe_abs_sf(d0), fe_abs_sf(z));
}
// Must the df32 tail keep this step from BLA? True when the level-0 node at this step has no
// validity radius, or δz is past it by more than 0.01 in log2. Both mode-2 kernels call it, and
// `--gputest` grades it per backend (row 17): `log2` is the GPU's own approximation, so a near-tie
// can go either way on different hardware (U26).
fn tail_no_skip(dzd: Cdf, r0: vec4<f32>) -> bool {
    return !(r0.w > 0.0) || log2(df_max_abs(dzd)) > log2(r0.w) + r0.z + 0.01;
}
// May this full step run in the tail? (Mirrored by both mode-2 kernels.) Z = 0 is fine: Z₀ = 0 after
// every rebase makes 2Z·δz exactly 0, and an extended dip reads 0 where 2Z·δz sits over 60 binary
// orders under δz², so floatexp's own fe_add drops it too. Only a SMALL NONZERO Z is sent to floatexp.
fn tail_step_ok(Z: Cdf, dz: Fe, dzd: Cdf, d_now: bool) -> bool {
    let zm = df_max_abs(Z);
    return tail_df32_ok() && (zm == 0.0 || zm >= TAIL_Z_MIN)
        && select(dz.e >= TAIL_DF32_MIN_E, df_max_abs(dzd) >= TAIL_DF32_MIN, d_now);
}

// 2^(1 − e) for a positive finite x of exponent e (x in [2^e, 2^(e+1))): x times it is in [2, 4).
// Exact, and a normal number for every normal x up to f32's top; 0 for inf or NaN. Built from
// the exponent bits: (255 − biased e) as a biased exponent.
fn pow2_down(x: f32) -> f32 {
    return bitcast<f32>(0x7F800000u - (bitcast<u32>(x) & 0x7F800000u));
}

// Slope normal for distance-estimate lighting: direction of u = z / (dz/dc).
// Only the *direction* matters (the magnitude is normalized away), so any positive
// real scale on the derivative — e.g. a floatexp exponent — cancels and we can pass
// just the derivative's mantissa. Returns (0,0) when the derivative is unavailable
// (→ the color pass leaves the pixel unlit).
// ⚠The DIRECT path's derivative is plain f32 and reaches 2^127 near the boundary: once |z|·|dz|
// passes 2^64, `ur² + ui²` overflows, m is inf, and the normal came out (0, 0) — an unlit pixel
// with a good derivative (a direct strip on the RX 6800 XT, 2026-10-30; `--gputest` row 18 has
// every card return it). So the sum is formed as before and only an overflowed one (inf, or NaN
// from an inf product: all exponent bits set, a bit test no fast-math rule can fold away) is
// formed again from z and the derivative each scaled by the power of two that puts its larger
// part in [2, 4), built from that part's exponent bits (`pow2_down`). Only a pixel that came out
// with no normal takes that branch: every other keeps its bits. An infinite derivative scales by
// 0 to NaN there and stays unlit, as before.
// ⛔Neither `ldexp` nor a division to scale them: DX12 and OpenGL compile `ldexp(x, -e)` as
// x·exp2(-e), and a GPU divides by multiplying with the reciprocal; 2^-127, and the reciprocal of
// a part past 2^126, are subnormals the GPU flushes to zero, which zeroed a derivative of 2^127
// (row 18 on the RTX 3080, 2026-10-30: both ways, on all three APIs). The test is on the sum,
// after it, so only a pixel with no normal pays for the branch. (A first version tested every
// pixel's operands before the sum. PLUTO's bench matrix told neither version from the old code:
// fitted as a·old + k, a was 0.99-1.02 against a run-to-run spread of 0.03-0.05, 2026-10-30.)
fn slope_normal(z: vec2<f32>, d: vec2<f32>) -> vec2<f32> {
    // u = z · conj(d)  (same direction as z/d)
    var ur = z.x * d.x + z.y * d.y;
    var ui = z.y * d.x - z.x * d.y;
    var m2 = ur * ur + ui * ui;
    if ((bitcast<u32>(m2) & 0x7F800000u) == 0x7F800000u) {
        let zs = z * pow2_down(max(abs(z.x), abs(z.y)));
        let ds = d * pow2_down(max(abs(d.x), abs(d.y)));
        ur = zs.x * ds.x + zs.y * ds.y;
        ui = zs.y * ds.x - zs.x * ds.y;
        m2 = ur * ur + ui * ui;
    }
    let m = sqrt(m2);
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
fn de_log2(z2: f32, d: vec2<f32>, dexp: f32) -> f32 {
    let zm = sqrt(max(z2, 1.0));
    let num = zm * log(zm); // |z|·ln|z|
    // log2|d|. ⭐Not from |d|² once that overflows f32 (|d| past about 1.8e19, which a slow escape's
    // derivative reaches while it is still a number): the distance came out -inf and the color pass
    // made NaN of it, on 2.5% of a Phoenix view at 1e3x and on Multibrot and Mandelbrot boundaries
    // (the coloring matrix, 2026-11-11). Below that the square, exactly as before.
    let dmag2 = d.x * d.x + d.y * d.y;
    var dl = 0.5 * log2(max(dmag2, 1.0e-30));
    if (dmag2 > 1.0e38) {
        let ax = abs(d.x);
        let ay = abs(d.y);
        let big = max(ax, ay);
        let r = min(ax, ay) / big;
        dl = log2(big) + 0.5 * log2(1.0 + r * r);
    }
    let dlog2 = dl + dexp;
    // @@STRIP_STEPLOG_BEGIN — a strip's module puts each texel's own size here (`strip.rs`).
    let steplog = log2(max(abs(iu.step.x), 1.0e-30)) + f32(iu.delta_exp);
    // @@STRIP_STEPLOG_END
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

// ---------------- the power families (ids 10..24, design/power-families.md) ----------------
// One id per (shape, power), as fractadyne_core::formula::family numbers them: shape 0 Multibrot
// (z^d), 1 Burning Ship (|x| + i|y|, then the power), 2 Tricorn (conj, then the power), 3 Celtic
// (the power, then |Re|), 4 Buffalo (the power, then |Re| and |Im|); d = 6..8 for Multibrot, 3..5
// for the folds. Every path tests `is_fam` explicitly: the older families' arms are untouched.
fn is_fam(f: u32) -> bool { return f >= 10u && f <= 24u; }
// The holomorphic ones (Multibrot d): a derivative, so a distance estimate.
fn fam_holo(f: u32) -> bool { return f >= 10u && f <= 12u; }
// The families a BLA tree is built for — the `z^d + c` ones (Mandelbrot, Multibrot 3–8) and the
// folds (Tricorn, Burning Ship, Celtic, Buffalo at every power): the host's `FormulaCaps::bla`,
// tested here too so `bla_on` alone never walks a tree for another formula.
fn bla_formula(f: u32) -> bool { return f <= 7u || is_fam(f); }
// The folds, whose tree is a real 2×2 one (`fractadyne_core::build_bla_fold`): the node's first two
// vec4s are each a matrix's four f32 mantissas, row-major, under the exponent in the third.
fn bla_fold(f: u32) -> bool { return (f >= 4u && f <= 7u) || (is_fam(f) && !fam_holo(f)); }
// The 2×2 matrix `m`·2^me applied to a floatexp complex `v` (as the column (re, im)).
fn fe_mat(m: vec4<f32>, me: i32, v: Fe) -> Fe {
    let re = df_add(df_mul_f32(v.m.re, m.x), df_mul_f32(v.m.im, m.y));
    let im = df_add(df_mul_f32(v.m.re, m.z), df_mul_f32(v.m.im, m.w));
    return fe_norm(cset(re, im), v.e + me);
}
fn fam_shape(f: u32) -> u32 { return (f - 10u) / 3u; }
fn fam_power(f: u32) -> u32 {
    let k = f - 10u;
    return select(3u + k % 3u, 6u + k, k < 3u);
}
// The escape test's |z|²: 256² for every family but Multibrot 8, whose escaping step from |z| = 256
// would square past f32 (|z|² up to 256^16 = 2^128): it escapes at 128.
fn bail2_of(f: u32) -> f32 { return select(256.0 * 256.0, 128.0 * 128.0, f == 12u); }
// b^d (d = 3..8) as the generated module chains it — square, and multiply by b on each set bit
// below the top one, most significant first — with the same helper calls, so a family's direct
// step is its generated module's bit for bit (self-test "generated … = built-in").
fn fam_pow_df(b: Cdf, d: u32) -> Cdf {
    switch (d) {
        case 3u: { return c_mul(c_sqr(b), b); }
        case 4u: { return c_sqr(c_sqr(b)); }
        case 5u: { return c_mul(c_sqr(c_sqr(b)), b); }
        case 6u: { return c_sqr(c_mul(c_sqr(b), b)); }
        case 7u: { return c_mul(c_sqr(c_mul(c_sqr(b), b)), b); }
        default: { return c_sqr(c_sqr(c_sqr(b))); } // 8
    }
}
// ⭐⭐THE EXACT-REPEAT CHECK (Brent). In the direct loops the next z is a function of z alone (c
// is fixed per pixel), so once z repeats BIT FOR BIT the arithmetic cycles forever and the pixel
// can never escape: it is the interior it would be at `max_iter`, and ending it at the repeat
// changes no output, only the time. The repeat is found by saving z at iterations 1, 2, 4, 8, …
// of the loop and comparing every step against the saved one, so a cycle of period p entered by
// step T is found by about 2·max(T, p).
// Why it matters: these loops run in plain f32 (`EFT_ADD_RESIDUALS` off), and near the critical
// point z^d falls below c's last bit (|z| < 0.126 for d = 8), so z^d + c rounds to exactly c and
// the orbit replays c's own, which repeats. Measured 2026-11-06 on a Multibrot 8 Julia set at
// c ≈ 0.32295 + 0.98330i: 1,500 pixels (0.09%) that cycle this way took 97% of the panel's
// iterations at 267,346 each, and every one repeats by iteration 568. (A double-precision orbit
// of those pixels escapes in 20–491 steps, so they were wrong as well as slow: the precision's
// doing, which this check neither causes nor hides. The honest step, `honest_step`, is the fix.)
// Only where z is the whole state: the built-in formulas but Phoenix (8, which reads z_{n−1})
// and Newton (9, which converges by its own test). A custom formula (1000) may keep variables
// of its own between steps, so it is never checked.
// The df32 perturbation loop (mode 0) checks its own state the same way: its next step reads only
// the reference index and δz (and the pixel's δc), so a repeat of both cycles for ever too.
// ⭐ORBIT STATISTICS TOO (2026-11-07). An interior pixel's statistics show only through the orbit
// trap (`coord_under`: stripe, TIA and decomposition draw the interior in the interior color), and
// by the repeat every point of the cycle has been through `aux_step`, so the trap's nearest
// approach is already final. Measured on a Multibrot 7 orbit-trap view at 6.4e6× and 250,000
// iterations: 0.5% of the samples were interior and took 85% of the frame's iterations.
fn cycle_check_on(f: u32) -> bool {
    return (f <= 7u || is_fam(f)) && (iu.tail_on & 2u) == 0u;
}
fn cdf_bits_eq(a: Cdf, b: Cdf) -> bool {
    return all(bitcast<vec4<u32>>(vec4<f32>(a.re, a.im)) == bitcast<vec4<u32>>(vec4<f32>(b.re, b.im)));
}
// A family's direct step without its `+ c`: the fold, the power, the fold.
fn fam_direct(f: u32, z: Cdf) -> Cdf {
    let shape = fam_shape(f);
    var b = z;
    if (shape == 1u) { b = cset(df_abs(z.re), df_abs(z.im)); }
    if (shape == 2u) { b = c_conj(z); }
    var w = fam_pow_df(b, fam_power(f));
    if (shape == 3u) { w = cset(df_abs(w.re), w.im); }
    if (shape == 4u) { w = cset(df_abs(w.re), df_abs(w.im)); }
    return w;
}
// Multibrot d's f'(z) = d·z^(d−1), plain f32 as deriv_factor.
fn fam_deriv(f: u32, z: vec2<f32>) -> vec2<f32> {
    let d = fam_power(f);
    var r = z;
    for (var k = 2u; k < d; k = k + 1u) {
        r = vec2<f32>(r.x * z.x - r.y * z.y, r.x * z.y + r.y * z.x);
    }
    return f32(d) * r;
}

// ⭐⭐THE DERIVATIVE AT AN ORBIT DIP (2026-11-21). The step D ← f'(z_n)·D takes z_n = Z_n + δz_n in
// plain f32 (`deriv_factor`, `fam_deriv`), which cannot hold it at a deep dip of the orbit: an
// extended-range sample reads Z as 0 (`orbit_cdf`), `fe_lo_f32` flushes a δz under f32's range, and
// a power's product underflows (5z⁴ from |z| < 2^-31). The factor came out 0 there and D restarted at
// 1 at every such dip, so past about 1e77x on a minibrot's path every distance read far too large:
// 4.04 px against a true 4.6e-5 px (mpmath) at the 9.3e78 minibrot's center, 1e77x, where it was exact
// at 1e70x (a periodic reference there, the dip renormalized). The δz step was given this handling for
// the same dips (the "LOAD-BEARING dip handling" below); the derivative was left on `orbit_cdf`. Where
// either test holds, the factor is formed in floatexp from the extended-range z; every other step
// keeps its f32 path, so a view whose orbits never dip that far is bit-identical.
fn deriv_dip(r: vec4<f32>, fp: vec2<f32>) -> bool {
    return orbit_is_ext(r) || max(abs(fp.x), abs(fp.y)) < 7.888609e-31; // 2^-100
}
// f'(z) = d·z^(d−1) in floatexp, d the escape degree: Mandelbrot and Multibrot 3–5 (formulas 0..3),
// Multibrot 6–8 (`fam_holo`); Phoenix's 2z is the default.
fn fe_deriv_factor(formula: u32, z: Fe) -> Fe {
    var d = 2u;
    if (formula <= 3u) { d = formula + 2u; } else if (fam_holo(formula)) { d = fam_power(formula); }
    var w = z;
    for (var k = 2u; k < d; k = k + 1u) { w = fe_mul(w, z); }
    return fe_scale(w, f32(d));
}
// (Z + δ)^d − Z^d along the power's own chain (square; multiply by the base on each set bit below
// the top one), one perturbation rule per link — P(sqr f) = (2·B + P)·P, P(f·g) = P(f)·W(g) +
// B(f)·P(g) with W = B + P — as the generated module's rule table (`ir::perturb`) derives it. Every
// term carries a P, so nothing cancels against Z^d, and the error grows with the chain's log₂ d
// links: a table of Z's powers built by d − 2 sequential products (a binomial in Horner form) was
// measured 1.6–2.3× further from the CPU for Multibrot 6–8.
// ⛔The full-size 2·B + P and Z + δ here are a plain `c_add`, which rounds to f32 and drops Z's low
// word: the rounding GitHub #5 removed from the generated module (`c_add_lo`, `custom.rs`
// `wide_operand`). Both ways of keeping the word slowed THIS kernel for every power family, measured
// warm on an RTX 3080 (Buffalo 5, 32x32, 30,000 iterations, 2026-11-16): `c_add_lo` 329 ms and the
// distributed products 2·(B·P) + P² 517 ms, against 133 ms, though `c_add_lo` is the cheaper
// arithmetic; the distributed build lost the device twice in the self-test, its passes priced for
// the old cost. A low-word fold here needs its own measurement (TODO.md).
fn fam_dpow_df(Z: Cdf, dz: Cdf, d: u32) -> Cdf {
    var B = Z;
    var P = dz;
    let W0 = c_add(Z, dz);
    var bit = 31u - countLeadingZeros(d);
    loop {
        if (bit == 0u) { break; }
        bit = bit - 1u;
        P = c_mul(c_add(c_two(B), P), P);
        B = c_sqr(B);
        if (((d >> bit) & 1u) == 1u) {
            P = c_add(c_mul(P, W0), c_mul(B, dz));
            B = c_mul(B, Z);
        }
    }
    return P;
}
// The same in floatexp (mode 2): every P in floatexp, so nothing underflows; B and the full-size
// values (B + P, Z + δ) in df32, which holds them — a perturbation's digits below df32's reach are
// negligible beside B (the generated module's `Class::Full`).
fn fam_dpow_fe(Z: Cdf, dz: Fe, d: u32) -> Fe {
    var B = Z;
    var P = dz;
    let W0 = c_add(Z, fe_to_cdf(dz));
    var bit = 31u - countLeadingZeros(d);
    loop {
        if (bit == 0u) { break; }
        bit = bit - 1u;
        P = fe_mul_cdf(P, c_add(c_two(B), fe_to_cdf(P)));
        B = c_sqr(B);
        if (((d >> bit) & 1u) == 1u) {
            P = fe_add(fe_mul_cdf(P, W0), fe_mul_cdf(dz, B));
            B = c_mul(B, Z);
        }
    }
    return P;
}
// A family's perturbed step without its `+ δc` (df32, mode 0). A fold perturbs by diffabs: on Z and
// δ before the power (Burning Ship), on Z^d and δ(z^d) after it (Celtic, Buffalo).
fn fam_pert_df(f: u32, Z: Cdf, dz: Cdf) -> Cdf {
    let shape = fam_shape(f);
    let d = fam_power(f);
    if (shape == 1u) {
        let bz = cset(df_abs(Z.re), df_abs(Z.im));
        let bd = cset(df_diffabs(Z.re, dz.re), df_diffabs(Z.im, dz.im));
        return fam_dpow_df(bz, bd, d);
    }
    if (shape == 2u) { return fam_dpow_df(c_conj(Z), c_conj(dz), d); }
    let dw = fam_dpow_df(Z, dz, d);
    if (shape == 0u) { return dw; }
    let W = fam_pow_df(Z, d);
    if (shape == 3u) { return cset(df_diffabs(W.re, dw.re), dw.im); }
    return cset(df_diffabs(W.re, dw.re), df_diffabs(W.im, dw.im));
}
// The same in floatexp (mode 2).
fn fam_pert_fe(f: u32, Z: Cdf, dz: Fe) -> Fe {
    let shape = fam_shape(f);
    let d = fam_power(f);
    if (shape == 1u) {
        let bz = cset(df_abs(Z.re), df_abs(Z.im));
        let bd = fe_from_sf(sf_diffabs(sf_from_df(Z.re), sf_re(dz)), sf_diffabs(sf_from_df(Z.im), sf_im(dz)));
        return fam_dpow_fe(bz, bd, d);
    }
    if (shape == 2u) { return fam_dpow_fe(c_conj(Z), fe_conj(dz), d); }
    let dw = fam_dpow_fe(Z, dz, d);
    if (shape == 0u) { return dw; }
    let W = fam_pow_df(Z, d);
    if (shape == 3u) { return fe_from_sf(sf_diffabs(sf_from_df(W.re), sf_re(dw)), sf_im(dw)); }
    return fe_from_sf(sf_diffabs(sf_from_df(W.re), sf_re(dw)), sf_diffabs(sf_from_df(W.im), sf_im(dw)));
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
    // The orbit statistics this frame accumulates: STAT_* bits (coloring plan B2). Not the method
    // on screen, which only the color pass reads (`ColorU.color_method`). See lib.rs `iter_stats`.
    orbit_stats: u32,
    stripe_freq: f32,      // stripe-average angular frequency
    trap_type: u32,        // orbit-trap shape: a TRAP_* (`trap_dist_of`)
    // Bitfield: bit 0 = accumulate orbit statistics into the aux target; bit 1 = stripe average
    // over the TAIL only (an exponential window, see `stripe_tail_on`); bits 2.. = that window's
    // length in iterations. Mirrors `aux_on_word` in lib.rs.
    aux_on: u32,
    sa_skip: u32,          // series-approximation skip (0 = none): seed δz at this iteration
    glitch_on: u32,        // 1 = flag Pauldelbrot-glitched pixels (multi-reference correction)
    sa_a: vec4<f32>,       // order-3 series coeffs (complex df32 mantissa): δz ≈ A·δc + B·δc² + C·δc³
    sa_b: vec4<f32>,
    sa_c: vec4<f32>,
    sa_a_exp: i32,         // per-coefficient base-2 exponents (floatexp)
    sa_b_exp: i32,
    sa_c_exp: i32,
    bla_on: u32,           // 1 = use the BLA tree (appended after the orbit at index orbit_len)
    start_iter: u32,       // iteration-range tiling (direct mode): resume the loop at this iteration
    end_iter: u32,         // ...and stop this dispatch here (fs_iterate_chunk); fs_iterate ignores both
    // Scattered-gather pass (`fs_iterate_gather`) only; zero on every other path. These are the two
    // words that already padded the struct to a 16-byte multiple (WGSL rounds a struct up to its
    // alignment, Rust does not), so giving them a meaning moved no existing field and changed no
    // size - which matters, because this one uniform is bound by EVERY iterate pipeline in the app,
    // the live view included.
    gather_w: u32,         // width in texels of the tiny gather texture
    gather_n: u32,         // how many of its texels carry a real coordinate
    // Bit 0: the mode-2 df32 tail phase is enabled (TAIL_DF32_MIN; `TAIL_DF32` tunable). Bit 1: the
    // exact-repeat check is OFF (`cycle_check_on`, direct and mode 0; `CYCLE_CHECK` tunable). Bit 2: the direct loops'
    // honest step past the critical point is OFF (`honest_power`; `HONEST_STEP` tunable). A whole
    // 16-byte row with its padding, mirrored by `IterUniforms::tail` in lib.rs.
    tail_on: u32,
    // `fs_iterate_chunk_fe` only: stop each pixel after this many EXECUTED steps (BLA skips + full
    // steps) in this pass; 0 = no cap. The step-bounded export passes (beta.150) set it. A former
    // padding word of the tail row, so no field moved (see `gather_w` above for why that matters).
    step_cap: u32,
    // `fs_iterate` only: a SPLIT live refresh (`split_n` > 1) renders the pixels of one checkerboard
    // set of `SPLIT_TILE`-texel tiles per pass — set `split_j` of `split_n` — and leaves the rest of
    // the texture as it is. The last two padding words of the tail row, so no field moved.
    split_n: u32,
    split_j: u32,
    // THE RENORMALIZED STEP (`rn_steps`; `RenormStep` in fractadyne-core derives every field):
    // u = a·z takes `rn_len` iterations per step as u ↦ u² + c′, c′ − RefC = (a·b)·δc, perturbed
    // about U[k] = a·Z[k·rn_len]; it leaves at |u|² > rn_r2 with z = Z + δu·(1/a) and δz/δc = Du·b.
    // `rn_ref_c` is plain df32; the others are a df32 mantissa and an exponent. `rn_len` 0 = off;
    // a pixel past `rn_dc_log2` (log₂|δc|) iterates as before. Mirrors `Renorm` in lib.rs.
    rn_a: vec4<f32>,
    rn_ab: vec4<f32>,
    rn_ref_c: vec4<f32>,
    rn_inv_a: vec4<f32>,
    rn_b: vec4<f32>,
    rn_a_exp: i32,
    rn_ab_exp: i32,
    rn_inv_a_exp: i32,
    rn_b_exp: i32,
    rn_len: u32,
    rn_r2: f32,
    rn_dc_log2: f32,
    // THE U-SPACE BLA (`renorm_bla_gpu` in fractadyne-core): 1 + its first node's index past the
    // orbit (nodes at `orbit_len + rn_bla_off − 1`), 0 = none; it covers `rn_bla_k` u-steps.
    rn_bla_off: u32,
    rn_bla_k: u32,
    // `fs_iterate_chunk_fe` only: 1 = a LIVE chunk pass counts the pixels it leaves running into
    // `CTR_CHUNK_RUNNING`, as a step-capped export pass always does (`MandelbrotParams::chunk_walk`;
    // the app sizes the walk's later passes by it). `Renorm::pad[0]` in lib.rs; a padding word, so no
    // field moved.
    count_running: u32,
    // The base the fine escape histogram is measured from (`CTR_NORM_HIST`; `Renorm::pad[1]` in
    // lib.rs, f32 bits): the bottom of the view's last escape-range reading. A padding word, so no
    // field moved.
    norm_base: f32,
    rn_pad3: u32,
    // EXPONENTIAL-MAP STRIPS (`design/zoom-video.md` §4; `StripMap` in lib.rs): read only by a strip's
    // own module (`strip.rs`, which splices `strip.wgsl` into the STRIP_* slots below): `map_s` = 2π/W,
    // `map_r` = the strip's outer radius over 2^delta_exp; `map_mode` 1 marks a strip. A row of its
    // own after the renormalized step's, so no field moved.
    map_mode: u32,
    map_s: f32,
    map_r: f32,
    map_pad: u32,
    // THE REFERENCE'S OWN CYCLE (`ref_fold` in lib.rs), for mode 0's exact-repeat check only: the
    // stored reference repeats exactly with period `fold_by` from index `fold_end − fold_by` to its
    // end, so the check compares reference indices reduced into that period. 0 = no fold. A row of
    // its own after the strip map's, so no field moved.
    fold_end: u32,
    fold_by: u32,
    // THE CRITICAL ORBIT of a Julia view (`crit_on`): its first sample's index in `reference` and
    // its sample count; 0 = none (every Mandelbrot view, and the old rule). The fold row's two
    // spare words, so no field moved.
    crit_at: u32,
    crit_len: u32,
    // THE ORBIT TRAP'S PLACEMENT (coloring plan F4; lib.rs `TrapU`, `trap_uniform`): `trap_dist`
    // measures in the shape's coordinates w = R(−θ)·(z − trap_c)·trap_inv and scales back by
    // trap_size. trap_on 0 (every field 0): the unit shape at the origin, no transform at all.
    // Two rows of their own after the fold row, so no field moved.
    trap_c: vec2<f32>,
    trap_turn: vec2<f32>,  // the turn's (cos, sin)
    trap_size: f32,
    trap_inv: f32,         // 1 / trap_size
    trap_on: u32,
    trap_phase: f32,      // the sine wave's phase, radians (0: as before)
    // THE INSIDE'S OWN TRAP (lib.rs `trap_in_uniform`): its placement as the trap's above, then
    // whether it is on and its shape. On, the iteration measures both traps and an interior pixel's
    // trap lane holds this one (`aux_interior`); off (zeros), the inside measures the trap above.
    trap_in_c: vec2<f32>,
    trap_in_turn: vec2<f32>,
    trap_in_size: f32,
    trap_in_inv: f32,
    trap_in_on: u32,
    trap_in_phase: f32,
    trap_in_enabled: u32,
    trap_in_type: u32,
    // Each trap's count (lib.rs `trap_in_uniform`), the trap's and the inside's own: the square
    // wave's harmonics, the sine wave's overlaid ones. What were the row's padding words.
    trap_harm: u32,
    trap_in_harm: u32,
    // THE VIEW'S TRAP OUTLINE (lib.rs `outline_row`, coloring plan F4 part 4): where its cells,
    // segment indices and segments start in `reference`, and 1 when it is there.
    outline_cells: u32,
    outline_idx: u32,
    outline_segs: u32,
    outline_on: u32,
    // THE TRAP'S TAIL (lib.rs `trap_tail_row`, coloring plan F4 part 5): each iteration every hit's
    // distance grows by trap_tail_g (1: off), a skipped run of `span` iterations by
    // 2^(span * trap_tail_inv) (0: off).
    trap_tail_g: f32,
    trap_tail_inv: f32,
    trap_tail_pad0: f32,
    trap_tail_pad1: f32,
};
@group(0) @binding(0) var<uniform> iu: IterU;
// Reference orbit as double-single: each Z_n = (re.hi, im.hi, re.lo, im.lo). When BLA is on,
// the BLA tree is appended right after (starting at index orbit_len): 4 vec4 per node —
// [A mantissa], [B mantissa], [a_exp, b_exp, r_exp, r_mant], [span, trap_min, ΣTIA, Σstripe]
// (the aux aggregates over the node's span; see `reference.rs`).
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
// vec4s a BLA node takes in `reference` (`BLA_NODE_VEC4S` in the Rust): A, B, the exponents and
// radius, [span, trap, TIA, stripe], [window, 0, 0, 0].
const BLA_NODE: u32 = 5u;
const CTR_BLA_SKIP: u32 = 3u;    // BLA multi-step skips taken
const CTR_MAXITER: u32 = 4u;     // fragments that exhausted max_iter
const CTR_ESC_MIN: u32 = 5u;     // min escaped smooth-iter (f32 bits; positive floats sort as u32)
const CTR_ESC_MAX: u32 = 6u;     // max escaped smooth-iter (f32 bits)
// ESCAPED PIXELS in the frame — the count the app checks before it lets a frame reach the screen
// or become the held snapshot (a walk that has not reached the view's first escape resolves to
// one flat interior color; nothing else on the wire can tell that frame from a finished one).
// Un-subsampled, like CTR_MAXITER: it is a COUNT against the frame's pixel total.
const CTR_ESC_COUNT: u32 = 7u;
// LOCAL GRADIENT of the escape field: Σ|Δ smooth-iter| between horizontally adjacent escaped
// pixels (×16, clamped), and the number of samples in that sum. Their ratio is the mean step the
// palette takes from one pixel to the next, which is what decides whether a fixed cycle ALIASES.
// ⭐The escape RANGE cannot decide that: a shallow view spans a hundred palette sweeps and still
// looks perfect because neighbors differ by ≪1 iteration, while a deep dense field aliases on a
// far narrower span. Range is a global statistic; aliasing is a local one.
const CTR_GRAD_SUM: u32 = 8u;
const CTR_GRAD_N: u32 = 9u;
// First of 12 log2 buckets of the same per-pair step (keep in sync with lib.rs `CTR_GRAD_HIST` /
// `GRAD_HIST_BUCKETS`): bucket 0 = step < 1, bucket b >= 1 = [2^(b-1), 2^b), bucket 11 open-ended.
// The mean above hides a dense aliasing region inside a smooth one; the DISTRIBUTION does not, and
// the app reads it against the palette cycle it is actually using. See lib.rs for the field case.
const CTR_GRAD_HIST: u32 = 10u;
// First of 24 log2 buckets of the escaped pixels' smooth-iteration count (keep in sync with lib.rs
// `CTR_ESC_HIST` / `ESC_HIST_BUCKETS`): bucket b = [2^b, 2^(b+1)), bucket 0 also takes < 1, bucket
// 23 open-ended. The app sizes a moving frame to how far MOST of the picture escapes, not its
// slowest pixel (lib.rs has the field case). Written only by `fs_resolve`, on the 4x4 grid.
const CTR_ESC_HIST: u32 = 22u;
fn esc_hist_commit(sm: f32) {
    if (sm >= 0.0) {
        let b = u32(clamp(floor(log2(max(sm, 1.0))), 0.0, 23.0));
        atomicAdd(&counters[CTR_ESC_HIST + b], 1u);
    }
}
// First of 416 buckets of a FINE histogram of the escaped pixels' smooth-iteration count, for the
// live palette window (keep in sync with lib.rs `CTR_NORM_HIST` / `NORM_HIST_BUCKETS` /
// `norm_hist_bucket`): 16 buckets per doubling of `v − norm_base + 1`, the last open-ended. The
// app takes the window's top from a quantile of it, so a few pixels escaping just under the
// iteration ask beside a minibrot no longer set the colors of the whole picture.
const CTR_NORM_HIST: u32 = 57u;
// ⭐THE ORBIT TRAP'S VALUE HISTOGRAM, for "Always fit range" with the orbit-trap method (keep in
// sync with lib.rs `CTR_TRAP_HIST` / `TRAP_HIST_*`): the colored value, −log₂ of the orbit's nearest
// approach to the trap (`coord_under`), in fixed buckets of 1/16 from −8 to 68. The app takes the
// palette window from its 0.5% and 99.5% quantiles: in the Mandelbrot plane most of a view's values
// can sit inside two units while a pixel on a nucleus reaches 66, so a min/max window would leave
// the picture as flat as no fitting at all. Counted on the 4x4 grid, interior pixels too (the trap
// colors them), from the single-pass iterate only: orbit statistics never take the chunked walk.
// Counted whenever the trap is ACCUMULATED, whatever is on screen: the app reads it only while the
// trap is shown, and a switch to the trap that only recolors then finds it already counted.
const CTR_TRAP_HIST: u32 = 473u;
const TRAP_HIST_LO: f32 = -8.0;
const TRAP_HIST_PER_UNIT: f32 = 16.0;
const TRAP_HIST_BUCKETS: u32 = 1216u;
// The same histogram over the INTERIOR pixels alone (`inside`), in the next 1216 slots (lib.rs
// `CTR_TRAP_HIST_IN`, coloring plan C4): the window the inside's fitted palette spans.
const CTR_TRAP_HIST_IN: u32 = 1689u;
fn trap_hist_bucket(t: f32) -> u32 {
    let pv = -log2(max(t, 1.0e-20));
    return u32(clamp((pv - TRAP_HIST_LO) * TRAP_HIST_PER_UNIT, 0.0, f32(TRAP_HIST_BUCKETS - 1u)));
}
fn trap_hist_commit(a: Aux, gx: f32, gy: f32, inside: bool) {
    if ((iu.aux_on & 1u) == 1u && (iu.orbit_stats & STAT_TRAP) != 0u && (i32(gx) & 3) == 0 && (i32(gy) & 3) == 0) {
        let k = trap_hist_bucket(a.trap);
        atomicAdd(&counters[CTR_TRAP_HIST + k], 1u);
        // The inside's fit spans what colors the inside: its own trap, when it has one.
        if (inside) {
            var ki = k;
            if (iu.trap_in_enabled != 0u) { ki = trap_hist_bucket(a.trap_in); }
            atomicAdd(&counters[CTR_TRAP_HIST_IN + ki], 1u);
        }
    }
}
// On the 4x4 grid of the escape-range commit; called where a whole frame's escapes are seen once
// each: `fs_resolve` on a chunked frame, the single-pass iterates' escape otherwise.
fn norm_hist_commit(sm: f32, p: vec2<i32>) {
    if (sm >= 0.0 && (p.x & 3) == 0 && (p.y & 3) == 0) {
        let x = max(sm - iu.norm_base, 0.0) + 1.0;
        let b = u32(clamp(floor(log2(x) * 16.0), 0.0, 415.0));
        atomicAdd(&counters[CTR_NORM_HIST + b], 1u);
    }
}

fn ctr_commit(n_rebase: u32, n_ext: u32, n_bla: u32) {
    if (n_rebase > 0u) { atomicAdd(&counters[CTR_REBASE], n_rebase); }
    if (n_ext > 0u) { atomicAdd(&counters[CTR_EXT_SAMPLE], n_ext); }
    if (n_bla > 0u) { atomicAdd(&counters[CTR_BLA_SKIP], n_bla); }
}

// STEP ACCOUNTING (keep in sync with lib.rs `CTR_STEP_*`): on a fixed 1-in-64 pixel grid, the
// loop trips a perturbation fragment EXECUTED against the iterations it ADVANCED. `CTR_BLA_SKIP`
// counts skips taken, not work done, so nothing else here can say how many iterations one executed
// step delivers — the number a comparison with another renderer's skip scheme needs. The sums are
// two-word (lo, hi): a 4K frame's sampled iterations pass 2^32. Pixel output is untouched.
const CTR_STEP_PX: u32 = 46u;   // sampled fragments that ran a perturbation loop
const CTR_STEP_EXEC: u32 = 47u; // lo/hi: loop trips (BLA skips + full steps)
const CTR_STEP_ITER: u32 = 49u; // lo/hi: iterations advanced
const CTR_STEP_FULL: u32 = 51u; // lo/hi: full (one-iteration) steps
const CTR_STEP_BIG: u32 = 53u;  // lo/hi: mode-2 full steps taken in the df32 tail phase (TAIL_DF32_MIN)
// Pixels a step-capped chunk pass (`step_cap` > 0) left still RUNNING. Cleared and read back by the
// host after every such pass: the tile is done when it reads 0. Counted only under a cap, so the
// live view's chunk passes pay no atomic for it.
const CTR_CHUNK_RUNNING: u32 = 55u;
// Pixels that took THE RENORMALIZED STEP (`rn_enter`), on the step-accounting grid above: the proof
// it ran, against `CTR_STEP_PX` sampled pixels.
const CTR_RENORM: u32 = 56u;
fn add_u64(slot: u32, v: u32) {
    if (v == 0u) { return; }
    let old = atomicAdd(&counters[slot], v);
    if (old + v < old) { atomicAdd(&counters[slot + 1u], 1u); } // carry (u32 add wraps)
}
fn step_commit(gx: f32, gy: f32, n_exec: u32, n_full: u32, n_big: u32, n_iter: u32) {
    if ((u32(gx) & 7u) != 0u || (u32(gy) & 7u) != 0u) { return; }
    atomicAdd(&counters[CTR_STEP_PX], 1u);
    add_u64(CTR_STEP_EXEC, n_exec);
    add_u64(CTR_STEP_ITER, n_iter);
    add_u64(CTR_STEP_FULL, n_full);
    add_u64(CTR_STEP_BIG, n_big);
}

// ⭐⭐THE RENORMALIZED STEP (`RenormStep` in fractadyne-core derives it; read that first). Near a
// minibrot, `rn_len` iterations of z² + c are, in u = a·z, the Mandelbrot map u ↦ u² + c′ itself,
// with c′ = RefC + (a·b)·δc. A pixel within the model's |δc| starts at u = 0 and takes a whole
// `rn_len`-iteration period per step in df32, instead of perturbation steps over it; it leaves at
// |u|² > `rn_r2` (z = u/a, iteration k·rn_len, on Z₀ = 0), and a pixel still inside at the cap
// never escapes. Mandelbrot only, never with aux statistics (they need every iterate) nor glitch
// detection (correction passes stay as they were).
const RN_MARGIN: f32 = 0.0009765625;  // 2^-10: the model is held to 2^-24, f32 to ~2^-23
// A pass under a step cap charges one renormalized step as 1/RN_STEP_WEIGHT of a perturbation step.
const RN_STEP_WEIGHT: u32 = 8u;
// `ref_n` (info ch2) of a chunked pixel paused INSIDE the renormalized step: never a real index.
const RN_PAUSED: f32 = -1.0;
fn rn_on() -> bool {
    return iu.rn_len > 0u && iu.formula == 0u && iu.julia == 0u && (iu.aux_on & 1u) == 0u
        && iu.glitch_on == 0u;
}
// The pixel's δc′ = (a·b)·δc in floatexp, or `ok = false` past the model's |δc|.
struct RnC {
    ok: bool,
    dcp: Fe,
};
fn rn_c(dc: Fe) -> RnC {
    var r: RnC;
    r.ok = false;
    r.dcp = fe_zero();
    let dcm = length(vec2<f32>(dc.m.re.x, dc.m.im.x));
    if (dcm > 0.0) {
        if (log2(dcm) + f32(max(dc.e, -100000)) > iu.rn_dc_log2) { return r; }
        r.dcp = fe_mul(fe_norm(cset(iu.rn_ab.xy, iu.rn_ab.zw), iu.rn_ab_exp), dc);
        if (r.dcp.e > 60) { return r; } // |δc′| beyond any use
    }
    r.ok = true;
    return r;
}
// Principal square root of a complex f32.
fn csqrt_f32(w: vec2<f32>) -> vec2<f32> {
    let r = length(w);
    let sx = sqrt(max((r + w.x) * 0.5, 0.0));
    let sy = sqrt(max((r - w.x) * 0.5, 0.0));
    return vec2<f32>(sx, select(sy, -sy, w.y < 0.0));
}
// c′ inside the main cardioid or the period-2 bulb of u² + c′ (with `RN_MARGIN`): an attracting
// cycle, so the pixel never leaves and never escapes — it can go straight to the cap.
fn rn_settled(dcp: Fe) -> bool {
    var c = vec2<f32>(iu.rn_ref_c.x, iu.rn_ref_c.z);
    if (dcp.e > -120) { c = c + fe_lo_f32(dcp); }
    let s = csqrt_f32(vec2<f32>(1.0 - 4.0 * c.x, -4.0 * c.y));
    if (length(vec2<f32>(1.0 - s.x, -s.y)) < 1.0 - RN_MARGIN) { return true; }
    return 4.0 * length(vec2<f32>(c.x + 1.0, c.y)) < 1.0 - RN_MARGIN;
}
// The u-space reference at step `kr`: U = a·Z_{kr·rn_len}, read from the orbit (extended dips too).
fn rn_ref_u(kr: u32) -> Fe {
    return fe_mul(fe_norm(cset(iu.rn_a.xy, iu.rn_a.zw), iu.rn_a_exp), orbit_fe(reference[kr * iu.rn_len]));
}
struct RnState {
    du: Fe,    // u − U_kr
    ddu: Fe,   // du/dc′
    k: u32,    // renormalized steps taken
    kr: u32,   // u-reference index (step kr reads Z_{kr·rn_len})
    out: bool, // left the model (|u|² > rn_r2)
    trips: u32, // loop trips this call (a u-step, or a u-space BLA skip of several)
};
// The u-space BLA is compiled in only for a render that has a tree (`iter_constants` in lib.rs):
// present but switched off, its code cost every other deep render 3–9% (measured at 1.2e148 and
// 4.2e275, 4K), and compiled out, nothing.
override RN_BLA: bool = false;
// The u-space BLA is in use (and so is the BLA: `bla_on` = 0 turns both off).
fn rn_bla_on() -> bool {
    return RN_BLA && iu.rn_bla_off > 0u && iu.rn_bla_k > 1u && iu.bla_on == 1u;
}
// A loop trip's charge against a pass's step cap, in 1/RN_STEP_WEIGHT steps: a plain u-step is
// one, a trip that may search the u-space tree costs about a perturbation step.
fn rn_trip_weight() -> u32 {
    return select(1u, RN_STEP_WEIGHT, rn_bla_on());
}
// Renormalized steps as PERTURBATION about the u-space reference: δu′ = 2U·δu + δu² + δc′. Plain u
// in df32 lost a tuned minibrot's structure — near a u-space return |u| is tiny and u² fell under
// df32's resolution beside c′ ≈ 1, so every cycle restarted the pixel (median error 34 iterations at
// the ladder's period 15,248) — the same failure the main loop's rebasing exists for, and the same
// cure: rebase (onto U_0 = 0) when |u| < |δu| and at the reference's end, keeping δu relative.
//
// ⭐THE U-SPACE BLA. Near a tuned minibrot the u-reference has its own period (998 steps at the
// ladder's 951,094 = 953·998), and a pixel follows it for several cycles; one u-step per `rn_len`
// iterations was 97% of that render. In u-space the parent's near-returns are gone, so the BLA
// tree over U (`rn_bla_*`, the main loop's layout and node format) skips along it as the main loop
// skips along Z: the same level search, the same validity test, `A·δu + B·δc′`, and a skip that
// would land outside the model (|u|² > rn_r2) takes a smaller one, so a pixel leaves on the same
// u-step it would have. Bounded by `k_max` (never past the cap), never by the pass's own end, so
// where passes split does not change a pixel; `trip_max` bounds the pass's work in loop trips.
// May a plain u-step update δu in df32? `tail_step_ok`'s rule on the u-step's operands: δu at least
// 2^-50, and U zero (after every rebase) or at least 2^-50; both well inside f32's range (|u| ≤ R ≤
// 2^60, so δu² ≤ 2^122).
fn rn_tail_ok(U: Fe, du: Fe) -> bool {
    let u_zero = U.m.re.x == 0.0 && U.m.im.x == 0.0;
    return tail_df32_ok() && (u_zero || (U.e >= TAIL_DF32_MIN_E && U.e <= 60))
        && du.e >= TAIL_DF32_MIN_E && du.e <= 60;
}
fn rn_steps(dcp: Fe, s0: RnState, k_end: u32, trip_max: u32) -> RnState {
    var s = s0;
    s.trips = 0u;
    let k_max = rn_k_max();
    // The tree's levels: level i has `((K − 1) >> i) + 1` nodes, level 0 first. No per-level
    // arrays (they live in memory, not registers): a trip starts at the highest level its index
    // is aligned to and walks the offsets down by subtraction.
    let kk = iu.rn_bla_k;
    var ub_levels = 0u;
    if (rn_bla_on()) {
        ub_levels = 32u - countLeadingZeros(kk - 1u) + 1u; // until a level has one node
    }
    let ub_base = iu.orbit_len + iu.rn_bla_off - 1u;
    // U at `s.kr`, kept from the trip that moved there (`rn_ref_u` is a pure function of the index;
    // a plain step used to compute it twice).
    var Uk = rn_ref_u(s.kr);
    let dcd = tail_dc(dcp);
    loop {
        if (s.k >= k_end || s.trips >= trip_max) { break; }
        s.trips = s.trips + 1u;
        // A trip whose δu the level-0 node at `kr` already rejects skips the search: every node the
        // search could take starts at `kr`, and a merged radius is at most its first child's
        // (`bla_merge`), so no level can accept it either. 70% of the trips at the ladder's period
        // 951,094 were such plain steps, each after a full failed search.
        var blocked = true;
        if (ub_levels > 0u && s.kr < kk) {
            let r0 = reference[ub_base + s.kr * BLA_NODE + 2u];
            blocked = !sf_lt(fe_abs_sf(s.du), sf_norm(vec2<f32>(r0.w, 0.0), i32(r0.z)));
        }
        if (!blocked) {
            let dum = fe_abs_sf(s.du);
            var applied = false;
            var l = ub_levels - 1u;
            if (s.kr != 0u) { l = min(l, countTrailingZeros(s.kr)); }
            var off = 0u;
            for (var i = 0u; i < l; i = i + 1u) { off = off + ((kk - 1u) >> i) + 1u; }
            loop {
                let j = s.kr >> l;
                if (j < ((kk - 1u) >> l) + 1u) {
                    let node = ub_base + (off + j) * BLA_NODE;
                    let v2 = reference[node + 2u];
                    let span = u32(reference[node + 3u].x);
                    if (span != 0u && s.kr + span <= kk && s.k + span <= k_max
                        && sf_lt(dum, sf_norm(vec2<f32>(v2.w, 0.0), i32(v2.z)))) {
                        let v0 = reference[node];
                        let v1 = reference[node + 1u];
                        let A = fe_norm(cset(v0.xy, v0.zw), i32(v2.x));
                        let B = fe_norm(cset(v1.xy, v1.zw), i32(v2.y));
                        let ndu = fe_add(fe_mul(A, s.du), fe_mul(B, dcp));
                        let nkr = s.kr + span;
                        let Un = rn_ref_u(nkr);
                        let u = fe_add(Un, ndu);
                        // A skip that would land outside the model takes a smaller one.
                        if (fe_mag2(u) <= iu.rn_r2) {
                            s.ddu = fe_add(fe_mul(A, s.ddu), B);
                            s.du = ndu;
                            s.kr = nkr;
                            s.k = s.k + span;
                            Uk = Un;
                            if (sf_lt(fe_abs_sf(u), fe_abs_sf(s.du)) || (s.kr + 1u) * iu.rn_len >= iu.orbit_len) {
                                s.du = u;
                                s.kr = 0u;
                                Uk = rn_ref_u(0u);
                            }
                            applied = true;
                            break;
                        }
                    }
                }
                if (l == 0u) { break; }
                l = l - 1u;
                off = off - (((kk - 1u) >> l) + 1u);
            }
            if (applied) { continue; }
        }
        let U = Uk;
        s.ddu = fe_add(fe_two(fe_mul(fe_add(U, s.du), s.ddu)), fe_one());
        if (rn_tail_ok(U, s.du)) {
            // The main loop's df32 tail (TAIL_DF32_MIN), for δu alone: the same operands in the same
            // order, an exact power-of-two scaling away from floatexp's, under the same guards
            // (`tail_step_ok`). The derivative, the rebase and the exit stay in floatexp.
            let dud = fe_to_cdf(s.du);
            s.du = fe_from_cdf(c_add(c_add(c_two(c_mul(fe_to_cdf(U), dud)), c_sqr(dud)), dcd));
        } else {
            s.du = fe_add(fe_add(fe_two(fe_mul(U, s.du)), fe_sqr(s.du)), dcp);
        }
        s.kr = s.kr + 1u;
        s.k = s.k + 1u;
        Uk = rn_ref_u(s.kr);
        let u = fe_add(Uk, s.du);
        // Rebase first (u is unchanged: U_0 = 0), so a pixel never leaves on the reference's last
        // sample — the main loop has no state there; it wraps before reaching it.
        if (sf_lt(fe_abs_sf(u), fe_abs_sf(s.du)) || (s.kr + 1u) * iu.rn_len >= iu.orbit_len) {
            s.du = u;
            s.kr = 0u;
            Uk = rn_ref_u(0u);
        }
        if (fe_mag2(u) > iu.rn_r2) {
            s.out = true;
            break;
        }
    }
    return s;
}
// Back to perturbation at iteration k·rn_len on Z_{kr·rn_len}: δz = δu/a, and δz/δc = (du/dc′)·b.
fn rn_dz(du: Fe) -> Fe {
    return fe_mul(fe_norm(cset(iu.rn_inv_a.xy, iu.rn_inv_a.zw), iu.rn_inv_a_exp), du);
}
fn rn_d(ddu: Fe) -> Fe {
    return fe_mul(ddu, fe_norm(cset(iu.rn_b.xy, iu.rn_b.zw), iu.rn_b_exp));
}
fn rn_k_max() -> u32 {
    return (iu.max_iter + iu.rn_len - 1u) / iu.rn_len;
}
// One pixel that took the step, on the step-accounting grid (see `CTR_RENORM`).
fn rn_commit(gx: f32, gy: f32) {
    if ((u32(gx) & 7u) != 0u || (u32(gy) & 7u) != 0u) { return; }
    atomicAdd(&counters[CTR_RENORM], 1u);
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

// One escaped pixel of THIS FRAME (see CTR_ESC_COUNT). Called once per escaped fragment on the
// single-pass iterate and once per escaped pixel per pass in `fs_resolve` — never at the chunk
// entry points' settle transition, which would double-count against the resolve.
//
// ⭐**SUBSAMPLED, ON THE SAME 4×4 GRID AS `esc_range_commit`** — the first version was not, which
// is the shape this file warns about twice: `esc_range_commit` is subsampled because "two atomics
// on two slots from ~1.6M threads is real contention", and the gradient histogram skips its
// bucket 0 because "nearly all samples would land on that ONE address — a same-address atomic
// serialized across the whole frame". Un-subsampled, this counter is exactly that: one address
// taking an increment from every escaped fragment, 8.68 MILLION of them on a 1676×1295 ss=2
// exterior-heavy frame.
//
// ⚠⚠**BUT IT WAS MEASURED, AND IT IS NOT EXPENSIVE — do not cite this as a performance fix.** It
// was suspected after a device loss on an RX 6800 XT (2026-09-21, a rebase storm at 1.76e6× where
// full-frame dispatches already ran 200–1000 ms). Interleaved A/B on the dev RTX 3080 at that
// exact view and sample count, 5 reps, medians: **5685 ms vs 5685 ms wall, max_dispatch 60 vs
// 59.5 ms** — a difference of 0, inside a run-to-run spread of ±150 ms. The GPU aggregates
// same-address atomics well enough that the un-subsampled form cost nothing here. It is
// subsampled anyway because the sample costs the consumer nothing (see below) and the pattern is
// one this file already refuses elsewhere; the device loss has a different cause.
// Output is unaffected either way: pixel-identical over all 8,681,680 pixels.
//
// ⭐The consumer does not lose anything. `render::content_has_detail` asks two questions — is this
// frame EMPTY, and does it clear a 0.1% floor — and both survive a 1/16 sample: a frame with no
// escaped pixel has none on the grid either, and 0.1% of 8.68M is 8,680 pixels, ~542 of which land
// on the grid. The app scales by `CTR_ESC_SUBSAMPLE` to recover a whole-frame estimate.
const CTR_ESC_SUBSAMPLE: u32 = 16u; // the 4x4 grid below; keep in sync with lib.rs
fn esc_count_commit(p: vec2<i32>) {
    if ((p.x & 3) == 0 && (p.y & 3) == 0) {
        atomicAdd(&counters[CTR_ESC_COUNT], 1u);
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
    trap_in: f32,  // the inside's own trap's, when it has one (`trap_in_enabled`)
};
// ---- stripe average over the TAIL only ------------------------------------------------------
// At extreme depth every pixel's orbit shadows the reference for all but its last few hundred
// iterates (δ grows from the pixel spacing to O(1) in ~ln(zoom)/λ steps), and the stripe term is
// a function of z alone — so the FULL-orbit mean is the same color for the whole view (2.5e246×,
// field report 2026-09-16). The tail mode replaces the plain mean with an exponentially-weighted
// mean over the last `tail_len` iterates: `s += (term − s)·α`, α = 1/tail_len, bias-corrected by
// `1 − (1−α)^n` so a SHORT orbit reads exactly like the plain mean (no look change on shallow
// views). Across a BLA-skipped span the same recurrence with the span's constant mean term `m`
// has the closed form `s = m + (s − m)·(1−α)^span`, which is what the fold applies — the plain
// mean is recovered as α → 0. `sac_sum` holds the EMA and `sac_prev` the previous one, so the
// `frac` blend in `aux_pack` works unchanged.
fn stripe_tail_on() -> bool {
    return (iu.aux_on & 2u) != 0u && (iu.orbit_stats & STAT_STRIPE) != 0u;
}
fn stripe_tail_alpha() -> f32 {
    return 1.0 / f32(max(iu.aux_on >> 2u, 1u));
}
// (1 − α)^k without pow's precision cliff at large k: exp(k · ln(1 − α)).
fn tail_decay(alpha: f32, k: f32) -> f32 {
    return exp(k * log(1.0 - alpha));
}
fn aux_init(z0: vec2<f32>) -> Aux {
    var a: Aux;
    a.sac_sum = 0.0; a.sac_prev = 0.0;
    a.tia_sum = 0.0; a.tia_prev = 0.0;
    a.n = 0.0; a.prev_abs = length(z0); a.trap = 1.0e30; a.trap_in = 1.0e30;
    return a;
}
// Fold one orbit point z into the running statistics — only those the frame ACCUMULATES (the
// `orbit_stats` bits: stripe→aux.x, TIA→aux.y, trap→aux.z; the final angle, aux.w, is written at
// escape by aux_pack or aux_escape and accumulates nothing). Runs only under `aux_on` bit 0, which a
// frame carrying the final angle alone leaves off. Computing every statistic every iteration — atan2+sin AND pow AND a
// trap distance — when a frame reads one cost ~10-17× at depth (stripe at 1e30× carried a pointless
// per-iteration `pow`). The statistics touch disjoint fields (TIA alone reads and writes
// `prev_abs`), so each keeps its own exact accumulation order with or without the others: a lane's
// value does not depend on what else is accumulated.
// How far z is from the orbit trap: its shape (core `TrapType`) placed, turned and sized (coloring
// plan F4; `trap_c` .. `trap_on`), measured in the shape's coordinates and scaled back, so it stays a
// distance in the orbit's plane. Mirrors core `TrapPlace::dist`, whose f64 the BLA aggregates fold.
// `trap_on` 0: the unit shape exactly as before F4, with no transform.
// The trap shapes of F4 part 3 (line, square, hexagon, star, sine, S curve) are compiled in only for a
// frame that draws one (lib.rs `iter_constants`): present but unused, their code cost the point, cross
// and circle frames 2-3% (2026-11-24, as the u-space BLA's did, `RN_BLA`). With TRAP_EXTRA off the
// chain below folds to the one before them, so those frames run the code they ran. Off, such a shape
// would measure as the point: every pipeline that can draw one must be built with it on.
override TRAP_EXTRA: bool = false;
// The sinc and the square wave (user 2026-11-25), compiled in only with TRAP_EXTRA and only for a frame
// that draws one (core `TrapTier::Waves`): kept out of the TRAP_EXTRA pipelines so the six shapes there
// keep their speed (code added there for the F4.4 outline cost them 7-10% on the RX 6800 XT).
override TRAP_WAVES: bool = false;
// A drawing's outline (coloring plan F4, part 4), compiled in only with the switches above and only
// for a frame that draws one (core `TrapTier::Outline`): its cell lookup and segment loop, present in
// the TRAP_EXTRA pipelines, slowed the six shapes there 7-10% on the RX 6800 XT and timed out the
// RTX 3070's trap matrix (2026-11-25). Off, the test below folds away and every other pipeline runs
// the code it ran. (Not TRAP_OUTLINE: that is the outline shape's id, generated from core.)
override TRAP_DRAWING: bool = false;
fn trap_dist_of(zf: vec2<f32>, ty: u32, on: u32, c: vec2<f32>, t: vec2<f32>, size: f32, inv: f32, phase: f32, harm: u32) -> f32 {
    var w = zf;
    if (on != 0u) {
        let d = zf - c;
        w = vec2<f32>(t.x * d.x + t.y * d.y, t.x * d.y - t.y * d.x) * inv;
    }
    var s: f32;
    if (ty == TRAP_CROSS) { s = min(abs(w.x), abs(w.y)); }
    else if (ty == TRAP_CIRCLE) { s = abs(length(w) - 1.0); }
    else if (TRAP_DRAWING && ty == TRAP_OUTLINE) { s = trap_outline_dist(w); }
    else if (TRAP_EXTRA) { s = trap_extra_dist(w, ty, phase, harm); }
    else { s = length(w); }
    if (on != 0u) { s = s * size; }
    return s;
}
// The distance from w to shape `ty` at size 1, centered at the origin and unturned, for the shapes of
// F4 part 3 (and the point, the one other shape that reaches here): core's `TrapType::unit_dist` in f32
// (the BLA aggregates fold that one, so the two must agree). The shapes' dimensions are core's,
// generated (`TRAP_STAR_INNER`, `TRAP_SINE_*`, `TRAP_S_RADIUS`). Exact distances with no angle taken
// (reflections and dot products), so a frame that draws one carries few extra registers: the
// angle-based forms cost 3% (measured 2026-11-24). Core's `TrapType::unit_dist` has the same forms.
fn trap_extra_dist(w: vec2<f32>, ty: u32, phase: f32, harm: u32) -> f32 {
    if (ty == TRAP_LINE) { return abs(w.y); }
    if (ty == TRAP_SQUARE) {
        // Inscribed in the unit circle, sides along the axes: half-side 1/sqrt 2.
        let q = abs(w) - vec2<f32>(0.70710678, 0.70710678);
        return abs(length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0));
    }
    if (ty == TRAP_HEXAGON) {
        // Corners on the unit circle, one on the real axis: apothem cos 30 degrees.
        let k = vec2<f32>(-0.86602540, 0.5);
        var p = abs(w);
        p = p - 2.0 * min(dot(k, p), 0.0) * k;
        p = p - vec2<f32>(clamp(p.x, -0.5, 0.5), 0.86602540);
        return length(p);
    }
    if (ty == TRAP_STAR) {
        // Points on the unit circle, one up; inner corners TRAP_STAR_INNER out.
        let k1 = vec2<f32>(0.80901699, -0.58778525);
        let k2 = vec2<f32>(-0.80901699, -0.58778525);
        var p = vec2<f32>(abs(w.x), w.y);
        p = p - 2.0 * max(dot(k1, p), 0.0) * k1;
        p = p - 2.0 * max(dot(k2, p), 0.0) * k2;
        p = vec2<f32>(abs(p.x), p.y - 1.0);
        let ba = TRAP_STAR_INNER * vec2<f32>(0.58778525, 0.80901699) - vec2<f32>(0.0, 1.0);
        let h = clamp(dot(p, ba) / dot(ba, ba), 0.0, 1.0);
        return length(p - ba * h);
    }
    // The sine wave with its harmonics overlaid (user 2026-11-25): in the waves' pipelines only, so
    // the Extra pipelines keep the plain wave below exactly as they had it.
    if (TRAP_WAVES && ty == TRAP_SINE && harm > 1u) { return trap_sine_overlaid(w, phase, harm); }
    if (ty == TRAP_SINE) {
        // Shifted along itself by the phase (user 2026-11-25; core `TrapType::unit_dist_at`).
        let k = TRAP_SINE_FREQ * w.x + phase;
        let slope = TRAP_SINE_AMP * TRAP_SINE_FREQ * cos(k);
        return abs(w.y - TRAP_SINE_AMP * sin(k)) / sqrt(1.0 + slope * slope);
    }
    if (ty == TRAP_S_CURVE) {
        // The bottom bowl is the top one turned half a turn about the origin.
        return min(trap_s_bowl(w), trap_s_bowl(-w));
    }
    if (TRAP_WAVES) {
        if (ty == TRAP_SINC) { return trap_sinc(w); }
        if (ty == TRAP_SQUARE_WAVE) { return trap_square_wave(w, phase, harm); }
    }
    return length(w);
}
// How far w is from the curve whose height and slope at w.x are f and df, to first order (core
// `curve_dist`).
fn trap_curve_dist(w: vec2<f32>, f: f32, df: f32) -> f32 {
    return abs(w.y - f) / sqrt(1.0 + df * df);
}
// The sinc y = sin(v)/v, v = TRAP_SINC_FREQ * x (core `sinc`, the same forms): near the center its
// series, where the quotient's slope loses its digits to cancellation.
fn trap_sinc(w: vec2<f32>) -> f32 {
    let v = TRAP_SINC_FREQ * w.x;
    let v2 = v * v;
    var f: f32;
    var df: f32;
    if (abs(v) < 0.5) {
        f = 1.0 - v2 / 6.0 * (1.0 - v2 / 20.0 * (1.0 - v2 / 42.0 * (1.0 - v2 / 72.0)));
        df = -v / 3.0 * (1.0 - v2 / 10.0 * (1.0 - v2 / 28.0 * (1.0 - v2 / 54.0 * (1.0 - v2 / 88.0))));
    } else {
        f = sin(v) / v;
        df = (v * cos(v) - sin(v)) / v2;
    }
    return trap_curve_dist(w, f, TRAP_SINC_FREQ * df);
}
// The sine wave and its next harmonics at the same height, the nearest of the curves
// TRAP_SINE_AMP * sin(k u), k = 1 to `harm`, u = TRAP_SINE_FREQ * x + phase (core `sine_overlaid`, the
// same forms): each harmonic's angle the last one's turned by u.
fn trap_sine_overlaid(w: vec2<f32>, phase: f32, harm: u32) -> f32 {
    let u = TRAP_SINE_FREQ * w.x + phase;
    let s1 = sin(u);
    let c1 = cos(u);
    var s = s1;
    var c = c1;
    var best: f32 = 3.0e38;
    let n = min(harm, TRAP_HARMONICS_MAX);
    for (var k = 1u; k <= n; k = k + 1u) {
        best = min(best, trap_curve_dist(w, TRAP_SINE_AMP * s, TRAP_SINE_AMP * TRAP_SINE_FREQ * f32(k) * c));
        let next = s * c1 + c * s1;
        c = c * c1 - s * s1;
        s = next;
    }
    return best;
}
// The square wave: its Fourier series TRAP_SINE_AMP * (4/pi) * sum of sin((2k-1) u) / (2k-1) over the
// first `harm` odd harmonics, u = TRAP_SINE_FREQ * x + phase (core `square_wave`, the same sum), each
// harmonic's angle the last one's turned by 2u, so one sine and cosine serve them all.
fn trap_square_wave(w: vec2<f32>, phase: f32, harm: u32) -> f32 {
    let u = TRAP_SINE_FREQ * w.x + phase;
    let s1 = sin(u);
    let c1 = cos(u);
    let s2 = 2.0 * s1 * c1;
    let c2 = c1 * c1 - s1 * s1;
    var s = s1;
    var c = c1;
    var y: f32 = 0.0;
    var dy: f32 = 0.0;
    let n = min(harm, TRAP_HARMONICS_MAX);
    for (var k = 0u; k < n; k = k + 1u) {
        y = y + s / f32(2u * k + 1u);
        dy = dy + c;
        let next = s * c2 + c * s2;
        c = c * c2 - s * s2;
        s = next;
    }
    let h = TRAP_SINE_AMP * 1.2732395;
    return trap_curve_dist(w, h * y, h * TRAP_SINE_FREQ * dy);
}
// The distance from p to the segment s = (a, b) as (ax, ay, bx, by): core `outline::seg_dist` in f32.
fn outline_seg_dist(p: vec2<f32>, s: vec4<f32>) -> f32 {
    let ab = s.zw - s.xy;
    let ap = p - s.xy;
    let t = clamp(dot(ap, ab) / dot(ab, ab), 0.0, 1.0);
    return length(ap - ab * t);
}
// The distance from w to the view's outline (coloring plan F4, part 4): core `Outline::dist` in f32,
// through the same cells and lists. Inside the grid's square the exact distance to the nearest
// segment of w's cell list; outside it, the distance to the square plus the outline's from there.
// With no outline uploaded, the point's.
fn trap_outline_dist(w: vec2<f32>) -> f32 {
    if (iu.outline_on == 0u) { return length(w); }
    let q = clamp(w, vec2<f32>(-OUTLINE_DOMAIN), vec2<f32>(OUTLINE_DOMAIN));
    let cell_w = 2.0 * OUTLINE_DOMAIN / f32(OUTLINE_GRID);
    let c = min(vec2<u32>(max(floor((q + OUTLINE_DOMAIN) / cell_w), vec2<f32>(0.0))), vec2<u32>(OUTLINE_GRID - 1u));
    let ci = c.y * OUTLINE_GRID + c.x;
    let cv = reference[iu.outline_cells + ci / 2u];
    let se = select(cv.xy, cv.zw, (ci & 1u) == 1u);
    let start = u32(se.x);
    let count = u32(se.y);
    var d = 3.0e38;
    for (var k = 0u; k < count; k = k + 1u) {
        let j = start + k;
        let iv = reference[iu.outline_idx + j / 4u];
        let si = u32(iv[j % 4u]);
        d = min(d, outline_seg_dist(q, reference[iu.outline_segs + si]));
    }
    return length(w - q) + d;
}
// The S curve's top bowl: the arc of radius TRAP_S_RADIUS about (0, TRAP_S_RADIUS) from angle 0
// counterclockwise to 270 degrees, measured in its own frame (its bisector, 135 degrees, turned up;
// the folded half's end at 135 degrees from up).
fn trap_s_bowl(w: vec2<f32>) -> f32 {
    let r = TRAP_S_RADIUS;
    let p = w - vec2<f32>(0.0, r);
    let q = vec2<f32>(abs(0.70710678 * (p.x + p.y)), 0.70710678 * (p.y - p.x));
    // Past the bowl's end, (sin, cos) of 135 degrees times r, the end is nearest. ⛔Written with
    // plain numbers: a `let` of a constant vector read by component made wgpu's pipeline-constant
    // step call this function invalid at start-up (2026-11-24), which naga's own validation passed.
    if (q.y < -q.x) { return length(q - vec2<f32>(0.70710678 * r, -0.70710678 * r)); }
    // ⛔The arc's distance from the UNTURNED p: in f32 the turn by 0.70710678 does not keep the
    // length, and |q| - r left 3e-8 where the curve passes the origin, the floor of every orbit that
    // returns to zero (a minibrot's inside read it as its trap, its BLA from core's f64 as 0:
    // 70% of the 9.3e78 minibrot's pixels differed, 2026-11-24).
    return abs(length(p) - r);
}
fn trap_dist(zf: vec2<f32>) -> f32 {
    return trap_dist_of(zf, iu.trap_type, iu.trap_on, iu.trap_c, iu.trap_turn, iu.trap_size, iu.trap_inv, iu.trap_phase, iu.trap_harm);
}
// The inside's own trap's distance (`trap_in_*`), measured as `trap_dist` measures the trap's.
fn trap_in_dist(zf: vec2<f32>) -> f32 {
    return trap_dist_of(zf, iu.trap_in_type, iu.trap_in_on, iu.trap_in_c, iu.trap_in_turn, iu.trap_in_size, iu.trap_in_inv, iu.trap_in_phase, iu.trap_in_harm);
}
fn aux_step(a: ptr<function, Aux>, zf: vec2<f32>, cmag: f32, power_f: f32) {
    let st = iu.orbit_stats;
    if ((st & STAT_TRAP) != 0u) {
        // Orbit trap: nearest approach to its shape (`trap_dist`). With its tail on (F4 part 5) every
        // earlier hit's distance first grows by trap_tail_g, doubling over the tail's length, so a hit
        // the pixel's orbit shared with every other pixel's long ago loses to its own late ones; off,
        // trap_tail_g is 1 and the product exact: as before.
        (*a).trap = min((*a).trap * iu.trap_tail_g, trap_dist(zf));
        if (iu.trap_in_enabled != 0u) { (*a).trap_in = min((*a).trap_in * iu.trap_tail_g, trap_in_dist(zf)); }
    }
    if ((st & STAT_STRIPE) != 0u) {
        // Stripe average: smooth orbit average of a sinusoid of the argument.
        let term = 0.5 + 0.5 * sin(iu.stripe_freq * atan2(zf.y, zf.x));
        (*a).sac_prev = (*a).sac_sum;
        if (stripe_tail_on()) {
            (*a).sac_sum = (*a).sac_sum + (term - (*a).sac_sum) * stripe_tail_alpha();
        } else {
            (*a).sac_sum = (*a).sac_sum + term;
        }
    }
    if ((st & STAT_TRIANGLE) != 0u) {
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
    // STAT_ANGLE: nothing to accumulate — aux_pack writes the final angle.
    (*a).n = (*a).n + 1.0;
}
// Fold a run of `span` reference iterates into the statistics, from its precomputed aggregate
// `agg` = [span, trap_min, ΣTIA, Σstripe] (a BLA node's fourth vec4, built from the reference with
// z ≈ Z): what a BLA skip folds, and what the series seed folds for the prefix it skips — for the
// statistics the frame accumulates, as `aux_step` does. The caller sets `prev_abs` (|z| where the
// run lands).
fn aux_fold_run(a: ptr<function, Aux>, agg: vec4<f32>, span: u32, extra: vec4<f32>, exact: bool) {
    let st = iu.orbit_stats;
    if ((st & STAT_TRAP) != 0u) {
        if (iu.trap_tail_inv > 0.0) {
            // The trap's tail (F4 part 5): the hits so far age by the run's length, held at 2^126 as
            // core's `trap_tail_factor` (finite in f32; a hit that old loses to any other), and the
            // node's aged lanes (`.z`, the inside's `.w`) hold the run's own, each aged to its end.
            // The series prefix folds the same lanes: the trap's nearest approach is a minimum, which
            // the drift from the reference by the prefix's end does not bias as it does the stripe's
            // window.
            let f = exp2(min(f32(span) * iu.trap_tail_inv, 126.0));
            (*a).trap = min((*a).trap * f, extra.z);
            if (iu.trap_in_enabled != 0u) { (*a).trap_in = min((*a).trap_in * f, extra.w); }
        } else {
            (*a).trap = min((*a).trap, agg.y);
            if (iu.trap_in_enabled != 0u) { (*a).trap_in = min((*a).trap_in, extra.y); }
        }
    }
    if ((st & STAT_TRIANGLE) != 0u) { (*a).tia_sum = (*a).tia_sum + agg.z; }
    if ((st & STAT_STRIPE) != 0u) {
        if (stripe_tail_on()) {
            // Exponential tail window across the skipped span. The node's fifth vec4 holds the
            // window-weighted sum W = Σ (1−α)^(span−1−i)·tᵢ (`merge_stripe` in the Rust), so
            // `s ← (1−α)^span·s + α·W` is the recurrence run term by term. A skip took the span's
            // plain mean before, which weighs every term alike where the window weighs the last
            // ones most: 0.23 off every-iteration through BLA at 1e30× (tail 16).
            // `exact` (a BLA skip): the node's window lane W, which reproduces the recurrence. The
            // series prefix passes false and keeps the span's plain mean `m`: by the prefix's end each
            // pixel has drifted from the reference, and folding the reference's own last terms
            // exactly measured worse there (0.17 against 0.029 at the same view).
            let al = stripe_tail_alpha();
            let d = tail_decay(al, f32(span));
            (*a).sac_prev = (*a).sac_sum;
            if (exact) {
                (*a).sac_sum = (*a).sac_sum * d + al * extra.x;
            } else {
                let m = agg.w / max(f32(span), 1.0);
                (*a).sac_sum = m + ((*a).sac_sum - m) * d;
            }
        } else {
            (*a).sac_sum = (*a).sac_sum + agg.w;
        }
    }
    (*a).n = (*a).n + f32(span);
}
// Pack the accumulated statistics into the aux target. `frac` is the fractional part
// of the smooth iteration count (blends the last two averages for a continuous result).
fn aux_pack(a: Aux, frac: f32, zf: vec2<f32>) -> vec4<f32> {
    var sac_avg: f32;
    var sac_prev: f32;
    if (stripe_tail_on()) {
        // Bias-corrected EMA: divide by the weight actually accumulated, 1 − (1−α)^n.
        let alpha = stripe_tail_alpha();
        sac_avg = a.sac_sum / max(1.0 - tail_decay(alpha, max(a.n, 1.0)), 1.0e-6);
        sac_prev = a.sac_prev / max(1.0 - tail_decay(alpha, max(a.n - 1.0, 1.0)), 1.0e-6);
    } else {
        sac_avg = a.sac_sum / max(a.n, 1.0);
        sac_prev = a.sac_prev / max(a.n - 1.0, 1.0);
    }
    let stripe = mix(sac_prev, sac_avg, frac);
    let tn = max(a.n - 1.0, 1.0);
    let tia_avg = a.tia_sum / tn;
    let tia_prev = a.tia_prev / max(a.n - 2.0, 1.0);
    let tia = mix(tia_prev, tia_avg, frac);
    return vec4<f32>(stripe, tia, a.trap, final_angle(zf));
}
const AUX_NONE: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0e30, 0.0);

// The final z's angle in turns, 0..1: decomposition's value. One expression for every path that
// writes it (aux_pack, aux_escape, the chunked resolve), so they agree bit for bit on the same z.
fn final_angle(zf: vec2<f32>) -> f32 {
    return atan2(zf.y, zf.x) * 0.15915494 + 0.5;
}
// The aux target of an ESCAPED pixel (`frac`, where it sits in its band; see `band_pos`): the running
// statistics and the final angle when the frame accumulates them (`aux_on` bit 0); else the final
// angle alone when the frame carries it (`orbit_stats`' STAT_ANGLE, coloring plan B8: read off the
// final z, so the frame iterated as one without statistics, the series, BLA and the renormalized step
// included); else nothing.
fn aux_escape(a: Aux, frac: f32, zf: vec2<f32>) -> vec4<f32> {
    if ((iu.aux_on & 1u) == 1u) { return aux_pack(a, frac, zf); }
    return aux_angle(zf);
}
// The final angle alone, when the frame carries it (else nothing): what an escaped pixel of a frame
// without running statistics gets, from the single-pass loops (`aux_escape`) and the chunked
// resolve alike.
fn aux_angle(zf: vec2<f32>) -> vec4<f32> {
    if ((iu.orbit_stats & STAT_ANGLE) != 0u) { return vec4<f32>(0.0, 0.0, 1.0e30, final_angle(zf)); }
    return AUX_NONE;
}
// The aux target of a pixel that reached the iteration limit: the running statistics when the frame
// accumulates them (the trap colors the inside), else nothing. Not the final angle alone: the color
// pass colors an interior texel by the interior mode, never by its angle, and the chunked resolve
// writes none there either, so the two paths agree.
fn aux_interior(a: Aux, zf: vec2<f32>) -> vec4<f32> {
    if ((iu.aux_on & 1u) == 1u) {
        // The inside's own trap, when it has one: an interior pixel's lane is its.
        var b = a;
        if (iu.trap_in_enabled != 0u) { b.trap = a.trap_in; }
        return aux_pack(b, 0.0, zf);
    }
    return AUX_NONE;
}

// Where an escaped pixel sits inside its iteration band, 0..1: the weight `aux_pack` blends the
// stripe and triangle-inequality averages over N and N−1 terms with (Härkönen's d),
// 1 − log_p(log|z_N| / log R). 1 when |z_N| has just passed the bailout R, 0 when it passed it by a
// whole power (|z_N| = R^p), so the blend meets itself across every band edge.
// ⛔Not `fract(smit)`: smit = N + 1 − log_p(log2|z_N|) carries the constant log_p(log2 R), a whole
// number only for p = 2 with R = 256, so every other power wrapped the weight inside the band and
// started it off zero at the edge: a ring along every band of Multibrot 3–8 and the folds at powers
// 3–5 (coloring plan A5b, 2026-11-08). At p = 2 the two agree except where |z_N| overshoots R^p,
// a sliver where `fract` wrapped and this clamps.
fn band_pos(mag2: f32, power_f: f32, bail2: f32) -> f32 {
    return clamp(1.0 - log(log(mag2) / log(bail2)) / max(log(power_f), 1.0e-3), 0.0, 1.0);
}

// Pauldelbrot glitch: a perturbed pixel is unreliable when its full value dips far below the
// reference (|z_n|² < GLITCH_TOL2·|Z_n|²) — the low-precision δz can't hold the cancellation.
// Flagged (only when iu.glitch_on) with a distinct sentinel in main.r for multi-reference
// correction: r = -1 interior, r ≥ 0 escaped, r = -2 glitched.
const GLITCH_TOL2: f32 = 1.0e-4;
const GLITCH_SENTINEL: vec4<f32> = vec4<f32>(-2.0, 0.0, 0.0, 1.0e30);

// ---------------- the iteration kernel ----------------
// Factored out of `fs_iterate` so a SECOND entry point can drive the identical arithmetic with a
// coordinate that does NOT come from the rasterizer - see `fs_iterate_gather`, which iterates a
// scattered LIST of pixels for multi-reference glitch correction.
// `gx`/`gy` are GLOBAL texel coordinates in the full iteration grid at the texel CENTER
// (integer + 0.5), which is exactly what `iu.px_offset + in.pos` yields in a tiled fragment pass -
// so from here down both entry points evaluate the same expressions on the same inputs. Both the
// integer part and the +0.5 are exact in f32 below 2^24, and so is their sum, so a coordinate
// rebuilt from an integer index reproduces the rasterized one bit-for-bit.
// !! Keep this body free of fragment-only operations (dpdx/dpdy/fwidth, discard, implicit-LOD
// sampling): the gather entry point's own fragment position is NOT the pixel being computed.
// !! And it must never read `gather_coords`. naga's global-use analysis is TRANSITIVE through
// callees, so a reference here would make `fs_iterate` require @group(1) @binding(4) too - and
// every one of its pipelines is built against a group-0-only layout, the live view included.
fn iterate_at(gx: f32, gy: f32) -> FragOut {
    // Pixel offset from the view center, from the *exact integer* texel coordinate
    // (the texel center = integer + 0.5, exact in f32 up to 2^24) times
    // the df32 per-texel step.
    // @@STRIP_AT_BEGIN — a strip's module places the texel on its polar grid here (`strip.rs`).
    let step_re = iu.step.xy;
    let step_im = iu.step.zw;
    let coord_re = gx - iu.res.x * 0.5;
    let coord_im = iu.res.y * 0.5 - gy;
    let off_re = df_mul_f32(step_re, coord_re);
    let off_im = df_mul_f32(step_im, coord_im);
    // @@STRIP_AT_END

    let bail2 = bail2_of(iu.formula);
    var iter: u32 = 0u;  // true iteration count
    var zf = vec2<f32>(0.0, 0.0);
    var escaped = false;
    // Event-counter tallies (committed once per fragment via ctr_commit).
    var n_rebase: u32 = 0u;
    var n_ext: u32 = 0u;
    var n_bla: u32 = 0u;
    // Step accounting (`step_commit`): full steps, and mode-2 full steps whose δz df32 could hold.
    var n_full: u32 = 0u;
    var n_big: u32 = 0u;

    // @@SPLIT_DIRECT_BEGIN — mode 1, which the floatexp half of a split iterate pipeline is cut without (`iter_split.rs`);
    // the other half is cut without mode 2 (`@@CUSTOM_FE`).
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
        // @@CUSTOM_INIT_BEGIN — a custom formula's init section, run once from z₀ (`custom.rs`).
        // @@CUSTOM_INIT_END
        // The exact-repeat check (`cycle_check_on`): z as last saved, and when it is next replaced.
        let cyc_on = cycle_check_on(iu.formula);
        var cyc_z = z;
        var cyc_k: u32 = 0u;
        var cyc_at: u32 = 1u;
        // The honest step past the critical point (`honest_step`): the power, 0 = never, and the radius.
        let hon_d = honest_power(iu.formula);
        let hon_r2 = honest_r2(hon_d, c);
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
                // @@CUSTOM_STEP_BEGIN — a custom formula's module replaces everything from here to
                // the matching END marker with its generated step (`custom.rs`). Comments only here.
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
                } else if (is_fam(iu.formula)) {
                    zn = fam_direct(iu.formula, z);
                    power_f = f32(fam_power(iu.formula));
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
                } else if (fam_holo(iu.formula)) {
                    // Multibrot d: D' = d·z^(d−1)·D (+1 in Mandelbrot mode), in df32 as Multibrot
                    // 3–5's. (Plain f32 here was measured no faster: 2.71× Mandelbrot either way.)
                    let d = fam_power(iu.formula);
                    dz = c_mul(c_scale(fam_pow_df(z, d - 1u), f32(d)), dz);
                    if (iu.julia == 0u) { dz = c_add(dz, one); }
                }
                zn = c_add(zn, c);
                if (hon_d != 0u && honest_step(z, hon_r2)) { zn = mb_step_h(z, hon_d, c); }
                // @@CUSTOM_STEP_END
                zprev = z;
                z = zn;
                iter = iter + 1u;
                zf = vec2<f32>(z.re.x, z.im.x);
                if ((iu.aux_on & 1u) == 1u) { aux_step(&aux, zf, cmag, power_f); }
                // @@CUSTOM_BAILOUT_BEGIN — a custom formula's own bailout replaces this test (`custom.rs`).
                if (dot(zf, zf) > bail2) { escaped = true; break; }
                // @@CUSTOM_BAILOUT_END
                if (cyc_on) {
                    // z repeated bit for bit: this arithmetic cycles from here on and the pixel can
                    // never escape, so it ends now as the interior it would be at `max_iter`.
                    cyc_k = cyc_k + 1u;
                    if (cdf_bits_eq(z, cyc_z)) { break; }
                    if (cyc_k == cyc_at) { cyc_z = z; cyc_at = cyc_at * 2u; }
                }
            }
        }
        if (!escaped) {
            atomicAdd(&counters[CTR_MAXITER], 1u);
            trap_hist_commit(aux, gx, gy, true);
            let aux_out = aux_interior(aux, zf);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        if (newton) {
            // Color by convergence speed (iteration count).
            esc_range_commit(f32(iter));
            esc_count_commit(vec2<i32>(i32(gx), i32(gy)));
            norm_hist_commit(f32(iter), vec2<i32>(i32(gx), i32(gy)));
            return FragOut(vec4<f32>(f32(iter), 0.0, 0.0, 1.0e30), AUX_NONE);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        // @@CUSTOM_SMOOTH_BEGIN — a custom formula's module clamps this line at 0 (`custom.rs`).
        let smit = f32(iter) + 1.0 - nu;
        // @@CUSTOM_SMOOTH_END
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u || iu.formula == 8u || fam_holo(iu.formula)) {
            nrm = slope_normal(zf, vec2<f32>(dz.re.x, dz.im.x));
            de = de_log2(mag2, vec2<f32>(dz.re.x, dz.im.x), 0.0);
        }
        trap_hist_commit(aux, gx, gy, false);
        let aux_out = aux_escape(aux, band_pos(mag2, power_f, bail2_of(iu.formula)), zf);
        esc_range_commit(smit);
        esc_count_commit(vec2<i32>(i32(gx), i32(gy)));
        norm_hist_commit(smit, vec2<i32>(i32(gx), i32(gy)));
        return FragOut(vec4<f32>(smit, nrm.x, nrm.y, de), aux_out);
    }
    // @@SPLIT_DIRECT_END
    // @@CUSTOM_FE_BEGIN — a custom formula's module replaces mode 2 from here to the matching END
    // marker with its own, leaner floatexp loop (`custom.rs`): no BLA, SA, glitch detection or
    // derivative, which are most of this pipeline's compile time.
    else if (iu.mode == 2u) {
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
        if (iu.bla_on == 1u && bla_formula(iu.formula) && iu.orbit_len > 1u) {
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
        // perturbation steps. The `z^d + c` families only (the CPU gates this: the formula's
        // `series_approximation`, no Julia). A statistics coloring method reaches it in one case
        // only, a BLA view whose reference escapes early (`sa_short_escaper`), and gets the skipped
        // prefix's statistics folded in below.
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
            // ⭐THE PREFIX'S STATISTICS (coloring plan A5a, 2026-11-08). The seed jumps `sa_skip`
            // iterations whose stripe, TIA and trap terms the loop never sees: a stripe or trap
            // export at 1e30× (skip 3,088 of a 3,090-sample reference) colored from its last few
            // iterations instead of the whole orbit, 75% of the channels off. Fold [0, sa_skip) out
            // of the BLA tree as a BLA skip folds its run: the binary decomposition of `sa_skip`
            // into aligned nodes, highest level first (each level once, so `p` stays aligned), with
            // z ≈ Z over the prefix, the approximation the series itself makes. The rule that
            // brings a statistics method here needs a BLA view, so the tree is there.
            // ⚠Stripe's TAIL window is approximate here, and no finer fold fixes it: the window
            // weighs the last iterations before the seed, where each pixel has already drifted from
            // the reference the aggregates were built from. Measured at 1e30× (tail 16): 0.029 off
            // every-iteration with this fold (the span's plain mean), 0.17 folding the reference's
            // terms exactly (the window lane, as a BLA skip does since 2026-11-15, which took BLA
            // alone from 0.23 off to 0.0000: a skip stays inside its radius, the prefix does not).
            if ((iu.aux_on & 1u) == 1u && bla_levels > 0u) {
                var p = 0u;
                var l = bla_levels;
                loop {
                    if (l == 0u || p >= iu.sa_skip) { break; }
                    l = l - 1u;
                    let stepn = 1u << l;
                    if (p + stepn > iu.sa_skip) { continue; }
                    let j = p >> l;
                    if (j >= bla_len[l]) { continue; }
                    let agg = reference[iu.orbit_len + (bla_off[l] + j) * BLA_NODE + 3u];
                    let span = u32(agg.x);
                    if (span == 0u) { break; } // a corrupt node: fold no further, never spin
                    let extra = reference[iu.orbit_len + (bla_off[l] + j) * BLA_NODE + 4u];
                    aux_fold_run(&aux, agg, span, extra, false); // the tail's plain-mean fold: see above
                    p = p + span;
                }
                let zs = orbit_cdf(reference[iu.sa_skip]);
                let dzs = fe_lo_f32(dz);
                aux.prev_abs = length(vec2<f32>(zs.re.x + dzs.x, zs.im.x + dzs.y));
            }
            // A PERIODIC reference may hand a skip of exactly its period (`orbit_len − 1`): the
            // pixel then starts on the last sample, Z_period ≈ 0, and wraps before its first step —
            // the same exact wrap the end-of-orbit rebase makes (see `OrbitPeriod`).
            if (ref_n + 1u >= iu.orbit_len) {
                dz = fe_sub(fe_add(orbit_fe(reference[ref_n]), dz), orbit_fe(reference[0]));
                ref_n = 0u;
            }
        }
        // ⭐THE RENORMALIZED STEP, from iteration 0 (see `rn_on`): whole periods as u ↦ u² + c′ until
        // the pixel leaves the model, then the loop below takes over from there; a pixel still inside
        // at the cap skips the loop and ends as a capped one.
        var n_rn: u32 = 0u;
        var rn_px = false;
        if (rn_on()) {
            let rc = rn_c(dc);
            if (rc.ok) {
                rn_px = true;
                let k_max = rn_k_max();
                var st: RnState;
                st.du = fe_zero();
                st.ddu = fe_zero();
                st.k = 0u;
                st.kr = 0u;
                st.out = false;
                if (rn_settled(rc.dcp)) {
                    st.k = k_max;
                } else {
                    st = rn_steps(rc.dcp, st, k_max, 0xffffffffu);
                    n_rn = st.trips;
                }
                if (st.out && st.k * iu.rn_len < iu.max_iter) {
                    dz = rn_dz(st.du);
                    D = rn_d(st.ddu);
                    iter = st.k * iu.rn_len;
                    ref_n = st.kr * iu.rn_len;
                } else {
                    iter = iu.max_iter;
                }
            }
        }
        // The df32 tail phase (TAIL_DF32_MIN): while `d_now`, δz lives in `dzd` and `dz` is stale.
        var dzd = cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0));
        var d_now = false;
        loop {
            if (iter >= iu.max_iter) { break; }
            // BLA works in floatexp: hand a df32 δz back first (exact at |δz| >= 2^-50), so every
            // skip decision is the one the all-floatexp loop would make.
            // In the df32 tail phase most steps cannot take a skip, and finding that out cost a floatexp
            // round trip and a search. No level's radius at ref_n exceeds level 0's (a merged radius is
            // at most its first child's), so a δz already past it rules every level out and the step
            // keeps its df32 δz. Exact either way: the round trip is a power-of-two scaling there, and
            // the tail test reads |δz| ≥ 2^-50 the same in both forms.
            var no_skip = false;
            if (d_now && bla_levels > 0u && ref_n < bla_len[0]) {
                let r0 = reference[iu.orbit_len + (bla_off[0] + ref_n) * BLA_NODE + 2u];
                no_skip = tail_no_skip(dzd, r0);
            }
            if (d_now && bla_levels > 0u && !no_skip) {
                dz = fe_from_cdf(dzd);
                d_now = false;
            }
            // BLA: skip 2^l reference steps at once while |δz| is within the merged validity
            // radius; revert to a lower level (ultimately a full step) on escape overshoot.
            // δz stays small in the BLA regime, so rebasing never triggers here.
            if (bla_levels > 0u && !no_skip) {
                let dzmag = fe_abs_sf(dz);
                var applied = false;
                var l = bla_levels;
                // Levels above ref_n's alignment would all `continue`: start at the highest it is aligned
                // to — the same levels in the same order, without walking the ~20 above it every step.
                if (ref_n != 0u) { l = min(l, countTrailingZeros(ref_n) + 1u); }
                loop {
                    if (l == 0u) { break; }
                    l = l - 1u;
                    let stepn = 1u << l;
                    if ((ref_n & (stepn - 1u)) != 0u) { continue; } // ref_n not aligned to 2^l
                    let j = ref_n >> l;
                    if (j >= bla_len[l]) { continue; }
                    let node = iu.orbit_len + (bla_off[l] + j) * BLA_NODE;
                    let v2 = reference[node + 2u];
                    let span = u32(reference[node + 3u].x);
                    // ⚠A ZERO span must never be applied. `iter += span` / `ref_n += span` is the
                    // only forward progress in this loop, so span == 0 makes the fragment spin
                    // until the driver's watchdog fires — a device loss with nothing in any log,
                    // which is the worst failure this codebase has. The bounds check below does NOT
                    // catch it (ref_n + 0 is still < orbit_len), and the CPU packer should never
                    // emit it — but a partially uploaded tree racing an orbit swap, or plain buffer
                    // corruption, would. One comparison converts "device lost" into "skip this
                    // node and iterate normally", i.e. at worst slightly slower pixels.
                    if (span == 0u) { continue; }
                    if (ref_n + span >= iu.orbit_len) { continue; } // keep reference[nref] valid
                    if (!sf_lt(dzmag, sf_norm(vec2<f32>(v2.w, 0.0), i32(v2.z)))) { continue; }
                    let v0 = reference[node];
                    let v1 = reference[node + 1u];
                    let A = fe_norm(cset(v0.xy, v0.zw), i32(v2.x));
                    let B = fe_norm(cset(v1.xy, v1.zw), i32(v2.y));
                    var ndz: Fe;
                    let fold = bla_fold(iu.formula);
                    if (fold) {
                        ndz = fe_add(fe_mat(v0, i32(v2.x), dz), fe_mat(v1, i32(v2.y), dc));
                    } else {
                        ndz = fe_add(fe_mul(A, dz), fe_mul(B, dc));
                    }
                    let nref = ref_n + span;
                    // orbit_cdf: the landing sample may be an extended-range dip (NaN-marked).
                    let rn = orbit_cdf(reference[nref]);
                    let ndzf = fe_lo_f32(ndz);
                    let zx = rn.re.x + ndzf.x;
                    let zy = rn.im.x + ndzf.y;
                    if (zx * zx + zy * zy > bail2) { continue; } // overshoot → drop a level
                    // (The folds carry no derivative: they have no distance estimate.)
                    if (bla_formula(iu.formula) && !fold) { D = fe_add(fe_mul(A, D), B); }
                    dz = ndz;
                    ref_n = nref;
                    iter = iter + span;
                    zf = vec2<f32>(zx, zy);
                    // Fold this node's precomputed aux aggregate over the `span` skipped iterates, so
                    // stripe/TIA/trap coloring stays correct across the skip instead of dropping the
                    // skipped run. `zf` is the actual landing value → prev_abs restores exactly.
                    if ((iu.aux_on & 1u) == 1u) {
                        // [span, trap_min, ΣTIA, Σstripe] and [window, 0, 0, 0]: inside a skip each pixel
                        // stays within the node's radius of the reference, so the window folds exactly.
                        aux_fold_run(&aux, reference[node + 3u], span, reference[node + 4u], true);
                        aux.prev_abs = length(zf);
                    }
                    n_bla = n_bla + 1u;
                    // Rebase at the BLA landing. The "δz stays small in the BLA regime ⇒ rebasing
                    // never triggers here" assumption (above) FAILS when the landing sample is a
                    // near-zero orbit dip: |Z_nref| ≈ 0 ⇒ |zfull| ≈ |δz|, so the Zhuoran condition
                    // |zfull| < |δz| can hold. Corpus 15's deep dendrite pixels must rebase exactly
                    // at these dips; without the check a valid skip lands past them and the pixel
                    // marches to the reference END, escaping prematurely there (~orbit_len) instead
                    // of deep — the dendrites vanish. Mirrors the full-step rebase below,
                    // INCLUDING its end-of-orbit arm: a skip may land on the LAST sample
                    // (`nref + span < orbit_len` allows it), and without the rebase there the next
                    // full step reads `reference[orbit_len]` — the first BLA node, not an orbit
                    // sample — into δz. Latent while every reference reaching its end was clamped
                    // (`partial`) or escaped; a PERIODIC reference (`OrbitPeriod`) wraps at its end
                    // every period, and every pixel then never escaped.
                    let zfe = fe_add(orbit_fe(reference[nref]), dz);
                    if (sf_lt(fe_abs_sf(zfe), fe_abs_sf(dz)) || nref + 1u >= iu.orbit_len) {
                        n_rebase = n_rebase + 1u;
                        dz = fe_sub(zfe, orbit_fe(reference[0]));
                        ref_n = 0u;
                    }
                    applied = true;
                    break;
                }
                if (applied) { continue; }
            }
            n_full = n_full + 1u;
            let r = reference[ref_n];
            let Z = orbit_cdf(r); // reference Z_n (df32; an extended dip reads (0,0) here)

            // ⭐THE df32 TAIL PHASE (see TAIL_DF32_MIN). Mirrored exactly in fs_iterate_chunk_fe.
            if (tail_step_ok(Z, dz, dzd, d_now)) {
                if (!d_now) {
                    dzd = fe_to_cdf(dz);
                    d_now = true;
                }
                n_big = n_big + 1u;
                // The derivative stays in floatexp, as mode 0's does; same operands as below.
                let zfn = vec2<f32>(Z.re.x + dzd.re.x, Z.im.x + dzd.im.x);
                let fp = deriv_factor(0u, zfn);
                D = fe_mul_c(D, fp.x, fp.y);
                if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
                // δz' = 2Z·δz + δz² + δc. |Z| and |δz| are both >= 2^-50 here (`tail_step_ok`;
                // an extended dip reads 0 and so never gets here), so every term stays in f32 range.
                dzd = c_add(c_add(c_two(c_mul(Z, dzd)), c_sqr(dzd)), tail_dc(dc));
                ref_n = ref_n + 1u;
                iter = iter + 1u;
                // Z_{n+1} + δz: an extended dip reads 0, under 2^-70 of δz (below df32's reach).
                let zfull_d = c_add(orbit_cdf(reference[ref_n]), dzd);
                zf = vec2<f32>(zfull_d.re.x, zfull_d.im.x);
                if ((iu.aux_on & 1u) == 1u) { aux_step(&aux, zf, cmag, power_f); }
                if (dot(zf, zf) > bail2) { escaped = true; break; }
                if (tail_rebase(zfull_d, dzd) || ref_n + 1u >= seg_end(ref_n)) {
                    n_rebase = n_rebase + 1u;
                    // Onto Z_0, or a Julia view's critical orbit (`rebase_crit_df`).
                    let d0 = c_sub(zfull_d, orbit_cdf(reference[0]));
                    if (rebase_crit_df(zfull_d, d0)) {
                        dzd = zfull_d;
                        ref_n = iu.crit_at;
                    } else {
                        dzd = d0;
                        ref_n = 0u;
                    }
                }
                continue;
            }
            if (d_now) {
                dz = fe_from_cdf(dzd);
                d_now = false;
            }

            // Derivative update D ← f'(z_n)·D (+1 Mandelbrot) using full z_n = Z_n + δz_n; in floatexp
            // at a dip f32 cannot hold (`deriv_dip`).
            if (iu.formula <= 3u) {
                let dzc = fe_lo_f32(dz);
                let zfn = vec2<f32>(Z.re.x + dzc.x, Z.im.x + dzc.y);
                let fp = deriv_factor(iu.formula, zfn);
                if (deriv_dip(r, fp)) {
                    D = fe_mul(D, fe_deriv_factor(iu.formula, fe_add(orbit_fe(r), dz)));
                } else {
                    D = fe_mul_c(D, fp.x, fp.y);
                }
                if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            } else if (iu.formula == 8u) {
                // Phoenix derivative: D' = 2·z_n·D + [1 if Mandelbrot] − 0.5·D_{n-1}.
                let dzc = fe_lo_f32(dz);
                let zfn = vec2<f32>(Z.re.x + dzc.x, Z.im.x + dzc.y);
                var dn: Fe;
                if (deriv_dip(r, 2.0 * zfn)) {
                    dn = fe_mul(D, fe_two(fe_add(orbit_fe(r), dz)));
                } else {
                    dn = fe_mul_c(D, 2.0 * zfn.x, 2.0 * zfn.y);
                }
                if (iu.julia == 0u) { dn = fe_add(dn, fe_one()); }
                dn = fe_sub(dn, fe_scale(Dprev, 0.5));
                Dprev = D;
                D = dn;
            } else if (fam_holo(iu.formula)) {
                let dzc = fe_lo_f32(dz);
                let fp = fam_deriv(iu.formula, vec2<f32>(Z.re.x + dzc.x, Z.im.x + dzc.y));
                if (deriv_dip(r, fp)) {
                    D = fe_mul(D, fe_deriv_factor(iu.formula, fe_add(orbit_fe(r), dz)));
                } else {
                    D = fe_mul_c(D, fp.x, fp.y);
                }
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
            } else if (is_fam(iu.formula)) {
                power_f = f32(fam_power(iu.formula));
                dz = fe_add(fam_pert_fe(iu.formula, Z, dz), dc);
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
            if ((iu.aux_on & 1u) == 1u) { aux_step(&aux, zf, cmag, power_f); }
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
                    step_commit(gx, gy, n_bla + n_full, n_full, n_big, iter);
                    return FragOut(GLITCH_SENTINEL, AUX_NONE);
                }
            }
            if (z2 > bail2) { escaped = true; break; }

            if (sf_lt(fe_abs_sf(zfull), fe_abs_sf(dz)) || ref_n + 1u >= seg_end(ref_n)) {
                n_rebase = n_rebase + 1u;
                // Rebase onto reference index 0: δz = (Z_{n+1} + δz) − Z₀. Z₀ = 0 for
                // Mandelbrot, the reference point for Julia (subtraction required) — or, for a
                // Julia view, onto its critical orbit (δz = z) when that leaves the smaller δz
                // (`rebase_crit_fe`).
                let d0 = fe_sub(zfull, orbit_fe(reference[0]));
                // Phoenix (two-term): also rebase δz_{n-1}. After rebasing to index 0 the "previous"
                // reference is Z_{-1}=0 (the orbit's initial z_prev), so δz_{n-1} → the full previous
                // value z_{n-1} = Z_{ref_n-1} + δz_{n-1}. Uses reference[ref_n-1] before ref_n resets.
                if (iu.formula == 8u) {
                    dz_prev = fe_add(orbit_fe(reference[ref_n - 1u]), dz_prev);
                }
                if (rebase_crit_fe(zfull, d0)) {
                    dz = zfull;
                    ref_n = iu.crit_at;
                } else {
                    dz = d0;
                    ref_n = 0u;
                }
            }
        }
        ctr_commit(n_rebase, n_ext, n_bla);
        step_commit(gx, gy, n_bla + n_full + n_rn, n_full, n_big, iter);
        if (rn_px) { rn_commit(gx, gy); }
        if (!escaped) {
            atomicAdd(&counters[CTR_MAXITER], 1u);
            trap_hist_commit(aux, gx, gy, true);
            let aux_out = aux_interior(aux, zf);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u || iu.formula == 8u || fam_holo(iu.formula)) {
            nrm = slope_normal(zf, vec2<f32>(D.m.re.x, D.m.im.x));
            de = de_log2(mag2, vec2<f32>(D.m.re.x, D.m.im.x), f32(D.e));
        }
        trap_hist_commit(aux, gx, gy, false);
        let aux_out = aux_escape(aux, band_pos(mag2, power_f, bail2_of(iu.formula)), zf);
        esc_range_commit(smit);
        esc_count_commit(vec2<i32>(i32(gx), i32(gy)));
        norm_hist_commit(smit, vec2<i32>(i32(gx), i32(gy)));
        return FragOut(vec4<f32>(smit, nrm.x, nrm.y, de), aux_out);
    }
    // @@CUSTOM_FE_END
    // @@SPLIT_PERT_BEGIN — mode 0, which the floatexp half of a split iterate pipeline is cut without (`iter_split.rs`).
    else {
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
            // A PERIODIC reference may hand a skip of exactly its period: wrap before the first
            // step (see the floatexp seeding above).
            if (ref_n + 1u >= iu.orbit_len) {
                dz = c_sub(c_add(orbit_cdf(reference[ref_n]), dz), orbit_cdf(reference[0]));
                ref_n = 0u;
            }
        }
        // The exact-repeat check (`cycle_check_on`) on this loop's state: the reference index and δz
        // as last saved, and when they are next replaced. The index it compares is `cyc_n`, `ref_n`
        // reduced into the reference's own cycle when it has one (`ref_fold` in lib.rs), else
        // `ref_n` itself: a pixel inside the reference's minibrot follows it, δz settles onto a cycle,
        // and `ref_n` only climbs, so without the fold its state never repeated.
        let cyc_on = cycle_check_on(iu.formula);
        var cyc_n = ref_n;
        if (iu.fold_by != 0u && cyc_n >= iu.fold_end) {
            let n0 = iu.fold_end - iu.fold_by;
            cyc_n = n0 + (cyc_n - n0) % iu.fold_by;
        }
        var cyc_dz = dz;
        var cyc_ref = cyc_n;
        var cyc_k: u32 = 0u;
        var cyc_at: u32 = 1u;
        loop {
            if (iter >= iu.max_iter) { break; }
            n_full = n_full + 1u; // mode 0 has no skip: every trip is one full step
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
            } else if (fam_holo(iu.formula)) {
                let fp = fam_deriv(iu.formula, vec2<f32>(z.re.x + dz.re.x, z.im.x + dz.im.x));
                D = fe_mul_c(D, fp.x, fp.y);
                if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            }
            // @@CUSTOM_PSTEP_BEGIN — a custom formula's module replaces the perturbed step from
            // here to the matching END marker with its generated one (`custom.rs`).
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
            } else if (is_fam(iu.formula)) {
                power_f = f32(fam_power(iu.formula));
                dz = c_add(fam_pert_df(iu.formula, z, dz), dc);
            } else if (iu.formula == 8u) {
                // Phoenix: δz' = 2Z·δz + δz² + δc − 0.5·δz_{n-1}
                let base = c_add(c_add(c_two(c_mul(z, dz)), c_sqr(dz)), dc);
                let dz_new = c_sub(base, c_scale(dz_prev, 0.5));
                dz_prev = dz;
                dz = dz_new;
            } else {
                dz = c_add(c_add(c_two(c_mul(z, dz)), c_sqr(dz)), dc);
            }
            // @@CUSTOM_PSTEP_END
            ref_n = ref_n + 1u;
            iter = iter + 1u;
            // orbit_cdf: an extended-range dip sample (NaN-marked) reads as (0,0) here.
            let rn = orbit_cdf(reference[ref_n]);
            let zr_full = df_add(rn.re, dz.re);
            let zi_full = df_add(rn.im, dz.im);
            zf = vec2<f32>(zr_full.x, zi_full.x);
            if ((iu.aux_on & 1u) == 1u) { aux_step(&aux, zf, cmag, power_f); }
            let z2 = dot(zf, zf);
            if (iu.glitch_on == 1u) {
                let zr2 = rn.re.x * rn.re.x + rn.im.x * rn.im.x;
                if (z2 < GLITCH_TOL2 * zr2) {
                    atomicAdd(&counters[CTR_GLITCH], 1u);
                    ctr_commit(n_rebase, n_ext, n_bla);
                    step_commit(gx, gy, n_bla + n_full, n_full, n_big, iter);
                    return FragOut(GLITCH_SENTINEL, AUX_NONE);
                }
            }
            if (z2 > bail2) { escaped = true; break; }
            let rebase_now = rebase_df32(z2, zf, dz);
            // @@CUSTOM_REBASE_BEGIN — a custom formula's module rebases PHASE-ALIGNED from here to
            // the matching END marker (a hybrid's reference index must stay ≡ iter mod phases).
            if (rebase_now || ref_n + 1u >= seg_end(ref_n)) {
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
                let d0 = cset(
                    df_sub(zr_full, r0.re),
                    df_sub(zi_full, r0.im),
                );
                // Onto Z_0, or a Julia view's critical orbit (`rebase_crit_df`).
                let zfull = cset(zr_full, zi_full);
                if (rebase_crit_df(zfull, d0)) {
                    dz = zfull;
                    ref_n = iu.crit_at;
                } else {
                    dz = d0;
                    ref_n = 0u;
                }
            }
            // @@CUSTOM_REBASE_END
            if (cyc_on) {
                // `cyc_n` follows `ref_n`: the index itself after a rebase (0, or a Julia view's
                // `crit_at` — the only ways `ref_n` is either here: both are segment starts it only
                // lands on by a rebase), else one on, folded back a period at `fold_end` (with no
                // fold, `fold_end` is 0 and the subtraction takes 0 from a rebased 0). The fold is
                // the reference's own, under `orbit_len <= crit_at`: a critical-orbit index never
                // meets it.
                let rebased = ref_n == 0u || (crit_on() && ref_n == iu.crit_at);
                cyc_n = select(cyc_n + 1u, ref_n, rebased);
                if (cyc_n == iu.fold_end) { cyc_n = cyc_n - iu.fold_by; }
                // The same index and δz, bit for bit: the next steps are the ones already taken
                // (escape and glitch tests included), so the pixel ends as the interior it would be
                // at `max_iter`.
                cyc_k = cyc_k + 1u;
                if (cyc_n == cyc_ref && cdf_bits_eq(dz, cyc_dz)) { break; }
                if (cyc_k == cyc_at) { cyc_dz = dz; cyc_ref = cyc_n; cyc_at = cyc_at * 2u; }
            }
        }
        ctr_commit(n_rebase, n_ext, n_bla);
        step_commit(gx, gy, n_bla + n_full, n_full, n_big, iter);
        if (!escaped) {
            atomicAdd(&counters[CTR_MAXITER], 1u);
            trap_hist_commit(aux, gx, gy, true);
            let aux_out = aux_interior(aux, zf);
            return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), aux_out);
        }
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        // @@CUSTOM_SMOOTH0_BEGIN — as the direct path's: a custom module clamps this line at 0.
        let smit = f32(iter) + 1.0 - nu;
        // @@CUSTOM_SMOOTH0_END
        var nrm = vec2<f32>(0.0, 0.0);
        var de = 1.0e30;
        if (iu.formula <= 3u || iu.formula == 8u || fam_holo(iu.formula)) {
            nrm = slope_normal(zf, vec2<f32>(D.m.re.x, D.m.im.x));
            de = de_log2(mag2, vec2<f32>(D.m.re.x, D.m.im.x), f32(D.e));
        }
        trap_hist_commit(aux, gx, gy, false);
        let aux_out = aux_escape(aux, band_pos(mag2, power_f, bail2_of(iu.formula)), zf);
        esc_range_commit(smit);
        esc_count_commit(vec2<i32>(i32(gx), i32(gy)));
        norm_hist_commit(smit, vec2<i32>(i32(gx), i32(gy)));
        return FragOut(vec4<f32>(smit, nrm.x, nrm.y, de), aux_out);
    }
    // @@SPLIT_PERT_END
}

@fragment
fn fs_iterate(in: VsOut) -> FragOut {
    ob_arm(); // keep two_prod's product opaque (see `ob`)
    // Global texel coordinate = this tile's offset + local fragment position.
    return iterate_at(iu.px_offset.x + in.pos.x, iu.px_offset.y + in.pos.y);
}

// Side of a split-refresh tile in texels. A GPU shades 2×2 quads packed into warps that cover a
// small screen-aligned block, so a split by ROWS would leave every warp half masked and each pass
// as slow as the whole frame; 16-texel tiles keep a warp's pixels in one pass, and are fine enough
// that each pass holds an even share of the picture's cost whatever its content.
const SPLIT_TILE: u32 = 16u;

// A SPLIT live refresh's geometry (`split_n` > 1, with `fs_iterate`): one quad per SPLIT_TILE tile,
// drawn instanced over the whole grid, and a tile outside set `split_j` collapses to a point, which
// rasterizes nothing. The pixels that do rasterize are `fs_iterate`'s own, bit for bit.
// ⛔NOT a `discard` in the fragment stage: it only DEMOTES the invocation to a helper, which (in the
// driver tested) went on through the whole iteration loop — a quarter-set pass measured as dear as
// the full frame. Geometry that never reaches the rasterizer costs nothing.
@vertex
fn vs_split_tiles(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let res = vec2<u32>(iu.res);
    let cols = (res.x + SPLIT_TILE - 1u) / SPLIT_TILE;
    let t = vec2<u32>(ii % cols, ii / cols);
    var out: VsOut;
    out.uv = vec2<f32>(0.0, 0.0);
    if ((t.x + t.y) % max(iu.split_n, 1u) != iu.split_j) {
        out.pos = vec4<f32>(-2.0, -2.0, 0.0, 1.0); // all six corners at one point: no area
        return out;
    }
    var corner = array<vec2<u32>, 6>(
        vec2<u32>(0u, 0u), vec2<u32>(1u, 0u), vec2<u32>(0u, 1u),
        vec2<u32>(0u, 1u), vec2<u32>(1u, 0u), vec2<u32>(1u, 1u),
    );
    // The tile's corner in texels, clamped to the texture on the last row and column.
    let p = vec2<f32>(min((t + corner[vi]) * SPLIT_TILE, res));
    let r = vec2<f32>(res);
    out.pos = vec4<f32>(p.x / r.x * 2.0 - 1.0, 1.0 - p.y / r.y * 2.0, 0.0, 1.0);
    out.uv = p / r;
    return out;
}

// ---------------- scattered-gather iterate (multi-reference glitch correction) ----------------
// Glitch correction wants "iterate exactly THESE N pixels", and N is a few hundred out of two
// million. Asking for whole frames is what made it the offline render's dominant cost: measured at
// 1.3e6x, passes 5..64 each re-iterated all 2,073,600 pixels to resolve one or two, and correction
// was 9.9 s of an 11.1 s render - about 4,000x the work it actually did. (Skipping tiles with no
// glitched pixel helped but is capped by tile granularity: scattered glitches touch 10 of 11 tiles.)
//
// So this entry point renders a TINY texture - ceil(sqrt(N)) wide - whose texel i takes its pixel
// coordinate from `gather_coords[i]` instead of from the rasterizer. The arithmetic below the
// coordinate is `iterate_at`, shared verbatim with `fs_iterate`, so a gathered pixel is
// bit-identical to the same pixel rendered in a full-frame pass: this is a pure cost removal with
// no quality trade and no policy decision. It also removes the readback, which dominated what the
// tile skip left behind (2 KB back instead of ~33 MB of allocate-copy-map per pass).
//
// Binding note: @group(1) bindings 0..3 are the chunk-state textures, which no gather pipeline
// binds and this entry point never reads. A pipeline layout may declare bindings its entry point
// does not use, but not the reverse, and each entry point is validated against its own layout - so
// a fifth, differently-typed binding in the same group is legal, and it keeps @group(0) (shared by
// every iterate pipeline, live view included) completely untouched.
@group(1) @binding(4) var<storage, read> gather_coords: array<vec2<u32>>;

@fragment
fn fs_iterate_gather(in: VsOut) -> FragOut {
    ob_arm(); // keep two_prod's product opaque (see `ob`)
    // in.pos is the texel center (integer + 0.5); truncation recovers the integer texel.
    let idx = u32(in.pos.y) * iu.gather_w + u32(in.pos.x);
    if (idx >= iu.gather_n) {
        // Padding texel in the last row - never read back. Returning before the loop keeps it free
        // AND keeps it out of the shared event counters, which a duplicated coordinate would inflate.
        return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), AUX_NONE);
    }
    let c = gather_coords[idx];
    // Rebuild the exact texel center the tiled pass would have rasterized for this pixel.
    return iterate_at(f32(c.x) + 0.5, f32(c.y) + 0.5);
}

// ---------------- iteration-range tiling (direct mode, resumable) ----------------
// Splits a pixel's 0..max_iter loop across several bounded dispatches so an arbitrarily high
// EXPLICIT iteration count can't run as one watchdog-tripping submission (zooming OUT from a deep
// ~4M-iter view to a shallow Direct one killed the device: no reference, no BLA skip, every in-set
// pixel grinds the full count — crash-1786499093). `fs_iterate_chunk` iterates only
// `[start_iter, min(end_iter, max_iter))` per dispatch, carrying per-pixel state between passes in
// three ping-pong textures; `fs_resolve` converts settled state into the normal iteration
// G-buffer that the color pass consumes. Scope: DIRECT mode (1) and DF32-PERTURBATION mode (0),
// holomorphic formulas 0..3 and 10..12, aux off (glitch detection IS supported — see ST_GLITCHED);
// everything else keeps single-pass `fs_iterate`,
// whose behavior is untouched. Mode 0 carries δz + the floatexp derivative + ref_n between
// passes, rebasing across chunk boundaries exactly as the single pass would.
//
// State layout (all Rgba32Float):
//   st_z    = (z.re.hi,  z.re.lo,  z.im.hi,  z.im.lo)     — running value, full df32
//   st_dz   = (dz.re.hi, dz.re.lo, dz.im.hi, dz.im.lo)    — derivative (DE/normal lighting)
//   st_meta = (iter_hi, iter_lo, status, smit)            — iteration count split into two small
//             integers (each exact in f32; a u32 bitcast could be flushed as a denormal by a
//             render target), settle status, and the smooth value computed AT escape.
// Counters: esc_range_commit / CTR_MAXITER fire exactly once per pixel, at the settle transition
// inside the chunk pass — so `fs_resolve` is pure and safe to run after every batch (progressive
// display). During progression a frame's escape-range only covers newly-escaped pixels; the
// normalize EMA absorbs that.
// (@group(1) @binding(4) is the gather pass's coordinate list - declared with `fs_iterate_gather`
// above, since nothing here reads it and nothing there reads these.)
@group(1) @binding(0) var st_z: texture_2d<f32>;
@group(1) @binding(1) var st_dz: texture_2d<f32>;
@group(1) @binding(2) var st_meta: texture_2d<f32>;
// Fourth state target, MODE-2 (floatexp) chunking only. Declared at module scope so both
// chunk entry points share one binding table; `fs_iterate_chunk` and `fs_resolve` never
// read it, so their 3-entry pipeline layouts stay valid (a layout may declare bindings an
// entry point does not use, but not the reverse).
@group(1) @binding(3) var st_exp: texture_2d<f32>;

struct ChunkOut {
    @location(0) z: vec4<f32>,
    @location(1) dz: vec4<f32>,
    // ⚠`meta` is a WGSL reserved word (like `smooth`) — hence `info`.
    @location(2) info: vec4<f32>,
};

// Mode-2 (floatexp) chunk output: the three above plus the derivative's floatexp EXPONENT.
// `dexp` rather than `exp` — same caution as `info` above, since `exp` is a builtin name.
struct ChunkOut4 {
    @location(0) z: vec4<f32>,
    @location(1) dz: vec4<f32>,
    @location(2) info: vec4<f32>,
    @location(3) dexp: vec4<f32>,
};

const ST_RUNNING: f32 = 0.0;
const ST_ESCAPED: f32 = 1.0;
const ST_INTERIOR: f32 = 2.0;
// Pauldelbrot-glitched (`glitch_on == 1` only): the pixel's low-precision δz lost the reference,
// so it is UNRELIABLE and the multi-reference corrector must re-render it against a nearer one.
// A settled status like the other two — the pass-through at the top of each chunk entry carries
// it to the end of the progression, and `fs_resolve` turns it into GLITCH_SENTINEL. ⚠The packing
// ceiling: `info_pack` stores `status·2^20 + (iter >> 12)`, so status ≤ 3 keeps the sum under
// 2^24 and therefore exact in an f32 render target. A fifth status would NOT be exact.
const ST_GLITCHED: f32 = 3.0;

// Unified info-channel layout (direct AND mode-0 perturbation):
//   ch0 = status·2^20 + (iter >> 12)   — both small integers, exact in f32
//   ch1 = iter & 4095
//   ch2 = smit when ESCAPED; ref_n while RUNNING (mode 0; direct stores 0)
//   ch3 = derivative EXPONENT (mode 0 carries D as floatexp; direct stores 0)
// iter is split so each part stays a small exact integer — a u32 bitcast could be flushed as a
// denormal by a render target. ref_n ≤ the 7.45M orbit cap and D.e are exact in f32 directly.
fn info_iter(m: vec4<f32>) -> u32 {
    let hi = u32(m.x) & 1048575u; // strip the status field
    return (hi << 12u) | u32(m.y);
}
fn info_status(m: vec4<f32>) -> f32 {
    return floor(m.x / 1048576.0);
}
fn info_pack(iter: u32, status: f32, ch2: f32, dexp: f32) -> vec4<f32> {
    return vec4<f32>(status * 1048576.0 + f32(iter >> 12u), f32(iter & 4095u), ch2, dexp);
}

@fragment
fn fs_iterate_chunk(in: VsOut) -> ChunkOut {
    ob_arm(); // keep two_prod's product opaque (see `ob`)
    // Pixel coordinate mapping — identical to fs_iterate's prologue.
    // @@STRIP_CHUNK_BEGIN
    let step_re = iu.step.xy;
    let step_im = iu.step.zw;
    let gx = iu.px_offset.x + in.pos.x;
    let gy = iu.px_offset.y + in.pos.y;
    let coord_re = gx - iu.res.x * 0.5;
    let coord_im = iu.res.y * 0.5 - gy;
    let off_re = df_mul_f32(step_re, coord_re);
    let off_im = df_mul_f32(step_im, coord_im);
    // @@STRIP_CHUNK_END

    let bail2 = bail2_of(iu.formula);
    let zero = vec2<f32>(0.0, 0.0);
    let one = cset(vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 0.0));
    let dsc = exp2(f32(iu.delta_exp));
    let stop = min(iu.end_iter, iu.max_iter);
    // Prior state, when resuming. The settled pass-through is common to both modes.
    let p = vec2<i32>(i32(in.pos.x), i32(in.pos.y));
    var sz = vec4<f32>(0.0);
    var sdz = vec4<f32>(0.0);
    var sm = vec4<f32>(0.0);
    if (iu.start_iter > 0u) {
        sz = textureLoad(st_z, p, 0);
        sdz = textureLoad(st_dz, p, 0);
        sm = textureLoad(st_meta, p, 0);
        // ⛔⛔STALE STATE: a RUNNING pixel always resumes at or past `start_iter` (it ran its last
        // window to `stop`, or starts at the series skip; in step-bounded passes `start_iter` is 1
        // and it took at least one step). Anything below that is a texel no pass wrote: the clear
        // (status RUNNING, iter 0) or an out-of-range read. A tile with an ODD width or height has
        // 2x2 quads straddling its scissor, and the GPU runs their outside lanes as HELPER
        // invocations, which read exactly such texels. Without this test they iterated from
        // iteration 0 to the window's end on EVERY resumed pass (their output discarded, their
        // counter atomics dropped, so nothing showed but the wall): 17x17 at 1e10 beside the cusp,
        // 243 ms a pass against 0 ms at 16x16 and 18x18; and at the neck 1e-6 (3.1 million
        // iterations) the passes grew past the 2 s watchdog - DEVICE LOST twice on the RTX 3080,
        // 2026-11-03 (scripts/pi_points.py). Shrinking the window cannot help: such a lane starts
        // from 0 whatever the window.
        if (info_status(sm) != ST_RUNNING || info_iter(sm) < iu.start_iter) {
            // Already settled (escaped or interior), or stale — pass state through unchanged.
            return ChunkOut(sz, sdz, sm);
        }
    }

    if (iu.mode == 1u) {
        // ---------------- direct (mode 1): z carried in full df32 ----------------
        // Same point derivation as fs_iterate's direct branch.
        let pr = df_add(vec2<f32>(iu.center.x, iu.center.z), df_mul_f32(off_re, dsc));
        let pi = df_add(vec2<f32>(iu.center.y, iu.center.w), df_mul_f32(off_im, dsc));
        var z: Cdf;
        var c: Cdf;
        if (iu.julia == 1u) {
            z = cset(pr, pi);
            c = cset(vec2<f32>(iu.julia_c.x, iu.julia_c.z), vec2<f32>(iu.julia_c.y, iu.julia_c.w));
        } else {
            z = cset(zero, zero);
            c = cset(pr, pi);
        }
        var dz: Cdf;
        if (iu.julia == 1u) { dz = one; } else { dz = cset(zero, zero); }
        var iter: u32 = 0u;
        var status: f32 = ST_RUNNING;
        var smit: f32 = 0.0;
        if (iu.start_iter > 0u) {
            z = cset(vec2<f32>(sz.x, sz.y), vec2<f32>(sz.z, sz.w));
            dz = cset(vec2<f32>(sdz.x, sdz.y), vec2<f32>(sdz.z, sdz.w));
            iter = info_iter(sm);
        }
        var power_f = 2.0;
        if (iu.formula == 1u) { power_f = 3.0; }
        else if (iu.formula == 2u) { power_f = 4.0; }
        else if (iu.formula == 3u) { power_f = 5.0; }
        else if (fam_holo(iu.formula)) { power_f = f32(fam_power(iu.formula)); }
        var zf = vec2<f32>(z.re.x, z.im.x);
        var escaped = false;
        // The exact-repeat check, as fs_iterate's direct branch (`cycle_check_on`), saved afresh
        // in each pass: a repeat inside one window ends the pixel as interior for every later one.
        let cyc_on = cycle_check_on(iu.formula);
        var cyc_z = z;
        var cyc_k: u32 = 0u;
        var cyc_at: u32 = 1u;
        var cycled = false;
        // The honest step, as fs_iterate's direct branch: resumed from the stored z, low word and all.
        let hon_d = honest_power(iu.formula);
        let hon_r2 = honest_r2(hon_d, c);
        loop {
            if (iter >= stop) { break; }
            // Formula step + derivative update — the exact arithmetic and ORDER of the direct
            // branch in fs_iterate (derivative uses the CURRENT z, before z advances), so a
            // chunked render is bit-identical to the single-pass one.
            // @@CUSTOM_CHUNK_STEP_BEGIN — a custom formula's module replaces the step from here to
            // the matching END marker; it carries z_{n-1} where the derivative is (no DE there).
            var zn: Cdf;
            if (iu.formula == 0u) {
                zn = c_sqr(z);
            } else if (iu.formula == 1u) {
                zn = c_mul(c_sqr(z), z);
            } else if (iu.formula == 2u) {
                zn = c_sqr(c_sqr(z));
            } else if (fam_holo(iu.formula)) {
                zn = fam_direct(iu.formula, z); // Multibrot 6–8, as fs_iterate's family arm
            } else {
                zn = c_mul(c_sqr(c_sqr(z)), z);
            }
            var fp: Cdf;
            if (iu.formula == 0u) {
                fp = c_two(z);
            } else if (iu.formula == 1u) {
                fp = c_scale(c_sqr(z), 3.0);
            } else if (iu.formula == 2u) {
                fp = c_scale(c_mul(c_sqr(z), z), 4.0);
            } else if (fam_holo(iu.formula)) {
                let d = fam_power(iu.formula);
                fp = c_scale(fam_pow_df(z, d - 1u), f32(d));
            } else {
                fp = c_scale(c_sqr(c_sqr(z)), 5.0);
            }
            dz = c_mul(fp, dz);
            if (iu.julia == 0u) { dz = c_add(dz, one); }
            zn = c_add(zn, c);
            if (hon_d != 0u && honest_step(z, hon_r2)) { zn = mb_step_h(z, hon_d, c); }
            z = zn;
            // @@CUSTOM_CHUNK_STEP_END
            iter = iter + 1u;
            zf = vec2<f32>(z.re.x, z.im.x);
            if (dot(zf, zf) > bail2) { escaped = true; break; }
            if (cyc_on) {
                cyc_k = cyc_k + 1u;
                if (cdf_bits_eq(z, cyc_z)) { cycled = true; break; }
                if (cyc_k == cyc_at) { cyc_z = z; cyc_at = cyc_at * 2u; }
            }
        }
        var smit_out = smit;
        if (escaped) {
            status = ST_ESCAPED;
            let mag2 = dot(zf, zf);
            let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
            // @@CUSTOM_CHUNK_SMOOTH_BEGIN — clamped at 0 in a custom module, as fs_iterate's.
            smit_out = f32(iter) + 1.0 - nu;
            // @@CUSTOM_CHUNK_SMOOTH_END
            esc_range_commit(smit_out);
        } else if (iter >= iu.max_iter || cycled) {
            // A repeat is the interior the budget would reach: settled now, counted as capped by
            // `fs_resolve` exactly as at `max_iter`.
            status = ST_INTERIOR;
        }
        return ChunkOut(
            vec4<f32>(z.re.x, z.re.y, z.im.x, z.im.y),
            vec4<f32>(dz.re.x, dz.re.y, dz.im.x, dz.im.y),
            info_pack(iter, status, smit_out, 0.0),
        );
    } else {
        // ---------------- df32 perturbation (mode 0): δz vs the reference orbit ----------------
        // The exact arithmetic and ORDER of fs_iterate's mode-0 branch (derivative first, using
        // Z[ref_n] + δz; then the formula δ-update; then advance and rebase-check), restricted to
        // the holomorphic formulas 0..3 and 10..12 with aux/glitch off — the app gates activation to that
        // scope. State: δz (df32) in st_z, the floatexp derivative's MANTISSA in st_dz and its
        // EXPONENT in info ch3, the reference position ref_n in info ch2 while running.
        let pert = cset(
            df_mul_f32(df_add(off_re, vec2<f32>(iu.ref_offset.x, iu.ref_offset.z)), dsc),
            df_mul_f32(df_add(off_im, vec2<f32>(iu.ref_offset.y, iu.ref_offset.w)), dsc),
        );
        var dz: Cdf;
        var dc: Cdf;
        if (iu.julia == 1u) {
            dz = pert;
            dc = cset(zero, zero);
        } else {
            dz = cset(zero, zero);
            dc = pert;
        }
        var D: Fe;
        if (iu.julia == 1u) { D = fe_one(); } else { D = fe_zero(); }
        var iter: u32 = 0u;
        var ref_n: u32 = 0u;
        var status: f32 = ST_RUNNING;
        var smit: f32 = 0.0;
        if (iu.start_iter > 0u) {
            dz = cset(vec2<f32>(sz.x, sz.y), vec2<f32>(sz.z, sz.w));
            D.m = cset(vec2<f32>(sdz.x, sdz.y), vec2<f32>(sdz.z, sdz.w));
            D.e = i32(sm.w);
            iter = info_iter(sm);
            ref_n = u32(sm.z);
        } else if (iu.sa_skip > 0u && iu.julia == 0u) {
            // Series-approximation seeding, exactly as the single-pass branch does it.
            let A = fe_norm(cset(iu.sa_a.xy, iu.sa_a.zw), iu.sa_a_exp);
            let B = fe_norm(cset(iu.sa_b.xy, iu.sa_b.zw), iu.sa_b_exp);
            let C = fe_norm(cset(iu.sa_c.xy, iu.sa_c.zw), iu.sa_c_exp);
            let dcf = fe_from_cdf(dc);
            let dc2 = fe_sqr(dcf);
            let dc3 = fe_mul(dc2, dcf);
            dz = fe_to_cdf(fe_add(fe_add(fe_mul(A, dcf), fe_mul(B, dc2)), fe_mul(C, dc3)));
            D = fe_add(fe_add(A, fe_scale(fe_mul(B, dcf), 2.0)), fe_scale(fe_mul(C, dc2), 3.0));
            iter = iu.sa_skip;
            ref_n = iu.sa_skip;
            // A PERIODIC reference may hand a skip of exactly its period: wrap before the first
            // step (see the floatexp seeding above).
            if (ref_n + 1u >= iu.orbit_len) {
                dz = c_sub(c_add(orbit_cdf(reference[ref_n]), dz), orbit_cdf(reference[0]));
                ref_n = 0u;
            }
        }
        // Step accounting: this pass's own advance (a first pass counts its SA seed as advanced).
        let iter0 = select(0u, iter, iu.start_iter > 0u);
        var n_full: u32 = 0u;
        var power_f = 2.0;
        if (iu.formula == 1u) { power_f = 3.0; }
        else if (iu.formula == 2u) { power_f = 4.0; }
        else if (iu.formula == 3u) { power_f = 5.0; }
        else if (fam_holo(iu.formula)) { power_f = f32(fam_power(iu.formula)); }
        var zf = vec2<f32>(0.0, 0.0);
        var z_full_re = vec2<f32>(0.0, 0.0); // full z, df32 — stored at escape for the resolve
        var z_full_im = vec2<f32>(0.0, 0.0);
        var escaped = false;
        var glitched = false;
        var n_rebase: u32 = 0u;
        loop {
            if (iter >= stop) { break; }
            n_full = n_full + 1u;
            let z = orbit_cdf(reference[ref_n]);
            // @@CUSTOM_CHUNK_PSTEP_BEGIN — a custom module replaces the derivative and the δ-update
            // from here to the matching END marker with its generated perturbed step.
            // Derivative update using the CURRENT full z (before the δ advances).
            let zfn = vec2<f32>(z.re.x + dz.re.x, z.im.x + dz.im.x);
            var fp: vec2<f32>;
            if (fam_holo(iu.formula)) { fp = fam_deriv(iu.formula, zfn); } else { fp = deriv_factor(iu.formula, zfn); }
            D = fe_mul_c(D, fp.x, fp.y);
            if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            // Formula δ-update (holomorphic families only — the app gates to `resumable_passes`:
            // Mandelbrot, Multibrot 3–8).
            if (fam_holo(iu.formula)) {
                dz = c_add(fam_pert_df(iu.formula, z, dz), dc); // as fs_iterate's family arm
            } else if (iu.formula == 1u) {
                let z2 = c_sqr(z); let dz2 = c_sqr(dz); let dz3 = c_mul(dz2, dz);
                var t = c_add(c_scale(c_mul(z2, dz), 3.0), c_scale(c_mul(z, dz2), 3.0));
                t = c_add(t, dz3); dz = c_add(t, dc);
            } else if (iu.formula == 2u) {
                let z2 = c_sqr(z); let z3 = c_mul(z2, z);
                let dz2 = c_sqr(dz); let dz3 = c_mul(dz2, dz); let dz4 = c_sqr(dz2);
                var t = c_scale(c_mul(z3, dz), 4.0);
                t = c_add(t, c_scale(c_mul(z2, dz2), 6.0));
                t = c_add(t, c_scale(c_mul(z, dz3), 4.0));
                t = c_add(t, dz4); dz = c_add(t, dc);
            } else if (iu.formula == 3u) {
                let z2 = c_sqr(z); let z3 = c_mul(z2, z); let z4 = c_sqr(z2);
                let dz2 = c_sqr(dz); let dz3 = c_mul(dz2, dz); let dz4 = c_sqr(dz2); let dz5 = c_mul(dz4, dz);
                var t = c_scale(c_mul(z4, dz), 5.0);
                t = c_add(t, c_scale(c_mul(z3, dz2), 10.0));
                t = c_add(t, c_scale(c_mul(z2, dz3), 10.0));
                t = c_add(t, c_scale(c_mul(z, dz4), 5.0));
                t = c_add(t, dz5); dz = c_add(t, dc);
            } else {
                dz = c_add(c_add(c_two(c_mul(z, dz)), c_sqr(dz)), dc);
            }
            // @@CUSTOM_CHUNK_PSTEP_END
            ref_n = ref_n + 1u;
            iter = iter + 1u;
            let rn = orbit_cdf(reference[ref_n]);
            z_full_re = df_add(rn.re, dz.re);
            z_full_im = df_add(rn.im, dz.im);
            zf = vec2<f32>(z_full_re.x, z_full_im.x);
            let z2m = dot(zf, zf);
            // Pauldelbrot glitch detection — the same test, operands and ORDER as fs_iterate's
            // mode-0 branch (glitch BEFORE escape; a pixel that satisfies both must land on the
            // same one either way, or a chunk boundary could change the answer).
            if (iu.glitch_on == 1u) {
                let zr2 = rn.re.x * rn.re.x + rn.im.x * rn.im.x;
                if (z2m < GLITCH_TOL2 * zr2) {
                    atomicAdd(&counters[CTR_GLITCH], 1u);
                    glitched = true;
                    break;
                }
            }
            if (z2m > bail2) { escaped = true; break; }
            // Zhuoran rebase: the δ overtook the reference (or the orbit ran out) — fold the
            // full value back onto the orbit start. The single-pass branch's own test, by the one
            // function both call (`rebase_df32`).
            let rebase_now = rebase_df32(z2m, zf, dz);
            // @@CUSTOM_CHUNK_REBASE_BEGIN — rebased PHASE-ALIGNED in a custom module, as fs_iterate's.
            if (rebase_now || ref_n + 1u >= seg_end(ref_n)) {
                n_rebase = n_rebase + 1u;
                let r0 = orbit_cdf(reference[0]);
                let d0 = cset(
                    df_sub(z_full_re, r0.re),
                    df_sub(z_full_im, r0.im),
                );
                // Onto Z_0, or a Julia view's critical orbit — fs_iterate's own decision.
                let zfull = cset(z_full_re, z_full_im);
                if (rebase_crit_df(zfull, d0)) {
                    dz = zfull;
                    ref_n = iu.crit_at;
                } else {
                    dz = d0;
                    ref_n = 0u;
                }
            }
            // @@CUSTOM_CHUNK_REBASE_END
        }
        ctr_commit(n_rebase, 0u, 0u);
        step_commit(gx, gy, n_full, n_full, 0u, iter - iter0);
        if (glitched) {
            // Settled as unreliable: the stored δz/derivative/ref_n are inert (the resolve reads
            // only the status), and every later chunk passes this through untouched.
            return ChunkOut(
                vec4<f32>(dz.re.x, dz.re.y, dz.im.x, dz.im.y),
                vec4<f32>(D.m.re.x, D.m.re.y, D.m.im.x, D.m.im.y),
                info_pack(iter, ST_GLITCHED, f32(ref_n), f32(D.e)),
            );
        }
        if (escaped) {
            status = ST_ESCAPED;
            let mag2 = dot(zf, zf);
            let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
            // @@CUSTOM_CHUNK_SMOOTH0_BEGIN — clamped at 0 in a custom module, as fs_iterate's.
            smit = f32(iter) + 1.0 - nu;
            // @@CUSTOM_CHUNK_SMOOTH0_END
            esc_range_commit(smit);
            // At escape the δ is no longer needed — store the FULL z so the resolve can shade
            // without knowing the reference, plus the derivative mantissa/exponent for DE.
            return ChunkOut(
                vec4<f32>(z_full_re.x, z_full_re.y, z_full_im.x, z_full_im.y),
                vec4<f32>(D.m.re.x, D.m.re.y, D.m.im.x, D.m.im.y),
                info_pack(iter, status, smit, f32(D.e)),
            );
        }
        if (iter >= iu.max_iter) {
            status = ST_INTERIOR;
        }
        return ChunkOut(
            vec4<f32>(dz.re.x, dz.re.y, dz.im.x, dz.im.y),
            vec4<f32>(D.m.re.x, D.m.re.y, D.m.im.x, D.m.im.y),
            info_pack(iter, status, f32(ref_n), f32(D.e)),
        );
    }
}

// ---------------------------------------------------------------------------
// Resumable MODE-2 (floatexp) chunk pass — FOUR state targets.
//
// Why a second entry point rather than widening `fs_iterate_chunk`: a shared four-target
// pipeline would make direct/mode-0 carry a fourth full-resolution Rgba32Float target they
// never read, and chunk state is ping-ponged — +265 MB at 4K, where the three existing targets
// already cost ~796 MB. The wasted write is only ~0.7 ms/settled frame, so VRAM is the whole
// argument. This module already has six fragment entries, so a seventh is the existing idiom;
// the state encoding stays in the shared `info_pack`/`info_iter`/`info_status` helpers.
//
// Why four targets at all: mode 2's running state is 13 floats against the 12 that three hold —
// δz mantissa (4) + δz exponent (1) + derivative mantissa (4) + derivative exponent (1) +
// status/iter (2) + ref_n (1). Mode 2 does not need to store full `z` while running (it is
// reconstructible as reference[ref_n] + δz), which frees a target — but the two floatexp
// exponents still cannot share one channel: an f32 holds integers exactly only to 2^24, so two
// packed fields get 12 bits each, and at 1e1105 the binary exponent is already ~ -3670.
//
// THE ESCAPE LAYOUT IS A CONTRACT WITH `fs_resolve`, NOT A FREE CHOICE. `fs_resolve` is
// deliberately mode-agnostic: at escape it reads the FULL z (df32) from st_z, the display
// derivative's MANTISSA from st_dz, smit from info ch2 and the derivative's floatexp EXPONENT
// from info ch3 (`sm.w`). So while RUNNING info ch3 carries the δz exponent and st_exp.ch0
// carries the derivative's; at ESCAPE info ch3 reverts to the derivative's exponent and st_exp
// goes unused. Parking the derivative exponent in st_exp unconditionally would leave resolve
// reading a δz exponent as the DE exponent and quietly corrupt distance-estimate shading and
// relief lighting on every chunked mode-2 escape. With the layout below, `fs_resolve` needs no
// change at all.
//
//   st_z    RUNNING δz mantissa (re.hi, re.lo, im.hi, im.lo)  | ESCAPE full z (df32)
//   st_dz   RUNNING derivative mantissa                       | ESCAPE derivative mantissa
//   st_meta RUNNING (status+iter_hi, iter_lo, ref_n, δz.e)    | ESCAPE ch2 smit, ch3 D.e
//   st_exp  RUNNING ch0 D.e, ch1..3 spare                     | ESCAPE unused
//
// Scope, inherited from the app's `chunk_over` gate exactly as mode 0's chunk body is: the
// HOLOMORPHIC formulas 0..3 and 10..12 only, aux coloring off. That is what keeps the 13-float budget
// honest — Phoenix (formula 8) additionally carries δz_{n-1} AND D_{n-1} (ten more values,
// which alone would blow four targets), and aux coloring carries a five-float orbit
// accumulator. Tricorn (4) and the abs families (5..7) are out of scope, so their δ-updates are
// deliberately not ported.
//
// ⭐Glitch detection IS in scope since beta.124: the sentinel needed no extra state after all,
// only a fourth SETTLED STATUS (ST_GLITCHED) in the existing status field, which `fs_resolve`
// maps back to GLITCH_SENTINEL. That closes the last latency-bound single-dispatch tile in the
// export family — the multi-reference corrector's base pass, which the 5K device loss
// (crash-1787292746) reached only because its 120 s deadline is checked BETWEEN tiles.
//
// Each pass rebuilds its own BLA level table from `iu.orbit_len` below, and the host must keep
// passing `bla_on` on chunk frames. The e100 device loss (beta.101) was glitch-correction passes
// running with `bla_on = 0` and an empty tree at 0.04 Gsteps/s against the base pass's 174 in the
// same frame — a chunk pass that silently lost its BLA would reproduce that exactly, and it would
// read as "chunking is slow" rather than "chunking is broken".
//
// A BLA skip can carry `iter` PAST `stop` in one cheap step (the whole point of a skip is that a
// span of 2^l costs one iteration's work, so this does not cost time) — the pass then simply
// stores an `iter` above the range it was asked for, and the next pass's `stop` is already behind
// it and passes the state through. Bit-identity is unaffected because the skip sequence is the
// same one the unchunked loop takes; but the host's `steps = px * delta_iter` cost model
// over-counts whenever BLA is live, which mode 0 never had to account for.
// The body of every floatexp chunk entry point (`fs_iterate_chunk_fe` and the export's packed pass
// below): the pixel at texel center `pos` of the tile, whose state is at texel `p` of the bound
// state set — the same texel in a whole-tile pass, its slot in a packed one.
// @@CUSTOM_CHUNK_FE_BEGIN — a custom formula's module replaces this whole function, to the
// matching END marker, with its own (`custom.rs`): same state layout, its generated step.
fn chunk_fe_at(pos: vec2<f32>, p: vec2<i32>) -> ChunkOut4 {
    chunk_slot = NO_SLOT;
    // Pixel coordinate mapping — identical to fs_iterate's prologue.
    // @@STRIP_FE_BEGIN
    let step_re = iu.step.xy;
    let step_im = iu.step.zw;
    let gx = iu.px_offset.x + pos.x;
    let gy = iu.px_offset.y + pos.y;
    let coord_re = gx - iu.res.x * 0.5;
    let coord_im = iu.res.y * 0.5 - gy;
    let off_re = df_mul_f32(step_re, coord_re);
    let off_im = df_mul_f32(step_im, coord_im);
    // @@STRIP_FE_END

    let bail2 = bail2_of(iu.formula);
    let stop = min(iu.end_iter, iu.max_iter);

    // Prior state, when resuming.
    var sz = vec4<f32>(0.0);
    var sdz = vec4<f32>(0.0);
    var sm = vec4<f32>(0.0);
    var se = vec4<f32>(0.0);
    if (iu.start_iter > 0u) {
        sz = textureLoad(st_z, p, 0);
        sdz = textureLoad(st_dz, p, 0);
        sm = textureLoad(st_meta, p, 0);
        se = textureLoad(st_exp, p, 0);
        // Stale state (a helper lane of an odd-sized tile): see `fs_iterate_chunk`. EXCEPT a pixel
        // paused inside the renormalized step: it stores iteration k·rn_len, which may lie below
        // `start_iter`, with `ref_n` = RN_PAUSED (-1) - never the 0 of a texel no pass wrote. (The
        // first version of this test failed selftest's "pixels paused inside it resume
        // bit-identically": 131,936 texels differed.)
        let stale = info_iter(sm) < iu.start_iter && sm.z != RN_PAUSED;
        if (info_status(sm) != ST_RUNNING || stale) {
            // Already settled (escaped or interior), or stale — pass state through unchanged.
            return ChunkOut4(sz, sdz, sm, se);
        }
    }

    // Pixel deviation as floatexp, exactly as fs_iterate's mode-2 branch derives it: `off_*` and
    // `ref_offset` are mantissas sharing the `delta_exp` exponent (no `dsc` rescale on this path).
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
    var D: Fe;
    if (iu.julia == 1u) { D = fe_one(); } else { D = fe_zero(); }
    var iter: u32 = 0u;
    var ref_n: u32 = 0u;
    var status: f32 = ST_RUNNING;
    var n_rebase: u32 = 0u;
    var n_ext: u32 = 0u;
    var n_bla: u32 = 0u;

    // BLA level layout, rebuilt per pass from the reference length — see the note above. Never a
    // fold's 2×2 tree: the folds are not resumable, and this pass reads a node as complex.
    var bla_off: array<u32, 32>;
    var bla_len: array<u32, 32>;
    var bla_levels = 0u;
    if (iu.bla_on == 1u && bla_formula(iu.formula) && !bla_fold(iu.formula) && iu.orbit_len > 1u) {
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

    if (iu.start_iter > 0u) {
        // `fe_make`, not `fe_norm`: the stored pair is already normalized, and re-normalizing a
        // restored value would make a pixel depend on where the frame happened to split.
        dz = fe_make(cset(vec2<f32>(sz.x, sz.y), vec2<f32>(sz.z, sz.w)), i32(sm.w));
        D = fe_make(cset(vec2<f32>(sdz.x, sdz.y), vec2<f32>(sdz.z, sdz.w)), i32(se.x));
        iter = info_iter(sm);
        ref_n = u32(sm.z);
    } else if (iu.sa_skip > 0u && iu.julia == 0u) {
        // Series-approximation seeding, exactly as the single-pass branch does it.
        let A = fe_norm(cset(iu.sa_a.xy, iu.sa_a.zw), iu.sa_a_exp);
        let B = fe_norm(cset(iu.sa_b.xy, iu.sa_b.zw), iu.sa_b_exp);
        let C = fe_norm(cset(iu.sa_c.xy, iu.sa_c.zw), iu.sa_c_exp);
        let dc2 = fe_sqr(dc);
        let dc3 = fe_mul(dc2, dc);
        dz = fe_add(fe_add(fe_mul(A, dc), fe_mul(B, dc2)), fe_mul(C, dc3));
        D = fe_add(fe_add(A, fe_scale(fe_mul(B, dc), 2.0)), fe_scale(fe_mul(C, dc2), 3.0));
        iter = iu.sa_skip;
        ref_n = iu.sa_skip;
        // A periodic reference's skip may be its period: wrap before the first step.
        if (ref_n + 1u >= iu.orbit_len) {
            dz = fe_sub(fe_add(orbit_fe(reference[ref_n]), dz), orbit_fe(reference[0]));
            ref_n = 0u;
        }
    }

    // ⭐THE RENORMALIZED STEP (see `rn_on` and fs_iterate's prologue). A pass may end inside it — a
    // step cap (each step charged 1/RN_STEP_WEIGHT) or an iteration window — and the pixel then
    // pauses with `ref_n` = RN_PAUSED: δu in the δz slot (its exponent in info ch3), du/dc′ in the
    // derivative slot, the u-reference index in st_exp ch2, iteration k·rn_len. δc′ is recomputed
    // from the pixel on resume, so where passes split never changes it.
    let iter_loaded = iter;
    var n_rn: u32 = 0u;
    var rn_px = false;
    if (rn_on()) {
        let resumed_in = iu.start_iter > 0u && sm.z == RN_PAUSED;
        if (iu.start_iter == 0u || resumed_in) {
            let rc = rn_c(dc);
            if (rc.ok) {
                rn_px = iu.start_iter == 0u;
                let k_max = rn_k_max();
                var st: RnState;
                st.du = fe_zero();
                st.ddu = fe_zero();
                st.k = 0u;
                st.kr = 0u;
                st.out = false;
                if (resumed_in) {
                    st.du = fe_make(cset(vec2<f32>(sz.x, sz.y), vec2<f32>(sz.z, sz.w)), i32(sm.w));
                    st.ddu = fe_make(cset(vec2<f32>(sdz.x, sdz.y), vec2<f32>(sdz.z, sdz.w)), i32(se.x));
                    st.k = iter / iu.rn_len;
                    st.kr = u32(se.z);
                } else if (rn_settled(rc.dcp)) {
                    st.k = k_max;
                }
                let k0 = st.k;
                // The window's last whole step, or the cap's own when the window reaches it.
                let k_end = select(stop / iu.rn_len, k_max, stop >= iu.max_iter);
                var trip_max = 0xffffffffu;
                if (iu.step_cap > 0u) { trip_max = iu.step_cap * RN_STEP_WEIGHT / rn_trip_weight(); }
                st = rn_steps(rc.dcp, st, max(k_end, k0), trip_max);
                n_rn = st.trips;
                if (st.out) {
                    if (st.k * iu.rn_len < iu.max_iter) {
                        dz = rn_dz(st.du);
                        D = rn_d(st.ddu);
                        iter = st.k * iu.rn_len;
                        ref_n = st.kr * iu.rn_len;
                    } else {
                        iter = iu.max_iter;
                    }
                } else if (st.k >= k_max) {
                    iter = iu.max_iter;
                } else {
                    // Paused inside: store it and leave this pass.
                    step_commit(gx, gy, n_rn, 0u, 0u, st.k * iu.rn_len - select(0u, iter_loaded, iu.start_iter > 0u));
                    if (rn_px) { rn_commit(gx, gy); }
                    if (iu.step_cap > 0u || iu.count_running == 1u) { chunk_slot = atomicAdd(&counters[CTR_CHUNK_RUNNING], 1u); }
                    return ChunkOut4(
                        vec4<f32>(st.du.m.re.x, st.du.m.re.y, st.du.m.im.x, st.du.m.im.y),
                        vec4<f32>(st.ddu.m.re.x, st.ddu.m.re.y, st.ddu.m.im.x, st.ddu.m.im.y),
                        info_pack(st.k * iu.rn_len, ST_RUNNING, RN_PAUSED, f32(st.du.e)),
                        vec4<f32>(f32(st.ddu.e), 0.0, f32(st.kr), 0.0),
                    );
                }
            }
        }
    }

    // Fixed up front from the formula (the single-pass loop assigns it inside each branch, which
    // is the same value for each formula in scope — but only if the loop body runs, and a resumed pass
    // may escape on its first iteration).
    var power_f = 2.0;
    if (iu.formula == 1u) { power_f = 3.0; }
    else if (iu.formula == 2u) { power_f = 4.0; }
    else if (iu.formula == 3u) { power_f = 5.0; }
    else if (fam_holo(iu.formula)) { power_f = f32(fam_power(iu.formula)); }

    var zf = vec2<f32>(0.0, 0.0);
    var z_full: Fe = fe_zero(); // full z = Z_{n+1} + δz, kept for the escape store
    var escaped = false;
    var glitched = false;
    // Step accounting: this pass's own advance (a first pass counts its SA seed as advanced).
    let iter0 = select(0u, iter_loaded, iu.start_iter > 0u);
    var n_full: u32 = 0u;
    var n_big: u32 = 0u;
    // The df32 tail phase (TAIL_DF32_MIN): while `d_now`, δz lives in `dzd` and `dz` is stale. A
    // pass always stores δz as floatexp (below), and the phase is a function of |δz| alone, so a
    // resumed pass re-enters it exactly where an unsplit walk would still be in it.
    var dzd = cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0));
    var d_now = false;
    loop {
        if (iter >= stop) { break; }
        // ⭐STEP-BOUNDED PASSES (beta.150): an iteration window bounds a mode-2 pass's NOMINAL work,
        // and with BLA skipping ~250 iterations a step that says little about its cost. Executed
        // steps are what a pass pays for, and one step's cost has a ceiling, so capping them bounds
        // the dispatch whatever the pixel does. A pass may stop between any two steps: the state
        // below round-trips exactly (the same property that lets a window end anywhere), so where
        // the passes split never changes a pixel.
        if (iu.step_cap > 0u && n_bla + n_full + n_rn * rn_trip_weight() / RN_STEP_WEIGHT >= iu.step_cap) { break; }
        // In the df32 tail phase most steps cannot take a skip, and finding that out cost a floatexp
        // round trip and a search. No level's radius at ref_n exceeds level 0's (a merged radius is
        // at most its first child's), so a δz already past it rules every level out and the step
        // keeps its df32 δz. Exact either way: the round trip is a power-of-two scaling there, and
        // the tail test reads |δz| ≥ 2^-50 the same in both forms.
        var no_skip = false;
        if (d_now && bla_levels > 0u && ref_n < bla_len[0]) {
            let r0 = reference[iu.orbit_len + (bla_off[0] + ref_n) * BLA_NODE + 2u];
            no_skip = tail_no_skip(dzd, r0);
        }
        if (d_now && bla_levels > 0u && !no_skip) {
            dz = fe_from_cdf(dzd);
            d_now = false;
        }
        // BLA: skip 2^l reference steps at once while |δz| is within the merged validity radius;
        // revert to a lower level (ultimately a full step) on escape overshoot. Verbatim from
        // fs_iterate's mode-2 branch minus the aux aggregate (aux is out of scope here).
        if (bla_levels > 0u && !no_skip) {
            let dzmag = fe_abs_sf(dz);
            var applied = false;
            var l = bla_levels;
            // Levels above ref_n's alignment would all `continue`: start at the highest it is aligned
            // to — the same levels in the same order, without walking the ~20 above it every step.
            if (ref_n != 0u) { l = min(l, countTrailingZeros(ref_n) + 1u); }
            loop {
                if (l == 0u) { break; }
                l = l - 1u;
                let stepn = 1u << l;
                if ((ref_n & (stepn - 1u)) != 0u) { continue; } // ref_n not aligned to 2^l
                let j = ref_n >> l;
                if (j >= bla_len[l]) { continue; }
                let node = iu.orbit_len + (bla_off[l] + j) * BLA_NODE;
                let v2 = reference[node + 2u];
                let span = u32(reference[node + 3u].x);
                // A ZERO span must never be applied — it is the only forward progress in this
                // loop, so span == 0 spins the fragment until the driver watchdog fires. See the
                // full note in fs_iterate.
                if (span == 0u) { continue; }
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
                D = fe_add(fe_mul(A, D), B); // a `z^d + c` family by `bla_levels` above
                dz = ndz;
                ref_n = nref;
                iter = iter + span;
                zf = vec2<f32>(zx, zy);
                n_bla = n_bla + 1u;
                // Rebase at the BLA landing: a near-zero orbit dip makes |z_full| ~ |δz|, so the
                // Zhuoran condition can hold even here. Mirrors the full-step rebase below — see
                // the corpus-15 dendrite note in fs_iterate — and, as there, at the orbit's
                // last sample too (a skip may land on it).
                let zfe = fe_add(orbit_fe(reference[nref]), dz);
                if (sf_lt(fe_abs_sf(zfe), fe_abs_sf(dz)) || nref + 1u >= iu.orbit_len) {
                    n_rebase = n_rebase + 1u;
                    dz = fe_sub(zfe, orbit_fe(reference[0]));
                    ref_n = 0u;
                }
                applied = true;
                break;
            }
            if (applied) { continue; }
        }
        n_full = n_full + 1u;
        let r = reference[ref_n];
        let Z = orbit_cdf(r); // reference Z_n (df32; an extended dip reads (0,0) here)

        // ⭐THE df32 TAIL PHASE — verbatim from fs_iterate's mode-2 branch (TAIL_DF32_MIN).
        if (tail_step_ok(Z, dz, dzd, d_now)) {
            if (!d_now) {
                dzd = fe_to_cdf(dz);
                d_now = true;
            }
            n_big = n_big + 1u;
            let zfn = vec2<f32>(Z.re.x + dzd.re.x, Z.im.x + dzd.im.x);
            let fp = deriv_factor(0u, zfn);
            D = fe_mul_c(D, fp.x, fp.y);
            if (iu.julia == 0u) { D = fe_add(D, fe_one()); }
            dzd = c_add(c_add(c_two(c_mul(Z, dzd)), c_sqr(dzd)), tail_dc(dc));
            ref_n = ref_n + 1u;
            iter = iter + 1u;
            let zfull_d = c_add(orbit_cdf(reference[ref_n]), dzd);
            zf = vec2<f32>(zfull_d.re.x, zfull_d.im.x);
            if (dot(zf, zf) > bail2) {
                // The escape store reads `z_full`: exact round trip, |z| > 256.
                z_full = fe_from_cdf(zfull_d);
                escaped = true;
                break;
            }
            if (tail_rebase(zfull_d, dzd) || ref_n + 1u >= seg_end(ref_n)) {
                n_rebase = n_rebase + 1u;
                // Onto Z_0, or a Julia view's critical orbit — fs_iterate's own decision.
                let d0 = c_sub(zfull_d, orbit_cdf(reference[0]));
                if (rebase_crit_df(zfull_d, d0)) {
                    dzd = zfull_d;
                    ref_n = iu.crit_at;
                } else {
                    dzd = d0;
                    ref_n = 0u;
                }
            }
            continue;
        }
        if (d_now) {
            dz = fe_from_cdf(dzd);
            d_now = false;
        }

        // Derivative update D <- f'(z_n)*D (+1 Mandelbrot) using the full z_n = Z_n + δz_n,
        // BEFORE the δ advances — the same order as the single-pass loop.
        let dzc = fe_lo_f32(dz);
        let zfn = vec2<f32>(Z.re.x + dzc.x, Z.im.x + dzc.y);
        var fp: vec2<f32>;
        if (fam_holo(iu.formula)) { fp = fam_deriv(iu.formula, zfn); } else { fp = deriv_factor(iu.formula, zfn); }
        // In floatexp at a dip f32 cannot hold (`deriv_dip`), as the single-pass loop.
        if ((iu.formula <= 3u || fam_holo(iu.formula)) && deriv_dip(r, fp)) {
            D = fe_mul(D, fe_deriv_factor(iu.formula, fe_add(orbit_fe(r), dz)));
        } else {
            D = fe_mul_c(D, fp.x, fp.y);
        }
        if (iu.julia == 0u) { D = fe_add(D, fe_one()); }

        if (fam_holo(iu.formula)) {
            // Multibrot 6–8, as fs_iterate's family arm (no extended-range dip handling, as 3–5).
            dz = fe_add(fam_pert_fe(iu.formula, Z, dz), dc);
        } else if (iu.formula == 1u) {
            // z^3: δz' = 3Z²δz + 3Z δz² + δz³ + δc
            let Z2 = c_sqr(Z);
            let dz2 = fe_sqr(dz);
            var t = fe_scale(fe_mul_cdf(dz, Z2), 3.0);
            t = fe_add(t, fe_scale(fe_mul_cdf(dz2, Z), 3.0));
            t = fe_add(t, fe_mul(dz2, dz));
            dz = fe_add(t, dc);
        } else if (iu.formula == 2u) {
            // z^4: δz' = 4Z³δz + 6Z²δz² + 4Z δz³ + δz⁴ + δc
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
            // z^5: δz' = 5Z⁴δz + 10Z³δz² + 10Z²δz³ + 5Z δz⁴ + δz⁵ + δc
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
        } else {
            // Mandelbrot: δz' = 2Z·δz + δz² + δc. THE LOAD-BEARING dip handling — at an
            // extended-range sample the 2Z·δz term must be computed in extended range; dropping
            // it re-glues every pixel to the reference each dip period (corpus 14/15).
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

        // Full value z = Z_{n+1} + δz in EXTENDED range. The f32 shortcut underflows on BOTH
        // sides at depth and silently disables Zhuoran rebasing — see the long note in fs_iterate.
        let rn = reference[ref_n];
        let Znfe = orbit_fe(rn);
        z_full = fe_add(Znfe, dz);
        zf = fe_lo_f32(z_full);
        let z2 = dot(zf, zf);
        // Pauldelbrot glitch detection — same test, operands and ORDER as fs_iterate's mode-2
        // branch: compared in scalar floatexp, never squared f32 (the f32 form underflows to
        // `0 < 0` at flushed samples and misses glitches at exactly the sensitive orbit indices).
        if (iu.glitch_on == 1u) {
            let zr = fe_abs_sf(Znfe);
            let ztol = sf_norm(vec2<f32>(zr.m.x * 1.0e-2, zr.m.y * 1.0e-2), zr.e);
            if (sf_lt(fe_abs_sf(z_full), ztol)) {
                atomicAdd(&counters[CTR_GLITCH], 1u);
                glitched = true;
                break;
            }
        }
        if (z2 > bail2) { escaped = true; break; }

        if (sf_lt(fe_abs_sf(z_full), fe_abs_sf(dz)) || ref_n + 1u >= seg_end(ref_n)) {
            n_rebase = n_rebase + 1u;
            // Rebase onto reference index 0: δz = (Z_{n+1} + δz) − Z₀ (Z₀ = 0 for Mandelbrot,
            // the reference point for Julia), or a Julia view's critical orbit — fs_iterate's own
            // decision. A pass ending one iteration either side of this must produce the same
            // pixel — that is what the chunk-boundary gates assert.
            let d0 = fe_sub(z_full, orbit_fe(reference[0]));
            if (rebase_crit_fe(z_full, d0)) {
                dz = z_full;
                ref_n = iu.crit_at;
            } else {
                dz = d0;
                ref_n = 0u;
            }
        }
    }
    ctr_commit(n_rebase, n_ext, n_bla);
    step_commit(gx, gy, n_bla + n_full + n_rn, n_full, n_big, iter - iter0);
    if (rn_px) { rn_commit(gx, gy); }
    // A pass that stops inside the tail phase stores δz as floatexp, like every other pass.
    if (d_now) {
        dz = fe_from_cdf(dzd);
    }
    if (glitched) {
        // Settled as unreliable — stored δz/derivative/ref_n are inert (the resolve reads only
        // the status), and every later chunk passes this through untouched.
        return ChunkOut4(
            vec4<f32>(dz.m.re.x, dz.m.re.y, dz.m.im.x, dz.m.im.y),
            vec4<f32>(D.m.re.x, D.m.re.y, D.m.im.x, D.m.im.y),
            info_pack(iter, ST_GLITCHED, f32(ref_n), f32(dz.e)),
            vec4<f32>(f32(D.e), 0.0, 0.0, 0.0),
        );
    }
    if (escaped) {
        status = ST_ESCAPED;
        let mag2 = dot(zf, zf);
        let nu = log(log(mag2) * 0.5 / log(2.0)) / log(power_f);
        let smit = f32(iter) + 1.0 - nu;
        esc_range_commit(smit);
        // At escape the δ is no longer needed: store the FULL z as df32 so the mode-agnostic
        // resolve can shade without the reference. |z| > 256 here, so fe_to_cdf's exponent range
        // is never the binding constraint, and its hi limbs equal fs_iterate's `fe_lo_f32(zfull)`
        // bit for bit (both scale by the same clamped exp2).
        let zc = fe_to_cdf(z_full);
        return ChunkOut4(
            vec4<f32>(zc.re.x, zc.re.y, zc.im.x, zc.im.y),
            vec4<f32>(D.m.re.x, D.m.re.y, D.m.im.x, D.m.im.y),
            info_pack(iter, status, smit, f32(D.e)),
            vec4<f32>(0.0),
        );
    }
    if (iter >= iu.max_iter) {
        status = ST_INTERIOR;
    }
    if (status == ST_RUNNING && (iu.step_cap > 0u || iu.count_running == 1u)) {
        chunk_slot = atomicAdd(&counters[CTR_CHUNK_RUNNING], 1u);
    }
    // Still running: δz mantissa + its exponent in info ch3, derivative mantissa + its exponent
    // in st_exp ch0. FE_ZERO_E (-1e9) round-trips exactly through f32 (1e9 = 1953125 * 2^9, and
    // 1953125 < 2^24), as does a -3670-ish exponent at 1e1105x.
    return ChunkOut4(
        vec4<f32>(dz.m.re.x, dz.m.re.y, dz.m.im.x, dz.m.im.y),
        vec4<f32>(D.m.re.x, D.m.re.y, D.m.im.x, D.m.im.y),
        info_pack(iter, status, f32(ref_n), f32(dz.e)),
        vec4<f32>(f32(D.e), 0.0, 0.0, 0.0),
    );
}
// @@CUSTOM_CHUNK_FE_END

// What `chunk_fe_at` left for its caller: a pixel that a step cap stopped while still running gets
// its index among this pass's running pixels (the `CTR_CHUNK_RUNNING` count it took, so the
// indices are 0..running with no gaps), every other pixel NO_SLOT.
var<private> chunk_slot: u32;
const NO_SLOT: u32 = 0xffffffffu;

@fragment
fn fs_iterate_chunk_fe(in: VsOut) -> ChunkOut4 {
    ob_arm(); // keep two_prod's product opaque (see `ob`)
    return chunk_fe_at(in.pos.xy, vec2<i32>(i32(in.pos.x), i32(in.pos.y)));
}

// ---------------- packed tails (the export's step-bounded passes) ----------------
// A step-bounded tile ends with a few pixels scattered across it: at 1.2e148 (4K) a pass with a
// tenth of its pixels running cost a third of a full pass, 8× the busy rate per step, and a
// hundredth 77×, because a warp shades a block of neighboring pixels and runs while any one of
// them does. So the export PACKS a thinning tile: every pass lists the pixels it leaves running
// (`pk_next`, indexed by `chunk_slot`), and once few enough run, `fs_pack_gather` copies their
// state into a small dense grid whose texel i iterates pixel `pk_cur[i]`. Passes then run over
// that grid (`fs_iterate_chunk_fe_packed`), and `vs/fs_pack_scatter` write it back into the
// tile's state, where `fs_resolve` finds every pixel as if the tile had never been packed.
//
// ⭐Bit-identical by construction: a packed texel runs `chunk_fe_at` on the same texel center and
// the same stored state as the whole-tile pass would (and the whole-tile passes use the same entry
// point, so no second compiled program is involved), and where the passes split never changes a
// pixel. Which slot a pixel lands in depends on atomic order and changes nothing.
struct PackU {
    packed: u32,  // 0 = a whole-tile pass (the pixel is the texel), 1 = texel i is pk_cur[i]
    n: u32,       // packed texels in use
    w: u32,       // the packed grid's width
    pad0: u32,
    tex: vec2<f32>, // the tile state textures' size (the scatter's point positions)
    pad1: vec2<f32>,
};
@group(2) @binding(0) var<uniform> pk: PackU;
@group(2) @binding(1) var<storage, read> pk_cur: array<vec2<u32>>;
@group(2) @binding(2) var<storage, read_write> pk_next: array<vec2<u32>>;

fn pack_pad() -> ChunkOut4 {
    return ChunkOut4(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0));
}

@fragment
fn fs_iterate_chunk_fe_packed(in: VsOut) -> ChunkOut4 {
    ob_arm(); // keep two_prod's product opaque (see `ob`)
    let p = vec2<i32>(i32(in.pos.x), i32(in.pos.y));
    var c = vec2<u32>(u32(p.x), u32(p.y));
    if (pk.packed == 1u) {
        let i = u32(p.y) * pk.w + u32(p.x);
        if (i >= pk.n) { return pack_pad(); }
        c = pk_cur[i];
    }
    // `f32(c) + 0.5` is the rasterizer's texel center exactly, so a whole-tile pass sees `in.pos`.
    let out = chunk_fe_at(vec2<f32>(f32(c.x) + 0.5, f32(c.y) + 0.5), p);
    if (chunk_slot != NO_SLOT) { pk_next[chunk_slot] = c; }
    return out;
}

// Texel i of the packed grid takes the state of tile pixel `pk_cur[i]` (the state set bound is the
// tile's).
@fragment
fn fs_pack_gather(in: VsOut) -> ChunkOut4 {
    let i = u32(in.pos.y) * pk.w + u32(in.pos.x);
    if (i >= pk.n) { return pack_pad(); }
    let c = vec2<i32>(pk_cur[i]);
    return ChunkOut4(textureLoad(st_z, c, 0), textureLoad(st_dz, c, 0), textureLoad(st_meta, c, 0), textureLoad(st_exp, c, 0));
}

// One point per packed texel, drawn at its tile pixel (the coordinate arrives as a vertex
// attribute: the list is the vertex buffer), carrying the packed state back (the state set bound
// is the packed grid's).
struct PackScatter {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) i: u32,
};

@vertex
fn vs_pack_scatter(@builtin(vertex_index) i: u32, @location(0) c: vec2<u32>) -> PackScatter {
    var o: PackScatter;
    let q = (vec2<f32>(c) + 0.5) / pk.tex;
    o.pos = vec4<f32>(q.x * 2.0 - 1.0, 1.0 - q.y * 2.0, 0.0, 1.0);
    o.i = i;
    return o;
}

@fragment
fn fs_pack_scatter(v: PackScatter) -> ChunkOut4 {
    let q = vec2<i32>(i32(v.i % pk.w), i32(v.i / pk.w));
    return ChunkOut4(textureLoad(st_z, q, 0), textureLoad(st_dz, q, 0), textureLoad(st_meta, q, 0), textureLoad(st_exp, q, 0));
}

// State → the normal iteration G-buffer (smooth/normal/DE + aux), same contract as fs_iterate's
// output, so the untouched color pass shades a chunked render identically. Pixels still RUNNING
// mid-progression display as interior (black) until their chunk settles them — the progressive
// display the host loop refreshes after each batch.
@fragment
fn fs_resolve(in: VsOut) -> FragOut {
    let p = vec2<i32>(i32(in.pos.x), i32(in.pos.y));
    let sz = textureLoad(st_z, p, 0);
    let sdz = textureLoad(st_dz, p, 0);
    let sm = textureLoad(st_meta, p, 0);
    let st = info_status(sm);
    if (st == ST_GLITCHED) {
        // The -2 sentinel the multi-reference corrector selects on (`smooth_iter < -1.5`). A
        // chunked glitch must read EXACTLY as a single-pass one or correction would miss it.
        return FragOut(GLITCH_SENTINEL, AUX_NONE);
    }
    if (st != ST_ESCAPED) {
        // ⭐⭐**THE CAPPED COUNT IS TAKEN HERE, NOT AT THE SETTLE TRANSITION.** The chunk entry
        // points used to `atomicAdd(CTR_MAXITER)` at the moment a pixel gave up, which counts each
        // pixel exactly once per WALK. That reads zero on a walk that has already finished — and a
        // finished walk is exactly the state a starved view sits in: `chunk=[35733,35733)`, nothing
        // transitioning, counters empty.
        //
        // ⛔The adaptive iteration budget reads this counter to decide whether to raise. Blind to a
        // completed walk, it saw `frac = 0.0000` on a frame where half the pixels were capped, so
        // it never raised, and a view needing ~600k iterations sat at the depth-scaled 35,733
        // forever — a permanently BLACK pane that survived restarts (2026-08-25 field report,
        // Mandelbrot side of a dual view at 4.6e39x).
        //
        // `fs_resolve` runs over the WHOLE frame on every chunked frame and sees the settled state,
        // so counting here reports the true capped total whether the walk is mid-flight or done.
        // ⚠NOT subsampled, unlike the escape-range commit beside it: this is a COUNT whose
        // denominator is the full pixel total (`counter_read.px`), so sampling a sixteenth of the
        // frame would under-report the fraction by 16x.
        // ⚠The chunk entry points no longer commit it, so this does not double-count. Every path
        // that runs `fs_iterate_chunk*` also runs `fs_resolve`.
        //
        // ⭐⭐2026-09-10 CORRECTION — count ONLY GENUINELY-CAPPED pixels (`ST_INTERIOR`), never ones
        // still iterating mid-chunk (`ST_RUNNING`). `ST_INTERIOR` is set only at `iter >= iu.max_iter`
        // (the FULL budget); a chunked walk leaves every not-yet-finished pixel `ST_RUNNING`, so the
        // old `st != ST_ESCAPED` counted still-running pixels as capped. A deep chunked walk then read
        // ~100% capped MID-FLIGHT — most pixels are still running most of the walk — and the adaptive
        // iteration boost's exhausted-revert fired on that false 100% and reverted a RESOLVABLE view
        // to solid BLACK, while the identical view with an explicit count (no boost, no revert)
        // rendered every window to completion and resolved. (Field report + `FRACTADYNE_TRACE=gpu` at
        // 9.34e80×: boost climbed to the full 10,000,000 appetite, "exhausted — 100.0% capped",
        // reverted to 1.0, black; auto vs explicit differed only in the revert.) Counting `ST_INTERIOR`
        // reports the TRUE capped total: 0 until pixels actually reach the budget, the real interior
        // fraction once the walk completes. It does NOT reintroduce the 2026-08-25 "finished walk reads
        // zero" bug — `ST_INTERIOR` is a settled STATE that persists on a completed walk, not the
        // one-shot transition the old chunk-entry `atomicAdd` counted.
        if (st == ST_INTERIOR) {
            atomicAdd(&counters[CTR_MAXITER], 1u);
        }
        return FragOut(vec4<f32>(-1.0, 0.0, 0.0, 1.0e30), AUX_NONE);
    }
    // An escaped pixel of the resolved frame, whatever pass settled it (see CTR_ESC_COUNT).
    // Same 4×4 grid as the escape-range commit below, and the same `p`.
    esc_count_commit(p);
    // Both modes store the FULL z (df32) in st_z at escape and the display derivative's mantissa
    // in st_dz; info ch2 = smit, ch3 = the derivative's floatexp exponent (0 for direct, whose
    // derivative is plain df32) — so the shading below is mode-agnostic and matches fs_iterate's.
    let zf = vec2<f32>(sz.x, sz.z);
    let d = vec2<f32>(sdz.x, sdz.z);
    let mag2 = dot(zf, zf);
    // @@STRIP_RESOLVE_AT_BEGIN — a strip's module sizes this texel for its distance estimate here.
    // @@STRIP_RESOLVE_AT_END
    // @@CUSTOM_RESOLVE_DE_BEGIN — a custom module has no derivative (st_dz carries z_{n-1}), so
    // it writes fs_iterate's no-DE values here instead.
    let nrm = slope_normal(zf, d);
    let de = de_log2(mag2, d, sm.w);
    // @@CUSTOM_RESOLVE_DE_END
    // ⭐⭐WHOLE-FRAME ESCAPE RANGE FOR LIVE AUTO-NORMALIZATION. The iterate passes commit the range
    // at each pixel's SETTLE TRANSITION, which on a chunked walk means only the pixels that
    // happened to settle inside THIS pass's iteration window — so the reading a chunked frame
    // produces is bounded by the window, not by the frame. Measured on a user's 9.83e27 view:
    // consecutive frames reported [46422,54614] then [54614,63736], i.e. adjacent 8k windows,
    // and `norm_range` was never fed at all (the walk-completion event that was supposed to
    // reassemble the whole range never fired), so "Normalize deep colors" silently did nothing
    // and the exterior aliased into speckle.
    //
    // `fs_resolve` sees the WHOLE FRAME every chunked frame, so committing here yields the range
    // of everything escaped SO FAR — chunk-agnostic, monotonically widening as the walk proceeds,
    // and needing no completion event.
    //
    // ⚠SUBSAMPLED, deliberately. Unlike the settle-transition commit (once per pixel per walk),
    // this runs for every escaped pixel every frame; two atomics on two slots from ~1.6M threads
    // is real contention. A 4×4 stride cuts that ~16× and still lands hundreds of thousands of
    // samples on a full frame. ⭐It also biases min/max INWARD, which is the right direction here:
    // the sibling "flat gray at shallow zoom" defect is outliers stretching the range.
    if ((p.x & 3) == 0 && (p.y & 3) == 0) {
        esc_range_commit(sm.z);
        esc_hist_commit(sm.z);
        norm_hist_commit(sm.z, p);
        // Local gradient against the RIGHT neighbor, same subsample. Bounds-checked rather than
        // relying on out-of-range textureLoad returning zero: a zeroed meta decodes to a status
        // that is not ST_ESCAPED today, but that is a coincidence to depend on.
        let qx = p.x + 1;
        if (f32(qx) < iu.res.x) {
            let smq = textureLoad(st_meta, vec2<i32>(qx, p.y), 0);
            if (info_status(smq) == ST_ESCAPED) {
                // ×16 and clamped to 12 bits. The cap bounds the u32 sum (a 4K frame subsampled
                // 4×4 is ~500k samples; 500k × 4095 stays under u32::MAX) and the coarse scale
                // costs only precision at the SMALL end, where losing a fractional gradient to
                // zero is the correct answer anyway — it means "no aliasing here".
                let d = abs(smq.z - sm.z);
                atomicAdd(&counters[CTR_GRAD_SUM], u32(clamp(d * 16.0, 0.0, 4095.0)));
                atomicAdd(&counters[CTR_GRAD_N], 1u);
                // One bucket per sample, the SAME samples as the sum. Unlike the sum this is not
                // clamped at 255 iterations: the top bucket simply absorbs everything past 2^10,
                // which is far beyond any Nyquist step the guard can ask about. `clamp` on the
                // float (not `min` on the u32) keeps an infinite step in range as well.
                //
                // ⭐**Bucket 0 is never written here.** Steps under one iteration are what every
                // smooth region produces, so in most frames nearly all samples would land on that
                // ONE address — a same-address atomic serialized across the whole frame. It carries
                // nothing CTR_GRAD_N does not already imply, so the readback derives it as
                // N − Σ(buckets 1..) and the hottest address is simply never touched. (A design
                // choice, not a measured win: the bench-matrix could not resolve a per-pixel cost
                // for the histogram either way — see the note on grad_hist_from_slots.)
                if (d >= 1.0) {
                    let hb = 1u + u32(clamp(floor(log2(d)), 0.0, 10.0));
                    atomicAdd(&counters[CTR_GRAD_HIST + hb], 1u);
                }
            }
        }
    }
    // @@STRIP_RESOLVE_OUT_BEGIN — a strip's module writes its count, fraction and angle here.
    // The final angle when the frame carries it (B8: a frame that accumulates running statistics
    // never chunks, so this is `aux_escape`'s answer for every frame that reaches here).
    return FragOut(vec4<f32>(sm.z, nrm.x, nrm.y, de), aux_angle(zf));
    // @@STRIP_RESOLVE_OUT_END
}

// ---------------- coloring pass (samples the iteration texture) ----------------
struct ColorU {
    lut_len: u32, // entries in the baked palette LUT at binding 3 (see `palette()`)
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
    color_method: u32, // a METHOD_* constant (fractadyne_core::coloring)
    aa_filter: u32,    // color-pass box-filter taps per axis (≥1); >1 anti-aliases an upscaled iter texture
    reproject: u32,    // 1 = reprojection: sample the frozen texture scaled+translated (no re-iterate)
    uv_off: vec2<f32>, // uv translation for the reprojection (fraction of the screen panned)
    uv_scale: f32,     // uv scale about center (1 = pan only; <1 = zoomed in since the frozen frame)
    vig_on: u32,       // 1 = spotlight vignette: dim everything outside a soft circle
    vig_dim: f32,      // how dark outside the spotlight (0..1)
    vig_soft: f32,     // soft-edge width (uv height fraction)
    vig_center: vec2<f32>, // spotlight center in screen uv (0..1)
    vig_radius: f32,   // spotlight radius (uv height fraction)
    // 1 = interpolate between LUT entries, 0 = nearest (hard bands). Occupies what used to be
    // `_pad_vig`, so removing the stop array below is the only size change this struct has taken.
    lut_smooth: u32,
    interior_col: vec4<f32>, // color for in-set (non-escaping) pixels; rgb in xyz
    out_res: vec2<f32>, // output rect size (px); with reproject, aspect-fits a frozen (old-size) frame
    // Palette-range mapping. 0 = linear (`cycle`/`offset` alone, the classic affine map);
    // 1 = LOG, where the escape value is compressed as log(v - norm_lo + 1) BEFORE the affine map.
    // Occupies what used to be `_pad_out`, so the uniform's size and alignment are unchanged.
    norm_mode: u32,
    norm_lo: f32,   // range floor the log is measured from (the frame's minimum escape value)
    // Analytic palette anti-aliasing, box-filtering the palette over each pixel's footprint via
    // `palette_box`: 0 = point sample; 1 = the footprint from the next pixels' coordinates; 2 = from
    // the distance estimate where the Smooth count has one (`de_footprint`, coloring plan C8).
    aa_palette: u32,
    // A dissolve from another palette map (`PaletteFade`): that map's color takes `fade_weight`
    // (0 = none). Mirrors the Rust `ColorUniforms` field for field.
    fade_weight: f32,
    fade_mode: u32,
    fade_lo: f32,
    fade_cycle: f32,
    fade_offset: f32,
    // Output pixels across the rect the color pass fills, or 0 for the texture's own base
    // (`width / ss`): what `fs_color` turns the stored per-TEXEL distance estimate into per-pixel
    // distance with (`de_px_shift`). In what was `_pad_fade0`; mirrors `ColorUniforms.display_w`.
    display_w: f32,
    // How an interior texel is shaded (an INTERIOR_* constant, coloring plan B5), then the interior
    // palette's length in the LUT buffer's second slot (0: the outside palette is used) and its
    // fetch mode, and the interior's own rate and offset. Mirrors `ColorUniforms`.
    interior_mode: u32,
    lut2_len: u32,
    lut2_smooth: u32,
    interior_cycle: f32,
    interior_offset: f32,
    // ln of the formula's escape degree for the distance-sized palette footprint (coloring plan
    // C8, `aa_palette == 2`), 0 where the formula has no distance estimate. Mirrors
    // `ColorUniforms.aa_ln_p`.
    aa_ln_p: f32,
    // The footprint's share for this frame's samples: 1, or 1/√(k+1) for progressive supersampling's
    // sample k (`aa_scale` in the Rust).
    aa_scale: f32,
    // Every `tap_stride`-th texel of a pixel's block, each standing for that many (0 or 1 = every
    // texel): the palette browser's thumbnails (`atlas_tap_stride` in the Rust). One pad word.
    tap_stride: u32,
    // The band texture's center (coloring plan E1, `band_texture_words`), in what was a pad word.
    band_tex_center: f32,
    // A coloring hook's parameters `p1()`..`p4()` (coloring plan B10; read only by a hook's module,
    // `color_hook.wgsl`). Mirrors `ColorUniforms.hook_p`.
    hook_p: vec4<f32>,
    // How the palette continues past its end (coloring plan C2, `fractadyne_core::coloring::Wrap`):
    // 0 repeat, 1 mirror, 2 clamp; and the flat bands a palette cycle is cut into (0: none).
    wrap: u32,
    steps: f32,
    // Edge shading (coloring plan C3, `edge_shaded`): 0 off, 1 toward `edge_col`, 2 toward the
    // inside's color (`interior_col`); its curve (`EdgeCurve::gpu_id`), in what were this row's pad
    // words. Then the width in output pixels and the strength, and the color. Mirrors
    // `ColorUniforms`.
    edge_mode: u32,
    edge_curve: u32,
    edge_px: f32,
    edge_strength: f32,
    // The band texture (coloring plan E1): the lane of the statistic that shifts the Smooth method's
    // palette position (0 none, 1 the stripe average `aux.r`, 2 the triangle inequality's `aux.g`)
    // and by how much, palette cycles per unit; in what were this row's pad words.
    band_tex: u32,
    band_tex_amount: f32,
    edge_col: vec4<f32>,
    // Equipotential lines (coloring plan C10, `grid_shaded`): on, the spacing, width and strength
    // in output pixels, then the color. Mirrors `ColorUniforms`.
    grid_on: u32,
    grid_spacing: f32,
    grid_width: f32,
    grid_strength: f32,
    grid_col: vec4<f32>,
    // The relief light's surface and highlight (coloring plan C5, `relief_highlight`): the slope from
    // the distance (45 degrees this many output pixels from the set; 0 = 45 degrees everywhere, the
    // classic relief), the highlight's strength (0 = none) and shininess, then its color. Mirrors
    // `ColorUniforms`.
    relief_steep_px: f32,
    relief_highlight: f32,
    relief_shininess: f32,
    // An export tile's apron, in what was this row's pad word (`fs_color`; mirrors
    // `ColorUniforms.tile_apron`).
    tile_apron: u32,
    relief_hl_col: vec4<f32>,
    // Histogram equalization (coloring plan C6, `equalized`): the frame's share and sweeps, the
    // dissolve's share and sweeps; then each map's remap knots 0..255 (knot 256 is 1), four a row.
    // Mirrors `ColorUniforms.eq`, `.eq_knots`, `.fade_eq_knots`.
    eq: vec4<f32>,
    eq_knots: array<vec4<f32>, 64>,
    fade_eq_knots: array<vec4<f32>, 64>,
    // The band texture only near the set (coloring plan E2): its distance in output pixels, 0
    // everywhere. Mirrors `ColorUniforms.band_tex_near`.
    band_tex_near: f32,
    _pad_bt0: f32,
    _pad_bt1: f32,
    _pad_bt2: f32,
};
@group(0) @binding(0) var<uniform> cu: ColorU;
@group(0) @binding(1) var iter_tex: texture_2d<f32>;
@group(0) @binding(2) var aux_tex: texture_2d<f32>;
// The baked palette (rgb + alpha per entry). A STORAGE buffer, not a uniform array: 1024 entries
// is 16 KB, which is exactly the downlevel `max_uniform_buffer_binding_size`, so a uniform would
// have left no room for the rest of `ColorU` on a conservative adapter. Not a texture either —
// the fetch below does its own wrap and its own nearest/linear choice, so a sampler would only add
// a binding and take the choice away.
@group(0) @binding(3) var<storage, read> lut: array<vec4<f32>>;

// ---------------------------------------------------------------------------
// Seed pass (tiled settle): nearest-neighbor upscale of the previous
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

// One indexed fetch into the baked LUT. Everything a palette can be — an 8-stop preset, a curved
// GIMP segment, an HSV sweep, a 256-band Fractint `.map` — arrives here already evaluated, so this
// costs the same for all of them and the shader never learns what a blend function is.
//
// Entry `i` holds the gradient at `(i + 0.5) / n` (texel centers — `Gradient::bake` in
// `fractadyne-color` is the other half of this contract, and `Lut::sample` is this function in
// Rust). `lut_smooth == 0` nearest-fetches, which is what keeps a `.map`'s bands hard; anything
// else interpolates, which is what gives a deep view palette-POSITION resolution at a high cycle.
// Indices WRAP rather than clamp: the palette is cycled by `fract`, so t = 1 and t = 0 are
// neighbors on screen.
fn palette(t_in: f32) -> vec3<f32> {
    return palette_in(t_in, 0u, cu.lut_len, cu.lut_smooth);
}

// [`palette`] over the LUT slot starting at entry `base`, of `len` entries: the outside palette at
// 0, the interior one at `LUT2_BASE` (coloring plan B5; generated with the coloring ids).
fn palette_in(t_in: f32, base: u32, len: u32, interp: u32) -> vec3<f32> {
    let n = i32(len);
    if (n <= 0) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    let b0 = i32(base);
    let t = fract(t_in);
    // (`interp`, not `smooth`: that is a reserved word in WGSL, which naga's stricter front end, the
    // custom formula module's, refuses.)
    if (interp == 0u) {
        return lut[b0 + clamp(i32(t * f32(n)), 0, n - 1)].xyz;
    }
    let x = t * f32(n) - 0.5;
    let i0 = i32(floor(x));
    let f = x - floor(x);
    let a = lut[b0 + ((i0 % n) + n) % n].xyz;
    let b = lut[b0 + (((i0 + 1) % n) + n) % n].xyz;
    return mix(a, b, f);
}

// The color of an interior texel (`coord_of` said INTERIOR_COORD; `a` its aux lanes), as
// `cu.interior_mode` says (coloring plan B5): under INTERIOR_TRAP the trap's value, nearer approach
// larger as the trap method has it, under the interior's own rate through the interior palette
// (the outside one when the request carries none); otherwise the one interior color. (The trap
// method's own interior, INTERIOR_METHOD, never arrives here: `coord_under` colors it.)
fn interior_shade(a: vec4<f32>) -> vec3<f32> {
    if (cu.interior_mode == INTERIOR_TRAP) {
        let t = -log2(max(a.b, 1.0e-20)) * cu.interior_cycle + cu.interior_offset;
        if (cu.lut2_len > 0u) {
            return palette_in(t, LUT2_BASE, cu.lut2_len, cu.lut2_smooth);
        }
        return palette(t);
    }
    return cu.interior_col.xyz;
}

// log2 of how many iteration texels span one OUTPUT pixel, set once at the top of `fs_color` (every
// function below that reads the distance estimate runs inside it). `de_log2` stores the distance in
// texels, and a texel is 1/ss of a pixel on a supersampled frame, more than a pixel on a reduced
// motion frame, and changes size while a reprojection follows a zoom; the Distance method and the
// glow are specified in output pixels, so they subtract this.
var<private> de_px_shift: f32;

// The size of one tap of `fs_color`'s average in output pixels, set beside `de_px_shift`: a texel
// (2^-de_px_shift), `tap_stride` of them for a thumbnail, and progressive supersampling's sample's
// share of that (`aa_scale`). What the equipotential lines' coverage is filtered over.
var<private> tap_px: f32;

// Interior pixels have no palette coordinate; `coord_of` returns this sentinel so callers color
// them with `interior_col` and the footprint estimate skips them. Far above any real coordinate
// (`cycle`·log(escape) + offset), and finite so arithmetic on it stays well-defined.
const INTERIOR_COORD: f32 = 1.0e30;

// The CONTINUOUS palette coordinate for one texel (before the `fract` wrap), per the selected
// method, or `INTERIOR_COORD` for an in-set pixel. Split out from `shade` so the color pass can
// finite-difference it between neighboring pixels to size the anti-aliasing footprint — and
// diffing the pre-`fract` value is what keeps that footprint free of wrap discontinuities.
// `m` = (smooth iter, normal.x, normal.y, DE log2); `a` = (stripe, TIA, trap, decomp).
fn coord_of(m: vec4<f32>, a: vec4<f32>) -> f32 {
    return coord_under(m, a, cu.cycle, cu.offset, cu.norm_mode, cu.norm_lo, 0u);
}

// The palette coordinate under the map a dissolve fades from (`cu.fade_*`).
fn coord_fade(m: vec4<f32>, a: vec4<f32>) -> f32 {
    return coord_under(m, a, cu.fade_cycle, cu.fade_offset, cu.fade_mode, cu.fade_lo, 1u);
}

// The palette coordinate of a texel under a palette map (`cycle`, `offset`, and the range mapping
// `mode` about `lo`, and the equalization of map `which`: 0 the frame's, 1 the dissolve's);
// `INTERIOR_COORD` for an interior texel, whatever the map.
fn coord_under(m: vec4<f32>, a: vec4<f32>, cycle: f32, offset: f32, mode: u32, lo: f32, which: u32) -> f32 {
    var pv: f32;
    var interior: bool = (m.r < 0.0);
    if (cu.color_method == METHOD_STRIPE) {
        pv = a.r;                                  // stripe average (0..1)
    } else if (cu.color_method == METHOD_TRIANGLE) {
        pv = a.g;                                  // triangle-inequality average (0..1)
    } else if (cu.color_method == METHOD_TRAP) {
        // The orbit trap colors the interior too, as its own (INTERIOR_METHOD, the default); a
        // view that chose another interior (B5) leaves it to `interior_shade`.
        interior = interior && cu.interior_mode != INTERIOR_METHOD;
        pv = -log2(max(a.b, 1.0e-20));             // nearer approach → larger value
    } else if (cu.color_method == METHOD_DISTANCE) {
        interior = (m.a > 1.0e20);                 // DE unavailable → interior
        pv = m.a - de_px_shift;                    // distance estimate, log2 output pixels
    } else if (cu.color_method == METHOD_FRINGE) {
        interior = interior || (m.a > 1.0e20);     // in the set, or no distance estimate
        pv = fringe_turn(m);                       // the slope's direction, in turns (0..1)
    } else if (cu.color_method == METHOD_DECOMPOSITION) {
        pv = a.a;                                  // decomposition angle (0..1)
    } else {
        pv = m.r;                                  // smooth iteration count
    }
    if (interior) { return INTERIOR_COORD; }
    // The Smooth method's band texture (coloring plan E1): a statistic moves each band edge along
    // its texture, by `band_tex_amount` palette cycles per unit of it from its center, after the
    // map. Off (0), nothing is added and every value is as before.
    var tex = 0.0;
    if (cu.band_tex != 0u && cu.color_method == METHOD_SMOOTH) {
        let s = select(a.r, a.g, cu.band_tex == 2u);
        tex = cu.band_tex_amount * (s - cu.band_tex_center);
        // E2: only near the set, the weight exp(-d / near) of the distance d in output pixels, so the
        // texture grows out of the filaments and fades into plain bands; with no distance (a formula
        // without one), plain bands.
        if (cu.band_tex_near > 0.0) {
            tex = tex * select(0.0, exp(-exp2(m.a - de_px_shift) / cu.band_tex_near), m.a < 1.0e20);
        }
    }
    // An equalized map (C6) first, apart: the plain map's expression below is the one the hook's
    // `palette_map` writes too, and the two must stay the same bits (B10's Smooth-hook check).
    if (eq_share(which) > 0.0) {
        if (tex != 0.0) { return map_equalized(pv, cycle, offset, mode, lo, which) + tex; }
        return map_equalized(pv, cycle, offset, mode, lo, which);
    }
    // Log range mapping. Escape values crowd towards the high end at depth — most of a deep
    // frame's pixels sit in the last few percent of the range — so a linear map spends nearly the
    // whole palette on a thin shell and flattens everything else. Compressing with log spreads
    // the palette over the range as the eye reads it, which is what keeps color stable through a
    // zoom video rather than washing out as the range grows.
    // `+ 1` keeps the argument ≥ 1 (log ≥ 0) so the floor of the range maps to 0 exactly.
    var pv_mapped = pv;
    if (mode != 0u) {
        pv_mapped = transfer_of(max(pv - lo, 0.0), mode);
    }
    if (tex != 0.0) { return pv_mapped * cycle + offset + tex; }
    return pv_mapped * cycle + offset;
}

// A value under an equalized palette map (C6): from its floor in every mode, a linear one too, the
// curve (C2), the remap of map `which` mixed in, then the offset.
fn map_equalized(pv: f32, cycle: f32, offset: f32, mode: u32, lo: f32, which: u32) -> f32 {
    let d = pv - lo;
    var x = d * cycle;
    if (mode != 0u) {
        x = transfer_of(max(d, 0.0), mode) * cycle;
    }
    return equalized(x, which) + offset;
}

// ---- Histogram equalization (coloring plan C6; the CPU twins are in fractadyne_core::coloring) ----

// Map `which`'s share of the remap (0: none) and its sweeps across the window.
fn eq_share(which: u32) -> f32 {
    return select(cu.eq.x, cu.eq.z, which == 1u);
}
fn eq_sweeps(which: u32) -> f32 {
    return select(cu.eq.y, cu.eq.w, which == 1u);
}

// Knot `j` (0..=256) of map `which`'s remap; the last is 1 and not stored.
fn eq_knot(which: u32, j: u32) -> f32 {
    if (j >= 256u) { return 1.0; }
    var r: vec4<f32>;
    if (which == 1u) { r = cu.fade_eq_knots[j >> 2u]; } else { r = cu.eq_knots[j >> 2u]; }
    return r[j & 3u];
}

// R(u) (`eq_remap`): linear between the knots, on at slope 1 past the window's two ends.
fn eq_remap(which: u32, u: f32) -> f32 {
    if (u <= 0.0) { return eq_knot(which, 0u) + u; }
    if (u >= 1.0) { return u; }
    let x = u * 256.0;
    let i = min(u32(floor(x)), 255u);
    let a = eq_knot(which, i);
    return a + (eq_knot(which, i + 1u) - a) * (x - f32(i));
}

// dR/du (`eq_slope`): the interval's, 1 outside [0, 1].
fn eq_slope_at(which: u32, u: f32) -> f32 {
    if (u < 0.0 || u >= 1.0) { return 1.0; }
    let i = min(u32(floor(u * 256.0)), 255u);
    return (eq_knot(which, i + 1u) - eq_knot(which, i)) * 256.0;
}

// The equalized coordinate (`equalized`): x = sweeps·u before the offset, mixed with sweeps·R(u).
fn equalized(x: f32, which: u32) -> f32 {
    let share = eq_share(which);
    let sweeps = eq_sweeps(which);
    return (1.0 - share) * x + share * sweeps * eq_remap(which, x / sweeps);
}

// d(equalized)/dx: how much faster the palette runs at `x` than under the fitted map alone.
fn equalized_rate(x: f32, which: u32) -> f32 {
    let share = eq_share(which);
    if (share <= 0.0) { return 1.0; }
    return (1.0 - share) + share * eq_slope_at(which, x / eq_sweeps(which));
}

// A palette map's curve (coloring plan C2) at a height `d` ≥ 0 above its floor: `mode` is
// `fractadyne_core::coloring::Transfer::gpu_id` (whose `apply` is the CPU twin): 1 log (the only
// one before C2, its expression unchanged), 2 square root, 3 cube root, 4 fourth root, 5 log-log,
// 6 square. Each is 0 at 0.
fn transfer_of(d: f32, mode: u32) -> f32 {
    switch (mode) {
        case 1u: { return log(d + 1.0); }
        case 2u: { return sqrt(d); }
        case 3u: { return select(0.0, pow(d, 1.0 / 3.0), d > 0.0); }
        case 4u: { return sqrt(sqrt(d)); }
        case 5u: { return log(log(d + 1.0) + 1.0); }
        case 6u: { return d * d; }
        default: { return d; }
    }
}

// The curve's slope dT/dd at `d` (`Transfer::slope`): the palette anti-aliasing's footprint is the
// map's rate times it. The roots' slopes are taken at d ≥ 1e-6 (unbounded at the floor).
fn transfer_slope(d: f32, mode: u32) -> f32 {
    let r = max(d, 1.0e-6);
    switch (mode) {
        case 1u: { return 1.0 / (d + 1.0); }
        case 2u: { return 0.5 / sqrt(r); }
        case 3u: { return pow(r, -2.0 / 3.0) / 3.0; }
        case 4u: { return 0.25 * pow(r, -0.75); }
        case 5u: { return 1.0 / ((d + 1.0) * (log(d + 1.0) + 1.0)); }
        case 6u: { return 2.0 * d; }
        default: { return 1.0; }
    }
}

// Cyclic antiderivative of the palette at texel coordinate `x` (may be < 0 or > n), from the
// prefix sum appended after the LUT at binding 3 (`psum[i] = lut[n + i]`, `psum[n]` the total).
// Each entry is a unit-width texel of constant color, so within a cycle the integral is
// `psum[floor(r)] + frac(r)·entries[floor(r)]`, plus `total` per whole cycle wrapped.
fn palette_psum_at(x: f32, n: i32, nf: f32, total: vec3<f32>) -> vec3<f32> {
    let cycles = floor(x / nf);
    let r = x - cycles * nf;                       // in [0, n)
    let i = clamp(i32(floor(r)), 0, n - 1);
    let f = r - floor(r);
    return cycles * total + lut[n + i].xyz + f * lut[i].xyz;
}

// Box-average the palette over a footprint `width` PALETTE CYCLES wide, centered at `coord`.
// This is the analytic anti-aliasing: `width` is how much of the palette a single pixel spans,
// so an undersampled pixel gets the mean over what it covers rather than a point sample that
// aliases into speckle. Continuous in `width` — `width ≤ 1 texel` returns exactly `palette`
// (so hard `.map` bands stay crisp while resolvable, and there is no seam where the filter turns
// on), `width ≥ 1 cycle` returns the DC mean; everything between is the exact cyclic box integral.
// Mirrors `Lut::sample_box` in `fractadyne-color`.
fn palette_box(coord: f32, width: f32) -> vec3<f32> {
    let n = i32(cu.lut_len);
    if (n <= 0) { return vec3<f32>(0.0, 0.0, 0.0); }
    let nf = f32(n);
    let w = clamp(width * nf, 0.0, nf);            // footprint in texels
    if (w <= 1.0) { return palette(coord); }       // sharp fetch (respects lut_smooth)
    let total = lut[n + n].xyz;                     // psum[n]
    let c = fract(coord) * nf;
    let hi = palette_psum_at(c + w * 0.5, n, nf, total);
    let lo = palette_psum_at(c - w * 0.5, n, nf, total);
    return (hi - lo) / w;
}

// ---- Steps and the wraps (coloring plan C2; the CPU twin is `fractadyne_core::coloring::wrap_coord`) ----

// The position cut into `cu.steps` flat bands a palette cycle, each the color at its start; as it
// is with no steps.
fn stepped(t: f32) -> f32 {
    if (cu.steps > 0.0) {
        return floor(t * cu.steps) / cu.steps;
    }
    return t;
}

// A position folded into the palette as Mirror (back and forth, period 2) or Clamp (held at the
// ends) takes it.
fn folded(t: f32) -> f32 {
    if (cu.wrap == 1u) {
        let p = t - 2.0 * floor(t * 0.5);
        return select(p, 2.0 - p, p > 1.0);
    }
    return clamp(t, 0.0, 1.0);
}

// The palette at a position 0..1 read WITHOUT wrapping (Mirror's turn and Clamp's end are the last
// color, which `palette`'s `fract` would make the first): the end texels hold.
fn palette_held(t: f32) -> vec3<f32> {
    let n = i32(cu.lut_len);
    if (n <= 0) { return vec3<f32>(0.0, 0.0, 0.0); }
    let tt = clamp(t, 0.0, 1.0);
    if (cu.lut_smooth == 0u) {
        return lut[clamp(i32(tt * f32(n)), 0, n - 1)].xyz;
    }
    let x = clamp(tt * f32(n) - 0.5, 0.0, f32(n - 1));
    let i0 = i32(floor(x));
    let i1 = min(i0 + 1, n - 1);
    return mix(lut[i0].xyz, lut[i1].xyz, x - floor(x));
}

// The palette's antiderivative along the UNFOLDED position `x` (texels), as Mirror or Clamp folds
// it: Mirror's period is 2n (forward, then back: the back half integrates the palette again, so a
// period holds it twice); Clamp extends the first color below 0 and the last above n.
fn palette_psum_folded(x: f32, n: i32, nf: f32, total: vec3<f32>) -> vec3<f32> {
    if (cu.wrap == 1u) {
        let per = 2.0 * nf;
        let k = floor(x / per);
        let r = x - k * per;
        var g: vec3<f32>;
        if (r <= nf) {
            g = palette_psum_at(r, n, nf, total);
        } else {
            g = 2.0 * total - palette_psum_at(per - r, n, nf, total);
        }
        return k * 2.0 * total + g;
    }
    if (x <= 0.0) { return x * lut[0].xyz; }
    if (x >= nf) { return total + (x - nf) * lut[n - 1].xyz; }
    return palette_psum_at(x, n, nf, total);
}

// The color at palette position `coord` under the view's Steps and Wrap, box-averaged over `width`
// cycles as `palette_box` (which it is, for Repeat): for Mirror and Clamp the box is taken over the
// folded palette, so a footprint across a turn or an end averages what the screen shows there.
fn palette_shaded(coord: f32, width: f32) -> vec3<f32> {
    let t = stepped(coord);
    if (cu.wrap == 0u) { return palette_box(t, width); }
    let n = i32(cu.lut_len);
    if (n <= 0) { return vec3<f32>(0.0, 0.0, 0.0); }
    let nf = f32(n);
    let w = clamp(width * nf, 0.0, nf);
    if (w <= 1.0) { return palette_held(folded(t)); }
    let total = lut[n + n].xyz;
    let c = t * nf;
    return (palette_psum_folded(c + w * 0.5, n, nf, total) - palette_psum_folded(c - w * 0.5, n, nf, total)) / w;
}

// The palette footprint of a Smooth tap from its distance estimate (coloring plan C8): how many
// palette cycles a tap `s` OUTPUT pixels across spans there, |dt/dμ| · s · |∇μ| with |∇μ| =
// 1 / (d · ln p), d the distance in output pixels (the stored DE, `|z|·ln|z| / |dz/dc|`, is exactly
// G/|∇G| for the Green's function G, and μ = −log_p G + const). `cycle` and the map (`mode`, `lo`)
// are the palette map's own, so a log map's dt/dμ = cycle / (μ − lo + 1). Negative where the texel
// has no distance (no derivative, or `aa_ln_p` 0): the caller keeps the neighbor-difference
// footprint. Unlike that one it does not reach across to the next pixel, which at a boundary may be
// a different band of the escape field altogether.
// Near the set this runs to many cycles and the tap takes the palette's mean color: right for a
// whole pixel in the decorations, which holds many filaments, and wrong for a small tap, which
// resolves their beads (see `AA_MIN_TAP`). log_p(1 + s/d), what μ does across one boundary, was
// measured and lost most of the gain at one sample a pixel (0.040 against 0.014, point 0.047).
fn de_footprint(m: vec4<f32>, cycle: f32, mode: u32, lo: f32, s: f32) -> f32 {
    if (cu.aa_ln_p <= 0.0 || m.a > 1.0e20 || m.r < 0.0) { return -1.0; }
    let d_px = exp2(m.a - de_px_shift);
    var dtdmu = abs(cycle);
    if (mode == 1u) {
        dtdmu = dtdmu / (max(m.r - lo, 0.0) + 1.0);
    } else if (mode != 0u) {
        dtdmu = dtdmu * transfer_slope(max(m.r - lo, 0.0), mode);
    }
    // An equalized map (C6) runs the palette faster where the pixels crowd: its footprint too.
    if (eq_share(0u) > 0.0) {
        var x = (m.r - lo) * cycle;
        if (mode != 0u) {
            x = transfer_of(max(m.r - lo, 0.0), mode) * cycle;
        }
        dtdmu = dtdmu * equalized_rate(x, 0u);
    }
    return dtdmu * s / max(d_px * cu.aa_ln_p, 1.0e-12);
}

// The smallest tap, in output pixels, whose palette is anti-aliased (C8). Below it, at 4x
// supersampling and up and after the first few samples of progressive supersampling, the taps are
// point sampled: there the average of the samples is already closer to the true picture than the
// filter, which takes a tap near a filament to the palette's mean and flattens the decorations'
// beads. Measured against 32x32 samples a pixel by the seahorse valley (point / distance-sized):
// 1 sample 0.047 / 0.014, 2x2 0.022 / 0.015, 4x4 0.011 / 0.013, 8x8 0.007 / ~0.012; at 61x and the
// overview the same crossing.
const AA_MIN_TAP: f32 = 0.499;

// Point-sample shade (interior → interior_col). Used by `view_average`; the main color pass uses
// `shade_aa` so it can band-limit the palette over the pixel footprint.
fn shade(m: vec4<f32>, a: vec4<f32>) -> vec3<f32> {
    let coord = coord_of(m, a);
    if (coord >= INTERIOR_COORD) { return interior_shade(a); }
    return fringed(palette_shaded(coord, 0.0), m);
}

// ---- The fringe (coloring plan C9) ----
// Dark lines along the set on a light field whose colors turn once around every feature: the
// palette at the slope's direction (`fringe_turn`), at a lightness from the distance estimate
// (`fringe_lightness`), its hue and chroma kept in OKLab. The prototype's look (the coloring
// research, after zoomasm's rainbow-fringe preset, written from its description); a palette of many
// hues gives the rainbow, any other its own colors.

// The fringe's palette position: the direction of the slope normal u = z / (dz/dc), in turns.
fn fringe_turn(m: vec4<f32>) -> f32 {
    return fract(atan2(m.b, m.g) * 0.15915494309 + 1.0);
}

// Dark at the set, light from FRINGE_WIDTH_PX output pixels away: 0.12 + 0.78·√clamp(d / width).
const FRINGE_L0: f32 = 0.12;
const FRINGE_LS: f32 = 0.78;
const FRINGE_WIDTH_PX: f32 = 1.0;
fn fringe_lightness(de: f32) -> f32 {
    let d = exp2(min(de - de_px_shift, 60.0));     // the distance in output pixels
    return FRINGE_L0 + FRINGE_LS * sqrt(clamp(d / FRINGE_WIDTH_PX, 0.0, 1.0));
}

// `c` under the fringe when it is the method (`m` the texel's main lane); `c` otherwise.
fn fringed(c: vec3<f32>, m: vec4<f32>) -> vec3<f32> {
    if (cu.color_method != METHOD_FRINGE) { return c; }
    return with_lightness(c, fringe_lightness(m.a));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
    let v = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(pow((v + 0.055) / 1.055, vec3<f32>(2.4)), v / 12.92, v <= vec3<f32>(0.04045));
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    let v = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(1.055 * pow(v, vec3<f32>(1.0 / 2.4)) - 0.055, 12.92 * v, v <= vec3<f32>(0.0031308));
}

// A display (sRGB-encoded) color to OKLab (Björn Ottosson's matrices, as `fractadyne_color::adjust`).
fn oklab_of(c: vec3<f32>) -> vec3<f32> {
    let r = srgb_decode(c);
    let lms = vec3<f32>(
        0.4122214708 * r.x + 0.5363325363 * r.y + 0.0514459929 * r.z,
        0.2119034982 * r.x + 0.6806995451 * r.y + 0.1073969566 * r.z,
        0.0883024619 * r.x + 0.2817188376 * r.y + 0.6299787005 * r.z,
    );
    let q = pow(max(lms, vec3<f32>(0.0)), vec3<f32>(1.0 / 3.0));
    return vec3<f32>(
        0.2104542553 * q.x + 0.7936177850 * q.y - 0.0040720468 * q.z,
        1.9779984951 * q.x - 2.4285922050 * q.y + 0.4505937099 * q.z,
        0.0259040371 * q.x + 0.7827717662 * q.y - 0.8086757660 * q.z,
    );
}

// OKLab to linear-light sRGB, not clamped (a channel outside 0..1 is out of gamut).
fn oklab_linear(lab: vec3<f32>) -> vec3<f32> {
    let l_ = lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z;
    let m_ = lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z;
    let s_ = lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    return vec3<f32>(
        4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3,
        -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3,
        -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3,
    );
}

fn in_gamut(v: vec3<f32>) -> bool {
    return all(v >= vec3<f32>(-1.0e-5)) && all(v <= vec3<f32>(1.0 + 1.0e-5));
}

// `c` (display color) at OKLab lightness `l`, hue and chroma kept, the chroma reduced by bisection
// until the color fits the gamut (0, gray at that lightness, always does). The CPU twin is
// `fractadyne_color::adjust::with_lightness`.
fn with_lightness(c: vec3<f32>, l: f32) -> vec3<f32> {
    let lab = oklab_of(c);
    let full = oklab_linear(vec3<f32>(l, lab.y, lab.z));
    if (in_gamut(full)) { return srgb_encode(full); }
    var lo = 0.0;
    var hi = 1.0;
    for (var k = 0; k < 12; k = k + 1) {
        let mid = 0.5 * (lo + hi);
        if (in_gamut(oklab_linear(vec3<f32>(l, lab.y * mid, lab.z * mid)))) { lo = mid; } else { hi = mid; }
    }
    return srgb_encode(oklab_linear(vec3<f32>(l, lab.y * lo, lab.z * lo)));
}

// A palette-coordinate step between neighboring pixels (the anti-aliasing footprint). The fringe's
// coordinate turns with the slope's direction, a whole `cycle` a turn, so across the turn's seam the
// step is the shorter way round: otherwise every seam pixel averaged whole palette sweeps into a
// gray ray out of every feature.
fn coord_step(d: f32, cycle: f32) -> f32 {
    let s = abs(d);
    if (cu.color_method != METHOD_FRINGE) { return s; }
    return min(s, abs(abs(cycle) - s));
}

// Band-limited shade: color one texel with the palette box-averaged over `width` cycles (the
// output pixel's palette-phase footprint). `width == 0` degenerates to the point sample, so this
// is a drop-in for `shade` when the anti-aliasing is off (`cu.aa_palette == 0`, width left 0).
// During a dissolve (`cu.fade_weight` > 0) the texel is also shaded under the map it fades from,
// with that map's own footprint `width_fade`, and the two colors are blended.
fn shade_aa(m: vec4<f32>, a: vec4<f32>, width: f32, width_fade: f32) -> vec3<f32> {
    let coord = coord_of(m, a);
    if (coord >= INTERIOR_COORD) { return interior_shade(a); }
    var col = palette_shaded(coord, width);
    if (cu.fade_weight > 0.0) {
        col = mix(col, palette_shaded(coord_fade(m, a), width_fade), cu.fade_weight);
    }
    return fringed(col, m);
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

// ---- Edge shading (coloring plan C3) ----
// An escaped texel near the set takes the edge color by its distance d from the set in output
// pixels, at the weight `edge_strength · (1 − f(min(d / edge_px, 1)))` (`fractadyne_core::coloring::
// edge_weight`): all of it at the set, none `edge_px` pixels out. Each texel is shaded before a
// pixel's texels are averaged, so the line is anti-aliased as the rest of the picture is. Toward
// the inside's color (mode 2) it is Chéritat's blend: the set's edge smoothed by the distance, on a
// frame of one sample a pixel too. Interior texels, and those with no distance estimate (a formula
// without a derivative), keep their color.

// f(x), x in 0..1 (`EdgeCurve::apply`): linear, square root, square, smoothstep, a hard step.
fn edge_curve_of(x: f32) -> f32 {
    switch (cu.edge_curve) {
        case 1u: { return sqrt(x); }
        case 2u: { return x * x; }
        case 3u: { return x * x * (3.0 - 2.0 * x); }
        case 4u: { return select(0.0, 1.0, x >= 1.0); }
        default: { return x; }
    }
}

// `c`, the color of the texel `m`, under the edge shading.
fn edge_shaded(c: vec3<f32>, m: vec4<f32>) -> vec3<f32> {
    if (cu.edge_mode == 0u || m.r < 0.0 || m.a > 1.0e20) { return c; }
    let d = exp2(min(m.a - de_px_shift, 60.0));   // the distance in output pixels
    let x = clamp(d / max(cu.edge_px, 0.05), 0.0, 1.0);
    let w = clamp(cu.edge_strength * (1.0 - edge_curve_of(x)), 0.0, 1.0);
    let e = select(cu.edge_col.xyz, cu.interior_col.xyz, cu.edge_mode == 2u);
    return mix(c, e, w);
}

// ---- Equipotential lines (coloring plan C10) ----
// Lines along the levels of the smooth count μ, `grid_width` output pixels wide and about
// `grid_spacing` apart wherever they are (`fractadyne_core::coloring::grid_weight`, the CPU twin):
// a texel d output pixels from the set is d · ln p pixels from the level one count away (|∇μ| =
// 1 / (d · ln p)), so it draws the lines at the multiples of 1/L, L the power of two that puts them
// spacing to twice that apart there, and fades in the next level's extra ones as the spacing widens.
// Each texel is shaded before a pixel's texels are averaged, its line coverage box-filtered over the
// tap (`tap_px`), so a line keeps its width at every anti-aliasing level. Faded out within 2 px of the
// set, and where μ's own f32 step (|μ| · 2^-23 of a count) reaches a pixel: there a line would move in
// whole-pixel steps (`GRID_PRECISION_FADE`). Interior texels, and those with no distance estimate,
// keep their color.

// The share of a tap `tap_px` across, centered `dist` output pixels from a line's center, that the
// line covers (`grid_cover`).
fn grid_cover(dist: f32) -> f32 {
    let r = tap_px;
    let lo = max(dist - 0.5 * r, -0.5 * cu.grid_width);
    let hi = min(dist + 0.5 * r, 0.5 * cu.grid_width);
    return clamp((hi - lo) / r, 0.0, 1.0);
}

// The coverage of the lines at the multiples of 1/`lines` (a power of two) for the count `mu`, `s`
// output pixels a count.
fn grid_level(mu: f32, s: f32, lines: f32) -> f32 {
    let f = abs(fract(lines * mu + 0.5) - 0.5) / lines;
    return grid_cover(f * s);
}

// `c`, the color of the texel `m`, under the equipotential lines.
fn grid_shaded(c: vec3<f32>, m: vec4<f32>) -> vec3<f32> {
    if (cu.grid_on == 0u || cu.aa_ln_p <= 0.0 || m.r < 0.0 || m.a > 1.0e20) { return c; }
    let d = exp2(min(m.a - de_px_shift, 60.0));   // the distance in output pixels
    let s = d * cu.aa_ln_p;                        // output pixels a count
    let lev = log2(max(s / max(cu.grid_spacing, 1.0), 1.0e-30));
    let l0 = clamp(floor(lev), -60.0, 60.0);
    let fr = clamp(lev - l0, 0.0, 1.0);
    // 2^l0 exactly (`ldexp` of a whole power), so `lines * mu` only shifts μ's exponent.
    let lines = ldexp(1.0, i32(l0));
    let cov = max(grid_level(m.r, s, lines), smoothstep(0.0, 1.0, fr) * grid_level(m.r, s, 2.0 * lines));
    let near = smoothstep(0.5, 2.0, d);
    let q = abs(m.r) * 1.1920929e-7 * s;           // μ's f32 step in output pixels
    // (`fine`, not `precise`: that is a reserved word in WGSL.)
    let fine = 1.0 - smoothstep(0.25, 1.0, q);
    return mix(c, cu.grid_col.xyz, clamp(cu.grid_strength * cov * near * fine, 0.0, 1.0));
}

// ---- The relief light's highlight (coloring plan C5) ----
// Blinn-Phong on the relief's surface (`fractadyne_core::coloring::relief_highlight`, the CPU twin):
// the surface normal (k·n, 1) for the unit slope direction `nu` and the slope `k`, the light `ld`
// round and `light_height` up, the eye straight above; (N·H)^shininess, H halfway between the light
// and the eye. It peaks where the surface tilts halfway toward the light.
fn relief_highlight(nu: vec2<f32>, k: f32, ld: vec2<f32>) -> f32 {
    let h = cu.light_height;
    let l = vec3<f32>(ld, h) / sqrt(1.0 + h * h);
    let hv = normalize(l + vec3<f32>(0.0, 0.0, 1.0));
    let nn = normalize(vec3<f32>(nu * k, 1.0));
    return pow(max(dot(nn, hv), 0.0), cu.relief_shininess);
}

@fragment
fn fs_color(in: VsOut) -> @location(0) vec4<f32> {
    let tex_dim = textureDimensions(iter_tex); // = (screen + apron) × ss
    let ss = max(cu.ss, 1u);
    // Texels a tap stands for: 1 (every texel) but for a palette-browser thumbnail at ss > 2.
    let stride = clamp(cu.tap_stride, 1u, ss);
    // An export tile's apron (`tile_apron`: its sides' widths in output pixels, a byte each, left,
    // top, right, bottom): pixels iterated past each side that has a neighboring tile, so the
    // relief's 3x3 neighborhood, and the palette anti-aliasing's next pixel, read the frame's
    // pixels there, not this tile's edge again. 0 but for a lit export's tiles (2026-11-26: a lit
    // export had a seam wherever fine detail crossed a tile's edge).
    let apron_lt = vec2<u32>(cu.tile_apron & 0xffu, (cu.tile_apron >> 8u) & 0xffu);
    let apron_rb = vec2<u32>((cu.tile_apron >> 16u) & 0xffu, (cu.tile_apron >> 24u) & 0xffu);
    let tex_px = tex_dim / ss;                     // the texture's pixels, the apron's included
    let screen_dim = tex_px - apron_lt - apron_rb; // the pixels this pass draws
    // ⭐The stored distance is per TEXEL; read it per OUTPUT pixel (`de_px_shift`). Texels across one
    // output pixel = the texture's width over the output's, times a reprojection's zoom since the
    // frozen frame (it magnifies the frozen texels). With no output width given that is log2(ss)
    // exactly — an export tile, the zoom-video assembler. Until 2026-11-08 nothing shifted, so a
    // 2×-supersampled view read every distance one octave too far: the Distance colors and the
    // glow bands moved whenever the anti-aliasing level, or the motion resolution, changed.
    let out_w = select(f32(tex_px.x), cu.display_w, cu.display_w > 0.0);
    let zoomed = select(1.0, cu.uv_scale, cu.reproject == 1u);
    de_px_shift = log2(max(f32(tex_dim.x) * zoomed, 1.0e-6) / max(out_w, 1.0));
    tap_px = max(cu.aa_scale * exp2(-de_px_shift) * f32(stride), 1.0e-3);
    // Pan reprojection: sample the frozen texture shifted by the panned offset, so the detailed
    // image slides with the cursor. Anything the pan drags in from outside the frozen frame
    // isn't rendered yet — fill it with the frame's average color (nicer than black) until the
    // view settles and re-iterates.
    var suv = in.uv;
    if (cu.reproject == 1u) {
        // Aspect-fit first: display the frozen (possibly old-size) iteration texture at NATIVE
        // scale, centered in the output rect, so a resize to a new aspect ratio doesn't stretch it —
        // the center stays centered and the revealed border fills with the average color below. When
        // the frozen frame matches the current size (pan / zoom reprojection) this is the identity.
        let fit = cu.out_res / max(vec2<f32>(screen_dim), vec2<f32>(1.0));
        suv = (in.uv - vec2<f32>(0.5)) * fit + vec2<f32>(0.5);
        // Then scale about the center + translate: follows both the zoom and pan since the frozen
        // frame was rendered (uv_scale == 1 → pure pan, the original behavior).
        suv = (suv - vec2<f32>(0.5)) * cu.uv_scale + vec2<f32>(0.5) - cu.uv_off;
        if (suv.x < 0.0 || suv.x > 1.0 || suv.y < 0.0 || suv.y > 1.0) {
            return vec4<f32>(view_average(), 1.0);
        }
    }
    let uv = clamp(suv, vec2<f32>(0.0), vec2<f32>(1.0));
    let pix = vec2<i32>(uv * vec2<f32>(screen_dim)) + vec2<i32>(apron_lt); // its pixel in the texture
    let maxc = vec2<i32>(tex_dim) - vec2<i32>(1, 1);

    // Analytic palette anti-aliasing footprint: how much palette PHASE this output pixel spans,
    // from the CONTINUOUS coord difference to the +1-pixel neighbors (one output pixel = ss
    // texels). Diffing the pre-`fract` coord (via `coord_of`) avoids wrap discontinuities;
    // interior neighbors (the INTERIOR_COORD sentinel) are skipped. It is computed ONCE per
    // output pixel, then each tap takes its own share of it: a tap is one texel, 1/ss of the pixel
    // across (and for progressive supersampling's sample k, 1/√(k+1) of that, `cu.aa_scale`).
    // ⛔The whole pixel's footprint on every tap, as this was until the palette anti-aliasing went
    // on by default (C8, 2026-11-13), filtered a supersampled frame twice, by the footprint and by
    // the taps' own average: blurrier than the pixel's true mean, and the more so the more samples.
    // Measured against 8x8 samples by the seahorse valley, ss 4: 0.0133 with it against 0.0106
    // point sampled. Zero when off ⇒ `shade_aa` is a point sample identical to the old `shade`.
    var pal_w = 0.0;
    var pal_wf = 0.0; // the same footprint under the map a dissolve fades from
    if (cu.aa_palette != 0u) {
        let o0 = clamp(pix * i32(ss), vec2<i32>(0, 0), maxc);
        // The next pixel right and below; at the texture's edge (the frame's, or an export tile's,
        // which holds nothing past it) the one before. Clamped, the edge pixel was its own neighbor,
        // its footprint 0, and an anti-aliased export had a point-sampled line at every tile seam.
        let past = (pix + vec2<i32>(1, 1)) * i32(ss) > maxc;
        let nx = select(pix.x + 1, pix.x - 1, past.x);
        let ny = select(pix.y + 1, pix.y - 1, past.y);
        let ox = clamp(vec2<i32>(nx, pix.y) * i32(ss), vec2<i32>(0, 0), maxc);
        let oy = clamp(vec2<i32>(pix.x, ny) * i32(ss), vec2<i32>(0, 0), maxc);
        let m0 = textureLoad(iter_tex, o0, 0);
        let a0 = textureLoad(aux_tex, o0, 0);
        let mx = textureLoad(iter_tex, ox, 0);
        let ax = textureLoad(aux_tex, ox, 0);
        let my = textureLoad(iter_tex, oy, 0);
        let ay = textureLoad(aux_tex, oy, 0);
        let c0 = coord_of(m0, a0);
        if (c0 < INTERIOR_COORD) {
            let cx = coord_of(mx, ax);
            let cy = coord_of(my, ay);
            if (cx < INTERIOR_COORD) { pal_w = max(pal_w, coord_step(cx - c0, cu.cycle)); }
            if (cy < INTERIOR_COORD) { pal_w = max(pal_w, coord_step(cy - c0, cu.cycle)); }
            if (cu.fade_weight > 0.0) {
                let f0 = coord_fade(m0, a0);
                if (cx < INTERIOR_COORD) { pal_wf = max(pal_wf, coord_step(coord_fade(mx, ax) - f0, cu.fade_cycle)); }
                if (cy < INTERIOR_COORD) { pal_wf = max(pal_wf, coord_step(coord_fade(my, ay) - f0, cu.fade_cycle)); }
            }
        }
        // Each tap's share: the differences span one pixel of ss texels, and a tap is one texel (and
        // its supersampling sample's share of that, `aa_scale`), or `stride` of them.
        let share = cu.aa_scale * f32(stride) / f32(ss);
        pal_w = pal_w * share;
        pal_wf = pal_wf * share;
        // Distance-sized (C8): the Smooth count's footprint from the pixel's own distance, where it
        // has one; the neighbor differences above stay for everything else. Its tap is one texel,
        // 2^-de_px_shift output pixels across (1/ss, more on a reduced frame or a reprojection's
        // magnified one), and its sample's share of that.
        let s = cu.aa_scale * exp2(-de_px_shift) * f32(stride);
        // Not with a band texture (E1): the statistic moves the position more than the count does
        // where its own texture is fine, which the next pixels' difference above sees and the
        // distance does not.
        if (cu.aa_palette == 2u && cu.color_method == METHOD_SMOOTH && cu.band_tex == 0u && c0 < INTERIOR_COORD) {
            let wd = de_footprint(m0, cu.cycle, cu.norm_mode, cu.norm_lo, s);
            if (wd >= 0.0) {
                pal_w = wd;
                if (cu.fade_weight > 0.0) {
                    // The same texel, so the fade map has a distance whenever this one does.
                    pal_wf = max(de_footprint(m0, cu.fade_cycle, cu.fade_mode, cu.fade_lo, s), 0.0);
                }
            }
        }
        // Taps under half a pixel are point sampled (`AA_MIN_TAP`), by either footprint.
        if (min(s, share) < AA_MIN_TAP) {
            pal_w = 0.0;
            pal_wf = 0.0;
        }
    }

    // Average a `taps×taps` block of iteration samples covering this pixel. With
    // supersampling, taps = ss (true SSAA). When the iteration texture was rendered
    // below display resolution (work-budget at deep zoom on a big window), `aa_filter`
    // widens the box so the upscaled, finely-banded texture is anti-aliased instead of
    // point-sampled into speckle. Also accumulates the slope normal (relief lighting).
    //
    // ⭐The box is CENTERED on this pixel's own texels. A box wider than them (`aa_filter`: a frame
    // rendered below display resolution) used to start at the pixel's first texel and run right and
    // down, so it drew the picture (taps - ss)/2 texels off: 6 screen px for a quarter-resolution
    // frame, shown between held frames that had no such offset, and the view's center jumped as they
    // alternated (2026-10-21). The overhang each side is whole or half a texel; a half one takes one
    // more texel each way at half weight. With no overhang (taps == ss) this is the plain block.
    let taps = max(ss, max(cu.aa_filter, 1u));
    let lead2 = i32(taps) - i32(ss);        // twice the overhang each side, texels
    let lead = (lead2 + 1) / 2;             // texels taken before the pixel's own
    let half_ends = (lead2 & 1) == 1;
    let n = i32(taps) + (lead2 & 1);        // samples a row
    var acc = vec3<f32>(0.0);
    var nacc = vec2<f32>(0.0);
    var dacc = 0.0;
    var count = 0.0;
    // The relief's slope from the distance (C5): the summed log distance, and weight, of the texels
    // that have one (escaped, a distance known).
    var kacc = 0.0;
    var kw = 0.0;
    // A stride above 1 reads every stride-th texel, the first half a stride in (ss 8, stride 4:
    // texels 2 and 6 of each row of eight); 1 is every texel, from the first.
    let tap_step = i32(stride);
    let tap_first = tap_step / 2;
    for (var dj: i32 = tap_first; dj < n; dj = dj + tap_step) {
        let wy = select(1.0, 0.5, half_ends && (dj == 0 || dj == n - 1));
        for (var di: i32 = tap_first; di < n; di = di + tap_step) {
            let w = wy * select(1.0, 0.5, half_ends && (di == 0 || di == n - 1));
            let texel = clamp(
                pix * i32(ss) + vec2<i32>(di - lead, dj - lead),
                vec2<i32>(0, 0),
                maxc,
            );
            let t = textureLoad(iter_tex, texel, 0);
            let ta = textureLoad(aux_tex, texel, 0);
            acc = acc + edge_shaded(grid_shaded(shade_aa(t, ta, pal_w, pal_wf), t), t) * w;
            nacc = nacc + vec2<f32>(t.g, t.b) * w;
            dacc = dacc + t.a * w;
            count = count + w;
            if (cu.relief_steep_px > 0.0 && t.r >= 0.0 && t.a < 1.0e20) {
                kacc = kacc + t.a * w;
                kw = kw + w;
            }
        }
    }
    var col = acc / count;
    // Distance-estimate relief lighting: light the surface whose normal is the
    // averaged slope. `light_height` raises the ambient floor (smaller = sharper).
    if (cu.light == 1u) {
        // ⭐⭐**THE RELIEF NORMAL IS LOW-PASSED ACROSS PIXELS, AND THE RADIUS IS THE MEASURED ONE.**
        // Lighting used to shade from `nacc / count` — the slope averaged over this pixel's
        // anti-aliasing taps alone. At depth that field has structure finer than the pixel grid,
        // so the Lambert term flickered between lit and back-facing on neighboring pixels and the
        // exterior stippled black. Reported at 3.1e34x; measured dark-pixel fraction attributable
        // to lighting was 28.84 pt at ss=1, 7.56 at ss=2, 2.03 at ss=4 — falling as 1/samples,
        // which is undersampling, not a numerical fault.
        //
        // ⛔**Averaging the TAPS harder cannot fix it, and that was measured before this was
        // written**: weighting the light by how much the taps agreed produced byte-identical
        // output, because the taps INSIDE a pixel already agree. The variation lives BETWEEN
        // adjacent pixels, so the filter has to span pixels — hence a 3x3 neighborhood in OUTPUT
        // pixel steps rather than a wider tap box.
        //
        // ⭐`|mean|` of unit normals is then a real confidence: 1 where the neighborhood agrees on
        // a surface, ~0 where it does not. Fading toward UNLIT there (`mix(1.0, lam, conf)`) means
        // an undersampled region loses its relief instead of gaining a random dark speckle — and
        // where the field IS well defined, `conf` is 1 and the result is unchanged.
        let ld = vec2<f32>(cos(cu.light_angle), sin(cu.light_angle));
        var nsm = vec2<f32>(0.0);
        var nn = 0.0;
        for (var oy: i32 = -1; oy <= 1; oy = oy + 1) {
            for (var ox: i32 = -1; ox <= 1; ox = ox + 1) {
                let q = clamp(
                    (pix + vec2<i32>(ox, oy)) * i32(ss),
                    vec2<i32>(0, 0),
                    maxc,
                );
                let tq = textureLoad(iter_tex, q, 0);
                nsm = nsm + vec2<f32>(tq.g, tq.b);
                nn = nn + 1.0;
            }
        }
        // Blend the neighborhood mean with this pixel's own tap mean so a genuinely sharp edge
        // keeps some of its own slope rather than being smeared entirely by its neighbors.
        let n = mix(nacc / count, nsm / nn, 0.75);
        let conf = length(n);
        if (conf > 1.0e-6) {
            let nu = n / conf;
            let diff = dot(nu, ld);
            // The surface's slope (C5, `fractadyne_core::coloring::relief_slope`): 1 everywhere, the
            // classic relief, whose shade is then (n·l + h) / (1 + h) as it always was; or from the
            // distance, 45 degrees `relief_steep_px` output pixels from the set, from this pixel's
            // mean log distance, flat (0) where none of its texels has one.
            var k = 1.0;
            if (cu.relief_steep_px > 0.0) {
                k = 0.0;
                if (kw > 0.0) {
                    k = clamp(cu.relief_steep_px * exp2(de_px_shift - kacc / kw), 0.0, 1.0e6);
                }
            }
            let lam = clamp((k * diff + cu.light_height) / (k + cu.light_height), 0.0, 1.0);
            let cf = clamp(conf, 0.0, 1.0);
            col = col * mix(1.0, lam, cf);
            // The highlight on that surface, as sure of itself as the shade (`cf`), and only where the
            // pixel has a distance: the set's inside, and a pixel with none, are flat and stay matte
            // (the 3x3 normal lights the inside's edge pixels too).
            if (cu.relief_highlight > 0.0 && kw > 0.0) {
                let hl = relief_highlight(nu, k, ld) * cu.relief_highlight * cf;
                col = min(col + hl * cu.relief_hl_col.xyz, vec3<f32>(1.0));
            }
        }
    }
    // Distance-estimate glow: bright contour bands at log-distance intervals from the
    // boundary; they densify near the filaments (→ glow) and flow when `de_phase`
    // animates. `de` channel carries log2(distance in texels), read here in output pixels; ≥20
    // means "far"/none.
    if (cu.de_on == 1u) {
        let dl = dacc / count - de_px_shift;
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

// ---------------- progressive on-settle supersampling (accumulate + present) ----------------
// While the view is settled and idle, the app re-renders it with a sub-pixel JITTER each frame
// (via `IterUniforms.px_offset`) and folds the shaded color into a running average, converging to
// the box-averaged (anti-aliased) image — the spatial supersampling that removes deep-zoom speckle,
// spread across idle frames so no single dispatch is large. `fs_accum` is the fold; `fs_present`
// blits the average to the surface. These reuse @group(0) slots the way the iterate and color
// passes already do (each pipeline binds only what its entry point reads).

struct AccumU {
    inv_weight: f32, // 1 / (sample_index + 1): the running-mean step for THIS sample
    _p0: f32,
    _p1: f32,
    _p2: f32,
};
@group(0) @binding(0) var<uniform> au: AccumU;
@group(0) @binding(1) var accum_frame: texture_2d<f32>; // this frame's freshly-shaded color
@group(0) @binding(2) var accum_prev: texture_2d<f32>;  // running average so far (ping-pong source)

// Numerically-stable running mean: avg += (sample - avg) / n. At n=1 (inv_weight=1) it returns the
// sample exactly, so a reset needs no clear — the first sample simply replaces the average.
@fragment
fn fs_accum(in: VsOut) -> @location(0) vec4<f32> {
    let dims = vec2<i32>(textureDimensions(accum_frame));
    let p = clamp(vec2<i32>(in.uv * vec2<f32>(dims)), vec2<i32>(0, 0), dims - vec2<i32>(1, 1));
    let cur = textureLoad(accum_frame, p, 0);
    let prev = textureLoad(accum_prev, p, 0);
    return prev + (cur - prev) * au.inv_weight;
}

// Present the accumulated average to the surface. Reuses the `accum_prev` texture slot (binding 2)
// rather than declaring a second sampled texture at the same slot; the present pipeline's bind
// group provides only binding 2 (the average to display).
@fragment
fn fs_present(in: VsOut) -> @location(0) vec4<f32> {
    let dims = vec2<i32>(textureDimensions(accum_prev));
    let p = clamp(vec2<i32>(in.uv * vec2<f32>(dims)), vec2<i32>(0, 0), dims - vec2<i32>(1, 1));
    return textureLoad(accum_prev, p, 0);
}

// ---------------- GPU primitive self-test (`--gputest`) ----------------
// Verifies the df32/floatexp building blocks THIS shader renders with, against CPU oracles —
// per machine. The error-free transforms above assume round-to-nearest with a FUSED fma; a
// driver or GPU where that assumption fails breaks every deep render silently, and until now
// it was verified only end-to-end by goldens on one GPU. Inputs derive from the pixel index by
// integer hashing + BITCAST (no floating arithmetic in construction, so the CPU reproduces
// them bit-exactly); one op family per ROW, one input set per COLUMN; each RGBA32F texel
// carries up to four f32 results. The CPU side (fractadyne-app `gputest`) regenerates the
// inputs and checks each op against an f64 / exact-EFT oracle.

fn gt_hash(x0: u32) -> u32 {
    var x = x0 * 747796405u + 2891336453u;
    x = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    return (x >> 22u) ^ x;
}
// An f32 assembled from hash bits: sign|exponent|mantissa, exponent confined to
// [emin, emin+espan) so each op sees a chosen magnitude band (never inf/NaN/denormal).
fn gt_f32(seed: u32, emin: i32, espan: u32) -> f32 {
    let mant = gt_hash(seed) & 0x007FFFFFu;
    let eb = u32(127 + emin + i32(gt_hash(seed ^ 0x9E3779B9u) % espan));
    let sign = (gt_hash(seed ^ 0x85EBCA6Bu) & 1u) << 31u;
    return bitcast<f32>(sign | (eb << 23u) | mant);
}
// A NORMALIZED df32 pair. `lo`'s exponent is derived from `hi`'s ACTUAL exponent so that
// |lo| <= ulp(hi)/2 — the invariant every df32 value in the renderer satisfies, and which
// df_mul/df_div's error analysis assumes. The first version of this drew lo from a fixed band
// and produced pairs with |lo| up to 52.8x over the limit, which makes a CORRECT df_mul look
// wrong (measured: 8.64e-13 against a 2.3e-13 tolerance) — a test that fails good code.
// Built from integer/bit manipulation only, never an error-free transform: normalizing via
// quick_two_sum would make the INPUTS depend on the very property under test, so a machine that
// folds EFTs would silently be measured on different inputs than the CPU oracle uses.
fn gt_df(seed: u32, emin: i32, espan: u32) -> vec2<f32> {
    let hi = gt_f32(seed, emin, espan);
    let he = i32((bitcast<u32>(hi) >> 23u) & 0xFFu) - 127;
    return vec2<f32>(hi, gt_f32(seed ^ 0xDEADBEEFu, he - 25, 1u));
}

// Identity that survives algebraic optimizers: a round-trip through the integer domain.
// Reassociation licenses (`(a+b)-a → b`) operate on float expressions; no mainstream
// compiler rewrites across a bitcast. Used by the armored EFT rows below to DIAGNOSE
// whether a wrong two_sum on this machine is compiler reassociation (armored row passes)
// or genuinely broken rounding (armored row fails too).
fn gt_opaque(x: f32) -> f32 {
    return bitcast<f32>(bitcast<u32>(x));
}
fn gt_two_sum_armored(a: f32, b: f32) -> vec2<f32> {
    let s = gt_opaque(a + b);
    let v = gt_opaque(s - a);
    let e = gt_opaque(a - gt_opaque(s - v)) + gt_opaque(b - v);
    return vec2<f32>(s, e);
}

// ---- Rows 19–23: the UNIFORM-DERIVED barrier (experiment, TODO PR-A1) ----
// `gt_opaque` above is provably the identity, so a compiler may fold through it — and NVIDIA's
// does. These rows OR in bits derived from a uniform the host sets to 1.0: they are zero at run
// time, but no compiler can prove it, so the expression stays opaque. Reported to restore
// double-float precision under DXC fast-math and Metal (October 2026); untested on NVIDIA's and
// AMD's SPIR-V compilers until these rows. OR rather than XOR, so nested barriers cannot cancel.
// `gtu` is bound by the gputest pass only (group 0 slot 0 is reused per entry point, as `iu`, `cu`
// and `au` do).
@group(0) @binding(0) var<uniform> gtu: vec4<f32>;
fn ub(x: f32) -> f32 {
    let zero_bits = bitcast<u32>(gtu.x) ^ 0x3f800000u;
    return bitcast<f32>(bitcast<u32>(x) | zero_bits);
}
fn ub_quick_two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = ub(a + b);
    let e = b - ub(s - a);
    return vec2<f32>(s, e);
}
fn ub_two_sum(a: f32, b: f32) -> vec2<f32> {
    let s = ub(a + b);
    let v = ub(s - a);
    let e = ub(a - ub(s - v)) + (b - v);
    return vec2<f32>(s, e);
}
fn ub_two_prod(a: f32, b: f32) -> vec2<f32> {
    let p = ub(a * b);
    let e = fma(a, b, -p);
    return vec2<f32>(p, e);
}
fn ub_df_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let s = ub_two_sum(a.x, b.x);
    let t = ub_two_sum(a.y, b.y);
    let c = ub(s.y + t.x);
    let v = ub_quick_two_sum(s.x, c);
    let w = ub(t.y + v.y);
    return ub_quick_two_sum(v.x, w);
}
fn ub_df_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let p = ub_two_prod(a.x, b.x);
    let e = ub(p.y + ub(a.x * b.y + a.y * b.x));
    return ub_quick_two_sum(p.x, e);
}
fn ub_df_mul_f32(a: vec2<f32>, b: f32) -> vec2<f32> {
    let p = ub_two_prod(a.x, b);
    let e = ub(p.y + a.y * b);
    return ub_quick_two_sum(p.x, e);
}
fn ub_c_sqr(a: Cdf) -> Cdf {
    let ri = ub_df_mul(a.re, a.im);
    let im2 = ub_df_mul(a.im, a.im);
    return cset(ub_df_add(ub_df_mul(a.re, a.re), vec2<f32>(-im2.x, -im2.y)), ub_df_add(ri, ri));
}
// An input set for the rebase-test rows (13–15): (z.x, z.y, δz.x, δz.y), hi limbs. By class: one in
// eight decisive (independent magnitudes), one in eight tiny (components near 2^-68, whose squares
// fall below f32's normal range: the lift's case in `rebase_df32`), the rest near ties (δz = z's
// components swapped, up to four low mantissa bits flipped in each).
fn gt_rebase_case(t: u32) -> vec4<f32> {
    let cls = gt_hash(t ^ 0x51ED270Bu) % 8u;
    let emin = select(-8, -72, cls == 1u);
    let a = gt_f32(t * 4u + 1u, emin, 8u);
    let b = gt_f32(t * 4u + 2u, emin, 8u);
    if (cls == 0u) {
        return vec4<f32>(a, b, gt_f32(t * 4u + 3u, -8, 8u), gt_f32(t * 4u + 4u, -8, 8u));
    }
    let h = gt_hash(t ^ 0x2545F491u);
    return vec4<f32>(a, b, bitcast<f32>(bitcast<u32>(b) ^ (h & 15u)), bitcast<f32>(bitcast<u32>(a) ^ ((h >> 4u) & 15u)));
}
fn gt_cdf(c: vec2<f32>) -> Cdf {
    return cset(vec2<f32>(c.x, 0.0), vec2<f32>(c.y, 0.0));
}

// Rows 13–17 of `fs_gputest`: the decisions the rebase and BLA-skip COUNTERS count (U26). Each is
// the renderer's own predicate, one decision per call (1 = true), four calls per texel, mostly on
// near ties, where a compiler that fuses a sum of squares differently, a sqrt or a log2 that is not
// correctly rounded, or subnormals flushed or kept decides otherwise. The CPU names the arithmetic
// that reproduces every decision, so whether a counter means the same thing on this backend is a
// reading. One function per row and no loop: FXC (DX12) aborts on the looped form.
//
// Row 13 — rebase test, df32 path (mode 0): `rebase_df32`, called as both kernels call it.
fn gt_row13(t: u32) -> f32 {
    let c = gt_rebase_case(t);
    let zf = c.xy;
    return select(0.0, 1.0, rebase_df32(dot(zf, zf), zf, gt_cdf(c.zw)));
}
// Row 14 — rebase test, floatexp (mode 2): `sf_lt(fe_abs_sf(z), fe_abs_sf(δz))`, the composition
// every floatexp rebase site calls, at an extended exponent both share.
fn gt_row14(t: u32) -> f32 {
    let c = gt_rebase_case(t);
    let e = i32(gt_hash(t ^ 0x68E31DA4u) % 6000u) - 3000;
    let z = fe_norm(gt_cdf(c.xy), e);
    let dz = fe_norm(gt_cdf(c.zw), e);
    return select(0.0, 1.0, sf_lt(fe_abs_sf(z), fe_abs_sf(dz)));
}
// Row 15 — rebase test, df32 tail (mode 2): `tail_rebase`.
fn gt_row15(t: u32) -> f32 {
    let c = gt_rebase_case(t);
    return select(0.0, 1.0, tail_rebase(gt_cdf(c.xy), gt_cdf(c.zw)));
}
// Row 16 — BLA validity test (mode 2): `sf_lt(fe_abs_sf(δz), radius)`, as the BLA walk calls it.
// δz's parts are a large x and a small y (zero, or about 2^-12 below x), in either order so that
// both fused forms are exercised, and the radius is |x| with its last bits moved by up to four: a
// near tie that a fused sum of squares rounds otherwise. One in eight radii sits a binade away
// instead (decisive).
fn gt_row16(t: u32) -> f32 {
    let x = gt_f32(t * 4u + 1u, -8, 8u);
    let ex = i32((bitcast<u32>(x) >> 23u) & 0xFFu) - 127;
    let y = select(0.0, gt_f32(t * 4u + 2u, ex - 13, 2u), (gt_hash(t ^ 0x1B873593u) & 1u) == 1u);
    let h = gt_hash(t ^ 0xCC9E2D51u);
    let base = bitcast<u32>(x) & 0x7FFFFFFFu;
    var rbits = base + (h % 9u) - 4u;
    if (((h >> 8u) % 8u) == 0u) { rbits = select(base + 0x800000u, base - 0x800000u, ((h >> 11u) & 1u) == 1u); }
    let r = bitcast<f32>(rbits);
    var c = vec2<f32>(x, y);
    if (((h >> 12u) & 1u) == 1u) { c = vec2<f32>(y, x); }
    let e = i32(gt_hash(t ^ 0x85EBCA77u) % 6000u) - 3000;
    let dz = fe_norm(gt_cdf(c), e);
    return select(0.0, 1.0, sf_lt(fe_abs_sf(dz), sf_norm(vec2<f32>(r, 0.0), e)));
}
// Row 17 — the df32 tail's BLA exit: `tail_no_skip`, with the GPU's own log2. The radius is 2^j
// and δz's larger part sits within a few last bits of 2^(j + k + 0.01), so the decision turns on
// log2's last bits. One in sixteen has no radius (always true).
fn gt_row17(t: u32) -> f32 {
    let j = i32(gt_hash(t ^ 0x7FEB352Du) % 60u) - 30;
    let kk = i32(gt_hash(t ^ 0x846CA68Bu) % 41u) - 20;
    let jit = gt_hash(t ^ 0x2C1B3C6Du) % 97u;
    let sign = (gt_hash(t ^ 0x165667B1u) & 1u) << 31u;
    let m = bitcast<f32>(sign | ((u32(127 + j + kk) << 23u) + 58300u + jit));
    let other = bitcast<f32>((u32(126 + j + kk) << 23u) | (gt_hash(t ^ 0x27D4EB2Fu) & 0x7FFFFFu));
    let none = (gt_hash(t ^ 0x9E3779B1u) % 16u) == 0u;
    let r = select(bitcast<f32>(u32(127 + j) << 23u), 0.0, none);
    var dzd = gt_cdf(vec2<f32>(m, other));
    if ((t & 1u) == 1u) { dzd = gt_cdf(vec2<f32>(other, m)); }
    return select(0.0, 1.0, tail_no_skip(dzd, vec4<f32>(0.0, 0.0, f32(kk), r)));
}
// Row 18 — `slope_normal`, the unit normal an escaped pixel is lit by, over the range the DIRECT
// path's derivative reaches: plain f32, up to 2^127. z's parts lie in [2^-6, 2^18), the derivative
// is scaled to 2^e with e in [-4, 127), so |z|·|dz| runs from about 2^-10 past f32's top, about half
// of it over 2^64, where `ur² + ui²` overflows (2026-10-30: a (0, 0) normal on a direct strip).
fn gt_normal_case(t: u32) -> vec2<f32> {
    let e = i32(gt_hash(t ^ 0x3C6EF372u) % 131u) - 4;
    let z = vec2<f32>(gt_f32(t * 4u + 1u, -6, 24u), gt_f32(t * 4u + 2u, -6, 24u));
    let d = vec2<f32>(gt_f32(t * 4u + 3u, e - 8, 9u), gt_f32(t * 4u + 4u, e - 8, 9u));
    return slope_normal(z, d);
}
// Does this device's compiler fuse two_prod's product into the add that consumes it (see `ob`)?
// Each texel is one df_mul of a hashed pair (`fs_gputest` row 3's operands), computed with the add
// residuals dropped and no barrier; `fusion::detect_product_fusion` compares the results with the
// code as written and compiles the barrier into this device's iterate pipelines if they differ.
@fragment
fn fs_fusion_probe(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let s = u32(pos.x) + 100003u * 3u;
    let r = df_mul(gt_df(s * 8u + 3u, -4, 8u), gt_df(s * 8u + 5u, -4, 8u));
    return vec4<f32>(r.x, r.y, 0.0, 0.0);
}
@fragment
fn fs_gputest(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    ob_zero = bitcast<u32>(gtu.x) ^ 0x3f800000u; // arm `ob` from this pass's own uniform (1.0)
    let ix = u32(pos.x);
    let op = u32(pos.y);
    let s = ix + 100003u * op; // decorrelate rows
    let a = gt_f32(s * 2u + 1u, -8, 16u);
    let b = gt_f32(s * 2u + 2u, -8, 16u);
    let ar = gt_df(s * 8u + 3u, -4, 8u);
    let ai = gt_df(s * 8u + 4u, -4, 8u);
    let br = gt_df(s * 8u + 5u, -4, 8u);
    let bi = gt_df(s * 8u + 6u, -4, 8u);
    switch op {
        case 0u: { let r = two_sum(a, b); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 1u: { let r = two_prod(a, b); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 2u: { let r = df_add(ar, br); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 3u: { let r = df_mul(ar, br); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 4u: { let r = df_div(ar, br); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 5u: {
            let z = c_sqr(cset(ar, ai));
            return vec4<f32>(z.re.x, z.re.y, z.im.x, z.im.y);
        }
        case 6u: {
            // fe_mul across a wide exponent spread; im.lo is dropped (4 output slots).
            let ea = i32(gt_hash(s ^ 17u) % 200u) - 100;
            let eb2 = i32(gt_hash(s ^ 23u) % 200u) - 100;
            let r = fe_mul(fe_norm(cset(ar, ai), ea), fe_norm(cset(br, bi), eb2));
            return vec4<f32>(r.m.re.x, r.m.re.y, f32(r.e), r.m.im.x);
        }
        case 7u: {
            // fe_add with exponent gaps up to ±40 (exercises the align/renorm path).
            let ea = i32(gt_hash(s ^ 31u) % 80u) - 40;
            let r = fe_add(fe_norm(cset(ar, ai), ea), fe_norm(cset(br, bi), 0));
            return vec4<f32>(r.m.re.x, r.m.re.y, f32(r.e), r.m.im.x);
        }
        case 8u: {
            // 64-step CONTRACTED Mandelbrot-form accumulation: z ← (z²)·2⁻¹ + c. The halving
            // (exact) keeps it bounded without branches, so the test measures pure df32
            // accumulation over a long dependent chain — the regime the goldens can't isolate.
            let cr = gt_df(s * 8u + 3u, -3, 3u);
            let ci = gt_df(s * 8u + 4u, -3, 3u);
            var z = cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0));
            for (var i = 0; i < 64; i++) {
                let q = c_sqr(z);
                z = c_add(cset(df_mul_f32(q.re, 0.5), df_mul_f32(q.im, 0.5)), cset(cr, ci));
            }
            return vec4<f32>(z.re.x, z.re.y, z.im.x, z.im.y);
        }
        case 9u: {
            // Julia-form accumulation: the per-pixel identity enters ONLY through z₀ (the
            // 2026-08-13 Julia direct-mode failure class); fixed c from exact literals.
            let cfix = cset(
                vec2<f32>(-0.7436, 0.0), vec2<f32>(0.1318, 0.0));
            var z = cset(gt_df(s * 8u + 3u, -3, 3u), gt_df(s * 8u + 4u, -3, 3u));
            for (var i = 0; i < 64; i++) {
                let q = c_sqr(z);
                z = c_add(cset(df_mul_f32(q.re, 0.5), df_mul_f32(q.im, 0.5)), cfix);
            }
            return vec4<f32>(z.re.x, z.re.y, z.im.x, z.im.y);
        }
        case 10u: {
            // two_sum with bitcast armor — the reassociation discriminator (see gt_opaque).
            let r = gt_two_sum_armored(a, b);
            return vec4<f32>(r.x, r.y, 0.0, 0.0);
        }
        case 11u: {
            // quick_two_sum ISOLATED. It is the other error-free transform the renderer leans on
            // — df_mul, df_div and df_add all end in it — and it was untested here while two_sum
            // was covered, which left a gap exactly where the 2026-08-14 AMD result is
            // unexplained (EFTs exact, yet df_mul only f32-accurate). Its shape
            // `e = b - ((a+b) - a)` is even easier to fold than two_sum's. Inputs are ORDERED by
            // magnitude because the algorithm requires |a| >= |b| — feeding it unordered pairs
            // would make a correct implementation look broken.
            let hi = select(b, a, abs(a) >= abs(b));
            let lo = select(a, b, abs(a) >= abs(b));
            let r = quick_two_sum(hi, lo);
            return vec4<f32>(r.x, r.y, 0.0, 0.0);
        }
        case 12u: {
            // df_mul DISSECTED. Every primitive above passes in isolation on AMD, yet df_mul
            // only reaches f32 accuracy there — so the precision is lost in the COMPOSITION, and
            // guessing which step has already been wrong twice. This row re-implements df_mul
            // inline and emits its three intermediates so the CPU side can say exactly which one
            // died:
            //   p.x  the rounded product          (always fine)
            //   p.y  two_prod's residual          zero here => inlining let the optimizer
            //                                     simplify fma(a,b,-(a*b)) to 0
            //   e    p.y + the cross terms        zero here => the cross-term line collapsed
            //   r.y  quick_two_sum's residual     zero here => the final renormalize folded
            // Exactly one of these being zero identifies the culprit; the measured f32-level
            // error is what you get if any of them is zero.
            let p = two_prod(ar.x, br.x);
            let e = p.y + (ar.x * br.y + ar.y * br.x);
            let r = quick_two_sum(p.x, e);
            return vec4<f32>(p.x, p.y, e, r.y);
        }
        case 13u: { return vec4<f32>(gt_row13(s * 4u), gt_row13(s * 4u + 1u), gt_row13(s * 4u + 2u), gt_row13(s * 4u + 3u)); }
        case 14u: { return vec4<f32>(gt_row14(s * 4u), gt_row14(s * 4u + 1u), gt_row14(s * 4u + 2u), gt_row14(s * 4u + 3u)); }
        case 15u: { return vec4<f32>(gt_row15(s * 4u), gt_row15(s * 4u + 1u), gt_row15(s * 4u + 2u), gt_row15(s * 4u + 3u)); }
        case 16u: { return vec4<f32>(gt_row16(s * 4u), gt_row16(s * 4u + 1u), gt_row16(s * 4u + 2u), gt_row16(s * 4u + 3u)); }
        case 17u: { return vec4<f32>(gt_row17(s * 4u), gt_row17(s * 4u + 1u), gt_row17(s * 4u + 2u), gt_row17(s * 4u + 3u)); }
        case 18u: {
            let a = gt_normal_case(s * 2u);
            let b = gt_normal_case(s * 2u + 1u);
            return vec4<f32>(a.x, a.y, b.x, b.y);
        }
        // Rows 19–23 mirror rows 0, 11, 2, 3 and 8 with the uniform-derived barrier (see `ub`).
        case 19u: { let r = ub_two_sum(a, b); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 20u: {
            let hi = select(b, a, abs(a) >= abs(b));
            let lo = select(a, b, abs(a) >= abs(b));
            let r = ub_quick_two_sum(hi, lo);
            return vec4<f32>(r.x, r.y, 0.0, 0.0);
        }
        case 21u: { let r = ub_df_add(ar, br); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 22u: { let r = ub_df_mul(ar, br); return vec4<f32>(r.x, r.y, 0.0, 0.0); }
        case 23u: {
            let cr = gt_df(s * 8u + 3u, -3, 3u);
            let ci = gt_df(s * 8u + 4u, -3, 3u);
            var z = cset(vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 0.0));
            for (var i = 0; i < 64; i++) {
                let q = ub_c_sqr(z);
                let h = cset(ub_df_mul_f32(q.re, 0.5), ub_df_mul_f32(q.im, 0.5));
                z = cset(ub_df_add(h.re, cr), ub_df_add(h.im, ci));
            }
            return vec4<f32>(z.re.x, z.re.y, z.im.x, z.im.y);
        }
        default: { return vec4<f32>(0.0, 0.0, 0.0, 0.0); }
    }
}
