use arboard::Clipboard;
use eframe::egui::{self, FontId, TextStyle};
use std::{
    // Arc<T>: Thread-safe reference-counting pointer to share data across threads.
    // Mutex<T>: Ensures safe access to shared data between threads.
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// TODO Persist data + run clipboard daemon as standalone

struct ClippyApp {
    // Arc<Mutex<T>> is used to share Vec<String> safely across threads.
    // Vec<String> keeps clipboard entries in order.
    history: Arc<Mutex<Vec<String>>>,
}

// Clipboard Listener
impl ClippyApp {
    fn new() -> Self {
        let history = Arc::new(Mutex::new(Vec::new()));
        // Creates a clone to use inside the clipboard monitoring thread.
        let history_clone = history.clone();

        // Start background clipboard monitoring thread
        thread::spawn(move || {
            let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
            loop {
                if let Ok(content) = clipboard.get_text() {
                    let mut hist = history_clone.lock().unwrap();
                    if !hist.contains(&content) && !&content.is_empty() {
                        hist.insert(0, content.clone()); // Insert new content at the top
                        if hist.len() > 20 {
                            hist.pop();
                        } // Keep only last 20 entries
                    }
                }
                thread::sleep(Duration::from_millis(800));
            }
        });

        Self { history }
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
                        // We need to lock the mutex before clearing the history iterable
                        if let Ok(mut hist) = self.history.lock() {
                            hist.clear();
                        }
                    }
                });
                ui.add_space(7.0);

                // Iterate through every value of the history
                if let Ok(history) = self.history.lock() {
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

fn main() -> eframe::Result<()> {
    // We create an options object to mention the viewport and the initial size
    // + the default settings

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([250.0, 340.0])
            .with_max_inner_size([350.0, 450.0])
            .with_maximize_button(false)
            .with_min_inner_size([200.0, 300.0])
            .with_position([250.0, 340.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Clippy",
        options,
        Box::new(|_cc| Ok(Box::new(ClippyApp::new()))),
    )
}
