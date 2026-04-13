use crate::app_state::{SharedAppState, WidgetBehaviorRuntime};
use crate::polling::scheduler;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

pub const WIDGET_WAKE_ZONE_WIDTH_PX: u32 = 14;
pub const WIDGET_IDLE_TIMEOUT_MS: u64 = 1_600;
pub const ALERT_WAKE_IDLE_TIMEOUT_MS: u64 = 2_600;

pub enum WidgetIntent {
    PointerEnter,
    PointerLeave,
    InteractionStart,
    AlertWake,
    IdleDeadline { session_id: u64 },
}

pub fn apply_widget_intent(
    runtime: &mut WidgetBehaviorRuntime,
    intent: WidgetIntent,
    now_ms: u64,
) -> bool {
    match intent {
        WidgetIntent::PointerEnter => {
            if runtime.mode == "interactive" {
                return false;
            }

            runtime.mode = "hover".into();
            runtime.placement = "visible".into();
            runtime.click_through_enabled = false;
            runtime.wake_source = Some("pointer".into());
            runtime.idle_deadline_at = None;
            runtime.interaction_session_id = runtime.interaction_session_id.saturating_add(1);
            true
        }
        WidgetIntent::PointerLeave => {
            if runtime.mode == "interactive" {
                runtime.idle_deadline_at = Some(now_ms + WIDGET_IDLE_TIMEOUT_MS);
                return true;
            }

            runtime.mode = "passive".into();
            runtime.placement = "hidden".into();
            runtime.click_through_enabled = false;
            runtime.wake_source = None;
            runtime.idle_deadline_at = None;
            runtime.interaction_session_id = runtime.interaction_session_id.saturating_add(1);
            true
        }
        WidgetIntent::InteractionStart => {
            runtime.mode = "interactive".into();
            runtime.placement = "visible".into();
            runtime.click_through_enabled = false;
            runtime.wake_source = Some("interaction".into());
            runtime.idle_deadline_at = Some(now_ms + WIDGET_IDLE_TIMEOUT_MS);
            runtime.interaction_session_id = runtime.interaction_session_id.saturating_add(1);
            true
        }
        WidgetIntent::AlertWake => {
            if runtime.mode == "interactive" {
                return false;
            }

            runtime.mode = "interactive".into();
            runtime.placement = "visible".into();
            runtime.click_through_enabled = false;
            runtime.wake_source = Some("alert".into());
            runtime.idle_deadline_at = Some(now_ms + ALERT_WAKE_IDLE_TIMEOUT_MS);
            runtime.interaction_session_id = runtime.interaction_session_id.saturating_add(1);
            true
        }
        WidgetIntent::IdleDeadline { session_id } => {
            if runtime.interaction_session_id != session_id || runtime.mode != "interactive" {
                return false;
            }

            runtime.mode = "passive".into();
            runtime.placement = "hidden".into();
            runtime.click_through_enabled = false;
            runtime.wake_source = None;
            runtime.idle_deadline_at = None;
            true
        }
    }
}

pub fn spawn_idle_timeout(
    app: AppHandle,
    state: SharedAppState,
    session_id: u64,
    deadline_at: u64,
) {
    tauri::async_runtime::spawn(async move {
        let wait_ms = deadline_at.saturating_sub(now()).max(1);
        tokio::time::sleep(Duration::from_millis(wait_ms)).await;

        let next_snapshot = state
            .update_with(|snapshot| {
                let _ = apply_widget_intent(
                    &mut snapshot.widget_runtime,
                    WidgetIntent::IdleDeadline { session_id },
                    now(),
                );
            })
            .await;

        let _ = scheduler::emit_snapshot(&app, &next_snapshot);
        let _ = crate::windows::sync_resident_surfaces(&app, &next_snapshot);
    });
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time drift")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::{apply_widget_intent, WidgetIntent, ALERT_WAKE_IDLE_TIMEOUT_MS, WIDGET_IDLE_TIMEOUT_MS};
    use crate::app_state::WidgetBehaviorRuntime;

    fn runtime() -> WidgetBehaviorRuntime {
        WidgetBehaviorRuntime::default()
    }

    #[test]
    fn promotes_pointer_entry_into_hover() {
        let mut runtime = runtime();

        let changed = apply_widget_intent(&mut runtime, WidgetIntent::PointerEnter, 1_000);

        assert!(changed);
        assert_eq!(runtime.mode, "hover");
        assert_eq!(runtime.placement, "visible");
        assert_eq!(runtime.wake_source.as_deref(), Some("pointer"));
    }

    #[test]
    fn interaction_start_promotes_to_interactive_with_idle_deadline() {
        let mut runtime = runtime();

        let changed = apply_widget_intent(&mut runtime, WidgetIntent::InteractionStart, 1_000);

        assert!(changed);
        assert_eq!(runtime.mode, "interactive");
        assert_eq!(runtime.idle_deadline_at, Some(1_000 + WIDGET_IDLE_TIMEOUT_MS));
        assert_eq!(runtime.wake_source.as_deref(), Some("interaction"));
    }

    #[test]
    fn alert_wake_does_not_reset_existing_interactive_session() {
        let mut runtime = runtime();
        let _ = apply_widget_intent(&mut runtime, WidgetIntent::InteractionStart, 1_000);
        let session_id = runtime.interaction_session_id;
        let deadline_at = runtime.idle_deadline_at;

        let changed = apply_widget_intent(&mut runtime, WidgetIntent::AlertWake, 1_200);

        assert!(!changed);
        assert_eq!(runtime.interaction_session_id, session_id);
        assert_eq!(runtime.idle_deadline_at, deadline_at);
    }

    #[test]
    fn alert_wake_promotes_passive_runtime_into_interactive() {
        let mut runtime = runtime();

        let changed = apply_widget_intent(&mut runtime, WidgetIntent::AlertWake, 1_000);

        assert!(changed);
        assert_eq!(runtime.mode, "interactive");
        assert_eq!(runtime.placement, "visible");
        assert_eq!(runtime.wake_source.as_deref(), Some("alert"));
        assert_eq!(runtime.idle_deadline_at, Some(1_000 + ALERT_WAKE_IDLE_TIMEOUT_MS));
    }

    #[test]
    fn idle_deadline_returns_interactive_session_to_hidden_passive() {
        let mut runtime = runtime();
        let _ = apply_widget_intent(&mut runtime, WidgetIntent::InteractionStart, 1_000);
        let session_id = runtime.interaction_session_id;

        let changed = apply_widget_intent(
            &mut runtime,
            WidgetIntent::IdleDeadline { session_id },
            2_800,
        );

        assert!(changed);
        assert_eq!(runtime.mode, "passive");
        assert_eq!(runtime.placement, "hidden");
        assert!(runtime.idle_deadline_at.is_none());
    }

    #[test]
    fn stale_idle_deadline_is_ignored() {
        let mut runtime = runtime();
        let _ = apply_widget_intent(&mut runtime, WidgetIntent::InteractionStart, 1_000);

        let changed = apply_widget_intent(
            &mut runtime,
            WidgetIntent::IdleDeadline { session_id: 0 },
            2_800,
        );

        assert!(!changed);
        assert_eq!(runtime.mode, "interactive");
    }
}
