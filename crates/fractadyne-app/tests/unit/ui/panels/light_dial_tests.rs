use super::{dial_angle_at, dial_point, light_dial};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

const RESET: f32 = 2.2;

/// One frame of a 200x100 window holding one light dial over `angle`: the dial's rect.
fn frame(ctx: &egui::Context, time: f64, events: Vec<egui::Event>, angle: &mut f32) -> egui::Rect {
    let mut rect = egui::Rect::NOTHING;
    let input = egui::RawInput {
        events,
        time: Some(time),
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(200.0, 100.0))),
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            rect = light_dial(ui, angle, RESET).rect;
        });
    });
    rect
}

fn button(at: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() }
}

#[test]
fn the_dial_turns_counterclockwise_from_the_right_with_up_a_quarter_turn() {
    // A screen point's y grows DOWN: the point above the middle is a quarter turn, as the relief
    // light at pi/2 lights the picture's top edges.
    let c = egui::pos2(50.0, 50.0);
    let near = |a: Option<f32>, want: f32| a.is_some_and(|a| (a - want).abs() < 1.0e-5);
    assert!(near(dial_angle_at(c, egui::pos2(60.0, 50.0)), 0.0));
    assert!(near(dial_angle_at(c, egui::pos2(50.0, 40.0)), FRAC_PI_2));
    assert!(near(dial_angle_at(c, egui::pos2(40.0, 50.0)), PI));
    assert!(near(dial_angle_at(c, egui::pos2(50.0, 60.0)), 3.0 * FRAC_PI_2));
    assert_eq!(dial_angle_at(c, c), None, "no direction at the middle");
    for a in [0.0f32, 0.4, 2.2, 3.5, 6.0] {
        let p = dial_point(c, 7.0, a);
        assert!(near(dial_angle_at(c, p), a), "angle {a}: the point and back again");
        let got = dial_angle_at(c, p).unwrap();
        assert!((0.0..=TAU).contains(&got));
    }
}

#[test]
fn a_press_and_a_drag_point_the_light_and_a_double_click_resets_it() {
    let ctx = egui::Context::default();
    let mut angle = 0.0f32;
    let rect = frame(&ctx, 0.0, vec![], &mut angle);
    let c = rect.center();
    let top = c - egui::vec2(0.0, rect.height() * 0.4);
    let bottom_left = c + egui::vec2(-rect.width() * 0.4, rect.height() * 0.4);

    // A press at the bottom left: the light comes from there.
    frame(&ctx, 0.1, vec![egui::Event::PointerMoved(bottom_left)], &mut angle);
    frame(&ctx, 0.2, vec![button(bottom_left, true)], &mut angle);
    assert!((angle - 1.25 * PI).abs() < 1.0e-4, "a press at the bottom left: {angle}");
    // Dragged to the top, then past the dial's edge to its left: the light follows the pointer.
    frame(&ctx, 0.3, vec![egui::Event::PointerMoved(top)], &mut angle);
    assert!((angle - FRAC_PI_2).abs() < 1.0e-4, "dragged to the top: {angle}");
    let outside = c - egui::vec2(60.0, 0.0);
    frame(&ctx, 0.4, vec![egui::Event::PointerMoved(outside)], &mut angle);
    assert!((angle - PI).abs() < 1.0e-4, "dragged out past the left edge: {angle}");
    frame(&ctx, 0.5, vec![button(outside, false)], &mut angle);
    // A pointer that only passes over it changes nothing.
    frame(&ctx, 2.0, vec![egui::Event::PointerMoved(top)], &mut angle);
    assert!((angle - PI).abs() < 1.0e-4, "a hover moved it: {angle}");

    // A double-click: the reset, though each press points the light too.
    let right = c + egui::vec2(rect.width() * 0.4, 0.0);
    frame(&ctx, 3.0, vec![egui::Event::PointerMoved(right)], &mut angle);
    for (k, pressed) in [true, false, true, false].into_iter().enumerate() {
        frame(&ctx, 3.05 + 0.05 * k as f64, vec![button(right, pressed)], &mut angle);
    }
    frame(&ctx, 3.3, vec![], &mut angle);
    assert_eq!(angle, RESET, "a double-click resets the light angle");
}
