# Bergag architecture

This document describes the intended architecture and the implementation boundaries through Milestone 9.

## 1. Layering

```text
┌──────────────────────────────────────────────────────────┐
│ React UI                                                 │
│ features/  components/  app/                             │
├──────────────────────────────────────────────────────────┤
│ Application services (use cases)                         │
│ profileService, settingsService, layoutService, ...      │
├──────────────────────────────────────────────────────────┤
│ Domain types and pure logic                              │
│ Profile, Layout, Theme, validation, defaults             │
├──────────────────────────────────────────────────────────┤
│ Infrastructure adapters                                  │
│ Tauri command client, sanitizer, feed parser, AI adapters│
├──────────────────────────────────────────────────────────┤
│ Native boundary (Tauri commands)                         │
│ validated, narrow commands; no generic shell/fs access   │
├──────────────────────────────────────────────────────────┤
│ Rust native services                                     │
│ SQLite, migrations, credentials, feed fetch, validation  │
└──────────────────────────────────────────────────────────┘
```

Business rules that can be tested in TypeScript live in `src/domain` and `src/application`. Rust owns only native concerns and persistence primitives: SQLite, migrations, secure storage, OS integration, and network work that must avoid webview restrictions.

## 2. Desktop shell

Tauri 2 is used for the native window and bridge. The frontend has no unrestricted filesystem or shell capability. The only IPC surface is `generate_handler!` with explicitly named commands.

Native capabilities are declared in `src-tauri/capabilities/default.json`. The app still grants only core window/event defaults. Notification, autostart, and updater plugins are initialized in Rust and reached through named Bergag commands; their plugin commands are not granted to the webview. Feed networking is implemented inside Rust, so no additional webview capability was needed.

## 3. Database

SQLite is accessed through `rusqlite` with:

- `foreign_keys = ON`
- WAL journaling
- busy timeout
- versioned SQL migrations in Rust
- integrity checks exposed through diagnostics
- FTS5 full-text index maintained by SQLite triggers

### Current schema

| Table                 | Purpose                                                                  |
| --------------------- | ------------------------------------------------------------------------ |
| `schema_migrations`   | Applied migration ledger.                                                |
| `app_settings`        | Global, non-secret settings such as active profile id.                   |
| `profiles`            | Separate user spaces. All profile-scoped data references `profiles(id)`. |
| `profile_settings`    | Per-profile key/value settings stored as JSON.                           |
| `layouts`             | Saved layout presets per profile. Exactly one active layout per profile. |
| `feed_folders`        | Per-profile feed grouping with case-insensitive unique names.            |
| `feeds`               | Subscriptions with normalized URLs, status, health, and fetch state.     |
| `feed_items`          | Discovered items, deduplicated by `(feed_id, guid)`.                     |
| `feed_refresh_logs`   | Per-fetch status, timing, counts, and redacted error messages.           |
| `article_sources`     | Sanitized HTML, plain text, word count, image URLs, and content source.  |
| `article_states`      | Read/saved/favorite/archive/queue flags, queue order, and read history.  |
| `article_history`     | Local opened/read/unread events.                                         |
| `article_notes`       | Private per-article notes.                                               |
| `article_highlights`  | Selected text with offsets and creation time.                            |
| `collections`         | Named per-profile collections with case-insensitive unique names.        |
| `collection_items`    | Article membership in collections.                                       |
| `article_images`      | Cached raster image bytes for explicit offline images.                   |
| `discovery_sources`   | Local, user-managed feed directory with category and trust.              |
| `labels`              | Per-profile tags and topics with case-insensitive unique names per kind. |
| `label_assignments`   | Article membership in tags/topics, scoped by profile.                    |
| `rules`               | Local automation definitions, enabled/smart-feed flags, last run time.   |
| `rule_runs`           | Local rule run history with status, matched/changed counts, message.     |
| `search_documents`    | Profile-scoped FTS5 index of titles, body text, authors, feed titles.    |
| `search_presets`      | Saved queries and scopes with case-insensitive unique names.             |
| `story_groups`        | Local story/coverage groups with an optional canonical item.             |
| `story_group_items`   | Article membership and similarity score for each story group.            |
| `dashboards`          | User-authored dashboard records with one default per profile.            |
| `dashboard_widgets`   | Bounded widget configuration attached to a dashboard.                    |
| `ai_providers`        | Non-secret AI provider configuration; credentials stay in Windows.       |
| `ai_usage_events`     | Local AI request history with redacted errors and reported token counts. |
| `integration_sources` | Public-feed integration metadata linked to ordinary `feeds.id`.          |

