import type { AppSnapshot } from "./alert-model";

export const APP_SNAPSHOT_EVENT = "watch-tower://snapshot-updated";

export interface SnapshotEventPayload {
  snapshot: AppSnapshot;
}
