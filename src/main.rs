mod ui;
mod clipboard;


use ui::ClippyApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Create an instance that will be shared between the main thread and the daemon thread
    let clippy_shared_instance = clipboard::Clippy::new();

    // Run the daemon thread
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

    // And the main thread
    eframe::run_native(
        "Clippy",
        options,
        Box::new(move |_cc| Ok(Box::new(ClippyApp::new(clippy_shared_instance)))),
    )
}
