use arboard::Clipboard;
use std::{collections::HashSet, thread, time::Duration};

fn main() {
    let mut clipboard = Clipboard::new().expect("Failed to access clipboard");
    let mut history = HashSet::new();

    loop {
        if let Ok(content) = clipboard.get_text() {
            if !history.contains(&content) {
                history.insert(content.clone());
                println!("Copied: {}", content);
            }
        }
        thread::sleep(Duration::from_millis(800));
    }
}
