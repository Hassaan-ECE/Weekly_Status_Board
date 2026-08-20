use crate::dates::display_date;
use crate::model::Task;
use crate::theme::Theme;
use crate::ui::app::{DragTask, Editing, Selection, StatusApp};
use crate::ui::input::TextInput;
use chrono::NaiveDate;
use gpui::{div, prelude::*, px, Entity, FontWeight, MouseButton, SharedString};

#[allow(non_snake_case)]
pub fn Card(
    task: &Task,
    theme: &Theme,
    today: NaiveDate,
    view_mode: bool,
    editing: &Editing,
    selection: &Selection,
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
    let project_id = task.project_id.clone();
    let task_title = task.title.clone();
    let task_column = task.column;
    let selected = matches!(selection, Selection::Card { id } if id == &task.id);
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
                el.on_click(move |event, window, cx| {
                    if event.click_count() >= 2 {
                        app_title.update(cx, |app, cx| {
                            app.start_edit_task_title(&tid, window, cx);
                        });
                    }
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
            .text_color(theme.foreground)
            .cursor_pointer()
            .child(date)
            .when(!view_mode, |el| {
                el.on_click(move |event, window, cx| {
                    if event.click_count() >= 2 {
                        app_due.update(cx, |app, cx| {
                            app.start_edit_task_due(&tid, window, cx);
                        });
                    }
                })
            })
            .into_any_element()
    };

    let mut card = div()
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
        .border_color(if selected {
            theme.primary
        } else {
            theme.border
        })
        .bg(theme.card)
        .child(title_el)
        .child(date_el);

    if selected && !view_mode {
        let app_del = app.clone();
        let tid_del = task_id.clone();
        card = card.child(
            div()
                .id(SharedString::from(format!("delete-{}", task_id)))
                .flex_none()
                .px_1()
                .py_0p5()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(|s| s.bg(theme.fill_4))
                .text_xs()
                .text_color(theme.muted)
                .child("×")
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    app_del.update(cx, |app, cx| {
                        app.delete_card(&tid_del, window, cx);
                    });
                }),
        );
        if let Some(dest) = task_column.left() {
            let app_left = app.clone();
            card = card.child(
                div()
                    .id(SharedString::from(format!("move-left-{}", task_id)))
                    .flex_none()
                    .px_1()
                    .py_0p5()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.fill_4))
                    .text_xs()
                    .child("[")
                    .on_click(move |_, _, cx| {
                        app_left.update(cx, |app, cx| {
                            app.move_selected_side(dest, cx);
                        });
                    }),
            );
        }
        if let Some(dest) = task_column.right() {
            let app_right = app.clone();
            card = card.child(
                div()
                    .id(SharedString::from(format!("move-right-{}", task_id)))
                    .flex_none()
                    .px_1()
                    .py_0p5()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.fill_4))
                    .text_xs()
                    .child("]")
                    .on_click(move |_, _, cx| {
                        app_right.update(cx, |app, cx| {
                            app.move_selected_side(dest, cx);
                        });
                    }),
            );
        }
    }

    if !view_mode {
        let app_sel = app.clone();
        let tid_sel = task_id.clone();
        card = card
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                cx.stop_propagation();
                app_sel.update(cx, |app, cx| {
                    app.select_card(&tid_sel, window, cx);
                });
            })
            .on_drag(
                DragTask {
                    id: task_id,
                    project_id,
                    title: task_title,
                },
                |drag, _, _, cx| cx.new(|_| drag.clone()),
            );
    }

    card
}
