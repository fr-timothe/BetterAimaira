import { openUrl } from '@tauri-apps/plugin-opener';
import * as m from '$lib/paraglide/messages.js';
import type { CalendarEvent } from './types';

export function eventStart(event: CalendarEvent): Date {
  return new Date(event.startsAt);
}

export function eventEnd(event: CalendarEvent): Date {
  return new Date(event.endsAt);
}

export function eventDurationMinutes(event: CalendarEvent): number {
  return Math.max(0, Math.round((eventEnd(event).getTime() - eventStart(event).getTime()) / 60_000));
}

/**
 * `locale` is not read here — the wording comes from the message catalogue,
 * which resolves the active locale on its own. It stays in the signature so a
 * caller's `$derived` keeps a dependency on the locale and re-runs when it
 * changes; Paraglide message functions are not reactive by themselves.
 */
export function formatDuration(minutes: number, locale = 'fr'): string {
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  if (hours === 0) return `${mins} min`;
  if (mins === 0) {
    return hours > 1
      ? m.duration_hour_other({ count: hours })
      : m.duration_hour_one({ count: hours });
  }
  return `${hours}h${mins.toString().padStart(2, '0')}`;
}

export function formatDurationRange(startsAt: string, endsAt: string): string {
  const start = new Date(startsAt).getTime();
  const end = new Date(endsAt).getTime();
  const diffMinutes = Math.max(0, Math.round((end - start) / 60_000));
  const hours = Math.floor(diffMinutes / 60);
  const minutes = diffMinutes % 60;
  if (hours > 0 && minutes > 0) {
    return `${hours}h${minutes.toString().padStart(2, '0')}`;
  }
  if (hours > 0) return `${hours}h`;
  return `${minutes}m`;
}

export type ParsedCourseDetails = {
  title: string;
  modality: string | null;
  teacher: string | null;
  room: string | null;
  campus: string | null;
};