Every profile-scoped table carries `profile_id` with `ON DELETE CASCADE`. Feed folders use a composite foreign key so a feed cannot reference a folder from another profile; deleting a folder sets its feeds' `folder_id` to `NULL` rather than deleting subscriptions. Article tables use composite `(profile_id, item_id)` foreign keys into `feed_items`, so deleting an item or profile cascades content, state, annotations, collection membership, and cached images. Labels and rules use composite `(profile_id, id)` uniqueness so assignments and run history cannot cross profiles. A discovery source references a subscribed feed with `ON DELETE SET NULL`; the subscription link is validated against the active profile in code. Search presets and story groups are profile-scoped; the FTS5 table is keyed by `profile_id`/`item_id` and kept in sync by triggers, including cascaded item deletions. Integration sources are profile-scoped, unique by normalized feed URL and by `(kind, source_key)`, and reference a feed with `ON DELETE SET NULL`. Deleting an integration source never deletes its feed or library content. Migration 9 adds profile-scoped due-feed, sorted-item, and case-insensitive feed-title indexes without changing content.

### Profile isolation

`switch_profile` changes a global pointer to a profile id. Every profile-owned query is keyed by that id, including feeds, folders, items, refresh logs, article content/state, annotations, collections, cached images, discovery sources, labels, rules, rule runs, search documents, saved searches, story groups, dashboards, widgets, AI configuration/usage, and integration sources. No credentials are stored in the database. AI credentials are stored in Windows Credential Manager through `keyring` 3.6.3; only non-secret provider identifiers and status flags live in SQLite.

## 4. Frontend

- React 19 + TypeScript + Vite.
- Tailwind CSS v4 with a custom CSS-variable design system.
- TanStack Query for async state and cache invalidation.
- Radix primitives for accessible dialogs, context menus, selects, switches, and tooltips.
- Lucide icons.
- Vitest + Testing Library for unit/component tests.

The UI is split into:

- `src/app`: providers and application composition.
- `src/domain`: types, defaults, validators, pure logic.
- `src/application`: service functions orchestrating commands.
- `src/infrastructure`: Tauri adapters and runtime detection.
- `src/features`: feature-specific UI.
- `src/components`: reusable design-system components.

## 5. Logging and diagnostics

Rust uses `tracing` with a rotating file appender in the application log directory. Diagnostics expose redacted information only: app version, OS, architecture, database path, migration count, integrity check, FTS5 availability, and recent log lines after credential-pattern redaction.

Secrets must never be passed to `tracing`, error dialogs, diagnostics, or exported settings.

## 6. Data flow example: switching profile

1. UI calls `profileService.switchProfile(id)`.
2. Rust validates that the profile exists.
3. Rust writes `active_profile_id` in `app_settings` inside a transaction.
4. TanStack Query invalidates only the previous and new profile's scoped data via `invalidateProfileData`, then refreshes the profile list and active-profile query.
5. UI re-renders with the new profile's isolated settings and layout.

## 7. Feed engine (Milestone 2)

Feed work is split between Rust and the UI:

- `src-tauri/src/feeds/network.rs` validates public `http`/`https` URLs, rejects userinfo, private/loopback/link-local addresses, and private redirect targets, pins DNS resolution for the request, and applies timeouts and response limits.
- `src-tauri/src/feeds/parser.rs` parses RSS, Atom, and JSON feeds, extracts autodiscovery links from HTML, and cleans text before persistence.
- `src-tauri/src/feeds/service.rs` orchestrates preview, discovery, subscription, manual refresh, scheduled refresh, backoff, and OPML import/export.
- `src-tauri/src/db/feeds.rs` owns feed persistence and state transitions. It never holds the SQLite mutex across network I/O.
- `src-tauri/src/commands/feeds.rs` exposes 15 explicit, validated `feeds_*` commands. No generic fetch, file, or shell command is exposed.
- `src-tauri/src/feeds/scheduler.rs` checks due feeds every 60 seconds and emits `feeds:refreshed`; the UI invalidates feed queries when that event arrives.

