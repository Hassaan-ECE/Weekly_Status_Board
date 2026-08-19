use crate::theme::Theme;
use gpui::{div, prelude::*, FontWeight, SharedString};

#[allow(non_snake_case)]
pub fn Footer(theme: &Theme, status: SharedString) -> impl IntoElement {
    div()
        .id("app-footer")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .flex_none()
        .px_3()
        .py_2()
        .border_t_1()
        .border_color(theme.border)
        .bg(theme.background)
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(theme.muted)
                .child(status),
        )
        .child(
            div()
                .flex_none()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground)
                .child("Built by Syed Hassaan Shah"),
        )
}
