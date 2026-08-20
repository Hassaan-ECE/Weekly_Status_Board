use crate::dates::{month_cells, month_title};
use crate::theme::Theme;
use crate::ui::app::StatusApp;
use chrono::{Datelike, NaiveDate};
use gpui::{div, prelude::*, px, Entity, FontWeight, MouseButton, SharedString};

#[allow(non_snake_case)]
pub fn CalendarPanel(
    theme: &Theme,
    visible_month: NaiveDate,
    selected: NaiveDate,
    today: NaiveDate,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let title: SharedString = month_title(visible_month).into();
    let cells = month_cells(visible_month);
    let weekdays = ["S", "M", "T", "W", "T", "F", "S"];

    let mut header_row = div()
        .id("cal-weekdays")
        .flex()
        .flex_row()
        .w_full();
    for (i, label) in weekdays.iter().enumerate() {
        header_row = header_row.child(
            div()
                .id(SharedString::from(format!("cal-wd-{i}")))
                .flex_1()
                .py_1()
                .flex()
                .justify_center()
                .text_xs()
                .text_color(theme.muted)
                .child(*label),
        );
    }

    let mut grid = div().id("cal-grid").flex().flex_col().w_full().gap_1();
    for (row_i, row) in cells.chunks(7).enumerate() {
        let mut r = div()
            .id(SharedString::from(format!("cal-row-{row_i}")))
            .flex()
            .flex_row()
            .w_full();
        for (col_i, cell) in row.iter().enumerate() {
            r = r.child(day_cell(theme, selected, today, *cell, row_i, col_i, app.clone()));
        }
        grid = grid.child(r);
    }

    let app_prev = app.clone();
    let app_next = app;
    div()
        .id("calendar-panel")
        .w(px(268.))
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .rounded(px(10.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.card)
        .shadow_md()
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(nav_btn("cal-prev", "<", theme, move |window, cx| {
                    app_prev.update(cx, |app, cx| app.shift_calendar(-1, window, cx));
                }))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_center()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground)
                        .child(title),
                )
                .child(nav_btn("cal-next", ">", theme, move |window, cx| {
                    app_next.update(cx, |app, cx| app.shift_calendar(1, window, cx));
                })),
        )
        .child(header_row)
        .child(grid)
}

fn nav_btn(
    id: &'static str,
    label: &'static str,
    theme: &Theme,
    on_click: impl Fn(&mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_none()
        .w(px(28.))
        .h(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(6.))
        .cursor_pointer()
        .hover(|s| s.bg(theme.fill_4))
        .text_color(theme.foreground)
        .child(label)
        .on_click(move |_, window, cx| on_click(window, cx))
}

fn day_cell(
    theme: &Theme,
    selected: NaiveDate,
    today: NaiveDate,
    cell: Option<NaiveDate>,
    row: usize,
    col: usize,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let id: SharedString = format!("cal-d-{row}-{col}").into();
    let Some(date) = cell else {
        return div()
            .id(id)
            .flex_1()
            .h(px(30.))
            .into_any_element();
    };
    let is_selected = date == selected;
    let is_today = date == today;
    let label: SharedString = format!("{}", date.day()).into();
    let (bg, fg, border) = if is_selected {
        (theme.primary, theme.primary_fg, theme.primary)
    } else if is_today {
        (theme.fill_4, theme.foreground, theme.primary)
    } else {
        (theme.card, theme.foreground, theme.card)
    };
    div()
        .id(id)
        .flex_1()
        .h(px(30.))
        .m_0p5()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(6.))
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_xs()
        .text_color(fg)
        .cursor_pointer()
        .hover(|s| {
            if is_selected {
                s
            } else {
                s.bg(theme.fill_4)
            }
        })
        .on_click(move |_, window, cx| {
            app.update(cx, |app, cx| {
                app.pick_calendar_date(date, window, cx);
            });
        })
        .child(label)
        .into_any_element()
}
