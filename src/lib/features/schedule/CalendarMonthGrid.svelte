<script lang="ts">
  import { tick } from 'svelte';
  import { ExternalLink, MapPin, UserRound } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import { monthHeaderDays } from './calendar-format.svelte';
  import type { CalendarFormat } from './calendar-format.svelte';
  import { monthGridDays, monthKeyTarget } from './calendar-navigation.svelte';
  import { monthCellBtn, panel, uppercaseTiny } from './calendar-styles';
  import {
    eventDurationMinutes,
    eventSecondary,
    eventTitle,
    formatDuration,
    getEventStatus,
    openExternalUrl,
    parseRoomAndTeacher,
  } from './course-utils';
  import { capitalizeFirst, isSameDay, isSameMonth } from './date-utils';
  import type { CalendarEvent } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    anchorDate: Date;
    activeDate: Date;
    /** The only cell carrying `tabindex="0"`; see `CalendarNavigation`. */
    monthFocusDate: Date;
    now: Date;
    locale: Locale;
    format: CalendarFormat;
    eventsForDay: (date: Date) => CalendarEvent[];
    onSelectDate: (date: Date) => void;
    onFocusDate: (date: Date) => void;
    onEventClick: (event: CalendarEvent) => void;
    onOpenTempo?: (event: CalendarEvent) => void;
  };

  let {
    anchorDate,
    activeDate,
    monthFocusDate,
    now,
    locale,
    format,
    eventsForDay,
    onSelectDate,
    onFocusDate,
    onEventClick,
    onOpenTempo,
  }: Props = $props();

  /**
   * The weeks, not a flat run of cells: `role="grid"` without a `role="row"`
   * layer is invalid, and the position an assistive technology announces comes
   * from that layer. `contents` keeps the seven columns the wrapper would
   * otherwise break.
   */
  const monthWeeks = $derived.by(() => {
    const days = monthGridDays(anchorDate);
    return Array.from({ length: days.length / 7 }, (_, week) =>
      days.slice(week * 7, week * 7 + 7)
    );
  });

  /**
   * The cells, in the same flat order, so the keyboard can move focus without
   * asking the DOM to find a button by attribute.
   */
  let cellElements: (HTMLButtonElement | null)[] = [];

  const activeDateEvents = $derived(eventsForDay(activeDate));
  const activeDateDurationMinutes = $derived.by(() =>
    activeDateEvents.reduce((total, event) => total + eventDurationMinutes(event), 0)
  );

  /**
   * Arrow keys walk the month, so the grid costs one tab stop instead of 42.
   * Moving out of the displayed month moves the month with it, which is what
   * makes the keyboard path equivalent to the pointer one.
   */
  function handleMonthKeydown(event: KeyboardEvent) {
    const next = monthKeyTarget(event.key, monthFocusDate);
    if (!next) return;

    event.preventDefault();
    onFocusDate(next);

    void tick().then(() => {
      const index = monthWeeks.flat().findIndex((day) => isSameDay(day, next));
      cellElements[index]?.focus();
    });
  }

  async function handleTempoClick(clickEvent: MouseEvent, event: CalendarEvent) {
    clickEvent.stopPropagation();
    if (onOpenTempo) {
      onOpenTempo(event);
      return;
    }
    await openExternalUrl(event.tempoUrl);
  }

  const detailRow =
    'grid grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-lg border p-3' +
    ' transition-control active:scale-(--press-scale) hover:border-primary-deep hover:shadow-sm';

  const tempoBtn =
    'inline-flex min-h-(--tap-min) items-center gap-1 rounded-pill border border-muted-strong' +
    ' bg-card px-3 text-xs font-bold text-primary-deep transition-control' +
    ' active:scale-(--press-scale) hover:bg-muted';
</script>

