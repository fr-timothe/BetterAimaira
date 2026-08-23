[← Documentation index](README.md)

# Application structure and platform strategy

> Local architecture notes for BetterAimaira. No web version is planned.
> Sections marked **planned** describe work that has not shipped yet.

## 1. Decisions

1. Keep one Svelte 5 interface and one Rust domain core for all targets.
2. Adapt layout from available window space, not device name or operating system.
3. Adapt interaction from capabilities (`pointer`, `hover`, keyboard, touch), not width alone.
4. Keep BetterAimaira identity stable through semantic theme tokens. Apply platform variants around navigation, density, motion, surfaces, and native integrations.
5. Use native Swift or Kotlin code only when a WebView cannot provide correct platform behavior: widgets, biometrics, background tasks, share sheets, and dynamic colors.
6. Keep core features available at every size. Change presentation and information density, not information architecture.
7. Treat cached data freshness, estimates, errors, and external service dependencies as first-class interface information.
8. Build static frontend assets for Tauri only. Do not deploy or support a browser-hosted version.

## 2. Product rules

Three practical product rules drive every screen:

- **Inform with minimum actions.** Next room, current or next course, average, absence balance, and urgent changes belong on the first screen. Secondary details remain grouped under their primary section.
- **Respect host platform expectations.** Navigation, safe areas, keyboard behavior, touch feedback, motion, dialogs, and native services must feel familiar on each platform.
- **Stay honest.** Estimated averages need an explicit label, stale data needs a last-sync timestamp, and offline mode or failed integrations must be visible near affected items.

Svelte patterns use the project's own primitives in `src/lib/components/ui/`, Tailwind CSS v4, Lucide Svelte, and semantic theme tokens. `components.json` keeps the shadcn-svelte CLI aliases usable, but no shadcn or Bits UI package is installed: every primitive is hand-written against the tokens.

Shared layout primitives:

| Pattern | Rule |
|---|---|
| `PageLayout` | Owns scroll region, page header, safe-area padding, and content width |
| `SectionList` | Groups settings or data lists into named sections |
| `DataList` + `DataItem` | Consistent leading icon, title, caption, status, and trailing action |
| Semantic HTML + theme classes | Preserves heading hierarchy and platform text scaling |
| Lucide wrapper or icon slot | Keeps size, color, and accessible label consistent |

Add an application-level pattern only after it repeats across features or manages real behavior.

## 3. Product information architecture

Primary destinations remain identical on every platform:

1. **Today.** Current or next class, room, countdown, urgent changes, and key metrics.
2. **Schedule.** Day and week navigation, course details, refresh, and calendar export.
3. **Grades.** Latest grades, subject trends, distribution, and simulator.
4. **Attendance.** Balance, status, absence history, and justification records.
5. **More.** Documents, directory, integrations, profile, settings, and diagnostics.

Today, Schedule, Grades, and Attendance form the persistent primary navigation. More contains lower-frequency destinations. Deep links resolve to the same routes on all targets.

## 4. Frontend structure

Target structure:

```text
src/
├── lib/
│   ├── components/
│   │   ├── ui/                 # own primitives, written against the tokens
│   │   ├── shell/              # AppShell, navigation, title bar, safe areas
│   │   └── patterns/           # PageLayout, SectionList, DataItem, status UI
│   ├── features/
│   │   ├── today/
│   │   ├── schedule/
│   │   ├── grades/
│   │   ├── attendance/
│   │   ├── documents/
│   │   └── settings/
│   ├── platform/
│   │   ├── capabilities.ts     # Typed feature availability
│   │   ├── profile.svelte.ts   # OS, window class, input, theme preferences
│   │   └── native.ts           # Native-service facade
│   ├── services/
│   │   ├── gateway.ts          # Typed Tauri IPC boundary
│   │   └── events.ts           # Backend event subscriptions
│   ├── state/                  # Session and cross-feature state only
│   ├── types/                  # Shared frontend contracts
│   └── utils/
├── routes/
│   ├── +layout.svelte          # Providers and root shell
│   ├── +layout.ts              # Static rendering configuration
│   ├── (auth)/
│   └── (app)/
│       ├── today/
│       ├── schedule/
│       ├── grades/
│       ├── attendance/
│       └── more/
├── app.css                     # Tokens, reset, global platform variants
└── app.html                    # Viewport and safe-area metadata
```

Rules:

