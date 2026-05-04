//! Input bridge - gilrs gamepad events to navigation actions

use gilrs::{Button, EventType, Gilrs};
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
    PreviousTab,
    NextTab,
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
    gilrs: Arc<Mutex<Option<Gilrs>>>,
    event_tx: mpsc::UnboundedSender<NavAction>,
    event_rx: Option<mpsc::UnboundedReceiver<NavAction>>,
}

impl InputBridge {
    /// Create a new input bridge
    pub fn new() -> Self {
        let gilrs = Gilrs::new().ok();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            gilrs: Arc::new(Mutex::new(gilrs)),
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    /// Get a receiver for navigation actions
    pub fn take_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<NavAction>> {
        self.event_rx.take()
    }

    /// Check if a gamepad is connected
    pub fn is_gamepad_connected(&self) -> bool {
        let gilrs = self.gilrs.lock().unwrap();
        gilrs
            .as_ref()
            .map(|g: &Gilrs| g.gamepads().count() > 0)
            .unwrap_or(false)
    }

    /// Poll for events - call this in your event loop
    pub fn poll_event(&mut self) -> Option<NavAction> {
        let mut gilrs_guard = self.gilrs.lock().unwrap();
        let gilrs: &mut Gilrs = match gilrs_guard.as_mut() {
            Some(g) => g,
            None => return None,
        };

        while let Some(event) = gilrs.next_event() {
            if let Some(action) = Self::map_event(event) {
                let _ = self.event_tx.send(action);
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
            EventType::ButtonPressed(Button::LeftTrigger, _) => Some(NavAction::PreviousTab), // LB
            EventType::ButtonPressed(Button::RightTrigger, _) => Some(NavAction::NextTab),    // RB
            _ => None,
        }
    }
}

impl Default for InputBridge {
    fn default() -> Self {
        Self::new()
    }
}
