import type { DashboardLayoutPreset } from "../../../shared/config-model";

interface LayoutPresetToggleProps {
  value: DashboardLayoutPreset;
  onChange: (value: DashboardLayoutPreset) => void;
}

export function LayoutPresetToggle({ value, onChange }: LayoutPresetToggleProps) {
  return (
    <div className="segmented-control" role="tablist" aria-label="Dashboard layout preset">
      {(["table", "list"] as const).map((preset) => (
        <button
          aria-selected={value === preset}
          className={`segmented-control__item ${
            value === preset ? "segmented-control__item--active" : ""
          }`}
          key={preset}
          role="tab"
          type="button"
          onClick={() => onChange(preset)}
        >
          {preset === "table" ? "Table" : "List"}
        </button>
      ))}
    </div>
  );
}
