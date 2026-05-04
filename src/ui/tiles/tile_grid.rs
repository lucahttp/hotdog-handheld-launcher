//! Tile Grid - Asymmetric grid layout for Metro tiles

use gpui::{
    div, App, Div, ElementId, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    StyleRefinement, Styled, Window,
    px,
};
use super::metro_tile::{MetroTile, TileSize};

pub struct TileData {
    pub title: String,
    pub icon_path: Option<String>,
    pub size: TileSize,
}

#[derive(IntoElement)]
pub struct TileGrid {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    pub tiles: Vec<TileData>,
    /// Which tile index is currently focused (passed from app navigation state)
    focused_index: Option<usize>,
}

impl TileGrid {
    pub fn new(id: impl Into<ElementId>, tiles: Vec<TileData>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            style: StyleRefinement::default(),
            tiles,
            focused_index: None,
        }
    }
    
    /// Set which tile index is focused (for keyboard navigation visual feedback)
    pub fn with_focused(mut self, index: Option<usize>) -> Self {
        self.focused_index = index;
        self
    }
}

impl InteractiveElement for TileGrid {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TileGrid {}

impl Styled for TileGrid {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TileGrid {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Layout: Left column with menu tiles, right column with hero/banner tiles
        
        let focused = self.focused_index;
        let mut left_col = div().flex().flex_col().gap(px(8.0));
        let mut right_col = div().flex().flex_col().gap(px(8.0));
        
        for (i, t) in self.tiles.iter().enumerate() {
            let is_tile_focused = focused == Some(i);
            let mut tile = MetroTile::new(i, &t.title)
                .size(t.size)
                .with_focus(is_tile_focused);
            if let Some(path) = &t.icon_path {
                tile = tile.icon(path);
            }
            
            if t.size == TileSize::MenuTile {
                left_col = left_col.child(tile);
            } else {
                right_col = right_col.child(tile);
            }
        }

        self.base
            .id(self.id)
            .flex()
            .flex_row()
            .gap(px(16.0))
            .pl(px(90.0)) // align with navbar
            .child(left_col)
            .child(right_col)
    }
}
