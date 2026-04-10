import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BootstrapPanel } from "./bootstrap-panel";

describe("BootstrapPanel", () => {
  it("submits the form with the entered values", async () => {
    const onSubmit = vi.fn();

    render(
      <BootstrapPanel
        isSaving={false}
        submitError={null}
        onSubmit={onSubmit}
      />,
    );

    fireEvent.change(screen.getByLabelText("API base URL"), {
      target: { value: "https://example.com" },
    });
    fireEvent.change(screen.getByLabelText("API key"), {
      target: { value: "secret" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save config/i }));

    expect(onSubmit).toHaveBeenCalled();
  });
});
