use chrono::NaiveDate;
use weekly_status_board::dates::{
    apply_date_roll, display_date, first_of_month, month_cells, parse_date, shift_month,
};
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

#[test]
fn month_cells_sunday_first_august_2026() {
    let month = first_of_month(d(2026, 8, 19));
    assert_eq!(month, d(2026, 8, 1));
    let cells = month_cells(month);
    assert_eq!(cells.len() % 7, 0);
    // 2026-08-01 is Saturday → six leading blanks.
    assert!(cells[0..6].iter().all(|c| c.is_none()));
    assert_eq!(cells[6], Some(d(2026, 8, 1)));
    assert_eq!(cells[6 + 30], Some(d(2026, 8, 31)));
}

#[test]
fn shift_month_wraps_year() {
    assert_eq!(shift_month(d(2026, 12, 1), 1), d(2027, 1, 1));
    assert_eq!(shift_month(d(2026, 1, 15), -1), d(2025, 12, 1));
}
