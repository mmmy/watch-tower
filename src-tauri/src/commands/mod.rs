use crate::app_state::{AlertPayload, AppConfig, AppSnapshot, SharedAppState};
use crate::polling::alerts_client::{self, ReadStatusInput};
use crate::polling::scheduler;
use crate::windows::hover_state::{apply_widget_intent, spawn_idle_timeout, WidgetIntent};
use std::time::{SystemTime, UNIX_EPOCH};
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

#[tauri::command]
pub async fn mark_alert_read(
    alert: AlertPayload,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    let requested_at = now();
    let pending_snapshot = state
        .update_with(|snapshot| {
            snapshot
                .alert_runtime
                .mark_pending_read(alert.clone(), requested_at);
        })
        .await;

    scheduler::emit_snapshot(&app, &pending_snapshot)?;
    crate::windows::sync_resident_surfaces(&app, &pending_snapshot)?;

    let config = state
        .current_config()
        .await
        .ok_or_else(|| "Cannot mark alerts as read before config is saved.".to_string())?;

    let result = alerts_client::set_read_status(
        &state.http_client,
        &config,
        &ReadStatusInput {
            symbol: alert.symbol.clone(),
            period: alert.period.clone(),
            signal_type: alert.signal_type.clone(),
            read: true,
        },
    )
    .await;

    match result {
        Ok(true) => {
            let next_snapshot = state
                .update_with(|snapshot| {
                    snapshot.alert_runtime.resolve_pending_read(true);
                })
                .await;

            scheduler::emit_snapshot(&app, &next_snapshot)?;
            crate::windows::sync_resident_surfaces(&app, &next_snapshot)?;
            Ok(next_snapshot)
        }
        Ok(false) => {
            rollback_pending_read(
                &app,
                state.inner().clone(),
                "Server returned false while marking the alert as read.".into(),
            )
            .await
        }
        Err(error) => {
            rollback_pending_read(
                &app,
                state.inner().clone(),
                format!("Read-status request failed: {error:?}"),
            )
            .await
        }
    }
}

#[tauri::command]
pub async fn open_alert_in_dashboard(
    alert: AlertPayload,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    let current_config = state
        .current_config()
        .await
        .ok_or_else(|| "Cannot open an alert before config is saved.".to_string())?;
    let next_config = apply_alert_group_selection(&current_config, &alert);

    if next_config.selected_group_id != current_config.selected_group_id {
        state.repository.save(&next_config)?;
    }

    let requested_at = now();
    let next_snapshot = state
        .update_with(|snapshot| {
            snapshot.config = Some(next_config.clone());
            snapshot
                .alert_runtime
                .set_dashboard_focus_intent(alert.clone(), requested_at);

            if snapshot
                .alert_runtime
                .active_alert
                .as_ref()
                .is_some_and(|active_alert| active_alert.id == alert.id)
            {
                snapshot.alert_runtime.active_alert = None;
                snapshot.alert_runtime.promote_next_alert();
            }
        })
        .await;

    crate::windows::restore_main_dashboard(&app)?;
    scheduler::emit_snapshot(&app, &next_snapshot)?;
    crate::windows::sync_resident_surfaces(&app, &next_snapshot)?;

    Ok(next_snapshot)
}

#[tauri::command]
pub async fn clear_dashboard_focus_intent(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    let next_snapshot = state
        .update_with(|snapshot| {
            snapshot.alert_runtime.clear_dashboard_focus_intent();
        })
        .await;

    scheduler::emit_snapshot(&app, &next_snapshot)?;
    crate::windows::sync_resident_surfaces(&app, &next_snapshot)?;

    Ok(next_snapshot)
}

#[tauri::command]
pub async fn set_notifications_enabled(
    enabled: bool,
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    let current_config = state
        .current_config()
        .await
        .ok_or_else(|| "Cannot change notification settings before config is saved.".to_string())?;
    let next_config = apply_notifications_enabled(&current_config, enabled);

    state.repository.save(&next_config)?;

    let next_snapshot = state
        .update_with(|snapshot| {
            snapshot.config = Some(next_config.clone());
        })
        .await;

    scheduler::emit_snapshot(&app, &next_snapshot)?;
    crate::windows::sync_resident_surfaces(&app, &next_snapshot)?;

    Ok(next_snapshot)
}

#[tauri::command]
pub async fn widget_pointer_enter(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    update_widget_runtime(&app, state.inner().clone(), WidgetIntent::PointerEnter).await
}

#[tauri::command]
pub async fn widget_pointer_leave(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    update_widget_runtime(&app, state.inner().clone(), WidgetIntent::PointerLeave).await
}

