# Slint Basic Architecture Plan

## Status

Completed.

This document is retained as the historical Phase 1 spike record. The spike proved that Watch Tower could replace the old Tauri + WebView shell with a native Slint shell while keeping runtime logic in Rust.

## What This Spike Was Meant To Prove

The original spike intentionally targeted only the minimum native shell:

- main window
- system tray icon
- floating widget window
- tray menu
- hide-on-close lifecycle

It did not aim for full feature parity. The purpose was to validate that:

- Slint could host the desktop shell on Windows
- tray-first lifecycle could work without Tauri
- a small floating widget could exist in the native shell
- the runtime boundary could stay in Rust

## Outcome

The spike succeeded.

It established the crate and UI structure that later became the production path:

- `native-shell/src/main.rs`
- `native-shell/src/lib.rs`
- `native-shell/src/shell.rs`
- `native-shell/src/runtime.rs`
- `native-shell/src/app_state.rs`
- `native-shell/src/tray.rs`
- `native-shell/src/widget_state.rs`
- `native-shell/ui/main-window.slint`

## What Changed After The Spike

The repository no longer has the old Tauri + React fallback shell in the main tree.

The migration continued past the spike and is now complete:

- Slint is the only active desktop shell in this worktree
- default repo scripts target `native-shell/`
- the legacy `src/` and `src-tauri/` trees were removed
- widget docking, reveal/hide, persistence, and lightweight animation were added
- main-window controls, grouped signal sections, and batch mark-read behavior were added

## Historical Scope Boundary

At the time of the spike, these items were intentionally deferred:

- full visual parity
- config persistence wiring
- advanced widget docking and animation behavior
- notifications and sound parity
- full workflow parity in the main window

Those items were completed in later phases and are no longer open work in this branch.

## References

- Migration completion record: `docs/plans/2026-04-18-001-refactor-slint-migration-plan.md`
- Active shell code: `native-shell/`
