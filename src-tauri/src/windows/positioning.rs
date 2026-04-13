use crate::app_state::WindowPolicyConfig;
use crate::windows::hover_state::WIDGET_WAKE_ZONE_WIDTH_PX;
use tauri::Monitor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetPlacement {
    pub visible_x: i32,
    pub hidden_x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub wake_zone_width: u32,
}

pub fn compute_widget_placement(
    monitor: &Monitor,
    policy: &WindowPolicyConfig,
) -> WidgetPlacement {
    let work_area = monitor.work_area();
    compute_widget_placement_from_work_area(
        work_area.position.x,
        work_area.position.y,
        work_area.size.width,
        work_area.size.height,
        policy,
    )
}

fn compute_widget_placement_from_work_area(
    origin_x: i32,
    origin_y: i32,
    work_width: u32,
    work_height: u32,
    policy: &WindowPolicyConfig,
) -> WidgetPlacement {
    let available_width = work_width.max(1);
    let available_height = work_height.max(1);

    let width = policy.widget_width.min(available_width as u64) as u32;
    let height = policy.widget_height.min(available_height as u64) as u32;

    let max_y = origin_y + available_height as i32 - height as i32;
    let desired_y = origin_y + policy.top_offset as i32;
    let y = desired_y.clamp(origin_y, max_y);

    let wake_zone_width = WIDGET_WAKE_ZONE_WIDTH_PX.min(width);
    let visible_x = match policy.dock_side.as_str() {
        "left" => origin_x,
        _ => origin_x + available_width as i32 - width as i32,
    };
    let hidden_x = match policy.dock_side.as_str() {
        "left" => origin_x - (width as i32 - wake_zone_width as i32),
        _ => origin_x + available_width as i32 - wake_zone_width as i32,
    };

    WidgetPlacement {
        visible_x,
        hidden_x,
        y,
        width,
        height,
        wake_zone_width,
    }
}

#[cfg(test)]
mod tests {
    use super::compute_widget_placement_from_work_area;
    use crate::app_state::WindowPolicyConfig;

    #[test]
    fn docks_to_the_right_edge_with_top_offset() {
        let policy = WindowPolicyConfig {
            dock_side: "right".into(),
            widget_width: 280,
            widget_height: 720,
            top_offset: 96,
        };

        let placement = compute_widget_placement_from_work_area(0, 0, 1920, 1080, &policy);

        assert_eq!(placement.visible_x, 1640);
        assert_eq!(placement.hidden_x, 1906);
        assert_eq!(placement.y, 96);
        assert_eq!(placement.width, 280);
        assert_eq!(placement.height, 720);
        assert_eq!(placement.wake_zone_width, 14);
    }

    #[test]
    fn docks_to_the_left_edge_and_clamps_to_available_work_area() {
        let policy = WindowPolicyConfig {
            dock_side: "left".into(),
            widget_width: 480,
            widget_height: 720,
            top_offset: 96,
        };

        let placement = compute_widget_placement_from_work_area(100, 40, 320, 400, &policy);

        assert_eq!(placement.visible_x, 100);
        assert_eq!(placement.hidden_x, -206);
        assert_eq!(placement.y, 40);
        assert_eq!(placement.width, 320);
        assert_eq!(placement.height, 400);
    }
}
