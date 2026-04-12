import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useAlertPopupEvents } from "./use-alert-popup-events";

function HookHarness() {
  const { popupView } = useAlertPopupEvents();

  return (
    <div>
      <div data-testid="popup-state">{popupView?.state ?? "missing"}</div>
      <div data-testid="popup-symbol">{popupView?.alert?.symbol ?? "none"}</div>
    </div>
  );
}

describe("useAlertPopupEvents", () => {
  it("provides an active fallback alert view in browser mode", async () => {
    render(<HookHarness />);

    await waitFor(() => {
      expect(screen.getByTestId("popup-state").textContent).toBe("active");
      expect(screen.getByTestId("popup-symbol").textContent).toBe("ETHUSDT");
    });
  });
});
