//! Blink OAuth v2 (PKCE) + homescreen camera stills.
//!
//! Blink retired the old `rest-prod ... /api/v5/login` password grant.
//! Mainstream now follows the same flow Home Assistant / blinkpy use:
//! authorize → CSRF sign-in → optional SMS/email PIN → token exchange,
//! then `homescreen` for cameras and JPEG thumbnails.

use crate::commands::home::HomeDevice;
use crate::db::{get_setting, set_setting, DbError, DbState};
use crate::security::{
    ensure_public_resolved_host, parse_public_https_url, path_is_within, public_http_client,
    validate_cache_file_stem, validate_dns_label,
};
use base64::Engine;
use keyring::Entry;
use reqwest::blocking::Client;
use reqwest::header::{
    HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE, USER_AGENT,
};
use reqwest::redirect::Policy;
use rusqlite::Connection;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

const KEYCHAIN_SERVICE: &str = "com.mainstream.lifeos.blink";
const SETTING_EMAIL: &str = "home.blink_email";
const SETTING_HARDWARE: &str = "home.blink_hardware_id";
const SETTING_ACCOUNT: &str = "home.blink_account_id";
const SETTING_TIER: &str = "home.blink_tier";
const SETTING_ACCESS: &str = "home.blink_access_token";
const SETTING_ACCESS_EXP: &str = "home.blink_access_expires";

const OAUTH_AUTHORIZE: &str = "https://api.oauth.blink.com/oauth/v2/authorize";
const OAUTH_SIGNIN: &str = "https://api.oauth.blink.com/oauth/v2/signin";
const OAUTH_2FA: &str = "https://api.oauth.blink.com/oauth/v2/2fa/verify";
const OAUTH_TOKEN: &str = "https://api.oauth.blink.com/oauth/token";
const TIER_INFO: &str = "https://rest-prod.immedia-semi.com/api/v1/users/tier_info";
const REDIRECT_URI: &str = "immedia-blink://applinks.blink.com/signin/callback";
const OAUTH_UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Mobile/15E148 Safari/604.1";
const TOKEN_UA: &str = "Blink/2511191620 CFNetwork/3860.200.71 Darwin/25.1.0";

pub struct BlinkOauthPending {
    client: Client,
    cookies: HashMap<String, String>,
    csrf_token: String,
    code_verifier: String,
    code_challenge: String,
    hardware_id: String,
}

pub type BlinkPendingState = Mutex<Option<BlinkOauthPending>>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlinkLoginResult {
    pub status: String,
    pub detail: Option<String>,
}

fn refresh_entry() -> Result<Entry, DbError> {
    Entry::new(KEYCHAIN_SERVICE, "oauth_refresh")
        .map_err(|e| DbError::Message(format!("keychain: {e}")))
}

pub fn load_refresh_token() -> Result<Option<String>, DbError> {
    match refresh_entry()?.get_password() {
        Ok(t) if !t.trim().is_empty() => Ok(Some(t)),
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(DbError::Message(format!("blink keychain: {e}"))),
    }
}

fn store_refresh_token(token: &str) -> Result<(), DbError> {
    refresh_entry()?
        .set_password(token)
        .map_err(|e| DbError::Message(format!("store blink refresh: {e}")))
}

pub fn blink_is_connected() -> Result<bool, DbError> {
    Ok(load_refresh_token()?.is_some())
}

fn oauth_client() -> Result<Client, DbError> {
    Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| DbError::Message(format!("http client: {e}")))
}

fn store_set_cookie(cookies: &mut HashMap<String, String>, headers: &HeaderMap) {
    for val in headers.get_all(SET_COOKIE) {
        let Ok(raw) = val.to_str() else { continue };
        let pair = raw.split(';').next().unwrap_or("").trim();
        let Some((name, value)) = pair.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty() {
            cookies.insert(name.to_string(), value.trim().to_string());
        }
    }
}

