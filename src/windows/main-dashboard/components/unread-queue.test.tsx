import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { DashboardRecoveryItemViewModel } from "../../../shared/view-models";
import { UnreadQueue } from "./unread-queue";

function item(overrides?: Partial<DashboardRecoveryItemViewModel>): DashboardRecoveryItemViewModel {
  return {
    alert: {
      id: "BTCUSDT:60:vegas",
      groupId: "btc-core",
      symbol: "BTCUSDT",
      period: "60",
      signalType: "vegas",
      side: 1,
      signalAt: 1_000,
    },
    source: "visible",
    isPendingRead: false,
    ...overrides,
  };
}

describe("UnreadQueue", () => {
  it("renders unread recovery items and wires actions", () => {
    const onOpenInDashboard = vi.fn();
    const onMarkRead = vi.fn();

    render(
      <UnreadQueue
        items={[item()]}
        onOpenInDashboard={onOpenInDashboard}
        onMarkRead={onMarkRead}
      />,
    );

    expect(screen.getByText("BTCUSDT · 60 · vegas")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Open detail" }));
    fireEvent.click(screen.getByRole("button", { name: "Mark read" }));

    expect(onOpenInDashboard).toHaveBeenCalledWith("BTCUSDT:60:vegas");
    expect(onMarkRead).toHaveBeenCalledWith("BTCUSDT:60:vegas");
  });

  it("renders an empty state when there are no recovery items", () => {
    render(
      <UnreadQueue
        items={[]}
        onOpenInDashboard={vi.fn()}
        onMarkRead={vi.fn()}
      />,
    );

    expect(screen.getByText("No unread recovery items")).toBeInTheDocument();
  });
});
