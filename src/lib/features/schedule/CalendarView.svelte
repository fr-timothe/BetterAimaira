<script lang="ts">
  import {
    Calendar,
    CalendarCheck,
    CalendarDays,
    CalendarSearch,
    ChevronLeft,
    ChevronRight,
    Clock,
    RefreshCw,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import FreshnessLabel from '$lib/components/ui/FreshnessLabel.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import CalendarDayStrip from './CalendarDayStrip.svelte';
  import CalendarMonthGrid from './CalendarMonthGrid.svelte';
  import CalendarTimeGrid from './CalendarTimeGrid.svelte';
  import CalendarViewSkeleton from './CalendarViewSkeleton.svelte';
  import CourseDetailModal from './CourseDetailModal.svelte';
  import DatePickerSheet from './DatePickerSheet.svelte';
  import { CalendarFormat } from './calendar-format.svelte';
  import { CalendarNavigation } from './calendar-navigation.svelte';
  import { panel } from './calendar-styles';
  import { addDays, dayKey, isSameDay, startOfWeek } from './date-utils';
  import { eventDurationMinutes, formatDuration } from './course-utils';
  import { gapMinutes } from './calendar-layout';
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
  const weekEvents = $derived(weekDays.flatMap((day) => eventsForDay(day)));

  const format = new CalendarFormat(
    () => locale,
    () => ({
      scope: navigation.scope,
      anchorDate: navigation.anchorDate,
      activeDate: navigation.activeDate,
      weekDays,
    })
  );

  const activeDateEvents = $derived(eventsForDay(navigation.activeDate));
  const activeDateDurationMinutes = $derived.by(() =>
    activeDateEvents.reduce((total, event) => total + eventDurationMinutes(event), 0)
  );
  const activeDateGapMinutes = $derived(gapMinutes(activeDateEvents));

  const ScopeIcon = $derived(
    navigation.scope === 'day' ? Clock : navigation.scope === 'week' ? CalendarDays : Calendar
  );

  function handleCourseClick(event: CalendarEvent) {
    if (onEventClick) onEventClick(event);
    else modalEvent = event;
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
  /** A recognised swipe owes the browser's synthetic click nothing. */
  let swipeConsumedClick = false;

  function handleSwipeStart(event: PointerEvent) {
    swipeTracking = event.pointerType !== 'mouse' && navigation.scope !== 'week';
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

  /**
   * Day and week hand the leftover height to the time grid. Month keeps its
   * natural size: its cells are a fixed grid, and squeezing them into whatever
   * is left is how a month view starts clipping its last week.
   */
  const fillsHeight = $derived(navigation.scope !== 'month');

  const container =
    'flex w-full flex-col gap-3 px-3 pt-3 pb-6' +
    ' md:gap-4 md:px-8 md:pt-6 md:pb-8' +
    ' lte-600:px-safe-2';
</script>

<div class={cn(container, fillsHeight && 'min-h-0 flex-1')}>
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
        <span>{format.scopeName}</span>
      </span>
      <h2
        class="max-w-full text-base leading-[1.3] font-extrabold wrap-anywhere text-foreground
               md:text-lg"
      >{format.periodLabel}</h2>
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
        <IconButton label={m.previous_period()} onclick={() => navigation.movePeriod(-1)}>
          <ChevronLeft size={18} strokeWidth={2.2} aria-hidden="true" />
        </IconButton>

        <Button variant="accent" size="sm" onclick={navigation.goToToday}>
          <CalendarCheck size={14} aria-hidden="true" />
          <span>{m.go_to_today()}</span>
        </Button>

        <IconButton label={m.next_period()} onclick={() => navigation.movePeriod(1)}>
          <ChevronRight size={18} strokeWidth={2.2} aria-hidden="true" />
        </IconButton>

        <IconButton label={m.calendar_pick_date()} onclick={() => (pickerOpen = true)}>
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
        options={format.scopeOptions}
        value={navigation.scope}
        label={m.calendar_scope_label()}
        onChange={(scope) => navigation.setScope(scope as CalendarScope)}
        class="md:w-[15rem]"
      />
    </div>
  </header>

  <!-- 2. Scope views. -->
  <main
    class={cn('relative flex flex-col gap-3', fillsHeight ? 'min-h-0 flex-1' : 'min-h-96')}
    onpointerdown={handleSwipeStart}
    onpointerup={handleSwipeEnd}
    onpointercancel={() => (swipeTracking = false)}
    onclickcapture={handleSwipeClick}
  >
    {#if loading && events.length === 0}
      <CalendarViewSkeleton ariaLabel={m.planning_loading()} />
    {:else if navigation.scope === 'day'}
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
        {#if isSameDay(navigation.activeDate, now)}
          <Badge tone="accent">{m.preview_today()}</Badge>
        {/if}
      </div>

      <CalendarDayStrip
        days={weekDays}
        activeDate={navigation.activeDate}
        {now}
        {format}
        {eventsForDay}
        onSelect={navigation.selectDate}
      />

      {#if activeDateEvents.length > 0}
        <CalendarTimeGrid
          days={[navigation.activeDate]}
          scope={navigation.scope}
          activeDate={navigation.activeDate}
          {now}
          {format}
          {eventsForDay}
          onSelectDate={navigation.selectDate}
          onEventClick={handleCourseClick}
        />
      {:else}
        <StateCard
          kind="empty"
          title={m.no_courses_day()}
          description={m.no_courses_day_description()}
          icon={CalendarCheck}
        />
      {/if}
    {:else if navigation.scope === 'week'}
      {#if weekEvents.length > 0}
        <CalendarTimeGrid
          days={weekDays}
          scope={navigation.scope}
          activeDate={navigation.activeDate}
          {now}
          {format}
          {eventsForDay}
          onSelectDate={navigation.selectDate}
          onEventClick={handleCourseClick}
        />
      {:else}
        <StateCard
          kind="empty"
          title={m.no_events_period()}
          description={m.no_courses_day_description()}
          icon={CalendarDays}
        />
      {/if}
    {:else if navigation.scope === 'month'}
      <CalendarMonthGrid
        anchorDate={navigation.anchorDate}
        activeDate={navigation.activeDate}
        monthFocusDate={navigation.monthFocusDate}
        {now}
        {locale}
        {format}
        {eventsForDay}
        onSelectDate={navigation.selectDate}
        onFocusDate={navigation.focusMonthDate}
        onEventClick={handleCourseClick}
        {onOpenTempo}
      />
    {/if}
  </main>

  <!-- 3. Date picker. -->
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

  <!-- 4. Course detail. -->
  <CourseDetailModal
    event={modalEvent}
    {locale}
    {now}
    onClose={() => (modalEvent = null)}
    {onOpenTempo}
  />
</div>
