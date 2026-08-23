<script lang="ts">
  import Skeleton from '$lib/components/ui/Skeleton.svelte';

  type Props = {
    ariaLabel: string;
    full?: boolean;
  };

  const { ariaLabel, full = false }: Props = $props();
</script>

<div
  class="calendar-skeleton"
  class:full
  role="status"
  aria-live="polite"
  aria-busy="true"
  aria-label={ariaLabel}
>
  {#if full}
    <div class="scope-skeleton">
      {#each Array(4) as _, index (index)}
        <Skeleton shape="block" width={index === 0 ? '6.5rem' : '5.25rem'} height="2.5rem" />
      {/each}
    </div>

    <div class="period-skeleton">
      <div class="period-copy-skeleton">
        <Skeleton shape="text" width="5rem" />
        <Skeleton shape="title" width="11rem" />
      </div>
      <div class="period-actions-skeleton">
        {#each Array(3) as _, index (index)}
          <Skeleton shape={index === 1 ? 'block' : 'circle'} width={index === 1 ? '6rem' : '2.75rem'} height="2.75rem" />
        {/each}
      </div>
    </div>

    <div class="ribbon-skeleton">
      {#each Array(7) as _, index (index)}
        <div class="day-skeleton">
          <Skeleton shape="text" width="2rem" />
          <Skeleton shape="title" width="1.75rem" />
          <Skeleton shape="circle" width="0.4rem" height="0.4rem" />
        </div>
      {/each}
    </div>
  {/if}

  <div class="calendar-body-skeleton">
    <div class="body-heading-skeleton">
      <div class="body-copy-skeleton">
        <Skeleton shape="title" width="9rem" />
        <Skeleton shape="text" width="6rem" />
      </div>
      <Skeleton shape="block" width="4.5rem" height="1.65rem" />
    </div>

    <div class="course-list-skeleton">
      {#each Array(3) as _, index (index)}
        <div class="course-row-skeleton">
          <div class="time-skeleton">
            <Skeleton shape="title" width="3rem" />
            <Skeleton shape="text" width="2.5rem" />
          </div>
          <div class="course-copy-skeleton">
            <Skeleton shape="title" width={index === 1 ? '62%' : '76%'} />
            <Skeleton shape="text" width="48%" />
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .calendar-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    min-height: 24rem;
  }

  .calendar-skeleton.full {
    width: 100%;
    padding: var(--space-4);
    box-sizing: border-box;
  }

  .scope-skeleton,
  .period-actions-skeleton,
  .ribbon-skeleton {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .scope-skeleton,
  .ribbon-skeleton {
    overflow: hidden;
  }

  .period-skeleton,
  .body-heading-skeleton {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .period-skeleton {
    min-height: 5rem;
    padding: var(--space-3) var(--space-4);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .period-copy-skeleton,
  .body-copy-skeleton,
  .course-copy-skeleton,
  .time-skeleton {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: var(--space-2);
  }

  .ribbon-skeleton {
    justify-content: space-between;
  }

  .day-skeleton {
    display: flex;
    min-width: 3.75rem;
    flex: 1;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-2);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
  }

  .calendar-body-skeleton {
    display: flex;
    min-height: 24rem;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .course-list-skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .course-row-skeleton {
    display: grid;
    grid-template-columns: 4rem minmax(0, 1fr);
    gap: var(--space-4);
    min-height: 5.5rem;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    background: var(--surface-sunken);
    border-radius: var(--radius-lg);
  }

  @media (min-width: 48rem) {
    .calendar-skeleton.full {
      padding: var(--space-5) var(--space-6) var(--space-6);
    }
  }
</style>
