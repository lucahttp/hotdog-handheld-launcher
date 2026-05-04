//! Process management - Game launching and lifecycle

mod launcher;
mod lifecycle;

pub use launcher::{launch_game, LaunchOptions};
// TODO: Add lifecycle management when needed
