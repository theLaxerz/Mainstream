//! Read-only access to macOS Messages `chat.db` (requires Full Disk Access).

use crate::db::DbError;
use crate::security::validate_imessage_ref;
use chrono::{TimeZone, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

const APPLE_EPOCH_UNIX: i64 = 978_307_200;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagesAccess {
    /// `"ok"`, `"needs_permission"`, or `"unavailable"`.
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadMessage {
    pub message_id: i64,
    pub chat_id: i64,
    pub chat_guid: String,
    pub chat_identifier: String,
    pub display_name: String,
    pub handle: Option<String>,
    pub text: String,
    /// ISO-8601 UTC timestamp.
    pub date: String,
    pub is_group: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadMessagesResult {
    pub access: MessagesAccess,
    pub messages: Vec<UnreadMessage>,
}

fn chat_db_path() -> Result<PathBuf, DbError> {
    let home = std::env::var_os("HOME").ok_or_else(|| {
        DbError::Message("HOME is not set; cannot locate Messages database".into())
    })?;
    Ok(PathBuf::from(home).join("Library/Messages/chat.db"))
}

fn is_permission_err(err: &std::io::Error) -> bool {
    if matches!(err.kind(), ErrorKind::PermissionDenied) {
        return true;
    }
    let msg = err.to_string().to_lowercase();
    msg.contains("operation not permitted") || msg.contains("authorization denied")
}

fn permission_result(detail: impl Into<String>) -> UnreadMessagesResult {
    UnreadMessagesResult {
        access: MessagesAccess {
            status: "needs_permission".into(),
            detail: Some(detail.into()),
        },
        messages: vec![],
    }
}

fn unavailable_result(detail: impl Into<String>) -> UnreadMessagesResult {
    UnreadMessagesResult {
        access: MessagesAccess {
            status: "unavailable".into(),
            detail: Some(detail.into()),
        },
        messages: vec![],
    }
}

/// Copy chat.db (+ WAL/SHM when present) into a private temp dir so we can read
/// while Messages.app holds the live database open.
fn stage_chat_db(src: &Path) -> Result<PathBuf, DbError> {
    let parent = src
        .parent()
        .ok_or_else(|| DbError::Message("invalid Messages database path".into()))?;

    let staging = std::env::temp_dir().join(format!(
        "mainstream-messages-{}-{}",
        std::process::id(),
        Utc::now().timestamp_millis()
    ));
    fs::create_dir_all(&staging)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&staging, fs::Permissions::from_mode(0o700))?;
    }

    let dst = staging.join("chat.db");
    fs::copy(src, &dst)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dst, fs::Permissions::from_mode(0o600));
    }

    for suffix in ["chat.db-wal", "chat.db-shm"] {
        let side = parent.join(suffix);
        if side.exists() {
            let _ = fs::copy(&side, staging.join(suffix));
        }
    }

    Ok(dst)
}

fn cleanup_staging(db_path: &Path) {
    if let Some(dir) = db_path.parent() {
        let _ = fs::remove_dir_all(dir);
    }
}

fn apple_date_to_iso(raw: i64) -> String {
    // Messages stores Cocoa Core Data timestamps; modern rows use nanoseconds.
    let seconds = if raw.abs() > 1_000_000_000_000_000 {
        raw / 1_000_000_000
    } else if raw.abs() > 1_000_000_000_000 {
        raw / 1_000_000
    } else if raw.abs() > 1_000_000_000 {
        raw / 1_000
    } else {
        raw
    };
    let unix = seconds + APPLE_EPOCH_UNIX;
    match Utc.timestamp_opt(unix, 0) {
        chrono::LocalResult::Single(dt) => dt.to_rfc3339(),
        _ => Utc::now().to_rfc3339(),
    }
}

/// Best-effort pull of the NSString payload from an `attributedBody` blob.
fn extract_from_attributed_body(blob: &[u8]) -> Option<String> {
    const MARKER: &[u8] = b"NSString";
    let mut i = 0;
    while i + MARKER.len() < blob.len() {
        if &blob[i..i + MARKER.len()] == MARKER {
            let mut j = i + MARKER.len();
            while j < blob.len() && blob[j] == 0 {
                j += 1;
            }
            // Length-prefixed variants: skip a short length byte when present.
            if j < blob.len() && blob[j] < 0x20 {
                j += 1;
            }
            let start = j;
            let mut end = start;
            while end < blob.len()
                && blob[end] != 0
                && blob[end].is_ascii()
                && !blob[end].is_ascii_control()
            {
                end += 1;
            }
            if end > start + 1 {
                let s = String::from_utf8_lossy(&blob[start..end])
                    .trim()
                    .to_string();
                if !s.is_empty() && s != "NSString" && s != "NSDictionary" {
                    return Some(s);
                }
            }
        }
        i += 1;
    }
    None
}

