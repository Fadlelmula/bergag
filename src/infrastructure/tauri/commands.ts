import type {
  AiCompletion,
  AiProvider,
  AiSettings,
  AiTestResult,
  AiUsageEvent,
  AiUsageStats,
  ArticleCacheStats,
  ArticleCollection,
  ArticleDetail,
  ArticleHighlight,
  ArticleHighlightEntry,
  ArticleHistoryEntry,
  ArticleListItem,
  ArticleNote,
  ArticleState,
  ArticleStatePatch,
  Briefing,
  Dashboard,
  DashboardWidget,
  Diagnostics,
  NotificationSettings,
  NotificationStatus,
  SecurityStatus,
  StartupStatus,
  TrayStatus,
  UpdateCheck,
  UpdateStatus,
  FeedHealth,
  LibraryStats,
  DiscoveredFeed,
  DiscoveryPreviewResult,
  DiscoverySource,
  Feed,
  FeedFolder,
  FeedItem,
  BackupRecord,
  BackupVerification,
  FeedPreview,
  IntegrationKind,
  IntegrationPlatform,
  IntegrationPreviewResult,
  IntegrationSource,
  Label,
  PrivacyDashboard,
  RestoreOutcome,
  LabelKind,
  LayoutRecord,
  NamedCount,
  OpmlImportResult,
  ReadingStats,
  StorageStats,
  Profile,
  RefreshLog,
  RefreshOutcome,
  Rule,
  RuleDefinition,
  RuleRun,
  SearchPreset,
  SearchResult,
  SearchScope,
  SearchStats,
  RelatedArticle,
  StoryDetection,
  StoryGroup,
  StoryTimelineBucket,
  SubscribeResult,
  TrustLevel,
} from "../../domain/models";
import { invoke } from "./client";

