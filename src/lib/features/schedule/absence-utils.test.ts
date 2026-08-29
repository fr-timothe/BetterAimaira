import { describe, expect, it } from 'bun:test';
import {
  absenceHours,
  absenceStatus,
  absenceTotals,
  calculatePresenceRate,
  type AbsenceTotals,
} from './absence-utils';
import type { AbsenceEntry } from './types';

function createMockAbsenceEntry(overrides: Partial<AbsenceEntry>): AbsenceEntry {
  return {
    id: '1',
    date: '23/08/2026',
    time: '09:00',
    course: 'Marketing',
    duration: '2h00',
    excused: null,
    reason: null,
    ...overrides,
  };
}

describe('absence-utils presence calculation', () => {
  it('returns 100% presence when 0 hours are missed', () => {
    const totals: AbsenceTotals = {
      count: 0,
      hours: 0,
      excusedHours: 0,
      unexcusedHours: 0,
      pendingHours: 0,
    };
    expect(calculatePresenceRate(totals, 1)).toBe(100);
    expect(calculatePresenceRate(totals, 2)).toBe(100);
  });

  it('calculates proportional presence for moderate missed hours', () => {
    const totals: AbsenceTotals = {
      count: 3,
      hours: 15,
      excusedHours: 5,
      unexcusedHours: 10,
      pendingHours: 0,
    };
    // 1 block = 300h baseline -> (300 - 15) / 300 * 100 = 95.0%
    const rate = calculatePresenceRate(totals, 1);
    expect(rate).toBe(95.0);
  });

  it('calculates approximately 50% presence when half the volume is missed', () => {
    const totals: AbsenceTotals = {
      count: 20,
      hours: 150,
      excusedHours: 0,
      unexcusedHours: 150,
      pendingHours: 0,
    };
    // 1 block = 300h baseline -> (300 - 150) / 300 * 100 = 50%
    const rate = calculatePresenceRate(totals, 1);
    expect(rate).toBe(50.0);
  });

  it('handles extreme absences gracefully without going below 0%', () => {
    const totals: AbsenceTotals = {
      count: 100,
      hours: 400,
      excusedHours: 0,
      unexcusedHours: 400,
      pendingHours: 0,
    };
    const rate = calculatePresenceRate(totals, 1);
    expect(rate).toBeGreaterThanOrEqual(0);
    expect(rate).toBeLessThanOrEqual(100);
  });
});

describe('absence-utils basic helpers', () => {
  it('correctly determines absence status', () => {
    expect(absenceStatus(createMockAbsenceEntry({ excused: true }))).toBe('excused');
    expect(absenceStatus(createMockAbsenceEntry({ excused: false }))).toBe('unexcused');
    expect(absenceStatus(createMockAbsenceEntry({ excused: null }))).toBe('pending');
  });

  it('correctly calculates absence hours from various formats', () => {
    expect(absenceHours(createMockAbsenceEntry({ duration: '3,25' }))).toBe(3.25);
    expect(absenceHours(createMockAbsenceEntry({ duration: '1h30' }))).toBe(1.5);
    expect(absenceHours(createMockAbsenceEntry({ duration: '2:45' }))).toBe(2.75);
    expect(absenceHours(createMockAbsenceEntry({ duration: '' }))).toBe(0);
  });

  it('accumulates totals accurately', () => {
    const entries = [
      createMockAbsenceEntry({ duration: '2h00', excused: true }),
      createMockAbsenceEntry({ duration: '3h00', excused: false }),
      createMockAbsenceEntry({ duration: '1h30', excused: null }),
    ];
    const totals = absenceTotals(entries);
    expect(totals.count).toBe(3);
    expect(totals.hours).toBe(6.5);
    expect(totals.excusedHours).toBe(2);
    expect(totals.unexcusedHours).toBe(3);
    expect(totals.pendingHours).toBe(1.5);
  });
});
