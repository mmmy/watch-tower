import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { LayoutPresetToggle } from "./layout-preset-toggle";

describe("LayoutPresetToggle", () => {
  it("switches between table and list presets", () => {
    const onChange = vi.fn();

    render(<LayoutPresetToggle value="table" onChange={onChange} />);

    fireEvent.click(screen.getByRole("tab", { name: "List" }));

    expect(onChange).toHaveBeenCalledWith("list");
  });
});
