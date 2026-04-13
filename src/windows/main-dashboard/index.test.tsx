import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { sanitizeConfigInput } from "../../shared/config-model";
import { MainDashboardPage } from "./index";

const useAppEventsMock = vi.fn();

vi.mock("./hooks/use-app-events", () => ({
  useAppEvents: () => useAppEventsMock(),
}));

describe("MainDashboardPage", () => {
  beforeEach(() => {
    useAppEventsMock.mockReset();
  });

  it("renders the productized dashboard layout for a configured snapshot", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 60,
      selectedGroupId: "btc-core",
      groups: [
        {
          id: "btc-core",
          symbol: "BTCUSDT",
          signalTypesText: "vegas,divMacd",
          selectedTimelinePeriod: "60",
        },
      ],
    });

    useAppEventsMock.mockReturnValue({
      snapshot: {
        bootstrapRequired: false,
        config,
        rawResponse: null,
        health: {
          status: "success",
          pollingIntervalSeconds: 60,
          isStale: false,
        },
        diagnostics: {
          source: "system",
          code: "CONFIG_READY",
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
          activeAlert: null,
          pendingAlerts: [],
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
      },
      isSaving: false,
      submitError: null,
      saveConfig: vi.fn(),
      pollNow: vi.fn(),
      selectGroup: vi.fn(),
      clearDashboardFocusIntent: vi.fn(),
    });

    render(<MainDashboardPage />);

    expect(screen.getByText("Watch groups")).toBeInTheDocument();
    expect(screen.getByText("Current group detail")).toBeInTheDocument();
    expect(screen.getByText("Layout & window policy")).toBeInTheDocument();
  });

  it("shows an empty-state detail panel when config has no groups", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 60,
      groups: [],
    });

    useAppEventsMock.mockReturnValue({
      snapshot: {
        bootstrapRequired: false,
        config,
        rawResponse: null,
        health: {
          status: "success",
          pollingIntervalSeconds: 60,
          isStale: false,
        },
        diagnostics: {
          source: "system",
          code: "CONFIG_READY",
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
          activeAlert: null,
          pendingAlerts: [],
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
      },
      isSaving: false,
      submitError: null,
      saveConfig: vi.fn(),
      pollNow: vi.fn(),
      selectGroup: vi.fn(),
      clearDashboardFocusIntent: vi.fn(),
    });

    render(<MainDashboardPage />);

    expect(screen.getByText("No active watch group")).toBeInTheDocument();
    expect(screen.getByText("No watch groups yet")).toBeInTheDocument();
  });
});
