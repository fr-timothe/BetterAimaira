<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE — structure A, "La pile".
   *
   * The phone drops the time grid. A day is a stack of full-width cards and
   * the empty time between two courses is drawn as a labelled rule instead of
   * being paid for at 4.5rem an hour. Week and month are the same stack under
   * sticky day headers, so no surface in this structure scrolls sideways and
   * the horizontal swipe means one thing everywhere: change period.
   */
  import { onMount, tick } from 'svelte';
  import { CalendarOff, ChevronRight } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import CourseDetailModal from '$lib/features/schedule/CourseDetailModal.svelte';
  import DatePickerSheet from '$lib/features/schedule/DatePickerSheet.svelte';
  import { monthGridDays } from '$lib/features/schedule/calendar-navigation.svelte';
  import {
    eventDurationMinutes,
    eventEnd,
    eventStart,
    eventTitle,
    formatDuration,
    getEventStatus,
    isCancelled,
    cancellationReason,
    parseRoomAndTeacher,
  } from '$lib/features/schedule/course-utils';
  import { capitalizeFirst, dayKey, isSameDay, isSameMonth } from '$lib/features/schedule/date-utils';
  import type { CalendarEvent } from '$lib/features/schedule/types';
  import { cn } from '$lib/utils';
  import ProtoShell from './ProtoShell.svelte';
  import ProtoTopBar from './ProtoTopBar.svelte';
  import { ProtoModel } from './proto-model.svelte';
  import { swipe } from './swipe';

  const model = new ProtoModel('day');
  const nav = model.navigation;

  onMount(model.startClock.bind(model));

  let detail = $state<CalendarEvent | null>(null);
  let pickerOpen = $state(false);
  let scroller = $state<HTMLDivElement | null>(null);

  /** JS cannot read the `prefers-reduced-motion` rule in app.css, so it asks again. */
  function protoScrollBehavior(): ScrollBehavior {
    if (typeof window === 'undefined') return 'auto';
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth';
  }

  /** A gap shorter than a break is not information, it is noise between cards. */
  const GAP_VISIBLE_FROM_MINUTES = 15;

  const days = $derived.by(() => {
    if (nav.scope === 'day') return [nav.activeDate];
    if (nav.scope === 'week') return model.weekDays;
    return monthGridDays(nav.anchorDate).filter((day) => isSameMonth(day, nav.anchorDate));
  });

  const offToday = $derived(!isSameDay(nav.activeDate, model.now));

  function dayTotals(dayEvents: CalendarEvent[]) {
    const busy = dayEvents.reduce((total, event) => total + eventDurationMinutes(event), 0);
    return { count: dayEvents.length, busy };
  }

  /** Minutes of nothing between the end of one course and the start of the next. */
  function gapBefore(dayEvents: CalendarEvent[], index: number): number {
    if (index === 0) return 0;
    return Math.max(
      0,
      Math.round(
        (eventStart(dayEvents[index]).getTime() - eventEnd(dayEvents[index - 1]).getTime()) / 60_000
      )
    );
  }

  /** The month grid jumps the same scroller rather than opening another view. */
  async function jumpToDay(date: Date) {
    nav.selectDate(date);
    await tick();
    scroller
      ?.querySelector<HTMLElement>(`[data-day="${dayKey(date)}"]`)
      ?.scrollIntoView({ behavior: protoScrollBehavior(), block: 'start' });
  }

</script>

