use crate::clipboard::Clippy;
use arboard::Clipboard;
use eframe::egui::{self, FontId, TextStyle};

pub struct ClippyApp {
    clippy_instance: Clippy,
}

impl ClippyApp {
    pub fn new(clippy_instance: Clippy) -> Self {
        Self { clippy_instance }
    }
}

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
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Clear history
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    if ui
                        .button("🗑")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.clippy_instance.clear_history();
                        // Minimize after clearing the history
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
                ui.add_space(10.0);

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
                                let _ = clipboard.set_text(value.clone());
                                // Minimize after copying
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
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
