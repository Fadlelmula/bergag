import type { Feed } from "../domain/models";

export function prepareFeedUrl(raw: string): string {
  return raw.trim();
}

export function formatRelativeTime(iso: string | null, reference = Date.now()): string {
  if (!iso) return "never";
  const timestamp = Date.parse(iso);
  if (Number.isNaN(timestamp)) return "unknown";
  const seconds = Math.max(0, Math.round((reference - timestamp) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  return new Date(timestamp).toLocaleDateString();
}

export function formatSchedule(iso: string | null, reference = Date.now()): string {
  if (!iso) return "not scheduled";
  const timestamp = Date.parse(iso);
  if (Number.isNaN(timestamp)) return "unknown";
  const seconds = Math.round((timestamp - reference) / 1000);
  if (seconds < 0) return "overdue";
  if (seconds < 60) return "soon";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `in ${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `in ${hours}h`;
  return `in ${Math.floor(hours / 24)}d`;
}

export function healthLabel(health: Feed["health"]): string {
  switch (health) {
    case "healthy":
      return "Healthy";
    case "degraded":
      return "Degraded";
    case "failing":
      return "Failing";
  }
}

export function feedTypeLabel(type: string): string {
  switch (type) {
    case "rss":
      return "RSS";
    case "atom":
      return "Atom";
    case "json":
      return "JSON";
    default:
      return "Unknown";
  }
}

export function refreshIntervalLabel(minutes: number): string {
  if (minutes < 60) return `${minutes} min`;
  if (minutes % 1440 === 0) return `${minutes / 1440} day${minutes === 1440 ? "" : "s"}`;
  if (minutes % 60 === 0) return `${minutes / 60} hour${minutes === 60 ? "" : "s"}`;
  return `${minutes} min`;
}
