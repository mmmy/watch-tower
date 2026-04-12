import type {
  AlertPayload,
  AppSnapshot,
  DashboardFocusIntent,
  NormalizedGroupSnapshot,
  NormalizedSignal,
  PollingStatus,
} from "./alert-model";
import { normalizeGroupSnapshot } from "./alert-model";

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
}

export interface AlertPopupViewModel {
  state: "idle" | "active";
  alert: AlertPayload | null;
  runtimeStatus: PollingStatus;
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
    };
  }

  if (snapshot.config.groups.length === 0) {
    return {
      state: "noGroups",
      groupSnapshot: null,
      runtimeStatus: getSnapshotRuntimeStatus(snapshot),
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
  };
}

export function buildAlertPopupViewModel(snapshot: AppSnapshot): AlertPopupViewModel {
  const activeAlert = snapshot.alertRuntime.activeAlert;
  const pendingAlertId = snapshot.alertRuntime.pendingRead?.alert.id;

  return {
    state: activeAlert ? "active" : "idle",
    alert: activeAlert,
    runtimeStatus: getSnapshotRuntimeStatus(snapshot),
    isPendingRead: Boolean(activeAlert && pendingAlertId === activeAlert.id),
  };
}

export function getDashboardFocusIntent(
  snapshot: AppSnapshot,
): DashboardFocusIntent | null {
  return snapshot.alertRuntime.dashboardFocusIntent;
}