Subscriptions are deduplicated by normalized feed URL within a profile. Refreshes use `ETag` and `Last-Modified` with `304 Not Modified` handling, exponential backoff after failures (5–10080 minute interval guard), a batch size of 20, and a concurrency limit of 3. Feed entry HTML is retained in Rust for sanitization; the webview only receives `safe_html` produced by the article layer, and feed summaries remain available as plain-text fallback content.

## 8. Reader and article system (Milestone 3)

- `src-tauri/migrations/0003_article_system.sql` adds article sources/states/history, notes, highlights, collections, collection membership, and cached images. It backfills sanitized source rows and default states for items discovered by Milestone 2.
- `src-tauri/src/articles/content.rs` sanitizes with `ammonia::Builder::empty()`: a strict tag/attribute allowlist, clean-content removal for script/style/embed-like tags, `noopener noreferrer` links, relative URL rewriting against the source URL, and `UrlRelative::Deny` without a base.
- `src-tauri/src/articles/extraction.rs` selects a likely article body with `scraper`, uses the same public-URL validator as feeds, caps HTML at 5 MB and images at 2 MB each, accepts only common raster image types, and caches at most eight images.
- `src-tauri/src/articles/service.rs` reads `reading.autoMarkRead` (default `false`) and `offline.cacheImages` (default `true`) from profile settings. Opening an article always records local history; full-page extraction happens only when the user requests it.
- `src-tauri/src/articles/store.rs` and `src-tauri/src/commands/articles.rs` expose 20 narrow `articles_*` commands for list modes, detail/open/extract, state and bulk updates, history, annotations, collections, and cache statistics/clearing. All commands resolve the active profile server-side and validate identifiers, text lengths, modes, and limits.
- `src/features/articles` implements the article list, reader, and activity views. `src/application/articleService.ts` holds pure mode mapping, badge derivation, empty-state copy, and formatting helpers. TanStack Query invalidates article queries on `feeds:refreshed` and local mutations.
- Cached images are rewritten to `data:` URLs in sanitized HTML. Images that were not cached have their `src` removed, so article rendering never makes an image network request. The file picker, shell, and generic network commands remain unexposed.

## 9. Discovery and organization (Milestone 4)

- `src-tauri/migrations/0004_discovery_organization.sql` adds `feeds.category`, `feeds.trust_level`, the local discovery directory, tags/topics, label assignments, rules, and rule runs.
- `src-tauri/src/discovery/service.rs` validates public feed URLs with the feed engine's existing validator, fetches a preview only on explicit Preview/Save/Subscribe, and copies category/trust to a feed when a directory source is subscribed or its trust changes.
- `src-tauri/src/discovery/store.rs` owns directory, label, rule, and run-history persistence. All reads and writes are keyed by the active profile. Rule JSON is bounded to 20 KB, 1–20 conditions/actions, 200-character text values, and same-profile label/feed references.
- `src-tauri/src/discovery/rules.rs` evaluates rules against at most 5,000 recent local items. Runs execute in one SQLite transaction, apply state and label changes, record matched/changed counts, and write local run history. `rules_run_enabled` runs enabled rules sequentially on explicit request; there is no background rule executor.
- `src-tauri/src/commands/discovery.rs` exposes 23 narrow commands for directory preview/add/subscribe/trust/delete, label CRUD and assignment, rule CRUD, explicit runs, history, and smart-feed lists. No generic search, filesystem, shell, or network command is exposed.
- `src/features/discovery` implements the Discover directory, preview, rule builder, smart-feed views, and run history. `src/features/labels/TopicsView.tsx` implements tags/topics and per-label article lists. `src/application/discoveryService.ts` holds pure filtering, rule parsing, summaries, and formatting helpers.
- Discovery is intentionally manual and local. There is no third-party search or directory provider; the directory is filtered locally by title, URL, or category.

## 10. Search and story intelligence (Milestone 5)

