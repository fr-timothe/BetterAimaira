<script lang="ts">
  import Skeleton from '$lib/components/ui/Skeleton.svelte';

  type Props = {
    ariaLabel: string;
    includeTabs?: boolean;
  };

  const { ariaLabel, includeTabs = false }: Props = $props();
</script>

<div class="account-skeleton" role="status" aria-live="polite" aria-busy="true" aria-label={ariaLabel}>
  {#if includeTabs}
    <div class="tabs-skeleton">
      {#each Array(4) as _, index (index)}
        <Skeleton shape="block" width={index === 2 ? '7rem' : '5.5rem'} height="2.75rem" />
      {/each}
    </div>
  {/if}

  <div class="identity-skeleton">
    <div class="identity-head-skeleton">
      <Skeleton shape="circle" width="4rem" height="4rem" />
      <div class="identity-copy-skeleton">
        <Skeleton shape="block" width="5rem" height="1.5rem" />
        <Skeleton shape="title" width="70%" />
        <Skeleton shape="text" width="45%" />
      </div>
    </div>
    <div class="chips-skeleton">
      {#each Array(3) as _, index (index)}
        <Skeleton shape="block" height="2.5rem" width={index === 1 ? '85%' : undefined} />
      {/each}
    </div>
  </div>

  <div class="panel-grid-skeleton">
    {#each Array(2) as _, panelIndex (panelIndex)}
      <div class="panel-skeleton">
        <div class="panel-heading-skeleton">
          <Skeleton shape="circle" width="2.75rem" height="2.75rem" />
          <Skeleton shape="title" width={panelIndex === 0 ? '55%' : '45%'} />
        </div>
        {#each Array(3) as _, rowIndex (rowIndex)}
          <div class="field-skeleton">
            <Skeleton shape="text" width="35%" />
            <Skeleton shape="text" width={rowIndex === 1 ? '62%' : '78%'} />
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .account-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .tabs-skeleton {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    overflow: hidden;
    padding: var(--space-1);
    background: var(--surface-sunken);
    border-radius: var(--radius-lg);
  }

  .identity-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5) var(--space-4);
    background: var(--muted);
    border-radius: var(--radius-xl);
  }

  .identity-head-skeleton,
  .panel-heading-skeleton {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    min-width: 0;
  }

  .identity-copy-skeleton {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: var(--space-2);
  }

  .chips-skeleton {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-2);
  }

  .panel-grid-skeleton {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-4);
  }

  .panel-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .field-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-height: 3.5rem;
    padding: var(--space-2) var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
  }

  @media (min-width: 48rem) {
    .identity-skeleton {
      padding: var(--space-6) var(--space-5);
    }

    .chips-skeleton,
    .panel-grid-skeleton {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
