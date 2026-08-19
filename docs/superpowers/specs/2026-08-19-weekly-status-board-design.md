# Weekly Status Board Design

**Date:** 2026-08-19  
**Status:** Draft for user review  
**Product:** Weekly Status Board  
**Location:** `C:\Projects\Active\Weekly_Status_Board`

## Summary

A personal Windows desktop app for a standing, project-grouped task board. Syed edits work on this laptop, then pastes a PNG into PowerPoint for the weekly manager update. The UI toolkit is **GPUI** (learning it vs Tauri/egui). Visual chrome follows **Inventory Management**. File/export habits follow **Gantt Chart Creator**.

Cards move Target → In Progress → Done. Due dates on open work never display as late: if a Target or In Progress date is before today, it becomes today. **Attended weekly meeting** clears Done after confirm.

## Goals

- Install and run on this Windows laptop as a native GPUI app.
- Three columns: **Target**, **In Progress**, **Done**, grouped by project.
- Each task is one line of text plus a due date.
- In-place edit of project names, task text, and dates.
- Add a project in a column; add a task under a project in that column.
- Move tasks between columns (drag, plus move-left / move-right).
- Drag column widths; zoom 25%–150% like Gantt (`Ctrl+-`, `Ctrl++`, `Ctrl+0`).
- **View** hides add controls so the board matches the PNG. Copy/Export always use that clean look.
- **Copy image** and **Export PNG** of the board workspace only, on-screen size, 2× pixel density.
- Portable JSON board files with New / Open / Save / Save As and autosave.
- In-session undo/redo (`Ctrl+Z` / `Ctrl+Y`).
- Light/dark theme from Inventory tokens. Footer: **Built by Syed Hassaan Shah**.

## Non-goals (v1)

- Multi-user / shared board / sharing with the team.
- Absorbing or rewriting Gantt Chart Creator.
- Native PowerPoint (`.pptx`) output.
- Extra task fields (notes, owner, priority, links).
- Showing lateness (“X days late”) or keeping the original missed date.
- Week-archive documents. The board is standing; Done is cleared after the meeting.
- Linux/macOS packaging.

## Window

- Title: `Weekly Status Board`.
- Default size: **1100×700**, centered. Minimum **1100×560**. User may grow it.
- Layout, top to bottom:
  1. Header toolbar (title + actions).
  2. Board workspace (three columns) — this is the export region.
  3. Footer status strip + credit.

## Visual language

Copy Inventory Management, not the original PowerPoint’s full-width navy/gold/green body.

| Token | Light | Dark |
|---|---|---|
| Font | DM Sans 400/500/600/700, then Segoe UI / system-ui | same |
| Background | `#FFFFFF` | ≈ `#0C0C0C` |
| Foreground | `#262626` | `#F5F5F5` |
| Border | black @ 8% | white @ 6% |
| Primary | `oklch(0.488 0.217 264)` ≈ `#5558E6` | `oklch(0.588 0.217 264)` |
| Radius | 10px surfaces; 8px cards/buttons | same |
| Credit | `Built by Syed Hassaan Shah` | same |

**Column headers** are the only loud color (full-width bar, white label):

| Column | Header fill (light) |
|---|---|
| Target | primary indigo `#5558E6` |
| In Progress | warning foreground `#B45309` |
| Done | success foreground `#047857` |

**Project headers:** bold black (`#111` light / `#F5F5F5` dark), 12px, on a quiet 4% fill bar. No left accent edge. Same in every column.

**Task cards:** white (or dark card) surface, 1px border, 8px radius. Text on the left (wraps). **Due date** is a fixed-width chip on the right, vertically centered, bold black, 4% fill. Chip does not move when the title is long.

**Column gap:** 6px. User-draggable gutters between columns.

**Add affordances (edit mode only):** `+` on the project row (add task). `+ Add project` dashed control at the bottom of the column.

**View mode:** hide `+`, `+ Add project`, and resize gutters. Board content (headers, projects, cards, dates) stays.

## Toolbar

Left: product name **Weekly Status Board**.

Right, in order:

