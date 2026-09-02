//! Browser OAuth (PKCE + loopback) for Gmail and Microsoft 365.
//!
//! Tokens live in the macOS Keychain. Mail is still fetched over IMAP using
//! XOAUTH2 so the existing importance filter and Informed Delivery parser keep
//! working — the user just clicks an account in the browser instead of pasting
//! an app password.

use crate::commands::email::{
    apply_provider_defaults, read_settings, EmailSettings, SETTING_AUTH, SETTING_GOOGLE_CLIENT_ID,
    SETTING_HOST, SETTING_MAILBOX, SETTING_MICROSOFT_CLIENT_ID, SETTING_PORT, SETTING_PROVIDER,
    SETTING_USER,
};
use crate::commands::open::open_with_system;
use crate::db::{get_setting, set_setting, DbError, DbState};
use crate::security::{public_http_client, validate_oauth_client_id, validate_oauth_token};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use imap::Authenticator;
use keyring::Entry;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::State;

const KEYCHAIN_SERVICE: &str = "com.mainstream.lifeos.email.oauth";
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(180);
const GOOGLE_AUTH: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const GOOGLE_SCOPE: &str = "https://mail.google.com/ https://www.googleapis.com/auth/userinfo.email openid";
const MICROSOFT_AUTH: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const MICROSOFT_TOKEN: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const MICROSOFT_SCOPE: &str =
    "offline_access openid email profile https://outlook.office.com/IMAP.AccessAsUser.All";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
}

pub struct XOAuth2 {
    pub user: String,
    pub token: String,
}

impl Authenticator for XOAuth2 {
    type Response = String;
    fn process(&self, _challenge: &[u8]) -> Self::Response {
        xoauth2_payload(&self.user, &self.token)
    }
}

pub fn xoauth2_payload(user: &str, token: &str) -> String {
    format!("user={user}\x01auth=Bearer {token}\x01\x01")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartEmailOauthInput {
    pub provider: String,
    pub client_id: Option<String>,
}

fn oauth_entry(provider: &str) -> Result<Entry, DbError> {
    Entry::new(KEYCHAIN_SERVICE, provider)
        .map_err(|e| DbError::Message(format!("keychain entry failed: {e}")))
}

pub fn store_tokens(provider: &str, tokens: &OAuthTokens) -> Result<(), DbError> {
    let json = serde_json::to_string(tokens)
        .map_err(|e| DbError::Message(format!("oauth token encode: {e}")))?;
    oauth_entry(provider)?
        .set_password(&json)
        .map_err(|e| DbError::Message(format!("failed to store OAuth tokens in Keychain: {e}")))
}

pub fn load_tokens(provider: &str) -> Result<Option<OAuthTokens>, DbError> {
    match oauth_entry(provider)?.get_password() {
        Ok(json) => {
            let tokens = serde_json::from_str(&json)
                .map_err(|e| DbError::Message(format!("oauth token decode: {e}")))?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(DbError::Message(format!(
            "failed to read OAuth tokens from Keychain: {e}"
        ))),
    }
}

pub fn delete_tokens(provider: &str) -> Result<(), DbError> {
    match oauth_entry(provider)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(DbError::Message(format!(
            "failed to delete OAuth tokens from Keychain: {e}"
        ))),
    }
}

pub fn has_oauth_tokens(provider: &str) -> bool {
    load_tokens(provider).ok().flatten().is_some()
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn pkce_pair() -> (String, String) {
    let raw = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let digest = Sha256::digest(raw.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(digest);
    (raw, challenge)
}

pub fn parse_http_callback(request: &str) -> Result<HashMap<String, String>, String> {
    let first = request.lines().next().unwrap_or("").trim();
    let path = first
        .strip_prefix("GET ")
        .and_then(|rest| rest.split_whitespace().next())
        .ok_or_else(|| "invalid OAuth callback request".to_string())?;
    let Some((_, query)) = path.split_once('?') else {
        return Ok(HashMap::new());
    };
    Ok(parse_query(query))
}

pub fn parse_query(query: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        if key.is_empty() {
            continue;
        }
        out.insert(url_decode(key), url_decode(value));
    }
    out
}

fn url_decode(input: &str) -> String {
    urlencoding::decode(input)
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| input.replace('+', " "))
}

pub fn email_from_id_token(id_token: &str) -> Option<String> {
    let payload = id_token.split('.').nth(1)?;
    let padded = match payload.len() % 4 {
        0 => payload.to_string(),
        n => format!("{payload}{}", "=".repeat(4 - n)),
    };
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| STANDARD.decode(padded.as_bytes()))
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    ["email", "preferred_username", "upn"]
        .iter()
        .find_map(|key| {
            value
                .get(*key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty() && s.contains('@'))
                .map(|s| s.to_string())
        })
}

