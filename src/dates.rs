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

pub fn first_of_month(date: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
}

pub fn shift_month(date: NaiveDate, delta: i32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 + delta;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }
    NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap_or(first_of_month(date))
}

pub fn month_title(date: NaiveDate) -> String {
    const NAMES: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let idx = date.month().saturating_sub(1) as usize;
    format!("{} {}", NAMES.get(idx).copied().unwrap_or(""), date.year())
}

/// Sunday-first month grid. `None` cells are padding.
pub fn month_cells(month: NaiveDate) -> Vec<Option<NaiveDate>> {
    let first = first_of_month(month);
    let pad = first.weekday().num_days_from_sunday() as usize;
    let mut cells = vec![None; pad];
    let mut day = first;
    loop {
        cells.push(Some(day));
        match day.succ_opt() {
            Some(next) if next.month() == first.month() => day = next,
            _ => break,
        }
    }
    while !cells.is_empty() && cells.len() % 7 != 0 {
        cells.push(None);
    }
    cells
}
