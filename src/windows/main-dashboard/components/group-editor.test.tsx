import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GroupEditor } from "./group-editor";

describe("GroupEditor", () => {
  it("edits the current group and saves it", () => {
    const onChange = vi.fn();
    const onSave = vi.fn();

    render(
      <GroupEditor
        group={{
          id: "btc",
          symbol: "BTCUSDT",
          signalTypesText: "vegas,divMacd",
          periods: ["240", "60", "15"],
          selectedTimelinePeriod: "60",
        }}
        isSaving={false}
        onChange={onChange}
        onSave={onSave}
      />,
    );

    fireEvent.change(screen.getByLabelText("Symbol"), {
      target: { value: "ETHUSDT" },
    });
    fireEvent.change(screen.getByLabelText("Default timeline period"), {
      target: { value: "240" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save group/i }));

    expect(onChange).toHaveBeenCalled();
    expect(onSave).toHaveBeenCalled();
  });
});
