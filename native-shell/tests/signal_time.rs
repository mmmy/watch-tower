use signal_desk_native::signal_time::{elapsed_candles, elapsed_label, period_to_ms};

#[test]
fn period_to_ms_supports_watch_tower_period_formats() {
    assert_eq!(period_to_ms("W"), Some(7 * 24 * 60 * 60 * 1000));
    assert_eq!(period_to_ms("D"), Some(24 * 60 * 60 * 1000));
    assert_eq!(period_to_ms("10D"), Some(10 * 24 * 60 * 60 * 1000));
    assert_eq!(period_to_ms("45S"), Some(45 * 1000));
    assert_eq!(period_to_ms("15"), Some(15 * 60 * 1000));
}

#[test]
fn elapsed_candles_counts_complete_periods_since_trigger() {
    let trigger_time = 1_000_000;
    let now_ms = trigger_time + (2 * 15 * 60 * 1000) + 59_000;

    let elapsed = elapsed_candles("15", trigger_time, now_ms);

    assert_eq!(elapsed.period_ms, Some(15 * 60 * 1000));
    assert_eq!(elapsed.elapsed_ms, Some((2 * 15 * 60 * 1000) + 59_000));
    assert_eq!(elapsed.elapsed_candles, Some(2));
}

#[test]
fn elapsed_candles_returns_null_fields_for_missing_trigger_or_invalid_period() {
    let missing_trigger = elapsed_candles("15", 0, 2_000_000);
    assert_eq!(missing_trigger.period_ms, None);
    assert_eq!(missing_trigger.elapsed_ms, None);
    assert_eq!(missing_trigger.elapsed_candles, None);

    let invalid_period = elapsed_candles("bad", 1_000_000, 2_000_000);
    assert_eq!(invalid_period.period_ms, None);
    assert_eq!(invalid_period.elapsed_ms, None);
    assert_eq!(invalid_period.elapsed_candles, None);
}

#[test]
fn elapsed_label_summarizes_age_for_model_readability() {
    assert_eq!(elapsed_label(0), None);
    assert_eq!(elapsed_label(45_000), Some("45秒前".to_string()));
    assert_eq!(elapsed_label(3 * 60 * 1000), Some("3分钟前".to_string()));
    assert_eq!(
        elapsed_label(2 * 60 * 60 * 1000),
        Some("2小时前".to_string())
    );
    assert_eq!(
        elapsed_label(3 * 24 * 60 * 60 * 1000),
        Some("3天前".to_string())
    );
}
