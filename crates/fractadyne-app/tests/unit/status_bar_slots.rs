use super::{
    center_readout, commas, fmt_zoom_log2, iter_readout, iter_slot_width, period_readout, zoom_readout,
    zoom_slot_width, MAX_ITER_LIMIT,
};

/// The centre readout's digits grow with the depth; unpadded, the bar wrapped differently at the
/// 2^49 and 2^24 stops of a stepped Home (canvas 1102 → 1104 px on a 1280×800 window, read as an
/// interaction: a black frame, 2026-10-26). One length at every depth, and ASCII only (one
/// monospace cell each: the elision is `...`, not `…`).
#[test]
fn the_centre_readout_never_changes_width_with_depth() {
    let x = fractadyne_core::BigFloat::from_f64(-0.08351966078548609, 4096);
    let y = fractadyne_core::BigFloat::from_f64(0.6563392665142136, 4096);
    let w0 = center_readout(&x, &y, 0.0).chars().count();
    let depths = (0..=20_000).step_by(7).map(f64::from).chain([-3.0, 24.3, 49.1, 62.9, 63.0, 83.0, 1663.37]);
    for l2 in depths {
        let s = center_readout(&x, &y, l2);
        assert_eq!(s.chars().count(), w0, "at 2^{l2}: {s:?}");
        assert!(s.is_ascii(), "at 2^{l2}: {s:?}");
    }
}

/// ⭐⭐**THE REGRESSION NET THE beta.149 REFLOW LOOP NEVER HAD.** The uitest height check passed
/// 26/26 on BOTH the broken and the fixed build, because it held the window width fixed and
/// never varied the ITERATION COUNT — the input that actually moves. This closes that gap at
/// the level where the invariant lives: the bar is monospace, so "never reflows as values
/// change" is exactly "the assembled readout has ONE length per shape, for every value it can
/// show". Deterministic on any machine — a pixel-level sweep would instead inherit each
/// font/DPI's own wrap point, which is how a gate starts crying wolf under load.
///
/// Catches both regressions this entry is about: dropping the slot padding (the original bug)
/// and a future conditional element appended to either readout ("any conditional status-bar
/// element is a reflow bug").
#[test]
fn status_readouts_never_change_width() {
    // Every decade of iteration count up to the panel ceiling, plus the report's own value.
    let mut iters: Vec<u32> = (0..8).map(|d| 10u32.pow(d)).collect();
    iters.extend([1_395_703, MAX_ITER_LIMIT, MAX_ITER_LIMIT - 1, 64]);
    let w0 = iter_readout(1).chars().count();
    for &it in &iters {
        let w = iter_readout(it).chars().count();
        assert_eq!(
            w, w0,
            "iter readout width moved ({w0} → {w} chars at iter {it}) — the bar can wrap when \
             the count changes, which restarts the render loop the count came from"
        );
    }
    // The whole supported magnification range, densely across the 1020-octave format switch
    // (decimal → scientific) — the same sweep discipline `zoom_slot_fits_every_magnification`
    // uses, applied to the ASSEMBLED readout in both shapes.
    let mut zooms: Vec<f64> = (0..=720).map(|i| i as f64 * 100.0).collect();
    zooms.extend((900..=1100).map(|i| i as f64));
    let single0 = zoom_readout(false, 0.0, 0.0).chars().count();
    let dual0 = zoom_readout(true, 0.0, 0.0).chars().count();
    for &z in &zooms {
        assert_eq!(zoom_readout(false, z, 0.0).chars().count(), single0, "single, log2 {z}");
        assert_eq!(zoom_readout(true, z, z / 2.0).chars().count(), dual0, "dual, log2 {z}");
    }
}

#[test]
fn zoom_slot_fits_every_magnification() {
    // Sweep octaves across the whole supported range, densely around the 1020-octave switch
    // between the decimal and scientific forms, and past e21000 (the deepest view this app
    // has rendered). A slot that is one char short reintroduces the wrap loop.
    let mut worst = (0usize, 0.0f64, String::new());
    let mut l2 = 0.0f64;
    while l2 <= 72_000.0 {
        let w = fmt_zoom_log2(l2).chars().count();
        if w > worst.0 {
            worst = (w, l2, fmt_zoom_log2(l2));
        }
        l2 += if (1015.0..1025.0).contains(&l2) { 0.05 } else { 0.37 };
    }
    assert!(
        worst.0 <= zoom_slot_width(),
        "zoom slot {} too narrow: {} chars at log2mag {} ({:?})",
        zoom_slot_width(), worst.0, worst.1, worst.2
    );
}

/// The ambient period slot obeys the same reflow law as the readouts above: ONE width for every
/// period it can show AND for the empty `—` state, or the finder result arriving (or the view
/// moving off a feature) would wrap the bar and restart the render loop. Swept up to the finder's
/// `max_period` clamp of 100,000, the widest value either solve path can report.
#[test]
fn period_readout_never_changes_width() {
    let w0 = period_readout(None).chars().count();
    let mut periods: Vec<u32> = (0..=5).map(|d| 10u32.pow(d)).collect();
    periods.extend([2, 3, 998, 12_345, 99_999, 100_000]);
    for &p in &periods {
        let w = period_readout(Some(p)).chars().count();
        assert_eq!(
            w, w0,
            "period readout width moved ({w0} → {w} chars at period {p}) — the bar can wrap when a \
             feature is solved or left, restarting the render loop"
        );
    }
}

#[test]
fn iter_slot_fits_the_iteration_ceiling() {
    assert_eq!(iter_slot_width(), commas(&MAX_ITER_LIMIT.to_string()).chars().count());
    // Every value the bar can show must fit the slot it reserves.
    for v in [1u32, 999, 1_000, 35_733, 224_000, 1_395_703, MAX_ITER_LIMIT] {
        assert!(
            commas(&v.to_string()).chars().count() <= iter_slot_width(),
            "{v} does not fit the reserved iteration slot"
        );
    }
}
