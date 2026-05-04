//! GPUI application setup and window management

use anyhow::Result;
use gpui::*;
use tokio::sync::mpsc;
use crate::ui::{ButtonHintBar, TileGrid, TileData, TileSize, theme, GameCarousel, GameItem};
use crate::ui::components::tab_bar::{TabBar, TabSelectedEvent};
use crate::input::NavAction;
use crate::scanner::{GameScanner, InstalledGame};

const TABS: &[&str] = &[
    "bing", "home", "social", "games", "tv & movies", "music", "apps", "settings",
];

// Navigation actions for keyboard/gamepad
actions!(launcher, [NavigateUp, NavigateDown, NavigateLeft, NavigateRight, SelectGame, Back]);

/// Which section currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    Tabs,
    Hero,
    GamesCarousel,
}

/// Hero section layout - which tile is focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroFocus {
    LeftColumn(usize),  // Index within left column buttons (0, 1, 2)
    CenterColumn,       // Center hero area
    RightColumn,       // Right column options
}

pub struct HandheldLauncher {
    focus_handle: FocusHandle,
    tiles: Vec<TileData>,
    active_tab_index: usize,
    games: Vec<InstalledGame>,
    is_scanning: bool,
    tab_bar: Entity<TabBar>,
    /// Current focused section
    focus_section: FocusSection,
    /// Current focused tab index (for tabs navigation)
    focused_tab_index: usize,
    /// Current hero focus position
    hero_focus: HeroFocus,
    /// Games carousel selected index
    games_carousel_index: usize,
}

impl HandheldLauncher {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let tiles = vec![
            TileData { title: "Open Tray".into(), icon_path: Some("assets/icons/disc.svg".into()), size: TileSize::MenuTile, focus_handle: cx.focus_handle() },
            TileData { title: "My Pins".into(), icon_path: Some("assets/icons/pin.svg".into()), size: TileSize::MenuTile, focus_handle: cx.focus_handle() },
            TileData { title: "Recent".into(), icon_path: Some("assets/icons/clock.svg".into()), size: TileSize::MenuTile, focus_handle: cx.focus_handle() },
            TileData { title: "Xbox 360: Metro UI".into(), icon_path: None, size: TileSize::HeroTile, focus_handle: cx.focus_handle() },
        ];

        // Context::spawn signature: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> R
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let games = cx.background_executor().spawn(async move {
                GameScanner::scan_all(None)
            }).await;

