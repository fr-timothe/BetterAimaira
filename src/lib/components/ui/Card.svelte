<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  type Props = {
    children: Snippet;
    /**
     * `plain` is the default panel. `sunken` recedes for nested content — the
     * only nesting a card is allowed, since a card inside a card is always wrong.
     * `ink` is the one emphatic surface, reserved for the current course.
     */
    tone?: 'plain' | 'sunken' | 'ink';
    /** Interactive cards get hover feedback and a pointer. */
    interactive?: boolean;
    padding?: 'none' | 'sm' | 'md' | 'lg';
    class?: string;
  };

  const {
    children,
    tone = 'plain',
    interactive = false,
    padding = 'md',
    class: className
  }: Props = $props();

  // Elevation is declared once. A plain card is a border; only the ink surface
  // and interactive hover reach for a shadow.
  const tones = {
    plain: 'bg-card border border-border-subtle',
    sunken: 'bg-surface-sunken',
    ink: 'bg-secondary text-secondary-foreground shadow-lg'
  } as const satisfies Record<NonNullable<Props['tone']>, string>;

  const paddings = {
    none: 'p-0',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-5'
  } as const satisfies Record<NonNullable<Props['padding']>, string>;
</script>

<div
  class={cn(
    'ui-card min-w-0 rounded-xl',
    tones[tone],
    paddings[padding],
    interactive &&
      'cursor-pointer transition-control hover:border-primary-deep hover:shadow-sm active:scale-[0.995]',
    className
  )}
>
  {@render children()}
</div>
