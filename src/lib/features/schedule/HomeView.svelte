<script lang="ts">
  import {
    AlertCircle,
    ArrowUpRight,
    Award,
    Calendar,
    CalendarDays,
    CheckCircle2,
    ChevronRight,
    Clock,
    Clock3,
    ExternalLink,
    Info,
    MapPin,
    RefreshCw,
    User,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import PageShell from '$lib/components/ui/PageShell.svelte';
  import SectionHeader from '$lib/components/ui/SectionHeader.svelte';
  import Skeleton from '$lib/components/ui/Skeleton.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import { isSameDay } from './date-utils';
  import {
    eventDurationMinutes,
    eventEnd,
    eventStart,
    eventTitle,
    formatDuration,
    formatDurationRange,
    getEventStatus,
    parseRoomAndTeacher,
  } from './course-utils';
  import { getDisplayName } from './portal-utils';
  import { cn } from '$lib/utils';
  import type { CalendarEvent, Grade, ScheduleState } from './types';

  type Props = {
    username: string;
    events: CalendarEvent[];
    grades: Grade[];
    gradesLoading: boolean;
    now: Date;
    locale: Locale;
    scheduleState?: ScheduleState;
    onOpenSchedule: (date?: Date) => void;
    onOpenGrades: () => void;
    onOpenTempo: (event: CalendarEvent) => Promise<void>;
    onRefresh: () => Promise<void>;
    refreshing: boolean;
    /** `null` means nothing has been fetched yet, which is not the same as "fetched at epoch". */
    fetchedAt: number | null;
  };

  let {
    username,
    events,
    grades,
    gradesLoading,
    now,
    locale,
    scheduleState,
    onOpenSchedule,
    onOpenGrades,
    onOpenTempo,
    onRefresh,
    refreshing,
    fetchedAt,
  }: Props = $props();

  let selectedCourseId = $state<string | null>(null);

  const timeFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      hour: '2-digit',
      minute: '2-digit',
    })
  );

  const displayName = $derived(getDisplayName(username, locale));

  const isScheduleLoading = $derived(scheduleState?.kind === 'loading');
  const isScheduleError = $derived(scheduleState?.kind === 'error');
  const isSessionExpired = $derived(
    scheduleState?.kind === 'error' && scheduleState.code === 'session_expired'
  );

  const todayEvents = $derived.by(() => {
    return events
      .filter((event) => isSameDay(eventStart(event), now))
      .sort((a, b) => eventStart(a).getTime() - eventStart(b).getTime());
  });

  const currentEvent = $derived.by(() => {
    const nowMs = now.getTime();
    return (
      todayEvents.find((event) => {
        const start = eventStart(event).getTime();
        const end = eventEnd(event).getTime();
        return start <= nowMs && nowMs < end;
      }) ?? null
    );
  });

  const upcomingEvents = $derived.by(() => {
    const nowMs = now.getTime();
    return todayEvents.filter((event) => eventStart(event).getTime() > nowMs);
  });

  const featuredEvent = $derived.by(() => currentEvent ?? upcomingEvents[0] ?? null);

  const featuredStatus = $derived.by(() => {
    if (!featuredEvent) return 'none';
    if (currentEvent && featuredEvent.id === currentEvent.id) return 'now';
    return 'upcoming';
  });

  const courseProgress = $derived.by(() => {
    if (!currentEvent) return 0;
    const start = eventStart(currentEvent).getTime();
    const end = eventEnd(currentEvent).getTime();
    if (end <= start) return 0;
    return Math.min(100, Math.max(0, ((now.getTime() - start) / (end - start)) * 100));
  });

  const remainingMinutesInCurrent = $derived.by(() => {
    if (!currentEvent) return 0;
    const end = eventEnd(currentEvent).getTime();
    return Math.max(0, Math.round((end - now.getTime()) / 60_000));
  });

  const minutesUntilNext = $derived.by(() => {
    if (!featuredEvent || currentEvent) return 0;
    const start = eventStart(featuredEvent).getTime();
    return Math.max(0, Math.round((start - now.getTime()) / 60_000));
  });

  const remainingCoursesCount = $derived.by(() => {
    const nowMs = now.getTime();
    return todayEvents.filter((event) => eventEnd(event).getTime() > nowMs).length;
  });

  const totalClassTimeMinutes = $derived.by(() => {
    return todayEvents.reduce((total, event) => total + eventDurationMinutes(event), 0);
  });

  const endOfDayTime = $derived.by(() => {
    if (todayEvents.length === 0) return '--:--';
    return timeFormatter.format(eventEnd(todayEvents[todayEvents.length - 1]));
  });

  const recentGrades = $derived(grades.slice(-2).reverse());

  const overallAverage = $derived.by(() => {
    if (grades.length === 0) return null;
    let sum = 0;
    let count = 0;
    for (const g of grades) {
      const clean = g.score.replace(',', '.').replace(/[^0-9.]/g, '');
      const num = parseFloat(clean);
      const scaleClean = g.scale ? g.scale.replace(',', '.').replace(/[^0-9.]/g, '') : '20';
      const scale = parseFloat(scaleClean);
      if (!Number.isNaN(num) && !Number.isNaN(scale) && scale > 0) {
        sum += (num / scale) * 20;
        count++;
      }
    }
    return count > 0 ? sum / count : null;
  });

  function formatAverage(avg: number | null): string {
    if (avg === null) return '--';
    return avg.toLocaleString(locale, { minimumFractionDigits: 1, maximumFractionDigits: 1 });
  }

  const totalClassTimeLabel = $derived(formatDuration(totalClassTimeMinutes, locale));

  /* Paraglide message functions are not reactive on their own, so every read of
     the catalogue is wrapped in a derived that touches `locale` first. */
  const copy = $derived.by(() => {
    locale;
    return {
      greeting: m.greetings(),
      refresh: m.sync_refresh(),
      currentCourse: m.home_current_course(),
      nextCourse: m.home_next_course(),
      gradeAverage: m.grade_average(),
      gradeCount: m.grade_count(),
      coursesRemaining: m.courses_remaining(),
      dayVolume: m.home_day_volume(),
      dayVolumeDescription: m.home_day_volume_description(),
      dayFinished: m.home_day_finished(),
      noClassToday: m.home_no_class_today(),
      noClassTodayDescription: m.home_no_class_today_description(),
      daySchedule: m.home_day_schedule(),
      planningLoading: m.planning_loading(),
      planningErrorHeading: m.planning_error_heading(),
      planningUnavailable: m.planning_unavailable(),
      planningSessionExpired: m.planning_session_expired(),
      retry: m.planning_retry(),
      viewCalendar: m.home_view_calendar(),
      statusLive: m.schedule_status_live(),
      openTempo: m.open_tempo(),
      recentGrades: m.home_recent_grades(),
      allGrades: m.view_all_grades(),
      noGrades: m.home_no_grades(),
    };
  });

  const endsInLabel = $derived.by(() => {
    locale;
    return m.home_ends_in({ duration: formatDuration(remainingMinutesInCurrent, locale) });
  });

  const startsInLabel = $derived.by(() => {
    locale;
    return m.home_starts_in({ duration: formatDuration(minutesUntilNext, locale) });
  });

  const gradesRecordedLabel = $derived.by(() => {
    locale;
    return m.home_grades_recorded({ count: grades.length });
  });

  const endsAtLabel = $derived.by(() => {
    locale;
    return m.home_ends_at({ time: endOfDayTime });
  });

  const dayCourseCountLabel = $derived.by(() => {
    locale;
    return m.day_course_count({ count: todayEvents.length });
  });

  const dayFinishedDescription = $derived.by(() => {
    locale;
    return m.home_day_finished_description({ count: todayEvents.length });
  });

  /* Per-row labels need the parameter at render time, so the derived hands back a
     formatter that is rebuilt whenever the locale changes. */
  const classAverageLabel = $derived.by(() => {
    locale;
    return (value: string) => m.home_class_average({ value });
  });

  const coefficientLabel = $derived.by(() => {
    locale;
    return (value: string) => m.grade_alert_coefficient({ value });
  });

  const showDaySection = $derived(!isScheduleError);
  const hero = 'flex min-w-0 flex-col justify-center min-h-[calc(16rem-(2*var(--space-4)))]';
  const heroPill =
    'inline-flex items-center gap-1 rounded-xs px-2 py-[0.2rem] text-2xs font-bold whitespace-nowrap';
  const inkPanel = 'min-h-64 h-full';
  const statCard = 'px-2.5 py-3 md:p-4';
  const statLabel =
    'block truncate text-2xs font-bold tracking-[0.04em] uppercase text-muted-foreground';
  const statValue =
    'mt-[0.2rem] flex items-baseline gap-[0.15rem] text-xl font-extrabold tabular-nums' +
    ' md:mt-1 md:text-2xl';
  const statUnit = 'truncate text-2xs font-semibold text-muted-foreground md:text-sm';
  const statSub =
    'mt-[0.2rem] block truncate text-2xs tabular-nums text-muted-foreground md:mt-1 md:text-xs';
  const dayCount = 'text-xs font-semibold tabular-nums text-muted-foreground';
  const widget = 'flex flex-col gap-3';
  const statRow =
    'grid grid-cols-3 gap-2 md:grid-cols-[repeat(auto-fit,minmax(11rem,1fr))] md:gap-3';
  const progressTime =
    'text-xs font-semibold tabular-nums text-ink-meta-dim';
  const countdown =
    'rounded-xs bg-ink-badge px-2 py-[0.2rem] text-xs font-bold tabular-nums whitespace-nowrap text-secondary-foreground';
  const metaRow = 'flex flex-wrap items-center gap-3 [&>span]:inline-flex [&>span]:items-center [&>span]:gap-1';
