use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::{thread, time::Duration};

pub const HISTORY_FILE_PATH: &str = ".clipboard_history.ron";

#[derive(Serialize, Deserialize)]
struct ClipboardHistory {
    entries: Vec<String>,
}

pub struct Clippy {
    // Arc<Mutex<T>> is used to share Vec<String> safely across threads.
    // Vec<String> keeps clipboard entries in order.
    pub history: Vec<String>,
}

impl Clippy {
    pub fn new() -> Self {
        // We load the old history when instanciating a new object
        Self {
            history: Self::load_history(),
        }
    }

    // Method to run the listening thread
    pub fn listen_for_clipboard_events(&self) {
        // Here we use clone() because the "move" directive when launching
        // the thread takes ownership of the "self", preventing us for calling &self.history
        let mut history = self.history.clone();
        thread::spawn(move || {
            let mut clipboard = Clipboard::new().expect("Failed to access clipboard");

            loop {
                if let Ok(content) = clipboard.get_text() {
                    if !history.contains(&content) && !content.trim().is_empty() {
                        history.insert(0, content.clone());
                        if history.len() > 100 {
                            history.pop();
                        }
                        Self::save_history(&history);
                    }
                }
                thread::sleep(Duration::from_millis(800));
            }
        });
    }

    // Save history to file
    fn save_history(history: &Vec<String>) {
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

    fn load_history() -> Vec<String> {
        if let Ok(file) = fs::File::open(HISTORY_FILE_PATH) {
            let reader = BufReader::new(file);
            if let Ok(history_data) = ron::de::from_reader::<_, ClipboardHistory>(reader) {
                return history_data.entries;
            }
        }
        Vec::new() // Return empty list if file doesn't exist or is invalid
    }

    pub fn clear_history(&mut self) {
        self.history.clear(); // Clear history in memory
        let _ = fs::remove_file(HISTORY_FILE_PATH); // Delete history file

        // We could also clear the current state of the keyboard
        // let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
        // let _ = clipboard.clear();
    }

    pub fn handle_client(&self, mut stream: TcpStream) {
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
    let clippy = Clippy::new();
    // Start the clipboard monitoring in another thread if needed
    clippy.listen_for_clipboard_events();

    // Start a TCP listener on a local port
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind");
    println!("Daemon listening on port 7878 ...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // In a real app, you'd probably spawn a thread here
                clippy.handle_client(stream);
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}
