use crate::db::DbError;
use std::process::Command;

/// Open a URL or macOS app path/bundle via the system `open` command.
#[tauri::command]
pub fn open_target(kind: String, target: String) -> Result<(), DbError> {
    open_with_system(&kind, &target)
}

pub fn open_with_system(kind: &str, target: &str) -> Result<(), DbError> {
    let target = target.trim();
    if target.is_empty() {
        return Err(DbError::Message("target is required".into()));
    }

    let status = match kind {
        "url" => Command::new("open").arg(target).status(),
        "app" => {
            // Prefer opening by path; fall back to -a / -b for names and bundle IDs.
            if target.ends_with(".app") || target.starts_with('/') {
                Command::new("open").arg(target).status()
            } else if target.contains('.') && !target.contains(' ') {
                // Likely a bundle identifier, e.g. com.apple.Safari
                Command::new("open").args(["-b", target]).status()
            } else {
                Command::new("open").args(["-a", target]).status()
            }
        }
        _ => {
            return Err(DbError::Message(
                "kind must be 'url' or 'app'".into(),
            ))
        }
    }
    .map_err(|e| DbError::Message(format!("failed to spawn open: {e}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(DbError::Message(format!(
            "open exited with status {status}"
        )))
    }
}
