<script lang="ts">
  import {
    AlertCircle,
    BookOpen,
    Calendar,
    Clock3,
    ExternalLink,
    MapPin,
    UserRound,
    X,
  } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import KindBadge from '$lib/components/ui/KindBadge.svelte';
  import Sheet from '$lib/components/ui/Sheet.svelte';
  import { capitalizeFirst } from './date-utils';
  import {
    eventEnd,
    eventSecondary,
    eventStart,
    eventTitle,
    formatDurationRange,
    getEventStatus,
    openExternalUrl,
    parseRoomAndTeacher,
  } from './course-utils';
  import type { CalendarEvent } from './types';
  import { cn } from '$lib/utils';

  type Props = {
    event: CalendarEvent | null;
    locale: Locale;
    now?: Date;
    onClose: () => void;
    onOpenTempo?: (event: CalendarEvent) => void;
  };

  let {
    event,
    locale,
    now = new Date(),
    onClose,
    onOpenTempo,
  }: Props = $props();

  const dateFormatter = $derived(
    new Intl.DateTimeFormat(locale, {
      weekday: 'long',
      day: 'numeric',
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

  const relativeTimeFormatter = $derived(
    new Intl.RelativeTimeFormat(locale, {
      numeric: 'auto',
    })
  );

  const startsAt = $derived(event ? eventStart(event) : null);
  const endsAt = $derived(event ? eventEnd(event) : null);
  const eventStatus = $derived(event ? getEventStatus(event, now) : 'finished');

  const courseTitle = $derived(event ? eventTitle(event, m.untitled_course()) : '');
  const courseSecondary = $derived(event ? eventSecondary(event) : null);
  const formattedDate = $derived(startsAt ? capitalizeFirst(dateFormatter.format(startsAt)) : '');
  const formattedTime = $derived(startsAt && endsAt ? `${timeFormatter.format(startsAt)} – ${timeFormatter.format(endsAt)}` : '');
  const durationLabel = $derived(event ? formatDurationRange(event.startsAt, event.endsAt) : '');

  const relativeCountdown = $derived.by(() => {
    if (!startsAt || eventStatus !== 'upcoming') return '';
    const diffMinutes = Math.ceil((startsAt.getTime() - now.getTime()) / 60_000);
    if (Math.abs(diffMinutes) < 60) {
      return relativeTimeFormatter.format(diffMinutes, 'minute');
    }
    return relativeTimeFormatter.format(Math.ceil(diffMinutes / 60), 'hour');
  });

  const parsedDetails = $derived(event ? parseRoomAndTeacher(event) : { room: null, teacher: null });

  async function handleTempoClick() {
    if (!event) return;
    if (onOpenTempo) {
      onOpenTempo(event);
      return;
    }
    await openExternalUrl(event.tempoUrl);
  }
  // A sunken field rather than a bordered white box: the sheet panel is already
  // `--card`, and a card inside a card only ever reads as a ghost.
  const cell = 'flex min-w-0 items-center gap-3';
  const detailCell = cell + ' rounded-lg bg-surface-sunken p-3';
  const plate = 'grid shrink-0 place-items-center bg-muted text-primary-deep';
  const cellLabel =
    'mb-1 block text-2xs font-semibold tracking-[0.03em] uppercase text-muted-foreground';
  const cellValue = 'text-base font-bold wrap-anywhere text-foreground';
</script>

{#if event}
  <Sheet title={m.course_details_title()} closeLabel={m.close()} {onClose} placement="center">
    <div class="flex flex-col gap-4 p-4 md:gap-5 md:p-6">
      <div class="flex items-start justify-between gap-3">
        <div class="flex flex-wrap items-center gap-2 pt-2">
          {#if eventStatus === 'live'}
            <Badge tone="live" dot>{m.schedule_status_live()}</Badge>
          {:else if eventStatus === 'upcoming'}
            <Badge tone="accent">
              {m.schedule_status_upcoming()}{#if relativeCountdown}&nbsp;({relativeCountdown}){/if}
            </Badge>
          {:else}
            <Badge tone="neutral">{m.schedule_status_finished()}</Badge>
          {/if}

          {#if event.kind}
            <KindBadge {event} />
          {/if}
        </div>

        <IconButton label={m.close()} variant="ghost" onclick={onClose}>
          <X size={20} strokeWidth={2.3} aria-hidden="true" />
        </IconButton>
      </div>

      <div>
        <h2
          class="mb-1 text-xl leading-[1.25] font-extrabold tracking-[-0.01em] text-balance
                 text-foreground md:text-2xl"
        >{courseTitle}</h2>
        {#if courseSecondary}
          <p class="text-base leading-[1.45] text-muted-foreground">{courseSecondary}</p>
        {/if}
      </div>

      <div
        class="grid grid-cols-1 gap-3 rounded-lg border border-border-subtle bg-surface-sunken p-4
               md:grid-cols-[1fr_auto_1fr] md:items-center md:p-5"
      >
        <div class={cell}>
          <span class={cn(plate, 'size-[2.35rem] rounded-md')}
            ><Calendar size={18} aria-hidden="true" /></span
          >
          <div>
            <small class={cellLabel}>{m.course_time()}</small>
            <strong class={cellValue}>{formattedDate}</strong>
          </div>
        </div>
        <div class="hidden h-[2.2rem] w-px bg-border-subtle md:block" aria-hidden="true"></div>
        <div class={cell}>
          <span class={cn(plate, 'size-[2.35rem] rounded-md')}
            ><Clock3 size={18} aria-hidden="true" /></span
          >
          <div>
            <!-- Clock and duration digits must not jump between ticks. -->
            <small class={cellLabel}>{m.course_duration()}</small>
            <strong class={cn(cellValue, 'tabular-nums')}>
              {formattedTime}
              <span class="text-sm font-semibold tabular-nums text-primary-deep"
                >({durationLabel})</span
              >
            </strong>
          </div>
        </div>
      </div>

      {#if parsedDetails.room || parsedDetails.teacher || (event.planification && event.planification !== courseTitle)}
        <div class="grid grid-cols-1 gap-3 min-[30rem]:grid-cols-2">
          {#if parsedDetails.room}
            <div class={detailCell}>
              <span class={cn(plate, 'size-[2.1rem] rounded-sm')}><MapPin size={17} aria-hidden="true" /></span>
              <div>
                <small class={cellLabel}>{m.course_room()}</small>
                <strong class={cellValue}>{parsedDetails.room}</strong>
              </div>
            </div>
          {/if}

          {#if parsedDetails.teacher}
            <div class={detailCell}>
              <span class={cn(plate, 'size-[2.1rem] rounded-sm')}><UserRound size={17} aria-hidden="true" /></span>
              <div>
                <small class={cellLabel}>{m.course_teacher()}</small>
                <strong class={cellValue}>{parsedDetails.teacher}</strong>
              </div>
            </div>
          {/if}

          {#if event.planification && event.planification !== courseTitle}
            <div class={cn(detailCell, 'col-span-full')}>
              <span class={cn(plate, 'size-[2.1rem] rounded-sm')}><BookOpen size={17} aria-hidden="true" /></span>
              <div>
                <small class={cellLabel}>{m.course_details()}</small>
                <strong class={cellValue}>{event.planification}</strong>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if event.externalComment}
        <div class="rounded-lg border border-warning-edge bg-warning-surface p-4">
          <div class="mb-1 flex items-center gap-2 text-sm font-bold text-warning-strong">
            <AlertCircle size={17} aria-hidden="true" />
            <strong>{m.course_notes()}</strong>
          </div>
          <p class="text-base leading-[1.5] text-warning-strong">{event.externalComment}</p>
        </div>
      {/if}

      <div class="flex flex-col gap-2 md:flex-row-reverse">
        {#if event.tempoUrl}
          <Button variant="primary" block onclick={handleTempoClick}>
            <span>{m.open_tempo()}</span>
            <ExternalLink size={18} aria-hidden="true" />
          </Button>
        {/if}

        <Button variant="outline" block onclick={onClose}>{m.close()}</Button>
      </div>
    </div>
  </Sheet>
{/if}

