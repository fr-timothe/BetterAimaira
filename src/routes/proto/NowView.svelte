<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE — structure C, "Maintenant d'abord".
   *
   * There is no control bar at all: an ink panel answers "which course, which
   * room, how long left" and is itself the header. The rest of the day scrolls
   * under it, and the scopes are a lens the panel opens rather than two more
   * views.
   *
   * NOTE for the build: the freshness line is hand-rolled here because
   * FreshnessLabel paints `--muted-foreground`, which is grey on ink. If this
   * structure is locked, that component gains an on-ink tone instead of a
   * second copy of the wording — DESIGN.md keeps sync state in one component.
   */
  import { onMount } from 'svelte';
  import { CalendarDays, CalendarOff, ChevronLeft, ChevronRight, ChevronUp, MapPin } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import CourseDetailModal from '$lib/features/schedule/CourseDetailModal.svelte';
  import { monthGridDays } from '$lib/features/schedule/calendar-navigation.svelte';
  import {
    eventDurationMinutes,
    eventEnd,
    eventStart,
    eventTitle,
    formatDuration,
    getEventStatus,
    isCancelled,
    parseRoomAndTeacher,
    relativeStartFromNow,
  } from '$lib/features/schedule/course-utils';
  import {
    addDays,
    capitalizeFirst,
    isSameDay,
    isSameMonth,
  } from '$lib/features/schedule/date-utils';
  import type { CalendarEvent } from '$lib/features/schedule/types';
  import { cn } from '$lib/utils';
  import ProtoShell from './ProtoShell.svelte';
  import { ProtoModel } from './proto-model.svelte';
  import { swipe } from './swipe';

  const model = new ProtoModel('day');
  const nav = model.navigation;

  onMount(model.startClock.bind(model));

  let detail = $state<CalendarEvent | null>(null);
  let lensOpen = $state(false);

  const dayEvents = $derived(model.eventsForDay(nav.activeDate));
  const isToday = $derived(isSameDay(nav.activeDate, model.now));

  /**
   * What the panel answers. On today it is the live or next course; on any
   * other day it is that day's first course, because the panel must always be
   * describing the day the rest of the screen shows.
   */
  const anchor = $derived.by(() => {
    if (!isToday) return dayEvents[0] ?? null;
    const nowMs = model.now.getTime();
    return (
      dayEvents.find(
        (event) =>
          eventStart(event).getTime() <= nowMs && nowMs < eventEnd(event).getTime()
      ) ??
      dayEvents.find((event) => eventStart(event).getTime() > nowMs) ??
      null
    );
  });

  const anchorLive = $derived(
    anchor !== null && isToday && getEventStatus(anchor, model.now) === 'live'
  );

  /** 0–1 of the way through the running course, for the progress bar. */
  const anchorProgress = $derived.by(() => {
    if (!anchor || !anchorLive) return null;
    const from = eventStart(anchor).getTime();
    const to = eventEnd(anchor).getTime();
    return Math.min(1, Math.max(0, (model.now.getTime() - from) / (to - from)));
  });

  const remainingLabel = $derived.by(() => {
    if (!anchor) return null;
    if (anchorLive) {
      const minutes = Math.max(
        0,
        Math.round((eventEnd(anchor).getTime() - model.now.getTime()) / 60_000)
      );
      return `${formatDuration(minutes)} restantes`;
    }
    if (isToday) return relativeStartFromNow(anchor, model.now);
    return formatDuration(eventDurationMinutes(anchor));
  });

  /** Everything after the anchor, which is what "la suite" means. */
  const rest = $derived.by(() => {
    if (!anchor) return dayEvents;
    const index = dayEvents.indexOf(anchor);
    return index === -1 ? dayEvents : dayEvents.slice(index + 1);
  });

  /**
   * The days after this one, up to three that actually carry courses. A light
   * day would otherwise leave two thirds of the screen empty under the panel,
   * and "rien" repeated is not information — the next real day is.
   */
  const upcoming = $derived.by(() => {
    const days: { date: Date; events: CalendarEvent[] }[] = [];
    for (let offset = 1; offset <= 14 && days.length < 3; offset += 1) {
      const date = addDays(nav.activeDate, offset);
      const events = model.eventsForDay(date);
      if (events.length > 0) days.push({ date, events });
    }
    return days;
  });

  const syncLabel = $derived(
    `Synchronisé à ${model.format.timeFormatter.format(new Date(model.fetchedAt))}`
  );

</script>

