mod app_state;
mod commands;
mod config;
mod polling;
mod tray;
mod windows;

use app_state::{AppSnapshot, SharedAppState};
use config::repository::ConfigRepository;
use polling::scheduler;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|error| error.to_string())?;
            let repository = ConfigRepository::new(config_dir.join("watch-tower-config.json"));
            let initial_config = repository.load()?;
            let shared_state = SharedAppState::new(repository, initial_config);
            let initial_snapshot: AppSnapshot = tauri::async_runtime::block_on(shared_state.current_snapshot());

            app.manage(shared_state.clone());
            windows::initialize_resident_surfaces(&app.handle(), &initial_snapshot)
                .map_err(|error| error.to_string())?;
            scheduler::spawn(app.handle().clone(), shared_state);

            Ok(())
        })
        .on_window_event(|window, event| {
            windows::handle_window_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap_state,
            commands::save_config,
            commands::select_group,
            commands::poll_now,
            commands::pause_polling,
            commands::resume_polling,
            commands::mark_alert_read,
            commands::open_alert_in_dashboard,
            commands::clear_dashboard_focus_intent,
            commands::set_notifications_enabled
        ])
        .run(tauri::generate_context!())
        .expect("error while running watch tower");
}
