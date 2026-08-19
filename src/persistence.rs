use crate::model::{BoardDocument, SCHEMA_VERSION};
use crate::zoom::clamp_zoom;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub active_path: Option<PathBuf>,
}

pub fn ensure_board_json_suffix(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if s.ends_with(".board.json") {
        path
    } else {
        PathBuf::from(format!("{s}.board.json"))
    }
}

fn tmp_sibling(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_owned();
    os.push(".tmp");
    PathBuf::from(os)
}

pub fn save_board(path: &Path, board: &BoardDocument) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create dir {}", parent.display()))?;
        }
    }
    let text = serde_json::to_string_pretty(board)?;
    let tmp = tmp_sibling(path);
    std::fs::write(&tmp, &text).with_context(|| format!("write {}", tmp.display()))?;
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Windows may refuse rename-over-existing; fall back to direct write.
            let _ = std::fs::remove_file(&tmp);
            std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
        }
    }
}

pub fn load_board(path: &Path) -> Result<BoardDocument> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let board: BoardDocument =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if board.version != SCHEMA_VERSION {
        bail!(
            "unsupported board version {} in {}",
            board.version,
            path.display()
        );
    }
    let mut board = board;
    board.zoom = clamp_zoom(board.zoom);
    board.set_column_widths(board.column_widths);
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

pub struct StartupBoard {
    pub board: BoardDocument,
    pub active_path: Option<PathBuf>,
    /// Path to write after date-roll (named file or draft). None if empty launch.
    pub persist_path: Option<PathBuf>,
    pub status: String,
}

pub fn load_startup_board() -> StartupBoard {
    let mut status = String::new();
    let session = load_session().unwrap_or(Session { active_path: None });

    let mut active_path = None;
    let mut persist_path = None;

    let board = if let Some(path) = session.active_path.filter(|p| p.exists()) {
        match load_board(&path) {
            Ok(board) => {
                active_path = Some(path.clone());
                persist_path = Some(path);
                board
            }
            Err(err) => {
                status = format!("{err:#}");
                BoardDocument::empty()
            }
        }
    } else if let Ok(draft) = draft_path() {
        if draft.exists() {
            match load_board(&draft) {
                Ok(board) => {
                    persist_path = Some(draft);
                    board
                }
                Err(err) => {
                    status = format!("{err:#}");
                    BoardDocument::empty()
                }
            }
        } else {
            BoardDocument::empty()
        }
    } else {
        BoardDocument::empty()
    };

    StartupBoard {
        board,
        active_path,
        persist_path,
        status,
    }
}
