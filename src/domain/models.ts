export interface Profile {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  is_locked: boolean;
  lock_hint: string | null;
}

export interface LayoutRecord {
  id: string;
  name: string;
  is_active: boolean;
  layout_json: string;
  created_at: string;
  updated_at: string;
}

export interface DbInfo {
  path: string;
  size_bytes: number;
  sqlite_version: string;
  migration_count: number;
  user_version: number;
  integrity_check: string;
  fts5_available: boolean;
}

export interface Diagnostics {
  app_version: string;
  tauri_version: string;
  os: string;
  arch: string;
  telemetry_enabled: boolean;
  log_dir: string;
  database: DbInfo;
}

export interface FeedFolder {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  feed_count: number;
}

export interface Feed {
  id: string;
  folder_id: string | null;
  title: string;
  site_url: string | null;
  feed_url: string;
  feed_type: "rss" | "atom" | "json" | "unknown";
  description: string | null;
  language: string | null;
  status: "active" | "paused";
  last_refreshed_at: string | null;
  last_success_at: string | null;
  next_refresh_at: string;
  refresh_interval_minutes: number;
  consecutive_failures: number;
  last_error: string | null;
  article_count: number;
  health: "healthy" | "degraded" | "failing";
  created_at: string;
  updated_at: string;
  category: string | null;
  trust_level: TrustLevel;
}

export type TrustLevel = "untrusted" | "low" | "medium" | "high";

