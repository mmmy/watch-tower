mod app_state;
mod commands;
mod config;
mod polling;

use app_state::SharedAppState;
use config::repository::ConfigRepository;
use polling::scheduler;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|error| error.to_string())?;
            let repository = ConfigRepository::new(config_dir.join("watch-tower-config.json"));
            let initial_config = repository.load()?;
            let shared_state = SharedAppState::new(repository, initial_config);

            app.manage(shared_state.clone());
            scheduler::spawn(app.handle().clone(), shared_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap_state,
            commands::save_config,
            commands::poll_now
        ])
        .run(tauri::generate_context!())
        .expect("error while running watch tower");
}
