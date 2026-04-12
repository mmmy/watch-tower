use crate::app_state::{AlertPayload, AppConfig, WatchGroupConfig};
use crate::polling::alerts_client::ApiSignalsResponse;
use std::collections::HashSet;

pub fn build_unread_alerts(config: &AppConfig, response: &ApiSignalsResponse) -> Vec<AlertPayload> {
    let mut alerts = vec![];

    for record in &response.data {
        let Some(signals) = &record.signals else {
            continue;
        };

        for (signal_type, signal) in signals {
            if signal.read {
                continue;
            }

            let Some(group_id) = resolve_group_id(config, &record.symbol, &record.period, signal_type)
            else {
                continue;
            };

            alerts.push(AlertPayload {
                id: alert_id(&record.symbol, &record.period, signal_type),
                group_id,
                symbol: record.symbol.clone(),
                period: record.period.clone(),
                signal_type: signal_type.clone(),
                side: signal.sd,
                signal_at: signal.t,
            });
        }
    }

    alerts.sort_by(|left, right| {
        right
            .signal_at
            .cmp(&left.signal_at)
            .then_with(|| left.symbol.cmp(&right.symbol))
            .then_with(|| left.period.cmp(&right.period))
            .then_with(|| left.signal_type.cmp(&right.signal_type))
    });

    alerts
}

pub fn compute_new_unread_alerts(
    config: &AppConfig,
    previous_response: Option<&ApiSignalsResponse>,
    current_response: &ApiSignalsResponse,
    suppressed_alert_ids: &HashSet<String>,
) -> Vec<AlertPayload> {
    let previous_unread_ids = previous_response
        .map(|response| {
            build_unread_alerts(config, response)
                .into_iter()
                .map(|alert| alert.id)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    build_unread_alerts(config, current_response)
        .into_iter()
        .filter(|alert| {
            !previous_unread_ids.contains(&alert.id) && !suppressed_alert_ids.contains(&alert.id)
        })
        .collect()
}

pub fn alert_id(symbol: &str, period: &str, signal_type: &str) -> String {
    format!(
        "{}:{}:{}",
        symbol.trim().to_uppercase(),
        period.trim(),
        signal_type.trim()
    )
}

fn resolve_group_id(
    config: &AppConfig,
    symbol: &str,
    period: &str,
    signal_type: &str,
) -> Option<String> {
    if let Some(selected_group) = config
        .groups
        .iter()
        .find(|group| group.id == config.selected_group_id)
        .filter(|group| group_matches(group, symbol, period, signal_type))
    {
        return Some(selected_group.id.clone());
    }

    config
        .groups
        .iter()
        .find(|group| group_matches(group, symbol, period, signal_type))
        .map(|group| group.id.clone())
}

fn group_matches(group: &WatchGroupConfig, symbol: &str, period: &str, signal_type: &str) -> bool {
    group.symbol.eq_ignore_ascii_case(symbol)
        && group.periods.iter().any(|candidate| candidate == period)
        && group.signal_types.iter().any(|candidate| candidate == signal_type)
}

#[cfg(test)]
mod tests {
    use super::{alert_id, build_unread_alerts, compute_new_unread_alerts};
    use crate::app_state::{AppConfig, DashboardPreferences, WatchGroupConfig, WindowPolicyConfig};
    use crate::polling::alerts_client::{ApiSignalEntry, ApiSignalRecord, ApiSignalsResponse};
    use std::collections::{HashMap, HashSet};

    fn test_config() -> AppConfig {
        AppConfig {
            api_base_url: "https://example.com".into(),
            api_key: "secret".into(),
            polling_interval_seconds: 60,
            notifications_enabled: true,
            selected_group_id: "btc-core".into(),
            groups: vec![
                WatchGroupConfig {
                    id: "btc-core".into(),
                    symbol: "BTCUSDT".into(),
                    signal_types: vec!["vegas".into(), "divMacd".into()],
                    periods: vec!["240".into(), "60".into()],
                    selected_timeline_period: "60".into(),
                },
                WatchGroupConfig {
                    id: "eth-swing".into(),
                    symbol: "ETHUSDT".into(),
                    signal_types: vec!["vegas".into()],
                    periods: vec!["240".into()],
                    selected_timeline_period: "240".into(),
                },
            ],
            dashboard: DashboardPreferences {
                layout_preset: "table".into(),
                density: "comfortable".into(),
            },
            window_policy: WindowPolicyConfig {
                dock_side: "right".into(),
                widget_width: 280,
                widget_height: 720,
                top_offset: 96,
            },
        }
    }

    fn response(records: Vec<ApiSignalRecord>) -> ApiSignalsResponse {
        ApiSignalsResponse {
            total: records.len() as u64,
            page: 1,
            page_size: 100,
            data: records,
        }
    }

    fn record(
        symbol: &str,
        period: &str,
        signals: impl IntoIterator<Item = (&'static str, ApiSignalEntry)>,
    ) -> ApiSignalRecord {
        ApiSignalRecord {
            symbol: symbol.into(),
            period: period.into(),
            t: 1_000,
            signals: Some(HashMap::from_iter(
                signals.into_iter().map(|(key, value)| (key.to_string(), value)),
            )),
        }
    }

    #[test]
    fn computes_new_unread_alerts_only_for_new_keys() {
        let config = test_config();
        let previous = response(vec![record(
            "BTCUSDT",
            "60",
            [(
                "vegas",
                ApiSignalEntry {
                    sd: 1,
                    t: 100,
                    read: false,
                },
            )],
        )]);
        let current = response(vec![record(
            "BTCUSDT",
            "60",
            [
                (
                    "vegas",
                    ApiSignalEntry {
                        sd: 1,
                        t: 100,
                        read: false,
                    },
                ),
                (
                    "divMacd",
                    ApiSignalEntry {
                        sd: -1,
                        t: 200,
                        read: false,
                    },
                ),
            ],
        )]);

        let alerts = compute_new_unread_alerts(&config, Some(&previous), &current, &HashSet::new());

        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, alert_id("BTCUSDT", "60", "divMacd"));
    }

    #[test]
    fn suppresses_pending_read_alert_ids_from_realerting() {
        let config = test_config();
        let current = response(vec![record(
            "BTCUSDT",
            "60",
            [(
                "vegas",
                ApiSignalEntry {
                    sd: 1,
                    t: 100,
                    read: false,
                },
            )],
        )]);
        let mut suppressed = HashSet::new();
        suppressed.insert(alert_id("BTCUSDT", "60", "vegas"));

        let alerts = compute_new_unread_alerts(&config, None, &current, &suppressed);

        assert!(alerts.is_empty());
    }

    #[test]
    fn prefers_selected_group_when_multiple_groups_could_match() {
        let mut config = test_config();
        config.groups.push(WatchGroupConfig {
            id: "btc-duplicate".into(),
            symbol: "BTCUSDT".into(),
            signal_types: vec!["vegas".into()],
            periods: vec!["60".into()],
            selected_timeline_period: "60".into(),
        });
        let current = response(vec![record(
            "BTCUSDT",
            "60",
            [(
                "vegas",
                ApiSignalEntry {
                    sd: 1,
                    t: 100,
                    read: false,
                },
            )],
        )]);

        let alerts = build_unread_alerts(&config, &current);

        assert_eq!(alerts[0].group_id, "btc-core");
    }
}