fn with_cookies(
    req: reqwest::blocking::RequestBuilder,
    cookies: &HashMap<String, String>,
) -> reqwest::blocking::RequestBuilder {
    if cookies.is_empty() {
        return req;
    }
    let header = cookies
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; ");
    req.header(COOKIE, header)
}

fn rest_client() -> Result<Client, DbError> {
    // Same redirect/SSRF policy as other untrusted fetches: Blink thumbnails and
    // region endpoints must not follow redirects onto loopback or link-local.
    public_http_client(30, None)
}

fn pkce_pair() -> (String, String) {
    let raw = Uuid::new_v4().as_bytes().to_vec();
    let extra = Uuid::new_v4().as_bytes().to_vec();
    let mut bytes = raw;
    bytes.extend_from_slice(&extra);
    let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes[..32]);
    let digest = Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
    (verifier, challenge)
}

fn extract_csrf(html: &str) -> Option<String> {
    let marker = "id=\"oauth-args\"";
    if let Some(idx) = html.find(marker) {
        let rest = &html[idx..];
        if let Some(gt) = rest.find('>') {
            let after = &rest[gt + 1..];
            if let Some(end) = after.find("</script>") {
                let json = after[..end].trim();
                if let Ok(v) = serde_json::from_str::<Value>(json) {
                    if let Some(t) = v
                        .get("csrf-token")
                        .or_else(|| v.get("csrfToken"))
                        .and_then(|x| x.as_str())
                    {
                        if !t.is_empty() {
                            return Some(t.to_string());
                        }
                    }
                }
            }
        }
    }
    for key in ["csrf-token", "csrfToken", "csrf_token"] {
        let pat = format!("\"{key}\"");
        if let Some(idx) = html.find(&pat) {
            let rest = &html[idx + pat.len()..];
            if let Some(colon) = rest.find(':') {
                let val = rest[colon + 1..].trim_start();
                if let Some(stripped) = val.strip_prefix('"') {
                    if let Some(end) = stripped.find('"') {
                        let token = &stripped[..end];
                        if token.len() >= 8 {
                            return Some(token.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn location_code(headers: &HeaderMap) -> Option<String> {
    let loc = headers.get("location")?.to_str().ok()?;
    for part in loc.split(['?', '&', '#']) {
        if let Some(code) = part.strip_prefix("code=") {
            return Some(
                urlencoding::decode(code)
                    .unwrap_or(std::borrow::Cow::Borrowed(code))
                    .into_owned(),
            );
        }
    }
    None
}

fn load_or_create_hardware_id(conn: &Connection) -> Result<String, DbError> {
    if let Some(existing) = get_setting(conn, SETTING_HARDWARE)? {
        let trimmed = existing.trim();
        if Uuid::parse_str(trimmed).is_ok() {
            return Ok(trimmed.to_ascii_uppercase());
        }
    }
    let id = Uuid::new_v4().to_string().to_ascii_uppercase();
    set_setting(conn, SETTING_HARDWARE, &id)?;
    Ok(id)
}

fn start_oauth_session(
    email: &str,
    password: &str,
    hardware_id: &str,
) -> Result<(BlinkOauthPending, BlinkLoginResult), DbError> {
    let (verifier, challenge) = pkce_pair();
    let client = oauth_client()?;
    let mut cookies = HashMap::new();

    let authorize = client
        .get(OAUTH_AUTHORIZE)
        .header(USER_AGENT, OAUTH_UA)
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .query(&[
            ("app_brand", "blink"),
            ("app_version", "50.1"),
            ("client_id", "ios"),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("device_brand", "Apple"),
            ("device_model", "iPhone16,1"),
            ("device_os_version", "26.1"),
            ("hardware_id", hardware_id),
            ("redirect_uri", REDIRECT_URI),
            ("response_type", "code"),
            ("scope", "client"),
        ])
        .send()
        .map_err(|e| DbError::Message(format!("Blink authorize: {e}")))?;
    store_set_cookie(&mut cookies, authorize.headers());
    let auth_status = authorize.status().as_u16();
    if !(200..400).contains(&auth_status) {
        return Err(DbError::Message(format!(
            "Blink authorize failed ({auth_status}). Try again in a minute."
        )));
    }

    let signin = with_cookies(
        client
            .get(OAUTH_SIGNIN)
            .header(USER_AGENT, OAUTH_UA)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        &cookies,
    )
    .send()
    .map_err(|e| DbError::Message(format!("Blink sign-in page: {e}")))?;
    store_set_cookie(&mut cookies, signin.headers());
    let html = signin
        .text()
        .map_err(|e| DbError::Message(format!("Blink sign-in body: {e}")))?;
    let csrf = extract_csrf(&html).ok_or_else(|| {
        DbError::Message("Blink sign-in page did not include a CSRF token.".into())
    })?;

    let resp = with_cookies(
        client
            .post(OAUTH_SIGNIN)
            .header(USER_AGENT, OAUTH_UA)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("Origin", "https://api.oauth.blink.com")
            .header("Referer", OAUTH_SIGNIN)
            .form(&[
                ("username", email),
                ("password", password),
                ("csrf-token", csrf.as_str()),
            ]),
        &cookies,
    )
    .send()
    .map_err(|e| DbError::Message(format!("Blink sign-in: {e}")))?;
    store_set_cookie(&mut cookies, resp.headers());
    let status = resp.status().as_u16();
    let pending = BlinkOauthPending {
        client,
        cookies,
        csrf_token: csrf,
        code_verifier: verifier,
        code_challenge: challenge,
        hardware_id: hardware_id.to_string(),
    };

    if status == 412 || status == 202 {
        return Ok((
            pending,
            BlinkLoginResult {
                status: "pin_required".into(),
                detail: Some("Blink sent a verification code to your phone or email.".into()),
            },
        ));
    }
    if (301..400).contains(&status) {
        return Ok((
            pending,
            BlinkLoginResult {
                status: "ok".into(),
                detail: None,
            },
        ));
    }
    Err(DbError::Message(
        "Blink rejected that email or password.".into(),
    ))
}

fn json_id(v: &Value) -> String {
    match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string().trim_matches('"').to_string(),
    }
}

fn finish_oauth(pending: &mut BlinkOauthPending) -> Result<(String, String, Option<u64>), DbError> {
    let resp = with_cookies(
        pending
            .client
            .get(OAUTH_AUTHORIZE)
            .header(USER_AGENT, OAUTH_UA)
            .header("Accept", "*/*")
            .header("Referer", OAUTH_SIGNIN)
            .query(&[
                ("app_brand", "blink"),
                ("app_version", "50.1"),
                ("client_id", "ios"),
                ("code_challenge", pending.code_challenge.as_str()),
                ("code_challenge_method", "S256"),
                ("device_brand", "Apple"),
                ("device_model", "iPhone16,1"),
                ("device_os_version", "26.1"),
                ("hardware_id", pending.hardware_id.as_str()),
                ("redirect_uri", REDIRECT_URI),
                ("response_type", "code"),
                ("scope", "client"),
            ]),
        &pending.cookies,
    )
    .send()
    .map_err(|e| DbError::Message(format!("Blink code: {e}")))?;
    store_set_cookie(&mut pending.cookies, resp.headers());
    let code = location_code(resp.headers())
        .ok_or_else(|| DbError::Message("Blink did not return an authorization code.".into()))?;

    let token_resp = with_cookies(
        pending
            .client
            .post(OAUTH_TOKEN)
            .header(USER_AGENT, TOKEN_UA)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&[
                ("app_brand", "blink"),
                ("client_id", "ios"),
                ("code", code.as_str()),
                ("code_verifier", pending.code_verifier.as_str()),
                ("grant_type", "authorization_code"),
                ("hardware_id", pending.hardware_id.as_str()),
                ("redirect_uri", REDIRECT_URI),
                ("scope", "client"),
            ]),
        &pending.cookies,
    )
    .send()
    .map_err(|e| DbError::Message(format!("Blink token: {e}")))?;
    if !token_resp.status().is_success() {
        return Err(DbError::Message("Blink token exchange failed.".into()));
    }
    let body: Value = token_resp
        .json()
        .map_err(|e| DbError::Message(format!("Blink token json: {e}")))?;
    let access = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DbError::Message("Blink token missing access_token.".into()))?
        .to_string();
    let refresh = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DbError::Message("Blink token missing refresh_token.".into()))?
        .to_string();
    let expires_in = body.get("expires_in").and_then(|v| v.as_u64());
    Ok((access, refresh, expires_in))
}

fn persist_tokens(
    conn: &Connection,
    access: &str,
    refresh: &str,
    expires_in: Option<u64>,
) -> Result<(), DbError> {
    store_refresh_token(refresh)?;
    set_setting(conn, SETTING_ACCESS, access)?;
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        + expires_in.unwrap_or(3600);
    set_setting(conn, SETTING_ACCESS_EXP, &exp.to_string())?;
    Ok(())
}

fn refresh_access_token(conn: &Connection) -> Result<String, DbError> {
    let hardware = load_or_create_hardware_id(conn)?;
    let refresh =
        load_refresh_token()?.ok_or_else(|| DbError::Message("Blink is not connected.".into()))?;
    let client = rest_client()?;
    let resp = client
        .post(OAUTH_TOKEN)
        .header(USER_AGENT, TOKEN_UA)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh.as_str()),
            ("client_id", "ios"),
            ("scope", "client"),
            ("hardware_id", hardware.as_str()),
        ])
        .send()
        .map_err(|e| DbError::Message(format!("Blink refresh: {e}")))?;
    if !resp.status().is_success() {
        return Err(DbError::Message(
            "Blink session expired — sign in again.".into(),
        ));
    }
    let body: Value = resp
        .json()
        .map_err(|e| DbError::Message(format!("Blink refresh json: {e}")))?;
    let access = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DbError::Message("Blink refresh missing access_token.".into()))?
        .to_string();
    if let Some(new_refresh) = body.get("refresh_token").and_then(|v| v.as_str()) {
        store_refresh_token(new_refresh)?;
    }
    let expires_in = body.get("expires_in").and_then(|v| v.as_u64());
    persist_tokens(
        conn,
        &access,
        &load_refresh_token()?.unwrap_or(refresh),
        expires_in,
    )?;
    Ok(access)
}

