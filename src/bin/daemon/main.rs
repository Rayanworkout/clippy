mod clipboard_daemon;

use std::sync::Arc;

use anyhow::Result;
use clipboard_daemon::Clippy;

const UI_SENDING_PORT: u32 = 7878;
const UI_LISTENING_PORT: u32 = 7879;

// Find a way to easily launch it (.exe, .deb)
// Monitor RAM usage
// Update README

fn main() -> Result<()> {

    // Init logging
    tracing_subscriber::fmt::init();

    let clippy = Arc::new(Clippy::new()?);

    // Spawn the UI listener thread. This works because listen_for_ui expects an Arc<Self>.
    tracing::info!("Clippy listening for UI requests on 127.0.0.1:{UI_LISTENING_PORT} ...");
    Arc::clone(&clippy).listen_for_ui();

    // Main thread
    tracing::info!("Clippy listening for clipboard changes and ready to send to UI on 127.0.0.1:{UI_SENDING_PORT} ...");
    clippy.monitor_clipboard_events()?;

    Ok(())
}
