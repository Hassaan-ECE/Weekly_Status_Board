use crate::dates::{display_date, parse_date};
use crate::history::History;
use crate::model::{BoardDocument, Column};
use crate::theme;
use crate::ui::dialogs;
use crate::ui::input::{self, TextInput};
use crate::ui::{board, footer, header};
use chrono::NaiveDate;
use gpui::{
    actions, div, prelude::*, px, rgb, Context, Entity, FocusHandle, Focusable, FontWeight,
    KeyBinding, MouseButton, MouseDownEvent, SharedString, Window,
};

actions!(
    board_edit,
    [
        CommitEdit,
        CancelEdit,
        DeleteSelected,
        MoveSelectedLeft,
        MoveSelectedRight
    ]
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Editing {
    None,
    ProjectName { id: String, column: Column },
    TaskTitle { id: String },
    TaskDue { id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    None,
    Card { id: String },
    Section { project_id: String, column: Column },
}

#[derive(Clone, Debug)]
pub struct DragTask {
    pub id: String,
    pub project_id: String,
    pub title: String,
}

impl Render for DragTask {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let title: SharedString = if self.title.is_empty() {
            "Untitled".into()
        } else {
            self.title.clone().into()
        };
        div()
            .px_2()
            .py_1()
            .rounded(px(8.))
            .bg(rgb(0x5558E6))
            .text_color(rgb(0xffffff))
            .text_sm()
            .shadow_md()
            .child(title)
    }
}

pub struct StatusApp {
    pub board: BoardDocument,
    pub history: History,
    pub view_mode: bool,
    pub status: String,
    pub theme_dark: bool,
    pub editing: Editing,
    pub selection: Selection,
    pub input: Entity<TextInput>,
    focus_handle: FocusHandle,
}

fn demo_board() -> BoardDocument {
    let mut board = BoardDocument::empty();
    let today = chrono::Local::now().date_naive();

    let (hvdc, target_task) = board.add_project(Column::Target);
    board.set_project_name(&hvdc, "HVDC");
    board.set_task_title(&target_task, "Order busbar hardware");
    board.set_task_due(&target_task, today);

    let progress_task = board.add_task(&hvdc, Column::InProgress);
    board.set_task_title(&progress_task, "Assemble rectifier rack");
    board.set_task_due(&progress_task, today.succ_opt().unwrap_or(today));

    // Empty Done section so project heading still shows with no cards.
    board.ensure_section(&hvdc, Column::Done);

    let (irhx, done_task) = board.add_project(Column::Done);
    board.set_project_name(&irhx, "IRHX");
    board.set_task_title(&done_task, "Ship spare fans");
    board.set_task_due(
        &done_task,
        NaiveDate::from_ymd_opt(2025, 12, 15).unwrap_or(today),
    );

    board
}

impl StatusApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let board = demo_board();
        let history = History::new(board.clone());
        let input = cx.new(|cx| TextInput::new(cx, "", ""));
        Self {
            board,
            history,
            view_mode: false,
            status: String::new(),
            theme_dark: false,
            editing: Editing::None,
            selection: Selection::None,
            input,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn bind_edit_keys(cx: &mut gpui::App) {
        input::bind_input_keys(cx);
        cx.bind_keys([
            KeyBinding::new("enter", CommitEdit, Some("TextInput")),
            KeyBinding::new("escape", CancelEdit, Some("TextInput")),
            KeyBinding::new("delete", DeleteSelected, None),
            KeyBinding::new("backspace", DeleteSelected, None),
            KeyBinding::new("[", MoveSelectedLeft, None),
            KeyBinding::new("]", MoveSelectedRight, None),
        ]);
    }

    fn zoom_label(&self) -> SharedString {
        let pct = (self.board.zoom * 100.0).round() as i32;
        format!("{pct}%").into()
    }

    fn today() -> NaiveDate {
        chrono::Local::now().date_naive()
    }

    fn focus_app(&self, window: &mut Window) {
        self.focus_handle.focus(window);
    }

    pub fn select_card(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.view_mode {
            return;
        }
        self.selection = Selection::Card { id: id.to_string() };
        if matches!(self.editing, Editing::None) {
            self.focus_app(window);
        }
        cx.notify();
    }

    pub fn select_section(
        &mut self,
        project_id: &str,
        column: Column,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode {
            return;
        }
        self.selection = Selection::Section {
            project_id: project_id.to_string(),
            column,
        };
        if matches!(self.editing, Editing::None) {
            self.focus_app(window);
        }
        cx.notify();
    }

    fn begin_edit(
        &mut self,
        editing: Editing,
        initial: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode {
            return;
        }
        if !matches!(self.editing, Editing::None) {
            self.commit_current(cx);
        }
        match &editing {
            Editing::TaskTitle { id } | Editing::TaskDue { id } => {
                self.selection = Selection::Card { id: id.clone() };
            }
            Editing::ProjectName { id, column } => {
                self.selection = Selection::Section {
                    project_id: id.clone(),
                    column: *column,
                };
            }
            Editing::None => {}
        }
        self.editing = editing;
        self.input.update(cx, |input, cx| {
            input.set_text(initial, cx);
        });
        self.input.read(cx).focus(window);
        cx.notify();
    }

    pub fn start_edit_project_name(
        &mut self,
        id: &str,
        column: Column,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = self
            .board
            .projects
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        self.begin_edit(
            Editing::ProjectName {
                id: id.to_string(),
                column,
            },
            name,
            window,
            cx,
        );
    }

    pub fn start_edit_task_title(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let title = self
            .board
            .task(id)
            .map(|t| t.title.clone())
            .unwrap_or_default();
        self.begin_edit(Editing::TaskTitle { id: id.to_string() }, title, window, cx);
    }

    pub fn start_edit_task_due(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let draft = self
            .board
            .task(id)
            .map(|t| display_date(t.due, Self::today()))
            .unwrap_or_default();
        self.begin_edit(Editing::TaskDue { id: id.to_string() }, draft, window, cx);
    }

    pub fn add_task_at(
        &mut self,
        project_id: &str,
        column: Column,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode {
            return;
        }
        let task_id = self.board.add_task(project_id, column);
        self.history.push(self.board.clone());
        self.begin_edit(Editing::TaskTitle { id: task_id }, String::new(), window, cx);
    }

    pub fn add_project_at(
        &mut self,
        column: Column,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode {
            return;
        }
        let (project_id, _) = self.board.add_project(column);
        self.history.push(self.board.clone());
        let name = self
            .board
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "New project".into());
        self.begin_edit(
            Editing::ProjectName {
                id: project_id,
                column,
            },
            name,
            window,
            cx,
        );
    }

    pub fn move_selected_side(&mut self, dest: Column, cx: &mut Context<Self>) {
        if self.view_mode || !matches!(self.editing, Editing::None) {
            return;
        }
        let Selection::Card { id } = &self.selection else {
            return;
        };
        let id = id.clone();
        self.board.move_task_side(&id, dest);
        self.history.push(self.board.clone());
        cx.notify();
    }

    pub fn move_task_to(
        &mut self,
        task_id: &str,
        dest_column: Column,
        dest_project_id: &str,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode {
            return;
        }
        self.board
            .move_task(task_id, dest_column, dest_project_id);
        self.history.push(self.board.clone());
        self.selection = Selection::Card {
            id: task_id.to_string(),
        };
        cx.notify();
    }

    fn delete_selected(&mut self, _: &DeleteSelected, window: &mut Window, cx: &mut Context<Self>) {
        if self.view_mode || !matches!(self.editing, Editing::None) {
            return;
        }
        match self.selection.clone() {
            Selection::None => {}
            Selection::Card { id } => {
                self.board.delete_task(&id);
                self.history.push(self.board.clone());
                self.selection = Selection::None;
                cx.notify();
            }
            Selection::Section { project_id, column } => {
                let has_tasks = !self.board.tasks_in(&project_id, column).is_empty();
                if !has_tasks {
                    self.board.delete_section(&project_id, column);
                    self.history.push(self.board.clone());
                    self.selection = Selection::None;
                    cx.notify();
                    return;
                }
                let answer = dialogs::confirm_delete_section(window, cx);
                cx.spawn(async move |this, cx| {
                    if answer.await.ok() != Some(1) {
                        return;
                    }
                    this.update(cx, |app, cx| {
                        app.board.delete_section(&project_id, column);
                        app.history.push(app.board.clone());
                        app.selection = Selection::None;
                        cx.notify();
                    })
                    .ok();
                })
                .detach();
            }
        }
    }

    fn move_selected_left(
        &mut self,
        _: &MoveSelectedLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode || !matches!(self.editing, Editing::None) {
            return;
        }
        let Selection::Card { id } = &self.selection else {
            return;
        };
        let Some(col) = self.board.task(id).map(|t| t.column) else {
            return;
        };
        let Some(dest) = col.left() else {
            return;
        };
        self.move_selected_side(dest, cx);
    }

    fn move_selected_right(
        &mut self,
        _: &MoveSelectedRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode || !matches!(self.editing, Editing::None) {
            return;
        }
        let Selection::Card { id } = &self.selection else {
            return;
        };
        let Some(col) = self.board.task(id).map(|t| t.column) else {
            return;
        };
        let Some(dest) = col.right() else {
            return;
        };
        self.move_selected_side(dest, cx);
    }

    fn commit_current(&mut self, cx: &mut Context<Self>) {
        if matches!(self.editing, Editing::None) {
            return;
        }
        let draft = self.input.read(cx).text();
        let before = self.board.clone();
        match self.editing.clone() {
            Editing::None => return,
            Editing::ProjectName { id, .. } => {
                self.board.set_project_name(&id, draft);
            }
            Editing::TaskTitle { id } => {
                self.board.set_task_title(&id, draft);
            }
            Editing::TaskDue { id } => {
                if let Some(parsed) = parse_date(&draft, Self::today()) {
                    self.board.set_task_due(&id, parsed);
                }
            }
        }
        if self.board != before {
            self.history.push(self.board.clone());
        }
        self.editing = Editing::None;
    }

    fn commit_edit(&mut self, _: &CommitEdit, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.editing, Editing::None) {
            return;
        }
        self.commit_current(cx);
        self.focus_app(window);
        cx.notify();
    }

    fn cancel_edit(&mut self, _: &CancelEdit, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.editing, Editing::None) {
            return;
        }
        self.editing = Editing::None;
        self.focus_app(window);
        cx.notify();
    }

    fn on_click_away(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.editing, Editing::None) {
            return;
        }
        if self.input.read(cx).contains_point(event.position) {
            return;
        }
        self.commit_current(cx);
        self.focus_app(window);
        cx.notify();
    }
}

impl Focusable for StatusApp {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for StatusApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_window_title("Weekly Status Board");
        let theme = theme::for_mode(self.theme_dark);
        let status: SharedString = self.status.clone().into();
        let zoom_label = self.zoom_label();
        let today = Self::today();
        let app = cx.entity();
        let input = self.input.clone();
        let editing = self.editing.clone();
        let selection = self.selection.clone();
        let view_mode = self.view_mode;

        div()
            .id("status-app-root")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family("Segoe UI")
            .font_weight(FontWeight::NORMAL)
            .on_action(cx.listener(Self::commit_edit))
            .on_action(cx.listener(Self::cancel_edit))
            .on_action(cx.listener(Self::delete_selected))
            .on_action(cx.listener(Self::move_selected_left))
            .on_action(cx.listener(Self::move_selected_right))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_click_away))
            .child(header::Header(&theme, zoom_label))
            .child(board::Board(
                &self.board,
                &theme,
                today,
                view_mode,
                &editing,
                &selection,
                input,
                app,
            ))
            .child(footer::Footer(&theme, status))
    }
}
