use arboard::Clipboard;
use eframe::egui::{self, FontId, TextStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::{fs::OpenOptions, io::Write};
use std::{
    // Arc<T>: Thread-safe reference-counting pointer to share data across threads.
    // Mutex<T>: Ensures safe access to shared data between threads.
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub const HISTORY_FILE_PATH: &str = "clipboard_history.ron";

#[derive(Serialize, Deserialize)]
struct ClipboardHistory {
    entries: Vec<String>,
}

pub struct ClippyApp {
    // Arc<Mutex<T>> is used to share Vec<String> safely across threads.
    // Vec<String> keeps clipboard entries in order.
    pub history: Arc<Mutex<Vec<String>>>,
}

impl ClippyApp {
    pub fn new() -> Self {
        let history = Arc::new(Mutex::new(Self::load_history())); // 🔹 Load history on startup
        let history_clone = history.clone();

        // Start clipboard monitoring thread
        thread::spawn(move || {
            let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
            loop {
                if let Ok(content) = clipboard.get_text() {
                    let mut hist = history_clone.lock().unwrap();
                    if !hist.contains(&content) && !content.is_empty() {
                        hist.insert(0, content.clone()); // Insert at the top
                        if hist.len() > 20 {
                            hist.pop();
                        }
                        Self::save_history(&hist);
                    }
                }
                thread::sleep(Duration::from_millis(800));
            }
        });

        Self { history }
    }

    // Save history to file
    pub fn save_history(history: &Vec<String>) {
        let history_data = ClipboardHistory {
            entries: history.clone(),
        };

        if let Ok(serialized) = ron::ser::to_string(&history_data) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(HISTORY_FILE_PATH)
            {
                let _ = file.write_all(serialized.as_bytes());
            }
        }
    }

    fn load_history() -> Vec<String> {
        if let Ok(file) = fs::File::open(HISTORY_FILE_PATH) {
            let reader = BufReader::new(file);
            if let Ok(history_data) = ron::de::from_reader::<_, ClipboardHistory>(reader) {
                return history_data.entries;
            }
        }
        Vec::new() // Return empty list if file doesn't exist or is invalid
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
