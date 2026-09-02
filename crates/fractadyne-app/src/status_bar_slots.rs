use super::{
    commas, fmt_zoom_log2, iter_readout, iter_slot_width, zoom_readout, zoom_slot_width,
    MAX_ITER_LIMIT,
};

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
