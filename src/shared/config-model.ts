export const MIN_POLLING_INTERVAL_SECONDS = 10;

export const PERIOD_ORDER = [
  "10D",
  "W",
  "4D",
  "3D",
  "2D",
  "D",
  "720",
  "480",
  "360",
  "240",
  "180",
  "120",
  "90",
  "60",
  "45",
  "30",
  "20",
  "15",
  "10",
  "8",
  "5",
  "4",
  "3",
  "2",
  "1",
] as const;

export type SupportedPeriod = (typeof PERIOD_ORDER)[number];

export interface WatchGroupConfig {
  id: string;
  symbol: string;
  signalTypes: string[];
  periods: SupportedPeriod[];
  selectedTimelinePeriod: SupportedPeriod;
}

export interface AppConfig {
  apiBaseUrl: string;
  apiKey: string;
  pollingIntervalSeconds: number;
  selectedGroupId: string;
  groups: WatchGroupConfig[];
}

export interface AppConfigInput {
  apiBaseUrl: string;
  apiKey: string;
  pollingIntervalSeconds: number;
  symbol: string;
  signalTypesText: string;
}

export function sanitizeConfigInput(input: AppConfigInput): AppConfig {
  const signalTypes = input.signalTypesText
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
  const normalizedSymbol = input.symbol.trim().toUpperCase();
  const baseUrl = input.apiBaseUrl.trim().replace(/\/+$/, "");
  const pollingIntervalSeconds = Math.max(
    MIN_POLLING_INTERVAL_SECONDS,
    Math.floor(input.pollingIntervalSeconds || MIN_POLLING_INTERVAL_SECONDS),
  );

  if (!baseUrl) {
    throw new Error("API base URL is required.");
  }

  if (!input.apiKey.trim()) {
    throw new Error("API key is required.");
  }

  if (!normalizedSymbol) {
    throw new Error("At least one symbol is required.");
  }

  if (signalTypes.length === 0) {
    throw new Error("At least one signal type is required.");
  }

  const groupId = normalizedSymbol.toLowerCase();

  return {
    apiBaseUrl: baseUrl,
    apiKey: input.apiKey.trim(),
    pollingIntervalSeconds,
    selectedGroupId: groupId,
    groups: [
      {
        id: groupId,
        symbol: normalizedSymbol,
        signalTypes,
        periods: [...PERIOD_ORDER],
        selectedTimelinePeriod: "60",
      },
    ],
  };
}
