# Brand assets

Vector sources for the BetterAimaira identity, built on the **"Slot Matrix"** concept
(*Le Signal de Créneau*).

## Files

| File | Format | Description | Use |
| :--- | :--- | :--- | :--- |
| [`logo.svg`](./logo.svg) | SVG (100x100) | Full icon on an ink-blue squircle (`#13253F`) | App icon, splash screen, avatar |
| [`mark.svg`](./mark.svg) | SVG (100x100) | Transparent mark for light backgrounds | In-app headers, navigation bars, badges |
| [`mark-dark.svg`](./mark-dark.svg) | SVG (100x100) | Transparent mark for dark backgrounds | Dark mode, dark footers |
| [`logo-lockup.svg`](./logo-lockup.svg) | SVG (280x80) | Horizontal lockup (symbol + "BetterAimaira" wordmark) | Desktop login header, documentation |

[`showcase/`](./showcase/) holds the final promotional video (`betteraimaira-presentation.mp4`), the animated 1080p WebP loop (`betteraimaira-demo.webp`), and screenshots.
[`screenshots/`](./screenshots/) holds the static images embedded in documentation.

## Symbolism and geometry

- **Schedule columns.** Rounded vertical pillars standing for the slots of a student timetable.
- **Active cyan column (`#00B9E8`).** The current or next course — the room, and the immediacy of
  the information.
- **Beacon dot.** Live synchronization and punctuality.
- **Monogram.** Three columns plus the low dot draw the silhouette of a capital A.

## Palette (FL-Theme)

- **Ink blue:** `#13253F` / `oklch(0.3098 0.0748 248.9089)`
- **Primary cyan:** `#00B9E8` / `oklch(0.7347 0.1447 228.9136)`
- **Secondary blue:** `#223D63` / `#3A5378`
- **White:** `#FFFFFF`

The full token set and the rules for applying it live in [DESIGN.md](../DESIGN.md).

## Svelte component

In application code, use the dedicated component
[`src/lib/assets/Logo.svelte`](../src/lib/assets/Logo.svelte) rather than importing an SVG:

```svelte
<script>
  import Logo from '$lib/assets/Logo.svelte';
</script>

<!-- Transparent mark for a header -->
<Logo size={24} variant="mark" />

<!-- Icon with the squircle background -->
<Logo size={40} variant="icon" />

<!-- Lockup with the wordmark -->
<Logo size={32} variant="lockup" />
```
