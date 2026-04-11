import type { DiagnosticsInfo, PollingHealth } from "../../../shared/alert-model";
import { formatTimestamp } from "../../../shared/period-utils";

interface PollingHealthPanelProps {
  health: PollingHealth;
  diagnostics: DiagnosticsInfo;
  runtimeStatus?: string;
  onPollNow: () => void;
}

function getHealthChipClass(status: PollingHealth["status"]) {
  if (status === "success") {
    return "status-chip--success";
  }

  if (status === "backoff" || status === "polling") {
    return "status-chip--warning";
  }

  if (status === "authError" || status === "configError" || status === "requestError") {
    return "status-chip--danger";
  }

  return "status-chip--neutral";
}

export function PollingHealthPanel({
  health,
  diagnostics,
  runtimeStatus,
  onPollNow,
}: PollingHealthPanelProps) {
  const resolvedRuntimeStatus = runtimeStatus ?? health.status;

  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h3>Polling health</h3>
          <div className="section__subtle">Host runtime status, retry horizon, and last sync markers.</div>
        </div>
        <button className="button button--secondary" type="button" onClick={onPollNow}>
          Poll now
        </button>
      </div>

      <div className="status-row">
        <span className={`status-chip ${getHealthChipClass(resolvedRuntimeStatus as PollingHealth["status"])}`}>
          {resolvedRuntimeStatus}
        </span>
        <span className={`status-chip ${health.isStale ? "status-chip--warning" : "status-chip--neutral"}`}>
          {health.isStale ? "stale snapshot" : "live snapshot"}
        </span>
      </div>

      <div className="summary-list">
        <div className="summary-item">
          <div className="summary-item__label">Last attempt</div>
          <div className="summary-item__value mono">{formatTimestamp(diagnostics.lastAttemptAt)}</div>
        </div>
        <div className="summary-item">
          <div className="summary-item__label">Last successful sync</div>
          <div className="summary-item__value mono">
            {formatTimestamp(diagnostics.lastSuccessfulSyncAt)}
          </div>
        </div>
        <div className="summary-item">
          <div className="summary-item__label">Next retry</div>
          <div className="summary-item__value mono">{formatTimestamp(diagnostics.nextRetryAt)}</div>
        </div>
      </div>
    </section>
  );
}
