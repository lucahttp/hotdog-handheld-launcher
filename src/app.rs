//! GPUI application — Handheld Launcher with Xbox 360 Metro UI.
//!
//! Navigation state lives in a single `FocusState` struct from `crate::navigation`.
//! The `Render` impl maps focus state → visual highlights on TabBar, TileGrid, and GameCarousel.
//! Directional input (keyboard + gamepad) arrives via `NavAction` and is dispatched
//! through `self.nav.handle(action)` which returns an optional `NavEffect`.

use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use gpui::*;
use tokio::sync::mpsc;

use crate::navigation::{NavEngine, NavEffect, FocusSection, NavCtx};
use crate::ui::{ButtonHintBar, TileGrid, TileData, TileSize, theme, GameCarousel, GameItem};
use crate::ui::components::tab_bar::{TabBar, TabSelectedEvent};
use crate::input::NavAction;
use crate::scanner::GameScanner;

// ── Asset source for SVG loading ────────────────────────────────────

struct FileAssetSource {
    base: PathBuf,
}

impl AssetSource for FileAssetSource {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let full = self.base.join(path);
        match fs::read(&full) {
            Ok(data) => Ok(Some(Cow::Owned(data))),
            Err(_) => Ok(None),
        }
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let full = self.base.join(path);
        match fs::read_dir(&full) {
            Ok(entries) => Ok(entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned().into())
                .collect()),
            Err(_) => Ok(vec![]),
        }
    }
}

// ── Tab definitions ──────────────────────────────────────────────────

const TABS: &[&str] = &[
    "bing", "home", "social", "games", "tv & movies", "music", "apps", "settings",
];

/// Tab index for the games tab (used for carousel navigation guard).
const GAMES_TAB: usize = 3;

// ── GPUI actions (keyboard bindings) ─────────────────────────────────

actions!(launcher, [
    NavigateUp, NavigateDown, NavigateLeft, NavigateRight, SelectGame, Back
]);

// ── Sample games ─────────────────────────────────────────────────────

fn sample_game_items() -> Vec<GameItem> {
    vec![
        GameItem { id: 0, title: "Halo 4".into(), icon_path: None, rating: Some(5.0) },
        GameItem { id: 1, title: "Call of Duty".into(), icon_path: None, rating: Some(4.0) },
        GameItem { id: 2, title: "FIFA 24".into(), icon_path: None, rating: Some(4.0) },
        GameItem { id: 3, title: "Forza Horizon".into(), icon_path: None, rating: Some(5.0) },
        GameItem { id: 4, title: "Minecraft".into(), icon_path: None, rating: Some(4.0) },
        GameItem { id: 5, title: "GTA V".into(), icon_path: None, rating: Some(5.0) },
        GameItem { id: 6, title: "Rocket League".into(), icon_path: None, rating: Some(4.0) },
        GameItem { id: 7, title: "Fortnite".into(), icon_path: None, rating: Some(3.0) },
        GameItem { id: 8, title: "Apex Legends".into(), icon_path: None, rating: Some(4.0) },
        GameItem { id: 9, title: "Warzone".into(), icon_path: None, rating: Some(3.0) },
    ]
}

fn sample_games_for_launch() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Halo 4", "C:\\Games\\Halo4\\halo4.exe"),
        ("Call of Duty", "C:\\Games\\CoD\\cod.exe"),
        ("FIFA 24", "C:\\Games\\FIFA24\\fifa.exe"),
        ("Forza Horizon", "C:\\Games\\Forza\\forza.exe"),
        ("Minecraft", "C:\\Games\\Minecraft\\minecraft.exe"),
        ("GTA V", "C:\\Games\\GTAV\\gta5.exe"),
        ("Rocket League", "C:\\Games\\RocketLeague\\rocketleague.exe"),
        ("Fortnite", "C:\\Games\\Fortnite\\fortnite.exe"),
        ("Apex Legends", "C:\\Games\\Apex\\apex.exe"),
        ("Warzone", "C:\\Games\\Warzone\\warzone.exe"),
    ]
}

