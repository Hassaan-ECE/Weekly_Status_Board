//! Window icon. GPUI 0.2.2 has no WindowOptions.icon; it loads resource ID 1
//! via LoadImageW. We also apply WM_SETICON from the embedded ICO so cargo run
//! shows the icon even when winres/rc.exe did not embed a resource.

#[cfg(windows)]
pub fn apply_window_icon(window: &impl raw_window_handle::HasWindowHandle) {
    if let Err(err) = apply_window_icon_inner(window) {
        eprintln!("WSB_ICON: {err:#}");
    }
}

#[cfg(not(windows))]
pub fn apply_window_icon<W>(_window: &W) {}

#[cfg(windows)]
fn apply_window_icon_inner(
    window: &impl raw_window_handle::HasWindowHandle,
) -> anyhow::Result<()> {
    use crate::export::hwnd_from_window;
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        LoadImageW, SendMessageW, SetClassLongPtrW, GCLP_HICON, GCLP_HICONSM, ICON_BIG, ICON_SMALL,
        IMAGE_ICON, LR_LOADFROMFILE, WM_SETICON,
    };

    let hwnd = hwnd_from_window(window)?;
    let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/app-icon.ico"));
    let path = std::env::temp_dir().join("weekly-status-board-app-icon.ico");
    std::fs::write(&path, bytes)?;
    let name = HSTRING::from(path.to_string_lossy().as_ref());

    unsafe {
        let big = LoadImageW(None, &name, IMAGE_ICON, 256, 256, LR_LOADFROMFILE)?;
        let small = LoadImageW(None, &name, IMAGE_ICON, 16, 16, LR_LOADFROMFILE)?;
        SendMessageW(
            hwnd,
            WM_SETICON,
            Some(WPARAM(ICON_BIG as usize)),
            Some(LPARAM(big.0 as isize)),
        );
        SendMessageW(
            hwnd,
            WM_SETICON,
            Some(WPARAM(ICON_SMALL as usize)),
            Some(LPARAM(small.0 as isize)),
        );
        SetClassLongPtrW(hwnd, GCLP_HICON, big.0 as isize);
        SetClassLongPtrW(hwnd, GCLP_HICONSM, small.0 as isize);
    }
    Ok(())
}
