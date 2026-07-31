mod commands;
mod db;

use db::{Db, DbState};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| format!("failed to create app data dir: {e}"))?;
            let db_path = data_dir.join("app.db");
            let database = Db::open(&db_path).map_err(|e| format!("failed to open db: {e}"))?;
            app.manage::<DbState>(Mutex::new(database));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::notes::list_notes,
            commands::notes::get_note,
            commands::notes::create_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::shortcuts::list_shortcuts,
            commands::shortcuts::create_shortcut,
            commands::shortcuts::update_shortcut,
            commands::shortcuts::delete_shortcut,
            commands::shortcuts::open_shortcut,
            commands::settings::get_setting_cmd,
            commands::settings::set_setting_cmd,
            commands::settings::list_settings,
            commands::settings::list_news_prefs,
            commands::settings::upsert_news_pref,
            commands::settings::delete_news_pref,
            commands::news::seed_default_news_feeds,
            commands::news::refresh_news,
            commands::news::list_news,
            commands::news::news_feedback,
            commands::news::open_news_item,
            commands::news::rerank_news,
            commands::news::get_news_last_refresh,
            commands::email::get_email_settings,
            commands::email::save_email_settings,
            commands::email::sync_email,
            commands::email::list_important_emails,
            commands::email::list_all_important_emails,
            commands::email::open_email,
            commands::open::open_target,
            commands::messages::list_unread_messages,
            commands::messages::list_all_unread_messages,
            commands::messages::messages_access_status,
            commands::messages::open_full_disk_access_settings,
            commands::messages::open_message_conversation,
            commands::finance::list_accounts,
            commands::finance::create_account,
            commands::finance::update_account,
            commands::finance::delete_account,
            commands::finance::list_categories,
            commands::finance::list_transactions,
            commands::finance::create_transaction,
            commands::finance::update_transaction,
            commands::finance::delete_transaction,
            commands::finance::get_finance_summary,
            commands::finance::import_transactions_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
