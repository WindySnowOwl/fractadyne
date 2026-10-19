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
