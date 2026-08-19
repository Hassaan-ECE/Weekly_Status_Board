//! Board-workspace PNG capture for Copy image / Export PNG.
//!
//! GPUI 0.2.2 has no public window-screenshot API. Capture uses Win32
//! PrintWindow(PW_RENDERFULLCONTENT) via HWND from `HasWindowHandle` (BitBlt is
//! blank on this DComp window). Task 17 AGENTS.md should note this path.

use anyhow::{bail, Context, Result};
use std::io::Cursor;
use std::path::Path;

/// Logical board-workspace rect in window client space (GPUI `Bounds<Pixels>`).
#[derive(Clone, Copy, Debug)]
pub struct BoardRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

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

pub fn ensure_png_suffix(path: std::path::PathBuf) -> std::path::PathBuf {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("png") => path,
        _ => {
            let mut s = path.into_os_string();
            s.push(".png");
            std::path::PathBuf::from(s)
        }
    }
}

#[cfg(windows)]
mod win {
    use super::*;
    use image::{imageops::FilterType, DynamicImage, ImageBuffer, RgbaImage};
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
        let _ = scale; // physical client pixels; densify happens in capture_board_png
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

            // DComp / GPUI windows often fail or blank BitBlt; PrintWindow is the real path.
            let pixels = match capture_dib(hwnd, width, height, false) {
                Ok(p) if !pixels_look_blank(&p) => p,
                Ok(_) => {
                    eprintln!(
                        "BitBlt capture looked blank; retrying PrintWindow(PW_RENDERFULLCONTENT)"
                    );
                    capture_dib(hwnd, width, height, true)
                        .with_context(|| format!("PrintWindow capture HWND={:?}", hwnd.0))?
                }
                Err(err) => {
                    eprintln!(
                        "BitBlt capture failed ({err:#}); retrying PrintWindow(PW_RENDERFULLCONTENT)"
                    );
                    capture_dib(hwnd, width, height, true)
                        .with_context(|| format!("PrintWindow capture HWND={:?}", hwnd.0))?
                }
            };
            if pixels_look_blank(&pixels) {
                bail!(
                    "capture still blank/black after BitBlt and PrintWindow ({}x{})",
                    width,
                    height
                );
            }
            encode_png(width, height, pixels)
        }
    }

    pub fn capture_window_png(window: &impl HasWindowHandle, scale: u32) -> Result<Vec<u8>> {
        let hwnd = hwnd_from_window(window)?;
        png_from_hwnd(hwnd, scale)
    }

    /// Capture full client via PrintWindow path, then crop to the board workspace.
    ///
    /// `board` is in logical/client pixels; multiplied by `scale_factor` for the
    /// physical crop. When `scale_factor < 1.5`, the crop is upscaled ×2 so the
    /// export still meets the spec’s 2× density on 100% DPI displays.
    pub fn capture_board_png(
        window: &impl HasWindowHandle,
        board: BoardRect,
        scale_factor: f32,
    ) -> Result<Vec<u8>> {
        if board.width <= 0.0 || board.height <= 0.0 {
            bail!("board is not laid out yet");
        }

        let full_png = capture_window_png(window, 1)
            .with_context(|| "capture window for board crop")?;
        let full = image::load_from_memory(&full_png)
            .context("decode captured PNG")?
            .to_rgba8();
        let img_w = full.width();
        let img_h = full.height();

        let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };

        let mut x0 = (board.x * scale).round() as i64;
        let mut y0 = (board.y * scale).round() as i64;
        let mut x1 = ((board.x + board.width) * scale).round() as i64;
        let mut y1 = ((board.y + board.height) * scale).round() as i64;

        x0 = x0.clamp(0, img_w as i64);
        y0 = y0.clamp(0, img_h as i64);
        x1 = x1.clamp(0, img_w as i64);
        y1 = y1.clamp(0, img_h as i64);

        if x1 <= x0 || y1 <= y0 {
            bail!(
                "empty board crop after clamp: logical=({:.1},{:.1},{:.1}x{:.1}) scale={scale} image={img_w}x{img_h} phys=({x0},{y0})-({x1},{y1})",
                board.x,
                board.y,
                board.width,
                board.height
            );
        }

        let crop_w = (x1 - x0) as u32;
        let crop_h = (y1 - y0) as u32;
        let cropped = image::imageops::crop_imm(&full, x0 as u32, y0 as u32, crop_w, crop_h)
            .to_image();

        // Spec wants ~2× density; at >=1.5 DPI the physical crop is already dense enough.
        let out_img: RgbaImage = if scale < 1.5 {
            image::imageops::resize(
                &cropped,
                crop_w.saturating_mul(2).max(1),
                crop_h.saturating_mul(2).max(1),
                FilterType::CatmullRom,
            )
        } else {
            cropped
        };

        let mut out = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(out_img)
            .write_to(&mut out, image::ImageFormat::Png)
            .context("encode cropped board PNG")?;
        Ok(out.into_inner())
    }
}

#[cfg(windows)]
pub use win::{capture_board_png, capture_window_png, hwnd_from_window, png_from_hwnd};

#[cfg(not(windows))]
pub fn capture_window_png<W>(_window: &W, _scale: u32) -> Result<Vec<u8>> {
    bail!("PNG capture is only implemented on Windows")
}

#[cfg(not(windows))]
pub fn capture_board_png<W>(
    _window: &W,
    _board: BoardRect,
    _scale_factor: f32,
) -> Result<Vec<u8>> {
    bail!("PNG capture is only implemented on Windows")
}