export function parseCourseDetails(event: CalendarEvent): ParsedCourseDetails {
  const rawSources = [event.planification, event.description, event.externalComment].filter(
    (item): item is string => Boolean(item && item.trim())
  );

  let title = '';
  let modality: string | null = null;
  let teacher: string | null = null;
  let room: string | null = null;
  let campus: string | null = null;

  // 1. Gather all non-empty lines across planification, description, externalComment
  const allLines: string[] = [];
  for (const source of rawSources) {
    const cleaned = source
      .replace(/<br\s*\/?>/gi, '\n')
      .replace(/<\/?p[^>]*>/gi, '\n')
      .replace(/<\/?div[^>]*>/gi, '\n')
      .replace(/&nbsp;/gi, ' ')
      .replace(/&amp;/gi, '&')
      .replace(/<[^>]*>/g, '');

    const lines = cleaned
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    for (const line of lines) {
      if (!allLines.includes(line)) {
        allLines.push(line);
      }
    }
  }

  if (allLines.length > 1) {
    // Multiline format
    const firstLine = allLines[0];
    const parenMatch = firstLine.match(/^(.*?)(?:\s*\(([^()]*)\))$/);
    if (parenMatch) {
      title = parenMatch[1]?.trim() || firstLine;
      const mod = parenMatch[2]?.trim();
      if (mod) modality = mod;
    } else {
      title = firstLine;
    }

    const remainingLines = allLines.slice(1);
    for (const line of remainingLines) {
      // 1. Campus line check
      if (
        line.match(/^\(?campus/i) ||
        line.includes('Campus') ||
        (line.startsWith('(') && line.endsWith(')'))
      ) {
        if (!campus) {
          campus = line.replace(/^\(|\)$/g, '').trim();
        }
        continue;
      }

      // 2. Room line check
      if (
        line.match(/^(?:salle|amphi|labo|bât(?:iment)?\.?|bat\.?|room)\b/i) ||
        line.match(/^[A-Z]{1,3}\s*[0-9]{2,4}[a-z]?$/i)
      ) {
        if (!room) {
          room = line;
        }
        continue;
      }

      // 3. Explicit teacher prefix check
      const prefixMatch = line.match(
        /\b(?:intervenant|enseignant|prof(?:esseur)?|M\.|Mme|Mr|Mrs|Pr|Dr)\b\.?\s*[:\s]\s*(.*)/i
      );
      if (prefixMatch) {
        if (!teacher) {
          teacher = prefixMatch[1].trim() || line;
        }
        continue;
      }

      // 4. Unprefixed teacher name
      if (!teacher && !line.match(/^(?:salle|amphi|bât|campus)/i)) {
        teacher = line;
        continue;
      }

      // Fallback room line
      if (!room && (line.toLowerCase().includes('salle') || line.toLowerCase().includes('amphi') || line.toLowerCase().includes('labo'))) {
        room = line;
      }
    }
  } else if (allLines.length === 1) {
    // Single-line format: e.g. "Team Bulding (Cours) DUCHEMIN Loïc Salle Ada LOVELACE (Campus Nord)"
    let text = allLines[0];

    // Extract Campus
    const campusMatch = text.match(/\(([^)]*\bCampus\b[^)]*)\)/i);
    if (campusMatch) {
      campus = campusMatch[0].replace(/^\(|\)$/g, '').trim();
      text = text.replace(campusMatch[0], ' ').trim();
    }

    // Extract Room
    const roomMatch =
      text.match(/(?:salle|amphi|bât(?:iment)?\.?|bat\.?|room|labo)\s+[^(\n]+(?:\([^)]+\))?/i) ||
      text.match(/\b(?:Amphi\s+[A-Z0-9]+|[A-Z]{1,3}\s*[0-9]{2,4}[a-z]?)\b/i);
    if (roomMatch) {
      room = roomMatch[0].trim();
      text = text.replace(roomMatch[0], ' ').trim();
    }

    // Extract Title, Modality, and leftover Teacher
    const titleModalityMatch = text.match(/^(.*?)(?:\s*\(([^()]*)\))(?:\s+(.*))?$/);
    if (titleModalityMatch) {
      title = titleModalityMatch[1].trim();
      const mod = titleModalityMatch[2].trim();
      if (mod) modality = mod;
      const leftover = titleModalityMatch[3]?.trim();
      if (leftover && !teacher) {
        teacher = leftover;
      }
    } else {
      title = text;
    }
  }

  // Fallback for missing fields
  if (!teacher || !room || !campus) {
    for (const text of rawSources) {
      if (!campus) {
        const campusMatch = text.match(/\(([^)]*\bCampus\b[^)]*)\)/i);
        if (campusMatch) campus = campusMatch[0].replace(/^\(|\)$/g, '').trim();
      }

      if (!room) {
        const roomMatch =
          text.match(/(?:salle|amphi|bât(?:iment)?\.?|bat\.?|room|labo)\s*[:\s]?\s*([^\n,·;-]+(?:\([^)]+\))?)/i) ||
          text.match(/\b(?:Amphi\s+[A-Z0-9]+|[A-Z]{1,3}\s*[0-9]{2,4}[a-z]?)\b/i);
        if (roomMatch) {
          room = roomMatch[0].trim();
        }
      }

      if (!teacher) {
        const prefixMatch = text.match(
          /\b(?:intervenant|enseignant|prof(?:esseur)?|M\.|Mme|Mr|Mrs|Pr|Dr)\b\.?\s*[:\s]\s*([A-Za-zÀ-ÿ\-]+(?:\s+[A-Za-zÀ-ÿ\.-]+)?)/i
        );
        if (prefixMatch) {
          teacher = prefixMatch[1]?.trim() || prefixMatch[0].trim();
        }
      }
    }
  }

  if (!title) {
    title = event.planification || event.description || event.kind || 'Cours sans titre';
  }

  if (title && (title.includes('Salle ') || title.includes('Campus') || (room && title.includes(room)))) {
    const titleMatch = title.match(/^(.*?)(?:\s*\(([^()]*)\))/);
    if (titleMatch) {
      const extractedTitle = titleMatch[1].trim();
      const mod = titleMatch[2].trim();
      if (mod && !modality) modality = mod;
      title = extractedTitle;
    }
  }

  return {
    title: title || 'Cours sans titre',
    modality,
    teacher,
    room,
    campus,
  };
}

export function eventTitle(event: CalendarEvent, fallback = 'Cours sans titre'): string {
  const details = parseCourseDetails(event);
  return details.title || fallback;
}

