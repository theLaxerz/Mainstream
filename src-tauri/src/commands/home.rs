//! Ring + Blink device status (credentials in Keychain).

use crate::db::{get_setting, set_setting, DbError, DbState};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

const KEYCHAIN_RING: &str = "com.mainstream.lifeos.ring";
const KEYCHAIN_BLINK: &str = "com.mainstream.lifeos.blink";
const SETTING_BLINK_UID: &str = "home.blink_device_uid";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeDevice {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub device_type: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSettings {
    pub ring_connected: bool,
    pub blink_connected: bool,
    pub blink_device_uid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveHomeCredentialsInput {
    pub ring_refresh_token: Option<String>,
    pub blink_email: Option<String>,
    pub blink_password: Option<String>,
    pub blink_device_uid: Option<String>,
}

fn ring_entry() -> Result<Entry, DbError> {
    Entry::new(KEYCHAIN_RING, "refresh_token")
        .map_err(|e| DbError::Message(format!("keychain: {e}")))
}

fn blink_entry(account: &str) -> Result<Entry, DbError> {
    Entry::new(KEYCHAIN_BLINK, account.trim())
        .map_err(|e| DbError::Message(format!("keychain: {e}")))
}

fn load_ring_token() -> Result<Option<String>, DbError> {
    match ring_entry()?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(DbError::Message(format!("ring keychain: {e}"))),
    }
}

fn load_blink_password(email: &str) -> Result<Option<String>, DbError> {
    if email.trim().is_empty() {
        return Ok(None);
    }
    match blink_entry(email)?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(DbError::Message(format!("blink keychain: {e}"))),
    }
}

fn http_client() -> Result<reqwest::blocking::Client, DbError> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|e| DbError::Message(format!("http: {e}")))
}

fn ring_access_token(refresh_token: &str) -> Result<String, DbError> {
    let client = http_client()?;
    let resp = client
        .post("https://oauth.ring.com/oauth/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "grant_type=refresh_token&refresh_token={}",
            urlencoding::encode(refresh_token)
        ))
        .send()
        .map_err(|e| DbError::Message(format!("ring oauth: {e}")))?;
    let status = resp.status();
    let body: Value = resp
        .json()
        .map_err(|e| DbError::Message(format!("ring oauth json: {e}")))?;
    if !status.is_success() {
        return Err(DbError::Message(format!(
            "Ring auth failed ({status}): {body}"
        )));
    }
    body.get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| DbError::Message("Ring response missing access_token".into()))
}

fn fetch_ring_devices(refresh_token: &str) -> Result<Vec<HomeDevice>, DbError> {
    let token = ring_access_token(refresh_token)?;
    let client = http_client()?;
    let resp = client
        .get("https://api.ring.com/clients_api/ring_devices")
        .bearer_auth(&token)
        .send()
        .map_err(|e| DbError::Message(format!("ring devices: {e}")))?;
    let status = resp.status();
    let body: Value = resp
        .json()
        .map_err(|e| DbError::Message(format!("ring devices json: {e}")))?;
    if !status.is_success() {
        return Err(DbError::Message(format!(
            "Ring devices failed ({status}): {body}"
        )));
    }

    let mut out = Vec::new();
    for key in ["doorbots", "stickup_cams", "authorized_doorbots", "chimes"] {
        if let Some(arr) = body.get(key).and_then(|v| v.as_array()) {
            for dev in arr {
                let id = dev
                    .get("id")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "ring".into());
                let desc = dev
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Ring device");
                let battery = dev.get("battery_life").and_then(|v| v.as_str());
                let status = if dev.get("alerts").and_then(|v| v.as_object()).is_some() {
                    "alert"
                } else {
                    "online"
                };
                out.push(HomeDevice {
                    id: format!("ring-{id}"),
                    name: desc.to_string(),
                    vendor: "ring".into(),
                    device_type: key.trim_end_matches('s').replace('_', " "),
                    status: status.into(),
                    detail: battery.map(|b| format!("Battery {b}")),
                });
            }
        }
    }
    Ok(out)
}

