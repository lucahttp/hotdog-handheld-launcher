//! Metro Tile component - Individual game/app tile

use gpui::{div, Element};
use crate::ui::theme::theme;

/// A single tile in the Metro grid
pub struct MetroTile {
    pub title: String,
    pub icon_path: Option<String>,
    pub is_focused: bool,
    pub tile_size: TileSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSize {
    Small1x1,  // Standard square tile
    Wide2x1,   // Wide rectangle tile
    Tall1x2,   // Tall rectangle tile (if needed)
}

impl Default for MetroTile {
    fn default() -> Self {
        Self {
            title: String::new(),
            icon_path: None,
            is_focused: false,
            tile_size: TileSize::Small1x1,
        }
    }
}

impl MetroTile {
    /// Create a new tile
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Default::default()
        }
    }
    
    /// Set tile size
    pub fn size(mut self, size: TileSize) -> Self {
        self.tile_size = size;
        self
    }
    
    /// Set focus state
    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }
    
    /// Build the GPUI element
    pub fn build(self) -> impl Element {
        let t = theme();
        let border = if self.is_focused {
            // Focused: 3px Xbox Green border + shadow
            div()
                .border(gpui::px(3.0))
                .border_color(t.border_focused)
                .shadow_md()
        } else {
            // Unfocused: 1px dark border
            div()
                .border(gpui::px(1.0))
                .border_color(t.border_default)
        };
        
        let size_style = match self.tile_size {
            TileSize::Small1x1 => ".size(px(150.0), px(150.0))",
            TileSize::Wide2x1 => ".size(px(310.0), px(150.0))",
            TileSize::Tall1x2 => ".size(px(150.0), px(310.0))",
        };
        
        // Base tile with background
        div()
            .merge(border)
            .bg(t.surface)
            .rounded(gpui::px(4.0))
            .child(self.title.clone())
    }
}
