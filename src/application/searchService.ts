import type { SearchScope } from "../domain/models";

export const SEARCH_SCOPES: ReadonlyArray<{ id: SearchScope; label: string }> = [
  { id: "all", label: "All articles" },
  { id: "unread", label: "Unread" },
  { id: "saved", label: "Read later" },
  { id: "favorites", label: "Favorites" },
  { id: "archive", label: "Archive" },
  { id: "queue", label: "Reading queue" },
];

export const SEARCH_OPERATOR_HELP: ReadonlyArray<readonly [string, string]> = [
  ["feed:name", "Restrict to a feed title."],
  ["folder:name", "Restrict to a feed folder."],
  ["tag:name", "Match a tag, topic, or label."],
  ["author:name", "Match the stored author."],
  ["title:text", "Search the title only."],
  ["category:name", "Match a feed category."],
  ["trust:level", "untrusted, low, medium, or high."],
  ["before:YYYY-MM-DD", "Published before a date."],
  ["after:YYYY-MM-DD", "Published on or after a date."],
  ["is:state", "read, unread, saved, favorite, archived, queued."],
  ["has:value", "notes or highlights."],
  ["-term", "Exclude a term or operator."],
];

export function searchScopeLabel(scope: SearchScope): string {
  return SEARCH_SCOPES.find((item) => item.id === scope)?.label ?? "All articles";
}

export function formatStoryBucketDate(value: string): string {
  if (value === "Undated") return "Undated";
  const date = new Date(`${value}T00:00:00`);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}
