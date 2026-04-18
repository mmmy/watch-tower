use signal_desk_native::app_state::AppState;
use signal_desk_native::runtime::{runtime_snapshot_from_config, AppConfig, WatchGroup};

#[test]
fn ui_snapshot_exposes_runtime_control_state() {
    let mut config = AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    };
    config.ui.always_on_top = false;
    config.ui.edge_mode = true;
    config.ui.edge_width = 240.0;
    config.ui.notifications = false;
    config.ui.sound = false;

    let mut runtime_snapshot = runtime_snapshot_from_config(config);
    runtime_snapshot.always_on_top = false;
    runtime_snapshot.edge_mode = true;

    let snapshot = AppState::new(runtime_snapshot).snapshot();

    assert!(!snapshot.always_on_top);
    assert!(snapshot.edge_mode);
    assert_eq!(snapshot.edge_width_label, "240 px");
    assert!(!snapshot.notifications_enabled);
    assert!(!snapshot.sound_enabled);
    assert!(snapshot.save_hint.contains("240px"));
    assert!(snapshot.save_hint.contains("通知关"));
    assert!(snapshot.save_hint.contains("声音关"));
}

#[test]
fn ui_snapshot_marks_unknown_connection_before_first_remote_attempt() {
    let runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });

    let snapshot = AppState::new(runtime_snapshot).snapshot();

    assert_eq!(snapshot.connection_label, "未连接");
    assert_eq!(snapshot.connection_tone, "lagging");
}

#[test]
fn ui_snapshot_marks_last_connection_failure() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });
    runtime_snapshot.last_connection_ok = Some(false);

    let snapshot = AppState::new(runtime_snapshot).snapshot();

    assert_eq!(snapshot.connection_label, "连接失败");
    assert_eq!(snapshot.connection_tone, "offline");
}

#[test]
fn ui_snapshot_marks_last_refresh_failure_for_widget_state() {
    let runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });
    let mut state = AppState::new(runtime_snapshot);

    assert!(!state.snapshot().last_refresh_failed);

    state.set_runtime_error(
        runtime_snapshot_from_config(AppConfig {
            groups: vec![WatchGroup::default()],
            ..Default::default()
        }),
        "timeout".into(),
    );

    let failed = state.snapshot();
    assert!(failed.last_refresh_failed);
    assert!(failed.status_text.contains("轮询失败"));
}

#[test]
fn header_rows_expose_group_unread_counts_and_support_batch_mark_read() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });

    runtime_snapshot.signals[0].group_name = "BTC Main".into();
    runtime_snapshot.signals[0].signal_type = "divMacd".into();
    runtime_snapshot.signals[0].period = "60".into();
    runtime_snapshot.signals[0].unread = true;

    runtime_snapshot.signals[1].group_name = "BTC Main".into();
    runtime_snapshot.signals[1].signal_type = "divMacd".into();
    runtime_snapshot.signals[1].period = "15".into();
    runtime_snapshot.signals[1].unread = true;

    runtime_snapshot.signals[2].group_name = "BTC Main".into();
    runtime_snapshot.signals[2].signal_type = "divMacd".into();
    runtime_snapshot.signals[2].period = "5".into();
    runtime_snapshot.signals[2].unread = false;

    runtime_snapshot.signals[3].group_name = "BTC Main".into();
    runtime_snapshot.signals[3].signal_type = "divRsi".into();
    runtime_snapshot.signals[3].period = "1".into();
    runtime_snapshot.signals[3].unread = true;

    runtime_snapshot.unread_count = runtime_snapshot
        .signals
        .iter()
        .filter(|signal| signal.unread)
        .count();

    let mut state = AppState::new(runtime_snapshot);
    let before = state.snapshot();

    assert!(before.signal_rows[0].is_header);
    assert_eq!(before.signal_rows[0].unread_count, 2);
    assert_eq!(before.unread_count, 3);

    let keys = state.activate_row_at(0);
    let after = state.snapshot();

    assert_eq!(keys.len(), 2);
    assert!(after.signal_rows[0].pending);
    assert_eq!(after.signal_rows[0].unread_count, 0);
    assert_eq!(after.unread_count, 1);
}

#[test]
fn signal_rows_no_longer_mark_single_items_as_read() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });

    runtime_snapshot.signals[0].group_name = "BTC Main".into();
    runtime_snapshot.signals[0].signal_type = "divMacd".into();
    runtime_snapshot.signals[0].period = "60".into();
    runtime_snapshot.signals[0].unread = true;

    runtime_snapshot.signals[1].group_name = "BTC Main".into();
    runtime_snapshot.signals[1].signal_type = "divMacd".into();
    runtime_snapshot.signals[1].period = "15".into();
    runtime_snapshot.signals[1].unread = true;

    runtime_snapshot.unread_count = runtime_snapshot
        .signals
        .iter()
        .filter(|signal| signal.unread)
        .count();

    let mut state = AppState::new(runtime_snapshot);
    let keys = state.activate_row_at(1);
    let after = state.snapshot();

    assert!(keys.is_empty());
    assert_eq!(after.signal_rows[0].unread_count, 2);
    assert_eq!(after.unread_count, 2);
}

#[test]
fn signal_rows_expose_timeline_marker_ratio_for_recent_events() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    runtime_snapshot.signals[0].group_name = "BTC Main".into();
    runtime_snapshot.signals[0].signal_type = "divMacd".into();
    runtime_snapshot.signals[0].period = "60".into();
    runtime_snapshot.signals[0].trigger_time = now_ms - 2 * 60 * 60 * 1000;

    let snapshot = AppState::new(runtime_snapshot).snapshot();
    let signal_row = &snapshot.signal_rows[1];

    assert!(signal_row.timeline_visible);
    assert!((signal_row.timeline_ratio - (57.0 / 59.0)).abs() < 0.05);
}

