import type { NormalizedSignal } from "../../../shared/alert-model";
import { getPeriodLabel } from "../../../shared/period-utils";

interface Timeline60DebugProps {
  signal: NormalizedSignal | null;
  period: string;
  signalType: string;
}

export function Timeline60Debug({ signal, period, signalType }: Timeline60DebugProps) {
  const bars = Array.from({ length: 60 }, (_, index) => {
    const isActive = signal?.barIndex === index;
    const mood = signal?.side === 1 ? "timeline__bar--bull" : "timeline__bar--bear";

    return (
      <div
        aria-label={isActive ? `Active bar ${index + 1}` : `Bar ${index + 1}`}
        className={`timeline__bar ${isActive ? `timeline__bar--active ${mood}` : ""}`}
        key={index}
      />
    );
  });

  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h2>Timeline 60</h2>
          <div className="section__subtle">
            {signalType} on {getPeriodLabel(period as never)} mapped into the latest 60 bars.
          </div>
        </div>
      </div>

      <div className="timeline">
        <div className="timeline__summary">
          <span className={`status-chip ${signal ? "status-chip--success" : "status-chip--neutral"}`}>
            {signal ? `bar #${(signal.barIndex ?? 0) + 1}` : "out of range / quiet"}
          </span>
          {signal ? (
            <span className={`status-chip ${signal.read ? "status-chip--neutral" : "status-chip--warning"}`}>
              {signal.read ? "server says read" : "server says unread"}
            </span>
          ) : null}
        </div>

        <div className="timeline__bars">{bars}</div>
        <div className="timeline__caption">
          If the signal timestamp falls outside the latest 60 buckets for this period, the timeline
          intentionally stays quiet. That is the expected validation behavior in `v0.1`.
        </div>
      </div>
    </section>
  );
}
