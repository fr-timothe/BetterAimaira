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
</script>

{#if event}
  <Sheet title={m.course_details_title()} closeLabel={m.close()} {onClose} placement="center">
    <div class="course-detail">
      <div class="detail-header">
        <div class="detail-tags">
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

      <div class="detail-title-block">
        <h2>{courseTitle}</h2>
        {#if courseSecondary}
          <p>{courseSecondary}</p>
        {/if}
      </div>

      <div class="schedule-box">
        <div class="schedule-row">
          <span class="icon-plate"><Calendar size={18} aria-hidden="true" /></span>
          <div>
            <small>{m.course_time()}</small>
            <strong>{formattedDate}</strong>
          </div>
        </div>
        <div class="schedule-divider" aria-hidden="true"></div>
        <div class="schedule-row">
          <span class="icon-plate"><Clock3 size={18} aria-hidden="true" /></span>
          <div>
            <small>{m.course_duration()}</small>
            <strong class="numeric">
              {formattedTime}
              <span class="duration-tag">({durationLabel})</span>
            </strong>
          </div>
        </div>
      </div>

      {#if parsedDetails.room || parsedDetails.teacher || (event.planification && event.planification !== courseTitle)}
        <div class="details-grid">
          {#if parsedDetails.room}
            <div class="detail-cell">
              <span class="icon-plate small"><MapPin size={17} aria-hidden="true" /></span>
              <div>
                <small>{m.course_room()}</small>
                <strong>{parsedDetails.room}</strong>
              </div>
            </div>
          {/if}

          {#if parsedDetails.teacher}
            <div class="detail-cell">
              <span class="icon-plate small"><UserRound size={17} aria-hidden="true" /></span>
              <div>
                <small>{m.course_teacher()}</small>
                <strong>{parsedDetails.teacher}</strong>
              </div>
            </div>
          {/if}

          {#if event.planification && event.planification !== courseTitle}
            <div class="detail-cell detail-full">
              <span class="icon-plate small"><BookOpen size={17} aria-hidden="true" /></span>
              <div>
                <small>{m.course_details()}</small>
                <strong>{event.planification}</strong>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if event.externalComment}
        <div class="comment-callout">
          <div class="comment-header">
            <AlertCircle size={17} aria-hidden="true" />
            <strong>{m.course_notes()}</strong>
          </div>
          <p>{event.externalComment}</p>
        </div>
      {/if}

      <div class="detail-actions">
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

<style>
  /* Sheet owns the panel surface, elevation, focus trap and scroll lock, so this
     file only styles the content it puts inside. */
  .course-detail {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
  }

  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .detail-tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    padding-top: var(--space-2);
  }

  .detail-title-block h2 {
    margin: 0 0 var(--space-1);
    color: var(--foreground);
    font-size: var(--text-xl);
    font-weight: var(--weight-heavy);
    line-height: 1.25;
    letter-spacing: -0.01em;
    text-wrap: balance;
  }

  .detail-title-block p {
    margin: 0;
    color: var(--muted-foreground);
    font-size: var(--text-base);
    line-height: 1.45;
  }

  .schedule-box {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
  }

  .schedule-row,
  .detail-cell {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: var(--space-3);
  }

  .icon-plate {
    display: grid;
    width: 2.35rem;
    height: 2.35rem;
    flex-shrink: 0;
    place-items: center;
    color: var(--primary-deep);
    background: var(--muted);
    border-radius: var(--radius-md);
  }

  .icon-plate.small {
    width: 2.1rem;
    height: 2.1rem;
    border-radius: var(--radius-sm);
  }

  .schedule-row small,
  .detail-cell small {
    display: block;
    margin-bottom: var(--space-1);
    color: var(--muted-foreground);
    font-size: var(--text-2xs);
    font-weight: var(--weight-semibold);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .schedule-row strong,
  .detail-cell strong {
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    overflow-wrap: anywhere;
  }

  /* Clock and duration digits must not jump between ticks. */
  .numeric {
    font-variant-numeric: tabular-nums;
  }

  .duration-tag {
    color: var(--primary-deep);
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    font-variant-numeric: tabular-nums;
  }

  .schedule-divider {
    display: none;
    width: 1px;
    height: 2.2rem;
    background: var(--border-subtle);
  }

  .details-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: var(--space-3);
  }

  /* A sunken field rather than a bordered white box: the sheet panel is already
     `--card`, and a card inside a card only ever reads as a ghost. */
  .detail-cell {
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-lg);
  }

  .detail-full {
    grid-column: 1 / -1;
  }

  .comment-callout {
    padding: var(--space-4);
    background: var(--warning-surface);
    border: 1px solid color-mix(in oklch, var(--warning) 34%, transparent);
    border-radius: var(--radius-lg);
  }

  .comment-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
    color: var(--warning-strong);
    font-size: var(--text-sm);
    font-weight: var(--weight-bold);
  }

  .comment-callout p {
    margin: 0;
    color: var(--warning-strong);
    font-size: var(--text-base);
    line-height: 1.5;
  }

  .detail-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  /* 30rem lets the two short detail cells sit side by side; 48rem is the primary
     hinge, where the panel is wide enough for the schedule box to go three-up. */
  @media (min-width: 30rem) {
    .details-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (min-width: 48rem) {
    .course-detail {
      gap: var(--space-5);
      padding: var(--space-6);
    }

    .detail-title-block h2 {
      font-size: var(--text-2xl);
    }

    .schedule-box {
      grid-template-columns: 1fr auto 1fr;
      align-items: center;
      padding: var(--space-5);
    }

    .schedule-divider {
      display: block;
    }

    .detail-actions {
      flex-direction: row-reverse;
    }
  }
</style>
