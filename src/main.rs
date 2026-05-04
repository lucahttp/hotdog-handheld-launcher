//! Handheld Launcher - Windows Shell Replacement
//! 
//! A blazing-fast, battery-optimized game launcher that replaces explorer.exe
//! on Windows handheld devices with Xbox 360 Metro aesthetics.

mod app;
mod daemon;
mod shell;
mod input;
mod focus;
mod ui;
mod process;
mod database;
mod scanner;

use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    log::info!("Starting Handheld Launcher");
    
    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let is_shell_mode = args.iter().any(|arg| arg == "--shell");
    
    if is_shell_mode {
        log::info!("Running in shell replacement mode");
    }
    
    // Run the daemon (main loop)
    daemon::run(is_shell_mode)?;
    
    Ok(())
}