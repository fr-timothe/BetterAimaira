# BetterAimaira design

<!-- impeccable:design-system -->

## Visual direction

BetterAimaira uses the approved FL-Theme as its semantic token base with a product-specific layout. The visual identity is its own.

`src/app.css` is the single source of truth for every token. A view that redeclares a colour, radius, shadow, duration or font size locally has drifted, not extended the system.

## Color

Base palette (FL-Theme):

- `--background`: cool near-white `oklch(0.968 0.0059 239.8195)`.
- `--foreground`, `--secondary`: ink blue `oklch(0.3098 0.0748 248.9089)`.
- `--primary`: cyan `oklch(0.7347 0.1447 228.9136)`.
- `--muted`: pale cyan `oklch(0.9352 0.0412 217.6616)`.
- `--border`: blue-gray `oklch(0.62 0.035 242)`.
- First vertical slice is light-only. Do not infer a dark scheme by inversion.

Two consequences decide which token to reach for:

- **`--primary` is a fill, never a line.** At L 0.735 it clears 4.5:1 against ink (`--primary-foreground`) but only 2.28:1 against white. It may fill a shape that carries ink text; it must never be a thin stroke, an icon, or text that means something.
- **`--primary-deep` is the accent that carries weight.** Accent text on card (6.6:1), on `--muted` (5.5:1), and white on `--primary-deep` as a filled button (6.6:1). Every meaningful cyan stroke or label uses this token.

Derived roles: `--primary-soft`, `--primary-deep-hover`, `--secondary-hover`, `--secondary-foreground` (white on ink), `--surface-sunken` (nested panels), `--surface-overlay` (scrims), `--border-subtle` (1px dividers on white), `--muted-strong`.

Status tones come in threes — `--success` / `--success-strong` / `--success-surface`, and the same shape for `warning` and `danger`. `*-strong` is the text and icon tone, the plain token is the fill, `*-surface` is the tinted background. **No state is ever communicated by colour alone**; the label says it too.

Course categories use `--category-{lecture,tutorial,lab,exam,project,other}-{surface,text}`, applied only through `KindBadge`. Every pair clears 4.5:1 on its own surface and on white.

## Typography

Inter ships with the app, self-hosted from `@fontsource-variable/inter` (latin + latin-ext, variable weight axis). A Tauri client has no CDN fallback, and the variable axis is what makes the weight steps real under `font-synthesis: none`.

Scale: `--text-2xs` `0.6875rem` through `--text-4xl` `2.25rem`. Weights: `--weight-normal` 400, `--weight-medium` 500, `--weight-semibold` 600, `--weight-bold` 700, `--weight-heavy` 800. A value between two steps is drift — pick a step.

Headings use heavy weight and compact line height; body copy remains regular and bounded. Tracking stays at `0` except display sizes, which take `-0.02em`. Any table, clock, grade or countdown carries `font-variant-numeric: tabular-nums` so digits do not jump between ticks.

## Shape and depth

