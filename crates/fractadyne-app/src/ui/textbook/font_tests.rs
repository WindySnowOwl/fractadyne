use super::*;

/// The bundled file IS the Latin Modern Math subset: its MATH table carries Latin Modern's values
/// (a wrong or re-made font would shift every formula without a single test noticing otherwise).
#[test]
fn the_font_is_latin_modern_maths_subset_with_its_math_table() {
    let f = font();
    assert_eq!(f.units_per_em, 1000.0);
    let c = &f.constants;
    assert_eq!((c.script_percent, c.script_script_percent), (70.0, 50.0));
    assert_eq!((c.axis_height, c.fraction_rule_thickness, c.radical_rule_thickness), (250.0, 40.0, 40.0));
    assert_eq!((c.fraction_numerator_shift_up, c.fraction_numerator_display_style_shift_up), (394.0, 677.0));
    assert_eq!(c.delimited_sub_formula_min_height, 1300.0);
    assert!(c.min_connector_overlap > 0.0);
}

/// Everything the layout can emit is drawable: the math italic alphabet, the operators, and every
/// size variant and assembly part of the stretchy glyphs — which only have characters because the
/// subset script gave them Private Use Area code points.
#[test]
fn every_glyph_the_layout_can_emit_has_a_character() {
    let f = font();
    for ch in ('a'..='z').chain('A'..='Z') {
        assert!(f.has(crate::ui::textbook::layout::italic(ch)), "math italic {ch}");
        assert!(f.has(ch), "upright {ch}");
    }
    for ch in "0123456789.+=,()|\u{2212}\u{22C5}\u{00D7}\u{221A}\u{03C0}\u{1D70B}".chars() {
        assert!(f.has(ch), "U+{:04X}", ch as u32);
    }
    for ch in ['(', ')', '|', '\u{221A}'] {
        let vs = f.vertical_variants(ch);
        assert!(vs.len() >= 5, "{ch}: {} variants", vs.len());
        assert_eq!(vs[0].0, ch, "the smallest variant is the glyph itself");
        assert!(vs.windows(2).all(|w| w[1].1 > w[0].1), "{ch}: variants grow");
        let parts = f.vertical_assembly(ch);
        assert!(parts.len() >= 3 && parts.iter().any(|p| p.extender), "{ch}: an assembly with an extender");
        for (g, _) in vs.iter().skip(1) {
            assert!(('\u{E000}'..='\u{F8FF}').contains(g), "{ch}: variant at U+{:04X}", *g as u32);
        }
        for p in parts {
            assert!(f.has(p.ch));
        }
    }
    // A glyph maps back to its ordinary character, not to a PUA alias.
    assert_eq!(f.char_of(f.glyph('(').unwrap()), Some('('));
}

/// egui rounds the rasteriser's scale to whole pixels; the layout must measure with the em that is
/// actually drawn, which is within half a pixel of the one asked for.
#[test]
fn the_rendered_em_is_the_one_egui_draws() {
    let f = font();
    for (size, ppp) in [(18.0, 1.0), (18.0, 1.5), (18.0, 2.0), (12.6, 1.5), (9.0, 1.25)] {
        let em = f.rendered_em(size, ppp);
        let k = f.height_unscaled / f.units_per_em;
        assert!((em - size).abs() * ppp * k <= 0.5 + 1e-4, "{size} pt at {ppp}x drew {em}");
        assert_eq!((em * ppp * k).round(), em * ppp * k, "a whole number of pixels");
    }
}
