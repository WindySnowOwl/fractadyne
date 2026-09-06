//! The colour picker popup: a saturation/value square, a hue strip, and numbers in a form a
//! person recognises.
//!
//! ⭐⭐**Why this is ours and not egui's.** egui's stock popup labels its two numeric modes `U8`
//! and `F` — the internal names for "gamma byte" and "linear float". They are precise and they are
//! meaningless to anyone who has not read egui's source. It also offers no HEX field, while the
//! gradient editor around it speaks hex everywhere, and it has no OK/Cancel, so there is no way to
//! back out of a colour you were only trying.
//!
//! ⚠**The 2D square and the hue strip are drawn here because egui's are private** —
//! `color_slider_1d` / `color_slider_2d` are not exported, and both public entry points
//! (`color_picker_color32`, `color_picker_hsva_2d`) draw the `U8`/`F` row before them. Taking the
//! whole widget was the only way to change the part in front of it.
//!
//! ⚠**Display-referred throughout**, like everything else in this app: the channel value IS the
//! byte the monitor shows, so 0–255 is a plain rescale of 0–1 and neither is gamma-converted.

use fractadyne_color::segment::{hsv_to_rgb, rgb_to_hsv};

/// The three ways the same colour is written, all shown at once.
///
/// ⭐⭐**No mode toggle.** The first version made these a picker — `Range 0-255` / `Range 0-1` /
/// `Hex` — which is one click away from whichever one you actually wanted, every time. They are
/// three renderings of ONE value and they cost three short rows, so all three are on screen and
/// all three are editable.
/// ⚠The labels stay in the user's vocabulary, not the implementation's: egui's stock picker calls
/// these `U8` and `F`, which are exactly right and mean nothing without its source.
pub(crate) const ROW_LABELS: [&str; 3] = ["0–255", "0–1", "Hex"];

/// What one frame of the popup decided.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum PickerOutcome {
    /// Still open; the colour may have changed.
    Open,
    /// Keep the colour as it now stands.
    Accept,
    /// Put back the colour the popup opened with.
    Cancel,
}

