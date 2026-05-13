use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use signal_desk_native::runtime::{parse_timeline_bars_input, AppConfig, RuntimeModel, WatchGroup};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("signal-desk-runtime-{name}-{nonce}"))
}

#[test]
fn runtime_model_updates_snapshot_for_shell_and_ui_flags() {
    let config = AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    };
    let mut runtime = RuntimeModel::new(config);

    let snapshot = runtime.set_always_on_top(false);
    assert!(!snapshot.always_on_top);
    assert!(!snapshot.config.ui.always_on_top);
    assert_eq!(snapshot.last_connection_ok, None);

    let snapshot = runtime.set_edge_mode(true);
    assert!(snapshot.edge_mode);
    assert!(snapshot.config.ui.edge_mode);

    let snapshot = runtime.set_edge_width(999.0);
    assert_eq!(snapshot.config.ui.edge_width, 480.0);

    let snapshot = runtime.set_notifications(false);
    assert!(!snapshot.config.ui.notifications);

    let snapshot = runtime.set_sound(false);
    assert!(!snapshot.config.ui.sound);
    assert_eq!(snapshot.last_connection_ok, None);
}

#[test]
fn group_timeline_bars_accepts_values_up_to_500() {
    let config = AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    };
    let mut runtime = RuntimeModel::new(config);

    let snapshot = runtime.set_group_timeline_bars("group-1", 500);
    assert_eq!(snapshot.config.groups[0].timeline_bars, 500);

    let snapshot = runtime.set_group_timeline_bars("group-1", 501);
    assert_eq!(snapshot.config.groups[0].timeline_bars, 500);
}

#[test]
fn timeline_bars_input_is_trimmed_clamped_and_ignores_invalid_text() {
    assert_eq!(parse_timeline_bars_input(" 375 "), Some(375));
    assert_eq!(parse_timeline_bars_input("7"), Some(10));
    assert_eq!(parse_timeline_bars_input("999"), Some(500));
    assert_eq!(parse_timeline_bars_input("abc"), None);
    assert_eq!(parse_timeline_bars_input(""), None);
}

#[test]
fn save_config_to_path_persists_current_runtime_settings() {
    let temp_dir = unique_temp_dir("save-config");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let config_path = temp_dir.join("config.yaml");

    let config = AppConfig {
        groups: vec![WatchGroup {
            id: "group-1".into(),
            name: "BTC Main".into(),
            symbol: "BTCUSDT".into(),
            periods: vec!["60".into()],
            signal_types: vec!["divMacd".into()],
            enabled: true,
            ..WatchGroup::default()
        }],
        ..Default::default()
    };
    let mut runtime = RuntimeModel::new(config);
    runtime.set_always_on_top(false);
    runtime.set_edge_mode(true);
    runtime.set_edge_width(222.0);
    runtime.set_notifications(false);
    runtime.set_sound(false);

    let snapshot = runtime
        .save_config_to_path(&config_path)
        .expect("save config to explicit path");

    assert!(!snapshot.always_on_top);
    assert!(snapshot.edge_mode);
    assert_eq!(snapshot.config.ui.edge_width, 222.0);
    assert!(!snapshot.config.ui.notifications);
    assert!(!snapshot.config.ui.sound);

    let content = fs::read_to_string(&config_path).expect("read saved config");
    let persisted: AppConfig = serde_yaml::from_str(&content).expect("parse saved config");

    assert!(!persisted.ui.always_on_top);
    assert!(persisted.ui.edge_mode);
    assert_eq!(persisted.ui.edge_width, 222.0);
    assert!(!persisted.ui.notifications);
    assert!(!persisted.ui.sound);
    assert_eq!(persisted.groups.len(), 1);

    fs::remove_dir_all(&temp_dir).expect("cleanup temp dir");
}
