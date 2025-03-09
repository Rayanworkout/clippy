use anyhow::{Context, Result};
use arboard::Clipboard;
use std::fs;
use std::io::{BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

// Refactor UI
// Order methods
// Clean code / Properly handle errors
// Monitor RAM usage
// Found a way to easily launch it (both binaries)
// Reorganize / modularize files
// Handle history file path depending on OS.
// Search through history
// Implement config file
// Update README
// Check what happens after the UI is closed and opened again

const HISTORY_FILE_PATH: &str = ".clipboard_history.ron";
const MAX_HISTORY_LENGTH: usize = 100;
const CLIPBOARD_REFRESH_RATE_MS: u64 = 800;
const TCP_PORT: u32 = 7878;

struct Clippy {
    clipboard: Mutex<Clipboard>,
    history: Mutex<Vec<String>>,
}

impl Clippy {
    fn new() -> Result<Self> {
        // Instanciate a clipboard object that will be used to access
        // or update the system clipboard.

        // We load the old history when instanciating
        // a new object to ensure history persistance
        Ok(Self {
            clipboard: Clipboard::new().context("Could not create a clipboard instance, the listener daemon can not run: {clipboard_error}")?.into(),
            history: Self::load_history()?.into(),
        })
    }

    /// Monitor clipboard changes and send a request to the UI on copy.
    fn monitor_clipboard_events(&self) -> Result<()> {
        println!("Clipboard daemon listening for clipboard changes ...");
        let mut consecutive_clipboard_failures = 0;

        loop {
            if let Ok(mut clipboard) = self.clipboard.lock() {
                match clipboard.get_text() {
                    Ok(content) => {
                        if consecutive_clipboard_failures > 0 {
                            consecutive_clipboard_failures = 0
                        }
                        if let Ok(mut history) = self.history.lock() {
                            if !history.contains(&content) && !content.trim().is_empty() {
                                // Insert new value at first index
                                history.insert(0, content);

                                // Keep only the wanted number of entries
                                if history.len() > MAX_HISTORY_LENGTH {
                                    history.pop();
                                }
                                // Save new history to file
                                self.save_history()?;
                                // Send the TCP request to the UI
                                self.send_history()?;
                            }
                        }
                    }
                    Err(clipboard_content_error) => {
                        eprintln!("Error getting the clipboard content: {clipboard_content_error}");
                        consecutive_clipboard_failures += 1;

                        if consecutive_clipboard_failures == 3 {
                            panic!("Error getting the clipboard content 3 times in a row, aborting daemon run.")
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(CLIPBOARD_REFRESH_RATE_MS));
        }
    }

    /// Save clipboard history to ron file.
    fn save_history(&self) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(HISTORY_FILE_PATH)
            .context(format!("Could not create or open {HISTORY_FILE_PATH}"))?;

        if let Ok(history) = self.history.lock() {
            let history_data = Vec::from(history.clone());
            let serialized_history = ron::ser::to_string(&history_data)
                .context("Could not serialize history when saving to file.")?;

            file.write_all(serialized_history.as_bytes())
                .context(format!(
                    "Could not write serialized history to {HISTORY_FILE_PATH}"
                ))?;
        }

        Ok(())
    }

    /// Loads the current history from the file.
    /// Static method.
    fn load_history() -> Result<Vec<String>> {
        let history: Vec<String> = fs::File::open(HISTORY_FILE_PATH)
            // We add some context to the rror in case we cannot open the file
            .context(format!("Could not open \"{HISTORY_FILE_PATH}\""))
            // And we chain an operation to deserialize the content if the opening works
            .and_then(|file| {
                let reader = BufReader::new(file);
                ron::de::from_reader(reader).context("Error deserializing clipboard history.")
            })
            // if any of these steps fail, we fall back to an empty Vec<String> and notify the user
            .unwrap_or_else(|load_error| {
                eprintln!(
                    "Could not load history: {load_error}\nFalling back to an empty history.",
                );
                Vec::new()
            });

        Ok(history)
    }

    fn clear_history(&self) -> Result<()> {
        if let Ok(mut history) = self.history.lock() {
            history.clear(); // Clear history in memory
            fs::remove_file(HISTORY_FILE_PATH).context("Could not delete the history file.")?;
        }

        // We could also clear the current state of the keyboard
        // self.clipboard.clear()?;
        Ok(())
    }

    /// Listen for directives coming from the UI
    /// for example clear_history() or the initial
    /// history request when starting.
    /// This way the UI can stop and start while always
    /// having an up to date history as long as
    /// the clipboard daemon is running.
    fn listen_for_ui(self: Arc<Self>) {
        println!("Clipboard daemon listening for UI TCP requests ...");
        let clippy = Arc::clone(&self);
        let _ = thread::spawn(move || -> Result<()> {
            let mut buffer = [0; 512];
            let listener = TcpListener::bind(format!("127.0.0.1:{TCP_PORT}")).context(format!(
                "UI listener could not bind to 127.0.0.1:{TCP_PORT}"
            ))?;

            for stream in listener.incoming() {
                let mut stream =
                    stream.context("Could not get stream from incoming UI connexion.")?;
                let size = stream
                    .read(&mut buffer)
                    .context("Could not read the incoming request from the UI.")?;

                let request = String::from_utf8_lossy(&buffer[..size]);

                if request.trim() == "GET_HISTORY" {
                    let history_str = if let Ok(history) = self.history.lock() {
                        // Format the history using its debug representation.
                        println!("Received request, sending history");
                        format!("{:?}", *history)
                    } else {
                        "[]".to_string()
                    };

                    stream
                        .write(history_str.as_bytes())
                        .context("Could not send the history to UI, stream.write() failed.")?;
                    // clippy.send_history(stream)?;
                } else if request.trim() == "RESET_HISTORY" {
                    clippy
                        .clear_history()
                        .context("Could not clear history after UI request.")?;

                    stream
                        .write(b"OK")
                        .context("Could not send the history to UI, stream.write() failed.")?;
                }
            }
            Ok(())
        }).join();
    }

    fn send_history(&self) -> Result<()> {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{TCP_PORT}"))
            .context(format!("Could not bind TcpStream to 127.0.0.1:{TCP_PORT}"))?;

        stream.write_all(format!("{:?}\n", self.history).as_bytes())?;

        stream
            .shutdown(Shutdown::Write)
            .context("Could not close the TCP connexion when sending history.")?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let clippy = Arc::new(Clippy::new()?);

    // Spawn the UI listener thread. This works because listen_for_ui expects an Arc<Self>.
    Arc::clone(&clippy).listen_for_ui();

    // Monitor clipboard events on the main thread.
    // clippy.monitor_clipboard_events()?;

    Ok(())
}
