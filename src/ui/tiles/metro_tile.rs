//! Metro Tile component - Individual game/app tile

use gpui::{
    div, prelude::FluentBuilder as _, App, Div, ElementId,
    InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, px, svg,
};
use crate::ui::theme::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSize {
    Small1x1,   // Standard square tile (150x150)
    Wide2x1,    // Wide rectangle tile (310x150)
    Tall1x2,    // Tall rectangle tile (150x310)
    Large2x2,   // Large square tile (310x310)
    MenuTile,   // Shorter wide tile for left menu (280x120)
    HeroTile,   // Very large banner tile (600x380)
}

#[derive(IntoElement)]
pub struct MetroTile {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    title: String,
    icon_path: Option<String>,
    tile_size: TileSize,
    /// Override focus state for keyboard navigation
    is_focused_override: Option<bool>,
}

impl MetroTile {
    pub fn new(id: impl Into<ElementId>, title: &str) -> Self {
        Self {
            id: id.into(),
            base: div(),
            style: StyleRefinement::default(),
            title: title.to_string(),
            icon_path: None,
            tile_size: TileSize::Small1x1,
            is_focused_override: None,
        }
    }
    
    pub fn size(mut self, size: TileSize) -> Self {
        self.tile_size = size;
        self
    }

    pub fn icon(mut self, path: &str) -> Self {
        self.icon_path = Some(path.to_string());
        self
    }
    
    /// Set focus state manually (for keyboard navigation)
    pub fn with_focus(mut self, is_focused: bool) -> Self {
        self.is_focused_override = Some(is_focused);
        self
    }
}

impl InteractiveElement for MetroTile {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for MetroTile {}

impl Styled for MetroTile {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for MetroTile {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = theme();
        
        // Use keyboard navigation focus override if set, otherwise no focus by default
        let is_focused = self.is_focused_override.unwrap_or(false);
        
        let (width, height) = match self.tile_size {
            TileSize::Small1x1 => (150.0, 150.0),
            TileSize::Wide2x1 => (310.0, 150.0),
            TileSize::Tall1x2 => (150.0, 310.0),
            TileSize::Large2x2 => (310.0, 310.0),
            TileSize::MenuTile => (280.0, 120.0),
            TileSize::HeroTile => (600.0, 380.0),
        };
        
        let bg_color = match self.tile_size {
            TileSize::HeroTile => gpui::rgb(0x1d1d1d), // Dark grey for banners
            _ => t.accent, // Primary green for standard/menu tiles
        };

        let mut content = div()
            .flex()
            .size_full()
            .justify_between()
            .items_end();

        // Render Icon if exists (centered or large)
        if let Some(path) = &self.icon_path {
            content = content.child(
                div().absolute().inset_0().flex().items_center().justify_center()
                    .child(svg().path(path.clone()).size(px(48.0)).text_color(gpui::white()))
            );
        }

        // Title
        content = content.child(
            div()
                .text_color(gpui::white())
                .text_size(px(20.0))
                .pl(px(8.0))
                .pb(px(4.0))
                .child(self.title)
        );
        
        // Focus animation: scale up + glow when focused
        self.base
            .id(self.id)
            .w(px(width))
            .h(px(height))
            .bg(bg_color)
            .rounded_none() // Sharp edges
            .p(px(8.0))
            .hover(|el| el.bg(t.accent_hover).cursor_pointer())
            .when(is_focused, |el| {
                el.border(px(4.0))
                  .border_color(gpui::white())
                  .shadow_lg()
                  .bg(t.accent_hover)
            })
            .when(!is_focused, |el| {
                el.border(px(2.0))
                  .border_color(gpui::transparent_black()) // Prevent layout shift
            })
            .child(content)
    }
}
