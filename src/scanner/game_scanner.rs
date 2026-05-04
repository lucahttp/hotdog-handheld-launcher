use std::fs;
use std::path::Path;
use serde_json::Value;

// steamlocate is cross-platform
use steamlocate::SteamDir;

// Usamos el repositorio de base de datos para leer los juegos agregados manualmente
use crate::database::game_repository::GameRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Steam,
    EpicGames,
    Xbox,
    Standalone,
}

#[derive(Debug, Clone)]
pub struct InstalledGame {
    pub name: String,
    pub platform: Platform,
    pub install_dir: String,
    pub icon_path: Option<String>,
    /// Platform-specific app ID (Steam app_id, Epic catalogItemId, etc.)
    pub app_id: Option<String>,
    /// Launch command: URL (steam://, epic://) or exe path
    pub launch_command: Option<String>,
}

// ── Playnite-style exclusion patterns ──────────────────────────────────

static EXCLUDED_EXE_NAMES: &[&str] = &[
    "unins000", "unins001", "uninstall", "setup", "setup_", "config", "config_",
    "dxsetup", "vcredist", "vc_redist", "dotnet", "dotnetfx",
    "unitycrashhandler", "unitycrashhandler32", "unitycrashhandler64",
    "ue4prereqsetup", "x64setup", "x86setup",
];

/// Registry entries that should never appear as games.
static EXCLUDED_REGISTRY_NAMES: &[&str] = &[
    // OS & drivers
    "microsoft ", "microsoft 365", "office", "onedrive", "windows ", "security ", "defender",
    "intel", "nvidia", "amd software", "chipset", "bluetooth", "realtek",
    "audio", "network", "driver", "update",
    // Anti-cheat / anti-virus
    "vanguard", "easyanticheat", "battleye",
    // Browsers
    "firefox", "chrome", "edge", "browser", "brave", "opera", "vivaldi", "torch",
    // Multimedia / design
    "adobe", "java", "python", "node.js", "docker", "git",
    "visual studio", "7-zip", "heidi", "freecad", "gigabyte", "gbt_",
    "bambu studio", "minimax agent", "newtek", "opencad", "speedhq",
    "zoo design", "steamworks",
    // Runtimes & SDKs
    "redistributable", "runtime", "sdk", "directx", "vc_redist",
    "vcredist", "dotnet", "framework", "visual c++",
    // Other known non-games
    "keyshot", "twain", "canon", "epson", "logitech", "obs studio",
    "discord", "telegram", "whatsapp", "slack", "teams", "zoom",
    "wps office", "libreoffice", "openoffice", "notepad", "powertoys",
    "everything", "greenshot", "sharex", "obsidian", "notion",
    "rider", "clion", "idea", "webstorm", "pycharm", "goland",
    "vmware", "virtualbox", "qemu", "wsl", "docker desktop",
    "hub", "desktop github",
];

// ── Executable finder ─────────────────────────────────────────────────

/// Scan a directory for the "best" game .exe (Playnite-style)
fn find_game_executable(install_dir: &str) -> Option<String> {
    let dir = Path::new(install_dir);
    if !dir.is_dir() {
        return None;
    }

    let mut candidates: Vec<(String, usize)> = Vec::new(); // (path, score)

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("exe") {
                continue;
            }

            let stem = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            if EXCLUDED_EXE_NAMES.iter().any(|pat| stem.contains(pat)) {
                continue;
            }

            // Score: prefer exe with same name as directory (most game engines do this)
            let dir_name = dir.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            let score: usize = if stem == dir_name {
                100 // Perfect match
            } else if stem.contains(&dir_name) || dir_name.contains(&stem) {
                50 // Partial name match
            } else {
                10 // Some random exe
            };

            // Prefer smaller file size (the main game exe, not an SDK tool)
            let size_bonus = if let Ok(meta) = fs::metadata(&path) {
                let size = meta.len();
                if size > 1_000_000 && size < 500_000_000 { 20 } else { 0 }
            } else { 0 };
            let score = score + size_bonus;

            candidates.push((path.to_string_lossy().to_string(), score));
        }
    }

    // Return highest-scored exe
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.first().map(|(p, _)| p.clone())
}

