//! Read Mail.app accounts already signed in on this Mac.
//!
//! Google and Microsoft Internet Accounts show up here, so the user can click
//! an account instead of pasting IMAP credentials. Full RFC822 source is still
//! used for Informed Delivery.

use crate::commands::email::{
    load_known_contacts, read_settings, score_importance, upsert_email, EmailSettings,
    EmailSyncResult, SETTING_AUTH, SETTING_HOST, SETTING_MAILAPP_ACCOUNT, SETTING_MAILBOX,
    SETTING_PORT, SETTING_PROVIDER, SETTING_USER,
};
use crate::db::{set_setting, DbError, DbState};
use crate::security::validate_mailapp_account_name;
use rusqlite::Connection;
use serde::Serialize;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const MAIL_LIST_TIMEOUT: Duration = Duration::from_secs(15);
const MAIL_SYNC_TIMEOUT: Duration = Duration::from_secs(25);
const MAIL_TIMEOUT_HINT: &str = "Mail.app did not respond in time. Finish any Mail or Outlook sign-in windows, then try again.";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MailAppAccount {
    pub name: String,
    pub user_name: String,
    pub kind: String,
    pub account_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MailAppAccountsResult {
    /// `"ok"`, `"needs_permission"`, or `"unavailable"`.
    pub status: String,
    pub detail: Option<String>,
    pub accounts: Vec<MailAppAccount>,
}

#[derive(Debug, Clone)]
pub struct MailAppMessage {
    pub id: i64,
    pub message_id: Option<String>,
    pub sender: String,
    pub subject: String,
    pub date_iso: Option<String>,
}

pub fn infer_account_kind(user_name: &str, account_type: &str) -> String {
    let user = user_name.trim().to_ascii_lowercase();
    let ty = account_type.trim().to_ascii_lowercase();
    if user.contains("@gmail.")
        || user.ends_with("@googlemail.com")
        || ty.contains("gmail")
    {
        return "google".into();
    }
    if ty.contains("exchange")
        || user.contains("@outlook.")
        || user.contains("@hotmail.")
        || user.contains("@live.")
        || user.contains("@msn.")
        || user.contains("@office365.")
        || user.contains("onmicrosoft.com")
    {
        return "microsoft".into();
    }
    if ty.contains("icloud") || user.contains("@icloud.") || user.contains("@me.com") || user.contains("@mac.com")
    {
        return "icloud".into();
    }
    if ty.contains("imap") {
        return "imap".into();
    }
    "other".into()
}

pub fn parse_account_rows(raw: &str) -> Vec<MailAppAccount> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(3, '\t');
        let name = parts.next().unwrap_or("").trim();
        let user_name = parts.next().unwrap_or("").trim();
        let account_type = parts.next().unwrap_or("").trim();
        if name.is_empty() {
            continue;
        }
        out.push(MailAppAccount {
            kind: infer_account_kind(user_name, account_type),
            name: name.to_string(),
            user_name: user_name.to_string(),
            account_type: account_type.to_string(),
        });
    }
    out
}

pub fn parse_sender(raw: &str) -> (Option<String>, Option<String>) {
    let raw = raw.trim();
    if raw.is_empty() {
        return (None, None);
    }
    if let Some(start) = raw.rfind('<') {
        if let Some(end) = raw[start + 1..].find('>') {
            let addr = raw[start + 1..start + 1 + end].trim();
            let name = raw[..start].trim().trim_matches('"').trim();
            return (
                if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                },
                if addr.is_empty() {
                    None
                } else {
                    Some(addr.to_string())
                },
            );
        }
    }
    if raw.contains('@') {
        (None, Some(raw.to_string()))
    } else {
        (Some(raw.to_string()), None)
    }
}

pub fn parse_message_rows(raw: &str) -> Vec<MailAppMessage> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(5, '\t');
        let id = parts.next().unwrap_or("").trim().parse::<i64>().unwrap_or(0);
        if id == 0 {
            continue;
        }
        let message_id = parts
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let sender = parts.next().unwrap_or("").to_string();
        let subject = parts.next().unwrap_or("(no subject)").to_string();
        let date_iso = parts
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        out.push(MailAppMessage {
            id,
            message_id,
            sender,
            subject,
            date_iso,
        });
    }
    out
}

fn applescript_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn wrap_with_timeout(source: &str, timeout: Duration) -> String {
    let secs = timeout.as_secs().saturating_sub(2).max(4);
    format!("with timeout of {secs} seconds\n{source}\nend timeout")
}

