//! Game Carousel - Horizontal scrolling game library

use gpui::{
    div, prelude::FluentBuilder, App, ElementId, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, StyleRefinement, Styled, Window, px,
};
use crate::ui::theme::theme;

/// A single game item in the carousel
#[derive(Clone)]
pub struct GameItem {
    pub id: usize,
    pub title: String,
    pub icon_path: Option<String>,
    pub rating: Option<f32>,  // 0-5 stars
}

/// Carousel component for horizontal game browsing
#[derive(IntoElement)]
pub struct GameCarousel {
    id: ElementId,
    style: StyleRefinement,
    games: Vec<GameItem>,
    selected_index: usize,
    focused_index: Option<usize>,
}

impl GameCarousel {
    pub fn new(id: impl Into<ElementId>, games: Vec<GameItem>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            games,
            selected_index: 0,
            focused_index: None,
        }
    }

    /// Set which game index is keyboard-focused (for visual border highlight).
    pub fn with_focused(mut self, index: Option<usize>) -> Self {
        self.focused_index = index;
        self
    }

    /// Set which game index is visually selected (larger card).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }
    
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
}

impl Styled for GameCarousel {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for GameCarousel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = theme();
        let selected = self.selected_index;
        let focused = self.focused_index;
        
        let games = self.games.clone();
        
        div()
            .id(self.id)
            .flex()
            .flex_row()
            .gap(px(16.0))
            .pl(px(90.0))
            .items_start()
            .children(games.into_iter().enumerate().map(|(i, game)| {
                let is_selected = i == selected;
                let is_focused = focused == Some(i);
                // Show keyboard focus highlight (white border) even if not selected
                let show_focus = is_selected || is_focused;
                
                // Selected item is larger
                let width = if is_selected { 200.0 } else { 150.0 };
                let height = if is_selected { 280.0 } else { 210.0 };
                let font_size = if is_selected { 24.0 } else { 16.0 };
                let border_width = if show_focus { 4.0 } else { 2.0 };
                
                let title = game.title.clone();
                
                // Build game box
                let mut box_content = div()
                    .flex_col()
                    .flex_1()
                    .gap(px(8.0))
                    .bg(t.surface)
                    .rounded_md()
                    .p(px(12.0));
                
                // Game cover placeholder (colored box with title)
                box_content = box_content.child(
                    div()
                        .w(px(width - 24.0))
                        .h(px(height - 80.0))
                        .bg(if show_focus { t.accent } else { t.surface_hover })
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(title.chars().next().unwrap_or('G').to_string())
                );
                
                // Title
                box_content = box_content.child(
                    div()
                        .text_color(t.text_primary)
                        .text_size(px(font_size))
                        .child(title)
                );
                
                // Rating stars
                if let Some(rating) = game.rating {
                    let full_stars = "★".repeat(rating as usize);
                    let empty_stars = "☆".repeat((5 - rating as usize) as usize);
                    box_content = box_content.child(
                        div()
                            .text_color(t.text_secondary)
                            .text_size(px(12.0))
                            .child(format!("{}{}", full_stars, empty_stars))
                    );
                }
                
                div()
                    .id(format!("game-{}", game.id))
                    .w(px(width))
                    .h(px(height))
                    .flex()
                    .flex_col()
                    .border(px(border_width))
                    .when(is_focused, |el: gpui::Stateful<gpui::Div>| {
                        el.border_color(gpui::white())
                    })
                    .when(is_selected && !is_focused, |el: gpui::Stateful<gpui::Div>| {
                        el.border_color(t.accent)
                    })
                    .when(is_selected, |el: gpui::Stateful<gpui::Div>| {
                        el.shadow_lg()
                    })
                    .child(box_content)
            }))
    }
}
