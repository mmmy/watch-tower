pub const MIN_POLLING_INTERVAL_SECONDS: u64 = 10;
pub const BACKOFF_MILLIS: u64 = 3 * 60 * 1000;

pub fn clamp_polling_interval(seconds: u64) -> u64 {
    seconds.max(MIN_POLLING_INTERVAL_SECONDS)
}

pub fn compute_backoff_until(now_ms: u64) -> u64 {
    now_ms + BACKOFF_MILLIS
}

pub fn sleep_duration_ms(
    polling_interval_seconds: u64,
    next_retry_at: Option<u64>,
    now_ms: u64,
) -> u64 {
    if let Some(retry_at) = next_retry_at {
        return retry_at.saturating_sub(now_ms);
    }

    clamp_polling_interval(polling_interval_seconds) * 1000
}

#[cfg(test)]
mod tests {
    use super::{clamp_polling_interval, compute_backoff_until, sleep_duration_ms, BACKOFF_MILLIS};

    #[test]
    fn clamps_polling_interval_to_minimum() {
        assert_eq!(clamp_polling_interval(1), 10);
    }

    #[test]
    fn computes_fixed_backoff_window() {
        assert_eq!(compute_backoff_until(1_000), 1_000 + BACKOFF_MILLIS);
    }

    #[test]
    fn prefers_backoff_sleep_when_retry_is_present() {
        assert_eq!(sleep_duration_ms(60, Some(10_000), 5_000), 5_000);
    }
}
