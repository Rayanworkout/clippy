use arboard::Clipboard;
use eframe::egui::{self};
use std::{
    // Arc<T>: Thread-safe reference-counting pointer to share data across threads.
    // Mutex<T>: Ensures safe access to shared data between threads.
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

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
                    if !hist.contains(&content) {
                        hist.insert(0, content.clone()); // Insert new content at the top
                        if hist.len() > 20 {
                            hist.pop();
                        } // Keep only last 20 entries
                    }
                }
                thread::sleep(Duration::from_millis(1000));
            }
        });

        Self { history }
    }
}

impl eframe::App for ClippyApp {
    // Handles UI updates.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
        .show(ctx, |ui| {
            ui.add_space(5.0);
            egui::ScrollArea::vertical()
            .show(ui, |ui| {
                let history = self.history.lock().unwrap();
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
        viewport: egui::ViewportBuilder::default().with_inner_size([250.0, 340.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Clippy",
        options,
        Box::new(|_cc| Ok(Box::new(ClippyApp::new()))),
    )
}