            let _ = this.update(cx, |launcher, cx| {
                launcher.games = games;
                launcher.is_scanning = false;
                cx.notify();
            });
        }).detach();
        
        // Create TabBar entity
        let tab_bar = cx.new(|cx| {
            TabBar::new("tab-bar", "home")
        });
        
        // Subscribe to tab selection events from TabBar
        cx.subscribe(&tab_bar, |this, _tab_bar, event: &TabSelectedEvent, cx| {
            log::info!("HandheldLauncher received TabSelectedEvent({})", event.0);
            this.active_tab_index = event.0;
            this.focused_tab_index = event.0;
            cx.notify();
        }).detach();

        Self {
            focus_handle,
            tiles,
            active_tab_index: 1, // Start on "home"
            games: Vec::new(),
            is_scanning: true,
            tab_bar,
            focus_section: FocusSection::Hero,
            focused_tab_index: 1,
            hero_focus: HeroFocus::LeftColumn(0),
            games_carousel_index: 0,
        }
    }

    pub fn handle_nav_action(&mut self, action: NavAction, cx: &mut Context<Self>) {
        use FocusSection::*;
        
        match action {
            NavAction::NavigateUp => {
                match self.focus_section {
                    Tabs => {
                        // Already at top of tabs, stay
                    }
                    Hero => {
                        // Move to tabs section
                        self.focus_section = FocusSection::Tabs;
                        cx.notify();
                    }
                    GamesCarousel => {
                        // Move up to hero section
                        self.focus_section = FocusSection::Hero;
                        cx.notify();
                    }
                }
            }
            NavAction::NavigateDown => {
                match self.focus_section {
                    Tabs => {
                        // Move to hero section
                        self.focus_section = FocusSection::Hero;
                        self.hero_focus = HeroFocus::LeftColumn(0);
                        cx.notify();
                    }
                    Hero => {
                        // Move down to games carousel (if on "games" tab)
                        if self.active_tab_index == 3 { // "games" tab
                            self.focus_section = FocusSection::GamesCarousel;
                            self.games_carousel_index = 0;
                            cx.notify();
                        }
                    }
                    GamesCarousel => {
                        // Already at bottom of carousel
                    }
                }
            }
            NavAction::NavigateLeft => {
                match self.focus_section {
                    Tabs => {
                        // Move left within tabs
                        if self.focused_tab_index > 0 {
                            self.focused_tab_index -= 1;
                            cx.notify();
                        }
                    }
                    Hero => {
                        // Move left within hero columns
                        match self.hero_focus {
                            HeroFocus::LeftColumn(_) => {
                                // Stay
                            }
                            HeroFocus::CenterColumn => {
                                self.hero_focus = HeroFocus::LeftColumn(0);
                                cx.notify();
                            }
                            HeroFocus::RightColumn => {
                                self.hero_focus = HeroFocus::CenterColumn;
                                cx.notify();
                            }
                        }
                    }
                    GamesCarousel => {
                        // Move left in carousel
                        if self.games_carousel_index > 0 {
                            self.games_carousel_index -= 1;
                            cx.notify();
                        }
                    }
                }
            }
            NavAction::NavigateRight => {
                match self.focus_section {
                    Tabs => {
                        // Move right within tabs
                        if self.focused_tab_index < TABS.len() - 1 {
                            self.focused_tab_index += 1;
                            cx.notify();
                        }
                    }
                    Hero => {
                        // Move right within hero columns
                        match self.hero_focus {
                            HeroFocus::LeftColumn(_) => {
                                self.hero_focus = HeroFocus::CenterColumn;
                                cx.notify();
                            }
                            HeroFocus::CenterColumn => {
                                self.hero_focus = HeroFocus::RightColumn;
                                cx.notify();
                            }
                            HeroFocus::RightColumn => {
                                // Stay
                            }
                        }
                    }
                    GamesCarousel => {
                        // Move right in carousel
                        if self.games_carousel_index < 9 { // 10 sample games
                            self.games_carousel_index += 1;
                            cx.notify();
                        }
                    }
                }
            }
            NavAction::PreviousTab => {
                if self.focused_tab_index > 0 {
                    self.focused_tab_index -= 1;
                    self.active_tab_index = self.focused_tab_index;
                    cx.notify();
                }
            }
            NavAction::NextTab => {
                if self.focused_tab_index < TABS.len() - 1 {
                    self.focused_tab_index += 1;
                    self.active_tab_index = self.focused_tab_index;
                    cx.notify();
                }
            }
            NavAction::Select => {
                match self.focus_section {
                    Tabs => {
                        // Select current tab
                        self.active_tab_index = self.focused_tab_index;
                        cx.notify();
                    }
                    Hero => {
                        // Launch focused item
                        log::info!("Selecting hero item: {:?}", self.hero_focus);
                    }
                    GamesCarousel => {
                        // Select focused game in carousel
                        log::info!("Selecting game at index: {}", self.games_carousel_index);
                    }
                }
            }
            _ => {}
        }
    }
}

