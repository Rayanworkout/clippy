use anyhow::{anyhow, Context, Result};
use ron::de::from_str;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use arboard::Clipboard;
use eframe::egui::{self, FontId, TextStyle};

#[derive(Clone)]
pub struct ClippyApp {
    pub history_cache: Arc<Mutex<Vec<String>>>,
}

const DAEMON_LISTENING_PORT: u32 = 7878;
const DAEMON_SENDING_PORT: u32 = 7879;

impl ClippyApp {
    fn new() -> Self {
        let empty_cache = Vec::new();
        let clippy = ClippyApp {
            history_cache: Arc::new(Mutex::new(empty_cache)),
        };

        let _ = clippy.fill_initial_history();

        clippy
    }

    fn listen_for_history_updates(&self, mut stream: TcpStream) {
        let mut buffer = Vec::new();
        stream
            .read_to_end(&mut buffer)
            .expect("Failed to read from stream");
        let request = String::from_utf8_lossy(&buffer);
        if let Ok(mut history) = self.history_cache.lock() {
            *history = from_str(&request).expect("Failed to parse RON");
        }
    }

    fn fill_initial_history(&self) -> Result<()> {
        let request_result = (|| -> Result<String> {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{DAEMON_SENDING_PORT}"))
                .context(format!(
                "Initial history request could not bind to \"127.0.0.1:{DAEMON_SENDING_PORT}\"."
            ))?;

            let request = "GET_HISTORY\n";

            stream
                .write_all(request.as_bytes())
                .expect("Failed to write to stream");

            // Read the server's response into a string.
            let mut response = String::new();
            stream
                .read_to_string(&mut response)
                .context("Failed to read from stream")?;

            Ok(response)
        })();

        let mut history = self
            .history_cache
            .lock()
            .map_err(|e| anyhow!("Could not acquire history lock: {}", e))?;

        if let Ok(old_history) = request_result {
            *history =
                from_str(&old_history).context("Failed to parse initial history with RON")?;
            println!("Successfully loaded the initial history.");
        } else {
            eprintln!(
                "Could not fetch history from clipboard daemon.\nFalling back to an empty history.\n",
            );
            *history = from_str("")?;
        }

        Ok(())
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
                        // self.clear_history();
                        // Minimize after clearing the history
                        // ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
                ui.add_space(10.0);

                // Iterate through every value of the history
                if let Ok(history) = self.history_cache.lock() {
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

    // Create a ClippyApp instance normally (not wrapped in an Arc).
    let clippy_ui = ClippyApp::new();
    let clippy_for_thread = clippy_ui.clone();

    // Spawn a background thread that periodically updates the shared history.
    thread::spawn(move || {
        let listener = TcpListener::bind(format!("127.0.0.1:{DAEMON_LISTENING_PORT}"))
            .expect("Could not bind");
        println!("UI server listening on port {DAEMON_LISTENING_PORT} ...");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let _history = clippy_for_thread.listen_for_history_updates(stream);
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    println!("Running app ...");

    // Pass the ClippyApp instance directly to run_native.
    eframe::run_native(
        "Clippy",
        options,
        Box::new(move |_cc| Ok(Box::new(clippy_ui))),
    )
}