<!-- One course. The left rail is the time axis this structure gives up as a
     dimension and keeps as a column, so a duration is still readable at a
     glance without costing the screen its height. -->
{#snippet courseCard(event: CalendarEvent)}
  {@const status = getEventStatus(event, model.now)}
  {@const details = parseRoomAndTeacher(event)}
  {@const cancelled = isCancelled(event)}
  {@const live = status === 'live' && !cancelled}

  <button
    type="button"
    class={cn(
      'flex w-full items-stretch gap-3 rounded-md border px-3 py-2.5 text-start',
      'transition-control active:scale-(--press-scale)',
      'fine-hover:border-primary-deep',
      live
        ? 'border-primary-deep bg-muted'
        : status === 'finished'
          ? 'border-border-subtle bg-surface-sunken'
          : 'border-border-subtle bg-card'
    )}
    onclick={() => (detail = event)}
  >
    <span class="flex w-[3.25rem] shrink-0 flex-col items-start gap-[0.1rem] pt-[0.1rem]">
      <span
        class={cn(
          'text-sm leading-none font-extrabold tabular-nums',
          status === 'finished' ? 'text-muted-foreground' : 'text-primary-deep'
        )}>{model.format.timeFormatter.format(eventStart(event))}</span
      >
      <span class="text-2xs leading-none font-semibold tabular-nums text-muted-foreground"
        >{formatDuration(eventDurationMinutes(event))}</span
      >
      <span class="text-2xs leading-none tabular-nums text-muted-foreground"
        >{model.format.timeFormatter.format(eventEnd(event))}</span
      >
    </span>

    <span class="flex min-w-0 flex-1 flex-col gap-1">
      <span class="flex min-w-0 items-start gap-2">
        <span
          class={cn(
            'min-w-0 flex-1 text-sm leading-[1.3] font-bold wrap-anywhere',
            cancelled && 'line-through decoration-danger-strong',
            status === 'finished' && !live ? 'text-muted-foreground' : 'text-foreground'
          )}>{eventTitle(event)}</span
        >
        <KindBadge {event} class="shrink-0" />
      </span>

      {#if details.room || details.teacher}
        <span class="min-w-0 truncate text-2xs font-semibold text-muted-foreground">
          {[details.room, details.teacher].filter(Boolean).join(' · ')}
        </span>
      {/if}

      {#if cancelled}
        <span class="flex items-center gap-1.5">
          <Badge tone="danger">Annulé</Badge>
          <span class="min-w-0 truncate text-2xs text-danger-strong"
            >{cancellationReason(event)}</span
          >
        </span>
      {:else if live}
        <span><Badge tone="live" dot>En cours</Badge></span>
      {/if}
    </span>

    <ChevronRight
      size={16}
      class="mt-1 shrink-0 self-start text-muted-foreground"
      aria-hidden="true"
    />
  </button>
{/snippet}

<!-- What the grid drew as empty height, drawn as a line instead. -->
{#snippet gapRule(minutes: number)}
  <div class="flex items-center gap-2 px-1 py-1" aria-hidden="true">
    <span class="h-px flex-1 bg-border-subtle"></span>
    <span class="text-2xs font-semibold tracking-[0.02em] text-muted-foreground"
      >{formatDuration(minutes)} libre</span
    >
    <span class="h-px flex-1 bg-border-subtle"></span>
  </div>
{/snippet}

<!-- Day scope ends at its day, which on a light Tuesday left two thirds of the
     screen empty. The week goes in that space as a footer inside the scroller,
     so it costs no permanent height — the day strip this structure removed,
     spent where there was nothing anyway. -->
{#snippet weekFooter()}
  <section class="mt-auto flex flex-col gap-2 pt-6">
    <h3 class="text-2xs font-extrabold tracking-[0.05em] uppercase text-muted-foreground">
      Le reste de la semaine
    </h3>
    <div class="grid grid-cols-6 gap-1.5">
      {#each model.weekDays as day (day.toISOString())}
        {@const count = model.eventsForDay(day).length}
        {@const isToday = isSameDay(day, model.now)}
        {@const isActive = isSameDay(day, nav.activeDate)}
        <button
          type="button"
          class={cn(
            'flex min-h-16 flex-col items-center justify-center gap-[0.15rem] rounded-md border',
            'transition-control active:scale-(--press-scale) fine-hover:border-primary-deep',
            isActive
              ? 'border-primary-deep bg-muted text-primary-deep'
              : isToday
                ? 'border-primary-deep bg-card text-foreground'
                : 'border-border-subtle bg-card text-muted-foreground'
          )}
          aria-pressed={isActive}
          onclick={() => nav.selectDate(day)}
        >
          <span class="text-2xs leading-none font-bold uppercase"
            >{model.format.weekdayShortFormatter.format(day).slice(0, 2)}</span
          >
          <span class="text-md leading-tight font-extrabold tabular-nums">{day.getDate()}</span>
          <span class="text-2xs leading-none font-semibold tabular-nums"
            >{count > 0 ? count : '—'}</span
          >
        </button>
      {/each}
    </div>
  </section>
{/snippet}

{#snippet dayStack(day: Date, withHeader: boolean)}
  {@const dayEvents = model.eventsForDay(day)}
  {@const totals = dayTotals(dayEvents)}
  {@const isToday = isSameDay(day, model.now)}

  <section class="flex flex-col" data-day={dayKey(day)}>
    {#if withHeader}
      <h3
        class={cn(
          'sticky top-0 z-sticky -mx-3 flex items-baseline gap-2 border-b border-border-subtle',
          'bg-background/92 px-3 py-1.5 backdrop-blur-[8px]'
        )}
      >
        <span
          class={cn(
            'text-sm leading-tight font-extrabold',
            isToday ? 'text-primary-deep' : 'text-foreground'
          )}
        >
          {capitalizeFirst(
            new Intl.DateTimeFormat('fr', { weekday: 'long', day: 'numeric' }).format(day)
          )}
        </span>
        {#if isToday}
          <Badge tone="accent">Aujourd’hui</Badge>
        {/if}
        <span class="ms-auto text-2xs font-semibold tabular-nums text-muted-foreground">
          {totals.count > 0 ? `${totals.count} · ${formatDuration(totals.busy)}` : '—'}
        </span>
      </h3>
    {:else}
      <p class="pb-1 text-xs font-semibold text-muted-foreground">
        {totals.count > 0
          ? `${totals.count} cours · ${formatDuration(totals.busy)} de présence`
          : 'Journée libre'}
      </p>
    {/if}

    {#if dayEvents.length === 0}
      {#if withHeader}
        <p class="px-1 py-3 text-xs text-muted-foreground">Aucun cours ce jour-là.</p>
      {:else}
        <div class="pt-2">
          <StateCard
            kind="empty"
            title="Aucun cours ce jour-là"
            description="Le portail n’a renvoyé aucun créneau pour cette date."
            icon={CalendarOff}
          />
        </div>
      {/if}
    {:else}
      <div class="flex flex-col gap-1.5 pt-1.5">
        {#each dayEvents as event, index (event.id)}
          {@const gap = gapBefore(dayEvents, index)}
          {#if gap >= GAP_VISIBLE_FROM_MINUTES}
            {@render gapRule(gap)}
          {/if}
          {@render courseCard(event)}
        {/each}
      </div>
    {/if}
  </section>
{/snippet}

<ProtoShell
  scope={nav.scope}
  {offToday}
  onScope={nav.setScope}
  onMove={nav.movePeriod}
  onToday={nav.goToToday}
>
  <ProtoTopBar
    label={model.format.periodLabel}
    fetchedAt={model.fetchedAt}
    onPick={() => (pickerOpen = true)}
  />

  <!-- Nothing in this structure scrolls sideways, so unlike the current view
       the swipe is unambiguous in all three scopes and never has to be
       disabled for one of them. -->
  <div class="flex min-h-0 flex-1 flex-col" use:swipe={{ onSwipe: nav.movePeriod }}>
    <!-- `min-h-full` on the inner column is what lets the week footer take
         `mt-auto`: on a light day it settles against the bottom bar instead of
         leaving 300px of nothing under it, and on a dense day it simply
         follows the last course. -->
    <div
      class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-3 pt-2 pb-4"
      bind:this={scroller}
    >
      <div class="flex min-h-full flex-col">
        {#if nav.scope === 'month'}
          <!-- The month is a density map over the same scroller, not a fourth
               view: a cell moves the scroll to that day's header. -->
          <div class="mb-3 grid grid-cols-7 gap-1">
            {#each monthGridDays(nav.anchorDate) as day (day.toISOString())}
              {@const count = model.eventsForDay(day).length}
              {@const outside = !isSameMonth(day, nav.anchorDate)}
              <button
                type="button"
                class={cn(
                  'flex min-h-11 flex-col items-center justify-center gap-[0.15rem] rounded-sm',
                  'border transition-control active:scale-(--press-scale)',
                  outside
                    ? 'border-transparent text-muted-foreground/60'
                    : isSameDay(day, model.now)
                      ? 'border-primary-deep bg-muted text-primary-deep'
                      : 'border-border-subtle bg-card text-foreground'
                )}
                onclick={() => jumpToDay(day)}
              >
                <span class="text-2xs leading-none font-bold tabular-nums">{day.getDate()}</span>
                <span
                  class={cn(
                    'h-[3px] w-4 rounded-pill',
                    count === 0
                      ? 'bg-border-subtle'
                      : count <= 2
                        ? 'bg-primary-soft'
                        : 'bg-primary-deep'
                  )}
                  aria-hidden="true"
                ></span>
              </button>
            {/each}
          </div>
        {/if}

        <div class="flex flex-col gap-4">
          {#each days as day (day.toISOString())}
            {@render dayStack(day, nav.scope !== 'day')}
          {/each}
        </div>

        {#if nav.scope === 'day'}
          {@render weekFooter()}
        {/if}
      </div>
    </div>
  </div>

</ProtoShell>

{#if pickerOpen}
  <DatePickerSheet
    activeDate={nav.activeDate}
    now={model.now}
    format={model.format}
    eventsForDay={model.eventsForDay}
    onPick={(date) => {
      nav.pickDate(date);
      pickerOpen = false;
    }}
    onClose={() => (pickerOpen = false)}
  />
{/if}

<CourseDetailModal
  event={detail}
  locale="fr"
  now={model.now}
  onClose={() => (detail = null)}
/>
