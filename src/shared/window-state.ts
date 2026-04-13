export const WIDGET_WAKE_ZONE_WIDTH_PX = 14;
export const WIDGET_REVEAL_DELAY_MS = 180;
export const WIDGET_IDLE_TIMEOUT_MS = 1600;

export type WidgetBehaviorMode = "passive" | "hover" | "interactive";
export type WidgetPlacementState = "visible" | "hidden";
export type WidgetWakeSource = "pointer" | "interaction" | "alert";

export interface WidgetBehaviorRuntime {
  mode: WidgetBehaviorMode;
  placement: WidgetPlacementState;
  clickThroughEnabled: boolean;
  clickThroughSupported: boolean;
  fallbackReason: string | null;
  wakeSource: WidgetWakeSource | null;
  interactionSessionId: number;
  idleDeadlineAt: number | null;
}

export function isWidgetHidden(runtime: WidgetBehaviorRuntime): boolean {
  return runtime.placement === "hidden";
}

export function canWidgetReceiveDirectInteraction(
  runtime: WidgetBehaviorRuntime,
): boolean {
  return runtime.mode === "interactive" || runtime.mode === "hover";
}

export function describeWidgetFallback(runtime: WidgetBehaviorRuntime): string | null {
  if (!runtime.fallbackReason) {
    return null;
  }

  return runtime.fallbackReason;
}
