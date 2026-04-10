import { describe, expect, it } from "vitest";
import { computeBarIndex, getPeriodDurationMinutes, sortPeriods } from "./period-utils";

describe("period-utils", () => {
  it("sorts the fixed 25-level stack in product order", () => {
    expect(sortPeriods(["15", "10D", "W", "D", "1"])).toEqual(["10D", "W", "D", "15", "1"]);
  });

  it("uses UTC+0 aligned daily buckets", () => {
    const now = Date.UTC(2026, 3, 10, 12, 0, 0);
    const yesterday = Date.UTC(2026, 3, 9, 3, 0, 0);

    expect(computeBarIndex(yesterday, "D", now)).toBe(58);
  });

  it("returns null when the signal is outside the last 60 bars", () => {
    const now = Date.UTC(2026, 3, 10, 12, 0, 0);
    const oldSignal = now - getPeriodDurationMinutes("60") * 60 * 1000 * 61;

    expect(computeBarIndex(oldSignal, "60", now)).toBeNull();
  });
});
