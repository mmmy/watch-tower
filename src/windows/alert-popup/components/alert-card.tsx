import type { AlertPopupViewModel } from "../../../shared/view-models";

interface AlertCardProps {
  popupView: AlertPopupViewModel;
  submitError?: string | null;
  onMarkRead: () => void;
  onOpenInDashboard: () => void;
}

export function AlertCard({
  popupView,
  submitError,
  onMarkRead,
  onOpenInDashboard,
}: AlertCardProps) {
  if (popupView.state !== "active" || !popupView.alert) {
    return (
      <div className="alert-popup__empty">
        <div className="alert-popup__eyebrow">Watch Tower Alert</div>
        <div className="alert-popup__title">No active alerts</div>
        <div className="alert-popup__body">New unread signals will appear here when the host runtime projects them into the popup surface.</div>
      </div>
    );
  }

  const sideLabel = popupView.alert.side > 0 ? "Bullish" : "Bearish";

  return (
    <div className="alert-popup__card">
      <div className="alert-popup__eyebrow">Watch Tower Alert</div>
      <div className="alert-popup__header">
        <div>
          <h1 className="alert-popup__title">{popupView.alert.symbol}</h1>
          <div className="alert-popup__meta">
            {popupView.alert.period} · {popupView.alert.signalType} · {sideLabel}
          </div>
        </div>
        <div className={`status-chip ${popupView.alert.side > 0 ? "status-chip--success" : "status-chip--danger"}`}>
          {sideLabel}
        </div>
      </div>

      <div className="alert-popup__body">
        {popupView.isPendingRead
          ? "Marking this alert as read. Keeping the card visible until the host confirms the write."
          : "A newly unread signal was detected. Open the dashboard for detail, or mark it as read directly from the popup."}
      </div>

      {submitError ? <div className="error-banner">{submitError}</div> : null}

      <div className="alert-popup__actions">
        <button
          type="button"
          className="button button--secondary"
          onClick={onOpenInDashboard}
        >
          Open in dashboard
        </button>
        <button
          type="button"
          className="button button--primary"
          onClick={onMarkRead}
          disabled={popupView.isPendingRead}
        >
          {popupView.isPendingRead ? "Marking..." : "Mark read"}
        </button>
      </div>
    </div>
  );
}
