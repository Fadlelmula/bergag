# Initial project plan

## Current state

`C:\Bergag` was empty and not a git repository. The machine has Node 26/npm 11, Rust 1.100 with the MSVC target, VS Build Tools 2022, and WebView2 Runtime. No existing user work was found to preserve.

## Proposed architecture

Tauri 2 + Rust for native concerns; React 19 + TypeScript + Vite + Tailwind v4 for the UI; SQLite via `rusqlite` with explicit migrations; TanStack Query for async state; Radix primitives for accessibility; Vitest/Testing Library and Rust tests for verification.

Rationale: Tauri gives a small native Windows shell with a secure IPC boundary. Keeping persistence and OS integration in Rust with narrow commands avoids exposing broad filesystem/shell capabilities to untrusted web content, while application rules stay in testable TypeScript.

## Major risks

| Risk                                    | Mitigation                                                                                      |
| --------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Web content XSS or privileged IPC abuse | Strict CSP, sanitization in M3, no shell/fs plugins, explicit command validation.               |
| Cross-profile leakage                   | `profile_id` on every scoped table, cascade deletes, isolation tests.                           |
| Migration damage or data loss           | Append-only migrations, transactions, backup before destructive restore later, migration tests. |
| Feed sites are unreliable               | Conditional requests, timeouts, backoff, concurrency limits, per-source isolation in M2.        |
| Extraction/security conflict            | Sanitize first, isolate rendering, never execute fetched scripts.                               |
| Credential exposure                     | OS credential store later, redaction tests, no secrets in DB/logs/exports.                      |
| Scope creep                             | Strict milestone gates; unimplemented UI is labeled honestly.                                   |
| Windows packaging prerequisites         | Verify toolchain early; report any installer-specific blocker precisely.                        |

## Milestone 1 task list

1. Initialize git and project structure.
2. Create docs, ignore rules, and placeholder-free README.
3. Scaffold Tauri 2, React, TypeScript, Vite, Tailwind, ESLint, Prettier, Vitest.
4. Implement SQLite migrations, app settings, profiles, layouts, diagnostics.
5. Implement validated Tauri commands and redacted file logging.
6. Implement frontend layers, providers, theme system, design tokens.
7. Implement shell: sidebar, resizable panels, foundation/status view, settings dialog.
8. Add tests: validation/defaults, component behavior, migrations, profile isolation, redaction.
9. Run formatter, lint, typecheck, unit tests, Rust tests, frontend build, Tauri build.
10. Report results and stop for approval.

## Milestone 1 acceptance criteria

- App builds and launches on Windows.
- Database initializes with versioned migrations and passes integrity check.
- Profiles are isolated and switching works.
- Settings and layouts persist across restarts.
- No fake production data or secrets are present.
- Basic tests pass.
- Security boundary is limited to explicit commands.
