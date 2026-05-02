//! Shell replacement integration - Windows registry and explorer management

use anyhow::Result;
use winreg::enums::*;
use winreg::RegKey;

/// Register this executable as the Windows shell (per-user, no admin required)
pub fn install_shell() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let winlogon = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
        KEY_WRITE,
    )?;
    
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    
    winlogon.set_value("Shell", &format!("{} --shell", exe_path_str))?;
    
    log::info!("Shell registered at: {}", exe_path_str);
    Ok(())
}

/// Remove this executable from shell registration, restore explorer.exe
pub fn uninstall_shell() -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let winlogon = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
        KEY_WRITE,
    )?;
    
    winlogon.set_value("Shell", &"explorer.exe")?;
    
    log::info!("Shell unregistered, explorer.exe restored");
    Ok(())
}

/// Check if we are currently running as the shell
pub fn is_shell() -> bool {
    std::env::args().any(|arg| arg == "--shell")
}