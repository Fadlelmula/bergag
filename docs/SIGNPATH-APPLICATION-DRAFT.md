# SignPath Foundation application draft (not submitted)

Copy/paste reference for <https://signpath.org/apply>. **Nothing here has been submitted.**

## Eligibility answers

- **Project name:** Bergag
- **Repository:** <https://github.com/Fadlelmula/bergag>
- **License:** MIT — <https://github.com/Fadlelmula/bergag/blob/main/LICENSE>. OSI-approved, no commercial dual-licensing.
- **Download URL:** <https://github.com/Fadlelmula/bergag/releases/tag/v0.1.0-m9>
- **Released version:** 0.1.0, tag `v0.1.0-m9`
- **Description:** Bergag is a local-first, account-free desktop information and reading platform for Windows. It stores feeds, articles, dashboards, notes, and backups in a local SQLite database. It has no telemetry, ads, accounts, or mandatory AI. Source is public under the MIT license.
- **Artifacts to sign:** Windows x64 MSI and NSIS installers — `Bergag_0.1.0_x64_en-US.msi` and `Bergag_0.1.0_x64-setup.exe`.
- **Build system:** GitHub Actions on GitHub-hosted `windows-latest` runners. Workflow: `.github/workflows/release-build.yml`, triggered on `refs/tags/v*`.
- **Documentation:** `README.md`, `ARCHITECTURE.md`, `DEVELOPMENT.md`, `SECURITY.md`, `docs/SIGNING.md`.
- **Copyright holder:** Fadlelmula, asserted in `LICENSE`. SignPath's certificate attests only to build provenance; copyright is not transferred.
- **Malware / potentially unwanted programs:** none. No telemetry, hidden analytics, auto-install, or system changes without explicit user action. Launch-on-sign-in and tray behavior are opt-in settings.
- **Proprietary components:** none. System libraries supplied by Windows (e.g. WebView2) are not bundled as proprietary product code.
- **Maintained:** yes — Milestones 1–9 were completed and signed off on 2026-11-21.

## Download URL caveat

The linked release currently contains **source only**. SignPath's eligibility rule says the project must already be released "in the form that should be signed." If they require downloadable installers, unsigned binaries would have to be attached to a release first — that reverses the earlier decision not to release binaries, so do it only with explicit approval.

## Post-acceptance checklist

1. Enable MFA on the GitHub account and on every account with write access. Status could not be confirmed through `gh api user` (the field was absent), so verify at <https://github.com/settings/security>.
2. Install the [SignPath GitHub App](https://github.com/apps/signpath) on `Fadlelmula/bergag` and add the predefined **GitHub.com** trusted build system to the SignPath organization.
3. In SignPath, create the Bergag project, link the repository, and add the `installers` artifact configuration from `.signpath/artifact-configurations/installers.xml`.
4. Create a signing policy using the SignPath Foundation certificate: named approver `Fadlelmula`; origin verification restricted to `.github/workflows/release-build.yml` on `refs/tags/v*`.
5. Add repository secret `SIGNPATH_API_TOKEN` and variables `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG`, `SIGNPATH_SIGNING_POLICY_SLUG`.
6. Enable branch protection on `main` (required review, no force pushes). Confirmed **not enabled** on 2026-11-21.
7. Push a `v*` tag to trigger the workflow, review the signing request in SignPath, approve it manually, confirm `Get-AuthenticodeSignature` reports `Valid`, then publish signed assets in a GitHub Release.

## Related documents

- [Code signing setup](SIGNING.md)
- [Release checklist](RELEASE-CHECKLIST.md)