pub struct GameScanner;

impl GameScanner {
    /// Ejecuta todos los escáneres y devuelve una lista unificada
    pub fn scan_all(repo: Option<&GameRepository>) -> Vec<InstalledGame> {
        let mut all_games = Vec::new();

        all_games.extend(Self::scan_steam());

        #[cfg(target_os = "windows")]
        {
            all_games.extend(Self::scan_epic());
            all_games.extend(Self::scan_xbox());
            all_games.extend(Self::scan_windows_registry());
        }

        if let Some(repo) = repo {
            all_games.extend(Self::scan_manual_database(repo));
        }

        all_games
    }

    // --- 1. STEAM (Multiplataforma) ---
    fn scan_steam() -> Vec<InstalledGame> {
        let mut games = Vec::new();
        if let Ok(steamdir) = SteamDir::locate() {
            if let Ok(libraries) = steamdir.libraries() {
                for library in libraries.flatten() {
                    for app in library.apps().flatten() {
                        let name = app.name.clone().unwrap_or_else(|| String::new());
                        // Skip known non-game Steam entries
                        let lower = name.to_lowercase();
                        if lower.contains("steamworks") || lower.contains("redistributable")
                            || lower.contains("runtime") || lower.contains("sdk")
                            || lower == "proton" || lower == "steam linux runtime"
                        {
                            continue;
                        }

                        let install_dir = library.resolve_app_dir(&app).to_string_lossy().into_owned();
                        let app_id_str = format!("{}", app.app_id);
                        let steam_url = format!("steam://rungameid/{}", app_id_str);
                        let exe = find_game_executable(&install_dir);

                        games.push(InstalledGame {
                            name,
                            platform: Platform::Steam,
                            icon_path: None,
                            install_dir: install_dir.clone(),
                            app_id: Some(app_id_str),
                            launch_command: exe.or(Some(steam_url)),
                        });
                    }
                }
            }
        }
        games
    }

    // --- 2. BASE DE DATOS (Añadidos Manualmente - Multiplataforma) ---
    fn scan_manual_database(repo: &GameRepository) -> Vec<InstalledGame> {
        let mut games = Vec::new();
        if let Ok(db_games) = repo.get_all() {
            for game in db_games {
                let launch = if game.path.to_lowercase().ends_with(".exe") {
                    Some(game.path.clone())
                } else {
                    find_game_executable(&game.path)
                };
                games.push(InstalledGame {
                    name: game.title,
                    platform: Platform::Standalone,
                    install_dir: game.path,
                    icon_path: game.icon_path,
                    app_id: None,
                    launch_command: launch,
                });
            }
        }
        games
    }

