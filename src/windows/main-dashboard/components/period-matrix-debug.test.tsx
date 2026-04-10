import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { normalizeGroupSnapshot } from "../../../shared/alert-model";
import { sanitizeConfigInput } from "../../../shared/config-model";
import { PeriodMatrixDebug } from "./period-matrix-debug";

describe("PeriodMatrixDebug", () => {
  it("renders quiet states and allows period selection", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 30,
      symbol: "BTCUSDT",
      signalTypesText: "vegas",
    });
    const snapshot = normalizeGroupSnapshot(config, config.selectedGroupId, null, Date.now());
    const onSelect = vi.fn();

    render(
      <PeriodMatrixDebug
        snapshot={snapshot}
        activePeriod="60"
        activeSignalType="vegas"
        onSelect={onSelect}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "60 vegas" }));

    expect(screen.getAllByText("Quiet").length).toBeGreaterThan(0);
    expect(onSelect).toHaveBeenCalledWith("60", "vegas");
  });
});
