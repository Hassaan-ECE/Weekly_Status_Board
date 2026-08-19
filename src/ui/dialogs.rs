use gpui::{App, PathPromptOptions, PromptLevel, Window};
use std::path::{Path, PathBuf};

/// Confirm deleting a project section that still has tasks.
/// Resolves to `Ok(1)` when the user chooses Delete (index 1); `Ok(0)` for Cancel.
pub fn confirm_delete_section(
    window: &mut Window,
    cx: &mut gpui::App,
) -> impl std::future::Future<Output = Result<usize, ()>> {
    let rx = window.prompt(
        PromptLevel::Warning,
        "Delete this project section?",
        Some("All tasks in this section will be removed."),
        &["Cancel", "Delete"],
        cx,
    );
    async move { rx.await.map_err(|_| ()) }
}

/// Open a board file.
///
/// gpui 0.2.2 `PathPromptOptions` has no file-type filter, so any file can be chosen.
pub fn prompt_open_board(
    cx: &mut App,
) -> impl std::future::Future<Output = Result<anyhow::Result<Option<Vec<PathBuf>>>, ()>> {
    let rx = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Open".into()),
    });
    async move { rx.await.map_err(|_| ()) }
}

/// Save As dialog. Suggested name is `board.board.json`.
///
/// gpui 0.2.2 save dialog has no file-type filter; append `.board.json` in the caller if needed.
pub fn prompt_save_board(
    directory: &Path,
    cx: &mut App,
) -> impl std::future::Future<Output = Result<anyhow::Result<Option<PathBuf>>, ()>> {
    let rx = cx.prompt_for_new_path(directory, Some("board.board.json"));
    async move { rx.await.map_err(|_| ()) }
}
