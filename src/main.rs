use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};
use std::path::PathBuf;
use weekly_status_board::theme;

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        window.set_window_title("Weekly Status Board");
        let theme = theme::light();
        div()
            .id("root-capture")
            .flex()
            .flex_col()
            .bg(theme.background)
            .size_full()
            .justify_center()
            .items_center()
            .gap_4()
            .text_color(theme.foreground)
            .cursor_pointer()
            .on_click(|_event, window, _cx| {
                let path = PathBuf::from("capture-proof.png");
                match weekly_status_board::export::capture_window_png(window, 1) {
                    Ok(bytes) => match weekly_status_board::export::write_png_file(&path, &bytes) {
                        Ok(()) => {
                            let abs = std::env::current_dir()
                                .map(|cwd| cwd.join(&path))
                                .unwrap_or_else(|_| path.clone());
                            eprintln!("wrote PNG {} ({} bytes)", abs.display(), bytes.len());
                        }
                        Err(err) => eprintln!("write PNG failed: {err:#}"),
                    },
                    Err(err) => eprintln!("capture PNG failed: {err:#}"),
                }
            })
            .child(format!("Weekly Status Board — {}", &self.text))
            .child(
                div()
                    .id("save-png")
                    .px_4()
                    .py_2()
                    .bg(theme.primary)
                    .text_color(theme.primary_fg)
                    .rounded_md()
                    .child("Save PNG"),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.), px(700.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Weekly Status Board".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "GPUI".into(),
                })
            },
        )
        .unwrap();
    });
}
