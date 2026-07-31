use crate::db::{now_iso, DbError, DbState};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNoteInput {
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNoteInput {
    pub id: i64,
    pub title: Option<String>,
    pub body: Option<String>,
}

#[tauri::command]
pub fn list_notes(state: State<'_, DbState>, limit: Option<i64>) -> Result<Vec<Note>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let limit = limit.unwrap_or(100);
    let mut stmt = db.conn().prepare(
        "SELECT id, title, body, created_at, updated_at
         FROM notes
         ORDER BY updated_at DESC
         LIMIT ?1",
    )?;
    let notes = stmt
        .query_map(params![limit], |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                body: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

#[tauri::command]
pub fn get_note(state: State<'_, DbState>, id: i64) -> Result<Option<Note>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db.conn().prepare(
        "SELECT id, title, body, created_at, updated_at FROM notes WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn create_note(state: State<'_, DbState>, input: CreateNoteInput) -> Result<Note, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let now = now_iso();
    let title = input.title.trim();
    if title.is_empty() {
        return Err(DbError::Message("title is required".into()));
    }
    let body = input.body.unwrap_or_default();
    db.conn().execute(
        "INSERT INTO notes (title, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![title, body, now, now],
    )?;
    let id = db.conn().last_insert_rowid();
    Ok(Note {
        id,
        title: title.to_string(),
        body,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_note(state: State<'_, DbState>, input: UpdateNoteInput) -> Result<Note, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let existing = get_note_inner(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("note {} not found", input.id)))?;

    let title = input
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or(existing.title);
    let body = input.body.unwrap_or(existing.body);
    let updated_at = now_iso();

    db.conn().execute(
        "UPDATE notes SET title = ?1, body = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, body, updated_at, input.id],
    )?;

    Ok(Note {
        id: input.id,
        title,
        body,
        created_at: existing.created_at,
        updated_at,
    })
}

#[tauri::command]
pub fn delete_note(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let changed = db
        .conn()
        .execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::Message(format!("note {} not found", id)));
    }
    Ok(())
}

fn get_note_inner(conn: &rusqlite::Connection, id: i64) -> Result<Option<Note>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, body, created_at, updated_at FROM notes WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}
