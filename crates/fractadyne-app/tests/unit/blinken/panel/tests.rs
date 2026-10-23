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
        assert_eq!(spec.lamp as usize, i, "lamp {} out of order", spec.label);
    }
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