fn current_access_token(conn: &Connection) -> Result<String, DbError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let exp = get_setting(conn, SETTING_ACCESS_EXP)?
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    if let Some(token) = get_setting(conn, SETTING_ACCESS)? {
        if !token.is_empty() && exp > now + 60 {
            return Ok(token);
        }
    }
    refresh_access_token(conn)
}

fn ensure_tier(conn: &Connection, access: &str) -> Result<(String, String), DbError> {
    if let (Some(tier), Some(account)) = (
        get_setting(conn, SETTING_TIER)?,
        get_setting(conn, SETTING_ACCOUNT)?,
    ) {
        if !tier.is_empty() && !account.is_empty() {
            if let (Ok(tier), Ok(account)) = (
                validate_dns_label(&tier, "Blink region"),
                validate_cache_file_stem(&account),
            ) {
                return Ok((tier, account));
            }
        }
    }
    let client = rest_client()?;
    let resp = client
        .get(TIER_INFO)
        .header(AUTHORIZATION, format!("Bearer {access}"))
        .header("TOKEN_AUTH", access)
        .send()
        .map_err(|e| DbError::Message(format!("Blink tier: {e}")))?;
    if !resp.status().is_success() {
        return Err(DbError::Message("Could not load Blink region info.".into()));
    }
    let body: Value = resp
        .json()
        .map_err(|e| DbError::Message(format!("Blink tier json: {e}")))?;
    let tier = validate_dns_label(
        body.get("tier")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DbError::Message("Blink tier missing.".into()))?,
        "Blink region",
    )?;
    let account = body
        .get("account_id")
        .map(|v| match v {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            _ => String::new(),
        })
        .filter(|s| !s.is_empty())
        .ok_or_else(|| DbError::Message("Blink account id missing.".into()))?;
    let account = validate_cache_file_stem(&account)?;
    set_setting(conn, SETTING_TIER, &tier)?;
    set_setting(conn, SETTING_ACCOUNT, &account)?;
    Ok((tier, account))
}

