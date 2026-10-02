// Life tile stepper (design/automata.md §4.2-4.3): the GPU twin of `fractadyne_core::life::Universe`.
//
// The universe is a pool of 64x64-cell tiles, a byte per cell packed four to a u32 (byte 0 is the
// leftmost cell), 1024 words a tile. Each generation reads the pool `src` and writes `dst`; only the
// tiles listed in `active` are stepped, each reading its eight neighbours through `links` (a missing
// neighbour reads as the background). The rule is the core's 512-bit table: bit `idx` is whether the
// 3x3 neighbourhood `idx` (NW most significant ... SE least, state-1 cells alive) is alive next.

struct Gen {
    // The 512-bit table, word w = bits 32w..32w+31, as four vec4s.
    rule: array<vec4<u32>, 4>,
    states: u32,
    // The state of every unstored cell this generation, and next.
    background: u32,
    next_background: u32,
    active_count: u32,
};

@group(0) @binding(0) var<uniform> G: Gen;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read_write> dst: array<u32>;
// Active index -> pool slot.
@group(0) @binding(3) var<storage, read> active_slots: array<u32>;
// Per slot: its eight neighbours' slots (NW, N, NE, W, E, SW, S, SE), or -1 for "not stored".
@group(0) @binding(4) var<storage, read> links: array<i32>;
// Per slot: the live rectangle in tile-local cells (x0, y0, x1, y1), exclusive ends; cells outside
// are forced dead (a bounded plane's walls). (0, 0, 64, 64) everywhere else.
@group(0) @binding(5) var<storage, read> clip: array<vec4<i32>>;
// [0]: breach count — a cell differing from the next background on an edge (or corner) whose
// neighbour is not stored, which could seed a birth the pool cannot hold. Must stay zero.
@group(0) @binding(6) var<storage, read_write> breach: array<atomic<u32>>;

const SIDE: i32 = 64;
const WORDS: u32 = 1024u;

fn neighbour_slot(slot: u32, dx: i32, dy: i32) -> i32 {
    let n = (dy + 1) * 3 + (dx + 1); // 0..8, the centre is 4
    let k = select(n, n - 1, n > 4);
    return links[slot * 8u + u32(k)];
}

// The state of cell (x, y), x and y in -1..64, relative to tile `slot`.
fn cell(slot: u32, x: i32, y: i32) -> u32 {
    let dx = select(select(0, 1, x >= SIDE), -1, x < 0);
    let dy = select(select(0, 1, y >= SIDE), -1, y < 0);
    var s = i32(slot);
    if (dx != 0 || dy != 0) {
        s = neighbour_slot(slot, dx, dy);
        if (s < 0) {
            return G.background;
        }
    }
    let i = u32(s) * 4096u + u32(y - dy * SIDE) * 64u + u32(x - dx * SIDE);
    return (src[i >> 2u] >> ((i & 3u) * 8u)) & 0xFFu;
}

fn alive(st: u32) -> u32 {
    return select(0u, 1u, st == 1u);
}

fn rule_bit(idx: u32) -> bool {
    let w = G.rule[idx >> 7u][(idx >> 5u) & 3u];
    return ((w >> (idx & 31u)) & 1u) != 0u;
}

// `Rule::next`: binary rules take the table; Generations age their dying cells.
fn next_state(st: u32, idx: u32) -> u32 {
    let a = rule_bit(idx);
    if (G.states == 2u || st == 0u) {
        return select(0u, 1u, a);
    }
    if (st == 1u) {
        return select(2u, 1u, a);
    }
    if (st + 1u >= G.states) {
        return 0u;
    }
    return st + 1u;
}

