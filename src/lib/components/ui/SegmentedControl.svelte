<script lang="ts">
  import { cn } from '$lib/utils';

  type Option = {
    value: string;
    label: string;
  };

  type Props = {
    options: Option[];
    value: string;
    /** Names the group for screen readers. Never reuse an option's own label. */
    label: string;
    size?: 'sm' | 'md';
    onChange: (value: string) => void;
    class?: string;
  };

  let { options, value, label, size = 'md', onChange, class: className }: Props = $props();

  let container: HTMLDivElement | undefined = $state();

  /**
   * Arrow / Home / End move between tabs, which is what a tablist owes the
   * keyboard. Without this the control is reachable but not operable.
   */
  function handleKeyDown(event: KeyboardEvent, index: number) {
    const last = options.length - 1;
    let next: number | null = null;

    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') next = index === last ? 0 : index + 1;
    else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') next = index === 0 ? last : index - 1;
    else if (event.key === 'Home') next = 0;
    else if (event.key === 'End') next = last;

    if (next === null) return;
    event.preventDefault();
    onChange(options[next].value);
    container?.querySelectorAll<HTMLButtonElement>('button')[next]?.focus();
  }

  const trackSizes = {
    sm: 'gap-[2px] rounded-sm p-[2px]',
    md: 'gap-1 rounded-md p-1'
  } as const satisfies Record<NonNullable<Props['size']>, string>;

  // `--tap-min` is a floor, and the design system names compact controls as
  // covered by it, so `sm` differs from `md` in rounding, padding and type —
  // never in height.
  const segmentSizes = {
    sm: 'min-h-(--tap-min) rounded-xs px-2 text-xs font-medium',
    md: 'min-h-(--tap-min) rounded-sm px-3 text-sm font-semibold'
  } as const satisfies Record<NonNullable<Props['size']>, string>;
</script>

<div
  class={cn(
    'ui-segmented flex overflow-x-auto border border-border-subtle bg-surface-sunken',
    // `scrollbar-width` is not universal across the platform webviews yet, so
    // the WebKit pseudo-element stays alongside it.
    'scrollbar-none [&::-webkit-scrollbar]:hidden',
    trackSizes[size],
    className
  )}
  role="tablist"
  aria-label={label}
  bind:this={container}
>
  {#each options as option, index (option.value)}
    {@const active = option.value === value}
    <button
      type="button"
      role="tab"
      class={cn(
        'min-w-0 flex-1 bg-transparent text-center whitespace-nowrap',
        'transition-control active:scale-(--press-scale)',
        segmentSizes[size],
        active
          ? 'bg-card text-primary-deep shadow-xs'
          : 'text-muted-foreground hover:text-foreground'
      )}
      aria-selected={active}
      tabindex={active ? 0 : -1}
      onclick={() => onChange(option.value)}
      onkeydown={(event) => handleKeyDown(event, index)}
    >
      {option.label}
    </button>
  {/each}
</div>