- Route files compose features. Domain behavior does not live in routes.
- Components do not call `invoke` directly. They call typed feature services through `gateway.ts`.
- Keep transient UI state local to the component or feature. Keep only session, sync status, and shared entities in global rune state.
- Split stores by feature instead of maintaining one large global store.
- Prefer derived runes for calculated view state. Effects handle external synchronization only.
- Keep serialization contracts stable and explicit. Rust validates domain logic.
- SvelteKit serves as router and build tool. `@sveltejs/adapter-static` emits assets embedded by Tauri; SSR and browser deployment are out of scope.

## 5. Rust and native structure

Target structure:

```text
src-tauri/
├── src/
│   ├── domain/                 # Models and pure business rules
│   ├── application/            # Use cases and service interfaces
│   ├── infrastructure/
│   │   ├── aimaira/            # HTTP, auth, scraping, portal adapters
│   │   ├── storage/            # SQLite repositories and migrations
│   │   └── credentials/        # Secure credential-store adapter
│   ├── commands/               # Thin Tauri IPC adapters
│   ├── platform/               # Desktop and mobile platform selection
│   └── lib.rs                  # Plugins, state, command registration
├── capabilities/               # Least-privilege permissions by target
└── plugins/                    # Custom native plugin only when required
    └── platform-services/
        ├── android/            # Kotlin
        ├── ios/                # Swift
        └── src/                # Shared Rust plugin API
```

Dependency flow:

```text
Svelte view -> feature service -> typed IPC -> command -> use case -> repository/adapter
                                              |
                                              +-> platform service
```

Rules:

- Commands deserialize, authorize, call one use case, and serialize results. No scraping or direct SQL inside commands.
- Shared HTTP, parsing, diffing, calculations, and cache logic stay in Rust.
- SQLite database and migrations remain shared. Platform code supplies the application data path and lifecycle hooks.
- Prefer official Tauri plugins. Add custom Kotlin/Swift bridges only for missing platform features.
- Use separate capabilities for desktop and mobile targets. Do not grant broad permissions across all platforms for single-target features.
- Expose capability checks to the frontend so unsupported actions are hidden or explained rather than failing on click.

## 6. Adaptive layout system

### 6.1 Window classes

Layout uses CSS viewport width, which matches available Tauri window space. Do not infer layout from operating system names or hardware models.

| Class | Width | Navigation | Main composition |
|---|---:|---|---|
| `compact` | `< 640px` | Bottom navigation | One column, focused detail, day schedule |
| `medium` | `640-1023px` | Navigation rail | Two panes where useful, 2-3 day schedule |
| `expanded` | `>= 1024px` | Full sidebar | Multi-column dashboard, week schedule, persistent detail |

Below `600px` height: use compact headers and scrollable dialogs, avoid vertically centered forms. Above `1440px` width: keep the shell full-width but cap reading and data canvases around `1440px`.

### 6.2 Shell behavior

**Compact**

- Bottom navigation has five destinations maximum and includes safe-area padding.
- One primary scroll container per route.
- Details open as full-screen routes or bottom sheets based on task depth.
- Schedule defaults to one day. Horizontal swipe can change the day, while visible buttons and keyboard actions remain available.
- Tables recompose into labeled rows or cards to avoid horizontal scrolling.

**Medium**

- Navigation rail uses icons and labels when width allows.
- Master-detail composition for course lists, documents, and settings.
- Dialogs render as sheets or bounded modals based on content size.
- Touch targets remain at least 44px for tablet touch screens.

**Expanded**

- Sidebar remains visible with a default width of 240px.
- Dashboard exposes key information directly.
- Weekly schedule and comparison tables use available width.
- Keyboard shortcuts, context menus, and hover details enhance navigation without replacing primary click paths.

### 6.3 Component responsiveness

- Page shell uses media queries because navigation depends on window width.
- Reusable panels use container queries because local width matters more than viewport width.
- Use CSS Grid and Flexbox reflow. Avoid JavaScript layout calculations when CSS handles the layout.
- Maintain logical DOM order for screen readers when visual grid order changes.
- Use `clamp()` for spacing and panel width, not for viewport-scaled font sizes.
- Respect `env(safe-area-inset-*)` on all four edges with `viewport-fit=cover`.
- Account for on-screen keyboards through visual viewport insets so focused fields and submit actions stay visible.
- Support portrait, landscape, split-screen, and window resizing without locking orientation.

### 6.4 Input and accessibility

- Detect `hover`, `pointer`, keyboard focus, and reduced motion independently from width.
- Minimum touch target: 44x44 CSS px on iOS surfaces; 48x48 on Android surfaces where spacing allows.
- Hover states never hide the only access path to information or actions.
- Icon controls receive accessible labels and tooltips.
- Maintain visible `:focus-visible`, logical tab order, Escape behavior, and platform modifier mapping (`Cmd` on macOS, `Ctrl` elsewhere).
- Support text zoom without clipping. Information status cannot rely on color alone.

