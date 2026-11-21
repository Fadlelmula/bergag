# Release checklist

This checklist is the single release-readiness reference for Bergag. A milestone is not ready for sign-off until every automated gate and every applicable manual check has been completed and recorded in `DEVELOPMENT.md`.

## Automated gates

Run from the repository root on Windows:

```powershell
cmd /c "npm run verify"
cmd /c "npm run verify:release"
```

`npm run verify` performs:

- `npm run format:check`
- `npm run lint`
- `npm run typecheck`
- `npm run test`
- `npm run build`
- `npm run test:rust`
- `npm run clippy`

`npm run verify:release` also runs `npm run tauri build`, producing unsigned MSI and NSIS x64 bundles under `src-tauri/target/release/bundle/`.

### Recorded Milestone 9 result

- [x] Format check, lint, typecheck, 78 frontend tests, and production build passed.
- [x] Rust: 104 normal tests passed; two ignored runtime tests passed when run explicitly with `BERGAG_RUNTIME_DB`.
- [x] Clippy: no warnings in Milestone 9 modules; 20 library warnings and 25 test-target warnings remain in pre-M9 modules.
- [x] MSI and NSIS x64 bundles were built locally at `src-tauri/target/release/bundle/msi/Bergag_0.1.0_x64_en-US.msi` and `src-tauri/target/release/bundle/nsis/Bergag_0.1.0_x64-setup.exe`.
- [x] Runtime launch confirmed schema 9, all nine migrations recorded once, all three performance indexes, `PRAGMA integrity_check = ok`, one profile, zero feeds, zero migrations on relaunch, and no error- or warning-level logs.
- [x] Runtime backup tests verified create/verify/delete and restore from a copy of the installed database, including safety backup, marker-profile removal, restored integrity, and unchanged source hash.

The previous M8 baseline was 54 frontend tests and 85 Rust tests. The current recorded baseline is 78 frontend tests and 104 normal Rust tests plus two ignored runtime-verification tests.

## Manual release checks

### Fresh install and upgrade

- [x] Upgrade from the signed-off M8 build preserved the profile and library; the app upgraded schema 8→9, recorded `performance_indexes` once, and seeded no content.
- [x] Second launch applies zero migrations.
- [x] `PRAGMA integrity_check` returns `ok` after launch.
- [x] Logs contain no error- or warning-level entries.
- [ ] A clean-machine fresh install is still to be exercised by a human before public release.

### Backup and restore

- [x] A verified backup can be created in the app-managed backup directory. There is no user-chosen export destination command; do not claim one.
- [x] The manifest records app version, schema version, profile list, creation time, and SHA-256 hash. Size is recorded in `index.json`, not the manifest. No credentials or live logs are included.
- [x] Restore into the active library requires explicit confirmation and creates a verified safety backup first; it never silently overwrites.
- [x] A corrupted backup is refused by the verification path and leaves the current library unchanged in unit tests.
- [x] Retention unit tests confirm only automatic records are pruned; restore and retention validate backup IDs before building paths.
- [ ] Restored profiles preserving every article, note, highlight, label, rule, dashboard, AI configuration, and integration metadata should be spot-checked against a populated library before release.

### Privacy and security

- [x] No telemetry, ads, hidden analytics, account requirement, or mandatory AI feature is present.
- [x] Private and public URL validation refuses loopback, private, link-local, and oversized targets for public feed traffic.
- [x] No shell, unrestricted filesystem, generic network, or dynamic SQL command is exposed to the webview.
- [x] Profile-scoped commands reject ids from another profile.
- [x] Destructive actions show explicit confirmation and explain what is kept.
- [x] AI remains opt-in with per-feature gates, per-request confirmation, and no secret returned to the UI.
- [x] Integration sources remain public-feed-only; Bluesky and website monitoring remain disabled.
- [x] Profile lock/encryption is unavailable and labeled honestly in both Security and Profiles settings.
- [x] Dashboard posture flags that are build-time assertions are labeled as such.
- [ ] No secrets, databases, WAL/SHM sidecars, live logs, backups, signing keys, or installers are staged. Run the artifact inspection below before any commit or handoff.

### Tray, notifications, and startup

- [x] Tray controls call real commands: show window, refresh due feeds, create a labelled local backup, and quit.
- [x] Close behavior is stored as a local quit/hide-to-tray setting; the UI uses radiogroup semantics and announces changes.
- [x] Notification permission/support state drives the controls, and denied, quiet-hour, unsupported, and failed sends are reported honestly.
- [x] Startup status is read through the real plugin command and is labeled unavailable on unsupported platforms.
- [ ] Human verification remains for: Windows tray icon creation and all four tray actions, close behavior across relaunch, permission states `granted`/`denied`/`prompt`, quiet-hour suppression, and writing/removing the Windows sign-in entry.