fn installed_game_to_item(game: &crate::scanner::InstalledGame, id: usize) -> GameItem {
    GameItem {
        id,
        title: game.name.clone(),
        icon_path: game.icon_path.clone(),
        rating: None,
    }
}

// ── Main application entity ──────────────────────────────────────────

pub struct HandheldLauncher {
    focus_handle: FocusHandle,
    tiles: Vec<TileData>,
    /// Current active tab (for content switching).
    active_tab: usize,
    games: Vec<crate::scanner::InstalledGame>,
    is_scanning: bool,
    tab_bar: Entity<TabBar>,
    /// Unified navigation engine — focus state + per-tab memory + view stack.
    nav: NavEngine,
    /// Cached game items (avoids re-allocating each render).
    sample_games: Vec<GameItem>,
}

impl HandheldLauncher {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let tiles = vec![
            TileData { title: "Open Tray".into(), icon_path: Some("assets/icons/disc.svg".into()), size: TileSize::MenuTile },
            TileData { title: "My Pins".into(), icon_path: Some("assets/icons/pin.svg".into()), size: TileSize::MenuTile },
            TileData { title: "Recent".into(), icon_path: Some("assets/icons/clock.svg".into()), size: TileSize::MenuTile },
            TileData { title: "Xbox 360: Metro UI".into(), icon_path: None, size: TileSize::HeroTile },
        ];

        let sample_games = sample_game_items();

        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let games = cx.background_executor().spawn(async {
                GameScanner::scan_all(None)
            }).await;

            let _ = this.update(cx, |launcher, cx| {
                launcher.games = games;
                launcher.is_scanning = false;
                cx.notify();
            });
        }).detach();

        let tab_bar = cx.new(|_cx| TabBar::new("tab-bar", "home"));

        cx.subscribe(&tab_bar, |this, _tb, event: &TabSelectedEvent, cx| {
            log::info!("TabBar clicked → tab {}", event.0);
            this.nav.switch_to_tab(event.0);
            this.active_tab = this.nav.active_tab;
            cx.notify();
        }).detach();

        Self {
            focus_handle,
            tiles,
            active_tab: 1,
            games: Vec::new(),
            is_scanning: true,
            tab_bar,
            nav: NavEngine::new(1), // start at "home" tab
            sample_games,
        }
    }

    /// Entry point for ALL navigation input (keyboard + gamepad).
    pub fn handle_nav_action(&mut self, action: NavAction, cx: &mut Context<Self>) {
        // Build context for the state machine
        let game_count = if self.active_tab == GAMES_TAB {
            if !self.is_scanning && !self.games.is_empty() {
                self.games.len()
            } else {
                self.sample_games.len()
            }
        } else {
            0
        };
        let ctx = if self.active_tab == GAMES_TAB && game_count > 0 {
            NavCtx::with_carousel(game_count)
        } else {
            NavCtx::no_carousel()
        };

        let effect = self.nav.handle(action, ctx);

        // Sync active_tab from NavEngine after handle
        self.active_tab = self.nav.active_tab;

        // Apply side effects
        match effect {
            NavEffect::SwitchTab(tab) => {
                self.active_tab = tab;
            }
            NavEffect::LaunchGame(idx) => {
                if idx < self.games.len() {
                    let game = &self.games[idx];
                    match &game.launch_command {
                        Some(cmd) if cmd.starts_with("steam://") || cmd.starts_with("com.epicgames.") => {
                            // URL-launch via shell (steam://, epic://)
                            log::info!("Launching via URL: {} = {}", game.name, cmd);
                            match open::that(cmd) {
                                Ok(()) => log::info!("URL launch initiated for {}", game.name),
                                Err(e) => log::error!("URL launch failed {}: {}", game.name, e),
                            }
                        }
                        Some(exe) if exe.to_lowercase().ends_with(".exe") => {
                            log::info!("Launching scanned game: {} @ {}", game.name, exe);
                            match crate::process::launch_game(crate::process::LaunchOptions {
                                exe_path: exe.clone(),
                                working_dir: Some(game.install_dir.clone()),
                                args: vec![],
                            }) {
                                Ok(h) => log::info!("Launched {} (PID {})", game.name, h.pid),
                                Err(e) => log::error!("Launch failed {}: {}", game.name, e),
                            }
                        }
                        _ => {
                            log::warn!("No usable launch path for {} ({:?})", game.name, game.launch_command);
                        }
                    }
                } else {
                    let games = sample_games_for_launch();
                    if idx < games.len() {
                        let (name, exe) = (games[idx].0, games[idx].1);
                        log::info!("Launching fallback sample game: {} @ {}", name, exe);
                        match crate::process::launch_game(crate::process::LaunchOptions {
                            exe_path: exe.to_string(),
                            working_dir: None,
                            args: vec![],
                        }) {
                            Ok(h) => log::info!("Launched {} (PID {})", name, h.pid),
                            Err(e) => log::error!("Launch failed {}: {}", name, e),
                        }
                    }
                }
            }
            NavEffect::ActivateTile(idx) => {
                if let Some(tile) = self.tiles.get(idx) {
                    log::info!("Tile activated: {}", tile.title);
                }
            }
            NavEffect::None => {}
        }

        // Propagate focus state to the TabBar entity for visual highlights.
        self.tab_bar.update(cx, |bar, _cx| {
            bar.set_active_tab(self.active_tab);
            bar.set_focused_tab(if self.nav.section() == FocusSection::Tabs {
                Some(self.nav.tab())
            } else {
                None
            });
        });

        cx.notify();
    }
}

