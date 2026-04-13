import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useEdgeWidgetEvents } from "./use-edge-widget-events";

function HookHarness() {
  const { widgetView } = useEdgeWidgetEvents();

  return (
    <div>
      <div data-testid="widget-state">{widgetView?.state ?? "loading"}</div>
      <div data-testid="runtime-status">{widgetView?.runtimeStatus ?? "loading"}</div>
      <div data-testid="widget-symbol">{widgetView?.groupSnapshot?.group.symbol ?? "none"}</div>
      <div data-testid="widget-mode">{widgetView?.widgetRuntime.mode ?? "none"}</div>
    </div>
  );
}

describe("useEdgeWidgetEvents", () => {
  it("builds a resident widget snapshot in browser fallback mode", async () => {
    render(<HookHarness />);

    await waitFor(() => {
      expect(screen.getByTestId("widget-state").textContent).toBe("ready");
      expect(screen.getByTestId("runtime-status").textContent).toBe("success");
      expect(screen.getByTestId("widget-symbol").textContent).toBe("BTCUSDT");
      expect(screen.getByTestId("widget-mode").textContent).toBe("passive");
    });
  });
});
