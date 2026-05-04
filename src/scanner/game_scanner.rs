use std::fs;
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

        // Recuperar los juegos agregados manualmente a través de la DB
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
                        games.push(InstalledGame {
                            name: app.name.clone().unwrap_or_else(|| "Unknown Steam Game".to_string()),
                            platform: Platform::Steam,
                            install_dir: library.resolve_app_dir(&app).to_string_lossy().into_owned(),
                            icon_path: None,
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
                games.push(InstalledGame {
                    name: game.title,
                    platform: Platform::Standalone,
                    install_dir: game.path,
                    icon_path: game.icon_path,
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
                if path.extension().unwrap_or_default() == "item" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            let name = json["DisplayName"].as_str().unwrap_or("Unknown");
                            let install_dir = json["InstallLocation"].as_str().unwrap_or("");
                            
                            games.push(InstalledGame {
                                name: name.to_string(),
                                platform: Platform::EpicGames,
                                install_dir: install_dir.to_string(),
                                icon_path: None,
                            });
                        }
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
            Err(_) => return games, // Falla silenciosa si el SO no lo soporta
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
                                .unwrap_or_else(|_| "Ruta Protegida (UWP)".to_string());

                            games.push(InstalledGame {
                                name: name.to_string(),
                                platform: Platform::Xbox,
                                install_dir,
                                icon_path: None,
                            });
                        }
                    }
                }
            }
        }
        games
    }

    // --- 5. STANDALONE (.exe / .msi) VIA REGISTRY ---
    #[cfg(target_os = "windows")]
    fn scan_windows_registry() -> Vec<InstalledGame> {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let mut games = Vec::new();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        let uninstall_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
        
        if let Ok(uninstall_key) = hklm.open_subkey(uninstall_path) {
            for key_name in uninstall_key.enum_keys().flatten() {
                if let Ok(app_key) = uninstall_key.open_subkey(&key_name) {
                    if let Ok(display_name) = app_key.get_value::<String, _>("DisplayName") {
                        let install_location: String = app_key
                            .get_value("InstallLocation")
                            .unwrap_or_default();
                            
                        games.push(InstalledGame {
                            name: display_name,
                            platform: Platform::Standalone,
                            install_dir: install_location,
                            icon_path: None,
                        });
                    }
                }
            }
        }
        games
    }
}
