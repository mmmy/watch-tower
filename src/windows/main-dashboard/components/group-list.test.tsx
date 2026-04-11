import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GroupList } from "./group-list";

describe("GroupList", () => {
  it("selects and deletes existing groups", () => {
    const onSelect = vi.fn();
    const onAdd = vi.fn();
    const onDelete = vi.fn();

    render(
      <GroupList
        groups={[
          { id: "btc", symbol: "BTCUSDT", signalTypesText: "vegas" },
          { id: "eth", symbol: "ETHUSDT", signalTypesText: "divMacd" },
        ]}
        selectedGroupId="btc"
        isSaving={false}
        onSelect={onSelect}
        onAdd={onAdd}
        onDelete={onDelete}
      />,
    );

    fireEvent.click(screen.getAllByRole("button", { name: /ETHUSDT/i })[0]!);
    fireEvent.click(screen.getByRole("button", { name: /Delete ETHUSDT/i }));
    fireEvent.click(screen.getByRole("button", { name: /add group/i }));

    expect(onSelect).toHaveBeenCalledWith("eth");
    expect(onDelete).toHaveBeenCalledWith("eth");
    expect(onAdd).toHaveBeenCalled();
  });
});
