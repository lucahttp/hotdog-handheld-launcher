//! Process management - Game launching and lifecycle

mod launcher;
mod lifecycle;

pub use launcher::{is_process_running, launch_game, GameHandle, LaunchOptions};
pub use lifecycle::{LifecycleManager, LifecycleState};
