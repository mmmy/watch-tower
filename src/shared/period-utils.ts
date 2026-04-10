import { PERIOD_ORDER, type SupportedPeriod } from "./config-model";

const MINUTE_PERIODS = new Set<SupportedPeriod>([
  "720",
  "480",
  "360",
  "240",
  "180",
  "120",
  "90",
  "60",
  "45",
  "30",
  "20",
  "15",
  "10",
  "8",
  "5",
  "4",
  "3",
  "2",
  "1",
]);

export function sortPeriods(periods: readonly string[]): SupportedPeriod[] {
  return [...periods]
    .filter((period): period is SupportedPeriod =>
      PERIOD_ORDER.includes(period as SupportedPeriod),
    )
    .sort((left, right) => PERIOD_ORDER.indexOf(left) - PERIOD_ORDER.indexOf(right));
}

export function getPeriodDurationMinutes(period: SupportedPeriod): number {
  if (period === "W") {
    return 7 * 24 * 60;
  }

  if (period === "D") {
    return 24 * 60;
  }

  if (period.endsWith("D")) {
    return Number.parseInt(period, 10) * 24 * 60;
  }

  if (MINUTE_PERIODS.has(period)) {
    return Number.parseInt(period, 10);
  }

  throw new Error(`Unsupported period: ${period}`);
}

export function getPeriodLabel(period: SupportedPeriod): string {
  return MINUTE_PERIODS.has(period) ? `${period}m` : period;
}

export function alignTimestampToUtcBucket(timestampMs: number, period: SupportedPeriod): number {
  const date = new Date(timestampMs);

  if (period === "W") {
    const utcDay = date.getUTCDay();
    const daysSinceMonday = (utcDay + 6) % 7;
    date.setUTCDate(date.getUTCDate() - daysSinceMonday);
    date.setUTCHours(0, 0, 0, 0);
    return date.getTime();
  }

  if (period === "D" || period.endsWith("D")) {
    const days = period === "D" ? 1 : Number.parseInt(period, 10);
    const span = 24 * 60 * 60 * 1000 * days;
    return Math.floor(timestampMs / span) * span;
  }

  const durationMs = getPeriodDurationMinutes(period) * 60 * 1000;
  return Math.floor(timestampMs / durationMs) * durationMs;
}

export function computeBarIndex(
  signalTimestampMs: number,
  period: SupportedPeriod,
  nowMs: number,
): number | null {
  const durationMs = getPeriodDurationMinutes(period) * 60 * 1000;
  const alignedNow = alignTimestampToUtcBucket(nowMs, period);
  const alignedSignal = alignTimestampToUtcBucket(signalTimestampMs, period);
  const distance = Math.floor((alignedNow - alignedSignal) / durationMs);

  if (distance < 0 || distance > 59) {
    return null;
  }

  return 59 - distance;
}

export function formatTimestamp(timestampMs: number | null): string {
  if (!timestampMs) {
    return "Not synced yet";
  }

  return new Intl.DateTimeFormat("en-GB", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    timeZone: "UTC",
  }).format(new Date(timestampMs));
}
