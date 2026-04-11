import type { AppSnapshot, NormalizedSignal } from "./alert-model";
import { normalizeGroupSnapshot } from "./alert-model";

export interface GroupViewModel {
  groupSnapshot: ReturnType<typeof normalizeGroupSnapshot>;
  selectedSignal: NormalizedSignal | null;
  activeSignalType: string;
  activePeriod: string;
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
