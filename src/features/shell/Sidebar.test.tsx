import * as Tooltip from "@radix-ui/react-tooltip";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NAV_ITEM_IDS } from "../../domain/navigation";
import { Sidebar } from "./Sidebar";

const activeProfile = {
  id: "profile-1",
  name: "Default",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
  is_locked: false,
  lock_hint: null,
};

function renderSidebar(onNavigate = vi.fn()) {
  render(
    <Tooltip.Provider>
      <Sidebar
        collapsed={false}
        width={248}
        order={[...NAV_ITEM_IDS]}
        hidden={[]}
        activeView="home"
        profiles={[activeProfile]}
        activeProfile={activeProfile}
        isSwitching={false}
        onNavigate={onNavigate}
        onToggleCollapse={vi.fn()}
        onHide={vi.fn()}
        onShow={vi.fn()}
        onMove={vi.fn()}
        onResetOrder={vi.fn()}
        onSwitchProfile={vi.fn()}
      />
    </Tooltip.Provider>,
  );
  return onNavigate;
}

describe("Sidebar", () => {
  it("marks implemented items as usable and future items as planned", () => {
    renderSidebar();
    expect(screen.getByTestId("nav-home")).toBeEnabled();
    expect(screen.getByTestId("nav-settings")).toBeEnabled();
    expect(screen.getByTestId("nav-allFeeds")).toBeEnabled();
    expect(screen.getByTestId("nav-unread")).toBeEnabled();
    expect(screen.getByTestId("nav-queue")).toBeEnabled();
    expect(screen.getByTestId("nav-highlights")).toBeEnabled();
    expect(screen.getByTestId("nav-discover")).toBeEnabled();
    expect(screen.getByTestId("nav-topics")).toBeEnabled();
    expect(screen.getByTestId("nav-search")).toBeEnabled();
    expect(screen.getByTestId("nav-briefings")).toBeEnabled();
    expect(screen.getByTestId("nav-statistics")).toBeEnabled();
    expect(screen.getByTestId("nav-ai")).toBeEnabled();
    expect(screen.queryAllByText("M3")).toHaveLength(0);
    expect(screen.queryAllByText("M4")).toHaveLength(0);
    expect(screen.queryAllByText("M5")).toHaveLength(0);
    expect(screen.queryAllByText("M6")).toHaveLength(0);
    expect(screen.queryAllByText("M7")).toHaveLength(0);
  });

  it("calls onNavigate for available items", () => {
    const onNavigate = renderSidebar();
    fireEvent.click(screen.getByTestId("nav-home"));
    expect(onNavigate).toHaveBeenCalledWith("home");
  });

  it("shows hidden items for restoration", () => {
    render(
      <Tooltip.Provider>
        <Sidebar
          collapsed={false}
          width={248}
          order={[...NAV_ITEM_IDS]}
          hidden={["topics"]}
          activeView="home"
          profiles={[activeProfile]}
          activeProfile={activeProfile}
          isSwitching={false}
          onNavigate={vi.fn()}
          onToggleCollapse={vi.fn()}
          onHide={vi.fn()}
          onShow={vi.fn()}
          onMove={vi.fn()}
          onResetOrder={vi.fn()}
          onSwitchProfile={vi.fn()}
        />
      </Tooltip.Provider>,
    );
    expect(screen.getByText("Hidden items (1)")).toBeInTheDocument();
    expect(screen.getByText("Show Topics")).toBeInTheDocument();
  });
});
