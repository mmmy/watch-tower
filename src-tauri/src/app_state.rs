use crate::config::repository::ConfigRepository;
use crate::polling::alerts_client::ApiSignalsResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

pub const APP_SNAPSHOT_EVENT: &str = "watch-tower://snapshot-updated";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatchGroupConfig {
    pub id: String,
    pub symbol: String,
    pub signal_types: Vec<String>,
    pub periods: Vec<String>,
    pub selected_timeline_period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DashboardPreferences {
    pub layout_preset: String,
    pub density: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WindowPolicyConfig {
    pub dock_side: String,
    pub widget_width: u64,
    pub widget_height: u64,
    pub top_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub polling_interval_seconds: u64,
    pub selected_group_id: String,
    pub groups: Vec<WatchGroupConfig>,
    #[serde(default = "default_dashboard_preferences")]
    pub dashboard: DashboardPreferences,
    #[serde(default = "default_window_policy")]
    pub window_policy: WindowPolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollingHealth {
    pub status: String,
    pub polling_interval_seconds: Option<u64>,
    pub is_stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsInfo {
    pub source: String,
    pub code: Option<String>,
    pub message: String,
    pub last_attempt_at: Option<u64>,
    pub last_successful_sync_at: Option<u64>,
    pub next_retry_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub bootstrap_required: bool,
    pub config: Option<AppConfig>,
    pub raw_response: Option<ApiSignalsResponse>,
    pub health: PollingHealth,
    pub diagnostics: DiagnosticsInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotEventPayload {
    pub snapshot: AppSnapshot,
}

#[derive(Clone)]
pub struct SharedAppState {
    pub repository: ConfigRepository,
    pub http_client: reqwest::Client,
    snapshot: Arc<Mutex<AppSnapshot>>,
    wake_signal: Arc<Notify>,
    poll_lock: Arc<Mutex<()>>,
}

impl SharedAppState {
    pub fn new(repository: ConfigRepository, config: Option<AppConfig>) -> Self {
        Self {
            repository,
            http_client: reqwest::Client::new(),
            snapshot: Arc::new(Mutex::new(AppSnapshot::from_config(config))),
            wake_signal: Arc::new(Notify::new()),
            poll_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn current_snapshot(&self) -> AppSnapshot {
        self.snapshot.lock().await.clone()
    }

    pub async fn current_config(&self) -> Option<AppConfig> {
        self.snapshot.lock().await.config.clone()
    }

    pub async fn replace_snapshot(&self, next_snapshot: AppSnapshot) {
        let mut snapshot = self.snapshot.lock().await;
        *snapshot = next_snapshot;
    }

    pub async fn update_with<F>(&self, update: F) -> AppSnapshot
    where
        F: FnOnce(&mut AppSnapshot),
    {
        let mut snapshot = self.snapshot.lock().await;
        update(&mut snapshot);
        snapshot.clone()
    }

    pub fn wake(&self) {
        self.wake_signal.notify_waiters();
    }

    pub async fn wait_for_wake(&self) {
        self.wake_signal.notified().await;
    }

    pub fn poll_lock(&self) -> Arc<Mutex<()>> {
        self.poll_lock.clone()
    }
}

impl AppSnapshot {
    pub fn from_config(config: Option<AppConfig>) -> Self {
        match config {
            Some(config) => Self {
                bootstrap_required: false,
                config: Some(config.clone()),
                raw_response: None,
                health: PollingHealth {
                    status: "idle".into(),
                    polling_interval_seconds: Some(config.polling_interval_seconds),
                    is_stale: false,
                },
                diagnostics: DiagnosticsInfo {
                    source: "system".into(),
                    code: Some("CONFIG_READY".into()),
                    message: "Config loaded. Waiting for the first poll cycle.".into(),
                    last_attempt_at: None,
                    last_successful_sync_at: None,
                    next_retry_at: None,
                },
            },
            None => Self {
                bootstrap_required: true,
                config: None,
                raw_response: None,
                health: PollingHealth {
                    status: "bootstrapRequired".into(),
                    polling_interval_seconds: None,
                    is_stale: false,
                },
                diagnostics: DiagnosticsInfo {
                    source: "config".into(),
                    code: Some("BOOTSTRAP_REQUIRED".into()),
                    message:
                        "Save API base URL, API key, symbol, and signal types to start polling."
                            .into(),
                    last_attempt_at: None,
                    last_successful_sync_at: None,
                    next_retry_at: None,
                },
            },
        }
    }
}

fn default_dashboard_preferences() -> DashboardPreferences {
    DashboardPreferences {
        layout_preset: "table".into(),
        density: "comfortable".into(),
    }
}

fn default_window_policy() -> WindowPolicyConfig {
    WindowPolicyConfig {
        dock_side: "right".into(),
        widget_width: 280,
        widget_height: 720,
        top_offset: 96,
    }
}
