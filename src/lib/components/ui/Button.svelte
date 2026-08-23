<script lang="ts">
  import type { Snippet } from 'svelte';
  import Spinner from './Spinner.svelte';

  type Props = {
    children: Snippet;
    /**
     * `primary` is the deep cyan fill — the one that clears 4.5:1 with white on
     * top. `ink` is the heavier commitment (sign in, sign out). `accent` fills
     * with brand cyan and takes ink text, so it never carries white.
     */
    variant?: 'primary' | 'ink' | 'accent' | 'ghost' | 'outline' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    type?: 'button' | 'submit';
    disabled?: boolean;
    loading?: boolean;
    /** Fills the inline axis of its container. */
    block?: boolean;
    title?: string;
    ariaLabel?: string;
    ariaPressed?: boolean;
    onclick?: (event: MouseEvent) => void;
  };

  const {
    children,
    variant = 'primary',
    size = 'md',
    type = 'button',
    disabled = false,
    loading = false,
    block = false,
    title,
    ariaLabel,
    ariaPressed,
    onclick
  }: Props = $props();
</script>

<button
  {type}
  {title}
  class="ui-button {variant} {size}"
  class:block
  disabled={disabled || loading}
  aria-label={ariaLabel}
  aria-pressed={ariaPressed}
  aria-busy={loading ? 'true' : undefined}
  {onclick}
>
  {#if loading}
    <Spinner size={size === 'sm' ? 14 : 16} />
  {/if}
  {@render children()}
</button>

<style>
  .ui-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    min-height: var(--tap-min);
    padding: 0 var(--space-4);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    font-weight: var(--weight-semibold);
    white-space: nowrap;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .ui-button.block {
    width: 100%;
  }

  .ui-button:active:not(:disabled) {
    transform: scale(var(--press-scale));
  }

  .ui-button:disabled {
    opacity: 0.62;
  }

  /* Sizes keep a 44px hit target even when the visual box is smaller: the sm
     variant grows its target with padding rather than shrinking below the floor. */
  .sm {
    min-height: var(--tap-min);
    padding: 0 var(--space-3);
    font-size: var(--text-sm);
  }

  .md {
    font-size: var(--text-base);
  }

  .lg {
    min-height: 3.1rem;
    padding: 0 var(--space-5);
    font-size: var(--text-md);
    font-weight: var(--weight-bold);
  }

  .primary {
    color: var(--card);
    background: var(--primary-deep);
  }

  .primary:hover:not(:disabled) {
    background: var(--primary-deep-hover);
  }

  .ink {
    color: var(--secondary-foreground);
    background: var(--secondary);
  }

  .ink:hover:not(:disabled) {
    background: var(--secondary-hover);
  }

  .accent {
    color: var(--primary-foreground);
    background: var(--primary);
  }

  .accent:hover:not(:disabled) {
    background: var(--primary-soft);
  }

  .ghost {
    color: var(--muted-foreground);
    background: transparent;
  }

  .ghost:hover:not(:disabled) {
    color: var(--foreground);
    background: var(--muted);
  }

  .outline {
    color: var(--foreground);
    background: var(--card);
    border-color: var(--border);
  }

  .outline:hover:not(:disabled) {
    color: var(--primary-deep);
    border-color: var(--primary-deep);
  }

  .danger {
    color: var(--card);
    background: var(--danger);
  }

  .danger:hover:not(:disabled) {
    background: var(--danger-strong);
  }
</style>
