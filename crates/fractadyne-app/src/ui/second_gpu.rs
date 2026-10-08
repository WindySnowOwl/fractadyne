//! Advanced ▸ Second graphics card: the live view's supersampling on another card
//! (`design/multi-gpu-live.md` L2). The window's card still draws everything; the other one renders
//! some of the jittered samples that sharpen a deep view after it stops moving.

use crate::gpu_worker::{second_card_choices, WorkerState};
use crate::FractadyneApp;

impl FractadyneApp {
    /// The setting's row and its status line. The cards come from a `--list-adapters` child process
    /// the first time the row is drawn (`farm_client::list_cards`), never from this process.
    pub(super) fn second_gpu_row(&mut self, ui: &mut egui::Ui) {
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
        let choices = self.worker_cards.as_deref().map(|c| second_card_choices(c, &self.gpu_name));
        let value = &mut self.render_cfg.worker_gpu;
        let selected = match value.trim() {
            "" => "Off".to_string(),
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
            "Another graphics card renders some of the extra samples that sharpen a deep view \
             after it stops moving, so the view reaches its final quality sooner. The card drawing \
             the window still draws everything. Two different card models draw a few pixels \
             slightly differently; the samples are averaged, so the picture lands between them. \
             If the second card fails, the view carries on with one.",
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
