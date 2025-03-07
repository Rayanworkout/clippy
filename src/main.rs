mod app;
mod clipboard;

use std::sync::Arc;

use app::ClippyApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let clippy_shared_instance = Arc::new(clipboard::Clippy::new());

    // Clone for the daemon thread
    let _clippy_daemon = clippy_shared_instance.clone();

    clippy_shared_instance.run();


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
        Box::new(move |_cc| Ok(Box::new(ClippyApp::new(clippy_shared_instance.clone())))),
    )
}
