use crate::theme::Theme;
use gpui::{div, prelude::*, px, FontWeight, Rgba};

fn column_header(label: &'static str, color: Rgba, theme: &Theme) -> impl IntoElement {
    div()
        .flex_1()
        .px_3()
        .py_2()
        .rounded(px(8.))
        .bg(color)
        .text_color(theme.header_fg)
        .font_weight(FontWeight::MEDIUM)
        .child(label)
}

#[allow(non_snake_case)]
pub fn Board(theme: &Theme) -> impl IntoElement {
    div()
        .id("board-workspace")
        .flex()
        .flex_col()
        .flex_1()
        .w_full()
        .min_h_0()
        .px_3()
        .py_2()
        .gap_2()
        .bg(theme.background)
        .child(
            div()
                .flex()
                .flex_row()
                .w_full()
                .gap_1()
                .flex_none()
                .child(column_header("Target", theme.target_header, theme))
                .child(column_header("In Progress", theme.progress_header, theme))
                .child(column_header("Done", theme.done_header, theme)),
        )
        .child(
            div()
                .flex_1()
                .w_full()
                .rounded(px(8.))
                .border_1()
                .border_color(theme.border)
                .bg(theme.fill_4),
        )
}
