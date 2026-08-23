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

  const plate = 'grid place-items-center bg-muted text-primary-deep';

  function openGrades() {
    onOpenGrades();
    onClose();
  }
</script>

<!-- Sheet owns the dialog mechanics this drawer used to fake: focus on open, a
     focus trap, focus restore, Escape and scroll lock. -->
<Sheet title={copy.title} placement="end" closeLabel={copy.close} {onClose}>
  <div class="grid grid-cols-[auto_1fr_auto] items-start gap-3 border-b border-border-subtle p-4">
    <span class="{plate} size-10 rounded-md"><Bell size={20} aria-hidden="true" /></span>
    <div>
      <h2 class="text-lg font-extrabold">{copy.title}</h2>
      <!-- A count line is only true when there is something to count. -->
      {#if alerts.length > 0}
        <p class="mt-1 text-base text-muted-foreground">{copy.summary}</p>
      {/if}
    </div>
    <IconButton variant="ghost" label={copy.close} onclick={onClose}>
      <X size={19} aria-hidden="true" />
    </IconButton>
  </div>

  {#if alerts.length > 0}
    <div class="flex flex-col gap-2 p-3">
      {#each alerts as grade (grade.id)}
        <article
          class="grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-md bg-surface-sunken p-3
                 transition-colors duration-fast ease-out hover:bg-muted"
        >
          <span class="{plate} size-9 rounded-sm">
            <BookOpenCheck size={18} aria-hidden="true" />
          </span>
          <div>
            <strong class="block text-base font-bold">{grade.subject}</strong>
            <p class="mt-1 text-sm text-muted-foreground">{grade.label}</p>
            {#if grade.coefficient}
              <small class="text-xs text-muted-foreground"
                >{m.grade_alert_coefficient({ value: grade.coefficient })}</small
              >
            {/if}
          </div>
          <b class="text-xl font-extrabold tabular-nums text-primary-deep">
            {grade.score}{#if grade.scale}<small class="text-xs">/{grade.scale}</small>{/if}
          </b>
        </article>
      {/each}
    </div>
  {:else}
    <!-- The quick action on Today opens this drawer unconditionally, so zero
         alerts is a state the drawer has to state rather than render blank. -->
    <div class="p-3">
      <StateCard
        kind="empty"
        icon={Bell}
        title={copy.emptyTitle}
        description={copy.emptyDescription}
      />
    </div>
  {/if}

  <div class="px-4 pt-3 pb-4">
    <Button variant="primary" block onclick={openGrades}>{copy.allGrades}</Button>
  </div>
</Sheet>