fn http_client() -> Result<reqwest::blocking::Client, DbError> {
    public_http_client(20, Some("MainstreamLifeOS/0.1 (+local; OAuth)"))
}

fn wait_for_callback(listener: &TcpListener, expected_state: &str) -> Result<String, DbError> {
    listener
        .set_nonblocking(true)
        .map_err(|e| DbError::Message(format!("oauth listener: {e}")))?;
    let started = Instant::now();
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let params = parse_http_callback(&request).unwrap_or_default();
                if params.is_empty() {
                    let _ = stream.write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n");
                    continue;
                }
                let body = if params.get("state").map(String::as_str) == Some(expected_state)
                    && params.get("code").is_some()
                {
                    html_page(
                        "Signed in",
                        "You can return to Mainstream. This window can be closed.",
                    )
                } else if params.contains_key("error") {
                    html_page(
                        "Sign-in didn’t finish",
                        "You can close this window and try again in Mainstream.",
                    )
                } else {
                    html_page("Almost done", "Return to Mainstream to finish connecting.")
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();

                if let Some(err) = params.get("error") {
                    let desc = params
                        .get("error_description")
                        .map(|s| format!(" ({s})"))
                        .unwrap_or_default();
                    return Err(DbError::Message(format!("Sign-in cancelled: {err}{desc}")));
                }
                let state = params.get("state").map(String::as_str).unwrap_or("");
                if state != expected_state {
                    return Err(DbError::Message(
                        "OAuth state mismatch — try signing in again.".into(),
                    ));
                }
                return params
                    .get("code")
                    .cloned()
                    .ok_or_else(|| DbError::Message("OAuth callback missing code".into()));
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                if started.elapsed() > CALLBACK_TIMEOUT {
                    return Err(DbError::Message(
                        "Timed out waiting for browser sign-in. Click Continue with Google or Microsoft and pick your account.".into(),
                    ));
                }
                thread::sleep(Duration::from_millis(80));
            }
            Err(err) => {
                return Err(DbError::Message(format!("OAuth callback failed: {err}")));
            }
        }
    }
}

fn html_page(title: &str, body: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset='utf-8'><title>{title}</title>
<style>
body{{font-family:-apple-system,BlinkMacSystemFont,sans-serif;background:#eaf3f1;color:#12262c;
display:flex;min-height:100vh;align-items:center;justify-content:center;margin:0}}
main{{background:#fff;padding:2rem 2.2rem;border-radius:18px;max-width:28rem;
box-shadow:0 18px 50px rgba(15,42,51,.18)}}
h1{{font-size:1.4rem;margin:0 0 .5rem}}
p{{margin:0;color:#3d5a63;line-height:1.45}}
</style></head><body><main><h1>{title}</h1><p>{body}</p></main></body></html>"
    )
}

fn resolve_client_id(conn: &Connection, provider: &str, override_id: Option<&str>) -> Result<String, DbError> {
    if let Some(id) = override_id.map(str::trim).filter(|s| !s.is_empty()) {
        let id = validate_oauth_client_id(id)?;
        let key = if provider == "google" {
            SETTING_GOOGLE_CLIENT_ID
        } else {
            SETTING_MICROSOFT_CLIENT_ID
        };
        set_setting(conn, key, &id)?;
        return Ok(id);
    }
    let key = if provider == "google" {
        SETTING_GOOGLE_CLIENT_ID
    } else {
        SETTING_MICROSOFT_CLIENT_ID
    };
    if let Some(id) = get_setting(conn, key)?.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        return validate_oauth_client_id(&id);
    }
    let env_key = if provider == "google" {
        "MAINSTREAM_GOOGLE_CLIENT_ID"
    } else {
        "MAINSTREAM_MICROSOFT_CLIENT_ID"
    };
    if let Ok(id) = std::env::var(env_key) {
        let id = id.trim().to_string();
        if !id.is_empty() {
            let id = validate_oauth_client_id(&id)?;
            set_setting(conn, key, &id)?;
            return Ok(id);
        }
    }
    Err(DbError::Message(client_id_help(provider).into()))
}