// ── Render ───────────────────────────────────────────────────────────

impl Render for HandheldLauncher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme();
        let tab_name = TABS[self.active_tab];

        let hero_content = match tab_name {
            "home" => self.render_home().into_any_element(),
            "games" => self.render_games().into_any_element(),
            _ => self.render_fallback(cx).into_any_element(),
        };

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(t.background)
            .flex()
            .flex_col()
            .justify_between()
            .child(self.tab_bar.clone())
            .child(div().flex_grow().child(hero_content))
            .child(ButtonHintBar::new("hint-bar"))
            .key_context(NAV_CONTEXT)
            .on_action(cx.listener(|this, _: &NavigateUp, _w, cx| {
                this.handle_nav_action(NavAction::NavigateUp, cx);
            }))
            .on_action(cx.listener(|this, _: &NavigateDown, _w, cx| {
                this.handle_nav_action(NavAction::NavigateDown, cx);
            }))
            .on_action(cx.listener(|this, _: &NavigateLeft, _w, cx| {
                this.handle_nav_action(NavAction::NavigateLeft, cx);
            }))
            .on_action(cx.listener(|this, _: &NavigateRight, _w, cx| {
                this.handle_nav_action(NavAction::NavigateRight, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectGame, _w, cx| {
                this.handle_nav_action(NavAction::Select, cx);
            }))
            .on_action(cx.listener(|this, _: &Back, _w, cx| {
                this.handle_nav_action(NavAction::Back, cx);
            }))
    }
}

// ── Per-tab render helpers ───────────────────────────────────────────

impl HandheldLauncher {
    fn display_games(&self) -> Vec<GameItem> {
        if !self.is_scanning && !self.games.is_empty() {
            self.games.iter()
                .enumerate()
                .map(|(i, game)| installed_game_to_item(game, i))
                .collect()
        } else {
            self.sample_games.clone()
        }
    }

    fn render_home(&self) -> impl IntoElement {
        let focus = self.nav.hero_focused_tile();
        let home_tiles: Vec<TileData> = self.tiles.iter().map(|td| TileData {
            title: td.title.clone(),
            icon_path: td.icon_path.clone(),
            size: td.size,
        }).collect();

        div()
            .flex_grow()
            .child(TileGrid::new("tile-grid", home_tiles).with_focused(focus))
    }

