//! Advanced ▸ Second graphics card: another device for the live view (`design/multi-gpu-live.md`
//! L2, L3). The window's card still draws everything; the other device renders frames of a moving
//! view and some of the jittered samples that sharpen a deep view after it stops moving. It can be
//! another card, or a second device on the window's own card, which adds no GPU but renders
//! without waiting for the screen's frames.

use crate::gpu_worker::{second_card_choices, WorkerState, SAME_CARD};
use crate::FractadyneApp;

/// The choice that opens a second device on the window's own card.
const SAME_CARD_LABEL: &str = "Same card";

impl FractadyneApp {
    /// The setting's row and its status line. The cards come from a `--list-adapters` child process
    /// the first time the row is drawn (`farm_client::list_cards`), never from this process.
    /// This machine's graphics cards (number, name), from a `--list-adapters` child process started
    /// on first use; `None` until it answers. Shared by this row and the Export dialog.
    pub(crate) fn card_list(&mut self) -> Option<&[(usize, String)]> {
        if self.worker_cards.is_none() && self.worker_cards_rx.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            self.worker_cards_rx = Some(rx);
            crate::render::spawn_named("fd-list-cards", move || {
                let _ = tx.send(crate::ui::farm_client::list_cards("second-gpu"));
            });
        }
        if let Some(rx) = &self.worker_cards_rx {
            if let Ok(cards) = rx.try_recv() {
                self.worker_cards = Some(cards);
                self.worker_cards_rx = None;
            }
        }
        self.worker_cards.as_deref()
    }

    pub(super) fn second_gpu_row(&mut self, ui: &mut egui::Ui) {
        let _ = self.card_list();
        let choices = self.worker_cards.as_deref().map(|c| second_card_choices(c, &self.gpu_name));
        let value = &mut self.render_cfg.worker_gpu;
        let selected = match value.trim() {
            "" => "Off".to_string(),
            SAME_CARD => SAME_CARD_LABEL.to_string(),
            v => choices
                .as_deref()
                .and_then(|c| c.iter().find(|(n, _)| n.to_string() == v))
                .map_or_else(|| format!("Card {v}"), |(_, name)| name.clone()), // the list shows the numbers
        };
        let row = crate::ui::labelled(ui, "Second graphics card", |ui| {
            // The label is wider than the panel's label column, so size to what is left of the row
            // (a fixed width ran past the panel's edge) and let a long card name end in an ellipsis.
            egui::ComboBox::from_id_salt("second_gpu")
                .width((ui.available_width() - 8.0).clamp(80.0, 180.0))
                .truncate()
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    ui.selectable_value(value, String::new(), "Off");
                    ui.selectable_value(value, SAME_CARD.to_string(), SAME_CARD_LABEL);
                    match choices.as_deref() {
                        None => {
                            ui.label(egui::RichText::new("Finding this machine's graphics cards…").weak());
                        }
                        Some([]) => {
                            ui.label(egui::RichText::new("This machine has no other graphics card.").weak());
                        }
                        Some(cs) => {
                            for (n, name) in cs {
                                ui.selectable_value(value, n.to_string(), format!("{n} · {name}"));
                            }
                        }
                    }
                })
                .response
        })
        .on_hover_text(
            "Another graphics device renders frames of a deep view while it moves, and some of the \
             extra samples that sharpen it after it stops, so the picture keeps up with a zoom and \
             reaches its final quality sooner. The card drawing the window still draws everything.\n\n\
             Another card adds its own speed. Two different card models draw a few pixels slightly \
             differently; if the second card fails, the view carries on with one.\n\n\
             \"Same card\" opens a second device on this card. It adds no speed but renders without waiting for the \
             screen's frames. It shares the card: a failure that resets the card ends the app, as a \
             failure of the window's own rendering does.",
        );
        if self.dialogs.uitest_advanced_open == Some(true) {
            row.scroll_to_me(Some(egui::Align::Center)); // the walk's screenshot of this row
        }
        // Always one line, so choosing a card never moves the rows below it.
        let state = &self.worker_state;
        let text = egui::RichText::new(state.text(self.worker_pinned)).small();
        let text = match state {
            WorkerState::Failed(_) | WorkerState::Lost(_) => text.color(ui.visuals().warn_fg_color),
            _ => text.weak(),
        };
        ui.add(egui::Label::new(text).truncate());
    }
}
