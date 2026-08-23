<script lang="ts">
  import { Bell, BookOpenCheck, X } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import Sheet from '$lib/components/ui/Sheet.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import type { Grade } from './types';

  type Props = {
    alerts: Grade[];
    locale: 'fr' | 'en';
    onClose: () => void;
    onOpenGrades: () => void;
  };

  let { alerts, locale, onClose, onOpenGrades }: Props = $props();

  const copy = $derived.by(() => {
    locale;
    return {
      title: m.new_grade_alerts(),
      summary: m.new_grade_alerts_summary({ count: alerts.length }),
      allGrades: m.view_all_grades(),
      close: m.close_notifications(),
      emptyTitle: m.alerts_empty(),
      emptyDescription: m.alerts_empty_description()
    };
  });

  function openGrades() {
    onOpenGrades();
    onClose();
  }
</script>

<!-- Sheet owns the dialog mechanics this drawer used to fake: focus on open, a
     focus trap, focus restore, Escape and scroll lock. -->
<Sheet title={copy.title} placement="end" closeLabel={copy.close} {onClose}>
  <div class="alert-header">
    <span class="alert-icon-plate"><Bell size={20} aria-hidden="true" /></span>
    <div class="alert-heading">
      <h2>{copy.title}</h2>
      <!-- A count line is only true when there is something to count. -->
      {#if alerts.length > 0}<p>{copy.summary}</p>{/if}
    </div>
    <IconButton variant="ghost" label={copy.close} onclick={onClose}>
      <X size={19} aria-hidden="true" />
    </IconButton>
  </div>

  {#if alerts.length > 0}
    <div class="alert-list">
      {#each alerts as grade (grade.id)}
        <article>
          <span class="alert-icon-plate item-plate">
            <BookOpenCheck size={18} aria-hidden="true" />
          </span>
          <div class="item-meta">
            <strong>{grade.subject}</strong>
            <p>{grade.label}</p>
            {#if grade.coefficient}
              <small>{m.grade_alert_coefficient({ value: grade.coefficient })}</small>
            {/if}
          </div>
          <b class="item-score">
            {grade.score}{#if grade.scale}<small>/{grade.scale}</small>{/if}
          </b>
        </article>
      {/each}
    </div>
  {:else}
    <!-- The quick action on Today opens this drawer unconditionally, so zero
         alerts is a state the drawer has to state rather than render blank. -->
    <div class="alert-empty">
      <StateCard
        kind="empty"
        icon={Bell}
        title={copy.emptyTitle}
        description={copy.emptyDescription}
      />
    </div>
  {/if}

  <div class="alert-footer">
    <Button variant="primary" block onclick={openGrades}>{copy.allGrades}</Button>
  </div>
</Sheet>

<style>
  .alert-header {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: var(--space-3);
    align-items: start;
    padding: var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
  }

  .alert-icon-plate {
    display: grid;
    width: 2.5rem;
    height: 2.5rem;
    place-items: center;
    color: var(--primary-deep);
    background: var(--muted);
    border-radius: var(--radius-md);
  }

  .alert-heading h2,
  .alert-heading p {
    margin: 0;
  }

  .alert-heading h2 {
    font-size: var(--text-lg);
    font-weight: var(--weight-heavy);
  }

  .alert-heading p {
    margin-top: var(--space-1);
    color: var(--muted-foreground);
    font-size: var(--text-base);
  }

  .alert-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
  }

  article {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: var(--space-3);
    align-items: center;
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .item-plate {
    width: 2.25rem;
    height: 2.25rem;
    border-radius: var(--radius-sm);
  }

  .item-meta strong {
    display: block;
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
  }

  .item-meta p {
    margin: var(--space-1) 0 0;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  .item-meta small {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .item-score {
    color: var(--primary-deep);
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
  }

  .item-score small {
    font-size: var(--text-xs);
  }

  .alert-empty {
    padding: var(--space-3);
  }

  .alert-footer {
    padding: var(--space-3) var(--space-4) var(--space-4);
  }

  @media (hover: hover) {
    article:hover {
      background: var(--muted);
    }
  }
</style>
