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
pub async fn poll_now(
    app: AppHandle,
    state: State<'_, SharedAppState>,
) -> Result<AppSnapshot, String> {
    scheduler::poll_once(&app, state.inner().clone()).await
}
