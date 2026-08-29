<script lang="ts">
  import { tick } from 'svelte';
  import type { CalendarFormat } from './calendar-format.svelte';
  import { scrollBehavior, uppercaseTiny } from './calendar-styles';
  import { isSameDay } from './date-utils';
  import type { CalendarEvent } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    days: Date[];
    activeDate: Date;
    now: Date;
    format: CalendarFormat;
    eventsForDay: (date: Date) => CalendarEvent[];
    onSelect: (date: Date) => void;
  };

  let { days, activeDate, now, format, eventsForDay, onSelect }: Props = $props();

  /**
   * The selected button is held by the binding rather than looked up by its
   * `aria-pressed` attribute: an ARIA value is what the strip tells assistive
   * technology, not a handle it should be reading its own DOM back through.
   */
  let dayElements: (HTMLButtonElement | null)[] = [];

  $effect(() => {
    const selected = activeDate;
    const strip = days;

    let cancelled = false;

    void tick().then(() => {
      if (cancelled) return;
      const index = strip.findIndex((day) => isSameDay(day, selected));
      dayElements[index]?.scrollIntoView({
        behavior: scrollBehavior(),
        block: 'nearest',
        inline: 'center',
      });
    });

    return () => {
      cancelled = true;
    };
  });

  const stripDayBtn =
    'flex min-h-18 min-w-14 flex-1 basis-0 flex-col items-center justify-center gap-[0.15rem]' +
    ' rounded-lg border bg-card px-1 py-2 transition-control active:scale-(--press-scale)';
</script>

<div
  class="flex gap-2 overflow-x-auto p-1 scrollbar-none [-webkit-overflow-scrolling:touch]
         [&::-webkit-scrollbar]:hidden"
>
  {#each days as day, index (day.toISOString())}
    {@const dayEventsCount = eventsForDay(day).length}
    {@const isDayToday = isSameDay(day, now)}
    {@const isDaySelected = isSameDay(day, activeDate)}

    <button
      type="button"
      class={cn(
        stripDayBtn,
        isDaySelected
          ? 'border-primary-deep bg-muted text-primary-deep'
          : cn(
              'text-muted-foreground hover:border-border hover:text-foreground',
              isDayToday ? 'border-primary-deep' : 'border-border-subtle'
            )
      )}
      aria-pressed={isDaySelected}
      onclick={() => onSelect(day)}
      bind:this={dayElements[index]}
    >
      <span class={uppercaseTiny}>{format.weekdayShortFormatter.format(day)}</span>
      <span class="text-xl leading-[1.2] font-extrabold tabular-nums">{day.getDate()}</span>
      <span class="text-2xs font-semibold tabular-nums"
        >{dayEventsCount > 0 ? dayEventsCount : '·'}</span
      >
    </button>
  {/each}
</div>
