# Watch Tower v0.5 Desktop Behavior Acceptance

## Widget Behavior Runtime

- [ ] The host snapshot projects a single widget behavior runtime with `passive`, `hover`, and `interactive` as the only stable modes.
- [ ] Hidden placement is represented as a `passive` placement result rather than a fourth stable mode.
- [ ] The widget behavior runtime is visible in both the edge widget view and dashboard diagnostics without the two surfaces drifting.

## Hide And Wake Loop

- [ ] The widget can return to a hidden passive placement without interrupting resident polling or losing the current group snapshot.
- [ ] Hovering the wake zone reveals the widget and promotes it into a visible session without reopening the main dashboard.
- [ ] Leaving the widget after interaction returns it to passive after the idle timeout instead of leaving it permanently expanded.
- [ ] Short hover noise or accidental edge passes do not leave the widget stuck in `hover` or `interactive`.

## Interaction And Fallback

- [ ] In `interactive`, the widget remains directly clickable and focusable for normal actions.
- [ ] When passive click-through is unsupported on the current platform, the widget still remains recoverable and the fallback is visible in diagnostics.
- [ ] A platform fallback never leaves the widget in a dead state where it is hidden, non-interactive, and impossible to wake.

## Alert Coordination

- [ ] A newly detected alert can promote a passive widget into a visible interactive session once.
- [ ] That alert-driven promotion does not alter `v0.4` popup backlog, dedupe, or read-writeback behavior.
- [ ] Repeated snapshots for the same alert do not keep re-promoting an already interactive widget session.

## Scope Guardrails

- [ ] `v0.5` does not add tray or widget group switching.
- [ ] `v0.5` does not add multi-popup orchestration, queue management, or market overview surfaces.
- [ ] `v0.5` does not introduce a new reminder runtime separate from the existing shared snapshot.
