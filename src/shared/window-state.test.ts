import { describe, expect, it } from "vitest";
import {
  canWidgetReceiveDirectInteraction,
  describeWidgetFallback,
  isWidgetHidden,
  type WidgetBehaviorRuntime,
} from "./window-state";

function runtime(
  overrides?: Partial<WidgetBehaviorRuntime>,
): WidgetBehaviorRuntime {
  return {
    mode: "passive",
    placement: "hidden",
    clickThroughEnabled: false,
    clickThroughSupported: true,
    fallbackReason: null,
    wakeSource: null,
    interactionSessionId: 0,
    idleDeadlineAt: null,
    ...overrides,
  };
}

describe("window-state", () => {
  it("detects when the widget is hidden", () => {
    expect(isWidgetHidden(runtime())).toBe(true);
    expect(isWidgetHidden(runtime({ placement: "visible" }))).toBe(false);
  });

  it("treats hover and interactive as directly operable modes", () => {
    expect(canWidgetReceiveDirectInteraction(runtime({ mode: "hover" }))).toBe(true);
    expect(canWidgetReceiveDirectInteraction(runtime({ mode: "interactive" }))).toBe(true);
    expect(canWidgetReceiveDirectInteraction(runtime({ mode: "passive" }))).toBe(false);
  });

  it("returns fallback copy only when one exists", () => {
    expect(describeWidgetFallback(runtime())).toBeNull();
    expect(
      describeWidgetFallback(
        runtime({
          clickThroughSupported: false,
          fallbackReason: "Passive click-through is not enabled on this platform build.",
        }),
      ),
    ).toContain("Passive click-through");
  });
});