<!-- The one list-shaped row left: the selected day beside the month grid. -->
{#snippet courseRow(event: CalendarEvent)}
  {@const status = getEventStatus(event)}
  {@const details = parseRoomAndTeacher(event)}
  {@const secondary = eventSecondary(event)}
  {@const live = status === 'live'}

  <div
    class={cn(
      detailRow,
      live ? 'border-primary-deep bg-muted' : 'border-border-subtle bg-surface-sunken'
    )}
  >
    <button
      type="button"
      class="grid w-full cursor-pointer grid-cols-[3.5rem_3px_minmax(0,1fr)] items-stretch gap-2.5
             rounded-sm bg-transparent p-0 text-start text-inherit
             lte-600:grid-cols-[3.25rem_3px_minmax(0,1fr)] lte-600:gap-2"
      onclick={() => onEventClick(event)}
    >
      <span class="flex min-w-0 flex-col pt-[0.15rem] tabular-nums">
        <strong class="text-base font-extrabold text-foreground"
          >{format.timeFormatter.format(new Date(event.startsAt))}</strong
        >
        <span class="text-xs text-muted-foreground"
          >{format.timeFormatter.format(new Date(event.endsAt))}</span
        >
      </span>

      <span class="relative flex h-full flex-col items-center">
        <span class="h-full w-[3px] rounded-pill bg-primary-deep"></span>
      </span>

      <span class="flex min-w-0 flex-col gap-1">
        {#if live || event.kind}
          <span class="flex min-w-0 max-w-full flex-wrap items-center gap-2">
            {#if live}<Badge tone="live" dot>{m.schedule_status_live()}</Badge>{/if}
            {#if event.kind}<KindBadge {event} />{/if}
          </span>
        {/if}

        <span class="min-w-0 text-base leading-[1.3] font-bold wrap-anywhere text-foreground"
          >{eventTitle(event)}</span
        >

        {#if secondary}
          <span class="min-w-0 text-xs wrap-anywhere text-muted-foreground">{secondary}</span>
        {/if}

        {#if details.room || details.teacher}
          <span
            class="flex min-w-0 max-w-full flex-wrap items-center gap-x-3 gap-y-1 text-xs
                   text-muted-foreground"
          >
            {#if details.room}
              <span class="inline-flex min-w-0 items-center gap-1 wrap-anywhere">
                <MapPin size={14} aria-hidden="true" />{details.room}
              </span>
            {/if}
            {#if details.teacher}
              <span class="inline-flex min-w-0 items-center gap-1 wrap-anywhere">
                <UserRound size={14} aria-hidden="true" />{details.teacher}
              </span>
            {/if}
          </span>
        {/if}
      </span>
    </button>

    {#if event.tempoUrl}
      <button
        type="button"
        class={tempoBtn}
        aria-label={m.open_tempo()}
        title={m.open_tempo()}
        onclick={(e) => handleTempoClick(e, event)}
      >
        <ExternalLink size={14} aria-hidden="true" />
        <span>Tempo</span>
      </button>
    {/if}
  </div>
{/snippet}

<section class="grid grid-cols-1 gap-4 min-[54rem]:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
  <div class={cn(panel, 'p-4 md:p-5')}>
    <div
      class={cn(
        'mb-3 grid grid-cols-7 text-center text-muted-foreground',
        uppercaseTiny,
        'tracking-normal'
      )}
      aria-hidden="true"
    >
      {#each monthHeaderDays as day (day.getTime())}
        <span>{format.weekdayShortFormatter.format(day)}</span>
      {/each}
    </div>

    <!-- One tab stop: the arrow keys walk the grid from the focused cell. -->
    <div
      class="grid grid-cols-7 gap-1"
      role="grid"
      aria-label={m.calendar_month_grid_label({ period: format.periodLabel })}
    >
      {#each monthWeeks as week, weekIndex (week[0].toISOString())}
        <div class="contents" role="row">
          {#each week as day, dayIndex (day.toISOString())}
            {@const dayEvents = eventsForDay(day)}
            {@const isDayInMonth = isSameMonth(day, anchorDate)}
            {@const isDayToday = isSameDay(day, now)}
            {@const isDaySelected = isSameDay(day, activeDate)}

            <button
              type="button"
              role="gridcell"
              tabindex={isSameDay(day, monthFocusDate) ? 0 : -1}
              class={cn(
                monthCellBtn,
                isDaySelected
                  ? 'border-primary-deep bg-muted'
                  : cn(
                      isDayInMonth ? 'bg-card' : 'bg-surface-sunken',
                      isDayToday ? 'border-primary-deep' : 'border-border-subtle'
                    )
              )}
              aria-selected={isDaySelected}
              aria-label={m.calendar_day_cell_label({
                date: format.dayFormatter.format(day),
                courses: format.dayCountLabel(dayEvents.length),
              })}
              onclick={() => onSelectDate(day)}
              onkeydown={handleMonthKeydown}
              bind:this={cellElements[weekIndex * 7 + dayIndex]}
            >
              <span
                class={cn(
                  'text-sm tabular-nums',
                  isDayToday || isDaySelected
                    ? 'font-extrabold text-primary-deep'
                    : isDayInMonth
                      ? 'font-bold text-foreground'
                      : 'font-medium text-muted-foreground'
                )}>{day.getDate()}</span
              >

              <!-- Density: the bar is scanned, the count is read. -->
              <span class="flex h-1 w-full items-center px-1" aria-hidden="true">
                {#if dayEvents.length > 0}
                  <span
                    class="h-1 rounded-pill bg-primary-deep"
                    style:width={`${(Math.min(dayEvents.length, 4) / 4) * 100}%`}
                  ></span>
                {/if}
              </span>

              <span
                class="text-2xs leading-none font-bold tabular-nums text-muted-foreground"
                aria-hidden="true">{dayEvents.length > 0 ? dayEvents.length : ''}</span
              >
            </button>
          {/each}
        </div>
      {/each}
    </div>
  </div>

  <div class={cn(panel, 'flex flex-col gap-3 p-4 min-[54rem]:p-5')}>
    <header
      class="flex items-center justify-between gap-3 border-b border-border-subtle pb-3
             lte-600:items-start"
    >
      <div class="min-w-0">
        <h3 class="mb-[0.15rem] text-lg font-extrabold wrap-anywhere"
          >{capitalizeFirst(format.dayFormatter.format(activeDate))}</h3
        >
        <p class="text-xs text-muted-foreground">
          {m.day_course_count({ count: activeDateEvents.length })}
          {#if activeDateDurationMinutes > 0}
            · {formatDuration(activeDateDurationMinutes, locale)}
          {/if}
        </p>
      </div>
      {#if isSameDay(activeDate, now)}
        <Badge tone="accent">{m.preview_today()}</Badge>
      {/if}
    </header>

    <div class="flex flex-col gap-2.5 min-[54rem]:max-h-[28rem] min-[54rem]:overflow-y-auto">
      {#if activeDateEvents.length > 0}
        {#each activeDateEvents as event (event.id)}
          {@render courseRow(event)}
        {/each}
      {:else}
        <p
          class="rounded-lg bg-surface-sunken px-4 py-6 text-center text-base
                 text-muted-foreground">{m.no_courses_day_description()}</p
        >
      {/if}
    </div>
  </div>
</section>
