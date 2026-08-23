import type { AbsenceBlock, AbsenceEntry, AbsencePeriod } from './types';

/** The portal states the excused column as `Oui`, `Non`, or nothing at all. */
export type AbsenceStatus = 'excused' | 'unexcused' | 'pending';

export type AbsenceTotals = {
  count: number;
  hours: number;
  excusedHours: number;
  unexcusedHours: number;
  pendingHours: number;
};

export function absenceStatus(entry: AbsenceEntry): AbsenceStatus {
  if (entry.excused === null) return 'pending';
  return entry.excused ? 'excused' : 'unexcused';
}

/**
 * `3,25` is the portal's own count of hours; `1h30` and `1:30` show up on other
 * campuses. An unreadable duration counts as no hours rather than as one — the
 * session is still listed, only its weight is unknown.
 */
export function absenceHours(entry: AbsenceEntry): number {
  const duration = entry.duration?.trim();
  if (!duration) return 0;

  const hoursAndMinutes = duration.match(/^(\d+)\s*(?:h|:)\s*(\d+)?/i);
  if (hoursAndMinutes) {
    const hours = Number.parseInt(hoursAndMinutes[1], 10);
    const minutes = hoursAndMinutes[2] ? Number.parseInt(hoursAndMinutes[2], 10) : 0;
    return hours + minutes / 60;
  }

  const decimal = duration.match(/^(\d+(?:[.,]\d+)?)/);
  if (decimal) return Number.parseFloat(decimal[1].replace(',', '.'));

  return 0;
}

export function blockEntries(block: AbsenceBlock): AbsenceEntry[] {
  return block.entries;
}

export function periodEntries(period: AbsencePeriod): AbsenceEntry[] {
  return period.blocks.flatMap((block) => block.entries);
}

export function absenceTotals(entries: AbsenceEntry[]): AbsenceTotals {
  const totals: AbsenceTotals = {
    count: entries.length,
    hours: 0,
    excusedHours: 0,
    unexcusedHours: 0,
    pendingHours: 0,
  };

  for (const entry of entries) {
    const hours = absenceHours(entry);
    totals.hours += hours;
    switch (absenceStatus(entry)) {
      case 'excused':
        totals.excusedHours += hours;
        break;
      case 'unexcused':
        totals.unexcusedHours += hours;
        break;
      case 'pending':
        totals.pendingHours += hours;
        break;
    }
  }

  return totals;
}

export function blockHours(block: AbsenceBlock): number {
  return absenceTotals(block.entries).hours;
}

/**
 * Running total of missed hours in portal order. The portal lists a block's
 * sessions oldest first, which is the only ordering it publishes.
 */
export function cumulativeHours(entries: AbsenceEntry[]): number[] {
  const series: number[] = [];
  let total = 0;

  for (const entry of entries) {
    total += absenceHours(entry);
    series.push(total);
  }

  return series;
}

/**
 * `29/09/2025` sorts as a string in the wrong order. Newest first, so the last
 * missed session is the first one read.
 */
export function sortEntriesByDateDesc(entries: AbsenceEntry[]): AbsenceEntry[] {
  return [...entries].sort((left, right) => entrySortKey(right) - entrySortKey(left));
}

function entrySortKey(entry: AbsenceEntry): number {
  const date = entry.date.match(/(\d{2})\/(\d{2})\/(\d{4})/);
  if (!date) return Number.NEGATIVE_INFINITY;
  const [, day, month, year] = date;
  const time = entry.time?.match(/^(\d{1,2}):(\d{2})/);
  const hours = time ? Number.parseInt(time[1], 10) : 0;
  const minutes = time ? Number.parseInt(time[2], 10) : 0;
  return Date.UTC(
    Number.parseInt(year, 10),
    Number.parseInt(month, 10) - 1,
    Number.parseInt(day, 10),
    hours,
    minutes
  );
}

/**
 * Calculates an attendance/presence percentage (0 to 100) based on missed hours.
 * At 0 missed hours -> 100% presence.
 * Base reference semester/annual volume is ~150h per block (minimum 300h).
 * As missed hours increase, the presence rate decreases smoothly.
 */
export function calculatePresenceRate(totals: AbsenceTotals, blockCount = 1): number {
  if (totals.hours <= 0) return 100;
  const baseVolume = Math.max(Math.max(blockCount, 1) * 150, 300);
  const totalVolume = Math.max(baseVolume, totals.hours + 20);
  const rate = Math.max(0, Math.min(100, ((totalVolume - totals.hours) / totalVolume) * 100));
  return Math.round(rate * 10) / 10;
}

/**
 * Maps a presence rate (0% to 100%) to a color hue:
 * - 0% -> 0 (Red)
 * - 50% -> 48 (Golden Yellow / Amber)
 * - 100% -> 142 (Emerald Green)
 */
export function presenceHue(rate: number): number {
  const clamped = Math.max(0, Math.min(100, Number.isFinite(rate) ? rate : 100));
  if (clamped <= 50) {
    const t = clamped / 50;
    return Math.round(0 + t * 48);
  }
  const t = (clamped - 50) / 50;
  return Math.round(48 + t * (142 - 48));
}

/**
 * Returns inline CSS variables for adaptive styling on presence cards.
 */
export function presenceColorStyle(rate: number): string {
  const clamped = Math.max(0, Math.min(100, Number.isFinite(rate) ? rate : 100));
  const hue = presenceHue(clamped);
  return `--presence-hue: ${hue}; --presence-rate: ${clamped}%;`;
}