fn rest_base(tier: &str) -> Result<String, DbError> {
    let tier = validate_dns_label(tier, "Blink region")?;
    Ok(format!("https://rest-{tier}.immedia-semi.com"))
}

fn auth_headers(access: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&format!("Bearer {access}")) {
        headers.insert(AUTHORIZATION, v);
    }
    if let Ok(v) = HeaderValue::from_str(access) {
        headers.insert("TOKEN_AUTH", v);
    }
    headers
}

fn thumbnail_url(base: &str, raw: &str) -> Option<String> {
    let mut path = raw.trim().to_string();
    if path.starts_with("http://") || path.starts_with("https://") {
        if !path.contains('.') {
            path.push_str(".jpg");
        }
        return parse_public_https_url(&path)
            .ok()
            .map(|u| u.as_str().to_string());
    }
    if path.contains(['\n', '\r', '\0', ' ', '\\']) || path.contains("..") {
        return None;
    }
    if !path.starts_with('/') {
        path = format!("/{path}");
    }
    if !path.contains(".jpg") && !path.contains('?') {
        path.push_str(".jpg");
    }
    parse_public_https_url(&format!("{base}{path}"))
        .ok()
        .map(|u| u.as_str().to_string())
}

fn thumb_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("blink-thumbs")
}

fn thumb_path(data_dir: &Path, device_id: &str) -> Result<PathBuf, DbError> {
    let stem = validate_cache_file_stem(device_id)?;
    Ok(thumb_dir(data_dir).join(format!("{stem}.jpg")))
}