</script>

<PageShell>
  <header
    class="flex flex-wrap items-center justify-between gap-3 border-b border-border-subtle pb-3"
  >
    <div class="flex min-w-0 flex-col">
      <h1
        class="text-xl leading-[1.2] font-extrabold tracking-[-0.02em] wrap-anywhere md:text-2xl"
      >{copy.greeting} {displayName}</h1>
    </div>

    <!-- `desktop-only` owns the display here; a display utility would lose to
         its !important rules without a word. -->
    <div class="desktop-only flex-none items-center gap-2">
      <Button variant="primary" size="sm" loading={refreshing} onclick={() => void onRefresh()}>
        {#if !refreshing}<RefreshCw size={15} aria-hidden="true" />{/if}
        {copy.refresh}
      </Button>
    </div>
  </header>

  <div class="min-h-64">
    <!-- One surface owns the schedule state, so loading never renders as "no class".
         Every branch carries `inkPanel` so the panel never changes height between
         a fetch, an error and the course itself. -->
    {#if isScheduleLoading}
      <Card tone="ink" padding="md" class={inkPanel}>
        <div class={cn(hero, 'gap-3')}>
          <Skeleton shape="text" width="8rem" />
          <Skeleton shape="title" width="65%" />
          <Skeleton shape="text" width="45%" />
          <Skeleton shape="block" height="2.5rem" />
        </div>
      </Card>

    {:else if isSessionExpired}
      <StateCard
        class={inkPanel}
        kind="expired"
        icon={AlertCircle}
        title={copy.planningErrorHeading}
        description={copy.planningSessionExpired}
      />
    {:else if isScheduleError}
      <StateCard
        class={inkPanel}
        kind="error"
        icon={AlertCircle}
        title={copy.planningErrorHeading}
        description={copy.planningUnavailable}
        actionLabel={copy.retry}
        onAction={() => void onRefresh()}
      />
    {:else if featuredEvent}
      {@const { room, teacher } = parseRoomAndTeacher(featuredEvent)}
      {@const durationText = formatDurationRange(featuredEvent.startsAt, featuredEvent.endsAt)}
      {@const startTime = timeFormatter.format(eventStart(featuredEvent))}
      {@const endTime = timeFormatter.format(eventEnd(featuredEvent))}

      <Card tone="ink" padding="md" class={inkPanel}>
        <section class={cn(hero, 'gap-2.5')}>
        <div class="flex flex-wrap items-center gap-2">
          {#if featuredStatus === 'now'}
            <Badge tone="live" dot>{copy.currentCourse}</Badge>
          {:else}
            <Badge tone="accent">
              <Clock3 size={12} aria-hidden="true" />
              {copy.nextCourse}
            </Badge>
          {/if}

          <KindBadge event={featuredEvent} />

          {#if room}
            <span class={cn(heroPill, 'bg-ink-chip text-secondary-foreground')}
              ><MapPin size={13} aria-hidden="true" /> {room}</span
            >
          {/if}
        </div>

        <h2 class="text-2xl leading-[1.2] font-extrabold tracking-[-0.02em] wrap-anywhere"
          >{eventTitle(featuredEvent)}</h2
        >

        <div class={cn(metaRow, 'text-sm text-ink-meta')}>
          {#if teacher}
            <span><User size={13} aria-hidden="true" /> {teacher}</span>
          {/if}
          <span class="tabular-nums">
            <Clock size={13} aria-hidden="true" /> {startTime} – {endTime} ({durationText})
          </span>
        </div>

        {#if featuredEvent.externalComment}
          <p class={cn(heroPill, 'w-fit bg-ink-note text-xs font-normal')}>
            <Info size={13} aria-hidden="true" />
            <span>{featuredEvent.externalComment}</span>
          </p>
        {/if}

        {#if featuredStatus === 'now'}
          <div class="mt-1 flex flex-wrap items-center gap-3">
            <span class={progressTime}>{startTime}</span>
            <div class="h-1.5 min-w-24 flex-1 overflow-hidden rounded-pill bg-ink-track">
              <div
                class="h-full rounded-pill bg-primary transition-[width] duration-slow ease-out"
                style="width: {courseProgress}%"
              ></div>
            </div>
            <span class={progressTime}>{endTime}</span>
            <span class={countdown}>{endsInLabel}</span>
          </div>
        {:else}
          <p class={cn(countdown, 'mt-1 w-fit')}>{startsInLabel}</p>
        {/if}
        </section>
      </Card>
    {:else}
      <StateCard
        class={inkPanel}
        kind="empty"
        icon={todayEvents.length > 0 ? CheckCircle2 : Calendar}
        title={todayEvents.length > 0 ? copy.dayFinished : copy.noClassToday}
        description={todayEvents.length > 0 ? dayFinishedDescription : copy.noClassTodayDescription}
        actionLabel={copy.viewCalendar}
        onAction={() => onOpenSchedule()}
      />
    {/if}
  </div>

  {#if isScheduleLoading}
    <div class={statRow}>
      {#each [0, 1, 2] as placeholder (placeholder)}
        <Card class={statCard}>
          <Skeleton shape="text" width="6rem" />
          <div class={statValue}><Skeleton shape="title" width="4rem" /></div>
          <Skeleton shape="text" width="7rem" />
        </Card>
      {/each}
    </div>
  {/if}

  {#if !isScheduleLoading && !isScheduleError}
    <div class={statRow}>
      <Card class={statCard}>
        <span class={statLabel}>{copy.gradeAverage}</span>
        <p class={statValue}>
          {formatAverage(overallAverage)}
          {#if overallAverage !== null}<span class={statUnit}>/20</span>{/if}
        </p>
        <small class={statSub}>{gradesRecordedLabel}</small>
      </Card>

      <Card class={statCard}>
        <span class={statLabel}>{copy.coursesRemaining}</span>
        <p class={statValue}>
          {remainingCoursesCount}
          <span class={statUnit}>{dayCourseCountLabel}</span>
        </p>
        <small class={statSub}>{endsAtLabel}</small>
      </Card>

      <Card class={statCard}>
        <span class={statLabel}>{copy.dayVolume}</span>
        <p class={statValue}>{totalClassTimeLabel}</p>
        <small class={statSub}>{copy.dayVolumeDescription}</small>
      </Card>
    </div>
  {/if}

  <div
    class={cn(
      'grid grid-cols-1 items-start gap-5',
      showDaySection && 'min-[62rem]:grid-cols-[1.7fr_1fr]'
    )}
  >
    {#if showDaySection}
      <section class="flex min-w-0 flex-col gap-3">
        <SectionHeader title={copy.daySchedule} icon={CalendarDays}>
          {#snippet actions()}
            {#if isScheduleLoading}
              <span class={dayCount}>{copy.planningLoading}</span>
            {:else}
              <span class={dayCount}>{dayCourseCountLabel} • {totalClassTimeLabel}</span>
            {/if}
          {/snippet}
        </SectionHeader>

        <div class="flex flex-col gap-3">
          {#if isScheduleLoading}
            {#each [0, 1, 2] as placeholder (placeholder)}
              <Skeleton shape="block" height="5.5rem" />
            {/each}
          {:else if todayEvents.length > 0}
            {#each todayEvents as event (event.id)}
              {@const status = getEventStatus(event, now)}
              {@const { room, teacher } = parseRoomAndTeacher(event)}
              {@const isLive = status === 'live'}
              {@const startTime = timeFormatter.format(eventStart(event))}
              {@const endTime = timeFormatter.format(eventEnd(event))}
              {@const duration = formatDuration(eventDurationMinutes(event), locale)}

              <Card interactive padding="none">
                <!-- The whole information block is one button, and the Tempo
                     action sits beside it so no control is nested inside another. -->
                <div
                  class="course-row"
                  class:live={isLive}
                  class:selected={selectedCourseId === event.id}
                >
                  <button
                    type="button"
                    class="grid min-h-(--tap-min) min-w-0 flex-1 grid-cols-[4.25rem_1fr]
                           items-center gap-3 rounded-xl bg-transparent p-3 text-left
                           text-inherit"
                    aria-pressed={selectedCourseId === event.id}
                    onclick={() => (selectedCourseId = event.id)}
                  >
                    <span class="flex min-w-0 flex-col items-start">
                      <span class="text-lg font-extrabold tabular-nums">{startTime}</span>
                      <span class="text-2xs font-bold tabular-nums text-primary-deep"
                        >{duration}</span
                      >
                      <span class="text-xs tabular-nums text-muted-foreground">{endTime}</span>
                    </span>

                    <span class="flex min-w-0 flex-col gap-1">
                      <span class="flex min-w-0 flex-wrap items-center gap-2">
                        <KindBadge {event} />
                        <span class="text-md font-bold wrap-anywhere">{eventTitle(event)}</span>
                        {#if isLive}
                          <Badge tone="accent">{copy.statusLive}</Badge>
                        {/if}
                      </span>

                      {#if room || teacher}
                        <span class={cn(metaRow, 'text-xs text-muted-foreground')}>
                          {#if room}
                            <span><MapPin size={12} aria-hidden="true" /> {room}</span>
                          {/if}
                          {#if teacher}
                            <span><User size={12} aria-hidden="true" /> {teacher}</span>
                          {/if}
                        </span>
                      {/if}

                      {#if event.externalComment}
                        <span
                          class="inline-flex w-fit items-center gap-1 rounded-xs bg-surface-sunken
                                 px-2 py-[0.15rem] text-xs"
                        >
                          <Info size={12} aria-hidden="true" />
                          <span>{event.externalComment}</span>
                        </span>
                      {/if}
                    </span>
                  </button>

                  <div class="flex flex-none items-center pr-2">
                    {#if event.tempoUrl}
                      <IconButton
                        label={copy.openTempo}
                        variant="ghost"
                        onclick={() => void onOpenTempo(event)}
                      >
                        <ExternalLink size={15} aria-hidden="true" />
                      </IconButton>
                    {:else}
                      <ChevronRight size={17} class="text-muted-foreground" aria-hidden="true" />
                    {/if}
                  </div>
                </div>
              </Card>
            {/each}
          {:else}
            <div
              class="flex min-h-22 items-center gap-3 rounded-xl border border-border-subtle
                     bg-card px-4 py-3 text-sm font-semibold text-muted-foreground
                     [&>svg]:flex-none [&>svg]:text-primary-deep"
            >
              <Calendar size={18} aria-hidden="true" />
              <span>{copy.noClassTodayDescription}</span>
            </div>
          {/if}
        </div>
      </section>
    {/if}

    <aside class="flex min-w-0 flex-col gap-4">
      {#if gradesLoading}
        <Card>
          <div class={widget} role="status" aria-live="polite" aria-busy="true" aria-label={copy.planningLoading}>
            <SectionHeader title={copy.recentGrades} icon={Award} level={3} />
            <div class="flex flex-col gap-2">
              {#each [0, 1] as placeholder (placeholder)}
                <div
                  class="flex min-h-16 items-center justify-between gap-3 rounded-lg
                         bg-surface-sunken p-3"
                >
                  <div class="flex min-w-0 flex-1 flex-col gap-2">
                    <Skeleton shape="title" width={placeholder === 0 ? '78%' : '64%'} />
                    <Skeleton shape="text" width="55%" />
                  </div>
                  <Skeleton shape="title" width="4rem" />
                </div>
              {/each}
            </div>
          </div>
        </Card>
      {:else if recentGrades.length === 0}
        <StateCard
          kind="empty"
          icon={Award}
          title={copy.noGrades}
          actionLabel={copy.allGrades}
          onAction={onOpenGrades}
        />
      {:else}
        <Card>
          <div class={widget}>
            <SectionHeader title={copy.recentGrades} icon={Award} level={3}>
              {#snippet actions()}
                <Button variant="ghost" size="sm" onclick={onOpenGrades}>
                  {copy.allGrades}
                  <ArrowUpRight size={13} aria-hidden="true" />
                </Button>
              {/snippet}
            </SectionHeader>

            <div class="flex flex-col gap-2">
              {#each recentGrades as grade (grade.id)}
                <Card tone="sunken" padding="sm">
                  <div class="flex items-center justify-between gap-3">
                    <span class="flex min-w-0 flex-col">
                      <span class="text-base font-bold wrap-anywhere">{grade.subject}</span>
                      <span class="text-xs text-muted-foreground">
                        {grade.label} • {coefficientLabel(grade.coefficient || '1.0')}
                      </span>
                    </span>

                    <span class="flex flex-none flex-col items-end">
                      <span class="text-xl font-extrabold tabular-nums">
                        {grade.score}<span class="text-xs font-bold text-muted-foreground"
                          >/{grade.scale || '20'}</span
                        >
                      </span>
                      {#if grade.average}
                        <span
                          class="text-2xs font-semibold tabular-nums text-muted-foreground"
                          >{classAverageLabel(grade.average)}</span
                        >
                      {/if}
                    </span>
                  </div>
                </Card>
              {/each}
            </div>
          </div>
        </Card>
      {/if}
    </aside>
  </div>
</PageShell>

<style>
  /* The row is one flex line whose parts are utilities; only the two states that
     paint the whole row stay here, because they key off a class this component
     sets on the row itself rather than on any single element. */
  .course-row {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: var(--space-2);
    border-radius: var(--radius-xl);
  }

  .course-row.live {
    background: var(--muted);
  }

  .course-row.selected {
    box-shadow: inset 0 0 0 2px var(--primary-deep);
  }
</style>
