use crate::commands::open::open_with_system;
use crate::db::{get_setting, now_iso, set_setting, DbError, DbState};
use chrono::{DateTime, Utc};
use feed_rs::parser;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

const DEFAULT_FEEDS_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../feeds.default.json"));

/// Recency half-life in hours for exponential decay.
const RECENCY_HALF_LIFE_HOURS: f64 = 36.0;
/// Clamp topic affinity into this range.
const TOPIC_AFFINITY_MIN: f64 = 0.25;
const TOPIC_AFFINITY_MAX: f64 = 3.0;
const FOLLOW_WEIGHT_BUMP: f64 = 0.25;
const LIKE_SOURCE_BUMP: f64 = 0.05;
const HIDE_SOURCE_DROP: f64 = 0.08;
const TOPIC_LIKE_DELTA: f64 = 0.15;
const TOPIC_HIDE_DELTA: f64 = 0.12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    pub id: i64,
    pub source_id: String,
    pub source_title: Option<String>,
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: String,
    pub score: f64,
    pub liked: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsRefreshResult {
    pub fetched_feeds: usize,
    pub upserted_items: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsFeedbackInput {
    pub item_id: i64,
    /// One of: like, hide, follow_source, mute_source
    pub action: String,
}

#[derive(Debug, Deserialize)]
struct DefaultFeed {
    title: String,
    url: String,
    weight: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FeedPref {
    feed_url: String,
    title: Option<String>,
    weight: f64,
    enabled: bool,
    muted: bool,
}

#[tauri::command]
pub fn seed_default_news_feeds(state: State<'_, DbState>) -> Result<usize, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    seed_feeds_if_empty(db.conn())
}

#[tauri::command]
pub fn refresh_news(state: State<'_, DbState>) -> Result<NewsRefreshResult, DbError> {
    let (prefs, client) = {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        seed_feeds_if_empty(db.conn())?;
        let prefs = load_enabled_prefs(db.conn())?;
        let client = http_client()?;
        (prefs, client)
    };

    let mut upserted = 0usize;
    let mut errors = Vec::new();
    let mut parsed: Vec<(String, ParsedEntry)> = Vec::new();

    for pref in &prefs {
        match fetch_feed(&client, &pref.feed_url) {
            Ok(entries) => {
                for entry in entries {
                    parsed.push((pref.feed_url.clone(), entry));
                }
            }
            Err(e) => errors.push(format!("{}: {e}", pref.feed_url)),
        }
    }

    {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        let conn = db.conn();
        let fetched_at = now_iso();
        for (source_id, entry) in parsed {
            if upsert_item(conn, &source_id, &entry, &fetched_at)? {
                upserted += 1;
            }
        }
        rerank_all(conn)?;
        set_setting(conn, "news_last_refresh_at", &fetched_at)?;
    }

    Ok(NewsRefreshResult {
        fetched_feeds: prefs.len(),
        upserted_items: upserted,
        errors,
    })
}

#[tauri::command]
pub fn list_news(
    state: State<'_, DbState>,
    limit: Option<i64>,
    include_hidden: Option<bool>,
) -> Result<Vec<NewsItem>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let limit = limit.unwrap_or(10).clamp(1, 500);
    let include_hidden = include_hidden.unwrap_or(false);

    let sql = if include_hidden {
        "SELECT n.id, n.source_id, p.title, n.title, n.url, n.summary, n.published_at,
                n.fetched_at, n.score, n.liked, n.hidden
         FROM news_items n
         LEFT JOIN news_prefs p ON p.feed_url = n.source_id
         ORDER BY n.score DESC, n.published_at DESC
         LIMIT ?1"
    } else {
        "SELECT n.id, n.source_id, p.title, n.title, n.url, n.summary, n.published_at,
                n.fetched_at, n.score, n.liked, n.hidden
         FROM news_items n
         LEFT JOIN news_prefs p ON p.feed_url = n.source_id
         WHERE n.hidden = 0
           AND (p.id IS NULL OR (p.enabled = 1 AND p.muted = 0))
         ORDER BY n.score DESC, n.published_at DESC
         LIMIT ?1"
    };

    let mut stmt = db.conn().prepare(sql)?;
    let items = stmt
        .query_map(params![limit], |row| {
            Ok(NewsItem {
                id: row.get(0)?,
                source_id: row.get(1)?,
                source_title: row.get(2)?,
                title: row.get(3)?,
                url: row.get(4)?,
                summary: row.get(5)?,
                published_at: row.get(6)?,
                fetched_at: row.get(7)?,
                score: row.get(8)?,
                liked: row.get::<_, i64>(9)? != 0,
                hidden: row.get::<_, i64>(10)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

#[tauri::command]
pub fn news_feedback(
    state: State<'_, DbState>,
    input: NewsFeedbackInput,
) -> Result<NewsItem, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let conn = db.conn();
    let action = input.action.trim().to_ascii_lowercase();

    let (source_id, title) = conn.query_row(
        "SELECT source_id, title FROM news_items WHERE id = ?1",
        params![input.item_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    )?;

    match action.as_str() {
        "like" => {
            conn.execute(
                "UPDATE news_items SET liked = 1, hidden = 0 WHERE id = ?1",
                params![input.item_id],
            )?;
            adjust_topic_affinity(conn, &title, TOPIC_LIKE_DELTA)?;
            bump_source_weight(conn, &source_id, LIKE_SOURCE_BUMP)?;
        }
        "hide" => {
            conn.execute(
                "UPDATE news_items SET hidden = 1, liked = 0 WHERE id = ?1",
                params![input.item_id],
            )?;
            adjust_topic_affinity(conn, &title, -TOPIC_HIDE_DELTA)?;
            bump_source_weight(conn, &source_id, -HIDE_SOURCE_DROP)?;
        }
        "follow_source" => {
            ensure_pref_row(conn, &source_id)?;
            conn.execute(
                "UPDATE news_prefs
                 SET muted = 0, enabled = 1,
                     weight = MAX(weight + ?2, 1.0)
                 WHERE feed_url = ?1",
                params![source_id, FOLLOW_WEIGHT_BUMP],
            )?;
        }
        "mute_source" => {
            ensure_pref_row(conn, &source_id)?;
            conn.execute(
                "UPDATE news_prefs SET muted = 1 WHERE feed_url = ?1",
                params![source_id],
            )?;
        }
        other => {
            return Err(DbError::Message(format!(
                "unknown news feedback action: {other}"
            )));
        }
    }

    rerank_all(conn)?;
    get_news_item(conn, input.item_id)
}

#[tauri::command]
pub fn open_news_item(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let url: String = db.conn().query_row(
        "SELECT url FROM news_items WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    open_with_system("url", &url)
}

#[tauri::command]
pub fn rerank_news(state: State<'_, DbState>) -> Result<usize, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    rerank_all(db.conn())
}

#[tauri::command]
pub fn get_news_last_refresh(state: State<'_, DbState>) -> Result<Option<String>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    get_setting(db.conn(), "news_last_refresh_at")
}

fn http_client() -> Result<reqwest::blocking::Client, DbError> {
    reqwest::blocking::Client::builder()
        .user_agent("MainstreamLifeOS/0.1 (+local; RSS reader)")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| DbError::Message(format!("http client: {e}")))
}

fn seed_feeds_if_empty(conn: &Connection) -> Result<usize, DbError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM news_prefs", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(0);
    }
    let feeds: Vec<DefaultFeed> = serde_json::from_str(DEFAULT_FEEDS_JSON)
        .map_err(|e| DbError::Message(format!("invalid feeds.default.json: {e}")))?;
    let mut inserted = 0usize;
    for feed in feeds {
        let url = feed.url.trim();
        if url.is_empty() {
            continue;
        }
        conn.execute(
            "INSERT OR IGNORE INTO news_prefs (feed_url, title, weight, enabled, muted)
             VALUES (?1, ?2, ?3, 1, 0)",
            params![url, feed.title, feed.weight],
        )?;
        inserted += 1;
    }
    Ok(inserted)
}

fn load_enabled_prefs(conn: &Connection) -> Result<Vec<FeedPref>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT feed_url, title, weight, enabled, muted
         FROM news_prefs
         WHERE enabled = 1 AND muted = 0
         ORDER BY id",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(FeedPref {
                feed_url: row.get(0)?,
                title: row.get(1)?,
                weight: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
                muted: row.get::<_, i64>(4)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

struct ParsedEntry {
    title: String,
    url: String,
    summary: Option<String>,
    published_at: Option<String>,
}

fn fetch_feed(
    client: &reqwest::blocking::Client,
    feed_url: &str,
) -> Result<Vec<ParsedEntry>, DbError> {
    let bytes = client
        .get(feed_url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.bytes())
        .map_err(|e| DbError::Message(format!("fetch failed: {e}")))?;

    let feed = parser::parse(bytes.as_ref())
        .map_err(|e| DbError::Message(format!("parse failed: {e}")))?;

    let mut out = Vec::new();
    for entry in feed.entries {
        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Untitled".into());

        let url = entry
            .links
            .iter()
            .find(|l| l.rel.as_deref() == Some("alternate"))
            .or_else(|| entry.links.first())
            .map(|l| l.href.clone())
            .or_else(|| {
                let id = entry.id.trim();
                if id.is_empty() {
                    None
                } else {
                    Some(id.to_string())
                }
            })
            .unwrap_or_default();
        let url = url.trim().to_string();
        if url.is_empty() {
            continue;
        }

        let summary = entry
            .summary
            .as_ref()
            .map(|s| strip_tags(&s.content))
            .or_else(|| {
                entry
                    .content
                    .as_ref()
                    .and_then(|c| c.body.as_ref().map(|b| strip_tags(b)))
            })
            .map(|s| truncate(&s, 280));

        let published_at = entry
            .published
            .or(entry.updated)
            .map(|dt| dt.to_rfc3339());

        out.push(ParsedEntry {
            title,
            url,
            summary,
            published_at,
        });
    }
    Ok(out)
}

fn upsert_item(
    conn: &Connection,
    source_id: &str,
    entry: &ParsedEntry,
    fetched_at: &str,
) -> Result<bool, DbError> {
    let changed = conn.execute(
        "INSERT INTO news_items (source_id, title, url, summary, published_at, fetched_at, score, liked, hidden)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, 0)
         ON CONFLICT(url) DO UPDATE SET
           source_id = excluded.source_id,
           title = excluded.title,
           summary = COALESCE(excluded.summary, news_items.summary),
           published_at = COALESCE(excluded.published_at, news_items.published_at),
           fetched_at = excluded.fetched_at",
        params![
            source_id,
            entry.title,
            entry.url,
            entry.summary,
            entry.published_at,
            fetched_at
        ],
    )?;
    Ok(changed > 0)
}

fn rerank_all(conn: &Connection) -> Result<usize, DbError> {
    let prefs = {
        let mut stmt =
            conn.prepare("SELECT feed_url, weight, enabled, muted FROM news_prefs")?;
        let map: HashMap<String, (f64, bool, bool)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)? != 0,
                    row.get::<_, i64>(3)? != 0,
                ))
            })?
            .filter_map(|r| r.ok())
            .map(|(url, w, enabled, muted)| (url, (w, enabled, muted)))
            .collect();
        map
    };

    let topics = load_topic_map(conn)?;
    let now = Utc::now();

    let mut stmt = conn.prepare(
        "SELECT id, source_id, title, published_at, fetched_at, hidden FROM news_items",
    )?;
    let rows: Vec<(i64, String, String, Option<String>, String, bool)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get::<_, i64>(5)? != 0,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    let mut updated = 0usize;
    for (id, source_id, title, published_at, fetched_at, hidden) in rows {
        let (weight, enabled, muted) = prefs
            .get(&source_id)
            .copied()
            .unwrap_or((1.0, true, false));

        let score = if hidden || muted || !enabled {
            0.0
        } else {
            let when = parse_time(published_at.as_deref())
                .or_else(|| parse_time(Some(&fetched_at)))
                .unwrap_or(now);
            let age_hours = (now - when).num_seconds().max(0) as f64 / 3600.0;
            let recency = 0.5_f64.powf(age_hours / RECENCY_HALF_LIFE_HOURS);
            let topic = topic_affinity(&title, &topics);
            (recency * weight.max(0.05) * topic).max(0.0)
        };

        conn.execute(
            "UPDATE news_items SET score = ?1 WHERE id = ?2",
            params![score, id],
        )?;
        updated += 1;
    }
    Ok(updated)
}

