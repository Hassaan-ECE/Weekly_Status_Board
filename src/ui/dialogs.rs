use gpui::{PromptLevel, Window};

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
