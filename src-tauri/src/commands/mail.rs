//! USPS Informed Delivery — parse digest emails, extract scans, OCR on macOS.

use crate::commands::email::{
    load_known_contacts, open_imap_session, parse_headers, read_settings, score_importance,
    upsert_email,
};
use crate::db::{now_iso, DbError, DbState};
use crate::security::{
    max_http_response_bytes, parse_public_https_url, path_is_within, public_http_client,
};
use mailparse::MailHeaderMap;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalMailPiece {
    pub id: i64,
    pub email_id: i64,
    pub digest_date: Option<String>,
    pub piece_index: i64,
    pub ocr_text: String,
    pub image_path: Option<String>,
    pub subject: String,
    pub synced_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalMailSyncResult {
    pub digests: usize,
    pub pieces: usize,
    pub ocr_ran: usize,
}

struct MimeBundle {
    html: Option<String>,
    inline_images: HashMap<String, Vec<u8>>,
}

fn is_informed_delivery(from_addr: Option<&str>, from_name: Option<&str>, subject: &str) -> bool {
    let from = from_addr.unwrap_or("").to_ascii_lowercase();
    let name = from_name.unwrap_or("").to_ascii_lowercase();
    let subj = subject.to_ascii_lowercase();

    let from_usps = from.contains("informeddelivery")
        || from.contains("informed-delivery")
        || (from.contains("usps.gov") && subj.contains("informed"))
        || name.contains("informed delivery")
        || (name.contains("usps") && subj.contains("digest"));

    let subject_match = subj.contains("informed delivery")
        || subj.contains("daily digest")
        || subj.contains("mail preview")
        || subj.contains("your mail");

    from_usps && subject_match
}

fn extract_mime_bundle(raw: &[u8]) -> MimeBundle {
    let mut html = None;
    let mut inline_images = HashMap::new();

    let Ok(parsed) = mailparse::parse_mail(raw) else {
        return MimeBundle {
            html: None,
            inline_images,
        };
    };

    fn walk(
        part: &mailparse::ParsedMail<'_>,
        html: &mut Option<String>,
        images: &mut HashMap<String, Vec<u8>>,
    ) {
        let ctype = part
            .headers
            .get_first_value("Content-Type")
            .unwrap_or_default()
            .to_ascii_lowercase();

        if ctype.starts_with("multipart/") {
            for sub in &part.subparts {
                walk(sub, html, images);
            }
            return;
        }

        if ctype.starts_with("text/html") {
            if let Ok(body) = part.get_body() {
                if html.is_none() || body.len() > html.as_ref().map(|h| h.len()).unwrap_or(0) {
                    *html = Some(body);
                }
            }
            return;
        }

        if ctype.starts_with("image/") {
            if let Ok(bytes) = part.get_body_raw() {
                if bytes.len() < 1024 {
                    return;
                }
                let cid = part
                    .headers
                    .get_first_value("Content-ID")
                    .map(|v| v.trim_matches(|c| c == '<' || c == '>').to_string())
                    .unwrap_or_else(|| format!("inline_{}", images.len()));
                images.insert(cid, bytes);
            }
        }
    }

    walk(&parsed, &mut html, &mut inline_images);
    MimeBundle {
        html,
        inline_images,
    }
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
}

fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    decode_html_entities(&out)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{name}=");
    let idx = lower.find(&needle)?;
    let rest = &tag[idx + needle.len()..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &rest[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

fn is_mail_scan_image(src: &str, alt: &str) -> bool {
    let src_l = src.to_ascii_lowercase();
    let alt_l = alt.to_ascii_lowercase();

    if src_l.starts_with("data:") && src_l.contains("image/") {
        return alt_l.len() > 3;
    }

    if src_l.contains("tracking") || src_l.contains("pixel") || src_l.contains("beacon") {
        return false;
    }

    if src_l.contains("usps") || src_l.contains("informeddelivery") {
        return true;
    }

    if alt_l.contains("scanned image")
        || alt_l.contains("mail piece")
        || alt_l.contains("envelope")
        || alt_l.contains("postcard")
    {
        return true;
    }

    (src_l.ends_with(".jpg") || src_l.ends_with(".jpeg") || src_l.ends_with(".png"))
        && !src_l.contains("logo")
        && !src_l.contains("social")
        && alt_l.len() > 8
}

fn parse_img_tags(html: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let lower = html.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(rel) = lower[search_from..].find("<img") {
        let start = search_from + rel;
        let Some(end_rel) = lower[start..].find('>') else {
            break;
        };
        let tag = &html[start..start + end_rel + 1];
        let src = attr_value(tag, "src").unwrap_or_default();
        let alt = attr_value(tag, "alt")
            .or_else(|| attr_value(tag, "title"))
            .unwrap_or_default();
        if !src.is_empty() && is_mail_scan_image(&src, &alt) {
            out.push((src, alt));
        }
        search_from = start + end_rel + 1;
    }
    out
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input.trim())
        .ok()
}

fn resolve_image_bytes(src: &str, inline: &HashMap<String, Vec<u8>>) -> Option<Vec<u8>> {
    let src = src.trim();
    if src.starts_with("cid:") {
        let key = src
            .trim_start_matches("cid:")
            .trim_matches(|c| c == '<' || c == '>');
        return inline.get(key).cloned();
    }
    if src.starts_with("data:image/") {
        let comma = src.find(',')?;
        return base64_decode(&src[comma + 1..]);
    }
    None
}

fn ocr_script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/ocr_image.swift")
}

fn run_vision_ocr(image_path: &Path) -> Option<String> {
    let script = ocr_script_path();
    if !script.exists() {
        return None;
    }
    let output = Command::new("swift")
        .arg(&script)
        .arg(image_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn cache_dir_for(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("mail_cache")
}

fn write_image_cache(
    cache_dir: &Path,
    email_id: i64,
    index: usize,
    bytes: &[u8],
) -> Result<PathBuf, DbError> {
    fs::create_dir_all(cache_dir)?;
    let ext = if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "png"
    } else {
        "jpg"
    };
    let path = cache_dir.join(format!("{email_id}_{index}.{ext}"));
    fs::write(&path, bytes)?;
    Ok(path)
}

fn merge_ocr(alt: &str, vision: Option<&str>) -> String {
    let alt = alt.trim();
    let generic = alt.is_empty()
        || alt.eq_ignore_ascii_case("image")
        || alt
            .to_ascii_lowercase()
            .contains("scanned image of your mail");

    if let Some(v) = vision {
        let v = v.trim();
        if !v.is_empty() {
            if generic {
                return v.to_string();
            }
            if !alt.contains(v) && v.len() > alt.len() {
                return format!("{alt}\n{v}");
            }
        }
    }
    if !alt.is_empty() && !generic {
        alt.to_string()
    } else {
        vision.unwrap_or("").trim().to_string()
    }
}

fn digest_date_from_subject(subject: &str, fallback_iso: Option<&str>) -> Option<String> {
    if let Some(idx) = subject.to_ascii_lowercase().find(" for ") {
        let rest = subject[idx + 5..].trim();
        if !rest.is_empty() {
            return Some(rest.trim_matches('.').to_string());
        }
    }
    fallback_iso.map(|s| s.to_string())
}

struct ParsedPiece {
    ocr_text: String,
    image_path: Option<PathBuf>,
    had_vision: bool,
}

fn fetch_url_bytes(url: &str) -> Option<Vec<u8>> {
    let url = parse_public_https_url(url).ok()?;
    let client = public_http_client(12, None).ok()?;
    let resp = client.get(url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let len = resp.content_length().unwrap_or(0);
    if len > max_http_response_bytes() {
        return None;
    }
    let bytes = resp.bytes().ok()?;
    if bytes.len() < 1024 || bytes.len() as u64 > max_http_response_bytes() {
        return None;
    }
    Some(bytes.to_vec())
}

fn extract_pieces_from_digest(
    raw: &[u8],
    email_id: i64,
    cache_dir: &Path,
) -> Result<Vec<ParsedPiece>, DbError> {
    let bundle = extract_mime_bundle(raw);
    let html = bundle.html.as_deref().unwrap_or("");
    let img_tags = parse_img_tags(html);

    let mut pieces = Vec::new();
    let mut orphan_images: Vec<Vec<u8>> = bundle
        .inline_images
        .values()
        .filter(|b| b.len() >= 1024)
        .cloned()
        .collect();

    for (idx, (src, alt)) in img_tags.iter().enumerate() {
        let bytes = resolve_image_bytes(src, &bundle.inline_images).or_else(|| {
            if src.starts_with("https://") {
                fetch_url_bytes(src)
            } else {
                None
            }
        });
        let bytes = bytes.or_else(|| orphan_images.pop());

        let mut image_path = None;
        let vision_text = if let Some(ref b) = bytes {
            if let Ok(path) = write_image_cache(cache_dir, email_id, idx, b) {
                image_path = Some(path.clone());
                run_vision_ocr(&path)
            } else {
                None
            }
        } else {
            None
        };

        let had_vision = vision_text.as_ref().is_some_and(|t| !t.trim().is_empty());
        let ocr_text = merge_ocr(alt, vision_text.as_deref());
        if ocr_text.is_empty() && image_path.is_none() {
            continue;
        }
        pieces.push(ParsedPiece {
            ocr_text: if ocr_text.is_empty() {
                "Mail piece (no text recognized)".into()
            } else {
                ocr_text
            },
            image_path,
            had_vision,
        });
    }

    for (i, bytes) in orphan_images.into_iter().enumerate() {
        let idx = pieces.len() + i;
        if let Ok(path) = write_image_cache(cache_dir, email_id, idx, &bytes) {
            let vision_text = run_vision_ocr(&path);
            let had_vision = vision_text.as_ref().is_some_and(|t| !t.trim().is_empty());
            let ocr_text = merge_ocr("", vision_text.as_deref());
            if ocr_text.is_empty() {
                continue;
            }
            pieces.push(ParsedPiece {
                ocr_text,
                image_path: Some(path),
                had_vision,
            });
        }
    }

    if pieces.is_empty() && !html.is_empty() {
        let plain = strip_tags(html);
        if plain.to_ascii_lowercase().contains("informed delivery") && plain.len() > 40 {
            pieces.push(ParsedPiece {
                ocr_text: plain.chars().take(800).collect(),
                image_path: None,
                had_vision: false,
            });
        }
    }

    Ok(pieces)
}

fn email_row_id(conn: &Connection, mailbox: &str, uid: i64) -> Result<Option<i64>, DbError> {
    let mut stmt = conn.prepare("SELECT id FROM emails WHERE mailbox = ?1 AND uid = ?2")?;
    let mut rows = stmt.query(params![mailbox, uid])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

fn replace_mail_pieces(
    conn: &Connection,
    email_id: i64,
    digest_date: Option<&str>,
    subject: &str,
    pieces: &[ParsedPiece],
) -> Result<usize, DbError> {
    conn.execute(
        "DELETE FROM mail_pieces WHERE email_id = ?1",
        params![email_id],
    )?;
    let synced_at = now_iso();
    let mut count = 0usize;
    for (i, piece) in pieces.iter().enumerate() {
        let image_path = piece
            .image_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());
        conn.execute(
            "INSERT INTO mail_pieces (
                email_id, digest_date, piece_index, ocr_text, image_path, subject, synced_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                email_id,
                digest_date,
                i as i64,
                piece.ocr_text,
                image_path,
                subject,
                synced_at,
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

fn imap_since_date(days: u32) -> String {
    let dt = chrono::Utc::now() - chrono::Duration::days(days as i64);
    dt.format("%d-%b-%Y").to_string()
}

fn fetch_full_message(
    session: &mut crate::commands::email::ImapSession,
    uid: u32,
) -> Result<Vec<u8>, DbError> {
    let set = uid.to_string();
    let messages = session
        .uid_fetch(&set, "(UID BODY.PEEK[])")
        .map_err(|e| DbError::Message(format!("IMAP FETCH body failed: {e}")))?;
    let msg = messages
        .iter()
        .next()
        .ok_or_else(|| DbError::Message("IMAP FETCH returned no message".into()))?;
    let body = msg
        .body()
        .or_else(|| msg.header())
        .ok_or_else(|| DbError::Message("IMAP message body missing".into()))?;
    Ok(body.to_vec())
}

fn ingest_digest(
    conn: &Connection,
    mailbox: &str,
    uid: i64,
    raw: &[u8],
    user_email: &str,
    known: &std::collections::HashSet<String>,
    cache_dir: &Path,
) -> Result<Option<(usize, usize)>, DbError> {
    let headers = parse_headers(raw);
    if !is_informed_delivery(
        headers.from_addr.as_deref(),
        headers.from_name.as_deref(),
        &headers.subject,
    ) {
        return Ok(None);
    }

    let (is_important, score, is_junk) = score_importance(&headers, user_email, known, mailbox);
    upsert_email(conn, uid, mailbox, &headers, is_important, score, is_junk)?;
    let Some(email_id) = email_row_id(conn, mailbox, uid)? else {
        return Ok(None);
    };
    let digest_date = digest_date_from_subject(&headers.subject, headers.date_iso.as_deref());
    let parsed = extract_pieces_from_digest(raw, email_id, cache_dir)?;
    let mut ocr_ran = 0usize;
    for p in &parsed {
        if p.had_vision {
            ocr_ran += 1;
        }
    }
    let n = replace_mail_pieces(
        conn,
        email_id,
        digest_date.as_deref(),
        &headers.subject,
        &parsed,
    )?;
    if n == 0 {
        return Ok(None);
    }
    Ok(Some((n, ocr_ran)))
}

fn sync_informed_delivery_mailapp(
    conn: &Connection,
    db_path: &Path,
    account: &str,
    user_email: &str,
) -> Result<PhysicalMailSyncResult, DbError> {
    let mailbox = format!("mailapp:{account}");
    let candidates = crate::commands::email_mailapp::fetch_informed_candidates(account, 30)?;
    let cache_dir = cache_dir_for(db_path);
    let known = load_known_contacts();
    let mut digests = 0usize;
    let mut pieces_total = 0usize;
    let mut ocr_ran = 0usize;
    let tmp_dir = std::env::temp_dir().join("mainstream-mailapp");

    for msg in candidates {
        let dest = tmp_dir.join(format!("{}.eml", msg.id));
        let raw = match crate::commands::email_mailapp::fetch_source(account, msg.id, &dest) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if let Some((n, ocr)) = ingest_digest(
            conn,
            &mailbox,
            msg.id,
            &raw,
            user_email,
            &known,
            &cache_dir,
        )? {
            digests += 1;
            pieces_total += n;
            ocr_ran += ocr;
        }
        let _ = std::fs::remove_file(dest);
    }

    Ok(PhysicalMailSyncResult {
        digests,
        pieces: pieces_total,
        ocr_ran,
    })
}

pub(crate) fn sync_informed_delivery(
    conn: &Connection,
    db_path: &Path,
) -> Result<PhysicalMailSyncResult, DbError> {
    let settings = read_settings(conn)?;
    if !settings.connected {
        return Err(DbError::Message(
            "Connect Google, Microsoft, or IMAP in Email settings before syncing physical mail."
                .into(),
        ));
    }
    if settings.auth == "mailapp" {
        let account = settings.mailapp_account.unwrap_or_default();
        return sync_informed_delivery_mailapp(conn, db_path, &account, &settings.user);
    }

    let (mut session, settings) = open_imap_session(conn)?;
    let mailbox = if settings.mailbox.trim().is_empty() {
        "INBOX".to_string()
    } else {
        settings.mailbox.trim().to_string()
    };

    session
        .select(&mailbox)
        .map_err(|e| DbError::Message(format!("IMAP select failed: {e}")))?;

    let since = imap_since_date(21);
    let criteria = format!("(OR FROM \"informeddelivery\" FROM \"usps.gov\") SINCE {since}");
    let mut uids: Vec<u32> = session
        .uid_search(&criteria)
        .map_err(|e| DbError::Message(format!("IMAP SEARCH informed delivery failed: {e}")))?
        .into_iter()
        .collect();

    uids.sort_unstable();
    uids.reverse();
    uids.truncate(30);

    let cache_dir = cache_dir_for(db_path);
    let known = load_known_contacts();
    let mut digests = 0usize;
    let mut pieces_total = 0usize;
    let mut ocr_ran = 0usize;

    for uid in uids {
        let raw = fetch_full_message(&mut session, uid)?;
        if let Some((n, ocr)) = ingest_digest(
            conn,
            &mailbox,
            uid as i64,
            &raw,
            &settings.user,
            &known,
            &cache_dir,
        )? {
            digests += 1;
            pieces_total += n;
            ocr_ran += ocr;
        }
    }

    let _ = session.logout();

    Ok(PhysicalMailSyncResult {
        digests,
        pieces: pieces_total,
        ocr_ran,
    })
}

fn map_mail_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PhysicalMailPiece> {
    Ok(PhysicalMailPiece {
        id: row.get(0)?,
        email_id: row.get(1)?,
        digest_date: row.get(2)?,
        piece_index: row.get(3)?,
        ocr_text: row.get(4)?,
        image_path: row.get(5)?,
        subject: row.get(6)?,
        synced_at: row.get(7)?,
    })
}

const MAIL_SELECT: &str = "SELECT id, email_id, digest_date, piece_index, ocr_text, image_path, subject, synced_at FROM mail_pieces";

#[tauri::command]
pub fn sync_physical_mail(state: State<'_, DbState>) -> Result<PhysicalMailSyncResult, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let path = db.path().to_path_buf();
    sync_informed_delivery(db.conn(), &path)
}

#[tauri::command]
pub fn list_physical_mail(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<PhysicalMailPiece>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let limit = limit.unwrap_or(20).max(1).min(200);
    let mut stmt = db.conn().prepare(&format!(
        "{MAIL_SELECT}
         ORDER BY synced_at DESC, digest_date DESC, piece_index ASC
         LIMIT ?1"
    ))?;
    let items = stmt
        .query_map(params![limit], map_mail_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

#[tauri::command]
pub fn physical_mail_image_base64(
    state: State<'_, DbState>,
    id: i64,
) -> Result<Option<String>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db
        .conn()
        .prepare("SELECT image_path FROM mail_pieces WHERE id = ?1")?;
    let path: Option<String> = stmt.query_row(params![id], |row| row.get(0)).optional()?;
    let Some(path) = path.filter(|p| !p.is_empty()) else {
        return Ok(None);
    };
    let requested = PathBuf::from(&path);
    let cache = cache_dir_for(db.path());
    if !path_is_within(&cache, &requested) {
        return Err(DbError::Message(
            "mail image path is outside the cache directory".into(),
        ));
    }
    let bytes = fs::read(&requested).map_err(DbError::Io)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    let mime = if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else {
        "image/jpeg"
    };
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(Some(format!("data:{mime};base64,{b64}")))
}
