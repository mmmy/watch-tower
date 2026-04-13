import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { StatusFooter } from "./status-footer";

describe("StatusFooter", () => {
  it("renders runtime status, stale state, and last sync metadata", () => {
    render(
      <StatusFooter
        runtimeStatus="paused"
        isStale
        widgetMode="passive"
        widgetPlacement="hidden"
        widgetFallback="Passive click-through is not enabled on this platform build."
        diagnostics={{
          source: "system",
          code: "POLLING_PAUSED",
          message: "Polling paused.",
          lastAttemptAt: null,
          lastSuccessfulSyncAt: 1_700_000_000_000,
          nextRetryAt: null,
        }}
      />,
    );

    expect(screen.getByText("paused")).toBeInTheDocument();
    expect(screen.getByText("stale snapshot")).toBeInTheDocument();
    expect(screen.getByText("POLLING_PAUSED")).toBeInTheDocument();
    expect(screen.getByText("passive")).toBeInTheDocument();
    expect(screen.getByText("hidden")).toBeInTheDocument();
    expect(screen.getByText(/Passive click-through/)).toBeInTheDocument();
  });
});
