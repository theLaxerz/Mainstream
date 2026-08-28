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
            app.manage(commands::blink::BlinkPendingState::new(None));
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
            commands::calendar::calendar_access_status,
            commands::calendar::list_calendar_events,
            commands::calendar::open_calendar_privacy_settings,
            commands::calendar::open_calendar_event,
            commands::mail::sync_physical_mail,
            commands::mail::list_physical_mail,
            commands::mail::physical_mail_image_base64,
            commands::health::get_health_settings,
            commands::health::save_health_settings,
            commands::health::import_health_export,
            commands::health::list_health_days,
            commands::health::health_today_summary,
            commands::home::get_home_settings,
            commands::home::save_home_credentials,
            commands::home::list_home_devices,
            commands::blink::blink_start_login,
            commands::blink::blink_verify_pin,
            commands::blink::blink_disconnect,
            commands::blink::home_device_image_base64,
            commands::blink::blink_capture_snapshot,
            commands::youtube::list_youtube_prefs,
            commands::youtube::upsert_youtube_pref,
            commands::youtube::delete_youtube_pref,
            commands::youtube::refresh_youtube,
            commands::youtube::list_youtube_items,
            commands::youtube::open_youtube_item,
            commands::streaming::list_streaming_providers,
            commands::streaming::get_streaming_settings,
            commands::streaming::save_streaming_settings,
            commands::streaming::refresh_streaming,
            commands::streaming::list_streaming_hot,
            commands::streaming::list_streaming_new,
            commands::streaming::open_streaming_item,
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
            commands::refresh::refresh_dashboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
