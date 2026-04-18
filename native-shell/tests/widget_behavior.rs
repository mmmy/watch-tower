use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use signal_desk_native::shell::interpolate_widget_placement;
use signal_desk_native::widget_state::{
    build_widget_placement, hide_widget_placement, load_widget_placement, restore_widget_placement,
    reveal_widget_placement, save_widget_placement, WidgetDockSide, WidgetPlacement, WorkArea,
};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("signal-desk-widget-{name}-{nonce}"))
}

#[test]
fn build_widget_placement_snaps_to_side_docks() {
    let work_area = Some(WorkArea {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    });

    let left = build_widget_placement(8.0, 120.0, 50.0, 50.0, work_area);
    assert_eq!(left.dock, WidgetDockSide::Left);
    assert!(left.auto_hidden);

    let right = build_widget_placement(1860.0, 120.0, 50.0, 50.0, work_area);
    assert_eq!(right.dock, WidgetDockSide::Right);
    assert!(right.auto_hidden);

    let top = build_widget_placement(400.0, 4.0, 50.0, 50.0, work_area);
    assert_eq!(top.dock, WidgetDockSide::Top);
    assert!(!top.auto_hidden);
}

#[test]
fn restore_and_visibility_helpers_preserve_hidden_side_docks() {
    let work_area = Some(WorkArea {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    });
    let saved = WidgetPlacement {
        x: 1896.0,
        y: 200.0,
        dock: WidgetDockSide::Right,
        auto_hidden: true,
    };

    let restored = restore_widget_placement(saved, 50.0, 50.0, work_area);
    assert_eq!(restored.dock, WidgetDockSide::Right);
    assert!(restored.auto_hidden);

    let revealed = reveal_widget_placement(restored, 50.0, work_area);
    assert_eq!(revealed.dock, WidgetDockSide::Right);
    assert!(!revealed.auto_hidden);

    let hidden = hide_widget_placement(revealed, 50.0, work_area);
    assert_eq!(hidden.dock, WidgetDockSide::Right);
    assert!(hidden.auto_hidden);
}

#[test]
fn widget_placement_round_trips_through_disk() {
    let temp_dir = unique_temp_dir("persistence");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let path = temp_dir.join("widget-placement.json");
    let placement = WidgetPlacement {
        x: 111.0,
        y: 222.0,
        dock: WidgetDockSide::Bottom,
        auto_hidden: false,
    };

    save_widget_placement(&path, &placement).expect("save widget placement");
    let loaded = load_widget_placement(&path).expect("load widget placement");

    assert_eq!(loaded, placement);

    fs::remove_dir_all(&temp_dir).expect("cleanup temp dir");
}

#[test]
fn interpolate_widget_placement_eases_toward_target() {
    let from = WidgetPlacement {
        x: 0.0,
        y: 100.0,
        dock: WidgetDockSide::Left,
        auto_hidden: true,
    };
    let to = WidgetPlacement {
        x: 200.0,
        y: 20.0,
        dock: WidgetDockSide::Left,
        auto_hidden: false,
    };

    let halfway = interpolate_widget_placement(from, to, 0.5);
    assert!(halfway.x > 100.0);
    assert!(halfway.y < 60.0);
    assert_eq!(halfway.dock, WidgetDockSide::Left);
    assert!(!halfway.auto_hidden);

    let finished = interpolate_widget_placement(from, to, 1.0);
    assert_eq!(finished, to);
}
