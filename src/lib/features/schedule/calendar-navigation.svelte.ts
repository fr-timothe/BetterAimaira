import {
  addDays,
  addMonths,
  isSameMonth,
  startOfDay,
  startOfMonth,
  startOfWeek,
} from './date-utils';
import type { CalendarScope } from './types';

export type PeriodChangeHandler = (startDate: Date, durationDays: number) => void | Promise<void>;

/**
 * Only the weeks the month actually touches. A fixed 42 cells adds a whole
 * trailing week of foreign days to a short February, and asks the portal for
 * six weeks when five are shown.
 */
export function monthWeekCount(month: Date): number {
  const first = startOfMonth(month);
  const gridStart = startOfWeek(first);
  const spannedDays = Math.round(
    (addMonths(first, 1).getTime() - gridStart.getTime()) / 86_400_000
  );
  return Math.ceil(spannedDays / 7);
}

/** The cells of a month grid: whole weeks, from the Monday the month opens on. */
export function monthGridDays(month: Date): Date[] {
  const gridStart = startOfWeek(startOfMonth(month));
  return Array.from({ length: monthWeekCount(month) * 7 }, (_, index) => addDays(gridStart, index));
}

/**
 * Where the month keyboard lands from `key`, or null when the key is not one
 * the grid answers. Kept as arithmetic on its own so the grid component only
 * has to move the focus and let the navigation state follow.
 */
export function monthKeyTarget(key: string, from: Date): Date | null {
  switch (key) {
    case 'ArrowLeft':
      return addDays(from, -1);
    case 'ArrowRight':
      return addDays(from, 1);
    case 'ArrowUp':
      return addDays(from, -7);
    case 'ArrowDown':
      return addDays(from, 7);
    case 'Home':
      return startOfWeek(from);
    case 'End':
      return addDays(startOfWeek(from), 6);
    case 'PageUp':
      return addMonths(from, -1);
    case 'PageDown':
      return addMonths(from, 1);
    default:
      return null;
  }
}

/**
 * The calendar's scope and date state machine, and the one place allowed to
 * tell the parent that the visible period moved. Every surface — the header
 * buttons, the day strip, the month grid, the date picker, the swipe — writes
 * through these methods, so the three cursors can never disagree about which
 * day is shown, which day is selected and which cell owns the tab stop.
 */
export class CalendarNavigation {
  scope = $state<CalendarScope>('week');
  /** The period the grid is built around: a day, a week or a month. */
  anchorDate = $state<Date>(startOfDay(new Date()));
  /** The day the detail surfaces describe. */
  activeDate = $state<Date>(startOfDay(new Date()));
  /**
   * The month grid is one tab stop, not 42. This is the cell the arrow keys
   * moved to, which is also the only cell carrying `tabindex="0"`.
   */
  monthFocusDate = $state<Date>(startOfDay(new Date()));

  #onPeriodChange: () => PeriodChangeHandler | undefined;

  constructor(onPeriodChange: () => PeriodChangeHandler | undefined) {
    this.#onPeriodChange = onPeriodChange;
  }

  setScope = (scope: CalendarScope) => {
    this.scope = scope;
    this.triggerPeriodChange(this.anchorDate, scope);
  };

  /**
   * The parent owns a selected date of its own; adopting it moves all three
   * cursors at once and stays silent, because the parent already knows.
   */
  adoptSelectedDate = (date: Date) => {
    const day = startOfDay(date);
    this.anchorDate = day;
    this.activeDate = day;
    this.monthFocusDate = day;
  };

  movePeriod = (direction: -1 | 1) => {
    let newAnchor: Date;
    switch (this.scope) {
      case 'day':
        newAnchor = addDays(this.anchorDate, direction);
        this.activeDate = newAnchor;
        break;
      case 'week':
        newAnchor = addDays(this.anchorDate, direction * 7);
        this.activeDate = startOfWeek(newAnchor);
        break;
      case 'month':
        newAnchor = addMonths(this.anchorDate, direction);
        this.activeDate = startOfMonth(newAnchor);
        break;
    }
    this.anchorDate = newAnchor;
    this.monthFocusDate = this.activeDate;
    this.triggerPeriodChange(newAnchor, this.scope);
  };

  goToToday = () => {
    const today = startOfDay(new Date());
    this.anchorDate = today;
    this.activeDate = today;
    this.monthFocusDate = today;
    this.triggerPeriodChange(today, this.scope);
  };

  selectDate = (date: Date) => {
    this.activeDate = startOfDay(date);
    this.monthFocusDate = this.activeDate;
    if (this.scope === 'day') {
      this.anchorDate = this.activeDate;
      this.triggerPeriodChange(this.activeDate, this.scope);
    }
  };

  /**
   * Move to a date *and* change scope, as one step.
   *
   * Doing it as `selectDate` then `setScope` is wrong twice over, and both
   * ways were live: `selectDate` only moves the anchor while the scope is
   * already `day`, so zooming from a month cell onto 8 September landed on the
   * week of 31 August — the anchor never left the month's own. And two calls
   * ask the parent for two periods when one is wanted, so the portal is
   * queried for a range nobody is going to look at.
   */
  zoomTo = (date: Date, scope: CalendarScope) => {
    const picked = startOfDay(date);
    this.scope = scope;
    this.anchorDate =
      scope === 'week' ? startOfWeek(picked) : scope === 'month' ? startOfMonth(picked) : picked;
    this.activeDate = picked;
    this.monthFocusDate = picked;
    this.triggerPeriodChange(this.anchorDate, scope);
  };

  /**
   * Replaces `<input type="week">`, which neither WKWebView nor WebKitGTK
   * implements: on those platforms it degrades to a text field expecting
   * `2026-W35`, which is not a control a student can operate.
   */
  pickDate = (date: Date) => {
    const picked = startOfDay(date);
    this.anchorDate = this.scope === 'week' ? startOfWeek(picked) : picked;
    this.activeDate = picked;
    this.monthFocusDate = picked;
    this.triggerPeriodChange(this.anchorDate, this.scope);
  };

  /**
   * Moving the month keyboard out of the displayed month moves the month with
   * it, which is what makes the keyboard path equivalent to the pointer one.
   */
  focusMonthDate = (date: Date) => {
    this.monthFocusDate = date;
    if (isSameMonth(date, this.anchorDate)) return;
    this.anchorDate = startOfMonth(date);
    this.triggerPeriodChange(this.anchorDate, 'month');
  };

  triggerPeriodChange = (date: Date, scope: CalendarScope) => {
    const onPeriodChange = this.#onPeriodChange();
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
      case 'month': {
        const first = startOfMonth(date);
        startDate = startOfWeek(first);
        durationDays = monthWeekCount(first) * 7;
        break;
      }
    }

    void onPeriodChange(startDate, durationDays);
  };
}
