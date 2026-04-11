import type { NormalizedPeriodSnapshot } from "../../../shared/alert-model";
import { formatTimestamp } from "../../../shared/period-utils";

interface PeriodRowProps {
  periodSnapshot: NormalizedPeriodSnapshot;
  signalTypes: string[];
}

export function PeriodRow({ periodSnapshot, signalTypes }: PeriodRowProps) {
  return (
    <div className="widget-period-row">
      <div className="widget-period-row__period">{periodSnapshot.period}</div>
      <div className="widget-period-row__signals">
        {signalTypes.map((signalType) => {
          const signal = periodSnapshot.signals[signalType];
          const stateLabel = signal ? (signal.side === 1 ? "Bullish" : "Bearish") : "Quiet";
          const stateClass = signal
            ? signal.side === 1
              ? "widget-signal--bull"
              : "widget-signal--bear"
            : "widget-signal--quiet";

          return (
            <div className={`widget-signal ${stateClass}`} key={`${periodSnapshot.period}-${signalType}`}>
              <div className="widget-signal__type">{signalType}</div>
              <div className="widget-signal__state">{stateLabel}</div>
              <div className="widget-signal__meta">
                {signal ? formatTimestamp(signal.signalAt) : "No current alert"}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
