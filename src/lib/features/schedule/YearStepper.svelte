<script lang="ts">
  import { ChevronLeft, ChevronRight } from 'lucide-svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import { periodStartYear } from './portal-utils';

  type Period = { id: string; label: string };

  type Props = {
    periods: Period[];
    /** The year on screen, already resolved to a real period by the caller. */
    selectedId: string | null;
    onSelect: (id: string) => void;
    label: string;
    previousLabel: string;
    nextLabel: string;
  };

  const { periods, selectedId, onSelect, label, previousLabel, nextLabel }: Props = $props();

  /** Oldest first, so stepping left is stepping back in time. */
  const orderedPeriods = $derived(
    [...periods].sort((left, right) => periodStartYear(left.label) - periodStartYear(right.label))
  );

  const selectedIndex = $derived(orderedPeriods.findIndex((period) => period.id === selectedId));

  const selectedLabel = $derived(
    orderedPeriods.find((period) => period.id === selectedId)?.label ?? '--'
  );

  function stepYear(delta: number) {
    const target = orderedPeriods[selectedIndex + delta];
    if (target) onSelect(target.id);
  }
</script>

<!-- The year is always named and always steppable, so an older year is one
     click away and the current one is never guessed from the rows below. -->
<div class="flex w-full items-center justify-between gap-2 md:w-auto">
  <IconButton
    label={previousLabel}
    variant="ghost"
    disabled={selectedIndex <= 0}
    onclick={() => stepYear(-1)}
  >
    <ChevronLeft size={17} aria-hidden="true" />
  </IconButton>

  <p class="flex flex-1 flex-wrap items-baseline justify-center gap-2">
    <span class="text-xs font-bold tracking-[0.04em] uppercase text-muted-foreground">{label}</span>
    <strong class="text-base font-bold tabular-nums text-foreground">{selectedLabel}</strong>
  </p>

  <IconButton
    label={nextLabel}
    variant="ghost"
    disabled={selectedIndex < 0 || selectedIndex >= orderedPeriods.length - 1}
    onclick={() => stepYear(1)}
  >
    <ChevronRight size={17} aria-hidden="true" />
  </IconButton>
</div>
