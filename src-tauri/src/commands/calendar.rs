//! macOS Calendar (EventKit) — upcoming events via a local Swift helper.

use crate::commands::open::open_with_system;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

const APPLE_EPOCH_UNIX: i64 = 978_307_200;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarAccess {
    /// `"ok"`, `"needs_permission"`, or `"unavailable"`.
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub start: String,
    pub end: String,
    pub is_all_day: bool,
    pub location: Option<String>,
    pub calendar_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CalendarScriptError {
    error: String,
    status: Option<String>,
}

fn calendar_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/fetch_calendar_events.swift")
}

fn run_calendar_script(days_back: i64, days_ahead: i64) -> Result<Vec<CalendarEvent>, CalendarAccess> {
    let script = calendar_script_path();
    if !script.exists() {
        return Err(CalendarAccess {
            status: "unavailable".into(),
            detail: Some("Calendar helper script is missing from the app bundle.".into()),
        });
    }

    let output = Command::new("swift")
        .arg(&script)
        .arg(days_back.to_string())
        .arg(days_ahead.to_string())
        .output()
        .map_err(|e| CalendarAccess {
            status: "unavailable".into(),
            detail: Some(format!("Failed to run Calendar helper: {e}")),
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() {
        if let Ok(err) = serde_json::from_str::<CalendarScriptError>(&stdout) {
            let status = err.status.unwrap_or_else(|| "error".into());
            if status == "needs_permission" {
                return Err(CalendarAccess {
                    status: "needs_permission".into(),
                    detail: Some(err.error),
                });
            }
            return Err(CalendarAccess {
                status: "error".into(),
                detail: Some(err.error),
            });
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CalendarAccess {
            status: "error".into(),
            detail: Some(if stderr.is_empty() {
                "Calendar helper exited with an error.".into()
            } else {
                stderr
            }),
        });
    }

    serde_json::from_str(&stdout).map_err(|e| CalendarAccess {
        status: "error".into(),
        detail: Some(format!("Failed to parse calendar events: {e}")),
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventsResult {
    pub access: CalendarAccess,
    pub events: Vec<CalendarEvent>,
}

fn fetch_events(days_back: i64, days_ahead: i64) -> CalendarEventsResult {
    match run_calendar_script(days_back, days_ahead) {
        Ok(events) => CalendarEventsResult {
            access: CalendarAccess {
                status: "ok".into(),
                detail: None,
            },
            events,
        },
        Err(access) => CalendarEventsResult {
            access,
            events: Vec::new(),
        },
    }
}

#[tauri::command]
pub fn calendar_access_status() -> CalendarAccess {
    match run_calendar_script(0, 1) {
        Ok(_) => CalendarAccess {
            status: "ok".into(),
            detail: None,
        },
        Err(access) => access,
    }
}

#[tauri::command]
pub fn list_calendar_events(
    limit: Option<i64>,
    days_ahead: Option<i64>,
    days_back: Option<i64>,
) -> CalendarEventsResult {
    let limit = limit.unwrap_or(12).clamp(1, 200);
    let days_ahead = days_ahead.unwrap_or(14).clamp(1, 90);
    let days_back = days_back.unwrap_or(0).clamp(0, 90);
    let mut result = fetch_events(days_back, days_ahead);
    result.events.truncate(limit as usize);
    result
}

#[tauri::command]
pub fn open_calendar_privacy_settings() -> Result<(), String> {
    let status = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Calendars")
        .status()
        .map_err(|e| format!("failed to open Calendar privacy settings: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open exited with status {status}"))
    }
}

#[tauri::command]
pub fn open_calendar_event(start_iso: String) -> Result<(), String> {
    let trimmed = start_iso.trim();
    if trimmed.is_empty() {
        return Err("Event start time is required.".into());
    }

    let parsed = DateTime::parse_from_rfc3339(trimmed)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            trimmed
                .parse::<DateTime<Utc>>()
                .map_err(|e| format!("invalid date: {e}"))
        })?;

    let calshow = parsed.timestamp() - APPLE_EPOCH_UNIX;
    let url = format!("calshow:{calshow}");
    open_with_system("url", &url).map_err(|e| e.to_string())
}
