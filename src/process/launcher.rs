//! Game launcher - Spawns and manages game processes

use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

/// Game launch options
pub struct LaunchOptions {
    /// Path to the game executable
    pub exe_path: String,
    /// Working directory (defaults to exe directory)
    pub working_dir: Option<String>,
    /// Command line arguments
    pub args: Vec<String>,
}

/// Launched game handle
pub struct GameHandle {
    pub pid: u32,
}

/// Spawn a game as a detached process
pub fn launch_game(options: LaunchOptions) -> Result<GameHandle> {
    let mut cmd = Command::new(&options.exe_path);

    // Set up detached process on Windows using raw Windows API constants
    // CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS | CREATE_NO_WINDOW
    // 0x00000200 | 0x00000010 | 0x08000000 = 0x08000210
    #[cfg(windows)]
    unsafe {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let flags = CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS | CREATE_NO_WINDOW;
        cmd.creation_flags(flags);
    }

    // Set working directory
    if let Some(work_dir) = &options.working_dir {
        cmd.current_dir(work_dir);
    } else if let Some(parent) = Path::new(&options.exe_path).parent() {
        cmd.current_dir(parent);
    }

    // Add arguments
    for arg in &options.args {
        cmd.arg(arg);
    }

    // Don't inherit stdin/stdout - detach from launcher
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    // Spawn the process
    let child = cmd.spawn().context("Failed to spawn game process")?;

    let pid = child.id();

    log::info!("Game launched with PID: {}", pid);

    Ok(GameHandle { pid })
}

/// Check if a process with the given PID is still running
#[cfg(windows)]
pub fn is_process_running(pid: u32) -> bool {
    use windows_sys::Win32::System::Threading::{
        CloseHandle, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);

        if handle != 0 {
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}

#[cfg(not(windows))]
pub fn is_process_running(_pid: u32) -> bool {
    // Non-Windows implementation stub
    false
}
