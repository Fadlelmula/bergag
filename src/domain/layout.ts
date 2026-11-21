import type { Density } from "./models";
import { NAV_ITEM_IDS, type NavItemId } from "./navigation";

export const PANEL_IDS = ["sidebar", "articleList", "readingPane"] as const;
export type PanelId = (typeof PANEL_IDS)[number];

export interface BergagLayout {
  version: 1;
  sidebar: {
    visible: boolean;
    collapsed: boolean;
    width: number;
    order: NavItemId[];
    hidden: NavItemId[];
  };
  panels: {
    articleList: { visible: boolean; width: number };
    readingPane: { visible: boolean };
  };
  order: PanelId[];
  density: Density;
}

export const DEFAULT_LAYOUT: BergagLayout = {
  version: 1,
  sidebar: {
    visible: true,
    collapsed: false,
    width: 248,
    order: [...NAV_ITEM_IDS],
    hidden: [],
  },
  panels: {
    articleList: { visible: true, width: 396 },
    readingPane: { visible: true },
  },
  order: [...PANEL_IDS],
  density: "comfortable",
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function clamp(value: unknown, minimum: number, maximum: number, fallback: number): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return fallback;
  return Math.min(maximum, Math.max(minimum, Math.round(value)));
}

function boolean(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function density(value: unknown): Density {
  return value === "compact" || value === "comfortable" || value === "spacious"
    ? value
    : "comfortable";
}

function navIdList(value: unknown, fallback: readonly NavItemId[]): NavItemId[] {
  if (!Array.isArray(value)) return [...fallback];
  const valid = value.filter(
    (item): item is NavItemId =>
      typeof item === "string" && (NAV_ITEM_IDS as readonly string[]).includes(item),
  );
  const missing = fallback.filter((item) => !valid.includes(item));
  return [...valid, ...missing];
}

function panelIdList(value: unknown): PanelId[] {
  if (!Array.isArray(value)) return [...PANEL_IDS];
  const valid = value.filter(
    (item): item is PanelId =>
      typeof item === "string" && (PANEL_IDS as readonly string[]).includes(item),
  );
  const missing = PANEL_IDS.filter((item) => !valid.includes(item));
  return [...valid, ...missing];
}

export function normalizeLayout(value: unknown): BergagLayout {
  if (!isRecord(value)) return structuredClone(DEFAULT_LAYOUT);
  const sidebar = isRecord(value.sidebar) ? value.sidebar : {};
  const panels = isRecord(value.panels) ? value.panels : {};
  const articleList = isRecord(panels.articleList) ? panels.articleList : {};
  const readingPane = isRecord(panels.readingPane) ? panels.readingPane : {};

  return {
    version: 1,
    sidebar: {
      visible: boolean(sidebar.visible, DEFAULT_LAYOUT.sidebar.visible),
      collapsed: boolean(sidebar.collapsed, DEFAULT_LAYOUT.sidebar.collapsed),
      width: clamp(sidebar.width, 200, 420, DEFAULT_LAYOUT.sidebar.width),
      order: navIdList(sidebar.order, DEFAULT_LAYOUT.sidebar.order),
      hidden: Array.isArray(sidebar.hidden)
        ? navIdList(sidebar.hidden, [])
        : [...DEFAULT_LAYOUT.sidebar.hidden],
    },
    panels: {
      articleList: {
        visible: boolean(articleList.visible, DEFAULT_LAYOUT.panels.articleList.visible),
        width: clamp(articleList.width, 280, 720, DEFAULT_LAYOUT.panels.articleList.width),
      },
      readingPane: {
        visible: boolean(readingPane.visible, DEFAULT_LAYOUT.panels.readingPane.visible),
      },
    },
    order: panelIdList(value.order),
    density: density(value.density),
  };
}

export function layoutFromJson(json: string): BergagLayout {
  try {
    return normalizeLayout(JSON.parse(json) as unknown);
  } catch {
    return structuredClone(DEFAULT_LAYOUT);
  }
}

export function layoutToJson(layout: BergagLayout): string {
  return JSON.stringify(layout);
}
