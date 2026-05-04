//! Tab Bar - Horizontal navigation tabs (header)

use gpui::{
    div, ClickEvent, ElementId, EventEmitter, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, px, Context,
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
    /// Which tab index is keyboard-focused (for visual highlight)
    focused_tab_index: Option<usize>,
}

impl TabBar {
    pub fn new(id: impl Into<ElementId>, active_tab: &str) -> Self {
        let tabs = vec![
            Tab::new("bing", "BING"),
            Tab::new("home", "HOME"),
            Tab::new("social", "SOCIAL"),
            Tab::new("games", "GAMES"),
            Tab::new("tv & movies", "TV & MOVIES"),
            Tab::new("music", "MUSIC"),
            Tab::new("apps", "APPS"),
            Tab::new("settings", "SETTINGS"),
        ];
        
        let active_tab_index = tabs.iter().position(|t| t.id == active_tab).unwrap_or(1);
        
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            tabs,
            active_tab_index,
            focused_tab_index: None,
        }
    }
    
    /// Set which tab is keyboard-focused (for visual highlight)
    pub fn with_focused_tab(mut self, index: Option<usize>) -> Self {
        self.focused_tab_index = index;
        self
    }

    /// Set focused tab via mutable reference (for use with Entity)
    pub fn set_focused_tab(&mut self, index: Option<usize>) {
        self.focused_tab_index = index;
    }

    /// Set active tab via mutable reference (for use with Entity)
    pub fn set_active_tab(&mut self, index: usize) {
        self.active_tab_index = index;
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
        let focused_index = self.focused_tab_index;
        
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
                let is_focused = focused_index == Some(i);
                // Show focus highlight when keyboard-focused (even if not active tab)
                let is_highlighted = is_active || is_focused;
                let text_color = if is_highlighted { t.text_primary } else { t.text_inactive };
                let font_size = if is_active { 48.0 } else { 32.0 };
                let tab_id = tab.id.clone();
                
                div()
                    .id(tab.id.clone())
                    .text_color(text_color)
                    .text_size(px(font_size))
                    .border(if is_focused { px(4.0) } else { px(0.0) })
                    .border_color(gpui::white())
                    .child(tab.label.clone())
                    .on_click(cx.listener(move |this: &mut TabBar, _event: &ClickEvent, _window: &mut Window, cx: &mut Context<TabBar>| {
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
