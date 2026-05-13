#[test]
fn main_signal_list_can_expand_with_window_height() {
    let source = include_str!("../ui/main-window.slint");
    let main_window = source
        .split("export component MainWindow")
        .nth(1)
        .expect("MainWindow component exists")
        .split("export component WidgetWindow")
        .next()
        .expect("MainWindow appears before WidgetWindow");
    let signal_list = main_window
        .split("ListView {")
        .nth(2)
        .expect("signal list is the second MainWindow ListView")
        .split("for signal-row")
        .next()
        .expect("signal list contains signal rows");

    assert!(
        !signal_list
            .lines()
            .any(|line| line.trim() == "height: 610px;"),
        "the main signal list must not use a fixed height because Slint turns it into a maximum window height"
    );
    assert!(
        signal_list.contains("min-height: 610px;"),
        "the main signal list should keep the previous useful default as a minimum height"
    );
    assert!(
        signal_list.contains("vertical-stretch: 1;"),
        "the main signal list should absorb extra window height"
    );
}
