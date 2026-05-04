//! Focus Ring - Manages GPUI focus state for tiles

/// Manages focus state for the tile grid
pub struct FocusRing {
    /// Current focused tile index
    focused_index: usize,
    /// Total tiles
    tile_count: usize,
}

impl FocusRing {
    /// Create a new focus ring
    pub fn new(tile_count: usize) -> Self {
        Self {
            focused_index: 0,
            tile_count,
        }
    }
    
    /// Get current focused index
    pub fn focused(&self) -> usize {
        self.focused_index
    }
    
    /// Set focused index
    pub fn set_focused(&mut self, index: usize) {
        if index < self.tile_count {
            self.focused_index = index;
        }
    }
    
    /// Check if a tile is focused
    pub fn is_focused(&self, index: usize) -> bool {
        self.focused_index == index
    }
    
    /// Move focus to next tile
    pub fn move_next(&mut self) {
        if self.focused_index < self.tile_count - 1 {
            self.focused_index += 1;
        }
    }
    
    /// Move focus to previous tile
    pub fn move_prev(&mut self) {
        if self.focused_index > 0 {
            self.focused_index -= 1;
        }
    }
}