#[tauri::command]
pub async fn widget_interaction_ping(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    update_widget_runtime(&app, state.inner().clone(), WidgetIntent::InteractionStart).await
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

async fn rollback_pending_read(
    app: &AppHandle,
    state: SharedAppState,
    message: String,
) -> Result<AppSnapshot, String> {
    let next_snapshot = state
        .update_with(|snapshot| {
            snapshot.alert_runtime.resolve_pending_read(false);
            snapshot.diagnostics.source = "request".into();
            snapshot.diagnostics.code = Some("READ_STATUS_FAILED".into());
            snapshot.diagnostics.message = message;
        })
        .await;

    scheduler::emit_snapshot(app, &next_snapshot)?;
    crate::windows::sync_resident_surfaces(app, &next_snapshot)?;
    Err(next_snapshot.diagnostics.message.clone())
}

fn apply_notifications_enabled(config: &AppConfig, enabled: bool) -> AppConfig {
    let mut next_config = config.clone();
    next_config.notifications_enabled = enabled;
    next_config
}

fn apply_alert_group_selection(config: &AppConfig, alert: &AlertPayload) -> AppConfig {
    if !config.groups.iter().any(|group| group.id == alert.group_id) {
        return config.clone();
    }

    let mut next_config = config.clone();
    next_config.selected_group_id = alert.group_id.clone();
    next_config
}

pub(crate) async fn update_widget_runtime(
    app: &AppHandle,
    state: SharedAppState,
    intent: WidgetIntent,
) -> Result<AppSnapshot, String> {
    let timestamp = now();
    let next_snapshot = state
        .update_with(|snapshot| {
            let _ = apply_widget_intent(&mut snapshot.widget_runtime, intent, timestamp);
        })
        .await;

    scheduler::emit_snapshot(app, &next_snapshot)?;
    crate::windows::sync_resident_surfaces(app, &next_snapshot)?;
    if let Some(deadline_at) = next_snapshot.widget_runtime.idle_deadline_at {
        spawn_idle_timeout(
            app.clone(),
            state,
            next_snapshot.widget_runtime.interaction_session_id,
            deadline_at,
        );
    }

    Ok(next_snapshot)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time drift")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::{
        apply_alert_group_selection, apply_notifications_enabled, select_group_in_config,
        set_polling_paused_in_snapshot,
    };
    use crate::app_state::{
        AlertPayload, AlertRuntime, AppConfig, AppSnapshot, DashboardPreferences,
        DiagnosticsInfo, PollingHealth, RuntimeInfo, WatchGroupConfig, WidgetBehaviorRuntime,
        WindowPolicyConfig,
    };
    use crate::windows::hover_state::{apply_widget_intent, WidgetIntent};

    fn test_config() -> AppConfig {
        AppConfig {
            api_base_url: "https://example.com".into(),
            api_key: "secret".into(),
            polling_interval_seconds: 60,
            notifications_enabled: true,
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
            alert_runtime: AlertRuntime::default(),
            widget_runtime: WidgetBehaviorRuntime::default(),
        }
    }

    fn test_alert() -> AlertPayload {
        AlertPayload {
            id: "ETHUSDT:240:divMacd".into(),
            group_id: "eth".into(),
            symbol: "ETHUSDT".into(),
            period: "240".into(),
            signal_type: "divMacd".into(),
            side: 1,
            signal_at: 1_000,
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

    #[test]
    fn applies_notification_setting_without_mutating_other_fields() {
        let config = test_config();

        let next_config = apply_notifications_enabled(&config, false);

        assert!(!next_config.notifications_enabled);
        assert_eq!(next_config.selected_group_id, config.selected_group_id);
        assert_eq!(next_config.groups.len(), config.groups.len());
    }

    #[test]
    fn applies_alert_group_selection_when_the_group_exists() {
        let config = test_config();

        let next_config = apply_alert_group_selection(&config, &test_alert());

        assert_eq!(next_config.selected_group_id, "eth");
        assert_eq!(next_config.api_base_url, config.api_base_url);
    }

    #[test]
    fn widget_interaction_promotes_runtime_to_interactive() {
        let mut snapshot = test_snapshot();

        let changed = apply_widget_intent(
            &mut snapshot.widget_runtime,
            WidgetIntent::InteractionStart,
            1_000,
        );

        assert!(changed);
        assert_eq!(snapshot.widget_runtime.mode, "interactive");
        assert_eq!(snapshot.widget_runtime.placement, "visible");
        assert!(snapshot.widget_runtime.idle_deadline_at.is_some());
    }
}
