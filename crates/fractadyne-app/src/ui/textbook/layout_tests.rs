use super::*;

const CTX: Ctx = Ctx { size_pt: 18.0, ppp: 1.5 };

fn units(st: St, v: f32) -> f32 {
    CTX.u(st, v)
}

fn glyphs_of(b: &LBox) -> Vec<(char, f32, f32)> {
    b.items
        .iter()
        .filter_map(|it| match *it {
            Item::Glyph { ch, x, y, .. } => Some((ch, x, y)),
            _ => None,
        })
        .collect()
}

fn rules_of(b: &LBox) -> Vec<(f32, f32, f32, f32)> {
    b.items
        .iter()
        .filter_map(|it| match *it {
            Item::Rule { x, y, w, h } => Some((x, y, w, h)),
            _ => None,
        })
        .collect()
}

fn frac(num: Vec<Node>, den: Vec<Node>) -> Node {
    Node::Frac { num, den }
}

/// TeX's rule 15: the bar on the math axis, numerator and denominator clear of it by the MATH gaps,
/// both centred over the wider of the two.
#[test]
fn a_fraction_sits_on_the_axis_with_its_gaps() {
    let c = &font().constants;
    let st = St::DISPLAY;
    let n = vec![Node::var("z"), Node::bin('+'), Node::var("c")];
    let d = vec![Node::var("z")];
    let b = node(&frac(n.clone(), d.clone()), st, &CTX);
    let (_, y, w, h) = rules_of(&b)[0];
    let axis = units(st, c.axis_height);
    assert!((y + h / 2.0 + axis).abs() < 1e-3, "bar centre {} on the axis {}", y + h / 2.0, -axis);
    assert!((h - units(st, c.fraction_rule_thickness)).abs() < 1e-4);
    // The numerator's ink bottom clears the bar by the display gap; the denominator's top likewise.
    let (nb, db) = (hlist(&n, st.num(), &CTX), hlist(&d, st.den(), &CTX));
    let num_baseline = glyphs_of(&b)[0].2;
    assert!((y) - (num_baseline + nb.depth) >= units(st, c.fraction_num_display_style_gap_min) - 1e-3);
    let den_baseline = glyphs_of(&b).last().unwrap().2;
    assert!((den_baseline - db.height) - (y + h) >= units(st, c.fraction_denom_display_style_gap_min) - 1e-3);
    // The bar spans the numerator (the wider); the denominator is centred under it.
    assert!((w - nb.width).abs() < 1e-3);
    // A numerator is set one style smaller: Text in Display, Script in Text.
    assert_eq!(St::DISPLAY.num().style, Style::Text);
    assert_eq!(St { style: Style::Text, cramped: false }.num().style, Style::Script);
    assert!(St::DISPLAY.den().cramped);
}

/// Rule 18: an exponent on a letter is raised by superscriptShiftUp at least, in script size.
#[test]
fn an_exponent_is_raised_and_set_smaller() {
    let c = &font().constants;
    let st = St::DISPLAY;
    let b = node(&Node::Scripts { base: Box::new(Node::var("z")), sup: Some(vec![Node::num("2")]), sub: None }, st, &CTX);
    let g = glyphs_of(&b);
    let (two_y, two_size) = match b.items[1] {
        Item::Glyph { y, size, .. } => (y, size),
        _ => unreachable!(),
    };
    assert!(-two_y >= units(st, c.superscript_shift_up) - 1e-3, "raised {}", -two_y);
    assert!((two_size - 18.0 * 0.7).abs() < 1e-4, "script size");
    // It starts after the letter's italic correction (z leans right).
    assert!(g[1].1 >= CTX.u(st, font().advance(italic('z'))) - 1e-3);
    // A compound base (a fraction) drops its exponent's baseline by at most the font's maximum.
    let f = node(
        &Node::Scripts { base: Box::new(frac(vec![Node::num("1")], vec![Node::var("z")])), sup: Some(vec![Node::num("2")]), sub: None },
        st,
        &CTX,
    );
    let fb = node(&frac(vec![Node::num("1")], vec![Node::var("z")]), st, &CTX);
    let sup_y = glyphs_of(&f).last().unwrap().2;
    assert!(-sup_y >= fb.height - units(st, c.superscript_baseline_drop_max) - 1e-3);
}

