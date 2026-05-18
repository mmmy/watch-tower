use std::collections::HashMap;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Result<Self, String> {
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .map_err(|err| err.to_string())?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            api_key: api_key.into(),
        })
    }

    pub fn post_json<TReq: Serialize, TRes: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &TReq,
    ) -> Result<TRes, String> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .json(body)
            .send()
            .map_err(|err| err.to_string())?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(format!("request failed: {} {}", status, body));
        }

        response.json::<TRes>().map_err(|err| err.to_string())
    }

    pub fn fetch_signal_list(&self, query: &SignalListQuery) -> Result<SignalListResponse, String> {
        self.post_json("/api/open/watch-list/symbol-signals", query)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalListQuery {
    pub symbols: String,
    pub periods: String,
    #[serde(rename = "signalTypes")]
    pub signal_types: String,
    pub page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct SignalListResponse {
    #[serde(default)]
    pub data: Vec<SignalListItem>,
}

#[derive(Debug, Deserialize)]
pub struct SignalListItem {
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_stringish")]
    pub period: String,
    #[serde(default)]
    pub signals: HashMap<String, RemoteSignalDetail>,
}

#[derive(Debug, Deserialize)]
pub struct RemoteSignalDetail {
    #[serde(default)]
    pub sd: i8,
    #[serde(default)]
    pub t: i64,
    #[serde(default)]
    pub read: bool,
}

pub(crate) fn deserialize_stringish<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(text) => Ok(text),
        serde_json::Value::Number(number) => Ok(number.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        other => Ok(other.to_string().trim_matches('"').to_string()),
    }
}
