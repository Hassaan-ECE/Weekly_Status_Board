use crate::history::History;
use crate::model::BoardDocument;
use crate::theme;
use crate::ui::{board, footer, header};
use gpui::{div, prelude::*, Context, FontWeight, SharedString, Window};

pub struct StatusApp {
    pub board: BoardDocument,
    pub history: History,
    pub view_mode: bool,
    pub status: String,
    pub theme_dark: bool,
}

impl StatusApp {
    pub fn new() -> Self {
        let board = BoardDocument::empty();
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
            .child(board::Board(&theme))
            .child(footer::Footer(&theme, status))
    }
}
