import { AlertCard } from "./components/alert-card";
import { useAlertPopupEvents } from "./hooks/use-alert-popup-events";

export function AlertPopupPage() {
  const { popupView, submitError, markRead, openInDashboard } = useAlertPopupEvents();

  return (
    <main className="alert-popup-shell">
      <section className="alert-popup panel">
        <AlertCard
          popupView={
            popupView ?? {
              state: "idle",
              alert: null,
              runtimeStatus: "idle",
              isPendingRead: false,
            }
          }
          submitError={submitError}
          onMarkRead={() => {
            if (popupView?.alert) {
              void markRead(popupView.alert);
            }
          }}
          onOpenInDashboard={() => {
            if (popupView?.alert) {
              void openInDashboard(popupView.alert);
            }
          }}
        />
      </section>
    </main>
  );
}
