<script lang="ts">
  import { tick } from 'svelte';
  import { ExternalLink, MapPin, UserRound } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import { monthHeaderDays } from './calendar-format.svelte';
  import type { CalendarFormat } from './calendar-format.svelte';
  import { categoryCode, categoryInk, categorySurface, spentSurface } from './category-tone';
  import { blockGeometry, layoutDay, windowHours } from './calendar-layout';
  import type { PositionedEvent, TimeWindow } from './calendar-layout';
  import { monthGridDays, monthKeyTarget } from './calendar-navigation.svelte';
  import { panel, uppercaseTiny } from './calendar-styles';
  import {
    courseCategory,
    eventDurationMinutes,
    eventSecondary,
    eventTitle,
    formatDuration,
    getEventStatus,
    isCancelled,
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
    /** Shared with the other zooms, so a cell is the same drawing made smaller. */
    timeWindow: TimeWindow;
    eventsForDay: (date: Date) => CalendarEvent[];
    onSelectDate: (date: Date) => void;
    onFocusDate: (date: Date) => void;
    /** A cell zooms one level out of the month, onto the week it belongs to. */
    onZoomWeek: (date: Date) => void;
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
    timeWindow,
    eventsForDay,
    onSelectDate,
    onFocusDate,
    onZoomWeek,
    onEventClick,
    onOpenTempo,
  }: Props = $props();

  /**
   * The weeks, not a flat run of cells: `role="grid"` without a `role="row"`
   * layer is invalid, and the position an assistive technology announces comes
   * from that layer.
   */
  const monthWeeks = $derived.by(() => {
    const days = monthGridDays(anchorDate);
    return Array.from({ length: days.length / 7 }, (_, week) =>
      days.slice(week * 7, week * 7 + 7)
    );
  });

  /** The cells, in the same flat order, so the keyboard can move focus directly. */
  let cellElements: (HTMLButtonElement | null)[] = [];

  const activeDateEvents = $derived(eventsForDay(activeDate));
  const activeDateDurationMinutes = $derived.by(() =>
    activeDateEvents.reduce((total, event) => total + eventDurationMinutes(event), 0)
  );

  const gridHours = $derived(windowHours(timeWindow));
  const spanHours = $derived((timeWindow.endMinutes - timeWindow.startMinutes) / 60);

  /**
   * Three anchors instead of the week's eleven: the band's first hour, its
   * middle, its last. The month used to draw its bars with no hour reference
   * at all, which made a mark's vertical position undecodable — the drawing
   * was the point and it could not be read. Three is enough to place a mark
   * and few enough that the gutter is not the loudest thing on screen.
   */
  const hourAnchors = $derived.by(() => {
    if (gridHours.length === 0) return [];
    const picks: { index: number; align: string }[] = [
      { index: 0, align: 'translate-y-0' },
      { index: Math.floor((gridHours.length - 1) / 2), align: '-translate-y-1/2' },
      { index: gridHours.length - 1, align: '-translate-y-full' },
    ];
    return picks.map(({ index, align }) => ({
      hour: gridHours[index],
      label: format.hourFormatter.format(new Date(2024, 0, 1, gridHours[index], 0)),
      ratio: index / spanHours,
      align,
    }));
  });

  /** The middle anchor, ruled across every cell so morning and afternoon read as halves. */
  const middayRatio = $derived(hourAnchors[1]?.ratio ?? 0.5);

  /** The cell's day-number line. The gutter reserves the same, or its anchors miss the band. */
  const HEADER_PX = 16;
  /** The cell's own vertical padding. */
  const CELL_PADDING_PX = 6;
  /** Below this a mark cannot hold its code; below twice it, not its hour either. */
  const CODE_FROM_PX = 15;
  const HOUR_FROM_PX = 30;

  /**
   * Measured, not guessed. Whether a mark can carry text depends on what an
   * hour is worth in pixels here, which depends on the band and on the height
   * the layout left the grid — so a threshold in minutes would clip text on
   * one window and waste room on another.
   */
  let gridHeight = $state(0);

  const bandHeightPx = $derived.by(() => {
    if (gridHeight === 0 || monthWeeks.length === 0) return 0;
    const rowHeight = (gridHeight - (monthWeeks.length - 1)) / monthWeeks.length;
    return Math.max(0, rowHeight - HEADER_PX - CELL_PADDING_PX);
  });

  function markHeightPx(block: PositionedEvent): number {
    const windowMinutes = timeWindow.endMinutes - timeWindow.startMinutes;
    if (windowMinutes <= 0) return 0;
    return ((block.toMinutes - block.fromMinutes) / windowMinutes) * bandHeightPx;
  }

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
    ' transition-control active:scale-(--press-scale) fine-hover:border-primary-deep';

  const tempoBtn =
    'inline-flex min-h-(--tap-min) items-center gap-1 rounded-pill border border-muted-strong' +
    ' bg-card px-3 text-xs font-bold text-primary-deep transition-control' +
    ' active:scale-(--press-scale) fine-hover:bg-muted';

  /**
   * The selected day's list needs width, so it exists only on a wide window —
   * and that decides what a cell does when tapped. With the pane on screen a
   * tap selects, because the selection is visible. Without it there is nothing
   * to show a selection in, so the tap zooms one level out of the month and
   * onto the week, which is the model the rest of this view runs on. The query
   * is the pane's own, so the two can never disagree about which is true.
   */
  const SIDE_PANE_QUERY = '(min-width: 54rem)';
  let hasSidePane = $state(false);

  $effect(() => {
    const query = window.matchMedia(SIDE_PANE_QUERY);
    const sync = () => (hasSidePane = query.matches);
    sync();
    query.addEventListener('change', sync);
    return () => query.removeEventListener('change', sync);
  });

  /** Narrower than the week's gutter: three labels, not eleven. */
  const GUTTER = '2.1rem';
  const columnTemplate = `${GUTTER} repeat(7, minmax(0, 1fr))`;
