use super::{decor_layer, decor_painter, egui};

/// Distinctive fills, so each shape can be picked out of the frame's flat shape list.
const FRACTAL: egui::Color32 = egui::Color32::from_rgb(1, 2, 3);
const DECOR: egui::Color32 = egui::Color32::from_rgb(4, 5, 6);
const DIALOG: egui::Color32 = egui::Color32::from_rgb(7, 8, 9);

/// Paint one frame containing all three, and return where each landed in paint order.
/// Earlier index = painted first = underneath.
fn paint_order() -> (usize, usize, usize) {
    let ctx = egui::Context::default();
    // ⚠A window FADES IN, and egui multiplies everything in it — including a marker painted
    // inside — by the fade alpha, so colour matching finds nothing while it is animating.
    ctx.style_mut(|s| s.animation_time = 0.0);
    let input = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    };
    let run = |ctx: &egui::Context| {
        ctx.clone().run(input(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.painter().rect_filled(ui.max_rect(), 0.0, FRACTAL);
            });
            egui::Window::new("dialog").show(ctx, |ui| {
                // Marked from INSIDE the window, so the shape is unambiguously in its layer —
                // the frame's own fill comes from the style and is not ours to recognise.
                let (_, r) = ui.allocate_space(egui::vec2(20.0, 20.0));
                ui.painter().rect_filled(r, 0.0, DIALOG);
            });
            // The decoration, drawn after the panel exactly as the app draws it.
            decor_painter(ctx).rect_filled(
                egui::Rect::from_min_size(egui::pos2(700.0, 500.0), egui::vec2(40.0, 40.0)),
                0.0,
                DECOR,
            );
        })
    };
    run(&ctx); // first frame: the window has no remembered size yet
    let out = run(&ctx);
    let find = |want: egui::Color32| {
        out.shapes
            .iter()
            .position(|cs| match &cs.shape {
                egui::Shape::Rect(r) => r.fill == want,
                _ => false,
            })
            .unwrap_or_else(|| panic!("no shape filled {want:?} in the frame"))
    };
    (find(FRACTAL), find(DECOR), find(DIALOG))
}

/// ⭐⭐**The reported bug, as an assertion about paint order.** Decoration drawn onto the
/// fractal — the brand mark, tour captions, callouts — must paint ABOVE the fractal and BELOW
/// every dialog. Both halves have failed in the field: the "Fd" mark over "Go to location" and
/// a tour caption over "Render tour" (`Order::Middle`, which is where `Window` lives).
///
/// ⚠This runs a real `egui` frame rather than asserting that some constant is `Background`,
/// because the interesting failure is not the order enum — it is where the shapes actually land
/// once panels, areas and `GraphicLayers::drain` have had their say. Restoring `Order::Middle`
/// turns it red, which is the bug that was reported.
#[test]
fn decoration_paints_above_the_fractal_and_below_dialogs() {
    let (fractal, decor, dialog) = paint_order();
    assert!(
        fractal < decor,
        "decoration painted at {decor}, before the fractal at {fractal} — it is invisible"
    );
    assert!(
        decor < dialog,
        "decoration painted at {decor}, after the dialog at {dialog} — it covers any dialog it \
         sits under"
    );
}

/// The mechanism the test above rests on, stated once: decoration shares the panels' layer, so
/// its position is fixed by when its shapes are added rather than by how layers are sequenced.
#[test]
fn decoration_shares_the_panel_layer() {
    assert_eq!(decor_layer(), egui::LayerId::background());
}
