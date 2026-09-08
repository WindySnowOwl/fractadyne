//! File ▸ Settings ▸ Reference cache…: the on-disk orbit cache, shown the way a browser shows its
//! cache — where it is, how full it is, the limit, and a way to empty it.
//!
//! ⭐**Why it is visible at all** (author's requirement, 2026-09-08). The cache writes megabytes
//! into a directory the user never chose, and grows to a gigabyte by default. Anything that does
//! that has to say where, how much, let the limit be changed, and let it be cleared — the four
//! things every browser's cache page has, for the same reason.
//!
//! **Clear** is destructive in the §8.2 sense only by weight: it costs TIME (every orbit can be
//! rebuilt by visiting the location again), never data. So it gets a red fill and an inline
//! confirmation, but not the reversed-row treatment of *Reset application state*, which deletes
//! things that cannot come back.

use crate::FractadyneApp;
use eframe::egui;

impl FractadyneApp {
    pub(crate) fn draw_orbit_cache_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialogs.orbit_cache_open {
            return;
        }
        use crate::refcache_persist as cache;
        let mut open = self.dialogs.orbit_cache_open;
        let mut close = false;
        let mut open_dir = false;
        let mut clear_now = false;
        let usage = cache::usage();
        let limit = self.orbit_cache_mb as u64 * 1024 * 1024;
        let dir = cache::dir();
        let enabled = cache::enabled();
        let mut confirm = self.dialogs.orbit_cache_confirm;
        let mut limit_mb = self.orbit_cache_mb;

