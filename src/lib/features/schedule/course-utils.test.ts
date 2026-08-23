import { describe, expect, it } from 'bun:test';
import {
  eventSecondary,
  eventTitle,
  parseCourseDetails,
  parseRoomAndTeacher,
} from './course-utils';
import type { CalendarEvent } from './types';

function createMockEvent(overrides: Partial<CalendarEvent>): CalendarEvent {
  return {
    id: '1',
    startsAt: '2026-08-17T09:00:00',
    endsAt: '2026-08-17T12:00:00',
    planification: '',
    description: '',
    kind: 'Cours',
    externalComment: '',
    tempoUrl: null,
    ...overrides,
  };
}

describe('course-utils parser', () => {
  it('parses standard Aimaira course with teacher, room and campus', () => {
    const event = createMockEvent({
      planification:
        'Team Bulding (Cours)\nDUCHEMIN Loïc\nSalle Ada LOVELACE\n(Campus Nord)',
    });

    const details = parseCourseDetails(event);
    expect(details.title).toBe('Team Bulding');
    expect(details.teacher).toBe('DUCHEMIN Loïc');
    expect(details.room).toBe('Salle Ada LOVELACE');
    expect(details.campus).toBe('Campus Nord');
    expect(details.modality).toBe('Cours');

    expect(eventTitle(event)).toBe('Team Bulding');
    const { teacher, room } = parseRoomAndTeacher(event);
    expect(teacher).toBe('DUCHEMIN Loïc');
    expect(room).toBe('Salle Ada LOVELACE');
  });

  it('correctly handles names containing letters like "dr" or "er" without false prefix match', () => {
    const event1 = createMockEvent({
      planification:
        'Projet Podcast (Cours)\nBERTHIER Adrien\nSalle Alan TURING (salle FabLab)\n(Campus Nord)',
    });
    const details1 = parseCourseDetails(event1);
    expect(details1.teacher).toBe('BERTHIER Adrien');

    const event2 = createMockEvent({
      planification: 'Wordpress (Cours)\nLARIVIÈRE Audrey',
    });
    const details2 = parseCourseDetails(event2);
    expect(details2.teacher).toBe('LARIVIÈRE Audrey');

    const event3 = createMockEvent({
      planification:
        'Commerce : fondamentaux (Cours)\nDELAVERGNE Christophe\nSalle Ada LOVELACE\n(Campus Nord)',
    });
    const details3 = parseCourseDetails(event3);
    expect(details3.teacher).toBe('DELAVERGNE Christophe');

    const event4 = createMockEvent({
      planification:
        'Semaine Humanitaire (Cours)\nVan Dermeulen Emilie\nSalle Ada LOVELACE\n(Campus Nord)',
    });
    const details4 = parseCourseDetails(event4);
    expect(details4.teacher).toBe('Van Dermeulen Emilie');
  });

  it('parses room codes such as HG09, SA106, QG01', () => {
    const event = createMockEvent({
      planification: 'Lancement LXP ()\nCOURTEMANCHE Marine\nHG09\n(Campus International)',
    });
    const details = parseCourseDetails(event);
    expect(details.title).toBe('Lancement LXP');
    expect(details.teacher).toBe('COURTEMANCHE Marine');
    expect(details.room).toBe('HG09');
    expect(details.campus).toBe('Campus International');
  });

  it('handles nested parentheses in subject names correctly', () => {
    const event = createMockEvent({
      planification: 'PEA (Programme Extra académique) ()\nDoran Maria',
    });
    const details = parseCourseDetails(event);
    expect(details.title).toBe('PEA (Programme Extra académique)');
    expect(details.teacher).toBe('Doran Maria');
    expect(details.room).toBeNull();
  });

  it('handles events with no teacher', () => {
    const event = createMockEvent({
      planification:
        'Réunion de rentrée pédagogique ()\nSalle Ada LOVELACE\n(Campus Nord)',
    });
    const details = parseCourseDetails(event);
    expect(details.title).toBe('Réunion de rentrée pédagogique');
    expect(details.teacher).toBeNull();
    expect(details.room).toBe('Salle Ada LOVELACE');
  });

  it('handles online and autonomy modalities in secondary label', () => {
    const onlineEvent = createMockEvent({
      planification:
        'Veille et étude de marché (EN LIGNE)\nBOISSEAU Marc\nSalle Blaise PASCAL\n(Campus Nord)',
    });
    expect(eventSecondary(onlineEvent)).toBe('EN LIGNE');

    const autonomyEvent = createMockEvent({
      planification: 'Projet Podcast (Autonomie)',
    });
    expect(eventSecondary(autonomyEvent)).toBe('Autonomie');
  });

  it('parses raw HTML with <br /> tags properly', () => {
    const event = createMockEvent({
      planification:
        'Team Bulding (Cours)\r\n<br />\r\nDUCHEMIN Loïc <br />\r\nSalle Ada LOVELACE\r\n (Campus Nord) \r\n',
    });
    const details = parseCourseDetails(event);
    expect(details.title).toBe('Team Bulding');
    expect(details.teacher).toBe('DUCHEMIN Loïc');
    expect(details.room).toBe('Salle Ada LOVELACE');
    expect(details.campus).toBe('Campus Nord');
  });

  it('parses flattened single-line strings gracefully as a fallback', () => {
    const event = createMockEvent({
      planification:
        'Team Bulding (Cours) DUCHEMIN Loïc Salle Ada LOVELACE (Campus Nord)',
    });
    const details = parseCourseDetails(event);
    expect(details.title).toBe('Team Bulding');
    expect(details.teacher).toBe('DUCHEMIN Loïc');
    expect(details.room).toBe('Salle Ada LOVELACE');
    expect(details.campus).toBe('Campus Nord');
  });

  it('parses explicit teacher prefixes (intervenant, prof, M., etc.)', () => {
    const event1 = createMockEvent({
      planification: 'Physique-Chimie\nIntervenant : M. Dupont\nSalle B12',
    });
    const details1 = parseCourseDetails(event1);
    expect(details1.title).toBe('Physique-Chimie');
    expect(details1.teacher).toBe('M. Dupont');
    expect(details1.room).toBe('Salle B12');

    const event2 = createMockEvent({
      planification: 'Mathématiques\nProfesseur : Marie Curie\nAmphi A',
    });
    const details2 = parseCourseDetails(event2);
    expect(details2.title).toBe('Mathématiques');
    expect(details2.teacher).toBe('Marie Curie');
    expect(details2.room).toBe('Amphi A');
  });

  it('handles empty planification with fallback', () => {
    const event = createMockEvent({
      planification: '',
      description: '',
      kind: 'Examen',
    });
    const details = parseCourseDetails(event);
    expect(details.title).toBe('Examen');
    expect(details.teacher).toBeNull();
    expect(details.room).toBeNull();
    expect(eventTitle(event, 'Fallback')).toBe('Examen');
  });
});
