use super::*;
use fractadyne_core::ir::parse::parse;

#[test]
fn parentheses_pair_by_depth_outside_comments() {
    let src = "z = sin((z + c)) ; (comment)\nt = (z";
    let ps = parens(src);
    let at = |i: usize| ps.iter().find(|p| p.at == i).copied().unwrap();
    // sin( at 7, ( at 8, ) at 14, ) at 15
    assert_eq!(at(7), Paren { at: 7, depth: 0, partner: Some(15) });
    assert_eq!(at(8), Paren { at: 8, depth: 1, partner: Some(14) });
    assert_eq!(at(14), Paren { at: 14, depth: 1, partner: Some(8) });
    assert_eq!(at(15), Paren { at: 15, depth: 0, partner: Some(7) });
    // The comment's are not parentheses; the last line's is unmatched.
    assert_eq!(ps.len(), 5);
    assert_eq!(ps[4].partner, None);
    assert_eq!(&src[ps[4].at..ps[4].at + 1], "(");
    // A stray close is unmatched too, and does not unbalance what follows.
    let ps = parens(") (z)");
    assert_eq!(ps[0].partner, None);
    assert_eq!(ps[1].partner, Some(4));
    // Byte offsets past multi-byte characters (a comment's, then code on the next line).
    let src = "; π √\n(z)";
    assert_eq!(parens(src).iter().map(|p| p.at).collect::<Vec<_>>(), vec![src.find('(').unwrap(), src.find(')').unwrap()]);
    assert_eq!(comments(src), vec![0..src.find('\n').unwrap()]);
    assert_eq!(comments("z ; to the end"), vec![2..14]);
}

#[test]
fn the_pair_at_the_cursor_is_the_one_before_it_else_after_it() {
    let src = "(a)(b)";
    let ps = parens(src);
    assert_eq!(pair_at(src, &ps, 3), Some((2, 0)), "just after the first ')'");
    assert_eq!(pair_at(src, &ps, 0), Some((0, 2)), "just before the first '('");
    assert_eq!(pair_at(src, &ps, 2), Some((2, 0)), "between 'a' and ')': the one after");
    assert_eq!(pair_at(src, &ps, 1), Some((0, 2)), "between '(' and 'a': the one before");
    assert_eq!(pair_at("(a", &parens("(a"), 1), None, "an unmatched one has no pair");
    assert_eq!(pair_at("ab", &[], 1), None);
}

fn look() -> Look {
    Look {
        font: egui::FontId::monospace(14.0),
        text: egui::Color32::WHITE,
        comment: egui::Color32::GRAY,
        depth: [egui::Color32::RED, egui::Color32::GREEN, egui::Color32::BLUE, egui::Color32::YELLOW],
        unmatched: egui::Color32::from_rgb(255, 0, 255),
        pair_bg: egui::Color32::DARK_GRAY,
    }
}

/// The format each byte of `src` is drawn with.
fn format_at(job: &LayoutJob, byte: usize) -> &TextFormat {
    &job.sections.iter().find(|s| s.byte_range.contains(&byte)).unwrap().format
}

#[test]
fn the_layout_colours_depths_highlights_the_pair_and_flags_strays() {
    let l = look();
    let src = "((z)) + (c ; (x)\n)";
    let job = layout_job(src, None, &l);
    // The sections tile the text exactly, in order.
    let mut end = 0;
    for s in &job.sections {
        assert_eq!(s.byte_range.start, end);
        end = s.byte_range.end;
    }
    assert_eq!(end, src.len());
    assert_eq!(job.text, src);
    assert_eq!(format_at(&job, 0).color, l.depth[0]);
    assert_eq!(format_at(&job, 1).color, l.depth[1]);
    assert_eq!(format_at(&job, 3).color, l.depth[1]);
    assert_eq!(format_at(&job, 4).color, l.depth[0]);
    assert_eq!(format_at(&job, 2).color, l.text, "z is plain");
    // The comment, its parenthesis included, is dimmed; the ( before it pairs with the ) after it.
    let semi = src.find(';').unwrap();
    assert_eq!(format_at(&job, semi + 2).color, l.comment);
    assert!(format_at(&job, semi + 2).italics);
    assert_eq!(format_at(&job, 8).color, l.depth[0], "paired across the comment's line");
    // Depths past four take the first colour again.
    let deep = layout_job("(((((z)))))", None, &l);
    assert_eq!(format_at(&deep, 4).color, l.depth[0]);
    // The pair at the cursor: text colour on the pair background, both ends; the rest unchanged.
    let job = layout_job(src, Some(5), &l);
    for b in [0, 4] {
        assert_eq!(format_at(&job, b).color, l.text);
        assert_eq!(format_at(&job, b).background, l.pair_bg);
    }
    assert_eq!(format_at(&job, 1).background, egui::Color32::TRANSPARENT);
    // A stray close is in the unmatched colour.
    let stray = layout_job("z)", None, &l);
    assert_eq!(format_at(&stray, 1).color, l.unmatched);
    // An empty text still has a format (the cursor's height comes from it).
    assert_eq!(layout_job("", None, &l).sections.len(), 1);
}

