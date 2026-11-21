export const NAV_ITEM_IDS = [
  "home",
  "allFeeds",
  "unread",
  "readLater",
  "favorites",
  "archive",
  "queue",
  "history",
  "discover",
  "briefings",
  "highlights",
  "topics",
  "statistics",
  "ai",
  "search",
  "settings",
] as const;

export type NavItemId = (typeof NAV_ITEM_IDS)[number];

export interface NavItemDefinition {
  id: NavItemId;
  label: string;
  description: string;
  milestone: number;
  available: boolean;
  section: "workspace" | "library" | "intelligence" | "system";
}

export const NAV_ITEMS: readonly NavItemDefinition[] = [
  {
    id: "home",
    label: "Home",
    description: "Foundation status and getting-started information",
    milestone: 1,
    available: true,
    section: "workspace",
  },
  {
    id: "allFeeds",
    label: "All Feeds",
    description: "Subscribed sources, folders, and feed health",
    milestone: 2,
    available: true,
    section: "library",
  },
  {
    id: "unread",
    label: "Unread",
    description: "Unread articles across all sources",
    milestone: 3,
    available: true,
    section: "library",
  },
  {
    id: "readLater",
    label: "Read Later",
    description: "Saved articles",
    milestone: 3,
    available: true,
    section: "library",
  },
  {
    id: "favorites",
    label: "Favorites",
    description: "Favorited articles",
    milestone: 3,
    available: true,
    section: "library",
  },
  {
    id: "archive",
    label: "Archive",
    description: "Archived articles",
    milestone: 3,
    available: true,
    section: "library",
  },
  {
    id: "queue",
    label: "Reading queue",
    description: "Ordered articles saved for later reading",
    milestone: 3,
    available: true,
    section: "library",
  },
  {
    id: "history",
    label: "History",
    description: "Reading history",
    milestone: 3,
    available: true,
    section: "library",
  },
  {
    id: "discover",
    label: "Discover",
    description: "Local source directory and automation rules",
    milestone: 4,
    available: true,
    section: "workspace",
  },
  {
    id: "briefings",
    label: "Briefings",
    description: "Daily and weekly local briefings",
    milestone: 6,
    available: true,
    section: "intelligence",
  },
  {
    id: "highlights",
    label: "Highlights",
    description: "Saved highlights and annotations",
    milestone: 3,
    available: true,
    section: "intelligence",
  },
  {
    id: "topics",
    label: "Topics",
    description: "Tags and cross-feed topics",
    milestone: 4,
    available: true,
    section: "intelligence",
  },
  {
    id: "statistics",
    label: "Statistics",
    description: "Private dashboards and reading analytics",
    milestone: 6,
    available: true,
    section: "intelligence",
  },
  {
    id: "ai",
    label: "AI",
    description: "Opt-in article assistance and local AI usage",
    milestone: 7,
    available: true,
    section: "intelligence",
  },
  {
    id: "search",
    label: "Search",
    description: "Full-text library search and story timelines",
    milestone: 5,
    available: true,
    section: "workspace",
  },
  {
    id: "settings",
    label: "Settings",
    description: "Application settings and diagnostics",
    milestone: 1,
    available: true,
    section: "system",
  },
] as const;

export const NAV_ITEM_MAP = new Map(NAV_ITEMS.map((item) => [item.id, item]));

export function getNavItem(id: NavItemId): NavItemDefinition {
  const item = NAV_ITEM_MAP.get(id);
  if (!item) {
    throw new Error(`Unknown navigation item: ${id}`);
  }
  return item;
}
