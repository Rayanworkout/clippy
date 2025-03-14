use crate::clippy_app::ClippyApp;

use arboard::Clipboard;
use eframe::egui::{self, FontId, TextStyle};

impl eframe::App for ClippyApp {
    // Handles UI updates.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(18.0, egui::FontFamily::Proportional),
        );
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Preferences", |ui| {
                    ui.checkbox(&mut self.minimize_on_copy, "Minimize on copy");
                    ui.checkbox(&mut self.minimize_on_clear, "Minimize on clear");
                    ui.add(
                        egui::Slider::new(&mut self.max_entry_display_length, 10..=500)
                            .text("max entry display length"),
                    );
                });
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    // Search input
                    ui.text_edit_singleline(&mut self.search_query);

                    ui.add_space(10.0);
                    // Clear history
                    if ui
                        .button("🗑")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        let _ = self.clear_history();
                        // Optionally minimize after clearing the history
                        if self.minimize_on_clear {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    }
                });
                ui.add_space(10.0);

                // Iterate through every value of the history
                if let Ok(history) = self.history_cache.lock() {
                    for value in history.iter() {
                        // Filtering based on search query
                        if self.search_query.trim().is_empty()
                            || value.trim().contains(&self.search_query)
                        {
                            ui.vertical_centered_justified(|ui| {
                                // We create a short version of the value but
                                // we keep the original to be copied
                                // only the first X characters
                                let short_value = if value.len() > self.max_entry_display_length {
                                    format!("{}...", &value[..self.max_entry_display_length])
                                } else {
                                    value.to_string()
                                };

                                if ui.button(short_value).clicked() {
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        match clipboard.set_text(value) {
                                            Ok(()) => {}
                                            Err(e) => {
                                                eprintln!(
                                                    "Could not set clipboard value on click: {e}"
                                                )
                                            }
                                        }
                                    }

                                    if self.minimize_on_copy {
                                        // Minimize after copying
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(
                                            true,
                                        ));
                                    }
                                }
                            });
                            ui.add_space(5.0);
                            ui.separator();
                            ui.add_space(5.0);
                        }
                    }
                }
            });
        });

        // Ensure UI updates regularly
        ctx.request_repaint();
    }
}