pub fn osascript_failure_message(stderr: &str, stdout: &str) -> String {
    let combined = format!("{stderr}\n{stdout}").to_ascii_lowercase();
    if combined.contains("not authorized")
        || combined.contains("(-1743)")
        || combined.contains("not allowed to send apple events")
        || combined.contains("osstatus error 1")
    {
        return "Mainstream needs Automation access to control Mail. System Settings → Privacy & Security → Automation → enable Mail for Mainstream.".into();
    }
    if combined.contains("appleevent timed out")
        || combined.contains("apple event timed out")
        || combined.contains("(-1712)")
    {
        return MAIL_TIMEOUT_HINT.into();
    }
    if combined.contains("application isn’t running")
        || combined.contains("application isn't running")
        || combined.contains("(-609)")
    {
        return "Mail.app did not respond. Open Mail once, then try again.".into();
    }
    let detail = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    if detail.is_empty() {
        "Mail.app: unknown error".into()
    } else {
        format!("Mail.app: {detail}")
    }
}

fn run_osascript(source: &str) -> Result<String, DbError> {
    run_osascript_timed(source, MAIL_LIST_TIMEOUT)
}

fn run_osascript_timed(source: &str, timeout: Duration) -> Result<String, DbError> {
    let wrapped = wrap_with_timeout(source, timeout);
    let mut child = Command::new("osascript")
        .arg("-e")
        .arg(&wrapped)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| DbError::Message(format!("osascript failed: {e}")))?;

    let mut stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| DbError::Message("osascript stdout missing".into()))?;
    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| DbError::Message("osascript stderr missing".into()))?;

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let stdout_h = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout_pipe.read_to_end(&mut buf);
            buf
        });
        let stderr_h = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stderr_pipe.read_to_end(&mut buf);
            buf
        });
        let stdout = stdout_h.join().unwrap_or_default();
        let stderr = stderr_h.join().unwrap_or_default();
        let _ = tx.send((stdout, stderr));
    });

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let (stdout_raw, stderr_raw) = rx.recv().unwrap_or_default();
                let stdout = String::from_utf8_lossy(&stdout_raw).trim().to_string();
                let stderr = String::from_utf8_lossy(&stderr_raw).trim().to_string();
                if status.success() {
                    return Ok(stdout);
                }
                return Err(DbError::Message(osascript_failure_message(
                    &stderr, &stdout,
                )));
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(DbError::Message(MAIL_TIMEOUT_HINT.into()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(40)),
            Err(e) => {
                let _ = child.kill();
                return Err(DbError::Message(format!("osascript failed: {e}")));
            }
        }
    }
}

fn is_permission_error(err: &DbError) -> bool {
    err.to_string().to_ascii_lowercase().contains("automation")
}

pub fn list_accounts() -> Result<MailAppAccountsResult, DbError> {
    let script = r#"
tell application "Mail"
  set out to ""
  repeat with a in accounts
    set n to name of a
    set u to ""
    try
      set u to user name of a
    end try
    set t to ""
    try
      set t to (account type of a as string)
    end try
    set out to out & n & tab & u & tab & t & linefeed
  end repeat
  return out
end tell
"#;
    match run_osascript(script) {
        Ok(raw) => Ok(MailAppAccountsResult {
            status: "ok".into(),
            detail: None,
            accounts: parse_account_rows(&raw),
        }),
        Err(err) if is_permission_error(&err) => Ok(MailAppAccountsResult {
            status: "needs_permission".into(),
            detail: Some(err.to_string()),
            accounts: Vec::new(),
        }),
        Err(err) => Ok(MailAppAccountsResult {
            status: "unavailable".into(),
            detail: Some(err.to_string()),
            accounts: Vec::new(),
        }),
    }
}

fn mailbox_key(account: &str) -> String {
    format!("mailapp:{account}")
}

