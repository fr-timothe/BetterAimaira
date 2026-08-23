# BetterAimaira

> Alternative client for Aimaira intranets.
> Built with Tauri 2.0 (Rust), Svelte 5, Tailwind CSS v4, and inspired by Papillon design principles.

---

## Targets and platforms

- **Desktop:** Windows (`.exe`, `.msi`), macOS (`.dmg`, `.app`), Linux (`.AppImage`, `.deb`)
- **Mobile:** Android (`.apk`, `.aab`), iOS (`.ipa` via Xcode)

---

## Scope and features

### Current implementation

- Adaptive login screen with French and English translations powered by Paraglide JS.
- Portal URL normalization: pasted deep links reduce to their HTTPS origin before authentication.
- Form authentication handled by Rust with a private cookie jar.
- Optional password persistence in the operating system credential store, with automatic re-authentication on application start.
- Monday-anchored weekly schedule through `/Calendar/LoadEvents`, matching the portal 7-day request window, with sanitized text and session-expiry handling.
- Planning settings (`urlTempoSeance`, `tempoLinkVisible`, `sundaysVisible`) read directly from the portal.
- Read-only Grades, Attendance, Profile, Documents, and Questionnaires views backed by semantic Rust adapters.
- Same-origin PDF downloads restricted to known absence, grade, and school document routes, with a 25 MiB limit.
- Adaptive Today, Schedule, Grades, Attendance, and More navigation backed by real Aimaira data.
- SQLite stores grade fingerprints and unread in-app alerts on launch. No background daemon, cloud relay, or push notifications are used.
- Read-only scope: administrative, billing, and remote write actions are excluded.

### 1. Schedule and planning
- Countdown to the next class, prominent room number, and instructor details.
- Visual timeline tracking current class progress.
- Day-based view on compact screens, multi-column grid on expanded windows.
- In-app alerts when newly published grades are detected on startup.
- iCal calendar export (`.ics` generation).

### 2. Grades and academic analytics
- Spline curves for grade trends across academic periods.
- Class distribution bars comparing student score with class average, minimum, and maximum.
- Subject-level trend indicators for course modules.
- Grade simulator calculating scores needed on upcoming exams to reach target averages.

### 3. Attendance and absence tracking
- Radial indicators for attendance percentage, remaining absence quota, and validated ECTS credits.
- Categorized absence records (Justified, Pending, Unjustified).

### 4. Campus directory
- Faculty directory with contact links.
- Student directory for group collaboration.
- Offline access for downloaded course documents.

### 5. Integrations
- Desktop tray and mobile widget support for upcoming class details.
- Webhook alerts for Discord and Telegram.
- Biometric authentication (Face ID, Touch ID, Windows Hello) for local unlocking.

---

## Tech stack

| Layer | Technologies |
|---|---|
| **Core and Backend** | [Rust](https://www.rust-lang.org/), [Tauri 2.0](https://v2.tauri.app/), `reqwest` (CookieJar), `scraper` (HTML DOM Parser), `rusqlite` (Cache), `keyring` (OS Vault) |
| **Frontend UI** | [Svelte 5](https://svelte.dev/) (Runes: `$state`, `$derived`), [TypeScript](https://www.typescriptlang.org/), [Tailwind CSS v4](https://tailwindcss.com/) |
| **Theme and Tokens** | `FL-Theme` via tweakcn (`primary: oklch(0.73 0.14 229)`, `radius: 0.75rem`), extended into the token system and shared primitives documented in [DESIGN.md](DESIGN.md) |
| **Typography** | [Inter](https://rsms.me/inter/) variable, self-hosted via `@fontsource-variable/inter` (latin + latin-ext) — a Tauri client has no CDN fallback |
| **Icons and Charts** | [Lucide Svelte](https://lucide.dev/), Svelte 5 SVG charts |
| **Package Manager** | [Bun](https://bun.sh/) |

---

## Documentation

- [Architecture Overview](docs/ARCHITECTURE.md): Rust backend, Tauri IPC, and data pipeline.
- [Application Structure and Platforms](docs/APP_STRUCTURE_AND_PLATFORMS.md): Adaptive layout rules, platform expression, and release matrix.
- [Design System and Guidelines](docs/DESIGN_SYSTEM.md): Tokens, chart specifications, and responsive layouts.
- [Integrations Specification](docs/INTEGRATIONS.md): iCal feeds, webhooks, widgets, and in-app alerts.
- [Rust Backend API](docs/BACKEND_API.md): Tauri commands, serialized contracts, document downloads, and error codes.
- [Performance](docs/PERFORMANCE.md): Baseline commands, platform profiling tools, bundle budgets, and safeguards.

---

## Getting started

### Prerequisites
- [Bun](https://bun.sh/) (>= 1.2)
- [Rust and Cargo](https://rustup.rs/) (>= 1.80)
- Node/TypeScript environment

### Development commands

```bash
# 1. Clone repository
git clone https://github.com/YourUser/BetterAimaira.git
cd BetterAimaira

# 2. Install dependencies
bun install

# 3. Start development
# Desktop version (1280x800, frameless custom titlebar)
bun run desktop:dev

# Mobile preview on desktop (forced mobile mode, 412x892 viewport, bottom navigation)
bun run mobile:dev

# 4. Native mobile development (emulator or physical device)
bun run android:dev
bun run ios:dev
```

### Build and cleanup

```bash
# Build desktop release bundle
bun run desktop:build

# Build desktop release, export bundle to dist-desktop/ and purge target/
bun run desktop:build:export

# Clean build artifacts (src-tauri/target, .svelte-kit, build)
bun run clean

# Clean intermediate compiler cache while preserving release bundles
bun run clean:cache
```

### Verification

```bash
bun run check
bun run build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

The browser development server (`bun run dev`) renders the interface but cannot perform real portal authentication. Run `bun run desktop:dev` or `bun run mobile:dev` to test Rust authentication and the system credential store.

### Development automation bridge

`bun run desktop:dev` starts a local automation bridge (`tauri-plugin-mcp-bridge`) on `127.0.0.1:9223` to drive and verify the running app against the portal during development. The bridge is excluded from release builds via the `dev-automation` Cargo feature and a `#[cfg(debug_assertions)]` guard in `src-tauri/src/lib.rs`. Its capability is configured in `src-tauri/capabilities/dev/dev-mcp-bridge.json`.

Do not enable this bridge in distributed builds.
