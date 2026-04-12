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

export type DockSide = "left" | "right";
export type DashboardLayoutPreset = "list" | "table";
export type DashboardDensity = "comfortable" | "compact";

export interface WatchGroupConfig {
  id: string;
  symbol: string;
  signalTypes: string[];
  periods: SupportedPeriod[];
  selectedTimelinePeriod: SupportedPeriod;
}

export interface DashboardPreferences {
  layoutPreset: DashboardLayoutPreset;
  density: DashboardDensity;
}

export interface WindowPolicyConfig {
  dockSide: DockSide;
  widgetWidth: number;
  widgetHeight: number;
  topOffset: number;
}

export interface AppConfig {
  apiBaseUrl: string;
  apiKey: string;
  pollingIntervalSeconds: number;
  notificationsEnabled: boolean;
  selectedGroupId: string;
  groups: WatchGroupConfig[];
  dashboard: DashboardPreferences;
  windowPolicy: WindowPolicyConfig;
}

export interface WatchGroupInput {
  id?: string;
  symbol: string;
  signalTypesText: string;
  periods?: SupportedPeriod[];
  selectedTimelinePeriod?: SupportedPeriod;
}

export interface AppConfigInput {
  apiBaseUrl: string;
  apiKey: string;
  pollingIntervalSeconds: number;
  notificationsEnabled?: boolean;
  symbol?: string;
  signalTypesText?: string;
  selectedGroupId?: string;
  groups?: WatchGroupInput[];
  layoutPreset?: DashboardLayoutPreset;
  density?: DashboardDensity;
  windowPolicy?: Partial<WindowPolicyConfig>;
}

export function createWatchGroupInput(
  partial?: Partial<WatchGroupInput>,
): WatchGroupInput {
  return {
    id: partial?.id,
    symbol: partial?.symbol ?? "",
    signalTypesText: partial?.signalTypesText ?? "vegas",
    periods: partial?.periods ? [...partial.periods] : [...PERIOD_ORDER],
    selectedTimelinePeriod: partial?.selectedTimelinePeriod ?? "60",
  };
}

export function sanitizeConfigInput(input: AppConfigInput): AppConfig {
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

  const groups = buildGroups(input);
  const selectedGroupId = resolveSelectedGroupId(groups, input.selectedGroupId);

  return {
    apiBaseUrl: baseUrl,
    apiKey: input.apiKey.trim(),
    pollingIntervalSeconds,
    notificationsEnabled: input.notificationsEnabled ?? true,
    selectedGroupId,
    groups,
    dashboard: {
      layoutPreset: input.layoutPreset ?? "table",
      density: input.density ?? "comfortable",
    },
    windowPolicy: {
      dockSide: input.windowPolicy?.dockSide ?? "right",
      widgetWidth: sanitizePositiveNumber(input.windowPolicy?.widgetWidth, 280),
      widgetHeight: sanitizePositiveNumber(input.windowPolicy?.widgetHeight, 720),
      topOffset: sanitizePositiveNumber(input.windowPolicy?.topOffset, 96),
    },
  };
}

export function toConfigInput(config: AppConfig): AppConfigInput {
  return {
    apiBaseUrl: config.apiBaseUrl,
    apiKey: config.apiKey,
    pollingIntervalSeconds: config.pollingIntervalSeconds,
    notificationsEnabled: config.notificationsEnabled,
    selectedGroupId: config.selectedGroupId,
    layoutPreset: config.dashboard.layoutPreset,
    density: config.dashboard.density,
    windowPolicy: { ...config.windowPolicy },
    groups: config.groups.map((group) =>
      createWatchGroupInput({
        id: group.id,
        symbol: group.symbol,
        signalTypesText: group.signalTypes.join(","),
        periods: [...group.periods],
        selectedTimelinePeriod: group.selectedTimelinePeriod,
      }),
    ),
  };
}

function buildGroups(input: AppConfigInput): WatchGroupConfig[] {
  if (input.groups) {
    const usedIds = new Set<string>();
    return input.groups.map((groupInput, index) => sanitizeGroupInput(groupInput, index, usedIds));
  }

  const groupInputs = [
    {
      symbol: input.symbol ?? "",
      signalTypesText: input.signalTypesText ?? "",
    },
  ];

  const usedIds = new Set<string>();
  return groupInputs.map((groupInput, index) => sanitizeGroupInput(groupInput, index, usedIds));
}

function sanitizeGroupInput(
  input: WatchGroupInput,
  index: number,
  usedIds: Set<string>,
): WatchGroupConfig {
  const normalizedSymbol = input.symbol.trim().toUpperCase();

  if (!normalizedSymbol) {
    throw new Error("Each group requires exactly one symbol.");
  }

  const signalTypes = input.signalTypesText
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean)
    .filter((value, valueIndex, allValues) => allValues.indexOf(value) === valueIndex);

  if (signalTypes.length === 0) {
    throw new Error(`Group ${normalizedSymbol} requires at least one signal type.`);
  }

  const periods = sanitizePeriods(input.periods);
  const selectedTimelinePeriod = periods.includes(input.selectedTimelinePeriod ?? "60")
    ? (input.selectedTimelinePeriod ?? "60")
    : periods[0];

  const proposedId = (input.id?.trim() || normalizedSymbol.toLowerCase()).replace(/\s+/g, "-");
  const id = ensureUniqueGroupId(proposedId, index, usedIds);

  return {
    id,
    symbol: normalizedSymbol,
    signalTypes,
    periods,
    selectedTimelinePeriod,
  };
}

function sanitizePeriods(periods?: SupportedPeriod[]): SupportedPeriod[] {
  if (!periods || periods.length === 0) {
    return [...PERIOD_ORDER];
  }

  const deduped = periods.filter(
    (period, index) => PERIOD_ORDER.includes(period) && periods.indexOf(period) === index,
  );

  if (deduped.length === 0) {
    return [...PERIOD_ORDER];
  }

  return PERIOD_ORDER.filter((period) => deduped.includes(period));
}

function ensureUniqueGroupId(baseId: string, index: number, usedIds: Set<string>): string {
  const safeBase = baseId || `group-${index + 1}`;

  if (!usedIds.has(safeBase)) {
    usedIds.add(safeBase);
    return safeBase;
  }

  let suffix = 2;
  let nextId = `${safeBase}-${suffix}`;

  while (usedIds.has(nextId)) {
    suffix += 1;
    nextId = `${safeBase}-${suffix}`;
  }

  usedIds.add(nextId);
  return nextId;
}

function resolveSelectedGroupId(groups: WatchGroupConfig[], requestedId?: string): string {
  if (groups.length === 0) {
    return "";
  }

  const normalizedRequestedId = requestedId?.trim();

  if (normalizedRequestedId && groups.some((group) => group.id === normalizedRequestedId)) {
    return normalizedRequestedId;
  }

  return groups[0]?.id ?? "";
}

function sanitizePositiveNumber(value: number | undefined, fallback: number): number {
  if (!value || Number.isNaN(value)) {
    return fallback;
  }

  return Math.max(1, Math.floor(value));
}