- `src-tauri/migrations/0005_search_story.sql` adds a standalone FTS5 `search_documents` table, `search_presets`, `story_groups`, and `story_group_items`. Six triggers keep the index in sync when items are inserted, updated, or deleted, when extracted article text changes, and when a feed title changes. Existing items are backfilled once.
- `src-tauri/src/search/query.rs` parses a bounded operator language: quoted phrases, negation, `feed:`, `folder:`, `tag:`/`topic:`/`label:`, `author:`, `title:`, `category:`, `trust:`, `before:`, `after:`, `is:`, and `has:`. Invalid operators, malformed dates, and unbalanced quotes are rejected with user-facing errors instead of being passed to SQL.
- `src-tauri/src/search/store.rs` runs parameterized SQL against the active profile. When the query contains searchable text it uses FTS5 `MATCH` with `snippet()` and `bm25()` ranking; filter-only queries use the article join without FTS. Scope filters (all, unread, saved, favorites, archive, queue) are applied in SQL. Presets are validated for length, uniqueness per profile, and scope, and stats report real local counts.
- `src-tauri/src/search/stories.rs` detects duplicate coverage by canonical URL (fragment and tracking parameters removed) or title-token Jaccard similarity of at least 0.72 within a 45-day window. Detection runs only on explicit request, adds ungrouped articles to matching groups, and creates a group for an article with no match. It never deletes or reorders articles. Related articles are scored from shared story groups and shared labels and include human-readable reasons.
- `src-tauri/src/commands/search.rs` exposes 12 narrow commands: search query/stats, preset list/save/delete, story list/get/timeline/detect/delete, and related articles. Every command resolves the active profile server-side and clamps limits.
- `src/features/search/SearchView.tsx` renders Search and Stories tabs in the reading pane. The Search tab provides the query form, scope selector, operator help, saved searches, snippets, and result navigation. The Stories tab provides the timeline, explicit detection, story cards, and grouping removal that keeps articles.
- `src/features/articles/ArticleReader.tsx` shows local related-article connections and can open them without leaving the reader.
- Search is lexical. Semantic/vector search is documented as planned and is not enabled; no model runs and no content leaves the device. Duplicate detection is heuristic and manual.

## 11. Dashboards and analytics (Milestone 6)

- `src-tauri/migrations/0006_dashboards_analytics.sql` adds profile-scoped `dashboards` and `dashboard_widgets` tables with a partial unique index for one default dashboard per profile, a composite cascade from widgets to dashboards, and a bounded widget-kind CHECK constraint.
- `src-tauri/src/dashboards/store.rs` enforces per-profile limits (12 dashboards, 24 widgets per dashboard), normalized unique names, default reassignment on deletion, validated widget titles/ranges/limits/positions, and cross-profile rejection.
- `src-tauri/src/analytics/store.rs` computes reading activity from `article_history`, library composition from stored profile rows, feed health from feed state and `feed_refresh_logs`, storage figures from SQLite pragmas and profile-owned blobs, and briefings from local article, history, highlight, and note rows. No analytics fact tables, seeds, external calls, or model inference are introduced.
- Feed-health errors pass through the existing redaction helper before reaching the UI. Storage figures are read-only; existing explicit cache cleanup remains in Settings → Offline.
- `src-tauri/src/commands/dashboards.rs` and `src-tauri/src/commands/analytics.rs` add 16 narrow commands. Every command resolves the active profile server-side and clamps limits; no generic query, shell, filesystem, or network capability is added.
- `src/features/analytics/StatisticsView.tsx` provides Reading, Dashboards, Library, Feed health, and Storage tabs. `DashboardPanel.tsx` provides the dashboard/widget editor, and `DashboardWidgetCard.tsx` renders each configured widget from fresh local query results.
- `src/features/analytics/BriefingsView.tsx` generates daily and weekly briefings on demand. Highlight rows keep their local item ID so a user can open the underlying article.
- `src/features/settings/StorageSection.tsx` shows real local disk-usage figures; Settings navigation no longer labels Storage as planned.

## 12. Optional AI system (Milestone 7)

