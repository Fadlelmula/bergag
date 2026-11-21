import type { DashboardWidget } from "../domain/models";

export const WIDGET_KINDS = [
  {
    kind: "unread_by_feed",
    label: "Unread by feed",
    description: "Feeds with the largest local unread counts.",
  },
  {
    kind: "recent_articles",
    label: "Recent articles",
    description: "Articles most recently discovered in this profile.",
  },
  {
    kind: "reading_activity",
    label: "Reading activity",
    description: "Daily open and read events from local article history.",
  },
  {
    kind: "top_labels",
    label: "Top labels",
    description: "Labels with the most article assignments.",
  },
  {
    kind: "feed_health",
    label: "Feed health",
    description: "Local refresh state and recent failures for subscriptions.",
  },
  {
    kind: "storage",
    label: "Local storage",
    description: "Database size and profile-owned cached content.",
  },
  {
    kind: "briefing",
    label: "Briefing",
    description: "A daily or weekly briefing generated from local data.",
  },
  {
    kind: "reading_queue",
    label: "Reading queue",
    description: "Articles you explicitly added to the reading queue.",
  },
] as const;

export type WidgetKind = (typeof WIDGET_KINDS)[number]["kind"];

export function isWidgetKind(value: string): value is WidgetKind {
  return WIDGET_KINDS.some((option) => option.kind === value);
}

export function widgetKindLabel(kind: string): string {
  return WIDGET_KINDS.find((option) => option.kind === kind)?.label ?? "Unknown widget";
}

export function widgetKindDescription(kind: string): string {
  return (
    WIDGET_KINDS.find((option) => option.kind === kind)?.description ??
    "This widget configuration is not supported."
  );
}

export function sortWidgets(widgets: DashboardWidget[]): DashboardWidget[] {
  return [...widgets].sort(
    (left, right) =>
      left.sort_order - right.sort_order || left.created_at.localeCompare(right.created_at),
  );
}

export function normalizeDashboardName(raw: string): string {
  return raw.trim().split(/\s+/).join(" ");
}

export function isValidWidgetTitle(raw: string): boolean {
  const title = raw.trim();
  return (
    title.length > 0 &&
    title.length <= 96 &&
    ![...title].some((character) => character.charCodeAt(0) < 32)
  );
}

export function briefingPeriodForDays(days: number): "daily" | "weekly" {
  return days <= 1 ? "daily" : "weekly";
}

export function usesDayRange(kind: string): boolean {
  return kind === "reading_activity" || kind === "briefing";
}

export function usesItemLimit(kind: string): boolean {
  return (
    kind === "unread_by_feed" ||
    kind === "recent_articles" ||
    kind === "top_labels" ||
    kind === "feed_health" ||
    kind === "reading_queue"
  );
}
