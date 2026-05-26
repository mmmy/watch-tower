use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;

use serde::Serialize;

use crate::notifications::SignalAlert;
use crate::runtime::HookConfig;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HookPayload {
    event: &'static str,
    triggered_at: i64,
    alerts: Vec<SignalAlert>,
}

pub fn emit_new_signal_hook(alerts: &[SignalAlert], config: &HookConfig) {
    if alerts.is_empty() || !config.enabled || config.command.trim().is_empty() {
        return;
    }

    let payload = HookPayload {
        event: "new_signals",
        triggered_at: now_ms(),
        alerts: alerts.to_vec(),
    };
    let Ok(payload) = serde_json::to_vec(&payload) else {
        return;
    };

    let command = config.command.clone();
    let args = config.args.clone();
    let _ = thread::Builder::new()
        .name("watch-tower-hook".into())
        .spawn(move || {
            let Ok(mut child) = Command::new(command)
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            else {
                return;
            };

            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&payload);
            }
        });
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::emit_new_signal_hook;
    use crate::notifications::{AlertLevel, SignalAlert};
    use crate::runtime::HookConfig;

    fn alert() -> SignalAlert {
        SignalAlert {
            symbol: "BTCUSDT".into(),
            group_name: "BTC Main".into(),
            period: "15".into(),
            signal_type: "divMacd".into(),
            side: 1,
            trigger_time: 123,
            level: AlertLevel::Normal,
        }
    }

    #[test]
    fn disabled_hook_does_not_panic() {
        emit_new_signal_hook(
            &[alert()],
            &HookConfig {
                enabled: false,
                command: "missing-command".into(),
                args: Vec::new(),
            },
        );
    }

    #[test]
    fn empty_alerts_do_not_trigger_hook() {
        emit_new_signal_hook(
            &[],
            &HookConfig {
                enabled: true,
                command: "missing-command".into(),
                args: Vec::new(),
            },
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn enabled_hook_spawns_command_with_json_payload() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let output_path = std::env::temp_dir().join(format!("watch-tower-hook-{nonce}.json"));

        emit_new_signal_hook(
            &[alert()],
            &HookConfig {
                enabled: true,
                command: "powershell".into(),
                args: vec![
                    "-NoProfile".into(),
                    "-NonInteractive".into(),
                    "-Command".into(),
                    format!(
                        "$data = [Console]::In.ReadToEnd(); [IO.File]::WriteAllText('{}', $data)",
                        output_path.display().to_string().replace('\'', "''")
                    ),
                ],
            },
        );

        let mut content = String::new();
        for _ in 0..40 {
            if let Ok(value) = fs::read_to_string(&output_path) {
                content = value;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        let _ = fs::remove_file(&output_path);
        assert!(content.contains("\"event\":\"new_signals\""));
        assert!(content.contains("\"symbol\":\"BTCUSDT\""));
        assert!(content.contains("\"groupName\":\"BTC Main\""));
        assert!(content.contains("\"triggerTime\":123"));
    }
}
