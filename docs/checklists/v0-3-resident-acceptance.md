# Watch Tower v0.3 Resident Acceptance

## Core Loop

- [ ] Closing the main dashboard hides it instead of exiting the app.
- [ ] The tray remains available after the dashboard is hidden.
- [ ] The tray can restore the main dashboard without losing the current selected group.
- [ ] The tray can explicitly quit the app.

## Widget

- [ ] The edge widget appears after config is available.
- [ ] The widget follows the saved `dockSide`, `widgetWidth`, `widgetHeight`, and `topOffset` policy.
- [ ] The widget shows the current selected group's 25-period resident view.
- [ ] Switching group in the dashboard updates the resident widget without recreating app state.

## Runtime Health

- [ ] `running`, `paused`, `backoff`, `authError`, and `stale snapshot` are visible outside the main dashboard.
- [ ] Pausing polling from the tray updates tray status and widget footer together.
- [ ] Resuming polling from the tray returns the scheduler to a live cycle without duplicating concurrent polls.
- [ ] Backoff keeps the latest good snapshot visible instead of blanking the resident surfaces.

## Scope Guardrails

- [ ] `v0.3` does not introduce popup alerts or system notifications.
- [ ] `v0.3` does not introduce tray/widget group switching.
- [ ] `v0.3` does not introduce auto-hide, hover wake, or click-through behavior.
