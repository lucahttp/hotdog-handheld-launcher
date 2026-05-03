//! Button Hint - Controller button hints (footer)

use gpui::{
    div, App, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, StyleRefinement, Styled, Window,
    px, Rgba,
};
use crate::ui::theme::theme;

/// Controller button hint
pub struct ButtonHint {
    pub button: String,  // "A", "B", etc.
    pub color: Rgba,
    pub label: String,
}

impl ButtonHint {
    pub fn new(button: &str, color: Rgba, label: &str) -> Self {
        Self {
            button: button.to_string(),
            color,
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
        let t = theme();
        let hints = vec![
            ButtonHint::new("A", t.success, "Select"),
            ButtonHint::new("B", t.danger, "Back"),
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
            .pr(px(64.0))  // Metro right margin
            .py(px(24.0))
            .bg(t.background)
            .justify_end() // Align hints to the right
            .children(self.hints.into_iter().map(|hint| {
                div().flex().flex_row()
                    .gap(px(8.0))
                    .items_center()
                    .child(
                        div()
                            .size(px(24.0))
                            .rounded_full()
                            .bg(hint.color)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(t.text_primary)
                            .child(hint.button)
                    )
                    .child(
                        div()
                            .text_color(t.text_primary)
                            .child(hint.label)
                    )
            }))
    }
}
