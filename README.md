# Watch Tower

Watch Tower is a Tauri desktop monitor for multi-period trading signals. It keeps a selected group resident on the desktop through the main dashboard, an edge widget, popup alerts, and a tray controller.

## What v0.6 adds

- Multiple popup streams can coexist by symbol instead of sharing a single global alert window.
- Queued or still-visible alerts can be recovered from the main dashboard.
- Trial-release docs now describe the resident, popup, and recovery loop end to end.

## Current Product Shape

- Main dashboard: configuration, group management, signal drill-down, diagnostics, and unread recovery.
- Edge widget: always-on resident summary for the selected group.
- Alert popup: symbol-scoped alert surface with direct mark-read and dashboard handoff actions.
- Tray controller: restore dashboard, pause polling, resume polling, and exit.

## Local Development

### Prerequisites

- Node.js 20+
- Rust stable toolchain
- Windows is the current primary platform target

### Install

```bash
npm install
```

### Run the desktop app

```bash
npm run tauri dev
```

### Run tests

```bash
npm test
cargo test
```

## Trial Setup

1. Launch the app and save a valid API base URL and API key.
2. Create at least one watch group with a symbol, signal types, and timeline focus.
3. Confirm the dashboard transitions from bootstrap into a live or polling state.
4. Hide the dashboard and verify the edge widget and tray continue reflecting host state.
5. Wait for unread alerts or use a controlled test backend to trigger them.

## Trial Validation Flow

Use this sequence for manual trial runs:

1. Trigger unread alerts for multiple symbols in the same poll window.
2. Verify popup windows are created per symbol and stack without overlapping.
3. Ignore at least one popup and confirm the unread recovery section in the dashboard still lists it.
4. Open one recovery item in the dashboard and mark another one read directly.
5. Confirm the widget, popup surfaces, and dashboard remain consistent after each action.

## Known Limits

- Polling is still the only sync mechanism; there is no websocket push path.
- The product does not yet provide a full notification center or complex mute rules.
- Group switching remains a dashboard responsibility; tray and widget stay intentionally minimal.
- Market overview is not part of the blocking v0.6 trial path.

## Repo Notes

- Desktop host code lives in `src-tauri/`.
- Shared TypeScript models live in `src/shared/`.
- Window-specific UI lives in `src/windows/`.
- Planning docs live in `docs/plans/`.
