mod ai;
mod analytics;
mod articles;
mod backup;
mod commands;
mod credentials;
mod dashboards;
mod db;
mod desktop;
mod discovery;
mod error;
mod feeds;
mod integrations;
mod logging;
mod privacy;
mod search;
mod security;
mod state;
mod validation;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let log_dir = app.path().app_log_dir()?;
            let data_dir = app.path().app_data_dir()?;
            let log_guard = logging::initialize(&log_dir)?;
            let db_path = data_dir.join("bergag.sqlite3");
            let backup_dir = data_dir.join("backups");
            let connection = db::initialize(&db_path)?;
            backup::service::initialize(&backup_dir)?;

            app.manage(AppState {
                connection: Mutex::new(connection),
                db_path,
                backup_dir,
                log_dir,
                log_guard: Mutex::new(Some(log_guard)),
            });
            feeds::scheduler::start(app.handle().clone());
            desktop::tray::setup(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::backups::backup_create,
            commands::backups::backup_list,
            commands::backups::backup_verify,
            commands::backups::backup_delete,
            commands::backups::backup_restore,
            commands::privacy::privacy_dashboard_get,
            commands::desktop::desktop_notifications_status,
            commands::desktop::desktop_notifications_update,
            commands::desktop::desktop_notifications_test,
            commands::desktop::desktop_tray_status,
            commands::desktop::desktop_tray_set_close_behavior,
            commands::desktop::desktop_startup_status,
            commands::desktop::desktop_startup_set_enabled,
            commands::desktop::desktop_updates_status,
            commands::desktop::desktop_updates_check,
            commands::desktop::desktop_security_status,
            commands::articles::articles_list,
            commands::articles::articles_get,
            commands::articles::articles_open,
            commands::articles::articles_extract,
            commands::articles::articles_set_state,
            commands::articles::articles_bulk_update,
            commands::articles::articles_history,
            commands::articles::articles_highlights,
            commands::articles::articles_add_note,
            commands::articles::articles_update_note,
            commands::articles::articles_delete_note,
            commands::articles::articles_add_highlight,
            commands::articles::articles_delete_highlight,
            commands::articles::articles_list_collections,
            commands::articles::articles_create_collection,
            commands::articles::articles_delete_collection,
            commands::articles::articles_add_to_collection,
            commands::articles::articles_remove_from_collection,
            commands::articles::articles_cache_stats,
            commands::articles::articles_clear_cache,
            commands::profiles::profiles_list,
            commands::profiles::profiles_get_active,
            commands::profiles::profiles_create,
            commands::profiles::profiles_rename,
            commands::profiles::profiles_switch,
            commands::profiles::profiles_delete,
            commands::settings::settings_get_profile,
            commands::settings::settings_set_profile,
            commands::settings::settings_delete_profile,
            commands::settings::settings_get_active_profile_id,
            commands::layouts::layouts_list,
            commands::layouts::layouts_get_active,
            commands::layouts::layouts_save,
            commands::layouts::layouts_activate,
            commands::layouts::layouts_delete,
            commands::diagnostics::diagnostics_get,
            commands::diagnostics::diagnostics_recent_logs,
            commands::feeds::feeds_list_folders,
            commands::feeds::feeds_create_folder,
            commands::feeds::feeds_delete_folder,
            commands::feeds::feeds_list,
            commands::feeds::feeds_get,
            commands::feeds::feeds_preview,
            commands::feeds::feeds_discover,
            commands::feeds::feeds_subscribe,
            commands::feeds::feeds_update,
            commands::feeds::feeds_delete,
            commands::feeds::feeds_refresh,
            commands::feeds::feeds_logs,
            commands::feeds::feeds_items,
            commands::feeds::feeds_export_opml,
            commands::feeds::feeds_import_opml,
            commands::discovery::discovery_list,
            commands::discovery::discovery_preview,
            commands::discovery::discovery_add,
            commands::discovery::discovery_subscribe,
            commands::discovery::discovery_set_trust,
            commands::discovery::discovery_delete,
            commands::discovery::labels_list,
            commands::discovery::labels_create,
            commands::discovery::labels_update,
            commands::discovery::labels_delete,
            commands::discovery::labels_for_item,
            commands::discovery::labels_items,
            commands::discovery::labels_assign,
            commands::discovery::labels_unassign,
            commands::discovery::rules_list,
            commands::discovery::rules_get,
            commands::discovery::rules_create,
            commands::discovery::rules_update,
            commands::discovery::rules_delete,
            commands::discovery::rules_run,
            commands::discovery::rules_run_enabled,
            commands::discovery::rules_history,
            commands::discovery::rules_smart_feed,
            commands::search::search_query,
            commands::search::search_stats,
            commands::search::search_presets_list,
            commands::search::search_presets_save,
            commands::search::search_presets_delete,
            commands::search::stories_list,
            commands::search::stories_get,
            commands::search::stories_timeline,
            commands::search::stories_detect,
            commands::search::stories_delete,
            commands::search::articles_related,
            commands::dashboards::dashboards_list,
            commands::dashboards::dashboard_save,
            commands::dashboards::dashboard_delete,
            commands::dashboards::dashboard_set_default,
            commands::dashboards::widgets_list,
            commands::dashboards::widget_add,
            commands::dashboards::widget_update,
            commands::dashboards::widget_delete,
            commands::analytics::analytics_reading,
            commands::analytics::analytics_library,
            commands::analytics::analytics_storage,
            commands::analytics::analytics_feed_health,
            commands::analytics::analytics_unread_feeds,
            commands::analytics::analytics_top_labels,
            commands::analytics::analytics_articles,
            commands::analytics::briefing_build,
            commands::ai::ai_settings_get,
            commands::ai::ai_settings_update,
            commands::ai::ai_providers_list,
            commands::ai::ai_provider_save,
            commands::ai::ai_provider_delete,
            commands::ai::ai_set_api_key,
            commands::ai::ai_clear_api_key,
            commands::ai::ai_provider_test,
            commands::ai::ai_summarize,
            commands::ai::ai_suggest_labels,
            commands::ai::ai_ask,
            commands::ai::ai_usage_stats,
            commands::ai::ai_usage_events,
            commands::ai::ai_usage_clear,
            commands::integrations::integrations_platforms_list,
            commands::integrations::integrations_preview,
            commands::integrations::integrations_add,
            commands::integrations::integrations_list,
            commands::integrations::integrations_set_trust,
            commands::integrations::integrations_refresh,
            commands::integrations::integrations_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Bergag");
}
