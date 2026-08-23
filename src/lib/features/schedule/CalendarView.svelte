<script lang="ts">
  import { tick } from 'svelte';
  import {
    Calendar,
    CalendarCheck,
    CalendarDays,
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
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
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
    formatDurationRange,
    getEventStatus,
    openExternalUrl,
    parseRoomAndTeacher,
  } from './course-utils';
  import type { CalendarEvent, CalendarScope } from './types';

  type Props = {
    events: CalendarEvent[];
    locale: Locale;
    sundaysVisible?: boolean;
    initialScope?: CalendarScope;
    selectedDate?: Date;
    now?: Date;
    loading?: boolean;
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
    onPeriodChange,
    onRefresh,
    onEventClick,
    onOpenTempo,
  }: Props = $props();

  type CourseRowVariant = 'timeline' | 'compact' | 'week' | 'detailed';

  type CourseRowShape = {
    /** `inline` prints the whole range in one run; `stacked` splits start and end. */
    time: 'inline' | 'stacked';
    endTime: boolean;
    duration: boolean;
    rail: boolean;
    status: 'all' | 'live' | 'none';
    secondary: boolean;
    teacher: boolean;
    tempo: boolean;
  };

  /**
   * The four scopes show the same course at four levels of detail. Keeping that
   * difference as data is what lets a single snippet own the button, the
   * keyboard behaviour and the status logic; only the grid differs per scope.
   */
  const rowShape: Record<CourseRowVariant, CourseRowShape> = {
    timeline: {
      time: 'stacked',
      endTime: true,
      duration: true,
      rail: true,
      status: 'all',
      secondary: true,
      teacher: true,
      tempo: true,
    },
    compact: {
      time: 'inline',
      endTime: false,
      duration: false,
      rail: false,
      status: 'none',
      secondary: false,
      teacher: true,
      tempo: true,
    },
    week: {
      time: 'stacked',
      endTime: false,
      duration: true,
      rail: false,
      status: 'none',
      secondary: false,
      teacher: false,
      tempo: false,
    },
    detailed: {
      time: 'stacked',
      endTime: true,
      duration: false,
      rail: true,
      status: 'live',
      secondary: true,
      teacher: true,
      tempo: true,
    },
  };

  let currentScope = $state<CalendarScope>('week');
  let anchorDate = $state<Date>(startOfDay(new Date()));
  let activeDate = $state<Date>(startOfDay(new Date()));
  let modalEvent = $state<CalendarEvent | null>(null);
  let ribbonRef = $state<HTMLDivElement | null>(null);

  $effect.pre(() => {
    if (initialScope) currentScope = initialScope;
  });

  $effect(() => {
    if (selectedDate) {
      anchorDate = startOfDay(selectedDate);
      activeDate = startOfDay(selectedDate);
    }
  });

  $effect(() => {
    activeDate;
    currentScope;
    void tick().then(() => {
      // The global `prefers-reduced-motion` rule cannot reach a JS scroll
      // option, so the preference is read here as well.
      const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      ribbonRef
        ?.querySelector<HTMLButtonElement>('[aria-pressed="true"]')
        ?.scrollIntoView({
          behavior: reduceMotion ? 'auto' : 'smooth',
          block: 'nearest',
          inline: 'center',
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

  const shortDayFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      weekday: 'short',
      day: 'numeric',
    })
  );

  const weekdayShortFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      weekday: 'short',
    })
  );

  const monthYearFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      month: 'long',
      year: 'numeric',
    })
  );

  const timeFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      hour: '2-digit',
      minute: '2-digit',
    })
  );

  const rangeFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    })
  );

  const sortedEvents = $derived.by(() =>
    [...events].sort(
      (a, b) => new Date(a.startsAt).getTime() - new Date(b.startsAt).getTime()
    )
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
  const monthGridDays = $derived(
    Array.from({ length: 42 }, (_, i) => addDays(monthGridStart, i))
  );

  // A fixed Monday-first week, only ever used to print weekday column titles.
  const monthHeaderDays = Array.from(
    { length: 7 },
    (_, i) => new Date(2024, 0, 1 + i)
  );

  const activeDateEvents = $derived(eventsForDay(activeDate));

  const periodLabel = $derived.by(() => {
    switch (currentScope) {
      case 'day':
        return capitalizeFirst(dayFormatter.format(anchorDate));
      case 'week': {
        const wEnd = addDays(weekStartDate, visibleWeekDaysCount - 1);
        const wNum = getWeekNumber(weekStartDate);
        const formattedRange = rangeFormatter.formatRange(weekStartDate, wEnd);
        return m.calendar_week_range({ week: wNum, range: formattedRange });
      }
      case 'month':
        return capitalizeFirst(monthYearFormatter.format(anchorDate));
    }
  });

  const currentLiveMinutePercent = $derived.by(() => {
    const hours = now.getHours();
    const minutes = now.getMinutes();
    if (hours < 8) return 0;
    if (hours >= 20) return 100;
    const totalMinutesFrom8 = (hours - 8) * 60 + minutes;
    return (totalMinutesFrom8 / (12 * 60)) * 100;
  });

  const isCurrentTimeVisibleOnDay = $derived(
    isSameDay(anchorDate, now) && now.getHours() >= 8 && now.getHours() < 20
  );

  const activeDateDurationMinutes = $derived.by(() => {
    return activeDateEvents.reduce((total, event) => total + eventDurationMinutes(event), 0);
  });

  const availableScopes = $derived.by(() => {
    locale;
    return [
      { id: 'day' as CalendarScope, label: m.scope_day(), shortLabel: m.scope_day(), icon: Clock },
      { id: 'week' as CalendarScope, label: m.scope_week(), shortLabel: 'Sem.', icon: CalendarDays },
      { id: 'month' as CalendarScope, label: m.scope_month(), shortLabel: m.scope_month(), icon: Calendar },
    ];
  });

  function eventsForDay(date: Date): CalendarEvent[] {
    return eventsByDay.get(dayKey(date)) ?? [];
  }

  function eventTime(event: CalendarEvent): string {
    const start = new Date(event.startsAt);
    const end = new Date(event.endsAt);
    return `${timeFormatter.format(start)} – ${timeFormatter.format(end)}`;
  }

  function setScope(scope: CalendarScope) {
    currentScope = scope;
    triggerPeriodChange(anchorDate, scope);
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
    triggerPeriodChange(newAnchor, currentScope);
  }

  function goToToday() {
    const today = startOfDay(new Date());
    anchorDate = today;
    activeDate = today;
    triggerPeriodChange(today, currentScope);
  }

  function selectDate(date: Date) {
    activeDate = startOfDay(date);
    if (currentScope === 'day') {
      anchorDate = activeDate;
      triggerPeriodChange(activeDate, currentScope);
    }
  }

  function formatWeekInputValue(date: Date): string {
    const weekStart = startOfWeek(date);
    const isoYear = addDays(weekStart, 3).getFullYear();
    return `${isoYear}-W${String(getWeekNumber(date)).padStart(2, '0')}`;
  }

  function handleWeekInputChange(event: Event) {
    const input = event.currentTarget;
    if (!(input instanceof HTMLInputElement)) return;
    const match = input.value.match(/^(\d{4})-W(\d{2})$/);
    if (!match) return;

    const year = Number.parseInt(match[1], 10);
    const week = Number.parseInt(match[2], 10);
    const januaryFourth = startOfDay(new Date(year, 0, 4));
    const selectedWeek = addDays(startOfWeek(januaryFourth), (week - 1) * 7);
    anchorDate = selectedWeek;
    activeDate = selectedWeek;
    triggerPeriodChange(selectedWeek, currentScope);
  }

  const weekInputValue = $derived(formatWeekInputValue(anchorDate));

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
      case 'month':
        startDate = startOfWeek(startOfMonth(date));
        durationDays = 42;
        break;
    }

    void onPeriodChange(startDate, durationDays);
  }

  function handleCourseClick(event: CalendarEvent) {
    if (onEventClick) {
      onEventClick(event);
    } else {
      modalEvent = event;
    }
  }

  async function handleTempoClick(e: MouseEvent, event: CalendarEvent) {
    e.stopPropagation();
    if (onOpenTempo) {
      onOpenTempo(event);
      return;
    }
    await openExternalUrl(event.tempoUrl);
  }
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

