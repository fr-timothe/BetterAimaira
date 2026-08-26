<div align="center">
  <img src="assets/logo.svg" height="200">
<h1>BetterAimaira</h1>
<p><strong>A local client that replaces the Aimaira student portal with an adaptive Tauri app.</strong>
<br>
<strong>Supporting <code>Windows</code>, <code>macOS</code>, <code>Linux</code>, <code>Android</code> and <code>iOS</code></strong></p>
<br>
<p>
<a href="https://github.com/fr-timothe/BetterAimaira/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/fr-timothe/BetterAimaira/release.yml?style=for-the-badge&label=Release&color=white&labelColor=black" alt="Release workflow"></a>
<a href="https://github.com/fr-timothe/BetterAimaira/releases"><img src="https://img.shields.io/github/v/release/fr-timothe/BetterAimaira?include_prereleases&style=for-the-badge&label=Latest&color=white&labelColor=black" alt="Latest release"></a>
<a href="https://github.com/fr-timothe/BetterAimaira/releases"><img src="https://img.shields.io/github/downloads/fr-timothe/BetterAimaira/total?style=for-the-badge&label=Downloads&color=white&labelColor=black" alt="Downloads"></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/License-GPL--3.0-white?style=for-the-badge&labelColor=black" alt="License"></a>
</p>
<br>
<p>
  <a href="README.md">🇫🇷 Français</a> &bull;
  <a href="README.en.md"><strong>🇬🇧 English</strong></a>
</p>
<br>
<p><strong>Quick Links</strong></p>
<p>
<a href="#screenshots"><img src="https://img.shields.io/badge/Screenshots-000000?style=for-the-badge" alt=""></a>
<a href="#main-features"><img src="https://img.shields.io/badge/Main_Features-000000?style=for-the-badge" alt=""></a>
<a href="#download"><img src="https://img.shields.io/badge/Download-000000?style=for-the-badge" alt=""></a>
<a href="docs/README.md"><img src="https://img.shields.io/badge/Documentation-000000?style=for-the-badge" alt=""></a>
</p>
<p>
<a href="#development"><img src="https://img.shields.io/badge/Development-000000?style=for-the-badge" alt=""></a>
<a href="#privacy"><img src="https://img.shields.io/badge/Privacy-000000?style=for-the-badge" alt=""></a>
<a href="#tech-stack"><img src="https://img.shields.io/badge/Tech_Stack-000000?style=for-the-badge" alt=""></a>
<a href="#license"><img src="https://img.shields.io/badge/License-000000?style=for-the-badge" alt=""></a>
</p>

<p align="center">
  <img src="assets/showcase/betteraimaira-demo.webp" alt="BetterAimaira Animated Showcase" width="100%" style="border-radius: 12px; box-shadow: 0 20px 40px rgba(0,0,0,0.3);" />
</p>
<p align="center">
  <small>🎥 <a href="assets/showcase/betteraimaira-presentation.mp4">Download 1080p Presentation Video (MP4)</a></small>
</p>
</div>

## Introduction

Aimaira is a web intranet used by schools for schedules, grades, attendance and documents.
BetterAimaira is a native client for it: the student pastes any page of their own portal, the Rust
core normalizes it to an HTTPS origin, authenticates, and the interface shows what matters first —
the current or next course, its room, and how fresh the data is.

The app talks to the configured portal and to nothing else. No cloud relay sits in the middle,
credentials live in the operating system credential store, and every view says whether it is
loading, empty, stale, offline, expired or broken instead of guessing. This first vertical slice is
strictly read-only: administrative, billing and remote write actions are out of scope.

## Screenshots

<div align="center">
 <table>
  <tr>
   <td align="center"><strong>Today — Live Course & Metrics</strong></td>
   <td align="center"><strong>Schedule — Day View</strong></td>
  </tr>
  <tr>
   <td><img src="assets/showcase/screenshot-1.png" width="100%"></td>
   <td><img src="assets/showcase/screenshot-2.png" width="100%"></td>
  </tr>
  <tr>
   <td align="center"><strong>Schedule — Week Grid</strong></td>
   <td align="center"><strong>Grades & Evaluations</strong></td>
  </tr>
  <tr>
   <td><img src="assets/showcase/screenshot-3.png" width="100%"></td>
   <td><img src="assets/showcase/screenshot-4.png" width="100%"></td>
  </tr>
 </table>
</div>

## Main Features

<details>
  <summary>Session and portal</summary>