{#snippet restRow(event: CalendarEvent)}
  {@const details = parseRoomAndTeacher(event)}
  {@const status = getEventStatus(event, model.now)}
  {@const cancelled = isCancelled(event)}

  <button
    type="button"
    class={cn(
      'flex w-full items-center gap-3 rounded-md border border-border-subtle px-3 py-2.5',
      'text-start transition-control active:scale-(--press-scale) fine-hover:border-primary-deep',
      status === 'finished' ? 'bg-surface-sunken' : 'bg-card'
    )}
    onclick={() => (detail = event)}
  >
    <span class="flex w-[3rem] shrink-0 flex-col">
      <span
        class={cn(
          'text-sm leading-none font-extrabold tabular-nums',
          status === 'finished' ? 'text-muted-foreground' : 'text-primary-deep'
        )}>{model.format.timeFormatter.format(eventStart(event))}</span
      >
      <span class="pt-[0.15rem] text-2xs leading-none tabular-nums text-muted-foreground"
        >{formatDuration(eventDurationMinutes(event))}</span
      >
    </span>
    <span class="flex min-w-0 flex-1 flex-col gap-[0.15rem]">
      <span
        class={cn(
          'truncate text-sm font-bold',
          cancelled && 'line-through decoration-danger-strong',
          status === 'finished' ? 'text-muted-foreground' : 'text-foreground'
        )}>{eventTitle(event)}</span
      >
      {#if details.room}
        <span class="truncate text-2xs font-semibold text-muted-foreground">{details.room}</span>
      {/if}
    </span>
    {#if cancelled}
      <Badge tone="danger">Annulé</Badge>
    {:else}
      <KindBadge {event} />
    {/if}
  </button>
{/snippet}

<ProtoShell>
  <!-- The panel is the header. Nothing sits above it. The pull that opens the
       lens is an accelerator: the labelled control below does the same thing,
       because a gesture with no control behind it is not reachable by
       keyboard. -->
  <section
    class="relative z-raised shrink-0 rounded-b-xl bg-secondary px-4 pt-safe-3 pb-3
           text-secondary-foreground shadow-lg"
    use:swipe={{
      onSwipe: nav.movePeriod,
      onPullDown: () => (lensOpen = true),
      enabled: !lensOpen,
    }}
  >
    <div class="flex items-center justify-between gap-2">
      <IconButton
        label="Jour précédent"
        variant="ghost"
        class="text-primary-soft"
        onclick={() => nav.movePeriod(-1)}
      >
        <ChevronLeft size={20} strokeWidth={2.2} aria-hidden="true" />
      </IconButton>

      <button
        type="button"
        class="min-h-(--tap-min) min-w-0 flex-1 truncate text-center text-sm font-bold
               transition-control active:scale-(--press-scale)"
        onclick={() => (lensOpen = !lensOpen)}
      >
        {capitalizeFirst(
          new Intl.DateTimeFormat('fr', { weekday: 'long', day: 'numeric', month: 'long' }).format(
            nav.activeDate
          )
        )}
      </button>

      <IconButton
        label="Jour suivant"
        variant="ghost"
        class="text-primary-soft"
        onclick={() => nav.movePeriod(1)}
      >
        <ChevronRight size={20} strokeWidth={2.2} aria-hidden="true" />
      </IconButton>
    </div>

    {#if lensOpen}
      <!-- The lens: the month, drawn on the panel it came out of. This is
           what replaces the week and month scopes. -->
      <div class="flex flex-col gap-1 pt-1 pb-1 animate-slide-up-in">
        <div class="grid grid-cols-7 gap-1">
          {#each monthGridDays(nav.anchorDate).slice(0, 7) as day (day.toISOString())}
            <span
              class="text-center text-2xs font-bold tracking-[0.04em] uppercase text-primary-soft"
              >{new Intl.DateTimeFormat('fr', { weekday: 'short' }).format(day).slice(0, 2)}</span
            >
          {/each}
        </div>
        <div class="grid grid-cols-7 gap-1">
          {#each monthGridDays(nav.anchorDate) as day (day.toISOString())}
            {@const count = model.eventsForDay(day).length}
            {@const outside = !isSameMonth(day, nav.activeDate)}
            <button
              type="button"
              class={cn(
                'flex min-h-11 flex-col items-center justify-center gap-[0.2rem] rounded-sm',
                'transition-control active:scale-(--press-scale)',
                isSameDay(day, nav.activeDate)
                  ? 'bg-primary text-primary-foreground'
                  : outside
                    ? 'text-primary-soft/50'
                    : 'text-secondary-foreground'
              )}
              onclick={() => {
                nav.pickDate(day);
                lensOpen = false;
              }}
            >
              <span class="text-2xs leading-none font-bold tabular-nums">{day.getDate()}</span>
              <span
                class={cn(
                  'h-[3px] w-3.5 rounded-pill',
                  count === 0
                    ? 'bg-current opacity-25'
                    : count <= 2
                      ? 'bg-primary-soft'
                      : 'bg-primary'
                )}
                aria-hidden="true"
              ></span>
            </button>
          {/each}
        </div>
      </div>
    {:else if anchor}
      {@const details = parseRoomAndTeacher(anchor)}
      <div class="flex flex-col gap-2 pt-1">
        <div class="flex items-center gap-2">
          {#if anchorLive}
            <Badge tone="live" dot>En cours</Badge>
          {:else}
            <span class="text-2xs font-bold tracking-[0.04em] uppercase text-primary-soft">
              {isToday ? 'Prochain cours' : 'Premier cours'}
            </span>
          {/if}
          <KindBadge event={anchor} />
        </div>

        <h2 class="text-2xl leading-[1.15] font-extrabold tracking-[-0.02em] wrap-anywhere">
          {eventTitle(anchor)}
        </h2>

        <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm font-semibold text-primary-soft">
          <span class="inline-flex items-center gap-1.5">
            <MapPin size={15} aria-hidden="true" />
            {details.room ?? 'Salle non communiquée'}
          </span>
          {#if details.teacher}<span class="truncate">{details.teacher}</span>{/if}
        </div>

        <div class="flex items-baseline justify-between gap-3 pt-0.5">
          <span class="text-md font-extrabold tabular-nums">
            {model.format.eventTimeRange(anchor)}
          </span>
          {#if remainingLabel}
            <span class="text-sm font-bold text-primary-soft">{remainingLabel}</span>
          {/if}
        </div>

        {#if anchorProgress !== null}
          <div
            class="h-1.5 w-full overflow-hidden rounded-pill bg-primary-soft/25"
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={Math.round(anchorProgress * 100)}
            aria-label="Progression du cours"
          >
            <div
              class="h-full rounded-pill bg-primary transition-[width] duration-normal ease-out"
              style:width={`${anchorProgress * 100}%`}
            ></div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex flex-col gap-2 py-4">
        <CalendarOff size={26} class="text-primary-soft" aria-hidden="true" />
        <h2 class="text-xl leading-tight font-extrabold">Journée libre</h2>
        <p class="text-sm font-semibold text-primary-soft">
          Le portail n’a renvoyé aucun créneau pour ce jour.
        </p>
      </div>
    {/if}

    <div class="flex items-center justify-between gap-2 pt-2">
      <span class="text-2xs font-semibold text-primary-soft">{syncLabel}</span>
      <button
        type="button"
        class="inline-flex min-h-8 items-center gap-1 rounded-pill px-2 text-2xs font-bold
               text-primary-soft transition-control active:scale-(--press-scale)"
        onclick={() => (lensOpen = !lensOpen)}
      >
        {#if lensOpen}
          <ChevronUp size={14} aria-hidden="true" />
          Replier
        {:else}
          <CalendarDays size={14} aria-hidden="true" />
          Choisir un jour
        {/if}
      </button>
    </div>
  </section>

  <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-3 pt-3 pb-4">
    {#if rest.length > 0}
      <h3 class="pb-2 text-xs font-extrabold tracking-[0.04em] uppercase text-muted-foreground">
        La suite
      </h3>
      <div class="flex flex-col gap-1.5">
        {#each rest as event (event.id)}
          {@render restRow(event)}
        {/each}
      </div>
    {:else if anchor}
      <p class="py-2 text-xs font-semibold text-muted-foreground">
        Plus rien après ce cours.
      </p>
    {/if}

    {#each upcoming as day (day.date.toISOString())}
      <h3
        class="flex items-baseline justify-between gap-2 pt-5 pb-2 text-xs font-extrabold
               tracking-[0.04em] uppercase text-muted-foreground"
      >
        <span>
          {capitalizeFirst(
            new Intl.DateTimeFormat('fr', { weekday: 'long', day: 'numeric' }).format(day.date)
          )}
        </span>
        <span class="tabular-nums normal-case">{day.events.length} cours</span>
      </h3>
      <div class="flex flex-col gap-1.5">
        {#each day.events as event (event.id)}
          {@render restRow(event)}
        {/each}
      </div>
    {/each}
  </div>
</ProtoShell>

<CourseDetailModal
  event={detail}
  locale="fr"
  now={model.now}
  onClose={() => (detail = null)}
/>
