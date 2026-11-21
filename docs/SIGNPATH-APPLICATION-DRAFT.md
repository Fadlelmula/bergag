# SignPath Foundation application answers

Source checked on 2026-11-21: <https://signpath.org/apply> (HubSpot form) and <https://signpath.org/terms>. The apply page embeds a HubSpot form, so its field list is not visible in the page HTML; the fields below come from a documented successful applicant's list plus SignPath's own eligibility terms.

The project owner reported submitting this application on 2026-11-21. No confirmation or reference number was recorded here, and SignPath has not responded. These answers are retained as the submitted record.

## Form fields

| Field              | Answer                                                                                                               |
| ------------------ | -------------------------------------------------------------------------------------------------------------------- |
| Project Name       | `Bergag`                                                                                                             |
| Repository URL     | `https://github.com/Fadlelmula/bergag`                                                                               |
| Homepage URL       | `https://github.com/Fadlelmula/bergag` (no separate website; the README is the project home)                         |
| Download URL       | `https://github.com/Fadlelmula/bergag/releases/tag/v0.1.0-m9`                                                        |
| Privacy Policy URL | `https://github.com/Fadlelmula/bergag/blob/main/README.md#code-signing-policy`                                       |
| Wikipedia URL      | leave empty — no Wikipedia article exists                                                                            |
| Tagline            | A local-first, account-free desktop information and reading platform for Windows, built on a private SQLite library. |
| Category           | Choose **Productivity** (or **Utilities** if Productivity is not listed)                                             |
| Maintainer Type    | `Individual`                                                                                                         |
| Build System       | `GitHub Actions`                                                                                                     |
| First Name         | `Fadlelmula` — confirm this matches your legal/personal name                                                         |
| Last Name          | Your surname; it is not present in the repository metadata, so fill this in yourself                                 |
| Email              | `work@fadlelmula.com` (the address in the signed-off commits)                                                        |
| License (if asked) | `MIT` — <https://github.com/Fadlelmula/bergag/blob/main/LICENSE>                                                     |

Tick the consent boxes for the SignPath Foundation Code of Conduct and for SignPath storing your application data.

## Required field: Description

> Bergag is a Windows desktop application that lets people collect web feeds and articles into a private, local reading library. It is built for readers who want to follow sources, preserve articles, organize research, and search their own archive without creating an account or depending on a cloud service. Privacy is the default: the app contacts the network only for sources the user adds or previews or for actions the user explicitly confirms, and optional AI features stay off until the user configures and requests them. The project's source is public under the MIT license.

## Required field: Reputation

> Bergag is a new project, so it has no download statistics, media coverage, blog posts, or community discussions to point to, and I will not claim any. What can be independently verified is the project's openness and release control: the complete source and history are public at https://github.com/Fadlelmula/bergag, the MIT license is at the repository root, the signed-off baseline is tagged and published at https://github.com/Fadlelmula/bergag/releases/tag/v0.1.0-m9, and the release build is defined in a public workflow at https://github.com/Fadlelmula/bergag/actions/workflows/release-build.yml. Scope, security posture, privacy limits, and release checks are documented in ROADMAP.md, SECURITY.md, README.md, and docs/RELEASE-CHECKLIST.md. GitHub activity is visible at https://github.com/Fadlelmula/bergag/activity and contributor history at https://github.com/Fadlelmula/bergag/graphs/contributors. The maintainer is the sole author and repository owner, @Fadlelmula. I understand this may fall short of the reputation expected for downloadable executables and am happy to provide any further evidence.

## Longer description (if the form offers a free-text field)

> Bergag is a Windows desktop application for collecting feeds and articles, organizing research with dashboards and saved searches, and keeping an app-managed local backup library. SQLite is the source of truth; the app has no account, telemetry, ads, or mandatory AI. Feeds are fetched only for subscriptions the user adds or previews, and AI is off until the user configures an endpoint and confirms each request. Releases are Windows x64 MSI and NSIS installers built from the public repository by `.github/workflows/release-build.yml` on GitHub-hosted runners. Users download from the GitHub Releases page. The project is MIT licensed and maintained by its sole author.

## Answers to likely follow-up questions

- **Is the project released?** Yes — a public GitHub Release exists at `https://github.com/Fadlelmula/bergag/releases/tag/v0.1.0-m9`, tagged from the signed-off baseline. Caveat: it currently contains source only, while SignPath's condition says the project must be released "in the form that should be signed." If the reviewer requires downloadable installers, the unsigned MSI/EXE would have to be attached — that reverses the earlier "no binaries released" decision and needs explicit approval.
- **Who owns the code?** Fadlelmula, under MIT. The team that develops and maintains the repository is also the team requesting signatures.
- **What is signed?** Our own builds only: `Bergag_0.1.0_x64_en-US.msi` and `Bergag_0.1.0_x64-setup.exe`, built from this repository.
- **Proprietary components?** None. Windows system libraries such as WebView2 are not bundled as product code.
- **Hacking tools?** No. Bergag validates public URLs and reports its own security posture; it does not scan the user's system for vulnerabilities or circumvent security measures.
- **System changes / uninstall?** Launch-on-sign-in and close-to-tray are explicit user settings. MSI and NSIS provide standard uninstall paths.
- **Privacy?** No telemetry or analytics; no data is transferred unless the user configures a feed, extraction, AI endpoint, or update check. The exact required sentence is published in the [Code signing policy](https://github.com/Fadlelmula/bergag/blob/main/README.md#code-signing-policy).

## Real risks to expect

- SignPath states that executable applications need "a certain verifiable reputation" and that acceptance is discretionary. A brand-new project with no download history may be declined even when every technical condition is met.
- All maintainers need MFA on GitHub and SignPath. GitHub MFA could not be confirmed through `gh api user` — verify manually.
- Branch protection on `main` is still not configured.

## After acceptance

See the checklist in [docs/SIGNING.md](SIGNING.md). In short: install the SignPath GitHub App, add the GitHub.com trusted build system, create the project and `installers` artifact configuration, configure the signing policy with manual approval, add the repository secret/variables, then push a `v*` tag and approve the request.
