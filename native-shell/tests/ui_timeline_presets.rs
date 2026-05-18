use std::fs;
use std::path::Path;

fn main_window_source() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/main-window.slint");
    fs::read_to_string(path).expect("read main window Slint source")
}

#[test]
fn timeline_window_shortcuts_include_common_large_presets() {
    let source = main_window_source();

    for preset in [200, 300, 400] {
        assert!(
            source.contains(&format!("root.set-signal-timeline-bars(index, {preset});")),
            "missing {preset}-bar timeline shortcut"
        );
    }
}
