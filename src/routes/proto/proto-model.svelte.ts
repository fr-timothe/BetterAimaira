/**
 * DEV-ONLY PROTOTYPE — not part of the shipped app.
 *
 * What the three candidate structures share: the same demo week, the same
 * clock, and the same `CalendarNavigation` / `CalendarFormat` the real view
 * already uses. Keeping the state machine identical is what makes the
 * comparison about composition instead of about three different behaviours.
 */
import { CalendarFormat } from '$lib/features/schedule/calendar-format.svelte';
import { CalendarNavigation } from '$lib/features/schedule/calendar-navigation.svelte';
import { addDays, dayKey, startOfWeek } from '$lib/features/schedule/date-utils';
import type { CalendarEvent, CalendarScope } from '$lib/features/schedule/types';
import { demoEvents, demoFetchedAt } from './demo';

/** The portal week the reference instance returns: Monday through Saturday. */
const VISIBLE_WEEK_DAYS = 6;

export class ProtoModel {
  now = $state(new Date());
  navigation = new CalendarNavigation(() => undefined);
  format: CalendarFormat;

  readonly events: CalendarEvent[];
  readonly fetchedAt: number;

  #byDay: Map<string, CalendarEvent[]>;

  constructor(initialScope: CalendarScope = 'day') {
    const reference = new Date();
    this.events = demoEvents(reference);
    this.fetchedAt = demoFetchedAt(reference);
    this.navigation.scope = initialScope;

    this.#byDay = new Map();
    for (const event of [...this.events].sort(
      (a, b) => new Date(a.startsAt).getTime() - new Date(b.startsAt).getTime()
    )) {
      const key = dayKey(new Date(event.startsAt));
      const bucket = this.#byDay.get(key);
      if (bucket) bucket.push(event);
      else this.#byDay.set(key, [event]);
    }

    this.format = new CalendarFormat(
      () => 'fr',
      () => ({
        scope: this.navigation.scope,
        anchorDate: this.navigation.anchorDate,
        activeDate: this.navigation.activeDate,
        weekDays: this.weekDays,
      })
    );
  }

  /** Drives the live badge and the now-line without a timer per component. */
  startClock() {
    const timer = setInterval(() => (this.now = new Date()), 30_000);
    return () => clearInterval(timer);
  }

  eventsForDay = (date: Date): CalendarEvent[] => this.#byDay.get(dayKey(date)) ?? [];

  weekDays = $derived.by(() => {
    const start = startOfWeek(this.navigation.anchorDate);
    return Array.from({ length: VISIBLE_WEEK_DAYS }, (_, index) => addDays(start, index));
  });

  weekEvents = $derived.by(() => this.weekDays.flatMap((day) => this.eventsForDay(day)));

  /** The course running now, or the next one today, or the next one at all. */
  currentOrNext = $derived.by(() => {
    const nowMs = this.now.getTime();
    const live = this.events.find(
      (event) =>
        new Date(event.startsAt).getTime() <= nowMs && nowMs < new Date(event.endsAt).getTime()
    );
    if (live) return { event: live, live: true };
    const next = this.events.find((event) => new Date(event.startsAt).getTime() > nowMs);
    return next ? { event: next, live: false } : null;
  });
}
