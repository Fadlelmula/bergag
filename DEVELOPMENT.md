# Development

## Environment

Verified development environment:

- Windows 11 (build 26300)
- Node.js 26.13.0
- npm 11.20.0
- Rust 1.100.0 (`x86_64-pc-windows-msvc`)
- Visual Studio Build Tools 2022 with MSVC
- Microsoft Edge WebView2 Runtime 157.0.4341.60

If `npm` is blocked by PowerShell execution policy, run commands with `cmd /c npm ...` or set a process-scoped bypass.

## Install

```powershell
cmd /c "npm install"
```

## Commands

| Command                  | Purpose                                                                       |
| ------------------------ | ----------------------------------------------------------------------------- |
| `npm run dev`            | Vite webview development server. Tauri IPC is unavailable in a plain browser. |
| `npm run preview`        | Preview the production frontend build locally.                                |
| `npm run tauri dev`      | Launch the native desktop app in development mode.                            |
| `npm run build`          | Type-check and build the frontend.                                            |
| `npm run tauri build`    | Build the Windows desktop bundle.                                             |
| `npm run test`           | Run Vitest once.                                                              |
| `npm run test:watch`     | Run Vitest in watch mode.                                                     |
| `npm run test:rust`      | Run Rust tests.                                                               |
| `npm run lint`           | Run ESLint.                                                                   |
| `npm run typecheck`      | Run TypeScript without emitting files.                                        |
| `npm run clippy`         | Run Clippy across all Rust targets.                                           |
| `npm run format`         | Format source files with Prettier.                                            |
| `npm run format:check`   | Check formatting.                                                             |
| `npm run verify`         | Run all non-packaging quality gates.                                          |
| `npm run verify:release` | Run all quality gates plus the actual MSI and NSIS builds.                    |

The single non-packaging gate sequence is `cmd /c "npm run verify"`; the full release sequence is `cmd /c "npm run verify:release"`, which repeats verification and then runs the real MSI and NSIS builds. See [docs/RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md) for the manual release checks that accompany these commands.

Optional compile-time update configuration can be supplied through `BERGAG_UPDATE_ENDPOINT` and `BERGAG_UPDATE_PUBLIC_KEY`. Both must be valid before the app offers a signed update check. Leaving them unset produces an honest “unconfigured” status, no network request, and no install control. A private signing key must never be added to this repository.

## Project structure

```text
src/                    React frontend
src-tauri/              Rust/Tauri native shell
src-tauri/migrations/   Versioned SQL migrations
docs/                   Design and milestone planning
scripts/                Icon and maintenance scripts
```

## Testing expectations

Every milestone must add tests for its own behavior plus regressions for:

- migration upgrades from previous versions,
- profile isolation,
- input validation,
- error and offline paths,
- destructive-action guards,
- security-sensitive redaction,
- backup creation, verification, corruption refusal, restore, safety backups, and automatic-only retention,
- notification permission/quiet-hour and tray/startup/update status DTOs,
- privacy/security dashboard counts and dormant profile-lock fields.

No milestone is complete with failing tests or a failing build.

## Troubleshooting

### `npm.ps1 cannot be loaded`

PowerShell execution policy blocks npm. Use `cmd /c "npm <command>"` or:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

### Rust linker not found

Ensure Visual Studio Build Tools include **Desktop development with C++** and launch commands from a Developer PowerShell, or verify `vcvars64.bat` is available under `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build`.

### Tauri build cannot find WebView2

Install the current WebView2 Runtime. This development machine already has it installed.

## Milestone 1 verification

Verified on 2026-11-20:

