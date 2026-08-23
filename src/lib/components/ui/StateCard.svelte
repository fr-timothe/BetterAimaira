<script lang="ts">
  import type { Snippet } from 'svelte';
  import Button from './Button.svelte';
  import Spinner from './Spinner.svelte';
  import type { IconComponent } from './icon';

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
  };

  const { kind, title, description, icon, actionLabel, onAction, footer }: Props = $props();

  const Icon = $derived(icon);
</script>

<div
  class="ui-state-card {kind}"
  role={kind === 'loading' ? 'status' : 'alert'}
  aria-live={kind === 'loading' ? 'polite' : 'assertive'}
  aria-busy={kind === 'loading' ? 'true' : undefined}
>
  <span class="state-icon">
    {#if kind === 'loading'}
      <Spinner size={22} />
    {:else if Icon}
      <Icon size={22} aria-hidden="true" />
    {/if}
  </span>

  <h2>{title}</h2>
  {#if description}<p>{description}</p>{/if}

  {#if actionLabel && onAction}
    <Button variant={kind === 'expired' ? 'ink' : 'primary'} onclick={onAction}>
      {actionLabel}
    </Button>
  {/if}

  {#if footer}
    <div class="state-footer">{@render footer()}</div>
  {/if}
</div>

<style>
  /* The one card that covers loading, empty, error and expired. Elevation is
     declared once — a border, no shadow — so it reads as a panel, not a float. */
  .ui-state-card {
    display: grid;
    justify-items: center;
    gap: var(--space-3);
    min-height: 16rem;
    align-content: center;
    padding: var(--space-8) var(--space-5);
    text-align: center;
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .state-icon {
    display: grid;
    width: 3rem;
    height: 3rem;
    place-items: center;
    border-radius: var(--radius-md);
  }

  .loading .state-icon {
    color: var(--primary-deep);
    background: var(--muted);
  }

  .empty .state-icon {
    color: var(--muted-foreground);
    background: var(--surface-sunken);
  }

  .error .state-icon,
  .expired .state-icon {
    color: var(--danger-strong);
    background: var(--danger-surface);
  }

  h2 {
    max-width: 32ch;
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--weight-bold);
    line-height: 1.25;
    text-wrap: balance;
  }

  p {
    max-width: 46ch;
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-base);
    line-height: 1.55;
    text-wrap: pretty;
  }

  .state-footer {
    margin-top: var(--space-1);
    font-size: var(--text-sm);
  }
</style>
