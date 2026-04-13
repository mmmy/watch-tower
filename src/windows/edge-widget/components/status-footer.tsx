import type { DiagnosticsInfo } from "../../../shared/alert-model";
import { formatTimestamp } from "../../../shared/period-utils";

interface StatusFooterProps {
  runtimeStatus: string;
  isStale: boolean;
  widgetMode: string;
  widgetPlacement: string;
  widgetFallback: string | null;
  diagnostics: DiagnosticsInfo;
}

export function StatusFooter({
  runtimeStatus,
  isStale,
  widgetMode,
  widgetPlacement,
  widgetFallback,
  diagnostics,
}: StatusFooterProps) {
  return (
    <footer className="widget-footer">
      <div className="widget-footer__chips">
        <span className="status-chip status-chip--warning">{runtimeStatus}</span>
        <span className={`status-chip ${isStale ? "status-chip--warning" : "status-chip--neutral"}`}>
          {isStale ? "stale snapshot" : "live snapshot"}
        </span>
        <span className="status-chip status-chip--neutral">{widgetMode}</span>
        <span className="status-chip status-chip--neutral">{widgetPlacement}</span>
      </div>
      <div className="widget-footer__meta">
        <span>{diagnostics.code ?? "RUNTIME"}</span>
        <span>{formatTimestamp(diagnostics.lastSuccessfulSyncAt)}</span>
      </div>
      {widgetFallback ? <div className="widget-footer__meta">{widgetFallback}</div> : null}
    </footer>
  );
}
