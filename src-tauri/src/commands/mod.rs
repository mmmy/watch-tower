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

    let snapshot = state
        .update_with(|snapshot| {
            *snapshot = AppSnapshot::from_config(Some(input.clone()));
        })
        .await;

    scheduler::emit_snapshot(&app, &snapshot)?;
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

    Ok(snapshot)
}

#[tauri::command]
pub async fn poll_now(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    scheduler::poll_once(&app, state.inner().clone()).await
}

fn select_group_in_config(config: &AppConfig, group_id: &str) -> Result<AppConfig, String> {
    if !config.groups.iter().any(|group| group.id == group_id) {
        return Err(format!("Unknown group id: {group_id}"));
    }

    let mut next_config = config.clone();
    next_config.selected_group_id = group_id.to_string();
    Ok(next_config)
}

#[cfg(test)]
mod tests {
    use super::select_group_in_config;
    use crate::app_state::{AppConfig, DashboardPreferences, WatchGroupConfig, WindowPolicyConfig};

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
}
