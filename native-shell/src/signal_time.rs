use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElapsedCandles {
    pub period_ms: Option<i64>,
    pub elapsed_ms: Option<i64>,
    pub elapsed_candles: Option<i64>,
}

pub fn period_to_ms(period: &str) -> Option<i64> {
    let normalized = period.trim().to_ascii_uppercase();
    match normalized.as_str() {
        "W" => Some(7 * 24 * 60 * 60 * 1000),
        "D" => Some(24 * 60 * 60 * 1000),
        _ if normalized.ends_with('D') => normalized
            .trim_end_matches('D')
            .parse::<i64>()
            .ok()
            .map(|days| days * 24 * 60 * 60 * 1000),
        _ if normalized.ends_with('S') => normalized
            .trim_end_matches('S')
            .parse::<i64>()
            .ok()
            .map(|seconds| seconds * 1000),
        _ => normalized
            .parse::<i64>()
            .ok()
            .map(|minutes| minutes * 60 * 1000),
    }
    .filter(|period_ms| *period_ms > 0)
}

pub fn elapsed_candles(period: &str, trigger_time: i64, now_ms: i64) -> ElapsedCandles {
    if trigger_time <= 0 {
        return null_elapsed_candles();
    }

    let Some(period_ms) = period_to_ms(period) else {
        return null_elapsed_candles();
    };

    let elapsed_ms = now_ms.saturating_sub(trigger_time).max(0);
    ElapsedCandles {
        period_ms: Some(period_ms),
        elapsed_ms: Some(elapsed_ms),
        elapsed_candles: Some(elapsed_ms / period_ms),
    }
}

pub fn elapsed_label(elapsed_ms: i64) -> Option<String> {
    if elapsed_ms <= 0 {
        return None;
    }

    let seconds = elapsed_ms / 1000;
    if seconds < 60 {
        return Some(format!("{seconds}秒前"));
    }

    let minutes = seconds / 60;
    if minutes < 60 {
        return Some(format!("{minutes}分钟前"));
    }

    let hours = minutes / 60;
    if hours < 24 {
        return Some(format!("{hours}小时前"));
    }

    let days = hours / 24;
    Some(format!("{days}天前"))
}

fn null_elapsed_candles() -> ElapsedCandles {
    ElapsedCandles {
        period_ms: None,
        elapsed_ms: None,
        elapsed_candles: None,
    }
}