fn blink_login(email: &str, password: &str, device_uid: &str) -> Result<(String, u64), DbError> {
    let client = http_client()?;
    let payload = serde_json::json!({
        "username": email,
        "password": password,
        "unique_id": device_uid,
        "client_id": "android",
        "reauth": true
    });
    let resp = client
        .post("https://rest-prod.immedia-semi.com/api/v5/login")
        .json(&payload)
        .send()
        .map_err(|e| DbError::Message(format!("blink login: {e}")))?;
    let status = resp.status();
    let body: Value = resp
        .json()
        .map_err(|e| DbError::Message(format!("blink login json: {e}")))?;
    if !status.is_success() {
        return Err(DbError::Message(format!(
            "Blink login failed ({status}): {body}"
        )));
    }
    let token = body
        .get("auth")
        .and_then(|a| a.get("token"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| DbError::Message("Blink login missing token".into()))?
        .to_string();
    let account_id = body
        .get("account")
        .and_then(|a| a.get("account_id"))
        .and_then(|v| v.as_u64())
        .ok_or_else(|| DbError::Message("Blink login missing account_id".into()))?;
    Ok((token, account_id))
}

fn fetch_blink_devices(email: &str, password: &str, device_uid: &str) -> Result<Vec<HomeDevice>, DbError> {
    let (token, account_id) = blink_login(email, password, device_uid)?;
    let client = http_client()?;
    let homes_url = format!(
        "https://rest-prod.immedia-semi.com/api/v1/accounts/{account_id}/homes"
    );
    let homes: Value = client
        .get(&homes_url)
        .header("TOKEN_AUTH", &token)
        .send()
        .map_err(|e| DbError::Message(format!("blink homes: {e}")))?
        .json()
        .map_err(|e| DbError::Message(format!("blink homes json: {e}")))?;

    let mut out = Vec::new();
    let homes_arr = homes
        .get("homes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for home in homes_arr {
        let home_id = home.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
        let cams_url = format!(
            "https://rest-prod.immedia-semi.com/api/v1/accounts/{account_id}/homes/{home_id}/cameras"
        );
        let cams: Value = client
            .get(&cams_url)
            .header("TOKEN_AUTH", &token)
            .send()
            .map_err(|e| DbError::Message(format!("blink cameras: {e}")))?
            .json()
            .map_err(|e| DbError::Message(format!("blink cameras json: {e}")))?;
        if let Some(arr) = cams.get("cameras").and_then(|v| v.as_array()) {
            for cam in arr {
                let id = cam.get("id").map(|v| v.to_string()).unwrap_or_default();
                let name = cam
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Blink camera");
                let enabled = cam.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                let battery = cam.get("battery").and_then(|v| v.as_str());
                out.push(HomeDevice {
                    id: format!("blink-{id}"),
                    name: name.to_string(),
                    vendor: "blink".into(),
                    device_type: "camera".into(),
                    status: if enabled { "armed" } else { "disabled" }.into(),
                    detail: battery.map(|b| format!("Battery {b}")),
                });
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn get_home_settings(state: State<'_, DbState>) -> Result<HomeSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let blink_device_uid = get_setting(db.conn(), SETTING_BLINK_UID)?.unwrap_or_else(|| {
        format!("mainstream-{}", uuid_simple())
    });
    Ok(HomeSettings {
        ring_connected: load_ring_token()?.is_some(),
        blink_connected: get_setting(db.conn(), "home.blink_email")?
            .filter(|e| !e.is_empty())
            .is_some()
            && load_blink_password(
                &get_setting(db.conn(), "home.blink_email")?.unwrap_or_default(),
            )?
            .is_some(),
        blink_device_uid,
    })
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{n:x}")
}

#[tauri::command]
pub fn save_home_credentials(
    state: State<'_, DbState>,
    input: SaveHomeCredentialsInput,
) -> Result<HomeSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    if let Some(token) = input.ring_refresh_token.as_deref() {
        let token = token.trim();
        if !token.is_empty() {
            ring_entry()?
                .set_password(token)
                .map_err(|e| DbError::Message(format!("store ring token: {e}")))?;
        }
    }
    if let Some(email) = input.blink_email.as_deref() {
        let email = email.trim();
        if !email.is_empty() {
            set_setting(db.conn(), "home.blink_email", email)?;
        }
    }
    if let Some(pw) = input.blink_password.as_deref() {
        let pw = pw.trim();
        if !pw.is_empty() {
            let email = get_setting(db.conn(), "home.blink_email")?.unwrap_or_default();
            if email.is_empty() {
                return Err(DbError::Message("Blink email required before password".into()));
            }
            blink_entry(&email)?
                .set_password(pw)
                .map_err(|e| DbError::Message(format!("store blink password: {e}")))?;
        }
    }
    if let Some(uid) = input.blink_device_uid.as_deref() {
        let uid = uid.trim();
        if !uid.is_empty() {
            set_setting(db.conn(), SETTING_BLINK_UID, uid)?;
        }
    }
    drop(db);
    get_home_settings(state)
}

pub(crate) fn fetch_home_devices(state: &DbState) -> Result<Option<Vec<HomeDevice>>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let blink_email = get_setting(db.conn(), "home.blink_email")?.unwrap_or_default();
    let blink_uid = get_setting(db.conn(), SETTING_BLINK_UID)?.unwrap_or_else(|| uuid_simple());
    let ring_configured = load_ring_token()?.is_some();
    let blink_configured =
        !blink_email.is_empty() && load_blink_password(&blink_email)?.is_some();
    drop(db);

    if !ring_configured && !blink_configured {
        return Ok(None);
    }

    let mut devices = Vec::new();
    let mut errors = Vec::new();

    if let Some(token) = load_ring_token()? {
        match fetch_ring_devices(&token) {
            Ok(mut d) => devices.append(&mut d),
            Err(e) => errors.push(format!("Ring: {e}")),
        }
    }

    if !blink_email.is_empty() {
        if let Some(pw) = load_blink_password(&blink_email)? {
            match fetch_blink_devices(&blink_email, &pw, &blink_uid) {
                Ok(mut d) => devices.append(&mut d),
                Err(e) => errors.push(format!("Blink: {e}")),
            }
        }
    }

    if devices.is_empty() && !errors.is_empty() {
        return Err(DbError::Message(errors.join(" · ")));
    }
    Ok(Some(devices))
}

#[tauri::command]
pub fn list_home_devices(state: State<'_, DbState>) -> Result<Vec<HomeDevice>, DbError> {
    Ok(fetch_home_devices(&state)?.unwrap_or_default())
}