- `School picker` before the form (129 schools running Aimaira, searchable by name, initials or group, the portal address is filled in on selection)
- `Manual address entry` always available, for a portal the list does not carry
- `Portal URL Normalization` (any pasted deep link reduces to its HTTPS origin)
- `HTTPS Only` (an HTTP portal is refused before credentials are sent)
- `Form Authentication In Rust` (anti-forgery token, private cookie jar)
- `Optional Password Persistence` (Windows Credential Manager, Keychain, Keystore, Secret Service)
- `Automatic Session Restore` on launch
- `First-run introduction` (a one-page tour, then the rights the platform wants granted)
- `Stable Error Codes` (translated in the interface, never raw diagnostics)
- `French And English` interface, complete from the first milestone
</details>

<details>
  <summary>Schedule and planning</summary>

- `Monday-Anchored Week` (matches the portal 7-day request window)
- `Current Or Next Course` with room, instructor and portal note
- `Countdown And Progress` for the ongoing course
- `Day Picker` on compact windows, `Week Grid` once the window fits it
- `Portal Planning Settings` (`urlTempoSeance`, `tempoLinkVisible`, `sundaysVisible`)
- `Tempo Session Link`, only when the portal reports it as visible
- `Sanitized Portal Text` (portal HTML becomes plain text before it leaves Rust)
</details>

<details>
  <summary>Grades, attendance and documents</summary>

- `Grades` (read-only `/Note`, semantic Rust adapter)
- `Launch Grade Sync` (SQLite fingerprints, silent first baseline)
- `In-App Grade Alerts` (home banner and notification drawer, no push service)
- `Attendance` (read-only `/Absence`)
- `Profile` (read-only `/Profil`)
- `Documents` (read-only `/Document`)
- `Questionnaires` (read-only, including response detail)
- `Safe PDF Download` (same-origin allowlist, PDF signature check, 25 MiB limit)
</details>

<details>
  <summary>Interface</summary>

- `Adaptive Layout` driven by window width, not device name
- `Five Destinations` (Today, Schedule, Grades, Attendance, More)
- `Floating Bottom Bar` on compact windows, `Drawer And Top Bar` on desktop
- `Honest State` (loading, empty, error, expired, offline, stale — never a lie)
- `Freshness Labels` on every cached surface
- `44px Interactive Floor`, visible focus, full keyboard navigation
- `Reduced Motion` and `Safe Area` support
- `Frameless Custom Titlebar` on desktop
</details>

<details>
  <summary>Updates and delivery</summary>

- `Single Update Feed` (one GitHub release read by every platform)
- `Signed In-Place Install` on desktop (minisign-verified, NSIS passive)
- `PackageInstaller Handover` on Android
- `AltStore Source Check` on iOS
- `Installed Version Readout` in the Profile tab of the More view
</details>

<details>
  <summary>Planned — not implemented yet</summary>

- `Dark Mode` (tokens exist, the slice ships light only)
- `Grade Analytics` (trend curves, class distribution, grade simulator)
- `Attendance Analytics` (radial quota and ECTS indicators)
- `Campus Directory` (faculty and student lists)
- `iCal Export` (`.ics` generation and local subscription server)
- `Webhook Alerts` (Discord, Telegram)
- `Widgets And Tray` (Android/iOS home screen, Windows tray, macOS menu bar)
- `Biometric Unlock` (Face ID, Touch ID, Windows Hello)
</details>

The planned group is specified in [docs/INTEGRATIONS.md](docs/INTEGRATIONS.md).

## Download

| Platform | Asset | Install |
|---|---|---|
| Windows | `BetterAimaira-v<version>-x86_64.exe` | NSIS installer, updates itself in place |
| macOS, Linux | `.dmg`, `.app`, `.AppImage`, `.deb` | built locally for now, same updater path |
| Android | `BetterAimaira-v<version>-universal.apk` | `arm64-v8a`, `armeabi-v7a` and `x86_64`, then the system install prompt |
| iOS | `BetterAimaira-v<version>-arm64.ipa` | AltStore/SideStore source, published manually |

One naming scheme throughout, `BetterAimaira-v<version>-<architecture>.<extension>`: the platform is
carried by the extension.

<a href="https://betteraimaira.montfrond.work/en/download"><img src="https://img.shields.io/badge/Download-Latest_Release-000000?style=for-the-badge" alt="Download latest release"></a>

Every platform reads the same release feed, and an installed build checks it three seconds after
launch.

