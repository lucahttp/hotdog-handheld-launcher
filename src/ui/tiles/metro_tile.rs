//! Metro Tile component - Individual game/app tile

use gpui::{
    div, prelude::FluentBuilder as _, App, Div, ElementId, FocusHandle,
    InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, px,
};
use crate::ui::theme::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSize {
    Small1x1,  // Standard square tile (150x150)
    Wide2x1,   // Wide rectangle tile (310x150)
    Tall1x2,   // Tall rectangle tile (150x310)
    Large2x2,  // Large square tile (310x310)
}

#[derive(IntoElement)]
pub struct MetroTile {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    title: String,
    icon_path: Option<String>,
    tile_size: TileSize,
    focus_handle: FocusHandle,
}

impl MetroTile {
    pub fn new(id: impl Into<ElementId>, title: &str, focus_handle: FocusHandle) -> Self {
        Self {
            id: id.into(),
            base: div(),
            style: StyleRefinement::default(),
            title: title.to_string(),
            icon_path: None,
            tile_size: TileSize::Small1x1,
            focus_handle,
        }
    }
    
    pub fn size(mut self, size: TileSize) -> Self {
        self.tile_size = size;
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
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = theme();
        
        let is_focused = self.focus_handle.is_focused(window);
        
        let (width, height) = match self.tile_size {
            TileSize::Small1x1 => (150.0, 150.0),
            TileSize::Wide2x1 => (310.0, 150.0),
            TileSize::Tall1x2 => (150.0, 310.0),
            TileSize::Large2x2 => (310.0, 310.0),
        };
        
        self.base
            .id(self.id)
            .track_focus(&self.focus_handle)
            .w(px(width))
            .h(px(height))
            .bg(t.accent) // Tiles are often solid accent color
            .rounded_none() // Metro is perfectly sharp
            .p(px(12.0))
            .flex()
            .flex_col()
            .justify_end()
            .hover(|el| el.bg(t.accent_hover).cursor_pointer())
            .when(is_focused, |el| {
                el.border(px(4.0))
                  .border_color(t.text_primary) // White border on focus
                  .shadow_lg()
                  .bg(t.accent_hover)
            })
            .when(!is_focused, |el| {
                el.border(px(2.0))
                  .border_color(gpui::transparent_black()) // invisible border to prevent layout shift
            })
            .child(
                div()
                    .text_color(t.text_primary)
                    .child(self.title)
            )
    }
}
