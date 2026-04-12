import { describe, expect, it } from "vitest";
import type { AppSnapshot } from "./alert-model";
import { sanitizeConfigInput } from "./config-model";
import {
  buildAlertPopupViewModel,
  buildGroupViewModel,
  buildResidentWidgetViewModel,
  getSnapshotRuntimeStatus,
} from "./view-models";

function createSnapshot(overrides?: Partial<AppSnapshot>): AppSnapshot {
  const config = sanitizeConfigInput({
    apiBaseUrl: "https://api.example.com",
    apiKey: "demo-key",
    pollingIntervalSeconds: 60,
    selectedGroupId: "eth-swing",
    groups: [
      {
        id: "btc-core",
        symbol: "BTCUSDT",
        signalTypesText: "vegas,divMacd",
        selectedTimelinePeriod: "60",
      },
      {
        id: "eth-swing",
        symbol: "ETHUSDT",
        signalTypesText: "vegas",
        selectedTimelinePeriod: "240",
      },
    ],
  });

  return {
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
      code: "SYNC_OK",
      message: "Latest signal snapshot loaded successfully.",
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
    ...overrides,
  };
}

describe("view-models", () => {
  it("builds a resident widget view for the selected group", () => {
    const snapshot = createSnapshot();

    const widgetView = buildResidentWidgetViewModel(snapshot);

    expect(widgetView.state).toBe("ready");
    expect(widgetView.groupSnapshot?.group.id).toBe("eth-swing");
    expect(widgetView.runtimeStatus).toBe("success");
  });

  it("surfaces paused as the resident runtime status without losing the selected group", () => {
    const snapshot = createSnapshot({
      runtime: {
        pollingPaused: true,
        lastActiveStatus: "success",
      },
    });

    const widgetView = buildResidentWidgetViewModel(snapshot);
    const dashboardView = buildGroupViewModel(snapshot);

    expect(getSnapshotRuntimeStatus(snapshot)).toBe("paused");
    expect(widgetView.runtimeStatus).toBe("paused");
    expect(widgetView.groupSnapshot?.group.id).toBe("eth-swing");
    expect(dashboardView?.groupSnapshot.group.id).toBe("eth-swing");
  });

  it("returns a no-groups resident state when config is present but empty", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://api.example.com",
      apiKey: "demo-key",
      pollingIntervalSeconds: 60,
      groups: [],
    });
    const snapshot = createSnapshot({
      config,
      health: {
        status: "configError",
        pollingIntervalSeconds: 60,
        isStale: false,
      },
      diagnostics: {
        source: "config",
        code: "NO_GROUPS",
        message: "Add at least one watch group to resume polling.",
        lastAttemptAt: null,
        lastSuccessfulSyncAt: null,
        nextRetryAt: null,
      },
    });

    const widgetView = buildResidentWidgetViewModel(snapshot);

    expect(widgetView.state).toBe("noGroups");
    expect(widgetView.groupSnapshot).toBeNull();
    expect(widgetView.runtimeStatus).toBe("configError");
  });

  it("builds an active popup view when an alert is present", () => {
    const snapshot = createSnapshot({
      alertRuntime: {
        activeAlert: {
          id: "BTCUSDT:60:vegas",
          groupId: "btc-core",
          symbol: "BTCUSDT",
          period: "60",
          signalType: "vegas",
          side: 1,
          signalAt: 1_000,
        },
        pendingAlerts: [],
        pendingRead: {
          alert: {
            id: "BTCUSDT:60:vegas",
            groupId: "btc-core",
            symbol: "BTCUSDT",
            period: "60",
            signalType: "vegas",
            side: 1,
            signalAt: 1_000,
          },
          requestedAt: 2_000,
        },
        dashboardFocusIntent: null,
      },
    });

    const popupView = buildAlertPopupViewModel(snapshot);

    expect(popupView.state).toBe("active");
    expect(popupView.alert?.id).toBe("BTCUSDT:60:vegas");
    expect(popupView.isPendingRead).toBe(true);
  });
});
