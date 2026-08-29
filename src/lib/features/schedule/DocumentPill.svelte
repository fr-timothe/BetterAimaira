<script lang="ts">
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import type { IconComponent } from '$lib/components/ui/icon';

  type Props = {
    label: string;
    icon: IconComponent;
    /** This document's own request is in flight; the spinner replaces the icon. */
    busy: boolean;
    onclick: () => void;
  };

  const { label, icon, busy, onclick }: Props = $props();

  const Icon = $derived(icon);
</script>

<!-- A pill, but still a tap target: the minimum height is the same one every
     other control in the app answers to. -->
<button
  type="button"
  class="inline-flex min-h-(--tap-min) cursor-pointer items-center gap-2
         rounded-pill border border-border-subtle bg-surface-sunken px-3
         py-[0.35rem] text-xs font-semibold text-foreground
         disabled:cursor-progress disabled:opacity-70
         enabled:hover:bg-muted"
  disabled={busy}
  {onclick}
>
  {#if busy}
    <Spinner size={14} />
  {:else}
    <Icon size={14} aria-hidden="true" />
  {/if}
  {label}
</button>
