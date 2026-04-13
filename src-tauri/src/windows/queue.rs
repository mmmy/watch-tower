use crate::app_state::AlertPopupStream;

pub const MAX_VISIBLE_POPUPS: usize = 3;
pub const ALERT_POPUP_SLOT_GAP: i32 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupWindowPlacement {
    pub label: String,
    pub symbol: String,
    pub x: i32,
    pub y: i32,
}

pub fn popup_window_label(symbol: &str) -> String {
    format!("alert-popup:{}", symbol.trim().to_uppercase())
}

pub fn is_popup_window_label(label: &str) -> bool {
    label == crate::windows::alert_popup::ALERT_POPUP_LABEL
        || label.starts_with("alert-popup:")
}

pub fn compute_popup_window_placements(
    monitor: &tauri::Monitor,
    dock_side: &str,
    top_offset: u64,
    width: u32,
    height: u32,
    edge_gap: i32,
    streams: &[AlertPopupStream],
) -> Vec<PopupWindowPlacement> {
    let work_area = monitor.work_area();
    let positions = compute_popup_slot_positions(
        work_area.position.x,
        work_area.position.y,
        work_area.size.width,
        work_area.size.height,
        dock_side,
        top_offset,
        width,
        height,
        edge_gap,
        streams.len(),
    );

    streams
        .iter()
        .zip(positions)
        .map(|(stream, (x, y))| {
            PopupWindowPlacement {
                label: popup_window_label(&stream.symbol),
                symbol: stream.symbol.clone(),
                x,
                y,
            }
        })
        .collect()
}

fn compute_popup_slot_positions(
    work_area_x: i32,
    work_area_y: i32,
    work_area_width: u32,
    work_area_height: u32,
    dock_side: &str,
    top_offset: u64,
    width: u32,
    height: u32,
    edge_gap: i32,
    stream_count: usize,
) -> Vec<(i32, i32)> {
    let max_y = work_area_y + work_area_height as i32 - height as i32;
    let desired_y = work_area_y + top_offset as i32;
    let base_y = desired_y.clamp(work_area_y, max_y);
    let x = match dock_side {
        "left" => work_area_x + edge_gap,
        _ => work_area_x + work_area_width as i32 - width as i32 - edge_gap,
    };

    (0..stream_count.min(MAX_VISIBLE_POPUPS))
        .filter_map(|index| {
            let y = base_y + index as i32 * (height as i32 + ALERT_POPUP_SLOT_GAP);
            (y <= max_y).then_some((x, y))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{compute_popup_slot_positions, is_popup_window_label, popup_window_label};

    #[test]
    fn popup_labels_are_symbol_scoped() {
        assert_eq!(popup_window_label("ethusdt"), "alert-popup:ETHUSDT");
        assert!(is_popup_window_label("alert-popup:ETHUSDT"));
        assert!(is_popup_window_label("alert-popup"));
        assert!(!is_popup_window_label("edge-widget"));
    }

    #[test]
    fn placement_respects_popup_limit_and_slot_spacing() {
        let placements = compute_popup_slot_positions(
            0,
            0,
            1920,
            1080,
            "right",
            96,
            360,
            188,
            16,
            4,
        );

        assert_eq!(placements.len(), 3);
        assert_eq!(placements[1].1 - placements[0].1, 208);
    }
}
