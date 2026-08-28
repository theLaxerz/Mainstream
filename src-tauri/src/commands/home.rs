//! Ring + Blink device status (credentials in Keychain).

use crate::commands::blink::{self, blink_is_connected};
use crate::db::{get_setting, set_setting, DbError, DbState};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use tauri::{AppHandle, Manager, State};

const KEYCHAIN_RING: &str = "com.mainstream.lifeos.ring";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeDevice {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub device_type: String,
    pub status: String,
    pub detail: Option<String>,
    #[serde(default)]
    pub thumbnail_available: bool,
    #[serde(default)]
    pub snapshot_ready: bool,
    #[serde(default)]
    pub network_id: Option<String>,
    #[serde(default)]
    pub camera_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSettings {
    pub ring_connected: bool,
    pub blink_connected: bool,
    pub blink_email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveHomeCredentialsInput {
    pub ring_refresh_token: Option<String>,
    pub blink_email: Option<String>,
}

fn ring_entry() -> Result<Entry, DbError> {
    Entry::new(KEYCHAIN_RING, "refresh_token")
        .map_err(|e| DbError::Message(format!("keychain: {e}")))
}

fn load_ring_token() -> Result<Option<String>, DbError> {
    match ring_entry()?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(DbError::Message(format!("ring keychain: {e}"))),
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
                    thumbnail_available: false,
                    snapshot_ready: false,
                    network_id: None,
                    camera_id: None,
                });
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn get_home_settings(state: State<'_, DbState>) -> Result<HomeSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    Ok(HomeSettings {
        ring_connected: load_ring_token()?.is_some(),
        blink_connected: blink_is_connected()?,
        blink_email: get_setting(db.conn(), "home.blink_email")?.unwrap_or_default(),
    })
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
    drop(db);
    get_home_settings(state)
}

pub(crate) fn fetch_home_devices(
    state: &DbState,
    data_dir: Option<&Path>,
) -> Result<Option<Vec<HomeDevice>>, DbError> {
    let ring_configured = load_ring_token()?.is_some();
    let blink_configured = blink_is_connected()?;

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

    if blink_configured {
        let blink = {
            let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
            blink::fetch_blink_cameras(db.conn(), data_dir)
        };
        match blink {
            Ok(mut d) => devices.append(&mut d),
            Err(e) => {
                errors.push(format!("Blink: {e}"));
                devices.push(HomeDevice {
                    id: "blink-error".into(),
                    name: "Blink",
                    vendor: "blink".into(),
                    device_type: "account".into(),
                    status: "error".into(),
                    detail: Some(e.to_string()),
                    thumbnail_available: false,
                    snapshot_ready: false,
                    network_id: None,
                    camera_id: None,
                });
            }
        }
    }

    if devices.is_empty() && !errors.is_empty() {
        return Err(DbError::Message(errors.join(" · ")));
    }
    Ok(Some(devices))
}

#[tauri::command]
pub fn list_home_devices(
    app: AppHandle,
    state: State<'_, DbState>,
) -> Result<Vec<HomeDevice>, DbError> {
    let data_dir = app.path().app_data_dir().ok();
    Ok(fetch_home_devices(&state, data_dir.as_deref())?.unwrap_or_default())
}
