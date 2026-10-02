// Life display pass (design/automata.md §4.3): cells -> the iteration texture, so the ordinary colour
// pass (`fs_color`) colours them. Writes what an escape-time pass writes: main = (value, 0, 0, 1e30)
// with value < 0 for "interior" (a dead cell), aux = AUX_NONE; and commits the escape-range counters
// the live normalization reads.
//
// value: 1.0 for a live cell; a Generations dying state fades towards 0.1; zoomed out, the fraction
// of live cells under the texel. One palette position therefore means the same intensity at every
// zoom.

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var clip = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.0, 1.0), vec2<f32>(2.0, 1.0), vec2<f32>(0.0, -1.0));
    var out: VsOut;
    out.pos = vec4<f32>(clip[vi], 0.0, 1.0);
    out.uv = uv[vi];
    return out;
}

struct FragOut {
    @location(0) main: vec4<f32>,
    @location(1) aux: vec4<f32>,
};

const AUX_NONE: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0e30, 0.0);
const INTERIOR: vec4<f32> = vec4<f32>(-1.0, 0.0, 0.0, 1.0e30);

// The iterate passes' event counters (group 0 = the view's iterate bind group; binding 2). Slot
// numbers and the 4x4 subsampling grid are `mandelbrot.wgsl`'s (CTR_ESC_MIN/MAX/COUNT), checked
// against the Rust constants by `life/tests.rs`.
@group(0) @binding(2) var<storage, read_write> counters: array<atomic<u32>>;
const CTR_ESC_MIN: u32 = 5u;
const CTR_ESC_MAX: u32 = 6u;
const CTR_ESC_COUNT: u32 = 7u;

struct LifeView {
    // Cell coordinates of texel (0, 0)'s top-left corner, relative to the top-left cell of grid
    // entry (0, 0); cells per texel.
    origin: vec2<f32>,
    step: f32,
    // 0: `grid` holds a pool slot per 64x64 tile (-1 = not stored, i.e. background);
    // 1: `grid` holds a density per bin of `bin` x `bin` cells (f32 bits).
    coarse: u32,
    grid: vec2<u32>,
    bin: f32,
    background: u32,
    states: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(1) @binding(0) var<uniform> L: LifeView;
@group(1) @binding(1) var<storage, read> pool: array<u32>;
@group(1) @binding(2) var<storage, read> grid: array<i32>;

fn state_at(cx: i32, cy: i32) -> u32 {
    let tx = cx >> 6u;
    let ty = cy >> 6u;
    if (cx < 0 || cy < 0 || u32(tx) >= L.grid.x || u32(ty) >= L.grid.y) {
        return L.background;
    }
    let slot = grid[u32(ty) * L.grid.x + u32(tx)];
    if (slot < 0) {
        return L.background;
    }
    let i = u32(slot) * 4096u + u32(cy & 63) * 64u + u32(cx & 63);
    return (pool[i >> 2u] >> ((i & 3u) * 8u)) & 0xFFu;
}

// A state's intensity: alive 1, dying states fading, dead 0.
fn intensity(st: u32) -> f32 {
    if (st == 0u) {
        return 0.0;
    }
    if (st == 1u || L.states <= 2u) {
        return 1.0;
    }
    return 1.0 - 0.9 * f32(st - 1u) / f32(L.states - 1u);
}

@fragment
fn fs_life(in: VsOut) -> FragOut {
    let p = in.pos.xy; // the texel centre, in texels
    let c = L.origin + p * L.step;
    var v = 0.0;
    if (L.coarse == 1u) {
        let b = floor(c / L.bin);
        if (b.x >= 0.0 && b.y >= 0.0 && u32(b.x) < L.grid.x && u32(b.y) < L.grid.y) {
            v = bitcast<f32>(grid[u32(b.y) * L.grid.x + u32(b.x)]);
        }
    } else if (L.step <= 1.0) {
        v = intensity(state_at(i32(floor(c.x)), i32(floor(c.y))));
    } else {
        // Several cells under the texel: the mean intensity of up to 8 x 8 cells spread evenly over
        // its footprint (the same cells every frame, so a still pattern stays still).
        let n = min(u32(ceil(L.step)), 8u);
        let lo = c - vec2<f32>(0.5 * L.step);
        let d = L.step / f32(n);
        var sum = 0.0;
        for (var j = 0u; j < n; j++) {
            for (var i = 0u; i < n; i++) {
                let q = lo + (vec2<f32>(f32(i), f32(j)) + 0.5) * d;
                sum += intensity(state_at(i32(floor(q.x)), i32(floor(q.y))));
            }
        }
        v = sum / f32(n * n);
    }
    if (v <= 0.0) {
        return FragOut(INTERIOR, AUX_NONE);
    }
    let t = vec2<i32>(p);
    if ((t.x & 3) == 0 && (t.y & 3) == 0) {
        let b = bitcast<u32>(v);
        atomicMin(&counters[CTR_ESC_MIN], b);
        atomicMax(&counters[CTR_ESC_MAX], b);
        atomicAdd(&counters[CTR_ESC_COUNT], 1u);
    }
    return FragOut(vec4<f32>(v, 0.0, 0.0, 1.0e30), AUX_NONE);
}
