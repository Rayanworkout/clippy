use std::io::{Read, Write};
use std::net::TcpStream;

use arboard::Clipboard;
// use clippy::clipboard::Clippy;
use eframe::egui::{self, FontId, TextStyle};

pub struct ClippyApp {}

impl ClippyApp {
    fn new() -> Self {
        Self {}
    }

    /// Fetch the history from the clipboard daemon
    /// through a TCP request
    fn get_history(&self) {
        let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Could not bind");

        let request = "GET_HISTORY\n";
        stream
            .write_all(request.as_bytes())
            .expect("Failed to write to stream");

        // Read the server's response into a string.
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("Failed to read from stream");

        println!("Response: {}", response);
    }
}

impl eframe::App for ClippyApp {
    // Handles UI updates.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let history = self.get_history();
        // let mut style = (*ctx.style()).clone();
        // style.text_styles.insert(
        //     TextStyle::Button,
        //     FontId::new(18.0, egui::FontFamily::Proportional),
        // );
        // ctx.set_style(style);

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     egui::ScrollArea::vertical().show(ui, |ui| {
        //         // Clear history
        //         ui.add_space(10.0);
        //         ui.vertical_centered(|ui| {
        //             if ui
        //                 .button("🗑")
        //                 .on_hover_cursor(egui::CursorIcon::PointingHand)
        //                 .clicked()
        //             {
        //                 // self.clippy_instance.clear_history();
        //                 // Minimize after clearing the history
        //                 ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        //             }
        //         });
        //         ui.add_space(10.0);

        //         // Iterate through every value of the history
        //         if let Ok(history) = self.get_history() {
        //             for value in history.iter() {
        //                 ui.vertical_centered_justified(|ui| {
        //                     // We create a short version of the value but
        //                     // we keep the original to be copied
        //                     // only the first 60 characters
        //                     const MAX_ENTRY_DISPLAY_LENGTH: usize = 60;
        //                     let short_value = if value.len() > MAX_ENTRY_DISPLAY_LENGTH {
        //                         format!("{}...", &value[..MAX_ENTRY_DISPLAY_LENGTH])
        //                     } else {
        //                         value.clone()
        //                     };

        //                     if ui
        //                         .button(short_value)
        //                         // We use the "Copy" cursor on hover
        //                         .on_hover_cursor(egui::CursorIcon::Copy)
        //                         .clicked()
        //                     {
        //                         let mut clipboard = Clipboard::new().unwrap();
        //                         let _ = clipboard.set_text(value.clone());
        //                         // Minimize after copying
        //                         ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        //                     }
        //                 });
        //                 ui.add_space(5.0);
        //                 ui.separator();
        //                 ui.add_space(5.0);
        //             }
        //         }
        //     });
        // });

        // // Ensure UI updates regularly
        // ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
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
        Box::new(move |_cc| Ok(Box::new(ClippyApp::new()))),
    )
}