#[test]
fn signal_rows_expose_timeline_direction_for_short_signals() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    runtime_snapshot.signals[0].group_name = "BTC Main".into();
    runtime_snapshot.signals[0].signal_type = "divMacd".into();
    runtime_snapshot.signals[0].period = "60".into();
    runtime_snapshot.signals[0].trigger_time = now_ms - 60 * 60 * 1000;
    runtime_snapshot.signals[0].side = -1;

    let snapshot = AppState::new(runtime_snapshot).snapshot();
    let signal_row = &snapshot.signal_rows[1];

    assert!(signal_row.timeline_visible);
    assert!(!signal_row.timeline_positive);
}

#[test]
fn signal_rows_follow_configured_period_order() {
    let config = AppConfig {
        groups: vec![WatchGroup {
            periods: vec!["10D".into(), "W".into(), "60".into(), "15".into(), "1".into()],
            ..WatchGroup::default()
        }],
        ..Default::default()
    };
    let mut runtime_snapshot = runtime_snapshot_from_config(config);

    runtime_snapshot.signals[0].period = "10D".into();
    runtime_snapshot.signals[1].period = "W".into();
    runtime_snapshot.signals[2].period = "60".into();
    runtime_snapshot.signals[3].period = "15".into();
    runtime_snapshot.signals[4].period = "1".into();

    runtime_snapshot.signals[0].trigger_time = 1;
    runtime_snapshot.signals[1].trigger_time = 5;
    runtime_snapshot.signals[2].trigger_time = 4;
    runtime_snapshot.signals[3].trigger_time = 3;
    runtime_snapshot.signals[4].trigger_time = 2;

    let snapshot = AppState::new(runtime_snapshot).snapshot();

    assert_eq!(snapshot.signal_rows[1].title, "10D");
    assert_eq!(snapshot.signal_rows[2].title, "W");
    assert_eq!(snapshot.signal_rows[3].title, "60");
    assert_eq!(snapshot.signal_rows[4].title, "15");
    assert_eq!(snapshot.signal_rows[5].title, "1");
}

#[test]
fn signal_rows_toggle_single_item_read_state() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });

    runtime_snapshot.signals[0].group_name = "BTC Main".into();
    runtime_snapshot.signals[0].signal_type = "divMacd".into();
    runtime_snapshot.signals[0].period = "60".into();
    runtime_snapshot.signals[0].unread = true;

    runtime_snapshot.unread_count = 1;

    let mut state = AppState::new(runtime_snapshot);
    let mark_read = state.toggle_signal_row_at(1);
    let after_read = state.snapshot();

    assert_eq!(
        mark_read,
        Some((
            signal_desk_native::runtime::SignalMutationInput {
                group_id: "group-1".into(),
                signal_type: "divMacd".into(),
                period: "60".into(),
            },
            true,
        ))
    );
    assert!(!after_read.signal_rows[1].unread);
    assert_eq!(after_read.unread_count, 0);

    let mark_unread = state.toggle_signal_row_at(1);
    let after_unread = state.snapshot();

    assert_eq!(
        mark_unread,
        Some((
            signal_desk_native::runtime::SignalMutationInput {
                group_id: "group-1".into(),
                signal_type: "divMacd".into(),
                period: "60".into(),
            },
            false,
        ))
    );
    assert!(after_unread.signal_rows[1].unread);
    assert_eq!(after_unread.unread_count, 1);
}

#[test]
fn unread_items_follow_signal_order_and_clear_after_mark_read() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });

    runtime_snapshot.signals[0].symbol = "BTCUSDT".into();
    runtime_snapshot.signals[0].group_name = "BTC Main".into();
    runtime_snapshot.signals[0].signal_type = "divMacd".into();
    runtime_snapshot.signals[0].period = "60".into();
    runtime_snapshot.signals[0].trigger_time = 3_600_000;
    runtime_snapshot.signals[0].unread = true;

    runtime_snapshot.signals[1].symbol = "BTCUSDT".into();
    runtime_snapshot.signals[1].group_name = "BTC Main".into();
    runtime_snapshot.signals[1].signal_type = "divMacd".into();
    runtime_snapshot.signals[1].period = "15".into();
    runtime_snapshot.signals[1].trigger_time = 7_200_000;
    runtime_snapshot.signals[1].unread = true;

    runtime_snapshot.unread_count = runtime_snapshot
        .signals
        .iter()
        .filter(|signal| signal.unread)
        .count();

    let mut state = AppState::new(runtime_snapshot);
    let before = state.snapshot();

    assert_eq!(before.unread_items.len(), 2);
    assert_eq!(before.unread_items[0].row_index, 2);
    assert_eq!(before.unread_items[0].symbol, "BTCUSDT");
    assert_eq!(before.unread_items[0].period, "15");
    assert!(before.unread_items[0].meta.contains("BTC Main"));
    assert!(before.unread_items[0].meta.contains("divMacd"));
    assert_eq!(before.unread_items[1].row_index, 1);

    let mark_read = state.toggle_signal_row_at(before.unread_items[0].row_index as usize);
    let after = state.snapshot();

    assert!(mark_read.is_some());
    assert_eq!(after.unread_count, 1);
    assert_eq!(after.unread_items.len(), 1);
    assert_eq!(after.unread_items[0].row_index, 1);
    assert_eq!(after.unread_items[0].period, "60");
}