[betteraimaira.montfrond.work/en/download](https://betteraimaira.montfrond.work/en/download) detects
the device, resolves the newest published release and carries the install steps platform by platform.

## Documentation

| Document | Covers |
|---|---|
| [Architecture](docs/ARCHITECTURE.md) | Rust backend, Tauri IPC, cache strategy, security boundaries |
| [Application structure and platforms](docs/APP_STRUCTURE_AND_PLATFORMS.md) | Adaptive layout rules, platform expression, release matrix |
| [Backend API](docs/BACKEND_API.md) | Tauri commands, serialized contracts, document downloads, error codes |
| [Design system](docs/DESIGN_SYSTEM.md) | Tokens, breakpoints, responsive layouts |
| [Design guidelines](DESIGN.md) | How the tokens are applied, shared primitives, honest state |
| [Integrations](docs/INTEGRATIONS.md) | iCal feeds, webhooks, widgets, in-app alerts |
| [Performance](docs/PERFORMANCE.md) | Baseline commands, profiling tools, bundle budgets |
| [Product](PRODUCT.md) | Users, scope, constraints, product principles |
| [Brand assets](assets/README.md) | Logo files, geometry, palette, Svelte component |
| [School directory](assets/schools/README.md) | The schools running Aimaira, portal addresses, logos, how to update the list |
| [Showcase site](site/README.md) | The Astro project in `site/`, bilingual content, `gh-pages` deploy |

## Showcase site

The public site lives in [`site/`](site/README.md): a landing page, a download page and a compatibility
page, in French and English, served from the root of
**[betteraimaira.montfrond.work](https://betteraimaira.montfrond.work)**. It is a standalone Astro
project with its own dependencies.

It also serves the logos the app draws in its school picker, under `/media/schools/`. That is why the
app does not bundle them: the list can grow without a release going out.

```bash
cd site
bun install
bun run dev      # http://localhost:4321
bun run build    # writes site/dist/
bun run check    # astro check
```

[`pages.yml`](.github/workflows/pages.yml) publishes it on every push to `master` that touches `site/`
or the showcase assets, by **committing into the `gh-pages` branch**. That branch also carries the
update feed installed builds poll, so the workflow preserves `CNAME`, `.nojekyll` and `updates/`, and
the Pages source must not be switched to GitHub Actions.

## Development

### Prerequisites

- [Bun](https://bun.sh/) >= 1.2
- [Rust and Cargo](https://rustup.rs/) >= 1.80
- [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/) for the targets you build

### Commands

```bash
# Clone and install
git clone https://github.com/fr-timothe/BetterAimaira.git
cd BetterAimaira
bun install

# Desktop (1280x800, frameless custom titlebar)
bun run desktop:dev

# Mobile preview on desktop (412x892 viewport, forced mobile mode)
bun run mobile:dev

# Native mobile, on an emulator or a device
bun run android:dev
bun run ios:dev
```

```bash
# Release bundle
bun run desktop:build

# Release bundle exported to dist-desktop/, target/ purged
bun run desktop:build:export

# Build artifacts (src-tauri/target, .svelte-kit, build)
bun run clean

# Intermediate compiler cache only, release bundles kept
bun run clean:cache
```

Pushing a `v*` tag runs [`.github/workflows/release.yml`](.github/workflows/release.yml), which
builds the Windows installer and the Android APK, writes both update manifests, publishes the
release, and commits the manifests to the `gh-pages` branch.

Android compares version codes, not version names: an APK whose code is not higher than the
installed one is not an update as far as the system is concerned. The code is derived from the
package version, never picked by hand:

```bash
# Writes bundle.android.versionCode into src-tauri/tauri.conf.json
bun run android:version-code

# Fails if the file is out of step, writes nothing
bun run android:version-code -- --check
```

The derivation is `major * 1_000_000 + minor * 10_000 + patch * 100 + prerelease`, where the
prerelease is its trailing number (`beta.5` → 5) and `99` for a final release: `0.1.1-beta.5`
gives `10105`, `0.1.1` gives `10199`. The release workflow re-derives it before building, so a
version bump that forgot it still ships a real update.

The app reads its update feed from <https://betteraimaira.montfrond.work>, served by GitHub Pages
off the `gh-pages` branch, one directory per channel:

| Channel | Manifest |
|---|---|
| Stable | <https://betteraimaira.montfrond.work/updates/stable/latest.json> |
| Beta | <https://betteraimaira.montfrond.work/updates/beta/latest.json> |

A build whose version carries a prerelease suffix follows the beta channel by default; the channel
can be changed in Settings. The custom domain lives in the `CNAME` file at the root of `gh-pages`:
deleting it cuts the update feed for every installed build.

### Verification

```bash
bun run check
bun run build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

`bun run dev` renders the interface in a browser but has no Rust side, so portal authentication and
the credential store are unavailable there. Use `bun run desktop:dev` or `bun run mobile:dev` to
exercise the real backend.

### Development automation bridge

`bun run desktop:dev` starts a local automation bridge (`tauri-plugin-mcp-bridge`) on
`127.0.0.1:9223` to drive and verify the running app against the portal during development. It is
excluded from release builds by the `dev-automation` Cargo feature and a `#[cfg(debug_assertions)]`
guard in `src-tauri/src/lib.rs`, and its capability lives in
`src-tauri/capabilities/dev/dev-mcp-bridge.json`.

Do not enable this bridge in distributed builds.

## Privacy

- **No cloud relay.** School data goes from the device to the portal the user configured, and nowhere
  else.
- **Usage counting on explicit consent, with no identifier.** Onboarding asks once whether the app
  may count its own usage; declining is the default and sends nothing, not even the refusal. The
  `distinct_id` is a UUID minted at launch and never persisted, so two runs cannot be correlated. The
  exhaustive list of events and of what they carry is in `src-tauri/src/analytics.rs`, project key
  included: it is a public write-only credential by design, it travels inside every client that
  reports, and the project it points at holds no school data.
- **No push service, no third-party analytics in the webview.** Grade checks run only while the app is
  open, and usage capture leaves from the Rust core, never from a script loaded next to student data.
- **Cookies stay in Rust.** The session jar is in memory and never crosses the IPC boundary.
- **Passwords go to the OS vault.** No plaintext password in SQLite or frontend preferences, and
  explicit sign-out clears the stored entry.
- **HTTPS is mandatory.** An HTTP portal is refused before credentials are sent.
- **Local cache only.** SQLite holds grade fingerprints, display data and unread alert state.
- **Read-only.** The client never writes to the portal.
- **Portal strings are untrusted.** They are rendered as plain text.

## Tech Stack

| Layer | Technologies |
|---|---|
| **Core and backend** | [Rust](https://www.rust-lang.org/), [Tauri 2.0](https://v2.tauri.app/), `reqwest` (cookie jar), `scraper` (HTML parsing), `rusqlite` (cache), `keyring` (OS vault) |
| **Frontend** | [Svelte 5](https://svelte.dev/) (runes), [TypeScript](https://www.typescriptlang.org/), [Tailwind CSS v4](https://tailwindcss.com/), [SvelteKit](https://svelte.dev/docs/kit) static adapter |
| **Theme and tokens** | [FL-Theme via tweakcn](https://tweakcn.com/r/themes/cmq57ht7w000204l2axo6ho9v), republished as Tailwind utilities by the `@theme inline` block in `src/app.css` — see [DESIGN.md](DESIGN.md) |
| **Typography** | [Inter](https://rsms.me/inter/) variable, self-hosted through `@fontsource-variable/inter` (a Tauri client has no CDN fallback) |
| **Icons and charts** | [Lucide Svelte](https://lucide.dev/), hand-written Svelte 5 SVG |
| **Internationalisation** | [Paraglide JS](https://inlang.com/m/gerre34r/library-inlang-paraglideJs) (French, English) |
| **Package manager** | [Bun](https://bun.sh/) |

## Contributing

Contributions are welcome. Before opening a pull request:

- Read [DESIGN.md](DESIGN.md): utilities at the element, tokens from `src/app.css`, no local colour
  or radius declarations.
- Keep every user-visible string in the Paraglide catalogue (`messages/`).
- Keep the read-only boundary. No new portal write route.
- Run the verification block above. `bun run check` and `cargo clippy -- -D warnings` must be clean.
- Commit messages in English, imperative, one short summary line.

[CONTRIBUTING.en.md](CONTRIBUTING.en.md) has the full set: the boundaries that do not move, what a
PR has to satisfy, and the commit conventions the release notes are built from. To report a bug or
ask a question instead: [SUPPORT.en.md](SUPPORT.en.md).

## License

[GPL-3.0](LICENSE). Aimaira is a third-party product; this project is an independent client and is
not affiliated with it.

The trademarks named here, and the school names and logos carried in `assets/schools/`, belong to
their owners and are not covered by the GPL-3.0. See [NOTICE.md](NOTICE.md).
