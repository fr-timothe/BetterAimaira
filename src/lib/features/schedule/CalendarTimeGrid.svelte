<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import type { CalendarFormat } from './calendar-format.svelte';
  import { categorySurface, spentSurface } from './category-tone';
  import {
    blockGeometry,
    layoutDay,
    ratioInWindow,
    timeWindowFor,
    windowHours,
  } from './calendar-layout';
  import type { PositionedEvent } from './calendar-layout';
  import { panel, uppercaseTiny } from './calendar-styles';
  import {
    courseCategory,
    getEventStatus,
    eventTitle,
    isCancelled,
    parseRoomAndTeacher,
  } from './course-utils';
  import { isSameDay } from './date-utils';
  import type { CalendarEvent, CalendarScope } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    /** The columns to draw: one at day zoom, the portal week at week zoom. */
    days: Date[];
    scope: CalendarScope;
    activeDate: Date;
    now: Date;
    format: CalendarFormat;
    eventsForDay: (date: Date) => CalendarEvent[];
    /** Week zoom only: a column header zooms the grid onto that day. */
    onZoomDay: (date: Date) => void;
    onEventClick: (event: CalendarEvent) => void;
  };

  let { days, scope, activeDate, now, format, eventsForDay, onZoomDay, onEventClick }: Props =
    $props();

  /**
   * The grid is laid out to fit the height it was given, not to a fixed hour
   * height that then has to be scrolled. That single change is what removes
   * the week's second scroll axis: with the whole band on screen the six
   * columns can share the width instead of demanding a minimum each, so
   * nothing scrolls sideways and the horizontal swipe is free to mean
   * "change period" at every zoom.
   *
   * These are the floors below which hours stop being readable and the band
   * starts scrolling instead of squeezing — a fourteen-hour day still has to
   * be usable. Day zoom's floor is higher because a block there carries three
   * lines of text.
   */
  const MIN_HOUR_REM = { day: 3, week: 2 } as const;

  const gridEvents = $derived(days.flatMap((day) => eventsForDay(day)));
  const timeWindow = $derived(timeWindowFor(gridEvents));
  const gridHours = $derived(windowHours(timeWindow));
  const gridSpanHours = $derived((timeWindow.endMinutes - timeWindow.startMinutes) / 60);
  const minBodyRem = $derived(MIN_HOUR_REM[scope === 'day' ? 'day' : 'week'] * gridSpanHours);

  /** Week zoom names its days in the header row; day zoom names its day above the grid. */
  const showHeaders = $derived(days.length > 1);

  /**
   * What a block can print is decided by how wide its column actually is, not
   * by how many columns there are. Reading it off the column count made a
   * desktop week — where a column is 190px — print the same stripped block as
   * a phone, and dropped the room from a card with room to spare.
   */
  const GUTTER_REM = 2.5;
  const COLUMN_GAP_PX = 4;
  /** Below this a course name has nowhere to go and the hour has to stand alone. */
  const DENSE_BELOW_PX = 110;
  /** Below this even the hour truncates, and the field carries the block alone. */
  const NAMED_FROM_PX = 60;

  let gridWidth = $state(0);
  let rootFontSize = $state(16);

  $effect(() => {
    const read = () => {
      rootFontSize = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
    };
    read();
    // Text zoom moves the gutter, which moves every column with it.
    window.addEventListener('resize', read);
    return () => window.removeEventListener('resize', read);
  });

  const columnWidthPx = $derived.by(() => {
    if (gridWidth === 0 || days.length === 0) return 0;
    const gutter = GUTTER_REM * rootFontSize;
    const gaps = COLUMN_GAP_PX * days.length;
    return Math.max(0, (gridWidth - gutter - gaps) / days.length);
  });

  /**
   * Before the first measurement the column count is the only thing known, and
   * guessing dense on a phone is the cheaper mistake: one frame of a stripped
   * block, rather than one frame of text spilling out of a 53px column.
   */
  const dense = $derived(
    columnWidthPx === 0 ? days.length > 1 : columnWidthPx < DENSE_BELOW_PX
  );

  function isNamed(lanes: number): boolean {
    if (!dense) return true;
    if (columnWidthPx === 0) return lanes === 1;
    return columnWidthPx / lanes >= NAMED_FROM_PX;
  }

  const columnTemplate = $derived(
    `var(--gutter-width) repeat(${days.length}, minmax(0, 1fr))`
  );
