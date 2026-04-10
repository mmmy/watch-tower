import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Timeline60Debug } from "./timeline-60-debug";

describe("Timeline60Debug", () => {
  it("marks the active bar when a signal is inside the latest 60 buckets", () => {
    render(
      <Timeline60Debug
        period="60"
        signalType="vegas"
        signal={{
          signalType: "vegas",
          period: "60",
          side: 1,
          read: false,
          signalAt: Date.now(),
          barIndex: 12,
        }}
      />,
    );

    expect(screen.getByLabelText("Active bar 13")).toBeInTheDocument();
  });
});
