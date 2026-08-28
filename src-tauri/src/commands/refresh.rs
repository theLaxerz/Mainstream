//! Orchestrates a full dashboard refresh across all sync-capable modules.

use crate::commands::email::{read_settings, sync_imap};
use crate::commands::health::try_import_configured;
use crate::commands::home::fetch_home_devices;
use crate::commands::mail::sync_informed_delivery;
use crate::commands::news::run_refresh_news;
use crate::commands::streaming::{load_api_key, load_enabled_providers, run_refresh_streaming};
use crate::commands::youtube::run_refresh_youtube;
use crate::db::{now_iso, DbError, DbState};
use rusqlite::Connection;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleRefreshResult {
    pub module: String,
    /// `"ok"`, `"skipped"`, or `"error"`.
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardRefreshResult {
    pub started_at: String,
    pub finished_at: String,
    pub modules: Vec<ModuleRefreshResult>,
}

fn push_ok(modules: &mut Vec<ModuleRefreshResult>, module: &str, detail: impl Into<String>) {
    modules.push(ModuleRefreshResult {
        module: module.into(),
        status: "ok".into(),
        detail: Some(detail.into()),
    });
}

fn push_skipped(modules: &mut Vec<ModuleRefreshResult>, module: &str, detail: impl Into<String>) {
    modules.push(ModuleRefreshResult {
        module: module.into(),
        status: "skipped".into(),
        detail: Some(detail.into()),
    });
}

fn push_error(modules: &mut Vec<ModuleRefreshResult>, module: &str, detail: impl Into<String>) {
    modules.push(ModuleRefreshResult {
        module: module.into(),
        status: "error".into(),
        detail: Some(detail.into()),
    });
}

fn email_configured(conn: &Connection) -> bool {
    read_settings(conn)
        .map(|s| {
            !s.host.trim().is_empty() && !s.user.trim().is_empty() && s.has_password
        })
        .unwrap_or(false)
}

fn youtube_configured(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM youtube_prefs WHERE enabled = 1",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)
    .unwrap_or(false)
}

fn streaming_configured(conn: &Connection) -> bool {
    load_api_key(conn).is_ok() && !load_enabled_providers(conn).is_empty()
}

fn refresh_email(state: &DbState, modules: &mut Vec<ModuleRefreshResult>) {
    let configured = {
        let db = match state.lock() {
            Ok(db) => db,
            Err(e) => {
                push_error(modules, "email", e.to_string());
                return;
            }
        };
        email_configured(db.conn())
    };

    if !configured {
        push_skipped(modules, "email", "IMAP not configured");
        return;
    }

    let result = {
        let db = match state.lock() {
            Ok(db) => db,
            Err(e) => {
                push_error(modules, "email", e.to_string());
                return;
            }
        };
        sync_imap(db.conn())
    };

    match result {
        Ok(r) => push_ok(
            modules,
            "email",
            format!(
                "Synced {} message(s) · {} important",
                r.fetched, r.important
            ),
        ),
        Err(e) => push_error(modules, "email", e.to_string()),
    }
}

fn refresh_mail(state: &DbState, modules: &mut Vec<ModuleRefreshResult>) {
    let (configured, db_path) = {
        let db = match state.lock() {
            Ok(db) => db,
            Err(e) => {
                push_error(modules, "mail", e.to_string());
                return;
            }
        };
        (email_configured(db.conn()), db.path().to_path_buf())
    };

    if !configured {
        push_skipped(modules, "mail", "Configure Email (IMAP) first");
        return;
    }

    let result = {
        let db = match state.lock() {
            Ok(db) => db,
            Err(e) => {
                push_error(modules, "mail", e.to_string());
                return;
            }
        };
        sync_informed_delivery(db.conn(), &db_path)
    };

    match result {
        Ok(r) => push_ok(
            modules,
            "mail",
            format!(
                "{} piece(s) from {} digest(s) · {} OCR",
                r.pieces, r.digests, r.ocr_ran
            ),
        ),
        Err(e) => push_error(modules, "mail", e.to_string()),
    }
}

fn refresh_news_module(state: &DbState, modules: &mut Vec<ModuleRefreshResult>) {
    match run_refresh_news(state) {
        Ok(r) => {
            let detail = if r.errors.is_empty() {
                format!(
                    "Updated {} stories from {} feed(s)",
                    r.upserted_items, r.fetched_feeds
                )
            } else {
                format!(
                    "Updated {} stories · {} feed error(s)",
                    r.upserted_items,
                    r.errors.len()
                )
            };
            push_ok(modules, "news", detail);
        }
        Err(e) => push_error(modules, "news", e.to_string()),
    }
}

fn refresh_youtube_module(state: &DbState, modules: &mut Vec<ModuleRefreshResult>) {
    let configured = {
        let db = match state.lock() {
            Ok(db) => db,
            Err(e) => {
                push_error(modules, "youtube", e.to_string());
                return;
            }
        };
        youtube_configured(db.conn())
    };

    if !configured {
        push_skipped(modules, "youtube", "No channels followed");
        return;
    }

    match run_refresh_youtube(state) {
        Ok(r) => {
            let detail = if r.errors.is_empty() {
                format!("Fetched {} new video(s) from {} channel(s)", r.upserted, r.channels)
            } else {
                format!(
                    "Fetched {} video(s) · {} channel error(s)",
                    r.upserted,
                    r.errors.len()
                )
            };
            push_ok(modules, "youtube", detail);
        }
        Err(e) => push_error(modules, "youtube", e.to_string()),
    }
}

fn refresh_streaming_module(state: &DbState, modules: &mut Vec<ModuleRefreshResult>) {
    let configured = {
        let db = match state.lock() {
            Ok(db) => db,
            Err(e) => {
                push_error(modules, "streaming", e.to_string());
                return;
            }
        };
        streaming_configured(db.conn())
    };

    if !configured {
        push_skipped(modules, "streaming", "TMDB API key or providers not configured");
        return;
    }

    match run_refresh_streaming(state) {
        Ok(r) => {
            let detail = if r.errors.is_empty() {
                format!("Updated {} title(s) across {} service(s)", r.upserted, r.providers)
            } else {
                format!(
                    "Updated {} title(s) · {} provider error(s)",
                    r.upserted,
                    r.errors.len()
                )
            };
            push_ok(modules, "streaming", detail);
        }
        Err(e) => push_error(modules, "streaming", e.to_string()),
    }
}

fn refresh_health_module(state: &DbState, modules: &mut Vec<ModuleRefreshResult>) {
    let result = {
        let db = match state.lock() {
            Ok(db) => db,
            Err(e) => {
                push_error(modules, "health", e.to_string());
                return;
            }
        };
        try_import_configured(db.conn())
    };

    match result {
        Ok(Some(days)) => push_ok(modules, "health", format!("Imported {days} day(s)")),
        Ok(None) => push_skipped(modules, "health", "No export path or file missing"),
        Err(e) => push_error(modules, "health", e.to_string()),
    }
}

fn refresh_home_module(state: &DbState, modules: &mut Vec<ModuleRefreshResult>) {
    match fetch_home_devices(state, None) {
        Ok(Some(devices)) => push_ok(
            modules,
            "home",
            format!("{} device(s) online", devices.len()),
        ),
        Ok(None) => push_skipped(modules, "home", "Ring/Blink not configured"),
        Err(e) => push_error(modules, "home", e.to_string()),
    }
}

#[tauri::command]
pub fn refresh_dashboard(state: State<'_, DbState>) -> Result<DashboardRefreshResult, DbError> {
    let started_at = now_iso();
    let mut modules = Vec::new();

    // Order matters: email before physical mail.
    refresh_email(&state, &mut modules);
    refresh_mail(&state, &mut modules);
    refresh_news_module(&state, &mut modules);
    refresh_youtube_module(&state, &mut modules);
    refresh_streaming_module(&state, &mut modules);
    refresh_health_module(&state, &mut modules);
    refresh_home_module(&state, &mut modules);

    push_skipped(
        &mut modules,
        "messages",
        "Reloads from local Messages database",
    );
    push_skipped(
        &mut modules,
        "calendar",
        "Reloads from macOS Calendar",
    );
    push_skipped(&mut modules, "finance", "Local ledger — no remote sync");
    push_skipped(&mut modules, "notes", "Local notes — no remote sync");
    push_skipped(&mut modules, "shortcuts", "Local shortcuts — no remote sync");

    Ok(DashboardRefreshResult {
        started_at,
        finished_at: now_iso(),
        modules,
    })
}
