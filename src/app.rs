//! GPUI application setup and window management

use anyhow::Result;
use gpui::*;
use crate::ui::{TabBar, ButtonHintBar, TileGrid, TileData, TileSize, theme};

pub struct HandheldLauncher {
    focus_handle: FocusHandle,
    tiles: Vec<TileData>,
}

impl HandheldLauncher {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        
        let tiles = vec![
            TileData { title: "Main Game".into(), size: TileSize::Large2x2, focus_handle: cx.focus_handle() },
            TileData { title: "Resume".into(), size: TileSize::Wide2x1, focus_handle: cx.focus_handle() },
            TileData { title: "Settings".into(), size: TileSize::Small1x1, focus_handle: cx.focus_handle() },
            TileData { title: "Store".into(), size: TileSize::Small1x1, focus_handle: cx.focus_handle() },
            TileData { title: "Ad/Highlight".into(), size: TileSize::Tall1x2, focus_handle: cx.focus_handle() },
        ];
        
        // tiles[0].focus_handle.focus(cx);

        Self {
            focus_handle,
            tiles,
        }
    }
}

impl Render for HandheldLauncher {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme();
        
        let tiles_cloned: Vec<TileData> = self.tiles.iter().map(|td| TileData {
            title: td.title.clone(),
            size: td.size.clone(),
            focus_handle: td.focus_handle.clone(),
        }).collect();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(t.background)
            .flex()
            .flex_col()
            .justify_between()
            .child(TabBar::new("tab-bar", "home"))
            .child(
                div()
                    .flex_grow()
                    .child(TileGrid::new("tile-grid", tiles_cloned))
            )
            .child(ButtonHintBar::new("hint-bar"))
    }
}

/// Initialize the GPUI application
pub fn init() -> Result<()> {
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
            let view = cx.new(|cx| HandheldLauncher::new(cx));
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        }).unwrap();
    });
    

    // gpui run loop doesn't return, but we need to satisfy the Result<()> signature
    Ok(())
}