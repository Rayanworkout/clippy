use crate::config::ClippyConfig;
use crate::DAEMON_LISTENING_PORT;
use crate::DAEMON_SENDING_PORT;
use anyhow::{anyhow, Context, Result};
use ron::de::from_str;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct ClippyApp {
    pub history_cache: Arc<Mutex<Vec<String>>>,
    pub search_query: String,
    pub config: ClippyConfig,
    pub style_needs_update: bool,
}

impl ClippyApp {
    pub fn new() -> Self {
        let empty_cache = Vec::new();

        let clippy = ClippyApp {
            history_cache: Arc::new(Mutex::new(empty_cache)),
            search_query: String::new(),
            config: confy::load("clippy", None).unwrap_or_default(),
            style_needs_update: true,
        };

        if let Err(initial_history_error) = clippy.fill_initial_history() {
            tracing::error!("An error occured when loading initial history in Clippy UI: {initial_history_error}.");
        }

        clippy
    }

    /// This method is used inside the UI (preferences)
    /// to toggle / edit config values.
    pub fn toggle_config_field(&mut self, field_name: &str) {
        let allowed_settings: Vec<&str> = vec![
            "minimize_on_copy",
            "minimize_on_clear",
            "dark_mode",
            "max_entry_display_length",
        ];

        if !allowed_settings.contains(&field_name) {
            tracing::error!("An invalid value was passed to ClippyApp.toggle_config_field()");
            return;
        }

        // Save the updated configuration
        let _ = confy::store("clippy", None, &self.config);

        // Log the change
        tracing::info!("{field_name} changed in config.");
    }

    pub fn listen_for_history_updates(self: Arc<Self>) {
        let clippy_app = Arc::clone(&self);
        thread::spawn(move || -> Result<()> {
            let listener = TcpListener::bind(format!("127.0.0.1:{DAEMON_LISTENING_PORT}"))
                .context(format!(
                    "Could not bind to 127.0.0.1:{DAEMON_LISTENING_PORT} when trying to listen for daemon history updates."
                ))?;

            tracing::info!("UI server listening on port {DAEMON_LISTENING_PORT} ...");

            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = Vec::new();

                        stream
                            .read_to_end(&mut buffer)
                            .context("Failed to read from stream")?;
                        let request = String::from_utf8_lossy(&buffer);

                        let mut history = clippy_app
                            .history_cache
                            .lock()
                            .map_err(|e| anyhow!("Could not acquire history lock: {}", e))?;

                        *history =
                            from_str(&request).context("Failed to parse history with RON")?;
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to accept connexion on {DAEMON_LISTENING_PORT}: {e} ..."
                        );
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
        } else {
            *history = from_str("")?;
            tracing::error!("Could not fetch history from clipboard daemon.\nFalling back to an empty history.\n");
        }
        tracing::info!("Successfully loaded initial history from clipboard daemon ...");
        Ok(())
    }

    pub fn clear_history(&mut self) -> Result<()> {
        let mut history = self
            .history_cache
            .lock()
            .map_err(|e| anyhow!("Could not acquire history lock: {}", e))?;

        history.clear();

        let request_result = (|| -> Result<String> {
            let mut stream = TcpStream::connect(format!("127.0.0.1:{DAEMON_SENDING_PORT}"))
                .context(format!(
                    "Clear history request could not bind to \"127.0.0.1:{DAEMON_SENDING_PORT}\"."
                ))?;

            // Send the RESET_HISTORY request to the server
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
            tracing::error!("Could not clear history: {e}\n");
        }

        Ok(())
    }
}
