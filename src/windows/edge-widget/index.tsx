import { EmptyState } from "./components/empty-state";
import { PeriodRow } from "./components/period-row";
import { StatusFooter } from "./components/status-footer";
import { useEdgeWidgetEvents } from "./hooks/use-edge-widget-events";

export function EdgeWidgetPage() {
  const {
    snapshot,
    widgetView,
    notifyPointerEnter,
    notifyPointerLeave,
    notifyInteraction,
  } = useEdgeWidgetEvents();

  return (
    <main className="edge-widget-shell">
      <section
        className="edge-widget"
        data-widget-mode={widgetView?.widgetRuntime.mode ?? snapshot?.widgetRuntime.mode ?? "loading"}
        data-widget-placement={
          widgetView?.widgetRuntime.placement ?? snapshot?.widgetRuntime.placement ?? "loading"
        }
        onMouseEnter={() => {
          void notifyPointerEnter();
        }}
        onMouseLeave={() => {
          void notifyPointerLeave();
        }}
        onMouseDown={() => {
          void notifyInteraction();
        }}
        onFocusCapture={() => {
          void notifyInteraction();
        }}
      >
        <header className="edge-widget__header">
          <div className="edge-widget__eyebrow">Watch Tower</div>
          <div>
            <h1>Resident watch</h1>
            <p>
              Keep the currently selected group visible without reopening the dashboard.
            </p>
          </div>
        </header>

        {widgetView?.state === "ready" && widgetView.groupSnapshot ? (
          <div className="edge-widget__body">
            <div className="edge-widget__group">
              <div className="edge-widget__symbol">{widgetView.groupSnapshot.group.symbol}</div>
              <div className="edge-widget__meta">
                {widgetView.groupSnapshot.group.signalTypes.join(" · ")}
              </div>
            </div>

            <div className="edge-widget__periods">
              {widgetView.groupSnapshot.periods.map((periodSnapshot) => (
                <PeriodRow
                  key={periodSnapshot.period}
                  periodSnapshot={periodSnapshot}
                  signalTypes={widgetView.groupSnapshot!.group.signalTypes}
                />
              ))}
            </div>
          </div>
        ) : widgetView?.state === "bootstrapRequired" ? (
          <EmptyState
            title="Bootstrap required"
            body="Save API credentials and at least one watch group in the dashboard to enable resident monitoring."
          />
        ) : widgetView?.state === "noGroups" ? (
          <EmptyState
            title="No watch groups yet"
            body="Add at least one single-symbol watch group in the dashboard to populate the resident widget."
          />
        ) : (
          <EmptyState
            title="Loading resident view"
            body="Waiting for the shared runtime snapshot to reach the widget."
          />
        )}

        <StatusFooter
          runtimeStatus={widgetView?.runtimeStatus ?? snapshot?.health.status ?? "loading"}
          isStale={snapshot?.health.isStale ?? false}
          widgetMode={widgetView?.widgetRuntime.mode ?? snapshot?.widgetRuntime.mode ?? "passive"}
          widgetPlacement={
            widgetView?.widgetRuntime.placement ?? snapshot?.widgetRuntime.placement ?? "hidden"
          }
          widgetFallback={widgetView?.widgetFallback ?? snapshot?.widgetRuntime.fallbackReason ?? null}
          diagnostics={
            snapshot?.diagnostics ?? {
              source: "system",
              code: null,
              message: "Resident widget is waiting for host diagnostics.",
              lastAttemptAt: null,
              lastSuccessfulSyncAt: null,
              nextRetryAt: null,
            }
          }
        />
      </section>
    </main>
  );
}
