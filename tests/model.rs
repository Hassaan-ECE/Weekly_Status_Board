use chrono::NaiveDate;
use weekly_status_board::model::{BoardDocument, Column};

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn move_creates_destination_section_and_drops_empty_source() {
    let mut board = BoardDocument::empty();
    let (project_id, task_id) = board.add_project(Column::Target);
    board.set_task_title(&task_id, "Ventilation");
    board.set_task_due(&task_id, day(2026, 8, 20));
    board.move_task(&task_id, Column::InProgress, &project_id);

    assert_eq!(board.task(&task_id).unwrap().column, Column::InProgress);
    assert!(!board.has_section(&project_id, Column::Target));
    assert!(board.has_section(&project_id, Column::InProgress));
}

#[test]
fn move_keeps_source_section_when_other_tasks_remain() {
    let mut board = BoardDocument::empty();
    let (project_id, t1) = board.add_project(Column::Target);
    let t2 = board.add_task(&project_id, Column::Target);
    board.move_task(&t1, Column::InProgress, &project_id);
    assert!(board.has_section(&project_id, Column::Target));
    assert_eq!(board.tasks_in(&project_id, Column::Target).len(), 1);
    assert_eq!(board.task(&t2).unwrap().column, Column::Target);
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
    assert!(!board.has_section(&a, Column::Target));
    assert!(!board.projects.iter().any(|p| p.id == a));
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
