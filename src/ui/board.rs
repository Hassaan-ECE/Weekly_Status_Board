use crate::model::{BoardDocument, Column as ModelColumn};
use crate::theme::Theme;
use crate::ui::app::{Editing, Selection, StatusApp};
use crate::ui::column::Column;
use crate::ui::input::TextInput;
use chrono::NaiveDate;
use gpui::{div, prelude::*, px, Entity};

#[allow(non_snake_case)]
pub fn Board(
    board: &BoardDocument,
    theme: &Theme,
    today: NaiveDate,
    view_mode: bool,
    editing: &Editing,
    selection: &Selection,
    input: Entity<TextInput>,
    app: Entity<StatusApp>,
) -> impl IntoElement {
    let widths = board.column_widths;

    div()
        .id("board-workspace")
        .flex()
        .flex_row()
        .flex_1()
        .w_full()
        .min_h_0()
        .px_3()
        .py_2()
        .gap(px(6.))
        .bg(theme.background)
        .child(Column(
            board,
            theme,
            today,
            ModelColumn::Target,
            widths[0],
            view_mode,
            editing,
            selection,
            input.clone(),
            app.clone(),
        ))
        .child(Column(
            board,
            theme,
            today,
            ModelColumn::InProgress,
            widths[1],
            view_mode,
            editing,
            selection,
            input.clone(),
            app.clone(),
        ))
        .child(Column(
            board,
            theme,
            today,
            ModelColumn::Done,
            widths[2],
            view_mode,
            editing,
            selection,
            input,
            app,
        ))
}