</script>

<!-- One course on the time axis. What it can print is decided by the zoom, not
     by a breakpoint: the same block draws three lines at day zoom and one at
     week zoom, and drops to a coloured field when a shared column leaves it
     twenty-odd pixels of width. -->
{#snippet eventBlock(block: PositionedEvent)}
  {@const event = block.event}
  {@const geometry = blockGeometry(block, timeWindow)}
  {@const status = getEventStatus(event, now)}
  {@const details = parseRoomAndTeacher(event)}
  {@const cancelled = isCancelled(event)}
  {@const live = status === 'live' && !cancelled}
  {@const spent = status === 'finished'}
  {@const named = isNamed(block.lanes)}

  <button
    type="button"
    class={cn(
      'absolute flex min-h-(--tap-min) min-w-0 flex-col overflow-hidden rounded-xs border',
      'text-start transition-control active:scale-(--press-scale)',
      'fine-hover:border-primary-deep fine-hover:shadow-sm',
      dense ? 'gap-0 px-1 py-0.5' : 'gap-[0.15rem] px-2 py-1.5',
      spent ? spentSurface : categorySurface(courseCategory(event.kind)),
      live ? 'border-primary-deep' : 'border-transparent',
      cancelled && 'border-danger-strong border-dashed'
    )}
    style:top={`${geometry.top}%`}
    style:height={`${geometry.height}%`}
    style:left={`calc(${geometry.left}% + ${geometry.left > 0 ? '2px' : '0px'})`}
    style:width={`calc(${geometry.width}% - 2px)`}
    aria-label={named
      ? undefined
      : `${eventTitle(event)}, ${format.eventTimeRange(event)}${details.room ? `, ${details.room}` : ''}`}
    onclick={() => onEventClick(event)}
  >
    <!-- The live badge rides beside the hour rather than at the block's bottom.
         Pinned there it sat exactly where the now-line crosses a running
         course, and a status chip with a rule struck through it reads as a
         rendering bug. -->
    <span class="flex min-w-0 items-center gap-1.5">
      <span class="shrink-0 text-2xs leading-[1.15] font-bold tabular-nums">
        <!-- A shared column is about 25px wide, where `08:00` truncates to
             `08:0` and reads as a defect. The compact hour fits and stays true. -->
        {named
          ? format.timeFormatter.format(new Date(event.startsAt))
          : format.hourFormatter.format(new Date(event.startsAt))}
      </span>
      {#if !dense && live}
        <Badge tone="live" dot>{m.schedule_status_live()}</Badge>
      {/if}
    </span>

    {#if named}
      <span
        class={cn(
          'min-w-0 font-extrabold',
          dense ? 'text-2xs leading-[1.15] hyphens-auto' : 'text-xs leading-[1.25] wrap-anywhere',
          cancelled && 'line-through'
        )}>{eventTitle(event)}</span
      >
    {/if}

    {#if !dense && details.room}
      <span class="min-w-0 truncate text-2xs leading-[1.2] font-medium">{details.room}</span>
    {/if}
  </button>
{/snippet}

<!-- Day zoom has one column, and on a wide window that column was 1200px of
     mostly empty band with three short lines pinned to its left edge. It takes
     a measure, the way a paragraph does; the week has six columns to spend the
     width on and keeps all of it. -->
<div
  class={cn(
    panel,
    'flex min-h-0 flex-1 flex-col overflow-hidden p-2 md:p-3',
    scope === 'day' && 'md:mx-auto md:w-full md:max-w-[56rem]'
  )}
>
  {#if showHeaders}
    <!-- The header row lives in its own grid rather than inside the body's, so
         the body can be the only thing that scrolls when a wide band forces it
         to. Both grids share one column template, which is what keeps a header
         over its column. -->
    <div
      class="grid shrink-0 gap-1 pb-1"
      style:--gutter-width="2.5rem"
      style:grid-template-columns={columnTemplate}
    >
      <span></span>
      {#each days as day (day.toISOString())}
        {@const isDayToday = isSameDay(day, now)}
        {@const isDayActive = isSameDay(day, activeDate)}
        <button
          type="button"
          class={cn(
            'flex min-h-9 flex-col items-center justify-center rounded-sm border',
            'transition-control active:scale-(--press-scale) fine-hover:border-primary-deep',
            isDayToday
              ? 'border-primary-deep bg-muted text-primary-deep'
              : isDayActive
                ? 'border-border bg-surface-sunken text-foreground'
                : 'border-transparent bg-surface-sunken text-muted-foreground'
          )}
          aria-label={m.calendar_zoom_day({ day: format.dayFormatter.format(day) })}
          onclick={() => onZoomDay(day)}
        >
          <span class={cn(uppercaseTiny, 'leading-none')}
            >{format.weekdayShortFormatter.format(day).slice(0, 2)}</span
          >
          <span class="text-xs leading-tight font-extrabold tabular-nums">{day.getDate()}</span>
        </button>
      {/each}
    </div>
  {/if}

  <div
    class={cn(
      'min-h-0 flex-1 overflow-y-auto overscroll-contain',
      'scrollbar-none [&::-webkit-scrollbar]:hidden'
    )}
    style:--min-body={`${minBodyRem}rem`}
  >
    <div
      class="grid h-full min-h-(--min-body) gap-1"
      style:--gutter-width="2.5rem"
      bind:clientWidth={gridWidth}
      style:grid-template-columns={columnTemplate}
      role="group"
      aria-label={m.calendar_grid_label({ period: format.periodLabel })}
    >
      <!-- The hour scale. The first and last labels are pulled inside the band:
           a half line hanging off the bottom is eight pixels of scroll, and
           this grid's whole claim is that there is none. -->
      <div class="relative" style:grid-column="1" style:grid-row="1">
        {#each gridHours as hour, index (hour)}
          <span
            class={cn(
              'absolute end-1.5 text-2xs font-semibold tabular-nums text-muted-foreground',
              index === 0
                ? 'translate-y-0'
                : index === gridHours.length - 1
                  ? '-translate-y-full'
                  : '-translate-y-1/2'
            )}
            style:top={`${(index / gridSpanHours) * 100}%`}
          >
            {format.timeFormatter.format(new Date(2024, 0, 1, hour, 0))}
          </span>
        {/each}
      </div>

      <!-- The hour rules, drawn once across every column. -->
      <div
        class="pointer-events-none relative col-start-2 col-end-[-1]"
        style:grid-row="1"
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
        {@const marker = isSameDay(day, now) ? ratioInWindow(now, timeWindow) : null}
        <div class="relative min-w-0" style:grid-column={index + 2} style:grid-row="1">
          {#each layoutDay(eventsForDay(day)) as block (block.event.id)}
            {@render eventBlock(block)}
          {/each}

          {#if marker !== null}
            <div
              class="pointer-events-none absolute inset-x-0 z-raised border-t-2 border-primary-deep"
              style:top={`${marker * 100}%`}
            >
              <span
                class="absolute -start-1 -top-[3px] size-1.5 rounded-full bg-primary-deep"
              ></span>
              <!-- DESIGN.md wants the now-line labelled with its hour. At week
                   zoom a column is 53px and the pill would cover the course
                   under it, so the label rides only where there is room. -->
              {#if !dense}
                <span
                  class="absolute end-0 -translate-y-1/2 rounded-pill bg-primary-deep px-1.5
                         text-2xs font-bold tabular-nums text-card"
                  >{format.timeFormatter.format(now)}</span
                >
              {/if}
              <span class="sr-only">{m.calendar_now()}</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>
