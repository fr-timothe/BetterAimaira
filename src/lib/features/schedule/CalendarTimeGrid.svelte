<script lang="ts">
  import { tick } from 'svelte';
  import * as m from '$lib/paraglide/messages.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import type { CalendarFormat } from './calendar-format.svelte';
  import {
    blockGeometry,
    layoutDay,
    ratioInWindow,
    timeWindowFor,
    windowHours,
  } from './calendar-layout';
  import type { PositionedEvent } from './calendar-layout';
  import { panel, scrollBehavior, uppercaseTiny } from './calendar-styles';
  import { getEventStatus, eventTitle, parseRoomAndTeacher } from './course-utils';
  import { isSameDay } from './date-utils';
  import type { CalendarEvent, CalendarScope } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    /** The columns to draw: one in day scope, the portal week otherwise. */
    days: Date[];
    scope: CalendarScope;
    activeDate: Date;
    now: Date;
    format: CalendarFormat;
    eventsForDay: (date: Date) => CalendarEvent[];
    onSelectDate: (date: Date) => void;
    onEventClick: (event: CalendarEvent) => void;
  };

  let { days, scope, activeDate, now, format, eventsForDay, onSelectDate, onEventClick }: Props =
    $props();

  /**
   * Row height of one hour, per scope. It is not a free aesthetic choice: at
   * 4.5rem an hour, the shortest slot the portal returns (30 min) is still
   * 36px tall, and the `min-h-(--tap-min)` floor on a block then only has to
   * stretch the rarest case instead of every second course.
   */
  const HOUR_HEIGHT_REM = { day: 5, week: 4.5 } as const;

  /** Below this a block has no room for a third line, so the room is dropped. */
  const ROOM_VISIBLE_FROM_MINUTES = 60;

  let gridScrollRef = $state<HTMLDivElement | null>(null);

  const gridEvents = $derived(days.flatMap((day) => eventsForDay(day)));
  const timeWindow = $derived(timeWindowFor(gridEvents));
  const gridHours = $derived(windowHours(timeWindow));
  const gridSpanHours = $derived((timeWindow.endMinutes - timeWindow.startMinutes) / 60);
  const hourHeightRem = $derived(scope === 'day' ? HOUR_HEIGHT_REM.day : HOUR_HEIGHT_REM.week);

  const firstEventRatio = $derived.by(() => {
    const first = gridEvents
      .map((event) => new Date(event.startsAt))
      .sort((a, b) => a.getTime() - b.getTime())[0];
    return first ? ratioInWindow(first, timeWindow) : null;
  });

  const scrolls = $derived(days.length > 1);
  /**
   * Day scope names its day in the period title and in the strip above, so a
   * third copy over a single column is noise.
   */
  const showHeaders = $derived(days.length > 1);
  const bodyRow = $derived(showHeaders ? 2 : 1);

  /**
   * The grid opens on the hour that matters rather than on its first row: now
   * when today is in view, otherwise the first course. The scroll is written on
   * the region itself so the page around it does not move.
   *
   * `days` is read here, not the scope and the anchor it is derived from: the
   * grid shows a different day as soon as the strip picks one, and a dependency
   * list naming the anchor left the day scope parked on the previous day.
   */
  $effect(() => {
    const columns = days;
    const window = timeWindow;
    const reference = now;
    // The row height decides the scrollable height the ratio is applied to.
    hourHeightRem;

    let cancelled = false;

    void tick().then(() => {
      if (cancelled) return;
      const region = gridScrollRef;
      if (!region) return;

      const ratio = columns.some((day) => isSameDay(day, reference))
        ? ratioInWindow(reference, window)
        : firstEventRatio;
      if (ratio === null) return;

      const target = ratio * region.scrollHeight - region.clientHeight / 2;
      region.scrollTo({ top: Math.max(0, target), behavior: scrollBehavior() });
    });

    return () => {
      cancelled = true;
      // A smooth scroll started by the previous run keeps travelling toward a
      // position this grid no longer shows. Re-issuing the offset it is at,
      // without animation, is the only way to stop one.
      gridScrollRef?.scrollTo({ top: gridScrollRef.scrollTop, behavior: 'auto' });
    };
  });
</script>

