<script lang="ts">
  import type { Snippet } from 'svelte';
  import Spinner from './Spinner.svelte';

  type Props = {
    /** The icon. Skipped entirely while `loading` is true. */
    children: Snippet;
    /** Required: an icon-only control has no visible name of its own. */
    label: string;
    variant?: 'ghost' | 'surface' | 'primary';
    size?: 'sm' | 'md';
    disabled?: boolean;
    /** Swaps the icon for a spinner and marks the control busy. */
    loading?: boolean;
    ariaPressed?: boolean;
    onclick?: (event: MouseEvent) => void;
  };

  const {
    children,
    label,
    variant = 'surface',
    size = 'md',
    disabled = false,
    loading = false,
    ariaPressed,
    onclick
  }: Props = $props();
</script>

<button
  type="button"
  class="ui-icon-button {variant} {size}"
  title={label}
  aria-label={label}
  aria-pressed={ariaPressed}
  aria-busy={loading ? 'true' : undefined}
  disabled={disabled || loading}
  {onclick}
>
  {#if loading}
    <Spinner size={size === 'sm' ? 13 : 17} />
  {:else}
    {@render children()}
  {/if}
</button>

<style>
  .ui-icon-button {
    display: grid;
    width: var(--tap-min);
    height: var(--tap-min);
    flex: 0 0 var(--tap-min);
    place-items: center;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .ui-icon-button.sm {
    width: 1.75rem;
    height: 1.75rem;
    flex: 0 0 1.75rem;
    border-radius: var(--radius-sm);
  }

  .ui-icon-button:active:not(:disabled) {
    transform: scale(var(--press-scale));
  }

  .ui-icon-button:disabled {
    opacity: 0.62;
  }

  .ghost {
    color: var(--muted-foreground);
    background: transparent;
  }

  .ghost:hover:not(:disabled) {
    color: var(--foreground);
    background: var(--muted);
  }

  .surface {
    color: var(--primary-deep);
    background: var(--card);
    border-color: var(--border-subtle);
  }

  .surface:hover:not(:disabled) {
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  .primary {
    color: var(--card);
    background: var(--primary-deep);
  }

  .primary:hover:not(:disabled) {
    background: var(--primary-deep-hover);
  }
</style>