- `src-tauri/migrations/0007_ai_system.sql` adds profile-scoped `ai_providers` and `ai_usage_events` tables with closed provider/feature CHECK constraints, bounded text fields, cascade/set-null foreign keys, and indexes. No secret material is stored in either table.
- `src-tauri/src/ai/models.rs`, `providers.rs`, `store.rs`, `service.rs`, and `src-tauri/src/credentials.rs` implement a closed provider abstraction and narrow persistence layer. Provider kinds are `openai`, `anthropic`, `gemini`, `ollama`, and `openai_compatible`; only Ollama is local, and it accepts a literal loopback HTTP endpoint only after an explicit privacy opt-in. Remote endpoints reuse the public-HTTPS validator, DNS pinning, redirect re-validation, timeouts, and byte limits.
- Secrets are scoped by `profile_id::provider_id` in Windows Credential Manager through `keyring` 3.6.3 with `windows-native`. There is no `get_api_key` command; the UI receives only set/remove/has-key status. Deleting a provider deletes its credential, and profile deletion removes credentials for all AI providers in that profile before the database row is removed.
- AI settings live in `profile_settings` under `ai.settings`. Defaults are fully off: AI disabled, no provider selected, all three features disabled, no content categories allowed, local providers disabled, and confirmation enabled. `AiSettingsDto` is validated on read and write.
- Prompts are assembled in Rust from the selected article's local title, feed title, bounded article text, and only the highlight, note, or label data allowed by privacy switches. Label suggestions include only existing local label names and return text; no labels are applied automatically.
- `src-tauri/src/commands/ai.rs` exposes 14 narrow commands for settings, provider CRUD/credential status/connection test, three explicit article features, and usage stats/events/clear. Every command resolves the active profile server-side, clamps list limits, and returns safe user messages. Provider failure details are redacted before storage or display.
- `src/features/ai/AIView.tsx` is a full view that selects a local article, shows readiness blockers, requires confirmation with the destination, and displays the bounded local response. `src/features/settings/AISection.tsx` manages providers, credentials, privacy gates, feature switches, connection tests, and the local usage dashboard.
- No scheduler, background refresh, embedding model, semantic search, tool calling, streaming API, auto-apply behavior, or provider telemetry is added. Token counts are recorded only when a provider reports them.

## 13. Public feed integrations (Milestone 8)

- `src-tauri/migrations/0008_integrations.sql` adds one profile-scoped `integration_sources` table. It stores only a closed kind, source key, display name, optional site URL, resolved/resolved-normalized feed URL, category, trust, status, redactable error text, timestamps, and a nullable `subscribed_feed_id`. There are no credential columns.
- `src-tauri/src/integrations/registry.rs` is a closed platform registry. It resolves YouTube channel IDs, Reddit subreddit names, GitHub `owner/repo` release Atom feeds, Hacker News front page, arXiv categories, public RSS/Atom URLs, Medium feed URLs, Substack subdomains, and Mastodon public account `.rss` URLs. Bluesky and true website change monitoring are listed as unavailable and are rejected before network access.
- `src-tauri/src/integrations/store.rs` owns profile-scoped persistence, duplicate detection, organization state, status transitions, and deletion. `src-tauri/src/integrations/service.rs` resolves adapters, previews feeds, reuses `feeds::service::subscribe` and `feeds::service::refresh`, and updates source status from real refresh outcomes.
- `src-tauri/src/commands/integrations.rs` exposes seven narrow commands: platforms list, preview, add, list, trust update, refresh, and delete. Each command resolves the active profile server-side and returns validated DTOs or safe user messages.
- Integration items live only in normal `feeds` and `feed_items` rows. Search, labels, rules, dashboards, analytics, reader, and OPML need no integration-specific code paths. Removing a source removes only its metadata row and leaves the feed and all library content intact.
- `src/features/settings/IntegrationsSection.tsx` shows real platform availability, performs explicit preview/add/refresh/remove actions, and displays duplicate notices derived from actual preview results. Planned platforms are disabled and labelled.

## 14. Security boundaries

- Webview content is untrusted, including fetched article HTML.
- Tauri CSP restricts scripts, styles, images, and network connections; article images are limited to local `data:` URLs.
- Commands validate all input lengths, formats, and identifiers, including camelCase article state patches mapped to Rust snake_case fields.
- No raw stack traces are returned to the UI.
- No shell, unrestricted file, or dynamic SQL command is exposed.
- The foundation schema contains dormant `is_locked` and `lock_hint` profile columns. They do not implement a lock, password, or encryption feature and are shown as unavailable in both Security and Profiles settings. No command sets them.

