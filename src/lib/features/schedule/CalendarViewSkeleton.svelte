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

  /**
   * The placeholder is the shape of the time grid it stands in for: hour gutter,
   * day headers, and blocks whose heights differ the way courses do. A list-shaped
   * skeleton in front of a grid announces a layout that never arrives.
   */
  const columns = [
    [
      { top: '6%', height: '18%' },
      { top: '28%', height: '20%' },
      { top: '62%', height: '30%' },
    ],
    [
      { top: '10%', height: '15%' },
      { top: '30%', height: '20%' },
    ],
    [{ top: '18%', height: '30%' }],
    [
      { top: '6%', height: '20%' },
      { top: '30%', height: '20%' },
      { top: '58%', height: '20%' },
    ],
    [
      { top: '30%', height: '20%' },
      { top: '62%', height: '30%' },
    ],
    [],
  ];
</script>

<div
  class={cn('flex min-h-96 flex-col gap-3', full && 'w-full p-4 md:px-6 md:pt-5 md:pb-6')}
  role="status"
  aria-live="polite"
  aria-busy="true"
  aria-label={ariaLabel}
>
  {#if full}
    <div
      class={cn(spread, 'min-h-20 flex-wrap rounded-xl border border-border-subtle bg-card px-4 py-3')}
    >
      <div class={copyStack}>
        <Skeleton shape="text" width="5rem" />
        <Skeleton shape="title" width="11rem" />
        <Skeleton shape="text" width="7rem" />
      </div>
      <div class="flex items-center gap-1.5">
        {#each Array(5) as _, index (index)}
          <Skeleton
            shape={index === 1 ? 'block' : 'circle'}
            width={index === 1 ? '6rem' : '2.75rem'}
            height="2.75rem"
          />
        {/each}
      </div>
      <Skeleton shape="block" width="15rem" height="2.75rem" />
    </div>
  {/if}

  <div class="flex items-center gap-2 overflow-hidden md:hidden">
    {#each Array(6) as _, index (index)}
      <div
        class="flex min-h-18 min-w-14 flex-1 flex-col items-center gap-2 rounded-lg border
               border-border-subtle bg-card px-2 py-3"
      >
        <Skeleton shape="text" width="2rem" />
        <Skeleton shape="title" width="1.75rem" />
      </div>
    {/each}
  </div>

  <div class="rounded-xl border border-border-subtle bg-card p-2 md:p-3">
    <div class="grid grid-cols-[3.25rem_repeat(6,minmax(0,1fr))] gap-x-1.5">
      <span></span>
      {#each columns as _, index (index)}
        <div class="mb-1 flex min-h-(--tap-min) flex-col items-center justify-center gap-1">
          <Skeleton shape="text" width="2.25rem" />
          <Skeleton shape="text" width="1.5rem" />
        </div>
      {/each}

      <div class="relative h-[18rem] md:h-[24rem]">
        {#each Array(7) as _, index (index)}
          <span class="absolute end-1.5" style:top={`${(index / 6) * 100}%`}>
            <Skeleton shape="text" width="2rem" />
          </span>
        {/each}
      </div>

      {#each columns as blocks, index (index)}
        <div class="relative h-[18rem] rounded-md bg-surface-sunken md:h-[24rem]">
          {#each blocks as block, blockIndex (blockIndex)}
            <span class="absolute inset-x-[3px]" style:top={block.top} style:height={block.height}>
              <Skeleton shape="block" width="100%" height="100%" />
            </span>
          {/each}
        </div>
      {/each}
    </div>
  </div>
</div>
