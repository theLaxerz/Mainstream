use crate::db::{now_iso, DbError, DbState};
use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;

const TITLE_MAX: usize = 200;
const NOTES_MAX: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub notes: String,
    pub due_on: Option<String>,
    pub priority: i64,
    pub completed: bool,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSummary {
    pub open: i64,
    pub overdue: i64,
    pub due_today: i64,
    pub upcoming: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskInput {
    pub title: String,
    pub notes: Option<String>,
    pub due_on: Option<String>,
    pub priority: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskInput {
    pub id: i64,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub due_on: Option<Option<String>>,
    pub priority: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTaskCompletedInput {
    pub id: i64,
    pub completed: bool,
}

fn normalize_title(title: &str) -> Result<String, DbError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(DbError::Message("title is required".into()));
    }
    if title.chars().count() > TITLE_MAX {
        return Err(DbError::Message(format!(
            "title must be {TITLE_MAX} characters or fewer"
        )));
    }
    Ok(title.to_string())
}

fn normalize_notes(notes: &str) -> Result<String, DbError> {
    if notes.chars().count() > NOTES_MAX {
        return Err(DbError::Message(format!(
            "notes must be {NOTES_MAX} characters or fewer"
        )));
    }
    Ok(notes.to_string())
}

fn normalize_due_on(raw: Option<&str>) -> Result<Option<String>, DbError> {
    let Some(value) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        DbError::Message("due date must be YYYY-MM-DD".into())
    })?;
    Ok(Some(value.to_string()))
}

fn normalize_priority(priority: i64) -> i64 {
    if priority > 0 { 1 } else { 0 }
}

fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    let completed: i64 = row.get(5)?;
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        notes: row.get(2)?,
        due_on: row.get(3)?,
        priority: row.get(4)?,
        completed: completed != 0,
        completed_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

const TASK_COLS: &str = "id, title, notes, due_on, priority, completed, completed_at, created_at, updated_at";

fn get_task_inner(conn: &Connection, id: i64) -> Result<Option<Task>, DbError> {
    let mut stmt = conn.prepare(&format!("SELECT {TASK_COLS} FROM tasks WHERE id = ?1"))?;
    let task = stmt
        .query_row(params![id], map_task)
        .optional()?;
    Ok(task)
}

fn today_local() -> String {
    chrono::Local::now().date_naive().format("%Y-%m-%d").to_string()
}

#[tauri::command]
pub fn list_tasks(
    state: State<'_, DbState>,
    limit: Option<i64>,
    include_completed: Option<bool>,
) -> Result<Vec<Task>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let limit = limit.unwrap_or(40).clamp(1, 400);
    let include_completed = include_completed.unwrap_or(true);
    let today = today_local();
    let filter = if include_completed {
        ""
    } else {
        "WHERE completed = 0"
    };
    // Open first, then overdue / today / upcoming / someday, high priority, then due date.
    let sql = format!(
        "SELECT {TASK_COLS} FROM tasks
         {filter}
         ORDER BY completed ASC,
           CASE
             WHEN completed = 1 THEN 4
             WHEN due_on IS NULL THEN 3
             WHEN due_on < ?1 THEN 0
             WHEN due_on = ?1 THEN 1
             ELSE 2
           END ASC,
           priority DESC,
           due_on IS NULL ASC,
           due_on ASC,
           id ASC
         LIMIT ?2"
    );
    let mut stmt = db.conn().prepare(&sql)?;
    let tasks = stmt
        .query_map(params![today, limit], map_task)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tasks)
}

#[tauri::command]
pub fn task_summary(state: State<'_, DbState>) -> Result<TaskSummary, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let today = today_local();
    let mut stmt = db.conn().prepare(
        "SELECT
            COALESCE(SUM(CASE WHEN completed = 0 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN completed = 0 AND due_on IS NOT NULL AND due_on < ?1 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN completed = 0 AND due_on = ?1 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN completed = 0 AND due_on IS NOT NULL AND due_on > ?1 THEN 1 ELSE 0 END), 0)
         FROM tasks",
    )?;
    let summary = stmt.query_row(params![today], |row| {
        Ok(TaskSummary {
            open: row.get(0)?,
            overdue: row.get(1)?,
            due_today: row.get(2)?,
            upcoming: row.get(3)?,
        })
    })?;
    Ok(summary)
}

#[tauri::command]
pub fn create_task(state: State<'_, DbState>, input: CreateTaskInput) -> Result<Task, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let title = normalize_title(&input.title)?;
    let notes = normalize_notes(input.notes.as_deref().unwrap_or(""))?;
    let due_on = normalize_due_on(input.due_on.as_deref())?;
    let priority = normalize_priority(input.priority.unwrap_or(0));
    let now = now_iso();
    db.conn().execute(
        "INSERT INTO tasks (title, notes, due_on, priority, completed, completed_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, NULL, ?5, ?6)",
        params![title, notes, due_on, priority, now, now],
    )?;
    let id = db.conn().last_insert_rowid();
    get_task_inner(db.conn(), id)?
        .ok_or_else(|| DbError::Message("failed to read created task".into()))
}

#[tauri::command]
pub fn update_task(state: State<'_, DbState>, input: UpdateTaskInput) -> Result<Task, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let existing = get_task_inner(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("task {} not found", input.id)))?;

    let title = match input.title {
        Some(t) => normalize_title(&t)?,
        None => existing.title,
    };
    let notes = match input.notes {
        Some(n) => normalize_notes(&n)?,
        None => existing.notes,
    };
    let due_on = match input.due_on {
        Some(value) => normalize_due_on(value.as_deref())?,
        None => existing.due_on,
    };
    let priority = match input.priority {
        Some(p) => normalize_priority(p),
        None => existing.priority,
    };
    let updated_at = now_iso();

    db.conn().execute(
        "UPDATE tasks SET title = ?1, notes = ?2, due_on = ?3, priority = ?4, updated_at = ?5 WHERE id = ?6",
        params![title, notes, due_on, priority, updated_at, input.id],
    )?;

    get_task_inner(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("task {} not found", input.id)))
}

#[tauri::command]
pub fn set_task_completed(
    state: State<'_, DbState>,
    input: SetTaskCompletedInput,
) -> Result<Task, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let existing = get_task_inner(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("task {} not found", input.id)))?;
    if existing.completed == input.completed {
        return Ok(existing);
    }
    let now = now_iso();
    let completed_at = if input.completed {
        Some(now.clone())
    } else {
        None
    };
    db.conn().execute(
        "UPDATE tasks SET completed = ?1, completed_at = ?2, updated_at = ?3 WHERE id = ?4",
        params![if input.completed { 1 } else { 0 }, completed_at, now, input.id],
    )?;
    get_task_inner(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("task {} not found", input.id)))
}

#[tauri::command]
pub fn delete_task(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let changed = db
        .conn()
        .execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::Message(format!("task {id} not found")));
    }
    Ok(())
}

#[tauri::command]
pub fn clear_completed_tasks(state: State<'_, DbState>) -> Result<i64, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let changed = db
        .conn()
        .execute("DELETE FROM tasks WHERE completed = 1", [])?;
    Ok(changed as i64)
}