export interface DiscoverySource {
  id: string;
  feed_url: string;
  site_url: string | null;
  title: string;
  category: string | null;
  description: string | null;
  trust_level: TrustLevel;
  subscribed_feed_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface DiscoveryPreviewResult {
  preview: FeedPreview;
  source: DiscoverySource | null;
}

export type LabelKind = "tag" | "topic";

export interface Label {
  id: string;
  kind: LabelKind;
  name: string;
  color: string | null;
  description: string | null;
  usage_count: number;
  created_at: string;
  updated_at: string;
}

export type RuleMatchMode = "all" | "any";

export type RuleCondition =
  | { field: "titleContains"; value: string }
  | { field: "summaryContains"; value: string }
  | { field: "contentContains"; value: string }
  | { field: "feedIs"; feedId: string }
  | { field: "feedTitleContains"; value: string }
  | { field: "categoryIs"; value: string }
  | { field: "trustAtLeast"; value: TrustLevel }
  | { field: "hasLabel"; labelId: string }
  | { field: "isRead"; value: boolean }
  | { field: "isSaved"; value: boolean }
  | { field: "isFavorite"; value: boolean }
  | { field: "isArchived"; value: boolean }
  | { field: "isQueued"; value: boolean };

export type RuleAction =
  | { action: "markRead"; value: boolean }
  | { action: "setSaved"; value: boolean }
  | { action: "setFavorite"; value: boolean }
  | { action: "setArchived"; value: boolean }
  | { action: "setQueued"; value: boolean }
  | { action: "addLabel"; labelId: string }
  | { action: "removeLabel"; labelId: string };

export interface RuleDefinition {
  matchMode: RuleMatchMode;
  conditions: RuleCondition[];
  actions: RuleAction[];
}

export interface Rule {
  id: string;
  name: string;
  description: string | null;
  rule_json: string;
  enabled: boolean;
  is_smart_feed: boolean;
  last_run_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface RuleRun {
  id: string;
  rule_id: string;
  started_at: string;
  finished_at: string;
  status: "success" | "error";
  matched_count: number;
  changed_count: number;
  message: string | null;
}

export interface FeedItem {
  id: string;
  feed_id: string;
  guid: string;
  title: string;
  url: string | null;
  author: string | null;
  summary: string | null;
  published_at: string | null;
  discovered_at: string;
}

export interface RefreshLog {
  id: string;
  feed_id: string;
  started_at: string;
  finished_at: string;
  status: "success" | "not_modified" | "error" | "skipped";
  http_status: number | null;
  item_count: number;
  new_item_count: number;
  duration_ms: number;
  message: string | null;
}

export interface DiscoveredFeed {
  title: string;
  url: string;
  feed_type: string;
  source_url: string;
}

export interface FeedPreview {
  title: string;
  site_url: string | null;
  feed_type: string;
  description: string | null;
  language: string | null;
  item_count: number;
  items: FeedItem[];
}

export type IntegrationKind =
  | "youtube"
  | "reddit"
  | "github"
  | "hacker_news"
  | "arxiv"
  | "podcast"
  | "medium"
  | "substack"
  | "mastodon"
  | "newsletter"
  | "website_feed"
  | "bluesky"
  | "website_monitor";

export interface IntegrationPlatform {
  kind: IntegrationKind;
  label: string;
  description: string;
  input_label: string;
  input_hint: string;
  available: boolean;
  unavailable_reason: string | null;
}

export interface IntegrationSource {
  id: string;
  kind: IntegrationKind;
  source_key: string;
  display_name: string;
  source_url: string | null;
  feed_url: string;
  feed_id: string | null;
  category: string | null;
  trust_level: TrustLevel;
  status: "active" | "error";
  last_error: string | null;
  created_at: string;
  updated_at: string;
}

export interface IntegrationPreviewResult {
  platform: IntegrationPlatform;
  resolved_feed_url: string;
  preview: FeedPreview;
  duplicate_feed: boolean;
  duplicate_source: boolean;
  existing_source: IntegrationSource | null;
}

export interface BackupRecord {
  id: string;
  label: string;
  created_at: string;
  schema_version: number;
  profile_count: number;
  size_bytes: number;
  automatic: boolean;
}

export interface BackupVerification {
  valid: boolean;
  reason: string | null;
}

export interface RestoreOutcome {
  safety_backup_id: string;
  schema_version: number;
  profile_count: number;
}

export interface PrivacyDashboard {
  app_version: string;
  schema_version: number;
  telemetry_enabled: boolean;
  ai_enabled: boolean;
  ai_provider_count: number;
  remote_ai_provider_count: number;
  local_ai_provider_count: number;
  ai_usage_event_count: number;
  integration_source_count: number;
  feed_count: number;
  article_count: number;
  note_count: number;
  highlight_count: number;
  label_count: number;
  rule_count: number;
  dashboard_count: number;
  profile_count: number;
  backup_count: number;
  database_path: string;
  backup_directory: string;
  log_directory: string;
  integrity_check: string;
}

export interface SubscribeResult {
  feed: Feed;
  preview: FeedPreview;
  duplicate: boolean;
}

export interface RefreshOutcome {
  feed_id: string;
  title: string;
  status: "success" | "not_modified" | "error";
  http_status: number | null;
  item_count: number;
  new_item_count: number;
  duration_ms: number;
  message: string | null;
}

export interface OpmlImportResult {
  imported: number;
  skipped: number;
  errors: string[];
}

export type ArticleMode =
  | "all"
  | "unread"
  | "saved"
  | "favorites"
  | "archive"
  | "history"
  | "queue"
  | "highlights"
  | "collection";

export type ArticlePresentation = "list" | "magazine" | "minimal";

export interface ArticleState {
  is_read: boolean;
  is_saved: boolean;
  is_favorite: boolean;
  is_archived: boolean;
  is_queued: boolean;
  queue_position: number | null;
  read_count: number;
  last_read_at: string | null;
}

export interface ArticleStatePatch {
  isRead?: boolean;
  isSaved?: boolean;
  isFavorite?: boolean;
  isArchived?: boolean;
  isQueued?: boolean;
}

export interface ArticleListItem {
  id: string;
  feed_id: string;
  feed_title: string;
  title: string;
  url: string | null;
  author: string | null;
  summary: string | null;
  published_at: string | null;
  discovered_at: string;
  word_count: number;
  state: ArticleState;
}

export interface ArticleImage {
  url: string;
  mime: string;
  data_url: string;
}

export interface ArticleNote {
  id: string;
  body: string;
  created_at: string;
  updated_at: string;
}

export interface ArticleHighlight {
  id: string;
  text: string;
  start_offset: number;
  end_offset: number;
  created_at: string;
}

export interface ArticleCollection {
  id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
  article_count: number;
}

export interface ArticleDetail {
  item: ArticleListItem;
  source: "feed" | "manual";
  content_text: string;
  safe_html: string;
  images: ArticleImage[];
  notes: ArticleNote[];
  highlights: ArticleHighlight[];
  collections: ArticleCollection[];
}

export interface ArticleHistoryEntry {
  id: string;
  item_id: string;
  title: string;
  feed_title: string;
  event_type: "opened" | "read" | "unread";
  occurred_at: string;
}

export interface ArticleHighlightEntry {
  id: string;
  item_id: string;
  article_title: string;
  feed_title: string;
  text: string;
  created_at: string;
}

export interface ArticleCacheStats {
  articles: number;
  article_bytes: number;
  images: number;
  image_bytes: number;
}

export interface ReadingSettings {
  autoMarkRead: boolean;
  fontSize: number;
  lineHeight: number;
  presentation: ArticlePresentation;
}

export interface OfflineSettings {
  cacheImages: boolean;
}

export type ThemeMode = "dark" | "light" | "system";
export type Density = "compact" | "comfortable" | "spacious";

export interface AppearanceSettings {
  theme: ThemeMode;
  accent: string;
  density: Density;
  reduceMotion: boolean;
}

export type SearchScope = "all" | "unread" | "saved" | "favorites" | "archive" | "queue";

export interface SearchResult {
  item: ArticleListItem;
  snippet: string;
  rank: number;
}

export interface SearchPreset {
  id: string;
  name: string;
  query: string;
  scope: SearchScope;
  created_at: string;
  updated_at: string;
}

export interface SearchStats {
  indexed_documents: number;
  presets: number;
  story_groups: number;
  story_items: number;
}

export interface StoryGroup {
  id: string;
  title: string;
  item_count: number;
  source_count: number;
  latest_at: string | null;
  created_at: string;
  items: ArticleListItem[];
}

export interface StoryTimelineBucket {
  date: string;
  stories: StoryGroup[];
}

export interface RelatedArticle {
  item: ArticleListItem;
  score: number;
  reasons: string[];
}

export interface StoryDetection {
  groups_created: number;
  items_grouped: number;
}

export interface ActivityDay {
  date: string;
  opened: number;
  read: number;
  unread: number;
}

export interface ReadingStats {
  days: number;
  activity: ActivityDay[];
  total_opened: number;
  total_read: number;
  highlights: number;
  notes: number;
  streak_days: number;
  last_read_at: string | null;
}

export interface LibraryStats {
  articles: number;
  feeds: number;
  folders: number;
  collections: number;
  labels: number;
  highlights: number;
  notes: number;
  stories: number;
  presets: number;
  words: number;
  oldest_article_at: string | null;
  newest_article_at: string | null;
}

export interface StorageStats {
  database_bytes: number;
  cached_image_bytes: number;
  cached_images: number;
  extracted_articles: number;
  text_characters: number;
}

export interface FeedHealth {
  id: string;
  title: string;
  status: string;
  health: "healthy" | "degraded" | "failing";
  consecutive_failures: number;
  failures_30d: number;
  last_refreshed_at: string | null;
  last_success_at: string | null;
  last_error: string | null;
  items: number;
  unread: number;
}

export interface NamedCount {
  id: string;
  name: string;
  count: number;
  kind: string | null;
}

export interface BriefingHighlight {
  item_id: string;
  article_title: string;
  text: string;
  created_at: string;
}

export interface Briefing {
  period: "daily" | "weekly";
  since: string;
  generated_at: string;
  new_articles: number;
  read_events: number;
  unread_total: number;
  highlights_saved: number;
  notes_added: number;
  top_feeds: NamedCount[];
  recent_highlights: BriefingHighlight[];
}

export interface Dashboard {
  id: string;
  name: string;
  description: string | null;
  is_default: boolean;
  widget_count: number;
  created_at: string;
  updated_at: string;
}

export interface DashboardWidget {
  id: string;
  dashboard_id: string;
  kind: string;
  title: string;
  days: number;
  item_limit: number;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface NotificationSettings {
  enabled: boolean;
  feedRefresh: boolean;
  backupComplete: boolean;
  quietHoursEnabled: boolean;
  quietHourStart: number;
  quietHourEnd: number;
}

export interface NotificationStatus {
  supported: boolean;
  permission: string;
  settings: NotificationSettings;
}

export interface TrayStatus {
  available: boolean;
  closeBehavior: "quit" | "tray";
}

export interface StartupStatus {
  available: boolean;
  enabled: boolean;
  detail: string;
}

export interface UpdateStatus {
  configured: boolean;
  signatureVerification: boolean;
  endpointHost: string | null;
  currentVersion: string;
  checkAvailable: boolean;
  installAvailable: boolean;
  detail: string;
}

export interface UpdateCheck {
  available: boolean;
  version: string | null;
  checkedAt: string;
  detail: string;
}

export interface SecurityStatus {
  profileLockAvailable: boolean;
  lockedProfileCount: number;
  lockDetail: string;
  credentialStore: string;
  credentialGetterCommand: boolean;
  sqliteSecretColumns: number;
  publicUrlValidation: boolean;
  htmlSanitization: boolean;
  logRedaction: boolean;
  genericShellOrFilesystemAccess: boolean;
}

export type {
  AiCompletion,
  AiFeatureSettings,
  AiPrivacySettings,
  AiProvider,
  AiProviderKind,
  AiSettings,
  AiTestResult,
  AiUsageEvent,
  AiUsageStats,
  NamedAiCount,
} from "./aiModels";
