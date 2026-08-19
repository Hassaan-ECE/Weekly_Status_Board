# Session start prompt — Weekly Status Board

Copy everything below the line into a **new chat** with workspace:

```text
C:\Projects\Active\Weekly_Status_Board
```

---

You are the parent: Grok 4.6 high. Follow `~/.grok/rules/orchestrate.md` (workers/explore = grok-4.5, advisor = grok-4.6 extra-high only if needed).

## Job

Execute the **already approved** Weekly Status Board implementation plan with **subagent-driven development**. Do not redesign. Do not re-brainstorm. Do not switch to Tauri/egui.

## Open this workspace first

```text
C:\Projects\Active\Weekly_Status_Board
```

GitHub: https://github.com/Hassaan-ECE/Weekly_Status_Board.git  
Branch today: `main` (spec + plan only; **no app code yet**).  
Create a feature branch / worktree before writing code. Do not implement on `main` unless I say so.

## Read first (in this order)

1. `docs/superpowers/specs/2026-08-19-weekly-status-board-design.md`
2. `docs/superpowers/plans/2026-08-19-weekly-status-board.md`
3. This file if you need the handoff again

Then invoke skill **subagent-driven-development** and execute that plan task-by-task.

## What this product is

Personal Windows GPUI app on this laptop. Standing board: **Target → In Progress → Done**, grouped by project. Cards are one line + due date. Dates on open work roll to **today** if past (so the PowerPoint never shows late). **Attended weekly meeting** clears Done. **Copy image / Export PNG** of the board workspace (View look, no + controls) pastes into PowerPoint like Gantt Chart Creator.

Visual chrome copies **Inventory Management** (`C:\Projects\Active\Inventory_Management`): DM Sans if possible, white/dark surfaces, indigo primary, footer **Built by Syed Hassaan Shah**. Colored column headers; bold black project names and date chips; date pinned on the right of each card.

This repo is also the **GPUI reference** for later apps (like Inventory is the visual reference). Keep `theme`, window shell, persistence, and PNG export copyable. Do not bury them in board-specific UI.

## Hard gates (stop and tell me)

- **Task 1:** a GPUI window must open on this Windows laptop.
- **Task 2:** that window must produce a real PNG of itself.
- If either fails, **do not** build the kanban UI. Record the error. Do not fake export.

## Constraints

- GPUI 0.2.2 + Rust. No Tauri, no React.
- Follow the plan’s file map. TDD for `model` / `dates` / `history` / `persistence` / `zoom`.
- Evidence before “works”: `cargo test` / `cargo run` output, and a PNG you actually opened.
- Subagent-driven-development: fresh **worker** per task, then spec review, then quality review. Continuous — do not pause between tasks to ask if you should continue.
- Implementer prompts must include the full task text from the plan (do not make the worker re-read the whole plan).
- If a task is BLOCKED on GPUI APIs, escalate to me with the exact compiler/runtime error.

## Out of scope

Gantt merge, rewriting Gantt in GPUI, team sharing, native `.pptx`, extra task fields, showing “X days late.”

## When done

`cargo test` green, `cargo run` shows the board, Copy/Export PNG pastes into PowerPoint, `AGENTS.md` says which capture path actually worked. Then finishing-a-development-branch options (do not merge to `main` unless I ask).
