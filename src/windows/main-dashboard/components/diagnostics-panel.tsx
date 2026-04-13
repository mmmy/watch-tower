import type { DiagnosticsInfo } from "../../../shared/alert-model";
import type { WidgetBehaviorRuntime } from "../../../shared/window-state";

interface DiagnosticsPanelProps {
  diagnostics: DiagnosticsInfo;
  issues: string[];
  widgetRuntime: WidgetBehaviorRuntime;
}

export function DiagnosticsPanel({
  diagnostics,
  issues,
  widgetRuntime,
}: DiagnosticsPanelProps) {
  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h3>Diagnostics</h3>
          <div className="section__subtle">
            Enough host + normalization context to explain why the shell looks the way it does.
          </div>
        </div>
      </div>

      <div className="diagnostics-list">
        <div className="diagnostic-item">
          <div className="diagnostic-item__label">Source</div>
          <div className="diagnostic-item__value">{diagnostics.source}</div>
        </div>
        <div className="diagnostic-item">
          <div className="diagnostic-item__label">Code</div>
          <div className="diagnostic-item__value mono">{diagnostics.code ?? "NONE"}</div>
        </div>
        <div className="diagnostic-item">
          <div className="diagnostic-item__label">Message</div>
          <div className="diagnostic-item__value">{diagnostics.message}</div>
        </div>
        <div className="diagnostic-item">
          <div className="diagnostic-item__label">Normalization notes</div>
          <div className="diagnostic-item__value">
            {issues.length > 0 ? issues.join(" ") : "No normalization issues surfaced from the current payload."}
          </div>
        </div>
        <div className="diagnostic-item">
          <div className="diagnostic-item__label">Widget mode</div>
          <div className="diagnostic-item__value">
            {widgetRuntime.mode} · {widgetRuntime.placement}
          </div>
        </div>
        <div className="diagnostic-item">
          <div className="diagnostic-item__label">Widget capability</div>
          <div className="diagnostic-item__value">
            {widgetRuntime.clickThroughSupported
              ? widgetRuntime.clickThroughEnabled
                ? "Passive click-through active"
                : "Passive click-through supported"
              : widgetRuntime.fallbackReason ?? "Fallback mode active"}
          </div>
        </div>
      </div>
    </section>
  );
}
