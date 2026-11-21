# Security policy

## Reporting a vulnerability

Do not open a public issue for a security vulnerability. Report it privately to the repository maintainer through GitHub's private vulnerability reporting feature, or by contacting the maintainer directly if private reporting is unavailable.

Include reproduction steps, affected version, and impact. Do not include live credentials or private article data.

## Credential handling

- Bergag must not ship API keys, tokens, passwords, or signing secrets.
- AI API credentials are stored in Windows Credential Manager through `keyring` 3.6.3 with `windows-native`, scoped by `profile_id::provider_id`. Future integration credentials must use the same OS-backed store.
- Only non-secret provider identifiers and configuration belong in application settings and SQLite. There is no command that returns a stored API key to the webview.
- Diagnostics and logs are redacted and must never expose credentials.
- Exported configuration excludes credentials by default. OPML export is not credential data, but it contains private feed URLs and folder names and must be handled as sensitive user data.

## Network and content model

Fetched feeds, webpages, and integration payloads are untrusted. HTML must be sanitized before rendering. Scripts, unsafe embeds, tracking pixels intentionally inserted by Bergag, and privileged command calls from web content are prohibited. Article content is stored as sanitized `safe_html`; feed summaries remain available as plain-text fallback. The webview never receives raw feed or webpage HTML for rendering.

Feed fetching validates schemes and targets: only public `http`/`https` URLs are accepted, userinfo and private/loopback/link-local addresses are rejected, redirects are re-validated, DNS is pinned for the request, and fetches have timeouts and response limits. Failures are isolated per feed and recorded as redacted refresh logs. Full-page article extraction is explicit and reuses the same validator with a 5 MB HTML cap. Discovery preview and directory subscription use the same public-URL validator and fetch only when the user explicitly previews, saves, or subscribes; there is no third-party search or directory provider. Image caching accepts only common raster types up to 2 MB each, stores at most eight images, and rewrites them to local `data:` URLs; uncached remote image sources are removed and the CSP blocks remote image loads. Local-provider exceptions such as Ollama are not enabled in the feed engine. The AI system introduces a separate, narrowly scoped Ollama provider that requires an explicit local-provider opt-in and accepts only literal loopback HTTP endpoints; no other local address is reachable through AI configuration.

## Discovery, labels, and rules

- The discovery directory, tags, topics, rules, and run history are profile-scoped. Reads and writes are keyed by the active profile, and cross-profile feed or label references are rejected.
- Rule definitions are untrusted structured input. They are bounded to 20 KB, 1–20 conditions and actions, and 200-character text values; rule JSON is deserialized into a closed enum, not executed as SQL or code.
- Rule execution is explicit. A rule runs only when the user chooses Run or Run enabled rules, applies changes inside one transaction, and records local run history. There is no hidden background mutation of the library.
- Labels, categories, and trust levels are user-authored metadata. They are stored locally and never transmitted to a remote service by Bergag core.

## Search, saved searches, and stories

- The FTS5 index, saved searches, story groups, and story membership are profile-scoped. Every search and story command resolves the active profile server-side and keys SQL by `profile_id`.
- Search input is parsed into a bounded operator structure, then executed with parameterized SQL. Invalid operators, dates, and unbalanced quotes are rejected before reaching SQLite. Text terms are never concatenated into SQL.
- The FTS index is maintained by local triggers from stored item and article text. Search does not make network requests, does not call a semantic model, and does not send queries or article content anywhere.
- Saved searches are limited to 1–500 query characters and 1–64 name characters, and are unique per profile after normalization.
- Duplicate detection runs only on explicit request, uses canonical URLs and title-token similarity, and never deletes or reorders articles. Removing a story group deletes only the grouping.
- Related articles and story timelines are computed locally from shared labels and story membership.

## Dashboards, briefings, and analytics

- Dashboards and widget configurations are profile-scoped. Every dashboard/analytics command resolves the active profile server-side; limits are bounded to 12 dashboards and 24 widgets per dashboard.
- Widget kinds are a closed set. They are configuration records, not executable code, and they read only local SQLite data when rendered.
- Analytics are computed on demand from existing local tables. No analytics fact tables, telemetry, remote analytics, model inference, demo data, or invented metrics are introduced.
- Briefings are generated on demand from local discovery, history, unread state, highlights, and notes. There is no scheduling, delivery, notification, or cloud service.
- Feed-health error text is redacted before display. Storage figures are read-only; cache cleanup remains an explicit separate action.
- Dashboard deletion removes only dashboard and widget configuration. Articles, feeds, reading history, annotations, and labels remain in the library.

## Optional AI

