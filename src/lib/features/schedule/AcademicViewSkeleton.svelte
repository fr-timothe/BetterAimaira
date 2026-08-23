<script lang="ts">
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import HeroStat from './HeroStat.svelte';

  type Props = {
    ariaLabel: string;
    heroLabel: string;
  };

  const { ariaLabel, heroLabel }: Props = $props();
</script>

<div class="academic-skeleton" role="status" aria-live="polite" aria-busy="true" aria-label={ariaLabel}>
  <div class="year-switch-skeleton">
    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
    <div class="year-copy-skeleton">
      <Skeleton shape="text" width="4.5rem" />
      <Skeleton shape="title" width="8rem" />
    </div>
    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
  </div>

  <HeroStat ariaLabel={ariaLabel} label={heroLabel} value="" loading loadingLabel={ariaLabel} />

  <div class="toolbar-skeleton">
    <Skeleton shape="block" width="9rem" height="2.75rem" />
    <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
  </div>

  <div class="block-list-skeleton">
    {#each Array(3) as _, index (index)}
      <div class="block-card-skeleton">
        <Skeleton shape="circle" width="2rem" height="2rem" />
        <div class="block-copy-skeleton">
          <Skeleton shape="title" width={index === 1 ? '52%' : '64%'} />
          <Skeleton shape="text" width="7rem" />
        </div>
        <Skeleton shape="title" width="3.5rem" />
      </div>
    {/each}
  </div>
</div>

<style>
  .academic-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .year-switch-skeleton {
    display: flex;
    align-items: center;
    align-self: center;
    justify-content: space-between;
    width: min(100%, 22rem);
    gap: var(--space-3);
  }

  .year-copy-skeleton {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }

  .toolbar-skeleton {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    min-height: var(--tap-min);
  }

  .block-list-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .block-card-skeleton {
    display: grid;
    grid-template-columns: 2rem minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--space-3);
    min-height: 4.75rem;
    padding: var(--space-3) var(--space-4);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .block-copy-skeleton {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: var(--space-2);
  }
</style>