fn clean_field(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn list_unread_script(account: &str, limit: usize) -> String {
    let name = applescript_string(account);
    format!(
        r#"
tell application "Mail"
  set acct to account {name}
  set theBox to missing value
  try
    set theBox to mailbox "INBOX" of acct
  end try
  if theBox is missing value then
    try
      set theBox to item 1 of (get mailboxes of acct)
    end try
  end if
  if theBox is missing value then error "No mailbox found for that account."
  set cutoff to (current date) - 14 * days
  set theMsgs to (messages of theBox whose date received > cutoff and read status is false)
  set out to ""
  set i to 0
  repeat with m in theMsgs
    set i to i + 1
    if i > {limit} then exit repeat
    set mid to id of m as string
    set rfc to ""
    try
      set rfc to message id of m
    end try
    set snd to sender of m
    set subj to subject of m
    set dateStr to ""
    try
      set dateStr to ((date received of m) as «class isot») as string
    on error
      try
        set dateStr to date received of m as string
      end try
    end try
    set out to out & mid & tab & rfc & tab & snd & tab & subj & tab & dateStr & linefeed
  end repeat
  return out
end tell
"#
    )
}

fn list_informed_script(account: &str, limit: usize) -> String {
    let name = applescript_string(account);
    format!(
        r#"
tell application "Mail"
  set acct to account {name}
  set theBox to mailbox "INBOX" of acct
  set cutoff to (current date) - 21 * days
  set theMsgs to (messages of theBox whose date received > cutoff and (sender contains "usps" or sender contains "informeddelivery" or sender contains "informed-delivery" or subject contains "Informed Delivery" or subject contains "Your Daily Digest"))
  set out to ""
  set i to 0
  repeat with m in theMsgs
    set i to i + 1
    if i > {limit} then exit repeat
    set mid to id of m as string
    set rfc to ""
    try
      set rfc to message id of m
    end try
    set snd to sender of m
    set subj to subject of m
    set dateStr to ""
    try
      set dateStr to ((date received of m) as «class isot») as string
    on error
      try
        set dateStr to date received of m as string
      end try
    end try
    set out to out & mid & tab & rfc & tab & snd & tab & subj & tab & dateStr & linefeed
  end repeat
  return out
end tell
"#
    )
}

fn source_script(account: &str, id: i64, path: &str) -> String {
    let name = applescript_string(account);
    let posix = applescript_string(path);
    format!(
        r#"
tell application "Mail"
  set acct to account {name}
  set theBox to mailbox "INBOX" of acct
  set theMsg to first message of theBox whose id is {id}
  set src to source of theMsg
end tell
set f to open for access POSIX file {posix} with write permission
set eof of f to 0
write src to f as «class utf8»
close access f
return {posix}
"#
    )
}

pub fn fetch_unread(account: &str, limit: usize) -> Result<Vec<MailAppMessage>, DbError> {
    let account = validate_mailapp_account_name(account)?;
    let raw = run_osascript_timed(&list_unread_script(&account, limit), MAIL_SYNC_TIMEOUT)?;
    Ok(parse_message_rows(&raw)
        .into_iter()
        .map(|mut msg| {
            msg.sender = clean_field(&msg.sender);
            msg.subject = clean_field(&msg.subject);
            msg
        })
        .collect())
}

pub fn fetch_informed_candidates(account: &str, limit: usize) -> Result<Vec<MailAppMessage>, DbError> {
    let account = validate_mailapp_account_name(account)?;
    let raw = run_osascript_timed(&list_informed_script(&account, limit), MAIL_SYNC_TIMEOUT)?;
    Ok(parse_message_rows(&raw))
}

pub fn fetch_source(account: &str, id: i64, dest: &Path) -> Result<Vec<u8>, DbError> {
    let account = validate_mailapp_account_name(account)?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    run_osascript_timed(
        &source_script(&account, id, &dest.to_string_lossy()),
        MAIL_SYNC_TIMEOUT,
    )?;
    Ok(std::fs::read(dest)?)
}

pub fn sync_mailapp_inbox(conn: &Connection) -> Result<EmailSyncResult, DbError> {
    let settings = read_settings(conn)?;
    let account = validate_mailapp_account_name(
        settings
            .mailapp_account
            .as_deref()
            .unwrap_or(""),
    )?;
    let mailbox = mailbox_key(&account);
    let messages = fetch_unread(&account, 200)?;
    let known = load_known_contacts();
    let mut still_unread = std::collections::HashSet::new();
    let mut fetched = 0usize;
    let mut important = 0usize;

    for msg in messages {
        still_unread.insert(msg.id);
        let (from_name, from_addr) = parse_sender(&msg.sender);
        let headers = crate::commands::email::ParsedHeaders {
            message_id: msg.message_id,
            from_addr,
            from_name,
            to_addrs: settings.user.clone(),
            subject: if msg.subject.trim().is_empty() {
                "(no subject)".into()
            } else {
                msg.subject
            },
            preview: String::new(),
            date_iso: msg.date_iso,
            has_list_unsubscribe: false,
            is_newsletter: false,
        };
        let (is_important, score, is_junk) =
            score_importance(&headers, &settings.user, &known, "INBOX");
        upsert_email(
            conn,
            msg.id,
            &mailbox,
            &headers,
            is_important,
            score,
            is_junk,
        )?;
        fetched += 1;
        if is_important {
            important += 1;
        }
    }

    let mut stmt = conn.prepare("SELECT uid FROM emails WHERE mailbox = ?1 AND is_unread = 1")?;
    let local_uids: Vec<i64> = stmt
        .query_map(rusqlite::params![mailbox], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);
    for uid in local_uids {
        if !still_unread.contains(&uid) {
            conn.execute(
                "UPDATE emails SET is_unread = 0 WHERE mailbox = ?1 AND uid = ?2",
                rusqlite::params![mailbox, uid],
            )?;
        }
    }

    Ok(EmailSyncResult {
        fetched,
        important,
        mailbox,
    })
}

#[tauri::command]
pub async fn list_mail_accounts() -> Result<MailAppAccountsResult, DbError> {
    tauri::async_runtime::spawn_blocking(list_accounts)
        .await
        .map_err(|e| DbError::Message(format!("Mail.app task failed: {e}")))?
}

fn connect_mail_account(state: &DbState, name: String) -> Result<EmailSettings, DbError> {
    let name = validate_mailapp_account_name(&name)?;
    let accounts = list_accounts()?;
    let account = accounts
        .accounts
        .iter()
        .find(|a| a.name == name)
        .cloned()
        .ok_or_else(|| {
            DbError::Message(format!(
                "Mail.app account '{name}' was not found. Open Mail and try again."
            ))
        })?;

    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    set_setting(db.conn(), SETTING_PROVIDER, "mailapp")?;
    set_setting(db.conn(), SETTING_AUTH, "mailapp")?;
    set_setting(db.conn(), SETTING_MAILAPP_ACCOUNT, &account.name)?;
    set_setting(
        db.conn(),
        SETTING_USER,
        if account.user_name.is_empty() {
            &account.name
        } else {
            &account.user_name
        },
    )?;
    set_setting(db.conn(), SETTING_MAILBOX, "INBOX")?;
    set_setting(db.conn(), SETTING_HOST, "mail.app")?;
    set_setting(db.conn(), SETTING_PORT, "0")?;
    read_settings(db.conn())
}

#[tauri::command]
pub async fn use_mail_account(
    app: AppHandle,
    name: String,
) -> Result<EmailSettings, DbError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DbState>();
        connect_mail_account(&state, name)
    })
    .await
    .map_err(|e| DbError::Message(format!("Mail.app task failed: {e}")))?
}

