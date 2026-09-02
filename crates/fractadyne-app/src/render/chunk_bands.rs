use super::*;
const N: usize = crate::tunables::CHUNK_BANDS;

#[test]
fn bands_are_octaves_from_a_fixed_base_not_a_fraction_of_the_ask() {
    // Band 0 is [0,256); band k is [256*2^(k-1), 256*2^k).
    assert_eq!(chunk_band_of(0), 0);
    assert_eq!(chunk_band_of(255), 0);
    assert_eq!(chunk_band_of(256), 1);
    assert_eq!(chunk_band_of(511), 1);
    assert_eq!(chunk_band_of(512), 2);
    assert_eq!(chunk_band_of(1023), 2);
    assert_eq!(chunk_band_of(1024), 3);
    // Monotonic, and saturating at the last band rather than panicking or wrapping.
    assert_eq!(chunk_band_of(u32::MAX), N - 1);
    let mut prev = 0;
    for cur in [0u32, 1, 255, 256, 700, 4096, 100_000, 5_000_000, u32::MAX] {
        let b = chunk_band_of(cur);
        assert!(b >= prev, "band must not go backwards at cur={cur}");
        assert!(b < N, "band {b} out of range at cur={cur}");
        prev = b;
    }
}

#[test]
fn the_2026_08_22_fatal_window_no_longer_shares_a_band_with_the_cheap_orbit_start() {
    // The field loss ran chunk=[768,1792) against an ask of 231,676. Under the old
    // ask-proportional rule every cursor below 14,480 was band 0, so that window's licence had
    // been earned by the passes at iterations 0..768. Octaves separate them.
    let old_band = |cur: u32, ask: u32| -> usize {
        ((cur as u64 * N as u64) / (ask.max(1) as u64)).min(N as u64 - 1) as usize
    };
    assert_eq!(old_band(0, 231_676), old_band(768, 231_676), "old rule: same band");
    assert_ne!(
        chunk_band_of(0),
        chunk_band_of(768),
        "the cheap orbit start must no longer license the fatal window"
    );
    assert_eq!(chunk_band_of(768), 2, "[512,1024)");
}

#[test]
fn a_lethal_retreat_sheds_every_earned_license_back_to_the_floor() {
    // The 2026-08-22 field shape: a band has EARNED a large license from this region's cheap
    // frames, and the pass that would shed it never gets priced because the saturated queue
    // stops releasing quick presents. The retreat has to shed it directly.
    let mut b = [0u32; N];
    b[2] = 20_000;
    b[3] = 1_024;
    assert_eq!(chunk_band_license(&b, 3, 256), 1_024, "earned before the retreat");
    chunk_band_retreat(&mut b);
    for band in 0..N {
        assert_eq!(
            chunk_band_license(&b, band, 256),
            256,
            "band {band} must reopen at the floor after a lethal retreat"
        );
    }
}

#[test]
fn every_unvisited_band_opens_at_the_floor_never_a_neighbours_license() {
    // The 10-second-single lesson: a 36k license earned on a band's cold beginning met a
    // mid-band storm. First contact is floor-sized everywhere, hostile or not.
    let mut b = [0u32; N];
    b[3] = 20_000;
    assert_eq!(chunk_band_license(&b, 4, 256), 256);
    assert_eq!(chunk_band_license(&b, 0, 256), 256);
    chunk_band_update(&mut b, 4, 256, 30.0, 400.0);
    assert_eq!(chunk_band_license(&b, 4, 256), 512); // earned, not inherited
}

#[test]
fn clearly_cheap_prices_take_the_fast_lane() {
    let mut b = [0u32; N];
    chunk_band_update(&mut b, 0, 256, 100.0, 400.0); // ≤ half target → ×2
    assert_eq!(b[0], 512);
    chunk_band_update(&mut b, 0, 512, 300.0, 400.0); // ≤ target → ×1.25
    assert_eq!(b[0], 640);
}

#[test]
fn every_price_above_target_moves_the_size_down() {
    // No hold gap: (1x, 2x] halves, past 2x quarters — an over-target size must never
    // re-dispatch itself unchanged.
    let mut b = [0u32; N];
    b[5] = 20_000;
    chunk_band_update(&mut b, 5, 20_000, 600.0, 400.0);
    assert_eq!(b[5], 10_000);
    chunk_band_update(&mut b, 5, 10_000, 1200.0, 400.0);
    assert_eq!(b[5], 2_500);
    chunk_band_update(&mut b, 5, 2, 1200.0, 400.0);
    assert_eq!(b[5], 1); // never zero: a shed band is still priced knowledge
}

#[test]
fn a_hot_band_does_not_shrink_its_cold_neighbours() {
    let mut b = [0u32; N];
    b[4] = 30_000;
    b[5] = 30_000;
    chunk_band_update(&mut b, 5, 30_000, 2000.0, 400.0);
    assert_eq!(b[4], 30_000, "the cold side keeps its own prices");
    assert_eq!(b[5], 7_500);
}

#[test]
fn garbage_prices_are_ignored_but_zero_is_clearly_cheap() {
    let mut b = [0u32; N];
    b[1] = 5_000;
    chunk_band_update(&mut b, 1, 5_000, -3.0, 400.0);
    chunk_band_update(&mut b, 1, 5_000, f64::NAN, 400.0);
    assert_eq!(b[1], 5_000, "NaN/negative license nothing");
    // Zero = a pass cheaper than the clock (and every headless-harness pass): fast lane.
    chunk_band_update(&mut b, 1, 5_000, 0.0, 400.0);
    assert_eq!(b[1], 10_000);
}
