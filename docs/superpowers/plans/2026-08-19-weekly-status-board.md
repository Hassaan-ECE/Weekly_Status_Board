# Weekly Status Board Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a personal Windows GPUI app that edits a project-grouped Target / In Progress / Done board and pastes a PNG into PowerPoint, with a module layout later apps can copy.

**Architecture:** Pure Rust domain (`model`, `dates`, `history`, `persistence`, `zoom`) with no GPUI types. GPUI sits in `theme` + `ui` + `export`. First two tasks are gates: a window must open on this laptop, then a PNG must come out of that window. If either fails, stop and do not build the board on a dead renderer.

**Tech Stack:** Rust 2021, `gpui 0.2.2`, serde/serde_json, uuid, chrono, image, Windows capture fallback (`windows` + `raw-window-handle` if GPUI has no screenshot API). No Tauri, no React.

**Spec:** `docs/superpowers/specs/2026-08-19-weekly-status-board-design.md`

**Reference rule:** Keep `theme`, window shell, persistence, and PNG export independent of board cards so a later GPUI app can copy those files.

---

## File map

Create these (do not dump board logic into `main.rs`):

| Path | Responsibility |
|---|---|
| `Cargo.toml` | Crate `weekly-status-board`, pinned deps |
| `src/lib.rs` | Module tree; tests use this crate |
| `src/main.rs` | `Application::run` only |
| `src/theme.rs` | Inventory light/dark tokens as GPUI colors |
| `src/model.rs` | `BoardDocument` and mutations |
| `src/dates.rs` | `M/D` parse/display + date-roll |
| `src/history.rs` | In-session undo/redo of `BoardDocument` |
| `src/persistence.rs` | `.board.json`, draft, session |
| `src/zoom.rs` | Clamp 0.25–1.50, step 0.10 |
| `src/export.rs` | Board region → PNG bytes; clipboard; file write |
| `src/ui/mod.rs` | UI module |
| `src/ui/app.rs` | Root view: header, board, footer, actions |
| `src/ui/header.rs` | Toolbar |
| `src/ui/footer.rs` | Status + `Built by Syed Hassaan Shah` |
| `src/ui/board.rs` | Three columns, widths, view-mode, zoom |
| `src/ui/column.rs` | Project sections, add controls |
| `src/ui/card.rs` | Title + pinned date chip |
| `src/ui/dialogs.rs` | Meeting confirm, delete-section confirm, native file prompts |
| `tests/model.rs` | Move, meeting-clear, widths |
| `tests/dates.rs` | Roll + parse/display |
| `tests/history.rs` | Undo restores Done |
| `tests/persistence.rs` | JSON round-trip, bad version |
| `tests/zoom.rs` | Clamp/step |
| `AGENTS.md` | How to run, GPUI Windows notes, what to copy next time |

---

### Task 1: GPUI window on this laptop (gate)

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: Create the crate files**

`Cargo.toml`:

```toml
[package]
name = "weekly-status-board"
version = "0.1.0"
edition = "2021"
authors = ["Syed Hassaan Shah"]
description = "Personal weekly status board (GPUI)"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", default-features = false, features = ["clock", "std", "serde"] }
gpui = "0.2.2"
image = "0.25"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }

[target.'cfg(windows)'.dependencies]
raw-window-handle = "0.6"
windows = { version = "0.61", features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
    "Win32_UI_WindowsAndMessaging",
] }
```

`src/lib.rs`:

```rust
pub mod dates;
pub mod history;
pub mod model;
pub mod persistence;
pub mod zoom;
```

Create stub files `src/dates.rs`, `src/history.rs`, `src/model.rs`, `src/persistence.rs`, `src/zoom.rs` each containing `// stub` so `cargo run` works. Replace the stubs in later tasks.

`src/main.rs` (official gpui.rs hello world, retitled):

```rust
use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        window.set_window_title("Weekly Status Board");
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .size_full()
            .justify_center()
            .items_center()
            .text_color(rgb(0x262626))
            .child(format!("Weekly Status Board — {}", &self.text))
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
```

If `TitlebarOptions` fields differ in 0.2.2, open `docs.rs/gpui/0.2.2` and match the struct. Do not invent extra window flags.

- [ ] **Step 2: Build**

Run from `C:\Projects\Active\Weekly_Status_Board`:

```
cargo build
```

Expected: compile success. First GPUI build can take several minutes.

If compile fails on Windows (missing DirectX, feature flags, `gpui` docs claiming macOS/Linux only): **stop**. Record the exact rustc error in `docs/superpowers/specs/2026-08-19-weekly-status-board-design.md` under a “Verified 2026-08-19” note. Do not start the board UI.

- [ ] **Step 3: Run the window**

```
cargo run
```

Expected: a 1100×700 window titled Weekly Status Board with white background and the hello text.

- [ ] **Step 4: Commit**

```
git add Cargo.toml Cargo.lock src
git commit -m "Open a GPUI window on Windows."
```

---

