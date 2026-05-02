//! Tile Grid - Asymmetric grid layout for Metro tiles

use super::metro_tile::{MetroTile, TileSize};

/// Represents the 2D grid layout for navigation
pub struct TileGrid {
    pub tiles: Vec<MetroTile>,
    pub columns: usize, // e.g., 4 columns
}

impl TileGrid {
    /// Create a new grid with some placeholder tiles
    pub fn new() -> Self {
        let tiles = vec![
            MetroTile::new("Game 1"),
            MetroTile::new("Game 2"),
            MetroTile::new("Game 3").size(TileSize::Wide2x1),
            MetroTile::new("Emulator"),
            MetroTile::new("Settings"),
            MetroTile::new("Game 4"),
        ];
        
        Self {
            tiles,
            columns: 4,
        }
    }
    
    /// Get tile at position
    pub fn tile_at(&self, row: usize, col: usize) -> Option<&MetroTile> {
        let index = row * self.columns + col;
        self.tiles.get(index)
    }
    
    /// Total tiles
    pub fn len(&self) -> usize {
        self.tiles.len()
    }
    
    /// Iterate tiles
    pub fn iter(&self) -> impl Iterator<Item = &MetroTile> {
        self.tiles.iter()
    }
}

impl Default for TileGrid {
    fn default() -> Self {
        Self::new()
    }
}
