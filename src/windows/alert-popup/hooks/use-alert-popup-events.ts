import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AlertPayload, AppSnapshot } from "../../../shared/alert-model";
import { sanitizeConfigInput } from "../../../shared/config-model";
import { APP_SNAPSHOT_EVENT, type SnapshotEventPayload } from "../../../shared/events";
import { buildAlertPopupViewModel } from "../../../shared/view-models";

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function currentPopupSymbol() {
  if (typeof window === "undefined") {
    return "ETHUSDT";
  }

  const label = (window as typeof window & {
    __TAURI_INTERNALS__?: {
      metadata?: {
        currentWindow?: {
          label?: string;
        };
      };
    };
  }).__TAURI_INTERNALS__?.metadata?.currentWindow?.label;

  if (typeof label === "string" && label.startsWith("alert-popup:")) {
    return label.slice("alert-popup:".length);
  }

  return "ETHUSDT";
}

function createFallbackSnapshot(): AppSnapshot {
  const now = Date.now();
  const activeAlert: AlertPayload = {
    id: "ETHUSDT:240:divMacd",
    groupId: "eth-swing",
    symbol: "ETHUSDT",
    period: "240",
    signalType: "divMacd",
    side: 1,
    signalAt: now - 240 * 60 * 1000,
  };

  return {
    bootstrapRequired: false,
    config: sanitizeConfigInput({
      apiBaseUrl: "https://api.example.com",
      apiKey: "demo-key",
      pollingIntervalSeconds: 60,
      notificationsEnabled: true,
      selectedGroupId: "btc-core",
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
          signalTypesText: "vegas,divMacd",
          selectedTimelinePeriod: "240",
        },
        {
          id: "sol-scalp",
          symbol: "SOLUSDT",
          signalTypesText: "vegas",
          selectedTimelinePeriod: "15",
        },
      ],
    }),
    rawResponse: null,
    health: {
      status: "success",
      pollingIntervalSeconds: 60,
      isStale: false,
    },
    diagnostics: {
      source: "system",
      code: "BROWSER_PREVIEW",
      message: "Browser preview is showing a mocked alert snapshot because Tauri runtime is not attached.",
      lastAttemptAt: now - 12_000,
      lastSuccessfulSyncAt: now - 12_000,
      nextRetryAt: null,
    },
    runtime: {
      pollingPaused: false,
      lastActiveStatus: null,
    },
    alertRuntime: {
      visiblePopupStreams: [
        {
          symbol: "ETHUSDT",
          alerts: [activeAlert],
        },
      ],
      queuedPopupStreams: [
        {
          symbol: "BTCUSDT",
          alerts: [
            {
              id: "BTCUSDT:60:vegas",
              groupId: "btc-core",
              symbol: "BTCUSDT",
              period: "60",
              signalType: "vegas",
              side: -1,
              signalAt: now - 60 * 60 * 1000,
            },
          ],
        },
        {
          symbol: "SOLUSDT",
          alerts: [
            {
              id: "SOLUSDT:15:vegas",
              groupId: "sol-scalp",
              symbol: "SOLUSDT",
              period: "15",
              signalType: "vegas",
              side: 1,
              signalAt: now - 15 * 60 * 1000,
            },
          ],
        },
      ],
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
  };
}

export function useAlertPopupEvents() {
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const popupSymbol = currentPopupSymbol();

  useEffect(() => {
    if (!isTauriRuntime()) {
      setSnapshot(createFallbackSnapshot());
      return undefined;
    }

    let disposed = false;
    let unlisten: (() => void) | undefined;

    async function bootstrap() {
      const initialSnapshot = await invoke<AppSnapshot>("get_bootstrap_state");

      if (!disposed) {
        setSnapshot(initialSnapshot);
      }

      unlisten = await listen<SnapshotEventPayload>(APP_SNAPSHOT_EVENT, (event) => {
        setSnapshot(event.payload.snapshot);
      });
    }

    bootstrap().catch((error: unknown) => {
      setSubmitError(error instanceof Error ? error.message : "Failed to load popup state.");
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const markRead = useCallback(async (alert: AlertPayload) => {
    setSubmitError(null);

    if (!isTauriRuntime()) {
      setSnapshot((currentSnapshot) => {
        if (!currentSnapshot) {
          return currentSnapshot;
        }

        return {
          ...currentSnapshot,
          alertRuntime: {
            ...currentSnapshot.alertRuntime,
            pendingRead: {
              alert,
              requestedAt: Date.now(),
            },
          },
        };
      });
      return;
    }

    try {
      const nextSnapshot = await invoke<AppSnapshot>("mark_alert_read", { alert });
      setSnapshot(nextSnapshot);
    } catch (error: unknown) {
      setSubmitError(error instanceof Error ? error.message : "Failed to mark alert as read.");
    }
  }, []);

  const openInDashboard = useCallback(async (alert: AlertPayload) => {
    setSubmitError(null);

    if (!isTauriRuntime()) {
      setSnapshot((currentSnapshot) =>
        currentSnapshot
          ? {
              ...currentSnapshot,
              alertRuntime: {
                ...currentSnapshot.alertRuntime,
                dashboardFocusIntent: {
                  alert,
                  requestedAt: Date.now(),
                },
                visiblePopupStreams: currentSnapshot.alertRuntime.visiblePopupStreams
                  .map((stream) =>
                    stream.symbol.toUpperCase() === alert.symbol.toUpperCase()
                      ? {
                          ...stream,
                          alerts: stream.alerts.filter(
                            (candidate) => candidate.id !== alert.id,
                          ),
                        }
                      : stream,
                  )
                  .filter((stream) => stream.alerts.length > 0),
                queuedPopupStreams: currentSnapshot.alertRuntime.queuedPopupStreams
                  .map((stream) =>
                    stream.symbol.toUpperCase() === alert.symbol.toUpperCase()
                      ? {
                          ...stream,
                          alerts: stream.alerts.filter(
                            (candidate) => candidate.id !== alert.id,
                          ),
                        }
                      : stream,
                  )
                  .filter((stream) => stream.alerts.length > 0),
              },
            }
          : currentSnapshot,
      );
      return;
    }

    try {
      const nextSnapshot = await invoke<AppSnapshot>("open_alert_in_dashboard", { alert });
      setSnapshot(nextSnapshot);
    } catch (error: unknown) {
      setSubmitError(error instanceof Error ? error.message : "Failed to open alert in dashboard.");
    }
  }, []);

  const popupView = useMemo(
    () => (snapshot ? buildAlertPopupViewModel(snapshot, popupSymbol) : null),
    [popupSymbol, snapshot],
  );

  return useMemo(
    () => ({
      snapshot,
      popupView,
      submitError,
      markRead,
      openInDashboard,
    }),
    [snapshot, popupView, submitError, markRead, openInDashboard],
  );
}
