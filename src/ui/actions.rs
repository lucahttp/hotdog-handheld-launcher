//! Actions for the handheld launcher

/// Tab selected action - dispatched when a tab is clicked
#[derive(Clone, Debug)]
pub struct TabSelected(pub String);

/// Navigation direction action
#[derive(Clone, Debug)]
pub struct NavigateTab(pub i32); // -1 for previous, 1 for next