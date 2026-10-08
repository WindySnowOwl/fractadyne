use super::*;

fn plan(w: u32, h: u32, tile: u32, occupancy: bool) -> ExportPlan {
    let (row_h, col_w) = (balanced_extent(h, tile), balanced_extent(w, tile));
    ExportPlan { w, h, ss: 2, tile, row_h, col_w, fe: occupancy, chunk_scope: true, occupancy, rn_bla: false, state_size: [0, 0] }
}

/// The split's tile grid covers every output pixel exactly once, for both kinds of tile.
#[test]
fn the_split_grid_covers_the_frame_once() {
    for &(w, h, tile, occ) in &[(1000, 700, 256, false), (1000, 700, 256, true), (3840, 2160, 1024, true), (17, 9, 64, false), (192, 120, 48, true)] {
        let p = plan(w, h, tile, occ);
        let mut seen = vec![0u8; (w * h) as usize];
        for [x0, y0, tw, th] in multi_tile_rects(&p) {
            assert!(tw > 0 && th > 0 && x0 + tw <= w && y0 + th <= h, "{w}x{h} tile {tile}: rect {x0},{y0} {tw}x{th}");
            for y in y0..y0 + th {
                for x in x0..x0 + tw {
                    seen[(y * w + x) as usize] += 1;
                }
            }
        }
        assert!(seen.iter().all(|&c| c == 1), "{w}x{h} tile {tile} occupancy {occ}: a pixel covered {} times", seen.iter().max().unwrap());
    }
}

/// Step-bounded tiles are the single device's grid exactly: equal rows and columns, which is what
/// makes a split of that render its tiles bit for bit.
#[test]
fn step_bounded_tiles_are_the_single_device_grid() {
    let p = plan(3840, 2160, 1024, true);
    let rects = multi_tile_rects(&p);
    assert!(rects.iter().all(|r| r[2] <= p.col_w && r[3] <= p.row_h));
    let cols = rects.iter().filter(|r| r[1] == 0).count() as u32;
    assert_eq!(cols, 3840u32.div_ceil(p.col_w));
}