/// The saturation/value square for a fixed hue.
///
/// ⚠Drawn as a MESH with per-vertex colours rather than a grid of filled rects: a 12×12 mesh is
/// 288 triangles the GPU interpolates smoothly, where visually-equivalent rects would be ~5000
/// draw items and still show banding at the cell edges.
fn sv_square(ui: &mut egui::Ui, size: f32, h: f32, s: &mut f32, v: &mut f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click_and_drag());
    const N: usize = 12;
    let mut mesh = egui::Mesh::default();
    for yi in 0..=N {
        for xi in 0..=N {
            let (sx, vy) = (xi as f32 / N as f32, 1.0 - yi as f32 / N as f32);
            let c = hsv_to_rgb(h, sx, vy);
            mesh.colored_vertex(
                egui::pos2(rect.min.x + sx * rect.width(), rect.min.y + (1.0 - vy) * rect.height()),
                crate::stop_color32(c),
            );
        }
    }
    let idx = |x: usize, y: usize| (y * (N + 1) + x) as u32;
    for yi in 0..N {
        for xi in 0..N {
            mesh.add_triangle(idx(xi, yi), idx(xi + 1, yi), idx(xi + 1, yi + 1));
            mesh.add_triangle(idx(xi, yi), idx(xi + 1, yi + 1), idx(xi, yi + 1));
        }
    }
    let pr = ui.painter_at(rect);
    pr.add(egui::Shape::mesh(mesh));
    pr.rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0_f32, ui.visuals().weak_text_color()),
        egui::StrokeKind::Inside,
    );
    // The cursor ring, drawn in both black and white so it is visible on any colour beneath it.
    let c = egui::pos2(rect.min.x + *s * rect.width(), rect.min.y + (1.0 - *v) * rect.height());
    pr.circle_stroke(c, 6.0, egui::Stroke::new(2.0_f32, egui::Color32::BLACK));
    pr.circle_stroke(c, 6.0, egui::Stroke::new(1.0_f32, egui::Color32::WHITE));

    if let (true, Some(p)) = (resp.dragged() || resp.clicked(), resp.interact_pointer_pos()) {
        *s = ((p.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        *v = 1.0 - ((p.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
        return true;
    }
    false
}

/// The hue strip.
fn hue_strip(ui: &mut egui::Ui, width: f32, h: &mut f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 16.0), egui::Sense::click_and_drag());
    let pr = ui.painter_at(rect);
    let steps = rect.width().ceil().max(1.0) as usize;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let x = rect.min.x + t * rect.width();
        pr.line_segment(
            [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
            egui::Stroke::new(1.5_f32, crate::stop_color32(hsv_to_rgb(t, 1.0, 1.0))),
        );
    }
    pr.rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0_f32, ui.visuals().weak_text_color()),
        egui::StrokeKind::Inside,
    );
    let x = rect.min.x + *h * rect.width();
    pr.line_segment(
        [egui::pos2(x, rect.min.y - 2.0), egui::pos2(x, rect.max.y + 2.0)],
        egui::Stroke::new(2.0_f32, egui::Color32::WHITE),
    );
    pr.line_segment(
        [egui::pos2(x, rect.min.y - 2.0), egui::pos2(x, rect.max.y + 2.0)],
        egui::Stroke::new(1.0_f32, egui::Color32::BLACK),
    );
    if let (true, Some(p)) = (resp.dragged() || resp.clicked(), resp.interact_pointer_pos()) {
        *h = ((p.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        return true;
    }
    false
}

/// Draw the picker body. `rgb` is edited in place; the return says what the user asked for.
///
/// ⚠**Hue is carried in `Ui` memory, not derived from the colour every frame.** A grey has no
/// hue — `rgb_to_hsv` reports 0 for anything unsaturated — so recomputing it would snap the hue
/// cursor to red the moment the user dragged the square to the left edge, and they would lose the
/// hue they were working in.
pub(crate) fn picker_body(ui: &mut egui::Ui, id: egui::Id, rgb: &mut [f32; 3]) -> PickerOutcome {
    const W: f32 = 216.0;
    // ⚠**The popup is only as wide as its content.** Without this the frame inherits the parent's
    // available width and the separator and the right-aligned button row stretch to fill it,
    // leaving a band of empty panel several times wider than anything in it.
    ui.set_max_width(W);
    ui.spacing_mut().item_spacing.y = 4.0;

    let (h0, s0, v0) = rgb_to_hsv(*rgb);
    let mut h = ui.data_mut(|d| d.get_temp::<f32>(id.with("h")).unwrap_or(h0));
    if s0 > 1.0e-4 {
        h = h0; // a saturated colour has a real hue; trust it
    }
    let (mut s, mut v) = (s0, v0);

    // ⭐**A large patch of the colour itself.** The swatch that opens this popup is 34×18 and is
    // usually behind the popup once it is open; a colour is the thing being chosen, so it gets
    // room to be looked at.
    let (patch, _) = ui.allocate_exact_size(egui::vec2(W, 44.0), egui::Sense::hover());
    ui.painter().rect_filled(patch, 3.0, crate::stop_color32(*rgb));
    ui.painter().rect_stroke(
        patch,
        3.0,
        egui::Stroke::new(1.0_f32, ui.visuals().weak_text_color()),
        egui::StrokeKind::Inside,
    );

    // ── The same colour in all three notations, each editable ────────────────────────────────
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(ROW_LABELS[0]).weak().small());
        for (i, name) in ["R", "G", "B"].into_iter().enumerate() {
            let mut b = (rgb[i].clamp(0.0, 1.0) * 255.0 + 0.5) as i32;
            if ui
                .add(egui::DragValue::new(&mut b).range(0..=255).prefix(name).speed(1.0))
                .changed()
            {
                rgb[i] = b as f32 / 255.0;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(ROW_LABELS[1]).weak().small());
        for (i, name) in ["R", "G", "B"].into_iter().enumerate() {
            let mut f = rgb[i];
            if ui
                .add(
                    egui::DragValue::new(&mut f)
                        .range(0.0..=1.0)
                        .prefix(name)
                        .speed(0.005)
                        .fixed_decimals(3),
                )
                .changed()
            {
                rgb[i] = f;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(ROW_LABELS[2]).weak().small());
        let hid = id.with("hex");
        let mut text =
            ui.data_mut(|d| d.get_temp::<String>(hid).unwrap_or_else(|| crate::hex_of(*rgb)));
        let r = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(78.0)
                .font(egui::TextStyle::Monospace),
        );
        // ⚠Held only while focused, or the field would re-render the current colour's full hex
        // over what the user is halfway through typing.
        if r.has_focus() || r.changed() {
            ui.data_mut(|d| d.insert_temp(hid, text.clone()));
        } else {
            ui.data_mut(|d| d.remove_temp::<String>(hid));
        }
        if r.changed() {
            if let Some(c) = crate::parse_hex_rgb(&text) {
                *rgb = c;
            }
        }
    });

    if sv_square(ui, W, h, &mut s, &mut v) {
        *rgb = hsv_to_rgb(h, s, v);
    }
    if hue_strip(ui, W, &mut h) {
        *rgb = hsv_to_rgb(h, s, v);
    }
    ui.data_mut(|d| d.insert_temp(id.with("h"), h));

    ui.add_space(2.0);
    ui.separator();
    let mut out = PickerOutcome::Open;
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if crate::theme::cancel_button(ui, "Cancel")
            .on_hover_text("Put back the colour this picker opened with")
            .clicked()
        {
            out = PickerOutcome::Cancel;
        }
        if crate::theme::confirm_button(ui, "OK").on_hover_text("Keep this colour").clicked() {
            out = PickerOutcome::Accept;
        }
    });
    out
}

// ⚠**`#[cfg(test)]` was lost when the `#[path]` attribute was added**, so this file was
// compiling into the RELEASE binary — caught by "unused import" warnings that could only
// appear if a test-only module was being built for real.
#[cfg(test)]
#[path = "color_picker_tests.rs"]
mod color_picker_tests;
