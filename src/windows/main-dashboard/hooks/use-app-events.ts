import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSnapshot } from "../../../shared/alert-model";
import {
  sanitizeConfigInput,
  type AppConfig,
  type AppConfigInput,
} from "../../../shared/config-model";
import { APP_SNAPSHOT_EVENT, type SnapshotEventPayload } from "../../../shared/events";

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function createFallbackSnapshot(configOverride?: AppConfig): AppSnapshot {
  const config =
    configOverride ??
    sanitizeConfigInput({
      apiBaseUrl: "https://api.example.com",
      apiKey: "demo-key",
      pollingIntervalSeconds: 60,
      selectedGroupId: "eth-swing",
      layoutPreset: "table",
      density: "comfortable",
      windowPolicy: {
        dockSide: "right",
        widgetWidth: 280,
        widgetHeight: 720,
        topOffset: 96,
      },
      groups: [
        {
          id: "btc-core",
          symbol: "BTCUSDT",
          signalTypesText: "vegas,divMacd,tdMd",
          selectedTimelinePeriod: "60",
        },
        {
          id: "eth-swing",
          symbol: "ETHUSDT",
          signalTypesText: "vegas,divMacd",
          selectedTimelinePeriod: "240",
        },
      ],
    });
  const now = Date.now();

  return {
    bootstrapRequired: false,
    config,
    rawResponse: {
      total: 3,
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
            vegas: { sd: 1, t: now - 2 * 60 * 60 * 1000, read: false },
            divMacd: { sd: -1, t: now - 18 * 60 * 60 * 1000, read: true },
            tdMd: { sd: 1, t: now - 4 * 60 * 60 * 1000, read: false },
          },
        },
        {
          symbol: "BTCUSDT",
          period: "15",
          t: now,
          signals: {
            vegas: { sd: -1, t: now - 11 * 15 * 60 * 1000, read: true },
            divMacd: { sd: 1, t: now - 3 * 15 * 60 * 1000, read: false },
          },
        },
        {
          symbol: "ETHUSDT",
          period: "240",
          t: now,
          signals: {
            vegas: { sd: -1, t: now - 3 * 240 * 60 * 1000, read: true },
            divMacd: { sd: 1, t: now - 240 * 60 * 1000, read: false },
          },
        },
        {
          symbol: "ETHUSDT",
          period: "60",
          t: now,
          signals: {
            vegas: { sd: -1, t: now - 5 * 60 * 60 * 1000, read: true },
            divMacd: { sd: 1, t: now - 2 * 60 * 60 * 1000, read: false },
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
      message: "Browser preview is showing a mocked snapshot because Tauri runtime is not attached.",
      lastAttemptAt: now - 12_000,
      lastSuccessfulSyncAt: now - 12_000,
      nextRetryAt: null,
    },
  };
}

export function useAppEvents() {
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

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
      setSubmitError(
        error instanceof Error ? error.message : "Failed to load bootstrap state.",
      );
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const saveConfig = useCallback(async (input: AppConfigInput) => {
    setSubmitError(null);
    setIsSaving(true);

    try {
      const nextConfig = sanitizeConfigInput(input);

      if (!isTauriRuntime()) {
        setSnapshot({
          ...createFallbackSnapshot(nextConfig),
          config: nextConfig,
        });
        return;
      }

      const nextSnapshot = await invoke<AppSnapshot>("save_config", { input: nextConfig });
      setSnapshot(nextSnapshot);
    } catch (error: unknown) {
      setSubmitError(error instanceof Error ? error.message : "Failed to save config.");
      throw error;
    } finally {
      setIsSaving(false);
    }
  }, []);

  const pollNow = useCallback(async () => {
    if (!isTauriRuntime()) {
      setSnapshot(createFallbackSnapshot());
      return;
    }

    await invoke<AppSnapshot>("poll_now");
  }, []);

  const selectGroup = useCallback(async (groupId: string) => {
    if (!isTauriRuntime()) {
      setSnapshot((currentSnapshot) => {
        if (!currentSnapshot?.config) {
          return currentSnapshot;
        }

        return createFallbackSnapshot({
          ...currentSnapshot.config,
          selectedGroupId: groupId,
        });
      });
      return;
    }

    const nextSnapshot = await invoke<AppSnapshot>("select_group", { groupId });
    setSnapshot(nextSnapshot);
  }, []);

  return useMemo(
    () => ({
      snapshot,
      isSaving,
      submitError,
      saveConfig,
      pollNow,
      selectGroup,
    }),
    [snapshot, isSaving, submitError, saveConfig, pollNow, selectGroup],
  );
}