- Frontend: lint, typecheck, format check, Vitest (11 tests), and Vite production build passed.
- Backend: 20 Rust tests passed, including migrations, profile isolation, validation, and redaction.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened a window titled “Bergag”, created `%APPDATA%\com.bergag.desktop\bergag.sqlite3`, and wrote redacted startup logs under `%LOCALAPPDATA%\com.bergag.desktop\logs\`.
- Database: schema version 1, migration `foundation` applied once, one default profile, and `PRAGMA integrity_check` returned `ok`.

## Milestone 2 verification

Verified on 2026-11-20:

- Frontend: format check, lint, typecheck, Vitest (16 tests), and Vite production build passed.
- Backend: 35 Rust tests passed, including feed parsing, autodiscovery, SSRF URL guards, OPML parsing, duplicate detection, backoff, refresh logs, profile isolation, and schema upgrades.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened successfully and the local database upgraded from schema version 1 to 2. A second launch applied no migrations.
- Database: migrations `foundation` and `feed_engine` each recorded once, schema version 2, one default profile, zero feeds/folders/items/refresh logs (no seeded demo data), and `PRAGMA integrity_check` returned `ok`.
- Logs: no errors, warnings, or panics were recorded during verification launches.

## Milestone 3 verification

Verified on 2026-11-20:

- Frontend: format check, lint, typecheck, Vitest (23 tests), and Vite production build passed.
- Backend: 42 Rust tests passed, including HTML sanitization, extraction selection, raster image validation, article list modes, annotations, collection cascades, cache clearing, and schema upgrades.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened successfully and upgraded the local database from schema version 2 to 3. A second launch applied no migrations.
- Database: migrations `foundation`, `feed_engine`, and `article_system` were each recorded once; there was one default profile and zero feeds, folders, items, article sources, states, collections, or cached images (no seeded demo data); `PRAGMA integrity_check` returned `ok`.
- Logs: no error-level entries were recorded during verification launches.

## Milestone 4 verification

Verified on 2026-11-20:

- Frontend: format check, lint, typecheck, Vitest (27 tests), and Vite production build passed.
- Backend: 48 Rust tests passed, including discovery-source isolation and upsert, label isolation and assignment counts, rule JSON reference validation, rule execution, state and label changes, run history, cross-profile denial, and schema upgrades.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened successfully and upgraded the local database from schema version 3 to 4. A second launch applied no migrations.
- Database: migrations `foundation`, `feed_engine`, `article_system`, and `discovery_organization` were each recorded once; there was one default profile and zero feeds, folders, items, discovery sources, labels, rules, rule runs, collections, or article sources (no seeded demo data); `PRAGMA integrity_check` returned `ok`.
- Logs: no error-level entries were recorded during verification launches.

## Milestone 5 verification

Verified on 2026-11-20:

- Frontend: format check, lint, typecheck, Vitest (30 tests), and Vite production build passed.
- Backend: 57 Rust tests passed, including operator parsing, filter-only search, FTS profile isolation, saved searches, stats, duplicate grouping, idempotent detection, story isolation, related-article connections, and schema upgrades.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened successfully and upgraded the local database from schema version 4 to 5. A second launch applied no migrations.
- Database: migrations `foundation`, `feed_engine`, `article_system`, `discovery_organization`, and `search_story` were each recorded once; the FTS5 `search_documents` table and six sync triggers were present; there was one default profile and zero feeds, folders, items, article sources, discovery sources, labels, rules, collections, search documents, saved searches, story groups, or story membership (no seeded demo data); `PRAGMA integrity_check` returned `ok`.
- Logs: no error- or warning-level entries were recorded during verification launches.

## Milestone 6 verification

Verified on 2026-11-20:

- Frontend: format check, lint, typecheck, Vitest (39 tests), and Vite production build passed.
- Backend: 61 Rust tests passed, including dashboard/widget validation and profile isolation, analytics counts, empty windows, invalid inputs, and schema upgrades.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened successfully and upgraded the local database from schema version 5 to 6. A second launch applied no migrations.
- Database: migrations `foundation`, `feed_engine`, `article_system`, `discovery_organization`, `search_story`, and `dashboards_analytics` were each recorded once; `dashboards`, `dashboard_widgets`, `idx_dashboards_default`, and `idx_dashboard_widgets_dashboard_order` were present; there was one default profile and zero feeds, items, dashboards, widgets, or other seeded content; `PRAGMA integrity_check` returned `ok`.
- Logs: no error- or warning-level entries were recorded during verification launches.

## Milestone 7 verification

Verified on 2026-11-20:

- Frontend: format check, lint, typecheck, Vitest (45 tests), and Vite production build passed.
- Backend: 75 Rust tests passed, including provider config validation, response parsing, credential key scoping, redacted usage errors, privacy-scoped prompts, profile isolation, usage stats, and schema upgrades.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened successfully and upgraded the local database from schema version 6 to 7. A second launch applied no migrations.
- Database: migrations `foundation`, `feed_engine`, `article_system`, `discovery_organization`, `search_story`, `dashboards_analytics`, and `ai_system` were each recorded once; `ai_providers`, `ai_usage_events`, `idx_ai_providers_profile_enabled`, `idx_ai_usage_profile_time`, and `idx_ai_usage_profile_feature` were present; there was one default profile and zero providers, usage events, dashboards, widgets, feed items, or other seeded content; `PRAGMA integrity_check` returned `ok`.
- Logs: no error- or warning-level entries were recorded during verification launches.

AI credentials use Windows Credential Manager through `keyring` 3.6.3 with `windows-native`. `keyring` 4.x was not used because it requires a newer Rust toolchain than the project's declared MSRV. To exercise a provider manually, configure it in Settings → AI, store its key, enable the desired feature and content category, then run an action from the AI view and confirm the destination.

## Milestone 8 verification

Verified on 2026-11-20:

- Frontend: format check, lint, typecheck, Vitest (54 tests), and Vite production build passed.
- Backend: 85 Rust tests passed, including adapter normalization, invalid/private/planned inputs, profile isolation, uniqueness and duplicate reuse, source state transitions, closed constraints, indexes, schema 7→8 upgrade, and zero seeded content.
- Clippy: no new Milestone 8 warnings; 20 pre-existing warnings remain outside the integration modules.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.
- Launch: the built `bergag.exe` opened successfully and upgraded the local database from schema version 7 to 8. A second launch applied no migrations.
- Database: migrations `foundation`, `feed_engine`, `article_system`, `discovery_organization`, `search_story`, `dashboards_analytics`, `ai_system`, and `integrations` were each recorded once; `integration_sources`, `idx_integration_sources_profile_kind`, `idx_integration_sources_profile_feed`, and `idx_integration_sources_profile_status` were present; there was one default profile and zero integration sources, feeds, feed items, discovery sources, labels, rules, dashboards, AI providers, or AI usage events (no seeded demo data); `PRAGMA integrity_check` returned `ok`.
- Logs: no error- or warning-level entries were recorded during verification launches.

Milestone 8 integrations require no credentials and cannot reach private targets because all adapter URLs pass through the existing public feed validator. To exercise a source manually, open Settings → Integrations, choose an available platform, enter a public identifier or feed URL, preview the real feed, then choose Add & subscribe. Remove only deletes the local integration metadata; the subscribed feed and saved articles remain.

## Milestone 9 verification

Verified on 2026-11-21:

- Frontend: format check, lint, typecheck, Vitest (78 tests), and Vite production build passed.
- Backend: 104 normal Rust tests passed. Two runtime-verification tests are ignored by default and were run explicitly with `BERGAG_RUNTIME_DB` pointing at the installed database; both passed.
- Clippy: no warnings in Milestone 9 modules. The pre-existing 20 library warnings and 25 test-target warnings remain in older modules.
- Packaging: `npm run tauri build` produced unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`. The MSI was 10.51 MiB and the NSIS installer 6.07 MiB.
- Launch: the built `bergag.exe` opened successfully. The runtime database is at schema 9 with all nine migrations recorded once. A relaunch applied 0 migrations.
- Database: `idx_feeds_profile_status_due`, `idx_feed_items_profile_sort`, and `idx_feeds_profile_title_nocase` were present; there was one default profile and zero feeds (no seeded demo data); `PRAGMA integrity_check` returned `ok`.
- Logs: the verification launches recorded no error- or warning-level entries.
- Backup runtime checks: the ignored tests copied the installed database, created and verified a backup, deleted it, then created a second backup, added a marker profile, restored from the backup, confirmed the marker was gone, verified the safety backup and source backup, checked integrity, and confirmed the installed source database hash was unchanged.

The ignored runtime tests are intentionally not part of the normal gate because they require a local installed database:

```powershell
$env:BERGAG_RUNTIME_DB = "$env:APPDATA\com.bergag.desktop\bergag.sqlite3"
cargo test --manifest-path src-tauri\Cargo.toml runtime_ -- --ignored --nocapture
```

To exercise Milestone 9 manually, open Settings → Backups to create, verify, and optionally restore a local backup; confirm that restore warns before replacing the library and that a safety backup appears. Settings → Privacy and Security should report local counts, paths, and build-time posture without invented metrics. Settings → Notifications should report the real OS permission state and never claim a denied or quiet-hour send succeeded. Settings → Tray, Startup, and Updates should show real state, with Updates reporting “Not configured” and installation unavailable in local builds. Profile lock is intentionally unavailable in both Security and Profiles settings.

Milestone 9 was signed off by the user on 2026-11-21 and was the final planned milestone. With authorization, the MIT license was added and a baseline commit and annotated tag `v0.1.0-m9` were created. Installer signing was explicitly deferred by the user. The source and tag were pushed to https://github.com/Fadlelmula/bergag; installer binaries were not released.
