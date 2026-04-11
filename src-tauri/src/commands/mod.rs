use crate::app_state::{AppConfig, AppSnapshot, SharedAppState};
use crate::polling::scheduler;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn get_bootstrap_state(state: State<'_, SharedAppState>) -> Result<AppSnapshot, String> {
    Ok(state.current_snapshot().await)
}

#[tauri::command]
pub async fn save_config(
    input: AppConfig,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    state.repository.save(&input)?;

    let paused = state.current_snapshot().await.runtime.polling_paused;
    let snapshot = state
        .update_with(|snapshot| {
            *snapshot = AppSnapshot::from_config(Some(input.clone()));
            if paused {
                set_polling_paused_in_snapshot(snapshot, true);
            }
        })
        .await;

    scheduler::emit_snapshot(&app, &snapshot)?;
    crate::windows::sync_resident_surfaces(&app, &snapshot)?;
    state.wake();

    Ok(snapshot)
}

#[tauri::command]
pub async fn select_group(
    group_id: String,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    let current_config = state
        .current_config()
        .await
        .ok_or_else(|| "Cannot switch groups before config is saved.".to_string())?;
    let next_config = select_group_in_config(&current_config, &group_id)?;

    state.repository.save(&next_config)?;

    let snapshot = state
        .update_with(|snapshot| {
            snapshot.bootstrap_required = false;
            snapshot.config = Some(next_config.clone());
            snapshot.health.polling_interval_seconds = Some(next_config.polling_interval_seconds);
        })
        .await;

    scheduler::emit_snapshot(&app, &snapshot)?;
    crate::windows::sync_resident_surfaces(&app, &snapshot)?;

    Ok(snapshot)
}

#[tauri::command]
pub async fn poll_now(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    scheduler::poll_once(&app, state.inner().clone()).await
}

#[tauri::command]
pub async fn pause_polling(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    set_polling_paused(&app, state.inner().clone(), true).await
}

#[tauri::command]
pub async fn resume_polling(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    set_polling_paused(&app, state.inner().clone(), false).await
}

fn select_group_in_config(config: &AppConfig, group_id: &str) -> Result<AppConfig, String> {
    if !config.groups.iter().any(|group| group.id == group_id) {
        return Err(format!("Unknown group id: {group_id}"));
    }

    let mut next_config = config.clone();
    next_config.selected_group_id = group_id.to_string();
    Ok(next_config)
}

pub(crate) async fn set_polling_paused(
    app: &AppHandle,
    state: SharedAppState,
    paused: bool,
) -> Result<AppSnapshot, String> {
    let snapshot = state
        .update_with(|snapshot| {
            set_polling_paused_in_snapshot(snapshot, paused);
        })
        .await;

    scheduler::emit_snapshot(app, &snapshot)?;
    crate::windows::sync_resident_surfaces(app, &snapshot)?;
    if !paused {
        state.wake();
    }

    Ok(snapshot)
}

pub(crate) fn set_polling_paused_in_snapshot(snapshot: &mut AppSnapshot, paused: bool) {
    if paused {
        if !snapshot.runtime.polling_paused {
            snapshot.runtime.last_active_status = Some(snapshot.health.status.clone());
        }
        snapshot.runtime.polling_paused = true;
        snapshot.diagnostics.source = "system".into();
        snapshot.diagnostics.code = Some("POLLING_PAUSED".into());
        snapshot.diagnostics.message =
            "Polling is paused. Resident surfaces keep the latest snapshot visible.".into();
        return;
    }

    snapshot.runtime.polling_paused = false;
    snapshot.health.status = snapshot
        .runtime
        .last_active_status
        .take()
        .unwrap_or_else(|| derive_active_status(snapshot));
    snapshot.diagnostics.source = "system".into();
    snapshot.diagnostics.code = Some("POLLING_RESUMED".into());
    snapshot.diagnostics.message = "Polling resumed. Waiting for the next scheduler cycle.".into();
}

