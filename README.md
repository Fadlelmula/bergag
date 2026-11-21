# Bergag

Bergag is a local-first desktop information and reading platform for Windows. It is being built as a private, account-free personal intelligence cockpit: a place to subscribe to sources, read and preserve articles, organize research, and optionally use AI without making the cloud a dependency.

> Status: **Milestones 1–9 approved and complete.** Milestone 9 — Backup, security, final polish was signed off on 2026-11-21 after local verification. The MIT license was added and a baseline commit/tag were created with user authorization. Installer signing was deferred by user decision; the source was published to a public GitHub repository (installer binaries were not released).

## Product principles

- **Local-first:** the local SQLite database is the source of truth. Core reading and organization work offline.
- **Privacy by default:** no telemetry, no ads, no tracking, no mandatory account, no background AI.
- **User control:** destructive or library-changing actions require explicit confirmation.
- **Real functionality:** no fake buttons, demo articles, invented metrics, or placeholder integrations presented as finished.

## Planned capabilities

RSS/Atom/JSON ingestion, feed discovery and autodiscovery, OPML, offline reading, extraction, search with operators, smart feeds, rules, collections, tags, topics, briefings, dashboards, private analytics, optional AI providers, modular integrations, profiles, backups, system tray, and Windows installer. Milestone 2 implemented the feed acquisition layer. Milestone 3 added the local reader: article list modes, sanitized HTML, explicit extraction, read state, saved/favorite/archive, history, notes, highlights, collections, bulk actions, offline images, a reading queue, and focus mode. Milestone 4 adds a private local discovery directory, feed preview and subscription, categories and trust levels, tags and topics, a visual rule builder, smart-feed views, and rule run history. Milestone 5 adds local FTS5 search with operators and scopes, saved searches, duplicate detection, story groups, a story timeline, and related-article connections. Milestone 6 adds user-authored dashboards and widgets, private on-demand analytics, reading statistics, feed health, local storage totals, and daily/weekly briefings generated only from stored data. Milestone 7 adds optional, explicit AI providers (OpenAI, Anthropic, Gemini, Ollama, and OpenAI-compatible HTTPS endpoints), Windows credential-store secrets, per-feature privacy gates, an AI privacy dashboard, and local usage history. Milestone 8 adds public, feed-backed integrations for YouTube, Reddit, GitHub releases, Hacker News, arXiv, podcasts, Medium, Substack, Mastodon public posts, public-newsletter RSS, and website feeds; Bluesky and true website change monitoring remain visibly planned. Milestone 9 adds verified app-managed local backups with safety-backup restore, read-only privacy and security dashboards, local notification preferences, tray and launch-on-sign-in controls, a signature-verified update-check-only framework, and honest security/profile-lock status; Advanced remains planned.

See [ROADMAP.md](ROADMAP.md) for exact status.

## Architecture overview

- **Desktop shell:** Tauri 2 + Rust for native integration and a narrow, validated command surface.
- **Frontend:** React + TypeScript + Vite + Tailwind CSS.
- **Database:** SQLite via `rusqlite`, with versioned migrations, foreign keys, WAL, and FTS5 readiness.
- **Feed engine:** Rust fetches and parses RSS/Atom/JSON directly, validates public URLs, applies ETag/Last-Modified and backoff, and exposes narrow `feeds_*` commands to the UI.
- **Article pipeline:** Rust sanitizes feed content with an allowlist, renders only locally stored `safe_html`, extracts original pages only on request, and stores cached images as data URLs when enabled.
- **Discovery and organization:** a local, user-managed source directory with feed preview, categories, and trust levels; tags and topics; and explicit local rules with smart-feed views and run history.
- **Search and stories:** a profile-scoped SQLite FTS5 index maintained by triggers, a documented operator parser, saved searches, explicit duplicate detection into story groups, a story timeline, and related articles derived from shared labels and story membership.
- **Dashboards and analytics:** profile-scoped dashboards with a bounded widget set edited by the user; reading, library, feed-health, storage, and briefing figures computed on demand from existing SQLite tables, with no telemetry, remote analytics, or invented metrics.
- **Optional AI:** profile-scoped provider configuration and usage events live in SQLite, while API keys live in Windows Credential Manager. A closed command surface performs explicit summary, label-suggestion, and question actions; no background AI runs and no article content is sent without opt-in and confirmation.
- **Integrations:** a closed registry resolves public identifiers and feed URLs into metadata-only, profile-scoped sources. Each source previews a real public feed, reuses the existing subscription and article pipeline, and never stores a credential; planned adapters remain disabled and make no requests.
- **Backups and desktop integration:** app-managed SQLite snapshots with SHA-256 manifests, integrity checks, safety backups, and bounded automatic retention; read-only local privacy/security status; explicit tray, notification, startup, and update-check commands. Update installation and profile encryption are unavailable.
- **Security:** no shell/filesystem plugins exposed to the frontend; all privileged work goes through explicit Tauri commands with input validation and redacted diagnostics. Backups exclude credentials, restore verifies before replacing the library, and dormant profile-lock fields never imply encryption.

See [ARCHITECTURE.md](ARCHITECTURE.md) and [SECURITY.md](SECURITY.md).

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for setup, commands, testing, and build instructions.

Quick start:

```powershell
npm install
npm run dev
```

Desktop development:

```powershell
npm run tauri dev
```

## Privacy overview

