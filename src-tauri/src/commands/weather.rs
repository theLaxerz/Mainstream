//! Current conditions via Open-Meteo (no API key).

use crate::db::{get_setting, now_iso, set_setting, DbError, DbState};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

const SETTING_PLACE: &str = "weather.place";
const SETTING_SNAPSHOT: &str = "weather.snapshot";
const CACHE_MINUTES: i64 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherPlace {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub admin: Option<String>,
    pub country: Option<String>,
    pub units: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherSnapshot {
    pub place: WeatherPlace,
    pub temperature: f64,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub weather_code: i64,
    pub condition: String,
    pub humidity: Option<f64>,
    pub wind_speed: Option<f64>,
    pub fetched_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWeatherPlaceInput {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub admin: Option<String>,
    pub country: Option<String>,
    pub units: Option<String>,
}

fn http_client() -> Result<reqwest::blocking::Client, DbError> {
    reqwest::blocking::Client::builder()
        .user_agent("MainstreamLifeOS/0.1 (+local; Open-Meteo)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| DbError::Message(format!("http client: {e}")))
}

pub fn condition_for_code(code: i64) -> &'static str {
    match code {
        0 => "Clear",
        1 => "Mostly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 | 53 | 55 | 56 | 57 => "Drizzle",
        61 | 63 | 65 | 66 | 67 => "Rain",
        71 | 73 | 75 | 77 => "Snow",
        80 | 81 | 82 => "Showers",
        85 | 86 => "Snow showers",
        95 | 96 | 99 => "Thunderstorm",
        _ => "Mixed",
    }
}

fn normalize_units(raw: Option<&str>) -> String {
    match raw.map(|s| s.trim().to_ascii_lowercase()) {
        Some(s) if s == "celsius" || s == "c" => "celsius".into(),
        _ => "fahrenheit".into(),
    }
}

fn load_place(conn: &Connection) -> Result<Option<WeatherPlace>, DbError> {
    let Some(json) = get_setting(conn, SETTING_PLACE)? else {
        return Ok(None);
    };
    if json.trim().is_empty() {
        return Ok(None);
    }
    let place: WeatherPlace = serde_json::from_str(&json)
        .map_err(|e| DbError::Message(format!("weather place: {e}")))?;
    Ok(Some(place))
}

fn load_snapshot(conn: &Connection) -> Result<Option<WeatherSnapshot>, DbError> {
    let Some(json) = get_setting(conn, SETTING_SNAPSHOT)? else {
        return Ok(None);
    };
    match serde_json::from_str(&json) {
        Ok(snap) => Ok(Some(snap)),
        Err(_) => Ok(None),
    }
}

fn snapshot_fresh(snap: &WeatherSnapshot) -> bool {
    DateTime::parse_from_rfc3339(&snap.fetched_at)
        .ok()
        .map(|dt| {
            let age = Utc::now().signed_duration_since(dt.with_timezone(&Utc));
            age.num_minutes().abs() < CACHE_MINUTES
        })
        .unwrap_or(false)
}

fn fetch_forecast(place: &WeatherPlace) -> Result<WeatherSnapshot, DbError> {
    let units = normalize_units(Some(&place.units));
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,relative_humidity_2m,wind_speed_10m&daily=weather_code,temperature_2m_max,temperature_2m_min&temperature_unit={}&wind_speed_unit=mph&timezone=auto&forecast_days=1",
        place.latitude, place.longitude, units
    );
    let client = http_client()?;
    let body = client
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status()?.text())
        .map_err(|e| DbError::Message(format!("weather fetch: {e}")))?;
    let value: Value = serde_json::from_str(&body)
        .map_err(|e| DbError::Message(format!("weather json: {e}")))?;

    let current = value.get("current").cloned().unwrap_or(Value::Null);
    let daily = value.get("daily").cloned().unwrap_or(Value::Null);
    let temperature = current
        .get("temperature_2m")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| DbError::Message("weather missing temperature".into()))?;
    let weather_code = current
        .get("weather_code")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let humidity = current.get("relative_humidity_2m").and_then(|v| v.as_f64());
    let wind_speed = current.get("wind_speed_10m").and_then(|v| v.as_f64());
    let high = daily
        .get("temperature_2m_max")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_f64());
    let low = daily
        .get("temperature_2m_min")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_f64());

    Ok(WeatherSnapshot {
        place: place.clone(),
        temperature,
        high,
        low,
        weather_code,
        condition: condition_for_code(weather_code).into(),
        humidity,
        wind_speed,
        fetched_at: now_iso(),
    })
}

