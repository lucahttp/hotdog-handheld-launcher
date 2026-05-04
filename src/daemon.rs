//! Main daemon loop - handles shell lifecycle, game monitoring, and UI state

use crate::input::InputBridge;
use anyhow::Result;

pub fn run(is_shell_mode: bool) -> Result<()> {
    log::info!(
        "Daemon starting in {} mode",
        if is_shell_mode { "shell" } else { "standalone" }
    );

    // Initialize input bridge (gamepad support)
    let mut input_bridge = InputBridge::new();
    let input_rx = input_bridge.take_receiver();

    if input_bridge.is_gamepad_connected() {
        log::info!("Gamepad connected");
    } else {
        log::info!("No gamepad detected - keyboard only mode");
    }

    // Spawn a dedicated OS thread for gamepad polling (sync, no Tokio needed).
    // Events are forwarded to GPUI via the mpsc channel already in input_bridge.
    std::thread::spawn(move || {
        loop {
            input_bridge.poll_event();
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
        }
    });

    // Start the GPUI application loop — receives input via input_rx channel
    crate::app::init(input_rx)?;

    Ok(())
}
