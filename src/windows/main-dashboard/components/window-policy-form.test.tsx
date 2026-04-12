import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WindowPolicyForm } from "./window-policy-form";

describe("WindowPolicyForm", () => {
  it("updates the layout policy and saves it", () => {
    const onChange = vi.fn();
    const onSave = vi.fn();

    render(
      <WindowPolicyForm
        layoutPreset="table"
        density="comfortable"
        notificationsEnabled
        windowPolicy={{
          dockSide: "right",
          widgetWidth: 280,
          widgetHeight: 720,
          topOffset: 96,
        }}
        isSaving={false}
        onChange={onChange}
        onSave={onSave}
      />,
    );

    fireEvent.change(screen.getByLabelText("Density"), {
      target: { value: "compact" },
    });
    fireEvent.change(screen.getByLabelText("Default dock side"), {
      target: { value: "left" },
    });
    fireEvent.change(screen.getByLabelText("System notifications"), {
      target: { value: "disabled" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save layout policy/i }));

    expect(onChange).toHaveBeenCalled();
    expect(onSave).toHaveBeenCalled();
  });
});
