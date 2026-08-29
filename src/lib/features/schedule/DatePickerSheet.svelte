<script lang="ts">
  import { untrack } from 'svelte';
  import { CalendarCheck, ChevronLeft, ChevronRight } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import Sheet from '$lib/components/ui/Sheet.svelte';
  import { monthHeaderDays } from './calendar-format.svelte';
  import type { CalendarFormat } from './calendar-format.svelte';
  import { monthGridDays } from './calendar-navigation.svelte';
  import { monthCellBtn, uppercaseTiny } from './calendar-styles';
  import { addMonths, capitalizeFirst, isSameDay, isSameMonth, startOfMonth } from './date-utils';
  import type { CalendarEvent } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    activeDate: Date;
    now: Date;
    format: CalendarFormat;
    eventsForDay: (date: Date) => CalendarEvent[];
    onPick: (date: Date) => void;
    onClose: () => void;
  };

  let { activeDate, now, format, eventsForDay, onPick, onClose }: Props = $props();

  /**
   * The month the sheet browses, which is deliberately not the calendar's:
   * paging through it must not move the view behind the sheet before a day is
   * actually picked. It opens on the selected day — read once, hence the
   * `untrack` — and dies with the sheet.
   */
  let pickerMonth = $state(untrack(() => startOfMonth(activeDate)));

  const pickerDays = $derived(monthGridDays(pickerMonth));
</script>

<Sheet title={m.calendar_pick_date_title()} closeLabel={m.close()} {onClose}>
  <div class="flex flex-col gap-3 p-4">
    <div class="flex items-center justify-between gap-2">
      <IconButton
        label={m.previous_period()}
        onclick={() => (pickerMonth = addMonths(pickerMonth, -1))}
      >
        <ChevronLeft size={18} strokeWidth={2.2} aria-hidden="true" />
      </IconButton>
      <strong class="text-md font-extrabold"
        >{capitalizeFirst(format.monthYearFormatter.format(pickerMonth))}</strong
      >
      <IconButton
        label={m.next_period()}
        onclick={() => (pickerMonth = addMonths(pickerMonth, 1))}
      >
        <ChevronRight size={18} strokeWidth={2.2} aria-hidden="true" />
      </IconButton>
    </div>

    <div
      class={cn(
        'grid grid-cols-7 text-center text-muted-foreground',
        uppercaseTiny,
        'tracking-normal'
      )}
      aria-hidden="true"
    >
      {#each monthHeaderDays as day (day.getTime())}
        <span>{format.weekdayShortFormatter.format(day)}</span>
      {/each}
    </div>

    <div class="grid grid-cols-7 gap-1">
      {#each pickerDays as day (day.toISOString())}
        {@const dayEvents = eventsForDay(day)}
        {@const isDayInMonth = isSameMonth(day, pickerMonth)}
        {@const isDayToday = isSameDay(day, now)}
        {@const isDaySelected = isSameDay(day, activeDate)}
        <button
          type="button"
          class={cn(
            monthCellBtn,
            isDaySelected
              ? 'border-primary-deep bg-muted text-primary-deep'
              : cn(
                  isDayInMonth ? 'bg-card' : 'bg-surface-sunken text-muted-foreground',
                  isDayToday ? 'border-primary-deep' : 'border-border-subtle'
                )
          )}
          aria-label={m.calendar_day_cell_label({
            date: format.dayFormatter.format(day),
            courses: format.dayCountLabel(dayEvents.length),
          })}
          onclick={() => onPick(day)}
        >
          <span class="text-sm font-bold tabular-nums">{day.getDate()}</span>
          {#if dayEvents.length > 0}
            <span class="size-[0.3rem] rounded-full bg-primary-deep" aria-hidden="true"></span>
          {/if}
        </button>
      {/each}
    </div>

    <Button variant="outline" block onclick={() => onPick(new Date())}>
      <CalendarCheck size={16} aria-hidden="true" />
      <span>{m.go_to_today()}</span>
    </Button>
  </div>
</Sheet>
