import { describe, expect, it } from "vitest";
import type { DashboardWidget } from "../domain/models";
import {
  briefingPeriodForDays,
  isValidWidgetTitle,
  normalizeDashboardName,
  sortWidgets,
  usesDayRange,
  usesItemLimit,
  widgetKindDescription,
  widgetKindLabel,
  WIDGET_KINDS,
} from "./dashboardService";

function widget(overrides: Partial<DashboardWidget> = {}): DashboardWidget {
  return {
    id: "widget-1",
    dashboard_id: "dashboard-1",
    kind: "storage",
    title: "Local storage",
    days: 30,
    item_limit: 5,
    sort_order: 0,
    created_at: "2026-11-20T10:00:00Z",
    updated_at: "2026-11-20T10:00:00Z",
    ...overrides,
  };
}

describe("dashboardService", () => {
  it("exposes documented widget types only", () => {
    expect(WIDGET_KINDS.map((option) => option.kind)).toEqual([
      "unread_by_feed",
      "recent_articles",
      "reading_activity",
      "top_labels",
      "feed_health",
      "storage",
      "briefing",
      "reading_queue",
    ]);
    expect(widgetKindLabel("reading_activity")).toBe("Reading activity");
    expect(widgetKindLabel("unknown")).toBe("Unknown widget");
    expect(widgetKindDescription("storage")).toContain("Database size");
  });

  it("orders widgets by saved position then creation time", () => {
    const first = widget({ id: "a", sort_order: 1, created_at: "2026-11-20T10:00:00Z" });
    const second = widget({ id: "b", sort_order: 0, created_at: "2026-11-21T10:00:00Z" });
    const third = widget({ id: "c", sort_order: 0, created_at: "2026-11-19T10:00:00Z" });
    expect(sortWidgets([first, second, third]).map((item) => item.id)).toEqual(["c", "b", "a"]);
  });

  it("validates names and titles without mutating input", () => {
    expect(normalizeDashboardName("  Daily   Brief  ")).toBe("Daily Brief");
    expect(isValidWidgetTitle(" Reading activity ")).toBe(true);
    expect(isValidWidgetTitle("")).toBe(false);
    expect(isValidWidgetTitle("x".repeat(97))).toBe(false);
    expect(isValidWidgetTitle("Bad\tName")).toBe(false);
  });

  it("maps briefing widgets to generated periods", () => {
    expect(briefingPeriodForDays(1)).toBe("daily");
    expect(briefingPeriodForDays(7)).toBe("weekly");
  });

  it("exposes only settings each widget actually uses", () => {
    expect(usesDayRange("reading_activity")).toBe(true);
    expect(usesDayRange("briefing")).toBe(true);
    expect(usesDayRange("storage")).toBe(false);

    expect(usesItemLimit("unread_by_feed")).toBe(true);
    expect(usesItemLimit("reading_queue")).toBe(true);
    expect(usesItemLimit("storage")).toBe(false);
    expect(usesItemLimit("reading_activity")).toBe(false);
    expect(usesItemLimit("briefing")).toBe(false);
  });
});
