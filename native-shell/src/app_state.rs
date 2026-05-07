use crate::runtime::{
    clamp_timeline_bars, RuntimeSignal, RuntimeSnapshot, SignalMutationInput,
    WatchGroupRowSortMode,
};
use chrono::{Local, TimeZone};

#[derive(Clone, Debug)]
pub struct UiSnapshot {
    pub unread_count: i32,
    pub unread_items: Vec<UiUnreadItem>,
    pub last_refresh_failed: bool,
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
    pub sort_label: String,
    pub timeline_bars: i32,
    pub sort_recent: bool,
    pub timeline_visible: bool,
    pub timeline_ratio: f32,
    pub timeline_positive: bool,
}

#[derive(Clone, Debug)]
pub struct UiUnreadItem {
    pub row_index: i32,
    pub symbol: String,
    pub period: String,
    pub meta: String,
    pub trigger_time: i64,
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
    unread_item: Option<UiUnreadItem>,
    section_key: Option<SignalSectionKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SignalSectionKey {
    group_id: String,
    signal_type: String,
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
        let last_refresh_failed = self.last_error.is_some();

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
        let connection = get_connection_state(self.runtime_snapshot.last_connection_ok);

        let config_location = crate::runtime::config_location_hint();
        let row_views = self.build_signal_row_views();
        let signal_rows = row_views.iter().map(|entry| entry.row.clone()).collect::<Vec<_>>();
        let mut unread_items = row_views
            .iter()
            .filter_map(|entry| entry.unread_item.clone())
            .collect::<Vec<_>>();
        unread_items.sort_by(|left, right| {
            right
                .trigger_time
                .cmp(&left.trigger_time)
                .then_with(|| left.row_index.cmp(&right.row_index))
        });

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
            unread_items,
            last_refresh_failed,
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

    pub fn toggle_signal_row_sort_mode_at(&mut self, index: usize) {
        let Some(section_key) = self
            .build_signal_row_views()
            .get(index)
            .and_then(|entry| entry.section_key.clone())
        else {
            return;
        };

        let next_mode = match self.sort_mode_for_section(&section_key) {
            WatchGroupRowSortMode::ConfigOrder => WatchGroupRowSortMode::RecentFirst,
            WatchGroupRowSortMode::RecentFirst => WatchGroupRowSortMode::ConfigOrder,
        };

        if let Some(group) = self
            .runtime_snapshot
            .config
            .groups
            .iter_mut()
            .find(|group| group.id == section_key.group_id)
        {
            group.row_sort_mode = next_mode;
        }
    }

    pub fn adjust_signal_timeline_bars_at(&mut self, index: usize, delta: i64) {
        let Some(section_key) = self
            .build_signal_row_views()
            .get(index)
            .and_then(|entry| entry.section_key.clone())
        else {
            return;
        };

        if let Some(group) = self
            .runtime_snapshot
            .config
            .groups
            .iter_mut()
            .find(|group| group.id == section_key.group_id)
        {
            group.timeline_bars = clamp_timeline_bars(group.timeline_bars + delta);
        }
    }

    pub fn set_signal_timeline_bars_at(&mut self, index: usize, timeline_bars: i64) {
        let Some(section_key) = self
            .build_signal_row_views()
            .get(index)
            .and_then(|entry| entry.section_key.clone())
        else {
            return;
        };

        if let Some(group) = self
            .runtime_snapshot
            .config
            .groups
            .iter_mut()
            .find(|group| group.id == section_key.group_id)
        {
            group.timeline_bars = clamp_timeline_bars(timeline_bars);
        }
    }

    pub fn group_display_settings_at(
        &self,
        index: usize,
    ) -> Option<(String, WatchGroupRowSortMode, i64)> {
        let section_key = self
            .build_signal_row_views()
            .get(index)
            .and_then(|entry| entry.section_key.clone())?;
        let group = self
            .runtime_snapshot
            .config
            .groups
            .iter()
            .find(|group| group.id == section_key.group_id)?;

        Some((
            group.id.clone(),
            group.row_sort_mode,
            clamp_timeline_bars(group.timeline_bars),
        ))
    }

    pub fn update_runtime_snapshot(&mut self, snapshot: RuntimeSnapshot) {
        self.runtime_snapshot = snapshot;
        self.last_error = None;
        self.pending_mark_read.clear();
    }

    pub fn set_runtime_error(&mut self, snapshot: RuntimeSnapshot, error: String) {
        self.runtime_snapshot = snapshot;
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
        let now_ms = current_time_ms();
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
                let section_key = SignalSectionKey {
                    group_id: group.id.clone(),
                    signal_type: signal_type.clone(),
                };
                let mut signals = signals
                    .into_iter()
                    .enumerate()
                    .collect::<Vec<_>>();

                let sort_mode = self.sort_mode_for_section(&section_key);
                let timeline_bars = self.timeline_bars_for_section(&section_key);

                if matches!(sort_mode, WatchGroupRowSortMode::RecentFirst) {
                    signals.sort_by(|(left_index, left_signal), (right_index, right_signal)| {
                        compare_signal_recency(left_signal, right_signal, now_ms, timeline_bars)
                            .then_with(|| left_index.cmp(right_index))
                    });
                }

                rows.push(SignalRowView {
                    row: UiSignalRow {
                        title: signals
                            .first()
                            .map(|(_, signal)| signal.symbol.clone())
                            .unwrap_or_default(),
                        meta: format!("{}  {}", group.name, signal_type),
                        is_header: true,
                        unread: signals.iter().any(|(_, signal)| signal.unread),
                        pending: signals.iter().any(|(_, signal)| {
                            self.pending_mark_read
                                .iter()
                                .any(|pending| pending == &signal_to_key(signal))
                        }),
                        unread_count: signals
                            .iter()
                            .filter(|(_, signal)| signal.unread)
                            .count() as i32,
                        sort_label: format!("{}·{} ▾", sort_mode.short_label(), timeline_bars),
                        timeline_bars: timeline_bars as i32,
                        sort_recent: matches!(sort_mode, WatchGroupRowSortMode::RecentFirst),
                        timeline_visible: false,
                        timeline_ratio: 0.0,
                        timeline_positive: true,
                    },
                    action: {
                        let unread_keys = signals
                            .iter()
                            .filter(|(_, signal)| signal.unread)
                            .map(|(_, signal)| signal_to_key(signal))
                            .collect::<Vec<_>>();
                        if unread_keys.is_empty() {
                            None
                        } else {
                            Some(RowAction::Group(unread_keys))
                        }
                    },
                    unread_item: None,
                    section_key: Some(section_key),
                });

                for (_, signal) in signals {
                    let key = signal_to_key(&signal);
                    let side = if signal.side >= 0 { "多" } else { "空" };
                    let timeline_ratio = timeline_marker_ratio(&signal, now_ms, timeline_bars);
                    let row_index = rows.len() as i32;

                    rows.push(SignalRowView {
                        row: UiSignalRow {
                            title: signal.period.clone(),
                            meta: format!("{} · {}", side, format_timestamp(signal.trigger_time)),
                            is_header: false,
                            unread: signal.unread,
                            pending: self.pending_mark_read.iter().any(|pending| pending == &key),
                            unread_count: 0,
                            sort_label: String::new(),
                            timeline_bars: 0,
                            sort_recent: false,
                            timeline_visible: timeline_ratio.is_some(),
                            timeline_ratio: timeline_ratio.unwrap_or(0.0),
                            timeline_positive: signal.side >= 0,
                        },
                        action: Some(RowAction::Single {
                            key,
                            read: signal.unread,
                        }),
                        unread_item: signal.unread.then(|| UiUnreadItem {
                            row_index,
                            symbol: signal.symbol.clone(),
                            period: signal.period.clone(),
                            meta: format!(
                                "{} · {} · {} · {}",
                                group.name,
                                signal.signal_type,
                                side,
                                format_timestamp(signal.trigger_time)
                            ),
                            trigger_time: signal.trigger_time,
                        }),
                        section_key: None,
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

    fn sort_mode_for_section(&self, section_key: &SignalSectionKey) -> WatchGroupRowSortMode {
        self.runtime_snapshot
            .config
            .groups
            .iter()
            .find(|group| group.id == section_key.group_id)
            .map(|group| group.row_sort_mode)
            .unwrap_or_default()
    }

    fn timeline_bars_for_section(&self, section_key: &SignalSectionKey) -> i64 {
        self.runtime_snapshot
            .config
            .groups
            .iter()
            .find(|group| group.id == section_key.group_id)
            .map(|group| clamp_timeline_bars(group.timeline_bars))
            .unwrap_or(60)
    }
}

fn signal_to_key(signal: &RuntimeSignal) -> SignalMutationInput {
    SignalMutationInput {
        group_id: signal.group_id.clone(),
        signal_type: signal.signal_type.clone(),
        period: signal.period.clone(),
    }
}

fn timeline_marker_ratio(signal: &RuntimeSignal, now_ms: i64, cell_count: i64) -> Option<f32> {
    if signal.trigger_time <= 0 {
        return None;
    }

    let cell_count = clamp_timeline_bars(cell_count).max(2);
    let period_ms = period_to_ms(&signal.period)?;
    if period_ms <= 0 {
        return None;
    }

    let elapsed_ms = now_ms.saturating_sub(signal.trigger_time).max(0);
    let candles_ago = elapsed_ms / period_ms;
    let active_index = cell_count - 1 - candles_ago;
    if !(0..cell_count).contains(&active_index) {
        return None;
    }

    Some(active_index as f32 / (cell_count - 1) as f32)
}

fn period_to_ms(period: &str) -> Option<i64> {
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
}

fn format_timestamp(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "n/a".to_string();
    }

    Local
        .timestamp_millis_opt(timestamp_ms)
        .single()
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

struct ConnectionState<'a> {
    label: &'a str,
    tone: &'a str,
}

fn get_connection_state(last_connection_ok: Option<bool>) -> ConnectionState<'static> {
    match last_connection_ok {
        Some(true) => ConnectionState {
            label: "连接正常",
            tone: "online",
        },
        Some(false) => ConnectionState {
            label: "连接失败",
            tone: "offline",
        },
        None => ConnectionState {
            label: "未连接",
            tone: "lagging",
        },
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

impl WatchGroupRowSortMode {
    fn short_label(self) -> &'static str {
        match self {
            WatchGroupRowSortMode::ConfigOrder => "配",
            WatchGroupRowSortMode::RecentFirst => "近",
        }
    }
}

fn compare_signal_recency(
    left: &RuntimeSignal,
    right: &RuntimeSignal,
    now_ms: i64,
    timeline_bars: i64,
) -> std::cmp::Ordering {
    match (
        timeline_marker_ratio(left, now_ms, timeline_bars),
        timeline_marker_ratio(right, now_ms, timeline_bars),
    ) {
        (Some(left_ratio), Some(right_ratio)) => right_ratio
            .partial_cmp(&left_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.trigger_time.cmp(&left.trigger_time)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::{format_timestamp, period_to_ms};
    use chrono::{Local, TimeZone};

    #[test]
    fn format_timestamp_uses_local_timezone() {
        let timestamp_ms = 1_710_000_000_000i64;
        let expected = Local
            .timestamp_millis_opt(timestamp_ms)
            .single()
            .expect("valid local timestamp")
            .format("%H:%M:%S")
            .to_string();

        assert_eq!(format_timestamp(timestamp_ms), expected);
    }

    #[test]
    fn period_to_ms_supports_second_periods() {
        assert_eq!(period_to_ms("45S"), Some(45_000));
        assert_eq!(period_to_ms("30s"), Some(30_000));
        assert_eq!(period_to_ms(" 15S "), Some(15_000));
    }
}
