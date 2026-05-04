//! Tab Bar - Horizontal navigation tabs (header)

use gpui::{
    div, ClickEvent, ElementId, EventEmitter, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, StyleRefinement, Styled, Window, px,
    Context,
};
use crate::ui::theme::theme;

/// Event emitted when a tab is clicked
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabSelectedEvent(pub usize);

/// Navigation tab
#[derive(Clone)]
pub struct Tab {
    pub id: String,
    pub label: String,
}

impl Tab {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
        }
    }
}

/// Tab bar component (horizontal header)
pub struct TabBar {
    id: ElementId,
    style: StyleRefinement,
    tabs: Vec<Tab>,
    active_tab_index: usize,
}

impl TabBar {
    pub fn new(id: impl Into<ElementId>, active_tab: &str) -> Self {
        let tabs = vec![
            Tab::new("bing", "bing"),
            Tab::new("home", "home"),
            Tab::new("social", "social"),
            Tab::new("games", "games"),
            Tab::new("tv & movies", "tv & movies"),
            Tab::new("music", "music"),
            Tab::new("apps", "apps"),
            Tab::new("settings", "settings"),
        ];
        
        let active_tab_index = tabs.iter().position(|t| t.id == active_tab).unwrap_or(1);
        
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            tabs,
            active_tab_index,
        }
    }
    
    pub fn active_tab_index(&self) -> usize {
        self.active_tab_index
    }
}

impl Styled for TabBar {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl Render for TabBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme();
        let active_index = self.active_tab_index;
        
        div()
            .id(self.id.clone())
            .gap(px(32.0))
            .pl(px(90.0))
            .pt(px(60.0))
            .pb(px(24.0))
            .bg(t.background)
            .items_end()
            .flex()
            .flex_row()
            .children(self.tabs.iter_mut().enumerate().map(|(i, tab)| {
                let is_active = i == active_index;
                let text_color = if is_active { t.text_primary } else { t.text_inactive };
                let font_size = if is_active { 48.0 } else { 32.0 };
                let tab_id = tab.id.clone();
                
                div()
                    .id(tab.id.clone())
                    .text_color(text_color)
                    .text_size(px(font_size))
                    .child(tab.label.clone())
                    .on_click(cx.listener(move |this: &mut TabBar, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<TabBar>| {
                        // Use captured tab_id from the closure
                        if let Some(index) = this.tabs.iter().position(|t| t.id == tab_id) {
                            log::info!("TabBar emitting TabSelectedEvent({})", index);
                            this.active_tab_index = index;
                            cx.emit(TabSelectedEvent(index));
                            cx.notify();
                        }
                    }))
            }))
    }
}

impl EventEmitter<TabSelectedEvent> for TabBar {}

impl IntoElement for TabBar {
    type Element = gpui::Stateful<gpui::Div>;
    
    fn into_element(self) -> Self::Element {
        let t = theme();
        
        div()
            .id(self.id)
            .gap(px(32.0))
            .pl(px(90.0))
            .pt(px(60.0))
            .pb(px(24.0))
            .bg(t.background)
            .items_end()
            .flex()
            .flex_row()
            .children(self.tabs.into_iter().enumerate().map(|(i, tab)| {
                let is_active = i == self.active_tab_index;
                let text_color = if is_active { t.text_primary } else { t.text_inactive };
                let font_size = if is_active { 48.0 } else { 32.0 };
                
                div()
                    .id(tab.id.clone())
                    .text_color(text_color)
                    .text_size(px(font_size))
                    .child(tab.label)
            }))
    }
}

impl InteractiveElement for TabBar {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        panic!("InteractiveElement not implemented for TabBar - use Render/IntoElement instead")
    }
}
