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

// ---- chunk_restart_clears_ledger: which walk restarts forget the band prices ------------------

const VIEW: ChunkSig = (0xABCD, 10_000_000, [551, 870], 1);
const OTHER_VIEW: ChunkSig = (0x1234, 10_000_000, [551, 870], 1);

fn with_jitter(view: ChunkSig, jitter_bits: u64) -> ChunkSig {
    (view.0 ^ jitter_bits, view.1, view.2, view.3)
}

#[test]
fn a_new_supersampling_sample_keeps_the_ledger() {
    // Same view, the jitter moved: the walk restarts (sig differs) but the ledger survives.
    let prev = with_jitter(VIEW, 0);
    let next = with_jitter(VIEW, 0x5555);
    assert_ne!(prev, next, "the walk itself must restart");
    assert!(!chunk_restart_clears_ledger(prev, VIEW, next, VIEW, false));
}

#[test]
fn another_view_while_settled_clears_it() {
    assert!(chunk_restart_clears_ledger(VIEW, VIEW, OTHER_VIEW, OTHER_VIEW, false));
    // ...even when the jitter moved too: the view decides, not the jitter.
    assert!(chunk_restart_clears_ledger(
        with_jitter(VIEW, 1),
        VIEW,
        with_jitter(OTHER_VIEW, 2),
        OTHER_VIEW,
        false
    ));
}

#[test]
fn interaction_never_clears_it() {
    assert!(!chunk_restart_clears_ledger(VIEW, VIEW, OTHER_VIEW, OTHER_VIEW, true));
}

#[test]
fn a_walk_that_never_ran_clears_as_before() {
    // A zeroed sig (fresh state, or a harness forcing a restart) says nothing about the view.
    let zero = (0, 0, [0, 0], 0);
    assert!(chunk_restart_clears_ledger(zero, VIEW, with_jitter(VIEW, 7), VIEW, false));
}

#[test]
fn an_unchanged_walk_clears_nothing() {
    assert!(!chunk_restart_clears_ledger(VIEW, VIEW, VIEW, VIEW, false));
}

// ---- walk_charged_px / walk_running_feed: charging a pass for the pixels still running --------

const ALL: u64 = 479_370;
const KNEE: u64 = 262_144;

#[test]
fn an_unknown_count_charges_the_whole_frame() {
    assert_eq!(walk_charged_px(ALL, KNEE, 7, 5_000, None), ALL);
}

#[test]
fn a_count_of_this_walk_charges_the_running_pixels_but_never_below_the_knee() {
    // Few running: latency-bound, charged the knee.
    assert_eq!(walk_charged_px(ALL, KNEE, 7, 5_000, Some((7, 4_000, 1_200))), KNEE);
    // Many running but fewer than the frame: charged what runs.
    let big = 4_000_000;
    assert_eq!(walk_charged_px(big, KNEE, 7, 5_000, Some((7, 4_000, 300_000))), 300_000);
    // Never more than the frame.
    assert_eq!(walk_charged_px(ALL, KNEE, 7, 5_000, Some((7, 4_000, 900_000))), ALL);
}

#[test]
fn another_walks_count_never_applies() {
    // A new sample's walk starts with every pixel running: the previous walk's tail count would
    // size its first passes for a few hundred pixels.
    assert_eq!(walk_charged_px(ALL, KNEE, 8, 5_000, Some((7, 4_000, 1_200))), ALL);
    // Walk 0 is "uncounted", never a match.
    assert_eq!(walk_charged_px(ALL, KNEE, 0, 5_000, Some((0, 4_000, 1_200))), ALL);
}

#[test]
fn a_pass_before_the_readings_cursor_is_not_bounded_by_it() {
    // Only LATER passes are bounded: at an earlier cursor more pixels may still run.
    assert_eq!(walk_charged_px(ALL, KNEE, 7, 3_999, Some((7, 4_000, 1_200))), ALL);
    assert_eq!(walk_charged_px(ALL, KNEE, 7, 4_000, Some((7, 4_000, 1_200))), KNEE);
}

#[test]
fn no_knee_means_no_charge() {
    assert_eq!(walk_charged_px(ALL, 0, 7, 5_000, Some((7, 4_000, 1_200))), ALL);
}

#[test]
fn readings_fold_into_the_current_walks_bound_only() {
    // First reading of walk 7.
    let b = walk_running_feed(None, 7, 7, 4_000, 50_000);
    assert_eq!(b, Some((7, 4_000, 50_000)));
    // A later one tightens it.
    let b = walk_running_feed(b, 7, 7, 9_000, 1_200);
    assert_eq!(b, Some((7, 9_000, 1_200)));
    // An earlier one landing late cannot loosen it.
    let b = walk_running_feed(b, 7, 7, 6_000, 20_000);
    assert_eq!(b, Some((7, 9_000, 1_200)));
    // An uncounted reading changes nothing.
    assert_eq!(walk_running_feed(b, 7, 0, 12_000, 0), b);
    // The previous walk's reading, landing after a restart: ignored, and the old bound dropped.
    assert_eq!(walk_running_feed(b, 8, 7, 12_000, 0), None);
    assert_eq!(walk_running_feed(None, 8, 7, 12_000, 0), None);
}

// ---- series_seed_bits: a refined series is another walk ---------------------------------------

#[test]
fn a_refined_series_is_another_walk() {
    let mut a = fractadyne_core::SeriesSkip::NONE;
    a.skip = 5513;
    a.a = [1.0, 0.0, 0.5, 0.0];
    let same = a;
    assert_eq!(series_seed_bits(&a), series_seed_bits(&same));
    let mut skip = a;
    skip.skip = 5535; // the 9.3e78x refinement
    assert_ne!(series_seed_bits(&a), series_seed_bits(&skip));
    let mut coef = a;
    coef.b[2] = 1.0e-7; // same skip, one coefficient moved
    assert_ne!(series_seed_bits(&a), series_seed_bits(&coef));
}
