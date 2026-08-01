use crate::commands::open::open_with_system;
use crate::db::{get_setting, now_iso, set_setting, DbError, DbState};
use imap::types::Fetch;
use keyring::Entry;
use mailparse::{addrparse, parse_mail, MailHeaderMap};
use native_tls::TlsConnector;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use tauri::State;

const KEYCHAIN_SERVICE: &str = "com.mainstream.lifeos.imap";
const SETTING_HOST: &str = "email.imap_host";
const SETTING_PORT: &str = "email.imap_port";
const SETTING_USER: &str = "email.imap_user";
const SETTING_MAILBOX: &str = "email.imap_mailbox";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailMessage {
    pub id: i64,
    pub uid: i64,
    pub mailbox: String,
    pub message_id: Option<String>,
    pub from_addr: Option<String>,
    pub from_name: Option<String>,
    pub to_addrs: String,
    pub subject: String,
    pub preview: String,
    pub date_iso: Option<String>,
    pub is_unread: bool,
    pub is_important: bool,
    pub importance_score: f64,
    pub has_list_unsubscribe: bool,
    pub is_junk: bool,
    pub message_url: Option<String>,
    pub synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailSettings {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub mailbox: String,
    pub has_password: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveEmailSettingsInput {
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub mailbox: Option<String>,
    /// When `None` or empty, keep the existing Keychain password.
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailSyncResult {
    pub fetched: usize,
    pub important: usize,
    pub mailbox: String,
}

pub(crate) struct ParsedHeaders {
    pub(crate) message_id: Option<String>,
    pub(crate) from_addr: Option<String>,
    pub(crate) from_name: Option<String>,
    pub(crate) to_addrs: String,
    pub(crate) subject: String,
    pub(crate) preview: String,
    pub(crate) date_iso: Option<String>,
    pub(crate) has_list_unsubscribe: bool,
    pub(crate) is_newsletter: bool,
}

fn keychain_entry(user: &str) -> Result<Entry, DbError> {
    let account = if user.trim().is_empty() {
        "imap"
    } else {
        user.trim()
    };
    Entry::new(KEYCHAIN_SERVICE, account)
        .map_err(|e| DbError::Message(format!("keychain entry failed: {e}")))
}

fn store_password(user: &str, password: &str) -> Result<(), DbError> {
    keychain_entry(user)?
        .set_password(password)
        .map_err(|e| DbError::Message(format!("failed to store IMAP password in Keychain: {e}")))
}

pub(crate) fn load_password(user: &str) -> Result<Option<String>, DbError> {
    match keychain_entry(user)?.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(DbError::Message(format!(
            "failed to read IMAP password from Keychain: {e}"
        ))),
    }
}

fn delete_password(user: &str) -> Result<(), DbError> {
    match keychain_entry(user)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(DbError::Message(format!(
            "failed to delete IMAP password from Keychain: {e}"
        ))),
    }
}

pub(crate) fn read_settings(conn: &Connection) -> Result<EmailSettings, DbError> {
    let host = get_setting(conn, SETTING_HOST)?.unwrap_or_default();
    let user = get_setting(conn, SETTING_USER)?.unwrap_or_default();
    let mailbox = get_setting(conn, SETTING_MAILBOX)?.unwrap_or_else(|| "INBOX".into());
    let port = get_setting(conn, SETTING_PORT)?
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(993);
    let has_password = if user.is_empty() {
        false
    } else {
        load_password(&user)?.is_some()
    };
    Ok(EmailSettings {
        host,
        port,
        user,
        mailbox,
        has_password,
    })
}

fn message_url_for(message_id: &Option<String>) -> Option<String> {
    let mid = message_id.as_ref()?.trim();
    if mid.is_empty() {
        return None;
    }
    let with_brackets = if mid.starts_with('<') && mid.ends_with('>') {
        mid.to_string()
    } else {
        format!("<{mid}>")
    };
    Some(format!("message://{}", urlencoding::encode(&with_brackets)))
}

fn header_value(parsed: &mailparse::ParsedMail<'_>, name: &str) -> Option<String> {
    parsed
        .headers
        .get_first_value(name)
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn parse_address_list(raw: &str) -> Vec<(Option<String>, String)> {
    let Ok(list) = addrparse(raw) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in list.iter() {
        match item {
            mailparse::MailAddr::Single(info) => {
                let addr = info.addr.trim().to_string();
                if !addr.is_empty() {
                    out.push((info.display_name.clone(), addr));
                }
            }
            mailparse::MailAddr::Group(group) => {
                if let Some(info) = group.addrs.first() {
                    out.push((
                        info.display_name
                            .clone()
                            .or_else(|| Some(group.group_name.clone())),
                        info.addr.trim().to_string(),
                    ));
                }
            }
        }
    }
    out
}

fn looks_like_promo_subject(subject: &str) -> bool {
    let s = subject.to_ascii_lowercase();
    const NEEDLES: &[&str] = &[
        "unsubscribe",
        "% off",
        "sale ends",
        "limited time",
        "newsletter",
        "weekly digest",
        "daily digest",
        "deal of",
        "promo",
        "coupon",
        "free shipping",
    ];
    NEEDLES.iter().any(|n| s.contains(n))
}

pub(crate) fn parse_headers(raw: &[u8]) -> ParsedHeaders {
    let Ok(parsed) = parse_mail(raw) else {
        return ParsedHeaders {
            message_id: None,
            from_addr: None,
            from_name: None,
            to_addrs: String::new(),
            subject: "(unable to parse)".into(),
            preview: String::new(),
            date_iso: None,
            has_list_unsubscribe: false,
            is_newsletter: false,
        };
    };

    let message_id = header_value(&parsed, "Message-ID").or_else(|| header_value(&parsed, "Message-Id"));
    let subject = header_value(&parsed, "Subject").unwrap_or_else(|| "(no subject)".into());
    let to_raw = header_value(&parsed, "To").unwrap_or_default();
    let to_addrs = parse_address_list(&to_raw)
        .into_iter()
        .map(|(_, addr)| addr)
        .collect::<Vec<_>>()
        .join(", ");

    let from_raw = header_value(&parsed, "From").unwrap_or_default();
    let from = parse_address_list(&from_raw).into_iter().next();
    let (from_name, from_addr) = match from {
        Some((name, addr)) => (name, Some(addr)),
        None => (None, None),
    };

    let date_iso = header_value(&parsed, "Date").and_then(|d| {
        // Prefer RFC2822 → ISO when chrono can parse; otherwise keep raw.
        mailparse::dateparse(&d)
            .ok()
            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.to_rfc3339())
            .or(Some(d))
    });

    let list_unsub = header_value(&parsed, "List-Unsubscribe");
    let precedence = header_value(&parsed, "Precedence")
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let auto_submitted = header_value(&parsed, "Auto-Submitted")
        .map(|v| v.to_ascii_lowercase())
        .unwrap_or_default();
    let list_id = header_value(&parsed, "List-Id");
    let x_campaign = header_value(&parsed, "X-Campaign-ID")
        .or_else(|| header_value(&parsed, "X-Mailer-Lite-Message-Id"))
        .or_else(|| header_value(&parsed, "X-SG-EID"));

    let has_list_unsubscribe = list_unsub.is_some();
    let is_newsletter = has_list_unsubscribe
        || list_id.is_some()
        || x_campaign.is_some()
        || matches!(precedence.as_str(), "bulk" | "list" | "junk")
        || (!auto_submitted.is_empty() && auto_submitted != "no")
        || looks_like_promo_subject(&subject);

    let preview = parsed
        .get_body()
        .ok()
        .map(|b| {
            b.chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                .take(160)
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();

    ParsedHeaders {
        message_id,
        from_addr,
        from_name,
        to_addrs,
        subject,
        preview,
        date_iso,
        has_list_unsubscribe,
        is_newsletter,
    }
}

fn is_junk_mailbox(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("junk")
        || n.contains("spam")
        || n.contains("bulk")
        || n.contains("trash")
        || n.contains("deleted")
}

fn normalize_addr(addr: &str) -> String {
    addr.trim().trim_matches(|c| c == '<' || c == '>').to_ascii_lowercase()
}

/// Best-effort contact signals from Messages `handle.id` values.
pub(crate) fn load_known_contacts() -> HashSet<String> {
    let mut out = HashSet::new();
    let Some(home) = dirs_home() else {
        return out;
    };
    let chat_db = home.join("Library/Messages/chat.db");
    if !chat_db.exists() {
        return out;
    }
    let Ok(conn) = Connection::open_with_flags(
        &chat_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) else {
        return out;
    };
    let Ok(mut stmt) = conn.prepare("SELECT id FROM handle LIMIT 2000") else {
        return out;
    };
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) else {
        return out;
    };
    for row in rows.flatten() {
        let trimmed = row.trim().to_ascii_lowercase();
        if trimmed.contains('@') || trimmed.chars().any(|c| c.is_ascii_digit()) {
            out.insert(trimmed);
        }
    }
    out
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub(crate) fn score_importance(
    headers: &ParsedHeaders,
    user_email: &str,
    known: &HashSet<String>,
    mailbox: &str,
) -> (bool, f64, bool) {
    let junk_mailbox = is_junk_mailbox(mailbox);
    if junk_mailbox {
        return (false, -100.0, true);
    }
    if headers.is_newsletter {
        return (false, -50.0, false);
    }

    let mut score = 10.0;
    let user = normalize_addr(user_email);
    let to_list: Vec<String> = headers
        .to_addrs
        .split(',')
        .map(normalize_addr)
        .filter(|s| !s.is_empty())
        .collect();

    if !user.is_empty() && to_list.iter().any(|a| a == &user) {
        score += 25.0;
    } else if to_list.len() == 1 {
        // Single recipient — often a direct mail even if address alias differs.
        score += 8.0;
    } else if to_list.len() > 4 {
        score -= 8.0;
    }

    if let Some(from) = headers.from_addr.as_deref() {
        let from_n = normalize_addr(from);
        if known.contains(&from_n) {
            score += 20.0;
        } else if let Some((local, _)) = from_n.split_once('@') {
            // Phone-like / short handles sometimes stored without domain in Messages.
            if known.iter().any(|k| k == local || k.contains(local)) {
                score += 12.0;
            }
        }
        // Generic noreply senders are rarely "important".
        if from_n.contains("noreply")
            || from_n.contains("no-reply")
            || from_n.contains("donotreply")
            || from_n.starts_with("notifications@")
            || from_n.starts_with("news@")
            || from_n.starts_with("marketing@")
        {
            score -= 30.0;
        }
    }

    if headers.has_list_unsubscribe {
        score -= 40.0;
    }

    let important = score >= 15.0;
    (important, score, false)
}

pub(crate) fn upsert_email(
    conn: &Connection,
    uid: u32,
    mailbox: &str,
    headers: &ParsedHeaders,
    important: bool,
    score: f64,
    is_junk: bool,
) -> Result<(), DbError> {
    let synced_at = now_iso();
    let message_url = message_url_for(&headers.message_id);
    conn.execute(
        "INSERT INTO emails (
            uid, mailbox, message_id, from_addr, from_name, to_addrs, subject, preview,
            date_iso, is_unread, is_important, importance_score, has_list_unsubscribe,
            is_junk, message_url, synced_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(mailbox, uid) DO UPDATE SET
            message_id = excluded.message_id,
            from_addr = excluded.from_addr,
            from_name = excluded.from_name,
            to_addrs = excluded.to_addrs,
            subject = excluded.subject,
            preview = excluded.preview,
            date_iso = excluded.date_iso,
            is_unread = 1,
            is_important = excluded.is_important,
            importance_score = excluded.importance_score,
            has_list_unsubscribe = excluded.has_list_unsubscribe,
            is_junk = excluded.is_junk,
            message_url = excluded.message_url,
            synced_at = excluded.synced_at",
        params![
            uid as i64,
            mailbox,
            headers.message_id,
            headers.from_addr,
            headers.from_name,
            headers.to_addrs,
            headers.subject,
            headers.preview,
            headers.date_iso,
            if important { 1 } else { 0 },
            score,
            if headers.has_list_unsubscribe { 1 } else { 0 },
            if is_junk { 1 } else { 0 },
            message_url,
            synced_at,
        ],
    )?;
    Ok(())
}

fn map_email_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EmailMessage> {
    Ok(EmailMessage {
        id: row.get(0)?,
        uid: row.get(1)?,
        mailbox: row.get(2)?,
        message_id: row.get(3)?,
        from_addr: row.get(4)?,
        from_name: row.get(5)?,
        to_addrs: row.get(6)?,
        subject: row.get(7)?,
        preview: row.get(8)?,
        date_iso: row.get(9)?,
        is_unread: row.get::<_, i64>(10)? != 0,
        is_important: row.get::<_, i64>(11)? != 0,
        importance_score: row.get(12)?,
        has_list_unsubscribe: row.get::<_, i64>(13)? != 0,
        is_junk: row.get::<_, i64>(14)? != 0,
        message_url: row.get(15)?,
        synced_at: row.get(16)?,
    })
}

const EMAIL_SELECT: &str = "SELECT id, uid, mailbox, message_id, from_addr, from_name, to_addrs,
        subject, preview, date_iso, is_unread, is_important, importance_score,
        has_list_unsubscribe, is_junk, message_url, synced_at
     FROM emails";

fn list_important(conn: &Connection, limit: Option<i64>) -> Result<Vec<EmailMessage>, DbError> {
    let limit = limit.unwrap_or(10).max(1);
    let mut stmt = conn.prepare(&format!(
        "{EMAIL_SELECT}
         WHERE is_unread = 1 AND is_important = 1 AND is_junk = 0
         ORDER BY importance_score DESC, date_iso DESC, id DESC
         LIMIT ?1"
    ))?;
    let items = stmt
        .query_map(params![limit], map_email_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

fn fetch_header_bytes(fetch: &Fetch) -> Option<Vec<u8>> {
    if let Some(header) = fetch.header() {
        return Some(header.to_vec());
    }
    // BODY.PEEK[HEADER] may land in body() depending on server/query.
    fetch.body().map(|b| b.to_vec())
}

fn sync_imap(conn: &Connection) -> Result<EmailSyncResult, DbError> {
    let settings = read_settings(conn)?;
    if settings.host.trim().is_empty() || settings.user.trim().is_empty() {
        return Err(DbError::Message(
            "Configure IMAP host and username in Email settings first.".into(),
        ));
    }
    let password = load_password(&settings.user)?.ok_or_else(|| {
        DbError::Message("IMAP password missing from Keychain — save it in Email settings.".into())
    })?;

    let mailbox = if settings.mailbox.trim().is_empty() {
        "INBOX".to_string()
    } else {
        settings.mailbox.trim().to_string()
    };
    if is_junk_mailbox(&mailbox) {
        return Err(DbError::Message(
            "Refusing to sync a Junk/Spam mailbox. Use INBOX (or another primary mailbox).".into(),
        ));
    }

    let tls = TlsConnector::builder()
        .build()
        .map_err(|e| DbError::Message(format!("TLS init failed: {e}")))?;

    let client = imap::connect((settings.host.as_str(), settings.port), settings.host.as_str(), &tls)
        .map_err(|e| DbError::Message(format!("IMAP connect failed: {e}")))?;

    let mut session = client
        .login(&settings.user, &password)
        .map_err(|e| DbError::Message(format!("IMAP login failed: {}", e.0)))?;

    // Skip obvious junk folders if the configured mailbox is INBOX — we only sync the chosen mailbox.
    session
        .select(&mailbox)
        .map_err(|e| DbError::Message(format!("IMAP select '{mailbox}' failed: {e}")))?;

    let uids = session
        .uid_search("UNSEEN")
        .map_err(|e| DbError::Message(format!("IMAP SEARCH UNSEEN failed: {e}")))?;

    // Cap work for v1 responsiveness.
    let mut uid_list: Vec<u32> = uids.into_iter().collect();
    uid_list.sort_unstable();
    uid_list.reverse();
    uid_list.truncate(200);

    let known = load_known_contacts();
    let mut fetched = 0usize;
    let mut important = 0usize;

    // Mark previously synced unread rows for this mailbox as stale, then refresh from server.
    // Messages no longer unseen will drop out of important lists after we clear is_unread for
    // UIDs not seen in this pass.
    let mut still_unread: HashSet<i64> = HashSet::new();

    if !uid_list.is_empty() {
        // Fetch in chunks to avoid huge IMAP commands.
        for chunk in uid_list.chunks(40) {
            let set = chunk
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let messages = session
                .uid_fetch(&set, "(UID BODY.PEEK[HEADER] BODY.PEEK[TEXT]<0.400>)")
                .map_err(|e| DbError::Message(format!("IMAP FETCH failed: {e}")))?;

            for msg in messages.iter() {
                let Some(uid) = msg.uid else { continue };
                still_unread.insert(uid as i64);
                let Some(header_bytes) = fetch_header_bytes(msg) else {
                    continue;
                };
                let mut headers = parse_headers(&header_bytes);
                if headers.preview.is_empty() {
                    if let Some(body) = msg.text() {
                        headers.preview = String::from_utf8_lossy(body)
                            .chars()
                            .filter(|c| !c.is_control() || *c == ' ')
                            .take(160)
                            .collect::<String>()
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");
                    }
                }
                let (is_important, score, is_junk) =
                    score_importance(&headers, &settings.user, &known, &mailbox);
                upsert_email(conn, uid, &mailbox, &headers, is_important, score, is_junk)?;
                fetched += 1;
                if is_important {
                    important += 1;
                }
            }
        }
    }

    // Any local unread for this mailbox not returned as UNSEEN is now read.
    let mut stmt = conn.prepare(
        "SELECT uid FROM emails WHERE mailbox = ?1 AND is_unread = 1",
    )?;
    let local_uids: Vec<i64> = stmt
        .query_map(params![mailbox], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);
    for uid in local_uids {
        if !still_unread.contains(&uid) {
            conn.execute(
                "UPDATE emails SET is_unread = 0 WHERE mailbox = ?1 AND uid = ?2",
                params![mailbox, uid],
            )?;
        }
    }

    let _ = session.logout();

    Ok(EmailSyncResult {
        fetched,
        important,
        mailbox,
    })
}

#[tauri::command]
pub fn get_email_settings(state: State<'_, DbState>) -> Result<EmailSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    read_settings(db.conn())
}

#[tauri::command]
pub fn save_email_settings(
    state: State<'_, DbState>,
    input: SaveEmailSettingsInput,
) -> Result<EmailSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let host = input.host.trim();
    let user = input.user.trim();
    if host.is_empty() || user.is_empty() {
        return Err(DbError::Message("IMAP host and username are required".into()));
    }
    let port = input.port.unwrap_or(993);
    let mailbox = input
        .mailbox
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("INBOX");

    let previous_user = get_setting(db.conn(), SETTING_USER)?.unwrap_or_default();

    set_setting(db.conn(), SETTING_HOST, host)?;
    set_setting(db.conn(), SETTING_PORT, &port.to_string())?;
    set_setting(db.conn(), SETTING_USER, user)?;
    set_setting(db.conn(), SETTING_MAILBOX, mailbox)?;

    if let Some(password) = input.password.as_deref() {
        let password = password.trim();
        if !password.is_empty() {
            // If username changed, drop the old Keychain item.
            if !previous_user.is_empty() && previous_user != user {
                let _ = delete_password(&previous_user);
            }
            store_password(user, password)?;
        }
    }

    read_settings(db.conn())
}

#[tauri::command]
pub fn sync_email(state: State<'_, DbState>) -> Result<EmailSyncResult, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    sync_imap(db.conn())
}

#[tauri::command]
pub fn list_important_emails(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<EmailMessage>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    list_important(db.conn(), limit)
}

#[tauri::command]
pub fn list_all_important_emails(
    state: State<'_, DbState>,
) -> Result<Vec<EmailMessage>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    list_important(db.conn(), Some(500))
}

#[tauri::command]
pub fn open_email(state: State<'_, DbState>, id: i64) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db.conn().prepare(
        "SELECT message_url, message_id FROM emails WHERE id = ?1",
    )?;
    let (message_url, message_id): (Option<String>, Option<String>) = stmt
        .query_row(params![id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|_| DbError::Message(format!("email {id} not found")))?;
    drop(stmt);
    drop(db);

    let url = message_url.or_else(|| message_url_for(&message_id));
    if let Some(url) = url {
        // Prefer message:// so Mail.app jumps to the message when it has it indexed.
        match open_with_system("url", &url) {
            Ok(()) => return Ok(()),
            Err(_) => {
                // Fall through to opening Mail.app.
            }
        }
    }

    // Fallback: just bring Mail.app forward.
    let status = Command::new("open")
        .args(["-a", "Mail"])
        .status()
        .map_err(|e| DbError::Message(format!("failed to open Mail.app: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(DbError::Message(format!(
            "open Mail.app exited with status {status}"
        )))
    }
}
