# Contributing

Bergag is developed in milestones. Do not introduce later-milestone features into an earlier milestone without an explicit scope decision and roadmap update.

## Ground rules

1. Preserve user data. No silent deletes, overwrites, or schema resets.
2. Never commit secrets, databases, backups, private logs, caches, environment files with real values, certificates, or signing keys.
3. Every visible control must do real work or be clearly labeled as planned/unavailable.
4. Add tests with behavior changes, including invalid inputs and failure paths.
5. Run formatting, linting, type checking, tests, and the actual build before requesting milestone review.
6. Keep AI opt-in and privacy disclosures explicit.

## Commit hygiene

Before committing, inspect staged files and search for credential-like strings. Never push or publish without explicit permission.

## Suggested workflow

```powershell
cmd /c "npm run verify"
cmd /c "npm run verify:release"
```

`npm run verify` runs the format check, lint, type check, frontend tests, production build, Rust tests, and Clippy. `npm run verify:release` adds the actual Tauri MSI and NSIS builds. See [docs/RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md) for manual release checks.
