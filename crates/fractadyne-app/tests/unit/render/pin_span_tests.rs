use super::*;

/// Every band of the (49,3) trace's shape priced: the cheap bands 1–5 at sixteen times their width
/// (0.2 ms passes on the ×16 lane), band 6 at 10,240 (a 16.5 ms pass of 8,192 grown ×1.25).
fn priced() -> [u32; crate::tunables::CHUNK_BANDS] {
    let mut bands = [0; crate::tunables::CHUNK_BANDS];
    for b in 1..=5 {
        bands[b] = 16 * (chunk_band_end(chunk_band_start(b)) - chunk_band_start(b));
    }
    bands[6] = 10_240;
    bands
}

#[test]
fn a_one_band_byte_is_the_band_index() {
    for b in 0..crate::tunables::CHUNK_BANDS {
        assert_eq!(span_band_code(b, b), b as u8, "a ledger recorded before spans reads the same");
        assert_eq!(span_band_decode(b as u8), (b, b));
    }
    assert_eq!(span_band_decode(span_band_code(1, 5)), (1, 5));
    assert_eq!(span_band_decode(span_band_code(0, 15)), (0, 15));
    assert_eq!(span_band_label(span_band_code(1, 5)), "1..5");
    assert_eq!(span_band_label(6), "6");
}

#[test]
fn band_starts_invert_band_of() {
    for b in 0..crate::tunables::CHUNK_BANDS {
        assert_eq!(chunk_band_of(chunk_band_start(b)), b);
    }
    assert_eq!((chunk_band_start(0), chunk_band_start(1), chunk_band_start(6)), (0, 256, 8_192));
}

#[test]
fn a_span_splits_into_whole_bands_and_the_rest() {
    let one: Vec<_> = span_band_lengths(6, 1_000).collect();
    assert_eq!(one, [(6, 1_000)]);
    let whole: Vec<_> = span_band_lengths(span_band_code(1, 5), 8_192 - 256).collect();
    assert_eq!(whole, [(1, 256), (2, 512), (3, 1_024), (4, 2_048), (5, 4_096)]);
    // Ended by the walk inside its last band.
    let cut: Vec<_> = span_band_lengths(span_band_code(1, 6), 8_192 - 256 + 1_000).collect();
    assert_eq!(cut, [(1, 256), (2, 512), (3, 1_024), (4, 2_048), (5, 4_096), (6, 1_000)]);
}

#[test]
fn a_pin_pass_runs_on_through_priced_bands_while_their_shares_fit() {
    // Bands 1–5 cost 1/16 of a pass each; band 6 would add 0.8 and the sum pass 1.
    let bands = priced();
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 256, 42_237, 100_000, 50_000, 1.0), (8_192, span_band_code(1, 5)));
    // From band 6's edge: band 6 alone (0.8), band 7 unpriced.
    assert_eq!(pin_span_end(&bands, 8_192, 16_384, bands[6], 256, 42_237, 100_000, 50_000, 1.0), (16_384, 6));
}

#[test]
fn a_band_without_prices_of_its_own_still_stops_the_pass() {
    let mut bands = priced();
    bands[3] = 0;
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 256, 42_237, 100_000, 50_000, 1.0), (1_024, span_band_code(1, 2)));
    // Even past the storm, where a cold band's licence is inherited: an inherited licence is not a price.
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 256, 42_237, 100_000, 100, 1.0), (1_024, span_band_code(1, 2)));
}

#[test]
fn a_pass_that_does_not_cover_its_start_band_does_not_span() {
    let bands = priced();
    // Starts inside the band (past the series skip).
    assert_eq!(pin_span_end(&bands, 300, 512, bands[1], 256, 42_237, 100_000, 50_000, 1.0), (512, 1));
    // Its licence (or the step) stopped it short of the band's end.
    assert_eq!(pin_span_end(&bands, 512, 900, bands[2], 256, 42_237, 100_000, 50_000, 1.0), (900, 2));
    // The walk ends in this band.
    assert_eq!(pin_span_end(&bands, 256, 400, bands[1], 256, 400, 100_000, 50_000, 1.0), (400, 1));
}

#[test]
fn a_span_stays_inside_the_step_cap_and_ends_at_the_walk() {
    let bands = priced();
    // 2,000 iterations: bands 1–3 (1,792) fit, band 4 (3,840) does not.
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 256, 42_237, 2_000, 50_000, 1.0), (2_048, span_band_code(1, 3)));
    // The walk ends inside band 4: the span takes the rest of the walk.
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 256, 3_000, 100_000, 50_000, 1.0), (3_000, span_band_code(1, 4)));
}

#[test]
fn a_span_takes_at_most_its_share_of_a_pass() {
    // Each band a fifth of a pass: at half a pass bands 1 and 2 (0.4) span, band 3 would take it to
    // 0.6; at a whole pass bands 1 to 5.
    let mut bands = [0; crate::tunables::CHUNK_BANDS];
    for b in 1..=8 {
        bands[b] = 5 * (chunk_band_end(chunk_band_start(b)) - chunk_band_start(b));
    }
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 1, 42_237, 100_000, 50_000, 0.5), (1_024, span_band_code(1, 2)));
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 1, 42_237, 100_000, 50_000, 1.0), (8_192, span_band_code(1, 5)));
}

#[test]
fn bands_licensed_at_their_width_admit_no_span() {
    // Each band's licence is its own width: one band is a whole pass.
    let mut bands = [0; crate::tunables::CHUNK_BANDS];
    for b in 1..=8 {
        bands[b] = chunk_band_end(chunk_band_start(b)) - chunk_band_start(b);
    }
    assert_eq!(pin_span_end(&bands, 256, 512, bands[1], 1, 42_237, 100_000, 50_000, 1.0), (512, 1));
}
