# Bergag roadmap

Legend: ✅ complete, 🚧 in progress, ⬜ planned, ⛔ blocked, 🕓 deferred.

Milestones are gates. Work does not proceed to the next milestone until the current one is reported and explicitly approved.

## Milestone 1 — Foundation ✅ (reported; sign-off recorded before Milestone 2)

- [x] Inspect repository and environment.
- [x] Create initial documentation and plan.
- [x] Tauri 2 desktop shell.
- [x] React/TypeScript/Vite/Tailwind architecture.
- [x] SQLite migrations and integrity checks.
- [x] Profile creation, rename, switch, delete, and isolation tests.
- [x] Core design system with dark/light themes and accent colors.
- [x] Sidebar with reordering, hiding, collapse, and reset.
- [x] Resizable/hideable multi-panel layout with persistence.
- [x] Settings shell with working foundation settings and honest planned labels.
- [x] Narrow validated Tauri command surface.
- [x] Redacted logging and diagnostics.
- [x] Basic unit, component, migration, and profile-isolation tests.
- [x] Build and launch verification on Windows.

Verified on 2026-11-20: frontend lint/typecheck/format/tests/build passed; 20 Rust tests passed; MSI and NSIS bundles built; Windows executable launched; local database initialized with schema version 1 and `integrity_check = ok`.

## Milestone 2 — Feed engine ✅ (reported; sign-off recorded before Milestone 3)

- [x] Append-only migration `0002_feed_engine.sql`.
- [x] RSS/Atom/JSON parsing, text cleanup, and HTML autodiscovery.
- [x] Public URL validation, SSRF/private-network guards, bounded fetch, and safe redirects.
- [x] Subscriptions, folders, profile isolation, and duplicate URL detection.
- [x] Preview, discover, subscribe, edit, pause, delete, and manual refresh commands.
- [x] ETag/Last-Modified, 304 handling, exponential backoff, concurrency limit, and 60-second scheduler with `feeds:refreshed`.
- [x] Stored items, refresh history, health state, and feed detail UI.
- [x] OPML import/export with validation and duplicate skipping.
- [x] Frontend feed UI: add/preview/discover, folders, refresh, settings, deletion, logs, items, and OPML dialogs.
- [x] Unit/component/integration tests, full quality gates, local build, and runtime migration verification.

Verified on 2026-11-20: frontend format/lint/typecheck/16 tests/build passed; 35 Rust tests passed; MSI and NSIS bundles built; the built app upgraded the local database from schema version 1 to 2, recorded each migration once, reported `integrity_check = ok`, and contained no seeded feed data. A second launch applied 0 migrations.

## Milestone 3 — Reader and article system ✅ (reported; sign-off recorded before Milestone 4)

- [x] Append-only migration `0003_article_system.sql` with article content/state/history, notes, highlights, collections, collection membership, cached images, indexes, and backfill from existing feed items.
- [x] Ammonia allowlist sanitization with clean-content removal, relative URL rewriting, link rel hardening, and no event-handler attributes.
- [x] Explicit full-text extraction with article-body selection, public URL validation, bounded fetches, raster image sniffing, and local image caching.
- [x] Article list modes for all, unread, read later, favorites, archive, queue, history, highlights, and collections.
- [x] List, magazine, and minimal presentation modes plus bulk read/save/favorite/queue/archive actions.
- [x] Reader with read/save/favorite/archive/queue controls, original link, focus mode, notes, highlights, and collection membership.
- [x] Local history and highlights overview panels with article navigation.
- [x] Reading and Offline settings, corrected privacy/shortcuts copy, and keyboard shortcuts J/K/Enter/M/S/V/F.
- [x] Unit/component tests, sanitization and extraction tests, profile-isolation tests, full quality gates, local build, and runtime migration verification.

Verified on 2026-11-20: frontend format/lint/typecheck/23 tests/build passed; 42 Rust tests passed; MSI and NSIS bundles built; the built app upgraded the runtime database from schema version 2 to 3, recorded `foundation`, `feed_engine`, and `article_system` each once, reported `integrity_check = ok`, and contained zero seeded articles. A second launch applied 0 migrations, and verification logs contained no error entries.

## Milestone 4 — Discovery and organization ✅ (approved)

