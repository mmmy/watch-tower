import { PERIOD_ORDER, type SupportedPeriod, type WatchGroupInput } from "../../../shared/config-model";

interface GroupEditorProps {
  group: WatchGroupInput | null;
  isSaving: boolean;
  onChange: (group: WatchGroupInput) => void;
  onSave: () => void;
}

export function GroupEditor({ group, isSaving, onChange, onSave }: GroupEditorProps) {
  if (!group) {
    return (
      <section className="panel section">
        <div className="section__header">
          <div>
            <h3>Group editor</h3>
            <div className="section__subtle">Select or create a group to configure it.</div>
          </div>
        </div>
      </section>
    );
  }

  const periodsText = (group.periods ?? PERIOD_ORDER).join(",");

  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h3>Group editor</h3>
          <div className="section__subtle">Refine one symbol, its signal set, and its default timeline focus.</div>
        </div>
      </div>

      <div className="field-grid field-grid--two">
        <div className="field">
          <label htmlFor="group-symbol">Symbol</label>
          <input
            id="group-symbol"
            value={group.symbol}
            onChange={(event) => onChange({ ...group, symbol: event.currentTarget.value })}
            placeholder="BTCUSDT"
          />
        </div>

        <div className="field">
          <label htmlFor="group-timeline">Default timeline period</label>
          <select
            id="group-timeline"
            value={group.selectedTimelinePeriod ?? "60"}
            onChange={(event) =>
              onChange({
                ...group,
                selectedTimelinePeriod: event.currentTarget.value as SupportedPeriod,
              })
            }
          >
            {(group.periods ?? PERIOD_ORDER).map((period) => (
              <option key={period} value={period}>
                {period}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="field">
        <label htmlFor="group-signals">Signal types</label>
        <textarea
          id="group-signals"
          value={group.signalTypesText}
          onChange={(event) => onChange({ ...group, signalTypesText: event.currentTarget.value })}
          placeholder="vegas,divMacd,tdMd"
        />
      </div>

      <div className="field">
        <label htmlFor="group-periods">Periods</label>
        <textarea
          id="group-periods"
          value={periodsText}
          onChange={(event) =>
            onChange({
              ...group,
              periods: event.currentTarget.value
                .split(",")
                .map((item) => item.trim())
                .filter((item): item is SupportedPeriod =>
                  PERIOD_ORDER.includes(item as SupportedPeriod),
                ),
            })
          }
          placeholder={PERIOD_ORDER.join(",")}
        />
        <div className="field__hint">Use comma-separated values from the fixed 25-level stack.</div>
      </div>

      <div className="actions">
        <button className="button button--primary" type="button" onClick={onSave} disabled={isSaving}>
          {isSaving ? "Saving…" : "Save group"}
        </button>
      </div>
    </section>
  );
}
