//! GPUI application setup and window management

use anyhow::Result;
use gpui::*;
use tokio::sync::mpsc;
use crate::ui::{ButtonHintBar, TileGrid, TileData, TileSize, theme};
use crate::ui::components::tab_bar::{TabBar, TabSelectedEvent};
use crate::input::NavAction;
use crate::scanner::{GameScanner, InstalledGame};

const TABS: &[&str] = &[
    "bing", "home", "social", "games", "tv & movies", "music", "apps", "settings",
];

pub struct HandheldLauncher {
    focus_handle: FocusHandle,
    tiles: Vec<TileData>,
    active_tab_index: usize,
    games: Vec<InstalledGame>,
    is_scanning: bool,
    tab_bar: Entity<TabBar>,
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
            cx.notify();
        }).detach();

        Self {
            focus_handle,
            tiles,
            active_tab_index: 1, // Start on "home"
            games: Vec::new(),
            is_scanning: true,
            tab_bar,
        }
    }

    pub fn handle_nav_action(&mut self, action: NavAction, cx: &mut Context<Self>) {
        match action {
            NavAction::PreviousTab => {
                if self.active_tab_index > 0 {
                    self.active_tab_index -= 1;
                    cx.notify();
                }
            }
            NavAction::NextTab => {
                if self.active_tab_index < TABS.len() - 1 {
                    self.active_tab_index += 1;
                    cx.notify();
                }
            }
            NavAction::Select => {
                // Gamepad A button - could launch game
            }
            _ => {}
        }
    }
}

impl Render for HandheldLauncher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme();

        let current_tab = TABS[self.active_tab_index];
        let tiles_to_render = if current_tab == "games" {
            if self.is_scanning {
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
            }
        } else {
            self.tiles.iter().map(|td| TileData {
                title: td.title.clone(),
                icon_path: td.icon_path.clone(),
                size: td.size.clone(),
                focus_handle: td.focus_handle.clone(),
            }).collect()
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
                    .child(TileGrid::new("tile-grid", tiles_to_render))
            )
            .child(ButtonHintBar::new("hint-bar"))
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