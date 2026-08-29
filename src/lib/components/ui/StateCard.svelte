<script lang="ts">
  import type { Snippet } from 'svelte';
  import Button from './Button.svelte';
  import Spinner from './Spinner.svelte';
  import type { IconComponent } from './icon';
  import { cn } from '$lib/utils';

  type Props = {
    /**
     * The six honest states. Only `error` and `expired` interrupt a screen
     * reader; everything else waits its turn, because an empty list or a
     * missing network path is not worth cutting off what is being read.
     * `expired` is the one state a retry cannot fix — it offers sign-in instead.
     * `offline` is a device problem and must never be dressed up as `error`.
     */
    kind: 'loading' | 'empty' | 'error' | 'expired' | 'offline' | 'stale';
    title: string;
    description?: string;
    /** Lucide icon. Ignored for `loading`, which shows a spinner instead. */
    icon?: IconComponent;
    actionLabel?: string;
    onAction?: () => void;
    /** The action is running: the button holds the label and shows a spinner. */
    actionLoading?: boolean;
    actionDisabled?: boolean;
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
    actionLoading = false,
    actionDisabled = false,
    footer,
    class: className
  }: Props = $props();

  const Icon = $derived(icon);

  // The danger surface is reserved for the two states that actually failed.
  // Offline and stale are honest reports, not alarms, so they read like `empty`.
  const iconTones = {
    loading: 'bg-muted text-primary-deep',
    empty: 'bg-surface-sunken text-muted-foreground',
    error: 'bg-danger-surface text-danger-strong',
    expired: 'bg-danger-surface text-danger-strong',
    offline: 'bg-surface-sunken text-muted-foreground',
    stale: 'bg-surface-sunken text-muted-foreground'
  } as const satisfies Record<Props['kind'], string>;

  // Assertive cuts off whatever the screen reader is saying, so it is spent
  // only where the user has to act before anything else is useful.
  const liveness = {
    loading: 'polite',
    empty: 'polite',
    error: 'assertive',
    expired: 'assertive',
    offline: 'polite',
    stale: 'polite'
  } as const satisfies Record<Props['kind'], 'polite' | 'assertive'>;

  const ariaLive = $derived(liveness[kind]);
</script>

<!-- The one card that covers all six honest states. Elevation is declared
     once — a border, no shadow — so it reads as a panel, not a float. -->
<div
  class={cn(
    'ui-state-card grid min-h-64 content-center justify-items-center gap-3',
    'rounded-xl border border-border-subtle bg-card px-5 py-8 text-center',
    className
  )}
  role={ariaLive === 'assertive' ? 'alert' : 'status'}
  aria-live={ariaLive}
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
    <Button
      variant={kind === 'expired' ? 'ink' : 'primary'}
      onclick={onAction}
      loading={actionLoading}
      disabled={actionDisabled}
    >
      {actionLabel}
    </Button>
  {/if}

  {#if footer}
    <div class="mt-1 text-sm">{@render footer()}</div>
  {/if}
</div>
