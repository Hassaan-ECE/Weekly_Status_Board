use crate::model::{BoardDocument, Column as ModelColumn, ThemeMode};
use crate::theme::Theme;
use crate::ui::app::{Editing, StatusApp};
use crate::ui::card::Card;
use crate::ui::input::TextInput;
use chrono::NaiveDate;
use gpui::{div, prelude::*, px, relative, rgb, Entity, FontWeight, Rgba, SharedString};

fn column_title(column: ModelColumn) -> &'static str {
    match column {
        ModelColumn::Target => "Target",
        ModelColumn::InProgress => "In Progress",
        ModelColumn::Done => "Done",
    }
}

fn column_header_color(column: ModelColumn, theme: &Theme) -> Rgba {
    match column {
        ModelColumn::Target => theme.target_header,
        ModelColumn::InProgress => theme.progress_header,
        ModelColumn::Done => theme.done_header,
    }
}

fn project_header_fg(board: &BoardDocument, theme: &Theme) -> Rgba {
    match board.theme {
        ThemeMode::Dark => theme.foreground,
        ThemeMode::Light => rgb(0x111111),
    }
}

fn project_section(
    board: &BoardDocument,
    theme: &Theme,
    today: NaiveDate,
    project_id: &str,
    project_name: &str,
    column: ModelColumn,
    view_mode: bool,
    editing: &Editing,
    input: Entity<TextInput>,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let header_id: SharedString = format!("project-{}-{}", project_id, column.index()).into();
    let project_fg = project_header_fg(board, theme);
    let tasks = board.tasks_in(project_id, column);
    let editing_name = matches!(
        editing,
        Editing::ProjectName { id, column: edit_col }
            if id == project_id && *edit_col == column
    );
    let pid = project_id.to_string();

    let mut cards = div()
        .id(SharedString::from(format!(
            "section-cards-{}-{}",
            project_id,
            column.index()
        )))
        .flex()
        .flex_col()
        .w_full()
        .gap_1()
        .px_1()
        .pb_2();

    for task in tasks {
        cards = cards.child(Card(
            task,
            theme,
            today,
            view_mode,
            editing,
            input.clone(),
            app.clone(),
        ));
    }

    let name_el = if editing_name {
        div()
            .id(SharedString::from(format!(
                "edit-project-{}",
                project_id
            )))
            .flex_1()
            .min_w_0()
            .child(input)
            .into_any_element()
    } else {
        let app_name = app.clone();
        let pid_name = pid.clone();
        let name: SharedString = project_name.to_string().into();
        div()
            .id(SharedString::from(format!(
                "project-name-{}",
                project_id
            )))
            .flex_1()
            .min_w_0()
            .cursor_pointer()
            .child(name)
            .when(!view_mode, |el| {
                el.on_click(move |_, window, cx| {
                    app_name.update(cx, |app, cx| {
                        app.start_edit_project_name(&pid_name, column, window, cx);
                    });
                })
            })
            .into_any_element()
    };

    let mut header = div()
        .id(header_id)
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .bg(theme.fill_4)
        .text_color(project_fg)
        .font_weight(FontWeight::BOLD)
        .text_size(px(12.))
        .child(name_el);

    if !view_mode {
        let app_add = app.clone();
        let pid_add = pid.clone();
        header = header.child(
            div()
                .id(SharedString::from(format!(
                    "add-task-{}-{}",
                    project_id,
                    column.index()
                )))
                .flex_none()
                .px_1p5()
                .py_0p5()
                .rounded(px(4.))
                .cursor_pointer()
                .hover(|s| s.bg(theme.border))
                .child("+")
                .on_click(move |_, window, cx| {
                    app_add.update(cx, |app, cx| {
                        app.add_task_at(&pid_add, column, window, cx);
                    });
                }),
        );
    }

    div()
        .id(SharedString::from(format!(
            "section-{}-{}",
            project_id,
            column.index()
        )))
        .flex()
        .flex_col()
        .w_full()
        .gap_1()
        .child(header)
        .child(cards)
}

#[allow(non_snake_case)]
pub fn Column(
    board: &BoardDocument,
    theme: &Theme,
    today: NaiveDate,
    column: ModelColumn,
    width: f32,
    view_mode: bool,
    editing: &Editing,
    input: Entity<TextInput>,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let header_color = column_header_color(column, theme);
    let title = column_title(column);
    let col_id: SharedString = format!("column-{}", column.index()).into();

    let mut body = div()
        .id(SharedString::from(format!("column-body-{}", column.index())))
        .flex()
        .flex_col()
        .flex_1()
        .w_full()
        .min_h_0()
        .gap_2()
        .pt_2()
        .overflow_y_scroll();

    for project in &board.projects {
        if board.has_section(&project.id, column) {
            body = body.child(project_section(
                board,
                theme,
                today,
                &project.id,
                &project.name,
                column,
                view_mode,
                editing,
                input.clone(),
                app.clone(),
            ));
        }
    }

    if !view_mode {
        let app_proj = app.clone();
        body = body.child(
            div()
                .id(SharedString::from(format!(
                    "add-project-{}",
                    column.index()
                )))
                .w_full()
                .mt_1()
                .px_2()
                .py_2()
                .rounded(px(8.))
                .border_1()
                .border_color(theme.border)
                .border_dashed()
                .text_color(theme.muted)
                .text_sm()
                .cursor_pointer()
                .hover(|s| s.bg(theme.fill_4))
                .child("+ Add project")
                .on_click(move |_, window, cx| {
                    app_proj.update(cx, |app, cx| {
                        app.add_project_at(column, window, cx);
                    });
                }),
        );
    }

    div()
        .id(col_id)
        .flex()
        .flex_col()
        .h_full()
        .min_w_0()
        .w(relative(width))
        .child(
            div()
                .id(SharedString::from(format!(
                    "column-header-{}",
                    column.index()
                )))
                .w_full()
                .flex_none()
                .px_3()
                .py_2()
                .rounded(px(8.))
                .bg(header_color)
                .text_color(theme.header_fg)
                .font_weight(FontWeight::MEDIUM)
                .child(title),
        )
        .child(body)
}
