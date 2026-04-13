import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAlertPopupEvents } from "./use-alert-popup-events";

const invokeMock = vi.fn();
const listenMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

function HookHarness() {
  const { popupView } = useAlertPopupEvents();

  return (
    <div>
      <div data-testid="popup-state">{popupView?.state ?? "missing"}</div>
      <div data-testid="popup-symbol">{popupView?.alert?.symbol ?? "none"}</div>
    </div>
  );
}

describe("useAlertPopupEvents", () => {
  afterEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
    listenMock.mockResolvedValue(() => {});
    (window as typeof window & {
      __TAURI_INTERNALS__?: {
        metadata?: {
          currentWindow?: {
            label?: string;
          };
        };
      };
    }).__TAURI_INTERNALS__ = undefined;
  });

  it("provides an active fallback alert view in browser mode", async () => {
    render(<HookHarness />);

    await waitFor(() => {
      expect(screen.getByTestId("popup-state").textContent).toBe("active");
      expect(screen.getByTestId("popup-symbol").textContent).toBe("ETHUSDT");
    });
  });

  it("selects the popup stream that matches the current window label", async () => {
    invokeMock.mockResolvedValue({
      bootstrapRequired: false,
      config: null,
      rawResponse: null,
      health: {
        status: "success",
        pollingIntervalSeconds: 60,
        isStale: false,
      },
      diagnostics: {
        source: "system",
        code: "SYNC_OK",
        message: "ready",
        lastAttemptAt: null,
        lastSuccessfulSyncAt: null,
        nextRetryAt: null,
      },
      runtime: {
        pollingPaused: false,
        lastActiveStatus: null,
      },
      alertRuntime: {
        visiblePopupStreams: [
          {
            symbol: "BTCUSDT",
            alerts: [
              {
                id: "BTCUSDT:60:vegas",
                groupId: "btc-core",
                symbol: "BTCUSDT",
                period: "60",
                signalType: "vegas",
                side: 1,
                signalAt: 1_000,
              },
            ],
          },
        ],
        queuedPopupStreams: [],
        pendingRead: null,
        dashboardFocusIntent: null,
      },
      widgetRuntime: {
        mode: "passive",
        placement: "hidden",
        clickThroughEnabled: false,
        clickThroughSupported: true,
        fallbackReason: null,
        wakeSource: null,
        interactionSessionId: 0,
        idleDeadlineAt: null,
      },
    });
    listenMock.mockResolvedValue(() => {});

    (window as typeof window & {
      __TAURI_INTERNALS__?: {
        metadata?: {
          currentWindow?: {
            label?: string;
          };
        };
      };
    }).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: {
          label: "alert-popup:BTCUSDT",
        },
      },
    };

    render(<HookHarness />);

    await waitFor(() => {
      expect(screen.getByTestId("popup-state").textContent).toBe("active");
      expect(screen.getByTestId("popup-symbol").textContent).toBe("BTCUSDT");
    });
  });
});
