//! Streaming availability and trending via TMDB (watch providers).

use crate::commands::open::open_with_system;
use crate::db::{get_setting, now_iso, set_setting, DbError, DbState};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

const SETTING_TMDB_KEY: &str = "streaming.tmdb_api_key";
const SETTING_PROVIDERS: &str = "streaming.enabled_providers";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingProvider {
    pub id: String,
    pub name: String,
    pub tmdb_provider_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingItem {
    pub id: i64,
    pub provider_id: String,
    pub provider_name: String,
    pub kind: String,
    pub tmdb_id: i64,
    pub media_type: String,
    pub title: String,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub release_date: Option<String>,
    pub score: f64,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingRefreshResult {
    pub providers: usize,
    pub upserted: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingSettings {
    pub has_api_key: bool,
    pub enabled_providers: Vec<String>,
}

pub fn default_providers() -> Vec<StreamingProvider> {
    vec![
        StreamingProvider {
            id: "prime".into(),
            name: "Prime Video".into(),
            tmdb_provider_id: 9,
        },
        StreamingProvider {
            id: "apple".into(),
            name: "Apple TV+".into(),
            tmdb_provider_id: 350,
        },
        StreamingProvider {
            id: "paramount".into(),
            name: "Paramount+".into(),
            tmdb_provider_id: 531,
        },
        StreamingProvider {
            id: "peacock".into(),
            name: "Peacock".into(),
            tmdb_provider_id: 386,
        },
        StreamingProvider {
            id: "amc".into(),
            name: "AMC+".into(),
            tmdb_provider_id: 526,
        },
        StreamingProvider {
            id: "netflix".into(),
            name: "Netflix".into(),
            tmdb_provider_id: 8,
        },
        StreamingProvider {
            id: "max".into(),
            name: "Max".into(),
            tmdb_provider_id: 1899,
        },
        StreamingProvider {
            id: "disney".into(),
            name: "Disney+".into(),
            tmdb_provider_id: 337,
        },
        StreamingProvider {
            id: "hulu".into(),
            name: "Hulu".into(),
            tmdb_provider_id: 15,
        },
    ]
}

fn http_client() -> Result<reqwest::blocking::Client, DbError> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|e| DbError::Message(format!("http: {e}")))
}

pub(crate) fn load_api_key(conn: &Connection) -> Result<String, DbError> {
    get_setting(conn, SETTING_TMDB_KEY)?
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| {
            DbError::Message(
                "Add a free TMDB API key in Streaming settings (themoviedb.org).".into(),
            )
        })
}

pub(crate) fn load_enabled_providers(conn: &Connection) -> Vec<String> {
    let defaults: Vec<String> = default_providers().iter().map(|p| p.id.clone()).collect();
    let Ok(Some(json)) = get_setting(conn, SETTING_PROVIDERS) else {
        return defaults;
    };
    serde_json::from_str::<Vec<String>>(&json).unwrap_or(defaults)
}

fn provider_by_id(id: &str) -> Option<StreamingProvider> {
    default_providers().into_iter().find(|p| p.id == id)
}

fn tmdb_get(client: &reqwest::blocking::Client, path: &str, key: &str) -> Result<Value, DbError> {
    let url = format!(
        "https://api.themoviedb.org/3{path}{}api_key={}",
        if path.contains('?') { "&" } else { "?" },
        urlencoding::encode(key)
    );
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| DbError::Message(format!("tmdb: {e}")))?;
    let status = resp.status();
    let body: Value = resp
        .json()
        .map_err(|e| DbError::Message(format!("tmdb json: {e}")))?;
    if !status.is_success() {
        return Err(DbError::Message(format!("TMDB error ({status})")));
    }
    Ok(body)
}