- New, Open, Save (Save As via Save menu or `Ctrl+Shift+S`)
- Copy image, Export PNG
- View (toggle)
- Dark Theme / Light Theme
- Zoom group: `−` / `NN%` / `+` (Gantt behavior)
- **Attended weekly meeting** (primary button)

Undo/Redo: `Ctrl+Z` / `Ctrl+Y`. Toolbar icons optional; keyboard is required.

## Interactions

### Edit in place

- Click project name, task title, or date chip to edit.
- Enter commits. Escape cancels. Click-away commits if the value is valid.
- Blank project name or blank task title restores the previous value.
- Dates accept `M/D` or `M/D/YYYY` in the local year; stored as ISO `YYYY-MM-DD`. Invalid date restores previous.

### Add

- **+** on a project row: new task under that project **in that column**, title focused, due date **today**.
- **+ Add project** at the bottom of a column: new project (name focused, default `New project`) plus one empty task under it in that column. Duplicate names are allowed; identity is the project id.

### Move

- Drag a card to another column (drop onto a project section or onto empty column space).
- If dropped on a project section, the task’s `project_id` stays unless dropped on a *different* project; dropping on a different project reparents.
- If dropped on empty column space in a column that already has this project section, keep `project_id` and change column.
- If that project has no section in the destination column, create the section.
- **Move right / move left** on a selected card: Target ↔ In Progress ↔ Done, same project. Creates the destination section if needed.
- Moving the last task out of a section leaves the empty project heading in the source column (empty sections are allowed).

### Delete

- Selected card: `Delete` or `Backspace` removes that task. No confirm.
- Selected project header: `Delete` or `Backspace` removes **that column’s section** and its tasks in that column only. Confirm if the section has any tasks. Other columns’ sections for that project stay. If no sections remain, drop the project record.

### View, zoom, widths

- **View** on: hide add controls and gutters. Off: editor chrome back.
- Copy/Export rasterize as if View is on, even when the toggle is off.
- Zoom: clamp `0.25..=1.50`, step `0.10`. `Ctrl+-` / `Ctrl++` / `Ctrl+0`. Clicking the percent resets to 100%. Zoom scales the board workspace. PNG is WYSIWYG at that zoom, 2× pixels.
- Column widths: three fractions summing to 1. Minimum fraction **0.18** per column. Saved in the board file.

### Attended weekly meeting

1. Confirm: “Clear all Done tasks? This cannot be undone after you quit.”
2. On confirm, delete every task in Done. Leave Done project headings that are now empty (user can delete headings). Target and In Progress unchanged.
3. In-session undo restores the deleted Done tasks.

## Data model

Pretty-printed JSON. Extension: **`.board.json`**. Schema `version: 1`.

```json
{
  "version": 1,
  "title": "Weekly Status Board",
  "theme": "light",
  "zoom": 1.0,
  "column_widths": [0.40, 0.30, 0.30],
  "projects": [
    { "id": "01…", "name": "HVDC" }
  ],
  "sections": [
    { "project_id": "01…", "column": "target" }
  ],
  "tasks": [
    {
      "id": "01…",
      "project_id": "01…",
      "column": "target",
      "title": "Review testing room ventilation with P3",
      "due": "2026-08-20"
    }
  ]
}
```

- `theme`: `"light"` | `"dark"`.
- `column`: `"target"` | `"in_progress"` | `"done"`.
- `column_widths`: three positive numbers; load normalizes to sum 1 and enforces the 0.18 minimum.
- `sections` is the source of truth for which project headings appear in which column, including empty headings.
- Display order: `projects` array order within each column. New projects append. User cannot reorder projects in v1 (YAGNI).
- Task order: array order within a `(project_id, column)` group. New tasks append. A drop onto a project section **appends** to that section. Reordering cards inside a section is out of v1.

Ids are ULIDs or UUIDs as strings. Do not reuse ids.

## Date roll

When the board is loaded, and when the local calendar date changes while the app is running:

For each task with `column` of `target` or `in_progress`: if `due < today` (local), set `due = today`.

- Done tasks never roll.
- Future dates never move.
- If the app was closed for a week, the next open jumps missed dates to **today** (not +1 per missed day).
- If any date changed, persist immediately when a file path exists; otherwise write the unnamed draft.