#[test]
fn the_prefix_is_a_name_being_typed_at_the_cursor() {
    assert_eq!(prefix_at("z = si", 6), Some((4, "si".into())));
    assert_eq!(prefix_at("z = SIn", 7), Some((4, "SIn".into())));
    assert_eq!(prefix_at("z = sin(z)", 7), Some((4, "sin".into())), "a whole name before '('");
    assert_eq!(prefix_at("z = si(z)", 6), Some((4, "si".into())), "a name before '(' completes");
    assert_eq!(prefix_at("z = sinh", 6), None, "the middle of a word does not");
    assert_eq!(prefix_at("z = 1e", 6), None, "an exponent is not a name");
    assert_eq!(prefix_at("z = 1.5e", 8), None);
    assert_eq!(prefix_at("z = .e", 6), None);
    assert_eq!(prefix_at("z = z ; si", 10), None, "not in a comment");
    assert_eq!(prefix_at("z = z ; x\nz = si", 16), Some((14, "si".into())), "the comment ended with its line");
    assert_eq!(prefix_at("π·co", 4), Some((2, "co".into())), "character positions, not bytes");
    assert_eq!(prefix_at("z", 0), None);
    assert_eq!(prefix_at("z", 5), None, "a stale cursor past the end");
    assert_eq!(prefix_at("z + ", 4), None);
}

#[test]
fn candidates_are_the_languages_names_and_the_formulas_variables() {
    let names = |src: &str, p: &str| candidates(src, p).into_iter().map(|c| c.insert).collect::<Vec<_>>();
    assert_eq!(names("", "si"), ["sin(", "sinh("]);
    assert_eq!(names("", "co"), ["conj(", "cos(", "cosh(", "cotan(", "cotanh("]);
    assert_eq!(names("", "SQ"), ["sqr(", "sqrt("], "the parser ignores case, so does completion");
    assert_eq!(names("", "pi"), ["pi", "pixel"]);
    assert_eq!(names("", "p"), ["p1", "p2", "p3", "p4", "p5", "pi", "pixel"]);
    // The formula's own variables, wherever a statement assigns one, but not z or a comment's.
    let src = "tmp = sqr(z), Total = tmp*2 ; tide = 3\nz = t";
    assert_eq!(names(src, "t"), ["tan(", "tanh(", "tmp", "total"]);
    assert_eq!(names(src, "z"), ["z"], "z is the language's, offered once");
    // Every name offered reads in the parser, used as offered.
    let all = candidates("", "");
    assert!(all.len() >= 28, "{} names", all.len());
    for c in &all {
        let src = match c.insert.strip_suffix('(') {
            Some(_) => format!("z = {}z) + c", c.insert),
            None => format!("z = z + {}", c.insert),
        };
        parse(&src).unwrap_or_else(|e| panic!("{:?} gives {src:?}, which does not parse: {e}", c.name));
    }
}

#[test]
fn a_list_opens_only_with_something_to_add() {
    assert!(completion("z = s", 5).is_none(), "one letter is too little");
    assert_eq!(completion("z = si", 6).map(|(s, p, cs)| (s, p, cs.len())), Some((4, "si".into(), 2)));
    assert!(completion("z = z + pixel", 13).is_none(), "a complete name with nothing longer");
    assert!(completion("z = z + p1", 10).is_none());
    assert!(completion("z = sin", 7).is_some(), "sin is complete, but sinh and the '(' are not");
    assert!(completion("z = zz", 6).is_none(), "nothing starts so");
}

#[test]
fn completing_replaces_the_prefix_and_places_the_cursor() {
    let sin = Candidate { name: "sin".into(), insert: "sin(".into(), hint: String::new() };
    let tmp = Candidate { name: "tmp".into(), insert: "tmp".into(), hint: String::new() };
    assert_eq!(complete("z = si", 4, 6, &sin), ("z = sin(".into(), 8));
    assert_eq!(complete("z = si + c", 4, 6, &sin), ("z = sin( + c".into(), 8));
    // Before an existing '(' the name goes in alone and the cursor steps past it.
    assert_eq!(complete("z = si(z)", 4, 6, &sin), ("z = sin(z)".into(), 8));
    assert_eq!(complete("z = TM", 4, 6, &tmp), ("z = tmp".into(), 7));
    assert_eq!(complete("π·si", 2, 4, &sin), ("π·sin(".into(), 6), "character positions");
}

// ---- The field itself, driven through a real (headless) egui context. ----

