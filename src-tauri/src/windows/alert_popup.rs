use crate::app_state::AppSnapshot;
use tauri::{
    Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub const ALERT_POPUP_LABEL: &str = "alert-popup";
const ALERT_POPUP_WIDTH: f64 = 360.0;
const ALERT_POPUP_HEIGHT: f64 = 188.0;
const ALERT_POPUP_EDGE_GAP: i32 = 16;

pub fn sync_alert_popup(app: &tauri::AppHandle, snapshot: &AppSnapshot) -> Result<(), String> {
    let Some(config) = snapshot.config.as_ref() else {
        hide_popup(app)?;
        return Ok(());
    };

    if snapshot.alert_runtime.active_alert.is_none() {
        hide_popup(app)?;
        return Ok(());
    }

    let window = ensure_alert_popup(app)?;
    let monitor = resolve_monitor(app, &window)?;
    let work_area = monitor.work_area();
    let width = ALERT_POPUP_WIDTH as u32;
    let height = ALERT_POPUP_HEIGHT as u32;
    let max_y = work_area.position.y + work_area.size.height as i32 - height as i32;
    let desired_y = work_area.position.y + config.window_policy.top_offset as i32;
    let y = desired_y.clamp(work_area.position.y, max_y);
    let x = match config.window_policy.dock_side.as_str() {
        "left" => work_area.position.x + ALERT_POPUP_EDGE_GAP,
        _ => work_area.position.x + work_area.size.width as i32 - width as i32 - ALERT_POPUP_EDGE_GAP,
    };

    window
        .set_size(PhysicalSize::new(width, height))
        .map_err(|error| error.to_string())?;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;

    Ok(())
}

fn hide_popup(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(ALERT_POPUP_LABEL) {
        window.hide().map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn ensure_alert_popup(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(ALERT_POPUP_LABEL) {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, ALERT_POPUP_LABEL, WebviewUrl::App("index.html".into()))
        .title("Watch Tower Alert")
        .inner_size(ALERT_POPUP_WIDTH, ALERT_POPUP_HEIGHT)
        .min_inner_size(ALERT_POPUP_WIDTH, ALERT_POPUP_HEIGHT)
        .max_inner_size(ALERT_POPUP_WIDTH, ALERT_POPUP_HEIGHT)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
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

    Err("No monitor available for alert popup placement.".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn alert_popup_dimensions_match_the_single_card_layout() {
        assert_eq!(super::ALERT_POPUP_WIDTH as u32, 360);
        assert_eq!(super::ALERT_POPUP_HEIGHT as u32, 188);
    }
}
