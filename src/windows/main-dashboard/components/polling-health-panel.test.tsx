import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PollingHealthPanel } from "./polling-health-panel";

describe("PollingHealthPanel", () => {
  it("shows degraded status details during backoff", () => {
    render(
      <PollingHealthPanel
        health={{ status: "backoff", pollingIntervalSeconds: 60, isStale: true }}
        diagnostics={{
          source: "request",
          code: "HTTP_429",
          message: "Backoff active",
          lastAttemptAt: Date.now(),
          lastSuccessfulSyncAt: Date.now() - 1_000,
          nextRetryAt: Date.now() + 60_000,
        }}
        onPollNow={vi.fn()}
      />,
    );

    expect(screen.getByText("backoff")).toBeInTheDocument();
    expect(screen.getByText("stale snapshot")).toBeInTheDocument();
  });
});
