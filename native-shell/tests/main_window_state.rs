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
fn ui_snapshot_marks_stale_connection_as_timed_out() {
    let mut runtime_snapshot = runtime_snapshot_from_config(AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    });
    runtime_snapshot.last_updated_at = 0;
    runtime_snapshot.config.poll.interval_secs = 60;

    let snapshot = AppState::new(runtime_snapshot).snapshot();

    assert_eq!(snapshot.connection_label, "连接超时");
    assert_eq!(snapshot.connection_tone, "offline");
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
