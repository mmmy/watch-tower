pub mod alert_popup;
pub mod edge_widget;
pub mod hover_state;
pub mod positioning;
pub mod queue;

use crate::app_state::AppSnapshot;
use tauri::{Manager, WindowEvent};

pub const MAIN_DASHBOARD_LABEL: &str = "main-dashboard";

pub fn initialize_resident_surfaces(
    app: &tauri::AppHandle,
    snapshot: &AppSnapshot,
) -> Result<(), String> {
    crate::tray::init(app, snapshot)?;
    edge_widget::sync_edge_widget(app, snapshot)?;
    alert_popup::sync_alert_popup(app, snapshot)?;
    Ok(())
}

pub fn sync_resident_surfaces(
    app: &tauri::AppHandle,
    snapshot: &AppSnapshot,
) -> Result<(), String> {
    crate::tray::sync(app, snapshot)?;
    edge_widget::sync_edge_widget(app, snapshot)?;
    alert_popup::sync_alert_popup(app, snapshot)?;
    Ok(())
}

pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    if window.label() != MAIN_DASHBOARD_LABEL {
        return;
    }

    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
    }
}

pub fn restore_main_dashboard(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_DASHBOARD_LABEL)
        .ok_or_else(|| "Main dashboard window is not available.".to_string())?;

    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;

    Ok(())
}