    fn render_games(&self) -> impl IntoElement {
        let t = theme();
        let cf = self.nav.carousel_focused();
        let sel = self.nav.focus.carousel;
        let games = self.display_games();
        let count = games.len();

        div()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_color(t.text_secondary)
                    .text_size(px(14.0))
                    .pl(px(90.0))
                    .child("sort by title (A-Z)"),
            )
            .child(
                GameCarousel::new("games-carousel", games)
                    .selected(sel.min(count.saturating_sub(1)))
                    .with_focused(if cf { Some(sel.min(count.saturating_sub(1))) } else { None }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(24.0))
                    .pl(px(90.0))
                    .pt(px(16.0))
                    .child(
                        div()
                            .text_color(t.text_secondary)
                            .text_size(px(14.0))
                            .child(format!("You have {} games", count)),
                    )
                    .child(
                        div()
                            .text_color(t.text_inactive)
                            .text_size(px(12.0))
                            .child("(A) Select   (B) Back   (X) More Options   (Y) Search"),
                    ),
            )
    }

    fn render_fallback(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        let tiles_to_render = if self.is_scanning {
            vec![TileData {
                title: "Scanning games...".into(),
                icon_path: None,
                size: TileSize::HeroTile,
            }]
        } else if self.games.is_empty() {
            vec![TileData {
                title: "No games found".into(),
                icon_path: None,
                size: TileSize::HeroTile,
            }]
        } else {
            self.games.iter().map(|g| TileData {
                title: g.name.clone().into(),
                icon_path: g.icon_path.clone().map(Into::into),
                size: TileSize::MenuTile,
            }).collect()
        };
        div().flex_grow().child(TileGrid::new("tile-grid", tiles_to_render))
    }
}

// ── App init ─────────────────────────────────────────────────────────

/// Initialize the GPUI application window.
pub fn init(input_rx: Option<mpsc::UnboundedReceiver<NavAction>>) -> Result<()> {
    log::info!("Initializing GPUI application");

    gpui_platform::application()
        .with_assets(FileAssetSource {
            base: std::env::current_dir().unwrap_or_default(),
        })
        .run(move |cx: &mut App| {
        gpui_component::init(cx);

        let options = gpui::WindowOptions {
            titlebar: Some(gpui::TitlebarOptions {
                title: None,
                appears_transparent: true,
                traffic_light_position: None,
            }),
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                origin: gpui::Point::new(0.0.into(), 0.0.into()),
                size: gpui::Size { width: 1280.0.into(), height: 720.0.into() },
            })),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            bind_navigation_keys(cx);

            let view = cx.new(|cx| {
                let launcher = HandheldLauncher::new(cx);

                if let Some(mut rx) = input_rx {
                    cx.spawn(async move |this: WeakEntity<HandheldLauncher>, cx| {
                        while let Some(action) = rx.recv().await {
                            let _ = this.update(cx, |view, cx| {
                                view.handle_nav_action(action, cx);
                            });
                        }
                    }).detach();
                }

                launcher
            });

            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap();
    });

    Ok(())
}

// ── Keyboard bindings ────────────────────────────────────────────────

const NAV_CONTEXT: &str = "HandheldLauncher";

fn bind_navigation_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", NavigateUp, Some(NAV_CONTEXT)),
        KeyBinding::new("down", NavigateDown, Some(NAV_CONTEXT)),
        KeyBinding::new("left", NavigateLeft, Some(NAV_CONTEXT)),
        KeyBinding::new("right", NavigateRight, Some(NAV_CONTEXT)),
        KeyBinding::new("enter", SelectGame, Some(NAV_CONTEXT)),
        KeyBinding::new("escape", Back, Some(NAV_CONTEXT)),
        KeyBinding::new("w", NavigateUp, Some(NAV_CONTEXT)),
        KeyBinding::new("s", NavigateDown, Some(NAV_CONTEXT)),
        KeyBinding::new("a", NavigateLeft, Some(NAV_CONTEXT)),
        KeyBinding::new("d", NavigateRight, Some(NAV_CONTEXT)),
    ]);
}