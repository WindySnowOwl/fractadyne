use super::*;

#[test]
fn a_price_reads_greener_when_cheap_and_redder_past_the_target() {
    let hue = |c: Color32| {
        let [r, g, _, _] = c.to_srgba_unmultiplied();
        g as f32 - r as f32
    };
    assert!(hue(price_color(2.0, 1.0)) > 0.0, "cheap is green");
    assert!(hue(price_color(30.0, 1.0)) < 0.0, "over the target is red");
    assert!(hue(price_color(2.0, 1.0)) > hue(price_color(12.0, 1.0)));
    assert!(hue(price_color(12.0, 1.0)) > hue(price_color(30.0, 1.0)));
    // Alpha passes through.
    assert_eq!(price_color(5.0, 0.0).a(), 0);
}

#[test]
fn counts_are_comma_grouped() {
    assert_eq!(group(0), "0");
    assert_eq!(group(999), "999");
    assert_eq!(group(16_384), "16,384");
    assert_eq!(group(1_234_567), "1,234,567");
}

#[test]
fn the_lamps_wrap_onto_more_rows_as_the_panel_narrows() {
    assert_eq!(bank_rows(10_000.0), 1);
    assert!(bank_rows(944.0) >= 2);
    assert!(bank_rows(300.0) > bank_rows(944.0));
}

#[test]
fn every_lamp_on_the_bank_has_its_key_in_order() {
    let keys: Vec<&str> = LAMP_SPECS.iter().map(|l| l.key).collect();
    assert_eq!(keys, LAMP_KEYS);
    for (i, spec) in LAMP_SPECS.iter().enumerate() {
        assert_eq!(spec.lamp as usize, i, "lamp {} out of order", spec.key);
    }
}

/// Runs `f` with a painter that has the panel's typefaces, as the app installs them.
fn with_panel_fonts(mut f: impl FnMut(&Painter)) {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    paint::install_fonts(&mut fonts);
    ctx.set_fonts(fonts);
    let raw = || egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0))), ..Default::default() };
    // The fonts take effect on the next frame.
    let _ = ctx.run(raw(), |_| {});
    let _ = ctx.run(raw(), |ctx| f(&ctx.layer_painter(egui::LayerId::background())));
}

#[test]
fn the_english_lamp_labels_break_as_they_always_did_and_a_long_translation_fits_its_cell() {
    with_panel_fonts(|p| {
        // Measured rather than reckoned from the letter count, the English labels keep their look:
        // only REF AHEAD goes on two lines, and none is drawn smaller.
        for spec in LAMP_SPECS.iter() {
            let label = lamp_label(spec.lamp);
            let (lines, k) = lamp_label_lines(p, &label);
            let want: Vec<String> = if spec.lamp == Lamp::RefAhead { vec!["REF".into(), "AHEAD".into()] } else { vec![label.clone()] };
            assert_eq!(lines, want, "{label}");
            assert_eq!(k, 1.0, "{label} drawn smaller");
        }
        // A longer label breaks at the space that keeps its longer line shortest...
        assert_eq!(lamp_label_lines(p, "REF EN AVANCE").0, ["REF EN", "AVANCE"]);
        // ...and a word wider than the cell is drawn smaller, not over its neighbours.
        let (lines, k) = lamp_label_lines(p, "ÜBERSCHREIBUNGEN");
        assert_eq!(lines.len(), 1);
        assert!((0.6..1.0).contains(&k), "{k}");
    });
}

#[test]
fn a_title_wider_than_its_room_is_drawn_smaller_and_an_english_one_is_not() {
    with_panel_fonts(|p| {
        let at = Pos2::ZERO;
        let full = title_rect(p, at, 1.0, "PICTURE AGE", f32::INFINITY);
        assert_eq!(title_rect(p, at, 1.0, "PICTURE AGE", 220.0), full, "the age meter's title fits its face");
        // Type is set in whole pixels: the scale is measured and stepped down until the title fits.
        let room = full.width() * 0.8;
        let squeezed = title_rect(p, at, 1.0, "PICTURE AGE", room);
        assert!(squeezed.width() <= room && squeezed.width() > full.width() * 0.6, "{squeezed:?} in {room} vs {full:?}");
    });
}

#[test]
fn sizes_cycle_in_order() {
    assert_eq!(Size::Off.next(), Size::Strip);
    assert_eq!(Size::Full.next(), Size::Off);
}

#[test]
fn a_pass_is_coloured_against_the_time_it_was_sized_for() {
    // The pin's 20 ms target is the price colour's.
    assert_eq!(target_color(12.0, 20.0, 1.0), price_color(12.0, 1.0));
    // A still view's 300 ms pass against its 400 ms target reads as a 15 ms pin pass would, not red.
    assert_eq!(target_color(300.0, 400.0, 1.0), price_color(15.0, 1.0));
    let [r, g, _, _] = target_color(600.0, 400.0, 1.0).to_srgba_unmultiplied();
    assert!(r > g, "past its target is red");
    // A target of nothing is not a division by zero.
    assert!(target_color(5.0, 0.0, 1.0).a() > 0);
}

#[test]
fn a_walk_pass_caption_names_its_walk_and_its_band() {
    let res = (2400, 1300);
    let whole = |band, settled| tile_labels(&Plane::Whole { res, band, settled, glow: 1.0 }, res).0;
    assert_eq!(whole(Some((1_024, 2_048)), false), "PIN PASS · BAND 1,024–2,048");
    assert_eq!(whole(Some((147_000, 200_000)), true), "SETTLE PASS · BAND 147,000–200,000");
    assert_eq!(whole(None, false), "MOVING FRAME · WHOLE PICTURE");
    assert_eq!(whole(None, true), "SETTLED FRAME · WHOLE PICTURE");
    let ssaa = |band| tile_labels(&Plane::Ssaa { res, samples: 3, band, glow: 1.0 }, res);
    assert_eq!(ssaa(Some((0, 65_536))).0, "SSAA · 3 DONE · BAND 0–65,536");
    assert_eq!(ssaa(None).0, "SSAA SAMPLE 3 · WHOLE FRAME");
    assert_eq!(ssaa(Some((0, 65_536))).2, "1 TILE/SAMPLE");
}

#[test]
fn the_close_button_sits_in_the_case_margin_at_every_size() {
    for size in [Size::Strip, Size::Panel, Size::Full] {
        for s in [0.35f32, 0.7, 1.0] {
            let r = Rect::from_min_size(pos2(100.0, 50.0), vec2(DESIGN_W, height(size, DESIGN_W)) * s);
            let c = close_center(r, s);
            let rad = 6.5 * s;
            // Inside the case, clear of its rounded corner and its edges...
            assert!(r.shrink(2.0 * s).contains(c - vec2(rad, rad)) && r.shrink(2.0 * s).contains(c + vec2(rad, rad)), "{size:?} at {s}");
            // ...and in the right-hand margin, where no instrument is drawn.
            assert!(c.x - rad >= r.right() - PAD_X * s, "{size:?} at {s}: {} vs {}", c.x - rad, r.right() - PAD_X * s);
        }
    }
}
