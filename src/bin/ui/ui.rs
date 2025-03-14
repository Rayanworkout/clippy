use crate::clippy_app::ClippyApp;
use crate::config::ClippyConfig;

use arboard::Clipboard;
use eframe::egui;

impl eframe::App for ClippyApp {
    // Handles UI updates.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        ctx.set_style(style);

        if self.config.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    ui.menu_button("Preferences", |ui| {
                        if ui
                            .checkbox(&mut self.config.minimize_on_copy, "Minimize on copy")
                            .clicked()
                        {
                           self.toggle_config_field("minimize_on_copy");
                        }
                        if ui
                            .checkbox(&mut self.config.minimize_on_clear, "Minimize on clear")
                            .clicked()
                        {
                           self.toggle_config_field("minimize_on_clear");
                        }

                        if ui
                            .add(
                                egui::Slider::new(
                                    &mut self.config.max_entry_display_length,
                                    10..=500,
                                )
                                .text("max entry display length"),
                            )
                            .changed()
                        {
                            // Update config
                            let _ = confy::store(
                                "clippy",
                                None,
                                ClippyConfig {
                                    minimize_on_copy: self.config.minimize_on_copy,
                                    dark_mode: self.config.dark_mode,
                                    max_entry_display_length: self.config.max_entry_display_length,
                                    minimize_on_clear: self.config.minimize_on_clear,
                                },
                            );

                            tracing::info!(
                                "Max entry display length set to {} characters.",
                                self.config.max_entry_display_length
                            );
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        let logo = if self.config.dark_mode {
                            "🌞"
                        } else {
                            "🌙"
                        };
                        if ui.button(logo).clicked() {
                           self.config.dark_mode = self.toggle_config_field("dark_mode");
                        }
                    });

                    ui.add_space(10.);
                })
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // Main content
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
                        if self.config.minimize_on_clear {
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
                                let short_value = if value.len()
                                    > self.config.max_entry_display_length
                                {
                                    format!("{}...", &value[..self.config.max_entry_display_length])
                                } else {
                                    value.to_string()
                                };

                                if ui.button(short_value).clicked() {
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        match clipboard.set_text(value) {
                                            Ok(()) => {}
                                            Err(e) => {
                                                tracing::error!(
                                                    "Could not set clipboard value on click: {e}"
                                                );
                                            }
                                        }
                                    }

                                    if self.config.minimize_on_copy {
                                        // Minimize after copying
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(
                                            true,
                                        ));
                                    }
                                }
                            });
                            ui.add_space(10.0);
                        }
                    }
                }
            });
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.);
                ui.add(egui::Hyperlink::from_label_and_url(
                    "Made with egui",
                    "https://github.com/emilk/egui",
                ));
                ui.add_space(10.);
                ui.add(egui::Hyperlink::from_label_and_url(
                    "Source Code",
                    "https://github.com/Rayanworkout/clippy",
                ))
            });
            ui.add_space(10.);
        });

        // Ensure UI updates regularly
        ctx.request_repaint();
    }
}
