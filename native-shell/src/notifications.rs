use crate::runtime::{RuntimeSignal, UiConfig};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AlertLevel {
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalAlert {
    symbol: String,
    group_name: String,
    period: String,
    signal_type: String,
    side: i8,
    trigger_time: i64,
    level: AlertLevel,
}

pub fn collect_new_alerts(previous: &[RuntimeSignal], next: &[RuntimeSignal]) -> Vec<SignalAlert> {
    let previous_by_key = previous
        .iter()
        .map(|signal| {
            (
                signal_key(signal),
                (signal.trigger_time, signal.unread, signal.deleted),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut alerts = next
        .iter()
        .filter(|signal| !signal.deleted && signal.unread && signal.trigger_time > 0)
        .filter_map(|signal| {
            let is_new = previous_by_key
                .get(&signal_key(signal))
                .map(|(trigger_time, unread, deleted)| {
                    *deleted || !*unread || *trigger_time != signal.trigger_time
                })
                .unwrap_or(true);

            if !is_new {
                return None;
            }

            Some(SignalAlert {
                symbol: signal.symbol.clone(),
                group_name: signal.group_name.clone(),
                period: signal.period.clone(),
                signal_type: signal.signal_type.clone(),
                side: signal.side,
                trigger_time: signal.trigger_time,
                level: alert_level_for_period(&signal.period),
            })
        })
        .collect::<Vec<_>>();

    alerts.sort_by(|left, right| {
        right
            .level
            .cmp(&left.level)
            .then_with(|| right.trigger_time.cmp(&left.trigger_time))
            .then_with(|| left.symbol.cmp(&right.symbol))
    });

    alerts
}

pub fn emit_alerts(alerts: &[SignalAlert], config: &UiConfig) {
    if alerts.is_empty() || (!config.notifications && !config.sound) {
        return;
    }

    #[cfg(target_os = "windows")]
    windows::emit_alerts(alerts, config);
}

fn signal_key(signal: &RuntimeSignal) -> (String, String, String) {
    (
        signal.group_id.clone(),
        signal.signal_type.clone(),
        signal.period.clone(),
    )
}

fn alert_level_for_period(period: &str) -> AlertLevel {
    let normalized = period.trim().to_ascii_uppercase();
    if normalized.ends_with('W')
        || normalized.ends_with('D')
        || normalized.ends_with('M')
    {
        return AlertLevel::Critical;
    }

    normalized
        .parse::<u64>()
        .ok()
        .map(|minutes| {
            if minutes >= 60 {
                AlertLevel::High
            } else {
                AlertLevel::Normal
            }
        })
        .unwrap_or(AlertLevel::High)
}

fn level_label(level: AlertLevel) -> &'static str {
    match level {
        AlertLevel::Normal => "普通",
        AlertLevel::High => "重点",
        AlertLevel::Critical => "高等级",
    }
}

fn signal_type_label(signal_type: &str) -> &str {
    match signal_type {
        "divMacd" => "MACD 背离",
        "divRsi" => "RSI 背离",
        other => other,
    }
}

fn side_label(side: i8) -> &'static str {
    if side >= 0 { "看多" } else { "看空" }
}

fn format_time(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "n/a".to_string();
    }

    let total_seconds = timestamp_ms / 1000;
    let seconds = total_seconds.rem_euclid(60);
    let minutes = (total_seconds / 60).rem_euclid(60);
    let hours = (total_seconds / 3600).rem_euclid(24);

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(target_os = "windows")]
mod windows {
    use super::{
        format_time, level_label, side_label, signal_type_label, AlertLevel, SignalAlert,
    };
    use crate::runtime::UiConfig;
    use std::process::Command;
    use std::sync::OnceLock;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MessageBeep, MB_ICONASTERISK, MB_ICONEXCLAMATION, MB_ICONHAND,
    };

    const APP_ID: &str = "WatchTower.SignalDesk";
    const APP_NAME: &str = "Watch Tower";
    static REGISTRATION: OnceLock<Result<(), String>> = OnceLock::new();

    pub fn emit_alerts(alerts: &[SignalAlert], config: &UiConfig) {
        if config.notifications {
            if let Err(error) = ensure_registered() {
                eprintln!("failed to register Windows toast app id: {error}");
            } else {
                for alert in alerts.iter().take(4) {
                    if let Err(error) = show_toast(alert) {
                        eprintln!("failed to show Windows toast: {error}");
                    }
                }
            }
        }

        if config.sound {
            play_level_beep(
                alerts
                    .iter()
                    .map(|alert| alert.level)
                    .max()
                    .unwrap_or(AlertLevel::Normal),
            );
        }
    }

    fn ensure_registered() -> Result<(), String> {
        REGISTRATION
            .get_or_init(|| winrt_toast::register(APP_ID, APP_NAME, None).map_err(|err| err.to_string()))
            .clone()
    }

    fn show_toast(alert: &SignalAlert) -> Result<(), String> {
        let scenario = scenario_attr(alert.level);
        let title = xml_escape(&format!(
            "{}信号 · {} · {} · {}",
            level_label(alert.level),
            alert.symbol,
            alert.period,
            side_label(alert.side)
        ));
        let body = xml_escape(&format!(
            "{} · {}",
            alert.group_name,
            signal_type_label(&alert.signal_type)
        ));
        let footer = xml_escape(&format!("触发时间 {} · 周期 {}", format_time(alert.trigger_time), alert.period));
        let tag = xml_escape(&format!(
            "{}-{}-{}",
            alert.symbol, alert.period, alert.signal_type
        ));
        let xml = format!(
            "<toast{}>\
                <visual>\
                    <binding template='ToastGeneric'>\
                        <text>{}</text>\
                        <text>{}</text>\
                        <text hint-style='captionSubtle'>{}</text>\
                    </binding>\
                </visual>\
                <actions>\
                    <action content='忽略' activationType='system' arguments='dismiss'/>\
                </actions>\
                <audio silent='true'/>\
            </toast>",
            scenario, title, body, footer
        );

        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null\n\
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] > $null\n\
$xml = @'\n{}\n'@\n\
$doc = New-Object Windows.Data.Xml.Dom.XmlDocument\n\
$doc.LoadXml($xml)\n\
$toast = [Windows.UI.Notifications.ToastNotification]::new($doc)\n\
$toast.Tag = '{}'\n\
$toast.Group = 'signal-desk'\n\
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('{}').Show($toast)\n",
            xml, tag, APP_ID
        );

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-Command",
                &script,
            ])
            .output()
            .map_err(|err| err.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    fn scenario_attr(level: AlertLevel) -> &'static str {
        match level {
            AlertLevel::Normal => "",
            AlertLevel::High => " scenario='reminder'",
            AlertLevel::Critical => " scenario='alarm'",
        }
    }

    fn play_level_beep(level: AlertLevel) {
        let tone = match level {
            AlertLevel::Normal => MB_ICONASTERISK,
            AlertLevel::High => MB_ICONEXCLAMATION,
            AlertLevel::Critical => MB_ICONHAND,
        };

        unsafe {
            MessageBeep(tone);
        }
    }

    fn xml_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::{alert_level_for_period, collect_new_alerts, AlertLevel};
    use crate::runtime::RuntimeSignal;

    fn signal(period: &str, trigger_time: i64, unread: bool) -> RuntimeSignal {
        RuntimeSignal {
            group_id: "group-1".into(),
            group_name: "BTC Main".into(),
            symbol: "BTCUSDT".into(),
            period: period.into(),
            signal_type: "divMacd".into(),
            side: 1,
            trigger_time,
            unread,
            deleted: false,
        }
    }

    #[test]
    fn collect_new_alerts_only_returns_fresh_unread_signals() {
        let previous = vec![signal("15", 100, true), signal("60", 200, false)];
        let next = vec![signal("15", 100, true), signal("60", 200, true), signal("5", 300, true)];

        let alerts = collect_new_alerts(&previous, &next);

        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].period, "60");
        assert_eq!(alerts[1].period, "5");
    }

    #[test]
    fn collect_new_alerts_sorts_by_level_then_time() {
        let previous = vec![signal("15", 100, false), signal("10D", 200, false)];
        let next = vec![signal("15", 300, true), signal("10D", 250, true)];

        let alerts = collect_new_alerts(&previous, &next);

        assert_eq!(alerts[0].period, "10D");
        assert_eq!(alerts[1].period, "15");
    }

    #[test]
    fn alert_levels_follow_period_size() {
        assert_eq!(alert_level_for_period("5"), AlertLevel::Normal);
        assert_eq!(alert_level_for_period("60"), AlertLevel::High);
        assert_eq!(alert_level_for_period("10D"), AlertLevel::Critical);
        assert_eq!(alert_level_for_period("W"), AlertLevel::Critical);
    }
}
