use gpui::{px, size, prelude::*, App, Application, Bounds, WindowBounds, WindowOptions};
use weekly_status_board::ui::app::StatusApp;

fn main() {
    Application::new().run(|cx: &mut App| {
        StatusApp::bind_edit_keys(cx);
        let bounds = Bounds::centered(None, size(px(1100.), px(700.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(1100.), px(560.))),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Weekly Status Board".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| StatusApp::new(cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