## 7. Platform expression

Brand tokens remain common: semantic colors, data-series colors, spacing scale, information hierarchy, and core icon vocabulary. Platform context sets a root attribute such as `data-platform="ios"`; components consume semantic variants instead of branching throughout templates.

| Target | Shared Svelte treatment | Native integration boundary |
|---|---|---|
| Windows | Fluent-compatible density, keyboard/focus navigation, standard system title bar, restrained acrylic when supported | Notifications, updater, tray, secure store, Windows Hello |
| macOS | System typography, sidebar and toolbars, menu conventions, `Cmd` shortcuts, window control clearance | Keychain, notifications, menu bar, updater, sharing |
| Linux | Clear GTK-compatible density, standard decorations, avoids blur and transparency assumptions | Secret service/keyring, notifications, system tray |
| Android | Material navigation bar/rail, ripple feedback, predictive-back route handling | Keystore/biometrics, WorkManager, share sheet, notifications, haptics |
| iOS | Tab/sidebar adaptation, safe areas, smooth transitions, edge-swipe back navigation | Keychain/Face ID/Touch ID, BackgroundTasks, share sheet, notifications, haptics, widgets |

The main interface is HTML and CSS inside a WebView, not native SwiftUI or Jetpack Compose controls.

Theme strategy:

- `system`, `light`, and `dark` modes.
- Semantic tokens (`background`, `surface`, `text`, `muted`, `border`, `primary`, `danger`, data colors), avoiding hard-coded platform colors inside feature components.
- System font stack by default.
- Android dynamic color and Windows accent colors are optional and require contrast validation.
- `prefers-reduced-motion` disables non-essential animations.

## 8. Native service facade

Frontend consumes one interface:

```typescript
type PlatformCapabilities = {
  biometrics: boolean;
  backgroundSync: "none" | "opportunistic" | "scheduled";
  calendarExport: "file" | "share" | "subscription";
  haptics: boolean;
  nativeShare: boolean;
  notifications: boolean;
  widgets: boolean;
};
```

Feature code checks capabilities before calling the facade, without importing platform-specific modules.

Native service priorities:

1. Secure credentials and local database paths.
2. Network lifecycle detection and foreground refresh.
3. Notifications and deep-link routing.
4. Native sharing and calendar export.
5. Biometrics for local unlocking.
6. Background scheduling.
7. Widgets, system tray, and menu bar extensions.

### Background execution

- Desktop can refresh while the process runs; system tray mode is optional and user-controlled.
- Android periodic work uses WorkManager and is scheduled by the operating system.
- iOS BackgroundTasks execution is opportunistic and scheduled by iOS.
- Fixed 15-minute background refresh intervals cannot be guaranteed on mobile. The app refreshes stale data on launch or resume, displays the last sync time, and schedules notifications from known schedule data.
- Widgets consume a shared snapshot written by the core app without scraping Aimaira independently.

## 9. Data and offline flow

Startup sequence:

1. Open local database and load session metadata.
2. Render cached Today data immediately.
3. Display freshness and offline status without blocking content.
4. Revalidate in the background when network and session allow.
5. Diff remote results in Rust, persist transactionally, and emit a typed update event.
6. Update visible feature stores and schedule supported notifications or widget snapshots.

Data results carry status metadata:

```text
data + fetchedAt + source + freshness + isEstimated + warnings
```

## 10. Cross-platform build and release

Platform build requirements:

- Windows artifacts build on Windows.
- macOS and iOS artifacts require macOS and Xcode.
- Linux artifacts build on Linux with WebKitGTK dependencies.
- Android requires Android SDK, NDK, JDK, and Rust Android targets.
- Store releases require signing identities and protected CI secrets.

CI stages:

1. **Shared checks:** `bun install --frozen-lockfile`, `bun run check`, `bun run build`.
2. **Rust checks:** formatting, linting, and tests on Windows, macOS, and Linux.
3. **Desktop package matrix:** Windows x64, macOS Apple Silicon and Intel, Linux x64.
4. **Mobile package matrix:** Android APK/AAB; iOS simulator build and signed release archive.
5. **Release:** signing, notarization, checksum generation, and staged rollouts.

Update delivery, implemented in `.github/workflows/release.yml`:

- Desktop reads a minisign-signed `latest.json` and installs in place.
- Android reads the same manifest and hands the APK to `PackageInstaller`.
- iOS reads an AltStore source; AltStore or SideStore performs the install.

