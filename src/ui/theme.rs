//! Xbox 360 Metro Theme - Color constants and styling

use gpui::color::Color;

/// Xbox 360 Metro color palette
pub struct MetroTheme {
    // Backgrounds
    pub background: Color,      // #0E0E0E - near-black
    pub surface: Color,        // #1A1A1A - tile background
    pub surface_hover: Color,  // #252525
    
    // Borders
    pub border_default: Color,  // #333333
    pub border_focused: Color, // #107C10 - Xbox Green
    
    // Text
    pub text_primary: Color,   // #FFFFFF - white
    pub text_secondary: Color,  // #999999 - grey
    pub text_inactive: Color,  // #666666
    
    // Accents
    pub accent: Color,         // #107C10 - Xbox Green
    pub accent_hover: Color,   // #8DC63F - lighter green
    
    // Status
    pub danger: Color,         // for B button (back)
    pub success: Color,        // for A button (select)
}

impl Default for MetroTheme {
    fn default() -> Self {
        Self {
            background: Color::rgb(0x0E0E0E),
            surface: Color::rgb(0x1A1A1A),
            surface_hover: Color::rgb(0x252525),
            border_default: Color::rgb(0x333333),
            border_focused: Color::rgb(0x107C10),
            text_primary: Color::WHITE,
            text_secondary: Color::rgb(0x999999),
            text_inactive: Color::rgb(0x666666),
            accent: Color::rgb(0x107C10),
            accent_hover: Color::rgb(0x8DC63F),
            danger: Color::rgb(0xE81123),   // Xbox red
            success: Color::rgb(0x107C10),  // Xbox green
        }
    }
}

/// Global theme instance
pub fn theme() -> MetroTheme {
    MetroTheme::default()
}
