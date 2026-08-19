use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MIN_COLUMN_FRACTION: f32 = 0.18;
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Column {
    Target,
    InProgress,
    Done,
}

impl Column {
    pub fn all() -> [Column; 3] {
        [Column::Target, Column::InProgress, Column::Done]
    }

    pub fn index(self) -> usize {
        match self {
            Column::Target => 0,
            Column::InProgress => 1,
            Column::Done => 2,
        }
    }

    pub fn right(self) -> Option<Column> {
        match self {
            Column::Target => Some(Column::InProgress),
            Column::InProgress => Some(Column::Done),
            Column::Done => None,
        }
    }

    pub fn left(self) -> Option<Column> {
        match self {
            Column::Target => None,
            Column::InProgress => Some(Column::Target),
            Column::Done => Some(Column::InProgress),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section {
    pub project_id: String,
    pub column: Column,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub column: Column,
    pub title: String,
    pub due: NaiveDate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardDocument {
    pub version: u32,
    pub title: String,
    pub theme: ThemeMode,
    pub zoom: f32,
    pub column_widths: [f32; 3],
    pub projects: Vec<Project>,
    pub sections: Vec<Section>,
    pub tasks: Vec<Task>,
}

impl BoardDocument {
    pub fn empty() -> Self {
        Self {
            version: SCHEMA_VERSION,
            title: "Weekly Status Board".into(),
            theme: ThemeMode::Light,
            zoom: 1.0,
            column_widths: [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
            projects: Vec::new(),
            sections: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn is_blank(&self) -> bool {
        self.projects.is_empty() && self.tasks.is_empty() && self.sections.is_empty()
    }

    fn new_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn add_project(&mut self, column: Column) -> (String, String) {
        let project_id = Self::new_id();
        self.projects.push(Project {
            id: project_id.clone(),
            name: "New project".into(),
        });
        self.ensure_section(&project_id, column);
        let task_id = self.add_task(&project_id, column);
        (project_id, task_id)
    }

    pub fn add_task(&mut self, project_id: &str, column: Column) -> String {
        self.ensure_section(project_id, column);
        let id = Self::new_id();
        self.tasks.push(Task {
            id: id.clone(),
            project_id: project_id.to_string(),
            column,
            title: String::new(),
            due: chrono::Local::now().date_naive(),
        });
        id
    }

    pub fn ensure_section(&mut self, project_id: &str, column: Column) {
        if !self.has_section(project_id, column) {
            self.sections.push(Section {
                project_id: project_id.to_string(),
                column,
            });
        }
    }

    pub fn has_section(&self, project_id: &str, column: Column) -> bool {
        self.sections
            .iter()
            .any(|s| s.project_id == project_id && s.column == column)
    }

    pub fn task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn set_task_title(&mut self, id: &str, title: impl Into<String>) {
        if let Some(t) = self.task_mut(id) {
            let next = title.into();
            if !next.trim().is_empty() {
                t.title = next;
            }
        }
    }

    pub fn set_task_due(&mut self, id: &str, due: NaiveDate) {
        if let Some(t) = self.task_mut(id) {
            t.due = due;
        }
    }

    pub fn set_project_name(&mut self, id: &str, name: impl Into<String>) {
        if let Some(p) = self.projects.iter_mut().find(|p| p.id == id) {
            let next = name.into();
            if !next.trim().is_empty() {
                p.name = next;
            }
        }
    }

    pub fn tasks_in(&self, project_id: &str, column: Column) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.project_id == project_id && t.column == column)
            .collect()
    }

    pub fn move_task(&mut self, task_id: &str, dest_column: Column, dest_project_id: &str) {
        let Some(idx) = self.tasks.iter().position(|t| t.id == task_id) else {
            return;
        };
        self.tasks[idx].column = dest_column;
        self.tasks[idx].project_id = dest_project_id.to_string();
        self.ensure_section(dest_project_id, dest_column);
        let task = self.tasks.remove(idx);
        self.tasks.push(task);
    }

    pub fn move_task_side(&mut self, task_id: &str, dest: Column) {
        let Some(project_id) = self.task(task_id).map(|t| t.project_id.clone()) else {
            return;
        };
        self.move_task(task_id, dest, &project_id);
    }

    pub fn delete_task(&mut self, task_id: &str) {
        self.tasks.retain(|t| t.id != task_id);
    }

    pub fn delete_section(&mut self, project_id: &str, column: Column) {
        self.tasks
            .retain(|t| !(t.project_id == project_id && t.column == column));
        self.sections
            .retain(|s| !(s.project_id == project_id && s.column == column));
        if !self.sections.iter().any(|s| s.project_id == project_id) {
            self.projects.retain(|p| p.id != project_id);
        }
    }

    pub fn clear_done(&mut self) {
        self.tasks.retain(|t| t.column != Column::Done);
    }

    pub fn set_column_widths(&mut self, widths: [f32; 3]) {
        let mut w = widths.map(|x| x.max(0.0));
        let mut sum: f32 = w.iter().sum();
        if sum <= f32::EPSILON {
            w = [1.0 / 3.0; 3];
            sum = 1.0;
        }
        w = w.map(|x| x / sum);
        for i in 0..3 {
            if w[i] < MIN_COLUMN_FRACTION {
                w[i] = MIN_COLUMN_FRACTION;
            }
        }
        let sum: f32 = w.iter().sum();
        self.column_widths = w.map(|x| x / sum);
        let mut guard = 0;
        while self.column_widths.iter().any(|x| *x < MIN_COLUMN_FRACTION - 1e-4) && guard < 8 {
            guard += 1;
            let mut w = self.column_widths;
            for i in 0..3 {
                if w[i] < MIN_COLUMN_FRACTION {
                    w[i] = MIN_COLUMN_FRACTION;
                }
            }
            let sum: f32 = w.iter().sum();
            self.column_widths = w.map(|x| x / sum);
        }
    }
}
