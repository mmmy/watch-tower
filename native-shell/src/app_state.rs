use crate::runtime::{RuntimeSignal, RuntimeSnapshot, SignalMutationInput};

#[derive(Clone, Debug)]
pub struct UiSnapshot {
    pub unread_count: i32,
    pub status_text: String,
    pub refresh_label: String,
    pub runtime_summary: String,
    pub save_hint: String,
    pub connection_label: String,
    pub connection_tone: String,
    pub stats_primary: String,
    pub stats_secondary: String,
    pub stats_tertiary: String,
    pub stats_quaternary: String,
    pub edge_width_label: String,
    pub signal_rows: Vec<UiSignalRow>,
    pub main_visible: bool,
    pub widget_visible: bool,
    pub always_on_top: bool,
    pub edge_mode: bool,
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct UiSignalRow {
    pub title: String,
    pub meta: String,
    pub is_header: bool,
    pub unread: bool,
    pub pending: bool,
    pub unread_count: i32,
    pub timeline_visible: bool,
    pub timeline_ratio: f32,
    pub timeline_positive: bool,
}

#[derive(Clone, Debug)]
enum RowAction {
    Group(Vec<SignalMutationInput>),
    Single {
        key: SignalMutationInput,
        read: bool,
    },
}

#[derive(Clone, Debug)]
struct SignalRowView {
    row: UiSignalRow,
    action: Option<RowAction>,
}

#[derive(Debug)]
pub struct AppState {
    runtime_snapshot: RuntimeSnapshot,
    last_error: Option<String>,
    pending_mark_read: Vec<SignalMutationInput>,
    main_visible: bool,
    widget_visible: bool,
}

impl AppState {
    pub fn new(runtime_snapshot: RuntimeSnapshot) -> Self {
        Self {
            runtime_snapshot,
            last_error: None,
            pending_mark_read: Vec::new(),
            main_visible: true,
            widget_visible: true,
        }
    }

    pub fn snapshot(&self) -> UiSnapshot {
        let enabled_groups = self
            .runtime_snapshot
            .config
            .groups
            .iter()
            .filter(|group| group.enabled)
            .count();
        let section_count = self
            .runtime_snapshot
            .signals
            .iter()
            .filter(|signal| !signal.deleted)
            .map(|signal| (signal.group_name.clone(), signal.signal_type.clone()))
            .collect::<std::collections::BTreeSet<_>>()
            .len();

        let status_text = if let Some(error) = &self.last_error {
            format!("轮询失败：{}", error)
        } else {
            format!("Tick {}", self.runtime_snapshot.last_tick)
        };

        let refresh_label = format!(
            "{} 个信号槽 · {} 个分组 · {} 秒轮询",
            self.runtime_snapshot.signals.len(),
            enabled_groups,
            self.runtime_snapshot.config.poll.interval_secs.max(1)
        );
        let save_hint = format!(
            "贴边宽度 {}px · 通知{} · 声音{}",
            self.runtime_snapshot.config.ui.edge_width.round() as i64,
            bool_label(self.runtime_snapshot.config.ui.notifications),
            bool_label(self.runtime_snapshot.config.ui.sound)
        );
        let connection = get_connection_state(
            self.runtime_snapshot.last_updated_at,
            self.runtime_snapshot.config.poll.interval_secs,
        );

        let config_location = crate::runtime::config_location_hint();
        let signal_rows = self
            .build_signal_row_views()
            .into_iter()
            .map(|entry| entry.row)
            .collect::<Vec<_>>();

        let runtime_summary = format!(
            "配置来源：{} · 最近更新(ms)：{} · API：{} · 信号数：{}",
            config_location,
            self.runtime_snapshot.last_updated_at,
            self.runtime_snapshot.config.api.base_url,
            self.runtime_snapshot.signals.len()
        );
        let stats_primary = format!("分组 {}", enabled_groups);
        let stats_secondary = format!("分区 {}", section_count);
        let stats_tertiary = format!(
            "Tick {} / {}s",
            self.runtime_snapshot.last_tick,
            self.runtime_snapshot.config.poll.interval_secs.max(1)
        );
        let stats_quaternary = format!(
            "最近 {}",
            format_timestamp(self.runtime_snapshot.last_updated_at)
        );

        UiSnapshot {
            unread_count: self.runtime_snapshot.unread_count as i32,
            status_text,
            refresh_label,
            runtime_summary,
            save_hint,
            connection_label: connection.label.to_string(),
            connection_tone: connection.tone.to_string(),
            stats_primary,
            stats_secondary,
            stats_tertiary,
            stats_quaternary,
            edge_width_label: format!(
                "{} px",
                self.runtime_snapshot.config.ui.edge_width.round() as i64
            ),
            signal_rows,
            main_visible: self.main_visible,
            widget_visible: self.widget_visible,
            always_on_top: self.runtime_snapshot.always_on_top,
            edge_mode: self.runtime_snapshot.edge_mode,
            notifications_enabled: self.runtime_snapshot.config.ui.notifications,
            sound_enabled: self.runtime_snapshot.config.ui.sound,
        }
    }