fn preview_text(text: Option<String>, attributed: Option<Vec<u8>>) -> String {
    if let Some(t) = text {
        let trimmed = t.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if let Some(blob) = attributed.as_deref() {
        if let Some(s) = extract_from_attributed_body(blob) {
            return s;
        }
    }
    "Attachment or unavailable preview".into()
}

fn display_label(
    display_name: Option<String>,
    handle: Option<&str>,
    chat_identifier: &str,
    is_group: bool,
) -> String {
    if let Some(name) = display_name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    if is_group {
        return "Group chat".into();
    }
    if let Some(h) = handle {
        let trimmed = h.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let id = chat_identifier.trim();
    if id.is_empty() {
        "Unknown".into()
    } else {
        id.to_string()
    }
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let sql = format!("PRAGMA table_info({table})");
    let Ok(mut stmt) = conn.prepare(&sql) else {
        return false;
    };
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
        return false;
    };
    let found = rows.flatten().any(|name| name == column);
    found
}

fn query_unread(conn: &Connection, limit: Option<i64>) -> Result<Vec<UnreadMessage>, DbError> {
    let has_is_read = column_exists(conn, "message", "is_read");
    let has_item_type = column_exists(conn, "message", "item_type");
    let has_assoc = column_exists(conn, "message", "associated_message_type");
    let has_style = column_exists(conn, "chat", "style");

    let unread_clause = if has_is_read {
        "(m.is_read = 0 OR (m.is_read IS NULL AND IFNULL(m.date_read, 0) = 0))"
    } else {
        "IFNULL(m.date_read, 0) = 0"
    };

    let mut filters = vec!["m.is_from_me = 0".to_string(), unread_clause.to_string()];
    if has_item_type {
        filters.push("IFNULL(m.item_type, 0) = 0".into());
    }
    if has_assoc {
        filters.push("IFNULL(m.associated_message_type, 0) = 0".into());
    }

    let group_expr = if has_style {
        "CASE WHEN IFNULL(c.style, 0) = 43 THEN 1 ELSE 0 END"
    } else {
        "CASE WHEN c.chat_identifier LIKE 'chat%' THEN 1 ELSE 0 END"
    };

    let limit_sql = if limit.is_some() { "LIMIT ?1" } else { "" };

    let sql = format!(
        "SELECT
            m.ROWID,
            c.ROWID,
            c.guid,
            IFNULL(c.chat_identifier, ''),
            c.display_name,
            h.id,
            m.text,
            m.attributedBody,
            m.date,
            {group_expr} AS is_group
         FROM message m
         JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
         JOIN chat c ON c.ROWID = cmj.chat_id
         LEFT JOIN handle h ON h.ROWID = m.handle_id
         WHERE {where_clause}
         ORDER BY m.date DESC
         {limit_sql}",
        where_clause = filters.join(" AND "),
    );

    let mut stmt = conn.prepare(&sql)?;
    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<UnreadMessage> {
        let text: Option<String> = row.get(6)?;
        let attributed: Option<Vec<u8>> = row.get(7)?;
        let date_raw: i64 = row.get(8)?;
        let is_group: i64 = row.get(9)?;
        let handle: Option<String> = row.get(5)?;
        let chat_identifier: String = row.get(3)?;
        let display_name: Option<String> = row.get(4)?;
        let is_group = is_group != 0;

        Ok(UnreadMessage {
            message_id: row.get(0)?,
            chat_id: row.get(1)?,
            chat_guid: row.get(2)?,
            chat_identifier: chat_identifier.clone(),
            display_name: display_label(
                display_name,
                handle.as_deref(),
                &chat_identifier,
                is_group,
            ),
            handle,
            text: preview_text(text, attributed),
            date: apple_date_to_iso(date_raw),
            is_group,
        })
    };

    let messages = if let Some(limit) = limit {
        stmt.query_map(params![limit], map_row)?
            .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], map_row)?
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(messages)
}

