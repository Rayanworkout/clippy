use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::{fs::OpenOptions, io::Write};
use std::{
    // Arc<T>: Thread-safe reference-counting pointer to share data across threads.
    // Mutex<T>: Ensures safe access to shared data between threads.
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub const HISTORY_FILE_PATH: &str = ".clipboard_history.ron";

#[derive(Serialize, Deserialize)]
struct ClipboardHistory {
    entries: Vec<String>,
}

pub struct Clippy {
    // Arc<Mutex<T>> is used to share Vec<String> safely across threads.
    // Vec<String> keeps clipboard entries in order.
    pub history: Arc<Mutex<Vec<String>>>,
}

impl Clippy {
    pub fn new() -> Self {
        // We load the old history when instanciating a new object
        let history = Arc::new(Mutex::new(Self::load_history()));
        Self { history }
    }

    // Method to run the listening thread
    pub fn listen_for_clipboard_events(&self) {
        // Here we use clone() because the "move" directive when launching
        // the thread takes ownership of the "self", preventing us for calling &self.history
        let history = self.history.clone();
        thread::spawn(move || {
            let mut clipboard = Clipboard::new().expect("Failed to access clipboard");

            loop {
                if let Ok(content) = clipboard.get_text() {
                    if let Ok(mut hist) = history.lock() {
                        if !hist.contains(&content) && !content.trim().is_empty() {
                            hist.insert(0, content.clone());
                            if hist.len() > 100 {
                                hist.pop();
                            }
                            Self::save_history(&hist);
                        }
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
            if let Ok(mut file) = OpenOptions::new()
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

    pub fn clear_history(&self) {
        if let Ok(mut hist) = self.history.lock() {
            hist.clear(); // Clear history in memory
            let _ = fs::remove_file(HISTORY_FILE_PATH); // Delete history file

            // We could also clear the current state of the keyboard
            // let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
            // let _ = clipboard.clear();
        }
    }
}
