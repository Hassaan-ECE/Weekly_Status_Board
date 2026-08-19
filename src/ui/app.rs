use crate::dates::{apply_date_roll, display_date, parse_date};
use crate::history::History;
use crate::model::{BoardDocument, Column};
use crate::persistence::{
    draft_path, ensure_board_json_suffix, load_board, load_startup_board, save_board, save_session,
    Session,
};
use crate::theme;
use crate::ui::dialogs;
use crate::ui::input::{self, TextInput};
use crate::ui::{board, footer, header};
use crate::zoom::{clamp_zoom, step_zoom};
use chrono::NaiveDate;
use gpui::{
    actions, div, prelude::*, px, rgb, Bounds, Context, Entity, FocusHandle, Focusable, FontWeight,
    KeyBinding, MouseButton, MouseDownEvent, Pixels, Point, SharedString, Window,
};
use std::path::PathBuf;

actions!(
    board_edit,
    [
        CommitEdit,
        CancelEdit,
        DeleteSelected,
        MoveSelectedLeft,
        MoveSelectedRight,
        ZoomIn,
        ZoomOut,
        ResetZoom,
        SaveAs,
        Undo,
        Redo
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
    pub active_path: Option<PathBuf>,
    pub view_mode: bool,
    pub status: String,
    pub theme_dark: bool,
    pub editing: Editing,
    pub selection: Selection,
    pub input: Entity<TextInput>,
    board_bounds: Option<Bounds<Pixels>>,
    resizing_gutter: Option<usize>,
    focus_handle: FocusHandle,
}

impl StatusApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let startup = load_startup_board();
        let mut board = startup.board;
        let mut status = startup.status;
        let active_path = startup.active_path;
        let today = chrono::Local::now().date_naive();
        if apply_date_roll(&mut board, today) {
            if let Some(path) = startup.persist_path.as_ref() {
                if let Err(err) = save_board(path, &board) {
                    status = format!("{err:#}");
                }
            }
        }
        let history = History::new(board.clone());
        let input = cx.new(|cx| TextInput::new(cx, "", ""));
        Self {
            board,
            history,
            active_path,
            view_mode: false,
            status,
            theme_dark: false,
            editing: Editing::None,
            selection: Selection::None,
            input,
            board_bounds: None,
            resizing_gutter: None,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn bind_edit_keys(cx: &mut gpui::App) {
        input::bind_input_keys(cx);
        cx.bind_keys([
            KeyBinding::new("enter", CommitEdit, Some("TextInput")),
            KeyBinding::new("escape", CancelEdit, Some("TextInput")),
            KeyBinding::new("delete", DeleteSelected, Some("StatusApp")),
            KeyBinding::new("backspace", DeleteSelected, Some("StatusApp")),
            KeyBinding::new("[", MoveSelectedLeft, Some("StatusApp")),
            KeyBinding::new("]", MoveSelectedRight, Some("StatusApp")),
            KeyBinding::new("ctrl--", ZoomOut, Some("StatusApp")),
            KeyBinding::new("ctrl-=", ZoomIn, Some("StatusApp")),
            KeyBinding::new("ctrl-0", ResetZoom, Some("StatusApp")),
            KeyBinding::new("ctrl-shift-s", SaveAs, Some("StatusApp")),
            KeyBinding::new("ctrl-z", Undo, Some("StatusApp")),
            KeyBinding::new("ctrl-y", Redo, Some("StatusApp")),
        ]);
    }

    fn zoom_label(&self) -> SharedString {
        let pct = (self.board.zoom * 100.0).round() as i32;
        format!("{pct}%").into()
    }

    fn persist_path(&self) -> Result<PathBuf, String> {
        if let Some(path) = &self.active_path {
            Ok(path.clone())
        } else {
            draft_path().map_err(|e| format!("{e:#}"))
        }
    }

    fn persist_now(&mut self) {
        match self.persist_path() {
            Ok(path) => {
                if let Err(err) = save_board(&path, &self.board) {
                    self.status = format!("{err:#}");
                }
            }
            Err(err) => self.status = err,
        }
    }

    fn save_dir(&self) -> PathBuf {
        if let Some(path) = &self.active_path {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    return parent.to_path_buf();
                }
            }
        }
        if let Some(home) = std::env::var_os("USERPROFILE") {
            let docs = PathBuf::from(home).join("Documents");
            if docs.is_dir() {
                return docs;
            }
        }
        crate::persistence::app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    fn remember_active_path(&mut self, path: PathBuf) {
        self.active_path = Some(path.clone());
        if let Err(err) = save_session(&Session {
            active_path: Some(path),
        }) {
            self.status = format!("{err:#}");
        }
    }

    fn clear_active_path_session(&mut self) {
        self.active_path = None;
        if let Err(err) = save_session(&Session { active_path: None }) {
            self.status = format!("{err:#}");
        }
    }

    fn reset_to_empty_unnamed(&mut self, cx: &mut Context<Self>) {
        self.board = BoardDocument::empty();
        self.history = History::new(self.board.clone());
        self.editing = Editing::None;
        self.selection = Selection::None;
        self.clear_active_path_session();
        if let Ok(draft) = draft_path() {
            let _ = save_board(&draft, &self.board);
        }
        self.status.clear();
        cx.notify();
    }

    fn apply_loaded_board(&mut self, path: PathBuf, mut board: BoardDocument, cx: &mut Context<Self>) {
        let changed = apply_date_roll(&mut board, Self::today());
        if changed {
            if let Err(err) = save_board(&path, &board) {
                self.status = format!("{err:#}");
            } else {
                self.status.clear();
            }
        } else {
            self.status.clear();
        }
        self.board = board;
        self.history = History::new(self.board.clone());
        self.editing = Editing::None;
        self.selection = Selection::None;
        self.remember_active_path(path);
        cx.notify();
    }

    fn load_opened_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match load_board(&path) {
            Ok(board) => self.apply_loaded_board(path, board, cx),
            Err(err) => {
                self.status = format!("{err:#}");
                self.board = BoardDocument::empty();
                self.history = History::new(self.board.clone());
                self.editing = Editing::None;
                self.selection = Selection::None;
                self.active_path = None;
                cx.notify();
            }
        }
    }

    fn write_to_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let path = ensure_board_json_suffix(path);
        match save_board(&path, &self.board) {
            Ok(()) => {
                self.status.clear();
                self.remember_active_path(path);
                cx.notify();
            }
            Err(err) => {
                self.status = format!("{err:#}");
                cx.notify();
            }
        }
    }

    pub fn new_board(&mut self, cx: &mut Context<Self>) {
        if self.active_path.is_none() && !self.board.is_blank() {
            self.save_as(cx, true);
            return;
        }
        if self.active_path.is_some() {
            self.persist_now();
        }
        self.reset_to_empty_unnamed(cx);
    }

    pub fn open_board(&mut self, cx: &mut Context<Self>) {
        let prompt = dialogs::prompt_open_board(cx);
        cx.spawn(async move |this, cx| {
            let outcome = prompt.await;
            this.update(cx, |app, cx| match outcome {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        app.load_opened_path(path, cx);
                    }
                }
                Ok(Ok(None)) => {}
                Ok(Err(err)) => {
                    app.status = format!("{err:#}");
                    cx.notify();
                }
                Err(()) => {}
            })
            .ok();
        })
        .detach();
    }

    pub fn save_board_action(&mut self, cx: &mut Context<Self>) {
        if self.active_path.is_some() {
            self.persist_now();
            cx.notify();
        } else {
            self.save_as(cx, false);
        }
    }

    pub fn save_as(&mut self, cx: &mut Context<Self>, then_new: bool) {
        let dir = self.save_dir();
        let prompt = dialogs::prompt_save_board(&dir, cx);
        cx.spawn(async move |this, cx| {
            let outcome = prompt.await;
            this.update(cx, |app, cx| match outcome {
                Ok(Ok(Some(path))) => {
                    app.write_to_path(path, cx);
                    if then_new && app.active_path.is_some() {
                        app.reset_to_empty_unnamed(cx);
                    }
                }
                Ok(Ok(None)) => {}
                Ok(Err(err)) => {
                    app.status = format!("{err:#}");
                    cx.notify();
                }
                Err(()) => {}
            })
            .ok();
        })
        .detach();
    }

    fn on_save_as(&mut self, _: &SaveAs, _: &mut Window, cx: &mut Context<Self>) {
        self.save_as(cx, false);
    }

    pub fn clear_done_after_meeting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let answer = dialogs::confirm_clear_done(window, cx);
        cx.spawn(async move |this, cx| {
            if answer.await.ok() != Some(1) {
                return;
            }
            this.update(cx, |app, cx| {
                app.board.clear_done();
                app.history.push(app.board.clone());
                app.persist_now();
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn on_undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        let Some(prev) = self.history.undo(self.board.clone()) else {
            return;
        };
        self.board = prev;
        self.editing = Editing::None;
        self.persist_now();
        cx.notify();
    }

    fn on_redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        let Some(next) = self.history.redo(self.board.clone()) else {
            return;
        };
        self.board = next;
        self.editing = Editing::None;
        self.persist_now();
        cx.notify();
    }

    pub fn toggle_view_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = !self.view_mode;
        if self.view_mode {
            self.resizing_gutter = None;
            if !matches!(self.editing, Editing::None) {
                self.editing = Editing::None;
                self.focus_app(window);
            }
        }
        cx.notify();
    }

    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.board.zoom = step_zoom(self.board.zoom, 1);
        self.persist_now();
        cx.notify();
    }

    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.board.zoom = step_zoom(self.board.zoom, -1);
        self.persist_now();
        cx.notify();
    }

    pub fn reset_zoom(&mut self, cx: &mut Context<Self>) {
        self.board.zoom = clamp_zoom(1.0);
        self.persist_now();
        cx.notify();
    }

    fn on_zoom_in(&mut self, _: &ZoomIn, _: &mut Window, cx: &mut Context<Self>) {
        self.zoom_in(cx);
    }

    fn on_zoom_out(&mut self, _: &ZoomOut, _: &mut Window, cx: &mut Context<Self>) {
        self.zoom_out(cx);
    }

    fn on_reset_zoom(&mut self, _: &ResetZoom, _: &mut Window, cx: &mut Context<Self>) {
        self.reset_zoom(cx);
    }

    pub fn set_board_bounds(&mut self, bounds: Bounds<Pixels>) {
        self.board_bounds = Some(bounds);
    }

    pub fn begin_column_resize(
        &mut self,
        gutter: usize,
        _position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode || gutter > 1 {
            return;
        }
        self.resizing_gutter = Some(gutter);
        cx.notify();
    }

    pub fn on_column_resize_move(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(gutter) = self.resizing_gutter else {
            return;
        };
        let Some(bounds) = self.board_bounds else {
            return;
        };
        let width = f32::from(bounds.size.width);
        if width <= f32::EPSILON {
            return;
        }
        let rel = ((f32::from(position.x) - f32::from(bounds.origin.x)) / width).clamp(0.0, 1.0);
        let mut widths = self.board.column_widths;
        match gutter {
            0 => {
                let fixed = widths[2];
                let available = (1.0 - fixed).max(0.0);
                let left = rel.clamp(0.18, (available - 0.18).max(0.18));
                widths[0] = left;
                widths[1] = (available - left).max(0.18);
            }
            1 => {
                let fixed = widths[0];
                let boundary = rel.clamp(fixed + 0.18, 1.0 - 0.18);
                widths[1] = (boundary - fixed).max(0.18);
                widths[2] = (1.0 - fixed - widths[1]).max(0.18);
            }
            _ => return,
        }
        self.board.set_column_widths(widths);
        cx.notify();
    }

    pub fn end_column_resize(&mut self, cx: &mut Context<Self>) {
        if self.resizing_gutter.take().is_some() {
            self.persist_now();
            cx.notify();
        }
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
        self.persist_now();
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
        self.persist_now();
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
        self.persist_now();
        cx.notify();
    }

    pub fn move_task_to(
        &mut self,
        task_id: &str,
        dest_column: Column,
        dest_project_id: &str,
        cx: &mut Context<Self>,
    ) {
        if self.view_mode || !matches!(self.editing, Editing::None) {
            return;
        }
        let Some(task) = self.board.task(task_id) else {
            return;
        };
        if task.column == dest_column && task.project_id == dest_project_id {
            return;
        }
        self.board
            .move_task(task_id, dest_column, dest_project_id);
        self.history.push(self.board.clone());
        self.persist_now();
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
                self.persist_now();
                self.selection = Selection::None;
                cx.notify();
            }
            Selection::Section { project_id, column } => {
                let has_tasks = !self.board.tasks_in(&project_id, column).is_empty();
                if !has_tasks {
                    self.board.delete_section(&project_id, column);
                    self.history.push(self.board.clone());
                    self.persist_now();
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
                        app.persist_now();
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
            self.persist_now();
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
            .key_context("StatusApp")
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
            .on_action(cx.listener(Self::on_zoom_in))
            .on_action(cx.listener(Self::on_zoom_out))
            .on_action(cx.listener(Self::on_reset_zoom))
            .on_action(cx.listener(Self::on_save_as))
            .on_action(cx.listener(Self::on_undo))
            .on_action(cx.listener(Self::on_redo))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_click_away))
            .child(header::Header(&theme, zoom_label, view_mode, app.clone()))
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