export const desktop = {
  listProfiles: () => invoke<Profile[]>("profiles_list"),
  getActiveProfile: () => invoke<Profile>("profiles_get_active"),
  createProfile: (name: string) => invoke<Profile>("profiles_create", { name }),
  renameProfile: (id: string, name: string) => invoke<Profile>("profiles_rename", { id, name }),
  switchProfile: (id: string) => invoke<Profile>("profiles_switch", { id }),
  deleteProfile: (id: string) => invoke<Profile[]>("profiles_delete", { id }),

  getProfileSetting: <T>(key: string) => invoke<T | null>("settings_get_profile", { key }),
  setProfileSetting: (key: string, value: unknown) =>
    invoke<void>("settings_set_profile", { key, value }),
  deleteProfileSetting: (key: string) => invoke<void>("settings_delete_profile", { key }),

  listLayouts: () => invoke<LayoutRecord[]>("layouts_list"),
  getActiveLayout: () => invoke<LayoutRecord | null>("layouts_get_active"),
  saveLayout: (name: string, layoutJson: string, activate = true) =>
    invoke<LayoutRecord>("layouts_save", { name, layoutJson, activate }),
  activateLayout: (id: string) => invoke<LayoutRecord>("layouts_activate", { id }),
  deleteLayout: (id: string) => invoke<LayoutRecord[]>("layouts_delete", { id }),

  getDiagnostics: () => invoke<Diagnostics>("diagnostics_get"),
  getRecentLogs: (limit = 50) => invoke<string[]>("diagnostics_recent_logs", { limit }),

  listFeedFolders: () => invoke<FeedFolder[]>("feeds_list_folders"),
  createFeedFolder: (name: string) => invoke<FeedFolder>("feeds_create_folder", { name }),
  deleteFeedFolder: (id: string) => invoke<void>("feeds_delete_folder", { id }),
  listFeeds: () => invoke<Feed[]>("feeds_list"),
  getFeed: (id: string) => invoke<Feed>("feeds_get", { id }),
  previewFeed: (url: string) => invoke<FeedPreview>("feeds_preview", { url }),
  discoverFeeds: (url: string) => invoke<DiscoveredFeed[]>("feeds_discover", { url }),
  subscribeFeed: (url: string, folderId: string | null) =>
    invoke<SubscribeResult>("feeds_subscribe", { url, folderId }),
  updateFeed: (
    id: string,
    title: string,
    folderId: string | null,
    status: "active" | "paused",
    refreshIntervalMinutes: number,
    category: string | null = null,
    trustLevel: TrustLevel = "untrusted",
  ) =>
    invoke<Feed>("feeds_update", {
      id,
      title,
      folderId,
      status,
      refreshIntervalMinutes,
      category,
      trustLevel,
    }),
  deleteFeed: (id: string) => invoke<void>("feeds_delete", { id }),
  refreshFeeds: (feedId: string | null = null) =>
    invoke<RefreshOutcome[]>("feeds_refresh", { feedId }),
  listFeedLogs: (id: string, limit = 20) => invoke<RefreshLog[]>("feeds_logs", { id, limit }),
  listFeedItems: (feedId: string, limit = 50) =>
    invoke<FeedItem[]>("feeds_items", { feedId, limit }),
  exportOpml: () => invoke<string>("feeds_export_opml"),
  importOpml: (xml: string) => invoke<OpmlImportResult>("feeds_import_opml", { xml }),

  listDiscoverySources: () => invoke<DiscoverySource[]>("discovery_list"),
  previewDiscoverySource: (url: string) =>
    invoke<DiscoveryPreviewResult>("discovery_preview", { url }),
  addDiscoverySource: (url: string, category: string | null, trustLevel: TrustLevel) =>
    invoke<DiscoveryPreviewResult>("discovery_add", { url, category, trustLevel }),
  subscribeDiscoverySource: (sourceId: string, folderId: string | null) =>
    invoke<Feed>("discovery_subscribe", { sourceId, folderId }),
  setDiscoveryTrust: (sourceId: string, trustLevel: TrustLevel) =>
    invoke<DiscoverySource>("discovery_set_trust", { sourceId, trustLevel }),
  deleteDiscoverySource: (sourceId: string) => invoke<void>("discovery_delete", { sourceId }),

  listLabels: (kind: LabelKind | null = null) => invoke<Label[]>("labels_list", { kind }),
  createLabel: (
    kind: LabelKind,
    name: string,
    color: string | null = null,
    description: string | null = null,
  ) => invoke<Label>("labels_create", { kind, name, color, description }),
  updateLabel: (
    id: string,
    name: string,
    color: string | null = null,
    description: string | null = null,
  ) => invoke<Label>("labels_update", { id, name, color, description }),
  deleteLabel: (id: string) => invoke<void>("labels_delete", { id }),
  labelsForItem: (itemId: string) => invoke<Label[]>("labels_for_item", { itemId }),
  assignLabel: (labelId: string, itemId: string) =>
    invoke<void>("labels_assign", { labelId, itemId }),
  unassignLabel: (labelId: string, itemId: string) =>
    invoke<void>("labels_unassign", { labelId, itemId }),
  labelItems: (labelId: string, limit = 200) =>
    invoke<ArticleListItem[]>("labels_items", { labelId, limit }),

  listRules: () => invoke<Rule[]>("rules_list"),
  getRule: (id: string) => invoke<Rule>("rules_get", { id }),
  createRule: (
    name: string,
    description: string | null,
    rule: RuleDefinition,
    enabled: boolean,
    isSmartFeed: boolean,
  ) =>
    invoke<Rule>("rules_create", {
      name,
      description,
      ruleJson: JSON.stringify(rule),
      enabled,
      isSmartFeed,
    }),
  updateRule: (
    id: string,
    name: string,
    description: string | null,
    rule: RuleDefinition,
    enabled: boolean,
    isSmartFeed: boolean,
  ) =>
    invoke<Rule>("rules_update", {
      id,
      name,
      description,
      ruleJson: JSON.stringify(rule),
      enabled,
      isSmartFeed,
    }),
  deleteRule: (id: string) => invoke<void>("rules_delete", { id }),
  runRule: (id: string) => invoke<RuleRun>("rules_run", { id }),
  runEnabledRules: () => invoke<RuleRun[]>("rules_run_enabled"),
  listRuleHistory: (id: string, limit = 20) => invoke<RuleRun[]>("rules_history", { id, limit }),
  smartFeedItems: (id: string, limit = 200) =>
    invoke<ArticleListItem[]>("rules_smart_feed", { id, limit }),

  search: (queryText: string, scope: SearchScope = "all", limit = 100) =>
    invoke<SearchResult[]>("search_query", { queryText, scope, limit }),
  searchStats: () => invoke<SearchStats>("search_stats"),
  listSearchPresets: () => invoke<SearchPreset[]>("search_presets_list"),
  saveSearchPreset: (
    name: string,
    queryText: string,
    scope: SearchScope = "all",
    id: string | null = null,
  ) => invoke<SearchPreset>("search_presets_save", { id, name, queryText, scope }),
  deleteSearchPreset: (id: string) => invoke<void>("search_presets_delete", { id }),
  listStories: (limit = 50) => invoke<StoryGroup[]>("stories_list", { limit }),
  getStory: (id: string) => invoke<StoryGroup>("stories_get", { id }),
  storyTimeline: (days = 30, limit = 100) =>
    invoke<StoryTimelineBucket[]>("stories_timeline", { days, limit }),
  detectStories: () => invoke<StoryDetection>("stories_detect"),
  deleteStory: (id: string) => invoke<void>("stories_delete", { id }),
  relatedArticles: (id: string, limit = 5) =>
    invoke<RelatedArticle[]>("articles_related", { id, limit }),

  listDashboards: () => invoke<Dashboard[]>("dashboards_list"),
  saveDashboard: (id: string | null, name: string, description: string | null) =>
    invoke<Dashboard>("dashboard_save", { id, name, description }),
  deleteDashboard: (id: string) => invoke<void>("dashboard_delete", { id }),
  setDefaultDashboard: (id: string) => invoke<Dashboard>("dashboard_set_default", { id }),
  listWidgets: (dashboardId: string) => invoke<DashboardWidget[]>("widgets_list", { dashboardId }),
  addWidget: (dashboardId: string, kind: string) =>
    invoke<DashboardWidget>("widget_add", { dashboardId, kind }),
  updateWidget: (id: string, title: string, days: number, itemLimit: number, sortOrder: number) =>
    invoke<DashboardWidget>("widget_update", { id, title, days, itemLimit, sortOrder }),
  deleteWidget: (id: string) => invoke<void>("widget_delete", { id }),

  analyticsReading: (days = 30) => invoke<ReadingStats>("analytics_reading", { days }),
  analyticsLibrary: () => invoke<LibraryStats>("analytics_library"),
  analyticsStorage: () => invoke<StorageStats>("analytics_storage"),
  analyticsFeedHealth: (limit = 20) => invoke<FeedHealth[]>("analytics_feed_health", { limit }),
  analyticsUnreadFeeds: (limit = 5) => invoke<NamedCount[]>("analytics_unread_feeds", { limit }),
  analyticsTopLabels: (limit = 5) => invoke<NamedCount[]>("analytics_top_labels", { limit }),
  analyticsArticles: (kind: string, limit = 5) =>
    invoke<ArticleListItem[]>("analytics_articles", { kind, limit }),
  buildBriefing: (period: "daily" | "weekly") => invoke<Briefing>("briefing_build", { period }),

  listArticles: (mode: string, collectionId: string | null = null, limit = 200) =>
    invoke<ArticleListItem[]>("articles_list", { mode, collectionId, limit }),
  getArticle: (id: string) => invoke<ArticleDetail>("articles_get", { id }),
  openArticle: (id: string) => invoke<void>("articles_open", { id }),
  extractArticle: (id: string) => invoke<ArticleDetail>("articles_extract", { id }),
  setArticleState: (id: string, patch: ArticleStatePatch) =>
    invoke<ArticleState>("articles_set_state", { id, patch }),
  bulkUpdateArticles: (ids: string[], patch: ArticleStatePatch) =>
    invoke<number>("articles_bulk_update", { ids, patch }),
  listArticleHistory: (limit = 200) => invoke<ArticleHistoryEntry[]>("articles_history", { limit }),
  listArticleHighlights: (limit = 500) =>
    invoke<ArticleHighlightEntry[]>("articles_highlights", { limit }),
  addArticleNote: (id: string, body: string) =>
    invoke<ArticleNote>("articles_add_note", { id, body }),
  updateArticleNote: (noteId: string, body: string) =>
    invoke<ArticleNote>("articles_update_note", { noteId, body }),
  deleteArticleNote: (noteId: string) => invoke<void>("articles_delete_note", { noteId }),
  addArticleHighlight: (id: string, text: string, startOffset: number, endOffset: number) =>
    invoke<ArticleHighlight>("articles_add_highlight", {
      id,
      text,
      startOffset,
      endOffset,
    }),
  deleteArticleHighlight: (highlightId: string) =>
    invoke<void>("articles_delete_highlight", { highlightId }),
  listCollections: () => invoke<ArticleCollection[]>("articles_list_collections"),
  createCollection: (name: string, description: string | null = null) =>
    invoke<ArticleCollection>("articles_create_collection", { name, description }),
  deleteCollection: (id: string) => invoke<void>("articles_delete_collection", { id }),
  addToCollection: (collectionId: string, itemId: string) =>
    invoke<void>("articles_add_to_collection", { collectionId, itemId }),
  removeFromCollection: (collectionId: string, itemId: string) =>
    invoke<void>("articles_remove_from_collection", { collectionId, itemId }),
  articleCacheStats: () => invoke<ArticleCacheStats>("articles_cache_stats"),
  clearArticleCache: (itemId: string | null = null) =>
    invoke<number>("articles_clear_cache", { itemId }),

  backupCreate: (label: string | null = null) => invoke<BackupRecord>("backup_create", { label }),
  backupList: () => invoke<BackupRecord[]>("backup_list"),
  backupVerify: (id: string) => invoke<BackupVerification>("backup_verify", { id }),
  backupDelete: (id: string) => invoke<void>("backup_delete", { id }),
  backupRestore: (id: string) => invoke<RestoreOutcome>("backup_restore", { id }),
  privacyDashboard: () => invoke<PrivacyDashboard>("privacy_dashboard_get"),

  desktopNotificationsStatus: () => invoke<NotificationStatus>("desktop_notifications_status"),
  desktopNotificationsUpdate: (settings: NotificationSettings) =>
    invoke<NotificationSettings>("desktop_notifications_update", { settings }),
  desktopNotificationsTest: () => invoke<boolean>("desktop_notifications_test"),
  desktopTrayStatus: () => invoke<TrayStatus>("desktop_tray_status"),
  desktopTraySetCloseBehavior: (behavior: "quit" | "tray") =>
    invoke<TrayStatus>("desktop_tray_set_close_behavior", { behavior }),
  desktopStartupStatus: () => invoke<StartupStatus>("desktop_startup_status"),
  desktopStartupSetEnabled: (enabled: boolean) =>
    invoke<StartupStatus>("desktop_startup_set_enabled", { enabled }),
  desktopUpdatesStatus: () => invoke<UpdateStatus>("desktop_updates_status"),
  desktopUpdatesCheck: () => invoke<UpdateCheck>("desktop_updates_check"),
  desktopSecurityStatus: () => invoke<SecurityStatus>("desktop_security_status"),

  integrationPlatforms: () => invoke<IntegrationPlatform[]>("integrations_platforms_list"),
  integrationPreview: (kind: IntegrationKind, input: string) =>
    invoke<IntegrationPreviewResult>("integrations_preview", { kind, input }),
  integrationAdd: (
    kind: IntegrationKind,
    input: string,
    category: string | null,
    trustLevel: TrustLevel,
    folderId: string | null = null,
  ) =>
    invoke<IntegrationSource>("integrations_add", {
      kind,
      input,
      category,
      trustLevel,
      folderId,
    }),
  integrationList: () => invoke<IntegrationSource[]>("integrations_list"),
  integrationSetTrust: (sourceId: string, trustLevel: TrustLevel) =>
    invoke<IntegrationSource>("integrations_set_trust", { sourceId, trustLevel }),
  integrationRefresh: (sourceId: string) =>
    invoke<RefreshOutcome[]>("integrations_refresh", { sourceId }),
  integrationDelete: (sourceId: string) => invoke<void>("integrations_delete", { sourceId }),

  aiSettingsGet: () => invoke<AiSettings>("ai_settings_get"),
  aiSettingsUpdate: (settings: AiSettings) =>
    invoke<AiSettings>("ai_settings_update", { settings }),
  aiProvidersList: () => invoke<AiProvider[]>("ai_providers_list"),
  aiProviderSave: (
    id: string | null,
    name: string,
    kind: string,
    endpoint: string,
    model: string,
    enabled: boolean,
  ) => invoke<AiProvider>("ai_provider_save", { id, name, kind, endpoint, model, enabled }),
  aiProviderDelete: (providerId: string) => invoke<void>("ai_provider_delete", { providerId }),
  aiSetApiKey: (providerId: string, apiKey: string) =>
    invoke<void>("ai_set_api_key", { providerId, apiKey }),
  aiClearApiKey: (providerId: string) => invoke<void>("ai_clear_api_key", { providerId }),
  aiProviderTest: (providerId: string) => invoke<AiTestResult>("ai_provider_test", { providerId }),
  aiSummarize: (itemId: string) => invoke<AiCompletion>("ai_summarize", { itemId }),
  aiSuggestLabels: (itemId: string) => invoke<AiCompletion>("ai_suggest_labels", { itemId }),
  aiAsk: (itemId: string, question: string) => invoke<AiCompletion>("ai_ask", { itemId, question }),
  aiUsageStats: (days = 30) => invoke<AiUsageStats>("ai_usage_stats", { days }),
  aiUsageEvents: (limit = 50) => invoke<AiUsageEvent[]>("ai_usage_events", { limit }),
  aiUsageClear: () => invoke<number>("ai_usage_clear"),
};