### Task 2: PNG capture proof (gate)

**Files:**
- Create: `src/export.rs`
- Modify: `src/lib.rs` (add `pub mod export;`)
- Modify: `src/main.rs` (button or key that writes `capture-proof.png`)

- [ ] **Step 1: Inspect gpui 0.2.2 for a screenshot API**

Search local cargo registry / docs for `screenshot`, `capture`, `RenderImage`, `ClipboardItem` image constructors on `Window` / `App`.

If a first-party capture exists, wrap it in `export::capture_window_png(window, scale: u32) -> anyhow::Result<Vec<u8>>`.

If not, implement Win32 fallback in `src/export.rs` (Windows-only):

```rust
use anyhow::{bail, Context, Result};
use image::{ImageBuffer, RgbaImage};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::io::Cursor;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
    ReleaseDC, SelectObject, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    GetDIBits,
};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

pub fn png_from_hwnd(hwnd: HWND, scale: u32) -> Result<Vec<u8>> {
    let _ = scale; // v1 captures 1× client pixels; Task 16 may request a 2× path if GPUI scale_factor is available
    unsafe {
        let mut rect = windows::Win32::Foundation::RECT::default();
        GetClientRect(hwnd, &mut rect).ok().context("GetClientRect")?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            bail!("capture bounds empty: {width}x{height}");
        }
        let hdc = GetDC(hwnd);
        if hdc.is_invalid() {
            bail!("GetDC failed");
        }
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, width, height);
        let old = SelectObject(mem, bmp);
        BitBlt(mem, 0, 0, width, height, hdc, 0, 0, SRCCOPY).ok().context("BitBlt")?;

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let got = GetDIBits(mem, bmp, 0, height as u32, Some(pixels.as_mut_ptr().cast()), &mut info, DIB_RGB_COLORS);
        SelectObject(mem, old);
        DeleteObject(bmp).ok()?;
        DeleteDC(mem).ok()?;
        ReleaseDC(hwnd, hdc);
        if got == 0 {
            bail!("GetDIBits failed");
        }
        // BGRA -> RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
        let img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, pixels)
            .context("ImageBuffer")?;
        let mut out = Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png)?;
        Ok(out.into_inner())
    }
}

pub fn write_png_file(path: &std::path::Path, bytes: &[u8]) -> Result<()> {
    if path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("png")) != Some(true) {
        bail!("destination must end in .png");
    }
    std::fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}
```

Obtaining `HWND`: after the window exists, get a platform handle from GPUI if exported. If `Window` does not expose `raw_window_handle`, use `GetForegroundWindow` **only for this proof** (document that Task 16 must capture the **board region**, not the whole foreground window).

Wire a click on the hello text (or a “Save PNG” child `div().on_click`) that writes `capture-proof.png` next to the exe / cwd and prints the path.

- [ ] **Step 2: Run and verify the file**

```
cargo run
```

Click the control. Then:

```
powershell -Command "(Get-Item capture-proof.png).Length"
```

Expected: file exists, size > 1000 bytes, opens as a real image of the window. If BitBlt is black/empty, try `PrintWindow` next. If still failing: **stop**. Do not implement the board.

- [ ] **Step 3: Commit**

```
git add src/export.rs src/lib.rs src/main.rs
git commit -m "Prove PNG capture from the GPUI window."
```

Delete `capture-proof.png` from the repo (it is gitignored if you add `*.png` except docs; do not commit the screenshot).

---

### Task 3: Theme tokens (copyable)

**Files:**
- Create: `src/theme.rs`
- Modify: `src/lib.rs` (`pub mod theme;`)
- Modify: `src/main.rs` to use `theme::light()`

- [ ] **Step 1: Add `src/theme.rs`**

```rust
use gpui::{rgb, Rgba};

#[derive(Clone, Copy)]
pub struct Theme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub card: Rgba,
    pub border: Rgba,
    pub muted: Rgba,
    pub fill_4: Rgba,
    pub primary: Rgba,
    pub primary_fg: Rgba,
    pub target_header: Rgba,
    pub progress_header: Rgba,
    pub done_header: Rgba,
    pub header_fg: Rgba,
}

pub fn light() -> Theme {
    Theme {
        background: rgb(0xffffff),
        foreground: rgb(0x262626),
        card: rgb(0xffffff),
        border: Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.08 },
        muted: rgb(0x686868),
        fill_4: Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.04 },
        primary: rgb(0x5558E6),
        primary_fg: rgb(0xffffff),
        target_header: rgb(0x5558E6),
        progress_header: rgb(0xB45309),
        done_header: rgb(0x047857),
        header_fg: rgb(0xffffff),
    }
}

pub fn dark() -> Theme {
    Theme {
        background: rgb(0x0C0C0C),
        foreground: rgb(0xF5F5F5),
        card: rgb(0x101010),
        border: Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.06 },
        muted: rgb(0x7E7E7E),
        fill_4: Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.04 },
        primary: rgb(0x7A7DF0),
        primary_fg: rgb(0xffffff),
        target_header: rgb(0x5558E6),
        progress_header: rgb(0xB45309),
        done_header: rgb(0x047857),
        header_fg: rgb(0xffffff),
    }
}

pub fn for_mode(dark_mode: bool) -> Theme {
    if dark_mode { dark() } else { light() }
}
```