impl Render for HandheldLauncher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme();

        let current_tab = TABS[self.active_tab_index];
        
        // Build hero content based on tab
        let hero_content = if current_tab == "home" {
            // Home tab shows the menu tiles (Open Tray, My Pins, Recent, Xbox 360: Metro UI)
            let home_tiles: Vec<TileData> = self.tiles.iter().map(|td| TileData {
                title: td.title.clone(),
                icon_path: td.icon_path.clone(),
                size: td.size.clone(),
                focus_handle: td.focus_handle.clone(),
            }).collect();
            div().flex_grow().child(TileGrid::new("tile-grid", home_tiles))
        } else if current_tab == "games" {
            // Games tab shows the carousel
            let sample_games = vec![
                GameItem { id: 0, title: "Halo 4".to_string(), icon_path: None, rating: Some(5.0) },
                GameItem { id: 1, title: "Call of Duty".to_string(), icon_path: None, rating: Some(4.0) },
                GameItem { id: 2, title: "FIFA 24".to_string(), icon_path: None, rating: Some(4.0) },
                GameItem { id: 3, title: "Forza Horizon".to_string(), icon_path: None, rating: Some(5.0) },
                GameItem { id: 4, title: "Minecraft".to_string(), icon_path: None, rating: Some(4.0) },
                GameItem { id: 5, title: "GTA V".to_string(), icon_path: None, rating: Some(5.0) },
                GameItem { id: 6, title: "Rocket League".to_string(), icon_path: None, rating: Some(4.0) },
                GameItem { id: 7, title: "Fortnite".to_string(), icon_path: None, rating: Some(3.0) },
                GameItem { id: 8, title: "Apex Legends".to_string(), icon_path: None, rating: Some(4.0) },
                GameItem { id: 9, title: "Warzone".to_string(), icon_path: None, rating: Some(3.0) },
            ];
            
            div()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .text_color(t.text_secondary)
                        .text_size(px(14.0))
                        .pl(px(90.0))
                        .child("sort by title (A-Z)")
                )
                .child(GameCarousel::new("games-carousel", sample_games))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(24.0))
                        .pl(px(90.0))
                        .pt(px(16.0))
                        .child(
                            div().text_color(t.text_secondary).text_size(px(14.0)).child("You have 10 games")
                        )
                        .child(
                            div().text_color(t.text_inactive).text_size(px(12.0)).child("(A) Select   (B) Back   (X) More Options   (Y) Search")
                        )
                )
        } else {
            // Games and other tabs show real scanned games
            let tiles_to_render = if self.is_scanning {
                vec![TileData { title: "Scanning games...".into(), icon_path: None, size: TileSize::HeroTile, focus_handle: cx.focus_handle() }]
            } else if self.games.is_empty() {
                vec![TileData { title: "No games found".into(), icon_path: None, size: TileSize::HeroTile, focus_handle: cx.focus_handle() }]
            } else {
                self.games.iter().map(|g| TileData {
                    title: g.name.clone().into(),
                    icon_path: g.icon_path.clone().map(Into::into),
                    size: TileSize::MenuTile,
                    focus_handle: cx.focus_handle(),
                }).collect()
            };
            div().flex_grow().child(TileGrid::new("tile-grid", tiles_to_render))
        };

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(t.background)
            .flex()
            .flex_col()
            .justify_between()
            .child(self.tab_bar.clone())
            .child(
                div()
                    .flex_grow()
                    .child(hero_content)
            )
            .child(ButtonHintBar::new("hint-bar"))
            .key_context(NAV_CONTEXT)
            .on_action(cx.listener(|this, action: &NavigateUp, _window, cx| {
                this.handle_nav_action(NavAction::NavigateUp, cx);
            }))
            .on_action(cx.listener(|this, action: &NavigateDown, _window, cx| {
                this.handle_nav_action(NavAction::NavigateDown, cx);
            }))
            .on_action(cx.listener(|this, action: &NavigateLeft, _window, cx| {
                this.handle_nav_action(NavAction::NavigateLeft, cx);
            }))
            .on_action(cx.listener(|this, action: &NavigateRight, _window, cx| {
                this.handle_nav_action(NavAction::NavigateRight, cx);
            }))
            .on_action(cx.listener(|this, action: &SelectGame, _window, cx| {
                this.handle_nav_action(NavAction::Select, cx);
            }))
            .on_action(cx.listener(|this, action: &Back, _window, cx| {
                this.handle_nav_action(NavAction::Back, cx);
            }))
    }
}

impl HandheldLauncher {
    fn on_tab_click(&mut self, tab_id: &str, cx: &mut Context<Self>) {
        if let Some(index) = TABS.iter().position(|&t| t == tab_id) {
            self.active_tab_index = index;
            cx.notify();
        }
    }
}

/// Initialize the GPUI application
pub fn init(input_rx: Option<mpsc::UnboundedReceiver<NavAction>>) -> Result<()> {
    log::info!("Initializing GPUI application");

    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);
        
        // Bind keyboard keys for navigation
        bind_navigation_keys(cx);

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
            let view = cx.new(|cx| {
                let launcher = HandheldLauncher::new(cx);

                if let Some(mut rx) = input_rx {
                    // Spawn input receiver loop inside the entity context
                    // Context::spawn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> R
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

// Keyboard bindings for navigation
const NAV_CONTEXT: &str = "HandheldLauncher";

fn bind_navigation_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", NavigateUp, Some(NAV_CONTEXT)),
        KeyBinding::new("down", NavigateDown, Some(NAV_CONTEXT)),
        KeyBinding::new("left", NavigateLeft, Some(NAV_CONTEXT)),
        KeyBinding::new("right", NavigateRight, Some(NAV_CONTEXT)),
        KeyBinding::new("enter", SelectGame, Some(NAV_CONTEXT)),
        KeyBinding::new("escape", Back, Some(NAV_CONTEXT)),
        // Alternative bindings
        KeyBinding::new("w", NavigateUp, Some(NAV_CONTEXT)),
        KeyBinding::new("s", NavigateDown, Some(NAV_CONTEXT)),
        KeyBinding::new("a", NavigateLeft, Some(NAV_CONTEXT)),
        KeyBinding::new("d", NavigateRight, Some(NAV_CONTEXT)),
    ]);
}