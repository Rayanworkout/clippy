use anyhow::{anyhow, Context, Result};
use ron::de::from_str;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use arboard::Clipboard;
use eframe::egui::{self, FontId, TextStyle};

#[derive(Clone)]
struct ClippyApp {
    history_cache: Arc<Mutex<Vec<String>>>,
    max_entry_display_length: usize,
    minimize_on_copy: bool,
    minimize_on_clear: bool,
    search_query: String,
}

const DAEMON_LISTENING_PORT: u32 = 7878;
const DAEMON_SENDING_PORT: u32 = 7879;

const MAX_ENTRY_DISPLAY_LENGTH: usize = 100;
const MINIMIZE_ON_COPY: bool = true;
const MINIMIZE_ON_CLEAR: bool = true;

impl ClippyApp {
    fn new() -> Self {
        let empty_cache = Vec::new();
        let clippy = ClippyApp {
            history_cache: Arc::new(Mutex::new(empty_cache)),
            max_entry_display_length: MAX_ENTRY_DISPLAY_LENGTH,
            minimize_on_copy: MINIMIZE_ON_COPY,
            minimize_on_clear: MINIMIZE_ON_CLEAR,
            search_query: String::new(),
        };

        let _ = clippy.fill_initial_history();

        clippy
    }

    fn listen_for_history_updates(self: Arc<Self>) {
        let clippy_app = Arc::clone(&self);
        thread::spawn(move || -> Result<()> {
            let listener = TcpListener::bind(format!("127.0.0.1:{DAEMON_LISTENING_PORT}"))
                .expect("Could not bind");
            println!("UI server listening on port {DAEMON_LISTENING_PORT} ...");

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = Vec::new();

                        stream
                            .read_to_end(&mut buffer)
                            .expect("Failed to read from stream");
                        let request = String::from_utf8_lossy(&buffer);

                        let mut history = clippy_app
                            .history_cache
                            .lock()
                            .map_err(|e| anyhow!("Could not acquire history lock: {}", e))?;

                        *history =
                            from_str(&request).context("Failed to parse history with RON")?;
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                    }
                }
            }
            Ok(())
        });
    }

    /// Fetch the initial history from the daemon with a
    /// TCP request. Uses an empty history if it fails.
    fn fill_initial_history(&self) -> Result<()> {
        let request_result = (|| -> Result<String> {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{DAEMON_SENDING_PORT}"))
                .context(format!(
                "Initial history request could not bind to \"127.0.0.1:{DAEMON_SENDING_PORT}\"."
            ))?;

            stream
                .write_all("GET_HISTORY\n".as_bytes())
                .context("Failed to write to stream when trying to get initial history.")?;

            // Read the server's response into a string.
            let mut response = String::new();
            stream
                .read_to_string(&mut response)
                .context("Failed to read from stream when trying to get initial history.")?;

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

    fn clear_history(&mut self) {
        let request_result = (|| -> Result<String> {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{DAEMON_SENDING_PORT}"))
                .context(format!(
                    "Clear history request could not bind to \"127.0.0.1:{DAEMON_SENDING_PORT}\"."
                ))?;

            stream
                .write("RESET_HISTORY\n".as_bytes())
                .expect("Failed to write to stream when trying to clear history.");

            // Read the server's response into a string.
            let mut response = String::new();
            stream
                .read_to_string(&mut response)
                .context("Failed to read from stream when trying to clear history.")?;
            Ok(response)
        })();

        if let Err(e) = request_result {
            eprintln!("Could not clear history: {e}\n");
        }
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
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Preferences", |ui| {
                    ui.checkbox(&mut self.minimize_on_copy, "Minimize on copy");
                    ui.checkbox(&mut self.minimize_on_clear, "Minimize on clear");
                    ui.add(
                        egui::Slider::new(&mut self.max_entry_display_length, 10..=500)
                            .text("max entry display length"),
                    );
                });
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    // Search input
                    ui.text_edit_singleline(&mut self.search_query);

                    ui.add_space(10.0);
                    // Clear history
                    if ui
                        .button("🗑")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.clear_history();
                        // Optionally minimize after clearing the history
                        if self.minimize_on_clear {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    }
                });
                ui.add_space(10.0);

                // Iterate through every value of the history
                if let Ok(history) = self.history_cache.lock() {
                    for value in history.iter() {
                        // Filtering based on search query
                        if self.search_query.trim().is_empty()
                            || value.trim().contains(&self.search_query)
                        {
                            ui.vertical_centered_justified(|ui| {
                                // We create a short version of the value but
                                // we keep the original to be copied
                                // only the first X characters
                                let short_value = if value.len() > self.max_entry_display_length {
                                    format!("{}...", &value[..self.max_entry_display_length])
                                } else {
                                    value.to_string()
                                };

                                if ui.button(short_value).clicked() {
                                    if let Ok(mut clipboard) = Clipboard::new() {
                                        match clipboard.set_text(value) {
                                            Ok(()) => {}
                                            Err(e) => {
                                                eprintln!(
                                                    "Could not set clipboard value on click: {e}"
                                                )
                                            }
                                        }
                                    }

                                    if self.minimize_on_copy {
                                        // Minimize after copying
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(
                                            true,
                                        ));
                                    }
                                }
                            });
                            ui.add_space(5.0);
                            ui.separator();
                            ui.add_space(5.0);
                        }
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
            .with_inner_size([350.0, 450.0])
            .with_max_inner_size([350.0, 450.0])
            .with_maximize_button(false)
            .with_min_inner_size([200.0, 300.0])
            .with_position([250.0, 340.0]),
        centered: true,
        ..Default::default()
    };

    // Create a ClippyApp instance normally (not wrapped in an Arc).
    let clippy_ui = Arc::new(ClippyApp::new());

    // Spawn a background thread that periodically updates the shared history.
    Arc::clone(&clippy_ui).listen_for_history_updates();

    println!("Running app ...");

    // Pass the ClippyApp instance directly to run_native.
    eframe::run_native(
        "Clippy",
        options,
        // We clone the inner value of Arc<ClippyApp> because Arc<ClippyApp> does not implement eframe::App
        Box::new(move |_cc| Ok(Box::new((*clippy_ui).clone()))),
    )
}
