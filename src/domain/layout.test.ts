import { describe, expect, it } from "vitest";
import { DEFAULT_LAYOUT, layoutFromJson, layoutToJson, normalizeLayout } from "./layout";
import { NAV_ITEM_IDS } from "./navigation";

describe("layout normalization", () => {
  it("returns a complete default layout", () => {
    expect(DEFAULT_LAYOUT.version).toBe(1);
    expect(DEFAULT_LAYOUT.sidebar.order).toEqual([...NAV_ITEM_IDS]);
    expect(DEFAULT_LAYOUT.panels.articleList.width).toBeGreaterThan(0);
  });

  it("merges partial persisted layouts with defaults", () => {
    const normalized = normalizeLayout({
      sidebar: { collapsed: true, width: 9999 },
      panels: { articleList: { visible: false } },
      density: "compact",
    });

    expect(normalized.sidebar.collapsed).toBe(true);
    expect(normalized.sidebar.width).toBe(420);
    expect(normalized.sidebar.order).toEqual([...NAV_ITEM_IDS]);
    expect(normalized.panels.articleList.visible).toBe(false);
    expect(normalized.panels.articleList.width).toBe(DEFAULT_LAYOUT.panels.articleList.width);
    expect(normalized.density).toBe("compact");
  });

  it("drops unknown navigation ids and keeps defaults", () => {
    const normalized = normalizeLayout({
      sidebar: { order: ["home", "not-a-real-item"], hidden: ["not-a-real-item"] },
    });
    expect(normalized.sidebar.order[0]).toBe("home");
    expect(normalized.sidebar.order).toContain("settings");
    expect(normalized.sidebar.hidden).toEqual([]);
  });

  it("round-trips through JSON and falls back on invalid JSON", () => {
    expect(layoutFromJson(layoutToJson(DEFAULT_LAYOUT))).toEqual(DEFAULT_LAYOUT);
    expect(layoutFromJson("{not-json")).toEqual(DEFAULT_LAYOUT);
  });
});
