use crate::history::History;
use crate::model::{BoardDocument, Column};
use crate::theme;
use crate::ui::{board, footer, header};
use chrono::NaiveDate;
use gpui::{div, prelude::*, Context, FontWeight, SharedString, Window};

pub struct StatusApp {
    pub board: BoardDocument,
    pub history: History,
    pub view_mode: bool,
    pub status: String,
    pub theme_dark: bool,
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
    pub fn new() -> Self {
        let board = demo_board();
        let history = History::new(board.clone());
        Self {
            board,
            history,
            view_mode: false,
            status: String::new(),
            theme_dark: false,
        }
    }

    fn zoom_label(&self) -> SharedString {
        let pct = (self.board.zoom * 100.0).round() as i32;
        format!("{pct}%").into()
    }
}

impl Default for StatusApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for StatusApp {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        window.set_window_title("Weekly Status Board");
        let theme = theme::for_mode(self.theme_dark);
        let status: SharedString = self.status.clone().into();
        let zoom_label = self.zoom_label();
        let today = chrono::Local::now().date_naive();
        let _ = self.view_mode;

        div()
            .id("status-app-root")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family("Segoe UI")
            .font_weight(FontWeight::NORMAL)
            .child(header::Header(&theme, zoom_label))
            .child(board::Board(&self.board, &theme, today))
            .child(footer::Footer(&theme, status))
    }
}
