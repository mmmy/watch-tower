use signal_desk_native::widget_state::{widget_anchor_position, WindowPlacement, WorkArea};

#[test]
fn widget_anchor_position_uses_top_right_monitor_space() {
    let placement = widget_anchor_position(
        WorkArea {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        },
        96.0,
        96.0,
    );

    assert_eq!(placement, WindowPlacement { x: 1796.0, y: 32.0 });
}

#[test]
fn widget_anchor_position_clamps_inside_tiny_work_area() {
    let placement = widget_anchor_position(
        WorkArea {
            x: 50.0,
            y: 75.0,
            width: 80.0,
            height: 90.0,
        },
        96.0,
        96.0,
    );

    assert_eq!(placement, WindowPlacement { x: 62.0, y: 87.0 });
}
