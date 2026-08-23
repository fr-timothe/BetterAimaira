<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  type Props = {
    children: Snippet;
    /**
     * Status tones carry meaning, so each pairs a tinted surface with a text
     * tone that clears 4.5:1 on it. Never let the tone be the only signal —
     * the label has to say it too.
     */
    tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'live';
    /** Small leading dot for live/attention states. */
    dot?: boolean;
    class?: string;
  };

  const { children, tone = 'neutral', dot = false, class: className }: Props = $props();

  const tones = {
    neutral: 'bg-category-other-surface text-category-other-text',
    accent: 'bg-muted text-primary-deep',
    success: 'bg-success-surface text-success-strong',
    warning: 'bg-warning-surface text-warning-strong',
    danger: 'bg-danger-surface text-danger-strong',
    live: 'bg-primary-deep text-card'
  } as const satisfies Record<NonNullable<Props['tone']>, string>;
</script>

<span
  class={cn(
    'ui-badge inline-flex items-center gap-1 rounded-pill px-2 py-[0.2rem]',
    'text-2xs font-bold whitespace-nowrap',
    tones[tone],
    className
  )}
>
  {#if dot}
    <span
      class={cn('size-[0.4rem] rounded-full bg-current', tone === 'live' && 'animate-pulse-beacon')}
      aria-hidden="true"
    ></span>
  {/if}
  {@render children()}
</span>
