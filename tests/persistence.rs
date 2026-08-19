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
    std::fs::write(
        &path,
        r#"{"version":99,"title":"x","theme":"light","zoom":1.0,"column_widths":[0.3,0.3,0.4],"projects":[],"sections":[],"tasks":[]}"#,
    )
    .unwrap();
    assert!(load_board(&path).is_err());
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("\"version\":99"));
}

#[test]
fn load_normalizes_and_clamps_column_widths() {
    let dir = tempfile_dir();
    let path = dir.join("widths.board.json");
    std::fs::write(
        &path,
        r#"{"version":1,"title":"x","theme":"light","zoom":1.0,"column_widths":[2.0,1.0,1.0],"projects":[],"sections":[],"tasks":[]}"#,
    )
    .unwrap();
    let loaded = load_board(&path).unwrap();
    let sum: f32 = loaded.column_widths.iter().sum();
    assert!((sum - 1.0).abs() < 1e-5);
    assert!(loaded.column_widths.iter().all(|x| *x >= 0.18 - 1e-5));
}

#[test]
fn load_clamps_zoom() {
    let dir = tempfile_dir();
    let path = dir.join("zoom.board.json");
    std::fs::write(
        &path,
        r#"{"version":1,"title":"x","theme":"light","zoom":9.0,"column_widths":[0.3,0.3,0.4],"projects":[],"sections":[],"tasks":[]}"#,
    )
    .unwrap();
    let loaded = load_board(&path).unwrap();
    assert_eq!(loaded.zoom, 1.5);
}

#[test]
fn ensure_board_json_suffix_appends_when_missing() {
    use weekly_status_board::persistence::ensure_board_json_suffix;
    let path = ensure_board_json_suffix(std::path::PathBuf::from(r"C:\tmp\notes"));
    assert!(path.to_string_lossy().ends_with(".board.json"));
    let already = ensure_board_json_suffix(std::path::PathBuf::from(r"C:\tmp\a.board.json"));
    assert_eq!(already, std::path::PathBuf::from(r"C:\tmp\a.board.json"));
}

fn tempfile_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wsb-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
