//! Input bridge - gilrs gamepad events to navigation actions

use gilrs::{Button, EventType, GilRs};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Navigation actions that result from input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Select,
    Back,
    Menu,
}

/// Direction for spatial navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Input bridge state
pub struct InputBridge {
    gilrs: Arc<Mutex<Option<GilRs>>>,
    event_tx: mpsc::UnboundedSender<NavAction>,
}

impl InputBridge {
    /// Create a new input bridge
    pub fn new() -> Self {
        let gilrs = GilRs::new().ok();
        let (event_tx, _event_rx) = mpsc::unbounded_channel();

        Self {
            gilrs: Arc::new(Mutex::new(gilrs)),
            event_tx,
        }
    }

    /// Get a receiver for navigation actions
    pub fn take_receiver(&mut self) -> mpsc::UnboundedReceiver<NavAction> {
        let (_tx, rx) = mpsc::unbounded_channel();
        rx
    }

    /// Check if a gamepad is connected
    pub fn is_gamepad_connected(&self) -> bool {
        let gilrs = self.gilrs.lock().unwrap();
        gilrs
            .as_ref()
            .map(|g| g.gamepads().count() > 0)
            .unwrap_or(false)
    }

    /// Poll for events - call this in your event loop
    pub fn poll_event(&mut self) -> Option<NavAction> {
        let mut gilrs_guard = self.gilrs.lock().unwrap();
        let gilrs = match gilrs_guard.as_mut() {
            Some(g) => g,
            None => return None,
        };

        while let Some(event) = gilrs.next_event() {
            if let Some(action) = Self::map_event(event) {
                return Some(action);
            }
        }

        None
    }

    /// Map a gilrs event to a navigation action
    fn map_event(event: gilrs::Event) -> Option<NavAction> {
        match event.event {
            EventType::ButtonPressed(Button::DPadUp, _) => Some(NavAction::NavigateUp),
            EventType::ButtonPressed(Button::DPadDown, _) => Some(NavAction::NavigateDown),
            EventType::ButtonPressed(Button::DPadLeft, _) => Some(NavAction::NavigateLeft),
            EventType::ButtonPressed(Button::DPadRight, _) => Some(NavAction::NavigateRight),
            EventType::ButtonPressed(Button::South, _) => Some(NavAction::Select), // Xbox A
            EventType::ButtonPressed(Button::East, _) => Some(NavAction::Back),    // Xbox B
            EventType::ButtonPressed(Button::Start, _) => Some(NavAction::Menu),
            _ => None,
        }
    }
}

impl Default for InputBridge {
    fn default() -> Self {
        Self::new()
    }
}