### Auto-update framework and installer

- [x] Update status is shown only from compile-time configuration; no fake version, progress, or result is displayed.
- [x] A build must contain a valid public HTTPS endpoint and minisign public key; otherwise status says unconfigured, no network check runs, and no install is offered.
- [x] `tauri.conf.json` supplies an empty updater plugin configuration so unconfigured local builds start successfully.
- [x] Installation stays disabled; no unverified installer is offered.
- [x] Installer architecture and version match the build: 0.1.0 x64 MSI and NSIS.
- [x] Signing and distribution are not performed without explicit permission.
- [x] License choice resolved: the user selected MIT on 2026-11-21 and the license file was added.

### Performance and accessibility

- [x] No fake progress is shown. Long-running network work is bounded by refresh batch 20, concurrency 3, 5 MB feed HTML, and 200 stored items per refresh.
- [x] Profile switching and feed refresh use scoped query invalidation; settings Search stats uses the correct profile-scoped key.
- [x] New settings rows are keyboard reachable, use labelled switches/buttons/radiogroups, expose live status and role-alert errors, and keep planned entries focusable.
- [x] Skip link targets `#bergag-workspace`, has a high-contrast focus style, and reduced motion is honored.
- [ ] Human verification remains for cold start, large-library search, text readability at supported zoom settings, and a full keyboard-only walkthrough.

## Known bounds

These are intentional bounds, not claims of unbounded scaling:

- 500 listed feeds per profile, 20-feed refresh batches, refresh concurrency 3, 60-second scheduler, and 200 stored feed items per refresh.
- 12 dashboards, 24 widgets per dashboard, 5,000-item rule evaluation, 45-day duplicate window, and 500-character saved searches.
- 5 MB extraction HTML, 2 MB per cached image, eight cached images per extracted article, and five retained automatic backups.

## Rollback and recovery

- Use Settings → Backups to restore a verified local backup. A verified safety backup is created before replacement.
- If an installed build is unusable, manually reinstall a previous unsigned bundle. There is no in-app installer, no in-app rollback, and no fake rollback progress.
- Rollback does not change the license, signing, or publishing decisions.

## Release decisions

- Local MSI and NSIS bundles are unsigned and unpublished.
- No signing key is configured or stored in the repository; signing requires explicit permission.
- The license is MIT, chosen by the user on 2026-11-21.
- The update endpoint and public key are unset in local builds.
- A baseline commit, annotated tag `v0.1.0-m9`, and MIT license file were created with explicit permission. The commit and tag were pushed to https://github.com/Fadlelmula/bergag; installer signing and binary release were not performed.

## Artifact inspection

Before publishing or handing off:

```powershell
git status --short
git diff --check
Get-ChildItem -Path . -Recurse -File -Include *.sqlite3,*.sqlite3-wal,*.sqlite3-shm,*.sqlite3-journal,*.log,*.p12,*.pfx,*.key,*.pem,*.msi,*.exe |
  Where-Object { $_.FullName -notmatch '\\target\\|\\node_modules\\' } |
  Select-Object -ExpandProperty FullName
Get-ChildItem -Path . -Recurse -File |
  Where-Object { $_.FullName -notmatch '\\target\\|\\node_modules\\|\\.git\\' } |
  Select-String -Pattern '(api[_-]?key|secret|password|token)\s*=' |
  Select-Object Path, LineNumber, Line
```

Review the full file list. Never stage databases, WAL/SHM/journal sidecars, logs, backups, credentials, certificates, signing keys, installers, `node_modules`, or `target` output.

## Sign-off note

Milestone 9 was signed off by the user on 2026-11-21 after review of the automated gates, runtime migration/backup verification, and unsigned bundles. It was the final planned milestone. The remaining human desktop checks in this checklist are still outstanding before a public release. With user authorization, the MIT license was added and a baseline commit and annotated tag `v0.1.0-m9` were created. Installer signing was explicitly deferred by the user; the source and tag were pushed to https://github.com/Fadlelmula/bergag, and installer binaries were not released.

## Baseline note

The signed-off baseline was created on 2026-11-21 as an initial commit with an annotated tag `v0.1.0-m9`, using the identity Fadlelmula <work@fadlelmula.com>. The commit and tag were pushed to the public repository https://github.com/Fadlelmula/bergag. Installer binaries were neither signed nor released.