fn download_thumbnail(
    client: &Client,
    headers: &HeaderMap,
    url: &str,
    dest: &Path,
) -> Result<bool, DbError> {
    let url = parse_public_https_url(url)?;
    ensure_public_resolved_host(&url)?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let resp = client
        .get(url)
        .headers(headers.clone())
        .send()
        .map_err(|e| DbError::Message(format!("Blink thumbnail: {e}")))?;
    if !resp.status().is_success() {
        return Ok(false);
    }
    let bytes = resp
        .bytes()
        .map_err(|e| DbError::Message(format!("Blink thumbnail body: {e}")))?;
    if bytes.len() < 32 || bytes.len() > 2_000_000 {
        return Ok(false);
    }
    std::fs::write(dest, &bytes)?;
    Ok(true)
}

fn parse_devices(homescreen: &Value, base: &str) -> Vec<(HomeDevice, Option<String>)> {
    let mut out = Vec::new();
    let groups = [
        ("cameras", "camera"),
        ("owls", "mini"),
        ("doorbells", "doorbell"),
        ("doorbell_buttons", "doorbell"),
    ];
    for (key, kind) in groups {
        let Some(arr) = homescreen.get(key).and_then(|v| v.as_array()) else {
            continue;
        };
        for cam in arr {
            let id = cam
                .get("id")
                .map(json_id)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "unknown".into());
            if validate_cache_file_stem(&id).is_err() {
                continue;
            }
            let name = cam
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Blink camera")
                .to_string();
            let enabled = cam.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            let status_raw = cam.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let battery = cam
                .get("battery")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    cam.get("signals")
                        .and_then(|s| s.get("battery"))
                        .map(json_id)
                });
            let temp = cam
                .get("signals")
                .and_then(|s| s.get("temp"))
                .and_then(|v| v.as_i64());
            let network = cam
                .get("network_id")
                .map(json_id)
                .filter(|s| !s.is_empty())
                .and_then(|n| validate_cache_file_stem(&n).ok());
            let mut detail_parts = Vec::new();
            if let Some(b) = battery {
                detail_parts.push(format!("Battery {b}"));
            }
            if let Some(t) = temp {
                detail_parts.push(format!("{t}°"));
            }
            let status = if !enabled {
                "disabled"
            } else if status_raw == "offline" {
                "offline"
            } else if status_raw.is_empty() {
                "online"
            } else {
                status_raw
            };
            let thumb = cam
                .get("thumbnail")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| thumbnail_url(base, s));
            let device_id = format!("blink-{id}");
            out.push((
                HomeDevice {
                    id: device_id,
                    name,
                    vendor: "blink".into(),
                    device_type: kind.into(),
                    status: status.into(),
                    detail: if detail_parts.is_empty() {
                        None
                    } else {
                        Some(detail_parts.join(" · "))
                    },
                    thumbnail_available: false,
                    snapshot_ready: kind == "camera" && network.is_some(),
                    network_id: network,
                    camera_id: Some(id),
                },
                thumb,
            ));
        }
    }
    out
}