    // --- 3. EPIC GAMES (Windows) ---
    #[cfg(target_os = "windows")]
    fn scan_epic() -> Vec<InstalledGame> {
        let mut games = Vec::new();
        let manifests_path = r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests";

        if let Ok(entries) = fs::read_dir(manifests_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().unwrap_or_default() != "item" {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        let name = json["DisplayName"].as_str().unwrap_or("Unknown");
                        let install_dir = json["InstallLocation"].as_str().unwrap_or("");
                        let app_id = json["CatalogItemId"].as_str()
                            .or_else(|| json["AppName"].as_str())
                            .map(|s| s.to_string());
                        // Epic launch URL
                        let app_name = json["AppName"].as_str().unwrap_or("");
                        let launch_url = format!("com.epicgames.launcher://apps/{}?action=launch&silent=true", app_name);
                        let exe = if !install_dir.is_empty() {
                            find_game_executable(install_dir)
                        } else {
                            None
                        };

                        games.push(InstalledGame {
                            name: name.to_string(),
                            platform: Platform::EpicGames,
                            install_dir: install_dir.to_string(),
                            icon_path: None,
                            app_id,
                            launch_command: exe.or(Some(launch_url)),
                        });
                    }
                }
            }
        }
        games
    }

    // --- 4. XBOX / GAME PASS (Nativo con windows crate) ---
    #[cfg(target_os = "windows")]
    fn scan_xbox() -> Vec<InstalledGame> {
        use windows::Management::Deployment::PackageManager;

        let mut games = Vec::new();

        let manager = match PackageManager::new() {
            Ok(m) => m,
            Err(_) => return games,
        };

        if let Ok(packages) = manager.FindPackages() {
            for package in packages {
                let is_game = || -> windows::core::Result<bool> {
                    let is_framework = package.IsFramework()?;
                    Ok(!is_framework)
                };

                if let Ok(true) = is_game() {
                    if let Ok(id) = package.Id() {
                        if let Ok(name) = id.Name() {
                            let install_dir = package
                                .InstalledLocation()
                                .and_then(|loc| loc.Path())
                                .map(|p| p.to_string())
                                .unwrap_or_else(|_| String::new());

                            let full_name = package
                                .Id()
                                .ok()
                                .and_then(|id| id.FullName().ok())
                                .map(|n| n.to_string());

                            games.push(InstalledGame {
                                name: name.to_string(),
                                platform: Platform::Xbox,
                                install_dir,
                                icon_path: None,
                                app_id: full_name,
                                launch_command: None, // UWP: shell:AppsFolder\AppId!App
                            });
                        }
                    }
                }
            }
        }
        games
    }

    // --- 5. STANDALONE (.exe) VIA REGISTRY (Playnite-style: filter + find exe) ---
    #[cfg(target_os = "windows")]
    fn scan_windows_registry() -> Vec<InstalledGame> {
        use winreg::enums::*;
        use winreg::RegKey;

        let mut games = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Check both HKLM and HKCU
        for hive in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
            let root = RegKey::predef(*hive);
            let sub_paths = if *hive == HKEY_LOCAL_MACHINE {
                vec![
                    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
                ]
            } else {
                vec![r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"]
            };

            for uninstall_path in sub_paths {
                if let Ok(uninstall_key) = root.open_subkey(uninstall_path) {
                    for key_name in uninstall_key.enum_keys().flatten() {
                        if let Ok(app_key) = uninstall_key.open_subkey(&key_name) {
                            // Skip if no DisplayName or empty
                            let display_name: String = match app_key.get_value("DisplayName") {
                                Ok(n) => n,
                                Err(_) => continue,
                            };
                            if display_name.trim().is_empty() {
                                continue;
                            }

                            // Playnite-style: skip known non-game entries
                            let lower = display_name.to_lowercase();
                            if EXCLUDED_REGISTRY_NAMES.iter().any(|pat| lower.contains(pat)) {
                                continue;
                            }

                            // Dedup
                            if !seen.insert(lower.clone()) {
                                continue;
                            }

                            let install_location: String = app_key
                                .get_value("InstallLocation")
                                .unwrap_or_default();

                            // Try to get DisplayIcon for .exe path
                            let icon_value: String = app_key
                                .get_value("DisplayIcon")
                                .unwrap_or_default();
                            let icon_path = if !icon_value.is_empty() && icon_value.to_lowercase().ends_with(".exe") {
                                // Clean up icon paths like "C:\path\game.exe,0"
                                Some(icon_value.split(',').next().unwrap_or(&icon_value).to_string())
                            } else {
                                None
                            };

                            // Find .exe: use DisplayIcon, or scan install dir, or fallback to empty
                            let launch_command = if let Some(ref exe) = icon_path {
                                if Path::new(exe).is_file() {
                                    Some(exe.clone())
                                } else if !install_location.is_empty() {
                                    find_game_executable(&install_location)
                                } else {
                                    None
                                }
                            } else if !install_location.is_empty() {
                                find_game_executable(&install_location)
                            } else {
                                None
                            };

                            // Only include registry entries where we found an actual .exe
                            if launch_command.is_none() {
                                continue;
                            }

                            games.push(InstalledGame {
                                name: display_name,
                                platform: Platform::Standalone,
                                install_dir: install_location,
                                icon_path,
                                app_id: None,
                                launch_command,
                            });
                        }
                    }
                }
            }
        }
        games
    }
}