/// An exponent after parentheses or bars that did not grow goes where LaTeX puts one after a plain
/// `)`: at superscriptShiftUp, as on a letter — not by the group's height, as after `\right)`.
#[test]
fn an_exponent_after_ordinary_delimiters_sits_as_on_a_letter() {
    let c = &font().constants;
    let st = St::DISPLAY;
    let sup_y = |base: Node| {
        let b = node(&Node::Scripts { base: Box::new(base), sup: Some(vec![Node::num("2")]), sub: None }, st, &CTX);
        glyphs_of(&b).last().unwrap().2
    };
    let bars = sup_y(Node::Fenced { open: '|', close: '|', body: vec![Node::var("z")] });
    let letter = sup_y(Node::var("z"));
    assert!((bars - letter).abs() < 1e-3, "|z|^2 at {bars}, z^2 at {letter}");
    assert!((-bars - units(st, c.superscript_shift_up)).abs() < 1e-3);
    // Grown ones raise it with the box, as \right) does.
    let tall = sup_y(Node::Fenced { open: '(', close: ')', body: vec![frac(vec![Node::num("1")], vec![Node::var("z")])] });
    assert!(-tall > -letter + 1.0, "{tall} vs {letter}");
}

/// TeX's spacing: a binary minus between operands gets medium spaces; one with nothing on its left
/// is a unary minus and gets none; a relation gets thick spaces.
#[test]
fn binary_and_unary_minus_and_relations_are_spaced_as_tex_spaces_them() {
    let st = St::DISPLAY;
    let em = CTX.em(st);
    let w = |nodes: &[Node]| hlist(nodes, st, &CTX).width;
    let (a, m, b) = (Node::var("a"), Node::bin('\u{2212}'), Node::var("b"));
    let plain = w(&[a.clone()]) + w(&[m.clone()]) + w(&[b.clone()]);
    assert!((w(&[a.clone(), m.clone(), b.clone()]) - plain - 2.0 * 4.0 * em / 18.0).abs() < 1e-3);
    assert!((w(&[m.clone(), b.clone()]) - w(&[m.clone()]) - w(&[b.clone()])).abs() < 1e-3, "unary: no space");
    let eq = Node::rel('=');
    assert!((w(&[a.clone(), eq.clone(), b.clone()]) - w(&[a.clone()]) - w(&[eq]) - w(&[b.clone()]) - 2.0 * 5.0 * em / 18.0).abs() < 1e-3);
    // In script style the binary spaces go.
    let s = St { style: Style::Script, cramped: false };
    let ws = |nodes: &[Node]| hlist(nodes, s, &CTX).width;
    assert!((ws(&[a.clone(), m.clone(), b.clone()]) - ws(&[a]) - ws(&[m]) - ws(&[b])).abs() < 1e-3);
    // An operator name before its parenthesised argument: no space — sin(z), not sin (z) — though
    // the parentheses grow; and none between a number and a parenthesis: 2(z + 1).
    let call = [Node::op("sin"), Node::Fenced { open: '(', close: ')', body: vec![Node::var("z")] }];
    assert!((w(&call) - w(&call[..1]) - w(&call[1..])).abs() < 1e-3);
    let times = [Node::num("2"), Node::Fenced { open: '(', close: ')', body: vec![Node::var("z")] }];
    assert!((w(&times) - w(&times[..1]) - w(&times[1..])).abs() < 1e-3);
    // A fraction is Inner: a thin space from an ordinary symbol beside it (2 ½), as TeX sets it.
    let half = [Node::num("2"), frac(vec![Node::num("1")], vec![Node::num("2")])];
    assert!((w(&half) - w(&half[..1]) - w(&half[1..]) - 3.0 * em / 18.0).abs() < 1e-3);
}

