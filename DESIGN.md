# BetterAimaira design

<!-- impeccable:design-system -->

## Visual direction

BetterAimaira uses the approved FL-Theme as its semantic token base with a product-specific layout. The visual identity is its own.

`src/app.css` is the single source of truth for every token. A view that redeclares a colour, radius, shadow, duration or font size locally has drifted, not extended the system.

## How the tokens are applied

Tailwind v4 is the styling layer. `@theme inline` in `src/app.css` republishes every token as a utility, so the token names below are also the class names: `--primary-deep` is `text-primary-deep` and `bg-primary-deep`, `--radius-xl` is `rounded-xl`, `--text-md` is `text-md`, `--shadow-sm` is `shadow-sm`. Spacing needs no bridge — the `--space-*` scale is Tailwind's own, step for step, so `var(--space-3)` is `p-3`.

Three families have no Tailwind namespace and are read straight from the token: `min-h-(--tap-min)`, `active:scale-(--press-scale)`, and `duration-fast` / `z-nav` through the `--transition-duration-*` and `--z-index-*` aliases.

**Safe areas are spent through the `*-safe-*` utilities, never by hand.** `--safe-top` / `--safe-right` / `--safe-bottom` / `--safe-left` are what the platform takes — status bar, gesture pill, display cutout — and `app.css` fills them from the Android bridge or `env()`. What a layout writes is `pt-safe-8`, `pb-safe-2`, `mx-safe-3`, `top-safe-6`: each utility **adds** the inset of its edge to that step of the spacing scale. Padding, margin and the four inset properties all have one. The pattern they replaced, `pt-[max(2rem,var(--safe-top))]`, is a defect and not a shorthand: on any phone whose status bar is taller than the step, `max()` resolves to exactly the inset and the first line of text lands against the clock. Two gutters are tokens rather than steps — `px-screen` (`--gutter-screen`, the side margin of a full-frame screen) and `--dock-clearance` (the dock's height plus the strip under it, what a scroller owes the dock). Nothing outside `app.css` reads `env(safe-area-inset-*)`.

**Utilities first.** Scoped `<style>` is for what a utility cannot say, and the file that keeps one says why at the top of the block. As of this pass that is: the three portal tables, whose display model switches between a card stack and real columns; the presence gradient and the SVG internals in `HeroStat`; the brand mark's own SVG paint in `Logo`; the shell's layout-mode block, where root classes must beat a width query on specificity; and the two states that paint a whole course row on Today. The calendar's time grid keeps none: its columns, hour rules and blocks are placed by inline `grid-row` / `top` / `height` values computed in `calendar-layout.ts`, because those numbers are data, not style.

**Global CSS must be layered.** `@import "tailwindcss"` declares `@layer theme, base, components, utilities`, and unlayered CSS outranks every utility regardless of specificity. Anything global added to `app.css` goes inside `@layer base`, or it silently wins against the utility that was supposed to override it. The one deliberate exception is `.desktop-only` / `.mobile-only`, which needs to beat a co-located `display` and is commented as such.

**Three ordering traps, all real, all found by measuring:**

- A `leading-*` must follow the `text-*` it belongs to. tailwind-merge reads `text-lg` as the size-and-leading shorthand and drops an earlier `leading-*`.
- `scale-*` writes the `scale` property, not `transform`. A press animation needs `transition-transform` (which covers `scale`), not `transition-[transform]`.
- A child sized down inside a parent with a unitless `line-height` used to inherit that number and recompute it. A `text-*` utility sets its own line-height, so the child needs the leading spelled out.

`rounded` on its own resolves to Tailwind's deprecated 0.25rem, not this system's radius: always name the step. `shadow`, `shadow-2xs` and `shadow-2xl` are unset on purpose so reaching for an unaudited elevation fails the build.

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

Course categories use `--category-{lecture,tutorial,lab,exam,project,other}-{surface,text}`, applied only through `category-tone.ts`. Every pair clears 4.5:1 on its own surface and on white, and contrast is symmetric, so the `text` tone also carries white when it is used as a ground.

That module is the rule's new home: it used to say "only through `KindBadge`", which held while a badge was the only thing wearing a category. A course block on the time grid wears the same category as a filled, positioned button with children, and it cannot be a badge — so the table moved down a level rather than being copied. `KindBadge` reads it, the calendar block reads it, nothing else spells a `--category-*` class. It also owns `categoryCode()`, the two- or three-character label a mark too small for a name still prints, because a category carried by hue alone is a state communicated by colour alone.

A course that is already over drops its category for `spentSurface` — the neutral pair — instead of being faded. `opacity` fades the label with the field, and an 11px label at 64% on a pale ground falls under the floor. The swap also earns something: nobody scans the past by category, so the palette ends up spent entirely on what is still coming.

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
| `PageShell` | the root padding recipe every view shares inside the authenticated shell |
| `ScreenShell` | the full-frame screens outside it — startup, school picker, introduction: own scroller, all four system insets, and the keyboard strip |
| `SectionHeader` | icon plate + title + subtitle + actions |
| `SegmentedControl` | tab bars, including arrow/Home/End keyboard support |
| `Switch` | on/off preferences, `role="switch"` with `aria-checked`, 44px target, busy state while the answer is being written |
| `Sheet` | modals and drawers, with focus trap, focus restore, Escape, scroll lock and a panel that rises above the on-screen keyboard |
| `Badge` | status tones with optional live dot |
| `KindBadge` | course category chip, painted from `category-tone.ts` |
| `FreshnessLabel` | sync state: fresh, stale, refreshing, failed, offline, never |

Shared keyframes live in `app.css`: `spin`, `pulse-soft`, `pulse-beacon`, `fade-in`, `slide-up-in`, `shimmer`. A local `@keyframes` of the same name gets scoped by Svelte and shadows the shared one — so components must not redeclare them.

## Authentication surface

- Expanded layout pairs a left ink-blue schedule signal with the login form.
- Compact layout removes the visual schedule but retains brand, language choice, and security guarantees.
- Portal address is empty by default. Supporting copy explains that any portal page can be pasted and normalized.
- The signal panel is abstract geometry plus the real clock. It states no course, room or name it does not have.
- Errors state the problem and recovery path. Loading, success, and credential-store warning states remain in the form region.

## Authenticated surfaces

- Compact windows and expanded desktop windows share the mobile-first structural baseline. On mobile screens (< 768px), a floating five-destination dock provides navigation with safe-area padding; on desktop windows (>= 768px), the dock is hidden and the rail handles navigation.

- **The dock is also where a view's own controls live.** A view fills the shell's control slot (`$lib/state/view-controls.svelte`) and the dock renders it as a row above its destinations, inside the same border, radius, shadow and blur. A view rendering its own bar above the dock is the thing this prevents: two floating surfaces of the same material stacked on each other read as a mistake, and the view has to guess a clearance it does not own. `--dock-clearance` is overridden from the dock's own measured height, because that height changes with the slot and a constant would be wrong in one of the two cases by construction. On an expanded window there is no dock, so the view keeps the same snippet in its own header, where there is width for it.
- Today exposes sync freshness first, then displays the current or next course in an ink-blue container. Time, location text, portal note, progress, and available Tempo action stay together.
- **Schedule is one time grid at three zoom levels.** Day, week and month are not three views: they are three magnifications of the same drawing, and every one of them is laid out to fit the height it was given rather than to a fixed hour height that then has to be scrolled. That is the decision the rest follows from. Courses sit on an hour scale, so a duration is a height and a free slot is a gap; the band opens on 08:00–18:00 and widens outward to whole hours until the earliest and latest course of the visible period fit; overlapping courses split their column into lanes.

  Because the whole band is on screen, the six columns share the width instead of each demanding a minimum, so **nothing scrolls sideways at any zoom**. The week used to scroll two axes inside a 360px box on a phone, and the horizontal swipe had to be disabled there to avoid a coin toss between two meanings on one axis. It is no longer disabled anywhere: a horizontal swipe changes period at every zoom. Zooming in is a tap — a week column header opens that day, a month cell opens that week — and it goes through `CalendarNavigation.zoomTo`, never through `selectDate` followed by `setScope`, which resolves the anchor against the scope it is leaving and asks the portal for a period nobody will look at.

  A block prints what its **measured** column width allows, not what its breakpoint suggests: below roughly 110px it drops to the hour and a hyphenated name, and a lane narrower than 60px keeps only the compact hour and its category field, saying the rest through `aria-label` and on tap. A line marks the current time in today's column, labelled with the hour only where a 53px column would not be covered by the label. Day zoom takes a measure on wide windows — one column across 1200px is a band, not a schedule.

  Month is the same drawing five times smaller, and it is only worth drawing because it stays decodable: every week row carries the hour scale in a gutter reduced to three anchors, the middle anchor is ruled across every cell so morning and afternoon read as halves, each mark carries its category code and, where it is tall enough, its start hour. The month is one field split by hairlines — `gap-px` over the border colour, so every rule is drawn exactly once and no cell needs a border — never thirty bordered cards. A coming exam is the single saturated mark; it is what a student opens a month for. The grid keeps one tab stop, walked with the arrow keys. The selected day's list sits beside it only where there is width for it (`54rem`), and that decides what a cell does when tapped: with the pane on screen a tap selects, without it a tap zooms.

  **Empty never removes the grid.** A band that still says which day it is and which hours it covers tells the reader more than a card standing where the grid was; the statement rides over it and takes no pointer, so the period can still be swiped away.
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

- Minimum control target is `--tap-min` (`2.75rem` / 44px). This is a floor, including for icon-only and compact controls. The one place a control's size is data rather than a choice is a course block on the time grid: its height is its duration. Since the grid fits the band to the height it was given, an hour is worth whatever that division leaves — and the block carries `min-h-(--tap-min)` so the rare quarter-hour course is still reachable. The band scrolls rather than squeezing below 3rem an hour at day zoom and 2rem at week zoom, which is what keeps a fourteen-hour day usable.
- Focus uses a high-contrast ink-blue outline. `outline: none` on a focusable element is only acceptable when a `:focus-visible` style replaces it in the same rule set.
- Any element with `onclick` is a `<button>`. A `div role="button"` must handle both `Enter` and `Space` with `preventDefault()` — so prefer the button.
- Icons are decorative unless they are the only label: `aria-hidden="true"` on the icon, the name on the control.
- Password visibility uses Lucide eye icons with translated accessible names.
- Locale controls expose pressed state.
- A date is never chosen with `<input type="week">` or `type="date"`. Neither WKWebView nor WebKitGTK implements the week picker, where it degrades into a text field expecting `2026-W35`; the app ships its own month sheet instead.
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

## Showcase site

The public site in `site/` is a second surface with its own project: Astro, its own `package.json`, four content pages and a 404. It serves the root of `betteraimaira.montfrond.work`. Everything below describes what that surface actually ships; the app's rules above still bind it unless a line here says otherwise.

**Tokens are copied, not imported.** A separate Astro project cannot `@import` `src/app.css`, and that file also carries Tauri-only concerns — safe areas, the titlebar's `z-index`, the app's keyframes. `site/src/styles/global.css` re-declares every FL-Theme value verbatim and says so at the top. A token changed in the app has to be changed there too; nothing enforces it.

**The display scale is an extension, not a drift.** The app stops at `--text-4xl` (`2.25rem`) because it is a dense information UI. The site adds `--text-5xl` `2.75rem`, `--text-6xl` `3.5rem`, `--text-7xl` `4.25rem`, and no more — the largest headline still wraps on a phone and stays under the 6rem display ceiling. Those three steps belong to this surface only. Every step, old and new, pins its own `--text-*--line-height`: overriding a `--text-*` key replaces only that key, so an unpinned step silently inherits Tailwind's own companion. Body steps keep the app's `1.5`; display steps tighten from `1.3` down to `1`.

Two families need an alias bridge Tailwind has no namespace for, both mirroring the app: `--font-weight-*` so `font-heavy` resolves to the 800 step, and `--transition-duration-*` so `duration-fast` resolves at all.

**The brand lockup is text.** `assets/logo-lockup.svg` sets the product name in a live `<text>` element. Loaded through `<img src>` an SVG cannot reach the page's `@font-face`, so the name rendered in the visitor's OS sans instead of Inter — the banned system display face, on the one element that is the identity — and its second half was painted `--primary`, roughly 2.3:1 as text on a near-white ground. `site/src/components/Lockup.astro` renders the mark as inline SVG and the name as real HTML text, `Better` in `--foreground` and `Aimaira` in `--primary-deep`. The mark keeps its own hardcoded paint and is `aria-hidden`, because the name is spelled out beside it. That SVG is no longer shipped to the site at all.

**Colour fields carry the page rhythm.** The sections alternate whole fields rather than scattering accents: `--background` for the hero, `--card` for the strengths and FAQ bands, `--background` again for the features, `--secondary` for the privacy section, `--card` for the video, `--background` for the platforms, `--muted` for the close. The ink field is the only one, and it is spent on the section the product rests on.

Two consequences of painting on ink:

- Secondary text on ink is `--primary-soft`, tinted from the brand hue. Never grey.
- `--primary` may be a decorative icon on ink, where it clears roughly 4.9:1. The app's rule that `--primary` is never a stroke is about white grounds, where it reads 2.28:1. On white this surface obeys it exactly: every meaningful rule, icon and label is `--primary-deep`.

**Two fills are easy to get wrong.** `--surface-sunken` is `L 0.978`, *lighter* than the page ground at `L 0.968` — it is for nesting inside white, and a panel sitting on the page ground has no edge in it. Such a panel takes `--muted`. And inline code chips are `--muted` on the ground and on white, but `--muted-strong` inside a `--muted` panel, or the chip is the same tint as its container and disappears; `Rich.astro` takes an `onMuted` prop for exactly that case.

**Groups are separated by rules, not containers.** The three strengths and the five platforms are hairline-separated columns and rows, not same-size cards. Same-size cards of icon plus heading plus text as a section's whole structure is the lazy container, and at five items in three columns it also leaves an orphan row.

**One authored motion moment.** The first viewport arrives once as an orchestrated sequence — headline at 0ms, lead at 90ms, actions at 180ms, the tinted field at 200ms, the screenshot at 290ms — through `.arrive` and a per-element `--arrive-delay`. Each element resolves out of a 6px blur as it settles, rather than only fading and sliding. Nothing below the fold has an entrance: an identical reveal repeated on every section is an effect, not a moment. The animation fills `backwards` only, so every element is visible by default, and `prefers-reduced-motion` removes it. `scroll-behavior: smooth` sits inside `@media (prefers-reduced-motion: no-preference)` — smooth scrolling is motion too, and this surface has no global `!important` rule to catch it.

**The 44px floor covers every link.** Not only icon-only controls: the header's brand link, its section links and every footer link carry `min-h-(--tap-min)` and drop their vertical padding, which is why the footer rows are as tall as they are. The locale control carries `min-w-(--tap-min)` as well, because hiding its text label below `sm` would otherwise leave a 38px box on the one viewport class where touch is guaranteed.

**The surfaces the browser draws are themed too.** Selection, the focus ring, the scrollbar, the underline offset, and tabular numerals on every version, size, date and step number are all set from the palette in `global.css`. Text selection is `--primary` with `--primary-foreground` on top; the focus ring is the app's `--ring`.

**Internationalisation does not go through Paraglide.** That is the app's tool. `site/src/i18n/content.ts` holds every visible string once per locale under one shared `Content` type, so a key added to French and forgotten in English fails `astro check` instead of leaving a hole in the page. French and English are authored side by side, not translated. Sizes and dates are formatted with `Intl` and the page's own locale.

**Honest state reaches the download page.** It resolves the newest release in the browser and never uses `/releases/latest`, which excludes prereleases and would 404 on a beta-only project. Every download anchor is server-rendered pointing at the releases page and only upgraded to a direct asset URL by script, so a blocked API degrades to a working page. When the fetch fails, the panel and all three platform rows say so — no slot is left on a loading line that never ends. Only the three platforms that actually publish an asset get a download control; macOS and Linux get build commands and a badge that names the difference.

**Rasters and their provenance.** `site/scripts/copy-media.mjs` copies only what a page references, from `assets/` and `static/`, and fails the build if a listed file is missing. Three app screenshots and the presentation video ship as they are; the portal content inside the screenshots is authored demonstration data, stated on the page under each one. One raster is generated: `assets/showcase/presentation-poster.webp`, the video's poster, `ffmpeg -ss 8 -i assets/showcase/betteraimaira-presentation.mp4 -frames:v 1 -c:v libwebp -quality 82`. The frame is taken from the passage showing the app at work rather than the film's title card, whose chips would have promoted one frame's marketing copy to a permanent claim the product does not make. `assets/showcase/betteraimaira-demo.webp` is never shipped: 19 MB.

**Navigation survives every width.** Below `48rem` the header's in-page links collapse into a `details` disclosure rather than disappearing: a 9700px page with no way to reach its own sections is not a simplification, and `PRODUCT.md` requires core features to stay reachable at every window size. The locale control moves inside that panel too, because at 390 the bar has 350px of content box and a lockup, a menu button, a locale control and the Download button do not all fit — a tap target is never the thing that gives. The disclosure button is the last item in the bar so its panel, anchored to its right edge, stays on screen.

**The video ships the native player and the script trades it away.** The markup carries `<video controls>`, which is the only working player when no script runs. The script then turns `controls` off and reveals a drawn play affordance over the poster; activating it puts `controls` back and starts playback, so a keyboard or screen-reader user reaches the full native player the moment they ask. The default control bar is the one surface on this page nobody designed, and it would otherwise sit across 1152px of it at rest.

**A reading measure belongs to the text, not to the section.** Every section container is `76rem` and every heading therefore starts on the same left edge. Where prose needs a narrower measure — the FAQ answers, a platform's install steps — the cap goes on the list or the paragraph inside. A container narrowed to hold a measure moves the page's own edge, which reads as a mistake in the one section that does it.

**Demonstration data is labelled where it is shown.** The app screenshots carry an authored student, courses and grades: no capture of a real portal is in this repository, and `PRODUCT.md` makes that a hard constraint. Every device frame is a `figure` whose caption says the data is written for the screenshot. The caption is content, not a disclaimer to be trimmed.

**Deployment is part of the design constraint.** Pages serves the `gh-pages` branch, and that branch also carries the update feed installed clients poll. The site is committed into it, preserving `CNAME`, `.nojekyll` and `updates/`. Switching Pages to the Actions source would take every installed updater offline.
