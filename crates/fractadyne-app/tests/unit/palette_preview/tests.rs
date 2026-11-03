use super::*;

/// One frame with a picker open and the pointer on `hover` (or on no row).
fn frame(p: &mut PalettePreview, now: f64, hover: Option<PaletteRow>) -> Settle {
    p.picker_open();
    if let Some(row) = hover {
        p.row(row, egui::Rect::NOTHING, true);
    }
    p.end_frame(now)
}

const EMBER: PaletteRow = PaletteRow::Preset(3);
const MINE: PaletteRow = PaletteRow::Saved(0);

#[test]
fn a_row_previews_only_after_the_pointer_rests_on_it() {
    let mut p = PalettePreview::default();
    let s = frame(&mut p, 10.0, Some(EMBER));
    assert_eq!(p.shown(), None);
    assert!(!s.changed);
    // A still pointer must be woken for its preview: egui repaints only on input.
    assert!(s.wake_in.is_some_and(|w| (w - DELAY_S).abs() < 1e-9), "{s:?}");
    let s = frame(&mut p, 10.0 + DELAY_S * 0.5, Some(EMBER));
    assert_eq!(p.shown(), None);
    assert!(s.wake_in.is_some_and(|w| (w - DELAY_S * 0.5).abs() < 1e-9), "{s:?}");
    let s = frame(&mut p, 10.0 + DELAY_S, Some(EMBER));
    assert_eq!(p.shown(), Some(EMBER));
    assert_eq!(s, Settle { changed: true, wake_in: None });
    // Resting on: nothing more to do.
    assert_eq!(frame(&mut p, 11.0, Some(EMBER)), Settle { changed: false, wake_in: None });
}

#[test]
fn sweeping_past_rows_previews_none_of_them() {
    let mut p = PalettePreview::default();
    let step = DELAY_S / 4.0;
    for (k, i) in (0..8).enumerate() {
        frame(&mut p, k as f64 * step, Some(PaletteRow::Preset(i)));
        assert_eq!(p.shown(), None, "row {i}");
    }
}

#[test]
fn moving_to_the_next_row_keeps_the_last_preview_until_its_own_pause_runs_out() {
    let mut p = PalettePreview::default();
    frame(&mut p, 0.0, Some(EMBER));
    frame(&mut p, DELAY_S, Some(EMBER));
    assert_eq!(p.shown(), Some(EMBER));
    // Crossing the gap between two rows, then onto the saved gradient: the view must not flash
    // the chosen palette in between.
    frame(&mut p, 1.0, None);
    assert_eq!(p.shown(), Some(EMBER));
    frame(&mut p, 1.05, Some(MINE));
    assert_eq!(p.shown(), Some(EMBER));
    frame(&mut p, 1.05 + DELAY_S, Some(MINE));
    assert_eq!(p.shown(), Some(MINE));
}

#[test]
fn resting_off_every_row_goes_back_to_the_chosen_palette() {
    let mut p = PalettePreview::default();
    frame(&mut p, 0.0, Some(EMBER));
    frame(&mut p, DELAY_S, Some(EMBER));
    frame(&mut p, 1.0, None);
    let s = frame(&mut p, 1.0 + DELAY_S, None);
    assert_eq!(p.shown(), None);
    assert!(s.changed);
}

#[test]
fn the_first_frame_with_no_picker_open_reverts_however_it_closed() {
    let mut p = PalettePreview::default();
    frame(&mut p, 0.0, Some(EMBER));
    frame(&mut p, DELAY_S, Some(EMBER));
    assert_eq!(p.shown(), Some(EMBER));
    // Escape, a click outside, the panel scrolled away: all the same to this, a frame in which no
    // picker said it was open.
    let s = p.end_frame(1.0);
    assert_eq!(p.shown(), None);
    assert_eq!(s, Settle { changed: true, wake_in: None });
    // And nothing carries over into the next opening: its first row waits its own pause.
    frame(&mut p, 2.0, Some(MINE));
    assert_eq!(p.shown(), None);
}

#[test]
fn a_closed_picker_with_nothing_shown_asks_for_nothing() {
    let mut p = PalettePreview::default();
    assert_eq!(p.end_frame(0.0), Settle { changed: false, wake_in: None });
    // A pause cut short by closing never fires later.
    frame(&mut p, 1.0, Some(EMBER));
    p.end_frame(1.1);
    assert_eq!(p.end_frame(5.0), Settle { changed: false, wake_in: None });
    assert_eq!(p.shown(), None);
}

#[test]
fn a_report_counts_for_one_frame_only() {
    let mut p = PalettePreview::default();
    frame(&mut p, 0.0, Some(EMBER));
    // The pointer is still on Ember but the picker did not report this frame: it has closed.
    p.row(EMBER, egui::Rect::NOTHING, true);
    p.end_frame(DELAY_S);
    assert_eq!(p.shown(), None);
}

#[test]
fn a_row_not_under_the_pointer_is_drawn_but_not_previewed() {
    let mut p = PalettePreview::default();
    let at = egui::Rect::from_min_size(egui::pos2(10.0, 40.0), egui::vec2(120.0, 18.0));
    for k in 0..3 {
        p.picker_open();
        p.row(EMBER, at, false);
        p.row(MINE, egui::Rect::NOTHING, false);
        p.end_frame(k as f64 * DELAY_S);
    }
    assert_eq!(p.shown(), None);
    // Where the last frame drew it: what the harness aims a pointer at.
    assert_eq!(p.row_rect(EMBER), Some(at));
    assert_eq!(p.row_rect(PaletteRow::Preset(9)), None);
    // A closed picker drew nothing.
    p.end_frame(10.0);
    assert_eq!(p.row_rect(EMBER), None);
}
