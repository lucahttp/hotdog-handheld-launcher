//! Xbox 360 Metro Theme - Color constants and styling

use gpui::{rgb, Rgba};

/// Xbox 360 Metro color palette
#[derive(Clone, Copy)]
pub struct MetroTheme {
    // Backgrounds
    pub background: Rgba,      // #0E0E0E - near-black
    pub surface: Rgba,        // #1A1A1A - tile background
    pub surface_hover: Rgba,  // #252525
    
    // Borders
    pub border_default: Rgba,  // #333333
    pub border_focused: Rgba, // #107C10 - Xbox Green
    
    // Text
    pub text_primary: Rgba,   // #FFFFFF - white
    pub text_secondary: Rgba,  // #999999 - grey
    pub text_inactive: Rgba,  // #666666
    
    // Accents
    pub accent: Rgba,         // #107C10 - Xbox Green
    pub accent_hover: Rgba,   // #8DC63F - lighter green
    
    // Status
    pub danger: Rgba,         // for B button (back)
    pub success: Rgba,        // for A button (select)
}

impl Default for MetroTheme {
    fn default() -> Self {
        Self {
            background: rgb(0x0E0E0E),
            surface: rgb(0x1A1A1A),
            surface_hover: rgb(0x252525),
            border_default: rgb(0x333333),
            border_focused: rgb(0x107C10),
            text_primary: rgb(0xFFFFFF),
            text_secondary: rgb(0x999999),
            text_inactive: rgb(0x666666),
            accent: rgb(0x107C10),
            accent_hover: rgb(0x8DC63F),
            danger: rgb(0xE81123),   // Xbox red
            success: rgb(0x107C10),  // Xbox green
        }
    }
}

/// Global theme instance
pub fn theme() -> MetroTheme {
    MetroTheme::default()
}
