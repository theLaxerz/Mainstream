//! macOS Calendar (EventKit) — upcoming events via an in-process EventKit bridge.
//!
//! Access is requested from the Mainstream process itself so TCC attributes the
//! prompt to this app (and Mainstream appears under Privacy → Calendars).

use crate::commands::open::open_with_system;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ffi::{c_char, CStr};
use std::process::Command;

const APPLE_EPOCH_UNIX: i64 = 978_307_200;
const CAL_OK: i32 = 0;
const CAL_NEEDS_PERMISSION: i32 = 1;

extern "C" {
    fn mainstream_calendar_events(
        days_back: i64,
        days_ahead: i64,
        json_out: *mut *mut c_char,
        error_out: *mut *mut c_char,
    ) -> i32;
    fn mainstream_calendar_string_free(s: *mut c_char);
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarAccess {
    /// `"ok"`, `"needs_permission"`, `"unavailable"`, or `"error"`.
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

fn take_cstring(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let value = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        mainstream_calendar_string_free(ptr);
        Some(value)
    }
}

fn fetch_events(days_back: i64, days_ahead: i64) -> CalendarEventsResult {
    let mut json_ptr: *mut c_char = std::ptr::null_mut();
    let mut error_ptr: *mut c_char = std::ptr::null_mut();
    let code = unsafe {
        mainstream_calendar_events(days_back, days_ahead, &mut json_ptr, &mut error_ptr)
    };
    let json = take_cstring(json_ptr);
    let detail = take_cstring(error_ptr);

    if code == CAL_OK {
        match json
            .as_deref()
            .map(serde_json::from_str::<Vec<CalendarEvent>>)
        {
            Some(Ok(events)) => CalendarEventsResult {
                access: CalendarAccess {
                    status: "ok".into(),
                    detail: None,
                },
                events,
            },
            Some(Err(e)) => CalendarEventsResult {
                access: CalendarAccess {
                    status: "error".into(),
                    detail: Some(format!("Failed to parse calendar events: {e}")),
                },
                events: Vec::new(),
            },
            None => CalendarEventsResult {
                access: CalendarAccess {
                    status: "error".into(),
                    detail: Some("Calendar helper returned no data.".into()),
                },
                events: Vec::new(),
            },
        }
    } else if code == CAL_NEEDS_PERMISSION {
        CalendarEventsResult {
            access: CalendarAccess {
                status: "needs_permission".into(),
                detail: detail.or_else(|| Some("Calendar access denied".into())),
            },
            events: Vec::new(),
        }
    } else {
        CalendarEventsResult {
            access: CalendarAccess {
                status: "error".into(),
                detail: detail.or_else(|| Some("Calendar helper exited with an error.".into())),
            },
            events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventsResult {
    pub access: CalendarAccess,
    pub events: Vec<CalendarEvent>,
}

#[tauri::command]
pub fn calendar_access_status() -> CalendarAccess {
    fetch_events(0, 1).access
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
    // Prefer the modern Privacy & Security deep link; fall back to the legacy pane.
    let urls = [
        "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Calendars",
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Calendars",
    ];

    let mut last_err = None;
    for url in urls {
        match Command::new("open").arg(url).status() {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => {
                last_err = Some(format!("open exited with status {status}"));
            }
            Err(e) => last_err = Some(e.to_string()),
        }
    }

    Err(last_err.unwrap_or_else(|| "failed to open Calendar privacy settings".into()))
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