fn client_id_help(provider: &str) -> &'static str {
    if provider == "google" {
        "Paste a Google OAuth Desktop client ID (one-time). Google Cloud Console → APIs & Services → Credentials → Create client → Desktop app. Mainstream uses a public PKCE client, so there is no secret."
    } else {
        "Paste a Microsoft public client ID (one-time). Azure Portal → App registrations → New registration → public client / mobile & desktop. Add redirect http://127.0.0.1 and enable public client flows."
    }
}

fn authorize_url(
    provider: &str,
    client_id: &str,
    redirect: &str,
    state: &str,
    challenge: &str,
) -> String {
    let (auth, scope, extra) = if provider == "google" {
        (
            GOOGLE_AUTH,
            GOOGLE_SCOPE,
            "&access_type=offline&prompt=select_account%20consent",
        )
    } else {
        (
            MICROSOFT_AUTH,
            MICROSOFT_SCOPE,
            "&prompt=select_account",
        )
    };
    format!(
        "{auth}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256{extra}",
        urlencoding::encode(client_id),
        urlencoding::encode(redirect),
        urlencoding::encode(scope),
        urlencoding::encode(state),
        urlencoding::encode(challenge),
    )
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    id_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

fn exchange_tokens(
    provider: &str,
    client_id: &str,
    redirect: &str,
    code: &str,
    verifier: &str,
) -> Result<(OAuthTokens, Option<String>), DbError> {
    let endpoint = if provider == "google" {
        GOOGLE_TOKEN
    } else {
        MICROSOFT_TOKEN
    };
    let mut form = vec![
        ("client_id", client_id.to_string()),
        ("code", code.to_string()),
        ("code_verifier", verifier.to_string()),
        ("grant_type", "authorization_code".into()),
        ("redirect_uri", redirect.to_string()),
    ];
    if provider == "microsoft" {
        form.push(("scope", MICROSOFT_SCOPE.into()));
    }
    let resp: TokenResponse = http_client()?
        .post(endpoint)
        .form(&form)
        .send()
        .map_err(|e| DbError::Message(format!("token exchange failed: {e}")))?
        .json()
        .map_err(|e| DbError::Message(format!("token response: {e}")))?;
    if let Some(err) = resp.error {
        let desc = resp.error_description.unwrap_or_default();
        return Err(DbError::Message(format!("OAuth token error: {err} {desc}").trim().into()));
    }
    let access = resp
        .access_token
        .ok_or_else(|| DbError::Message("OAuth token response missing access_token".into()))?;
    let tokens = OAuthTokens {
        access_token: access,
        refresh_token: resp.refresh_token.filter(|s| !s.is_empty()),
        expires_at: now_unix() + resp.expires_in.unwrap_or(3600) - 60,
    };
    Ok((tokens, resp.id_token))
}

fn refresh_tokens(provider: &str, client_id: &str, refresh_token: &str) -> Result<OAuthTokens, DbError> {
    let endpoint = if provider == "google" {
        GOOGLE_TOKEN
    } else {
        MICROSOFT_TOKEN
    };
    let mut form = vec![
        ("client_id", client_id.to_string()),
        ("grant_type", "refresh_token".into()),
        ("refresh_token", refresh_token.to_string()),
    ];
    if provider == "microsoft" {
        form.push(("scope", MICROSOFT_SCOPE.into()));
    }
    let resp: TokenResponse = http_client()?
        .post(endpoint)
        .form(&form)
        .send()
        .map_err(|e| DbError::Message(format!("token refresh failed: {e}")))?
        .json()
        .map_err(|e| DbError::Message(format!("token refresh response: {e}")))?;
    if let Some(err) = resp.error {
        let desc = resp.error_description.unwrap_or_default();
        return Err(DbError::Message(
            format!("Reconnect {provider} in Email settings ({err} {desc})").trim().into(),
        ));
    }
    let access = resp
        .access_token
        .ok_or_else(|| DbError::Message("token refresh missing access_token".into()))?;
    Ok(OAuthTokens {
        access_token: access,
        refresh_token: resp
            .refresh_token
            .filter(|s| !s.is_empty())
            .or_else(|| Some(refresh_token.to_string())),
        expires_at: now_unix() + resp.expires_in.unwrap_or(3600) - 60,
    })
}

fn lookup_google_email(access_token: &str) -> Option<String> {
    let resp = http_client().ok()?.get(GOOGLE_USERINFO)
        .bearer_auth(access_token)
        .send()
        .ok()?;
    let value: serde_json::Value = resp.json().ok()?;
    value
        .get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| s.contains('@'))
}