pub fn fetch_blink_cameras(
    conn: &Connection,
    data_dir: Option<&Path>,
) -> Result<Vec<HomeDevice>, DbError> {
    if load_refresh_token()?.is_none() {
        return Ok(vec![]);
    }
    let access = current_access_token(conn)?;
    let (tier, account) = ensure_tier(conn, &access)?;
    let base = rest_base(&tier)?;
    let url = format!("{base}/api/v3/accounts/{account}/homescreen");
    let client = rest_client()?;
    let headers = auth_headers(&access);
    let resp = client
        .get(&url)
        .headers(headers.clone())
        .send()
        .map_err(|e| DbError::Message(format!("Blink homescreen: {e}")))?;
    if !resp.status().is_success() {
        return Err(DbError::Message(
            "Could not load Blink cameras. Try signing in again.".into(),
        ));
    }
    let body: Value = resp
        .json()
        .map_err(|e| DbError::Message(format!("Blink homescreen json: {e}")))?;
    let parsed = parse_devices(&body, &base);
    let mut devices = Vec::new();
    for (mut device, thumb_url) in parsed {
        if let (Some(dir), Some(url)) = (data_dir, thumb_url) {
            let dest = thumb_path(dir, &device.id)?;
            if download_thumbnail(&client, &headers, &url, &dest).unwrap_or(false) {
                device.thumbnail_available = true;
            } else if dest.is_file() {
                device.thumbnail_available = true;
            }
        }
        devices.push(device);
    }
    Ok(devices)
}

#[tauri::command]
pub fn blink_start_login(
    state: State<'_, DbState>,
    pending: State<'_, BlinkPendingState>,
    email: String,
    password: String,
) -> Result<BlinkLoginResult, DbError> {
    let email = email.trim().to_string();
    let password = password.trim().to_string();
    if email.is_empty() || password.is_empty() {
        return Err(DbError::Message(
            "Blink email and password are required.".into(),
        ));
    }
    let hardware = {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        set_setting(db.conn(), SETTING_EMAIL, &email)?;
        load_or_create_hardware_id(db.conn())?
    };
    let (session, result) = start_oauth_session(&email, &password, &hardware)?;
    if result.status == "ok" {
        let mut session = session;
        let (access, refresh, expires_in) = finish_oauth(&mut session)?;
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        persist_tokens(db.conn(), &access, &refresh, expires_in)?;
        let _ = ensure_tier(db.conn(), &access);
        *pending
            .lock()
            .map_err(|e| DbError::Message(e.to_string()))? = None;
        return Ok(BlinkLoginResult {
            status: "connected".into(),
            detail: Some("Blink is connected.".into()),
        });
    }
    *pending
        .lock()
        .map_err(|e| DbError::Message(e.to_string()))? = Some(session);
    Ok(result)
}

