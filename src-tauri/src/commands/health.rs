//! Apple Health `export.xml` import (local file / zip).

use crate::db::{get_setting, now_iso, set_setting, DbError, DbState};
use crate::security::{max_health_export_bytes, validate_health_export_path};
use chrono::{DateTime, Local, NaiveDate};
use quick_xml::events::Event;
use quick_xml::Reader;
use rusqlite::params;
use serde::Serialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tauri::State;
use zip::ZipArchive;

const SETTING_EXPORT_PATH: &str = "health.export_path";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthDay {
    pub day: String,
    pub steps: i64,
    pub sleep_minutes: i64,
    pub avg_heart_rate: Option<f64>,
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSettings {
    pub export_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthImportResult {
    pub days_updated: usize,
    pub export_path: String,
}

#[derive(Default)]
struct DayAgg {
    steps: f64,
    sleep_minutes: f64,
    heart_sum: f64,
    heart_count: u32,
}

fn parse_apple_date(raw: &str) -> Option<NaiveDate> {
    let trimmed = raw.trim();
    let date_part = trimmed.split_whitespace().next()?;
    NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()
}

fn record_day(start: &str, end: Option<&str>) -> Option<NaiveDate> {
    parse_apple_date(start).or_else(|| end.and_then(parse_apple_date))
}

fn ingest_export_xml<R: Read>(
    reader: R,
    aggs: &mut std::collections::HashMap<NaiveDate, DayAgg>,
) -> Result<(), DbError> {
    let mut xml = Reader::from_reader(BufReader::new(reader));
    xml.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match xml.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                if e.name().as_ref() != b"Record" {
                    buf.clear();
                    continue;
                }
                let mut record_type = String::new();
                let mut value = String::new();
                let mut start_date = String::new();
                let mut end_date = String::new();
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = attr.unescape_value().unwrap_or_default().to_string();
                    match key.as_str() {
                        "type" => record_type = val,
                        "value" => value = val,
                        "startDate" => start_date = val,
                        "endDate" => end_date = val,
                        _ => {}
                    }
                }
                if let Some(day) = record_day(&start_date, Some(&end_date)) {
                    let entry = aggs.entry(day).or_default();
                    if record_type == "HKQuantityTypeIdentifierStepCount" {
                        if let Ok(v) = value.parse::<f64>() {
                            entry.steps += v;
                        }
                    } else if record_type == "HKQuantityTypeIdentifierHeartRate" {
                        if let Ok(v) = value.parse::<f64>() {
                            entry.heart_sum += v;
                            entry.heart_count += 1;
                        }
                    } else if record_type == "HKCategoryTypeIdentifierSleepAnalysis" {
                        if let (Ok(start), Ok(end)) = (
                            DateTime::parse_from_str(&start_date, "%Y-%m-%d %H:%M:%S %z")
                                .or_else(|_| DateTime::parse_from_rfc3339(&start_date)),
                            DateTime::parse_from_str(&end_date, "%Y-%m-%d %H:%M:%S %z")
                                .or_else(|_| DateTime::parse_from_rfc3339(&end_date)),
                        ) {
                            let mins = (end - start).num_minutes().max(0) as f64;
                            entry.sleep_minutes += mins;
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(DbError::Message(format!("Health XML parse error: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn open_export_reader(path: &Path) -> Result<Box<dyn Read>, DbError> {
    validate_health_export_path(&path.to_string_lossy())?;
    let meta = std::fs::metadata(path).map_err(DbError::Io)?;
    if meta.len() > max_health_export_bytes() {
        return Err(DbError::Message(
            "Health export is too large to import safely".into(),
        ));
    }
    let lower = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if lower == "zip" {
        let file = File::open(path).map_err(DbError::Io)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| DbError::Message(format!("zip: {e}")))?;
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| DbError::Message(format!("zip entry: {e}")))?;
            let name = file.name().to_ascii_lowercase();
            if name.contains("..") {
                continue;
            }
            if name.ends_with("export.xml") || name.ends_with("apple_health_export/export.xml") {
                if file.size() > max_health_export_bytes() {
                    return Err(DbError::Message(
                        "Health export.xml inside zip is too large".into(),
                    ));
                }
                let mut limited = (&mut file).take(max_health_export_bytes());
                let mut buf = Vec::new();
                limited.read_to_end(&mut buf).map_err(DbError::Io)?;
                if buf.len() as u64 >= max_health_export_bytes() {
                    return Err(DbError::Message(
                        "Health export.xml inside zip is too large".into(),
                    ));
                }
                return Ok(Box::new(std::io::Cursor::new(buf)));
            }
        }
        return Err(DbError::Message(
            "Zip does not contain export.xml (Apple Health export)".into(),
        ));
    }
    let file = File::open(path).map_err(DbError::Io)?;
    let limited = BufReader::new(file).take(max_health_export_bytes());
    Ok(Box::new(limited))
}

fn upsert_aggs(
    conn: &rusqlite::Connection,
    aggs: &std::collections::HashMap<NaiveDate, DayAgg>,
) -> Result<usize, DbError> {
    let imported_at = now_iso();
    let mut count = 0usize;
    for (day, agg) in aggs {
        let avg_hr = if agg.heart_count > 0 {
            Some(agg.heart_sum / agg.heart_count as f64)
        } else {
            None
        };
        conn.execute(
            "INSERT INTO health_daily (day, steps, sleep_minutes, avg_heart_rate, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(day) DO UPDATE SET
               steps = excluded.steps,
               sleep_minutes = excluded.sleep_minutes,
               avg_heart_rate = excluded.avg_heart_rate,
               imported_at = excluded.imported_at",
            params![
                day.format("%Y-%m-%d").to_string(),
                agg.steps.round() as i64,
                agg.sleep_minutes.round() as i64,
                avg_hr,
                imported_at,
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

fn import_path(conn: &rusqlite::Connection, path: &Path) -> Result<usize, DbError> {
    let mut aggs = std::collections::HashMap::new();
    let reader = open_export_reader(path)?;
    ingest_export_xml(reader, &mut aggs)?;
    upsert_aggs(conn, &aggs)
}

pub(crate) fn try_import_configured(conn: &rusqlite::Connection) -> Result<Option<usize>, DbError> {
    let export_path = match get_setting(conn, SETTING_EXPORT_PATH)? {
        Some(path) if !path.trim().is_empty() => path,
        _ => return Ok(None),
    };
    let path = Path::new(export_path.trim());
    if !path.exists() {
        return Ok(None);
    }
    import_path(conn, path).map(Some)
}

#[tauri::command]
pub fn get_health_settings(state: State<'_, DbState>) -> Result<HealthSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let export_path = get_setting(db.conn(), SETTING_EXPORT_PATH)?.unwrap_or_default();
    Ok(HealthSettings { export_path })
}

#[tauri::command]
pub fn save_health_settings(
    state: State<'_, DbState>,
    export_path: String,
) -> Result<HealthSettings, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let trimmed = export_path.trim();
    if !trimmed.is_empty() {
        validate_health_export_path(trimmed)?;
    }
    set_setting(db.conn(), SETTING_EXPORT_PATH, trimmed)?;
    let export_path = get_setting(db.conn(), SETTING_EXPORT_PATH)?.unwrap_or_default();
    Ok(HealthSettings { export_path })
}

#[tauri::command]
pub fn import_health_export(
    state: State<'_, DbState>,
    path: Option<String>,
) -> Result<HealthImportResult, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let export_path = path
        .filter(|p| !p.trim().is_empty())
        .or_else(|| get_setting(db.conn(), SETTING_EXPORT_PATH).ok().flatten())
        .ok_or_else(|| DbError::Message("Set a Health export path first.".into()))?;
    validate_health_export_path(export_path.trim())?;
    let p = Path::new(export_path.trim());
    if !p.exists() {
        return Err(DbError::Message(format!(
            "Health export not found at {}",
            p.display()
        )));
    }
    let days = import_path(db.conn(), p)?;
    Ok(HealthImportResult {
        days_updated: days,
        export_path: export_path.trim().to_string(),
    })
}

#[tauri::command]
pub fn list_health_days(
    state: State<'_, DbState>,
    limit: Option<i64>,
) -> Result<Vec<HealthDay>, DbError> {
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let limit = limit.unwrap_or(7).clamp(1, 90);
    let mut stmt = db.conn().prepare(
        "SELECT day, steps, sleep_minutes, avg_heart_rate, imported_at
         FROM health_daily
         ORDER BY day DESC
         LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(HealthDay {
                day: row.get(0)?,
                steps: row.get(1)?,
                sleep_minutes: row.get(2)?,
                avg_heart_rate: row.get(3)?,
                imported_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[tauri::command]
pub fn health_today_summary(state: State<'_, DbState>) -> Result<Option<HealthDay>, DbError> {
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let db = state.lock().map_err(|e| DbError::Message(e.to_string()))?;
    let mut stmt = db.conn().prepare(
        "SELECT day, steps, sleep_minutes, avg_heart_rate, imported_at
         FROM health_daily WHERE day = ?1",
    )?;
    let mut rows = stmt.query(params![today])?;
    if let Some(row) = rows.next()? {
        Ok(Some(HealthDay {
            day: row.get(0)?,
            steps: row.get(1)?,
            sleep_minutes: row.get(2)?,
            avg_heart_rate: row.get(3)?,
            imported_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}
