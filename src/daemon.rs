//! Main daemon loop - handles shell lifecycle, game monitoring, and UI state

use crate::input::InputBridge;
use anyhow::Result;

pub fn run(is_shell_mode: bool) -> Result<()> {
    log::info!(
        "Daemon starting in {} mode",
        if is_shell_mode { "shell" } else { "standalone" }
    );

    // Initialize input bridge (gamepad support)
    let input_bridge = InputBridge::new();

    if input_bridge.is_gamepad_connected() {
        log::info!("Gamepad connected");
    } else {
        log::info!("No gamepad detected - keyboard only mode");
    }

    // Start input polling (in daemon mode, runs in background)
    if is_shell_mode {
        let _input_handle = crate::input::start_input_loop(input_bridge);
        log::info!("Input polling started");
    }

    log::info!("Daemon initialized successfully");
    log::info!("Shell replacement mode: {}", is_shell_mode);

    // In shell mode, this would run indefinitely
    // For now, we just return Ok to allow compilation

    Ok(())
}
