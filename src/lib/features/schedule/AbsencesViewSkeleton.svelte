<script lang="ts">
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import HeroStat from './HeroStat.svelte';

  type Props = {
    ariaLabel: string;
    heroLabel: string;
  };

  const { ariaLabel, heroLabel }: Props = $props();

  const copyStack = 'flex min-w-0 flex-col gap-2';
</script>

<div
  class="flex flex-col gap-5"
  role="status"
  aria-live="polite"
  aria-busy="true"
  aria-label={ariaLabel}
>
  <div class="flex w-[min(100%,22rem)] items-center justify-between gap-3 self-center">
    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
    <div class="flex min-w-0 flex-1 flex-col items-center gap-2">
      <Skeleton shape="text" width="4.5rem" />
      <Skeleton shape="title" width="8rem" />
    </div>
    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
  </div>

  <HeroStat
    {ariaLabel}
    label={heroLabel}
    value=""
    loading
    loadingMetricCount={4}
    loadingLabel={ariaLabel}
    showCurve={false}
  />

  <div class="flex min-h-(--tap-min) items-center justify-end gap-2">
    <Skeleton shape="block" width="9rem" height="2.75rem" />
    <div class="desktop-only">
      <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
    </div>
  </div>

  <div class="flex flex-col gap-3">
    {#each Array(3) as _, index (index)}
      <div
        class="grid min-h-19 grid-cols-[1.125rem_minmax(0,1fr)_auto] items-center gap-3
               rounded-xl border border-border-subtle bg-card px-4 py-3"
      >
        <Skeleton shape="block" width="1.125rem" height="1.125rem" />
        <div class={copyStack}>
          <Skeleton shape="title" width={index === 1 ? '52%' : '64%'} />
          <Skeleton shape="text" width="7rem" />
        </div>
        <Skeleton shape="title" width="3.5rem" />
      </div>
    {/each}
  </div>
</div>
