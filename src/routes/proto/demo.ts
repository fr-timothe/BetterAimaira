/**
 * DEV-ONLY PROTOTYPE DATA — not part of the shipped app.
 *
 * Authored demonstration courses shaped exactly like what the portal returns:
 * `planification` is the multi-line blob `parseCourseDetails` reads, `kind` is
 * the raw portal wording `courseCategory` maps onto a tone. No capture of a
 * real portal is in this repository and none may be added, so every name,
 * room and teacher below is invented.
 *
 * The whole `src/routes/proto/` tree exists to compare three mobile structures
 * for the Schedule view and is deleted once one is locked.
 */
import type { CalendarEvent } from '$lib/features/schedule/types';
import { addDays, startOfWeek } from '$lib/features/schedule/date-utils';

type Slot = {
  /** 0 = Monday of the reference week. */
  day: number;
  from: string;
  to: string;
  title: string;
  kind: string;
  teacher: string;
  room: string;
  /** Portal wording that makes `isCancelled` true. */
  cancelled?: string;
};

const CAMPUS = '(Campus Nord)';

/**
 * One realistic school week: a dense Monday, a light Wednesday, an exam, a
 * 45-minute slot, two overlapping courses on Thursday, and a free Saturday —
 * so every state a structure has to survive is reachable by navigating.
 */
const WEEK: Slot[] = [
  { day: 0, from: '08:00', to: '10:00', title: 'Analyse numérique', kind: 'CM', teacher: 'DUBREUIL Camille', room: 'Amphi Curie' },
  { day: 0, from: '10:15', to: '12:15', title: 'Réseaux et protocoles', kind: 'TD', teacher: 'NAVARRO Élise', room: 'Salle B204' },
  { day: 0, from: '13:30', to: '16:30', title: 'Systèmes embarqués', kind: 'TP', teacher: 'FONTAINE Marc', room: 'Labo E12' },
  { day: 0, from: '16:45', to: '17:30', title: 'Suivi de projet', kind: 'Projet', teacher: 'ROUSSEL Anaïs', room: 'Salle A110' },

  { day: 1, from: '09:00', to: '12:00', title: 'Probabilités appliquées', kind: 'CM', teacher: 'BERTRAND Youssef', room: 'Amphi Curie' },
  { day: 1, from: '14:00', to: '16:00', title: 'Anglais technique', kind: 'TD', teacher: 'HOLLAND Sarah', room: 'Salle C12' },

  { day: 2, from: '10:00', to: '12:00', title: 'Architecture logicielle', kind: 'CM', teacher: 'LEMAIRE Pierre', room: 'Amphi Fermat' },

  { day: 3, from: '08:00', to: '11:00', title: 'Traitement du signal', kind: 'TP', teacher: 'FONTAINE Marc', room: 'Labo E14' },
  { day: 3, from: '10:00', to: '11:30', title: 'Entretien de mi-parcours', kind: 'Projet', teacher: 'ROUSSEL Anaïs', room: 'Salle A110' },
  { day: 3, from: '13:00', to: '15:00', title: 'Bases de données', kind: 'TD', teacher: 'NAVARRO Élise', room: 'Salle B204' },
  { day: 3, from: '15:15', to: '17:15', title: 'Droit du numérique', kind: 'CM', teacher: 'MARCHAND Claire', room: 'Amphi Fermat', cancelled: 'Séance annulée : intervenante absente' },

  { day: 4, from: '08:30', to: '11:30', title: 'Partiel — Analyse numérique', kind: 'Examen', teacher: 'DUBREUIL Camille', room: 'Amphi Curie' },
  { day: 4, from: '14:00', to: '18:00', title: 'Atelier projet transverse', kind: 'Projet', teacher: 'ROUSSEL Anaïs', room: 'Labo E12' },
];

/** The portal never sends the same week twice; these shift the following one. */
const NEXT_WEEK_SHIFT: Slot[] = WEEK.filter((slot) => slot.day !== 3).map((slot) => ({
  ...slot,
  day: (slot.day + 1) % 6,
  cancelled: undefined,
}));

function at(weekStart: Date, dayOffset: number, time: string): string {
  const [hours, minutes] = time.split(':').map(Number);
  const date = addDays(weekStart, dayOffset);
  date.setHours(hours, minutes, 0, 0);
  return date.toISOString();
}

function toEvent(weekStart: Date, weekIndex: number, slot: Slot, index: number): CalendarEvent {
  const planification = [
    `${slot.title} (${slot.kind})`,
    slot.teacher,
    slot.room,
    CAMPUS,
  ].join('\n');

  return {
    id: `proto-${weekIndex}-${slot.day}-${index}`,
    startsAt: at(weekStart, slot.day, slot.from),
    endsAt: at(weekStart, slot.day, slot.to),
    planification,
    description: planification,
    kind: slot.kind,
    externalComment: slot.cancelled ?? '',
    tempoUrl: slot.kind === 'TP' ? 'https://example.invalid/tempo' : null,
  };
}

/**
 * Three weeks back and three forward, so moving the period always lands on
 * something. The reference week is the one containing `reference`.
 */
export function demoEvents(reference: Date): CalendarEvent[] {
  const base = startOfWeek(reference);
  const events: CalendarEvent[] = [];

  for (let weekIndex = -3; weekIndex <= 3; weekIndex += 1) {
    const weekStart = addDays(base, weekIndex * 7);
    const slots = weekIndex % 2 === 0 ? WEEK : NEXT_WEEK_SHIFT;
    slots.forEach((slot, index) => events.push(toEvent(weekStart, weekIndex, slot, index)));
  }

  return events;
}

/**
 * A plausible last-sync time: recent enough that `FreshnessLabel` reads fresh,
 * so the prototypes are compared on their structure rather than on a warning.
 */
export function demoFetchedAt(reference: Date): number {
  return reference.getTime() - 90_000;
}