{#snippet courseRow(event: CalendarEvent, variant: CourseRowVariant)}
  {@const status = getEventStatus(event)}
  {@const details = parseRoomAndTeacher(event)}
  {@const shape = rowShape[variant]}
  {@const secondary = shape.secondary ? eventSecondary(event) : null}
  {@const teacher = shape.teacher ? details.teacher : null}
  {@const showStatus = shape.status === 'all' || (shape.status === 'live' && status === 'live')}

  <div
    class="course-row row-{variant}"
    class:is-live={status === 'live'}
    class:is-finished={status === 'finished'}
  >
    <button type="button" class="row-open" onclick={() => handleCourseClick(event)}>
      <span class="row-time">
        {#if shape.time === 'inline'}
          <strong>{eventTime(event)}</strong>
        {:else}
          <strong>{timeFormatter.format(new Date(event.startsAt))}</strong>
          {#if shape.endTime}
            <span>{timeFormatter.format(new Date(event.endsAt))}</span>
          {/if}
        {/if}
        {#if shape.duration}
          <small>{formatDurationRange(event.startsAt, event.endsAt)}</small>
        {/if}
      </span>

      {#if shape.rail}
        <span class="row-rail">
          <span class="rail-node"></span>
          <span class="rail-bar"></span>
        </span>
      {/if}

      <span class="row-body">
        {#if showStatus || event.kind}
          <span class="row-tags">
            {#if showStatus}{@render statusBadge(status)}{/if}
            {#if event.kind}<KindBadge {event} />{/if}
          </span>
        {/if}

        <span class="row-title">{eventTitle(event)}</span>

        {#if secondary}
          <span class="row-subtitle">{secondary}</span>
        {/if}

        {#if details.room || teacher}
          <span class="row-meta">
            {#if details.room}
              <span class="meta-item"><MapPin size={14} aria-hidden="true" />{details.room}</span>
            {/if}
            {#if teacher}
              <span class="meta-item"><UserRound size={14} aria-hidden="true" />{teacher}</span>
            {/if}
          </span>
        {/if}
      </span>
    </button>

    {#if shape.tempo && event.tempoUrl}
      <button
        type="button"
        class="row-tempo"
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

<div class="calendar-container">
  <!-- 1. Scope Selector Bar (Pill buttons with always visible labels) -->
  <div class="scope-bar-wrapper">
    <div class="scope-segmented-bar" role="tablist" aria-label={m.calendar_scope_label()}>
      {#each availableScopes as s (s.id)}
        <button
          type="button"
          role="tab"
          class="scope-pill"
          class:active={currentScope === s.id}
          aria-selected={currentScope === s.id}
          aria-label={s.label}
          onclick={() => setScope(s.id)}
        >
          <s.icon size={15} strokeWidth={currentScope === s.id ? 2.5 : 1.9} aria-hidden="true" />
          <span class="scope-label-full">{s.label}</span>
          <span class="scope-label-short">{s.shortLabel}</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- 2. Period Navigation Header -->
  <header class="period-navigation-card">
    <div class="period-title-block">
      <span class="scope-indicator-tag">
        {#if currentScope === 'day'}
          <Clock size={12} aria-hidden="true" />
        {:else if currentScope === 'week'}
          <CalendarDays size={12} aria-hidden="true" />
        {:else if currentScope === 'month'}
          <Calendar size={12} aria-hidden="true" />
        {:else}
          <CalendarDays size={12} aria-hidden="true" />
        {/if}
        <span>{availableScopes.find((s) => s.id === currentScope)?.label ?? ''}</span>
      </span>
      <h2 class="period-label">{periodLabel}</h2>
    </div>

    <div class="nav-button-group">
      <button
        type="button"
        class="nav-icon-btn"
        aria-label={m.previous_period()}
        title={m.previous_period()}
        onclick={() => movePeriod(-1)}
      >
        <ChevronLeft size={18} strokeWidth={2.2} aria-hidden="true" />
      </button>

      <button
        type="button"
        class="today-pill-btn"
        onclick={goToToday}
      >
        <CalendarCheck size={14} aria-hidden="true" />
        <span>{m.go_to_today()}</span>
      </button>

      <button
        type="button"
        class="nav-icon-btn"
        aria-label={m.next_period()}
        title={m.next_period()}
        onclick={() => movePeriod(1)}
      >
        <ChevronRight size={18} strokeWidth={2.2} aria-hidden="true" />
      </button>

      {#if currentScope === 'week'}
        <input
          class="native-week-picker"
          type="week"
          value={weekInputValue}
          aria-label={m.scope_week()}
          title={m.scope_week()}
          onchange={handleWeekInputChange}
        />
      {/if}

      {#if onRefresh}
        <div class="period-actions desktop-only">
          <button
            type="button"
            class="nav-icon-btn"
            aria-label={m.sync_refresh()}
            title={m.sync_refresh()}
            disabled={loading}
            onclick={() => void onRefresh?.()}
          >
            <RefreshCw size={16} strokeWidth={2.2} class={loading ? 'icon-spinning' : ''} aria-hidden="true" />
          </button>
        </div>
      {/if}
    </div>
  </header>

  <!-- 3. Quick Date Selector Ribbon (visible in week and day scopes) -->
  {#if currentScope === 'week' || currentScope === 'day'}
    <div class="quick-ribbon-container" bind:this={ribbonRef}>
      {#each weekDays as day (day.toISOString())}
        {@const dayEventsCount = eventsForDay(day).length}
        {@const isDayToday = isSameDay(day, now)}
        {@const isDaySelected = isSameDay(day, activeDate)}

        <button
          type="button"
          class="ribbon-day-btn"
          class:today={isDayToday}
          class:active={isDaySelected}
          aria-pressed={isDaySelected}
          onclick={() => selectDate(day)}
        >
          <span class="ribbon-day-name">{weekdayShortFormatter.format(day)}</span>
          <span class="ribbon-day-number">{day.getDate()}</span>
          <span class="ribbon-dot-slot">
            {#if dayEventsCount > 0}
              <span class="ribbon-event-dot"></span>
            {/if}
          </span>
        </button>
      {/each}
    </div>
  {/if}

  <!-- 4. Interactive Scope Views -->
  <main class="scope-content-viewport">
    {#if loading && events.length > 0}
      <div class="calendar-loading-overlay" role="status" aria-live="polite">
        <Spinner size={28} />
        <span>{m.planning_loading()}</span>
      </div>
    {/if}

    {#if loading && events.length === 0}
      <CalendarViewSkeleton ariaLabel={m.planning_loading()} />
    <!-- SCOPE 1: 'day' (Jour - Vertical Timeline) -->
    {:else if currentScope === 'day'}
      <section class="day-timeline-view">
        <div class="timeline-header-card">
          <div>
            <p>
              {m.day_course_count({ count: activeDateEvents.length })}
              {#if activeDateDurationMinutes > 0}
                • {formatDuration(activeDateDurationMinutes, locale)}
              {/if}
            </p>
          </div>
          {#if isSameDay(anchorDate, now)}
            <Badge tone="accent">{m.preview_today()}</Badge>
          {/if}
        </div>

        {#if activeDateEvents.length > 0}
          <div class="day-schedule-track">
            <div class="timeline-cards-list">
              {#each activeDateEvents as event (event.id)}
                {@render courseRow(event, 'timeline')}
              {/each}
            </div>
          </div>
        {:else}
          <StateCard
            kind="empty"
            title={m.no_courses_day()}
            description={m.no_courses_day_description()}
            icon={CalendarCheck}
          />
        {/if}
      </section>

    <!-- SCOPE 2: 'week' (Semaine - Grille complète Desktop, liste groupée sur Mobile) -->
    {:else if currentScope === 'week'}
      <!-- Mobile Week View: every day is visible, grouped in chronological order. -->
      <div class="mobile-week-view">
        {#each weekDays as day (day.toISOString())}
          {@const dayEvents = eventsForDay(day)}
          {@const isDayToday = isSameDay(day, now)}
          {@const isDayActive = isSameDay(day, activeDate)}

          <section class="mobile-week-day" class:today={isDayToday} class:active={isDayActive}>
            <button
              type="button"
              class="mobile-week-day-header"
              aria-pressed={isDayActive}
              onclick={() => selectDate(day)}
            >
              <span class="mobile-week-day-copy">
                <strong>{capitalizeFirst(shortDayFormatter.format(day))}</strong>
                <span>{m.day_course_count({ count: dayEvents.length })}</span>
              </span>
              {#if isDayToday}
                <Badge tone="accent">{m.preview_today()}</Badge>
              {/if}
            </button>

            <div class="mobile-week-events">
              {#each dayEvents as event (event.id)}
                {@render courseRow(event, 'compact')}
              {:else}
                <p class="mobile-week-empty">{m.no_courses_day()}</p>
              {/each}
            </div>
          </section>
        {/each}
      </div>

      <!-- Desktop Week View (Multi-column grid 6 or 7 days) -->
      <section class="week-grid-view desktop-week-grid" style:--week-cols={visibleWeekDaysCount}>
        {#each weekDays as day (day.toISOString())}
          {@const dayEvents = eventsForDay(day)}
          {@const isDayToday = isSameDay(day, now)}
          {@const isDayActive = isSameDay(day, activeDate)}

          <div
            class="week-column-cell"
            class:column-today={isDayToday}
            class:column-active={isDayActive}
          >
            <button type="button" class="week-col-header" onclick={() => selectDate(day)}>
              <span class="week-col-date">
                <strong>{weekdayShortFormatter.format(day)}</strong>
                <span class="day-number-circle">{day.getDate()}</span>
              </span>
              <span class="week-count-badge">{dayEvents.length}</span>
            </button>

            <div class="week-col-body">
              {#each dayEvents as event (event.id)}
                {@render courseRow(event, 'week')}
              {:else}
                <div class="week-empty-slot">
                  <span>-</span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </section>

    <!-- SCOPE 3: 'month' (Mois - Grille calendrier interactif + détail du jour) -->
    {:else if currentScope === 'month'}
      <section class="month-scope-view">
        <div class="month-calendar-card">
          <!-- Weekday column titles -->
          <div class="month-grid-header">
            {#each monthHeaderDays as day (day.getTime())}
              <span>{weekdayShortFormatter.format(day)}</span>
            {/each}
          </div>

          <!-- 42-day Month Grid -->
          <div class="month-days-grid">
            {#each monthGridDays as day (day.toISOString())}
              {@const dayEvents = eventsForDay(day)}
              {@const isDayInMonth = isSameMonth(day, anchorDate)}
              {@const isDayToday = isSameDay(day, now)}
              {@const isDaySelected = isSameDay(day, activeDate)}

              <button
                type="button"
                class="month-cell-btn"
                class:out-of-month={!isDayInMonth}
                class:today={isDayToday}
                class:selected={isDaySelected}
                aria-pressed={isDaySelected}
                onclick={() => selectDate(day)}
              >
                <span class="cell-day-num">{day.getDate()}</span>

                <span class="cell-indicators">
                  {#if dayEvents.length > 0}
                    {#if dayEvents.length <= 3}
                      {#each dayEvents.slice(0, 3) as dayEvent (dayEvent.id)}
                        <span class="event-dot"></span>
                      {/each}
                    {:else}
                      <span class="event-dot"></span>
                      <span class="event-count-mini">+{dayEvents.length}</span>
                    {/if}
                  {/if}
                </span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Selected Day Detailed Courses List -->
        <div class="month-selected-day-panel">
          <header class="selected-day-header">
            <div>
              <h3>{capitalizeFirst(dayFormatter.format(activeDate))}</h3>
              <p>
                {m.day_course_count({ count: activeDateEvents.length })}
                {#if activeDateDurationMinutes > 0}
                  • {formatDuration(activeDateDurationMinutes, locale)}
                {/if}
              </p>
            </div>
            {#if isSameDay(activeDate, now)}
              <Badge tone="accent">{m.preview_today()}</Badge>
            {/if}
          </header>

          <div class="selected-day-events-list">
            {#if activeDateEvents.length > 0}
              {#each activeDateEvents as event (event.id)}
                {@render courseRow(event, 'detailed')}
              {/each}
            {:else}
              <p class="panel-empty">{m.no_courses_day_description()}</p>
            {/if}
          </div>
        </div>
      </section>

    {/if}
  </main>

  <!-- 5. Course Detail Modal Popup -->
  <CourseDetailModal
    event={modalEvent}
    {locale}
    {now}
    onClose={() => (modalEvent = null)}
    {onOpenTempo}
  />
</div>

<style>
  .calendar-container {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    width: 100%;
    margin: 0;
    padding: var(--space-3) var(--space-3) var(--space-6);
    box-sizing: border-box;
  }

  /* 1. Scope Segmented Bar */
  .scope-bar-wrapper {
    display: flex;
    justify-content: center;
    width: 100%;
  }

  .scope-segmented-bar {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    width: 100%;
    max-width: 30rem;
    gap: 2px;
    padding: 3px;
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    box-sizing: border-box;
  }

  .scope-pill {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    min-height: 2.35rem;
    padding: 0.25rem 0.2rem;
    color: var(--muted-foreground);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    white-space: nowrap;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .scope-pill:active {
    transform: scale(var(--press-scale));
  }

  .scope-pill.active {
    color: var(--primary-deep);
    background: var(--card);
    border-color: var(--border-subtle);
    font-weight: var(--weight-heavy);
    box-shadow: var(--shadow-xs);
  }

  .scope-label-full {
    display: none;
  }

  .scope-label-short {
    display: inline;
    font-size: var(--text-2xs);
    letter-spacing: -0.01em;
  }

  @media (min-width: 26rem) {
    .scope-label-full {
      display: inline;
    }

    .scope-label-short {
      display: none;
    }
  }

  @media (hover: hover) {
    .scope-pill:hover:not(.active) {
      color: var(--foreground);
      background: color-mix(in oklch, var(--card) 60%, transparent);
    }
  }

  /* 2. Period Navigation Header */
  .period-navigation-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2-5);
    padding: var(--space-3) var(--space-3-5);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xs);
  }

  .period-title-block {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.15rem;
    min-width: 0;
    flex: 1 1 auto;
  }

  .scope-indicator-tag {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    min-width: 0;
    color: var(--primary-deep);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    overflow-wrap: anywhere;
  }

  .period-label {
    margin: 0;
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-heavy);
    line-height: 1.3;
    overflow-wrap: anywhere;
    max-width: 100%;
  }

  .nav-button-group {
    display: flex;
    align-items: center;
    gap: var(--space-1-5);
    flex-shrink: 0;
  }

  .nav-icon-btn {
    display: grid;
    width: 2.25rem;
    height: 2.25rem;
    flex: 0 0 2.25rem;
    place-items: center;
    color: var(--foreground);
    background: var(--surface-sunken);
    border: 0;
    border-radius: 50%;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .nav-icon-btn:active {
    transform: scale(var(--press-scale));
  }

  .today-pill-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    min-height: 2.25rem;
    padding: 0 var(--space-2-5);
    color: var(--primary-deep);
    background: var(--muted);
    border: 0;
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .today-pill-btn:active {
    transform: scale(var(--press-scale));
  }

  .native-week-picker {
    min-width: 8.5rem;
    min-height: 2.25rem;
    padding: 0 var(--space-2);
    color: var(--foreground);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    font: inherit;
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    font-variant-numeric: tabular-nums;
  }

  @media (hover: hover) {
    .nav-icon-btn:hover {
      background: var(--muted);
    }

    .today-pill-btn:hover {
      background: var(--muted-strong);
    }
  }

  .period-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  :global(.icon-spinning) {
    animation: spin var(--duration-spin) linear infinite;
  }

  /* 3. Quick Date Selector Ribbon */
  .quick-ribbon-container {
    display: flex;
    gap: var(--space-2);
    overflow-x: auto;
    padding: var(--space-1);
    scrollbar-width: none;
    -webkit-overflow-scrolling: touch;
  }

  .quick-ribbon-container::-webkit-scrollbar {
    display: none;
  }

  .ribbon-day-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1 1 0;
    min-width: 3.5rem;
    min-height: 4.5rem;
    padding: var(--space-2) var(--space-1);
    color: var(--muted-foreground);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .ribbon-day-btn:active {
    transform: scale(var(--press-scale));
  }

  .ribbon-day-btn.today {
    border-color: var(--primary-deep);
  }

  .ribbon-day-btn.active {
    color: var(--primary-deep);
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  @media (hover: hover) {
    .ribbon-day-btn:hover {
      color: var(--foreground);
      border-color: var(--border);
    }
  }

  .ribbon-day-name {
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .ribbon-day-number {
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
  }

  .ribbon-dot-slot {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 0.5rem;
    margin-top: 0.15rem;
  }

  .ribbon-event-dot {
    width: 0.35rem;
    height: 0.35rem;
    background: var(--primary-deep);
    border-radius: 50%;
  }

  /* 4. Scope Viewport */
  .scope-content-viewport {
    position: relative;
    min-height: 24rem;
  }

  .calendar-loading-overlay {
    position: absolute;
    inset: 0;
    z-index: var(--z-raised);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--muted-foreground);
    font-size: var(--text-base);
    background: color-mix(in oklch, var(--card) 78%, transparent);
    backdrop-filter: blur(4px);
    border-radius: var(--radius-xl);
  }

  /* ------------------------------------------------------------------
     Shared course row. One markup, one keyboard contract, one status
     rendering; each scope only re-lays it out.
     ------------------------------------------------------------------ */
  .course-row {
    position: relative;
    min-width: 0;
  }

  .row-open {
    width: 100%;
    padding: 0;
    color: inherit;
    text-align: start;
    background: transparent;
    border: 0;
    font: inherit;
    cursor: pointer;
  }

  .row-time {
    display: flex;
    flex-direction: column;
    min-width: 0;
    font-variant-numeric: tabular-nums;
  }

  .row-time strong {
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-heavy);
  }

  .row-time span {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .row-time small {
    margin-top: var(--space-1);
    color: var(--primary-deep);
    font-size: var(--text-2xs);
    font-weight: var(--weight-semibold);
  }

  .row-rail {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .rail-node {
    z-index: var(--z-raised);
    width: 0.75rem;
    height: 0.75rem;
    background: var(--card);
    border: 2px solid var(--primary-deep);
    border-radius: 50%;
  }

  .is-live .rail-node {
    background: var(--primary);
  }

  .rail-bar {
    flex: 1;
    width: 2px;
    background: var(--border-subtle);
  }

  .row-body {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: var(--space-1);
  }

  .row-tags {
    display: flex;
    min-width: 0;
    max-width: 100%;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .row-title {
    min-width: 0;
    color: var(--foreground);
    font-size: var(--text-md);
    font-weight: var(--weight-heavy);
    line-height: 1.3;
    overflow-wrap: anywhere;
  }

  .row-subtitle {
    min-width: 0;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
    overflow-wrap: anywhere;
  }

  .row-meta {
    display: flex;
    min-width: 0;
    max-width: 100%;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-3);
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  .meta-item {
    display: inline-flex;
    min-width: 0;
    max-width: 100%;
    align-items: center;
    gap: var(--space-1);
    overflow-wrap: anywhere;
  }

  .row-tempo {
    display: inline-flex;
    min-height: var(--tap-min);
    align-items: center;
    gap: var(--space-1);
    padding: 0 var(--space-3);
    color: var(--primary-deep);
    background: var(--card);
    border: 1px solid var(--muted-strong);
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .row-tempo:active {
    transform: scale(var(--press-scale));
  }

  @media (hover: hover) {
    .row-tempo:hover {
      background: var(--muted);
    }
  }

  /* Variant: day timeline. Its surface covers only the third column, while both
     the open button and the Tempo action have to sit inside that surface — so it
     is painted by a grid-placed pseudo-element rather than by either of them. */
  .row-timeline {
    display: grid;
    grid-template-columns: 3.6rem 1rem minmax(0, 1fr);
    gap: 0 var(--space-2-5);
    transition: transform var(--duration-instant) var(--ease-out);
  }

  .row-timeline::before {
    grid-column: 3;
    grid-row: 1 / -1;
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
    content: '';
  }

  .row-timeline.is-live::before {
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  .row-timeline.is-finished .row-title {
    color: var(--muted-foreground);
  }

  .row-timeline .row-open {
    display: grid;
    grid-column: 1 / -1;
    grid-row: 1;
    grid-template-columns: 3.6rem 1rem minmax(0, 1fr);
    gap: 0 var(--space-2-5);
    align-items: stretch;
    border-radius: var(--radius-lg);
  }

  .row-timeline .row-time {
    padding-top: var(--space-2-5);
  }

  .row-timeline .rail-node {
    margin-top: var(--space-3-5);
  }

  .row-timeline .row-body {
    padding: var(--space-3-5);
    gap: var(--space-1-5);
  }

  .row-timeline .row-tempo {
    grid-column: 3;
    grid-row: 2;
    justify-self: start;
    margin: 0 var(--space-3-5) var(--space-3-5);
  }

  .row-timeline:active {
    transform: scale(var(--press-scale));
  }

  @media (hover: hover) {
    .row-timeline:hover::before {
      border-color: var(--primary-deep);
      box-shadow: var(--shadow-sm);
    }
  }

  /* Variant: 3-day compact card */
  .row-compact {
    display: grid;
    gap: var(--space-2);
    padding: var(--space-3);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .row-compact .row-open {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--space-1) var(--space-2);
    align-items: center;
    border-radius: var(--radius-sm);
  }

  /* `display: contents` lets the body's children join the row's own grid, so the
     time and the tags share a line without a second wrapper. */
  .row-compact .row-body {
    display: contents;
  }

  .row-compact .row-time {
    grid-area: 1 / 1;
  }

  .row-compact .row-time strong {
    color: var(--primary-deep);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
  }

  .row-compact .row-tags {
    grid-area: 1 / 2;
    justify-self: end;
  }

  .row-compact .row-title {
    grid-area: 2 / 1 / auto / -1;
    font-size: var(--text-base);
  }

  .row-compact .row-meta {
    grid-area: 3 / 1 / auto / -1;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-1);
    font-size: var(--text-xs);
  }

  .row-compact .row-tempo {
    justify-self: start;
  }

  .row-compact.is-live {
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  .row-compact:active {
    transform: scale(var(--press-scale));
  }

  /* Variant: week grid card */
  .row-week {
    display: grid;
    gap: var(--space-1);
    padding: var(--space-2);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .row-week .row-open {
    display: grid;
    gap: var(--space-1);
    border-radius: var(--radius-sm);
  }

  .row-week .row-body {
    display: contents;
  }

  .row-week .row-time {
    grid-row: 1;
    flex-direction: row;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-1);
  }

  .row-week .row-time strong {
    color: var(--primary-deep);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
  }

  .row-week .row-time small {
    margin-top: 0;
    color: var(--muted-foreground);
  }

  .row-week .row-title {
    display: -webkit-box;
    grid-row: 2;
    overflow: hidden;
    font-size: var(--text-sm);
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .row-week .row-meta {
    grid-row: 3;
    font-size: var(--text-2xs);
  }

  .row-week .row-tags {
    grid-row: 4;
    justify-self: start;
  }

  .row-week.is-live {
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  .row-week:active {
    transform: scale(var(--press-scale));
  }

  /* Variant: month panel detailed row */
  .row-detailed {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--space-2);
    align-items: center;
    padding: var(--space-3);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .row-detailed .row-open {
    display: grid;
    grid-template-columns: 3.5rem 3px minmax(0, 1fr);
    gap: var(--space-2-5);
    align-items: stretch;
    border-radius: var(--radius-sm);
  }

  .row-detailed .row-time {
    padding-top: 0.15rem;
  }

  .row-detailed .row-time strong {
    font-size: var(--text-base);
    font-weight: var(--weight-heavy);
  }

  .row-detailed .row-time span {
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }

  .row-detailed .rail-node {
    display: none;
  }

  .row-detailed .row-rail {
    height: 100%;
    display: flex;
    align-items: center;
  }

  .row-detailed .rail-bar {
    width: 3px;
    height: 100%;
    background: var(--primary-deep);
    border-radius: var(--radius-pill);
  }

  .row-detailed .row-body {
    padding-left: 0;
    gap: var(--space-1);
  }

  .row-detailed .row-title {
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
  }

  .row-detailed .row-subtitle,
  .row-detailed .row-meta {
    font-size: var(--text-xs);
  }

  .row-detailed.is-live {
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  .row-detailed:active {
    transform: scale(var(--press-scale));
  }

  @media (hover: hover) {
    .row-compact:hover,
    .row-week:hover,
    .row-detailed:hover {
      border-color: var(--primary-deep);
      box-shadow: var(--shadow-sm);
    }
  }

  /* SCOPE 1: Day View */
  .day-timeline-view {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .timeline-header-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-4) var(--space-5);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .timeline-header-card > div,
  .selected-day-header > div {
    min-width: 0;
  }

  .timeline-header-card p {
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
  }

  .day-schedule-track {
    padding: var(--space-5);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .time-grid-container {
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .live-time-indicator {
    position: absolute;
    right: 0;
    left: 0;
    z-index: var(--z-raised);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    pointer-events: none;
    transform: translateY(-50%);
  }

  .live-pill {
    padding: 0.2rem var(--space-2);
    color: var(--card);
    background: var(--primary-deep);
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
  }

  .live-line {
    flex: 1;
    height: 2px;
    background: var(--primary-deep);
  }

  .timeline-cards-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  /* SCOPE 2: Week View */
  .week-grid-view {
    display: grid;
    grid-template-columns: repeat(var(--week-cols, 6), minmax(9rem, 1fr));
    gap: var(--space-2);
    overflow-x: auto;
  }

  .week-column-cell {
    display: flex;
    flex-direction: column;
    min-height: 18rem;
    padding: var(--space-3);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .week-column-cell.column-today {
    border-color: var(--primary-deep);
  }

  .week-col-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-height: var(--tap-min);
    padding: 0 0 var(--space-2);
    margin-bottom: var(--space-2);
    color: inherit;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--border-subtle);
    font: inherit;
    cursor: pointer;
  }

  .week-col-date {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .week-col-date strong {
    color: var(--foreground);
    font-size: var(--text-sm);
    font-weight: var(--weight-heavy);
    text-transform: uppercase;
  }

  .day-number-circle {
    display: grid;
    width: 1.5rem;
    height: 1.5rem;
    place-items: center;
    color: var(--foreground);
    background: var(--surface-sunken);
    border-radius: 50%;
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
  }

  .column-today .day-number-circle {
    color: var(--primary-foreground);
    background: var(--primary);
  }

  .week-count-badge {
    padding: 0.2rem var(--space-2);
    color: var(--muted-foreground);
    background: var(--surface-sunken);
    border-radius: var(--radius-pill);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
  }

  .week-col-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    flex: 1;
  }

  .week-empty-slot {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 4rem;
    color: var(--muted-foreground);
  }

  /* SCOPE 3: Month View */
  .month-scope-view {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-4);
  }

  .month-calendar-card {
    padding: var(--space-5);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .month-grid-header {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    text-align: center;
    margin-bottom: var(--space-3);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    text-transform: uppercase;
  }

  .month-days-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: var(--space-1);
  }

  .month-cell-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-1);
    min-height: var(--tap-min);
    padding: var(--space-2) var(--space-1);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .month-cell-btn:active {
    transform: scale(var(--press-scale));
  }

  .month-cell-btn.out-of-month {
    opacity: 0.55;
  }

  .month-cell-btn.today {
    border-color: var(--primary-deep);
  }

  .month-cell-btn.today .cell-day-num,
  .month-cell-btn.selected .cell-day-num {
    color: var(--primary-deep);
    font-weight: var(--weight-heavy);
  }

  .month-cell-btn.selected {
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  @media (hover: hover) {
    .month-cell-btn:hover {
      background: var(--muted);
      border-color: var(--primary-deep);
    }
  }

  .cell-day-num {
    color: var(--foreground);
    font-size: var(--text-sm);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
  }

  .cell-indicators {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.2rem;
    min-height: 0.5rem;
  }

  .event-dot {
    width: 0.35rem;
    height: 0.35rem;
    background: var(--primary-deep);
    border-radius: 50%;
  }

  .event-count-mini {
    color: var(--primary-deep);
    font-size: var(--text-2xs);
    font-weight: var(--weight-heavy);
    line-height: 1;
  }

  /* Mobile Week View */
  .mobile-week-view {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .mobile-week-day {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .mobile-week-day-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    width: 100%;
    min-height: var(--tap-min);
    padding: var(--space-2) var(--space-3);
    color: inherit;
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    box-sizing: border-box;
    font: inherit;
    text-align: start;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .mobile-week-day-header:active {
    transform: scale(var(--press-scale));
  }

  .mobile-week-day.today .mobile-week-day-header {
    background: var(--muted);
    border-color: var(--primary-deep);
  }

  .mobile-week-day.active .mobile-week-day-header {
    box-shadow: inset 0 0 0 2px var(--primary-deep);
  }

  .mobile-week-day-copy {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 0.15rem;
  }

  .mobile-week-day-copy strong {
    color: var(--foreground);
    font-size: var(--text-md);
    font-weight: var(--weight-heavy);
    overflow-wrap: anywhere;
  }

  .mobile-week-day-copy span {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .mobile-week-events {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: 0 var(--space-1);
  }

  .mobile-week-empty {
    margin: 0;
    padding: var(--space-3);
    color: var(--muted-foreground);
    background: var(--surface-sunken);
    border: 1px dashed var(--border-subtle);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }

  .mobile-week-day + .mobile-week-day {
    padding-top: var(--space-1);
  }

  .mobile-week-day + .mobile-week-day::before {
    height: 1px;
    margin-bottom: var(--space-2);
    background: var(--border-subtle);
    content: '';
  }

  .mobile-week-day-header:focus-visible {
    outline: 2px solid var(--primary-deep);
    outline-offset: 2px;
  }

  .mobile-week-day-header :global(.ui-badge) {
    flex: 0 0 auto;
  }

  @media (hover: hover) {
    .mobile-week-day-header:hover {
      background: var(--muted);
    }
  }

  .desktop-week-grid {
    display: none;
  }

  .month-selected-day-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
  }

  .selected-day-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .selected-day-header h3 {
    margin: 0 0 0.15rem;
    font-size: var(--text-lg);
    font-weight: var(--weight-heavy);
    overflow-wrap: anywhere;
  }

  .selected-day-header p {
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .selected-day-events-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2-5);
  }

  .panel-empty {
    margin: 0;
    padding: var(--space-6) var(--space-4);
    color: var(--muted-foreground);
    background: var(--surface-sunken);
    border-radius: var(--radius-lg);
    font-size: var(--text-base);
    text-align: center;
  }

  /* Responsive: 48rem is the primary hinge; the two secondary steps below exist
     because the week grid and the month split need more room than that. */
  @media (min-width: 48rem) {
    .calendar-container {
      gap: var(--space-5);
      padding: var(--space-6) var(--space-8) var(--space-8);
    }

    .mobile-week-view {
      display: none;
    }

    .desktop-week-grid {
      display: grid;
    }

    .period-label {
      font-size: var(--text-lg);
    }

    .row-timeline {
      grid-template-columns: 4.25rem 1.25rem minmax(0, 1fr);
      gap: 0 var(--space-3);
    }

    .row-timeline .row-open {
      grid-template-columns: 4.25rem 1.25rem minmax(0, 1fr);
      gap: 0 var(--space-3);
    }

    .row-timeline .row-time {
      padding-top: var(--space-3);
    }

    .row-timeline .rail-node {
      margin-top: var(--space-4);
    }

    .row-timeline .row-body {
      padding: var(--space-4);
      gap: var(--space-2);
    }

    .row-timeline .row-tempo {
      margin: 0 var(--space-4) var(--space-4);
    }

  }

  @media (min-width: 54rem) {
    .month-scope-view {
      grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
    }

    .month-selected-day-panel {
      padding: var(--space-5);
    }

    .selected-day-events-list {
      overflow-y: auto;
      max-height: 28rem;
    }
  }

  @media (min-width: 56rem) {
    .week-grid-view {
      grid-template-columns: repeat(var(--week-cols, 6), minmax(0, 1fr));
    }
  }

  @media (max-width: 30rem) {
    .calendar-container {
      padding-right: max(var(--space-2), env(safe-area-inset-right));
      padding-left: max(var(--space-2), env(safe-area-inset-left));
    }

    .period-navigation-card {
      align-items: stretch;
      flex-wrap: wrap;
    }

    .period-title-block {
      flex-basis: 100%;
    }

    .nav-button-group {
      width: 100%;
      justify-content: space-between;
    }

    .row-detailed .row-open {
      grid-template-columns: 3.25rem 3px minmax(0, 1fr);
      gap: var(--space-2);
    }

    .timeline-header-card,
    .selected-day-header {
      align-items: flex-start;
    }

  }
</style>
