import { useEffect, useState } from "react";
import type { AppConfigInput } from "../../../shared/config-model";

interface BootstrapPanelProps {
  initialValues?: Partial<AppConfigInput>;
  isSaving: boolean;
  submitError: string | null;
  heading?: string;
  subtle?: string;
  submitLabel?: string;
  showGroupFields?: boolean;
  onSubmit: (input: AppConfigInput) => Promise<void> | void;
}

export function BootstrapPanel({
  initialValues,
  isSaving,
  submitError,
  heading = "Bootstrap & window policy",
  subtle = "Save the minimum viable runtime config and let the host start polling.",
  submitLabel = "Save config & start polling",
  showGroupFields = true,
  onSubmit,
}: BootstrapPanelProps) {
  const [values, setValues] = useState<AppConfigInput>({
    apiBaseUrl: initialValues?.apiBaseUrl ?? "",
    apiKey: initialValues?.apiKey ?? "",
    pollingIntervalSeconds: initialValues?.pollingIntervalSeconds ?? 60,
    symbol: initialValues?.symbol ?? "BTCUSDT",
    signalTypesText: initialValues?.signalTypesText ?? "vegas,divMacd",
    selectedGroupId: initialValues?.selectedGroupId,
    groups: initialValues?.groups,
    layoutPreset: initialValues?.layoutPreset,
    density: initialValues?.density,
    windowPolicy: initialValues?.windowPolicy,
  });

  useEffect(() => {
    if (!initialValues) {
      return;
    }

    setValues((current) => ({
      ...current,
      ...initialValues,
    }));
  }, [initialValues]);

  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h2>{heading}</h2>
          <div className="section__subtle">{subtle}</div>
        </div>
      </div>

      <form
        className="bootstrap-form"
        onSubmit={async (event) => {
          event.preventDefault();
          await onSubmit(values);
        }}
      >
        <div className="field-grid">
          <div className="field">
            <label htmlFor="api-base-url">API base URL</label>
            <input
              id="api-base-url"
              value={values.apiBaseUrl}
              onChange={(event) => {
                const value = event.currentTarget.value;
                return (
                setValues((current) => ({
                  ...current,
                  apiBaseUrl: value,
                }))
                );
              }}
              placeholder="https://example.com"
            />
          </div>

          <div className="field">
            <label htmlFor="api-key">API key</label>
            <input
              id="api-key"
              value={values.apiKey}
              onChange={(event) => {
                const value = event.currentTarget.value;
                return (
                setValues((current) => ({
                  ...current,
                  apiKey: value,
                }))
                );
              }}
              placeholder="x-api-key"
            />
          </div>
        </div>

        <div className={`field-grid ${showGroupFields ? "field-grid--two" : ""}`}>
          {showGroupFields ? (
            <div className="field">
              <label htmlFor="symbol">Symbol</label>
              <input
                id="symbol"
                value={values.symbol}
                onChange={(event) => {
                  const value = event.currentTarget.value;
                  return (
                  setValues((current) => ({
                    ...current,
                    symbol: value,
                  }))
                  );
                }}
                placeholder="BTCUSDT"
              />
            </div>
          ) : null}

          <div className="field">
            <label htmlFor="polling-seconds">Polling interval (seconds)</label>
            <input
              id="polling-seconds"
              type="number"
              min={10}
              value={values.pollingIntervalSeconds}
              onChange={(event) => {
                const value = Number(event.currentTarget.value);
                return (
                setValues((current) => ({
                  ...current,
                  pollingIntervalSeconds: value,
                }))
                );
              }}
            />
          </div>
        </div>

        {showGroupFields ? (
          <div className="field">
            <label htmlFor="signal-types">Signal types</label>
            <textarea
              id="signal-types"
              value={values.signalTypesText}
              onChange={(event) => {
                const value = event.currentTarget.value;
                return (
                setValues((current) => ({
                  ...current,
                  signalTypesText: value,
                }))
                );
              }}
              placeholder="vegas,divMacd,tdMd"
            />
            <div className="field__hint">
              One group equals one symbol plus one or more signal types. Periods default to the
              fixed 25-level stack required by the product doc.
            </div>
          </div>
        ) : null}

        {submitError ? <div className="error-banner">{submitError}</div> : null}

        <div className="actions">
          <button className="button button--primary" type="submit" disabled={isSaving}>
            {isSaving ? "Saving…" : submitLabel}
          </button>
        </div>
      </form>
    </section>
  );
}
