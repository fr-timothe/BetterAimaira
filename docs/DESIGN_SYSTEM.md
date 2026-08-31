[← Documentation index](README.md)

# Design system and guidelines

BetterAimaira prioritizes direct information access, clear data visualizations, and platform-adapted layouts.

---

## 1. Core principles

1. **Primary information first.** Current or next class, time, and location appear immediately when available.
2. **Direct actions.** Refresh, session links, day navigation, language, and account actions remain in their relevant view context.
3. **Honest state.** Freshness, loading, session expiry, portal errors, and credential store failures are explicitly shown.
4. **Platform familiarity.** Compact windows use touch navigation and safe-area insets; expanded windows use persistent navigation without altering information hierarchy.

---

## 2. Color palette and tokens (FL-Theme)

The visual theme combines high-contrast accents with slate body tones.

The approved source is [FL-Theme on tweakcn](https://tweakcn.com/themes/cmq57ht7w000204l2axo6ho9v). The initial vertical slice ships light mode only. BetterAimaira maintains layout and product identity while preserving published semantic token values.

| Token | Light Value | Dark Value | Purpose |
|---|---|---|---|
| `--primary` | `oklch(0.7347 0.1447 228.9136)` (Cyan) | `oklch(0.7347 0.1447 228.9136)` (Cyan) | Action buttons, active tabs, grade badges |
| `--background` | `oklch(0.9680 0.0059 239.8195)` | `oklch(0.1600 0.0300 248.9089)` | Global page background |
| `--card` | `oklch(1.0000 0 0)` | `oklch(0.2000 0.0350 248.9089)` | Content cards, widgets, list items |
| `--foreground` | `oklch(0.3098 0.0748 248.9089)` | `oklch(0.9800 0.0100 240.0000)` | Primary text |
| `--muted` | `oklch(0.9352 0.0412 217.6616)` | `oklch(0.2500 0.0350 248.9089)` | Secondary backgrounds, badge fills |
| `--radius` | `0.75rem` (12px) | `0.75rem` (12px) | Card and button corner radius |

---

## 3. Data visualization

The vertical slice ships no charts. Numbers are read as numbers: `HeroStat` for
the one headline figure a view is about, `HeroMetric` for the row of secondary
figures under it, and a plotted trend line behind the hero when — and only
when — the caller passes a real series.

`HeroStat` draws that line itself: a quadratic spline through the series'
midpoints, coloured per segment by its slope so a rising run reads as rising
before the numbers are read at all. Given no series it falls back to decorative
geometry, which is the honest thing to draw when there is nothing to plot.

---

## 4. Adaptive layout

Layout follows available Tauri window width. Full architecture, platform variants, native boundaries, and verification matrices are detailed in [Application structure and platform strategy](APP_STRUCTURE_AND_PLATFORMS.md).

### Compact (`< 48rem`)
- Floating five-destination dock with safe-area padding.
- Single-column layout with focused details and the day schedule.
- Touch-first targets with visible button alternatives for swipe gestures.

### Expanded (`>= 48rem`)
- Fixed-width icon navigation rail; each destination's name is served on hover.
- Multi-column dashboard, the portal week grid, and comparison tables that
  assemble as real columns once six of them fit.

### Secondary steps

`48rem` is the primary hinge and the only one a view should introduce without a
reason. Where a layout genuinely needs more room than that, it says so at the
element: `min-[30rem]` (two short detail cells side by side), `min-[54rem]`
(the month grid beside its selected-day panel), `min-[56rem]` (the week grid
stops reserving a minimum column width), and the `lte-600` / `lte-820` variants
for the two compact steps that pre-date the hinge.

### Shared rules
- Media queries for the app shell, expressed as Tailwind variants at the element rather than as a block at the end of a file.
- Independent detection of `hover`, `pointer`, keyboard focus, and reduced motion.
- Respect the safe-area insets across mobile and desktop windows, through the `*-safe-*` utilities (`pt-safe-8`, `pb-safe-2`, `mx-safe-3`, …) rather than the `--safe-*` tokens by hand, and never through `env(safe-area-inset-*)` directly (see `src/lib/native-insets.ts` and the utility block in `src/app.css`). Each utility adds the inset to a step of the spacing scale; `max(step, inset)` is a defect, because it spends the design's gutter on the system's strip.
- Present identical core features across all window widths through reflow rather than removal.

### Authentication surface
- Compact windows use a single login surface with brand mark and language selector.
- Expanded windows pair the form with a schedule preview signal.
- Form controls use visible borders, the `--radius-md` corner (12px), and the `--tap-min` 44px interactive floor.
- Error messages identify the cause and recovery step. Password visibility control includes an accessible label and tooltip.
