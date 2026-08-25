<script lang="ts">
  import { tick } from 'svelte';
  import {
    Calendar,
    CalendarCheck,
    CalendarDays,
    CalendarSearch,
    ChevronLeft,
    ChevronRight,
    Clock,
    ExternalLink,
    MapPin,
    RefreshCw,
    UserRound,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import FreshnessLabel from '$lib/components/ui/FreshnessLabel.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import Sheet from '$lib/components/ui/Sheet.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import CalendarViewSkeleton from './CalendarViewSkeleton.svelte';
  import CourseDetailModal from './CourseDetailModal.svelte';
  import {
    addDays,
    addMonths,
    capitalizeFirst,
    dayKey,
    getWeekNumber,
    isSameDay,
    isSameMonth,
    startOfDay,
    startOfMonth,
    startOfWeek,
  } from './date-utils';
  import {
    eventDurationMinutes,
    eventSecondary,
    eventTitle,
    formatDuration,
    getEventStatus,
    openExternalUrl,
    parseRoomAndTeacher,
  } from './course-utils';
  import {
    blockGeometry,
    gapMinutes,
    layoutDay,
    ratioInWindow,
    timeWindowFor,
    windowHours,
  } from './calendar-layout';
  import type { PositionedEvent } from './calendar-layout';
  import type { CalendarEvent, CalendarScope } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    events: CalendarEvent[];
    locale: Locale;
    sundaysVisible?: boolean;
    initialScope?: CalendarScope;
    selectedDate?: Date;
    now?: Date;
    loading?: boolean;
    /** Epoch ms of the last successful fetch, for the freshness statement. */
    fetchedAt?: number | null;
    /** A refresh failed while these events were already on screen. */
    refreshFailed?: boolean;
    onPeriodChange?: (startDate: Date, durationDays: number) => void | Promise<void>;
    onRefresh?: () => void | Promise<void>;
    onEventClick?: (event: CalendarEvent) => void;
    onOpenTempo?: (event: CalendarEvent) => void;
  };

  let {
    events = [],
    locale,
    sundaysVisible = false,
    initialScope = 'week',
    selectedDate,
    now = new Date(),
    loading = false,
    fetchedAt = null,
    refreshFailed = false,
    onPeriodChange,
    onRefresh,
    onEventClick,
    onOpenTempo,
  }: Props = $props();

  /**
   * Row height of one hour, per scope. It is not a free aesthetic choice: at
   * 4.5rem an hour, the shortest slot the portal returns (30 min) is still
   * 36px tall, and the `min-h-(--tap-min)` floor on a block then only has to
   * stretch the rarest case instead of every second course.
   */
  const HOUR_HEIGHT_REM = { day: 5, week: 4.5 } as const;

  /** Below this a block has no room for a third line, so the room is dropped. */
  const ROOM_VISIBLE_FROM_MINUTES = 60;

  let currentScope = $state<CalendarScope>('week');
  let anchorDate = $state<Date>(startOfDay(new Date()));
  let activeDate = $state<Date>(startOfDay(new Date()));
  let modalEvent = $state<CalendarEvent | null>(null);
  let pickerOpen = $state(false);
  let pickerMonth = $state<Date>(startOfMonth(new Date()));
  /**
   * The month grid is one tab stop, not 42. This is the cell the arrow keys
   * moved to, which is also the only cell carrying `tabindex="0"`.
   */
  let monthFocusDate = $state<Date>(startOfDay(new Date()));
  let stripRef = $state<HTMLDivElement | null>(null);
  let gridScrollRef = $state<HTMLDivElement | null>(null);
  let monthGridRef = $state<HTMLDivElement | null>(null);

  $effect.pre(() => {
    if (initialScope) currentScope = initialScope;
  });

  $effect(() => {
    if (selectedDate) {
      anchorDate = startOfDay(selectedDate);
      activeDate = startOfDay(selectedDate);
      monthFocusDate = startOfDay(selectedDate);
    }
  });

  function prefersReducedMotion(): boolean {
    // The global `prefers-reduced-motion` rule in app.css cannot reach a JS
    // scroll option, so the preference is read again here.
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }

  $effect(() => {
    activeDate;
    currentScope;
    void tick().then(() => {
      stripRef?.querySelector<HTMLButtonElement>('[aria-pressed="true"]')?.scrollIntoView({
        behavior: prefersReducedMotion() ? 'auto' : 'smooth',
        block: 'nearest',
        inline: 'center',
      });
    });
  });

  /**
   * The grid opens on the hour that matters rather than on its first row: now
   * when today is in view, otherwise the first course. The scroll is written on
   * the region itself so the page around it does not move.
   */
  $effect(() => {
    currentScope;
    anchorDate;
    timeWindow;
    void tick().then(() => {
      const region = gridScrollRef;
      if (!region) return;

      const ratio = gridDays.some((day) => isSameDay(day, now))
        ? ratioInWindow(now, timeWindow)
        : firstEventRatio;
      if (ratio === null) return;

      const target = ratio * region.scrollHeight - region.clientHeight / 2;
      region.scrollTo({
        top: Math.max(0, target),
        behavior: prefersReducedMotion() ? 'auto' : 'smooth',
      });
    });
  });

  const dayFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      weekday: 'long',
      day: 'numeric',
      month: 'long',
      year: 'numeric',
    })
  );

  const weekdayShortFormatter = $derived(
    new Intl.DateTimeFormat(locale, { weekday: 'short' })
  );

  const monthYearFormatter = $derived(
    new Intl.DateTimeFormat(locale, { month: 'long', year: 'numeric' })
  );

  const timeFormatter = $derived(
    new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' })
  );

  const rangeFormatter = $derived(
    new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'short', year: 'numeric' })
  );

  const sortedEvents = $derived.by(() =>
    [...events].sort((a, b) => new Date(a.startsAt).getTime() - new Date(b.startsAt).getTime())
  );

  const eventsByDay = $derived.by(() => {
    const index = new Map<string, CalendarEvent[]>();
    for (const event of sortedEvents) {
      const key = dayKey(new Date(event.startsAt));
      const dayEvents = index.get(key);
      if (dayEvents) dayEvents.push(event);
      else index.set(key, [event]);
    }
    return index;
  });

  const visibleWeekDaysCount = $derived(sundaysVisible ? 7 : 6);
  const weekStartDate = $derived(startOfWeek(anchorDate));
  const weekDays = $derived(
    Array.from({ length: visibleWeekDaysCount }, (_, i) => addDays(weekStartDate, i))
  );

  const monthFirstDay = $derived(startOfMonth(anchorDate));
  const monthGridStart = $derived(startOfWeek(monthFirstDay));
  /**
   * Only the weeks the month actually touches. A fixed 42 cells adds a whole
   * trailing week of foreign days to a short February, and asks the portal for
   * six weeks when five are shown.
   */
  const monthWeekCount = $derived.by(() => {
    const nextMonth = addMonths(monthFirstDay, 1);
    const spannedDays = Math.round(
      (nextMonth.getTime() - monthGridStart.getTime()) / 86_400_000
    );
    return Math.ceil(spannedDays / 7);
  });
  const monthGridDays = $derived(
    Array.from({ length: monthWeekCount * 7 }, (_, i) => addDays(monthGridStart, i))
  );

  /** A fixed Monday-first week, only ever used to print weekday column titles. */
  const monthHeaderDays = Array.from({ length: 7 }, (_, i) => new Date(2024, 0, 1 + i));

  const pickerFirstDay = $derived(startOfMonth(pickerMonth));
  const pickerGridStart = $derived(startOfWeek(pickerFirstDay));
  const pickerWeekCount = $derived.by(() => {
    const nextMonth = addMonths(pickerFirstDay, 1);
    const spannedDays = Math.round(
      (nextMonth.getTime() - pickerGridStart.getTime()) / 86_400_000
    );
    return Math.ceil(spannedDays / 7);
  });
  const pickerDays = $derived(
    Array.from({ length: pickerWeekCount * 7 }, (_, i) => addDays(pickerGridStart, i))
  );

  /** The days the time grid draws: one in day scope, the portal week otherwise. */
  const gridDays = $derived(currentScope === 'day' ? [activeDate] : weekDays);
  const gridEvents = $derived.by(() => gridDays.flatMap((day) => eventsForDay(day)));
  const timeWindow = $derived(timeWindowFor(gridEvents));
  const gridHours = $derived(windowHours(timeWindow));
  const gridSpanHours = $derived((timeWindow.endMinutes - timeWindow.startMinutes) / 60);
  const hourHeightRem = $derived(
    currentScope === 'day' ? HOUR_HEIGHT_REM.day : HOUR_HEIGHT_REM.week
  );

  const firstEventRatio = $derived.by(() => {
    const first = gridEvents
      .map((event) => new Date(event.startsAt))
      .sort((a, b) => a.getTime() - b.getTime())[0];
    return first ? ratioInWindow(first, timeWindow) : null;
  });

  const activeDateEvents = $derived(eventsForDay(activeDate));
  const activeDateDurationMinutes = $derived.by(() =>
    activeDateEvents.reduce((total, event) => total + eventDurationMinutes(event), 0)
  );
  const activeDateGapMinutes = $derived(gapMinutes(activeDateEvents));

  const periodLabel = $derived.by(() => {
    switch (currentScope) {
      case 'day':
        return capitalizeFirst(dayFormatter.format(activeDate));
      case 'week': {
        const weekEnd = addDays(weekStartDate, visibleWeekDaysCount - 1);
        return m.calendar_week_range({
          week: getWeekNumber(weekStartDate),
          range: rangeFormatter.formatRange(weekStartDate, weekEnd),
        });
      }
      case 'month':
        return capitalizeFirst(monthYearFormatter.format(anchorDate));
    }
  });

  const scopeOptions = $derived.by(() => {
    locale;
    return [
      { value: 'day', label: m.scope_day() },
      { value: 'week', label: m.scope_week() },
      { value: 'month', label: m.scope_month() },
    ];
  });

  const ScopeIcon = $derived(
    currentScope === 'day' ? Clock : currentScope === 'week' ? CalendarDays : Calendar
  );

  const scopeName = $derived.by(() => {
    locale;
    switch (currentScope) {
      case 'day':
        return m.scope_day();
      case 'week':
        return m.scope_week();
      case 'month':
        return m.scope_month();
    }
  });

  function eventsForDay(date: Date): CalendarEvent[] {
    return eventsByDay.get(dayKey(date)) ?? [];
  }

  function eventTimeRange(event: CalendarEvent): string {
    return `${timeFormatter.format(new Date(event.startsAt))} – ${timeFormatter.format(new Date(event.endsAt))}`;
  }

  function dayCountLabel(date: Date): string {
    return m.day_course_count({ count: eventsForDay(date).length });
  }

  function setScope(scope: string) {
    currentScope = scope as CalendarScope;
    triggerPeriodChange(anchorDate, currentScope);
  }

  function movePeriod(direction: -1 | 1) {
    let newAnchor: Date;
    switch (currentScope) {
      case 'day':
        newAnchor = addDays(anchorDate, direction);
        activeDate = newAnchor;
        break;
      case 'week':
        newAnchor = addDays(anchorDate, direction * 7);
        activeDate = startOfWeek(newAnchor);
        break;
      case 'month':
        newAnchor = addMonths(anchorDate, direction);
        activeDate = startOfMonth(newAnchor);
        break;
    }
    anchorDate = newAnchor;
    monthFocusDate = activeDate;
    triggerPeriodChange(newAnchor, currentScope);
  }

  function goToToday() {
    const today = startOfDay(new Date());
    anchorDate = today;
    activeDate = today;
    monthFocusDate = today;
    triggerPeriodChange(today, currentScope);
  }

  function selectDate(date: Date) {
    activeDate = startOfDay(date);
    monthFocusDate = activeDate;
    if (currentScope === 'day') {
      anchorDate = activeDate;
      triggerPeriodChange(activeDate, currentScope);
    }
  }

  function openPicker() {
    pickerMonth = startOfMonth(activeDate);
    pickerOpen = true;
  }

  /**
   * Replaces `<input type="week">`, which neither WKWebView nor WebKitGTK
   * implements: on those platforms it degrades to a text field expecting
   * `2026-W35`, which is not a control a student can operate.
   */
  function pickDate(date: Date) {
    const picked = startOfDay(date);
    anchorDate = currentScope === 'week' ? startOfWeek(picked) : picked;
    activeDate = picked;
    monthFocusDate = picked;
    pickerOpen = false;
    triggerPeriodChange(anchorDate, currentScope);
  }

  function triggerPeriodChange(date: Date, scope: CalendarScope) {
    if (!onPeriodChange) return;
    let startDate: Date;
    let durationDays: number;

    switch (scope) {
      case 'day':
        startDate = startOfDay(date);
        durationDays = 1;
        break;
      case 'week':
        startDate = startOfWeek(date);
        durationDays = 7;
        break;
      case 'month': {
        const first = startOfMonth(date);
        const gridStart = startOfWeek(first);
        const spannedDays = Math.round(
          (addMonths(first, 1).getTime() - gridStart.getTime()) / 86_400_000
        );
        startDate = gridStart;
        durationDays = Math.ceil(spannedDays / 7) * 7;
        break;
      }
    }

    void onPeriodChange(startDate, durationDays);
  }

  function handleCourseClick(event: CalendarEvent) {
    if (onEventClick) onEventClick(event);
    else modalEvent = event;
  }

  async function handleTempoClick(e: MouseEvent, event: CalendarEvent) {
    e.stopPropagation();
    if (onOpenTempo) {
      onOpenTempo(event);
      return;
    }
    await openExternalUrl(event.tempoUrl);
  }

  /**
   * Arrow keys walk the month, so the grid costs one tab stop instead of 42.
   * Moving out of the displayed month moves the month with it, which is what
   * makes the keyboard path equivalent to the pointer one.
   */
  function handleMonthKeydown(event: KeyboardEvent) {
    let next: Date | null = null;

    switch (event.key) {
      case 'ArrowLeft':
        next = addDays(monthFocusDate, -1);
        break;
      case 'ArrowRight':
        next = addDays(monthFocusDate, 1);
        break;
      case 'ArrowUp':
        next = addDays(monthFocusDate, -7);
        break;
      case 'ArrowDown':
        next = addDays(monthFocusDate, 7);
        break;
      case 'Home':
        next = startOfWeek(monthFocusDate);
        break;
      case 'End':
        next = addDays(startOfWeek(monthFocusDate), 6);
        break;
      case 'PageUp':
        next = addMonths(monthFocusDate, -1);
        break;
      case 'PageDown':
        next = addMonths(monthFocusDate, 1);
        break;
      default:
        return;
    }

    event.preventDefault();
    monthFocusDate = next;

    if (!isSameMonth(next, anchorDate)) {
      anchorDate = startOfMonth(next);
      triggerPeriodChange(anchorDate, 'month');
    }

    void tick().then(() => {
      monthGridRef
        ?.querySelector<HTMLButtonElement>(`[data-day="${dayKey(monthFocusDate)}"]`)
        ?.focus();
    });
  }

  /**
   * Horizontal drag moves the period. Week scope is excluded on purpose: there
   * the same gesture already scrolls the day columns, and two meanings on one
   * axis is how a swipe becomes a coin toss.
   */
  const SWIPE_DISTANCE = 64;
  let swipeStartX = 0;
  let swipeStartY = 0;
  let swipeTracking = false;

  function handleSwipeStart(event: PointerEvent) {
    swipeTracking = event.pointerType !== 'mouse' && currentScope !== 'week';
    swipeStartX = event.clientX;
    swipeStartY = event.clientY;
  }

  function handleSwipeEnd(event: PointerEvent) {
    if (!swipeTracking) return;
    swipeTracking = false;

    const deltaX = event.clientX - swipeStartX;
    const deltaY = event.clientY - swipeStartY;
    if (Math.abs(deltaX) < SWIPE_DISTANCE || Math.abs(deltaX) < Math.abs(deltaY) * 1.5) return;

    movePeriod(deltaX < 0 ? 1 : -1);
  }

  const container =
    'flex w-full flex-col gap-3 px-3 pt-3 pb-6' +
    ' md:gap-4 md:px-8 md:pt-6 md:pb-8' +
    ' lte-600:pr-[max(var(--space-2),var(--safe-right))]' +
    ' lte-600:pl-[max(var(--space-2),var(--safe-left))]';

  const panel = 'rounded-xl border border-border-subtle bg-card';

  const uppercaseTiny = 'text-xs font-bold tracking-[0.04em] uppercase';

  const stripDayBtn =
    'flex min-h-18 min-w-14 flex-1 basis-0 flex-col items-center justify-center gap-[0.15rem]' +
    ' rounded-lg border bg-card px-1 py-2 transition-control active:scale-(--press-scale)';

  /** One block on the time grid. Elevation is the border; hover adds the lift. */
  const blockBase =
    'absolute flex min-h-(--tap-min) min-w-0 flex-col gap-[0.1rem] overflow-hidden rounded-sm' +
    ' border border-l-[3px] px-1.5 py-1 text-start transition-control' +
    ' active:scale-(--press-scale) hover:border-primary-deep hover:shadow-sm';

  const monthCellBtn =
    'flex min-h-(--tap-min) w-full cursor-pointer flex-col items-center justify-center gap-[0.15rem]' +
    ' rounded-md border px-1 py-1.5 transition-control active:scale-(--press-scale)' +
    ' hover:border-primary-deep hover:bg-muted';

  const detailRow =
    'grid grid-cols-[minmax(0,1fr)_auto] items-center gap-2 rounded-lg border p-3' +
    ' transition-control active:scale-(--press-scale) hover:border-primary-deep hover:shadow-sm';

  const tempoBtn =
    'inline-flex min-h-(--tap-min) items-center gap-1 rounded-pill border border-muted-strong' +
    ' bg-card px-3 text-xs font-bold text-primary-deep transition-control' +
    ' active:scale-(--press-scale) hover:bg-muted';