pub fn run_refresh_weather(
    state: &DbState,
    force: bool,
) -> Result<Option<WeatherSnapshot>, DbError> {
    let place = {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        load_place(db.conn())?
    };
    let Some(place) = place else {
        return Ok(None);
    };

    if !force {
        let cached = {
            let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
            load_snapshot(db.conn())?
        };
        if let Some(snap) = cached {
            if snapshot_fresh(&snap) && snap.place.latitude == place.latitude {
                return Ok(Some(snap));
            }
        }
    }

    let snap = fetch_forecast(&place)?;
    {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        let json = serde_json::to_string(&snap)
            .map_err(|e| DbError::Message(format!("weather snapshot: {e}")))?;
        set_setting(db.conn(), SETTING_SNAPSHOT, &json)?;
    }
    Ok(Some(snap))
}

#[tauri::command]
pub fn search_weather_places(query: String) -> Result<Vec<WeatherPlace>, DbError> {
    let q = query.trim();
    if q.len() < 2 {
        return Ok(vec![]);
    }
    if q.len() > 100 {
        return Err(DbError::Message("search query is too long".into()));
    }
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=6&language=en&format=json",
        urlencoding::encode(q)
    );
    let client = http_client()?;
    let body = client
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status()?.text())
        .map_err(|e| DbError::Message(format!("geocode: {e}")))?;
    let value: Value = serde_json::from_str(&body)
        .map_err(|e| DbError::Message(format!("geocode json: {e}")))?;
    let Some(results) = value.get("results").and_then(|v| v.as_array()) else {
        return Ok(vec![]);
    };
    Ok(results
        .iter()
        .filter_map(|row| {
            let name = row.get("name")?.as_str()?.to_string();
            let latitude = row.get("latitude")?.as_f64()?;
            let longitude = row.get("longitude")?.as_f64()?;
            Some(WeatherPlace {
                name,
                latitude,
                longitude,
                admin: row
                    .get("admin1")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                country: row
                    .get("country")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                units: "fahrenheit".into(),
            })
        })
        .collect())
}

#[tauri::command]
pub fn get_weather(state: State<'_, DbState>) -> Result<Option<WeatherSnapshot>, DbError> {
    run_refresh_weather(&state, false)
}

#[tauri::command]
pub fn save_weather_place(
    state: State<'_, DbState>,
    input: SaveWeatherPlaceInput,
) -> Result<WeatherSnapshot, DbError> {
    let units = normalize_units(input.units.as_deref());
    let place = WeatherPlace {
        name: input.name.trim().to_string(),
        latitude: input.latitude,
        longitude: input.longitude,
        admin: input.admin.filter(|s| !s.trim().is_empty()),
        country: input.country.filter(|s| !s.trim().is_empty()),
        units,
    };
    if place.name.is_empty() {
        return Err(DbError::Message("place name is required".into()));
    }
    {
        let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
        let json = serde_json::to_string(&place)
            .map_err(|e| DbError::Message(format!("weather place: {e}")))?;
        set_setting(db.conn(), SETTING_PLACE, &json)?;
    }
    run_refresh_weather(&state, true)?
        .ok_or_else(|| DbError::Message("weather place saved but forecast missing".into()))
}

#[tauri::command]
pub fn clear_weather_place(state: State<'_, DbState>) -> Result<(), DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    set_setting(db.conn(), SETTING_PLACE, "")?;
    set_setting(db.conn(), SETTING_SNAPSHOT, "")?;
    Ok(())
}
