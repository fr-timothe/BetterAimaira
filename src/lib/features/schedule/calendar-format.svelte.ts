import * as m from '$lib/paraglide/messages.js';
import type { Locale } from '$lib/paraglide/runtime.js';
import { capitalizeFirst, getWeekNumber } from './date-utils';
import type { CalendarEvent, CalendarScope } from './types';

/** A fixed Monday-first week, only ever used to print weekday column titles. */
export const monthHeaderDays = Array.from({ length: 7 }, (_, index) => new Date(2024, 0, 1 + index));

/** What the period title has to know, read fresh on every locale or date change. */
export type CalendarPeriod = {
  scope: CalendarScope;
  anchorDate: Date;
  activeDate: Date;
  /** The visible week, Monday first; its length is the Sunday setting. */
  weekDays: Date[];
};

/**
 * Every string the calendar prints from a date, in one place. The formatters
 * are rebuilt on a locale change rather than reused, because an
 * `Intl.DateTimeFormat` is bound to the locale it was constructed with.
 */
export class CalendarFormat {
  #locale: () => Locale;
  #period: () => CalendarPeriod;

  constructor(locale: () => Locale, period: () => CalendarPeriod) {
    this.#locale = locale;
    this.#period = period;
  }

  dayFormatter = $derived.by(
    () =>
      new Intl.DateTimeFormat(this.#locale(), {
        weekday: 'long',
        day: 'numeric',
        month: 'long',
        year: 'numeric',
      })
  );

  weekdayShortFormatter = $derived.by(
    () => new Intl.DateTimeFormat(this.#locale(), { weekday: 'short' })
  );

  monthYearFormatter = $derived.by(
    () => new Intl.DateTimeFormat(this.#locale(), { month: 'long', year: 'numeric' })
  );

  timeFormatter = $derived.by(
    () => new Intl.DateTimeFormat(this.#locale(), { hour: '2-digit', minute: '2-digit' })
  );

  rangeFormatter = $derived.by(
    () =>
      new Intl.DateTimeFormat(this.#locale(), { day: 'numeric', month: 'short', year: 'numeric' })
  );

  periodLabel = $derived.by(() => {
    const period = this.#period();
    switch (period.scope) {
      case 'day':
        return capitalizeFirst(this.dayFormatter.format(period.activeDate));
      case 'week': {
        const weekStart = period.weekDays[0];
        const weekEnd = period.weekDays[period.weekDays.length - 1];
        return m.calendar_week_range({
          week: getWeekNumber(weekStart),
          range: this.rangeFormatter.formatRange(weekStart, weekEnd),
        });
      }
      case 'month':
        return capitalizeFirst(this.monthYearFormatter.format(period.anchorDate));
    }
  });

  scopeName = $derived.by(() => {
    switch (this.#period().scope) {
      case 'day':
        return m.scope_day();
      case 'week':
        return m.scope_week();
      case 'month':
        return m.scope_month();
    }
  });

  scopeOptions = $derived.by(() => {
    return [
      { value: 'day', label: m.scope_day() },
      { value: 'week', label: m.scope_week() },
      { value: 'month', label: m.scope_month() },
    ];
  });

  eventTimeRange = (event: CalendarEvent): string =>
    `${this.timeFormatter.format(new Date(event.startsAt))} – ${this.timeFormatter.format(new Date(event.endsAt))}`;

  dayCountLabel = (count: number): string => m.day_course_count({ count });
}