pub fn ensure_access_token(conn: &Connection, provider: &str) -> Result<String, DbError> {
    let mut tokens = load_tokens(provider)?.ok_or_else(|| {
        DbError::Message(format!(
            "No {provider} sign-in on file. Continue with {provider} in Email settings."
        ))
    })?;
    if tokens.expires_at > now_unix() && !tokens.access_token.is_empty() {
        return validate_oauth_token(&tokens.access_token);
    }
    let refresh = tokens.refresh_token.clone().ok_or_else(|| {
        DbError::Message(format!(
            "{provider} session expired. Continue with {provider} in Email settings."
        ))
    })?;
    let refresh = validate_oauth_token(&refresh)?;
    let client_id = resolve_client_id(conn, provider, None)?;
    tokens = refresh_tokens(provider, &client_id, &refresh)?;
    store_tokens(provider, &tokens)?;
    validate_oauth_token(&tokens.access_token)
}

fn normalize_provider(raw: &str) -> Result<&'static str, DbError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "google" | "gmail" => Ok("google"),
        "microsoft" | "outlook" | "office365" | "m365" => Ok("microsoft"),
        other => Err(DbError::Message(format!(
            "Unsupported sign-in provider '{other}'. Use google or microsoft."
        ))),
    }
}

#[tauri::command]
pub fn start_email_oauth(
    state: State<'_, DbState>,
    input: StartEmailOauthInput,
) -> Result<EmailSettings, DbError> {
    let provider = normalize_provider(&input.provider)?;
    let client_id = {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        resolve_client_id(db.conn(), provider, input.client_id.as_deref())?
    };

    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| DbError::Message(format!("Could not start local sign-in listener: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| DbError::Message(format!("oauth bind: {e}")))?
        .port();
    let redirect = format!("http://127.0.0.1:{port}");
    let (verifier, challenge) = pkce_pair();
    let state_token = uuid::Uuid::new_v4().simple().to_string();
    let url = authorize_url(provider, &client_id, &redirect, &state_token, &challenge);
    open_with_system("url", &url)?;

    let code = wait_for_callback(&listener, &state_token)?;
    let (tokens, id_token) = exchange_tokens(provider, &client_id, &redirect, &code, &verifier)?;
    let mut email = id_token
        .as_deref()
        .and_then(email_from_id_token);
    if email.is_none() && provider == "google" {
        email = lookup_google_email(&tokens.access_token);
    }
    let email = email.ok_or_else(|| {
        DbError::Message(
            "Signed in, but the account email was missing. Try again and click the account you use for mail.".into(),
        )
    })?;

    store_tokens(provider, &tokens)?;
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    set_setting(db.conn(), SETTING_PROVIDER, provider)?;
    set_setting(db.conn(), SETTING_AUTH, "oauth")?;
    set_setting(db.conn(), SETTING_USER, &email)?;
    set_setting(db.conn(), SETTING_MAILBOX, "INBOX")?;
    let (host, port) = apply_provider_defaults(provider);
    set_setting(db.conn(), SETTING_HOST, host)?;
    set_setting(db.conn(), SETTING_PORT, &port.to_string())?;
    read_settings(db.conn())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xoauth2_payload_matches_rfc() {
        let raw = xoauth2_payload("ada@gmail.com", "tok123");
        assert_eq!(raw, "user=ada@gmail.com\x01auth=Bearer tok123\x01\x01");
    }

    #[test]
    fn pkce_challenge_is_s256() {
        let (verifier, challenge) = pkce_pair();
        assert_eq!(verifier.len(), 64);
        let digest = Sha256::digest(verifier.as_bytes());
        assert_eq!(challenge, URL_SAFE_NO_PAD.encode(digest));
        assert!(!challenge.contains('+') && !challenge.contains('/'));
    }

    #[test]
    fn parse_callback_extracts_code_and_state() {
        let req = "GET /?code=abc%2Fde&state=xyz HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        let params = parse_http_callback(req).unwrap();
        assert_eq!(params.get("code").unwrap(), "abc/de");
        assert_eq!(params.get("state").unwrap(), "xyz");
    }

    #[test]
    fn parse_callback_ignores_favicon() {
        let req = "GET /favicon.ico HTTP/1.1\r\n\r\n";
        let params = parse_http_callback(req).unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn email_from_id_token_reads_payload() {
        let payload = URL_SAFE_NO_PAD.encode(
            br#"{"email":"ada@outlook.com","preferred_username":"ada@outlook.com"}"#,
        );
        let token = format!("aaa.{payload}.sig");
        assert_eq!(
            email_from_id_token(&token).as_deref(),
            Some("ada@outlook.com")
        );
    }
}
