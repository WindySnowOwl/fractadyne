// Assembling a zoom-video frame from exponential-map strips (`design/zoom-video.md` §6.2;
// `assemble.rs`). A module of its own: nothing here is compiled into the iterate or colour modules.
//
// The pass fills a FRAME-SHAPED main target — the frame's (w·ss)×(h·ss) sample grid, each sample
// `(smooth count, slope normal x, y, log2 distance estimate)` exactly as an iterate pass writes it —
// from the strips, and the app's own colour pass (`fs_color`) colours it as it colours any frame.
// Each sample takes the NEAREST texel: an iteration count is never interpolated (§6.2); the
// colour pass shades every sample and averages the colours, as it does for supersampling.
//
// Geometry (`fractadyne_core::ExpMap::locate` is the CPU definition this mirrors): a sample at
// `d` samples from the frame's centre lies `A − log2 d` octaves inside strip 0's outer radius, `A`
// = r0_log2 − log2(sample spacing). The CPU splits `A` into whole octaves and a fraction, so the
// sample's own part, `A_f − log2 d`, stays a small number an f32 resolves to ~1e-3 of a row at any
// depth. Its whole part picks the strip; the fraction × rows-an-octave the row; the angle the column.

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

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

// Keep in step with `AssembleUniforms` in assemble.rs.
struct AssembleU {
    res: vec2<f32>,          // the frame's sample grid, (w·ss, h·ss)
    width: f32,              // texels around a strip (W)
    rows: f32,               // rows in a strip
    rows_per_octave: f32,    // W·ln2/2π for a conformal strip
    sample_offset: f32,      // where in its texel a strip samples (0.5: the centre)
    depth_frac: f32,         // A_f: the fractional octaves of A
    rel_base: i32,           // A_i − k_first: strip k = k_first + rel_base + floor(A_f − log2 d)
    first_layer: i32,        // the array layer holding strip k_first (strip k is in layer k mod layers)
    layers: i32,             // the array's layers
    rel_last: i32,           // the deepest strip there is, relative to k_first
    turns: f32,              // the frame's rotation φ, in turns, added to every sample's angle
    rot_cos: f32,            // cos φ, sin φ: a plane vector seen on the screen is R(−φ) of it
    rot_sin: f32,
    log2_row_step: f32,      // log2 of a row's step in ln r (2π/W for a conformal strip)
    piece_width: i32,        // a strip is held in pieces this wide around its circle (W: one piece)
    pieces: i32,             // pieces a strip: strip slot s, piece p is the array's layer s·pieces + p
    texels: u32,             // floats a texel holds: 1 the count, 2 the count and log2 DE, 4 all
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> au: AssembleU;
@group(0) @binding(1) var strips: texture_2d_array<f32>;

// atan(x) for 0 ≤ x ≤ 1, |error| ≤ 2e-8 (Abramowitz & Stegun 4.4.49). ⚠Not the built-in: WGSL
// promises atan2 only to 4096 ULP, which on a 6,921-texel strip could move a sample by a texel.
fn atan_unit(x: f32) -> f32 {
    let x2 = x * x;
    return x * (1.0 + x2 * (-0.3333314528 + x2 * (0.1999355085 + x2 * (-0.1420889944
        + x2 * (0.1065626393 + x2 * (-0.0752896400 + x2 * (0.0429096138
        + x2 * (-0.0161657367 + x2 * 0.0028662257))))))));
}

// The angle of (x, y) in turns, [0, 1], anticlockwise from +x: reduced to the first octant exactly
// (by swaps and reflections), where `atan_unit` is good to 2e-8 radians.
fn atan2_turns(y: f32, x: f32) -> f32 {
    let ax = abs(x);
    let ay = abs(y);
    let hi = max(ax, ay);
    var t = 0.0;
    if (hi > 0.0) {
        t = atan_unit(min(ax, ay) / hi) * 0.15915494309;
    }
    if (ay > ax) { t = 0.25 - t; }
    if (x < 0.0) { t = 0.5 - t; }
    if (y < 0.0) { t = 1.0 - t; }
    return t;
}

@fragment
fn fs_assemble(in: VsOut) -> @location(0) vec4<f32> {
    // The sample's offset from the frame's centre in samples, imaginary up (as `iterate_at` lays a
    // frame out). A sample nearer the centre than half a sample — the centre sample of an odd grid
    // — is read at half a sample: the texel there is as near as any.
    let dx = in.pos.x - 0.5 * au.res.x;
    let dy = 0.5 * au.res.y - in.pos.y;
    let d2 = max(dx * dx + dy * dy, 0.25);
    let s = au.depth_frac - 0.5 * log2(d2);
    let ks = floor(s);
    var rel = au.rel_base + i32(ks);
    if (rel < 0) {
        return vec4<f32>(-2.0, 0.0, 0.0, 1.0e30); // outside the dive's first strip: never computed
    }
    let rows = i32(au.rows);
    var j = i32(floor((s - ks) * au.rows_per_octave + 0.5 - au.sample_offset));
    if (rel > au.rel_last) {
        // Deeper than the dive's last strip (within half a pixel of the last frame's centre):
        // its innermost row is the nearest there is.
        rel = au.rel_last;
        j = rows - 1;
    }
    j = clamp(j, 0, rows - 1);
    let w = i32(au.width);
    let t = atan2_turns(dy, dx) + au.turns;
    var i = i32(floor(fract(t) * au.width + 0.5 - au.sample_offset));
    if (i >= w) { i = i - w; }
    if (i < 0) { i = i + w; }
    let slot = (au.first_layer + rel) % au.layers;
    // A strip wider than the card's textures is held in pieces around its circle (`strip_pieces`).
    let piece = i / au.piece_width;
    let held = textureLoad(strips, vec2<i32>(i - piece * au.piece_width, j), slot * au.pieces + piece, 0);
    // The window holds only what the colouring reads (`WindowTexels`): what it leaves out is zero
    // here and never read by the colour pass.
    var q = held;
    if (au.texels == 1u) {
        q = vec4<f32>(held.x, 0.0, 0.0, 0.0);
    } else if (au.texels == 2u) {
        q = vec4<f32>(held.x, 0.0, 0.0, held.y);
    }
    if (q.x < 0.0) {
        return q; // interior (-1) or glitched (-2): as rendered
    }
    // The distance estimate is in this texel's own (radial) size; the colour pass reads it in
    // samples. log2(texel / sample) = A − (the texel's depth) + log2(row step), with A − k =
    // A_f − (k − A_i) for the strip actually read (the deepest one, when clamped above).
    let kk = f32(rel - au.rel_base);
    let texel_over_sample = (au.depth_frac - kk) - (f32(j) + au.sample_offset) / au.rows_per_octave + au.log2_row_step;
    // The normal is in the plane's frame; on a rotated frame the plane turns by −φ.
    let n = vec2<f32>(au.rot_cos * q.y + au.rot_sin * q.z, -au.rot_sin * q.y + au.rot_cos * q.z);
    return vec4<f32>(q.x, n.x, n.y, q.w + texel_over_sample);
}