- [x] Append-only migration `0004_discovery_organization.sql` with feed category/trust, local discovery sources, tags/topics, label assignments, rules, rule runs, indexes, and profile-scoped foreign keys.
- [x] Private local discovery directory: manual feed-URL addition, preview, local search/filter, categories, trust levels, and subscription into a folder.
- [x] Feed category and trust-level editing, synced from the directory when a source is subscribed or its trust changes.
- [x] Tags and topics with color/description, usage counts, article assignment, and per-label article lists.
- [x] Visual rule builder with all/any conditions, text/feed/category/trust/label/state conditions, state and label actions, and a live plain-language summary.
- [x] Explicit rule execution for one rule or all enabled rules, smart-feed views, and local run history with matched/changed counts.
- [x] Discover and Topics views, reader label assignment, Discovery and Rules settings summaries, and enabled navigation.
- [x] Rust tests for discovery-source isolation and upsert, label isolation and assignment counts, rule JSON reference validation, rule execution, state changes, label actions, history, and cross-profile denial.
- [x] Frontend tests for directory filtering/categories, trust labels, rule parsing, summaries, and condition formatting.
- [x] Full quality gates, local build, and runtime migration verification.

Verified on 2026-11-20: frontend format/lint/typecheck/27 tests/build passed; 48 Rust tests passed; MSI and NSIS bundles built; the built app upgraded the runtime database from schema version 3 to 4, recorded each migration once, reported `integrity_check = ok`, and contained zero seeded discovery sources, labels, rules, or rule runs. A second launch applied 0 migrations.

Milestone 4 does not add a third-party search or directory provider. Discovery is a user-managed local directory of feed URLs. Rules are manual; background rule execution is not implemented.

## Milestone 5 — Search and story intelligence ✅ (reported; sign-off recorded before Milestone 6)

- [x] Append-only migration `0005_search_story.sql` with a profile-scoped FTS5 `search_documents` table, six sync triggers, backfill, saved searches, story groups, and story membership.
- [x] Documented search operator parser: quoted phrases, negation, feed/folder/label/author/title/category/trust/date/state/annotation filters, and invalid-operator rejection.
- [x] Scoped search over all, unread, read later, favorites, archive, and reading queue, with FTS snippets and bm25 ranking plus a filter-only fallback.
- [x] Saved searches with validated names, queries, scopes, uniqueness, and deletion.
- [x] Explicit duplicate detection by canonical URL and title-token similarity within a 45-day window, story groups, and a story timeline.
- [x] Related articles derived from shared story groups and labels, with human-readable connection reasons.
- [x] Search & Stories full view with Search/Stories tabs, operator help, presets, result rows, duplicate detection, timeline buckets, and story removal that keeps articles.
- [x] Reader related-article section, enabled Search navigation, and a Search settings summary with real local counts.
- [x] Rust tests for operator parsing, filter-only searches, FTS profile isolation, presets, stats, duplicate grouping, idempotent detection, story isolation, and related-article connections.
- [x] Frontend tests for scopes, operator documentation, and story bucket formatting.
- [x] Full quality gates, local build, and runtime migration verification.

Verified on 2026-11-20: frontend format/lint/typecheck/30 tests/build passed; 57 Rust tests passed; MSI and NSIS bundles built; the built app upgraded the runtime database from schema version 4 to 5, recorded `search_story` once, created the FTS table and six sync triggers, reported `integrity_check = ok`, and contained zero seeded search documents, presets, story groups, or library content. A second launch applied 0 migrations and logs contained no error or warning entries.

Semantic search is documented as planned, not implemented. Duplicate detection runs only on explicit request, is heuristic, and never deletes articles. Story removal deletes only the grouping, not the underlying articles.

## Milestone 6 — Dashboards and analytics ✅ (reported; sign-off recorded before Milestone 7)

- [x] Append-only migration `0006_dashboards_analytics.sql` with profile-scoped dashboards and bounded widget configuration.
- [x] User-authored dashboard creation, rename, description, deletion, default selection, ordering, and per-profile isolation.
- [x] Widget editor with validated title, day range, item limit, and position; no widget executes code or fetches remote data.
- [x] Eight real widget types: unread by feed, recent articles, reading activity, top labels, feed health, local storage, briefing, and reading queue.
- [x] Private Statistics full view with reading activity, library composition, feed health, storage totals, and dashboard tabs.
- [x] Daily and weekly briefings generated on demand from local article discovery, read history, unread state, highlights, and notes.
- [x] Storage settings summary showing real local database and profile-owned cache figures; no retention or deletion controls were faked.
- [x] Enabled Briefings and Statistics navigation; M6 no longer appears as planned.
- [x] Rust tests for dashboard/widget validation, profile isolation, cascades, analytics counts, empty windows, and input validation.
- [x] Frontend tests for widget kinds, ordering, validation, briefing period mapping, chart scaling, count formatting, and reading-activity detection.
- [x] Full quality gates, local build, and runtime migration verification.

