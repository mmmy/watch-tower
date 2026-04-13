import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./app";

vi.mock("./windows/main-dashboard", () => ({
  MainDashboardPage: () => <div>Main dashboard page</div>,
}));

vi.mock("./windows/alert-popup", () => ({
  AlertPopupPage: () => <div>Alert popup page</div>,
}));

vi.mock("./windows/edge-widget", () => ({
  EdgeWidgetPage: () => <div>Edge widget page</div>,
}));

function setCurrentWindowLabel(label?: string) {
  (window as typeof window & {
    __TAURI_INTERNALS__?: {
      metadata?: {
        currentWindow?: {
          label?: string;
        };
      };
    };
  }).__TAURI_INTERNALS__ = label
    ? {
        metadata: {
          currentWindow: {
            label,
          },
        },
      }
    : undefined;
}

describe("App", () => {
  afterEach(() => {
    setCurrentWindowLabel(undefined);
  });

  it("routes dynamic popup labels to the alert popup page", () => {
    setCurrentWindowLabel("alert-popup:ETHUSDT");

    render(<App />);

    expect(screen.getByText("Alert popup page")).toBeInTheDocument();
  });

  it("routes unknown labels back to the main dashboard page", () => {
    setCurrentWindowLabel("main-dashboard");

    render(<App />);

    expect(screen.getByText("Main dashboard page")).toBeInTheDocument();
  });
});
