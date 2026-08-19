use crate::theme::Theme;
use crate::ui::app::StatusApp;
use gpui::{div, prelude::*, px, Entity, FontWeight, SharedString};

fn quiet_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
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
        .on_click(on_click)
}

fn toggle_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    pressed: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .flex_none()
        .px_2()
        .py_1()
        .rounded(px(8.))
        .border_1()
        .border_color(if pressed { theme.primary } else { theme.border })
        .bg(if pressed { theme.primary } else { theme.fill_4 })
        .text_color(if pressed {
            theme.primary_fg
        } else {
            theme.foreground
        })
        .text_sm()
        .cursor_pointer()
        .child(label.into())
        .on_click(on_click)
}

fn primary_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
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
        .on_click(on_click)
}

#[allow(non_snake_case)]
pub fn Header(
    theme: &Theme,
    zoom_label: SharedString,
    view_mode: bool,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let app_new = app.clone();
    let app_open = app.clone();
    let app_save = app.clone();
    let app_copy = app.clone();
    let app_export = app.clone();
    let app_view = app.clone();
    let app_zoom_out = app.clone();
    let app_zoom_in = app.clone();
    let app_zoom_reset = app.clone();
    let app_meeting = app.clone();

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
                .child(quiet_button("btn-new", "New", theme, move |_, _, cx| {
                    app_new.update(cx, |app, cx| {
                        app.new_board(cx);
                    });
                }))
                .child(quiet_button("btn-open", "Open", theme, move |_, _, cx| {
                    app_open.update(cx, |app, cx| {
                        app.open_board(cx);
                    });
                }))
                .child(quiet_button("btn-save", "Save", theme, move |_, _, cx| {
                    app_save.update(cx, |app, cx| {
                        app.save_board_action(cx);
                    });
                }))
                .child(quiet_button(
                    "btn-copy",
                    "Copy image",
                    theme,
                    move |_, window, cx| {
                        app_copy.update(cx, |app, cx| {
                            app.copy_image(window, cx);
                        });
                    },
                ))
                .child(quiet_button(
                    "btn-export",
                    "Export PNG",
                    theme,
                    move |_, window, cx| {
                        app_export.update(cx, |app, cx| {
                            app.export_png(window, cx);
                        });
                    },
                ))
                .child(toggle_button(
                    "btn-view",
                    "View",
                    theme,
                    view_mode,
                    move |_, window, cx| {
                        app_view.update(cx, |app, cx| {
                            app.toggle_view_mode(window, cx);
                        });
                    },
                ))
                .child(quiet_button("btn-theme", "Dark Theme", theme, |_, _, _| {}))
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
                        .child(quiet_button("btn-zoom-out", "−", theme, move |_, _, cx| {
                            app_zoom_out.update(cx, |app, cx| {
                                app.zoom_out(cx);
                            });
                        }))
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
                                .on_click(move |_, _, cx| {
                                    app_zoom_reset.update(cx, |app, cx| {
                                        app.reset_zoom(cx);
                                    });
                                }),
                        )
                        .child(quiet_button("btn-zoom-in", "+", theme, move |_, _, cx| {
                            app_zoom_in.update(cx, |app, cx| {
                                app.zoom_in(cx);
                            });
                        })),
                )
                .child(primary_button(
                    "btn-meeting",
                    "Attended weekly meeting",
                    theme,
                    move |_, window, cx| {
                        app_meeting.update(cx, |app, cx| {
                            app.clear_done_after_meeting(window, cx);
                        });
                    },
                )),
        )
}