Displayed dates use local `M/D` (no year) when the year is the current local year; otherwise `M/D/YYYY`.

## Persistence

| Kind | Location |
|---|---|
| Named board | User-chosen `*.board.json` |
| Unnamed draft | App data dir `draft.board.json` |
| Session | App data dir `session.json` with `active_path` (null if draft) |

- After Open or Save As, committed edits autosave to that path.
- **Save** forces a write. Unnamed **Save** is Save As.
- **New**: if the current board is unnamed and non-blank, ask Save As before replacing. If it is named, autosave then open a blank unnamed board.
- On launch, reopen `active_path` if the file still exists; else draft; else empty board. Then date-roll.
- Corrupt JSON: do not overwrite the file. Show the OS/parse error in the footer. Keep an empty in-memory board. User can Open another file or Save As.

Gantt-style recovery: the draft exists so a crash of an unnamed board is not a total loss.

## Export

- **Region:** board workspace only (three columns). Not header, not footer, not add controls, not gutters.
- **Geometry:** on-screen workspace size at current zoom and column widths. Raster scale **2**.
- **Background:** solid theme background (white in light).
- **Copy image:** PNG bytes to the Windows clipboard.
- **Export PNG:** native Save dialog, `.png` only, write bytes.
- Shared pipeline for copy and export.
- If the workspace has not been laid out (width or height ≤ 0) or capture fails, footer shows the **real** error (capture API, empty bounds, clipboard, permission, path). Do not replace it with a generic “export failed.”

### Capture strategy

GPUI has no Gantt-style SVG export. Implementation order:

1. Prove a GPUI window on this machine can produce a PNG of a view (GPUI screenshot/render-to-image if present).
2. If not, capture the board rectangle with Win32 (`BitBlt` / `PrintWindow`) and encode PNG.
3. If both fail, **stop** and choose another path before building the rest of the board. Do not fake export.

## Architecture

Rust / GPUI crate (pin a crates.io version that **builds on this Windows machine** in the first slice). No Tauri, no React.

Suggested modules (names may shift; boundaries must not):

| Module | Responsibility |
|---|---|
| `model` | `BoardDocument`, ids, columns, validation, equality |
| `dates` | parse/display `M/D`, date-roll |
| `history` | in-memory undo/redo snapshots of `BoardDocument` |
| `persistence` | load/save JSON, draft, session |
| `export` | capture board → PNG bytes; clipboard; write file |
| `theme` | Inventory tokens, light/dark |
| `ui` | window, header, columns, cards, zoom, dialogs |

Keep date-roll, move, meeting-clear, JSON round-trip, and zoom clamp **pure** so they unit-test without GPUI.

## Error handling

| Failure | Behavior |
|---|---|
| File missing / unreadable | Footer: path + OS error. Don’t delete the file. |
| JSON invalid | Footer: parse error. Don’t overwrite. |
| Save permission / disk | Footer: path + OS error. Memory unchanged. User can Save As. |
| Export/clipboard | Footer: underlying error. |
| Capture not ready | Footer: board is not laid out yet. |

Destructive meeting-clear always confirms.

## Testing

Must have Rust unit tests for:

- Date-roll: past Target/In Progress → today; future unchanged; Done unchanged; closed-for-a-week jump.
- Move: column change; section created on destination; empty source section remains.
- Meeting-clear: Done tasks gone; other columns intact; undo restores.
- JSON: round-trip; unknown `version` rejected; width normalize + 0.18 clamp.
- Zoom clamp 0.25–1.50, step rounding.

Manual on Windows: open app, edit, View, zoom, resize columns, Copy image into PowerPoint, Export PNG, meeting-clear, dark theme.

## First slice (must prove before the board)

A GPUI window on this laptop with Inventory background, product title, and a PNG capture of some view. If that slice fails, do not implement the kanban UI on an unproven renderer/export path.

## Out of scope reminders

Gantt merge, GPUI rewrite of Gantt, team sharing, `.pptx`, notes/priority fields, original-due-date history.
