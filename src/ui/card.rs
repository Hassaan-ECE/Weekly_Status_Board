use crate::dates::display_date;
use crate::model::Task;
use crate::theme::Theme;
use crate::ui::app::{Editing, StatusApp};
use crate::ui::input::TextInput;
use chrono::NaiveDate;
use gpui::{div, prelude::*, px, rgb, Entity, FontWeight, SharedString};

#[allow(non_snake_case)]
pub fn Card(
    task: &Task,
    theme: &Theme,
    today: NaiveDate,
    view_mode: bool,
    editing: &Editing,
    input: Entity<TextInput>,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let title: SharedString = if task.title.is_empty() {
        "Untitled".into()
    } else {
        task.title.clone().into()
    };
    let date: SharedString = display_date(task.due, today).into();
    let id: SharedString = format!("card-{}", task.id).into();
    let task_id = task.id.clone();
    let editing_title = matches!(editing, Editing::TaskTitle { id } if id == &task.id);
    let editing_due = matches!(editing, Editing::TaskDue { id } if id == &task.id);

    let title_el = if editing_title {
        div()
            .id(SharedString::from(format!("edit-title-{}", task.id)))
            .flex_1()
            .min_w_0()
            .child(input.clone())
            .into_any_element()
    } else {
        let app_title = app.clone();
        let tid = task_id.clone();
        div()
            .id(SharedString::from(format!("title-{}", task.id)))
            .flex_1()
            .min_w_0()
            .whitespace_normal()
            .text_sm()
            .text_color(theme.foreground)
            .cursor_pointer()
            .child(title)
            .when(!view_mode, |el| {
                el.on_click(move |_, window, cx| {
                    app_title.update(cx, |app, cx| {
                        app.start_edit_task_title(&tid, window, cx);
                    });
                })
            })
            .into_any_element()
    };

    let date_el = if editing_due {
        div()
            .id(SharedString::from(format!("edit-due-{}", task.id)))
            .flex_none()
            .w(px(72.))
            .child(input)
            .into_any_element()
    } else {
        let app_due = app.clone();
        let tid = task_id.clone();
        div()
            .id(SharedString::from(format!("due-{}", task.id)))
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
            .cursor_pointer()
            .child(date)
            .when(!view_mode, |el| {
                el.on_click(move |_, window, cx| {
                    app_due.update(cx, |app, cx| {
                        app.start_edit_task_due(&tid, window, cx);
                    });
                })
            })
            .into_any_element()
    };

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
        .child(title_el)
        .child(date_el)
}
