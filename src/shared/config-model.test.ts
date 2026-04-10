import { describe, expect, it } from "vitest";
import { MIN_POLLING_INTERVAL_SECONDS, sanitizeConfigInput } from "./config-model";

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
});
