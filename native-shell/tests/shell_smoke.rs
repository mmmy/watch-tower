use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use signal_desk_native::app_state::AppState;
use signal_desk_native::config::load_config_from_candidates;
use signal_desk_native::runtime::{runtime_snapshot_from_config, AppConfig, WatchGroup};
use signal_desk_native::tray::{command_from_menu_id, TrayCommand};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("signal-desk-native-{name}-{nonce}"))
}

#[test]
fn builds_app_state_from_seeded_runtime_snapshot() {
    let config = AppConfig {
        groups: vec![
            WatchGroup {
                id: "enabled".into(),
                name: "BTC".into(),
                symbol: "BTCUSDT".into(),
                periods: vec!["60".into(), "15".into()],
                signal_types: vec!["divMacd".into(), "divRsi".into()],
                enabled: true,
            },
            WatchGroup {
                id: "disabled".into(),
                name: "ETH".into(),
                symbol: "ETHUSDT".into(),
                periods: vec!["5".into()],
                signal_types: vec!["divMacd".into()],
                enabled: false,
            },
        ],
        ..Default::default()
    };

    let runtime_snapshot = runtime_snapshot_from_config(config);
    let app_snapshot = AppState::new(runtime_snapshot.clone()).snapshot();

    assert_eq!(runtime_snapshot.signals.len(), 4);
    assert_eq!(runtime_snapshot.unread_count, 0);
    assert_eq!(app_snapshot.unread_count, 0);
    assert!(app_snapshot.main_visible);
    assert!(app_snapshot.widget_visible);
    assert_eq!(app_snapshot.signal_rows.len(), 6);
}

#[test]
fn missing_config_candidates_fall_back_to_defaults() {
    let missing = PathBuf::from("Z:\\definitely-missing\\config.yaml");
    let config = load_config_from_candidates(&[missing]);

    assert_eq!(config.api.base_url, "http://127.0.0.1:8787");
    assert_eq!(config.groups.len(), 1);
    assert_eq!(config.groups[0].symbol, "BTCUSDT");
}

#[test]
fn malformed_config_candidates_fall_back_to_defaults() {
    let temp_dir = unique_temp_dir("bad-config");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let bad_config = temp_dir.join("config.yaml");
    fs::write(&bad_config, "groups: [this is not valid yaml").expect("write bad config");

    let config = load_config_from_candidates(&[bad_config]);

    assert_eq!(config.api.base_url, "http://127.0.0.1:8787");
    assert_eq!(config.groups.len(), 1);

    fs::remove_dir_all(&temp_dir).expect("cleanup temp dir");
}

#[test]
fn tray_menu_ids_map_to_commands() {
    assert!(matches!(
        command_from_menu_id("toggle-main"),
        Some(TrayCommand::ToggleMainWindow)
    ));
    assert!(matches!(
        command_from_menu_id("toggle-widget"),
        Some(TrayCommand::ToggleWidgetWindow)
    ));
    assert!(matches!(
        command_from_menu_id("toggle-pin"),
        Some(TrayCommand::ToggleAlwaysOnTop)
    ));
    assert!(matches!(
        command_from_menu_id("refresh"),
        Some(TrayCommand::RefreshSignals)
    ));
    assert!(matches!(
        command_from_menu_id("quit"),
        Some(TrayCommand::Quit)
    ));
    assert!(command_from_menu_id("unknown").is_none());
}
