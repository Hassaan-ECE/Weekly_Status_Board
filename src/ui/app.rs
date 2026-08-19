use crate::dates::{display_date, parse_date};
use crate::history::History;
use crate::model::{BoardDocument, Column};
use crate::theme;
use crate::ui::input::{self, TextInput};
use crate::ui::{board, footer, header};
use chrono::NaiveDate;
use gpui::{
    actions, div, prelude::*, Context, Entity, FontWeight, KeyBinding, MouseButton, MouseDownEvent,
    SharedString, Window,
};

actions!(board_edit, [CommitEdit, CancelEdit]);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Editing {
    None,
    ProjectName { id: String, column: Column },
    TaskTitle { id: String },
    TaskDue { id: String },
}

pub struct StatusApp {
    pub board: BoardDocument,
    pub history: History,
    pub view_mode: bool,
    pub status: String,
    pub theme_dark: bool,
    pub editing: Editing,
    pub input: Entity<TextInput>,
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
            input,
        }
    }

    pub fn bind_edit_keys(cx: &mut gpui::App) {
        input::bind_input_keys(cx);
        cx.bind_keys([
            KeyBinding::new("enter", CommitEdit, Some("TextInput")),
            KeyBinding::new("escape", CancelEdit, Some("TextInput")),
        ]);
    }

    fn zoom_label(&self) -> SharedString {
        let pct = (self.board.zoom * 100.0).round() as i32;
        format!("{pct}%").into()
    }

    fn today() -> NaiveDate {
        chrono::Local::now().date_naive()
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
        window.blur();
        cx.notify();
    }

    fn cancel_edit(&mut self, _: &CancelEdit, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.editing, Editing::None) {
            return;
        }
        self.editing = Editing::None;
        window.blur();
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
        window.blur();
        cx.notify();
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
        let view_mode = self.view_mode;

        div()
            .id("status-app-root")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family("Segoe UI")
            .font_weight(FontWeight::NORMAL)
            .on_action(cx.listener(Self::commit_edit))
            .on_action(cx.listener(Self::cancel_edit))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_click_away))
            .child(header::Header(&theme, zoom_label))
            .child(board::Board(
                &self.board,
                &theme,
                today,
                view_mode,
                &editing,
                input,
                app,
            ))
            .child(footer::Footer(&theme, status))
    }
}
