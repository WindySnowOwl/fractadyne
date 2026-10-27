// ---------------- exponential-map strips (spliced by strip.rs) ----------------
// `design/zoom-video.md` §3-4; `fractadyne_core::ExpMap` is the CPU side of the geometry.
//
// This file is appended to the iterate module ONLY for a strip's own pipelines, and strip.rs fills
// mandelbrot.wgsl's STRIP_* slots with calls into it. Every other pipeline compiles mandelbrot.wgsl
// exactly as it is: the same code written into the shared module — even compiled out with an
// `override` — cost every other render 1-2% (37-38 of 43 bench-matrix segments slower on both of
// PLUTO's cards, 2026-10-27, against an A/A control of 24/43 and 16/43).
//
// Texel (gx, gy) of a W-wide strip — gx, gy the texel's position in the full grid, integer + 0.5 at
// its centre plus any jitter — is at angle 2π·gx/W anticlockwise from the positive real axis and
// radius map_r·exp(−map_s·gy) from the centre, map_s = 2π/W, row 0 outermost, in units of
// 2^delta_exp. Formed in f32: a strip texel is ~1e-3 of its radius, so f32 places a sample to ~1e-4
// of a texel — it moves the sample, never the arithmetic that follows from it.

// log2 of this pixel's texel side in the plane, for the distance estimate (`de_log2`'s slot):
// set by the prologue that places the texel, or by `fs_resolve`'s slot.
var<private> strip_steplog: f32;

struct StripOff {
    re: vec2<f32>,
    im: vec2<f32>,
}

fn strip_set_steplog(gy: f32) {
    strip_steplog = log2(iu.map_r * iu.map_s) - iu.map_s * gy * 1.4426950 + f32(iu.delta_exp);
}

// The texel's offset from the centre, as df32 mantissas (×2^delta_exp), and its size.
fn strip_offset(gx: f32, gy: f32) -> StripOff {
    let r = iu.map_r * exp(-iu.map_s * gy);
    let cs = cos_sin_turns(gx / iu.res.x);
    strip_set_steplog(gy);
    return StripOff(vec2<f32>(r * cs.x, 0.0), vec2<f32>(r * cs.y, 0.0));
}

// (cos, sin) of `t` turns, to ~1e-7. ⚠Not the built-in cos/sin: WGSL promises those only to 2^-11
// absolute, which on a 6,921-texel strip could move a sample by half a texel. The angle is reduced
// to the nearest quarter turn EXACTLY (in turns, before any π: `4t` and its rounding are exact), so
// the polynomials see |x| ≤ π/4, where the cos series to x⁸ and the sin series to x⁹ are good to
// 3e-8.
fn cos_sin_turns(t: f32) -> vec2<f32> {
    let u = t - floor(t);
    let q = u * 4.0;
    let qi = floor(q + 0.5);
    let x = (q - qi) * 1.5707963;
    let x2 = x * x;
    let c = 1.0 + x2 * (-0.5 + x2 * (0.041666668 + x2 * (-0.0013888889 + x2 * 0.0000248016)));
    let s = x * (1.0 + x2 * (-0.16666667 + x2 * (0.008333334 + x2 * (-0.00019841270 + x2 * 0.0000027557319))));
    switch (u32(qi) & 3u) {
        case 1u: { return vec2<f32>(-s, c); }
        case 2u: { return vec2<f32>(-c, -s); }
        case 3u: { return vec2<f32>(s, -c); }
        default: { return vec2<f32>(c, s); }
    }
}

// A strip's aux target, replacing the colouring statistics: the EXACT iteration count as two small
// integers (as the chunk state keeps it, `info_pack`: a u32 bitcast could be flushed as a denormal),
// the smooth count's fraction `1 − nu`, and the final z's angle in turns — zoomasm's N, NF and T
// (`design/zoom-video.md` §5.1). The main target keeps the smooth count as one f32 as everywhere,
// which past ~2^20 iterations has lost most of its fraction. Written by `fs_resolve` only: a strip
// always renders on the chunked path.
fn strip_aux(iter: u32, frac: f32, zf: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(f32(iter >> 12u), f32(iter & 4095u), frac, atan2(zf.y, zf.x) * 0.15915494 + 0.5);
}

// The escape-time power of the built-in formula, as the chunk passes choose it (for `fs_resolve`,
// which recomputes a strip's fraction from the stored z).
fn chunk_power() -> f32 {
    if (iu.formula == 1u) { return 3.0; }
    if (iu.formula == 2u) { return 4.0; }
    if (iu.formula == 3u) { return 5.0; }
    if (fam_holo(iu.formula)) { return f32(fam_power(iu.formula)); }
    return 2.0;
}
