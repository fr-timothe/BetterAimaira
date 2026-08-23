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
</script>

<PageShell>
  <header class="home-head">
    <div class="head-text">
      <h1 class="head-greeting">{copy.greeting} {displayName}</h1>
    </div>

    <div class="head-actions desktop-only">
      <Button variant="primary" size="sm" loading={refreshing} onclick={() => void onRefresh()}>
        {#if !refreshing}<RefreshCw size={15} aria-hidden="true" />{/if}
        {copy.refresh}
      </Button>
    </div>
  </header>

  <div class="home-primary">
    <!-- One surface owns the schedule state, so loading never renders as "no class". -->
    {#if isScheduleLoading}
      <Card tone="ink" padding="md">
        <div class="hero hero-skeleton">
          <Skeleton shape="text" width="8rem" />
          <Skeleton shape="title" width="65%" />
          <Skeleton shape="text" width="45%" />
          <Skeleton shape="block" height="2.5rem" />
        </div>
      </Card>

    {:else if isSessionExpired}
      <StateCard
        kind="expired"
        icon={AlertCircle}
        title={copy.planningErrorHeading}
        description={copy.planningSessionExpired}
      />
    {:else if isScheduleError}
      <StateCard
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

      <Card tone="ink" padding="md">
        <section class="hero">
        <div class="hero-badges">
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
            <span class="hero-chip"><MapPin size={13} aria-hidden="true" /> {room}</span>
          {/if}
        </div>

        <h2 class="hero-title">{eventTitle(featuredEvent)}</h2>

        <div class="hero-meta">
          {#if teacher}
            <span><User size={13} aria-hidden="true" /> {teacher}</span>
          {/if}
          <span class="hero-time">
            <Clock size={13} aria-hidden="true" /> {startTime} – {endTime} ({durationText})
          </span>
        </div>

        {#if featuredEvent.externalComment}
          <p class="hero-note">
            <Info size={13} aria-hidden="true" />
            <span>{featuredEvent.externalComment}</span>
          </p>
        {/if}

        {#if featuredStatus === 'now'}
          <div class="hero-progress">
            <span class="progress-time">{startTime}</span>
            <div class="progress-track">
              <div class="progress-fill" style="width: {courseProgress}%"></div>
            </div>
            <span class="progress-time">{endTime}</span>
            <span class="hero-countdown">{endsInLabel}</span>
          </div>
        {:else}
          <p class="hero-countdown hero-countdown-block">{startsInLabel}</p>
        {/if}
        </section>
      </Card>
    {:else}
      <StateCard
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
    <div class="stat-row">
      {#each [0, 1, 2] as placeholder (placeholder)}
        <Card>
          <Skeleton shape="text" width="6rem" />
          <div class="stat-value"><Skeleton shape="title" width="4rem" /></div>
          <Skeleton shape="text" width="7rem" />
        </Card>
      {/each}
    </div>
  {/if}

  {#if !isScheduleLoading && !isScheduleError}
    <div class="stat-row">
      <Card>
        <span class="stat-label">{copy.gradeAverage}</span>
        <p class="stat-value">
          {formatAverage(overallAverage)}
          {#if overallAverage !== null}<span class="stat-unit">/20</span>{/if}
        </p>
        <small class="stat-sub">{gradesRecordedLabel}</small>
      </Card>

      <Card>
        <span class="stat-label">{copy.coursesRemaining}</span>
        <p class="stat-value">
          {remainingCoursesCount}
          <span class="stat-unit">{dayCourseCountLabel}</span>
        </p>
        <small class="stat-sub">{endsAtLabel}</small>
      </Card>

      <Card>
        <span class="stat-label">{copy.dayVolume}</span>
        <p class="stat-value">{totalClassTimeLabel}</p>
        <small class="stat-sub">{copy.dayVolumeDescription}</small>
      </Card>
    </div>
  {/if}

  <div class="home-columns" class:single={!showDaySection}>
    {#if showDaySection}
      <section class="home-day">
        <SectionHeader title={copy.daySchedule} icon={CalendarDays}>
          {#snippet actions()}
            {#if isScheduleLoading}
              <span class="day-count">{copy.planningLoading}</span>
            {:else}
              <span class="day-count">{dayCourseCountLabel} • {totalClassTimeLabel}</span>
            {/if}
          {/snippet}
        </SectionHeader>

        <div class="day-list">
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
                <div
                  class="course-row"
                  class:live={isLive}
                  class:selected={selectedCourseId === event.id}
                >
                  <button
                    type="button"
                    class="course-open"
                    aria-pressed={selectedCourseId === event.id}
                    onclick={() => (selectedCourseId = event.id)}
                  >
                    <span class="course-time">
                      <span class="time-start">{startTime}</span>
                      <span class="time-span">{duration}</span>
                      <span class="time-end">{endTime}</span>
                    </span>

                    <span class="course-body">
                      <span class="course-head">
                        <KindBadge {event} />
                        <span class="course-name">{eventTitle(event)}</span>
                        {#if isLive}
                          <Badge tone="accent">{copy.statusLive}</Badge>
                        {/if}
                      </span>

                      {#if room || teacher}
                        <span class="course-meta">
                          {#if room}
                            <span><MapPin size={12} aria-hidden="true" /> {room}</span>
                          {/if}
                          {#if teacher}
                            <span><User size={12} aria-hidden="true" /> {teacher}</span>
                          {/if}
                        </span>
                      {/if}

                      {#if event.externalComment}
                        <span class="course-note">
                          <Info size={12} aria-hidden="true" />
                          <span>{event.externalComment}</span>
                        </span>
                      {/if}
                    </span>
                  </button>

                  <div class="course-action">
                    {#if event.tempoUrl}
                      <IconButton
                        label={copy.openTempo}
                        variant="ghost"
                        onclick={() => void onOpenTempo(event)}
                      >
                        <ExternalLink size={15} aria-hidden="true" />
                      </IconButton>
                    {:else}
                      <ChevronRight size={17} class="course-chevron" aria-hidden="true" />
                    {/if}
                  </div>
                </div>
              </Card>
            {/each}
          {:else}
            <div class="day-empty">
              <Calendar size={18} aria-hidden="true" />
              <span>{copy.noClassTodayDescription}</span>
            </div>
          {/if}
        </div>
      </section>
    {/if}

    <aside class="home-sidebar">
      {#if gradesLoading}
        <Card>
          <div class="widget" role="status" aria-live="polite" aria-busy="true" aria-label={copy.planningLoading}>
            <SectionHeader title={copy.recentGrades} icon={Award} level={3} />
            <div class="grade-list">
              {#each [0, 1] as placeholder (placeholder)}
                <div class="grade-row-skeleton">
                  <div class="grade-copy-skeleton">
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
          <div class="widget">
            <SectionHeader title={copy.recentGrades} icon={Award} level={3}>
              {#snippet actions()}
                <Button variant="ghost" size="sm" onclick={onOpenGrades}>
                  {copy.allGrades}
                  <ArrowUpRight size={13} aria-hidden="true" />
                </Button>
              {/snippet}
            </SectionHeader>

            <div class="grade-list">
              {#each recentGrades as grade (grade.id)}
                <Card tone="sunken" padding="sm">
                  <div class="grade-row">
                    <span class="grade-identity">
                      <span class="grade-subject">{grade.subject}</span>
                      <span class="grade-label">
                        {grade.label} • {coefficientLabel(grade.coefficient || '1.0')}
                      </span>
                    </span>

                    <span class="grade-score">
                      <span class="grade-value">
                        {grade.score}<span class="grade-scale">/{grade.scale || '20'}</span>
                      </span>
                      {#if grade.average}
                        <span class="grade-class-average">{classAverageLabel(grade.average)}</span>
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
  /* Header */
  .home-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .head-text {
    display: flex;
    min-width: 0;
    flex-direction: column;
  }

  .head-greeting {
    margin: 0;
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    line-height: 1.2;
    letter-spacing: -0.02em;
    overflow-wrap: anywhere;
  }

  .head-actions {
    display: flex;
    align-items: center;
    flex: 0 0 auto;
    gap: var(--space-2);
  }

  @media (min-width: 48rem) {
    .head-greeting {
      font-size: var(--text-2xl);
    }
  }

  /* Hero — the one ink surface, so its own text tones derive from the ink pair. */
  .home-primary {
    min-height: 16rem;
  }

  .home-primary :global(.ui-card),
  .home-primary :global(.ui-state-card) {
    min-height: 16rem;
    height: 100%;
    box-sizing: border-box;
  }

  .hero {
    display: flex;
    min-width: 0;
    min-height: calc(16rem - (2 * var(--space-4)));
    flex-direction: column;
    justify-content: center;
    gap: var(--space-2-5);
  }

  .hero-skeleton {
    gap: var(--space-3);
  }

  .hero-badges {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .hero-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0.2rem var(--space-2);
    color: var(--secondary-foreground);
    background: color-mix(in oklch, var(--secondary-foreground) 16%, transparent);
    border-radius: var(--radius-xs);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    white-space: nowrap;
  }

  .hero-title {
    margin: 0;
    font-size: var(--text-2xl);
    font-weight: var(--weight-heavy);
    line-height: 1.2;
    letter-spacing: -0.02em;
    overflow-wrap: anywhere;
  }

  .hero-meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-3);
    color: color-mix(in oklch, var(--secondary-foreground) 84%, var(--secondary));
    font-size: var(--text-sm);
  }

  .hero-meta span {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .hero-time {
    font-variant-numeric: tabular-nums;
  }

  .hero-note {
    display: inline-flex;
    align-items: center;
    width: fit-content;
    gap: var(--space-1);
    margin: 0;
    padding: 0.2rem var(--space-2);
    background: color-mix(in oklch, var(--secondary-foreground) 14%, transparent);
    border-radius: var(--radius-xs);
    font-size: var(--text-xs);
  }

  .hero-progress {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin-top: var(--space-1);
  }

  .progress-time {
    color: color-mix(in oklch, var(--secondary-foreground) 80%, var(--secondary));
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    font-variant-numeric: tabular-nums;
  }

  .progress-track {
    min-width: 6rem;
    height: 0.375rem;
    flex: 1;
    background: color-mix(in oklch, var(--secondary-foreground) 24%, transparent);
    border-radius: var(--radius-pill);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--primary);
    border-radius: var(--radius-pill);
    transition: width var(--duration-slow) var(--ease-out);
  }

  .hero-countdown {
    padding: 0.2rem var(--space-2);
    color: var(--secondary-foreground);
    background: color-mix(in oklch, var(--secondary-foreground) 18%, transparent);
    border-radius: var(--radius-xs);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .hero-countdown-block {
    width: fit-content;
    margin: var(--space-1) 0 0;
  }

  /* Day stats */
  .stat-row {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2);
  }

  .stat-row :global(.ui-card) {
    padding: var(--space-3) var(--space-2-5);
  }

  .stat-label {
    display: block;
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stat-value {
    display: flex;
    align-items: baseline;
    gap: 0.15rem;
    margin: 0.2rem 0 0;
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
  }

  .stat-unit {
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-weight: var(--weight-semibold);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stat-sub {
    display: block;
    margin-top: 0.2rem;
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-variant-numeric: tabular-nums;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (min-width: 48rem) {
    .stat-row {
      grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
      gap: var(--space-3);
    }

    .stat-row :global(.ui-card) {
      padding: var(--space-4);
    }

    .stat-value {
      font-size: var(--text-2xl);
      margin: var(--space-1) 0 0;
    }

    .stat-unit {
      font-size: var(--text-sm);
    }

    .stat-sub {
      font-size: var(--text-xs);
      margin-top: var(--space-1);
    }
  }

  /* Columns */
  .home-columns {
    display: grid;
    grid-template-columns: 1fr;
    align-items: start;
    gap: var(--space-5);
  }

  @media (min-width: 62rem) {
    .home-columns {
      grid-template-columns: 1.7fr 1fr;
    }

    .home-columns.single {
      grid-template-columns: 1fr;
    }
  }

  .home-day {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: var(--space-3);
  }

  .day-count {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    font-variant-numeric: tabular-nums;
  }

  .day-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .day-empty {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: 5.5rem;
    padding: var(--space-3) var(--space-4);
    color: var(--muted-foreground);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xl);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .day-empty :global(svg) {
    flex: 0 0 auto;
    color: var(--primary-deep);
  }

  .grade-row-skeleton {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    min-height: 4rem;
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-lg);
  }

  .grade-copy-skeleton {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: var(--space-2);
  }

  /* Course row — the whole information block is one button, and the Tempo action
     sits beside it so no control is nested inside another. */
  .course-row {
    display: flex;
    align-items: center;
    min-width: 0;
    gap: var(--space-2);
    border-radius: var(--radius-xl);
  }

  .course-row.live {
    background: var(--muted);
  }

  .course-row.selected {
    box-shadow: inset 0 0 0 2px var(--primary-deep);
  }

  .course-open {
    display: grid;
    grid-template-columns: 4.25rem 1fr;
    align-items: center;
    min-width: 0;
    min-height: var(--tap-min);
    flex: 1;
    gap: var(--space-3);
    padding: var(--space-3);
    color: inherit;
    background: transparent;
    border: 0;
    border-radius: var(--radius-xl);
    text-align: left;
  }

  .course-time {
    display: flex;
    min-width: 0;
    flex-direction: column;
    align-items: flex-start;
  }

  .time-start {
    font-size: var(--text-lg);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
  }

  .time-span {
    color: var(--primary-deep);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    font-variant-numeric: tabular-nums;
  }

  .time-end {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }

  .course-body {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: var(--space-1);
  }

  .course-head {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    min-width: 0;
    gap: var(--space-2);
  }

  .course-name {
    font-size: var(--text-md);
    font-weight: var(--weight-bold);
    overflow-wrap: anywhere;
  }

  .course-meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-3);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .course-meta span {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .course-note {
    display: inline-flex;
    align-items: center;
    width: fit-content;
    gap: var(--space-1);
    padding: 0.15rem var(--space-2);
    background: var(--surface-sunken);
    border-radius: var(--radius-xs);
    font-size: var(--text-xs);
  }

  .course-action {
    display: flex;
    align-items: center;
    flex: 0 0 auto;
    padding-right: var(--space-2);
  }

  :global(.course-chevron) {
    color: var(--muted-foreground);
  }

  /* Sidebar */
  .home-sidebar {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: var(--space-4);
  }

  .widget {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .grade-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .grade-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .grade-identity {
    display: flex;
    min-width: 0;
    flex-direction: column;
  }

  .grade-subject {
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    overflow-wrap: anywhere;
  }

  .grade-label {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }

  .grade-score {
    display: flex;
    flex: 0 0 auto;
    flex-direction: column;
    align-items: flex-end;
  }

  .grade-value {
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    font-variant-numeric: tabular-nums;
  }

  .grade-scale {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
  }

  .grade-class-average {
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-weight: var(--weight-semibold);
    font-variant-numeric: tabular-nums;
  }
</style>
