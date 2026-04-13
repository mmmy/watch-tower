import { describe, expect, it } from "vitest";
import type { AppSnapshot } from "./alert-model";
import { sanitizeConfigInput } from "./config-model";
import {
  buildAlertPopupViewModel,
  buildDashboardRecoveryViewModel,
  buildGroupViewModel,
  buildResidentWidgetViewModel,
  getSnapshotRuntimeStatus,
} from "./view-models";
import type { WidgetBehaviorRuntime } from "./window-state";

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
      visiblePopupStreams: [],
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
    } satisfies WidgetBehaviorRuntime,
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
    expect(widgetView.widgetRuntime.mode).toBe("passive");
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

  it("surfaces widget fallback details alongside the resident projection", () => {
    const snapshot = createSnapshot({
      widgetRuntime: {
        mode: "passive",
        placement: "visible",
        clickThroughEnabled: false,
        clickThroughSupported: false,
        fallbackReason: "Passive click-through is not enabled on this platform build.",
        wakeSource: null,
        interactionSessionId: 3,
        idleDeadlineAt: null,
      },
    });

    const widgetView = buildResidentWidgetViewModel(snapshot);

    expect(widgetView.widgetFallback).toContain("Passive click-through");
    expect(widgetView.widgetRuntime.clickThroughSupported).toBe(false);
  });

  it("builds an active popup view when an alert is present", () => {
    const snapshot = createSnapshot({
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
    expect(popupView.symbol).toBe("BTCUSDT");
    expect(popupView.alert?.id).toBe("BTCUSDT:60:vegas");
    expect(popupView.isPendingRead).toBe(true);
  });

  it("selects the matching popup stream by symbol and exposes stream queue metadata", () => {
    const snapshot = createSnapshot({
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
                signalAt: 2_000,
              },
            ],
          },
          {
            symbol: "ETHUSDT",
            alerts: [
              {
                id: "ETHUSDT:240:divMacd",
                groupId: "eth-swing",
                symbol: "ETHUSDT",
                period: "240",
                signalType: "divMacd",
                side: -1,
                signalAt: 1_500,
              },
              {
                id: "ETHUSDT:60:vegas",
                groupId: "eth-swing",
                symbol: "ETHUSDT",
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
    });

    const popupView = buildAlertPopupViewModel(snapshot, "ETHUSDT");

    expect(popupView.symbol).toBe("ETHUSDT");
    expect(popupView.alert?.id).toBe("ETHUSDT:240:divMacd");
    expect(popupView.queuedCount).toBe(1);
    expect(popupView.streamSize).toBe(2);
  });

  it("builds a recovery projection across visible and queued popup streams", () => {
    const snapshot = createSnapshot({
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
                signalAt: 2_000,
              },
            ],
          },
        ],
        queuedPopupStreams: [
          {
            symbol: "ETHUSDT",
            alerts: [
              {
                id: "ETHUSDT:240:divMacd",
                groupId: "eth-swing",
                symbol: "ETHUSDT",
                period: "240",
                signalType: "divMacd",
                side: -1,
                signalAt: 1_500,
              },
            ],
          },
        ],
        pendingRead: null,
        dashboardFocusIntent: null,
      },
    });

    const recoveryItems = buildDashboardRecoveryViewModel(snapshot);

    expect(recoveryItems).toHaveLength(2);
    expect(recoveryItems[0]?.source).toBe("visible");
    expect(recoveryItems[1]?.source).toBe("queued");
    expect(recoveryItems[1]?.alert.symbol).toBe("ETHUSDT");
  });
});