</script>

<!-- The one list-shaped row left: the selected day beside the month grid. -->
{#snippet courseRow(event: CalendarEvent)}
  {@const status = getEventStatus(event, now)}
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

<!-- One day, drawn on the same time axis as the other zooms. -->
{#snippet dayMarks(dayEvents: CalendarEvent[])}
  {#each layoutDay(dayEvents) as block (block.event.id)}
    {@const geometry = blockGeometry(block, timeWindow)}
    {@const category = courseCategory(block.event.kind)}
    {@const height = markHeightPx(block)}
    <!-- A mark that shares its column has half the width; below that there is
         no room for a code and the field carries the category alone. -->
    {@const roomy = block.lanes === 1}
    {@const spent = getEventStatus(block.event, now) === 'finished'}
    <!-- An exam still to come is the one mark a student opens a month for, so
         it is the single saturated fill and everything else stays a pale
         field. A focal point, not a second colour scheme. -->
    {@const shout = category === 'exam' && !spent}
    <span
      class={cn(
        'absolute flex min-h-[4px] flex-col items-start overflow-hidden rounded-[2px] px-[2px]',
        shout
          ? categoryInk(category)
          : spent
            ? cn(spentSurface, 'border border-border-subtle')
            : cn(categorySurface(category), 'border border-current'),
        isCancelled(block.event) && 'border-dashed opacity-50'
      )}
      style:top={`${geometry.top}%`}
      style:height={`${geometry.height}%`}
      style:left={`${geometry.left}%`}
      style:width={`calc(${geometry.width}% - 1px)`}
      aria-hidden="true"
    >
      {#if roomy && height >= CODE_FROM_PX}
        <span class="text-2xs leading-[1.05] font-extrabold">{categoryCode(category)}</span>
      {/if}
      {#if roomy && height >= HOUR_FROM_PX}
        <span class="text-2xs leading-[1.05] font-normal tabular-nums"
          >{format.timeFormatter.format(new Date(block.event.startsAt))}</span
        >
      {/if}
    </span>
  {/each}
{/snippet}

<section
  class="grid min-h-0 flex-1 grid-cols-1 gap-4 min-[54rem]:grid-cols-[minmax(0,1.5fr)_minmax(0,1fr)]"
>
  <!-- The month is one field split by hairlines, not thirty bordered cards:
       `gap-px` over the border colour draws every rule exactly once, which is
       also why no cell carries a border of its own. -->
  <div class={cn(panel, 'flex min-h-[22rem] min-w-0 flex-col gap-1.5 p-2 md:p-3')}>
    <div
      class={cn('grid shrink-0 gap-px text-center text-muted-foreground', uppercaseTiny)}
      style:grid-template-columns={columnTemplate}
      aria-hidden="true"
    >
      <span></span>
      {#each monthHeaderDays as day (day.getTime())}
        <!-- Two letters, not one: lundi, mardi and mercredi all start with the
             same letter in French and a one-letter header is a guessing game. -->
        <span>{format.weekdayShortFormatter.format(day).slice(0, 2)}</span>
      {/each}
    </div>

    <div
      class="min-h-0 flex-1 overflow-hidden rounded-md border border-border-subtle
             bg-border-subtle"
    >
      <!-- One tab stop: the arrow keys walk the grid from the focused cell. -->
      <div
        class="grid h-full gap-px"
        style:grid-template-columns={columnTemplate}
        style:grid-template-rows={`repeat(${monthWeeks.length}, minmax(0, 1fr))`}
        role="grid"
        aria-label={m.calendar_month_grid_label({ period: format.periodLabel })}
        bind:clientHeight={gridHeight}
      >
        {#each monthWeeks as week, weekIndex (week[0].toISOString())}
          <div class="contents" role="row">
            <!-- The gutter reserves the cell's header height so its anchors
                 land on the band and not beside it. -->
            <div class="flex min-h-0 flex-col bg-card pe-1" role="presentation" aria-hidden="true">
              <span class="block h-4 shrink-0"></span>
              <span class="relative min-h-0 flex-1">
                {#each hourAnchors as anchor (anchor.hour)}
                  <span
                    class={cn(
                      'absolute end-0 text-2xs leading-none font-semibold tabular-nums',
                      'text-muted-foreground',
                      anchor.align
                    )}
                    style:top={`${anchor.ratio * 100}%`}>{anchor.label}</span
                  >
                {/each}
              </span>
            </div>

            {#each week as day, dayIndex (day.toISOString())}
              {@const isDayInMonth = isSameMonth(day, anchorDate)}
              {@const dayEvents = isDayInMonth ? eventsForDay(day) : []}
              {@const isDayToday = isSameDay(day, now)}
              {@const isDaySelected = isSameDay(day, activeDate)}

              <button
                type="button"
                role="gridcell"
                tabindex={isSameDay(day, monthFocusDate) ? 0 : -1}
                class={cn(
                  'relative flex min-h-0 cursor-pointer flex-col px-1 pt-0.5 pb-1 text-start',
                  'transition-control active:scale-(--press-scale) fine-hover:bg-muted',
                  isDayInMonth ? 'bg-card' : 'bg-background',
                  isDayToday && 'bg-muted',
                  isDaySelected && 'outline-2 -outline-offset-2 outline-primary-deep'
                )}
                aria-selected={isDaySelected}
                aria-label={m.calendar_day_cell_label({
                  date: format.dayFormatter.format(day),
                  courses: format.dayCountLabel(dayEvents.length),
                })}
                onclick={() => {
                  onSelectDate(day);
                  if (!hasSidePane) onZoomWeek(day);
                }}
                onkeydown={handleMonthKeydown}
                bind:this={cellElements[weekIndex * 7 + dayIndex]}
              >
                <span class="flex h-4 shrink-0 items-center justify-between gap-1">
                  <span
                    class={cn(
                      'text-xs leading-none tabular-nums',
                      !isDayInMonth
                        ? 'font-semibold text-muted-foreground'
                        : isDayToday
                          ? 'font-extrabold text-primary-deep'
                          : 'font-bold text-foreground'
                    )}>{day.getDate()}</span
                  >
                  {#if dayEvents.length > 0}
                    <span
                      class="text-2xs leading-none font-semibold tabular-nums text-muted-foreground"
                      aria-hidden="true">{dayEvents.length}</span
                    >
                  {/if}
                </span>

                <span class="relative min-h-0 w-full flex-1">
                  {#if isDayInMonth}
                    <span
                      class="absolute inset-x-0 border-t border-border-subtle"
                      style:top={`${middayRatio * 100}%`}
                      aria-hidden="true"
                    ></span>
                  {/if}
                  {@render dayMarks(dayEvents)}
                </span>
              </button>
            {/each}
          </div>
        {/each}
      </div>
    </div>
  </div>

  {#if hasSidePane}
  <div class={cn(panel, 'flex min-h-0 flex-col gap-3 p-5')}>
    <header
      class="flex shrink-0 items-center justify-between gap-3 border-b border-border-subtle pb-3
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

    <div class="flex min-h-0 flex-col gap-2.5 overflow-y-auto">
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
  {/if}
</section>
