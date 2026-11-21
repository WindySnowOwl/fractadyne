// ---- The coloring hook's interface (fractadyne_gpu::color_hook, design/coloring-plan.md B10) ----
// Spliced after the coloring pass, which `color_hook::build` cut out of the fixed module with its
// `shade_aa` and `shade` renamed `builtin_shade_aa` and `builtin_shade`: the two below take their
// places, so `fs_color` (and the reprojection's `view_average`) color every sample through the
// hook's `custom_color`. The fixed module itself never sees this file.

// One sample of the picture (a texel of the iteration texture), as the hook's `custom_color` gets
// it. The lanes are the ones `--render-lanes` writes, by the same names.
struct Lanes {
    // The smooth iteration count; below 0 in the set (-1) and on a glitched texel (-2).
    count: f32,
    // The slope normal, the direction of z / (dz/dc): a unit vector where there is one.
    normal_x: f32,
    normal_y: f32,
    // log2 of the distance to the set in OUTPUT pixels (1e20 or more where there is none).
    de_log2: f32,
    // The orbit statistics. A hook that reads one gets it accumulated, whatever the method.
    stripe: f32,
    triangle: f32,
    trap: f32,
    angle: f32,
    // In the set (count below 0).
    inside: bool,
    // The view's method's palette position here (its value under Cycle, Offset and the range
    // map), 1e30 inside.
    coord: f32,
    // The color the view's method gives this sample without a hook, inside included.
    color: vec3<f32>,
}

// The hook's four parameters, the sliders the dialog shows (`// @param` lines name them).
fn p1() -> f32 { return cu.hook_p.x; }
fn p2() -> f32 { return cu.hook_p.y; }
fn p3() -> f32 { return cu.hook_p.z; }
fn p4() -> f32 { return cu.hook_p.w; }

// The view's palette map applied to any value: the range map (log when the view uses it), then
// the rate and Offset, exactly as the view's METHOD maps its own value. `palette(palette_map(l.count))`
// is Smooth. The same arithmetic as `coord_under`, so it gives the same bits. ⚠The rate is the
// method's: under Smooth it is palettes per ITERATION (a few hundredths), so a value from 0 to 1
// (a statistic, a direction in turns) hardly moves through the palette under it. Map such a value
// with a rate of your own instead: `palette(v * rate + palette_offset())`.
fn palette_map(v: f32) -> f32 {
    var x = v;
    if (cu.norm_mode == 1u) {
        x = log(max(v - cu.norm_lo, 0.0) + 1.0);
    }
    return x * cu.cycle + cu.offset;
}

// The view's Offset (where the palette starts), for a value a hook maps at its own rate.
fn palette_offset() -> f32 {
    return cu.offset;
}

// The inside palette at position `t` (wrapped): the view's own, else the outside palette.
fn palette_inside(t: f32) -> vec3<f32> {
    if (cu.lut2_len > 0u) {
        return palette_in(t, LUT2_BASE, cu.lut2_len, cu.lut2_smooth);
    }
    return palette(t);
}

fn hook_lanes(m: vec4<f32>, a: vec4<f32>, col: vec3<f32>) -> Lanes {
    var l: Lanes;
    l.count = m.r;
    l.normal_x = m.g;
    l.normal_y = m.b;
    l.de_log2 = m.a - de_px_shift;
    l.stripe = a.r;
    l.triangle = a.g;
    l.trap = a.b;
    l.angle = a.a;
    l.inside = m.r < 0.0;
    l.coord = coord_of(m, a);
    l.color = col;
    return l;
}

// A hook's color, clamped to 0..1. A channel that is not a finite number (a division by zero, the
// log of a negative) would spread through every average it joins (the supersampling taps,
// progressive supersampling's running mean), so it is black instead: tested on the bits, which no
// compiler folds away as it may fold `c != c`.
fn hook_finite(c: vec3<f32>) -> vec3<f32> {
    let e = bitcast<vec3<u32>>(c) & vec3<u32>(0x7f800000u);
    return select(vec3<f32>(0.0), clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), e != vec3<u32>(0x7f800000u));
}

fn shade_aa(m: vec4<f32>, a: vec4<f32>, width: f32, width_fade: f32) -> vec3<f32> {
    return hook_finite(custom_color(hook_lanes(m, a, builtin_shade_aa(m, a, width, width_fade))));
}

fn shade(m: vec4<f32>, a: vec4<f32>) -> vec3<f32> {
    return hook_finite(custom_color(hook_lanes(m, a, builtin_shade(m, a))));
}
