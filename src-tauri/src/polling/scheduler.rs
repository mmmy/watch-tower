use crate::app_state::{
    AppSnapshot, DiagnosticsInfo, PollingHealth, SharedAppState, SnapshotEventPayload,
    APP_SNAPSHOT_EVENT,
};
use crate::polling::alerts_client::{self, FetchError};
use crate::polling::backoff::{clamp_polling_interval, compute_backoff_until, sleep_duration_ms};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

pub fn spawn(app: AppHandle, state: SharedAppState) {
    tauri::async_runtime::spawn(async move {
        loop {
            let current_config = state.current_config().await;

            if current_config.is_none() {
                state.wait_for_wake().await;
                continue;
            }

            let _ = poll_once(&app, state.clone()).await;

            let snapshot = state.current_snapshot().await;
            let now_ms = now();
            let polling_interval_seconds = snapshot
                .config
                .as_ref()
                .map(|config| config.polling_interval_seconds)
                .unwrap_or(60);
            let sleep_for = sleep_duration_ms(
                polling_interval_seconds,
                snapshot.diagnostics.next_retry_at,
                now_ms,
            );

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(sleep_for.max(250))) => {}
                _ = state.wait_for_wake() => {}
            }
        }
    });
}

pub async fn poll_once(app: &AppHandle, state: SharedAppState) -> Result<AppSnapshot, String> {
    let _guard = state.poll_lock().lock_owned().await;
    let config = match state.current_config().await {
        Some(config) => config,
        None => {
            let snapshot = state.current_snapshot().await;
            emit_snapshot(app, &snapshot)?;
            return Ok(snapshot);
        }
    };

    let attempt_started_at = now();

    if config.groups.is_empty() {
        let current_snapshot = state.current_snapshot().await;
        let next_snapshot = build_empty_groups_snapshot(&current_snapshot, &config, attempt_started_at);
        state.replace_snapshot(next_snapshot.clone()).await;
        emit_snapshot(app, &next_snapshot)?;
        return Ok(next_snapshot);
    }

    let polling_snapshot = state
        .update_with(|snapshot| {
            snapshot.bootstrap_required = false;
            snapshot.health = PollingHealth {
                status: "polling".into(),
                polling_interval_seconds: Some(clamp_polling_interval(
                    config.polling_interval_seconds,
                )),
                is_stale: snapshot.raw_response.is_some(),
            };
            snapshot.diagnostics.last_attempt_at = Some(attempt_started_at);
            snapshot.diagnostics.source = "request".into();
            snapshot.diagnostics.code = Some("POLLING".into());
            snapshot.diagnostics.message = "Polling the watch-list signal endpoint.".into();
        })
        .await;
    emit_snapshot(app, &polling_snapshot)?;

    let next_snapshot = match alerts_client::fetch_signals(&state.http_client, &config).await {
        Ok(response) => AppSnapshot {
            bootstrap_required: false,
            config: Some(config.clone()),
            raw_response: Some(response),
            health: PollingHealth {
                status: "success".into(),
                polling_interval_seconds: Some(clamp_polling_interval(
                    config.polling_interval_seconds,
                )),
                is_stale: false,
            },
            diagnostics: DiagnosticsInfo {
                source: "system".into(),
                code: Some("SYNC_OK".into()),
                message: "Latest signal snapshot loaded successfully.".into(),
                last_attempt_at: Some(attempt_started_at),
                last_successful_sync_at: Some(attempt_started_at),
                next_retry_at: None,
            },
        },
        Err(FetchError::Unauthorized) => {
            state
                .update_with(|snapshot| {
                    snapshot.health = PollingHealth {
                        status: "authError".into(),
                        polling_interval_seconds: Some(clamp_polling_interval(
                            config.polling_interval_seconds,
                        )),
                        is_stale: snapshot.raw_response.is_some(),
                    };
                    snapshot.diagnostics = DiagnosticsInfo {
                        source: "request".into(),
                        code: Some("HTTP_401".into()),
                        message: "API key was rejected by the server.".into(),
                        last_attempt_at: Some(attempt_started_at),
                        last_successful_sync_at: snapshot.diagnostics.last_successful_sync_at,
                        next_retry_at: None,
                    };
                })
                .await
        }
        Err(FetchError::Backoff(status_code)) => {
            let retry_at = compute_backoff_until(attempt_started_at);
            state
                .update_with(|snapshot| {
                    snapshot.health = PollingHealth {
                        status: "backoff".into(),
                        polling_interval_seconds: Some(clamp_polling_interval(
                            config.polling_interval_seconds,
                        )),
                        is_stale: snapshot.raw_response.is_some(),
                    };
                    snapshot.diagnostics = DiagnosticsInfo {
                        source: "request".into(),
                        code: Some(format!("HTTP_{status_code}")),
                        message:
                            "Server asked the client to back off. Keeping the latest good snapshot."
                                .into(),
                        last_attempt_at: Some(attempt_started_at),
                        last_successful_sync_at: snapshot.diagnostics.last_successful_sync_at,
                        next_retry_at: Some(retry_at),
                    };
                })
                .await
        }
        Err(FetchError::Http(status_code)) => {
            state
                .update_with(|snapshot| {
                    snapshot.health = PollingHealth {
                        status: "requestError".into(),
                        polling_interval_seconds: Some(clamp_polling_interval(
                            config.polling_interval_seconds,
                        )),
                        is_stale: snapshot.raw_response.is_some(),
                    };
                    snapshot.diagnostics = DiagnosticsInfo {
                        source: "request".into(),
                        code: Some(format!("HTTP_{status_code}")),
                        message: format!(
                            "Unexpected HTTP status {status_code} from signal endpoint."
                        ),
                        last_attempt_at: Some(attempt_started_at),
                        last_successful_sync_at: snapshot.diagnostics.last_successful_sync_at,
                        next_retry_at: None,
                    };
                })
                .await
        }
        Err(FetchError::Network(message)) | Err(FetchError::Deserialize(message)) => {
            state
                .update_with(|snapshot| {
                    snapshot.health = PollingHealth {
                        status: "requestError".into(),
                        polling_interval_seconds: Some(clamp_polling_interval(
                            config.polling_interval_seconds,
                        )),
                        is_stale: snapshot.raw_response.is_some(),
                    };
                    snapshot.diagnostics = DiagnosticsInfo {
                        source: "request".into(),
                        code: Some("REQUEST_FAILURE".into()),
                        message,
                        last_attempt_at: Some(attempt_started_at),
                        last_successful_sync_at: snapshot.diagnostics.last_successful_sync_at,
                        next_retry_at: None,
                    };
                })
                .await
        }
    };

    state.replace_snapshot(next_snapshot.clone()).await;
    emit_snapshot(app, &next_snapshot)?;

    Ok(next_snapshot)
}

