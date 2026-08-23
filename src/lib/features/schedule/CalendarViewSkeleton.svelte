<script lang="ts">
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import { cn } from '$lib/utils';

  type Props = {
    ariaLabel: string;
    full?: boolean;
  };

  const { ariaLabel, full = false }: Props = $props();

  const copyStack = 'flex min-w-0 flex-col gap-2';
  const spread = 'flex items-center justify-between gap-4';
</script>

<div
  class={cn('flex min-h-96 flex-col gap-4', full && 'w-full p-4 md:px-6 md:pt-5 md:pb-6')}
  role="status"
  aria-live="polite"
  aria-busy="true"
  aria-label={ariaLabel}
>
  {#if full}
    <div class="flex items-center gap-2 overflow-hidden">
      {#each Array(4) as _, index (index)}
        <Skeleton shape="block" width={index === 0 ? '6.5rem' : '5.25rem'} height="2.5rem" />
      {/each}
    </div>

    <div class={cn(spread, 'min-h-20 rounded-xl border border-border-subtle bg-card px-4 py-3')}>
      <div class={copyStack}>
        <Skeleton shape="text" width="5rem" />
        <Skeleton shape="title" width="11rem" />
      </div>
      <div class="flex items-center gap-2">
        {#each Array(3) as _, index (index)}
          <Skeleton
            shape={index === 1 ? 'block' : 'circle'}
            width={index === 1 ? '6rem' : '2.75rem'}
            height="2.75rem"
          />
        {/each}
      </div>
    </div>

    <div class="flex items-center justify-between gap-2 overflow-hidden">
      {#each Array(7) as _, index (index)}
        <div
          class="flex min-w-15 flex-1 flex-col items-center gap-2 rounded-lg border
                 border-border-subtle bg-card px-2 py-3"
        >
          <Skeleton shape="text" width="2rem" />
          <Skeleton shape="title" width="1.75rem" />
          <Skeleton shape="circle" width="0.4rem" height="0.4rem" />
        </div>
      {/each}
    </div>
  {/if}

  <div
    class="flex min-h-96 flex-col gap-3 rounded-xl border border-border-subtle bg-card p-4"
  >
    <div class={spread}>
      <div class={copyStack}>
        <Skeleton shape="title" width="9rem" />
        <Skeleton shape="text" width="6rem" />
      </div>
      <Skeleton shape="block" width="4.5rem" height="1.65rem" />
    </div>

    <div class="flex flex-col gap-3">
      {#each Array(3) as _, index (index)}
        <div
          class="grid min-h-22 grid-cols-[4rem_minmax(0,1fr)] items-center gap-4
                 rounded-lg bg-surface-sunken px-4 py-3"
        >
          <div class={copyStack}>
            <Skeleton shape="title" width="3rem" />
            <Skeleton shape="text" width="2.5rem" />
          </div>
          <div class={copyStack}>
            <Skeleton shape="title" width={index === 1 ? '62%' : '76%'} />
            <Skeleton shape="text" width="48%" />
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
