//! Window PNG capture for the Task 2 proof gate.
//!
//! GPUI 0.2.2 has no public window-screenshot API (`screenshot` / pixel readback
//! on `Window`). Capture uses Win32 GDI (BitBlt, with PrintWindow fallback).
//! Prefer `HasWindowHandle` from the focused GPUI `Window` (available in click
//! handlers). Task 16 must capture the board region, not rely on foreground-window
//! heuristics.

use anyhow::{bail, Context, Result};
use std::io::Cursor;
use std::path::Path;

pub fn write_png_file(path: &Path, bytes: &[u8]) -> Result<()> {
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("png"))
        != Some(true)
    {
        bail!("destination must end in .png");
    }
    std::fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

#[cfg(windows)]
mod win {
    use super::*;
    use image::{ImageBuffer, RgbaImage};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
        SRCCOPY,
    };
    use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    /// PW_RENDERFULLCONTENT (undocumented in older SDKs; value 2).
    const PW_RENDERFULLCONTENT: PRINT_WINDOW_FLAGS = PRINT_WINDOW_FLAGS(2);

    pub fn hwnd_from_window(window: &impl HasWindowHandle) -> Result<HWND> {
        let handle = window
            .window_handle()
            .map_err(|err| anyhow::anyhow!("Window::window_handle: {err}"))?;
        match handle.as_raw() {
            RawWindowHandle::Win32(h) => Ok(HWND(h.hwnd.get() as *mut _)),
            other => bail!("expected Win32 window handle, got {other:?}"),
        }
    }

    fn pixels_look_blank(pixels: &[u8]) -> bool {
        if pixels.is_empty() {
            return true;
        }
        // Sample every 16th pixel; treat near-black RGBA as blank.
        let mut checked = 0u32;
        let mut dark = 0u32;
        for chunk in pixels.chunks_exact(4).step_by(16) {
            checked += 1;
            let (r, g, b, a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
            if a < 8 || (r < 8 && g < 8 && b < 8) {
                dark += 1;
            }
        }
        checked > 0 && dark * 100 / checked >= 98
    }

    fn bgra_to_rgba(pixels: &mut [u8]) {
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
    }

    fn encode_png(width: i32, height: i32, pixels: Vec<u8>) -> Result<Vec<u8>> {
        let img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, pixels)
            .context("ImageBuffer::from_raw")?;
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png)
            .context("encode PNG")?;
        Ok(out.into_inner())
    }

    unsafe fn capture_dib(hwnd: HWND, width: i32, height: i32, use_print_window: bool) -> Result<Vec<u8>> {
        let hdc = GetDC(Some(hwnd));
        if hdc.is_invalid() {
            bail!("GetDC failed");
        }
        let mem = CreateCompatibleDC(Some(hdc));
        if mem.is_invalid() {
            ReleaseDC(Some(hwnd), hdc);
            bail!("CreateCompatibleDC failed");
        }
        let bmp = CreateCompatibleBitmap(hdc, width, height);
        if bmp.is_invalid() {
            let _ = DeleteDC(mem);
            ReleaseDC(Some(hwnd), hdc);
            bail!("CreateCompatibleBitmap failed");
        }
        let old = SelectObject(mem, bmp.into());

        let blit_ok = if use_print_window {
            PrintWindow(hwnd, mem, PW_RENDERFULLCONTENT).as_bool()
        } else {
            BitBlt(mem, 0, 0, width, height, Some(hdc), 0, 0, SRCCOPY).is_ok()
        };
        if !blit_ok {
            SelectObject(mem, old);
            let _ = DeleteObject(bmp.into());
            let _ = DeleteDC(mem);
            ReleaseDC(Some(hwnd), hdc);
            if use_print_window {
                bail!("PrintWindow failed");
            }
            bail!("BitBlt failed");
        }

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let got = GetDIBits(
            mem,
            bmp,
            0,
            height as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        SelectObject(mem, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(Some(hwnd), hdc);
        if got == 0 {
            bail!("GetDIBits failed");
        }
        bgra_to_rgba(&mut pixels);
        Ok(pixels)
    }

    pub fn png_from_hwnd(hwnd: HWND, scale: u32) -> Result<Vec<u8>> {
        let _ = scale; // v1 captures 1× client pixels; Task 16 may request a 2× path
        if hwnd.is_invalid() {
            bail!("HWND is null");
        }
        unsafe {
            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect).context("GetClientRect")?;
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            if width <= 0 || height <= 0 {
                bail!("capture bounds empty: {width}x{height}");
            }

            let mut pixels = capture_dib(hwnd, width, height, false)
                .with_context(|| format!("BitBlt capture HWND={:?}", hwnd.0))?;
            if pixels_look_blank(&pixels) {
                eprintln!(
                    "BitBlt capture looked blank; retrying PrintWindow(PW_RENDERFULLCONTENT)"
                );
                pixels = capture_dib(hwnd, width, height, true)
                    .with_context(|| format!("PrintWindow capture HWND={:?}", hwnd.0))?;
                if pixels_look_blank(&pixels) {
                    bail!(
                        "capture still blank/black after BitBlt and PrintWindow ({}x{})",
                        width,
                        height
                    );
                }
            }
            encode_png(width, height, pixels)
        }
    }

    pub fn capture_window_png(window: &impl HasWindowHandle, scale: u32) -> Result<Vec<u8>> {
        let hwnd = hwnd_from_window(window)?;
        png_from_hwnd(hwnd, scale)
    }
}

#[cfg(windows)]
pub use win::{capture_window_png, hwnd_from_window, png_from_hwnd};

#[cfg(not(windows))]
pub fn capture_window_png<W>(_window: &W, _scale: u32) -> Result<Vec<u8>> {
    bail!("PNG capture is only implemented on Windows")
}
