//! The apron a lit export's tiles iterate (`apron_plan`, `tile_apron_sides`, `pack_tile_apron`):
//! the relief's 3x3 neighborhood is the frame's at a tile's edge (2026-11-26, the seam).

use super::*;

/// Without an apron the plan is the one it always was.
#[test]
fn no_apron_is_the_plan_as_before() {
    for &(tile, occ) in &[(256u32, false), (64, true), (16, false), (1024, true)] {
        let (t, rh, cw, st) = apron_plan(tile, 3840, 2160, 2, occ, 0);
        assert_eq!(t, tile);
        assert_eq!((rh, cw), (balanced_extent(2160, tile), balanced_extent(3840, tile)));
        let want = if occ { [cw * 2, rh * 2] } else { [tile * 2, tile * 2] };
        assert_eq!(st, want, "tile {tile} occupancy {occ}");
    }
}

/// With one, the tile gives up two pixels, so what it iterates (the tile and its apron) is no more
/// than planned, and the chunk state holds every tile the wall-adaptive cap can cut (16 px up to the
/// ceiling, `export_tile_cap`), with its apron.
#[test]
fn an_apron_keeps_the_planned_area_and_fits_the_state() {
    for ss in [1u32, 2, 4] {
        for tile in [16u32, 17, 18, 64, 255, 2048] {
            let (t, _, _, st) = apron_plan(tile, 3840, 2160, ss, false, 1);
            assert_eq!(t + 2, tile, "tile {tile}");
            for cap in 16..=t.max(16) {
                assert!((cap + 2) * ss <= st[0] && (cap + 2) * ss <= st[1], "ss {ss} tile {tile} cap {cap}: state {st:?}");
            }
            if tile >= 18 {
                assert_eq!(st, [tile * ss, tile * ss], "the state as planned: tile {tile}");
            }
        }
        for tile in [64u32, 1024] {
            let (t, rh, cw, st) = apron_plan(tile, 3840, 2160, ss, true, 1);
            assert!(rh <= t && cw <= t, "the occupancy grid within the tile");
            assert_eq!(st, [(cw + 2) * ss, (rh + 2) * ss]);
        }
    }
}

/// The apron is on the sides that have a neighboring tile, never past the frame's edges.
#[test]
fn the_apron_is_on_the_sides_with_a_neighbor() {
    let (w, h) = (100, 60);
    assert_eq!(tile_apron_sides(0, 0, 100, 60, w, h, 1), [0, 0, 0, 0], "one tile, the whole frame");
    assert_eq!(tile_apron_sides(0, 0, 50, 30, w, h, 1), [0, 0, 1, 1], "the top-left tile");
    assert_eq!(tile_apron_sides(50, 30, 50, 30, w, h, 1), [1, 1, 0, 0], "the bottom-right tile");
    assert_eq!(tile_apron_sides(30, 20, 40, 20, w, h, 1), [1, 1, 1, 1], "an inner tile");
    assert_eq!(tile_apron_sides(30, 20, 40, 20, w, h, 0), [0, 0, 0, 0], "no apron, none");
    assert_eq!(pack_tile_apron([1, 0, 1, 1]), 0x0101_0001);
}

/// A spotlight placed for a tile measures every pixel as the frame does: the same distance from
/// its center in units of the radius, whichever tile the pixel falls in (`tile_vignette`).
#[test]
fn a_spotlight_is_the_frames_in_every_tile() {
    let (w, h) = (640u32, 360u32);
    let (center, radius, soft) = ([0.3f32, 0.55], 0.28f32, 0.08f32);
    // The frame's measure of a pixel center: distance in frame heights, over the radius.
    let frame = |x: f32, y: f32| ((((x / w as f32 - center[0]) * w as f32 / h as f32).powi(2) + (y / h as f32 - center[1]).powi(2)).sqrt()) / radius;
    for &(tx0, ty0, tw, th) in &[(0u32, 0u32, 200u32, 200u32), (200, 0, 200, 200), (400, 200, 200, 160), (600, 200, 40, 160)] {
        let (c, r, s) = tile_vignette(center, radius, soft, tx0, ty0, tw, th, w, h);
        assert!((s / r - soft / radius).abs() < 1e-6, "the softness keeps its share of the radius");
        for &(i, j) in &[(0u32, 0u32), (tw / 2, th / 3), (tw - 1, th - 1)] {
            let (x, y) = ((tx0 + i) as f32 + 0.5, (ty0 + j) as f32 + 0.5);
            // The shader's measure in the tile: its uv, its aspect, its height.
            let (u, v) = ((i as f32 + 0.5) / tw as f32, (j as f32 + 0.5) / th as f32);
            let tile = (((u - c[0]) * tw as f32 / th as f32).powi(2) + (v - c[1]).powi(2)).sqrt() / r;
            assert!((tile - frame(x, y)).abs() < 1e-4, "tile {tx0},{ty0} pixel {i},{j}: {tile} against the frame's {}", frame(x, y));
        }
    }
}
