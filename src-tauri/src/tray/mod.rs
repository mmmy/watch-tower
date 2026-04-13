use crate::app_state::AppSnapshot;
use crate::commands;
use crate::windows;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::Manager;

const TRAY_ID: &str = "watch-tower-tray";
const STATUS_ITEM_ID: &str = "status";
const TOGGLE_POLLING_ITEM_ID: &str = "toggle-polling";
const RESTORE_ITEM_ID: &str = "restore-dashboard";
const QUIT_ITEM_ID: &str = "quit";

pub struct ManagedTray {
    #[allow(dead_code)]
    tray: TrayIcon<tauri::Wry>,
    status_item: MenuItem<tauri::Wry>,
    toggle_item: MenuItem<tauri::Wry>,
}

impl ManagedTray {
    fn sync_from_snapshot(&self, snapshot: &AppSnapshot) -> Result<(), String> {
        self.status_item
            .set_text(format!("Status: {}", runtime_status_label(snapshot)))
            .map_err(|error| error.to_string())?;
        self.toggle_item
            .set_text(toggle_polling_menu_text(snapshot))
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

pub fn init(app: &tauri::AppHandle, snapshot: &AppSnapshot) -> Result<(), String> {
    if let Some(tray) = app.try_state::<ManagedTray>() {
        return tray.sync_from_snapshot(snapshot);
    }

    let status_item =
        MenuItem::with_id(app, STATUS_ITEM_ID, "Status: idle", false, None::<&str>)
            .map_err(|error: tauri::Error| error.to_string())?;
    let toggle_item = MenuItem::with_id(
        app,
        TOGGLE_POLLING_ITEM_ID,
        toggle_polling_menu_text(snapshot),
        true,
        None::<&str>,
    )
    .map_err(|error: tauri::Error| error.to_string())?;
    let restore_item =
        MenuItem::with_id(app, RESTORE_ITEM_ID, "Open dashboard", true, None::<&str>)
            .map_err(|error: tauri::Error| error.to_string())?;
    let quit_item =
        MenuItem::with_id(app, QUIT_ITEM_ID, "Quit Watch Tower", true, None::<&str>)
            .map_err(|error: tauri::Error| error.to_string())?;

    let menu = Menu::with_items(app, &[&status_item, &toggle_item, &restore_item, &quit_item])
        .map_err(|error| error.to_string())?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "Tray icon requires a default window icon.".to_string())?;

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .icon(icon)
        .tooltip("Watch Tower")
        .on_menu_event(|app, event| {
            let state = app.state::<crate::app_state::SharedAppState>().inner().clone();
            match event.id().as_ref() {
                TOGGLE_POLLING_ITEM_ID => {
                    tauri::async_runtime::spawn({
                        let app = app.clone();
                        async move {
                            let paused = state.current_snapshot().await.runtime.polling_paused;
                            let _ = commands::set_polling_paused(&app, state, !paused).await;
                        }
                    });
                }
                RESTORE_ITEM_ID => {
                    let _ = windows::restore_main_dashboard(app);
                }
                QUIT_ITEM_ID => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)
        .map_err(|error| error.to_string())?;

    let managed = ManagedTray {
        tray,
        status_item,
        toggle_item,
    };
    managed.sync_from_snapshot(snapshot)?;
    app.manage(managed);

    Ok(())
}

pub fn sync(app: &tauri::AppHandle, snapshot: &AppSnapshot) -> Result<(), String> {
    init(app, snapshot)
}

fn runtime_status_label(snapshot: &AppSnapshot) -> String {
    if snapshot.runtime.polling_paused {
        return if snapshot.health.is_stale {
            "paused · stale".into()
        } else {
            "paused".into()
        };
    }

    if snapshot.health.is_stale {
        format!("{} · stale", snapshot.health.status)
    } else {
        snapshot.health.status.clone()
    }
}

fn toggle_polling_menu_text(snapshot: &AppSnapshot) -> &'static str {
    if snapshot.runtime.polling_paused {
        "Resume polling"
    } else {
        "Pause polling"
    }
}

#[cfg(test)]
mod tests {
    use super::{runtime_status_label, toggle_polling_menu_text};
    use crate::app_state::{
        AlertRuntime, AppSnapshot, DiagnosticsInfo, PollingHealth, RuntimeInfo,
        WidgetBehaviorRuntime,
    };

    fn test_snapshot() -> AppSnapshot {
        AppSnapshot {
            bootstrap_required: false,
            config: None,
            raw_response: None,
            health: PollingHealth {
                status: "success".into(),
                polling_interval_seconds: Some(60),
                is_stale: false,
            },
            diagnostics: DiagnosticsInfo {
                source: "system".into(),
                code: Some("SYNC_OK".into()),
                message: "ready".into(),
                last_attempt_at: None,
                last_successful_sync_at: None,
                next_retry_at: None,
            },
            runtime: RuntimeInfo {
                polling_paused: false,
                last_active_status: None,
            },
            alert_runtime: AlertRuntime::default(),
            widget_runtime: WidgetBehaviorRuntime::default(),
        }
    }

    #[test]
    fn labels_paused_runtime_separately_from_health_status() {
        let mut snapshot = test_snapshot();
        snapshot.runtime.polling_paused = true;
        snapshot.health.is_stale = true;

        assert_eq!(runtime_status_label(&snapshot), "paused · stale");
        assert_eq!(toggle_polling_menu_text(&snapshot), "Resume polling");
    }

    #[test]
    fn labels_active_runtime_with_the_current_health_status() {
        let mut snapshot = test_snapshot();
        snapshot.health.status = "backoff".into();

        assert_eq!(runtime_status_label(&snapshot), "backoff");
        assert_eq!(toggle_polling_menu_text(&snapshot), "Pause polling");
    }
}
