use chrono::{Datelike, NaiveDate};
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