</script>

{#snippet statusBadge(status: 'live' | 'upcoming' | 'finished')}
  {#if status === 'live'}
    <Badge tone="live" dot>{m.schedule_status_live()}</Badge>
  {:else if status === 'upcoming'}
    <Badge tone="accent">{m.schedule_status_upcoming()}</Badge>
  {:else}
    <Badge tone="neutral">{m.schedule_status_finished()}</Badge>
  {/if}
{/snippet}

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
      blockBase,
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
    onclick={() => handleCourseClick(event)}
  >
    <span
      class={cn(
        'text-2xs font-bold tabular-nums',
        status === 'finished' ? 'text-muted-foreground' : 'text-primary-deep'
      )}
    >
      {eventTimeRange(event)}
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
{#snippet timeGrid(days: Date[])}
  {@const scrolls = days.length > 1}
  <!-- Day scope names its day in the period title and in the strip above, so a
       third copy over a single column is noise. -->
  {@const showHeaders = days.length > 1}
  {@const bodyRow = showHeaders ? 2 : 1}
  <div
    class={cn(
      panel,
      'relative overflow-y-auto px-2 pb-2 md:px-3 md:pb-3',
      'max-h-[26rem] md:max-h-[38rem]',
      'scrollbar-none [&::-webkit-scrollbar]:hidden'
    )}
    bind:this={gridScrollRef}
  >
    <div
      class={cn(
        scrolls && 'overflow-x-auto scrollbar-none [&::-webkit-scrollbar]:hidden md:overflow-x-visible'
      )}
    >
      <div
        class="grid gap-x-1.5"
        style:--hour-height={`${hourHeightRem}rem`}
        style:grid-template-columns={`3.25rem repeat(${days.length}, minmax(${scrolls ? '8.5rem' : '0'}, 1fr))`}
        style:grid-template-rows={
          showHeaders
            ? `auto calc(var(--hour-height) * ${gridSpanHours})`
            : `calc(var(--hour-height) * ${gridSpanHours})`
        }
        role="group"
        aria-label={m.calendar_grid_label({ period: periodLabel })}
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
            onclick={() => selectDate(day)}
          >
            <span class="flex items-baseline gap-1">
              <span class={uppercaseTiny}>{weekdayShortFormatter.format(day)}</span>
              <span class="text-md font-extrabold tabular-nums">{day.getDate()}</span>
            </span>
            <span class="text-2xs font-semibold">{dayCountLabel(day)}</span>
          </button>
        {/each}

        <!-- The hour scale, pinned while the columns scroll sideways. -->
        <div class="sticky start-0 z-raised col-start-1 bg-card" style:grid-row={bodyRow}>
          <div class="relative h-full">
            {#each gridHours as hour, index (hour)}
              <span
                class={cn(
                  'absolute end-1.5 text-2xs font-semibold tabular-nums text-muted-foreground',
                  index === 0 ? 'translate-y-0' : '-translate-y-1/2'
                )}
                style:top={`${(index / gridSpanHours) * 100}%`}
              >
                {timeFormatter.format(new Date(2024, 0, 1, hour, 0))}
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
          <div
            class="relative rounded-md"
            style:grid-column={index + 2}
            style:grid-row={bodyRow}
          >
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
                  {timeFormatter.format(now)}
                </span>
                <span class="sr-only">{m.calendar_now()}</span>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
{/snippet}

{#snippet dayStrip()}
  <div
    class="flex gap-2 overflow-x-auto p-1 scrollbar-none [-webkit-overflow-scrolling:touch]
           [&::-webkit-scrollbar]:hidden"
    bind:this={stripRef}
  >
    {#each weekDays as day (day.toISOString())}
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
        onclick={() => selectDate(day)}
      >
        <span class={uppercaseTiny}>{weekdayShortFormatter.format(day)}</span>
        <span class="text-xl leading-[1.2] font-extrabold tabular-nums">{day.getDate()}</span>
        <span class="text-2xs font-semibold tabular-nums"
          >{dayEventsCount > 0 ? dayEventsCount : '·'}</span
        >
      </button>
    {/each}
  </div>
{/snippet}

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
      onclick={() => handleCourseClick(event)}
    >
      <span class="flex min-w-0 flex-col pt-[0.15rem] tabular-nums">
        <strong class="text-base font-extrabold text-foreground"
          >{timeFormatter.format(new Date(event.startsAt))}</strong
        >
        <span class="text-xs text-muted-foreground"
          >{timeFormatter.format(new Date(event.endsAt))}</span
        >
      </span>

      <span class="relative flex h-full flex-col items-center">
        <span class="h-full w-[3px] rounded-pill bg-primary-deep"></span>
      </span>

      <span class="flex min-w-0 flex-col gap-1">
        {#if live || event.kind}
          <span class="flex min-w-0 max-w-full flex-wrap items-center gap-2">
            {#if live}{@render statusBadge(status)}{/if}
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

<div class={container}>
  <!-- 1. One control bar: scope, period, navigation, freshness. -->
  <header
    class={cn(
      panel,
      'flex flex-col gap-2.5 p-3',
      'md:flex-row md:items-center md:justify-between md:gap-4 md:px-4'
    )}
  >
    <div class="flex min-w-0 flex-col gap-[0.15rem]">
      <span
        class="inline-flex min-w-0 items-center gap-1 text-2xs font-bold tracking-[0.05em]
               uppercase wrap-anywhere text-primary-deep"
      >
        <ScopeIcon size={12} aria-hidden="true" />
        <span>{scopeName}</span>
      </span>
      <h2
        class="max-w-full text-base leading-[1.3] font-extrabold wrap-anywhere text-foreground
               md:text-lg"
      >{periodLabel}</h2>
      <FreshnessLabel
        {fetchedAt}
        {locale}
        refreshing={loading}
        failed={refreshFailed}
        class="mt-[0.15rem]"
      />
    </div>

    <div class="flex flex-col gap-2 md:flex-row md:items-center md:gap-3">
      <div class="flex items-center gap-1.5 lte-600:justify-between">
        <IconButton label={m.previous_period()} onclick={() => movePeriod(-1)}>
          <ChevronLeft size={18} strokeWidth={2.2} aria-hidden="true" />
        </IconButton>

        <Button variant="accent" size="sm" onclick={goToToday}>
          <CalendarCheck size={14} aria-hidden="true" />
          <span>{m.go_to_today()}</span>
        </Button>

        <IconButton label={m.next_period()} onclick={() => movePeriod(1)}>
          <ChevronRight size={18} strokeWidth={2.2} aria-hidden="true" />
        </IconButton>

        <IconButton label={m.calendar_pick_date()} onclick={openPicker}>
          <CalendarSearch size={17} strokeWidth={2.1} aria-hidden="true" />
        </IconButton>

        {#if onRefresh}
          <IconButton
            label={m.sync_refresh()}
            loading={loading}
            onclick={() => void onRefresh?.()}
          >
            <RefreshCw size={17} strokeWidth={2.2} aria-hidden="true" />
          </IconButton>
        {/if}
      </div>

      <SegmentedControl
        options={scopeOptions}
        value={currentScope}
        label={m.calendar_scope_label()}
        onChange={setScope}
        class="md:w-[15rem]"
      />
    </div>
  </header>

  <!-- 2. Scope views. -->
  <main
    class="relative flex min-h-96 flex-col gap-3"
    onpointerdown={handleSwipeStart}
    onpointerup={handleSwipeEnd}
    onpointercancel={() => (swipeTracking = false)}
  >
    {#if loading && events.length === 0}
      <CalendarViewSkeleton ariaLabel={m.planning_loading()} />
    {:else if currentScope === 'day'}
      <div class={cn(panel, 'flex items-center justify-between gap-3 px-4 py-3 lte-600:items-start')}>
        <p class="min-w-0 text-sm text-muted-foreground">
          {m.day_course_count({ count: activeDateEvents.length })}
          {#if activeDateDurationMinutes > 0}
            · {formatDuration(activeDateDurationMinutes, locale)}
          {/if}
          {#if activeDateGapMinutes > 0}
            · {m.calendar_free_time({ duration: formatDuration(activeDateGapMinutes, locale) })}
          {/if}
        </p>
        {#if isSameDay(activeDate, now)}
          <Badge tone="accent">{m.preview_today()}</Badge>
        {/if}
      </div>

      {@render dayStrip()}

      {#if activeDateEvents.length > 0}
        {@render timeGrid([activeDate])}
      {:else}
        <StateCard
          kind="empty"
          title={m.no_courses_day()}
          description={m.no_courses_day_description()}
          icon={CalendarCheck}
        />
      {/if}
    {:else if currentScope === 'week'}
      {#if gridEvents.length > 0}
        {@render timeGrid(weekDays)}
      {:else}
        <StateCard
          kind="empty"
          title={m.no_events_period()}
          description={m.no_courses_day_description()}
          icon={CalendarDays}
        />
      {/if}
    {:else if currentScope === 'month'}
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
              <span>{weekdayShortFormatter.format(day)}</span>
            {/each}
          </div>

          <!-- One tab stop: the arrow keys walk the grid from the focused cell. -->
          <div
            class="grid grid-cols-7 gap-1"
            role="grid"
            aria-label={m.calendar_month_grid_label({ period: periodLabel })}
            bind:this={monthGridRef}
          >
            {#each monthGridDays as day (day.toISOString())}
              {@const dayEvents = eventsForDay(day)}
              {@const isDayInMonth = isSameMonth(day, anchorDate)}
              {@const isDayToday = isSameDay(day, now)}
              {@const isDaySelected = isSameDay(day, activeDate)}

              <button
                type="button"
                role="gridcell"
                data-day={dayKey(day)}
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
                  date: dayFormatter.format(day),
                  courses: dayCountLabel(day),
                })}
                onclick={() => selectDate(day)}
                onkeydown={handleMonthKeydown}
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
        </div>

        <div class={cn(panel, 'flex flex-col gap-3 p-4 min-[54rem]:p-5')}>
          <header
            class="flex items-center justify-between gap-3 border-b border-border-subtle pb-3
                   lte-600:items-start"
          >
            <div class="min-w-0">
              <h3 class="mb-[0.15rem] text-lg font-extrabold wrap-anywhere"
                >{capitalizeFirst(dayFormatter.format(activeDate))}</h3
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
    {/if}
  </main>

  <!-- 3. Date picker. -->
  {#if pickerOpen}
    <Sheet
      title={m.calendar_pick_date_title()}
      closeLabel={m.close()}
      onClose={() => (pickerOpen = false)}
    >
      <div class="flex flex-col gap-3 p-4">
        <div class="flex items-center justify-between gap-2">
          <IconButton
            label={m.previous_period()}
            onclick={() => (pickerMonth = addMonths(pickerMonth, -1))}
          >
            <ChevronLeft size={18} strokeWidth={2.2} aria-hidden="true" />
          </IconButton>
          <strong class="text-md font-extrabold"
            >{capitalizeFirst(monthYearFormatter.format(pickerMonth))}</strong
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
            <span>{weekdayShortFormatter.format(day)}</span>
          {/each}
        </div>

        <div class="grid grid-cols-7 gap-1">
          {#each pickerDays as day (day.toISOString())}
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
                date: dayFormatter.format(day),
                courses: dayCountLabel(day),
              })}
              onclick={() => pickDate(day)}
            >
              <span class="text-sm font-bold tabular-nums">{day.getDate()}</span>
              {#if eventsForDay(day).length > 0}
                <span class="size-[0.3rem] rounded-full bg-primary-deep" aria-hidden="true"></span>
              {/if}
            </button>
          {/each}
        </div>

        <Button variant="outline" block onclick={() => pickDate(new Date())}>
          <CalendarCheck size={16} aria-hidden="true" />
          <span>{m.go_to_today()}</span>
        </Button>
      </div>
    </Sheet>
  {/if}

  <!-- 4. Course detail. -->
  <CourseDetailModal
    event={modalEvent}
    {locale}
    {now}
    onClose={() => (modalEvent = null)}
    {onOpenTempo}
  />
</div>