        egui::Window::new("Reference cache")
            .open(&mut open)
            .resizable(false)
            .default_width(480.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(
                        "At extreme depth the slow part of a view is its reference orbit — up to \
                         an hour of arbitrary-precision arithmetic. Fractadyne keeps finished \
                         orbits here, so returning to a location you have visited, or zooming \
                         somewhere near it, takes seconds instead. One orbit is about 4 MB.",
                    )
                    .weak()
                    .small(),
                );
                if !enabled {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Off for this run (a task invocation, or --no-orbit-cache). Nothing is \
                             read or written until the next ordinary launch.",
                        )
                        .color(crate::theme::danger_color(ui.ctx()))
                        .small(),
                    );
                }
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Location").weak().small());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(
                            dir.as_ref()
                                .map(|d| d.display().to_string())
                                .unwrap_or_else(|| "(no config directory available)".into()),
                        )
                        .monospace()
                        .small(),
                    );
                    if dir.as_ref().is_some_and(|d| d.exists()) && ui.small_button("Open folder").clicked() {
                        open_dir = true;
                    }
                });
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Usage").weak().small());
                let frac = if limit > 0 { (usage.bytes as f64 / limit as f64).min(1.0) as f32 } else { 0.0 };
                ui.add(
                    egui::ProgressBar::new(frac)
                        .text(format!(
                            "{} of {} — {} orbit{}",
                            cache::fmt_bytes(usage.bytes),
                            cache::fmt_bytes(limit),
                            usage.entries,
                            if usage.entries == 1 { "" } else { "s" }
                        ))
                        .desired_width(ui.available_width()),
                );
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Limit").weak().small());
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut limit_mb)
                            .range(64..=262_144)
                            .speed(16.0)
                            .suffix(" MB"),
                    )
                    .on_hover_text(
                        "When the cache grows past this, the orbits that were CHEAPEST to build \
                         are removed first — a deep orbit that took an hour outlives any number of \
                         quick ones, however recently those were used.",
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "≈ {} orbits at the live cap",
                            (limit_mb as u64 * 1024 * 1024) / (crate::LIVE_REF_CAP as u64 * 16)
                        ))
                        .weak()
                        .small(),
                    );
                });
                ui.add_space(8.0);

                // What it holds, most valuable first — the order in which eviction would NOT take
                // them. A handful of rows says more than a count.
                let rows = cache::entries_summary();
                if !rows.is_empty() {
                    ui.label(egui::RichText::new("Kept (most valuable first)").weak().small());
                    egui::Grid::new("orbit-cache-rows").num_columns(4).spacing([14.0, 2.0]).show(ui, |ui| {
                        ui.label(egui::RichText::new("iterations").weak().small());
                        ui.label(egui::RichText::new("precision").weak().small());
                        ui.label(egui::RichText::new("size").weak().small());
                        ui.label(egui::RichText::new("last used").weak().small());
                        ui.end_row();
                        for (len, prec, bytes, when) in rows.iter().rev().take(6) {
                            ui.label(egui::RichText::new(format!("{len}")).monospace().small());
                            ui.label(egui::RichText::new(format!("{prec} bits")).monospace().small());
                            ui.label(egui::RichText::new(cache::fmt_bytes(*bytes)).monospace().small());
                            ui.label(egui::RichText::new(ago(*when)).small());
                            ui.end_row();
                        }
                        if rows.len() > 6 {
                            ui.label(egui::RichText::new(format!("… and {} more", rows.len() - 6)).weak().small());
                            ui.end_row();
                        }
                    });
                    ui.add_space(8.0);
                }

                ui.separator();
                if confirm {
                    ui.label(format!(
                        "Delete {} orbit{} ({})? Rebuilding them costs time, not data.",
                        usage.entries,
                        if usage.entries == 1 { "" } else { "s" },
                        cache::fmt_bytes(usage.bytes)
                    ));
                    // Cancel first, the red action second — the reversed order every destructive
                    // confirm in the app uses (`UI-DESIGN.md` §8.2).
                    crate::theme::action_row(ui, |ui| {
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("Delete").color(egui::Color32::WHITE))
                                    .fill(egui::Color32::from_rgb(0xB0, 0x3A, 0x30)),
                            )
                            .clicked()
                        {
                            clear_now = true;
                        }
                        if crate::theme::cancel_button(ui, "Cancel").clicked() {
                            confirm = false;
                        }
                    });
                } else {
                    // The Reset dialog's layout, for the same reason (§8.2): the red button goes
                    // RIGHTMOST, out of the slot the eye reaches first, and Close sits where the
                    // affirmative usually is. Right-to-left, so the red one is added first.
                    crate::theme::action_row(ui, |ui| {
                        let can_clear = usage.entries > 0;
                        if ui
                            .add_enabled(
                                can_clear,
                                egui::Button::new(egui::RichText::new("Clear cache…").color(egui::Color32::WHITE))
                                    .fill(egui::Color32::from_rgb(0xB0, 0x3A, 0x30)),
                            )
                            .on_hover_text("Deletes every stored orbit. Asks first.")
                            .clicked()
                        {
                            confirm = true;
                        }
                        if crate::theme::cancel_button(ui, "Close").clicked() {
                            close = true;
                        }
                    });
                }
            });

        self.dialogs.orbit_cache_open = open && !close;
        self.dialogs.orbit_cache_confirm = confirm && self.dialogs.orbit_cache_open;
        if limit_mb != self.orbit_cache_mb {
            self.orbit_cache_mb = limit_mb;
            cache::set_budget_bytes(limit_mb as u64 * 1024 * 1024);
        }
        if open_dir {
            if let Some(d) = dir {
                let url = format!("file:///{}", d.display().to_string().replace('\\', "/"));
                ctx.open_url(egui::OpenUrl::new_tab(url));
            }
        }
        if clear_now {
            self.dialogs.orbit_cache_confirm = false;
            match cache::clear() {
                Ok(n) => self.set_toast(format!("Reference cache cleared — {n} orbit{} removed.", if n == 1 { "" } else { "s" }), ctx),
                Err(e) => self.set_toast(format!("Could not clear the reference cache: {e}"), ctx),
            }
        }
    }
}

/// "just now" / "3 min ago" / "2 h ago" / "5 d ago", for the rows.
fn ago(t: std::time::SystemTime) -> String {
    let s = std::time::SystemTime::now().duration_since(t).map(|d| d.as_secs()).unwrap_or(0);
    if s < 60 {
        "just now".into()
    } else if s < 3600 {
        format!("{} min ago", s / 60)
    } else if s < 86_400 {
        format!("{} h ago", s / 3600)
    } else {
        format!("{} d ago", s / 86_400)
    }
}
