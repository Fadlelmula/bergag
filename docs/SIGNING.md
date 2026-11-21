# Code signing (SignPath Foundation)

## Current status

**No SignPath application has been submitted and no installer has been signed.** Local and CI builds are unsigned and reported honestly as such. This document records the prepared prerequisites and the remaining owner-only steps.

SignPath Foundation provides free code signing to qualifying open-source projects. The certificate belongs to SignPath Foundation, so Windows will show **SignPath Foundation** as the publisher, not an individual. SignPath signs only binaries it can trace back to a build performed by GitHub Actions on GitHub-hosted runners from this repository, and every signing request requires manual approval. SmartScreen reputation may still produce warnings for early releases; the shared certificate generally builds reputation faster than a brand-new personal certificate.

## What is prepared in this repository

- `LICENSE` — MIT, an OSI-approved license with no commercial dual-licensing.
- Public source repository: <https://github.com/Fadlelmula/bergag>.
- A [source-only release](https://github.com/Fadlelmula/bergag/releases/tag/v0.1.0-m9) providing a stable download URL for the application; no installer binaries are attached.
- [docs/SIGNPATH-APPLICATION-DRAFT.md](SIGNPATH-APPLICATION-DRAFT.md) — ready-to-paste application answers and a post-acceptance checklist.
- `README.md` — product description, build instructions, download location, privacy statement, and a Code signing policy section.
- `.github/workflows/release-build.yml` — builds the unsigned MSI and NSIS installers on a `windows-latest` runner, packs both into one ZIP, uploads that ZIP as the unsigned artifact, and submits it to SignPath only when the SignPath values are configured. It verifies `Get-AuthenticodeSignature` status before uploading signed artifacts. It does **not** publish a GitHub Release.
- `.signpath/artifact-configurations/installers.xml` — a `<zip-file>` artifact configuration with a required `version` parameter that Authenticode-signs both `Bergag_${version}_x64_en-US.msi` and `Bergag_${version}_x64-setup.exe` inside that ZIP. This matches the documented SignPath syntax. SignPath can generate a refined configuration from an uploaded artifact; use that if you also want to deep-sign the embedded `bergag.exe` inside the MSI.

## Owner-only steps (not performed)

1. Enable multi-factor authentication on the GitHub account and on all accounts with write access.
2. Apply at <https://signpath.org/apply>. Provide:
   - Project name: Bergag
   - Repository: <https://github.com/Fadlelmula/bergag>
   - License: MIT (`LICENSE` at repository root)
   - Download URL: the [v0.1.0-m9 source-only release](https://github.com/Fadlelmula/bergag/releases/tag/v0.1.0-m9), created 2026-11-21. If SignPath requires downloadable installers for the "released" criterion, attaching unsigned binaries is a separate decision that needs explicit approval.
   - Description: a local-first, account-free Windows reading application; installers are MSI and NSIS built by `release-build.yml` on GitHub-hosted runners. Ready-to-paste answers are in [docs/SIGNPATH-APPLICATION-DRAFT.md](SIGNPATH-APPLICATION-DRAFT.md).
3. After acceptance, sign in to <https://app.signpath.io>, enable MFA, install the [SignPath GitHub App](https://github.com/apps/signpath) for this repository, and add the predefined **GitHub.com** trusted build system to the organization.
4. In SignPath, create the project and link it to this repository; add the `installers` artifact configuration from `.signpath/artifact-configurations/installers.xml`; create a signing policy using the SignPath Foundation certificate with a named approver and origin verification restricted to `release-build.yml` on `refs/tags/v*`.
5. Add repository secrets and variables:
   - Secret: `SIGNPATH_API_TOKEN`
   - Variables: `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG`, `SIGNPATH_SIGNING_POLICY_SLUG`
6. Enforce branch protection on `main` (review required, no force pushes) to satisfy the OSS team-role requirements.
7. Review and manually approve each signing request. Only after approval and signature verification should a release be published.

## Trust and limitations

- Self-signed certificates are not a substitute: Windows still treats them as untrusted and SmartScreen still blocks public downloads.
- Signing removes the "Unknown publisher" state; it does not guarantee first-run SmartScreen reputation.
- EV certificates no longer skip SmartScreen reputation checks (changed in 2024), so paying for EV solely for that reason is not justified.
- Copyright of the Bergag source remains with Fadlelmula, as asserted in `LICENSE`. SignPath's signature verifies build provenance only; it does not transfer ownership. Windows shows SignPath Foundation as the signer because the certificate belongs to them, not because they wrote or own the software.
- No signing, release publication, or repository configuration change is made without explicit owner authorization.

## Related documents

- [Release checklist](RELEASE-CHECKLIST.md)
- [Security posture](../SECURITY.md)
- [Development commands](../DEVELOPMENT.md)
