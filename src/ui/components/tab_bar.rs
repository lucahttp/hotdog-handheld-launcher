//! Tab Bar - Horizontal navigation tabs (header)

use gpui::{
    div, App, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window,
    px,
};
use crate::ui::theme::theme;

/// Navigation tab
#[derive(Clone)]
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
#[derive(IntoElement)]
pub struct TabBar {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    tabs: Vec<Tab>,
}

impl TabBar {
    pub fn new(id: impl Into<ElementId>, active_tab: &str) -> Self {
        let tabs = vec![
            Tab::new("bing", "bing"),
            Tab::new("home", "home"),
            Tab::new("social", "social"),
            Tab::new("games", "games"),
            Tab::new("apps", "apps"),
        ];
        
        let mut tabs = tabs.into_iter().map(|mut t| {
            if t.id == active_tab {
                t.is_active = true;
            }
            t
        }).collect();
        
        Self {
            id: id.into(),
            base: div().flex().flex_row(),
            style: StyleRefinement::default(),
            tabs,
        }
    }
}

impl InteractiveElement for TabBar {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TabBar {}

impl Styled for TabBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TabBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = theme();
        
        self.base
            .id(self.id)
            .gap(px(32.0))
            .pl(px(64.0))  // Metro UI significant left margin
            .pt(px(48.0))  // Top margin
            .pb(px(24.0))  // Bottom margin before tiles
            .bg(t.background)
            .items_end()   // Align texts to bottom baseline
            .children(self.tabs.into_iter().map(|tab| {
                let text_color = if tab.is_active { t.text_primary } else { t.text_inactive };
                let font_size = if tab.is_active { 48.0 } else { 32.0 };
                
                div()
                    .text_color(text_color)
                    .text_size(px(font_size))
                    //.font_weight(gpui::FontWeight::LIGHT) // Metro uses light fonts
                    .child(tab.label)
            }))
    }
}
