<script lang="ts">
  import {
    CalendarCheck,
    CalendarOff,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    RefreshCw,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Button from '$lib/components/ui/Button.svelte';
  import FreshnessLabel from '$lib/components/ui/FreshnessLabel.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import { viewControls } from '$lib/state/view-controls.svelte';
  import CalendarMonthGrid from './CalendarMonthGrid.svelte';
  import CalendarTimeGrid from './CalendarTimeGrid.svelte';
  import CalendarViewSkeleton from './CalendarViewSkeleton.svelte';
  import CourseDetailModal from './CourseDetailModal.svelte';
  import DatePickerSheet from './DatePickerSheet.svelte';
  import { CalendarFormat } from './calendar-format.svelte';
  import { CalendarNavigation, monthGridDays } from './calendar-navigation.svelte';
  import { timeWindowFor } from './calendar-layout';
  import { addDays, dayKey, isSameDay, isSameMonth, startOfWeek } from './date-utils';
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

  const navigation = new CalendarNavigation(() => onPeriodChange);

  let modalEvent = $state<CalendarEvent | null>(null);
  let pickerOpen = $state(false);

  $effect.pre(() => {
    if (initialScope) navigation.scope = initialScope;
  });

  $effect(() => {
    if (selectedDate) navigation.adoptSelectedDate(selectedDate);
  });

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

  function eventsForDay(date: Date): CalendarEvent[] {
    return eventsByDay.get(dayKey(date)) ?? [];
  }

  const visibleWeekDaysCount = $derived(sundaysVisible ? 7 : 6);
  const weekStartDate = $derived(startOfWeek(navigation.anchorDate));
  const weekDays = $derived(
    Array.from({ length: visibleWeekDaysCount }, (_, i) => addDays(weekStartDate, i))
  );

  const format = new CalendarFormat(
    () => locale,
    () => ({
      scope: navigation.scope,
      anchorDate: navigation.anchorDate,
      activeDate: navigation.activeDate,
      weekDays,
    })
  );

  /**
   * The columns the current zoom draws, and the events inside them. The month
   * reads the whole displayed month so its cells share one band with the other
   * zooms — a mark's height means the same thing wherever you are standing.
   */
  const zoomDays = $derived.by(() => {
    if (navigation.scope === 'day') return [navigation.activeDate];
    if (navigation.scope === 'week') return weekDays;
    return monthGridDays(navigation.anchorDate).filter((day) =>
      isSameMonth(day, navigation.anchorDate)
    );
  });

  const zoomEvents = $derived(zoomDays.flatMap((day) => eventsForDay(day)));
  const timeWindow = $derived(timeWindowFor(zoomEvents));
  const hasEvents = $derived(zoomEvents.length > 0);

  const offToday = $derived(!isSameDay(navigation.activeDate, now));

  function handleCourseClick(event: CalendarEvent) {
    if (onEventClick) onEventClick(event);
    else modalEvent = event;
  }

  /** Zooming in on a day from the week's header row, and on a week from a month cell. */
  const zoomToDay = (date: Date) => navigation.zoomTo(date, 'day');
  const zoomToWeek = (date: Date) => navigation.zoomTo(date, 'week');

  /**
   * Horizontal drag moves the period, at every zoom. The week used to be
   * excluded because its columns owned that axis; now that the whole week fits
   * the width, nothing scrolls sideways and the gesture means one thing
   * everywhere.
   */
  const SWIPE_DISTANCE = 64;
  let swipeStartX = 0;
  let swipeStartY = 0;
  let swipeTracking = false;
  /** A recognised swipe owes the browser's synthetic click nothing. */
  let swipeConsumedClick = false;

  function handleSwipeStart(event: PointerEvent) {
    swipeTracking = event.pointerType !== 'mouse';
    swipeConsumedClick = false;
    swipeStartX = event.clientX;
    swipeStartY = event.clientY;
  }

  function handleSwipeEnd(event: PointerEvent) {
    if (!swipeTracking) return;
    swipeTracking = false;

    const deltaX = event.clientX - swipeStartX;
    const deltaY = event.clientY - swipeStartY;
    if (Math.abs(deltaX) < SWIPE_DISTANCE || Math.abs(deltaX) < Math.abs(deltaY) * 1.5) return;

    // The gesture started on whatever the finger landed on, a course block
    // included, and the browser still fires a click on it. Left alone that
    // opens the detail of a course from the period the swipe just left.
    swipeConsumedClick = true;
    navigation.movePeriod(deltaX < 0 ? 1 : -1);
  }

  function handleSwipeClick(event: MouseEvent) {
    if (!swipeConsumedClick) return;
    swipeConsumedClick = false;
    event.stopPropagation();
    event.preventDefault();
  }

  // On a compact window the controls are a row of the dock rather than a bar of
  // their own; see `view-controls.svelte.ts` for why. On an expanded window the
  // dock is hidden and the header below carries the same snippet.
  $effect(() => viewControls.claim(periodControls));

  const container =
    'flex w-full min-h-0 flex-1 flex-col gap-2 px-3 pt-2 pb-3' +
    ' md:gap-3 md:px-8 md:pt-6 md:pb-8' +
    ' lte-600:px-safe-2';
</script>

<!-- Previous period, the three zooms, next period. One snippet, rendered in the
     dock on a compact window and in this view's header on an expanded one. -->
{#snippet periodControls()}
  <IconButton label={m.previous_period()} variant="ghost" onclick={() => navigation.movePeriod(-1)}>
    <ChevronLeft size={19} strokeWidth={2.2} aria-hidden="true" />
  </IconButton>

  <SegmentedControl
    options={format.scopeOptions}
    value={navigation.scope}
    label={m.calendar_scope_label()}
    onChange={(scope) => navigation.setScope(scope as CalendarScope)}
    class="min-w-0 flex-1 border-transparent bg-muted md:max-w-[18rem]"
  />

  <IconButton label={m.next_period()} variant="ghost" onclick={() => navigation.movePeriod(1)}>
    <ChevronRight size={19} strokeWidth={2.2} aria-hidden="true" />
  </IconButton>
{/snippet}

<div class={container}>
  <!-- One line where the incumbent spent a 200px panel: the period names itself
       and is the door to the date sheet, the freshness statement keeps its
       place because honest state is not a polish item, and everything else
       moved into the dock's control row. -->
  <header class="flex shrink-0 flex-col border-b border-border-subtle pb-1">
    <div class="flex items-center gap-2 md:gap-3">
      <!-- The period is the door to the date sheet, which is why there is no
           separate calendar button beside it any more: the same sheet reached
           twice from one row is a control the row cannot afford at 390px. -->
      <button
        type="button"
        class="flex min-h-(--tap-min) min-w-0 items-center gap-1 rounded-sm pe-1 text-start
               transition-control active:scale-(--press-scale)"
        onclick={() => (pickerOpen = true)}
      >
        <span class="truncate text-base leading-tight font-extrabold text-foreground md:text-lg"
          >{format.periodLabel}</span
        >
        <ChevronDown
          size={16}
          strokeWidth={2.4}
          class="shrink-0 text-primary-deep"
          aria-hidden="true"
        />
        <span class="sr-only">{m.calendar_pick_date()}</span>
      </button>

      <!-- The expanded window has no dock, so the same controls sit here. -->
      <div class="desktop-only ms-auto hidden min-w-0 flex-1 items-center gap-2 md:flex">
        {@render periodControls()}
      </div>

      {#if onRefresh}
        <IconButton
          label={m.sync_refresh()}
          variant="ghost"
          {loading}
          class="ms-auto shrink-0 md:ms-0"
          onclick={() => void onRefresh?.()}
        >
          <RefreshCw size={17} strokeWidth={2.2} aria-hidden="true" />
        </IconButton>
      {/if}
    </div>

    <!-- Freshness always shows, so this row always exists — and giving the
         return-to-today action the space beside it is what keeps the grid from
         changing height every time the reader pages off today. -->
    <div class="flex min-h-7 items-center justify-between gap-2">
      <FreshnessLabel
        {fetchedAt}
        {locale}
        refreshing={loading}
        failed={refreshFailed}
        class="min-w-0"
      />
      {#if offToday}
        <Button variant="accent" size="sm" onclick={navigation.goToToday}>
          <CalendarCheck size={14} aria-hidden="true" />
          <span>{m.go_to_today()}</span>
        </Button>
      {/if}
    </div>
  </header>

  <!-- The zooms. Every one of them is laid out to fit the height it was given,
       so none of them scrolls sideways and the swipe below owns the horizontal
       axis outright. -->
  <main
    class="relative flex min-h-0 flex-1 flex-col"
    onpointerdown={handleSwipeStart}
    onpointerup={handleSwipeEnd}
    onpointercancel={() => (swipeTracking = false)}
    onclickcapture={handleSwipeClick}
  >
    {#if loading && events.length === 0}
      <CalendarViewSkeleton ariaLabel={m.planning_loading()} />
    {:else}
      <!-- Empty does not remove the grid. The band still says which day it is
           and which hours it covers, which a card standing where the grid was
           does not. The statement rides over it and takes no pointer, so the
           period can still be swiped away. -->
      {#if !hasEvents}
        <div
          class="pointer-events-none absolute inset-0 z-raised flex items-center justify-center p-6"
          role="status"
        >
          <div
            class="flex max-w-[20rem] flex-col items-center gap-2 rounded-lg border
                   border-border-subtle bg-card/92 px-5 py-4 text-center shadow-sm
                   backdrop-blur-[6px]"
          >
            <CalendarOff size={24} class="text-primary-deep" aria-hidden="true" />
            <p class="text-sm leading-tight font-extrabold text-foreground">
              {navigation.scope === 'day' ? m.no_courses_day() : m.no_events_period()}
            </p>
            <p class="text-xs leading-relaxed text-muted-foreground">
              {m.no_events_period_description()}
            </p>
          </div>
        </div>
      {/if}

      {#if navigation.scope === 'month'}
        <CalendarMonthGrid
          anchorDate={navigation.anchorDate}
          activeDate={navigation.activeDate}
          monthFocusDate={navigation.monthFocusDate}
          {now}
          {locale}
          {format}
          {timeWindow}
          {eventsForDay}
          onSelectDate={navigation.selectDate}
          onFocusDate={navigation.focusMonthDate}
          onZoomWeek={zoomToWeek}
          onEventClick={handleCourseClick}
          {onOpenTempo}
        />
      {:else}
        <CalendarTimeGrid
          days={navigation.scope === 'day' ? [navigation.activeDate] : weekDays}
          scope={navigation.scope}
          activeDate={navigation.activeDate}
          {now}
          {format}
          {eventsForDay}
          onZoomDay={zoomToDay}
          onEventClick={handleCourseClick}
        />
      {/if}
    {/if}
  </main>

  {#if pickerOpen}
    <DatePickerSheet
      activeDate={navigation.activeDate}
      {now}
      {format}
      {eventsForDay}
      onPick={(date) => {
        navigation.pickDate(date);
        pickerOpen = false;
      }}
      onClose={() => (pickerOpen = false)}
    />
  {/if}

  <CourseDetailModal
    event={modalEvent}
    {locale}
    {now}
    onClose={() => (modalEvent = null)}
    {onOpenTempo}
  />
</div>
