<script lang="ts">
  import type { Snippet } from 'svelte';
  import { ChevronDown } from 'lucide-svelte';
  import { cn } from '$lib/utils';

  type Props = {
    /** Unique in the page: what the header's `aria-controls` points at. */
    panelId: string;
    title: string;
    /** The date range the portal packs into the block label, if it carries one. */
    range?: string | null;
    /** The one figure the collapsed header states — an average, a total. */
    value: string;
    unit: string;
    /** Names that figure, since the header has no room to label it. */
    valueTitle: string;
    open: boolean;
    onToggle: () => void;
    children: Snippet;
  };

  const { panelId, title, range = null, value, unit, valueTitle, open, onToggle, children }: Props =
    $props();
</script>

<section
  class={cn('overflow-hidden rounded-xl border bg-card', open ? 'border-border' : 'border-border-subtle')}
>
  <h2>
    <button
      type="button"
      class="flex w-full cursor-pointer items-center gap-3 bg-transparent p-4
             text-left hover:bg-surface-sunken"
      aria-expanded={open}
      aria-controls={panelId}
      onclick={onToggle}
    >
      <span
        class={cn(
          'inline-flex shrink-0 text-muted-foreground transition-transform duration-fast ease-[ease]',
          open && 'rotate-180'
        )}
      >
        <ChevronDown size={18} aria-hidden="true" />
      </span>
      <span class="flex min-w-0 flex-1 flex-col gap-[0.15rem]">
        <span class="text-base leading-[1.3] font-bold text-foreground">{title}</span>
        {#if range}
          <small class="text-xs tabular-nums text-muted-foreground">{range}</small>
        {/if}
      </span>
      <span class="inline-flex shrink-0 items-baseline gap-[0.1rem]" title={valueTitle}>
        <strong class="text-xl font-extrabold tabular-nums text-primary-deep">{value}</strong>
        <small class="text-xs font-semibold text-muted-foreground">{unit}</small>
      </span>
    </button>
  </h2>

  <!-- The header keeps its own padding, so the panel only re-opens the gap the
       separator needs. -->
  <div
    id={panelId}
    class="mx-4 flex flex-col gap-4 border-t border-border-subtle py-4
           [&[hidden]]:hidden"
    hidden={!open}
  >
    {@render children()}
  </div>
</section>
