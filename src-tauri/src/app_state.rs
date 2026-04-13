use crate::config::repository::ConfigRepository;
use crate::polling::alerts_client::ApiSignalsResponse;
use crate::platform;
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
    #[serde(default = "default_notifications_enabled")]
    pub notifications_enabled: bool,
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
pub struct RuntimeInfo {
    pub polling_paused: bool,
    pub last_active_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AlertPayload {
    pub id: String,
    pub group_id: String,
    pub symbol: String,
    pub period: String,
    pub signal_type: String,
    pub side: i8,
    pub signal_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingReadState {
    pub alert: AlertPayload,
    pub requested_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DashboardFocusIntent {
    pub alert: AlertPayload,
    pub requested_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AlertPopupStream {
    pub symbol: String,
    pub alerts: Vec<AlertPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AlertRuntime {
    pub visible_popup_streams: Vec<AlertPopupStream>,
    pub queued_popup_streams: Vec<AlertPopupStream>,
    pub pending_read: Option<PendingReadState>,
    pub dashboard_focus_intent: Option<DashboardFocusIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WidgetBehaviorRuntime {
    pub mode: String,
    pub placement: String,
    pub click_through_enabled: bool,
    pub click_through_supported: bool,
    pub fallback_reason: Option<String>,
    pub wake_source: Option<String>,
    pub interaction_session_id: u64,
    pub idle_deadline_at: Option<u64>,
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
    pub runtime: RuntimeInfo,
    pub alert_runtime: AlertRuntime,
    pub widget_runtime: WidgetBehaviorRuntime,
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
                runtime: RuntimeInfo {
                    polling_paused: false,
                    last_active_status: None,
                },
                alert_runtime: AlertRuntime::default(),
                widget_runtime: WidgetBehaviorRuntime::default(),
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
                runtime: RuntimeInfo {
                    polling_paused: false,
                    last_active_status: None,
                },
                alert_runtime: AlertRuntime::default(),
                widget_runtime: WidgetBehaviorRuntime::default(),
            },
        }
    }
}

impl Default for WidgetBehaviorRuntime {
    fn default() -> Self {
        let (click_through_supported, fallback_reason) = platform::default_click_through_support();

        Self {
            mode: "passive".into(),
            placement: "hidden".into(),
            click_through_enabled: false,
            click_through_supported,
            fallback_reason,
            wake_source: None,
            interaction_session_id: 0,
            idle_deadline_at: None,
        }
    }
}

impl AlertRuntime {
    pub const MAX_VISIBLE_POPUP_STREAMS: usize = 3;

    pub fn contains_alert_id(&self, alert_id: &str) -> bool {
        self.visible_popup_streams
            .iter()
            .chain(self.queued_popup_streams.iter())
            .flat_map(|stream| stream.alerts.iter())
            .any(|alert| alert.id == alert_id)
            || self
            .pending_read
            .as_ref()
            .is_some_and(|pending| pending.alert.id == alert_id)
    }

    pub fn enqueue_new_alerts(&mut self, alerts: Vec<AlertPayload>) {
        for alert in alerts {
            if self.contains_alert_id(&alert.id) {
                continue;
            }

            self.upsert_alert(alert);
        }

        self.repartition_streams();
    }

    pub fn suppressed_alert_ids(&self) -> Vec<String> {
        self.pending_read
            .iter()
            .map(|pending| pending.alert.id.clone())
            .collect()
    }

    pub fn mark_pending_read(&mut self, alert: AlertPayload, requested_at: u64) {
        self.pending_read = Some(PendingReadState {
            alert,
            requested_at,
        });
    }

    pub fn resolve_pending_read(&mut self, succeeded: bool) {
        let Some(pending_read) = self.pending_read.take() else {
            return;
        };

        if !succeeded {
            return;
        }

        self.remove_alert(&pending_read.alert.id);
    }

    pub fn remove_alert(&mut self, alert_id: &str) -> bool {
        let mut removed = false;

        self.visible_popup_streams.retain_mut(|stream| {
            let original_len = stream.alerts.len();
            stream.alerts.retain(|alert| alert.id != alert_id);
            removed |= original_len != stream.alerts.len();
            !stream.alerts.is_empty()
        });

        self.queued_popup_streams.retain_mut(|stream| {
            let original_len = stream.alerts.len();
            stream.alerts.retain(|alert| alert.id != alert_id);
            removed |= original_len != stream.alerts.len();
            !stream.alerts.is_empty()
        });

        if removed {
            self.repartition_streams();
        }

        removed
    }
    pub fn set_dashboard_focus_intent(&mut self, alert: AlertPayload, requested_at: u64) {
        self.dashboard_focus_intent = Some(DashboardFocusIntent {
            alert,
            requested_at,
        });
    }

    pub fn clear_dashboard_focus_intent(&mut self) {
        self.dashboard_focus_intent = None;
    }

    fn upsert_alert(&mut self, alert: AlertPayload) {
        if let Some(stream) = self
            .visible_popup_streams
            .iter_mut()
            .find(|stream| stream.symbol.eq_ignore_ascii_case(&alert.symbol))
        {
            stream.alerts.push(alert);
            sort_stream_alerts(stream);
            return;
        }

        if let Some(stream) = self
            .queued_popup_streams
            .iter_mut()
            .find(|stream| stream.symbol.eq_ignore_ascii_case(&alert.symbol))
        {
            stream.alerts.push(alert);
            sort_stream_alerts(stream);
            return;
        }

        let mut stream = AlertPopupStream {
            symbol: alert.symbol.clone(),
            alerts: vec![alert],
        };
        sort_stream_alerts(&mut stream);

        if self.visible_popup_streams.len() < Self::MAX_VISIBLE_POPUP_STREAMS {
            self.visible_popup_streams.push(stream);
        } else {
            self.queued_popup_streams.push(stream);
        }
    }

    fn repartition_streams(&mut self) {
        let mut streams = self
            .visible_popup_streams
            .drain(..)
            .chain(self.queued_popup_streams.drain(..))
            .collect::<Vec<_>>();

        for stream in &mut streams {
            sort_stream_alerts(stream);
        }

        streams.sort_by(compare_stream_priority);

        let split_index = streams.len().min(Self::MAX_VISIBLE_POPUP_STREAMS);
        self.visible_popup_streams = streams.drain(..split_index).collect();
        self.queued_popup_streams = streams;
    }
}

fn sort_stream_alerts(stream: &mut AlertPopupStream) {
    stream.alerts.sort_by(|left, right| {
        right
            .signal_at
            .cmp(&left.signal_at)
            .then_with(|| left.period.cmp(&right.period))
            .then_with(|| left.signal_type.cmp(&right.signal_type))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn compare_stream_priority(left: &AlertPopupStream, right: &AlertPopupStream) -> std::cmp::Ordering {
    latest_signal_at(right)
        .cmp(&latest_signal_at(left))
        .then_with(|| left.symbol.cmp(&right.symbol))
}

fn latest_signal_at(stream: &AlertPopupStream) -> u64 {
    stream
        .alerts
        .first()
        .map(|alert| alert.signal_at)
        .unwrap_or_default()
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

fn default_notifications_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{AlertPayload, AlertRuntime};

    fn alert(symbol: &str, period: &str, signal_type: &str, signal_at: u64) -> AlertPayload {
        AlertPayload {
            id: format!("{symbol}:{period}:{signal_type}"),
            group_id: format!("{}-group", symbol.to_lowercase()),
            symbol: symbol.into(),
            period: period.into(),
            signal_type: signal_type.into(),
            side: 1,
            signal_at,
        }
    }

    #[test]
    fn reuses_the_same_visible_stream_for_same_symbol_alerts() {
        let mut runtime = AlertRuntime::default();

        runtime.enqueue_new_alerts(vec![
            alert("BTCUSDT", "60", "vegas", 1_000),
            alert("BTCUSDT", "240", "divMacd", 2_000),
        ]);

        assert_eq!(runtime.visible_popup_streams.len(), 1);
        assert_eq!(runtime.visible_popup_streams[0].symbol, "BTCUSDT");
        assert_eq!(runtime.visible_popup_streams[0].alerts.len(), 2);
        assert_eq!(runtime.visible_popup_streams[0].alerts[0].period, "240");
    }

    #[test]
    fn queues_lower_priority_streams_after_visible_capacity_is_reached() {
        let mut runtime = AlertRuntime::default();

        runtime.enqueue_new_alerts(vec![
            alert("BTCUSDT", "60", "vegas", 4_000),
            alert("ETHUSDT", "60", "vegas", 3_000),
            alert("SOLUSDT", "60", "vegas", 2_000),
            alert("XRPUSDT", "60", "vegas", 1_000),
        ]);

        assert_eq!(runtime.visible_popup_streams.len(), 3);
        assert_eq!(runtime.queued_popup_streams.len(), 1);
        assert_eq!(runtime.queued_popup_streams[0].symbol, "XRPUSDT");
    }

    #[test]
    fn removing_a_visible_alert_promotes_the_next_queued_stream() {
        let mut runtime = AlertRuntime::default();

        runtime.enqueue_new_alerts(vec![
            alert("BTCUSDT", "60", "vegas", 4_000),
            alert("ETHUSDT", "60", "vegas", 3_000),
            alert("SOLUSDT", "60", "vegas", 2_000),
            alert("XRPUSDT", "60", "vegas", 1_000),
        ]);

        let removed = runtime.remove_alert("SOLUSDT:60:vegas");

        assert!(removed);
        assert_eq!(runtime.visible_popup_streams.len(), 3);
        assert!(runtime
            .visible_popup_streams
            .iter()
            .any(|stream| stream.symbol == "XRPUSDT"));
        assert!(runtime.queued_popup_streams.is_empty());
    }
}
