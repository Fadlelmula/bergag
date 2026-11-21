# Changelog

All notable changes to Bergag will be documented here.

> Sign-off: Milestone 9 was approved by the user on 2026-11-21 after local verification. It was the final planned milestone. With authorization, the MIT license was added and a baseline commit and annotated tag `v0.1.0-m9` were created. Installer signing was deferred by user decision; the source and tag were pushed to https://github.com/Fadlelmula/bergag without releasing installer binaries.

## [Unreleased] — 2026-11-21 — Milestone 9: Backup, security, final polish

### Added

- Migration `0009_performance_indexes` with profile-scoped due-feed, sorted-item, and case-insensitive feed-title indexes; feed lists and parsed refresh items are bounded, refresh no longer holds the SQLite mutex across the whole result loop, and profile-switch/feed-refresh cache invalidation is scoped to affected profiles.
- App-managed SQLite backups with a JSON manifest containing app version, schema version, profile list, creation time, and SHA-256 hash. Size is stored in the app-managed index, not the manifest. Creation, listing, verification, deletion, and restore are exposed through five narrow commands.
- Restore that verifies checksum, manifest, schema compatibility, profile list, and SQLite integrity before replacing the library; it creates a verified safety backup first, rejects newer schemas and corruption, migrates older restored schemas, and validates backup IDs before retention deletion.
- Profile deletion safety backup and retention that prunes only the five newest automatic backups in the app-managed directory.
- Real privacy, security, notification, tray, startup, update, and profile status surfaces; profile lock remains unavailable and posture flags are labeled as build-time assertions where they are not live scans.
- Windows tray actions for Show, Refresh due feeds, Create local backup, and Quit; real close behavior and launch-on-sign-in settings; local notification preferences with permission/support-aware controls. Notifications currently cover tray refresh, tray backup, and the explicit test only.
- Update status and signed-check framework using compile-time HTTPS endpoint and minisign public key values, with empty local configuration so startup does not fail. Installation remains disabled and no fake progress is shown.
- Settings sections for Backups, Privacy, Notifications, Tray, Startup, Updates, Security, and Profiles. Advanced remains planned.
- Accessibility fixes: descriptive backup action names, live status regions, radiogroup/group semantics, an operable startup switch, honest active-profile errors, focusable planned settings entries, modal shortcut suppression, higher-contrast skip link, resize-handle value text, and reduced-motion handling.
- Release checklist, single gate commands, artifact exclusions, update build variables, and explicit unsigned/signing/license/publishing decisions.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (78 tests), and `npm run build` passed.
- Rust tests passed with 104 normal tests; two ignored runtime-verification tests were run separately with `BERGAG_RUNTIME_DB` and passed backup create/verify/delete and restore against a copy of the installed database.
- Clippy showed no warnings in M9 modules; 20 library warnings and 25 test-target warnings remain in pre-M9 modules.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app opened at schema 9, recorded all nine migrations once, created all three performance indexes, returned `PRAGMA integrity_check = ok`, contained one default profile and zero feeds, applied zero migrations on relaunch, and logged no error- or warning-level entries.
- Runtime restore verification confirmed a verified safety backup, removal of a marker profile, a valid source backup, a valid restored integrity check, and an unchanged source database hash.

### Notes

- Backups are app-managed snapshots rather than arbitrary exports; manual backups persist until deleted, and only the five newest automatic backups are pruned.
- Update installation, profile lock/encryption, Bluesky, true website change monitoring, semantic search, and Advanced settings remain unavailable or planned and are labeled as such.
- The MIT license, baseline commit, and annotated tag were created with explicit permission. Installer signing was deferred by user decision; the source and tag were pushed to https://github.com/Fadlelmula/bergag without releasing installer binaries.

## [Unreleased] — 2026-11-20 — Milestone 8: Public feed integrations

### Added