#[tauri::command]
pub fn open_internet_accounts() -> Result<(), DbError> {
    crate::commands::open::open_with_system(
        "url",
        "x-apple.systempreferences:com.apple.Internet-Accounts-Settings.extension",
    )
    .or_else(|_| {
        crate::commands::open::open_with_system(
            "url",
            "x-apple.systempreferences:com.apple.preference.internetaccounts",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_google_and_microsoft_accounts() {
        assert_eq!(infer_account_kind("ada@gmail.com", "imap"), "google");
        assert_eq!(
            infer_account_kind("ada@outlook.com", "imap"),
            "microsoft"
        );
        assert_eq!(
            infer_account_kind("ada@contoso.com", "exchange"),
            "microsoft"
        );
        assert_eq!(infer_account_kind("ada@icloud.com", "iCloud"), "icloud");
    }

    #[test]
    fn parses_account_tsv() {
        let rows = parse_account_rows(
            "Gmail\tada@gmail.com\timap\nOutlook\tada@outlook.com\texchange\n",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].kind, "google");
        assert_eq!(rows[1].kind, "microsoft");
    }

    #[test]
    fn parses_sender_display_and_address() {
        assert_eq!(
            parse_sender("Ada Lovelace <ada@gmail.com>"),
            (Some("Ada Lovelace".into()), Some("ada@gmail.com".into()))
        );
        assert_eq!(
            parse_sender("ada@outlook.com"),
            (None, Some("ada@outlook.com".into()))
        );
    }

    #[test]
    fn parses_message_tsv() {
        let rows = parse_message_rows(
            "42\t<id@mail>\tAda <ada@x.com>\tHello\t2026-08-28T12:00:00Z\n",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 42);
        assert_eq!(rows[0].subject, "Hello");
        assert_eq!(rows[0].message_id.as_deref(), Some("<id@mail>"));
    }

    #[test]
    fn maps_mail_timeout_and_permission_errors() {
        assert!(osascript_failure_message("AppleEvent timed out. (-1712)", "")
            .contains("did not respond in time"));
        assert!(osascript_failure_message("not authorized to send Apple events", "")
            .contains("Automation"));
        assert!(osascript_failure_message("Application isn’t running. (-609)", "")
            .contains("Open Mail"));
    }
}
