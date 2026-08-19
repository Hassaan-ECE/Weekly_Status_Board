# Weekly Status Board

Personal Windows desktop board for weekly manager updates. Cards move Target → In Progress → Done, grouped by project. Copy/Export PNG pastes into PowerPoint.

Built with **GPUI**. Treat this repo as the GPUI reference for later apps (theme, window shell, JSON files, PNG export). See `AGENTS.md`.

**Status:** implementation in progress on `feat/weekly-status-board` (window, board UI, JSON persistence, View/zoom, meeting clear, Copy/Export PNG, dark theme).

## Run

```
cargo run
```

First GPUI compile is slow. Requires Windows.

## Test

```
cargo test
```

See `docs/superpowers/specs/2026-08-19-weekly-status-board-design.md`.