If `Rgba { r,g,b,a }` is not public that way in 0.2.2, use `rgba(0x00000014)` or `hsla` from gpui. Match the crate.

- [ ] **Step 2: Use `light()` as the hello background**

Replace `rgb(0xffffff)` in `main.rs` with `theme::light().background` and text with `.foreground`.

- [ ] **Step 3: Commit**

```
git add src/theme.rs src/lib.rs src/main.rs
git commit -m "Add Inventory-style GPUI theme tokens."
```

---

### Task 4: Board model (TDD)

**Files:**
- Replace: `src/model.rs`
- Create: `tests/model.rs`

- [ ] **Step 1: Write failing tests**

`tests/model.rs`:

```rust
use chrono::NaiveDate;
use weekly_status_board::model::{BoardDocument, Column};

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn move_creates_destination_section_and_keeps_empty_source() {
    let mut board = BoardDocument::empty();
    let (project_id, task_id) = board.add_project(Column::Target);
    board.set_task_title(&task_id, "Ventilation");
    board.set_task_due(&task_id, day(2026, 8, 20));
    board.move_task(&task_id, Column::InProgress, &project_id);

    assert_eq!(board.task(&task_id).unwrap().column, Column::InProgress);
    assert!(board.has_section(&project_id, Column::Target));
    assert!(board.has_section(&project_id, Column::InProgress));
    assert!(board.tasks_in(&project_id, Column::Target).is_empty());
}

#[test]
fn drop_on_other_project_reparents() {
    let mut board = BoardDocument::empty();
    let (a, task_id) = board.add_project(Column::Target);
    let (b, _) = board.add_project(Column::InProgress);
    board.move_task(&task_id, Column::InProgress, &b);
    let task = board.task(&task_id).unwrap();
    assert_eq!(task.project_id, b);
    assert_eq!(task.column, Column::InProgress);
    assert!(board.has_section(&a, Column::Target));
}

#[test]
fn clear_done_leaves_other_columns() {
    let mut board = BoardDocument::empty();
    let (p, t1) = board.add_project(Column::Target);
    let t2 = board.add_task(&p, Column::Done);
    board.set_task_title(&t2, "Repaired units");
    board.clear_done();
    assert!(board.task(&t2).is_none());
    assert!(board.task(&t1).is_some());
    assert!(board.has_section(&p, Column::Done));
}

#[test]
fn widths_normalize_and_clamp() {
    let mut board = BoardDocument::empty();
    board.set_column_widths([2.0, 1.0, 1.0]);
    let w = board.column_widths;
    let sum: f32 = w.iter().sum();
    assert!((sum - 1.0).abs() < 1e-5);
    assert!(w.iter().all(|x| *x >= 0.18 - 1e-5));
}

#[test]
fn delete_empty_section_drops_project_when_last() {
    let mut board = BoardDocument::empty();
    let (p, t) = board.add_project(Column::Target);
    board.delete_task(&t);
    board.delete_section(&p, Column::Target);
    assert!(board.projects.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

```
cargo test --test model
```

Expected: FAIL (module missing types / methods).

- [ ] **Step 3: Implement `src/model.rs`**

```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MIN_COLUMN_FRACTION: f32 = 0.18;
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Column {
    Target,
    InProgress,
    Done,
}

impl Column {
    pub fn all() -> [Column; 3] {
        [Column::Target, Column::InProgress, Column::Done]
    }

    pub fn index(self) -> usize {
        match self {
            Column::Target => 0,
            Column::InProgress => 1,
            Column::Done => 2,
        }
    }

    pub fn right(self) -> Option<Column> {
        match self {
            Column::Target => Some(Column::InProgress),
            Column::InProgress => Some(Column::Done),
            Column::Done => None,
        }
    }