- AI is off by default. A provider must be added and selected, AI must be enabled, the specific feature must be enabled, and article text consent must be granted before any article feature can run. Label suggestions additionally require label-name consent. Confirmation is on by default for every request.
- Only three explicit AI actions exist: article summarization, label suggestions, and article Q&A. There is no AI scheduler, background project, auto-labeling, embedding generation, semantic search, streaming request, tool use, or content upload outside those actions.
- Remote providers require public HTTPS endpoints and reuse the feed engine's DNS pinning, private-address rejection, redirect re-validation, 45-second timeout, and 2 MB response cap. Ollama is a separate exception: loopback-only HTTP, bounded model/endpoint text, no redirects, and no credentials.
- API keys are written only to Windows Credential Manager. No API key is returned to the webview, stored in SQLite/profile settings, included in diagnostics, or written to logs. Usage-event error text is redacted before storage.
- Prompts are assembled in Rust from bounded local article text plus only the optional highlights, notes, and label names the user has allowed. Provider responses are trimmed, capped, and shown as plain text.
- Provider configuration and usage events are profile-scoped. Deleting a provider removes its credential and detaches historical usage from that provider; deleting a profile removes credentials for its AI providers as part of the same validated operation.
- The AI privacy dashboard reports only local usage rows and provider-reported token counts. It does not estimate or invent tokens, costs, or activity.

## Public feed integrations

- Milestone 8 integrations are public and credential-free. The `integration_sources` table stores only metadata and a nullable link to an ordinary `feeds.id`; it has no token, password, API key, or cookie columns.
- Platform kinds are a closed registry. Adapters normalize bounded identifier or URL input to exact public feed URLs and reject invalid formats, private addresses, oversized input, and planned adapters before any request.
- Preview, add, refresh, and remove are explicit user actions. Adding reuses the existing public-feed validator, parser, sanitizer, subscription path, scheduler, and refresh log; no integration command opens an arbitrary URL, shell, file, or authenticated API.
- Duplicate detection uses the active profile's normalized feed URLs and integration keys. Cross-profile source ids are rejected, and unique constraints prevent conflicting metadata.
- Refresh failures are stored as redacted status text on the integration source. Removing a source deletes only metadata; the feed, articles, annotations, labels, and reading history remain local.
- Bluesky and true website change monitoring are visibly planned and disabled. They make no network requests and are not represented as available features.

## Backups and restore

- Backups are app-managed SQLite snapshots with a JSON manifest containing app version, schema version, profile list, creation time, and SHA-256 hash. Backup size is recorded in the app-managed `index.json`, not the manifest. No credentials, live logs, or arbitrary files are included.
- Restore verifies checksum, manifest fields, schema compatibility, profile list, and SQLite `integrity_check` before changing the active library. A verified safety backup is created first. Corrupt or incompatible backups are refused without touching the current database; older restored schemas are migrated after verification.
- Only the five newest automatic backups are pruned. Backup identifiers are validated before file paths are built. Manual backups, the active database, and files outside the app-managed backup directory are never deleted by retention.
- Profile deletion also creates a verified safety backup before removal. Restoring a backup is an explicit action with a confirmation that states the current library will be replaced; there is no destination picker or silent overwrite.

## Desktop boundary

The webview receives only explicit Tauri commands. The app must not expose unrestricted shell execution, arbitrary filesystem access, or dynamic SQL. Capabilities are deny-by-default and expanded only for implemented features. Notification, autostart, and updater plugins run behind named Bergag commands; their plugin commands are not granted to the webview.

The Privacy and Security dashboards count and display real local rows, paths, permission state, and schema fields. Flags for command-surface shape, URL validation, sanitization, and log redaction are build-time assertions covered by Rust tests; the UI labels them as such rather than claiming a live capability scan.

## Notifications, tray, startup, and updates

- Desktop notifications are off by default and store only local preferences. Actual OS permission and support state are reported; denied, quiet-hour, unsupported, and failed sends are reported honestly. Notifications are limited to tray-triggered feed refresh, tray-triggered backup, and an explicit test. The 60-second scheduler, in-app refresh, and in-app backup do not notify. No notification content is sent to a server.
- Tray actions call the same underlying services as their in-app equivalents: show window, refresh due feeds, create a labelled local backup, and quit. Close behavior is a local quit/hide-to-tray setting. If the tray is unavailable, no tray is created and the control is disabled.
- Launch-on-sign-in changes only a Windows sign-in entry and is explicit. Non-Windows builds report it unavailable and create no entry.
- Update checks are opt-in. A build must contain a valid public HTTPS endpoint and minisign public key; otherwise status says unconfigured, no network check runs, and no install is offered. Installation stays disabled in local builds. No fake progress or unverified installer is shown.

## Profile lock status

The `is_locked` and `lock_hint` columns are dormant schema fields. Bergag does not encrypt, password-protect, or restrict access by profile. Security and Profiles settings both label the lock unavailable, no command sets those fields, and no UI implies encryption that does not exist.

## Supported versions

Only the latest released version receives security fixes while the project is pre-1.0.

## Threat model summary

Primary assets: local article library, notes/highlights, labels, rules, search queries and saved searches, story groups, the discovery directory, dashboard/widget configuration, private analytics, profile separation, and optional credentials. Primary threats: malicious feed/web content, cross-profile data leakage, crafted search/rule or label input, credential exposure, unsafe native capabilities, destructive operations, and compromised backups. Mitigations are documented in `ARCHITECTURE.md` and enforced through tests and review.
