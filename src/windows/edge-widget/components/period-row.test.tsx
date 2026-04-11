import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { normalizeGroupSnapshot } from "../../../shared/alert-model";
import { sanitizeConfigInput } from "../../../shared/config-model";
import { PeriodRow } from "./period-row";

describe("PeriodRow", () => {
  it("renders quiet and active signal states for a period", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 60,
      symbol: "BTCUSDT",
      signalTypesText: "vegas,divMacd",
    });
    const now = Date.now();
    const snapshot = normalizeGroupSnapshot(
      config,
      config.selectedGroupId,
      {
        total: 1,
        page: 1,
        pageSize: 25,
        data: [
          {
            symbol: "BTCUSDT",
            period: "60",
            t: now,
            signals: {
              vegas: { sd: 1, t: now - 60_000, read: false },
            },
          },
        ],
      },
      now,
    );

    render(
      <PeriodRow
        periodSnapshot={snapshot.periods.find((period) => period.period === "60")!}
        signalTypes={snapshot.group.signalTypes}
      />,
    );

    expect(screen.getByText("Bullish")).toBeInTheDocument();
    expect(screen.getByText("Quiet")).toBeInTheDocument();
  });
});
