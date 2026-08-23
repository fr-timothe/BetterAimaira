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
  import { cn } from '$lib/utils';

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

  /**
   * The same four levels of detail, as classes. Every element the row owns has
   * one entry per variant, so a scope's whole look is readable in one place
   * instead of spread over four blocks of descendant selectors.
   */
  type RowSkin = {
    root: string;
    open: string;
    time: string;
    strong: string;
    span: string;
    small: string;
    rail: string;
    node: string;
    bar: string;
    body: string;
    tags: string;
    title: string;
    subtitle: string;
    meta: string;
    tempo: string;
  };

  const rowBase = {
    root: 'course-row relative min-w-0',
    open: 'w-full cursor-pointer bg-transparent p-0 text-start text-inherit',
    time: 'flex min-w-0 flex-col tabular-nums',
    strong: 'text-base font-extrabold text-foreground',
    span: 'text-xs text-muted-foreground',
    small: 'mt-1 text-2xs font-semibold text-primary-deep',
    rail: 'relative flex flex-col items-center',
    node: 'z-raised size-3 rounded-full border-2 border-primary-deep bg-card',
    bar: 'w-0.5 flex-1 bg-border-subtle',
    body: 'flex min-w-0 flex-col gap-1',
    tags: 'flex min-w-0 max-w-full flex-wrap items-center gap-2',
    // The leading is applied after the variant size on purpose: tailwind-merge
    // reads `text-base` as a size-and-leading shorthand and would drop an earlier
    // `leading-*`.
    title: 'min-w-0 text-md font-extrabold wrap-anywhere text-foreground',
    subtitle: 'min-w-0 text-sm wrap-anywhere text-muted-foreground',
    meta:
      'flex min-w-0 max-w-full flex-wrap items-center gap-x-3 gap-y-1 text-sm text-muted-foreground',
    tempo:
      'inline-flex min-h-(--tap-min) items-center gap-1 rounded-pill border border-muted-strong' +
      ' bg-card px-3 text-xs font-bold text-primary-deep transition-control' +
      ' active:scale-(--press-scale) hover:bg-muted'
  } as const satisfies RowSkin;

  /** The three card variants share one surface and one press/hover behaviour. */
  const metaItem = 'inline-flex min-w-0 max-w-full items-center gap-1 wrap-anywhere';

  const cardSurface =
    'rounded-lg border transition-control active:scale-(--press-scale)' +
    ' hover:border-primary-deep hover:shadow-sm';

  // The timeline widens its own rail column once there is room for it.
  const timelineGrid =
    'grid-cols-[3.6rem_1rem_minmax(0,1fr)] gap-x-2.5 gap-y-0' +
    ' md:grid-cols-[4.25rem_1.25rem_minmax(0,1fr)] md:gap-x-3';

  const rowSkin: Record<CourseRowVariant, Partial<RowSkin>> = {
    // Its surface covers only the third column, while both the open button and
    // the Tempo action sit inside it — so it is painted by a grid-placed
    // pseudo-element rather than by either of them.
    timeline: {
      root:
        'grid ' + timelineGrid + ' transition-transform duration-instant ease-out' +
        ' active:scale-(--press-scale)' +
        " before:col-start-3 before:row-start-1 before:row-end-[-1] before:content-['']" +
        ' before:rounded-lg before:border' +
        ' before:transition-[background-color,border-color,box-shadow] before:duration-fast' +
        ' before:ease-out hover:before:border-primary-deep hover:before:shadow-sm',
      open: 'grid col-span-full row-start-1 items-stretch rounded-lg ' + timelineGrid,
      time: 'pt-2.5 md:pt-3',
      node: 'mt-3.5 md:mt-4',
      body: 'gap-1.5 p-3.5 md:gap-2 md:p-4',
      tempo:
        'col-start-3 row-start-2 mx-3.5 mt-0 mb-3.5 justify-self-start md:mx-4 md:mb-4'
    },
    compact: {
      root: 'grid gap-2 p-3 ' + cardSurface,
      open: 'grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-2 gap-y-1 rounded-sm',
      // `contents` lets the body's children join the row's own grid, so the time
      // and the tags share a line without a second wrapper.
      body: 'contents',
      time: '[grid-area:1/1]',
      strong: 'text-xs font-bold text-primary-deep',
      tags: '[grid-area:1/2] justify-self-end',
      title: '[grid-area:2/1/auto/-1] text-base',
      meta: '[grid-area:3/1/auto/-1] flex-col items-start gap-1 text-xs',
      tempo: 'justify-self-start'
    },
    week: {
      root: 'grid gap-1 rounded-md border p-2 transition-control active:scale-(--press-scale)' +
        ' hover:border-primary-deep hover:shadow-sm',
      open: 'grid gap-1 rounded-sm',
      body: 'contents',
      time: 'row-start-1 flex-row items-baseline justify-between gap-1',
      strong: 'text-2xs font-bold text-primary-deep',
      small: 'mt-0 text-muted-foreground',
      title: 'row-start-2 line-clamp-2 text-sm',
      meta: 'row-start-3 text-2xs',
      tags: 'row-start-4 justify-self-start'
    },
    detailed: {
      root: 'grid grid-cols-[minmax(0,1fr)_auto] items-center gap-2 p-3 ' + cardSurface,
      open:
        'grid grid-cols-[3.5rem_3px_minmax(0,1fr)] items-stretch gap-2.5 rounded-sm' +
        ' lte-600:grid-cols-[3.25rem_3px_minmax(0,1fr)] lte-600:gap-2',
      time: 'pt-[0.15rem]',
      node: 'hidden',
      rail: 'h-full items-center',
      bar: 'h-full w-[3px] rounded-pill bg-primary-deep',
      body: 'gap-1 pl-0',
      title: 'text-base font-bold',
      subtitle: 'text-xs',
      meta: 'text-xs'
    }
  };

  /** The live and finished states paint the row, so they close over the variant. */
  function rowInk(variant: CourseRowVariant, live: boolean) {
    if (variant === 'timeline') {
      return live
        ? 'before:border-primary-deep before:bg-muted'
        : 'before:border-border-subtle before:bg-surface-sunken';
    }
    return live
      ? 'border-primary-deep bg-muted'
      : 'border-border-subtle bg-surface-sunken';
  }

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
  const container =
    'flex w-full flex-col gap-3 px-3 pt-3 pb-6' +
    ' md:gap-5 md:px-8 md:pt-6 md:pb-8' +
    ' lte-600:pr-[max(var(--space-2),env(safe-area-inset-right))]' +
    ' lte-600:pl-[max(var(--space-2),env(safe-area-inset-left))]';

  const scopePill =
    'relative flex min-h-[2.35rem] items-center justify-center gap-1 rounded-md border' +
    ' border-transparent px-[0.2rem] py-1 text-xs font-semibold whitespace-nowrap' +
    ' transition-control active:scale-(--press-scale)';

  const navIconBtn =
    'grid size-9 flex-none place-items-center rounded-full bg-surface-sunken text-foreground' +
    ' transition-control active:scale-(--press-scale) hover:bg-muted';

  const panel = 'rounded-xl border border-border-subtle bg-card';

  const ribbonDayBtn =
    'flex min-h-18 min-w-14 flex-1 basis-0 flex-col items-center justify-center rounded-lg' +
    ' border bg-card px-1 py-2 transition-control active:scale-(--press-scale)';

  const dayHeaderRow =
    'flex items-center justify-between gap-3 lte-600:items-start';

  const uppercaseTiny =
    'text-xs font-bold tracking-[0.04em] uppercase';

  const dot = 'size-[0.35rem] rounded-full bg-primary-deep';
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
  {@const skin = rowSkin[variant]}
  {@const live = status === 'live'}

  <div class={cn(rowBase.root, skin.root, rowInk(variant, live))}>
    <button
      type="button"
      class={cn(rowBase.open, skin.open)}
      onclick={() => handleCourseClick(event)}
    >
      <span class={cn(rowBase.time, skin.time)}>
        {#if shape.time === 'inline'}
          <strong class={cn(rowBase.strong, skin.strong)}>{eventTime(event)}</strong>
        {:else}
          <strong class={cn(rowBase.strong, skin.strong)}
            >{timeFormatter.format(new Date(event.startsAt))}</strong
          >
          {#if shape.endTime}
            <span class={cn(rowBase.span, skin.span)}
              >{timeFormatter.format(new Date(event.endsAt))}</span
            >
          {/if}
        {/if}
        {#if shape.duration}
          <small class={cn(rowBase.small, skin.small)}
            >{formatDurationRange(event.startsAt, event.endsAt)}</small
          >
        {/if}
      </span>

      {#if shape.rail}
        <span class={cn(rowBase.rail, skin.rail)}>
          <span class={cn(rowBase.node, skin.node, live && 'bg-primary')}></span>
          <span class={cn(rowBase.bar, skin.bar)}></span>
        </span>
      {/if}

      <span class={cn(rowBase.body, skin.body)}>
        {#if showStatus || event.kind}
          <span class={cn(rowBase.tags, skin.tags)}>
            {#if showStatus}{@render statusBadge(status)}{/if}
            {#if event.kind}<KindBadge {event} />{/if}
          </span>
        {/if}

        <span
          class={cn(
            rowBase.title,
            skin.title,
            'leading-[1.3]',
            variant === 'timeline' && status === 'finished' && 'text-muted-foreground'
          )}>{eventTitle(event)}</span
        >

        {#if secondary}
          <span class={cn(rowBase.subtitle, skin.subtitle)}>{secondary}</span>
        {/if}

        {#if details.room || teacher}
          <span class={cn(rowBase.meta, skin.meta)}>
            {#if details.room}
              <span class={metaItem}><MapPin size={14} aria-hidden="true" />{details.room}</span>
            {/if}
            {#if teacher}
              <span class={metaItem}><UserRound size={14} aria-hidden="true" />{teacher}</span>
            {/if}
          </span>
        {/if}
      </span>
    </button>

    {#if shape.tempo && event.tempoUrl}
      <button
        type="button"
        class={cn(rowBase.tempo, skin.tempo)}
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
  <!-- 1. Scope Selector Bar (Pill buttons with always visible labels) -->
  <div class="flex w-full justify-center">
    <div
      class="grid w-full max-w-[30rem] grid-cols-3 gap-[2px] rounded-lg border
             border-border-subtle bg-surface-sunken p-[3px]"
      role="tablist"
      aria-label={m.calendar_scope_label()}
    >
      {#each availableScopes as s (s.id)}
        <button
          type="button"
          role="tab"
          class={cn(
            scopePill,
            currentScope === s.id
              ? 'border-border-subtle bg-card font-extrabold text-primary-deep shadow-xs'
              : 'bg-transparent text-muted-foreground hover:bg-card-hover hover:text-foreground'
          )}
          aria-selected={currentScope === s.id}
          aria-label={s.label}
          onclick={() => setScope(s.id)}
        >
          <s.icon size={15} strokeWidth={currentScope === s.id ? 2.5 : 1.9} aria-hidden="true" />
          <span class="hidden min-[26rem]:inline">{s.label}</span>
          <span class="inline text-2xs tracking-[-0.01em] min-[26rem]:hidden"
            >{s.shortLabel}</span
          >
        </button>
      {/each}
    </div>
  </div>

  <!-- 2. Period Navigation Header -->
  <header
    class={cn(
      panel,
      'flex items-center justify-between gap-2.5 px-3.5 py-3 shadow-xs',
      'lte-600:flex-wrap lte-600:items-stretch'
    )}
  >
    <div
      class="flex min-w-0 flex-auto flex-col items-start gap-[0.15rem] lte-600:basis-full"
    >
      <span
        class="inline-flex min-w-0 items-center gap-1 text-2xs font-bold tracking-[0.05em]
               uppercase wrap-anywhere text-primary-deep"
      >
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
      <h2
        class="period-label max-w-full text-base leading-[1.3] font-extrabold wrap-anywhere
               text-foreground md:text-lg"
      >{periodLabel}</h2>
    </div>

    <div
      class="flex flex-none items-center gap-1.5 lte-600:w-full lte-600:justify-between"
    >
      <button
        type="button"
        class={navIconBtn}
        aria-label={m.previous_period()}
        title={m.previous_period()}
        onclick={() => movePeriod(-1)}
      >
        <ChevronLeft size={18} strokeWidth={2.2} aria-hidden="true" />
      </button>

      <button
        type="button"
        class="inline-flex min-h-9 items-center gap-1 rounded-pill bg-muted
               px-2.5 text-xs font-bold text-primary-deep transition-control
               active:scale-(--press-scale) hover:bg-muted-strong"
        onclick={goToToday}
      >
        <CalendarCheck size={14} aria-hidden="true" />
        <span>{m.go_to_today()}</span>
      </button>

      <button
        type="button"
        class={navIconBtn}
        aria-label={m.next_period()}
        title={m.next_period()}
        onclick={() => movePeriod(1)}
      >
        <ChevronRight size={18} strokeWidth={2.2} aria-hidden="true" />
      </button>

      {#if currentScope === 'week'}
        <input
          class="min-h-9 min-w-[8.5rem] rounded-md border border-border-subtle bg-surface-sunken
                 px-2 text-xs font-semibold tabular-nums text-foreground"
          type="week"
          value={weekInputValue}
          aria-label={m.scope_week()}
          title={m.scope_week()}
          onchange={handleWeekInputChange}
        />
      {/if}

      {#if onRefresh}
        <!-- `desktop-only` owns the display here; a display utility would lose to
             its !important rules without a word. -->
        <div class="desktop-only items-center gap-2">
          <button
            type="button"
            class={navIconBtn}
            aria-label={m.sync_refresh()}
            title={m.sync_refresh()}
            disabled={loading}
            onclick={() => void onRefresh?.()}
          >
            <RefreshCw size={16} strokeWidth={2.2} class={loading ? 'animate-spin' : ''} aria-hidden="true" />
          </button>
        </div>
      {/if}
    </div>
  </header>

  <!-- 3. Quick Date Selector Ribbon (visible in week and day scopes) -->
  {#if currentScope === 'week' || currentScope === 'day'}
    <div
      class="flex gap-2 overflow-x-auto p-1 scrollbar-none
             [-webkit-overflow-scrolling:touch] [&::-webkit-scrollbar]:hidden"
      bind:this={ribbonRef}
    >
      {#each weekDays as day (day.toISOString())}
        {@const dayEventsCount = eventsForDay(day).length}
        {@const isDayToday = isSameDay(day, now)}
        {@const isDaySelected = isSameDay(day, activeDate)}

        <button
          type="button"
          class={cn(
            ribbonDayBtn,
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
          <span class="mt-[0.15rem] flex h-2 items-center justify-center">
            {#if dayEventsCount > 0}
              <span class={dot}></span>
            {/if}
          </span>
        </button>
      {/each}
    </div>
  {/if}

  <!-- 4. Interactive Scope Views -->
  <main class="relative min-h-96">
    {#if loading && events.length > 0}
      <div
        class="absolute inset-0 z-raised flex flex-col items-center justify-center gap-3
               rounded-xl bg-card-scrim text-base text-muted-foreground backdrop-blur-[4px]"
        role="status"
        aria-live="polite"
      >
        <Spinner size={28} />
        <span>{m.planning_loading()}</span>
      </div>
    {/if}

    {#if loading && events.length === 0}
      <CalendarViewSkeleton ariaLabel={m.planning_loading()} />
    <!-- SCOPE 1: 'day' (Jour - Vertical Timeline) -->
    {:else if currentScope === 'day'}
      <section class="flex flex-col gap-4">
        <div class={cn(panel, dayHeaderRow, 'px-5 py-4')}>
          <div class="min-w-0">
            <p class="text-sm text-muted-foreground">
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
          <div class={cn(panel, 'p-5')}>
            <div class="flex flex-col gap-3">
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
      <div class="flex flex-col gap-3 md:hidden">
        {#each weekDays as day (day.toISOString())}
          {@const dayEvents = eventsForDay(day)}
          {@const isDayToday = isSameDay(day, now)}
          {@const isDayActive = isSameDay(day, activeDate)}

          <section class="mobile-week-day flex flex-col gap-3">
            <button
              type="button"
              class={cn(
                'flex min-h-(--tap-min) w-full items-center justify-between gap-2 rounded-lg',
                'border px-3 py-2 text-start text-inherit transition-control',
                'active:scale-(--press-scale) hover:bg-muted',
                'focus-visible:outline-2 focus-visible:outline-offset-2',
                'focus-visible:outline-primary-deep',
                isDayToday ? 'border-primary-deep bg-muted' : 'border-border-subtle bg-card',
                isDayActive && 'shadow-[inset_0_0_0_2px_var(--primary-deep)]'
              )}
              aria-pressed={isDayActive}
              onclick={() => selectDate(day)}
            >
              <span class="flex min-w-0 flex-col gap-[0.15rem]">
                <strong class="text-md font-extrabold wrap-anywhere text-foreground"
                  >{capitalizeFirst(shortDayFormatter.format(day))}</strong
                >
                <span class="text-xs text-muted-foreground"
                  >{m.day_course_count({ count: dayEvents.length })}</span
                >
              </span>
              {#if isDayToday}
                <Badge tone="accent" class="flex-none">{m.preview_today()}</Badge>
              {/if}
            </button>

            <div class="flex flex-col gap-2 px-1">
              {#each dayEvents as event (event.id)}
                {@render courseRow(event, 'compact')}
              {:else}
                <p
                  class="rounded-md border border-dashed border-border-subtle bg-surface-sunken
                         p-3 text-sm text-muted-foreground"
                >{m.no_courses_day()}</p>
              {/each}
            </div>
          </section>
        {/each}
      </div>

      <!-- Desktop Week View (Multi-column grid 6 or 7 days) -->
      <!-- The column count is written per render, so the template reads it. -->
      <section
        class="hidden gap-2 overflow-x-auto md:grid
               grid-cols-[repeat(var(--week-cols,6),minmax(9rem,1fr))]
               min-[56rem]:grid-cols-[repeat(var(--week-cols,6),minmax(0,1fr))]"
        style:--week-cols={visibleWeekDaysCount}
      >
        {#each weekDays as day (day.toISOString())}
          {@const dayEvents = eventsForDay(day)}
          {@const isDayToday = isSameDay(day, now)}
          {@const isDayActive = isSameDay(day, activeDate)}

          <div
            class={cn(
              panel,
              'flex min-h-72 flex-col p-3',
              isDayToday && 'border-primary-deep'
            )}
          >
            <button
              type="button"
              class="mb-2 flex min-h-(--tap-min) w-full cursor-pointer items-center
                     justify-between border-b border-border-subtle bg-transparent px-0 pt-0
                     pb-2 text-inherit"
              onclick={() => selectDate(day)}
            >
              <span class="flex items-center gap-1">
                <strong class="text-sm font-extrabold uppercase text-foreground"
                  >{weekdayShortFormatter.format(day)}</strong
                >
                <span
                  class={cn(
                    'grid size-6 place-items-center rounded-full text-xs font-bold tabular-nums',
                    isDayToday
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-surface-sunken text-foreground'
                  )}>{day.getDate()}</span
                >
              </span>
              <span
                class="rounded-pill bg-surface-sunken px-2 py-[0.2rem] text-2xs font-bold
                       tabular-nums text-muted-foreground">{dayEvents.length}</span
              >
            </button>

            <div class="flex flex-1 flex-col gap-2">
              {#each dayEvents as event (event.id)}
                {@render courseRow(event, 'week')}
              {:else}
                <div class="flex h-16 items-center justify-center text-muted-foreground">
                  <span>-</span>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </section>

    <!-- SCOPE 3: 'month' (Mois - Grille calendrier interactif + détail du jour) -->
    {:else if currentScope === 'month'}
      <section
        class="grid grid-cols-1 gap-4 min-[54rem]:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]"
      >
        <div class={cn(panel, 'p-5')}>
          <!-- Weekday column titles -->
          <div
            class={cn(
              'mb-3 grid grid-cols-7 text-center text-muted-foreground',
              uppercaseTiny,
              'tracking-normal'
            )}
          >
            {#each monthHeaderDays as day (day.getTime())}
              <span>{weekdayShortFormatter.format(day)}</span>
            {/each}
          </div>

          <!-- 42-day Month Grid -->
          <div class="grid grid-cols-7 gap-1">
            {#each monthGridDays as day (day.toISOString())}
              {@const dayEvents = eventsForDay(day)}
              {@const isDayInMonth = isSameMonth(day, anchorDate)}
              {@const isDayToday = isSameDay(day, now)}
              {@const isDaySelected = isSameDay(day, activeDate)}

              <button
                type="button"
                class={cn(
                  'month-cell-btn flex min-h-(--tap-min) cursor-pointer flex-col items-center',
                  'justify-between gap-1 rounded-md border px-1 py-2 transition-control',
                  'active:scale-(--press-scale) hover:border-primary-deep hover:bg-muted',
                  !isDayInMonth && 'opacity-55',
                  isDaySelected
                    ? 'border-primary-deep bg-muted'
                    : cn('bg-surface-sunken', isDayToday ? 'border-primary-deep' : 'border-border-subtle')
                )}
                aria-pressed={isDaySelected}
                onclick={() => selectDate(day)}
              >
                <span
                  class={cn(
                    'text-sm tabular-nums',
                    isDayToday || isDaySelected
                      ? 'font-extrabold text-primary-deep'
                      : 'font-bold text-foreground'
                  )}>{day.getDate()}</span
                >

                <span class="flex min-h-2 items-center justify-center gap-[0.2rem]">
                  {#if dayEvents.length > 0}
                    {#if dayEvents.length <= 3}
                      {#each dayEvents.slice(0, 3) as dayEvent (dayEvent.id)}
                        <span class={dot}></span>
                      {/each}
                    {:else}
                      <span class={dot}></span>
                      <span class="text-2xs leading-none font-extrabold text-primary-deep">+{dayEvents.length}</span>
                    {/if}
                  {/if}
                </span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Selected Day Detailed Courses List -->
        <div class={cn(panel, 'flex flex-col gap-3 p-4 min-[54rem]:p-5')}>
          <header class={cn(dayHeaderRow, 'border-b border-border-subtle pb-3')}>
            <div class="min-w-0">
              <h3 class="mb-[0.15rem] text-lg font-extrabold wrap-anywhere"
                >{capitalizeFirst(dayFormatter.format(activeDate))}</h3
              >
              <p class="text-xs text-muted-foreground">
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

          <div
            class="flex flex-col gap-2.5 min-[54rem]:max-h-[28rem] min-[54rem]:overflow-y-auto"
          >
            {#if activeDateEvents.length > 0}
              {#each activeDateEvents as event (event.id)}
                {@render courseRow(event, 'detailed')}
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
  /* The separator between two adjacent days. There is no sibling-combinator
     variant, and the line belongs to the pair rather than to either day. */
  .mobile-week-day + .mobile-week-day {
    padding-top: var(--space-1);
  }

  .mobile-week-day + .mobile-week-day::before {
    height: 1px;
    margin-bottom: var(--space-2);
    background: var(--border-subtle);
    content: '';
  }
</style>
