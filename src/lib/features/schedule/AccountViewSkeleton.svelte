<script lang="ts">
  import Skeleton from '$lib/components/ui/Skeleton.svelte';

  type Props = {
    ariaLabel: string;
    includeTabs?: boolean;
  };

  const { ariaLabel, includeTabs = false }: Props = $props();

  const headingRow = 'flex min-w-0 items-center gap-4';
  const twoUp = 'grid grid-cols-1 gap-4 md:grid-cols-2';
</script>

<div
  class="flex flex-col gap-4"
  role="status"
  aria-live="polite"
  aria-busy="true"
  aria-label={ariaLabel}
>
  {#if includeTabs}
    <div class="flex items-center gap-1 overflow-hidden rounded-lg bg-surface-sunken p-1">
      {#each Array(4) as _, index (index)}
        <Skeleton shape="block" width={index === 2 ? '7rem' : '5.5rem'} height="2.75rem" />
      {/each}
    </div>
  {/if}

  <div class="flex flex-col gap-4 rounded-xl bg-muted px-4 py-5 md:px-5 md:py-6">
    <div class={headingRow}>
      <Skeleton shape="circle" width="4rem" height="4rem" />
      <div class="flex min-w-0 flex-1 flex-col gap-2">
        <Skeleton shape="block" width="5rem" height="1.5rem" />
        <Skeleton shape="title" width="70%" />
        <Skeleton shape="text" width="45%" />
      </div>
    </div>
    <div class="grid grid-cols-1 gap-2 md:grid-cols-2">
      {#each Array(3) as _, index (index)}
        <Skeleton shape="block" height="2.5rem" width={index === 1 ? '85%' : undefined} />
      {/each}
    </div>
  </div>

  <div class={twoUp}>
    {#each Array(2) as _, panelIndex (panelIndex)}
      <div class="flex flex-col gap-3 rounded-xl border border-border-subtle bg-card p-4">
        <div class={headingRow}>
          <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
          <Skeleton shape="title" width={panelIndex === 0 ? '55%' : '45%'} />
        </div>
        {#each Array(3) as _, rowIndex (rowIndex)}
          <div class="flex min-h-14 flex-col gap-2 rounded-md bg-surface-sunken px-3 py-2">
            <Skeleton shape="text" width="35%" />
            <Skeleton shape="text" width={rowIndex === 1 ? '62%' : '78%'} />
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>
