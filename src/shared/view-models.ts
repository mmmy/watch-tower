import type {
  AlertPayload,
  AlertPopupStream,
  AppSnapshot,
  DashboardFocusIntent,
  NormalizedGroupSnapshot,
  NormalizedSignal,
  PollingStatus,
} from "./alert-model";
import { normalizeGroupSnapshot } from "./alert-model";
import { describeWidgetFallback, type WidgetBehaviorRuntime } from "./window-state";

export interface GroupViewModel {
  groupSnapshot: ReturnType<typeof normalizeGroupSnapshot>;
  selectedSignal: NormalizedSignal | null;
  activeSignalType: string;
  activePeriod: string;
}

export interface ResidentWidgetViewModel {
  state: "ready" | "bootstrapRequired" | "noGroups";
  groupSnapshot: NormalizedGroupSnapshot | null;
  runtimeStatus: PollingStatus;
  widgetRuntime: WidgetBehaviorRuntime;
  widgetFallback: string | null;
}

export interface AlertPopupViewModel {
  state: "idle" | "active";
  symbol: string | null;
  alert: AlertPayload | null;
  queuedCount: number;
  streamSize: number;
  runtimeStatus: PollingStatus;
  isPendingRead: boolean;
}

export interface DashboardRecoveryItemViewModel {
  alert: AlertPayload;
  source: "visible" | "queued";
  isPendingRead: boolean;
}

export function getSnapshotRuntimeStatus(snapshot: AppSnapshot): PollingStatus {
  return snapshot.runtime.pollingPaused ? "paused" : snapshot.health.status;
}

export function buildGroupViewModel(
  snapshot: AppSnapshot,
  activePeriod?: string,
  activeSignalType?: string,
  nowMs = Date.now(),
): GroupViewModel | null {
  if (!snapshot.config || snapshot.config.groups.length === 0) {
    return null;
  }

  const groupSnapshot = normalizeGroupSnapshot(
    snapshot.config,
    snapshot.config.selectedGroupId,
    snapshot.rawResponse,
    nowMs,
  );
  const resolvedSignalType =
    activeSignalType ?? groupSnapshot.group.signalTypes[0] ?? "vegas";
  const resolvedPeriod =
    activePeriod ?? groupSnapshot.group.selectedTimelinePeriod ?? groupSnapshot.periods[0]?.period;
  const selectedPeriodSnapshot = groupSnapshot.periods.find(
    (periodSnapshot) => periodSnapshot.period === resolvedPeriod,
  );
  const selectedSignal = selectedPeriodSnapshot?.signals[resolvedSignalType] ?? null;

  return {
    groupSnapshot,
    selectedSignal,
    activeSignalType: resolvedSignalType,
    activePeriod: resolvedPeriod ?? "",
  };
}

export function buildResidentWidgetViewModel(
  snapshot: AppSnapshot,
  nowMs = Date.now(),
): ResidentWidgetViewModel {
  if (snapshot.bootstrapRequired || !snapshot.config) {
    return {
      state: "bootstrapRequired",
      groupSnapshot: null,
      runtimeStatus: getSnapshotRuntimeStatus(snapshot),
      widgetRuntime: snapshot.widgetRuntime,
      widgetFallback: describeWidgetFallback(snapshot.widgetRuntime),
    };
  }

  if (snapshot.config.groups.length === 0) {
    return {
      state: "noGroups",
      groupSnapshot: null,
      runtimeStatus: getSnapshotRuntimeStatus(snapshot),
      widgetRuntime: snapshot.widgetRuntime,
      widgetFallback: describeWidgetFallback(snapshot.widgetRuntime),
    };
  }

  return {
    state: "ready",
    groupSnapshot: normalizeGroupSnapshot(
      snapshot.config,
      snapshot.config.selectedGroupId,
      snapshot.rawResponse,
      nowMs,
    ),
    runtimeStatus: getSnapshotRuntimeStatus(snapshot),
    widgetRuntime: snapshot.widgetRuntime,
    widgetFallback: describeWidgetFallback(snapshot.widgetRuntime),
  };
}

export function buildAlertPopupViewModel(
  snapshot: AppSnapshot,
  symbol?: string,
): AlertPopupViewModel {
  const stream = resolvePopupStream(snapshot.alertRuntime.visiblePopupStreams, snapshot.alertRuntime.queuedPopupStreams, symbol);
  const activeAlert = stream?.alerts[0] ?? null;
  const pendingAlertId = snapshot.alertRuntime.pendingRead?.alert.id;

  return {
    state: activeAlert ? "active" : "idle",
    symbol: stream?.symbol ?? null,
    alert: activeAlert,
    queuedCount: Math.max((stream?.alerts.length ?? 0) - 1, 0),
    streamSize: stream?.alerts.length ?? 0,
    runtimeStatus: getSnapshotRuntimeStatus(snapshot),
    isPendingRead: Boolean(activeAlert && pendingAlertId === activeAlert.id),
  };
}

export function buildDashboardRecoveryViewModel(
  snapshot: AppSnapshot,
): DashboardRecoveryItemViewModel[] {
  const pendingAlertId = snapshot.alertRuntime.pendingRead?.alert.id;

  return [
    ...snapshot.alertRuntime.visiblePopupStreams.flatMap((stream) =>
      stream.alerts.map((alert) => ({
        alert,
        source: "visible" as const,
        isPendingRead: pendingAlertId === alert.id,
      })),
    ),
    ...snapshot.alertRuntime.queuedPopupStreams.flatMap((stream) =>
      stream.alerts.map((alert) => ({
        alert,
        source: "queued" as const,
        isPendingRead: pendingAlertId === alert.id,
      })),
    ),
  ];
}

export function getDashboardFocusIntent(
  snapshot: AppSnapshot,
): DashboardFocusIntent | null {
  return snapshot.alertRuntime.dashboardFocusIntent;
}

function resolvePopupStream(
  visibleStreams: AlertPopupStream[],
  queuedStreams: AlertPopupStream[],
  symbol?: string,
): AlertPopupStream | null {
  if (symbol) {
    return (
      visibleStreams.find((stream) => stream.symbol.toUpperCase() === symbol.toUpperCase()) ??
      queuedStreams.find((stream) => stream.symbol.toUpperCase() === symbol.toUpperCase()) ??
      null
    );
  }

  return visibleStreams[0] ?? queuedStreams[0] ?? null;
}