- Migration `0008_integrations` with profile-scoped, metadata-only integration sources, closed kind/status/trust checks, bounded fields, `ON DELETE SET NULL` feed links, uniqueness constraints, and indexes.
- Closed platform registry and deterministic adapters for YouTube channels, Reddit subreddits, GitHub releases, Hacker News front page, arXiv categories, public podcast/newsletter/website feeds, Medium feeds, Substack publications, and Mastodon public account feeds.
- Visible planned entries for Bluesky and true website change monitoring; both reject input before any request and are disabled in the interface.
- Real preview, add-and-subscribe, list, trust update, manual refresh, and remove flows through seven narrow `integrations_*` commands with server-side profile resolution and redacted refresh errors.
- Reuse of the existing feed subscription, scheduler, reader, search, labels, rules, dashboards, and OPML paths; integration sources store metadata only and reference ordinary `feeds.id` rows.
- Settings → Integrations section with real availability, preview results, duplicate notices from actual previews, connected-source list, trust changes, explicit refresh, and removal wording that preserves the feed and saved articles.
- Rust tests for adapters, invalid/private/planned inputs, uniqueness and profile isolation, source state updates/removal, schema constraints and indexes, schema 7→8 upgrade, and zero seeded content.
- Frontend tests for availability, preview rendering, duplicate notices, mutation invalidation, and destructive-action wording.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (54 tests), and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 85 tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets` showed no new Milestone 8 warnings; 20 pre-existing warnings remain.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app upgraded the runtime database from schema version 7 to 8, recorded `integrations` once, created `integration_sources` and its three indexes, returned `PRAGMA integrity_check = ok`, and contained zero seeded integration sources or library content. A second launch applied 0 migrations with no error- or warning-level log entries.

### Notes

- Integrations never require an account or credential in this milestone. Removing a source removes only profile-scoped integration metadata; the feed and library content remain.
- Authenticated integrations, Bluesky, and true website change monitoring are not implemented and are not presented as available.
- Sign-off recorded on 2026-11-20 after review; Milestone 9 — Backup, security, final polish was authorized to start.

## [Unreleased] — 2026-11-20 — Milestone 7: Optional AI system

### Added

- Migration `0007_ai_system` with profile-scoped AI provider configuration and usage events; credentials are never stored in SQLite.
- Provider abstraction for OpenAI, Anthropic, Gemini, Ollama, and OpenAI-compatible endpoints with public HTTPS validation, DNS pinning, redirect re-validation, bounded requests/responses, and provider-specific response parsing.
- Windows Credential Manager storage via `keyring` 3.6.3, with profile/provider-scoped keys, no API-key getter command, and credential cleanup on provider or profile deletion.
- AI settings with master switch, per-feature opt-ins, content-category privacy gates, loopback-provider opt-in, and per-request confirmation default.
- Explicit article summarization, label suggestions, and article Q&A built only from selected local SQLite content; label suggestions do not auto-apply labels.
- AI full view with article selection, readiness blockers, confirmed destination, bounded plain-text results, and local usage summary.
- AI settings section with provider CRUD, credential status, connection tests, privacy switches, and an honest usage dashboard.
- 14 narrow `ai_*` commands with server-side active-profile resolution, redacted usage errors, and no generic shell, filesystem, or network capability.
- Rust tests for provider validation, response parsing, credential scoping, redaction, privacy-scoped prompts, usage isolation, and migration upgrades.
- Frontend tests for AI setting defaults, readiness gates, and enabled navigation.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (45 tests), and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 75 tests.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app upgraded the runtime database from schema version 6 to 7, recorded `ai_system` once, created both AI tables and their indexes, returned `PRAGMA integrity_check = ok`, and contained zero seeded providers, usage events, dashboards, widgets, or library content. A second launch applied 0 migrations with no error- or warning-level log entries.

### Notes

- AI never runs in the background and no article content is sent without explicit opt-in plus confirmation.
- Token counts appear only when a provider reports them; the dashboard never invents usage, costs, or model activity.
- Signed off by the user on 2026-11-20 after review; Milestone 8 was authorized to start.

## [Unreleased] — 2026-11-20 — Milestone 6: Dashboards and analytics

### Added

- Migration `0006_dashboards_analytics` with profile-scoped dashboards, one default per profile, bounded widget kinds, validated ranges, indexes, and cascade deletion of widget settings.
- Dashboard editor with create, rename, description, delete, default selection, widget add/edit/remove, ordering, and per-profile isolation.
- Eight real widget types: unread by feed, recent articles, reading activity, top labels, feed health, local storage, briefing, and reading queue.
- Statistics full view with reading activity, library composition, feed health, storage totals, and dashboard tabs; all figures are computed on demand from local SQLite data.
- Daily and weekly briefings generated on demand from stored discovery, read history, unread state, highlights, and notes, with highlight rows that open the underlying article.
- Storage settings section with real database and profile-cache figures; no fake retention or cleanup controls.
- 16 narrow dashboard/analytics commands with server-side active-profile resolution and clamped limits; no generic query, shell, filesystem, or network capability was added.
- Rust tests for dashboard/widget validation and profile isolation, analytics counts, empty windows, and invalid inputs.
- Frontend tests for widget kinds, ordering, validation, briefing period mapping, chart scaling, count formatting, and reading-activity detection.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (39 tests), and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 61 tests.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app upgraded the runtime database from schema version 5 to 6, recorded `dashboards_analytics` once, created both new tables and their indexes, kept zero seeded dashboards, widgets, or library content, and returned `PRAGMA integrity_check = ok`; a second launch applied 0 migrations with no error- or warning-level log entries.

### Notes

- Briefings have no scheduler, delivery, notification, or cloud service; they are generated only when opened.
- Analytics include no demo, inferred, or remote metrics. Storage figures are read-only, and cleanup remains an explicit action in Settings → Offline.

## [Unreleased] — 2026-11-20 — Milestone 5: Search and story intelligence

### Added

- Migration `0005_search_story` with a profile-scoped FTS5 `search_documents` table, six sync triggers, backfill, saved searches, story groups, story membership, indexes, and profile-scoped foreign keys.
- Documented search operator parser for quoted phrases, negation, feed/folder/label/author/title/category/trust/date/state/annotation filters, with rejection of malformed operators and quotes.
- Scoped local search across all, unread, read later, favorites, archive, and reading queue, using FTS5 snippets and bm25 ranking with a filter-only fallback.
- Saved searches with validated names, queries, scopes, per-profile uniqueness, and deletion.
- Explicit duplicate detection using canonical URLs and title-token similarity within a 45-day window, story groups, and a story timeline.
- Related articles derived from shared story groups and labels, with connection reasons shown in the reader.
- Search & Stories full view with Search/Stories tabs, operator help, presets, result rows, duplicate detection, timeline buckets, and story removal that keeps articles.
- Enabled Search navigation and a Search settings summary showing real indexed-document, preset, story, and grouped-article counts.
- 12 narrow `search_*`, `stories_*`, and `articles_related` Tauri commands; no filesystem, shell, generic network, or semantic-model capability was added.
- Rust tests for operator parsing, filter-only search, FTS profile isolation, presets, stats, duplicate grouping, idempotent detection, story isolation, and related-article connections.
- Frontend tests for supported scopes, operator documentation, and story bucket formatting.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (30 tests), and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 57 tests.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app upgraded the runtime database from schema version 4 to 5, recorded `search_story` once, created the FTS table and six sync triggers, kept zero seeded search documents, presets, story groups, or library content, and returned `PRAGMA integrity_check = ok`; a second launch applied 0 migrations with no error- or warning-level log entries.

### Notes

- Search is lexical FTS5 over locally stored content. Semantic search remains planned and is not enabled.
- Duplicate detection is heuristic, runs only on explicit request, and never deletes or reorders articles.
- Removing a story group deletes only the grouping; underlying articles remain in the library.

## [Unreleased] — 2026-11-20 — Milestone 4: Discovery and organization

### Added

- Migration `0004_discovery_organization` with feed category/trust, a local discovery directory, tags/topics, label assignments, rules, rule runs, indexes, and profile-scoped composite foreign keys.
- Private local discovery directory with manual feed-URL addition, preview, local title/URL/category filtering, categories, trust levels, and subscription into a feed folder.
- Feed category and trust-level editing, synced from directory sources on subscribe and trust changes.
- Tags and topics with optional color/description, usage counts, article assignment in the reader, and per-label article lists.
- Visual rule builder with all/any matching, text/feed/category/trust/label/state conditions, state and label actions, and a live plain-language summary.
- Explicit rule execution for one rule or all enabled rules, smart-feed views, and local run history with matched/changed counts.
- Discover and Topics views, Discovery and Rules settings summaries, enabled navigation, and 23 narrow `discovery_*`, `labels_*`, and `rules_*` Tauri commands.
- Rust tests for discovery-source isolation and upsert, label isolation and assignment counts, rule JSON reference validation, rule execution, state changes, label actions, history, and cross-profile denial.
- Frontend tests for directory filtering/categories, trust labels, rule parsing, summaries, and condition formatting.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (27 tests), and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 48 tests.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app upgraded the runtime database to schema version 4, applied each migration once, kept zero seeded discovery sources, labels, rules, or rule runs, and returned `PRAGMA integrity_check = ok`; a second launch applied 0 migrations with no error-level log entries.

### Notes

- Discovery is a user-managed local directory of feed URLs. No third-party search or directory provider is used.
- Rules run only on explicit request. Background rule execution is not implemented.

## [Unreleased] — 2026-11-20 — Milestone 3: Reader and article system

### Added

- Migration `0003_article_system` with article sources/states/history, notes, highlights, collections, collection membership, cached images, indexes, and backfill from stored feed items.
- Ammonia allowlist sanitization for feed and extracted HTML, with clean-content removal, safe relative links, hardened `rel`, and no inline event handlers.
- Explicit full-page extraction using the public URL validator, bounded response sizes, article-body selection, common raster image sniffing, and at most eight locally cached images per article.
- Article list modes for all, unread, read later, favorites, archive, queue, history, highlights, and collections, with list/magazine/minimal presentation and bulk state actions.
- Reader toolbar for read/save/favorite/archive/queue, original link, focus mode, private notes, selected-text highlights, collection membership, extraction, and local cache information.
- History and highlights overview panels, Reading and Offline settings, corrected privacy/shortcuts copy, and J/K/Enter/M/S/V/F shortcuts.
- 20 narrow `articles_*` Tauri commands with server-side active-profile resolution, validated patches, and no new filesystem, shell, or generic network capability.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (23 tests), and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 42 tests, including sanitization, extraction, list modes, annotations, collection cascades, profile isolation, and cache clearing.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app upgraded the runtime database to schema version 3, applied each migration once, kept zero seeded articles, and returned `PRAGMA integrity_check = ok`; a second launch applied 0 migrations with no error-level log entries.

## [Unreleased] — 2026-11-20 — Milestone 2: Feed engine

### Added

- Migration `0002_feed_engine` with profile-scoped folders, feeds, discovered items, and refresh logs; deleting a folder keeps its feeds.
- RSS, Atom, and JSON feed parsing, text cleanup, HTML feed autodiscovery, and preview before subscribing.
- Validated public HTTP/HTTPS fetching with private/loopback/link-local target rejection, redirect revalidation, DNS pinning, timeouts, and response limits.
- Subscription CRUD, folders, duplicate URL detection, pause/resume, edit, manual refresh, and per-feed health with exponential backoff.
- ETag/Last-Modified conditional requests, `304 Not Modified` handling, refresh history, and a 60-second scheduler that emits `feeds:refreshed`.
- OPML import with validation and duplicate skipping, plus OPML export with a privacy warning.
- All Feeds UI with feed list, folder grouping, context menus, confirmation dialogs, feed detail, stored items, refresh logs, and settings dialogs.
- 15 narrow `feeds_*` Tauri commands; no filesystem, shell, or generic network capability was exposed to the frontend.

### Verification

- `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm run test` (16 tests), and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 35 tests.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built app upgraded the runtime database to schema version 2, applied each migration once, kept zero seeded feeds, and returned `PRAGMA integrity_check = ok`; a second launch applied 0 migrations.

## [0.1.0] — 2026-11-20 — Milestone 1: Foundation

### Added

- Initial product documentation, architecture, roadmap, security policy, development guide, and contribution guide.
- Tauri 2 Windows desktop shell with a narrow, explicitly enumerated command surface and deny-by-default capabilities.
- React 19, TypeScript, Vite, Tailwind CSS v4, TanStack Query, Radix primitives, and Lucide icons.
- SQLite persistence through bundled `rusqlite`, WAL, foreign keys, append-only SQL migrations, integrity checks, and FTS5 readiness detection.
- Profile create, rename, switch, and delete with active-profile isolation and guarded last-profile deletion.
- Per-profile appearance settings, layout defaults, normalization, persistence, and validation.
- Accessible application shell with sidebar navigation, honest planned-feature labels, context-menu reordering/hiding, collapse, resizable panels, status bar, settings dialog, and shortcuts dialog.
- Redacted rotating file logs and local diagnostics with exact app/Tauri versions, database state, OS, architecture, and recent redacted logs.
- Generated spider-lily application icon plus Windows icon set.
- Unit, component, migration, profile-isolation, validation, and redaction tests.

### Verification

- `npm run lint`, `npm run typecheck`, `npm run format:check`, `npm run test`, and `npm run build` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` passed with 20 tests.
- `npm run tauri build` produced unsigned MSI and NSIS x64 bundles.
- The built Windows executable launched successfully; the database initialized with schema version 1 and `PRAGMA integrity_check` returned `ok`.
