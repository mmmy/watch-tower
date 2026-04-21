use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use signal_desk_native::main_window_state::{
    load_main_window_state, save_main_window_state, MainWindowState,
};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("signal-desk-main-window-{name}-{nonce}"))
}

#[test]
fn main_window_state_round_trips_through_disk() {
    let temp_dir = unique_temp_dir("round-trip");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let path = temp_dir.join("main-window-state.json");
    let state = MainWindowState {
        width: 888.0,
        height: 666.0,
        maximized: true,
    };

    save_main_window_state(&path, &state).expect("save main window state");
    let loaded = load_main_window_state(&path).expect("load main window state");

    assert_eq!(loaded, state);

    fs::remove_dir_all(&temp_dir).expect("cleanup temp dir");
}

#[test]
fn invalid_window_dimensions_are_sanitized_when_loading() {
    let temp_dir = unique_temp_dir("sanitize");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let path = temp_dir.join("main-window-state.json");
    fs::write(
        &path,
        r#"{"width":0,"height":-12,"maximized":false}"#,
    )
    .expect("write main window state");

    let loaded = load_main_window_state(&path).expect("load sanitized main window state");

    assert_eq!(loaded.width, 40.0);
    assert_eq!(loaded.height, 100.0);
    assert!(!loaded.maximized);

    fs::remove_dir_all(&temp_dir).expect("cleanup temp dir");
}
