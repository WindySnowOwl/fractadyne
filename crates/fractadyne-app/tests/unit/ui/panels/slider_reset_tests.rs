use super::slider_double_clicked;

/// One frame of a 400x200 window holding one slider over `value`: its rect, whether egui's own
/// `double_clicked` fired, and whether [`slider_double_clicked`] did.
fn frame(ctx: &egui::Context, time: f64, events: Vec<egui::Event>, value: &mut f32) -> (egui::Rect, bool, bool) {
    let mut out = (egui::Rect::NOTHING, false, false);
    let input = egui::RawInput {
        events,
        time: Some(time),
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0))),
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let r = ui.add(egui::Slider::new(value, 0.0..=1.0));
            out = (r.rect, r.double_clicked(), slider_double_clicked(&r));
        });
    });
    out
}

/// A double-click at `at` (as `frame`s a few ms apart): whether egui's `double_clicked` and
/// [`slider_double_clicked`] fired on any of its frames.
fn double_click(ctx: &egui::Context, at: egui::Pos2, value: &mut f32) -> (bool, bool) {
    let button = |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    let (mut egui_says, mut ours) = (false, false);
    let steps = [vec![egui::Event::PointerMoved(at)], vec![button(true)], vec![button(false)], vec![button(true)], vec![button(false)], vec![]];
    for (k, events) in steps.into_iter().enumerate() {
        let (_, e, o) = frame(ctx, 1.0 + 0.05 * k as f64, events, value);
        egui_says |= e;
        ours |= o;
    }
    (egui_says, ours)
}

#[test]
fn a_double_click_on_the_slider_track_counts_where_egui_alone_does_not() {
    let ctx = egui::Context::default();
    let mut v = 0.5f32;
    let (rect, _, _) = frame(&ctx, 0.0, vec![], &mut v);
    assert!(rect.width() > 40.0, "the slider was laid out: {rect:?}");
    // On the track, well left of the value box at its right end.
    let (egui_says, ours) = double_click(&ctx, egui::pos2(rect.left() + 12.0, rect.center().y), &mut v);
    assert!(!egui_says, "egui's own double_clicked fired on the track (the reason for the helper is gone)");
    assert!(ours, "a double-click on the track was not seen");
}

#[test]
fn a_double_click_soon_after_a_click_counts_though_egui_calls_it_a_triple() {
    let ctx = egui::Context::default();
    let mut v = 0.5f32;
    let (rect, _, _) = frame(&ctx, 0.0, vec![], &mut v);
    let at = egui::pos2(rect.left() + 12.0, rect.center().y);
    let button = |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    // One click (released at 0.54 s), then a double-click from 1.00 s, as the uitest walk fed it:
    // its first release is too late to pair with the click (0.48 s, over egui's 0.3 s), but its
    // second lands within 0.6 s of the click's, so egui counts it as a third click, not a second.
    for (k, events) in [vec![egui::Event::PointerMoved(at)], vec![button(true)], vec![button(false)]].into_iter().enumerate() {
        frame(&ctx, 0.5 + 0.02 * k as f64, events, &mut v);
    }
    let (mut egui_double, mut ours) = (false, false);
    for (k, events) in [vec![button(true)], vec![button(false)], vec![button(true)], vec![button(false)], vec![]].into_iter().enumerate() {
        let mut input_double = false;
        let (_, _, o) = frame(&ctx, 1.0 + 0.02 * k as f64, events, &mut v);
        ctx.input(|i| input_double = i.pointer.button_double_clicked(egui::PointerButton::Primary));
        egui_double |= input_double;
        ours |= o;
    }
    assert!(!egui_double, "egui counted the double-click as a double (the triple case is gone)");
    assert!(ours, "a double-click soon after a click was not seen");
}

#[test]
fn no_double_click_off_the_slider_and_none_for_a_single_click() {
    let ctx = egui::Context::default();
    let mut v = 0.5f32;
    let (rect, _, _) = frame(&ctx, 0.0, vec![], &mut v);
    // Below the slider: the window's empty space.
    let (_, ours) = double_click(&ctx, egui::pos2(rect.left() + 12.0, rect.bottom() + 60.0), &mut v);
    assert!(!ours, "a double-click off the slider counted");
    // One click on the track: no double-click.
    let at = egui::pos2(rect.left() + 12.0, rect.center().y);
    let button = |pressed| egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    let mut seen = false;
    for (k, events) in [vec![egui::Event::PointerMoved(at)], vec![button(true)], vec![button(false)], vec![]].into_iter().enumerate() {
        seen |= frame(&ctx, 5.0 + 0.05 * k as f64, events, &mut v).2;
    }
    assert!(!seen, "a single click counted as a double-click");
}
