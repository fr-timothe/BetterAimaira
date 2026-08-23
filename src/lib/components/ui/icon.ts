import type { Bell } from 'lucide-svelte';

/**
 * The shape of a Lucide icon as this version declares it.
 *
 * `lucide-svelte@1.0.1` still ships its icons as `SvelteComponentTyped`
 * classes, which do not satisfy Svelte 5's `Component` type. Typing an `icon`
 * prop as `Component` therefore rejects every icon in the library, so the
 * primitives borrow the type from a concrete icon instead — they all share one
 * shape. `import type` means nothing is pulled in at runtime.
 */
export type IconComponent = typeof Bell;
