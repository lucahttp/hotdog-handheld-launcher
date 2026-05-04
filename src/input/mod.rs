//! Input handling - gamepad and keyboard input

pub mod input_bridge;

pub use input_bridge::{InputBridge, NavAction};

/// Start the input polling loop in a background task
pub fn start_input_loop(mut bridge: InputBridge) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Some(action) = bridge.poll_event() {
                log::debug!("Input action: {:?}", action);
                // TODO: Send to UI layer via event channel
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60fps
        }
    })
}
