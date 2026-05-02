# Metro Shell UI

Xbox 360 Metro-style interface for the handheld launcher.

## Structure

```
ui/
├── theme.rs           # Color constants and styling (MetroTheme)
├── mod.rs             # Module exports
├── tiles/
│   ├── mod.rs         # Tiles module exports
│   ├── metro_tile.rs  # Individual tile component (MetroTile)
│   └── tile_grid.rs  # Grid layout manager (TileGrid)
└── components/
    ├── mod.rs        # Components module exports
    ├── tab_bar.rs     # Horizontal navigation tabs (TabBar)
    └── button_hint.rs # Controller button hints (ButtonHintBar)
```

## Theme Colors

| Element | Color | Hex |
|---------|-------|-----|
| Background | Near-black | #0E0E0E |
| Surface/Tiles | Dark grey | #1A1A1A |
| Border Default | Medium grey | #333333 |
| Border Focused | Xbox Green | #107C10 |
| Text Primary | White | #FFFFFF |
| Text Inactive | Grey | #666666 |
| Accent | Xbox Green | #107C10 |
| Danger | Xbox Red | #E81123 |

## Components

### MetroTile
A single tile in the Metro grid layout. Supports:
- `TileSize::Small1x1` - Standard square (150x150px)
- `TileSize::Wide2x1` - Wide rectangle (310x150px)
- `TileSize::Tall1x2` - Tall rectangle (150x310px)

Focused state shows 3px Xbox Green border with shadow.

### TileGrid
Manages the 2D grid layout for navigation with configurable columns.

### TabBar
Horizontal navigation header with tab items.

### ButtonHintBar
Footer displaying controller button hints (A=Select, B=Back).

## Usage

```rust
use crate::ui::{MetroTile, TileGrid, TabBar, ButtonHintBar, theme};

// Create themed components
let tiles = TileGrid::new();
let tabs = TabBar::new();
let hints = ButtonHintBar::new();

// Create custom tile
let tile = MetroTile::new("My Game")
    .size(TileSize::Wide2x1)
    .focused(true);
```

## Notes

This is a **skeleton** implementation. The actual GPUI element building syntax
may need adjustment based on the gpui 0.2 API. Focus on the data structures
and styling constants first.
