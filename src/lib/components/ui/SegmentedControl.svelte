<script lang="ts">
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
  };

  let { options, value, label, size = 'md', onChange }: Props = $props();

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
</script>

<div class="ui-segmented {size}" role="tablist" aria-label={label} bind:this={container}>
  {#each options as option, index (option.value)}
    <button
      type="button"
      role="tab"
      class="segment {size}"
      class:active={option.value === value}
      aria-selected={option.value === value}
      tabindex={option.value === value ? 0 : -1}
      onclick={() => onChange(option.value)}
      onkeydown={(event) => handleKeyDown(event, index)}
    >
      {option.label}
    </button>
  {/each}
</div>

<style>
  .ui-segmented {
    display: flex;
    gap: var(--space-1);
    padding: var(--space-1);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow-x: auto;
    scrollbar-width: none;
  }

  .ui-segmented.sm {
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius-sm);
  }

  .ui-segmented::-webkit-scrollbar {
    display: none;
  }

  .segment {
    flex: 1 1 0%;
    min-width: 0;
    min-height: var(--tap-min);
    padding: 0 var(--space-3);
    color: var(--muted-foreground);
    background: transparent;
    border: 0;
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    white-space: nowrap;
    text-align: center;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .segment.sm {
    min-height: 1.75rem;
    padding: 0 var(--space-2);
    border-radius: var(--radius-xs);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  .segment:hover:not(.active) {
    color: var(--foreground);
  }

  .segment:active {
    transform: scale(var(--press-scale));
  }

  .segment.active {
    color: var(--primary-deep);
    background: var(--card);
    box-shadow: var(--shadow-xs);
  }
</style>
