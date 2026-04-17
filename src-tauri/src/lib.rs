use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{
    async_runtime::spawn,
    menu::{MenuBuilder, MenuEvent, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl,
    WebviewWindowBuilder, WindowEvent,
};
use tokio::time::sleep;

const MAIN_WINDOW: &str = "main";
const WIDGET_WINDOW: &str = "widget";
const RUNTIME_EVENT: &str = "runtime://state-changed";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct ApiConfig {
    base_url: String,
    api_key: String,
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
struct PollConfig {
    interval_secs: u64,
    page_size: u32,
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
struct UiConfig {
    edge_mode: bool,
    edge_width: f64,
    always_on_top: bool,
    notifications: bool,
    sound: bool,
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
struct WatchGroup {
    id: String,
    name: String,
    symbol: String,
    periods: Vec<String>,
    signal_types: Vec<String>,
    enabled: bool,
}

impl Default for WatchGroup {
    fn default() -> Self {
        Self {
            id: "group-1".into(),
            name: "BTC Main".into(),
            symbol: "BTCUSDT".into(),
            periods: vec!["60".into(), "15".into(), "5".into(), "1".into()],
            signal_types: vec!["divMacd".into()],
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct AppConfig {
    api: ApiConfig,
    poll: PollConfig,
    ui: UiConfig,
    groups: Vec<WatchGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeSignal {
    group_id: String,
    group_name: String,
    symbol: String,
    period: String,
    signal_type: String,
    side: i8,
    trigger_time: i64,
    unread: bool,
    deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeSnapshot {
    config: AppConfig,
    signals: Vec<RuntimeSignal>,
    unread_count: usize,
    last_tick: u64,
    last_updated_at: i64,
    always_on_top: bool,
    edge_mode: bool,
    main_visible: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct SignalMutationInput {
    group_id: String,
    signal_type: String,
    period: String,
}

#[derive(Debug, Serialize)]
struct SignalListRequest {
    symbols: String,
    periods: String,
    #[serde(rename = "signalTypes")]
    signal_types: String,
    page: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
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
struct DeleteSignalRequest {
    symbol: String,
    period: String,
    #[serde(rename = "signalType")]
    signal_type: String,
}

#[derive(Debug, Deserialize)]
struct SignalListResponse {
    #[serde(default)]
    data: Vec<SignalListItem>,
}

#[derive(Debug, Deserialize)]
struct SignalListItem {
    symbol: String,
    #[serde(deserialize_with = "deserialize_stringish")]
    period: String,
    #[serde(default)]
    signals: HashMap<String, RemoteSignalDetail>,
}

#[derive(Debug, Deserialize)]
struct RemoteSignalDetail {
    #[serde(default)]
    sd: i8,
    #[serde(default)]
    t: i64,
    #[serde(default)]
    read: bool,
}

#[derive(Debug, Clone, Copy)]
struct WindowRestoreBounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

#[derive(Debug, Default)]
struct RuntimeStore {
    config: AppConfig,
    signals: Vec<RuntimeSignal>,
    unread_count: usize,
    last_tick: u64,
    last_updated_at: i64,
    always_on_top: bool,
    edge_mode: bool,
    main_visible: bool,
    restore_bounds: Option<WindowRestoreBounds>,
}

#[derive(Clone)]
struct SharedState(Arc<Mutex<RuntimeStore>>);

impl SharedState {
    fn new(store: RuntimeStore) -> Self {
        Self(Arc::new(Mutex::new(store)))
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn deserialize_stringish<'de, D>(deserializer: D) -> Result<String, D::Error>
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

fn default_config() -> AppConfig {
    AppConfig {
        groups: vec![WatchGroup::default()],
        ..Default::default()
    }
}

fn push_config_candidates(candidates: &mut Vec<PathBuf>, base: PathBuf) {
    let mut current = Some(base.as_path());
    let mut depth = 0;

    while let Some(path) = current {
        candidates.push(path.join("config.yaml"));
        candidates.push(path.join("config.yaml.example"));
        current = path.parent();
        depth += 1;

        if depth >= 4 {
            break;
        }
    }
}

fn resolve_config_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current) = std::env::current_dir() {
        let current_name = current.file_name().and_then(|value| value.to_str());
        if current_name == Some("src-tauri") {
            if let Some(parent) = current.parent() {
                push_config_candidates(&mut candidates, parent.to_path_buf());
            }
            push_config_candidates(&mut candidates, current);
        } else {
            push_config_candidates(&mut candidates, current);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            push_config_candidates(&mut candidates, parent.to_path_buf());
        }
    }

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.clone()))
        .collect()
}

fn load_config() -> AppConfig {
    for candidate in resolve_config_candidates() {
        if let Ok(content) = fs::read_to_string(&candidate) {
            if let Ok(config) = serde_yaml::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }

    default_config()
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

impl RuntimeStore {
    fn new(config: AppConfig) -> Self {
        let mut store = Self {
            always_on_top: config.ui.always_on_top,
            edge_mode: config.ui.edge_mode,
            main_visible: true,
            config,
            ..Default::default()
        };
        store.signals = seed_signals(&store.config);
        store.recompute_unread();
        store.last_updated_at = now_ms();
        store
    }

    fn recompute_unread(&mut self) {
        self.unread_count = self
            .signals
            .iter()
            .filter(|signal| signal.unread && !signal.deleted)
            .count();
    }

    fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            config: self.config.clone(),
            signals: self.signals.clone(),
            unread_count: self.unread_count,
            last_tick: self.last_tick,
            last_updated_at: self.last_updated_at,
            always_on_top: self.always_on_top,
            edge_mode: self.edge_mode,
            main_visible: self.main_visible,
        }
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
            self.recompute_unread();
            return true;
        }

        false
    }

    fn delete_signal(&mut self, input: &SignalMutationInput) -> bool {
        if let Some(signal) = self.signals.iter_mut().find(|signal| {
            !signal.deleted
                && signal.group_id == input.group_id
                && signal.signal_type == input.signal_type
                && signal.period == input.period
        }) {
            signal.deleted = true;
            signal.unread = false;
            self.last_updated_at = now_ms();
            self.recompute_unread();
            return true;
        }

        false
    }

    fn apply_remote_signals(&mut self, signals: Vec<RuntimeSignal>) {
        self.signals = signals;
        self.last_tick += 1;
        self.last_updated_at = now_ms();
        self.recompute_unread();
    }
}

fn emit_snapshot(app: &AppHandle, snapshot: RuntimeSnapshot) {
    let _ = app.emit(RUNTIME_EVENT, snapshot);
}

fn with_store<T>(state: &State<'_, SharedState>, f: impl FnOnce(&mut RuntimeStore) -> T) -> T {
    let mut guard = state.0.lock().expect("runtime store poisoned");
    f(&mut guard)
}

async fn post_json<TReq: Serialize, TRes: for<'de> Deserialize<'de>>(
    config: &AppConfig,
    path: &str,
    body: &TReq,
) -> Result<TRes, String> {
    let url = format!(
        "{}/{}",
        config.api.base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    );

    let response = Client::new()
        .post(url)
        .header("x-api-key", &config.api.api_key)
        .json(body)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("request failed: {} {}", status, body));
    }

    response.json::<TRes>().await.map_err(|err| err.to_string())
}

async fn post_json_unit<TReq: Serialize>(
    config: &AppConfig,
    path: &str,
    body: &TReq,
) -> Result<(), String> {
    let url = format!(
        "{}/{}",
        config.api.base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    );

    let response = Client::new()
        .post(url)
        .header("x-api-key", &config.api.api_key)
        .json(body)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("request failed: {} {}", status, body));
    }

    Ok(())
}

async fn fetch_runtime_signals(config: &AppConfig) -> Result<Vec<RuntimeSignal>, String> {
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
        let request = SignalListRequest {
            symbols: group.symbol.clone(),
            periods: group.periods.join(","),
            signal_types: group.signal_types.join(","),
            page: 1,
            page_size: config.poll.page_size,
        };

        let response: SignalListResponse = post_json(
            config,
            "/api/open/watch-list/symbol-signals",
            &request,
        )
        .await?;

        for item in response.data {
            for (signal_type, detail) in item.signals {
                if let Some(index) = index_by_key.get(&(
                    group.id.clone(),
                    signal_type.clone(),
                    item.period.clone(),
                )) {
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

async fn refresh_runtime_from_api(
    app: &AppHandle,
    shared_state: &SharedState,
) -> Result<RuntimeSnapshot, String> {
    let config = {
        let guard = shared_state.0.lock().expect("runtime store poisoned");
        guard.config.clone()
    };

    let signals = fetch_runtime_signals(&config).await?;
    let snapshot = {
        let mut guard = shared_state.0.lock().expect("runtime store poisoned");
        guard.apply_remote_signals(signals);
        guard.snapshot()
    };

    emit_snapshot(app, snapshot.clone());
    Ok(snapshot)
}

fn window_visible(window: &tauri::WebviewWindow) -> bool {
    window.is_visible().unwrap_or(false)
}

fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "main window not found".to_string())?;

    window.show().map_err(|err| err.to_string())?;
    let _ = window.unminimize();
    let _ = window.set_focus();
    Ok(())
}

fn toggle_main_window(app: &AppHandle, state: &SharedState) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "main window not found".to_string())?;

    let is_visible = window_visible(&window);
    if is_visible {
        window.hide().map_err(|err| err.to_string())?;
    } else {
        window.show().map_err(|err| err.to_string())?;
        let _ = window.set_focus();
    }

    let snapshot = {
        let mut guard = state.0.lock().expect("runtime store poisoned");
        guard.main_visible = !is_visible;
        guard.snapshot()
    };
    emit_snapshot(app, snapshot);
    Ok(())
}

fn position_widget(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WIDGET_WINDOW) {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let width = 112.0;
            let x = (size.width as f64 / scale) - width - 28.0;
            let y = 32.0;
            let _ = window.set_position(tauri::Position::Logical(LogicalPosition::new(x, y)));
        }
    }
}

fn ensure_widget_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(WIDGET_WINDOW).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        WIDGET_WINDOW,
        WebviewUrl::App("index.html?view=widget".into()),
    )
    .title("Signal Desk Widget")
    .inner_size(112.0, 112.0)
    .min_inner_size(112.0, 112.0)
    .max_inner_size(112.0, 112.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .visible(true)
    .build()?;

    let _ = window.set_ignore_cursor_events(false);
    position_widget(app);
    Ok(())
}

fn update_main_window_pinning(app: &AppHandle, pinned: bool) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "main window not found".to_string())?;
    window
        .set_always_on_top(pinned)
        .map_err(|err| err.to_string())
}