fn load_unread(limit: Option<i64>) -> UnreadMessagesResult {
    let src = match chat_db_path() {
        Ok(p) => p,
        Err(e) => return unavailable_result(e.to_string()),
    };

    match fs::metadata(&src) {
        Ok(_) => {}
        Err(e) if is_permission_err(&e) => {
            return permission_result(
                "Mainstream needs Full Disk Access to read your Messages database.",
            );
        }
        Err(e) => {
            return unavailable_result(format!(
                "Messages database not found at {}: {e}",
                src.display()
            ));
        }
    }

    let staged = match stage_chat_db(&src) {
        Ok(p) => p,
        Err(DbError::Io(e)) if is_permission_err(&e) => {
            return permission_result(
                "Mainstream needs Full Disk Access to read your Messages database.",
            );
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("permission")
                || msg.to_lowercase().contains("authorization")
                || msg.to_lowercase().contains("operation not permitted")
            {
                return permission_result(
                    "Mainstream needs Full Disk Access to read your Messages database.",
                );
            }
            return unavailable_result(msg);
        }
    };

    let result = (|| -> Result<Vec<UnreadMessage>, DbError> {
        let conn = Connection::open(&staged)?;
        // Ensure we don't try to write to the staged copy accidentally.
        conn.pragma_update(None, "query_only", true)?;
        query_unread(&conn, limit)
    })();

    cleanup_staging(&staged);

    match result {
        Ok(messages) => UnreadMessagesResult {
            access: MessagesAccess {
                status: "ok".into(),
                detail: None,
            },
            messages,
        },
        Err(e) => {
            let msg = e.to_string();
            if msg.to_lowercase().contains("permission")
                || msg.to_lowercase().contains("authorization")
                || msg.to_lowercase().contains("operation not permitted")
            {
                permission_result(
                    "Mainstream needs Full Disk Access to read your Messages database.",
                )
            } else {
                unavailable_result(msg)
            }
        }
    }
}

/// Top unread Messages (newest first). Pass `limit` (default 10) or use
/// [`list_all_unread_messages`] for the full list.
#[tauri::command]
pub fn list_unread_messages(limit: Option<i64>) -> UnreadMessagesResult {
    let limit = limit.unwrap_or(10).clamp(0, 200);
    load_unread(Some(limit))
}

/// Full unread Messages list, newest first (capped at 500).
#[tauri::command]
pub fn list_all_unread_messages() -> UnreadMessagesResult {
    load_unread(Some(500))
}

/// Probe whether `chat.db` is readable (Full Disk Access).
#[tauri::command]
pub fn messages_access_status() -> MessagesAccess {
    load_unread(Some(0)).access
}

/// Open System Settings to the Full Disk Access privacy pane.
#[tauri::command]
pub fn open_full_disk_access_settings() -> Result<(), DbError> {
    // Prefer the modern Privacy & Security deep link; fall back to the legacy pane.
    let urls = [
        "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_AllFiles",
        "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
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

    Err(DbError::Message(last_err.unwrap_or_else(|| {
        "failed to open Full Disk Access settings".into()
    })))
}

fn encode_imessage_target(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'+' | b'@' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Open a conversation in Messages.app using chat identifier / guid.
#[tauri::command]
pub fn open_message_conversation(
    chat_identifier: String,
    chat_guid: Option<String>,
) -> Result<(), DbError> {
    let identifier = chat_identifier.trim();
    let guid = chat_guid
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if identifier.is_empty() && guid.is_none() {
        return Err(DbError::Message(
            "chat_identifier or chat_guid is required".into(),
        ));
    }
    let identifier = if identifier.is_empty() {
        String::new()
    } else {
        validate_imessage_ref(identifier)?
    };
    let guid = match guid {
        Some(g) => Some(validate_imessage_ref(g)?),
        None => None,
    };
    let guid = guid.as_deref();

    // 1:1 chats: imessage://address works reliably.
    // Group chats: try guid-based URL, then fall back to identifier.
    let candidates: Vec<String> = if identifier.starts_with("chat") || identifier.is_empty() {
        let mut v = Vec::new();
        if let Some(g) = guid {
            v.push(format!("imessage://{}", encode_imessage_target(g)));
        }
        if !identifier.is_empty() {
            v.push(format!(
                "imessage://{}",
                encode_imessage_target(&identifier)
            ));
        }
        v
    } else {
        let mut v = vec![format!(
            "imessage://{}",
            encode_imessage_target(&identifier)
        )];
        if let Some(g) = guid {
            v.push(format!("imessage://{}", encode_imessage_target(g)));
        }
        v
    };

    for url in &candidates {
        if let Ok(status) = Command::new("open").arg(url).status() {
            if status.success() {
                return Ok(());
            }
        }
    }

    // Last resort: bring Messages.app forward.
    let status = Command::new("open")
        .args(["-a", "Messages"])
        .status()
        .map_err(|e| DbError::Message(format!("failed to open Messages: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(DbError::Message(format!(
            "failed to open conversation (open exited with {status})"
        )))
    }
}