    pub fn left(self) -> Option<Column> {
        match self {
            Column::Target => None,
            Column::InProgress => Some(Column::Target),
            Column::Done => Some(Column::InProgress),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    pub project_id: String,
    pub column: Column,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub column: Column,
    pub title: String,
    pub due: NaiveDate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardDocument {
    pub version: u32,
    pub title: String,
    pub theme: ThemeMode,
    pub zoom: f32,
    pub column_widths: [f32; 3],
    pub projects: Vec<Project>,
    pub sections: Vec<Section>,
    pub tasks: Vec<Task>,
}

impl BoardDocument {
    pub fn empty() -> Self {
        Self {
            version: SCHEMA_VERSION,
            title: "Weekly Status Board".into(),
            theme: ThemeMode::Light,
            zoom: 1.0,
            column_widths: [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
            projects: Vec::new(),
            sections: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn is_blank(&self) -> bool {
        self.projects.is_empty() && self.tasks.is_empty() && self.sections.is_empty()
    }

    fn new_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn add_project(&mut self, column: Column) -> (String, String) {
        let project_id = Self::new_id();
        self.projects.push(Project {
            id: project_id.clone(),
            name: "New project".into(),
        });
        self.ensure_section(&project_id, column);
        let task_id = self.add_task(&project_id, column);
        (project_id, task_id)
    }

    pub fn add_task(&mut self, project_id: &str, column: Column) -> String {
        self.ensure_section(project_id, column);
        let id = Self::new_id();
        self.tasks.push(Task {
            id: id.clone(),
            project_id: project_id.to_string(),
            column,
            title: String::new(),
            due: chrono::Local::now().date_naive(),
        });
        id
    }

    pub fn ensure_section(&mut self, project_id: &str, column: Column) {
        if !self.has_section(project_id, column) {
            self.sections.push(Section {
                project_id: project_id.to_string(),
                column,
            });
        }
    }

    pub fn has_section(&self, project_id: &str, column: Column) -> bool {
        self.sections.iter().any(|s| s.project_id == project_id && s.column == column)
    }

    pub fn task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn set_task_title(&mut self, id: &str, title: impl Into<String>) {
        if let Some(t) = self.task_mut(id) {
            let next = title.into();
            if !next.trim().is_empty() {
                t.title = next;
            }
        }
    }

    pub fn set_task_due(&mut self, id: &str, due: NaiveDate) {
        if let Some(t) = self.task_mut(id) {
            t.due = due;
        }
    }

    pub fn set_project_name(&mut self, id: &str, name: impl Into<String>) {
        if let Some(p) = self.projects.iter_mut().find(|p| p.id == id) {
            let next = name.into();
            if !next.trim().is_empty() {
                p.name = next;
            }
        }
    }

    pub fn tasks_in(&self, project_id: &str, column: Column) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.project_id == project_id && t.column == column)
            .collect()
    }

    pub fn move_task(&mut self, task_id: &str, dest_column: Column, dest_project_id: &str) {
        let Some(idx) = self.tasks.iter().position(|t| t.id == task_id) else { return };
        self.tasks[idx].column = dest_column;
        self.tasks[idx].project_id = dest_project_id.to_string();
        self.ensure_section(dest_project_id, dest_column);
        let task = self.tasks.remove(idx);
        self.tasks.push(task);
    }

    pub fn move_task_side(&mut self, task_id: &str, dest: Column) {
        let Some(project_id) = self.task(task_id).map(|t| t.project_id.clone()) else { return };
        self.move_task(task_id, dest, &project_id);
    }

    pub fn delete_task(&mut self, task_id: &str) {
        self.tasks.retain(|t| t.id != task_id);
    }

    pub fn delete_section(&mut self, project_id: &str, column: Column) {
        self.tasks.retain(|t| !(t.project_id == project_id && t.column == column));
        self.sections.retain(|s| !(s.project_id == project_id && s.column == column));
        if !self.sections.iter().any(|s| s.project_id == project_id) {
            self.projects.retain(|p| p.id != project_id);
        }
    }

    pub fn clear_done(&mut self) {
        self.tasks.retain(|t| t.column != Column::Done);
    }

    pub fn set_column_widths(&mut self, widths: [f32; 3]) {
        let mut w = widths.map(|x| x.max(0.0));
        let mut sum: f32 = w.iter().sum();
        if sum <= f32::EPSILON {
            w = [1.0 / 3.0; 3];
            sum = 1.0;
        }
        w = w.map(|x| x / sum);
        for i in 0..3 {
            if w[i] < MIN_COLUMN_FRACTION {
                w[i] = MIN_COLUMN_FRACTION;
            }
        }
        let sum: f32 = w.iter().sum();
        self.column_widths = w.map(|x| x / sum);
        let mut guard = 0;
        while self.column_widths.iter().any(|x| *x < MIN_COLUMN_FRACTION - 1e-4) && guard < 8 {
            guard += 1;
            let mut w = self.column_widths;
            for i in 0..3 {
                if w[i] < MIN_COLUMN_FRACTION {
                    w[i] = MIN_COLUMN_FRACTION;
                }
            }
            let sum: f32 = w.iter().sum();
            self.column_widths = w.map(|x| x / sum);
        }
    }
}
```

- [ ] **Step 4: Run tests**

```
cargo test --test model
```

Expected: PASS.

- [ ] **Step 5: Commit**

```
git add src/model.rs tests/model.rs
git commit -m "Add board document model and mutation tests."
```

---

### Task 5: Dates (TDD)

**Files:**
- Replace: `src/dates.rs`
- Create: `tests/dates.rs`

- [ ] **Step 1: Write failing tests**

```rust
use chrono::NaiveDate;
use weekly_status_board::dates::{apply_date_roll, display_date, parse_date};
use weekly_status_board::model::{BoardDocument, Column};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

#[test]
fn roll_past_open_work_to_today_not_done_not_future() {
    let mut board = BoardDocument::empty();
    let (p, t_past) = board.add_project(Column::Target);
    board.set_task_due(&t_past, d(2026, 8, 1));
    let t_future = board.add_task(&p, Column::InProgress);
    board.set_task_due(&t_future, d(2026, 12, 1));
    let t_done = board.add_task(&p, Column::Done);
    board.set_task_due(&t_done, d(2026, 8, 12));
    let today = d(2026, 8, 19);
    let changed = apply_date_roll(&mut board, today);
    assert!(changed);
    assert_eq!(board.task(&t_past).unwrap().due, today);
    assert_eq!(board.task(&t_future).unwrap().due, d(2026, 12, 1));
    assert_eq!(board.task(&t_done).unwrap().due, d(2026, 8, 12));
}

#[test]
fn closed_week_jumps_to_today() {
    let mut board = BoardDocument::empty();
    let (_, t) = board.add_project(Column::InProgress);
    board.set_task_due(&t, d(2026, 8, 10));
    apply_date_roll(&mut board, d(2026, 8, 19));
    assert_eq!(board.task(&t).unwrap().due, d(2026, 8, 19));
}

#[test]
fn parse_and_display_current_year_omits_year() {
    let today = d(2026, 8, 19);
    assert_eq!(parse_date("8/21", today).unwrap(), d(2026, 8, 21));
    assert_eq!(parse_date("8/21/2025", today).unwrap(), d(2025, 8, 21));
    assert_eq!(display_date(d(2026, 8, 21), today), "8/21");
    assert_eq!(display_date(d(2025, 8, 21), today), "8/21/2025");
}
```

- [ ] **Step 2: Run to see FAIL**

```
cargo test --test dates
```

- [ ] **Step 3: Implement `src/dates.rs`**

```rust
use chrono::NaiveDate;
use crate::model::{BoardDocument, Column};

pub fn apply_date_roll(board: &mut BoardDocument, today: NaiveDate) -> bool {
    let mut changed = false;
    for task in &mut board.tasks {
        if matches!(task.column, Column::Target | Column::InProgress) && task.due < today {
            task.due = today;
            changed = true;
        }
    }
    changed
}

pub fn parse_date(input: &str, today: NaiveDate) -> Option<NaiveDate> {
    let s = input.trim();
    let parts: Vec<&str> = s.split('/').collect();
    match parts.as_slice() {
        [m, d] => NaiveDate::from_ymd_opt(today.year(), m.parse().ok()?, d.parse().ok()?),
        [m, d, y] => NaiveDate::from_ymd_opt(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?),
        _ => None,
    }
}

pub fn display_date(date: NaiveDate, today: NaiveDate) -> String {
    if date.year() == today.year() {
        format!("{}/{}", date.month(), date.day())
    } else {
        format!("{}/{}/{}", date.month(), date.day(), date.year())
    }
}

use chrono::Datelike;
```

Put `use chrono::Datelike;` at the top with the other uses.

- [ ] **Step 4: `cargo test --test dates` — PASS**

- [ ] **Step 5: Commit**

```
git add src/dates.rs tests/dates.rs
git commit -m "Add date display, parse, and overdue roll."
```

---

### Task 6: Undo history (TDD)

**Files:**
- Replace: `src/history.rs`
- Create: `tests/history.rs`

- [ ] **Step 1: Failing test**

```rust
use weekly_status_board::history::History;
use weekly_status_board::model::{BoardDocument, Column};

#[test]
fn undo_restores_cleared_done() {
    let mut board = BoardDocument::empty();
    let (p, _) = board.add_project(Column::Done);
    board.set_project_name(&p, "IRHX");
    let mut history = History::new(board.clone());
    board.clear_done();
    history.push(board.clone());
    assert!(board.tasks.is_empty());
    board = history.undo(board).expect("undo");
    assert_eq!(board.tasks.len(), 1);
    board = history.redo(board).expect("redo");
    assert!(board.tasks.is_empty());
}
```

- [ ] **Step 2: FAIL with `cargo test --test history`**

- [ ] **Step 3: Implement**

```rust
use crate::model::BoardDocument;

pub struct History {
    undo: Vec<BoardDocument>,
    redo: Vec<BoardDocument>,
}

impl History {
    pub fn new(current: BoardDocument) -> Self {
        Self { undo: vec![current], redo: Vec::new() }
    }

    pub fn push(&mut self, current: BoardDocument) {
        self.undo.push(current);
        self.redo.clear();
    }

    pub fn undo(&mut self, current: BoardDocument) -> Option<BoardDocument> {
        if self.undo.len() < 2 {
            return None;
        }
        self.redo.push(current);
        self.undo.pop();
        self.undo.last().cloned()
    }

    pub fn redo(&mut self, current: BoardDocument) -> Option<BoardDocument> {
        let next = self.redo.pop()?;
        self.undo.push(next.clone());
        let _ = current;
        Some(next)
    }
}
```

`push` must be called **after** a committed edit, with the new document. `new` stores the baseline so the first undo returns to it. Adjust the test if the stack model is “undo pops current then previous”: keep one consistent rule — **stack of snapshots; undo = previous snapshot**.

- [ ] **Step 4: PASS `cargo test --test history`**

- [ ] **Step 5: Commit** `Add in-session board undo and redo.`

---

### Task 7: Persistence (TDD)

**Files:**
- Replace: `src/persistence.rs`
- Create: `tests/persistence.rs`

- [ ] **Step 1: Failing tests**

```rust
use weekly_status_board::model::{BoardDocument, Column, SCHEMA_VERSION};
use weekly_status_board::persistence::{load_board, save_board};

#[test]
fn round_trip_pretty_json() {
    let dir = tempfile_dir();
    let path = dir.join("demo.board.json");
    let mut board = BoardDocument::empty();
    let (p, t) = board.add_project(Column::Target);
    board.set_project_name(&p, "HVDC");
    board.set_task_title(&t, "SOW");
    save_board(&path, &board).unwrap();
    let loaded = load_board(&path).unwrap();
    assert_eq!(loaded.version, SCHEMA_VERSION);
    assert_eq!(loaded.projects[0].name, "HVDC");
    assert_eq!(loaded.tasks[0].title, "SOW");
}

#[test]
fn rejects_unknown_version_without_writing() {
    let dir = tempfile_dir();
    let path = dir.join("bad.board.json");
    std::fs::write(&path, r#"{"version":99,"title":"x","theme":"light","zoom":1.0,"column_widths":[0.3,0.3,0.4],"projects":[],"sections":[],"tasks":[]}"#).unwrap();
    assert!(load_board(&path).is_err());
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("\"version\":99"));
}

fn tempfile_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wsb-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
```

- [ ] **Step 2: FAIL `cargo test --test persistence`**

- [ ] **Step 3: Implement `src/persistence.rs`**

```rust
use crate::model::{BoardDocument, SCHEMA_VERSION};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub active_path: Option<PathBuf>,
}

pub fn save_board(path: &Path, board: &BoardDocument) -> Result<()> {
    let text = serde_json::to_string_pretty(board)?;
    std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
}

pub fn load_board(path: &Path) -> Result<BoardDocument> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let board: BoardDocument = serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if board.version != SCHEMA_VERSION {
        bail!("unsupported board version {} in {}", board.version, path.display());
    }
    Ok(board)
}

pub fn app_data_dir() -> Result<PathBuf> {
    let base = std::env::var_os("APPDATA").context("APPDATA not set")?;
    let dir = PathBuf::from(base).join("WeeklyStatusBoard");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn draft_path() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("draft.board.json"))
}

pub fn session_path() -> Result<PathBuf> {
    Ok(app_data_dir()?.join("session.json"))
}

pub fn load_session() -> Result<Session> {
    let path = session_path()?;
    if !path.exists() {
        return Ok(Session { active_path: None });
    }
    let text = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn save_session(session: &Session) -> Result<()> {
    std::fs::write(session_path()?, serde_json::to_string_pretty(session)?)?;
    Ok(())
}
```

- [ ] **Step 4: PASS `cargo test --test persistence`**

- [ ] **Step 5: Commit** `Save and load portable .board.json files.`

---

### Task 8: Zoom clamp (TDD)

**Files:**
- Replace: `src/zoom.rs`
- Create: `tests/zoom.rs`

- [ ] **Step 1: Test**

```rust
use weekly_status_board::zoom::{clamp_zoom, step_zoom, MIN_ZOOM, MAX_ZOOM};

#[test]
fn clamp_and_step() {
    assert_eq!(clamp_zoom(0.0), MIN_ZOOM);
    assert_eq!(clamp_zoom(9.0), MAX_ZOOM);
    assert!((step_zoom(1.0, 1) - 1.1).abs() < 1e-6);
    assert!((step_zoom(1.0, -1) - 0.9).abs() < 1e-6);
    assert_eq!(step_zoom(0.25, -1), MIN_ZOOM);
}
```

- [ ] **Step 2: FAIL then implement**

```rust
pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 1.50;
pub const ZOOM_STEP: f32 = 0.10;

pub fn clamp_zoom(zoom: f32) -> f32 {
    ((zoom * 10.0).round() / 10.0).clamp(MIN_ZOOM, MAX_ZOOM)
}

pub fn step_zoom(zoom: f32, dir: i32) -> f32 {
    clamp_zoom(zoom + dir as f32 * ZOOM_STEP)
}
```

- [ ] **Step 3: PASS and commit** `Add Gantt-style zoom clamp and steps.`

---

### Task 9: App shell (header / board slot / footer)

**Files:**
- Create: `src/ui/mod.rs`, `src/ui/app.rs`, `src/ui/header.rs`, `src/ui/footer.rs`, `src/ui/board.rs`
- Modify: `src/lib.rs` (`pub mod ui;`)
- Replace: `src/main.rs` to open `ui::app::StatusApp`

- [ ] **Step 1: Root view layout**

`src/ui/mod.rs`:

```rust
pub mod app;
pub mod board;
pub mod footer;
pub mod header;
```

`StatusApp` holds `board: BoardDocument`, `history: History`, `view_mode: bool`, `status: String`, `theme_dark: bool`.

Render a full-size column:

1. `header::Header`
2. `board::Board` (`flex_1`, this is the export region — give it `id("board-workspace")`)
3. `footer::Footer` with status on the left and **Built by Syed Hassaan Shah** on the right, `font_weight(500)`

Window min size: 1100×560. Default 1100×700. Title `Weekly Status Board`. Background `theme.background`. Text `DM Sans` if you can register a font from a bundled file; otherwise Segoe UI (system). If bundling DM Sans, put files under `assets/fonts/` and load in `main` via gpui font APIs. Do not block the shell on fonts.

Header buttons (even if no-ops until later tasks): New, Open, Save, Copy image, Export PNG, View, Dark Theme, `−` / `100%` / `+`, **Attended weekly meeting** (primary fill `theme.primary`).

- [ ] **Step 2: `cargo run` — shell visible, credit in footer**

- [ ] **Step 3: Commit** `Add GPUI app shell with Inventory chrome.`

---

### Task 10: Read-only board from model

**Files:**
- Create: `src/ui/column.rs`, `src/ui/card.rs`
- Modify: `src/ui/board.rs`

- [ ] **Step 1: Seed one demo board in `StatusApp::new` for visual check, loaded from `BoardDocument`**

Render three columns using `board.column_widths`. Each column: colored header (Target indigo, In Progress `#B45309`, Done `#047857`), then for each `project` in `projects` order, if `has_section(project, column)`, draw:

- Project bar: bold, `#111` / dark fg, 12px, 4% fill, **no left edge line**
- Cards: row, title wraps left, date chip right 46px, bold black, vertically centered

Column gap 6px. Empty section still shows the project bar.

- [ ] **Step 2: Run and match the v5 mockup (minus + controls)**

- [ ] **Step 3: Commit** `Render project-grouped board columns from the model.`

---

### Task 11: Add + in-place edit

**Files:**
- Modify: `src/ui/column.rs`, `src/ui/card.rs`, `src/ui/app.rs`

- [ ] **Step 1: Add controls (hidden when `view_mode`)**

- `+` on project row → `board.add_task(project_id, column)`, push history, focus new title
- `+ Add project` at column bottom → `board.add_project(column)`, focus project name

In-place edit: click project name / title / date chip. Enter commits, Escape reverts. Blank name/title restores previous (`set_project_name` / `set_task_title` already ignore blank). Date uses `parse_date`; invalid restores previous.

GPUI text input: start from `crates.io` gpui 0.2.2 `examples/input.rs` pattern (copy the **minimum** single-line field into `src/ui/input.rs` if there is no built-in). Do not take a 700-line editor if a one-line `div` + `on_key_down` + `EntityInputHandler` works. If `gpui-component` 0.2 compiles against `gpui 0.2.2` and gives `Input`, using it is allowed — pin the version in `Cargo.toml` and note it in `AGENTS.md`.

- [ ] **Step 2: Manual: add HVDC + a task, rename, set date 8/21**

- [ ] **Step 3: Commit** `Add in-place editing and new project/task controls.`

---

### Task 12: Move and delete

**Files:**
- Modify: `src/ui/card.rs`, `src/ui/app.rs`

- [ ] **Step 1: Selection + keys**

Selected card: `Delete` / `Backspace` → `delete_task` (no confirm).  
Selected project header: `Delete` → if section has tasks, confirm via `cx.prompt` / `dialogs.rs`; then `delete_section`.

Move right / left: `]` / `[` or buttons on the selected card calling `move_task_side`. Drag: `on_drag` the task id, `on_drop` on a project section (reparent + column) or column body (same project, that column). Append to destination section (`move_task` already appends). Do not implement intra-section reorder.

Push history on every committed move/delete.

- [ ] **Step 2: Manual: move Target → In Progress → Done; empty heading remains**

- [ ] **Step 3: Commit** `Move and delete cards and project sections.`

---

### Task 13: View, zoom, column drag

**Files:**
- Modify: `src/ui/header.rs`, `src/ui/board.rs`, `src/ui/app.rs`

- [ ] **Step 1: Wire controls**

- View toggle sets `view_mode`. When true, hide `+`, Add project, gutters.
- Zoom: header `−` / percent / `+`. Actions `ZoomIn`, `ZoomOut`, `ResetZoom`. Bind `ctrl--`, `ctrl-=`, `ctrl-0`. Percent click resets to 1.0. Store `board.zoom = clamp_zoom(...)`. Apply with `window.set_rem_size` **only on the board child** (`with_rem_size`) so header/footer stay readable.
- Gutters: drag changes `column_widths` via `set_column_widths`. Min 0.18.

- [ ] **Step 2: Manual: View hides pluses; zoom 90%; widen Target**

- [ ] **Step 3: Commit** `Add view mode, zoom, and column resize.`

---

### Task 14: Files, autosave, date-roll on load

**Files:**
- Modify: `src/ui/app.rs`, `src/ui/dialogs.rs`, `src/persistence.rs`

- [ ] **Step 1: New / Open / Save / Save As**

Use `App::prompt_for_paths` / `PathPromptOptions` from gpui 0.2.2 (open vs save). Filter `*.board.json`.

On launch: `load_session`; if `active_path` exists load it; else `draft.board.json` if present; else `empty()`. Then `apply_date_roll(..., Local::now().date_naive())`. If roll changed and a path exists, `save_board`.

After named Open/Save As: autosave on each committed edit. Unnamed: write draft. Save with no path = Save As. New: unnamed non-blank → Save As prompt; named → autosave then `empty()`.

Corrupt file: set `status` to the **parse/OS error**, keep empty in-memory board, **do not write** the bad file.

- [ ] **Step 2: Manual: save, quit, reopen; past due became today**

- [ ] **Step 3: Commit** `Persist boards and roll overdue dates on load.`

---

### Task 15: Meeting clear + undo keys

**Files:**
- Modify: `src/ui/app.rs`, `src/ui/dialogs.rs`

- [ ] **Step 1: Primary button confirms**

Message: `Clear all Done tasks? This cannot be undone after you quit.`  
Buttons: Cancel, Clear. On Clear: `history.push` then `clear_done`.

`Ctrl+Z` / `Ctrl+Y` call `history.undo` / `redo`.

- [ ] **Step 2: Manual: clear Done, undo restores**

- [ ] **Step 3: Commit** `Clear Done after the weekly meeting, with undo.`

---

### Task 16: Copy image / Export PNG of the board

**Files:**
- Modify: `src/export.rs`, `src/ui/app.rs`

- [ ] **Step 1: Capture the board workspace only**

Record board bounds during prepaint (`id("board-workspace")` layout bounds). Capture that rectangle (not header/footer). Force view-mode look for the frame (hide pluses even if `view_mode` is false). Raster: logical size × `window.scale_factor()` (and extra ×2 if scale_factor is already 1 and the spec’s 2× is still needed). Shared bytes for copy + export.

Copy: `cx.write_to_clipboard` with `ClipboardItem` image if gpui 0.2.2 supports image clipboard; otherwise CF_DIB / PNG via Windows clipboard APIs. If only Unicode text is supported, use Win32 clipboard PNG (`RegisterClipboardFormat("PNG")`) and put the real error in the footer on failure.

Export: save dialog, `.png` only, `write_png_file`.

Empty bounds → footer `board is not laid out yet`. Other failures → `path + OS/capture error`, never “export failed”.

- [ ] **Step 2: Manual: Copy image, paste into PowerPoint. Export PNG and open it. Confirm no toolbar, no pluses.**

- [ ] **Step 3: Commit** `Export the board workspace as a PowerPoint-ready PNG.`

---

### Task 17: Dark theme + AGENTS.md (reference)

**Files:**
- Modify: `src/ui/app.rs`, `src/theme.rs`
- Create: `AGENTS.md`
- Modify: `README.md`

- [ ] **Step 1: Theme toggle persists on `board.theme`**

Button label Dark Theme / Light Theme. Re-render from `theme::for_mode`.

- [ ] **Step 2: Write `AGENTS.md`** so the next GPUI app can copy this repo:

```markdown
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
PNG comes from [describe the path that actually worked in Task 2/16].
```

Fill the Capture section with the **real** method (GPUI API vs BitBlt).

- [ ] **Step 3: `cargo test` all, `cargo clippy --all-targets -- -D warnings` if feasible**

- [ ] **Step 4: Commit** `Add dark theme and GPUI reference notes.`

- [ ] **Step 5: Push**

```
git push origin main
```

---

## Spec coverage

| Spec item | Task |
|---|---|
| GPUI window on this laptop | 1 |
| PNG capture gate | 2, 16 |
| Inventory theme + credit | 3, 9, 17 |
| Target / In Progress / Done + project groups | 10 |
| Title + date chip | 10 |
| In-place edit, add project/task | 11 |
| Move / delete | 12 |
| View, zoom, column drag | 13 |
| JSON, draft, session, date-roll | 5, 7, 14 |
| Meeting clear + undo | 6, 15 |
| Copy/Export board only, 2×, real errors | 16 |
| Dark theme | 17 |
| Copyable GPUI reference | file map + 17 |
| No Gantt merge / no pptx / no sharing | omitted |

## Stop conditions

- Task 1 window does not open → stop.
- Task 2 PNG is not a real picture of the window → stop.
- Do not implement Tasks 9–16 on a renderer that cannot export.
