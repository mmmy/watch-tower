use crate::app_state::AppConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const SIGNALS_ENDPOINT: &str = "/api/open/watch-list/symbol-signals";
const READ_STATUS_ENDPOINT: &str = "/api/open/watch-list/symbol-alert/read-status";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSignalEntry {
    pub sd: i8,
    pub t: u64,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSignalRecord {
    pub symbol: String,
    pub period: String,
    pub t: u64,
    pub signals: Option<std::collections::HashMap<String, ApiSignalEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSignalsResponse {
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub data: Vec<ApiSignalRecord>,
}

#[derive(Debug)]
pub enum FetchError {
    Unauthorized,
    Backoff(u16),
    Http(u16),
    Network(String),
    Deserialize(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignalRequestBody {
    symbols: String,
    periods: String,
    signal_types: String,
    page: u32,
    page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadStatusInput {
    pub symbol: String,
    pub period: String,
    pub signal_type: String,
    pub read: bool,
}

pub async fn fetch_signals(
    client: &reqwest::Client,
    config: &AppConfig,
) -> Result<ApiSignalsResponse, FetchError> {
    let body = build_signal_request_body(config);
    let response = client
        .post(format!("{}{}", config.api_base_url, SIGNALS_ENDPOINT))
        .header("x-api-key", &config.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| FetchError::Network(error.to_string()))?;

    let status = response.status();

    if status.is_success() {
        return response
            .json::<ApiSignalsResponse>()
            .await
            .map_err(|error| FetchError::Deserialize(error.to_string()));
    }

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(FetchError::Unauthorized);
    }

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
        return Err(FetchError::Backoff(status.as_u16()));
    }

    Err(FetchError::Http(status.as_u16()))
}

pub async fn set_read_status(
    client: &reqwest::Client,
    config: &AppConfig,
    input: &ReadStatusInput,
) -> Result<bool, FetchError> {
    let response = client
        .post(format!("{}{}", config.api_base_url, READ_STATUS_ENDPOINT))
        .header("x-api-key", &config.api_key)
        .json(input)
        .send()
        .await
        .map_err(|error| FetchError::Network(error.to_string()))?;

    let status = response.status();

    if status.is_success() {
        return response
            .json::<bool>()
            .await
            .map_err(|error| FetchError::Deserialize(error.to_string()));
    }

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(FetchError::Unauthorized);
    }

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
        return Err(FetchError::Backoff(status.as_u16()));
    }

    Err(FetchError::Http(status.as_u16()))
}

pub fn build_signal_request_body(config: &AppConfig) -> serde_json::Value {
    let mut symbols = BTreeSet::new();
    let mut periods = BTreeSet::new();
    let mut signal_types = BTreeSet::new();

    for group in &config.groups {
        symbols.insert(group.symbol.clone());

        for period in &group.periods {
            periods.insert(period.clone());
        }

        for signal_type in &group.signal_types {
            signal_types.insert(signal_type.clone());
        }
    }

    serde_json::to_value(SignalRequestBody {
        symbols: symbols.into_iter().collect::<Vec<_>>().join(","),
        periods: periods.into_iter().collect::<Vec<_>>().join(","),
        signal_types: signal_types.into_iter().collect::<Vec<_>>().join(","),
        page: 1,
        page_size: 100,
    })
    .expect("request body")
}

#[cfg(test)]
mod tests {
    use super::{build_signal_request_body, ReadStatusInput};
    use crate::app_state::{AppConfig, DashboardPreferences, WatchGroupConfig, WindowPolicyConfig};

    #[test]
    fn builds_union_request_body_across_groups() {
        let config = AppConfig {
            api_base_url: "https://example.com".into(),
            api_key: "secret".into(),
            polling_interval_seconds: 60,
            notifications_enabled: true,
            selected_group_id: "btc".into(),
            groups: vec![
                WatchGroupConfig {
                    id: "btc".into(),
                    symbol: "BTCUSDT".into(),
                    signal_types: vec!["vegas".into()],
                    periods: vec!["60".into(), "240".into()],
                    selected_timeline_period: "60".into(),
                },
                WatchGroupConfig {
                    id: "eth".into(),
                    symbol: "ETHUSDT".into(),
                    signal_types: vec!["divMacd".into()],
                    periods: vec!["15".into()],
                    selected_timeline_period: "15".into(),
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
        };

        let payload = build_signal_request_body(&config);

        assert_eq!(payload["symbols"], "BTCUSDT,ETHUSDT");
        assert_eq!(payload["signalTypes"], "divMacd,vegas");
    }

    #[test]
    fn serializes_read_status_input_with_the_server_contract_keys() {
        let payload = serde_json::to_value(ReadStatusInput {
            symbol: "BTCUSDT".into(),
            period: "60".into(),
            signal_type: "vegas".into(),
            read: true,
        })
        .expect("serialize");

        assert_eq!(payload["symbol"], "BTCUSDT");
        assert_eq!(payload["period"], "60");
        assert_eq!(payload["signalType"], "vegas");
        assert_eq!(payload["read"], true);
    }
}
