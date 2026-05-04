//! Button Hint - Controller button hints (footer)

use gpui::{
    div, App, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window,
    px, svg,
};
use crate::ui::theme::theme;

/// Controller button hint
pub struct ButtonHint {
    pub svg_path: String,
    pub label: String,
}

impl ButtonHint {
    pub fn new(svg_path: &str, label: &str) -> Self {
        Self {
            svg_path: svg_path.to_string(),
            label: label.to_string(),
        }
    }
}

/// Footer with controller button hints
#[derive(IntoElement)]
pub struct ButtonHintBar {
    id: ElementId,
    base: Div,
    style: StyleRefinement,
    hints: Vec<ButtonHint>,
}

impl ButtonHintBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        let hints = vec![
            ButtonHint::new("assets/icons/xbox_a.svg", "Select"),
            ButtonHint::new("assets/icons/xbox_y.svg", "Search / Eject"),
        ];
        
        Self {
            id: id.into(),
            base: div().flex().flex_row(),
            style: StyleRefinement::default(),
            hints,
        }
    }
}

impl InteractiveElement for ButtonHintBar {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for ButtonHintBar {}

impl Styled for ButtonHintBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for ButtonHintBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = theme();
        
        self.base
            .id(self.id)
            .gap(px(32.0))
            .pl(px(90.0))  // Align with left margin
            .pr(px(64.0))
            .py(px(24.0))
            .bg(t.background)
            .justify_start() // Align hints to the left
            .children(self.hints.into_iter().map(|hint| {
                div().flex().flex_row()
                    .gap(px(8.0))
                    .items_center()
                    .child(
                        svg()
                            .path(hint.svg_path)
                            .size(px(24.0))
                    )
                    .child(
                        div()
                            .text_color(t.text_primary)
                            .text_size(px(20.0))
                            .child(hint.label)
                    )
            }))
    }
}