fn apply_edge_mode(app: &AppHandle, enabled: bool, edge_width: f64) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "main window not found".to_string())?;

    if enabled {
        let _ = window.set_size(tauri::Size::Logical(LogicalSize::new(edge_width, 840.0)));
        if let Ok(Some(monitor)) = window.current_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let x = (size.width as f64 / scale) - edge_width - 12.0;
            let y = 80.0;
            let _ = window.set_position(tauri::Position::Logical(LogicalPosition::new(x, y)));
        }
    } else {
        let _ = window.set_size(tauri::Size::Logical(LogicalSize::new(560.0, 760.0)));
        let _ = window.center();
    }

    Ok(())
}

fn capture_main_window_bounds(app: &AppHandle) -> Option<WindowRestoreBounds> {
    let window = app.get_webview_window(MAIN_WINDOW)?;
    let position = window.outer_position().ok()?;
    let size = window.inner_size().ok()?;

    Some(WindowRestoreBounds {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn restore_main_window_bounds(app: &AppHandle, bounds: WindowRestoreBounds) -> Result<(), String> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| "main window not found".to_string())?;

    window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
            bounds.width,
            bounds.height,
        )))
        .map_err(|err| err.to_string())?;
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
            bounds.x,
            bounds.y,
        )))
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn build_tray(app: &App) -> tauri::Result<()> {
    let toggle_main = MenuItemBuilder::with_id("toggle_main", "Toggle Main Window").build(app)?;
    let simulate_signal = MenuItemBuilder::with_id("simulate_signal", "Simulate Signal").build(app)?;
    let toggle_pin = MenuItemBuilder::with_id("toggle_pin", "Toggle Pin").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle_main)
        .item(&simulate_signal)
        .item(&toggle_pin)
        .separator()
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Signal Desk")
        .on_menu_event(|app, event| handle_menu_event(app, event))
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                if let Some(state) = tray.app_handle().try_state::<SharedState>() {
                    let _ = toggle_main_window(&tray.app_handle(), &state);
                }
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let Some(state) = app.try_state::<SharedState>() else {
        return;
    };

    match event.id().as_ref() {
        "toggle_main" => {
            let _ = toggle_main_window(app, &state);
        }
        "simulate_signal" => {
            let app_handle = app.clone();
            let shared_state = state.inner().clone();
            spawn(async move {
                if refresh_runtime_from_api(&app_handle, &shared_state).await.is_ok() {
                    let _ = show_main_window(&app_handle);
                }
            });
        }
        "toggle_pin" => {
            let snapshot = {
                let mut guard = state.0.lock().expect("runtime store poisoned");
                guard.always_on_top = !guard.always_on_top;
                let _ = update_main_window_pinning(app, guard.always_on_top);
                guard.snapshot()
            };
            emit_snapshot(app, snapshot);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn spawn_runtime_loop(app: AppHandle) {
    spawn(async move {
        if let Some(state) = app.try_state::<SharedState>() {
            let _ = refresh_runtime_from_api(&app, &state).await;
        }

        loop {
            let interval_secs = app
                .try_state::<SharedState>()
                .and_then(|state| state.0.lock().ok().map(|guard| guard.config.poll.interval_secs))
                .unwrap_or(60)
                .clamp(12, 600);

            sleep(Duration::from_secs(interval_secs)).await;

            let Some(state) = app.try_state::<SharedState>() else {
                continue;
            };

            if let Ok(snapshot) = refresh_runtime_from_api(&app, &state).await {
                if snapshot.unread_count > 0 {
                    let _ = show_main_window(&app);
                }
            }
        }
    });
}

#[tauri::command]
fn get_runtime_snapshot(state: State<'_, SharedState>) -> RuntimeSnapshot {
    with_store(&state, |store| store.snapshot())
}

#[tauri::command]
async fn mark_signal_read(
    app: AppHandle,
    state: State<'_, SharedState>,
    input: SignalMutationInput,
    read: bool,
) -> Result<RuntimeSnapshot, String> {
    let (config, symbol) = with_store(&state, |store| {
        let signal = store
            .signals
            .iter()
            .find(|signal| {
                !signal.deleted
                    && signal.group_id == input.group_id
                    && signal.signal_type == input.signal_type
                    && signal.period == input.period
            })
            .cloned();
        signal.map(|signal| (store.config.clone(), signal.symbol))
    })
    .ok_or_else(|| "signal not found".to_string())?;

    let request = ReadStatusRequest {
        symbol,
        period: input.period.clone(),
        signal_type: input.signal_type.clone(),
        read,
    };
    post_json_unit(
        &config,
        "/api/open/watch-list/symbol-alert/read-status",
        &request,
    )
    .await?;

    let snapshot = with_store(&state, |store| {
        if !store.mark_signal_read(&input, read) {
            return Err("signal not found".to_string());
        }
        Ok(store.snapshot())
    })?;
    emit_snapshot(&app, snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
async fn delete_signal(
    app: AppHandle,
    state: State<'_, SharedState>,
    input: SignalMutationInput,
) -> Result<RuntimeSnapshot, String> {
    let (config, symbol) = with_store(&state, |store| {
        let signal = store
            .signals
            .iter()
            .find(|signal| {
                !signal.deleted
                    && signal.group_id == input.group_id
                    && signal.signal_type == input.signal_type
                    && signal.period == input.period
            })
            .cloned();
        signal.map(|signal| (store.config.clone(), signal.symbol))
    })
    .ok_or_else(|| "signal not found".to_string())?;

    let request = DeleteSignalRequest {
        symbol,
        period: input.period.clone(),
        signal_type: input.signal_type.clone(),
    };
    post_json_unit(
        &config,
        "/api/open/watch-list/symbol-alert/delete-signal",
        &request,
    )
    .await?;

    let snapshot = with_store(&state, |store| {
        if !store.delete_signal(&input) {
            return Err("signal not found".to_string());
        }
        Ok(store.snapshot())
    })?;
    emit_snapshot(&app, snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
async fn trigger_mock_signal(
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<RuntimeSnapshot, String> {
    let snapshot = refresh_runtime_from_api(&app, &state).await?;
    let _ = show_main_window(&app);
    Ok(snapshot)
}

#[tauri::command]
fn toggle_main(app: AppHandle, state: State<'_, SharedState>) -> Result<(), String> {
    toggle_main_window(&app, &state)
}

#[tauri::command]
fn set_always_on_top(
    app: AppHandle,
    state: State<'_, SharedState>,
    pinned: bool,
) -> Result<RuntimeSnapshot, String> {
    update_main_window_pinning(&app, pinned)?;
    let snapshot = with_store(&state, |store| {
        store.always_on_top = pinned;
        store.snapshot()
    });
    emit_snapshot(&app, snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
fn set_edge_mode(
    app: AppHandle,
    state: State<'_, SharedState>,
    enabled: bool,
) -> Result<RuntimeSnapshot, String> {
    let (edge_width, restore_bounds, should_capture) = with_store(&state, |store| {
        (
            store.config.ui.edge_width,
            store.restore_bounds,
            enabled && !store.edge_mode,
        )
    });

    if should_capture {
        let captured = capture_main_window_bounds(&app);
        with_store(&state, |store| {
            store.restore_bounds = captured;
        });
    }

    if enabled {
        apply_edge_mode(&app, true, edge_width)?;
    } else if let Some(bounds) = restore_bounds {
        restore_main_window_bounds(&app, bounds)?;
    } else {
        apply_edge_mode(&app, false, edge_width)?;
    }

    let snapshot = with_store(&state, |store| {
        store.edge_mode = enabled;
        if !enabled {
            store.restore_bounds = None;
        }
        store.snapshot()
    });
    emit_snapshot(&app, snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
fn set_edge_width(
    app: AppHandle,
    state: State<'_, SharedState>,
    width: f64,
) -> Result<RuntimeSnapshot, String> {
    let normalized = width.clamp(160.0, 480.0);
    let edge_mode = with_store(&state, |store| {
        store.config.ui.edge_width = normalized;
        store.edge_mode
    });

    if edge_mode {
        apply_edge_mode(&app, true, normalized)?;
    }

    let snapshot = with_store(&state, |store| store.snapshot());
    emit_snapshot(&app, snapshot.clone());
    Ok(snapshot)
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = load_config();
    let store = RuntimeStore::new(config);
    let shared_state = SharedState::new(store);

    tauri::Builder::default()
        .manage(shared_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_runtime_snapshot,
            mark_signal_read,
            delete_signal,
            trigger_mock_signal,
            toggle_main,
            set_always_on_top,
            set_edge_mode,
            set_edge_width,
            quit_app
        ])
        .setup(|app| {
            ensure_widget_window(&app.handle())?;
            build_tray(app)?;

            let (always_on_top, edge_mode, edge_width) = if let Some(state) = app.try_state::<SharedState>()
            {
                let guard = state.0.lock().expect("runtime store poisoned");
                (guard.always_on_top, guard.edge_mode, guard.config.ui.edge_width)
            } else {
                (true, false, 120.0)
            };

            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = window.set_title("Signal Desk Console");
                let _ = window.set_size(tauri::Size::Logical(LogicalSize::new(560.0, 760.0)));
                let _ = window.center();
                let _ = window.set_always_on_top(always_on_top);
            }

            let _ = apply_edge_mode(&app.handle(), edge_mode, edge_width);

            let app_handle = app.handle().clone();
            spawn_runtime_loop(app_handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == MAIN_WINDOW {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                    if let Some(state) = window.app_handle().try_state::<SharedState>() {
                        let snapshot = {
                            let mut guard = state.0.lock().expect("runtime store poisoned");
                            guard.main_visible = false;
                            guard.snapshot()
                        };
                        emit_snapshot(&window.app_handle(), snapshot);
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
