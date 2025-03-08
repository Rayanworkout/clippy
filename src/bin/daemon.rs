use clippy::clipboard::Clippy;
fn main() {
    // Run the daemon thread
    Clippy::new().listen_for_clipboard_events();
}