/// score = recency × source_weight × topic_affinity
/// recency = 0.5 ^ (age_hours / 36)
/// topic_affinity = clamp(1 + Σ term scores in title, 0.25..3)
fn topic_affinity(title: &str, topics: &HashMap<String, f64>) -> f64 {
    let mut affinity = 1.0;
    for term in tokenize(title) {
        if let Some(delta) = topics.get(&term) {
            affinity += delta;
        }
    }
    affinity.clamp(TOPIC_AFFINITY_MIN, TOPIC_AFFINITY_MAX)
}

fn load_topic_map(conn: &Connection) -> Result<HashMap<String, f64>, DbError> {
    let mut stmt = conn.prepare("SELECT term, score FROM news_topic_affinity")?;
    let map = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(map)
}

fn adjust_topic_affinity(conn: &Connection, title: &str, delta: f64) -> Result<(), DbError> {
    for term in tokenize(title) {
        conn.execute(
            "INSERT INTO news_topic_affinity (term, score) VALUES (?1, ?2)
             ON CONFLICT(term) DO UPDATE SET score = news_topic_affinity.score + excluded.score",
            params![term, delta],
        )?;
    }
    // Soft-clamp extreme affinities in storage
    conn.execute(
        "UPDATE news_topic_affinity SET score = MAX(MIN(score, 2.0), -2.0)",
        [],
    )?;
    Ok(())
}

