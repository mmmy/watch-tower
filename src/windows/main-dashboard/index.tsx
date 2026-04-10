import { useMemo, useState } from "react";
import { buildGroupViewModel } from "../../shared/view-models";
import { BootstrapPanel } from "./components/bootstrap-panel";
import { ConfigSummary } from "./components/config-summary";
import { DiagnosticsPanel } from "./components/diagnostics-panel";
import { PeriodMatrixDebug } from "./components/period-matrix-debug";
import { PollingHealthPanel } from "./components/polling-health-panel";
import { Timeline60Debug } from "./components/timeline-60-debug";
import { useAppEvents } from "./hooks/use-app-events";

export function MainDashboardPage() {
  const { snapshot, isSaving, submitError, saveConfig, pollNow } = useAppEvents();
  const [activePeriod, setActivePeriod] = useState<string | undefined>(undefined);
  const [activeSignalType, setActiveSignalType] = useState<string | undefined>(undefined);

  const viewModel = useMemo(
    () => (snapshot ? buildGroupViewModel(snapshot, activePeriod, activeSignalType) : null),
    [snapshot, activePeriod, activeSignalType],
  );

  const bootstrapInitialValues = snapshot?.config
    ? {
        apiBaseUrl: snapshot.config.apiBaseUrl,
        apiKey: snapshot.config.apiKey,
        pollingIntervalSeconds: snapshot.config.pollingIntervalSeconds,
        symbol: snapshot.config.groups[0]?.symbol ?? "",
        signalTypesText: snapshot.config.groups[0]?.signalTypes.join(",") ?? "",
      }
    : undefined;

  return (
    <main className="shell">
      <div className="dashboard">
        <section className="dashboard__hero">
          <div className="panel hero-card">
            <span className="hero-card__eyebrow">Watch Tower v0.1</span>
            <div>
              <h1>Thin shell. Real signal chain.</h1>
              <p>
                This desktop verification surface exists to prove the host can load config, poll
                real alerts, normalize a single-symbol group, and make 25-level + 60-bar math
                legible before we invest in the full control console.
              </p>
            </div>
            <div className="hero-card__meta">
              <div className="hero-meta">
                <div className="hero-meta__label">Current mode</div>
                <div className="hero-meta__value">
                  {snapshot?.bootstrapRequired ? "Bootstrap required" : "Verification shell"}
                </div>
              </div>
              <div className="hero-meta">
                <div className="hero-meta__label">Focus</div>
                <div className="hero-meta__value">One group, one symbol, fixed 25 periods</div>
              </div>
              <div className="hero-meta">
                <div className="hero-meta__label">Runtime</div>
                <div className="hero-meta__value">{snapshot?.health.status ?? "loading"}</div>
              </div>
            </div>
          </div>

          <BootstrapPanel
            initialValues={bootstrapInitialValues}
            isSaving={isSaving}
            submitError={submitError}
            onSubmit={saveConfig}
          />
        </section>

        {snapshot && viewModel ? (
          <section className="panel-grid">
            <div className="stack">
              <ConfigSummary config={snapshot.config!} />
              <PollingHealthPanel
                diagnostics={snapshot.diagnostics}
                health={snapshot.health}
                onPollNow={pollNow}
              />
              <DiagnosticsPanel
                diagnostics={snapshot.diagnostics}
                issues={viewModel.groupSnapshot.issues}
              />
            </div>

            <div className="stack">
              <PeriodMatrixDebug
                activePeriod={viewModel.activePeriod}
                activeSignalType={viewModel.activeSignalType}
                snapshot={viewModel.groupSnapshot}
                onSelect={(period, signalType) => {
                  setActivePeriod(period);
                  setActiveSignalType(signalType);
                }}
              />
              <Timeline60Debug
                period={viewModel.activePeriod}
                signal={viewModel.selectedSignal}
                signalType={viewModel.activeSignalType}
              />
            </div>
          </section>
        ) : (
          <section className="panel section">
            <div className="empty-state">
              <div className="empty-state__title">Waiting for the first valid snapshot</div>
              <div className="empty-state__body">
                Save a valid config to start the host poller. Once the desktop runtime has a
                snapshot, this surface will render the single-group matrix, the 60-bar mapping,
                and host diagnostics in one place.
              </div>
            </div>
          </section>
        )}
      </div>
    </main>
  );
}