pub fn emit_snapshot(app: &AppHandle, snapshot: &AppSnapshot) -> Result<(), String> {
    app.emit(
        APP_SNAPSHOT_EVENT,
        SnapshotEventPayload {
            snapshot: snapshot.clone(),
        },
    )
    .map_err(|error| error.to_string())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time drift")
        .as_millis() as u64
}

fn build_empty_groups_snapshot(
    current_snapshot: &AppSnapshot,
    config: &crate::app_state::AppConfig,
    attempt_started_at: u64,
) -> AppSnapshot {
    AppSnapshot {
        bootstrap_required: false,
        config: Some(config.clone()),
        raw_response: current_snapshot.raw_response.clone(),
        health: PollingHealth {
            status: "configError".into(),
            polling_interval_seconds: Some(clamp_polling_interval(config.polling_interval_seconds)),
            is_stale: current_snapshot.raw_response.is_some(),
        },
        diagnostics: DiagnosticsInfo {
            source: "config".into(),
            code: Some("NO_GROUPS".into()),
            message: "Add at least one watch group to resume polling.".into(),
            last_attempt_at: Some(attempt_started_at),
            last_successful_sync_at: current_snapshot.diagnostics.last_successful_sync_at,
            next_retry_at: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{build_empty_groups_snapshot, sleep_duration_ms};
    use crate::app_state::{
        AppConfig, AppSnapshot, DashboardPreferences, DiagnosticsInfo, PollingHealth,
        WindowPolicyConfig,
    };

    #[test]
    fn prefers_backoff_retry_over_interval() {
        assert_eq!(sleep_duration_ms(60, Some(9_000), 4_000), 5_000);
    }

    #[test]
    fn empty_group_configs_surface_a_config_error_without_polling() {
        let current_snapshot = AppSnapshot {
            bootstrap_required: false,
            config: None,
            raw_response: None,
            health: PollingHealth {
                status: "idle".into(),
                polling_interval_seconds: Some(60),
                is_stale: false,
            },
            diagnostics: DiagnosticsInfo {
                source: "system".into(),
                code: Some("CONFIG_READY".into()),
                message: "ready".into(),
                last_attempt_at: None,
                last_successful_sync_at: Some(1_000),
                next_retry_at: None,
            },
        };
        let config = AppConfig {
            api_base_url: "https://example.com".into(),
            api_key: "secret".into(),
            polling_interval_seconds: 60,
            selected_group_id: "".into(),
            groups: vec![],
            dashboard: DashboardPreferences {
                layout_preset: "table".into(),
                density: "comfortable".into(),
            },
            window_policy: WindowPolicyConfig {
                dock_side: "right".into(),
                widget_width: 280,
                widget_height: 720,
                top_offset: 96,
            },
        };

        let next_snapshot = build_empty_groups_snapshot(&current_snapshot, &config, 2_000);

        assert_eq!(next_snapshot.health.status, "configError");
        assert_eq!(next_snapshot.diagnostics.code.as_deref(), Some("NO_GROUPS"));
        assert_eq!(
            next_snapshot.diagnostics.message,
            "Add at least one watch group to resume polling."
        );
        assert_eq!(next_snapshot.diagnostics.last_successful_sync_at, Some(1_000));
    }
}
