import { SvelteDate } from 'svelte/reactivity';

export function startOfDay(date: Date): Date {
  const result = new SvelteDate(date);
  result.setHours(0, 0, 0, 0);
  return result;
}

export function startOfWeek(date: Date): Date {
  const result = startOfDay(date);
  const mondayOffset = (result.getDay() + 6) % 7;
  result.setDate(result.getDate() - mondayOffset);
  return startOfDay(result);
}

export function startOfMonth(date: Date): Date {
  const result = startOfDay(date);
  result.setDate(1);
  return result;
}

export function addDays(date: Date, days: number): Date {
  const result = new SvelteDate(date);
  result.setDate(result.getDate() + days);
  return startOfDay(result);
}

export function addMonths(date: Date, months: number): Date {
  const result = new SvelteDate(date);
  result.setMonth(result.getMonth() + months);
  return startOfDay(result);
}

export function isSameDay(left: Date, right: Date): boolean {
  return (
    left.getFullYear() === right.getFullYear() &&
    left.getMonth() === right.getMonth() &&
    left.getDate() === right.getDate()
  );
}

export function isSameWeek(left: Date, right: Date): boolean {
  return isSameDay(startOfWeek(left), startOfWeek(right));
}

export function isSameMonth(left: Date, right: Date): boolean {
  return (
    left.getFullYear() === right.getFullYear() &&
    left.getMonth() === right.getMonth()
  );
}

export function dayKey(date: Date): string {
  return `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`;
}

export function getWeekNumber(date: Date): number {
  const target = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
  const dayNum = target.getUTCDay() || 7;
  target.setUTCDate(target.getUTCDate() + 4 - dayNum);
  const yearStart = new Date(Date.UTC(target.getUTCFullYear(), 0, 1));
  return Math.ceil(((target.getTime() - yearStart.getTime()) / 86_400_000 + 1) / 7);
}

export function capitalizeFirst(text: string): string {
  if (!text) return '';
  return text.charAt(0).toUpperCase() + text.slice(1);
}
