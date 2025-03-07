use arboard::Clipboard;

use std::{
    // Arc<T>: Thread-safe reference-counting pointer to share data across threads.
    // Mutex<T>: Ensures safe access to shared data between threads.
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

pub fn start_clipboard_listener(history: Arc<Mutex<Vec<String>>>) {
    let history_clone = history.clone();

    thread::spawn(move || {
        let mut clipboard = Clipboard::new().expect("Failed to access clipboard");

        loop {
            if let Ok(content) = clipboard.get_text() {
                let mut hist = history_clone.lock().unwrap();
                if !hist.contains(&content) && !content.trim().is_empty() {
                    hist.insert(0, content.clone());
                    if hist.len() > 20 {
                        hist.pop();
                    }
                    crate::ui::ClippyApp::save_history(&hist);
                }
            }
            thread::sleep(Duration::from_millis(800));
        }
    });
}
