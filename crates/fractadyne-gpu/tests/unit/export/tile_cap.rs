use super::export_tile_cap;

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
