import { describe, expect, it } from "vitest";
import {
  MIN_POLLING_INTERVAL_SECONDS,
  sanitizeConfigInput,
  type SupportedPeriod,
} from "./config-model";

describe("sanitizeConfigInput", () => {
  it("normalizes a single-symbol group with multiple signal types", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com/",
      apiKey: "secret",
      pollingIntervalSeconds: 30,
      symbol: "btcusdt",
      signalTypesText: "vegas, divMacd, tdMd",
    });

    expect(config.apiBaseUrl).toBe("https://example.com");
    expect(config.groups[0]?.symbol).toBe("BTCUSDT");
    expect(config.groups[0]?.signalTypes).toEqual(["vegas", "divMacd", "tdMd"]);
    expect(config.selectedGroupId).toBe("btcusdt");
    expect(config.dashboard.layoutPreset).toBe("table");
    expect(config.windowPolicy.dockSide).toBe("right");
  });

  it("clamps invalid polling values to the minimum floor", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 1,
      symbol: "BTCUSDT",
      signalTypesText: "vegas",
    });

    expect(config.pollingIntervalSeconds).toBe(MIN_POLLING_INTERVAL_SECONDS);
  });

  it("builds multiple groups, keeps a valid selected group, and applies explicit preferences", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 45,
      selectedGroupId: "eth-swing",
      layoutPreset: "list",
      density: "compact",
      windowPolicy: {
        dockSide: "left",
        widgetWidth: 300,
        widgetHeight: 760,
        topOffset: 120,
      },
      groups: [
        {
          id: "btc-core",
          symbol: "BTCUSDT",
          signalTypesText: "vegas, divMacd",
          periods: ["240", "60", "15"] as SupportedPeriod[],
          selectedTimelinePeriod: "240",
        },
        {
          id: "eth-swing",
          symbol: "ETHUSDT",
          signalTypesText: "tdMd, vegas",
          periods: ["D", "240", "60"] as SupportedPeriod[],
          selectedTimelinePeriod: "D",
        },
      ],
    });

    expect(config.groups).toHaveLength(2);
    expect(config.selectedGroupId).toBe("eth-swing");
    expect(config.groups[1]?.symbol).toBe("ETHUSDT");
    expect(config.groups[1]?.signalTypes).toEqual(["tdMd", "vegas"]);
    expect(config.groups[1]?.periods).toEqual(["D", "240", "60"]);
    expect(config.dashboard).toEqual({
      layoutPreset: "list",
      density: "compact",
    });
    expect(config.windowPolicy).toEqual({
      dockSide: "left",
      widgetWidth: 300,
      widgetHeight: 760,
      topOffset: 120,
    });
  });

  it("falls back to the first valid group when the selected group is missing", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 60,
      selectedGroupId: "missing-group",
      groups: [
        {
          symbol: "BTCUSDT",
          signalTypesText: "vegas",
        },
        {
          symbol: "ETHUSDT",
          signalTypesText: "divMacd",
        },
      ],
    });

    expect(config.groups).toHaveLength(2);
    expect(config.selectedGroupId).toBe(config.groups[0]?.id);
    expect(config.groups[0]?.selectedTimelinePeriod).toBe("60");
  });

  it("rejects groups that do not include a symbol", () => {
    expect(() =>
      sanitizeConfigInput({
        apiBaseUrl: "https://example.com",
        apiKey: "secret",
        pollingIntervalSeconds: 60,
        groups: [
          {
            symbol: "   ",
            signalTypesText: "vegas",
          },
        ],
      }),
    ).toThrow("Each group requires exactly one symbol.");
  });

  it("allows an empty group list for the dashboard empty state", () => {
    const config = sanitizeConfigInput({
      apiBaseUrl: "https://example.com",
      apiKey: "secret",
      pollingIntervalSeconds: 60,
      groups: [],
    });

    expect(config.groups).toEqual([]);
    expect(config.selectedGroupId).toBe("");
  });
});