<!-- One course on the time axis. Its height is its duration, so what it can
     print depends on how long it lasts. -->
{#snippet eventBlock(block: PositionedEvent)}
  {@const event = block.event}
  {@const geometry = blockGeometry(block, timeWindow)}
  {@const status = getEventStatus(event)}
  {@const details = parseRoomAndTeacher(event)}
  {@const minutes = block.toMinutes - block.fromMinutes}
  {@const live = status === 'live'}

  <button
    type="button"
    class={cn(
      // One block on the time grid. Elevation is the border; hover adds the lift.
      'absolute flex min-h-(--tap-min) min-w-0 flex-col gap-[0.1rem] overflow-hidden rounded-sm' +
        ' border border-l-[3px] px-1.5 py-1 text-start transition-control' +
        ' active:scale-(--press-scale) hover:border-primary-deep hover:shadow-sm',
      live
        ? 'border-primary-deep bg-muted'
        : status === 'finished'
          ? 'border-border-subtle border-l-border bg-surface-sunken'
          : 'border-border-subtle border-l-primary-deep bg-card'
    )}
    style:top={`${geometry.top}%`}
    style:height={`${geometry.height}%`}
    style:left={`calc(${geometry.left}% + ${geometry.left > 0 ? '2px' : '0px'})`}
    style:width={`calc(${geometry.width}% - 2px)`}
    onclick={() => onEventClick(event)}
  >
    <span
      class={cn(
        'text-2xs font-bold tabular-nums',
        status === 'finished' ? 'text-muted-foreground' : 'text-primary-deep'
      )}
    >
      {format.eventTimeRange(event)}
    </span>
    <span
      class={cn(
        'min-w-0 text-xs leading-[1.25] font-extrabold wrap-anywhere',
        status === 'finished' ? 'text-muted-foreground' : 'text-foreground'
      )}
    >
      {eventTitle(event)}
    </span>
    {#if minutes >= ROOM_VISIBLE_FROM_MINUTES && details.room}
      <span class="min-w-0 text-2xs wrap-anywhere text-muted-foreground">{details.room}</span>
    {/if}
    {#if live}
      <span class="mt-auto"><Badge tone="live" dot>{m.schedule_status_live()}</Badge></span>
    {/if}
  </button>
{/snippet}

<!-- The grid itself. Day scope passes one column, week scope the portal week;
     the header row and the hour gutter live in the same grid so they can never
     drift out of alignment with the columns. -->
<!-- One element scrolls both axes. Splitting them made the horizontal wrapper
     a scrollport of its own, and the day headers then stuck to its top — that
     is, to the top of the whole grid — instead of to the visible edge. -->
<div
  class={cn(
    panel,
    // No inline padding on the scrollport: whatever sits in it is scrolled
    // content, and a course would show through the strip of card to the left
    // of the pinned hour scale. The trailing gap moves onto the grid, where
    // it travels with the columns instead.
    'relative overflow-auto pb-2 md:pb-3',
    // The grid takes the height the header and the strip leave it, down to a
    // floor below which the page scrolls instead of squeezing the hours.
    'min-h-[18rem] flex-1',
    // The gesture stays inside the grid: without this, reaching its end hands
    // the swipe to the viewport, which reads it as a pull to refresh.
    'overscroll-contain',
    'scrollbar-none [&::-webkit-scrollbar]:hidden'
  )}
  bind:this={gridScrollRef}
>
  <div
    class="grid gap-x-1.5 pe-2 md:pe-3"
    style:--hour-height={`${hourHeightRem}rem`}
    style:--gutter-width="3.25rem"
    style:grid-template-columns={`var(--gutter-width) repeat(${days.length}, minmax(${scrolls ? '8.5rem' : '0'}, 1fr))`}
    style:grid-template-rows={
      showHeaders
        ? `auto calc(var(--hour-height) * ${gridSpanHours})`
        : `calc(var(--hour-height) * ${gridSpanHours})`
    }
    role="group"
    aria-label={m.calendar_grid_label({ period: format.periodLabel })}
  >
    <!-- Row 1: the day headers, over one backdrop so nothing shows through
         the column gaps while the band scrolls under them. -->
    {#if showHeaders}
      <div
        class="sticky top-0 z-sticky col-start-1 col-end-[-1] row-start-1 bg-card"
        aria-hidden="true"
      ></div>
    {/if}

    {#each showHeaders ? days : [] as day, index (day.toISOString())}
      {@const isDayToday = isSameDay(day, now)}
      {@const isDayActive = isSameDay(day, activeDate)}
      <button
        type="button"
        class={cn(
          'sticky top-0 z-sticky mb-1 flex min-h-(--tap-min) flex-col items-center',
          'justify-center gap-[0.1rem] rounded-md border px-1 py-1 transition-control',
          'active:scale-(--press-scale) hover:border-primary-deep',
          isDayToday
            ? 'border-primary-deep bg-muted text-primary-deep'
            : 'border-transparent bg-surface-sunken text-muted-foreground',
          isDayActive && !isDayToday && 'border-border text-foreground'
        )}
        style:grid-column={index + 2}
        style:grid-row="1"
        aria-pressed={isDayActive}
        onclick={() => onSelectDate(day)}
      >
        <span class="flex items-baseline gap-1">
          <span class={uppercaseTiny}>{format.weekdayShortFormatter.format(day)}</span>
          <span class="text-md font-extrabold tabular-nums">{day.getDate()}</span>
        </span>
        <span class="text-2xs font-semibold"
          >{format.dayCountLabel(eventsForDay(day).length)}</span
        >
      </button>
    {/each}

    <!-- The corner above the hour scale, masked the same way: it comes after
         the headers so it paints over the one scrolling underneath it. -->
    {#if showHeaders}
      <div
        class="pointer-events-none sticky top-0 z-sticky col-start-1 col-end-[-1] row-start-1"
        aria-hidden="true"
      >
        <div class="sticky start-0 h-full w-(--gutter-width) bg-card"></div>
      </div>
    {/if}

    <!-- The hour scale, pinned while the columns scroll sideways. A sticky
         grid item may only travel inside its own grid area, so pinning it
         on the first column let it slide off after one gutter width; the
         strip is instead a sticky child of a wrapper spanning every column,
         which is the box it is allowed to travel across. The wrapper takes
         no pointer events, so the columns it covers stay clickable. -->
    <div
      class="pointer-events-none relative z-sticky col-start-1 col-end-[-1]"
      style:grid-row={bodyRow}
    >
      <div class="sticky start-0 h-full w-(--gutter-width) bg-card">
        {#each gridHours as hour, index (hour)}
          <span
            class={cn(
              'absolute end-1.5 text-2xs font-semibold tabular-nums text-muted-foreground',
              index === 0 ? 'translate-y-0' : '-translate-y-1/2'
            )}
            style:top={`${(index / gridSpanHours) * 100}%`}
          >
            {format.timeFormatter.format(new Date(2024, 0, 1, hour, 0))}
          </span>
        {/each}
      </div>
    </div>

    <!-- The hour rules, drawn once across every column. -->
    <div
      class="pointer-events-none relative col-start-2 col-end-[-1]"
      style:grid-row={bodyRow}
      aria-hidden="true"
    >
      {#each gridHours as hour, index (hour)}
        <span
          class="absolute inset-x-0 border-t border-border-subtle"
          style:top={`${(index / gridSpanHours) * 100}%`}
        ></span>
      {/each}
    </div>

    {#each days as day, index (day.toISOString())}
      {@const dayEvents = eventsForDay(day)}
      {@const isDayToday = isSameDay(day, now)}
      {@const marker = isDayToday ? ratioInWindow(now, timeWindow) : null}
      <div class="relative rounded-md" style:grid-column={index + 2} style:grid-row={bodyRow}>
        {#each layoutDay(dayEvents) as block (block.event.id)}
          {@render eventBlock(block)}
        {/each}

        {#if marker !== null}
          <div
            class="pointer-events-none absolute inset-x-0 z-raised border-t-2 border-primary-deep"
            style:top={`${marker * 100}%`}
          >
            <span
              class="absolute end-0 -translate-y-1/2 rounded-pill bg-primary-deep px-1.5
                     text-2xs font-bold tabular-nums text-card"
            >
              {format.timeFormatter.format(now)}
            </span>
            <span class="sr-only">{m.calendar_now()}</span>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>
