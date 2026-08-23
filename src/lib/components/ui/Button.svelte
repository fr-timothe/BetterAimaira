<script lang="ts">
  import type { Snippet } from 'svelte';
  import Spinner from './Spinner.svelte';
  import { cn } from '$lib/utils';

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
    class?: string;
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
    onclick,
    class: className
  }: Props = $props();

  const variants = {
    primary: 'bg-primary-deep text-card enabled:hover:bg-primary-deep-hover',
    ink: 'bg-secondary text-secondary-foreground enabled:hover:bg-secondary-hover',
    accent: 'bg-primary text-primary-foreground enabled:hover:bg-primary-soft',
    ghost: 'bg-transparent text-muted-foreground enabled:hover:bg-muted enabled:hover:text-foreground',
    outline:
      'border-border bg-card text-foreground enabled:hover:border-primary-deep enabled:hover:text-primary-deep',
    danger: 'bg-danger text-card enabled:hover:bg-danger-strong'
  } as const satisfies Record<NonNullable<Props['variant']>, string>;

  // Sizes keep a 44px hit target even when the visual box is smaller: the sm
  // variant grows its target with padding rather than shrinking below the floor.
  const sizes = {
    sm: 'px-3 text-sm',
    md: 'text-base',
    lg: 'min-h-[3.1rem] px-5 text-md font-bold'
  } as const satisfies Record<NonNullable<Props['size']>, string>;
</script>

<button
  {type}
  {title}
  class={cn(
    'ui-button inline-flex min-h-(--tap-min) items-center justify-center gap-2 px-4',
    'rounded-md border border-transparent font-semibold whitespace-nowrap',
    'transition-control disabled:opacity-62 enabled:active:scale-(--press-scale)',
    sizes[size],
    variants[variant],
    block && 'w-full',
    className
  )}
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
