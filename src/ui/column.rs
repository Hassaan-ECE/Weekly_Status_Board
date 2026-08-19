use crate::model::{BoardDocument, Column as ModelColumn, ThemeMode};
use crate::theme::Theme;
use crate::ui::card::Card;
use chrono::NaiveDate;
use gpui::{div, prelude::*, px, relative, rgb, FontWeight, Rgba, SharedString};

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
) -> impl IntoElement {
    let header_id: SharedString = format!("project-{}-{}", project_id, column.index()).into();
    let name: SharedString = project_name.to_string().into();
    let project_fg = project_header_fg(board, theme);
    let tasks = board.tasks_in(project_id, column);

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
        cards = cards.child(Card(task, theme, today));
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
        .child(
            div()
                .id(header_id)
                .w_full()
                .px_2()
                .py_1()
                .bg(theme.fill_4)
                .text_color(project_fg)
                .font_weight(FontWeight::BOLD)
                .text_size(px(12.))
                .child(name),
        )
        .child(cards)
}

#[allow(non_snake_case)]
pub fn Column(
    board: &BoardDocument,
    theme: &Theme,
    today: NaiveDate,
    column: ModelColumn,
    width: f32,
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
            ));
        }
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
