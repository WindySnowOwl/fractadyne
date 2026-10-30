use super::*;

/// The e60205 farm reference: 1,645,846 samples, so 105 MB of room beside the build, plus the margin.
const E60205_LEN: u32 = 1_645_846;

#[test]
fn an_arrived_orbit_is_taken_once_there_is_room_for_it() {
    let need = E60205_LEN as u64 * 64 + REF_MEM_MARGIN;
    assert_eq!(pickup_look(E60205_LEN, Some(need + 1), false), PickupLook::Take);
    assert_eq!(pickup_look(E60205_LEN, None, false), PickupLook::Take, "memory unknown counts as room");
    // Short, then short again (said only the first time), then room: the third look takes it.
    let mut said = false;
    let mut looks = Vec::new();
    for available in [need - 1, 1_000_000_000, need + 1] {
        let look = pickup_look(E60205_LEN, Some(available), said);
        said |= look == PickupLook::Short { say: true };
        looks.push(look);
    }
    assert_eq!(looks, [PickupLook::Short { say: true }, PickupLook::Short { say: false }, PickupLook::Take]);
}

#[test]
fn room_is_the_orbit_and_its_tables_not_a_second_bignum_build() {
    // The e4000 gate's orbit (2,008,192 samples) on a machine with 1.9 GB free: 129 MB and the margin
    // fit. Sized as a second bignum build (`est_ref_bytes`, 13.6 GB there) it refused every pickup.
    assert_eq!(pickup_look(2_008_192, Some(1_900_000_000), false), PickupLook::Take);
    assert!(est_ref_bytes(2_008_192, 13_300) > 1_900_000_000);
}
