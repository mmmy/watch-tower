use crate::runtime::{RuntimeSignal, RuntimeSnapshot, SignalMutationInput};

#[derive(Clone, Debug)]
pub struct UiSnapshot {
    pub unread_count: i32,
    pub status_text: String,
    pub refresh_label: String,
    pub runtime_summary: String,
    pub stats_primary: String,
    pub stats_secondary: String,
    pub stats_tertiary: String,
    pub stats_quaternary: String,
    pub signal_rows: Vec<UiSignalRow>,
    pub main_visible: bool,
    pub widget_visible: bool,
}

#[derive(Clone, Debug)]
pub struct UiSignalRow {
    pub title: String,
    pub meta: String,
    pub is_header: bool,
    pub unread: bool,
    pub pending: bool,
}

#[derive(Clone, Debug)]
struct SignalRowView {
    row: UiSignalRow,
    key: Option<SignalMutationInput>,
}

#[derive(Debug)]
pub struct AppState {
    runtime_snapshot: RuntimeSnapshot,
    last_error: Option<String>,
    pending_mark_read: Option<SignalMutationInput>,
    main_visible: bool,
    widget_visible: bool,
}

impl AppState {
    pub fn new(runtime_snapshot: RuntimeSnapshot) -> Self {
        Self {
            runtime_snapshot,
            last_error: None,
            pending_mark_read: None,
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
            format!("Polling failed: {}", error)
        } else {
            format!(
                "Polling live. Tick {} with {} unread signal(s)",
                self.runtime_snapshot.last_tick, self.runtime_snapshot.unread_count
            )
        };

        let refresh_label = format!(
            "{} tracked signal slot(s) across {} group(s) at {}s polling",
            self.runtime_snapshot.signals.len(),
            enabled_groups,
            self.runtime_snapshot.config.poll.interval_secs.max(1)
        );

        let config_location = crate::runtime::config_location_hint();
        let signal_rows = self
            .build_signal_row_views()
            .into_iter()
            .map(|entry| entry.row)
            .collect::<Vec<_>>();

        let runtime_summary = format!(
            "Config source: {} | Last update(ms): {} | API: {} | Signals: {}",
            config_location,
            self.runtime_snapshot.last_updated_at,
            self.runtime_snapshot.config.api.base_url,
            self.runtime_snapshot.signals.len()
        );
        let stats_primary = format!("{} enabled group(s)", enabled_groups);
        let stats_secondary = format!("{} section(s)", section_count);
        let stats_tertiary = format!(
            "Tick {} @ {}s poll",
            self.runtime_snapshot.last_tick,
            self.runtime_snapshot.config.poll.interval_secs.max(1)
        );
        let stats_quaternary = format!(
            "Updated {}",
            format_timestamp(self.runtime_snapshot.last_updated_at)
        );

        UiSnapshot {
            unread_count: self.runtime_snapshot.unread_count as i32,
            status_text,
            refresh_label,
            runtime_summary,
            stats_primary,
            stats_secondary,
            stats_tertiary,
            stats_quaternary,
            signal_rows,
            main_visible: self.main_visible,
            widget_visible: self.widget_visible,
        }
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
        self.pending_mark_read = None;
    }

    pub fn set_runtime_error(&mut self, error: String) {
        self.last_error = Some(error);
        self.pending_mark_read = None;
    }

    pub fn signal_key_at(&self, index: usize) -> Option<SignalMutationInput> {
        self.build_signal_row_views()
            .get(index)
            .and_then(|entry| entry.key.clone())
    }

    pub fn set_pending_mark_read(&mut self, signal: SignalMutationInput) {
        self.pending_mark_read = Some(signal);
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
        let mut rows = Vec::new();
        let mut current_group: Option<(String, String)> = None;

        for signal in self.sorted_live_signals() {
            let group_key = (signal.group_name.clone(), signal.signal_type.clone());
            if current_group.as_ref() != Some(&group_key) {
                rows.push(SignalRowView {
                    row: UiSignalRow {
                        title: format!("{} / {}", signal.group_name, signal.signal_type),
                        meta: signal.symbol.clone(),
                        is_header: true,
                        unread: false,
                        pending: false,
                    },
                    key: None,
                });
                current_group = Some(group_key);
            }

            let read_state = if signal.unread { "UNREAD" } else { "READ" };
            let side = if signal.side >= 0 { "LONG" } else { "SHORT" };
            let key = SignalMutationInput {
                group_id: signal.group_id.clone(),
                signal_type: signal.signal_type.clone(),
                period: signal.period.clone(),
            };

            rows.push(SignalRowView {
                row: UiSignalRow {
                    title: format!("{} {}", signal.symbol, signal.period),
                    meta: format!(
                        "{} · {} · {}",
                        read_state,
                        side,
                        format_timestamp(signal.trigger_time)
                    ),
                    is_header: false,
                    unread: signal.unread,
                    pending: self.pending_mark_read.as_ref() == Some(&key),
                },
                key: Some(key),
            });
        }

        rows
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
    let days = total_seconds / 86_400;

    format!("d{} {:02}:{:02}:{:02}", days, hours, minutes, seconds)
}
