use super::{export_tile_cap, TileCap};

/// The RTX 3070 at the 9.3e78 minibrot (Decomposition, 200,000 iterations, 2026-11-15): a 132-px
/// tile took 599 ms, the halved 66x132 one 649 ms. The wall is one pixel's chain, not the area:
/// the cap goes back to 132 and holds there, where it had gone on halving to 16 (462 tiles, 30 s).
#[test]
fn a_halving_that_bought_nothing_is_undone_and_not_repeated() {
    let mut t = TileCap::new(2048);
    t.cap = 132;
    t.observe(132 * 132, 599.0);
    assert_eq!(t.cap, 66, "the first hot tile halves, as before");
    t.observe(66 * 132, 649.0);
    assert_eq!(t.cap, 132, "half the area, no faster: back to the larger tile");
    t.observe(132 * 132, 610.0);
    assert_eq!(t.cap, 132, "and it does not halve below it again");
    // Cheap tiles still grow it, and a hot one brings it back only to the floor.
    t.observe(132 * 132, 20.0);
    assert_eq!(t.cap, 264);
    t.observe(264 * 200, 700.0);
    assert_eq!(t.cap, 132);
}

/// Where the area IS the cost, halving goes on as before.
#[test]
fn a_halving_that_paid_keeps_halving() {
    let mut t = TileCap::new(2048);
    t.cap = 256;
    t.observe(256 * 256, 1800.0);
    assert_eq!(t.cap, 128);
    t.observe(128 * 128, 520.0); // a quarter of the area, a third of the time: area-bound
    assert_eq!(t.cap, 64);
    // A cheap tile and then a hot one is not evidence either way.
    t.observe(64 * 64, 30.0);
    assert_eq!(t.cap, 128);
    t.observe(128 * 128, 600.0);
    assert_eq!(t.cap, 64);
}

/// In the lethal band a tile halves whatever was learned.
#[test]
fn the_lethal_band_still_halves_past_the_floor() {
    let mut t = TileCap::new(2048);
    t.cap = 132;
    t.observe(132 * 132, 599.0);
    t.observe(66 * 132, 649.0);
    assert_eq!(t.cap, 132);
    t.observe(132 * 132, 1200.0);
    assert_eq!(t.cap, 66, "past SERIAL_MAX_MS the floor gives way");
}

#[test]
fn a_hot_tile_halves_the_next_and_a_cheap_one_doubles_it() {
    assert_eq!(export_tile_cap(64, 800.0, 2048), 32);
    assert_eq!(export_tile_cap(64, 50.0, 2048), 128);
    assert_eq!(export_tile_cap(64, 250.0, 2048), 64, "the band between holds");
}

#[test]
fn the_floor_and_the_static_ceiling_both_hold() {
    assert_eq!(export_tile_cap(16, 5000.0, 2048), 16, "never below 16");
    assert_eq!(export_tile_cap(70, 10.0, 70), 70, "never above the nominal bound");
    assert_eq!(export_tile_cap(2048, 10.0, 2048), 2048);
}

#[test]
fn garbage_walls_change_nothing()  {
    assert_eq!(export_tile_cap(64, f64::NAN, 2048), 64);
    assert_eq!(export_tile_cap(64, -1.0, 2048), 64);
}