## 11. Development workflows and flavor configurations

BetterAimaira provides dedicated development targets to test both desktop and mobile behaviors rapidly:

### 11.1 Desktop development
- **Configuration:** `src-tauri/tauri.conf.json` (merged with platform overrides `tauri.windows.conf.json`, `tauri.macos.conf.json`, `tauri.linux.conf.json`).
- **Window:** Frameless `1280x800` (min `680x580`), `decorations: false`, custom `TitleBar.svelte` handling OS window dragging and traffic lights / window controls.
- **CSS Mode:** `html.desktop-app`, `--titlebar-height: 36px`.
- **Command:** `bun run desktop:dev` (or `bun run dev:desktop`).

### 11.2 Mobile simulation on desktop
- **Configuration:** `src-tauri/tauri.mobile-dev.conf.json`.
- **Window:** Standard smartphone viewport `412x892` (min `360x580`), `decorations: false` (frameless mobile viewport without desktop window title or drag bar).
- **URL & Mode:** `http://localhost:1420?mobile` automatically activates `html.mobile-app` mode, removes desktop titlebar (`--titlebar-height: 0px`), hides the desktop burger button, and displays the persistent bottom navigation bar.
- **Command:** `bun run mobile:dev` (or `bun run dev:mobile`).

### 11.3 Native mobile development
- **Android:** `bun run android:dev` (runs on connected Android device or emulator).
- **iOS:** `bun run ios:dev` (runs on connected iOS device or Xcode simulator).

## 12. Verification matrix

### Automated checks

- Svelte type checking and static build on each change (`bun run check`, `bun run build`).
- Rust unit tests for parsers, calculations, diffs, migrations, and capability mappings.
- Contract tests for IPC payloads and error shapes.
- Component tests for loading, stale, offline, empty, and error states.
- Visual checks across compact (`360x800`), medium (`820x1180`), and desktop (`1440x900`) viewports.
- Keyboard navigation and reduced-motion checks.

### Real devices

Before releasing feature-complete builds, verify:

- Windows 11 with keyboard, mouse, and resized narrow windows.
- macOS on Apple Silicon.
- Linux under Wayland and X11.
- Physical iOS devices across supported screen sizes.
- Physical Android devices across supported screen sizes.
- Tablet or split-screen sessions on both mobile platforms.

## 13. Delivery phases

### Phase 1: Portable foundation

- Static Tauri-only SvelteKit setup.
- Shared contracts, IPC gateway, app shell, window classes, and safe areas.
- Today, Schedule, Grades, Attendance, and Profile views.
- SQLite cache, foreground sync, and freshness indicators.
- Light, dark, and system themes with accessibility baseline.

### Phase 2: Platform quality

- Desktop keyboard navigation, menus, tray, and updater.
- Mobile back navigation, share sheets, haptics, and notification permissions.
- Secure credential stores and biometric local unlocking.
- Platform-specific theme adjustments.

### Phase 3: Extensions

- Android and iOS widgets with desktop menu/tray summaries.
- Opportunistic background sync.
- Store packaging, signing, and staged updates.
- Optional dynamic system accent colors.

## 14. Non-goals and guardrails

- No browser deployment, PWA, or browser fallbacks.
- No separate mobile and desktop frontend codebases.
- No platform checks scattered across feature components.
- No dependence on hover, swipe, or exact background timers for core workflows.
- No hidden stale or estimated data status.
- No custom native plugins when official Tauri plugins exist.
- No widgets in initial releases to minimize platform-specific complexity.

## 15. Open decisions

- Minimum supported OS versions for Windows, macOS, Linux, Android, and iOS.
- Tablet optimization level for initial releases.
- Background tray behavior on desktop.
- Calendar export approach: direct `.ics` file sharing vs. local subscription server.
- Dark mode token fine-tuning following the light-mode vertical slice.

## 16. Sources

Official sources used during research:

- Tauri prerequisites and target build constraints: https://v2.tauri.app/start/prerequisites/
- Tauri mobile plugin architecture: https://v2.tauri.app/develop/plugins/develop-mobile/
- Tauri GitHub release pipeline: https://v2.tauri.app/distribute/pipelines/github/
- Apple layout and safe areas: https://developer.apple.com/design/human-interface-guidelines/layout
- Android window size classes: https://developer.android.com/develop/adaptive-apps/guides/use-window-size-classes
- Windows responsive breakpoints: https://learn.microsoft.com/en-us/windows/apps/design/layout/screen-sizes-and-breakpoints-for-responsive-design
