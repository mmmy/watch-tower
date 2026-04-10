import { describe, expect, it } from "vitest";
import { normalizeGroupSnapshot } from "./alert-model";
import { sanitizeConfigInput } from "./config-model";

describe("normalizeGroupSnapshot", () => {
  it("maps a response into period keyed signal snapshots", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 30,
      symbol: "BTCUSDT",
      signalTypesText: "vegas,divMacd",
    });
    const now = Date.UTC(2026, 3, 10, 12, 0, 0);

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
              vegas: { sd: 1, t: now - 2 * 60 * 60 * 1000, read: false },
              divMacd: { sd: -1, t: now - 5 * 60 * 60 * 1000, read: true },
            },
          },
        ],
      },
      now,
    );

    const hourPeriod = snapshot.periods.find((item) => item.period === "60");

    expect(hourPeriod?.signals.vegas?.side).toBe(1);
    expect(hourPeriod?.signals.divMacd?.read).toBe(true);
  });

  it("records issues for malformed records without crashing the rest of the snapshot", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 30,
      symbol: "BTCUSDT",
      signalTypesText: "vegas",
    });

    const snapshot = normalizeGroupSnapshot(
      config,
      config.selectedGroupId,
      {
        total: 1,
        page: 1,
        pageSize: 25,
        data: [{ symbol: "BTCUSDT", period: "60", t: Date.now() }],
      },
      Date.now(),
    );

    expect(snapshot.issues).toContain("Record for 60 is missing a signals object.");
    expect(snapshot.periods.find((item) => item.period === "60")?.signals.vegas).toBeNull();
  });
});
