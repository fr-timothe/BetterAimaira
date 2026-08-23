<script lang="ts">
  import type { Snippet } from 'svelte';
  import Spinner from './Spinner.svelte';
  import { cn } from '$lib/utils';

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
    class?: string;
  };

  const {
    children,
    label,
    variant = 'surface',
    size = 'md',
    disabled = false,
    loading = false,
    ariaPressed,
    onclick,
    class: className
  }: Props = $props();

  const variants = {
    ghost: 'bg-transparent text-muted-foreground enabled:hover:bg-muted enabled:hover:text-foreground',
    surface:
      'border-border-subtle bg-card text-primary-deep enabled:hover:border-primary-deep enabled:hover:bg-muted',
    primary: 'bg-primary-deep text-card enabled:hover:bg-primary-deep-hover'
  } as const satisfies Record<NonNullable<Props['variant']>, string>;

  // `sm` is the only step allowed below the 44px floor, and only where an
  // ancestor row already guarantees the target.
  const sizes = {
    sm: 'size-7 rounded-sm',
    md: 'size-(--tap-min) rounded-md'
  } as const satisfies Record<NonNullable<Props['size']>, string>;
</script>

<button
  type="button"
  class={cn(
    'ui-icon-button grid flex-none place-items-center border border-transparent transition-control',
    'disabled:opacity-62 enabled:active:scale-(--press-scale)',
    sizes[size],
    variants[variant],
    className
  )}
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