fn bump_source_weight(conn: &Connection, feed_url: &str, delta: f64) -> Result<(), DbError> {
    ensure_pref_row(conn, feed_url)?;
    conn.execute(
        "UPDATE news_prefs
         SET weight = MAX(0.1, MIN(3.0, weight + ?2))
         WHERE feed_url = ?1",
        params![feed_url, delta],
    )?;
    Ok(())
}

fn ensure_pref_row(conn: &Connection, feed_url: &str) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR IGNORE INTO news_prefs (feed_url, title, weight, enabled, muted)
         VALUES (?1, NULL, 1.0, 1, 0)",
        params![feed_url],
    )?;
    Ok(())
}

fn get_news_item(conn: &Connection, id: i64) -> Result<NewsItem, DbError> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.source_id, p.title, n.title, n.url, n.summary, n.published_at,
                n.fetched_at, n.score, n.liked, n.hidden
         FROM news_items n
         LEFT JOIN news_prefs p ON p.feed_url = n.source_id
         WHERE n.id = ?1",
    )?;
    let item = stmt.query_row(params![id], |row| {
        Ok(NewsItem {
            id: row.get(0)?,
            source_id: row.get(1)?,
            source_title: row.get(2)?,
            title: row.get(3)?,
            url: row.get(4)?,
            summary: row.get(5)?,
            published_at: row.get(6)?,
            fetched_at: row.get(7)?,
            score: row.get(8)?,
            liked: row.get::<_, i64>(9)? != 0,
            hidden: row.get::<_, i64>(10)? != 0,
        })
    })?;
    Ok(item)
}

fn tokenize(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "of", "to", "in", "on", "for", "with", "at",
        "by", "from", "as", "is", "are", "was", "were", "be", "this", "that", "it", "its",
        "new", "how", "why", "what", "when", "who", "your", "you", "we", "our", "their",
    ];
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !STOP.contains(w))
        .take(12)
        .map(|w| w.to_string())
        .collect()
}

fn parse_time(value: Option<&str>) -> Option<DateTime<Utc>> {
    let value = value?;
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|ndt| ndt.and_utc())
        })
}

fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{truncated}…")
}
