// L-system segment pass (design/lsystems.md §5): each segment a quad, widened to the line width
// with round ends, into the iteration texture — so the ordinary colour pass (`fs_color`) colours
// it. Writes what an escape-time pass writes: main = (value, 0, 0, 1e30), aux = AUX_NONE; the
// texture is cleared to "interior" (main.r < 0) first, so what no line covers is the background.
// Commits the escape-range counters the live normalization reads.

struct LView {
    // A walked pixel p (from the walked view's centre, y up) shows at p * scale + offset pixels from
    // this view's centre: the last walk, drawn under a view that has moved since.
    offset: vec2<f32>,
    scale: f32,
    // Half the line width, texels.
    half: f32,
    // The target, texels; texels a pixel.
    size: vec2<f32>,
    ss: f32,
    // How much of the curve is drawn (1: all): a segment past it is hidden, the one it falls in
    // shortened to it, a filled shape shown once it is passed.
    progress: f32,
};

// A vertex no fragment lands on (every corner of a hidden instance is this one point).
const HIDDEN: vec4<f32> = vec4<f32>(-2.0, -2.0, 0.0, 1.0);

@group(1) @binding(0) var<uniform> V: LView;

// The iterate passes' event counters (group 0 = the view's iterate bind group; binding 2), as the
// Life display pass commits them.
@group(0) @binding(2) var<storage, read_write> counters: array<atomic<u32>>;
const CTR_ESC_MIN: u32 = 5u;
const CTR_ESC_MAX: u32 = 6u;
const CTR_ESC_COUNT: u32 = 7u;

const AUX_NONE: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0e30, 0.0);

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    // The segment's ends in texels (y down), for the fragment's distance.
    @location(0) @interpolate(flat) a: vec2<f32>,
    @location(1) @interpolate(flat) b: vec2<f32>,
    @location(2) @interpolate(flat) value: f32,
};

// Walked pixels -> this target's texels (y down).
fn to_texel(p: vec2<f32>) -> vec2<f32> {
    let q = (p * V.scale + V.offset) * V.ss;
    return vec2<f32>(0.5 * V.size.x + q.x, 0.5 * V.size.y - q.y);
}

@vertex
fn vs_segment(
    @builtin(vertex_index) vi: u32,
    @location(0) pa: vec2<f32>,
    @location(1) pb_in: vec2<f32>,
    @location(2) value: f32,
    @location(3) t: vec2<f32>,
) -> VsOut {
    var out: VsOut;
    if (t.x >= V.progress) {
        out.pos = HIDDEN;
        out.a = vec2<f32>(0.0);
        out.b = vec2<f32>(0.0);
        out.value = value;
        return out;
    }
    // The segment the drawing has reached: as far along it as the drawing has got.
    var pb = pb_in;
    if (t.y > V.progress) {
        pb = pa + (pb_in - pa) * clamp((V.progress - t.x) / (t.y - t.x), 0.0, 1.0);
    }
    let a = to_texel(pa);
    let b = to_texel(pb);
    let ab = b - a;
    let len = length(ab);
    var d = vec2<f32>(1.0, 0.0);
    if (len > 1.0e-6) {
        d = ab / len;
    }
    let n = vec2<f32>(-d.y, d.x);
    // A rectangle around the segment, a half-width beyond it every way (the round ends' room).
    let h = V.half + 0.5;
    var corner = array<vec2<f32>, 6>(
        vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let c = corner[vi];
    let along = select(-h, len + h, c.x > 0.5);
    let p = a + d * along + n * (c.y * h);
    out.pos = vec4<f32>(p.x / V.size.x * 2.0 - 1.0, 1.0 - p.y / V.size.y * 2.0, 0.0, 1.0);
    out.a = a;
    out.b = b;
    out.value = value;
    return out;
}

struct FragOut {
    @location(0) main: vec4<f32>,
    @location(1) aux: vec4<f32>,
};

// A filled polygon's triangles (drawn before the lines, so lines lie over fills).
struct TriOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) value: f32,
};

@vertex
fn vs_triangle(
    @builtin(vertex_index) vi: u32,
    @location(0) pa: vec2<f32>,
    @location(1) pb: vec2<f32>,
    @location(2) pc: vec2<f32>,
    @location(3) value: f32,
    @location(4) t: f32,
) -> TriOut {
    var corners = array<vec2<f32>, 3>(pa, pb, pc);
    let p = to_texel(corners[vi]);
    var out: TriOut;
    out.pos = vec4<f32>(p.x / V.size.x * 2.0 - 1.0, 1.0 - p.y / V.size.y * 2.0, 0.0, 1.0);
    if (t >= V.progress) {
        out.pos = HIDDEN;
    }
    out.value = value;
    return out;
}

@fragment
fn fs_triangle(in: TriOut) -> FragOut {
    let tx = vec2<i32>(in.pos.xy);
    if ((tx.x & 3) == 0 && (tx.y & 3) == 0) {
        let bits = bitcast<u32>(in.value);
        atomicMin(&counters[CTR_ESC_MIN], bits);
        atomicMax(&counters[CTR_ESC_MAX], bits);
        atomicAdd(&counters[CTR_ESC_COUNT], 1u);
    }
    return FragOut(vec4<f32>(in.value, 0.0, 0.0, 1.0e30), AUX_NONE);
}

@fragment
fn fs_segment(in: VsOut) -> FragOut {
    let p = in.pos.xy; // the texel centre
    let ab = in.b - in.a;
    let l2 = dot(ab, ab);
    var t = 0.0;
    if (l2 > 0.0) {
        t = clamp(dot(p - in.a, ab) / l2, 0.0, 1.0);
    }
    if (distance(p, in.a + t * ab) > V.half) {
        discard;
    }
    // The iterate passes' 4x4 subsampling grid (`ESC_COUNT_SUBSAMPLE`): the count is read as theirs.
    let tx = vec2<i32>(p);
    if ((tx.x & 3) == 0 && (tx.y & 3) == 0) {
        let bits = bitcast<u32>(in.value);
        atomicMin(&counters[CTR_ESC_MIN], bits);
        atomicMax(&counters[CTR_ESC_MAX], bits);
        atomicAdd(&counters[CTR_ESC_COUNT], 1u);
    }
    return FragOut(vec4<f32>(in.value, 0.0, 0.0, 1.0e30), AUX_NONE);
}
