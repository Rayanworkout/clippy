use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::{thread, time::Duration};

const HISTORY_FILE_PATH: &str = ".clipboard_history.ron";

#[derive(Serialize, Deserialize)]
struct ClipboardHistory {
    entries: Vec<String>,
}

pub struct Clippy {
    history: Mutex<Vec<String>>,
}

impl Clippy {
    pub fn new() -> Self {
        // We load the old history when instanciating a new object
        Self {
            history: Self::load_history(),
        }
    }

    // Method to run the listening thread
    pub fn listen_for_clipboard_events(self: std::sync::Arc<Self>) {
        let clippy_clone = std::sync::Arc::clone(&self);
        thread::spawn(move || {
            let mut clipboard = Clipboard::new().expect("Failed to access clipboard");

            loop {
                match clipboard.get_text() {
                    Ok(content) => {
                        if let Ok(mut history) = clippy_clone.history.lock() {
                            if !history.contains(&content) && !content.trim().is_empty() {
                                history.insert(0, content.clone());
                                if history.len() > 100 {
                                    history.pop();
                                }
                                // We drop the lock in order not to prevent a dead lock
                                // if both the loop and self.save_history() hold the lock()
                                // They would be waiting for each other indefinitely.
                                drop(history);
                                self.save_history();
                            }
                        }
                        thread::sleep(Duration::from_millis(800));
                    }
                    Err(e) => {
                        println!("Error getting the clipboard content: {}", e);
                    }
                }
            }
        });
    }

    // Save history to file
    fn save_history(&self) {
        if let Ok(history) = self.history.lock() {
            let history_data = ClipboardHistory {
                entries: history.clone(),
            };

            if let Ok(serialized) = ron::ser::to_string(&history_data) {
                if let Ok(mut file) = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(HISTORY_FILE_PATH)
                {
                    let _ = file.write_all(serialized.as_bytes());
                }
            }
        }
    }

    /// Loads the current history from the file.
    /// Static method.
    fn load_history() -> Mutex<Vec<String>> {
        if let Ok(file) = fs::File::open(HISTORY_FILE_PATH) {
            let reader = BufReader::new(file);
            if let Ok(history_data) = ron::de::from_reader::<_, ClipboardHistory>(reader) {
                return Mutex::new(history_data.entries);
            }
        }
        Mutex::new(Vec::new()) // Return empty list if file doesn't exist or is invalid
    }

    pub fn clear_history(&mut self) {
        if let Ok(mut history) = self.history.lock() {
            history.clear(); // Clear history in memory
            let _ = fs::remove_file(HISTORY_FILE_PATH); // Delete history file
        }

        // We could also clear the current state of the keyboard
        // let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
        // let _ = clipboard.clear();
    }

    pub fn listen_for_history_requests(&self, mut stream: TcpStream) {
        let mut buffer = [0; 512];
        if let Ok(size) = stream.read(&mut buffer) {
            let request = String::from_utf8_lossy(&buffer[..size]);
            println!("{}", request.trim());
            // if request.trim() == "GET_HISTORY" {
            //     if let Ok(history) = self.history.lock() {
            //         // Format the history using its debug representation.
            //         let history_str = format!("{:?}", *history);
            //         let _ = stream.write(history_str.as_bytes());
            //     }
            // } else if request.trim() == "CLEAR_HISTORY" {
            //     self.clear_history();
            //     let _ = stream.write(b"History cleared");
            // }
        }
    }
}

fn main() {
    // We wrap our Clippy instance in an Arc (Atomic Reference Counted pointer)
    // to allow safe shared ownership across multiple threads. Cloning the Arc
    // only increases the reference count, so the underlying Clippy instance is not duplicated.
    // This lets us call methods on the same instance in both the clipboard monitoring thread
    // and the TCP listener without moving ownership permanently.
    let clippy = std::sync::Arc::new(Clippy::new());
    // This method spawns a new thread that runs an infinite loop
    // listening for nw content copied
    clippy.clone().listen_for_clipboard_events();

    // Start a TCP listener on a local port
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind");
    println!("Daemon listening on port 7878 ...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                clippy.listen_for_history_requests(stream);
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
