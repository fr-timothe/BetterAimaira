import type { CalendarEvent } from './types';
import { eventDurationMinutes, eventStart } from './course-utils';

/**
 * Geometry for the time-axis calendar. Everything here is pure arithmetic on
 * minutes-from-midnight so it can be tested without a DOM, and so the component
 * only ever converts a ratio into a percentage.
 */

export type TimeWindow = {
  /** Minutes from midnight of the first hour line, always on the hour. */
  startMinutes: number;
  /** Minutes from midnight of the last hour line, always on the hour. */
  endMinutes: number;
};

export type PositionedEvent = {
  event: CalendarEvent;
  fromMinutes: number;
  toMinutes: number;
  /** 0-based lane inside the overlap cluster this event belongs to. */
  lane: number;
  /** Number of lanes its cluster needs, so a solo course still fills its day. */
  lanes: number;
};

export type BlockGeometry = {
  /** Percentages of the day column, ready for `top` / `height` / `left` / `width`. */
  top: number;
  height: number;
  left: number;
  width: number;
};

const MINUTES_PER_DAY = 24 * 60;

/** A school day, and the band the grid falls back to when nothing is scheduled. */
const DEFAULT_START_MINUTES = 8 * 60;
const DEFAULT_END_MINUTES = 18 * 60;

/** Below this the hour rows collapse into an unreadable strip. */
const MIN_SPAN_MINUTES = 6 * 60;

/** A 15-minute course still needs a legible block and a reachable tap target. */
const MIN_BLOCK_MINUTES = 30;

export function minutesOfDay(date: Date): number {
  return date.getHours() * 60 + date.getMinutes();
}

/**
 * Start and end of an event inside the day it starts on. A course running past
 * midnight is clamped rather than wrapped: the portal has never returned one,
 * and a negative height would be worse than a truncated block.
 */
export function eventMinutes(event: CalendarEvent): { fromMinutes: number; toMinutes: number } {
  const fromMinutes = minutesOfDay(eventStart(event));
  const toMinutes = Math.min(MINUTES_PER_DAY, fromMinutes + eventDurationMinutes(event));
  return { fromMinutes, toMinutes };
}

/**
 * The visible band. It opens on the default school day and widens outward to
 * whole hours until every event fits, so an 07:30 lab or a 20:00 exam is never
 * cropped out of the grid it belongs to.
 */
export function timeWindowFor(events: CalendarEvent[]): TimeWindow {
  let start = DEFAULT_START_MINUTES;
  let end = DEFAULT_END_MINUTES;

  for (const event of events) {
    const { fromMinutes, toMinutes } = eventMinutes(event);
    if (fromMinutes < start) start = fromMinutes;
    if (toMinutes > end) end = toMinutes;
  }

  start = Math.max(0, Math.floor(start / 60) * 60);
  end = Math.min(MINUTES_PER_DAY, Math.ceil(end / 60) * 60);

  if (end - start < MIN_SPAN_MINUTES) {
    end = Math.min(MINUTES_PER_DAY, start + MIN_SPAN_MINUTES);
    start = Math.max(0, end - MIN_SPAN_MINUTES);
  }

  return { startMinutes: start, endMinutes: end };
}

export function windowHours(window: TimeWindow): number[] {
  const hours: number[] = [];
  for (let minutes = window.startMinutes; minutes <= window.endMinutes; minutes += 60) {
    hours.push(minutes / 60);
  }
  return hours;
}

/**
 * Places a day's events on lanes. Events that overlap in time share a cluster
 * and split its width; events that only touch end-to-start do not, so a normal
 * day of back-to-back courses stays full width.
 */
export function layoutDay(events: CalendarEvent[]): PositionedEvent[] {
  const sorted = events
    .map((event) => ({ event, ...eventMinutes(event) }))
    .sort((left, right) => left.fromMinutes - right.fromMinutes || right.toMinutes - left.toMinutes);

  const positioned: PositionedEvent[] = [];
  let cluster: PositionedEvent[] = [];
  let clusterEnd = -1;
  /** Last occupied minute per lane, reset with every cluster. */
  let laneEnds: number[] = [];

  function closeCluster() {
    const lanes = laneEnds.length;
    for (const block of cluster) block.lanes = lanes;
    cluster = [];
    laneEnds = [];
    clusterEnd = -1;
  }

  for (const item of sorted) {
    if (cluster.length > 0 && item.fromMinutes >= clusterEnd) closeCluster();

    let lane = laneEnds.findIndex((end) => end <= item.fromMinutes);
    if (lane === -1) {
      lane = laneEnds.length;
      laneEnds.push(item.toMinutes);
    } else {
      laneEnds[lane] = item.toMinutes;
    }

    const block: PositionedEvent = { ...item, lane, lanes: 1 };
    cluster.push(block);
    positioned.push(block);
    clusterEnd = Math.max(clusterEnd, item.toMinutes);
  }

  if (cluster.length > 0) closeCluster();

  return positioned;
}

export function blockGeometry(block: PositionedEvent, window: TimeWindow): BlockGeometry {
  const span = Math.max(1, window.endMinutes - window.startMinutes);
  const from = Math.max(window.startMinutes, block.fromMinutes);
  const to = Math.min(window.endMinutes, Math.max(block.toMinutes, from + MIN_BLOCK_MINUTES));

  const width = 100 / block.lanes;

  return {
    top: ((from - window.startMinutes) / span) * 100,
    height: Math.max(0, ((to - from) / span) * 100),
    left: width * block.lane,
    width
  };
}

/** Where the current time sits in the band, or null when it is outside it. */
export function ratioInWindow(now: Date, window: TimeWindow): number | null {
  const minutes = minutesOfDay(now);
  if (minutes < window.startMinutes || minutes > window.endMinutes) return null;
  const span = Math.max(1, window.endMinutes - window.startMinutes);
  return (minutes - window.startMinutes) / span;
}

/**
 * Minutes between the first and last course of a day that are not spent in
 * class. Overlapping courses are merged first, so a doubled slot does not
 * invent free time.
 */
export function gapMinutes(events: CalendarEvent[]): number {
  const intervals = events
    .map(eventMinutes)
    .sort((left, right) => left.fromMinutes - right.fromMinutes);

  if (intervals.length < 2) return 0;

  let busy = 0;
  let mergedFrom = intervals[0].fromMinutes;
  let mergedTo = intervals[0].toMinutes;

  for (const interval of intervals.slice(1)) {
    if (interval.fromMinutes <= mergedTo) {
      mergedTo = Math.max(mergedTo, interval.toMinutes);
      continue;
    }
    busy += mergedTo - mergedFrom;
    mergedFrom = interval.fromMinutes;
    mergedTo = interval.toMinutes;
  }
  busy += mergedTo - mergedFrom;

  const dayStart = intervals[0].fromMinutes;
  const dayEnd = intervals.reduce((latest, item) => Math.max(latest, item.toMinutes), 0);

  return Math.max(0, dayEnd - dayStart - busy);
}
