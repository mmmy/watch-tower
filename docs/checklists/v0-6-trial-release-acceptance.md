# Watch Tower v0.6 Trial Release Acceptance

## Popup Orchestration

- [ ] New unread alerts from different symbols can create separate popup windows in the same poll cycle.
- [ ] Same-symbol follow-up alerts reuse the existing popup stream instead of opening duplicate windows.
- [ ] Visible popup windows stack in a stable order without overlapping.
- [ ] When visible popup capacity is exceeded, overflow items remain recoverable rather than disappearing.

## Dashboard Recovery

- [ ] The main dashboard shows unread recovery items for alerts that are still visible or queued.
- [ ] Opening a recovery item routes the dashboard to the correct group, period, and signal type.
- [ ] Marking a recovery item as read removes it from recovery without desynchronizing popup state.
- [ ] A failed read-status write keeps the item actionable instead of silently dropping it.

## Cross-Surface Consistency

- [ ] Popup actions and dashboard recovery actions both use the same host command semantics.
- [ ] Handling an alert updates the shared snapshot consistently across dashboard, popup, tray, and widget surfaces.
- [ ] Multi-symbol popup orchestration does not change the existing `v0.4` unread diff or `v0.5` widget behavior guarantees.

## Trial Packaging And Docs

- [ ] Dynamic popup labels still retain IPC access through the alert-popup capability definition.
- [ ] README is sufficient for a first-time tester to configure the app and validate the resident + popup + recovery loop.
- [ ] Differences between dev mode and packaged mode are recorded during the trial pass instead of being left implicit.
- [ ] The trial run documents any platform-specific gaps that remain outside the current scope.
