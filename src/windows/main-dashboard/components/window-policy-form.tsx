import type {
  DashboardDensity,
  DashboardLayoutPreset,
  DockSide,
  WindowPolicyConfig,
} from "../../../shared/config-model";
import { LayoutPresetToggle } from "./layout-preset-toggle";

interface WindowPolicyFormProps {
  layoutPreset: DashboardLayoutPreset;
  density: DashboardDensity;
  windowPolicy: Partial<WindowPolicyConfig> | undefined;
  isSaving: boolean;
  onChange: (next: {
    layoutPreset: DashboardLayoutPreset;
    density: DashboardDensity;
    windowPolicy: Partial<WindowPolicyConfig>;
  }) => void;
  onSave: () => void;
}

export function WindowPolicyForm({
  layoutPreset,
  density,
  windowPolicy,
  isSaving,
  onChange,
  onSave,
}: WindowPolicyFormProps) {
  const nextWindowPolicy = {
    dockSide: windowPolicy?.dockSide ?? "right",
    widgetWidth: windowPolicy?.widgetWidth ?? 280,
    widgetHeight: windowPolicy?.widgetHeight ?? 720,
    topOffset: windowPolicy?.topOffset ?? 96,
  };

  return (
    <section className="panel section">
      <div className="section__header">
        <div>
          <h3>Layout & window policy</h3>
          <div className="section__subtle">
            Store the minimum dashboard and widget preferences needed for the resident MVP.
          </div>
        </div>
      </div>

      <div className="field">
        <label>Layout preset</label>
        <LayoutPresetToggle
          value={layoutPreset}
          onChange={(nextLayoutPreset) =>
            onChange({
              layoutPreset: nextLayoutPreset,
              density,
              windowPolicy: nextWindowPolicy,
            })
          }
        />
      </div>

      <div className="field-grid field-grid--two">
        <div className="field">
          <label htmlFor="dashboard-density">Density</label>
          <select
            id="dashboard-density"
            value={density}
            onChange={(event) =>
              onChange({
                layoutPreset,
                density: event.currentTarget.value as DashboardDensity,
                windowPolicy: nextWindowPolicy,
              })
            }
          >
            <option value="comfortable">Comfortable</option>
            <option value="compact">Compact</option>
          </select>
        </div>

        <div className="field">
          <label htmlFor="dock-side">Default dock side</label>
          <select
            id="dock-side"
            value={nextWindowPolicy.dockSide}
            onChange={(event) =>
              onChange({
                layoutPreset,
                density,
                windowPolicy: {
                  ...nextWindowPolicy,
                  dockSide: event.currentTarget.value as DockSide,
                },
              })
            }
          >
            <option value="left">Left</option>
            <option value="right">Right</option>
          </select>
        </div>
      </div>

      <div className="field-grid field-grid--three">
        <div className="field">
          <label htmlFor="widget-width">Widget width</label>
          <input
            id="widget-width"
            min={160}
            type="number"
            value={nextWindowPolicy.widgetWidth}
            onChange={(event) =>
              onChange({
                layoutPreset,
                density,
                windowPolicy: {
                  ...nextWindowPolicy,
                  widgetWidth: Number(event.currentTarget.value),
                },
              })
            }
          />
        </div>

        <div className="field">
          <label htmlFor="widget-height">Widget height</label>
          <input
            id="widget-height"
            min={320}
            type="number"
            value={nextWindowPolicy.widgetHeight}
            onChange={(event) =>
              onChange({
                layoutPreset,
                density,
                windowPolicy: {
                  ...nextWindowPolicy,
                  widgetHeight: Number(event.currentTarget.value),
                },
              })
            }
          />
        </div>

        <div className="field">
          <label htmlFor="top-offset">Top offset</label>
          <input
            id="top-offset"
            min={0}
            type="number"
            value={nextWindowPolicy.topOffset}
            onChange={(event) =>
              onChange({
                layoutPreset,
                density,
                windowPolicy: {
                  ...nextWindowPolicy,
                  topOffset: Number(event.currentTarget.value),
                },
              })
            }
          />
        </div>
      </div>

      <div className="actions">
        <button className="button button--primary" type="button" onClick={onSave} disabled={isSaving}>
          {isSaving ? "Saving…" : "Save layout policy"}
        </button>
      </div>
    </section>
  );
}
