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
    /// Start the screen eyedropper and drop what it takes into THIS field.
    ///
    /// ⭐The stop row has a Pick button, but the picker is where you are when you decide a colour
    /// is wrong — sending the user back out to a different button to sample one is the kind of
    /// detour that makes a tool feel like several tools.
    Pick,
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

    // ── One table: R/G/B as column headers, one row per notation ─────────────────────────────
    //
    // ⭐**Every cell is `add_sized`, and that is the whole trick.** Laid out as three independent
    // `horizontal` rows the values wandered — a two-digit `115` and a five-character `0.450` are
    // different widths, so nothing under "G" lined up with anything else under "G". ⚠And
    // `allocate_ui_with_layout` does NOT fix it: it shrinks to its content, so asking for a 56 px
    // cell and putting a narrow label in it still gives a narrow cell. That was the first attempt
    // and the columns came out 23 px apart in the header and 72 px apart in the rows.
    // `add_sized` forces the width and centres the widget in it, which is what a column is.
    const LABEL_W: f32 = 44.0;
    const COL_W: f32 = 56.0;
    let head = |ui: &mut egui::Ui, t: &str| {
        ui.add_sized(
            [COL_W, 14.0],
            egui::Label::new(egui::RichText::new(t).weak().small()),
        );
    };
    let row_label = |ui: &mut egui::Ui, t: &str| {
        ui.add_sized(
            [LABEL_W, 20.0],
            egui::Label::new(egui::RichText::new(t).weak().small()),
        );
    };

    ui.horizontal(|ui| {
        ui.add_sized([LABEL_W, 14.0], egui::Label::new(""));
        for name in ["R", "G", "B"] {
            head(ui, name);
        }
    });
    ui.horizontal(|ui| {
        row_label(ui, ROW_LABELS[0]);
        for i in 0..3 {
            let mut b = (rgb[i].clamp(0.0, 1.0) * 255.0 + 0.5) as i32;
            if ui
                .add_sized([COL_W, 20.0], egui::DragValue::new(&mut b).range(0..=255).speed(1.0))
                .changed()
            {
                rgb[i] = b as f32 / 255.0;
            }
        }
    });
    ui.horizontal(|ui| {
        row_label(ui, ROW_LABELS[1]);
        for i in 0..3 {
            let mut f = rgb[i];
            if ui
                .add_sized(
                    [COL_W, 20.0],
                    egui::DragValue::new(&mut f).range(0.0..=1.0).speed(0.005).fixed_decimals(3),
                )
                .changed()
            {
                rgb[i] = f;
            }
        }
    });
    // ⭐⭐**Hex reads as three bytes and edits as one string.** Split into columns it lines up
    // with the two rows above — `73 05 05` under R, G and B — which is the whole point of the
    // table. But a hex value is TYPED as `#730505`, one token, so the moment it is being edited it
    // collapses back to a single field. Display and entry want different shapes; each gets the one
    // it wants instead of both compromising.
    let editing_id = id.with("hexedit");
    let focus_id = id.with("hexfocus");
    let editing = ui.data_mut(|d| d.get_temp::<bool>(editing_id).unwrap_or(false));
    ui.horizontal(|ui| {
        row_label(ui, ROW_LABELS[2]);
        if editing {
            let hid = id.with("hex");
            let mut text =
                ui.data_mut(|d| d.get_temp::<String>(hid).unwrap_or_else(|| crate::hex_of(*rgb)));
            let r = ui.add_sized(
                [COL_W * 3.0, 20.0],
                egui::TextEdit::singleline(&mut text).font(egui::TextStyle::Monospace),
            );
            // ⚠Focus is requested the frame AFTER the field appears — on the frame the user clicks
            // the display, the field does not exist yet, so there is nothing to focus.
            if ui.data_mut(|d| d.get_temp::<bool>(focus_id).unwrap_or(false)) {
                r.request_focus();
                ui.data_mut(|d| d.insert_temp(focus_id, false));
            }
            if r.changed() {
                if let Some(c) = crate::parse_hex_rgb(&text) {
                    *rgb = c;
                }
            }
            if r.has_focus() || r.changed() {
                ui.data_mut(|d| d.insert_temp(hid, text.clone()));
            } else {
                // Focus gone: drop the draft and go back to the aligned display.
                ui.data_mut(|d| {
                    d.remove_temp::<String>(hid);
                    d.insert_temp(editing_id, false);
                });
            }
        } else {
            let mut clicked = false;
            for b in crate::rgb_bytes(*rgb) {
                clicked |= ui
                    .add_sized(
                        [COL_W, 20.0],
                        egui::Label::new(egui::RichText::new(format!("{b:02x}")).monospace())
                            .sense(egui::Sense::click()),
                    )
                    .on_hover_text("Click to type a hex value")
                    .clicked();
            }
            if clicked {
                ui.data_mut(|d| {
                    d.insert_temp(editing_id, true);
                    d.insert_temp(focus_id, true);
                });
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
    crate::theme::action_row(ui, |ui| {
        if crate::theme::cancel_button(ui, "Cancel")
            .on_hover_text("Put back the colour this picker opened with")
            .clicked()
        {
            out = PickerOutcome::Cancel;
        }
        if crate::theme::confirm_button(ui, "OK").on_hover_text("Keep this colour").clicked() {
            out = PickerOutcome::Accept;
        }
        // ⚠On the far LEFT of the row: it is not a commit, and sitting beside OK it would be read
        // as one. The picker stays open while sampling, so the colour arrives here to be judged.
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    crate::eyedropper::supported(),
                    egui::Button::new(format!("{} Pick", crate::icons::PICK)),
                )
                .on_hover_text("Take a colour from anywhere on screen")
                .clicked()
            {
                out = PickerOutcome::Pick;
            }
        });
    });
    out
}

// ⚠**`#[cfg(test)]` was lost when the `#[path]` attribute was added**, so this file was
// compiling into the RELEASE binary — caught by "unused import" warnings that could only
// appear if a test-only module was being built for real.
#[cfg(test)]
#[path = "color_picker_tests.rs"]
mod color_picker_tests;
