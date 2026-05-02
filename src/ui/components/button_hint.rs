//! Button Hint - Controller button hints (footer)

use gpui::{h_flex, div, Element, px, Color};
use crate::ui::theme::theme;

/// Controller button hint
pub struct ButtonHint {
    pub button: String,  // "A", "B", etc.
    pub color: Color,
    pub label: String,
}

impl ButtonHint {
    pub fn new(button: &str, color: Color, label: &str) -> Self {
        Self {
            button: button.to_string(),
            color,
            label: label.to_string(),
        }
    }
}

/// Footer with controller button hints
pub struct ButtonHintBar {
    pub hints: Vec<ButtonHint>,
}

impl ButtonHintBar {
    pub fn new() -> Self {
        let t = theme();
        let hints = vec![
            ButtonHint::new("A", t.success, "Select"),
            ButtonHint::new("B", t.danger, "Back"),
        ];
        
        Self { hints }
    }
    
    /// Build the GPUI element
    pub fn build(&self) -> impl Element {
        let t = theme();
        
        h_flex()
            .h_flex()
            .gap_6()
            .px_6()
            .py_3()
            .bg(t.background)
            .children(self.hints.iter().map(|hint| {
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .size(px(24.0))
                            .rounded_full()
                            .bg(hint.color)
                            .child(hint.button.clone())
                    )
                    .child(hint.label.clone())
            }))
    }
}

impl Default for ButtonHintBar {
    fn default() -> Self {
        Self::new()
    }
}