## 15. Backup, desktop integration, and final polish (Milestone 9)

Implemented in the working tree and locally verified; sign-off is pending. See `DEVELOPMENT.md` and `docs/RELEASE-CHECKLIST.md` for evidence and the remaining interactive checks.

- `src-tauri/src/backup/service.rs` writes an app-managed SQLite snapshot plus a JSON manifest containing app version, schema version, profile list, creation time, and SHA-256 hash. Backup size is recorded in the app-managed `index.json`, not the manifest. The directory contains `index.json`, `<uuid>.sqlite3`, and `<uuid>.manifest.json` files. Restore verifies manifest/schema/profile compatibility, SQLite `integrity_check`, and checksum before taking a verified safety backup and replacing the active database. Older restored schemas are migrated; newer schemas are rejected. Manual backups persist until deleted; only the five newest automatic backups (including safety and tray backups) are pruned, and backup identifiers are validated before any deletion path is built.
- `src-tauri/src/commands/backups.rs` exposes five narrow commands: create, list, verify, delete, and restore. There is no user-chosen export destination or per-profile export command. Profile deletion creates a verified safety backup before removing a profile. `src-tauri/src/privacy/mod.rs` exposes one read-only dashboard command over actual local counts and storage paths; posture flags are build-time assertions covered by tests.
- `src-tauri/src/desktop/notifications.rs` persists local notification preferences in `app_settings`, derives support from the plugin permission-state result, evaluates real permission state through `tauri-plugin-notification`, supports quiet hours, and sends notifications only for tray-triggered feed refresh, tray-triggered backup, and the explicit test command. The 60-second scheduler, in-app refresh, and in-app backup do not notify. `desktop.rs` exposes status/update/test/tray/startup/updates/security commands. No plugin command is granted to the webview.
- `src-tauri/src/desktop/tray.rs` installs a Windows tray icon with Show, Refresh due feeds, Create local backup, and Quit actions. The backup is labelled “Tray backup” and is automatic for retention purposes. Close behavior is a local setting: quit or hide-to-tray. On unsupported platforms the tray is not created and the setting is labelled unavailable.
- `src-tauri/src/desktop/startup.rs` uses `tauri-plugin-autostart` for an explicit launch-on-sign-in toggle, with honest unavailability on non-Windows builds.
- `src-tauri/src/desktop/updates.rs` reports signed-update configuration from compile-time `BERGAG_UPDATE_ENDPOINT` and `BERGAG_UPDATE_PUBLIC_KEY` values. Checks use HTTPS and minisign verification only when both are valid; the endpoint host is reported only when fully configured. `tauri.conf.json` supplies an empty updater configuration so unconfigured local builds start cleanly. Installation remains disabled and no progress is simulated when unconfigured.
- `src-tauri/src/security/status.rs` reports the compiled security posture: dormant profile-lock fields, no credential getter command, no secret-like SQLite columns, validated URL/sanitization/redaction paths, and no generic shell/filesystem command. Command-surface and redaction flags are build-time assertions covered by tests, not live capability scans.
- `src/features/settings` renders real Backups, Privacy, Notifications, Tray, Startup, Updates, Security, and Profiles sections. Only Advanced remains planned. Honest unavailable states replace fake controls; profile lock is never presented as encryption. Planned navigation entries remain focusable and announce their status.
- Performance work adds three indexes (migration 9), scopes profile-switch and feed-refresh cache invalidation to affected profiles, bounds feed lists to 500 entries and stored refresh items to 200 per feed, reduces refresh mutex contention by locking per result, and fixes the settings Search stats cache key. Explicit bounds include 20-feed refresh batches with concurrency 3, a 60-second scheduler, 12 dashboards, 24 widgets, eight cached images, 5 MB extraction HTML, 2 MB per image, 5,000-item rule evaluation, a 45-day duplicate window, and 500-character saved searches.
- Accessibility adds a keyboard skip link to `#bergag-workspace`, reduced-motion handling, stable button names with `aria-busy`, live status regions, labelled switches, radiogroup semantics, role-alert errors, focusable planned entries, modal shortcut suppression, and resize-handle value text.
