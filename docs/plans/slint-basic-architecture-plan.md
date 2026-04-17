# Slint Basic Architecture Plan

## Goal

Build a first native Rust UI spike that replaces the current Tauri + WebView shell with a Slint-based desktop shell.

This spike is intentionally limited to the minimum architecture:

- Main window
- System tray icon
- Floating widget window
- System tray menu

The goal of this phase is to prove that Signal Desk can run with a native UI shell and lower baseline resource usage, without chasing full feature parity.

## Why This Spike

The current app already keeps most runtime and state management in Rust, while the React/Tauri layer is primarily a presentation shell. That makes the project a good candidate for a staged migration where we preserve the Rust runtime model and swap the UI shell first.

Slint is the preferred target for this spike because it is a native GUI toolkit for Rust, is designed for low-footprint apps, supports desktop windows, and supports event loops that can stay alive for tray-style applications even when no window is visible.

References:

- [Slint docs](https://docs.slint.dev/index.html)
- [Slint desktop guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/)
- [Slint Rust event loop docs](https://docs.slint.dev/latest/docs/rust/slint/fn.run_event_loop_until_quit)
- [tray-icon crate docs](https://docs.rs/tray-icon/latest/tray_icon/)

## Scope

### In Scope

- Create a new native desktop entrypoint based on Slint
- Show a main window with placeholder Signal Desk content
- Show a small floating widget window with placeholder unread count
- Keep the app alive via system tray even when windows are hidden
- Add tray menu items for:
  - Show/hide main window
  - Show/hide widget
  - Refresh placeholder state
  - Quit
- Define the Rust-side state boundary between runtime data and UI
- Leave enough structure so later phases can reconnect polling, config persistence, and signal actions

### Out of Scope

- Full visual parity with the current React UI
- API integration and real signal polling migration
- Config editing, persistence rewiring, or advanced settings UI
- Widget drag, edge docking, auto-hide, and animation parity
- Notifications and sound
- Cross-platform polishing beyond basic Windows viability

## Target Architecture

### 1. Runtime Core Stays in Rust

Keep the existing Rust data model as the source of truth:

- app config
- runtime snapshot
- signal list
- unread count
- visibility state

For this spike, we should introduce a UI-facing app state layer that is Rust-native and Slint-friendly, instead of going through Tauri commands/events.

### 2. Replace Tauri Shell With Native App Shell

Replace the Tauri-specific shell responsibilities with native Rust components:

- `Slint` for window UI
- `tray-icon` for tray icon + tray menu
- shared Rust state for communication between tray actions and UI windows

This first spike does not need to delete Tauri immediately. We can build the Slint shell beside the existing implementation and switch the app entrypoint once the spike is stable.

### 3. Two Windows, One Shared State Model

Define two windows:

- `MainWindow`
  - standard application window
  - placeholder toolbar/status area
  - placeholder group list area
- `WidgetWindow`
  - always-on-top floating window
  - small footprint
  - placeholder unread badge / compact status

Both windows should observe the same Rust-side state and respond to shared actions.

### 4. Tray-First Lifecycle

The app lifecycle should be tray-first:

- starting the app creates the tray icon
- windows may be created eagerly for the spike, but the event loop should remain alive even when they are hidden
- closing a window should hide it instead of terminating the process
- only the tray Quit action should end the process

## Proposed File Layout

This is the minimum structure for the spike:

```text
src-native/
  main.rs
  app_state.rs
  tray.rs
  windows/
    main_window.slint
    widget_window.slint
    mod.rs
```

Notes:

- `src-tauri/` remains temporarily as the current implementation until the Slint spike is proven
- the new native shell should be isolated enough that we can iterate without breaking the current app

## Phase 1 Deliverables

The spike is successful when all of the following are true:

1. A new worktree branch contains a documented Slint migration path.
2. The app can launch a Slint main window.
3. The app can show a floating widget window.
4. A tray icon appears with a working menu.
5. Tray menu actions can show/hide the main window and widget.
6. Closing windows does not quit the process.
7. Quit from tray exits cleanly.

## Implementation Plan

### Step 1. Scaffold the native shell

- add Slint dependencies
- add a new native entrypoint
- create placeholder `.slint` window files
- compile a basic main window

### Step 2. Add shared state

- define a minimal `AppState`
- expose unread count, connection text, and window visibility
- wire simple state update methods for tray and UI

### Step 3. Add tray support

- create tray icon
- define tray menu items
- connect menu events to state changes and window visibility changes

### Step 4. Add floating widget

- create always-on-top widget window
- render placeholder unread count
- wire show/hide behavior from tray

### Step 5. Stabilize lifecycle

- ensure app keeps running when windows are hidden
- ensure quit is explicit
- confirm Windows startup behavior is sane for tray-first usage

## Risks

### Tray/Event-Loop Integration

Slint and tray handling must cooperate on the same desktop event loop model. This is the biggest technical unknown in the spike and should be validated early.

### Widget Window Behavior

Always-on-top, transparent, and compact floating windows are central to the product feel. The spike should keep the first widget simple and avoid advanced drag/dock behavior until the shell is stable.

### Parallel Shell Period

While Tauri and Slint coexist, the repo will have two UI shells. That adds short-term complexity, but it reduces migration risk and keeps the rollback path simple.

## Acceptance Criteria For This Planning Step

This document is complete enough if the next implementation pass can start immediately with:

- one chosen UI toolkit: Slint
- one chosen tray approach: `tray-icon`
- one chosen scope boundary: basic shell only
- one chosen migration style: parallel spike, not full rewrite in one pass

## Next Step

Implement the Phase 1 scaffold in this worktree:

- create the Slint app skeleton
- add tray support
- add main + widget windows with placeholder content
- verify the app can run as a tray-first native shell on Windows
