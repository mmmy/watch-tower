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
}

#[derive(Clone, Debug)]
enum RowAction {
    Single(SignalMutationInput),
    Group(Vec<SignalMutationInput>),
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
                RowAction::Single(key) => vec![key],
                RowAction::Group(keys) => keys,
            })
            .unwrap_or_default();

        if keys.is_empty() {
            return keys;
        }

        self.pending_mark_read = keys.clone();
        self.mark_keys_read(&keys, true);
        keys
    }

    fn sorted_live_signals(&self) -> Vec<RuntimeSignal> {
        let mut live_signals = self
            .runtime_snapshot
            .signals
            .iter()
            .filter(|signal| !signal.deleted)
            .cloned()
            .collect::<Vec<_>>();

        live_signals.sort_by(|left, right| {
            left.group_name
                .cmp(&right.group_name)
                .then_with(|| left.signal_type.cmp(&right.signal_type))
                .then_with(|| right.unread.cmp(&left.unread))
                .then_with(|| right.trigger_time.cmp(&left.trigger_time))
                .then_with(|| left.symbol.cmp(&right.symbol))
                .then_with(|| left.period.cmp(&right.period))
        });

        live_signals
    }

    fn build_signal_row_views(&self) -> Vec<SignalRowView> {
        let mut sections: Vec<((String, String), Vec<RuntimeSignal>)> = Vec::new();
        let mut current_key: Option<(String, String)> = None;

        for signal in self.sorted_live_signals() {
            let group_key = (signal.group_name.clone(), signal.signal_type.clone());
            if current_key.as_ref() == Some(&group_key) {
                if let Some((_, rows)) = sections.last_mut() {
                    rows.push(signal);
                    continue;
                }
            }
            sections.push((group_key, vec![signal]));
            current_key = Some(sections.last().expect("section inserted").0.clone());
        }

        let mut rows = Vec::new();
        for ((group_name, signal_type), signals) in sections {
            let section_keys = signals.iter().map(signal_to_key).collect::<Vec<_>>();
            let unread_keys = signals
                .iter()
                .filter(|signal| signal.unread)
                .map(signal_to_key)
                .collect::<Vec<_>>();
            let pending_group = section_keys
                .iter()
                .any(|key| self.pending_mark_read.iter().any(|pending| pending == key));

            rows.push(SignalRowView {
                row: UiSignalRow {
                    title: signals
                        .first()
                        .map(|signal| signal.symbol.clone())
                        .unwrap_or_default(),
                    meta: format!("{}  {}", group_name, signal_type),
                    is_header: true,
                    unread: unread_keys.len() > 0,
                    pending: pending_group,
                    unread_count: unread_keys.len() as i32,
                },
                action: if unread_keys.is_empty() {
                    None
                } else {
                    Some(RowAction::Group(unread_keys))
                },
            });

            for signal in signals {
                let key = signal_to_key(&signal);
                let read_state = if signal.unread { "未读" } else { "已读" };
                let side = if signal.side >= 0 { "多" } else { "空" };

                rows.push(SignalRowView {
                    row: UiSignalRow {
                        title: signal.period.clone(),
                        meta: format!(
                            "{} · {} · {}",
                            read_state,
                            side,
                            format_timestamp(signal.trigger_time)
                        ),
                        is_header: false,
                        unread: signal.unread,
                        pending: self.pending_mark_read.iter().any(|pending| pending == &key),
                        unread_count: 0,
                    },
                    action: if signal.unread {
                        Some(RowAction::Single(key))
                    } else {
                        None
                    },
                });
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
