//! Process lifecycle - Manages UI state during game execution

use super::launcher::{launch_game, GameHandle, LaunchOptions};
use std::sync::{Arc, Mutex};

/// Lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// Launcher is idle, showing UI
    Idle,
    /// A game is running
    GameRunning,
    /// Game exited, transitioning back to idle
    Transitioning,
}

/// Manages the lifecycle of the UI relative to game execution
pub struct LifecycleManager {
    state: Arc<Mutex<LifecycleState>>,
    current_pid: Arc<Mutex<Option<u32>>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(LifecycleState::Idle)),
            current_pid: Arc::new(Mutex::new(None)),
        }
    }

    /// Get current state
    pub fn state(&self) -> LifecycleState {
        *self.state.lock().unwrap()
    }

    /// Check if a game is currently running
    pub fn is_game_running(&self) -> bool {
        self.state() == LifecycleState::GameRunning
    }

    /// Launch a game and transition to GameRunning state
    pub fn launch_game(&self, options: LaunchOptions) -> Result<GameHandle> {
        let handle = launch_game(options)?;

        *self.state.lock().unwrap() = LifecycleState::GameRunning;
        *self.current_pid.lock().unwrap() = Some(handle.pid);

        log::info!("Lifecycle: transitioned to GameRunning, PID {}", handle.pid);

        Ok(handle)
    }

    /// Called when the game process exits
    pub fn on_game_exit(&self, pid: u32) {
        let mut current = self.current_pid.lock().unwrap();
        if *current == Some(pid) {
            *self.state.lock().unwrap() = LifecycleState::Idle;
            *current = None;
            log::info!("Lifecycle: transitioned to Idle");
        }
    }

    /// Poll for game exit (call periodically)
    /// Returns Some(pid) if the game just exited, None if still running or no game
    pub fn poll_game_exit(&self) -> Option<u32> {
        let pid = *self.current_pid.lock().unwrap();

        if let Some(pid) = pid {
            if !super::launcher::is_process_running(pid) {
                self.on_game_exit(pid);
                return Some(pid);
            }
        }

        None
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
