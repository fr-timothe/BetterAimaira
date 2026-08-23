<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  type Props = {
    title: string;
    /** Already formatted with the caller's locale. */
    value: string;
    /** Trails the value: `/20`, `h`. */
    unit?: string;
    /** Status tone. The title always says it too — colour is never the signal. */
    tone?: 'neutral' | 'success' | 'warning' | 'danger';
    /** Small leading icon for the title. */
    icon?: Snippet;
    class?: string;
  };

  const { title, value, unit, tone = 'neutral', icon, class: className }: Props = $props();

  // A subtle tint and a crisp value: the tile stays legible in a row of four,
  // and the title carries the same information in words.
  const tiles = {
    neutral: 'border-border-subtle bg-card',
    success: 'border-success-line bg-success-veil',
    warning: 'border-warning-line bg-warning-veil',
    danger: 'border-danger-line bg-danger-veil'
  } as const satisfies Record<NonNullable<Props['tone']>, string>;

  const inks = {
    neutral: 'text-foreground',
    success: 'text-success-strong',
    warning: 'text-warning-strong',
    danger: 'text-danger-strong'
  } as const satisfies Record<NonNullable<Props['tone']>, string>;
</script>

<div
  class={cn(
    'hero-metric flex w-full min-w-0 flex-col items-center justify-center gap-1 rounded-lg',
    'border px-2 py-2.5 text-center shadow-xs',
    'transition-[border-color,background-color] duration-fast ease-out',
    tiles[tone],
    className
  )}
>
  <span
    class="inline-flex w-full min-w-0 items-center justify-center gap-1 text-center text-xs
           leading-[1.25] font-medium text-muted-foreground"
  >
    {#if icon}
      <span
        class={cn('inline-flex shrink-0 items-center justify-center', tone !== 'neutral' && inks[tone])}
        aria-hidden="true"
      >
        {@render icon()}
      </span>
    {/if}
    <span class="min-w-0 wrap-break-word break-normal text-balance">{title}</span>
  </span>
  <span
    class={cn(
      'inline-flex items-baseline justify-center gap-[0.1rem] text-lg leading-[1.2] font-bold tabular-nums',
      inks[tone]
    )}
  >
    {value}{#if unit}<small class="text-xs leading-[1.2] font-medium text-muted-foreground">{unit}</small>{/if}
  </span>
</div>
