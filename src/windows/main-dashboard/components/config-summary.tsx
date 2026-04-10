import type { AppConfig } from "../../../shared/config-model";
import { getPeriodLabel } from "../../../shared/period-utils";

interface ConfigSummaryProps {
  config: AppConfig;
}

export function ConfigSummary({ config }: ConfigSummaryProps) {
  const selectedGroup = config.groups.find((group) => group.id === config.selectedGroupId) ?? config.groups[0];

  if (!selectedGroup) {
    return null;
  }

  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h3>Config summary</h3>
          <div className="section__subtle">What the host is currently polling.</div>
        </div>
      </div>

      <div className="summary-list">
        <div className="summary-item">
          <div className="summary-item__label">Base URL</div>
          <div className="summary-item__value mono">{config.apiBaseUrl}</div>
        </div>
        <div className="summary-item">
          <div className="summary-item__label">Selected group</div>
          <div className="summary-item__value">
            {selectedGroup.symbol} · {selectedGroup.signalTypes.join(", ")}
          </div>
        </div>
        <div className="summary-item">
          <div className="summary-item__label">Polling</div>
          <div className="summary-item__value">{config.pollingIntervalSeconds}s minimum cadence</div>
        </div>
        <div className="summary-item">
          <div className="summary-item__label">Timeline focus</div>
          <div className="summary-item__value">{getPeriodLabel(selectedGroup.selectedTimelinePeriod)}</div>
        </div>
      </div>
    </section>
  );
}
