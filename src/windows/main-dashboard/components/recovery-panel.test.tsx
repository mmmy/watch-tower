import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { DashboardRecoveryItemViewModel } from "../../../shared/view-models";
import { RecoveryPanel } from "./recovery-panel";

function item(overrides?: Partial<DashboardRecoveryItemViewModel>): DashboardRecoveryItemViewModel {
  return {
    alert: {
      id: "ETHUSDT:240:divMacd",
      groupId: "eth-swing",
      symbol: "ETHUSDT",
      period: "240",
      signalType: "divMacd",
      side: -1,
      signalAt: 2_000,
    },
    source: "queued",
    isPendingRead: false,
    ...overrides,
  };
}

describe("RecoveryPanel", () => {
  it("surfaces the unread recovery count and nested queue", () => {
    render(
      <RecoveryPanel
        items={[item()]}
        onOpenInDashboard={vi.fn()}
        onMarkRead={vi.fn()}
      />,
    );

    expect(screen.getByText("Unread recovery")).toBeInTheDocument();
    expect(screen.getByText("1 items")).toBeInTheDocument();
    expect(screen.getByText("ETHUSDT · 240 · divMacd")).toBeInTheDocument();
  });
});
