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
    let board: BoardDocument =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    if board.version != SCHEMA_VERSION {
        bail!(
            "unsupported board version {} in {}",
            board.version,
            path.display()
        );
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