Verified on 2026-11-20: frontend format/lint/typecheck/39 tests/build passed; 61 Rust tests passed; MSI and NSIS bundles built; the built app upgraded the runtime database from schema version 5 to 6, recorded `dashboards_analytics` once, created both new tables and their indexes, reported `integrity_check = ok`, and contained zero seeded dashboards, widgets, or library content. A second launch applied 0 migrations and logs contained no error or warning entries.

Briefings are generated on demand; there is no scheduling, delivery, notification, or cloud service. Analytics are computed from existing local tables and never include inferred, demo, or remote metrics.

## Milestone 7 — AI system ✅ (approved)

- [x] Append-only migration `0007_ai_system.sql` with profile-scoped provider configuration and local usage events only; no credentials in SQLite.
- [x] Provider abstraction for OpenAI, Anthropic, Gemini, Ollama, and OpenAI-compatible endpoints with bounded request/response limits and provider-specific request/response mapping.
- [x] Public HTTPS validation, DNS pinning, redirect re-validation, and a narrow loopback-only Ollama exception gated by an explicit local-provider opt-in.
- [x] Windows Credential Manager storage via `keyring` 3.6.3 with profile/provider-scoped keys; no API-key getter command and no secrets in DTOs, settings, SQLite, logs, or backups.
- [x] AI settings with master switch, per-feature opt-ins, content-category privacy gates, local-provider opt-in, and per-request confirmation default.
- [x] Explicit article summarization, label suggestions, and article Q&A; prompts are built from selected local SQLite content only.
- [x] AI privacy dashboard and local usage history with real request, failure, character, and provider-reported token counts; no seeded activity.
- [x] Narrow `ai_*` command surface with server-side active-profile resolution, validation, redaction, and no generic shell/filesystem/network capability.
- [x] Rust tests for provider config validation, response parsing, credential key scoping, redacted usage errors, privacy-scoped prompts, profile isolation, usage stats, and migration upgrades.
- [x] Frontend tests for AI setting defaults, readiness gates, and enabled navigation.
- [x] Full quality gates, local build, and runtime migration verification.

Verified on 2026-11-20: frontend format/lint/typecheck/45 tests/build passed; 75 Rust tests passed; MSI and NSIS bundles built; the built app upgraded the runtime database from schema version 6 to 7, recorded `ai_system` once, created both AI tables and their indexes, returned `PRAGMA integrity_check = ok`, and contained zero seeded providers, usage events, dashboards, widgets, or library content. A second launch applied 0 migrations and logs contained no error or warning entries.

AI never runs in the background. Article text, highlights, notes, and label names are included only when the relevant privacy switches are enabled, and each action is confirmed before a request is sent. Deleting a provider removes its credential; deleting a profile removes its AI provider credentials as part of the same validated operation.

Sign-off recorded on 2026-11-20 after review; Milestone 8 — Integrations was authorized to start.

## Milestone 8 — Integrations ✅ (approved)

Public, user-added, feed-backed integrations only. No credentials, accounts, authenticated APIs, or third-party directory services. Bluesky and true website change monitoring remain planned and disabled.

- [x] Append-only migration `0008_integrations.sql` with profile-scoped `integration_sources`, closed kind/status/trust CHECK constraints, bounded metadata-only columns, `ON DELETE SET NULL` feed links, unique keys, and indexes; no secrets in SQLite.
- [x] Closed platform registry for YouTube, Reddit, GitHub releases, Hacker News, arXiv, podcasts, Medium, Substack, Mastodon public posts, public-newsletter RSS, website feeds, plus disabled planned entries for Bluesky and website monitoring.
- [x] Deterministic adapters that resolve public identifiers and URLs to exact feed URLs, validate input formats and lengths, reject private or oversized URLs, and refuse planned adapters before any network access.
- [x] Real preview, add-and-subscribe, list, trust update, manual refresh, and remove flows that reuse the existing feed/article pipeline and preserve the feed plus library content when a source is removed.
- [x] Narrow `integrations_*` command surface with server-side active-profile resolution, validation, redaction of refresh errors, and no generic shell/filesystem/network capability.
- [x] Settings → Integrations section with honest availability, real form state, duplicate notices from actual previews, connected-source list, trust changes, refresh, and explicit removal confirmation wording.
- [x] Rust tests for adapter normalization, invalid/private/planned inputs, profile isolation, duplicates, source state updates/removal, constraint enforcement, index creation, schema 7→8 upgrade, and zero seeded content.
- [x] Frontend tests for platform availability, preview rendering, duplicate notices, mutation invalidation, and destructive-action wording.
- [x] Full quality gates, local build, runtime migration verification, and unsigned Windows bundles.

