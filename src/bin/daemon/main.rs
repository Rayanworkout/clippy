mod clipboard_daemon;

use std::sync::Arc;

use anyhow::Result;
use clipboard_daemon::Clippy;

const UI_SENDING_PORT: u32 = 7878;
const UI_LISTENING_PORT: u32 = 7879;

fn main() -> Result<()> {
    let clippy = Arc::new(Clippy::new()?);

    // Spawn the UI listener thread. This works because listen_for_ui expects an Arc<Self>.
    println!("Clippy listening for UI requests on {UI_LISTENING_PORT} ...");
    Arc::clone(&clippy).listen_for_ui();

    // Main thread
    println!("Clippy listening for clipboard changes and ready to send to UI on port {UI_SENDING_PORT} ...");
    clippy.monitor_clipboard_events()?;

    Ok(())
}
