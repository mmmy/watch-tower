import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { AlertPopupViewModel } from "../../../shared/view-models";
import { AlertCard } from "./alert-card";

function activePopupView(overrides?: Partial<AlertPopupViewModel>): AlertPopupViewModel {
  return {
    state: "active",
    alert: {
      id: "BTCUSDT:60:vegas",
      groupId: "btc-core",
      symbol: "BTCUSDT",
      period: "60",
      signalType: "vegas",
      side: 1,
      signalAt: 1_000,
    },
    runtimeStatus: "success",
    isPendingRead: false,
    ...overrides,
  };
}

describe("AlertCard", () => {
  it("renders the active alert payload and action buttons", () => {
    render(
      <AlertCard
        popupView={activePopupView()}
        onMarkRead={vi.fn()}
        onOpenInDashboard={vi.fn()}
      />,
    );

    expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
    expect(screen.getByText("60 · vegas · Bullish")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Mark read" })).toBeInTheDocument();
  });

  it("disables the mark-read button while the pending read is in flight", () => {
    render(
      <AlertCard
        popupView={activePopupView({ isPendingRead: true })}
        onMarkRead={vi.fn()}
        onOpenInDashboard={vi.fn()}
      />,
    );

    expect(screen.getByRole("button", { name: "Marking..." })).toBeDisabled();
  });

  it("wires both popup actions to their handlers", () => {
    const onMarkRead = vi.fn();
    const onOpenInDashboard = vi.fn();

    render(
      <AlertCard
        popupView={activePopupView()}
        onMarkRead={onMarkRead}
        onOpenInDashboard={onOpenInDashboard}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Open in dashboard" }));
    fireEvent.click(screen.getByRole("button", { name: "Mark read" }));

    expect(onOpenInDashboard).toHaveBeenCalledTimes(1);
    expect(onMarkRead).toHaveBeenCalledTimes(1);
  });
});
