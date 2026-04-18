---
title: Slint Migration Implementation Plan
type: refactor
status: completed
date: 2026-04-18
completed: 2026-04-18
---

# Slint Migration Implementation Plan

## Final State

Completed.

Watch Tower now runs as a single-shell Windows-native Slint application in this worktree.

The old Tauri + React implementation has been removed from the main repo path. Default developer scripts and documentation now target `native-shell/` only.

## Final Outcomes

- `native-shell/` is the active desktop shell
- `npm run dev`, `npm run check`, `npm run build`, and `npm run test` all target `native-shell/`
- `src/`, `src-tauri/`, `public/`, `index.html`, `vite.config.ts`, and the TypeScript build configs were removed
- widget docking, persistence, reveal/hide behavior, and lightweight animation were implemented
- main-window runtime controls, section grouping, and batch mark-read behavior were implemented
- the Rust-native runtime/config boundary was preserved and expanded

## Completed Requirements Trace

- R1. The Slint shell became the real Windows app entrypoint.
- R2. Runtime controls were migrated into the Slint runtime and shell wiring.
- R3. The main window now supports the core daily workflows needed from the old shell.
- R4. The widget now supports docking, persistence, reveal/hide behavior, context actions, and animation.
- R5. The migration is no longer in a reversible dual-shell phase in this worktree because the legacy shell was removed.
- R6. The default workflow is Windows-native and does not rely on WSL.

## Completed Units

- [x] **Unit 1: Turn the Slint spike into a testable shell module**
- [x] **Unit 2: Achieve runtime and config-command parity**
- [x] **Unit 3: Migrate tray, window lifecycle, and default entrypoint wiring**
- [x] **Unit 4: Reach main-window workflow parity**
- [x] **Unit 5: Reach widget behavior parity on Windows**
- [x] **Unit 6: Cut over to Slint as the default Windows shell and retire the fallback path**

## Resulting Repo Shape

```text
docs/
native-shell/
scripts/
.gitignore
README.md
config.yaml
config.yaml.example
package.json
widget-placement.json
```

## Notes On Removed Scope

The migration plan originally assumed a temporary dual-shell period where Tauri would remain available as a fallback. That is no longer true in this worktree. The old shell was removed entirely instead of being preserved long-term.

This means the following plan sections are now historical only:

- references to `src-tauri/src/lib.rs` and `src/App.tsx` as active repo code
- references to `src-tauri/` as an available fallback shell
- cutover steps that assumed a later removal pass

## Verification Summary

The final Slint-only repo path is verified by the default scripts:

- `npm run check`
- `npm run test`

The native test suite covers:

- shell smoke behavior
- runtime/config parity behavior
- main-window state mapping
- lifecycle placement behavior
- widget docking, persistence, and animation helpers

## Historical References

- Phase 1 spike record: `docs/plans/slint-basic-architecture-plan.md`
- Active desktop shell: `native-shell/`