fn field_id() -> egui::Id {
    egui::Id::new("formula_editor_test_field")
}

fn key(k: egui::Key) -> egui::Event {
    egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }
}

/// One frame: the field, then a button after it (so Tab has somewhere to move focus to).
fn frame(ctx: &egui::Context, events: Vec<egui::Event>, text: &mut String, st: &mut Completion) {
    let input = egui::RawInput {
        events,
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))),
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            source_field(ui, field_id(), text, 4, "", st);
            let _ = ui.button("Apply");
        });
    });
}

/// A focused field holding `text` with the cursor at its end.
fn focused_field(text: &mut String, st: &mut Completion) -> egui::Context {
    let ctx = egui::Context::default();
    frame(&ctx, vec![], text, st);
    store_cursor(&ctx, field_id(), text.chars().count());
    ctx.memory_mut(|m| m.request_focus(field_id()));
    frame(&ctx, vec![], text, st);
    frame(&ctx, vec![], text, st);
    assert!(ctx.memory(|m| m.has_focus(field_id())), "the test field has focus");
    ctx
}

fn has_focus(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.has_focus(field_id()))
}

#[test]
fn tab_completes_and_keeps_the_focus_in_the_field() {
    let (mut text, mut st) = ("z = ".to_string(), Completion::default());
    let ctx = focused_field(&mut text, &mut st);
    frame(&ctx, vec![egui::Event::Text("si".into())], &mut text, &mut st);
    assert_eq!(text, "z = si");
    frame(&ctx, vec![key(egui::Key::Tab)], &mut text, &mut st);
    assert_eq!(text, "z = sin(");
    assert!(has_focus(&ctx), "Tab was the list's, not a move to the next widget");
    // Typing continues where the completion left the cursor.
    frame(&ctx, vec![egui::Event::Text("z) + c".into())], &mut text, &mut st);
    assert_eq!(text, "z = sin(z) + c");
}

#[test]
fn arrows_choose_and_enter_takes_the_choice() {
    let (mut text, mut st) = ("z = ".to_string(), Completion::default());
    let ctx = focused_field(&mut text, &mut st);
    frame(&ctx, vec![egui::Event::Text("si".into())], &mut text, &mut st);
    frame(&ctx, vec![key(egui::Key::ArrowDown)], &mut text, &mut st);
    frame(&ctx, vec![key(egui::Key::Enter)], &mut text, &mut st);
    assert_eq!(text, "z = sinh(", "the second entry, and no line break");
    assert!(has_focus(&ctx));
}

#[test]
fn enter_after_a_complete_name_is_a_new_line_and_esc_closes_the_list_only() {
    let (mut text, mut st) = ("z = z + ".to_string(), Completion::default());
    let ctx = focused_field(&mut text, &mut st);
    frame(&ctx, vec![egui::Event::Text("pixel".into())], &mut text, &mut st);
    frame(&ctx, vec![key(egui::Key::Enter)], &mut text, &mut st);
    assert_eq!(text, "z = z + pixel\n", "nothing to complete: Enter is the field's");
    // Esc closes an open list and leaves the field focused; the next letter reopens it.
    frame(&ctx, vec![egui::Event::Text("t = co".into())], &mut text, &mut st);
    frame(&ctx, vec![key(egui::Key::Escape)], &mut text, &mut st);
    assert!(has_focus(&ctx), "Esc was the list's");
    frame(&ctx, vec![key(egui::Key::Tab)], &mut text, &mut st);
    assert_eq!(text, "z = z + pixel\nt = co", "the list stayed closed: Tab did not complete");
    let reopened = completion(&text, text.chars().count()).is_some();
    assert!(reopened);
}

#[test]
fn a_press_on_the_list_takes_the_name_before_the_list_goes() {
    let (mut text, mut st) = ("z = ".to_string(), Completion::default());
    let ctx = focused_field(&mut text, &mut st);
    frame(&ctx, vec![egui::Event::Text("sq".into())], &mut text, &mut st);
    let area = ctx.memory(|m| m.area_rect(field_id().with("completion"))).expect("the list is drawn");
    // Inside the first row (sqr), past the popup frame's margin.
    let at = area.min + egui::vec2(14.0, 14.0);
    frame(&ctx, vec![egui::Event::PointerMoved(at)], &mut text, &mut st);
    assert_eq!(ctx.layer_id_at(at).map(|l| l.order), Some(egui::Order::Foreground), "the press is on the list");
    frame(
        &ctx,
        vec![egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() }],
        &mut text,
        &mut st,
    );
    assert_eq!(text, "z = sqr(");
    frame(
        &ctx,
        vec![egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() }],
        &mut text,
        &mut st,
    );
    assert!(has_focus(&ctx), "focus came back to the field");
}