Verified on 2026-11-20: frontend format/lint/typecheck/54 tests/build passed; 85 Rust tests passed; clippy showed no new Milestone 8 warnings (20 pre-existing warnings remain); `npm run tauri build` produced unsigned MSI and NSIS x64 bundles; the built app upgraded the runtime database from schema version 7 to 8, recorded `integrations` once, created `integration_sources` and all three indexes, returned `PRAGMA integrity_check = ok`, and contained zero seeded integration sources or library content. A second launch applied 0 migrations and logs contained no error or warning entries.

Integration sources store only metadata and a link to a normal `feeds.id`. Removing a source never removes its feed, articles, notes, highlights, labels, or reading history; it only removes the profile's integration metadata. Credential-backed integrations are deliberately out of scope and would require the OS credential store with no getter command if added later.

Sign-off recorded on 2026-11-20 after review; Milestone 9 — Backup, security, final polish was authorized to start.

## Milestone 9 — Backup, security, final polish ✅ (approved and signed off)

Complete the local-first release surface without adding accounts, telemetry, hidden automation, or fake controls.

- [x] Verified app-managed local backups with a manifest containing app version, schema version, profile list, creation time, and integrity hash; backup size is recorded in the app-managed index, and no credentials or live logs are included. There is no user-chosen export destination command.
- [x] Explicit restore confirmation before replacing the active library, with schema compatibility checks, corruption refusal, a verified safety backup, and no silent overwrite.
- [x] Migration safety checks for backup/restore, including newer-schema rejection, older-schema migration after restore, integrity checks, and retention of the active database.
- [x] Retention rules that prune only the five newest automatic backups in the app-managed directory, validated to prevent identifier-based path traversal; manual backups and the active database are never pruned.
- [x] Privacy dashboard that reports real local counts, paths, network surfaces, AI status, integration scope, and activity; posture flags are identified as build-time assertions covered by tests.
- [x] Honest profile lock behavior: dormant `is_locked`/`lock_hint` fields are shown as unavailable in both Security and Profiles, and no lock, unlock, or encryption command exists.
- [x] Real notification controls for tray refresh, tray backup, and the explicit test; OS permission and support state are reported, with denial, quiet hours, and unavailable platforms handled without broken controls.
- [x] System tray controls for Show, Refresh due feeds, Create local backup, and Quit, with a real local close behavior setting.
- [x] Startup behavior setting with explicit Windows sign-in control and honest unavailability on unsupported platforms.
- [x] Auto-update framework that reports only real checks and requires HTTPS plus minisign verification; installation remains disabled and no fake progress or unverified installer is shown.
- [x] Installer and release documentation covering unsigned local builds, signing decision, license decision, no publishing without permission, rollback via verified backups, and artifact exclusions.
- [x] Performance pass with migration 9 indexes, scoped profile/feed invalidation, bounded feed lists and parsed items, reduced refresh mutex contention, and a corrected Search stats cache key; explicit bounds are documented rather than presenting unbounded scaling.
- [x] Accessibility pass for keyboard reachability, focus, labels, roles, live status, error messages, planned-entry focusability, skip link, reduced motion, and modal shortcut handling.
- [x] Complete architecture, security, development, and release-checklist documentation, including artifact exclusions and the single gate command sequence.
- [x] Full quality gates, runtime migration/backup verification, and local unsigned Windows bundles.

Verified on 2026-11-21: frontend format check, lint, typecheck, 78 Vitest tests, and production build passed; Rust tests passed with 104 normal tests and two ignored runtime-verification tests that were run separately with `BERGAG_RUNTIME_DB`; Clippy showed no warnings in M9 modules (pre-existing warnings remain in older modules); `npm run tauri build` produced unsigned MSI and NSIS x64 bundles. The built app opened at schema 9 with all nine migrations recorded once, all three performance indexes present, `PRAGMA integrity_check = ok`, one default profile, zero feeds, zero migrations on relaunch, and no error- or warning-level log entries. Runtime ignored tests verified backup create/verify/delete and restore from a copy of the installed database, including the safety backup and unchanged source hash.

Interactive OS-level checks that require a human at the desktop remain enumerated in `docs/RELEASE-CHECKLIST.md`.

Sign-off recorded on 2026-11-21 after review of the automated gates, runtime migration/backup verification, and unsigned bundles. Milestone 9 was the final planned milestone, so the implementation roadmap is now complete. With user authorization, the MIT license was added and a baseline commit and annotated tag `v0.1.0-m9` were created. Installer signing was explicitly deferred by the user; publishing is pending creation of a remote repository. These release decisions are recorded in `docs/RELEASE-CHECKLIST.md`.
