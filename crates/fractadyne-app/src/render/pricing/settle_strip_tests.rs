//! `settle_strip`: the piece of a settle tile that fits the admission.

use super::*;

const TILE: [u32; 4] = [884, 0, 884, 884];
/// Steps per base-pixel row of `TILE` at ss 2 and 4,627 iterations.
const PER_ROW: u64 = 884 * 4 * 4627;

#[test]
fn a_tile_the_admission_covers_comes_back_whole() {
    assert_eq!(settle_strip(TILE, 0, PER_ROW, PER_ROW * 884), TILE);
    assert_eq!(settle_strip(TILE, 0, PER_ROW, u64::MAX), TILE);
}

#[test]
fn a_tile_over_the_admission_goes_out_as_whole_rows_that_fit() {
    // The RTX 3080 case: a 1.45e10-step tile after a wall cap of 8.9e9.
    let cap = 8_900_000_000;
    let first = settle_strip(TILE, 0, PER_ROW, cap);
    assert_eq!(first, [884, 0, 884, (cap / PER_ROW) as u32]);
    assert!(tile_steps(first, 2, 4627) <= cap);
    let second = settle_strip(TILE, first[3], PER_ROW, cap);
    assert_eq!(second[1], first[3]);
    assert_eq!(second[1] + second[3], 884, "the remainder closes the tile");
}

#[test]
fn the_strips_cover_the_tile_exactly_whatever_the_admission_does() {
    // The admission may move between strips; rows must neither repeat nor skip.
    let mut row = 0;
    let mut covered = 0;
    for adm in [PER_ROW * 100, PER_ROW * 7, PER_ROW * 300, 1, PER_ROW * 2_000].iter().cycle() {
        let s = settle_strip(TILE, row, PER_ROW, *adm);
        assert_eq!(s[1], TILE[1] + row);
        covered += s[3];
        row += s[3];
        if row >= TILE[3] {
            break;
        }
    }
    assert_eq!(covered, TILE[3]);
}

#[test]
fn an_admission_under_one_row_still_sends_one_row() {
    assert_eq!(settle_strip(TILE, 10, PER_ROW, 5)[3], 1);
}
