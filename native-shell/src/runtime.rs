use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api_client::{ApiClient, SignalListQuery};
use crate::config;
use crate::notifications;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    pub base_url: String,
    pub api_key: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8787".into(),
            api_key: "signal-desk-dev-key".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PollConfig {
    pub interval_secs: u64,
    pub page_size: u32,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            interval_secs: 60,
            page_size: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub edge_mode: bool,
    pub edge_width: f64,
    pub always_on_top: bool,
    pub notifications: bool,
    pub sound: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            edge_mode: false,
            edge_width: 120.0,
            always_on_top: true,
            notifications: true,
            sound: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HookConfig {
    pub enabled: bool,
    pub command: String,
    pub args: Vec<String>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            command: String::new(),
            args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchGroupRowSortMode {
    ConfigOrder,
    RecentFirst,
}

impl Default for WatchGroupRowSortMode {
    fn default() -> Self {
        Self::ConfigOrder
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchGroup {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub periods: Vec<String>,
    pub signal_types: Vec<String>,
    pub row_sort_mode: WatchGroupRowSortMode,
    pub timeline_bars: i64,
    pub active_levels_only: bool,
    pub enabled: bool,
}

impl Default for WatchGroup {
    fn default() -> Self {
        Self {
            id: "group-1".into(),
            name: "BTC Main".into(),
            symbol: "BTCUSDT".into(),
            periods: vec!["60".into(), "15".into(), "5".into(), "1".into()],
            signal_types: vec!["divMacd".into()],
            row_sort_mode: WatchGroupRowSortMode::ConfigOrder,
            timeline_bars: 60,
            active_levels_only: true,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub poll: PollConfig,
    pub ui: UiConfig,
    pub hooks: HookConfig,
    pub groups: Vec<WatchGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSignal {
    pub group_id: String,
    pub group_name: String,
    pub symbol: String,
    pub period: String,
    pub signal_type: String,
    pub side: i8,
    pub trigger_time: i64,
    pub unread: bool,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub config: AppConfig,
    pub signals: Vec<RuntimeSignal>,
    pub unread_count: usize,
    pub last_tick: u64,
    pub last_updated_at: i64,
    pub last_connection_ok: Option<bool>,
    pub always_on_top: bool,
    pub edge_mode: bool,
    pub main_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalMutationInput {
    pub group_id: String,
    pub signal_type: String,
    pub period: String,
}

#[derive(Debug, Serialize)]
struct ReadStatusRequest {
    symbol: String,
    period: String,
    #[serde(rename = "signalType")]
    signal_type: String,
    read: bool,
}

#[derive(Debug, Serialize)]
struct ReadStatusBatchRequest {
    items: Vec<ReadStatusRequest>,
}

#[derive(Debug, Deserialize)]
struct ReadStatusBatchResponse {
    #[serde(default)]
    results: Vec<ReadStatusBatchResult>,
}

#[derive(Debug, Deserialize)]
struct ReadStatusBatchResult {
    symbol: String,
    #[serde(deserialize_with = "crate::api_client::deserialize_stringish")]
    period: String,
    #[serde(rename = "signalType")]
    signal_type: String,
    read: bool,
    #[serde(default)]
    success: bool,
}

#[derive(Debug)]
struct RuntimeStore {
    config: AppConfig,
    signals: Vec<RuntimeSignal>,
    unread_count: usize,
    last_tick: u64,
    last_updated_at: i64,
    last_connection_ok: Option<bool>,
    always_on_top: bool,
    edge_mode: bool,
    main_visible: bool,
}

#[derive(Clone, Debug)]
pub enum RuntimeCommand {
    RefreshNow,
    MarkSignalRead {
        input: SignalMutationInput,
        read: bool,
    },
    MarkSignalsRead {
        inputs: Vec<SignalMutationInput>,
        read: bool,
    },
    SetAlwaysOnTop(bool),
    SetEdgeMode(bool),
    SetEdgeWidth(f64),
    SetNotifications(bool),
    SetSound(bool),
    SetGroupRowSortMode {
        group_id: String,
        sort_mode: WatchGroupRowSortMode,
    },
    SetGroupTimelineBars {
        group_id: String,
        timeline_bars: i64,
    },
    SetGroupActiveLevelsOnly {
        group_id: String,
        active_levels_only: bool,
    },
    SaveConfig,
    Quit,
}

#[derive(Clone)]
pub struct RuntimeHandles {
    command_tx: Sender<RuntimeCommand>,
}

impl RuntimeHandles {
    pub fn request_refresh(&self) {
        let _ = self.command_tx.send(RuntimeCommand::RefreshNow);
    }

    pub fn request_quit(&self) {
        let _ = self.command_tx.send(RuntimeCommand::Quit);
    }

    pub fn request_mark_read(&self, input: SignalMutationInput, read: bool) {
        let _ = self
            .command_tx
            .send(RuntimeCommand::MarkSignalRead { input, read });
    }

    pub fn request_mark_read_batch(&self, inputs: Vec<SignalMutationInput>, read: bool) {
        let _ = self
            .command_tx
            .send(RuntimeCommand::MarkSignalsRead { inputs, read });
    }

    pub fn request_set_always_on_top(&self, pinned: bool) {
        let _ = self.command_tx.send(RuntimeCommand::SetAlwaysOnTop(pinned));
    }

    pub fn request_set_edge_mode(&self, enabled: bool) {
        let _ = self.command_tx.send(RuntimeCommand::SetEdgeMode(enabled));
    }

    pub fn request_set_edge_width(&self, width: f64) {
        let _ = self.command_tx.send(RuntimeCommand::SetEdgeWidth(width));
    }

    pub fn request_set_notifications(&self, enabled: bool) {
        let _ = self
            .command_tx
            .send(RuntimeCommand::SetNotifications(enabled));
    }

    pub fn request_set_sound(&self, enabled: bool) {
        let _ = self.command_tx.send(RuntimeCommand::SetSound(enabled));
    }

    pub fn request_set_group_row_sort_mode(
        &self,
        group_id: String,
        sort_mode: WatchGroupRowSortMode,
    ) {
        let _ = self.command_tx.send(RuntimeCommand::SetGroupRowSortMode {
            group_id,
            sort_mode,
        });
    }

    pub fn request_set_group_timeline_bars(&self, group_id: String, timeline_bars: i64) {
        let _ = self.command_tx.send(RuntimeCommand::SetGroupTimelineBars {
            group_id,
            timeline_bars,
        });
    }

    pub fn request_set_group_active_levels_only(&self, group_id: String, active_levels_only: bool) {
        let _ = self
            .command_tx
            .send(RuntimeCommand::SetGroupActiveLevelsOnly {
                group_id,
                active_levels_only,
            });
    }

    pub fn request_save_config(&self) {
        let _ = self.command_tx.send(RuntimeCommand::SaveConfig);
    }
}

impl RuntimeStore {
    fn new(config: AppConfig) -> Self {
        let mut config = config;
        sanitize_watch_groups(&mut config.groups);
        let mut store = Self {
            always_on_top: config.ui.always_on_top,
            edge_mode: config.ui.edge_mode,
            main_visible: true,
            config,
            signals: Vec::new(),
            unread_count: 0,
            last_tick: 0,
            last_updated_at: now_ms(),
            last_connection_ok: None,
        };
        store.signals = seed_signals(&store.config);
        store.recompute_unread();
        store
    }

    fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            config: self.config.clone(),
            signals: self.signals.clone(),
            unread_count: self.unread_count,
            last_tick: self.last_tick,
            last_updated_at: self.last_updated_at,
            last_connection_ok: self.last_connection_ok,
            always_on_top: self.always_on_top,
            edge_mode: self.edge_mode,
            main_visible: self.main_visible,
        }
    }

    fn recompute_unread(&mut self) {
        self.unread_count = self
            .signals
            .iter()
            .filter(|signal| signal.unread && !signal.deleted)
            .count();
    }

    fn apply_remote_signals(&mut self, signals: Vec<RuntimeSignal>, advance_tick: bool) {
        self.signals = signals;
        if advance_tick {
            self.last_tick += 1;
        }
        self.last_updated_at = now_ms();
        self.last_connection_ok = Some(true);
        self.recompute_unread();
    }

    fn mark_signal_read(&mut self, input: &SignalMutationInput, read: bool) -> bool {
        if let Some(signal) = self.signals.iter_mut().find(|signal| {
            !signal.deleted
                && signal.group_id == input.group_id
                && signal.signal_type == input.signal_type
                && signal.period == input.period
        }) {
            signal.unread = !read;
            self.last_updated_at = now_ms();
            self.last_connection_ok = Some(true);
            self.recompute_unread();
            return true;
        }

        false
    }

    fn config_for_persistence(&self) -> AppConfig {
        let mut config = self.config.clone();
        config.ui.always_on_top = self.always_on_top;
        config.ui.edge_mode = self.edge_mode;
        sanitize_watch_groups(&mut config.groups);
        config
    }
}

pub struct RuntimeModel {
    store: RuntimeStore,
}

impl RuntimeModel {
    pub fn new(config: AppConfig) -> Self {
        Self {
            store: RuntimeStore::new(config),
        }
    }

    pub fn load() -> Self {
        Self::new(config::load_config())
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        self.store.snapshot()
    }

    pub fn refresh_from_api(&mut self) -> Result<RuntimeSnapshot, String> {
        self.refresh_from_api_with_tick(true)
    }

    fn refresh_from_api_with_tick(
        &mut self,
        advance_tick: bool,
    ) -> Result<RuntimeSnapshot, String> {
        refresh_runtime_from_api(&mut self.store, advance_tick)
    }

    pub fn mark_signal_read_remote(
        &mut self,
        input: &SignalMutationInput,
        read: bool,
    ) -> Result<RuntimeSnapshot, String> {
        mark_signal_read_remote(&mut self.store, input, read)
    }

    pub fn mark_signals_read_batch_remote(
        &mut self,
        inputs: &[SignalMutationInput],
        read: bool,
    ) -> Result<RuntimeSnapshot, String> {
        mark_signals_read_batch_remote(&mut self.store, inputs, read)
    }

    pub fn set_always_on_top(&mut self, pinned: bool) -> RuntimeSnapshot {
        self.store.always_on_top = pinned;
        self.store.config.ui.always_on_top = pinned;
        self.store.snapshot()
    }

    pub fn set_edge_mode(&mut self, enabled: bool) -> RuntimeSnapshot {
        self.store.edge_mode = enabled;
        self.store.config.ui.edge_mode = enabled;
        self.store.snapshot()
    }

    pub fn set_edge_width(&mut self, width: f64) -> RuntimeSnapshot {
        self.store.config.ui.edge_width = width.clamp(160.0, 480.0);
        self.store.snapshot()
    }

    pub fn set_notifications(&mut self, enabled: bool) -> RuntimeSnapshot {
        self.store.config.ui.notifications = enabled;
        self.store.snapshot()
    }

    pub fn set_sound(&mut self, enabled: bool) -> RuntimeSnapshot {
        self.store.config.ui.sound = enabled;
        self.store.snapshot()
    }

    pub fn set_group_row_sort_mode(
        &mut self,
        group_id: &str,
        sort_mode: WatchGroupRowSortMode,
    ) -> RuntimeSnapshot {
        if let Some(group) = self
            .store
            .config
            .groups
            .iter_mut()
            .find(|group| group.id == group_id)
        {
            group.row_sort_mode = sort_mode;
        }
        self.store.snapshot()
    }

    pub fn set_group_timeline_bars(
        &mut self,
        group_id: &str,
        timeline_bars: i64,
    ) -> RuntimeSnapshot {
        if let Some(group) = self
            .store
            .config
            .groups
            .iter_mut()
            .find(|group| group.id == group_id)
        {
            group.timeline_bars = clamp_timeline_bars(timeline_bars);
        }
        self.store.snapshot()
    }

    pub fn set_group_active_levels_only(
        &mut self,
        group_id: &str,
        active_levels_only: bool,
    ) -> RuntimeSnapshot {
        if let Some(group) = self
            .store
            .config
            .groups
            .iter_mut()
            .find(|group| group.id == group_id)
        {
            group.active_levels_only = active_levels_only;
        }
        self.store.snapshot()
    }

    pub fn save_config(&mut self) -> Result<RuntimeSnapshot, String> {
        let path = config::resolve_config_path_for_write();
        self.save_config_to_path(&path)
    }

    pub fn save_config_to_path(&mut self, path: &Path) -> Result<RuntimeSnapshot, String> {
        let config = self.store.config_for_persistence();
        config::save_config_to_path(&config, path)?;
        self.store.config = config;
        Ok(self.store.snapshot())
    }

    pub fn mark_last_connection_failed(&mut self) -> RuntimeSnapshot {
        self.store.last_connection_ok = Some(false);
        self.store.snapshot()
    }
}

pub fn runtime_snapshot_from_config(config: AppConfig) -> RuntimeSnapshot {
    RuntimeStore::new(config).snapshot()
}

pub fn load_runtime_snapshot() -> RuntimeSnapshot {
    RuntimeModel::load().snapshot()
}

pub fn spawn_runtime_loop<F, E>(on_snapshot: F, on_error: E) -> RuntimeHandles
where
    F: Fn(RuntimeSnapshot) + Send + Sync + 'static,
    E: Fn(String, RuntimeSnapshot) + Send + Sync + 'static,
{
    let (tx, rx) = mpsc::channel::<RuntimeCommand>();
    let on_snapshot = Arc::new(on_snapshot);
    let on_error = Arc::new(on_error);

    thread::spawn(move || {
        let mut runtime = RuntimeModel::load();
        on_snapshot(runtime.snapshot());

        match runtime.refresh_from_api_with_tick(false) {
            Ok(snapshot) => on_snapshot(snapshot),
            Err(error) => on_error(error, runtime.mark_last_connection_failed()),
        }

        loop {
            let wait_for = Duration::from_secs(runtime.snapshot().config.poll.interval_secs.max(1));
            match rx.recv_timeout(wait_for) {
                Ok(RuntimeCommand::RefreshNow) => match runtime.refresh_from_api() {
                    Ok(snapshot) => on_snapshot(snapshot),
                    Err(error) => on_error(error, runtime.mark_last_connection_failed()),
                },
                Ok(RuntimeCommand::MarkSignalRead { input, read }) => {
                    match runtime.mark_signal_read_remote(&input, read) {
                        Ok(snapshot) => on_snapshot(snapshot),
                        Err(error) => on_error(error, runtime.mark_last_connection_failed()),
                    }
                }
                Ok(RuntimeCommand::MarkSignalsRead { inputs, read }) => {
                    match runtime.mark_signals_read_batch_remote(&inputs, read) {
                        Ok(snapshot) => on_snapshot(snapshot),
                        Err(error) => on_error(error, runtime.mark_last_connection_failed()),
                    }
                }
                Ok(RuntimeCommand::SetAlwaysOnTop(pinned)) => {
                    on_snapshot(runtime.set_always_on_top(pinned));
                }
                Ok(RuntimeCommand::SetEdgeMode(enabled)) => {
                    on_snapshot(runtime.set_edge_mode(enabled));
                }
                Ok(RuntimeCommand::SetEdgeWidth(width)) => {
                    on_snapshot(runtime.set_edge_width(width));
                }
                Ok(RuntimeCommand::SetNotifications(enabled)) => {
                    on_snapshot(runtime.set_notifications(enabled));
                }
                Ok(RuntimeCommand::SetSound(enabled)) => {
                    on_snapshot(runtime.set_sound(enabled));
                }
                Ok(RuntimeCommand::SetGroupRowSortMode {
                    group_id,
                    sort_mode,
                }) => {
                    on_snapshot(runtime.set_group_row_sort_mode(&group_id, sort_mode));
                }
                Ok(RuntimeCommand::SetGroupTimelineBars {
                    group_id,
                    timeline_bars,
                }) => {
                    on_snapshot(runtime.set_group_timeline_bars(&group_id, timeline_bars));
                }
                Ok(RuntimeCommand::SetGroupActiveLevelsOnly {
                    group_id,
                    active_levels_only,
                }) => {
                    on_snapshot(
                        runtime.set_group_active_levels_only(&group_id, active_levels_only),
                    );
                }
                Ok(RuntimeCommand::SaveConfig) => match runtime.save_config() {
                    Ok(snapshot) => on_snapshot(snapshot),
                    Err(error) => on_error(error, runtime.snapshot()),
                },
                Ok(RuntimeCommand::Quit) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => match runtime.refresh_from_api() {
                    Ok(snapshot) => on_snapshot(snapshot),
                    Err(error) => on_error(error, runtime.mark_last_connection_failed()),
                },
            }
        }
    });

    RuntimeHandles { command_tx: tx }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn seed_signals(config: &AppConfig) -> Vec<RuntimeSignal> {
    let mut signals = Vec::new();

    for group in config.groups.iter().filter(|group| group.enabled) {
        for signal_type in &group.signal_types {
            for period in &group.periods {
                signals.push(RuntimeSignal {
                    group_id: group.id.clone(),
                    group_name: group.name.clone(),
                    symbol: group.symbol.clone(),
                    period: period.clone(),
                    signal_type: signal_type.clone(),
                    side: 1,
                    trigger_time: 0,
                    unread: false,
                    deleted: false,
                });
            }
        }
    }

    signals
}

fn fetch_runtime_signals(config: &AppConfig) -> Result<Vec<RuntimeSignal>, String> {
    let client = ApiClient::new(&config.api.base_url, &config.api.api_key)?;

    let mut signals = seed_signals(config);
    let mut index_by_key = HashMap::new();

    for (index, signal) in signals.iter().enumerate() {
        index_by_key.insert(
            (
                signal.group_id.clone(),
                signal.signal_type.clone(),
                signal.period.clone(),
            ),
            index,
        );
    }

    for group in config.groups.iter().filter(|group| group.enabled) {
        let request = SignalListQuery {
            symbols: group.symbol.clone(),
            periods: group.periods.join(","),
            signal_types: group.signal_types.join(","),
            page: 1,
            page_size: config.poll.page_size,
        };

        let response = client.fetch_signal_list(&request)?;

        for item in response.data {
            for (signal_type, detail) in item.signals {
                if let Some(index) =
                    index_by_key.get(&(group.id.clone(), signal_type.clone(), item.period.clone()))
                {
                    let signal = &mut signals[*index];
                    signal.symbol = item.symbol.clone();
                    signal.side = if detail.sd >= 0 { 1 } else { -1 };
                    signal.trigger_time = detail.t;
                    signal.unread = !detail.read;
                }
            }
        }
    }

    Ok(signals)
}

fn refresh_runtime_from_api(
    store: &mut RuntimeStore,
    advance_tick: bool,
) -> Result<RuntimeSnapshot, String> {
    let previous = store.signals.clone();
    let signals = fetch_runtime_signals(&store.config)?;
    let alerts = if advance_tick {
        notifications::collect_new_alerts(&previous, &signals)
    } else {
        Vec::new()
    };
    store.apply_remote_signals(signals, advance_tick);
    notifications::emit_alerts(&alerts, &store.config.ui);
    crate::hooks::emit_new_signal_hook(&alerts, &store.config.hooks);
    Ok(store.snapshot())
}

fn mark_signal_read_remote(
    store: &mut RuntimeStore,
    input: &SignalMutationInput,
    read: bool,
) -> Result<RuntimeSnapshot, String> {
    mark_signals_read_batch_remote(store, std::slice::from_ref(input), read)
}

fn mark_signals_read_batch_remote(
    store: &mut RuntimeStore,
    inputs: &[SignalMutationInput],
    read: bool,
) -> Result<RuntimeSnapshot, String> {
    let mut items = Vec::new();
    let mut requested = Vec::new();

    for input in inputs {
        let signal = store
            .signals
            .iter()
            .find(|signal| {
                !signal.deleted
                    && signal.group_id == input.group_id
                    && signal.signal_type == input.signal_type
                    && signal.period == input.period
            })
            .cloned()
            .ok_or_else(|| "signal not found".to_string())?;

        items.push(ReadStatusRequest {
            symbol: signal.symbol,
            period: input.period.clone(),
            signal_type: input.signal_type.clone(),
            read,
        });
        requested.push(input.clone());
    }

    if items.is_empty() {
        return Ok(store.snapshot());
    }

    let client = ApiClient::new(&store.config.api.base_url, &store.config.api.api_key)?;

    let response: ReadStatusBatchResponse = client.post_json(
        "/api/open/watch-list/symbol-alert/read-status/batch",
        &ReadStatusBatchRequest { items },
    )?;

    let applied_inputs = response
        .results
        .iter()
        .filter(|result| result.success)
        .flat_map(|result| {
            requested.iter().filter(|input| {
                input.period == result.period
                    && input.signal_type == result.signal_type
                    && result.read == read
                    && store.signals.iter().any(|signal| {
                        !signal.deleted
                            && signal.group_id == input.group_id
                            && signal.period == input.period
                            && signal.signal_type == input.signal_type
                            && signal.symbol == result.symbol
                    })
            })
        })
        .cloned()
        .collect::<Vec<_>>();

    for input in &applied_inputs {
        store.mark_signal_read(input, read);
    }

    Ok(store.snapshot())
}

pub fn config_location_hint() -> String {
    config::config_location_hint()
}

pub fn clamp_timeline_bars(value: i64) -> i64 {
    value.clamp(10, 500)
}

pub fn parse_timeline_bars_input(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    trimmed.parse::<i64>().ok().map(clamp_timeline_bars)
}

fn sanitize_watch_groups(groups: &mut [WatchGroup]) {
    for group in groups {
        group.timeline_bars = clamp_timeline_bars(group.timeline_bars);
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::{ApiConfig, AppConfig, RuntimeStore, WatchGroup};

    #[test]
    fn apply_remote_signals_can_refresh_without_advancing_tick() {
        let config = AppConfig {
            groups: vec![WatchGroup::default()],
            ..Default::default()
        };
        let mut store = RuntimeStore::new(config);
        let mut signals = store.signals.clone();
        signals[0].trigger_time = 123;

        store.apply_remote_signals(signals, false);

        assert_eq!(store.last_tick, 0);
        assert_eq!(store.signals[0].trigger_time, 123);
    }

    #[test]
    fn apply_remote_signals_advances_tick_for_real_refreshes() {
        let config = AppConfig {
            groups: vec![WatchGroup::default()],
            ..Default::default()
        };
        let mut store = RuntimeStore::new(config);
        let mut signals = store.signals.clone();
        signals[0].trigger_time = 456;

        store.apply_remote_signals(signals, true);

        assert_eq!(store.last_tick, 1);
        assert_eq!(store.signals[0].trigger_time, 456);
    }

    #[test]
    fn mark_signals_read_batch_remote_posts_batch_and_applies_successful_results() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buffer = [0_u8; 8192];
            let bytes_read = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);

            assert!(request
                .starts_with("POST /api/open/watch-list/symbol-alert/read-status/batch HTTP/1.1"));
            assert!(request.contains("x-api-key: test-key"));
            assert!(request.contains("\"items\":["));
            assert!(request.contains(
                "{\"symbol\":\"BTCUSDT\",\"period\":\"60\",\"signalType\":\"divMacd\",\"read\":true}"
            ));
            assert!(request.contains(
                "{\"symbol\":\"BTCUSDT\",\"period\":\"15\",\"signalType\":\"divMacd\",\"read\":true}"
            ));

            let response_body = concat!(
                "{\"success\":1,\"failed\":1,\"results\":[",
                "{\"symbol\":\"BTCUSDT\",\"period\":\"60\",\"signalType\":\"divMacd\",\"read\":true,\"success\":true},",
                "{\"symbol\":\"BTCUSDT\",\"period\":\"15\",\"signalType\":\"divMacd\",\"read\":true,\"success\":false,\"reason\":\"signal_not_found\"}",
                "]}"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        let config = AppConfig {
            api: ApiConfig {
                base_url,
                api_key: "test-key".into(),
            },
            groups: vec![WatchGroup::default()],
            ..Default::default()
        };
        let mut store = RuntimeStore::new(config);
        store.signals[0].unread = true;
        store.signals[1].unread = true;
        store.recompute_unread();

        let snapshot = super::mark_signals_read_batch_remote(
            &mut store,
            &[
                super::SignalMutationInput {
                    group_id: "group-1".into(),
                    signal_type: "divMacd".into(),
                    period: "60".into(),
                },
                super::SignalMutationInput {
                    group_id: "group-1".into(),
                    signal_type: "divMacd".into(),
                    period: "15".into(),
                },
            ],
            true,
        )
        .expect("batch mark read succeeds");

        server.join().expect("server thread");
        assert!(!snapshot.signals[0].unread);
        assert!(snapshot.signals[1].unread);
        assert_eq!(snapshot.unread_count, 1);
    }

    #[test]
    fn mark_signal_read_remote_uses_batch_endpoint_for_single_item() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut buffer = [0_u8; 8192];
            let bytes_read = stream.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);

            assert!(request
                .starts_with("POST /api/open/watch-list/symbol-alert/read-status/batch HTTP/1.1"));
            assert!(request.contains(
                "{\"symbol\":\"BTCUSDT\",\"period\":\"60\",\"signalType\":\"divMacd\",\"read\":false}"
            ));

            let response_body = concat!(
                "{\"success\":1,\"failed\":0,\"results\":[",
                "{\"symbol\":\"BTCUSDT\",\"period\":\"60\",\"signalType\":\"divMacd\",\"read\":false,\"success\":true}",
                "]}"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        let config = AppConfig {
            api: ApiConfig {
                base_url,
                api_key: "test-key".into(),
            },
            groups: vec![WatchGroup::default()],
            ..Default::default()
        };
        let mut store = RuntimeStore::new(config);

        let snapshot = super::mark_signal_read_remote(
            &mut store,
            &super::SignalMutationInput {
                group_id: "group-1".into(),
                signal_type: "divMacd".into(),
                period: "60".into(),
            },
            false,
        )
        .expect("single mark unread succeeds");

        server.join().expect("server thread");
        assert!(snapshot.signals[0].unread);
        assert_eq!(snapshot.unread_count, 1);
    }
}
