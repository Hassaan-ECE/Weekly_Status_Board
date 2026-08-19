use crate::theme::Theme;
use gpui::{div, prelude::*, px, FontWeight, SharedString};

fn quiet_button(id: impl Into<SharedString>, label: impl Into<SharedString>, theme: &Theme) -> impl IntoElement {
    div()
        .id(id.into())
        .flex_none()
        .px_2()
        .py_1()
        .rounded(px(8.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.fill_4)
        .text_color(theme.foreground)
        .text_sm()
        .cursor_pointer()
        .child(label.into())
        .on_click(|_, _, _| {})
}

fn primary_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .id(id.into())
        .flex_none()
        .px_3()
        .py_1()
        .rounded(px(8.))
        .bg(theme.primary)
        .text_color(theme.primary_fg)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .child(label.into())
        .on_click(|_, _, _| {})
}

#[allow(non_snake_case)]
pub fn Header(theme: &Theme, zoom_label: SharedString) -> impl IntoElement {
    div()
        .id("app-header")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .flex_none()
        .px_3()
        .py_2()
        .gap_2()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.background)
        .child(
            div()
                .flex_none()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground)
                .child("Weekly Status Board"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .flex_wrap()
                .child(quiet_button("btn-new", "New", theme))
                .child(quiet_button("btn-open", "Open", theme))
                .child(quiet_button("btn-save", "Save", theme))
                .child(quiet_button("btn-copy", "Copy image", theme))
                .child(quiet_button("btn-export", "Export PNG", theme))
                .child(quiet_button("btn-view", "View", theme))
                .child(quiet_button("btn-theme", "Dark Theme", theme))
                .child(
                    div()
                        .id("zoom-group")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .px_1()
                        .py_0p5()
                        .rounded(px(8.))
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.fill_4)
                        .child(quiet_button("btn-zoom-out", "−", theme))
                        .child(
                            div()
                                .id("btn-zoom-label")
                                .flex_none()
                                .px_2()
                                .py_1()
                                .text_sm()
                                .text_color(theme.foreground)
                                .cursor_pointer()
                                .child(zoom_label)
                                .on_click(|_, _, _| {}),
                        )
                        .child(quiet_button("btn-zoom-in", "+", theme)),
                )
                .child(primary_button(
                    "btn-meeting",
                    "Attended weekly meeting",
                    theme,
                )),
        )
}
