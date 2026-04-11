import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useAppEvents } from "./use-app-events";

function HookHarness() {
  const { snapshot, saveConfig, selectGroup } = useAppEvents();

  return (
    <div>
      <div data-testid="selected-group">{snapshot?.config?.selectedGroupId ?? "none"}</div>
      <div data-testid="layout-preset">{snapshot?.config?.dashboard.layoutPreset ?? "none"}</div>
      <button type="button" onClick={() => selectGroup("btc-core")}>
        Select BTC
      </button>
      <button
        type="button"
        onClick={() =>
          saveConfig({
            apiBaseUrl: "https://api.example.com",
            apiKey: "demo-key",
            pollingIntervalSeconds: 60,
            selectedGroupId: "btc-core",
            layoutPreset: "list",
            density: "compact",
            windowPolicy: {
              dockSide: "left",
              widgetWidth: 320,
              widgetHeight: 680,
              topOffset: 80,
            },
            groups: [
              {
                id: "btc-core",
                symbol: "BTCUSDT",
                signalTypesText: "vegas,divMacd",
                selectedTimelinePeriod: "240",
              },
            ],
          })
        }
      >
        Save list layout
      </button>
    </div>
  );
}

describe("useAppEvents", () => {
  it("switches the selected group in browser fallback mode", async () => {
    render(<HookHarness />);

    await waitFor(() => {
      expect(screen.getByTestId("selected-group").textContent).toBe("eth-swing");
    });

    fireEvent.click(screen.getByRole("button", { name: "Select BTC" }));

    await waitFor(() => {
      expect(screen.getByTestId("selected-group").textContent).toBe("btc-core");
    });
  });

  it("persists layout preferences through the fallback save path", async () => {
    render(<HookHarness />);

    fireEvent.click(screen.getByRole("button", { name: "Save list layout" }));

    await waitFor(() => {
      expect(screen.getByTestId("selected-group").textContent).toBe("btc-core");
      expect(screen.getByTestId("layout-preset").textContent).toBe("list");
    });
  });
});
