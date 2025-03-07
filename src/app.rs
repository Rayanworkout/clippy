use std::sync::Arc;

use crate::clipboard::Clippy;
use arboard::Clipboard;
use eframe::egui::{self, FontId, TextStyle};

pub struct ClippyApp {
    clippy_instance: Arc<Clippy>,
}

impl ClippyApp {
    pub fn new() -> Self {
        let clippy_instance = Arc::new(Clippy::new());
        let clippy_instance_clone = clippy_instance.clone();
        Self {
            clippy_instance: clippy_instance_clone,
        }
    }
}

impl eframe::App for ClippyApp {
    // Handles UI updates.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(16.0, egui::FontFamily::Proportional), // Increase button font size to 16
        );
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(5.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Clear history
                ui.add_space(2.0);
                ui.vertical_centered(|ui| {
                    if ui
                        .button("🗑")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        // We empty the clipboard otherwise the entry contained inside will be
                        // added to the list when clicking on "clear"
                        if let Ok(mut clipboard) = Clipboard::new() {
                            // Set the clipboard to empty string
                            let _ = clipboard.clear();
                        }
                        self.clippy_instance.clear_history();
                    }
                });
                ui.add_space(7.0);

                // Iterate through every value of the history
                if let Ok(history) = self.clippy_instance.history.lock() {
                    for value in history.iter() {
                        ui.vertical_centered_justified(|ui| {
                            // We create a short version of the value but
                            // we keep the original to be copied
                            // only the first 60 characters
                            const MAX_ENTRY_DISPLAY_LENGTH: usize = 60;
                            let short_value = if value.len() > MAX_ENTRY_DISPLAY_LENGTH {
                                format!("{}...", &value[..MAX_ENTRY_DISPLAY_LENGTH])
                            } else {
                                value.clone()
                            };

                            if ui
                                .button(short_value)
                                // We use the "Copy" cursor on hover
                                .on_hover_cursor(egui::CursorIcon::Copy)
                                .clicked()
                            {
                                let mut clipboard = Clipboard::new().unwrap();
                                clipboard.set_text(value.clone()).unwrap();
                            }
                        });
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);
                    }
                }
            });
        });

        // Ensure UI updates regularly
        ctx.request_repaint();
    }
}
