use crate::app_state::AppSnapshot;
use crate::windows::queue::{compute_popup_window_placements, is_popup_window_label};
use tauri::{
    Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub const ALERT_POPUP_LABEL: &str = "alert-popup";
pub const ALERT_POPUP_WIDTH: f64 = 360.0;
pub const ALERT_POPUP_HEIGHT: f64 = 188.0;
pub const ALERT_POPUP_EDGE_GAP: i32 = 16;

pub fn sync_alert_popup(app: &tauri::AppHandle, snapshot: &AppSnapshot) -> Result<(), String> {
    let Some(config) = snapshot.config.as_ref() else {
        hide_all_popups(app)?;
        return Ok(());
    };

    let Some(monitor) = resolve_monitor(app) else {
        hide_all_popups(app)?;
        return Ok(());
    };

    let placements = compute_popup_window_placements(
        &monitor,
        &config.window_policy.dock_side,
        config.window_policy.top_offset,
        ALERT_POPUP_WIDTH as u32,
        ALERT_POPUP_HEIGHT as u32,
        ALERT_POPUP_EDGE_GAP,
        &snapshot.alert_runtime.visible_popup_streams,
    );

    let active_labels = placements
        .iter()
        .map(|placement| placement.label.clone())
        .collect::<Vec<_>>();

    for placement in placements {
        let window = ensure_alert_popup(app, &placement.label)?;
        window
            .set_size(PhysicalSize::new(
                ALERT_POPUP_WIDTH as u32,
                ALERT_POPUP_HEIGHT as u32,
            ))
            .map_err(|error| error.to_string())?;
        window
            .set_position(PhysicalPosition::new(placement.x, placement.y))
            .map_err(|error| error.to_string())?;
        window.show().map_err(|error| error.to_string())?;
    }

    hide_inactive_popups(app, &active_labels)?;

    Ok(())
}

fn hide_all_popups(app: &tauri::AppHandle) -> Result<(), String> {
    hide_inactive_popups(app, &[])
}

fn hide_inactive_popups(app: &tauri::AppHandle, active_labels: &[String]) -> Result<(), String> {
    for (label, window) in app.webview_windows() {
        if !is_popup_window_label(label.as_str()) {
            continue;
        }

        if active_labels.iter().any(|active_label| active_label == label.as_str()) {
            continue;
        }

        window.hide().map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn ensure_alert_popup(app: &tauri::AppHandle, label: &str) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(label) {
        return Ok(window);
    }

    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
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

fn resolve_monitor(app: &tauri::AppHandle) -> Option<tauri::Monitor> {
    app.primary_monitor().ok().flatten()
}

#[cfg(test)]
mod tests {
    #[test]
    fn alert_popup_dimensions_match_the_multi_card_layout() {
        assert_eq!(super::ALERT_POPUP_WIDTH as u32, 360);
        assert_eq!(super::ALERT_POPUP_HEIGHT as u32, 188);
    }
}
