use arboard::Clipboard;
use std::fs;
use std::io::{BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::{thread, time::Duration};

// TODO
// Add comments to describe behaviour
// Clean code / Properly handle errors
// Monitor RAM usage
// Use variables for TCP address:port
// Found a way to easily launch it (both binaries)
// Reorganize / modularize files
// Handle history file path depending on OS.
// Search through history
// Update README

const HISTORY_FILE_PATH: &str = ".clipboard_history.ron";
const MAX_HISTORY_LENGTH: usize = 100;

struct Clippy {
    clipboard: Clipboard,
    history: Vec<String>,
}

impl Clippy {
    fn new() -> Self {
        let clipboard = match Clipboard::new() {
            Ok(clip) => clip,
            Err(clipboard_error) => {
                panic!(
                    "Could not create a clipboard instance, the listener daemon can not run: {clipboard_error}"
                );
            }
        };

        // We load the old history when instanciating a new object
        Self {
            clipboard,
            history: Self::load_history(),
        }
    }

    /// Monitor clipboard changes and send a request to the UI on copy.
    pub fn listen_for_clipboard_events(&mut self) {
        loop {
            match self.clipboard.get_text() {
                Ok(content) => {
                    if !self.history.contains(&content) && !content.trim().is_empty() {
                        self.history.insert(0, content);
                        if self.history.len() > MAX_HISTORY_LENGTH {
                            self.history.pop();
                        }
                        // Save new history to file
                        self.save_history();
                        // Send the TCP request to the UI
                        self.send_updated_history();
                    }
                }

                Err(clipboard_content_error) => {
                    eprintln!("Error getting the clipboard content: {clipboard_content_error}");
                }
            }
            thread::sleep(Duration::from_millis(800));
        }
    }

    /// Save clipboard history to ron file.
    fn save_history(&self) {
        match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(HISTORY_FILE_PATH)
        {
            Ok(mut file) => {
                let history_data = Vec::from(self.history.clone());
                let serialized_history =
                    ron::ser::to_string(&history_data).unwrap_or_else(|serialization_error| {
                        panic!("Could not serialize history: {serialization_error}");
                    });

                file.write_all(serialized_history.as_bytes())
                    .unwrap_or_else(|write_error| {
                        panic!("Could not write serialized history to {HISTORY_FILE_PATH}: {write_error}");
                    });
            }
            Err(e) => {
                panic!("Could not create or open {HISTORY_FILE_PATH}: {e}");
            }
        }
    }

    /// Loads the current history from the file.
    /// Static method.
    fn load_history() -> Vec<String> {
        match fs::File::open(HISTORY_FILE_PATH) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match ron::de::from_reader::<_, Vec<String>>(reader) {
                    Ok(history_data) => history_data,
                    Err(deser_err) => {
                        eprintln!("Error deserializing history: {deser_err}");
                        Vec::new()
                    }
                }
            }
            Err(open_err) => {
                eprintln!(
                    "Error opening file {HISTORY_FILE_PATH}: {open_err}\nFalling back to an empty history.",
                );
                Vec::new()
            }
        }
    }

    fn _clear_history(&mut self) {
        self.history.clear(); // Clear history in memory
        let _ = fs::remove_file(HISTORY_FILE_PATH); // Delete history file

        // We could also clear the current state of the keyboard
        // let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
        // let _ = clipboard.clear();
    }

    fn send_updated_history(&self) {
        let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Could not bind");
        let history_str = format!("{:?}\n", self.history);
        stream
            .write_all(history_str.as_bytes())
            .expect("Could not send message");

        stream
            .shutdown(Shutdown::Write)
            .expect("Could not close the connexion");
    }
}

fn main() {
    // We wrap our Clippy instance in an Arc (Atomic Reference Counted pointer)
    // to allow safe shared ownership across multiple threads. Cloning the Arc
    // only increases the reference count, so the underlying Clippy instance is not duplicated.
    // This lets us call methods on the same instance in both the clipboard monitoring thread
    // and the TCP listener without moving ownership permanently.
    let mut clippy = Clippy::new();

    // This method spawns a new thread that runs an infinite loop
    // listening for new content copied
    clippy.listen_for_clipboard_events();
}