    pub fn always_on_top(&self) -> bool {
        self.runtime_snapshot.always_on_top
    }

    pub fn set_main_visible(&mut self, visible: bool) {
        self.main_visible = visible;
    }

    pub fn set_widget_visible(&mut self, visible: bool) {
        self.widget_visible = visible;
    }

    pub fn toggle_main_visible(&mut self) -> bool {
        self.main_visible = !self.main_visible;
        self.main_visible
    }

    pub fn toggle_widget_visible(&mut self) -> bool {
        self.widget_visible = !self.widget_visible;
        self.widget_visible
    }

    pub fn update_runtime_snapshot(&mut self, snapshot: RuntimeSnapshot) {
        self.runtime_snapshot = snapshot;
        self.last_error = None;
        self.pending_mark_read.clear();
    }

    pub fn set_runtime_error(&mut self, error: String) {
        self.last_error = Some(error);
        self.pending_mark_read.clear();
    }

    pub fn activate_row_at(&mut self, index: usize) -> Vec<SignalMutationInput> {
        let keys = self
            .build_signal_row_views()
            .get(index)
            .and_then(|entry| entry.action.clone())
            .map(|action| match action {
                RowAction::Group(keys) => keys,
                RowAction::Single { .. } => Vec::new(),
            })
            .unwrap_or_default();

        if keys.is_empty() {
            return keys;
        }

        self.pending_mark_read = keys.clone();
        self.mark_keys_read(&keys, true);
        keys
    }

    pub fn toggle_signal_row_at(&mut self, index: usize) -> Option<(SignalMutationInput, bool)> {
        let action = self
            .build_signal_row_views()
            .get(index)
            .and_then(|entry| entry.action.clone());

        let RowAction::Single { key, read } = action? else {
            return None;
        };

        self.pending_mark_read = vec![key.clone()];
        self.mark_keys_read(std::slice::from_ref(&key), read);
        Some((key, read))
    }

    fn build_signal_row_views(&self) -> Vec<SignalRowView> {
        let mut rows = Vec::new();
        for group in self
            .runtime_snapshot
            .config
            .groups
            .iter()
            .filter(|group| group.enabled)
        {
            for signal_type in &group.signal_types {
                let signals = group
                    .periods
                    .iter()
                    .filter_map(|period| {
                        self.runtime_snapshot
                            .signals
                            .iter()
                            .find(|signal| {
                                !signal.deleted
                                    && signal.group_id == group.id
                                    && signal.signal_type == *signal_type
                                    && signal.period == *period
                            })
                            .cloned()
                    })
                    .collect::<Vec<_>>();

                if signals.is_empty() {
                    continue;
                }

                rows.push(SignalRowView {
                    row: UiSignalRow {
                        title: signals
                            .first()
                            .map(|signal| signal.symbol.clone())
                            .unwrap_or_default(),
                        meta: format!("{}  {}", group.name, signal_type),
                        is_header: true,
                        unread: signals.iter().any(|signal| signal.unread),
                        pending: signals.iter().any(|signal| {
                            self.pending_mark_read
                                .iter()
                                .any(|pending| pending == &signal_to_key(signal))
                        }),
                        unread_count: signals.iter().filter(|signal| signal.unread).count() as i32,
                        timeline_visible: false,
                        timeline_ratio: 0.0,
                        timeline_positive: true,
                    },
                    action: {
                        let unread_keys = signals
                            .iter()
                            .filter(|signal| signal.unread)
                            .map(signal_to_key)
                            .collect::<Vec<_>>();
                        if unread_keys.is_empty() {
                            None
                        } else {
                            Some(RowAction::Group(unread_keys))
                        }
                    },
                });

                for signal in signals {
                    let key = signal_to_key(&signal);
                    let side = if signal.side >= 0 { "多" } else { "空" };
                    let timeline_ratio = timeline_marker_ratio(&signal, current_time_ms());

                    rows.push(SignalRowView {
                        row: UiSignalRow {
                            title: signal.period.clone(),
                            meta: format!("{} · {}", side, format_timestamp(signal.trigger_time)),
                            is_header: false,
                            unread: signal.unread,
                            pending: self.pending_mark_read.iter().any(|pending| pending == &key),
                            unread_count: 0,
                            timeline_visible: timeline_ratio.is_some(),
                            timeline_ratio: timeline_ratio.unwrap_or(0.0),
                            timeline_positive: signal.side >= 0,
                        },
                        action: Some(RowAction::Single {
                            key,
                            read: signal.unread,
                        }),
                    });
                }
            }
        }

        rows
    }

