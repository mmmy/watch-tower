use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use signal_desk_native::runtime::{AppConfig, RuntimeModel, WatchGroup};

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

    let snapshot = runtime.set_edge_mode(true);
    assert!(snapshot.edge_mode);
    assert!(snapshot.config.ui.edge_mode);

    let snapshot = runtime.set_edge_width(999.0);
    assert_eq!(snapshot.config.ui.edge_width, 480.0);

    let snapshot = runtime.set_notifications(false);
    assert!(!snapshot.config.ui.notifications);

    let snapshot = runtime.set_sound(false);
    assert!(!snapshot.config.ui.sound);
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