export function eventSecondary(event: CalendarEvent): string | null {
  const details = parseCourseDetails(event);
  if (details.modality && details.modality.length > 0 && details.modality.toLowerCase() !== 'cours') {
    return details.modality;
  }
  if (event.planification && event.description && event.description !== event.planification) {
    const descDetails = parseCourseDetails({ ...event, planification: event.description });
    if (descDetails.title && descDetails.title !== details.title) {
      return descDetails.title;
    }
  }
  return details.campus || event.kind || null;
}

export function parseRoomAndTeacher(event: CalendarEvent): {
  room: string | null;
  teacher: string | null;
  campus?: string | null;
} {
  const details = parseCourseDetails(event);
  return {
    room: details.room,
    teacher: details.teacher,
    campus: details.campus,
  };
}

/**
 * The `includes()` tests match the French labels the portal sends, so they stay
 * untranslated; only the returned label goes through the catalogue. `locale`
 * follows the same convention as `formatDuration`: unused here, kept so a
 * caller's `$derived` re-runs on a locale change.
 */
export function courseTypeBadge(event: CalendarEvent, locale = 'fr'): string {
  if (!event.kind) return m.course_type_class();
  const k = event.kind.trim();
  const lower = k.toLowerCase();
  if (lower.includes('travail dirigé') || k.toUpperCase() === 'TD') return 'TD';
  if (lower.includes('cours magistral') || k.toUpperCase() === 'CM') return 'CM';
  if (lower.includes('travaux pratiques') || k.toUpperCase() === 'TP') return 'TP';
  if (lower.includes('projet')) return m.course_type_project();
  if (lower.includes('examen') || lower.includes('évaluation') || lower.includes('partiel')) {
    return m.course_type_exam();
  }
  return k;
}

export type CourseCategory = 'lecture' | 'tutorial' | 'lab' | 'exam' | 'project' | 'other';

/**
 * Maps a portal `kind` string onto one of six categories. The colours live in
 * `--category-*` tokens and are applied by `KindBadge`, so a course looks the
 * same in every view.
 */
export function courseCategory(kind?: string | null): CourseCategory {
  const k = (kind || '').toLowerCase();
  if (k.includes('cm') || k.includes('magistral')) return 'lecture';
  if (k.includes('td') || k.includes('dirigé')) return 'tutorial';
  if (k.includes('tp') || k.includes('pratique')) return 'lab';
  if (k.includes('exam') || k.includes('eval') || k.includes('partiel') || k.includes('cc')) {
    return 'exam';
  }
  if (k.includes('projet') || k.includes('conf') || k.includes('workshop')) return 'project';
  return 'other';
}

export function getEventStatus(event: CalendarEvent, now: Date = new Date()): 'live' | 'upcoming' | 'finished' {
  const start = eventStart(event).getTime();
  const end = eventEnd(event).getTime();
  const nowMs = now.getTime();
  if (start <= nowMs && nowMs < end) return 'live';
  if (start > nowMs) return 'upcoming';
  return 'finished';
}

export function isCancelled(event: CalendarEvent): boolean {
  const raw = `${event.externalComment} ${event.description} ${event.kind} ${event.planification}`.toLowerCase();
  return (
    raw.includes('annul') ||
    raw.includes('cancel') ||
    raw.includes('absent') ||
    raw.includes('supprim') ||
    raw.includes('report')
  );
}

/** `locale` is the reactivity anchor described on `formatDuration`. */
export function cancellationReason(event: CalendarEvent, locale = 'fr'): string {
  const comment = event.externalComment?.trim();
  if (comment) return comment;
  const raw = `${event.description} ${event.kind}`.toLowerCase();
  if (raw.includes('absent')) {
    return m.course_cancelled_teacher_absent();
  }
  return m.course_cancelled();
}

/** `locale` is the reactivity anchor described on `formatDuration`. */
export function relativeStartFromNow(event: CalendarEvent, now: Date, locale = 'fr'): string {
  const diffMs = eventStart(event).getTime() - now.getTime();
  const minutes = Math.ceil(diffMs / 60_000);
  if (minutes <= 0) return m.time_now();
  if (minutes < 60) return m.course_starts_in_minutes({ minutes });
  const hours = Math.floor(minutes / 60);
  const remMins = minutes % 60;
  if (remMins === 0) return m.course_starts_in_hours({ hours });
  return m.course_starts_in_hours_minutes({ hours, minutes: remMins });
}

export async function openExternalUrl(url: string | null): Promise<void> {
  if (!url) return;
  try {
    await openUrl(url);
  } catch {
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}