fn derive_active_status(snapshot: &AppSnapshot) -> String {
    if snapshot.bootstrap_required || snapshot.config.is_none() {
        return "bootstrapRequired".into();
    }

    if snapshot
        .config
        .as_ref()
        .is_some_and(|config| config.groups.is_empty())
    {
        return "configError".into();
    }

    if snapshot.diagnostics.next_retry_at.is_some() {
        return "backoff".into();
    }

    if snapshot.raw_response.is_some() {
        return "success".into();
    }

    "idle".into()
}

#[cfg(test)]
mod tests {
    use super::{select_group_in_config, set_polling_paused_in_snapshot};
    use crate::app_state::{
        AppConfig, AppSnapshot, DashboardPreferences, DiagnosticsInfo, PollingHealth, RuntimeInfo,
        WatchGroupConfig, WindowPolicyConfig,
    };

    fn test_config() -> AppConfig {
        AppConfig {
            api_base_url: "https://example.com".into(),
            api_key: "secret".into(),
            polling_interval_seconds: 60,
            selected_group_id: "btc".into(),
            groups: vec![
                WatchGroupConfig {
                    id: "btc".into(),
                    symbol: "BTCUSDT".into(),
                    signal_types: vec!["vegas".into()],
                    periods: vec!["240".into(), "60".into()],
                    selected_timeline_period: "60".into(),
                },
                WatchGroupConfig {
                    id: "eth".into(),
                    symbol: "ETHUSDT".into(),
                    signal_types: vec!["divMacd".into()],
                    periods: vec!["240".into(), "60".into()],
                    selected_timeline_period: "240".into(),
                },
            ],
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
        }
    }

    fn test_snapshot() -> AppSnapshot {
        AppSnapshot {
            bootstrap_required: false,
            config: Some(test_config()),
            raw_response: None,
            health: PollingHealth {
                status: "success".into(),
                polling_interval_seconds: Some(60),
                is_stale: false,
            },
            diagnostics: DiagnosticsInfo {
                source: "system".into(),
                code: Some("SYNC_OK".into()),
                message: "Latest signal snapshot loaded successfully.".into(),
                last_attempt_at: Some(1_000),
                last_successful_sync_at: Some(1_000),
                next_retry_at: None,
            },
            runtime: RuntimeInfo {
                polling_paused: false,
                last_active_status: None,
            },
        }
    }

    #[test]
    fn switches_to_a_known_group_without_mutating_other_config() {
        let config = test_config();

        let next_config = select_group_in_config(&config, "eth").expect("group selection");

        assert_eq!(next_config.selected_group_id, "eth");
        assert_eq!(next_config.api_base_url, config.api_base_url);
        assert_eq!(next_config.groups.len(), config.groups.len());
    }

    #[test]
    fn rejects_unknown_groups() {
        let config = test_config();

        let error = select_group_in_config(&config, "sol").expect_err("unknown group");

        assert!(error.contains("Unknown group id"));
    }

    #[test]
    fn pausing_marks_runtime_as_paused_without_discarding_config() {
        let mut snapshot = test_snapshot();

        set_polling_paused_in_snapshot(&mut snapshot, true);

        assert!(snapshot.runtime.polling_paused);
        assert_eq!(
            snapshot
                .config
                .as_ref()
                .map(|config| config.selected_group_id.as_str()),
            Some("btc")
        );
        assert_eq!(snapshot.diagnostics.code.as_deref(), Some("POLLING_PAUSED"));
    }

    #[test]
    fn resuming_restores_backoff_status_when_retry_is_pending() {
        let mut snapshot = test_snapshot();
        snapshot.health.status = "backoff".into();
        snapshot.diagnostics.next_retry_at = Some(9_000);

        set_polling_paused_in_snapshot(&mut snapshot, true);
        set_polling_paused_in_snapshot(&mut snapshot, false);

        assert!(!snapshot.runtime.polling_paused);
        assert_eq!(snapshot.health.status, "backoff");
        assert_eq!(snapshot.diagnostics.next_retry_at, Some(9_000));
        assert_eq!(snapshot.diagnostics.code.as_deref(), Some("POLLING_RESUMED"));
    }
}