    fn mark_keys_read(&mut self, keys: &[SignalMutationInput], read: bool) {
        if keys.is_empty() {
            return;
        }

        self.runtime_snapshot.signals = self
            .runtime_snapshot
            .signals
            .iter()
            .cloned()
            .map(|mut signal| {
                if keys.iter().any(|key| {
                    key.group_id == signal.group_id
                        && key.signal_type == signal.signal_type
                        && key.period == signal.period
                }) {
                    signal.unread = !read;
                }
                signal
            })
            .collect();

        self.runtime_snapshot.unread_count = self
            .runtime_snapshot
            .signals
            .iter()
            .filter(|signal| signal.unread && !signal.deleted)
            .count();
    }
}

fn signal_to_key(signal: &RuntimeSignal) -> SignalMutationInput {
    SignalMutationInput {
        group_id: signal.group_id.clone(),
        signal_type: signal.signal_type.clone(),
        period: signal.period.clone(),
    }
}

fn timeline_marker_ratio(signal: &RuntimeSignal, now_ms: i64) -> Option<f32> {
    const CELL_COUNT: i64 = 60;

    if signal.trigger_time <= 0 {
        return None;
    }

    let period_ms = period_to_ms(&signal.period)?;
    if period_ms <= 0 {
        return None;
    }

    let elapsed_ms = now_ms.saturating_sub(signal.trigger_time).max(0);
    let candles_ago = elapsed_ms / period_ms;
    let active_index = CELL_COUNT - 1 - candles_ago;
    if !(0..CELL_COUNT).contains(&active_index) {
        return None;
    }

    Some(active_index as f32 / (CELL_COUNT - 1) as f32)
}

fn period_to_ms(period: &str) -> Option<i64> {
    match period {
        "W" => Some(7 * 24 * 60 * 60 * 1000),
        "D" => Some(24 * 60 * 60 * 1000),
        _ if period.ends_with('D') => period
            .trim_end_matches('D')
            .parse::<i64>()
            .ok()
            .map(|days| days * 24 * 60 * 60 * 1000),
        _ => period
            .parse::<i64>()
            .ok()
            .map(|minutes| minutes * 60 * 1000),
    }
}

fn format_timestamp(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "n/a".to_string();
    }

    let total_seconds = timestamp_ms / 1000;
    let seconds = total_seconds.rem_euclid(60);
    let minutes = (total_seconds / 60).rem_euclid(60);
    let hours = (total_seconds / 3600).rem_euclid(24);

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

struct ConnectionState<'a> {
    label: &'a str,
    tone: &'a str,
}

fn get_connection_state(last_updated_at: i64, interval_secs: u64) -> ConnectionState<'static> {
    let elapsed_ms = current_time_ms().saturating_sub(last_updated_at);
    let expected_ms = interval_secs.max(1) as i64 * 1000;

    if elapsed_ms <= expected_ms * 2 {
        ConnectionState {
            label: "连接正常",
            tone: "online",
        }
    } else if elapsed_ms <= expected_ms * 4 {
        ConnectionState {
            label: "连接延迟",
            tone: "lagging",
        }
    } else {
        ConnectionState {
            label: "连接超时",
            tone: "offline",
        }
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn bool_label(value: bool) -> &'static str {
    if value {
        "开"
    } else {
        "关"
    }
}
