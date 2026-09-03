<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE — not part of the shipped app.
   *
   * One 48px line replacing the 200px control panel: the period names itself
   * and is the door to the date sheet, and the freshness statement keeps its
   * place because honest state is not a polish item.
   */
  import { ChevronDown } from 'lucide-svelte';
  import FreshnessLabel from '$lib/components/ui/FreshnessLabel.svelte';

  type Props = {
    label: string;
    fetchedAt: number;
    onPick: () => void;
  };

  const { label, fetchedAt, onPick }: Props = $props();
</script>

<header
  class="flex shrink-0 items-center justify-between gap-2 border-b border-border-subtle
         bg-card px-3"
>
  <button
    type="button"
    class="flex min-h-(--tap-min) min-w-0 items-center gap-1 rounded-sm pe-1 text-start
           transition-control active:scale-(--press-scale)"
    onclick={onPick}
  >
    <span class="truncate text-base leading-tight font-extrabold text-foreground">{label}</span>
    <ChevronDown size={16} strokeWidth={2.4} class="shrink-0 text-primary-deep" aria-hidden="true" />
    <span class="sr-only">Choisir une date</span>
  </button>

  <FreshnessLabel {fetchedAt} locale="fr" class="shrink-0" />
</header>
