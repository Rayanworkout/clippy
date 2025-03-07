mod clipboard;
mod ui;

use eframe::egui;
use std::sync::{Arc, Mutex};
use ui::ClippyApp;

// TODO Persist data + run clipboard daemon as standalone
fn main() -> eframe::Result<()> {
    let history = Arc::new(Mutex::new(Vec::new()));

    // Start clipboard listener
    clipboard::start_clipboard_listener(history.clone());

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
        Box::new(|_cc| Ok(Box::new(ClippyApp::new(history)))),
    )
}
