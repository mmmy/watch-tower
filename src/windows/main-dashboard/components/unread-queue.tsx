import type { DashboardRecoveryItemViewModel } from "../../../shared/view-models";

interface UnreadQueueProps {
  items: DashboardRecoveryItemViewModel[];
  onOpenInDashboard: (alertId: string) => void;
  onMarkRead: (alertId: string) => void;
}

export function UnreadQueue({
  items,
  onOpenInDashboard,
  onMarkRead,
}: UnreadQueueProps) {
  if (items.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state__title">No unread recovery items</div>
        <div className="empty-state__body">
          Active and queued popup streams will surface here when alerts still need attention.
        </div>
      </div>
    );
  }

  return (
    <div className="summary-list">
      {items.map(({ alert, source, isPendingRead }) => (
        <div className="summary-item" key={alert.id}>
          <div className="summary-item__label">
            {source === "visible" ? "Visible stream" : "Queued stream"}
          </div>
          <div className="summary-item__value">
            {alert.symbol} · {alert.period} · {alert.signalType}
          </div>
          <div className="section__subtle">
            {alert.side > 0 ? "Bullish" : "Bearish"} · {isPendingRead ? "marking as read" : "needs action"}
          </div>
          <div className="actions">
            <button
              type="button"
              className="button button--secondary"
              onClick={() => onOpenInDashboard(alert.id)}
            >
              Open detail
            </button>
            <button
              type="button"
              className="button button--primary"
              onClick={() => onMarkRead(alert.id)}
              disabled={isPendingRead}
            >
              {isPendingRead ? "Marking..." : "Mark read"}
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
