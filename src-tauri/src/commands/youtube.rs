//! YouTube subscriptions via public channel RSS feeds.

use crate::commands::open::open_with_system;
use crate::db::{now_iso, DbError, DbState};
use crate::security::{
    public_http_client, validate_youtube_channel_id, validate_youtube_watch_url,
};
use chrono::{DateTime, Utc};
use feed_rs::parser;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubePref {
    pub id: i64,
    pub channel_id: String,
    pub title: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeItem {
    pub id: i64,
    pub video_id: String,
    pub channel_id: String,
    pub channel_title: Option<String>,
    pub title: String,
    pub url: String,
    pub published_at: Option<String>,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeRefreshResult {
    pub channels: usize,
    pub upserted: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertYoutubePrefInput {
    pub channel_id: String,
    pub title: Option<String>,
    pub enabled: Option<bool>,
}

fn http_client() -> Result<reqwest::blocking::Client, DbError> {
    public_http_client(20, Some("MainstreamLifeOS/0.1 (+local; YouTube RSS)"))
}

fn normalize_channel_id(raw: &str) -> String {
    let s = raw.trim();
    if s.contains("channel/") {
        if let Some(idx) = s.rfind("channel/") {
            let rest = &s[idx + 8..];
            return rest
                .split(&['/', '?', '&'][..])
                .next()
                .unwrap_or(rest)
                .to_string();
        }
    }
    if s.starts_with("UC") && s.len() >= 20 {
        return s.to_string();
    }
    s.to_string()
}

fn feed_url(channel_id: &str) -> String {
    format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={}",
        urlencoding::encode(channel_id)
    )
}

fn video_id_from_url(url: &str) -> Option<String> {
    if let Some(v) = url.split("v=").nth(1) {
        return Some(v.split('&').next()?.to_string());
    }
    if url.contains("/shorts/") {
        return url
            .split("/shorts/")
            .nth(1)
            .map(|s| s.split('?').next().unwrap_or(s).to_string());
    }
    None
}

#[tauri::command]
pub fn list_youtube_prefs(state: State<'_, DbState>) -> Result<Vec<YoutubePref>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db
        .conn()
        .prepare("SELECT id, channel_id, title, enabled FROM youtube_prefs ORDER BY id ASC")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(YoutubePref {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                title: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn upsert_youtube_pref(
    state: State<'_, DbState>,
    input: UpsertYoutubePrefInput,
) -> Result<YoutubePref, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let channel_id = validate_youtube_channel_id(&normalize_channel_id(&input.channel_id))?;
    if channel_id.is_empty() {
        return Err(DbError::Message("channel_id is required".into()));
    }
    let enabled = input.enabled.unwrap_or(true);
    db.conn().execute(
        "INSERT INTO youtube_prefs (channel_id, title, enabled)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(channel_id) DO UPDATE SET
           title = COALESCE(excluded.title, youtube_prefs.title),
           enabled = excluded.enabled",
        params![channel_id, input.title, if enabled { 1 } else { 0 }],
    )?;
    let id = db.conn().last_insert_rowid();
    let pref = db.conn().query_row(
        "SELECT id, channel_id, title, enabled FROM youtube_prefs WHERE channel_id = ?1",
        params![channel_id],
        |row| {
            Ok(YoutubePref {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                title: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
            })
        },
    )?;
    let _ = id;
    Ok(pref)
}

#[tauri::command]
pub fn delete_youtube_pref(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    db.conn()
        .execute("DELETE FROM youtube_prefs WHERE id = ?1", params![id])?;
    Ok(())
}

fn upsert_video(
    conn: &Connection,
    channel_id: &str,
    channel_title: Option<&str>,
    title: &str,
    url: &str,
    published_at: Option<&str>,
    fetched_at: &str,
) -> Result<bool, DbError> {
    let url = validate_youtube_watch_url(url)?;
    let video_id = video_id_from_url(&url).unwrap_or_else(|| url.clone());
    let changed = conn.execute(
        "INSERT INTO youtube_items (video_id, channel_id, channel_title, title, url, published_at, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(video_id) DO UPDATE SET
           title = excluded.title,
           url = excluded.url,
           published_at = excluded.published_at,
           fetched_at = excluded.fetched_at",
        params![
            video_id,
            channel_id,
            channel_title,
            title,
            url,
            published_at,
            fetched_at
        ],
    )?;
    Ok(changed > 0)
}

#[tauri::command]
pub fn refresh_youtube(state: State<'_, DbState>) -> Result<YoutubeRefreshResult, DbError> {
    run_refresh_youtube(&state)
}

pub(crate) fn run_refresh_youtube(state: &DbState) -> Result<YoutubeRefreshResult, DbError> {
    let prefs: Vec<YoutubePref> = {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        let mut stmt = db.conn().prepare(
            "SELECT id, channel_id, title, enabled FROM youtube_prefs WHERE enabled = 1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(YoutubePref {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                title: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let client = http_client()?;
    let fetched_at = now_iso();
    let mut upserted = 0usize;
    let mut errors = Vec::new();

    for pref in &prefs {
        let Ok(channel_id) = validate_youtube_channel_id(&pref.channel_id) else {
            errors.push(format!("{}: invalid channel id", pref.channel_id));
            continue;
        };
        let url = feed_url(&channel_id);
        match client.get(&url).send().and_then(|r| r.bytes()) {
            Ok(bytes) => match parser::parse(bytes.as_ref()) {
                Ok(feed) => {
                    let channel_title = feed
                        .title
                        .as_ref()
                        .map(|t| t.content.trim().to_string())
                        .or(pref.title.clone());
                    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
                    for entry in feed.entries {
                        let link = entry
                            .links
                            .first()
                            .map(|l| l.href.clone())
                            .unwrap_or_default();
                        if link.is_empty() {
                            continue;
                        }
                        let Ok(link) = validate_youtube_watch_url(&link) else {
                            continue;
                        };
                        let title = entry
                            .title
                            .map(|t| t.content.trim().to_string())
                            .unwrap_or_else(|| "Video".into());
                        let published = entry
                            .published
                            .or(entry.updated)
                            .map(|d: DateTime<Utc>| d.to_rfc3339());
                        if upsert_video(
                            db.conn(),
                            &channel_id,
                            channel_title.as_deref(),
                            &title,
                            &link,
                            published.as_deref(),
                            &fetched_at,
                        )? {
                            upserted += 1;
                        }
                    }
                }
                Err(e) => errors.push(format!("{}: parse {e}", pref.channel_id)),
            },
            Err(e) => errors.push(format!("{}: {e}", pref.channel_id)),
        }
    }

    Ok(YoutubeRefreshResult {
        channels: prefs.len(),
        upserted,
        errors,
    })
}

#[tauri::command]
pub fn list_youtube_items(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<YoutubeItem>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let limit = limit.unwrap_or(12).clamp(1, 200);
    let mut stmt = db.conn().prepare(
        "SELECT id, video_id, channel_id, channel_title, title, url, published_at, fetched_at
         FROM youtube_items
         ORDER BY published_at DESC, id DESC
         LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(YoutubeItem {
                id: row.get(0)?,
                video_id: row.get(1)?,
                channel_id: row.get(2)?,
                channel_title: row.get(3)?,
                title: row.get(4)?,
                url: row.get(5)?,
                published_at: row.get(6)?,
                fetched_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn open_youtube_item(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let url: String = db.conn().query_row(
        "SELECT url FROM youtube_items WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    open_with_system("url", &url)
}
