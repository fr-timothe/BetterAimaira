import { describe, expect, it } from 'bun:test';
import {
  blockGeometry,
  eventMinutes,
  gapMinutes,
  layoutDay,
  ratioInWindow,
  timeWindowFor,
  windowHours,
} from './calendar-layout';
import type { CalendarEvent } from './types';

function createEvent(startsAt: string, endsAt: string, id = startsAt): CalendarEvent {
  return {
    id,
    startsAt,
    endsAt,
    planification: 'Cours',
    description: '',
    kind: 'Cours',
    externalComment: '',
    tempoUrl: null,
  };
}

describe('eventMinutes', () => {
  it('reads start and end as minutes from midnight', () => {
    const event = createEvent('2026-08-25T08:30:00', '2026-08-25T10:00:00');
    expect(eventMinutes(event)).toEqual({ fromMinutes: 510, toMinutes: 600 });
  });

  it('clamps an event that would run past midnight', () => {
    const event = createEvent('2026-08-25T23:00:00', '2026-08-26T01:00:00');
    expect(eventMinutes(event).toMinutes).toBe(24 * 60);
  });
});

describe('timeWindowFor', () => {
  it('falls back to the school day when there is nothing to show', () => {
    expect(timeWindowFor([])).toEqual({ startMinutes: 8 * 60, endMinutes: 18 * 60 });
  });

  it('keeps the default band when every course fits inside it', () => {
    const events = [createEvent('2026-08-25T09:00:00', '2026-08-25T12:00:00')];
    expect(timeWindowFor(events)).toEqual({ startMinutes: 8 * 60, endMinutes: 18 * 60 });
  });

  it('widens outward to whole hours around an early lab and a late exam', () => {
    const events = [
      createEvent('2026-08-25T07:30:00', '2026-08-25T09:00:00'),
      createEvent('2026-08-25T18:15:00', '2026-08-25T20:15:00'),
    ];
    expect(timeWindowFor(events)).toEqual({ startMinutes: 7 * 60, endMinutes: 21 * 60 });
  });

  it('never returns a band too short to draw', () => {
    const events = [createEvent('2026-08-25T10:00:00', '2026-08-25T10:30:00')];
    const window = timeWindowFor(events);
    expect(window.endMinutes - window.startMinutes).toBeGreaterThanOrEqual(6 * 60);
  });

  it('lists one hour line per hour, both bounds included', () => {
    expect(windowHours({ startMinutes: 8 * 60, endMinutes: 11 * 60 })).toEqual([8, 9, 10, 11]);
  });
});

describe('layoutDay', () => {
  it('gives a full-width lane to back-to-back courses', () => {
    const blocks = layoutDay([
      createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00', 'a'),
      createEvent('2026-08-25T10:00:00', '2026-08-25T12:00:00', 'b'),
    ]);
    expect(blocks.map((block) => [block.lane, block.lanes])).toEqual([
      [0, 1],
      [0, 1],
    ]);
  });

  it('splits two overlapping courses into two lanes', () => {
    const blocks = layoutDay([
      createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00', 'a'),
      createEvent('2026-08-25T09:00:00', '2026-08-25T11:00:00', 'b'),
    ]);
    expect(blocks.map((block) => [block.lane, block.lanes])).toEqual([
      [0, 2],
      [1, 2],
    ]);
  });

  it('reuses a freed lane inside the same cluster', () => {
    const blocks = layoutDay([
      createEvent('2026-08-25T08:00:00', '2026-08-25T09:00:00', 'a'),
      createEvent('2026-08-25T08:30:00', '2026-08-25T11:00:00', 'b'),
      createEvent('2026-08-25T09:00:00', '2026-08-25T10:00:00', 'c'),
    ]);
    const byId = new Map(blocks.map((block) => [block.event.id, block]));
    expect(byId.get('a')?.lane).toBe(0);
    expect(byId.get('b')?.lane).toBe(1);
    expect(byId.get('c')?.lane).toBe(0);
    expect(new Set(blocks.map((block) => block.lanes))).toEqual(new Set([2]));
  });

  it('does not let one crowded morning shrink an unrelated afternoon', () => {
    const blocks = layoutDay([
      createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00', 'a'),
      createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00', 'b'),
      createEvent('2026-08-25T14:00:00', '2026-08-25T16:00:00', 'c'),
    ]);
    const afternoon = blocks.find((block) => block.event.id === 'c');
    expect(afternoon?.lanes).toBe(1);
  });
});

describe('blockGeometry', () => {
  const window = { startMinutes: 8 * 60, endMinutes: 18 * 60 };

  it('maps a mid-morning course onto the band', () => {
    const [block] = layoutDay([createEvent('2026-08-25T10:00:00', '2026-08-25T11:00:00')]);
    const geometry = blockGeometry(block, window);
    expect(geometry.top).toBeCloseTo(20);
    expect(geometry.height).toBeCloseTo(10);
    expect(geometry.left).toBe(0);
    expect(geometry.width).toBe(100);
  });

  it('halves the width of each lane in a two-lane cluster', () => {
    const blocks = layoutDay([
      createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00', 'a'),
      createEvent('2026-08-25T09:00:00', '2026-08-25T11:00:00', 'b'),
    ]);
    const geometries = blocks.map((block) => blockGeometry(block, window));
    expect(geometries.map((geometry) => geometry.width)).toEqual([50, 50]);
    expect(geometries.map((geometry) => geometry.left)).toEqual([0, 50]);
  });

  it('keeps a very short course tall enough to read and to tap', () => {
    const [block] = layoutDay([createEvent('2026-08-25T10:00:00', '2026-08-25T10:15:00')]);
    expect(blockGeometry(block, window).height).toBeCloseTo(5);
  });

  it('crops a course that starts before the band instead of overflowing it', () => {
    const [block] = layoutDay([createEvent('2026-08-25T07:00:00', '2026-08-25T09:00:00')]);
    const geometry = blockGeometry(block, { startMinutes: 8 * 60, endMinutes: 18 * 60 });
    expect(geometry.top).toBe(0);
    expect(geometry.height).toBeCloseTo(10);
  });
});

describe('ratioInWindow', () => {
  const window = { startMinutes: 8 * 60, endMinutes: 18 * 60 };

  it('places the current time inside the band', () => {
    expect(ratioInWindow(new Date('2026-08-25T13:00:00'), window)).toBeCloseTo(0.5);
  });

  it('returns null before and after the band', () => {
    expect(ratioInWindow(new Date('2026-08-25T06:30:00'), window)).toBeNull();
    expect(ratioInWindow(new Date('2026-08-25T22:00:00'), window)).toBeNull();
  });
});

describe('gapMinutes', () => {
  it('is zero for a single course', () => {
    expect(gapMinutes([createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00')])).toBe(0);
  });

  it('counts the hole between two courses', () => {
    expect(
      gapMinutes([
        createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00', 'a'),
        createEvent('2026-08-25T12:00:00', '2026-08-25T14:00:00', 'b'),
      ])
    ).toBe(120);
  });

  it('does not invent free time out of an overlap', () => {
    expect(
      gapMinutes([
        createEvent('2026-08-25T08:00:00', '2026-08-25T10:00:00', 'a'),
        createEvent('2026-08-25T09:00:00', '2026-08-25T11:00:00', 'b'),
      ])
    ).toBe(0);
  });
});
