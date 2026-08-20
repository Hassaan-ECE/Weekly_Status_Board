# Weekly Status Board

Windows GPUI app. Spec: `docs/superpowers/specs/2026-08-19-weekly-status-board-design.md`.

## Run
`cargo run` from this directory. First GPUI compile is slow.

## Test
`cargo test`

## What to copy into a new GPUI app
- `src/theme.rs` — Inventory tokens
- `src/ui/app.rs` header/footer shell
- `src/persistence.rs` + `src/export.rs` — JSON + PNG
- Do not copy `model`/`dates` unless you are making another board

## Capture
GPUI 0.2.2 has no public window-screenshot API. PNG capture is Win32:

1. Get HWND via `raw_window_handle::HasWindowHandle` on the GPUI `Window`.
2. Capture the full window with **PrintWindow(PW_RENDERFULLCONTENT)** — BitBlt is blank on this DComp window.
3. Crop to the board workspace using GetWindowRect + ClientToScreen titlebar offset (`nc_top` / `nc_left`) plus board bounds × `scale_factor`.
4. Copy: Win32 clipboard with registered `PNG` **and** `CF_DIB` (PowerPoint/Ctrl+V). GPUI's image clipboard is PNG-only and does not paste in Office.
5. Export: save dialog + `write_png_file`.

See `src/export.rs` (`capture_board_png`).
