<script lang="ts">
  import type { Snippet } from 'svelte';
  import Button from './Button.svelte';
  import Spinner from './Spinner.svelte';
  import type { IconComponent } from './icon';
  import { cn } from '$lib/utils';

  type Props = {
    /**
     * `loading` announces politely; `error` and `expired` announce assertively.
     * `expired` is the one state a retry cannot fix — it offers sign-in instead.
     */
    kind: 'loading' | 'empty' | 'error' | 'expired';
    title: string;
    description?: string;
    /** Lucide icon. Ignored for `loading`, which shows a spinner instead. */
    icon?: IconComponent;
    actionLabel?: string;
    onAction?: () => void;
    /** Extra content below the action, e.g. a secondary link. */
    footer?: Snippet;
    class?: string;
  };

  const {
    kind,
    title,
    description,
    icon,
    actionLabel,
    onAction,
    footer,
    class: className
  }: Props = $props();

  const Icon = $derived(icon);

  const iconTones = {
    loading: 'bg-muted text-primary-deep',
    empty: 'bg-surface-sunken text-muted-foreground',
    error: 'bg-danger-surface text-danger-strong',
    expired: 'bg-danger-surface text-danger-strong'
  } as const satisfies Record<Props['kind'], string>;
</script>

<!-- The one card that covers loading, empty, error and expired. Elevation is
     declared once — a border, no shadow — so it reads as a panel, not a float. -->
<div
  class={cn(
    'ui-state-card grid min-h-64 content-center justify-items-center gap-3',
    'rounded-xl border border-border-subtle bg-card px-5 py-8 text-center',
    className
  )}
  role={kind === 'loading' ? 'status' : 'alert'}
  aria-live={kind === 'loading' ? 'polite' : 'assertive'}
  aria-busy={kind === 'loading' ? 'true' : undefined}
>
  <span class={cn('grid size-12 place-items-center rounded-md', iconTones[kind])}>
    {#if kind === 'loading'}
      <Spinner size={22} />
    {:else if Icon}
      <Icon size={22} aria-hidden="true" />
    {/if}
  </span>

  <h2 class="max-w-[32ch] text-lg leading-[1.25] font-bold text-balance">{title}</h2>
  {#if description}
    <p class="max-w-[46ch] text-base leading-[1.55] text-pretty text-muted-foreground">
      {description}
    </p>
  {/if}

  {#if actionLabel && onAction}
    <Button variant={kind === 'expired' ? 'ink' : 'primary'} onclick={onAction}>
      {actionLabel}
    </Button>
  {/if}

  {#if footer}
    <div class="mt-1 text-sm">{@render footer()}</div>
  {/if}
</div>
