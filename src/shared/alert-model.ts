import type { AppConfig, SupportedPeriod, WatchGroupConfig } from "./config-model";
import { computeBarIndex, sortPeriods } from "./period-utils";

export interface ApiSignalEntry {
  sd: 1 | -1;
  t: number;
  read: boolean;
}

export interface ApiSignalRecord {
  symbol: string;
  period: string;
  t: number;
  signals?: Record<string, ApiSignalEntry | undefined>;
}

export interface ApiSignalsResponse {
  total: number;
  page: number;
  pageSize: number;
  data: ApiSignalRecord[];
}

export type PollingStatus =
  | "bootstrapRequired"
  | "idle"
  | "polling"
  | "paused"
  | "success"
  | "authError"
  | "backoff"
  | "configError"
  | "requestError";

export interface RuntimeInfo {
  pollingPaused: boolean;
  lastActiveStatus: PollingStatus | null;
}

export interface DiagnosticsInfo {
  source: "system" | "config" | "request" | "normalization";
  code: string | null;
  message: string;
  lastAttemptAt: number | null;
  lastSuccessfulSyncAt: number | null;
  nextRetryAt: number | null;
}

export interface PollingHealth {
  status: PollingStatus;
  pollingIntervalSeconds: number | null;
  isStale: boolean;
}

export interface AppSnapshot {
  bootstrapRequired: boolean;
  config: AppConfig | null;
  rawResponse: ApiSignalsResponse | null;
  health: PollingHealth;
  diagnostics: DiagnosticsInfo;
  runtime: RuntimeInfo;
}

export interface NormalizedSignal {
  signalType: string;
  period: SupportedPeriod;
  side: 1 | -1;
  read: boolean;
  signalAt: number;
  barIndex: number | null;
}

export interface NormalizedPeriodSnapshot {
  period: SupportedPeriod;
  symbol: string;
  signals: Record<string, NormalizedSignal | null>;
}

export interface NormalizedGroupSnapshot {
  group: WatchGroupConfig;
  periods: NormalizedPeriodSnapshot[];
  issues: string[];
}

export function normalizeGroupSnapshot(
  config: AppConfig,
  groupId: string,
  rawResponse: ApiSignalsResponse | null,
  nowMs: number,
): NormalizedGroupSnapshot {
  const group = config.groups.find((candidate) => candidate.id === groupId) ?? config.groups[0];

  if (!group) {
    throw new Error("No group configuration available.");
  }

  const issues: string[] = [];
  const periodLookup = new Map<string, ApiSignalRecord>();

  for (const item of rawResponse?.data ?? []) {
    if (item.symbol.toUpperCase() !== group.symbol.toUpperCase()) {
      continue;
    }

    periodLookup.set(item.period, item);
  }

  const periods = sortPeriods(group.periods).map((period) => {
    const record = periodLookup.get(period);
    const signals = Object.fromEntries(
      group.signalTypes.map((signalType) => {
        const signal = record?.signals?.[signalType];

        if (!record?.signals) {
          return [signalType, null];
        }

        if (!signal) {
          return [signalType, null];
        }

        return [
          signalType,
          {
            signalType,
            period,
            side: signal.sd,
            read: signal.read,
            signalAt: signal.t,
            barIndex: computeBarIndex(signal.t, period, nowMs),
          } satisfies NormalizedSignal,
        ];
      }),
    ) as Record<string, NormalizedSignal | null>;

    if (record && !record.signals) {
      issues.push(`Record for ${period} is missing a signals object.`);
    }

    return {
      period,
      symbol: group.symbol,
      signals,
    } satisfies NormalizedPeriodSnapshot;
  });

  return {
    group,
    periods,
    issues,
  };
}
