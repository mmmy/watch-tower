# Watch Tower v0.4 Alert Closure Acceptance

## New Unread Detection

- [ ] A newly unread signal from any configured group is projected into the host alert runtime.
- [ ] The first newly unread signal becomes the active popup alert.
- [ ] Additional newly unread signals in the same poll cycle are preserved in the single-window backlog instead of being dropped.
- [ ] A repeated unread from later poll cycles does not retrigger the same popup or system notification.

## Popup And Notification Behavior

- [ ] When system notifications are enabled, a new unread signal produces both the popup card and a system notification.
- [ ] When system notifications are disabled, the popup still appears but no system notification is sent.
- [ ] The popup shows the correct `symbol`, `period`, `signalType`, and bullish/bearish direction for the active alert.
- [ ] After the active alert is handled, the next backlog alert becomes the active popup without opening multiple visible popup windows.

## Dashboard Handoff

- [ ] Opening an alert from the popup restores the main dashboard if it was hidden.
- [ ] Opening an alert from the popup switches the dashboard to the alert's group when that group is not currently selected.
- [ ] Opening an alert from the popup focuses the dashboard on the alert's `period` and `signalType`, rather than leaving the user to relocate it manually.

## Read Status Loop

- [ ] Marking an alert as read enters a visible pending state before the host confirms the write.
- [ ] A successful `read-status` write removes the handled alert and promotes the next backlog alert when one exists.
- [ ] A failed `read-status` write clears the pending state, restores the alert to an actionable unread state, and surfaces an explicit failure message.
- [ ] A pending optimistic read does not cause the same unread signal to re-alert while the write is still in flight.

## Scope Guardrails

- [ ] `v0.4` does not introduce tray/widget group switching.
- [ ] `v0.4` does not introduce multi-window popup stacking or queue orchestration.
- [ ] `v0.4` does not introduce `auto-hide`, `hover wake`, or `click-through`.
- [ ] `v0.4` only offers a single persisted notification on/off control rather than a full notification center.
