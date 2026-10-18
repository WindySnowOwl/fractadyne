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

/// The 2^260 trace's shape (cont-4, 2026-10-18): every band's last pass cost 1.5 ms over its whole
/// width; the walk ends 1,550 iterations into band 9, which cost 1.5 ms over those.
fn costed() -> ([f32; crate::tunables::CHUNK_BANDS], [u32; crate::tunables::CHUNK_BANDS], [u32; crate::tunables::CHUNK_BANDS]) {
    let (mut ms, mut iters, mut len) = ([0.0; crate::tunables::CHUNK_BANDS], [0; crate::tunables::CHUNK_BANDS], [0; crate::tunables::CHUNK_BANDS]);
    for b in 1..=9 {
        ms[b] = 1.5;
        iters[b] = if b == 9 { 1_550 } else { chunk_band_end(chunk_band_start(b)) - chunk_band_start(b) };
        len[b] = 35_138;
    }
    (ms, iters, len)
}

fn licensed() -> [u32; crate::tunables::CHUNK_BANDS] {
    let mut bands = [0; crate::tunables::CHUNK_BANDS];
    for b in 1..=9 {
        bands[b] = 4_096;
    }
    bands
}

#[test]
fn a_priced_span_runs_on_while_the_bands_measured_costs_fit() {
    let (ms, iters, len) = costed();
    let costs = BandCosts { ms: &ms, iters: &iters, len: &len };
    let bands = licensed();
    // 10 ms (half the pin target): bands 1–6 at 1.5 ms each is 9.0, band 7 would make 10.5.
    let ((end, band), cost) = pin_span_end_priced(&costs, &bands, 256, 512, 4_096, 67_086, 100_000, 35_138, 10.0);
    assert_eq!((end, band), (16_384, span_band_code(1, 6)));
    assert!((cost - 9.0).abs() < 1e-9, "{cost}");
    // The whole pin target: on to the walk's end, inside band 9 (13.5 ms), in one pass.
    let ((end, band), _) = pin_span_end_priced(&costs, &bands, 256, 512, 4_096, 67_086, 100_000, 35_138, 20.0);
    assert_eq!((end, band), (67_086, span_band_code(1, 9)));
    // ⛔The licences never enter the sum: one priced 4,096 for a band 32,768 wide spans nothing less.
    // The rate model's pass size still caps the span.
    let ((end, _), _) = pin_span_end_priced(&costs, &bands, 256, 512, 4_096, 67_086, 4_000, 35_138, 20.0);
    assert_eq!(end, 4_096);
}

#[test]
fn a_priced_span_stops_at_a_band_with_no_cost_or_no_licence() {
    let (mut ms, iters, len) = costed();
    ms[4] = 0.0;
    let costs = BandCosts { ms: &ms, iters: &iters, len: &len };
    let ((end, _), _) = pin_span_end_priced(&costs, &licensed(), 256, 512, 4_096, 67_086, 100_000, 35_138, 20.0);
    assert_eq!(end, 2_048, "band 4 was never timed");
    let (ms, iters, len) = costed();
    let costs = BandCosts { ms: &ms, iters: &iters, len: &len };
    let mut bands = licensed();
    bands[5] = 0;
    let ((end, _), _) = pin_span_end_priced(&costs, &bands, 256, 512, 4_096, 67_086, 100_000, 35_138, 20.0);
    assert_eq!(end, 4_096, "band 5 has no licence (a retreat shed it)");
}

#[test]
fn a_priced_span_enters_the_storm_band_only_on_this_references_cost() {
    // The reference escapes at 35,138: its first wrap's rebase storm sits in band 8 [32,768, 65,536).
    let (ms, iters, mut len) = costed();
    len[8] = 30_000;
    let costs = BandCosts { ms: &ms, iters: &iters, len: &len };
    let ((end, _), _) = pin_span_end_priced(&costs, &licensed(), 256, 512, 4_096, 67_086, 100_000, 35_138, 20.0);
    assert_eq!(end, 32_768, "band 8's cost was measured against another length");
    // With the wrap past the walk (200,000 is in band 10), bands measured against another length are
    // entered as any other.
    let ((end, _), _) = pin_span_end_priced(&costs, &licensed(), 256, 512, 4_096, 67_086, 100_000, 200_000, 20.0);
    assert_eq!(end, 67_086);
    // …and with it early (3,000, band 4), the span stops at band 4's edge.
    let ((end, _), _) = pin_span_end_priced(&costs, &licensed(), 256, 512, 4_096, 67_086, 100_000, 3_000, 20.0);
    assert_eq!(end, 2_048);
}

#[test]
fn a_band_is_predicted_at_its_cost_scaled_up_past_the_iterations_it_covered() {
    let (ms, iters, len) = costed();
    let costs = BandCosts { ms: &ms, iters: &iters, len: &len };
    // Band 9 cost 1.5 ms over 1,550 iterations: 3,100 are predicted at 3.0, 1,000 at 1.5 (never less).
    assert_eq!(costs.predict(9, 3_100), Some(3.0));
    assert_eq!(costs.predict(9, 1_000), Some(1.5));
    assert_eq!(costs.predict(10, 1_000), None);
    // A start band with no cost of its own counts its licence's share of the 20 ms target (256 of
    // 4,096 = 1.25 ms), as `pin_span_end` would.
    let (mut ms, iters, len) = costed();
    ms[1] = 0.0;
    let costs = BandCosts { ms: &ms, iters: &iters, len: &len };
    let ((end, _), cost) = pin_span_end_priced(&costs, &licensed(), 256, 512, 4_096, 67_086, 100_000, 35_138, 3.0);
    assert_eq!(end, 1_024);
    assert!((cost - 2.75).abs() < 1e-9, "{cost}");
}
