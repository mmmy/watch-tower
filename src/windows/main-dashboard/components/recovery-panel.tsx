import type { DashboardRecoveryItemViewModel } from "../../../shared/view-models";
import { UnreadQueue } from "./unread-queue";

interface RecoveryPanelProps {
  items: DashboardRecoveryItemViewModel[];
  onOpenInDashboard: (alertId: string) => void;
  onMarkRead: (alertId: string) => void;
}

export function RecoveryPanel({
  items,
  onOpenInDashboard,
  onMarkRead,
}: RecoveryPanelProps) {
  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h3>Unread recovery</h3>
          <div className="section__subtle">
            Recover alerts that are still visible or queued across popup streams.
          </div>
        </div>
        <span className="status-chip status-chip--neutral">{items.length} items</span>
      </div>

      <UnreadQueue
        items={items}
        onOpenInDashboard={onOpenInDashboard}
        onMarkRead={onMarkRead}
      />
    </section>
  );
}