fn upsert_item(
    conn: &Connection,
    provider: &StreamingProvider,
    kind: &str,
    media_type: &str,
    item: &Value,
    fetched_at: &str,
) -> Result<bool, DbError> {
    let tmdb_id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    if tmdb_id == 0 {
        return Ok(false);
    }
    let title = item
        .get("title")
        .or_else(|| item.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let overview = item
        .get("overview")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let poster_path = item
        .get("poster_path")
        .and_then(|v| v.as_str())
        .map(|s| format!("https://image.tmdb.org/t/p/w342{s}"));
    let release_date = item
        .get("release_date")
        .or_else(|| item.get("first_air_date"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let score = item
        .get("popularity")
        .or_else(|| item.get("vote_average"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let changed = conn.execute(
        "INSERT INTO streaming_items (
            provider_id, provider_name, kind, tmdb_id, media_type, title, overview,
            poster_path, release_date, score, fetched_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(provider_id, kind, tmdb_id, media_type) DO UPDATE SET
            title = excluded.title,
            overview = excluded.overview,
            poster_path = excluded.poster_path,
            release_date = excluded.release_date,
            score = excluded.score,
            fetched_at = excluded.fetched_at",
        params![
            provider.id,
            provider.name,
            kind,
            tmdb_id,
            media_type,
            title,
            overview,
            poster_path,
            release_date,
            score,
            fetched_at,
        ],
    )?;
    Ok(changed > 0)
}

#[tauri::command]
pub fn list_streaming_providers() -> Vec<StreamingProvider> {
    default_providers()
}

#[tauri::command]
pub fn get_streaming_settings(state: State<'_, DbState>) -> Result<StreamingSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let has_api_key = get_setting(db.conn(), SETTING_TMDB_KEY)?
        .filter(|k| !k.trim().is_empty())
        .is_some();
    let enabled_providers = load_enabled_providers(db.conn());
    Ok(StreamingSettings {
        has_api_key,
        enabled_providers,
    })
}

#[tauri::command]
pub fn save_streaming_settings(
    state: State<'_, DbState>,
    api_key: Option<String>,
    enabled_providers: Option<Vec<String>>,
) -> Result<StreamingSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    if let Some(key) = api_key.as_deref() {
        let key = key.trim();
        if !key.is_empty() {
            set_setting(db.conn(), SETTING_TMDB_KEY, key)?;
        }
    }
    if let Some(ids) = enabled_providers {
        let json = serde_json::to_string(&ids)
            .map_err(|e| DbError::Message(format!("providers json: {e}")))?;
        set_setting(db.conn(), SETTING_PROVIDERS, &json)?;
    }
    drop(db);
    get_streaming_settings(state)
}

#[tauri::command]
pub fn refresh_streaming(state: State<'_, DbState>) -> Result<StreamingRefreshResult, DbError> {
    run_refresh_streaming(&state)
}

pub(crate) fn run_refresh_streaming(state: &DbState) -> Result<StreamingRefreshResult, DbError> {
    let (key, enabled) = {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        (load_api_key(db.conn())?, load_enabled_providers(db.conn()))
    };
    let client = http_client()?;
    let fetched_at = now_iso();
    let mut upserted = 0usize;
    let mut errors = Vec::new();
    let since = (Utc::now() - Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    for pid in &enabled {
        let Some(provider) = provider_by_id(pid) else {
            continue;
        };
        // Hot: trending on this provider (discover TV + movie)
        for media_type in ["tv", "movie"] {
            let path = format!(
                "/discover/{media_type}?watch_region=US&with_watch_providers={}&sort_by=popularity.desc",
                provider.tmdb_provider_id
            );
            match tmdb_get(&client, &path, &key) {
                Ok(body) => {
                    if let Some(results) = body.get("results").and_then(|v| v.as_array()) {
                        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
                        for item in results.iter().take(8) {
                            if upsert_item(
                                db.conn(),
                                &provider,
                                "hot",
                                media_type,
                                item,
                                &fetched_at,
                            )? {
                                upserted += 1;
                            }
                        }
                    }
                }
                Err(e) => errors.push(format!("{} hot {media_type}: {e}", provider.name)),
            }
        }

        // New: recently released on provider
        for media_type in ["tv", "movie"] {
            let date_key = if media_type == "movie" {
                "primary_release_date.gte"
            } else {
                "first_air_date.gte"
            };
            let path = format!(
                "/discover/{media_type}?watch_region=US&with_watch_providers={}&sort_by={date_key}.desc&{date_key}={since}",
                provider.tmdb_provider_id
            );
            match tmdb_get(&client, &path, &key) {
                Ok(body) => {
                    if let Some(results) = body.get("results").and_then(|v| v.as_array()) {
                        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
                        for item in results.iter().take(6) {
                            if upsert_item(
                                db.conn(),
                                &provider,
                                "new",
                                media_type,
                                item,
                                &fetched_at,
                            )? {
                                upserted += 1;
                            }
                        }
                    }
                }
                Err(e) => errors.push(format!("{} new {media_type}: {e}", provider.name)),
            }
        }
    }

    Ok(StreamingRefreshResult {
        providers: enabled.len(),
        upserted,
        errors,
    })
}

fn list_by_kind(conn: &Connection, kind: &str, limit: i64) -> Result<Vec<StreamingItem>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, provider_name, kind, tmdb_id, media_type, title, overview,
                poster_path, release_date, score, fetched_at
         FROM streaming_items
         WHERE kind = ?1
         ORDER BY score DESC, release_date DESC, id DESC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![kind, limit], |row| {
            Ok(StreamingItem {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                provider_name: row.get(2)?,
                kind: row.get(3)?,
                tmdb_id: row.get(4)?,
                media_type: row.get(5)?,
                title: row.get(6)?,
                overview: row.get(7)?,
                poster_path: row.get(8)?,
                release_date: row.get(9)?,
                score: row.get(10)?,
                fetched_at: row.get(11)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn list_streaming_hot(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<StreamingItem>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    list_by_kind(db.conn(), "hot", limit.unwrap_or(12).clamp(1, 100))
}

#[tauri::command]
pub fn list_streaming_new(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<StreamingItem>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    list_by_kind(db.conn(), "new", limit.unwrap_or(12).clamp(1, 100))
}

#[tauri::command]
pub fn open_streaming_item(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let (media_type, tmdb_id): (String, i64) = db.conn().query_row(
        "SELECT media_type, tmdb_id FROM streaming_items WHERE id = ?1",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let path = if media_type == "tv" {
        format!("https://www.themoviedb.org/tv/{tmdb_id}")
    } else {
        format!("https://www.themoviedb.org/movie/{tmdb_id}")
    };
    open_with_system("url", &path)
}