Bergag does not enable telemetry, require an account, configure an AI provider, or send article content anywhere by default. Feed fetching is explicit: it happens only for feeds you subscribe to or manually refresh, plus the local scheduler for active subscriptions. Full-text extraction is also explicit and uses the same public-URL validation. Remote article image URLs are removed unless the image was fetched and cached locally, and the webview CSP blocks remote image loads. The discovery directory is local and user-managed: it stores feed URLs you choose and fetches them only when you preview or subscribe. No third-party discovery service is used. Search runs entirely against the local SQLite FTS5 index; saved searches, story groups, and related-article connections stay on this device. Duplicate detection runs only when you choose Detect duplicates, and it never deletes or reorders articles. Semantic search is not enabled: no model runs and no content is uploaded. AI is optional and off by default: you must add a provider, enable AI, enable each feature, allow the content categories to be sent, and confirm each request. Remote AI endpoints must use public HTTPS with DNS pinning; Ollama is a narrow loopback-only exception that also requires an explicit opt-in. API keys are stored in Windows Credential Manager and are never returned to the interface, written to SQLite, or logged. Integrations use only public feeds you explicitly preview and add; they require no account or credential and store only source metadata linked to an ordinary local subscription. Bluesky and true website change monitoring remain planned and disabled. Third-party discovery providers remain planned.

## Known limitations

Semantic search remains unimplemented and visibly marked as planned. Briefings, dashboards, private analytics, and explicit AI features are implemented. AI has no scheduler, automatic labeling, streaming responses, tool use, embeddings, or local model runner beyond an explicitly configured Ollama HTTP endpoint; label suggestions return text only and do not apply labels automatically. Search is lexical FTS5 over locally stored titles, article text, authors, and feed titles; it does not understand meaning or synonyms. Duplicate detection is a manual, heuristic match on canonical URLs and title-token similarity within a 45-day window, so it can miss paraphrased coverage and it creates a story group for every ungrouped article. Discovery is manual: you add feed URLs to a private local directory and can filter that directory by title, URL, or category. There is no third-party search provider. Rules run only when you choose Run or Run enabled rules; there is no hidden background mutation. Image caching is limited to common raster types and at most eight images per extracted article. OPML export contains your feed URLs and folder names, so keep it private. Integrations support public feeds only: authenticated accounts, private subreddits, private repositories, and email-only newsletters are not available. Bluesky and true website change monitoring are visibly planned and make no requests. Local MSI and NSIS bundles build successfully but are unsigned and unpublished. Backups stay in the app-managed backup directory; there is no user-chosen export destination or per-profile export, and only the five newest automatic backups are pruned. Profile lock, encryption, and update installation are unavailable; local builds have no signed update endpoint or public key and never fake a check or progress. Notifications fire only for tray refresh, tray backup, and the explicit test, not for in-app or scheduler events. Privacy and Security posture flags are build-time assertions covered by tests rather than live scans. The project is licensed under MIT; installer signing remains deferred and installer binaries were not released, but the source is published at https://github.com/Fadlelmula/bergag.

## Build from source

Building requires Windows, Node.js 20+, Rust (stable), WiX Toolset v3, and NSIS. From the repository root:

```powershell
npm ci
cmd /c "npm run verify:release"
```

The unsigned MSI and NSIS installers are written to `src-tauri/target/release/bundle/`. See [DEVELOPMENT.md](DEVELOPMENT.md) for the full command reference and gate descriptions.

## Downloads

Source and the signed-off baseline are published at <https://github.com/Fadlelmula/bergag>. Installer binaries have not been released. When releases are published they will appear on the [GitHub Releases page](https://github.com/Fadlelmula/bergag/releases); do not treat any other location as official. See [docs/SIGNING.md](docs/SIGNING.md) for the current signing status.

## Code signing policy

Free code signing provided by SignPath.io, certificate by SignPath Foundation. Bergag is preparing an application to the SignPath Foundation open-source program; **signing is not active yet** and current builds are unsigned. SignPath's certificate attests only to build provenance: copyright of the Bergag source remains with Fadlelmula, as asserted by [LICENSE](LICENSE). Windows displays SignPath Foundation as the signer because the certificate belongs to them, not because they own the software. See [docs/SIGNING.md](docs/SIGNING.md) for the owner-only setup steps and limitations.

- **Authors and reviewers:** the repository owner (`@Fadlelmula`), through repository permissions and branch protection on `main`.
- **Approver:** the repository owner (`@Fadlelmula`). Each signing request requires manual approval and is restricted to the public `release-build.yml` workflow on `refs/tags/v*` running on GitHub-hosted runners.
- **System changes:** launch-on-sign-in and close/tray behavior are explicit user settings. Bergag does not change system configuration without the user's action, and MSI/NSIS installation provides an uninstall path.
- **Privacy statement:** Bergag has no telemetry, analytics, ads, or account, and sends no user data to systems the user has not specified. Network activity is limited to public feeds and article URLs the user subscribes to or previews (including a local 60-second refresh schedule for active subscriptions), explicit full-page extraction, locally cached images, optional AI endpoints the user configures and confirms per request, and update checks only when a signed endpoint and public key are compiled in. No article content, library data, or credential is transmitted by default. Third-party components and any public feed or integration the user adds have their own terms; see [SECURITY.md](SECURITY.md).

## License

Bergag is licensed under the [MIT License](LICENSE), added on 2026-11-21 with the user's explicit choice. See [docs/RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md) for the remaining release decisions.