Radius: `--radius-xs` `0.375rem`, `--radius-sm` `0.5rem`, `--radius-md` `0.75rem` (FL-Theme's card radius), `--radius-lg` `1rem`, `--radius-xl` `1.25rem`, `--radius-pill` for small controls only.

Elevation: `--shadow-xs` through `--shadow-xl`, each with an offset and a soft blur. **Declare elevation once — a border or a shadow, never both.** A 1px border under a wide soft shadow is a ghost card. Structure comes from colour fields and borders; a shadow is reserved for surfaces that genuinely float (sheets, the ink hero, interactive hover).

Layer order is a scale, not a guess: `--z-raised` 10, `--z-sticky` 20, `--z-nav` 30, `--z-sidebar` 40, `--z-overlay` 100, `--z-drawer` 110, `--z-modal` 120, `--z-titlebar` 300. The titlebar owns the top because it carries the window's own close button — no in-app surface may cover it.

## Shared primitives

`src/lib/components/ui/` owns every pattern that appears more than once. A view that hand-rolls one of these has drifted:

| Primitive | Owns |
|---|---|
| `Button` | primary / ink / accent / ghost / outline / danger, three sizes, loading state |
| `IconButton` | icon-only controls, 44px guaranteed, required `label`, loading state |
| `Spinner` | the single spin animation |
| `Skeleton` | text / title / block / circle placeholders, one pulse grammar |
| `StateCard` | loading, empty, error and expired states with their action |
| `Card` | plain / sunken / ink surfaces, optional interactive hover |
| `PageShell` | the root padding recipe every view shares |
| `SectionHeader` | icon plate + title + subtitle + actions |
| `SegmentedControl` | tab bars, including arrow/Home/End keyboard support |
| `Sheet` | modals and drawers, with focus trap, focus restore, Escape and scroll lock |
| `Badge` | status tones with optional live dot |
| `KindBadge` | course category tone, resolved from `courseCategory()` |
| `FreshnessLabel` | sync state: fresh, stale, refreshing, failed, offline, never |

Shared keyframes live in `app.css`: `spin`, `pulse-soft`, `pulse-beacon`, `fade-in`, `slide-up-in`, `shimmer`. A local `@keyframes` of the same name gets scoped by Svelte and shadows the shared one — so components must not redeclare them.

## Authentication surface

- Expanded layout pairs a left ink-blue schedule signal with the login form.
- Compact layout removes the visual schedule but retains brand, language choice, and security guarantees.
- Portal address is empty by default. Supporting copy explains that any portal page can be pasted and normalized.
- The signal panel is abstract geometry plus the real clock. It states no course, room or name it does not have.
- Errors state the problem and recovery path. Loading, success, and credential-store warning states remain in the form region.

## Authenticated surfaces

- Compact windows and expanded desktop windows share the mobile-first structural baseline. They use a sticky top app bar with a menu trigger, brand badge, active view indicator, notification indicator, and user profile pill. On mobile screens (< 768px), a floating five-destination bottom bar provides quick navigation with safe-area padding; on desktop windows (>= 768px), the bottom bar is hidden while the drawer and top bar handle navigation.
- Today exposes sync freshness first, then displays the current or next course in an ink-blue container. Time, location text, portal note, progress, and available Tempo action stay together.
- Schedule uses a touch-scrollable day picker and daily list on compact and medium windows, then presents the portal six- or seven-day week when the window can display it without clipping.
- More groups Profile, Documents, and Questionnaires as accessible tabs. Profile owns the language selector and sign-out action. Sign-out requires confirmation in a modal.
- Grades, Attendance, Profile, Documents, and Questionnaires render semantic data returned by Rust without inferring values from localized portal strings. Questionnaire responses remain read-only.
- Expanded desktop windows maintain the centered layout with a minimum window size of `680x580`, using a slide-out drawer for profile, navigation, preferences, and quick actions.
- Aimaira strings are treated as untrusted content and displayed only as plain text.

## Honest state

This is a product promise, not a polish item. Every data surface distinguishes six states and none of them lies:

- **loading** — a skeleton or `StateCard kind="loading"`. Never an empty state while a request is in flight.
- **empty** — the portal returned nothing for this scope, and says so.
- **error** — the problem and the recovery path, with a retry that can actually succeed.
- **expired** — the session is gone; the action is sign-in, not retry.
- **offline** — the device has no network path. Read from `connectivity` (`$lib/state/connectivity.svelte`), and never report a device problem as a portal outage.
- **stale** — cached data is on screen. `FreshnessLabel` states when it was fetched.

A refresh that fails while data is already displayed must say so. Swallowing the failure and leaving stale data unmarked is the one behaviour this section exists to forbid.

## Interaction

- Minimum control target is `--tap-min` (`2.75rem` / 44px). This is a floor, including for icon-only and compact controls.
- Focus uses a high-contrast ink-blue outline. `outline: none` on a focusable element is only acceptable when a `:focus-visible` style replaces it in the same rule set.
- Any element with `onclick` is a `<button>`. A `div role="button"` must handle both `Enter` and `Space` with `preventDefault()` — so prefer the button.
- Icons are decorative unless they are the only label: `aria-hidden="true"` on the icon, the name on the control.
- Password visibility uses Lucide eye icons with translated accessible names.
- Locale controls expose pressed state.
- Motion is limited to loading feedback and short control transitions. `--press-scale` is the single tap-feedback value.

## Responsive rules

- Base styles target compact touch windows. `48rem` (768px) is the primary hinge and the only breakpoint a view should introduce without reason. Secondary steps, when a layout genuinely needs one, use `rem` — never a px literal beside a rem one in the same file.
- Compact and desktop views share navigation patterns to avoid layout duplication.
- Hover styling belongs inside `@media (hover: hover)` so it does not fire on tap.
- Security, no-cloud, and read-only statements stay visible at every size.
- `prefers-reduced-motion` is handled globally in `app.css` with `!important` on the universal selector; components must not redeclare it. JS-driven motion is **not** covered by that rule — `scrollIntoView({ behavior: 'smooth' })` and friends must check `matchMedia('(prefers-reduced-motion: reduce)')` themselves.

## Internationalisation

Every user-visible string goes through Paraglide (`import * as m from '$lib/paraglide/messages.js'`). A `locale === 'fr' ? … : …` ternary in markup is a defect: it hard-codes two languages into the component and bypasses the message catalogue. Dates and numbers are formatted with `Intl` and an explicit locale.

## Assets

- `static/favicon.svg` and `assets/logo.svg` carry the official brand mark: "The Slot Matrix" (Le Signal de Créneau).
- In-app brand mark is provided by the `$lib/assets/Logo.svelte` component.
- In-app feature icons come from Lucide Svelte. No generated raster ships in this surface.
