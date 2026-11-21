export const queryKeys = {
  profiles: ["profiles"] as const,
  activeProfile: ["profiles", "active"] as const,
  appearance: (profileId: string) => ["settings", profileId, "appearance"] as const,
  settings: (profileId: string, key: string) => ["settings", profileId, key] as const,
  activeLayout: (profileId: string) => ["layouts", profileId, "active"] as const,
  layouts: (profileId: string) => ["layouts", profileId] as const,
  diagnostics: ["diagnostics"] as const,
  recentLogs: ["diagnostics", "logs"] as const,
  feedFolders: (profileId: string) => ["feeds", profileId, "folders"] as const,
  feeds: (profileId: string) => ["feeds", profileId, "list"] as const,
  feedItems: (profileId: string, feedId: string) => ["feeds", profileId, "items", feedId] as const,
  feedLogs: (profileId: string, feedId: string) => ["feeds", profileId, "logs", feedId] as const,
  articles: (profileId: string) => ["articles", profileId] as const,
  articleList: (profileId: string, mode: string, collectionId: string | null) =>
    ["articles", profileId, "list", mode, collectionId ?? "none"] as const,
  articleDetail: (profileId: string, itemId: string) =>
    ["articles", profileId, "detail", itemId] as const,
  articleCollections: (profileId: string) => ["articles", profileId, "collections"] as const,
  articleHistory: (profileId: string) => ["articles", profileId, "history"] as const,
  articleHighlights: (profileId: string) => ["articles", profileId, "highlights"] as const,
  articleCacheStats: (profileId: string) => ["articles", profileId, "cache"] as const,
  discoverySources: (profileId: string) => ["discovery", profileId, "sources"] as const,
  labels: (profileId: string) => ["labels", profileId] as const,
  labelsForItem: (profileId: string, itemId: string) =>
    ["labels", profileId, "item", itemId] as const,
  labelItems: (profileId: string, labelId: string) =>
    ["labels", profileId, "items", labelId] as const,
  rules: (profileId: string) => ["rules", profileId] as const,
  ruleHistory: (profileId: string, ruleId: string) =>
    ["rules", profileId, "history", ruleId] as const,
  smartFeed: (profileId: string, ruleId: string) => ["rules", profileId, "smart", ruleId] as const,
  search: (profileId: string, query: string, scope: string) =>
    ["search", profileId, query, scope] as const,
  searchPresets: (profileId: string) => ["search", profileId, "presets"] as const,
  searchStats: (profileId: string) => ["search", profileId, "stats"] as const,
  stories: (profileId: string) => ["stories", profileId, "list"] as const,
  storyTimeline: (profileId: string, days: number) =>
    ["stories", profileId, "timeline", days] as const,
  storyDetail: (profileId: string, storyId: string) =>
    ["stories", profileId, "detail", storyId] as const,
  relatedArticles: (profileId: string, itemId: string) =>
    ["articles", profileId, "related", itemId] as const,
  dashboards: (profileId: string) => ["dashboards", profileId] as const,
  dashboardWidgets: (profileId: string, dashboardId: string) =>
    ["dashboards", profileId, dashboardId, "widgets"] as const,
  dashboardWidgetData: (
    profileId: string,
    widgetId: string,
    kind: string,
    days: number,
    limit: number,
  ) => ["dashboards", profileId, "data", widgetId, kind, days, limit] as const,
  readingStats: (profileId: string, days: number) =>
    ["analytics", profileId, "reading", days] as const,
  libraryStats: (profileId: string) => ["analytics", profileId, "library"] as const,
  storageStats: (profileId: string) => ["analytics", profileId, "storage"] as const,
  feedHealth: (profileId: string, limit: number) =>
    ["analytics", profileId, "feedHealth", limit] as const,
  unreadFeeds: (profileId: string, limit: number) =>
    ["analytics", profileId, "unreadFeeds", limit] as const,
  topLabels: (profileId: string, limit: number) =>
    ["analytics", profileId, "topLabels", limit] as const,
  analyticsArticles: (profileId: string, kind: string, limit: number) =>
    ["analytics", profileId, "articles", kind, limit] as const,
  briefing: (profileId: string, period: string) => ["briefings", profileId, period] as const,
  aiSettings: (profileId: string) => ["ai", profileId, "settings"] as const,
  aiProviders: (profileId: string) => ["ai", profileId, "providers"] as const,
  aiUsageStats: (profileId: string, days: number) => ["ai", profileId, "usage", days] as const,
  aiUsageEvents: (profileId: string, limit: number) =>
    ["ai", profileId, "usage", "events", limit] as const,
  aiArticles: (profileId: string) => ["ai", profileId, "articles"] as const,
  backups: ["backups"] as const,
  privacyDashboard: ["privacy", "dashboard"] as const,
  desktopNotifications: ["desktop", "notifications"] as const,
  desktopTray: ["desktop", "tray"] as const,
  desktopStartup: ["desktop", "startup"] as const,
  desktopUpdates: ["desktop", "updates"] as const,
  desktopSecurity: ["desktop", "security"] as const,
  integrationPlatforms: ["integrations", "platforms"] as const,
  integrationSources: (profileId: string) => ["integrations", profileId, "sources"] as const,
};
