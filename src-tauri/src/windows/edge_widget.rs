use crate::app_state::AppSnapshot;
use crate::windows::positioning::compute_widget_placement;
use tauri::{
    Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub const EDGE_WIDGET_LABEL: &str = "edge-widget";

pub fn sync_edge_widget(app: &tauri::AppHandle, snapshot: &AppSnapshot) -> Result<(), String> {
    let Some(config) = snapshot.config.as_ref() else {
        if let Some(window) = app.get_webview_window(EDGE_WIDGET_LABEL) {
            window.hide().map_err(|error| error.to_string())?;
        }
        return Ok(());
    };

    let window = ensure_edge_widget(app, config.window_policy.widget_width, config.window_policy.widget_height)?;
    let monitor = resolve_monitor(app, &window)?;
    let placement = compute_widget_placement(&monitor, &config.window_policy);

    window
        .set_size(PhysicalSize::new(placement.width, placement.height))
        .map_err(|error| error.to_string())?;
    window
        .set_position(PhysicalPosition::new(placement.x, placement.y))
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;

    Ok(())
}

fn ensure_edge_widget(
    app: &tauri::AppHandle,
    width: u64,
    height: u64,
) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(EDGE_WIDGET_LABEL) {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, EDGE_WIDGET_LABEL, WebviewUrl::App("index.html".into()))
        .title("Watch Tower Widget")
        .inner_size(width as f64, height as f64)
        .min_inner_size(160.0, 320.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .transparent(true)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())
}

fn resolve_monitor(
    app: &tauri::AppHandle,
    window: &WebviewWindow,
) -> Result<tauri::Monitor, String> {
    if let Ok(Some(monitor)) = window.current_monitor() {
        return Ok(monitor);
    }

    if let Ok(Some(monitor)) = app.primary_monitor() {
        return Ok(monitor);
    }

    Err("No monitor available for edge widget placement.".into())
}
