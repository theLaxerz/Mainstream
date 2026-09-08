use crate::db::{now_iso, DbError, DbState};
use crate::security::{validate_app_target, validate_open_url};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    pub id: i64,
    pub label: String,
    pub kind: String,
    pub target: String,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShortcutInput {
    pub label: String,
    pub kind: String,
    pub target: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShortcutInput {
    pub id: i64,
    pub label: Option<String>,
    pub kind: Option<String>,
    pub target: Option<String>,
    pub sort_order: Option<i64>,
}

fn validate_kind(kind: &str) -> Result<&str, DbError> {
    match kind {
        "url" | "app" => Ok(kind),
        _ => Err(DbError::Message("kind must be 'url' or 'app'".into())),
    }
}

fn validate_shortcut_target(kind: &str, target: &str) -> Result<String, DbError> {
    match kind {
        "url" => validate_open_url(target),
        "app" => validate_app_target(target),
        _ => Err(DbError::Message("kind must be 'url' or 'app'".into())),
    }
}

#[tauri::command]
pub fn list_shortcuts(state: State<'_, DbState>) -> Result<Vec<Shortcut>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db.conn().prepare(
        "SELECT id, label, kind, target, sort_order, created_at
         FROM shortcuts
         ORDER BY sort_order ASC, id ASC",
    )?;
    let items = stmt
        .query_map([], |row| {
            Ok(Shortcut {
                id: row.get(0)?,
                label: row.get(1)?,
                kind: row.get(2)?,
                target: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

#[tauri::command]
pub fn create_shortcut(
    state: State<'_, DbState>,
    input: CreateShortcutInput,
) -> Result<Shortcut, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let label = input.label.trim();
    let target = input.target.trim();
    if label.is_empty() || target.is_empty() {
        return Err(DbError::Message("label and target are required".into()));
    }
    let kind = validate_kind(input.kind.trim())?.to_string();
    let target = validate_shortcut_target(&kind, target)?;
    let sort_order = input.sort_order.unwrap_or(0);
    let created_at = now_iso();

    db.conn().execute(
        "INSERT INTO shortcuts (label, kind, target, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![label, kind, target, sort_order, created_at],
    )?;
    let id = db.conn().last_insert_rowid();
    Ok(Shortcut {
        id,
        label: label.to_string(),
        kind,
        target,
        sort_order,
        created_at,
    })
}

#[tauri::command]
pub fn update_shortcut(
    state: State<'_, DbState>,
    input: UpdateShortcutInput,
) -> Result<Shortcut, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let existing = get_shortcut_inner(db.conn(), input.id)?
        .ok_or_else(|| DbError::Message(format!("shortcut {} not found", input.id)))?;

    let label = input
        .label
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or(existing.label);
    let kind = if let Some(k) = input.kind.as_deref() {
        validate_kind(k.trim())?.to_string()
    } else {
        existing.kind
    };
    let target = input
        .target
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or(existing.target);
    let target = validate_shortcut_target(&kind, &target)?;
    let sort_order = input.sort_order.unwrap_or(existing.sort_order);

    db.conn().execute(
        "UPDATE shortcuts SET label = ?1, kind = ?2, target = ?3, sort_order = ?4 WHERE id = ?5",
        params![label, kind, target, sort_order, input.id],
    )?;

    Ok(Shortcut {
        id: input.id,
        label,
        kind,
        target,
        sort_order,
        created_at: existing.created_at,
    })
}

#[tauri::command]
pub fn delete_shortcut(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let changed = db
        .conn()
        .execute("DELETE FROM shortcuts WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::Message(format!("shortcut {} not found", id)));
    }
    Ok(())
}

#[tauri::command]
pub fn open_shortcut(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let shortcut = get_shortcut_inner(db.conn(), id)?
        .ok_or_else(|| DbError::Message(format!("shortcut {} not found", id)))?;
    drop(db);
    crate::commands::open::open_with_system(&shortcut.kind, &shortcut.target)
}

fn get_shortcut_inner(conn: &rusqlite::Connection, id: i64) -> Result<Option<Shortcut>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, label, kind, target, sort_order, created_at FROM shortcuts WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Shortcut {
            id: row.get(0)?,
            label: row.get(1)?,
            kind: row.get(2)?,
            target: row.get(3)?,
            sort_order: row.get(4)?,
            created_at: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}
