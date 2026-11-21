import type { NavItemId } from "../domain/navigation";
import type { ArticleMode, ArticleState } from "../domain/models";
import { formatRelativeTime } from "./feedService";

export const ARTICLE_MODES: readonly ArticleMode[] = [
  "all",
  "unread",
  "saved",
  "favorites",
  "archive",
  "queue",
  "history",
  "highlights",
  "collection",
];

const MODE_LABELS: Record<ArticleMode, string> = {
  all: "All articles",
  unread: "Unread",
  saved: "Read later",
  favorites: "Favorites",
  archive: "Archive",
  queue: "Reading queue",
  history: "History",
  highlights: "Highlights",
  collection: "Collection",
};

const NAV_MODES: Partial<Record<NavItemId, ArticleMode>> = {
  unread: "unread",
  readLater: "saved",
  favorites: "favorites",
  archive: "archive",
  history: "history",
  highlights: "highlights",
  queue: "queue",
};

export const ARTICLE_NAV_IDS: readonly NavItemId[] = [
  "unread",
  "readLater",
  "favorites",
  "archive",
  "queue",
  "history",
  "highlights",
];

export function isArticleView(view: NavItemId): boolean {
  return ARTICLE_NAV_IDS.includes(view);
}

export function articleModeFromNav(view: NavItemId): ArticleMode {
  return NAV_MODES[view] ?? "unread";
}

export function articleModeLabel(mode: ArticleMode): string {
  return MODE_LABELS[mode];
}

export function articleEmptyMessage(mode: ArticleMode): string {
  switch (mode) {
    case "unread":
      return "Nothing is unread right now. Refresh a feed or mark an article unread to add it here.";
    case "saved":
      return "No saved articles. Use Save or the S shortcut while reading.";
    case "favorites":
      return "No favorites yet. Favorite an article to keep it close.";
    case "archive":
      return "The archive is empty.";
    case "queue":
      return "The reading queue is empty. Add an article from the reader.";
    case "history":
      return "No reading history has been recorded.";
    case "highlights":
      return "No highlights yet. Select article text while reading to save it.";
    case "collection":
      return "This collection is empty.";
    default:
      return "No articles have been discovered yet. Subscribe to a feed first.";
  }
}

export function articleStateBadges(state: ArticleState): string[] {
  const badges: string[] = [];
  if (!state.is_read) badges.push("Unread");
  if (state.is_saved) badges.push("Saved");
  if (state.is_favorite) badges.push("Favorite");
  if (state.is_archived) badges.push("Archived");
  if (state.is_queued)
    badges.push(`Queue${state.queue_position ? ` ${state.queue_position}` : ""}`);
  return badges;
}

export function formatArticleDate(value: string | null, reference = Date.now()): string {
  return formatRelativeTime(value, reference);
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function errorMessage(error: unknown, fallback = "The operation failed."): string {
  return error instanceof Error ? error.message : error ? String(error) : fallback;
}
