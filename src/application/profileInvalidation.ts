import type { QueryClient, QueryKey } from "@tanstack/react-query";

const PROFILE_SCOPED_ROOTS = new Set([
  "settings",
  "layouts",
  "feeds",
  "articles",
  "labels",
  "rules",
  "discovery",
  "search",
  "stories",
  "dashboards",
  "analytics",
  "briefings",
  "ai",
  "integrations",
]);

function isScopedTo(key: QueryKey, profileId: string): boolean {
  return typeof key[0] === "string" && key[1] === profileId && PROFILE_SCOPED_ROOTS.has(key[0]);
}

export function invalidateProfileData(
  queryClient: QueryClient,
  profileIds: Array<string | undefined>,
): Promise<void> {
  const ids = [...new Set(profileIds.filter((id): id is string => Boolean(id)))];
  if (ids.length === 0) return Promise.resolve();
  return queryClient.invalidateQueries({
    predicate: (query) => ids.some((id) => isScopedTo(query.queryKey, id)),
  });
}
