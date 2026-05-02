//! Grid Navigator - 2D spatial focus navigation

use crate::input::NavAction;
use crate::ui::tiles::{TileGrid, TileSize};

/// Navigation direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Grid position (row, column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position(pub usize, pub usize);

/// Spatial focus navigator for 2D grid
pub struct GridNavigator {
    /// Current focused tile index
    focused_index: usize,
    /// Grid layout info
    columns: usize,
    /// Tile sizes for spatial calculations
    tile_sizes: Vec<TileSize>,
}

impl GridNavigator {
    /// Create a new navigator with the given grid
    pub fn new(grid: &TileGrid) -> Self {
        Self {
            focused_index: 0, // Start with first tile focused
            columns: grid.columns,
            tile_sizes: grid.tiles.iter().map(|t| t.tile_size).collect(),
        }
    }
    
    /// Get current focused index
    pub fn focused(&self) -> usize {
        self.focused_index
    }
    
    /// Set focused index
    pub fn set_focused(&mut self, index: usize) {
        if index < self.tile_sizes.len() {
            self.focused_index = index;
        }
    }
    
    /// Navigate in a direction
    pub fn navigate(&mut self, direction: Direction) -> usize {
        let (row, col) = self.current_position();
        
        let (new_row, new_col) = match direction {
            Direction::Up => {
                // Find first non-tall tile in column above
                let above_row = row.saturating_sub(1);
                (above_row, col)
            }
            Direction::Down => {
                // Move down, handling tall tiles
                let below_row = row + 1;
                let max_row = (self.tile_sizes.len() / self.columns).saturating_sub(1);
                (below_row.min(max_row), col)
            }
            Direction::Left => {
                // Move left within row, wrap to end of previous row if at start
                if col > 0 {
                    (row, col - 1)
                } else if row > 0 {
                    // Wrap to end of previous row
                    let prev_row = row - 1;
                    let prev_row_cols = self.columns_for_row(prev_row);
                    (prev_row, prev_row_cols.saturating_sub(1))
                } else {
                    (row, col) // Already at start
                }
            }
            Direction::Right => {
                // Move right, wrap to start of next row if at end
                let max_col = self.effective_columns_for_tile(self.focused_index) - 1;
                if col < max_col {
                    (row, col + 1)
                } else if col >= self.columns - 1 {
                    // At end of row, go to start of next row
                    let next_row = row + 1;
                    let max_row = (self.tile_sizes.len() / self.columns).saturating_sub(1);
                    if next_row <= max_row {
                        (next_row, 0)
                    } else {
                        (row, col) // Already at end
                    }
                } else {
                    (row, col + 1)
                }
            }
        };
        
        // Calculate new index from position
        let new_index = self.index_at(new_row, new_col);
        
        if new_index != self.focused_index {
            self.focused_index = new_index;
        }
        
        self.focused_index
    }
    
    /// Handle a navigation action
    pub fn handle_action(&mut self, action: NavAction) -> Option<usize> {
        match action {
            NavAction::NavigateUp => Some(self.navigate(Direction::Up)),
            NavAction::NavigateDown => Some(self.navigate(Direction::Down)),
            NavAction::NavigateLeft => Some(self.navigate(Direction::Left)),
            NavAction::NavigateRight => Some(self.navigate(Direction::Right)),
            _ => None, // Select, Back, Menu don't change focus
        }
    }
    
    /// Get current position as (row, col)
    pub fn current_position(&self) -> (usize, usize) {
        let row = self.focused_index / self.columns;
        let col = self.focused_index % self.columns;
        (row, col)
    }
    
    /// Get index at position
    fn index_at(&self, row: usize, col: usize) -> usize {
        row * self.columns + col
    }
    
    /// Columns in a specific row (accounts for wide tiles)
    fn columns_for_row(&self, row: usize) -> usize {
        let start = row * self.columns;
        let end = (start + self.columns).min(self.tile_sizes.len());
        self.tile_sizes[start..end]
            .iter()
            .filter(|&&size| size != TileSize::Tall1x2)
            .count()
            .max(1)
    }
    
    /// Effective columns a tile occupies
    fn effective_columns_for_tile(&self, index: usize) -> usize {
        match self.tile_sizes.get(index) {
            Some(TileSize::Wide2x1) => 2,
            Some(TileSize::Tall1x2) => 1,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_navigation() {
        // Create a simple 3x3 grid (9 tiles)
        let grid = TileGrid::new();
        let mut nav = GridNavigator::new(&grid);
        
        assert_eq!(nav.focused(), 0); // Start at index 0
        
        // Navigate right
        nav.navigate(Direction::Right);
        assert_eq!(nav.focused(), 1);
        
        // Navigate down
        nav.navigate(Direction::Down);
        assert_eq!(nav.focused(), 4); // row 1, col 1
    }
    
    #[test]
    fn test_edge_wrap_right() {
        let grid = TileGrid::new();
        let mut nav = GridNavigator::new(&grid);
        
        // Set to last column (column 3 in a 4-column grid)
        nav.set_focused(3);
        assert_eq!(nav.focused(), 3);
        
        // Navigate right - should wrap to next row
        let new_index = nav.navigate(Direction::Right);
        // Should move to column 0 of next row
        assert!(new_index >= 4);
    }
}