#[tauri::command]
pub fn blink_verify_pin(
    state: State<'_, DbState>,
    pending: State<'_, BlinkPendingState>,
    pin: String,
) -> Result<BlinkLoginResult, DbError> {
    let pin = pin.trim().to_string();
    if pin.len() < 4 {
        return Err(DbError::Message(
            "Enter the Blink verification code.".into(),
        ));
    }
    let mut session = pending
        .lock()
        .map_err(|e| DbError::Message(e.to_string()))?
        .take()
        .ok_or_else(|| {
            DbError::Message("No Blink sign-in in progress. Enter your password again.".into())
        })?;
    let resp = with_cookies(
        session
            .client
            .post(OAUTH_2FA)
            .header(USER_AGENT, OAUTH_UA)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("Origin", "https://api.oauth.blink.com")
            .header("Referer", OAUTH_SIGNIN)
            .form(&[
                ("2fa_code", pin.as_str()),
                ("csrf-token", session.csrf_token.as_str()),
                ("remember_me", "false"),
            ]),
        &session.cookies,
    )
    .send()
    .map_err(|e| DbError::Message(format!("Blink PIN: {e}")))?;
    store_set_cookie(&mut session.cookies, resp.headers());
    let status = resp.status().as_u16();
    if !(200..400).contains(&status) {
        *pending
            .lock()
            .map_err(|e| DbError::Message(e.to_string()))? = Some(session);
        return Err(DbError::Message(
            "That Blink code was not accepted. Check the SMS/email and try again.".into(),
        ));
    }
    let (access, refresh, expires_in) = finish_oauth(&mut session)?;
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    persist_tokens(db.conn(), &access, &refresh, expires_in)?;
    let _ = ensure_tier(db.conn(), &access);
    Ok(BlinkLoginResult {
        status: "connected".into(),
        detail: Some("Blink is connected.".into()),
    })
}

#[tauri::command]
pub fn blink_disconnect(state: State<'_, DbState>) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let _ = refresh_entry()?.delete_credential();
    for key in [
        SETTING_ACCESS,
        SETTING_ACCESS_EXP,
        SETTING_ACCOUNT,
        SETTING_TIER,
    ] {
        set_setting(db.conn(), key, "")?;
    }
    Ok(())
}

#[tauri::command]
pub fn home_device_image_base64(app: AppHandle, id: String) -> Result<Option<String>, DbError> {
    if !id.starts_with("blink-") {
        return Ok(None);
    }
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| DbError::Message(e.to_string()))?;
    let Ok(path) = thumb_path(&data_dir, &id) else {
        return Ok(None);
    };
    let cache = thumb_dir(&data_dir);
    if !path_is_within(&cache, &path) {
        return Ok(None);
    }
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(Some(format!("data:image/jpeg;base64,{b64}")))
}

#[tauri::command]
pub fn blink_capture_snapshot(
    app: AppHandle,
    state: State<'_, DbState>,
    id: String,
) -> Result<HomeDevice, DbError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| DbError::Message(e.to_string()))?;
    let camera_id = validate_cache_file_stem(id.strip_prefix("blink-").unwrap_or(&id))?;
    let (access, base, network) = {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        let access = current_access_token(db.conn())?;
        let (tier, _) = ensure_tier(db.conn(), &access)?;
        let devices = fetch_blink_cameras(db.conn(), None)?;
        let network = devices
            .iter()
            .find(|d| d.id == id)
            .and_then(|d| d.network_id.clone())
            .ok_or_else(|| DbError::Message("Camera network not found.".into()))?;
        let network = validate_cache_file_stem(&network)?;
        (access, rest_base(&tier)?, network)
    };
    let client = rest_client()?;
    let headers = auth_headers(&access);
    let url = format!("{base}/network/{network}/camera/{camera_id}/thumbnail");
    let resp = client
        .post(&url)
        .headers(headers.clone())
        .send()
        .map_err(|e| DbError::Message(format!("Blink snap: {e}")))?;
    if !resp.status().is_success() {
        return Err(DbError::Message(
            "Blink could not start a new snapshot. The camera may be offline.".into(),
        ));
    }
    // Give the sync module a moment, then reload homescreen + thumbnail.
    std::thread::sleep(Duration::from_secs(4));
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let devices = fetch_blink_cameras(db.conn(), Some(&data_dir))?;
    devices
        .into_iter()
        .find(|d| d.id == id)
        .ok_or_else(|| DbError::Message("Camera not found after snapshot.".into()))
}