// One thread computes one word: four cells of one row. A workgroup is 16 x 4 threads = four rows;
// dispatch (16, active_count, 1).
@compute @workgroup_size(16, 4, 1)
fn cs_step(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
    let t = wid.y;
    if (t >= G.active_count) {
        return;
    }
    let slot = active_slots[t];
    let y = i32(wid.x * 4u + lid.y);
    let x0 = i32(lid.x * 4u);
    // Rows y-1..y+1, columns x0-1..x0+4.
    var r: array<array<u32, 6>, 3>;
    for (var j = 0; j < 3; j++) {
        for (var i = 0; i < 6; i++) {
            r[j][i] = cell(slot, x0 - 1 + i, y - 1 + j);
        }
    }
    let c = clip[slot];
    var word = 0u;
    var differs = array<bool, 4>(false, false, false, false);
    for (var k = 0; k < 4; k++) {
        let idx = (alive(r[0][k]) << 8u) | (alive(r[0][k + 1]) << 7u) | (alive(r[0][k + 2]) << 6u)
            | (alive(r[1][k]) << 5u) | (alive(r[1][k + 1]) << 4u) | (alive(r[1][k + 2]) << 3u)
            | (alive(r[2][k]) << 2u) | (alive(r[2][k + 1]) << 1u) | alive(r[2][k + 2]);
        var ns = next_state(r[1][k + 1], idx);
        let x = x0 + k;
        if (x < c.x || y < c.y || x >= c.z || y >= c.w) {
            ns = 0u;
        }
        differs[k] = ns != G.next_background;
        word |= ns << (u32(k) * 8u);
    }
    dst[slot * WORDS + u32(y) * 16u + lid.x] = word;

    // The breach tripwire.
    let any_differs = differs[0] || differs[1] || differs[2] || differs[3];
    var bad = false;
    if (any_differs) {
        if (y == 0 && neighbour_slot(slot, 0, -1) < 0) { bad = true; }
        if (y == SIDE - 1 && neighbour_slot(slot, 0, 1) < 0) { bad = true; }
    }
    if (x0 == 0 && differs[0]) {
        if (neighbour_slot(slot, -1, 0) < 0) { bad = true; }
        if (y == 0 && neighbour_slot(slot, -1, -1) < 0) { bad = true; }
        if (y == SIDE - 1 && neighbour_slot(slot, -1, 1) < 0) { bad = true; }
    }
    if (x0 == SIDE - 4 && differs[3]) {
        if (neighbour_slot(slot, 1, 0) < 0) { bad = true; }
        if (y == 0 && neighbour_slot(slot, 1, -1) < 0) { bad = true; }
        if (y == SIDE - 1 && neighbour_slot(slot, 1, 1) < 0) { bad = true; }
    }
    if (bad) {
        atomicAdd(&breach[0], 1u);
    }
}

// Per-tile statistics after a batch: how many cells differ from the background, so the host can
// grow and shrink the tile set. One workgroup a tile, one thread a row; dispatch (active_count, 1, 1).
struct Stats {
    background: u32,
    active_count: u32,
    _pad0: u32,
    _pad1: u32,
};

// Bindings of their own (7-10), so no entry point sees two variables at one binding.
@group(0) @binding(7) var<uniform> S: Stats;
@group(0) @binding(8) var<storage, read> cur: array<u32>;
@group(0) @binding(9) var<storage, read> stat_active: array<u32>;
// Per active index: the number of cells differing from the background.
@group(0) @binding(10) var<storage, read_write> population: array<u32>;

var<workgroup> row_counts: array<u32, 64>;

@compute @workgroup_size(64, 1, 1)
fn cs_stats(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
    let t = wid.x;
    var n = 0u;
    if (t < S.active_count) {
        let slot = stat_active[t];
        for (var w = 0u; w < 16u; w++) {
            let word = cur[slot * WORDS + lid.x * 16u + w];
            for (var k = 0u; k < 4u; k++) {
                n += select(0u, 1u, ((word >> (k * 8u)) & 0xFFu) != S.background);
            }
        }
    }
    row_counts[lid.x] = n;
    workgroupBarrier();
    if (lid.x == 0u && t < S.active_count) {
        var total = 0u;
        for (var i = 0u; i < 64u; i++) {
            total += row_counts[i];
        }
        population[t] = total;
    }
}