/// Delimiters grow with their content: the plain glyph around a letter, a larger size variant around
/// a display fraction, an assembly of parts around a tower of them — and always cover the content.
#[test]
fn parentheses_grow_from_glyph_to_variant_to_assembly() {
    let st = St::DISPLAY;
    let fenced = |body: Vec<Node>| node(&Node::Fenced { open: '(', close: ')', body }, st, &CTX);
    let open_glyphs = |b: &LBox| glyphs_of(b).into_iter().filter(|(_, x, _)| *x == 0.0).map(|(c, _, _)| c).collect::<Vec<_>>();
    assert_eq!(open_glyphs(&fenced(vec![Node::var("z")])), ['(']);
    let one = fenced(vec![frac(vec![Node::var("z")], vec![Node::num("2")])]);
    let g = open_glyphs(&one);
    assert_eq!(g.len(), 1);
    assert!(('\u{E000}'..='\u{F8FF}').contains(&g[0]), "a size variant, not the plain glyph");
    let mut tower = vec![Node::var("z")];
    for _ in 0..6 {
        tower = vec![frac(tower, vec![Node::num("2")])];
    }
    let tall = fenced(tower.clone());
    assert!(open_glyphs(&tall).len() >= 3, "an assembly: {:?}", open_glyphs(&tall));
    // Coverage: the delimiter spans at least TeX's 90.1% of the content's extent about the axis.
    let inner = hlist(&tower, st, &CTX);
    let axis = units(st, font().constants.axis_height);
    let delta = (inner.height - axis).max(inner.depth + axis);
    assert!(tall.height + tall.depth >= 2.0 * delta * 0.901 - 1e-3, "{} vs {}", tall.height + tall.depth, 2.0 * delta);
    // Each delimiter, variant or assembly, is centred on the axis (the content need not be).
    for target in [0.0, 30.0, 200.0] {
        let d = delimiter('(', target, st, &CTX);
        assert!(((d.height - d.depth) / 2.0 - axis).abs() < 1e-3, "{target}: {} / {}", d.height, d.depth);
        assert!(d.height + d.depth >= target - 1e-3, "{target}: tall enough");
    }
}

/// Rule 11: the vinculum clears the radicand by the radical gap and reaches across it.
#[test]
fn a_radical_covers_its_radicand() {
    let c = &font().constants;
    let st = St::DISPLAY;
    let body = vec![Node::var("z"), Node::bin('+'), Node::var("c")];
    let b = node(&Node::Radical(body.clone()), st, &CTX);
    let inner = hlist(&body, st.cramp(), &CTX);
    let (x, y, w, h) = rules_of(&b)[0];
    assert!(-(y + h) - inner.height >= units(st, c.radical_display_style_vertical_gap) - 1e-3);
    assert!((w - inner.width).abs() < 1e-3);
    assert!((x + w - b.width).abs() < 1e-3, "the vinculum ends with the radicand");
}

/// `align*`: rows line up at their first relation.
#[test]
fn statements_align_at_their_equals_signs() {
    let r1 = vec![Node::var("t"), Node::rel('='), Node::var("z")];
    let r2 = vec![Node::var("z"), Node::var("z"), Node::var("z"), Node::rel('='), Node::var("t")];
    let laid = rows(&[r1, r2], &CTX);
    let eqs: Vec<f32> = laid
        .items
        .iter()
        .filter_map(|it| match *it {
            Item::Glyph { ch: '=', x, .. } => Some(x),
            _ => None,
        })
        .collect();
    assert_eq!(eqs.len(), 2);
    assert!((eqs[0] - eqs[1]).abs() < 1e-3, "{eqs:?}");
    assert!(laid.size.x > 0.0 && laid.size.y > 0.0);
}

/// Nothing the layout emits is missing from the font (the math family has no fallback).
#[test]
fn every_emitted_glyph_is_in_the_font() {
    let st = St::DISPLAY;
    let mut tower = vec![Node::var("z")];
    for _ in 0..6 {
        tower = vec![frac(tower, vec![Node::num("2")])];
    }
    let all = vec![
        Node::Fenced { open: '|', close: '|', body: tower.clone() },
        Node::Radical(tower.clone()),
        Node::Overline(vec![Node::var("t")]),
        Node::Scripts { base: Box::new(Node::var("p")), sup: None, sub: Some(vec![Node::num("1")]) },
        Node::Fenced { open: '(', close: ')', body: tower },
        Node::Slot,
    ];
    let b = hlist(&all, st, &CTX);
    for (ch, _, _) in glyphs_of(&b) {
        assert!(font().has(ch), "U+{:04X}", ch as u32);
    }
}
