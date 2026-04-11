import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSnapshot } from "../../../shared/alert-model";
import { sanitizeConfigInput } from "../../../shared/config-model";
import { APP_SNAPSHOT_EVENT, type SnapshotEventPayload } from "../../../shared/events";
import { buildResidentWidgetViewModel } from "../../../shared/view-models";

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function createFallbackSnapshot(): AppSnapshot {
  const config = sanitizeConfigInput({
    apiBaseUrl: "https://api.example.com",
    apiKey: "demo-key",
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
  const now = Date.now();

  return {
    bootstrapRequired: false,
    config,
    rawResponse: {
      total: 2,
      page: 1,
      pageSize: 25,
      data: [
        {
          symbol: "BTCUSDT",
          period: "240",
          t: now,
          signals: {
            vegas: { sd: 1, t: now - 2 * 240 * 60 * 1000, read: false },
            divMacd: { sd: -1, t: now - 6 * 240 * 60 * 1000, read: true },
          },
        },
        {
          symbol: "BTCUSDT",
          period: "60",
          t: now,
          signals: {
            vegas: { sd: -1, t: now - 2 * 60 * 60 * 1000, read: false },
            divMacd: { sd: 1, t: now - 4 * 60 * 60 * 1000, read: true },
          },
        },
      ],
    },
    health: {
      status: "success",
      pollingIntervalSeconds: 60,
      isStale: false,
    },
    diagnostics: {
      source: "system",
      code: "BROWSER_PREVIEW",
      message: "Browser preview is showing a mocked resident snapshot because Tauri runtime is not attached.",
      lastAttemptAt: now - 12_000,
      lastSuccessfulSyncAt: now - 12_000,
      nextRetryAt: null,
    },
    runtime: {
      pollingPaused: false,
      lastActiveStatus: null,
    },
  };
}

export function useEdgeWidgetEvents() {
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);

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

    bootstrap().catch(() => {
      if (!disposed) {
        setSnapshot(createFallbackSnapshot());
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const widgetView = useMemo(
    () => (snapshot ? buildResidentWidgetViewModel(snapshot) : null),
    [snapshot],
  );

  return useMemo(
    () => ({
      snapshot,
      widgetView,
    }),
    [snapshot, widgetView],
  );
}
