//! Tab Bar - Horizontal navigation tabs (header)

use gpui::{h_flex, div, Element, px};
use crate::ui::theme::theme;

/// Navigation tab
pub struct Tab {
    pub id: String,
    pub label: String,
    pub is_active: bool,
}

impl Tab {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            is_active: false,
        }
    }
    
    pub fn active(mut self) -> Self {
        self.is_active = true;
        self
    }
}

/// Tab bar component (horizontal header)
pub struct TabBar {
    pub tabs: Vec<Tab>,
}

impl TabBar {
    pub fn new() -> Self {
        let tabs = vec![
            Tab::new("home", "home").active(),
            Tab::new("games", "games"),
            Tab::new("emulators", "emulators"),
            Tab::new("settings", "settings"),
        ];
        
        Self { tabs }
    }
    
    /// Build the GPUI element
    pub fn build(&self) -> impl Element {
        let t = theme();
        
        h_flex()
            .h_flex()
            .gap_4()
            .px_6()
            .py_4()
            .bg(t.background)
            .children(self.tabs.iter().map(|tab| {
                let text_color = if tab.is_active { t.text_primary } else { t.text_inactive };
                let font_size = if tab.is_active { 18.0 } else { 14.0 };
                
                div()
                    .text_color(text_color)
                    .font_size(px(font_size))
                    .child(tab.label.clone())
            }))
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}
