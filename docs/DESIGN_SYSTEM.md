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

## 3. Data visualization specifications

### A. Grade progression spline curve (GradeTrendChart.svelte)
- SVG cubic Bézier spline with gradient area fill (`oklch(0.73 0.14 229 / 0.15)`).
- Hover or touch scrub reveals date, grade, and evaluation name.
- Dashed horizontal benchmark lines indicate class average and passing grade (10/20).

### B. Triple radial ring widget (TripleRadialProgress.svelte)
- Ring 1 (Outer, Cyan): Grade percentage (`overallAverage / 20 * 100%`).
- Ring 2 (Middle, Emerald): Attendance rate (`(quota - absentHours) / quota * 100%`).
- Ring 3 (Inner, Amber): ECTS credit completion (`validatedCredits / totalCredits * 100%`).

### C. Class distribution bar chart (GradeDistributionBar.svelte)
- Stacked horizontal range bar showing minimum to maximum promo scores.
- Class average indicated by a contrasting marker.
- Student score indicated by a primary accent marker.

### D. Live class countdown and timeline card (HeroNextCourse.svelte)
- In-progress classes show a pulsing indicator and percentage progress bar.
- Upcoming classes show a countdown time (for example, "In 14 min").
- Finished state shows "No more classes today".
- Room numbers use display typography (`text-3xl font-black`) for quick recognition.

---

## 4. Adaptive layout

Layout follows available Tauri window width. Full architecture, platform variants, native boundaries, and verification matrices are detailed in [Application structure and platform strategy](APP_STRUCTURE_AND_PLATFORMS.md).

### Compact (`< 640px`)
- Bottom navigation with safe-area padding and five destinations maximum.
- Single-column layout with focused details and day schedule.
- Touch-first targets with visible button alternatives for swipe gestures.

### Medium (`640-1023px`)
- Navigation rail.
- Two-pane or master-detail layout where useful.

### Expanded (`>= 1024px`)
- Persistent sidebar, collapsible near lower boundary.
- Multi-column dashboard, weekly schedule, and comparison tables with bounded maximum width.

### Shared rules
- Container queries for reusable panels and media queries for app shell.
- Independent detection of `hover`, `pointer`, keyboard focus, and reduced motion.
- Respect `env(safe-area-inset-*)` values across mobile and desktop windows.
- Present identical core features across all window widths through reflow rather than removal.

### Authentication surface
- Compact windows use a single login surface with brand mark and language selector.
- Expanded windows pair the form with a schedule preview signal.
- Form controls use visible borders, `0.6rem` corner radii, and minimum `44px` interactive targets.
- Error messages identify the cause and recovery step. Password visibility control includes an accessible label and tooltip.
