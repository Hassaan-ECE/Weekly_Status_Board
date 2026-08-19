use crate::dates::display_date;
use crate::model::Task;
use crate::theme::Theme;
use chrono::NaiveDate;
use gpui::{div, prelude::*, px, rgb, FontWeight, SharedString};

#[allow(non_snake_case)]
pub fn Card(task: &Task, theme: &Theme, today: NaiveDate) -> impl IntoElement {
    let title: SharedString = if task.title.is_empty() {
        "Untitled".into()
    } else {
        task.title.clone().into()
    };
    let date: SharedString = display_date(task.due, today).into();
    let id: SharedString = format!("card-{}", task.id).into();

    div()
        .id(id)
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap_2()
        .px_2()
        .py_1()
        .rounded(px(8.))
        .border_1()
        .border_color(theme.border)
        .bg(theme.card)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .whitespace_normal()
                .text_sm()
                .text_color(theme.foreground)
                .child(title),
        )
        .child(
            div()
                .flex_none()
                .w(px(46.))
                .flex()
                .items_center()
                .justify_center()
                .py_0p5()
                .rounded(px(6.))
                .bg(theme.fill_4)
                .font_weight(FontWeight::BOLD)
                .text_xs()
                .text_color(rgb(0x111111))
                .child(date),
        )
}
