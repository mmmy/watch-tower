import type { NormalizedGroupSnapshot } from "../../../shared/alert-model";
import { formatTimestamp } from "../../../shared/period-utils";

interface PeriodMatrixDebugProps {
  snapshot: NormalizedGroupSnapshot;
  activePeriod: string;
  activeSignalType: string;
  onSelect: (period: string, signalType: string) => void;
}

export function PeriodMatrixDebug({
  snapshot,
  activePeriod,
  activeSignalType,
  onSelect,
}: PeriodMatrixDebugProps) {
  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h2>25-period signal matrix</h2>
          <div className="section__subtle">
            Single-group, single-symbol verification of normalized signal state.
          </div>
        </div>
      </div>

      <div className="matrix">
        {snapshot.periods.map((periodSnapshot) => (
          <div className="matrix__row" key={periodSnapshot.period}>
            <div className="matrix__period">{periodSnapshot.period}</div>
            <div className="matrix__signals">
              {snapshot.group.signalTypes.map((signalType) => {
                const signal = periodSnapshot.signals[signalType];
                const stateClass = signal
                  ? signal.side === 1
                    ? "signal-cell--bull"
                    : "signal-cell--bear"
                  : "signal-cell--quiet";
                const isActive =
                  activePeriod === periodSnapshot.period && activeSignalType === signalType;

                return (
                  <button
                    aria-label={`${periodSnapshot.period} ${signalType}`}
                    className={`signal-cell ${stateClass} ${isActive ? "signal-cell--active" : ""}`}
                    key={`${periodSnapshot.period}-${signalType}`}
                    type="button"
                    onClick={() => onSelect(periodSnapshot.period, signalType)}
                  >
                    <div className="signal-cell__name">{signalType}</div>
                    <div className="signal-cell__state">
                      <span>{signal ? (signal.side === 1 ? "Bullish" : "Bearish") : "Quiet"}</span>
                      <span
                        className={`signal-dot ${
                          signal
                            ? signal.side === 1
                              ? "signal-dot--bull"
                              : "signal-dot--bear"
                            : "signal-dot--quiet"
                        }`}
                      />
                    </div>
                    <div className="signal-cell__meta">
                      {signal ? formatTimestamp(signal.signalAt) : "No current alert"}
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>
    </section>
  );
}
