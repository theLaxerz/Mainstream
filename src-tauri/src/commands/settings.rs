use crate::db::{get_setting, set_setting, DbError, DbState};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsPref {
    pub id: i64,
    pub feed_url: String,
    pub title: Option<String>,
    pub weight: f64,
    pub enabled: bool,
    pub muted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertNewsPrefInput {
    pub feed_url: String,
    pub title: Option<String>,
    pub weight: Option<f64>,
    pub enabled: Option<bool>,
    pub muted: Option<bool>,
}

#[tauri::command]
pub fn get_setting_cmd(state: State<'_, DbState>, key: String) -> Result<Option<String>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    get_setting(db.conn(), &key)
}

#[tauri::command]
pub fn set_setting_cmd(
    state: State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    set_setting(db.conn(), &key, &value)
}

#[tauri::command]
pub fn list_settings(state: State<'_, DbState>) -> Result<Vec<Setting>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db.conn().prepare("SELECT key, value FROM settings ORDER BY key")?;
    let items = stmt
        .query_map([], |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

#[tauri::command]
pub fn list_news_prefs(state: State<'_, DbState>) -> Result<Vec<NewsPref>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db.conn().prepare(
        "SELECT id, feed_url, title, weight, enabled, muted FROM news_prefs ORDER BY id",
    )?;
    let items = stmt
        .query_map([], |row| {
            Ok(NewsPref {
                id: row.get(0)?,
                feed_url: row.get(1)?,
                title: row.get(2)?,
                weight: row.get(3)?,
                enabled: row.get::<_, i64>(4)? != 0,
                muted: row.get::<_, i64>(5)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

#[tauri::command]
pub fn upsert_news_pref(
    state: State<'_, DbState>,
    input: UpsertNewsPrefInput,
) -> Result<NewsPref, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let feed_url = input.feed_url.trim();
    if feed_url.is_empty() {
        return Err(DbError::Message("feed_url is required".into()));
    }
    let weight = input.weight.unwrap_or(1.0);
    let enabled = if input.enabled.unwrap_or(true) { 1 } else { 0 };
    let muted = if input.muted.unwrap_or(false) { 1 } else { 0 };

    db.conn().execute(
        "INSERT INTO news_prefs (feed_url, title, weight, enabled, muted)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(feed_url) DO UPDATE SET
           title = excluded.title,
           weight = excluded.weight,
           enabled = excluded.enabled,
           muted = excluded.muted",
        params![feed_url, input.title, weight, enabled, muted],
    )?;

    let mut stmt = db.conn().prepare(
        "SELECT id, feed_url, title, weight, enabled, muted FROM news_prefs WHERE feed_url = ?1",
    )?;
    let pref = stmt.query_row(params![feed_url], |row| {
        Ok(NewsPref {
            id: row.get(0)?,
            feed_url: row.get(1)?,
            title: row.get(2)?,
            weight: row.get(3)?,
            enabled: row.get::<_, i64>(4)? != 0,
            muted: row.get::<_, i64>(5)? != 0,
        })
    })?;
    Ok(pref)
}

#[tauri::command]
pub fn delete_news_pref(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let changed = db
        .conn()
        .execute("DELETE FROM news_prefs WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::Message(format!("news_pref {} not found", id)));
    }
    Ok(())
}
