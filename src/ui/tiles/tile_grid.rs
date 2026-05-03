//! Tile Grid - Asymmetric grid layout for Metro tiles

use gpui::{
    div, App, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window,
    px, FocusHandle,
};
use super::metro_tile::{MetroTile, TileSize};

pub struct TileData {
    pub title: String,
    pub size: TileSize,
    pub focus_handle: FocusHandle,
}

#[derive(IntoElement)]
pub struct TileGrid {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    pub tiles: Vec<TileData>,
    pub columns: usize,
}

impl TileGrid {
    pub fn new(id: impl Into<ElementId>, tiles: Vec<TileData>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            style: StyleRefinement::default(),
            tiles,
            columns: 3, // Default column count
        }
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
        // We will hardcode a Metro-style layout using nested flex columns/rows
        // Assuming self.tiles has exactly 5 tiles for the Home view:
        // 0: Large2x2 (Main Game)
        // 1: Wide2x1 (Resume/Last Played)
        // 2: Small1x1 (Settings)
        // 3: Small1x1 (Store)
        // 4: Tall1x2 (Ad/Highlight)
        
        let t0 = self.tiles.get(0).unwrap();
        let t1 = self.tiles.get(1).unwrap();
        let t2 = self.tiles.get(2).unwrap();
        let t3 = self.tiles.get(3).unwrap();
        let t4 = self.tiles.get(4).unwrap();

        self.base
            .id(self.id)
            .flex()
            .flex_row()
            .gap(px(10.0))
            .pl(px(64.0)) // align with pivot
            .child(
                // Col 1: Large Tile
                MetroTile::new(0, &t0.title, t0.focus_handle.clone()).size(t0.size)
            )
            .child(
                // Col 2: Stacked
                div().flex().flex_col().gap(px(10.0))
                    .child(MetroTile::new(1, &t1.title, t1.focus_handle.clone()).size(t1.size))
                    .child(
                        div().flex().flex_row().gap(px(10.0))
                            .child(MetroTile::new(2, &t2.title, t2.focus_handle.clone()).size(t2.size))
                            .child(MetroTile::new(3, &t3.title, t3.focus_handle.clone()).size(t3.size))
                    )
            )
            .child(
                // Col 3: Tall Tile
                div().flex().flex_col().gap(px(10.0))
                    .child(MetroTile::new(4, &t4.title, t4.focus_handle.clone()).size(t4.size))
            )
    }
}
