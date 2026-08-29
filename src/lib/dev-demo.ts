import type { CalendarEvent, Grade, PortalPage } from '$lib/features/schedule/types';

export function isDemoMode(): boolean {
  if (typeof window === 'undefined') return false;
  return localStorage.getItem('betteraimaira_demo') === 'true';
}

export function toggleDemoMode(enable?: boolean) {
  if (typeof window === 'undefined') return;
  const next = enable !== undefined ? enable : !isDemoMode();
  localStorage.setItem('betteraimaira_demo', next ? 'true' : 'false');
  window.location.reload();
}

if (typeof window !== 'undefined') {
  (window as unknown as { __toggleDemo: typeof toggleDemoMode }).__toggleDemo = toggleDemoMode;
}

const getWeekEvents = (): CalendarEvent[] => {
  const now = new Date();
  const monday = new Date(now);
  const dayOfWeek = (monday.getDay() + 6) % 7; // 0 for Monday
  monday.setDate(monday.getDate() - dayOfWeek);

  const makeDate = (dayOffset: number, hours: number, minutes: number) => {
    const d = new Date(monday);
    d.setDate(d.getDate() + dayOffset);
    d.setHours(hours, minutes, 0, 0);
    return d.toISOString();
  };

  const inProgressStart = new Date(now.getTime() - 45 * 60_000).toISOString();
  const inProgressEnd = new Date(now.getTime() + 75 * 60_000).toISOString();

  const events: CalendarEvent[] = [];

  // 1. Today's events with one in progress
  events.push(
    {
      id: 'evt-today-1',
      startsAt: makeDate(dayOfWeek, 8, 30),
      endsAt: makeDate(dayOfWeek, 10, 30),
      planification: 'Algorithmique Avancée & Graphes (CM)\nPr. Michel Dubois\nAmphi Turing (Campus Est)',
      description: 'Algorithmique Avancée & Graphes (CM)\nPr. Michel Dubois\nAmphi Turing (Campus Est)',
      kind: 'CM',
      externalComment: '',
      tempoUrl: 'https://tempo.example.com/session/101'
    },
    {
      id: 'evt-today-2',
      startsAt: inProgressStart,
      endsAt: inProgressEnd,
      planification: 'Intelligence Artificielle & Deep Learning (Cours)\nDr. Elena Vance\nSalle B204 - Labo IA (Campus Est)',
      description: 'Intelligence Artificielle & Deep Learning (Cours)\nDr. Elena Vance\nSalle B204 - Labo IA (Campus Est)',
      kind: 'CM',
      externalComment: '',
      tempoUrl: 'https://tempo.example.com/session/102'
    },
    {
      id: 'evt-today-3',
      startsAt: makeDate(dayOfWeek, 14, 0),
      endsAt: makeDate(dayOfWeek, 16, 30),
      planification: 'Systèmes Distribués & Cloud (TD)\nDr. Sarah Chen\nLabo Réseaux 302 (Campus Est)',
      description: 'Systèmes Distribués & Cloud (TD)\nDr. Sarah Chen\nLabo Réseaux 302 (Campus Est)',
      kind: 'TD',
      externalComment: '',
      tempoUrl: 'https://tempo.example.com/session/103'
    },
    {
      id: 'evt-today-4',
      startsAt: makeDate(dayOfWeek, 16, 45),
      endsAt: makeDate(dayOfWeek, 18, 45),
      planification: 'Sécurité & Cryptographie Moderne (Atelier)\nPr. Antoine Girard\nSalle C105 (Campus Est)',
      description: 'Sécurité & Cryptographie Moderne (Atelier)\nPr. Antoine Girard\nSalle C105 (Campus Est)',
      kind: 'TP',
      externalComment: '',
      tempoUrl: 'https://tempo.example.com/session/104'
    }
  );

  // 2. Add full week days
  const weekScheduleTemplate = [
    // Monday
    [
      { h1: 8, m1: 30, h2: 10, m2: 30, title: 'Algorithmique Avancée & Graphes (CM)\nPr. Michel Dubois\nAmphi Turing', kind: 'CM' },
      { h1: 10, m1: 45, h2: 12, m2: 45, title: 'Algorithmique Avancée (TD)\nPr. Michel Dubois\nSalle B204', kind: 'TD' },
      { h1: 14, m1: 0, h2: 17, m2: 0, title: 'Systèmes Distribués & Cloud (TP)\nDr. Sarah Chen\nLabo Réseaux 302', kind: 'TP' }
    ],
    // Tuesday
    [
      { h1: 9, m1: 0, h2: 12, m2: 0, title: 'Intelligence Artificielle & Deep Learning (CM)\nDr. Elena Vance\nAmphi Lovelace', kind: 'CM' },
      { h1: 13, m1: 30, h2: 15, m2: 30, title: 'Sécurité des Systèmes (Cours)\nPr. Antoine Girard\nSalle A102', kind: 'CM' },
      { h1: 15, m1: 45, h2: 18, m2: 15, title: 'Projet Transversal & Hackathon (Atelier)\nÉquipe Pédagogique\nAtelier Innovation 4', kind: 'Projet' }
    ],
    // Wednesday
    [
      { h1: 8, m1: 30, h2: 11, m2: 30, title: 'Génie Logiciel & DevOps (TP)\nMme Claire Bernard\nLabo Info 105', kind: 'TP' },
      { h1: 13, m1: 30, h2: 16, m2: 30, title: 'Base de Données NoSQL & Big Data (CM)\nDr. Karim Leroy\nAmphi Turing', kind: 'CM' }
    ],
    // Thursday
    [
      { h1: 9, m1: 0, h2: 12, m2: 0, title: 'Cloud Computing & Kubernetes (TD)\nDr. Henri Moreau\nSalle C301', kind: 'TD' },
      { h1: 14, m1: 0, h2: 17, m2: 0, title: 'Anglais Professionnel & Pitch (Séminaire)\nMme Jennifer Smith\nSalle Langues 12', kind: 'Langues' }
    ],
    // Friday
    [
      { h1: 8, m1: 30, h2: 11, m2: 30, title: 'Architecture Microservices (TP)\nPr. Michel Dubois\nLabo Réseaux 302', kind: 'TP' },
      { h1: 13, m1: 30, h2: 15, m2: 30, title: 'Éthique & Droit du Numérique (CM)\nMe. François Bernard\nAmphi Lovelace', kind: 'CM' },
      { h1: 16, m1: 0, h2: 18, m2: 0, title: 'Conférence Tech & IA (Séminaire)\nInvité Industrie Tech\nGrand Auditorium', kind: 'Conf' }
    ]
  ];

  let idCount = 10;
  weekScheduleTemplate.forEach((dayCourses, dIdx) => {
    if (dIdx === dayOfWeek) return; // Skip today, already added above
    dayCourses.forEach((c) => {
      events.push({
        id: `evt-w-${dIdx}-${idCount++}`,
        startsAt: makeDate(dIdx, c.h1, c.m1),
        endsAt: makeDate(dIdx, c.h2, c.m2),
        planification: c.title,
        description: c.title,
        kind: c.kind,
        externalComment: '',
        tempoUrl: null
      });
    });
  });

  return events;
};

const mockGrades: Grade[] = [
  {
    id: 'g1',
    subject: 'Architecture Logicielle & Microservices',
    label: 'Projet Backend & CI/CD',
    score: '18.5',
    scale: '/20',
    coefficient: '3.0',
    average: '14.2'
  },
  {
    id: 'g2',
    subject: 'Intelligence Artificielle & Deep Learning',
    label: 'Examen Final - CNN & Transformers',
    score: '17.0',
    scale: '/20',
    coefficient: '4.0',
    average: '13.8'
  },
  {
    id: 'g3',
    subject: 'Sécurité des Systèmes & Cryptographie',
    label: 'Contrôle Continu 2',
    score: '16.0',
    scale: '/20',
    coefficient: '3.5',
    average: '12.9'
  },
  {
    id: 'g4',
    subject: 'Systèmes Distribués & Consensus',
    label: 'TP Noté Raft & Paxos',
    score: '19.0',
    scale: '/20',
    coefficient: '3.0',
    average: '14.5'
  },
  {
    id: 'g5',
    subject: 'Cloud Computing & Kubernetes',
    label: 'Déploiement Helm & Terraform',
    score: '16.5',
    scale: '/20',
    coefficient: '2.5',
    average: '13.6'
  }
];

const mockPortalGradesPage: PortalPage = {
  resource: 'grades',
  fetchedAt: Date.now(),
  title: 'Bulletins de notes',
  headings: ['Année 2025-2026 - Semestre 8'],
  tables: [],
  fields: [],
  documents: [
    {
      kind: 'gradeBulletin',
      label: 'Bulletin officiel Semestre 8.pdf',
      requestPath: '/docs/s8.pdf',
      suggestedFilename: 'Bulletin_S8.pdf'
    },
    {
      kind: 'schoolCertificate',
      label: 'Certificat de scolarité 2025-2026.pdf',
      requestPath: '/docs/certif.pdf',
      suggestedFilename: 'Certificat_Scolarite_2025_2026.pdf'
    }
  ],
  gradePeriods: [
    {
      id: 'p2025-2026',
      label: '2025 - 2026 (Semestre 8)',
      blocks: [
        {
          id: 'b1',
          label: 'Bloc 1 : Ingénierie Logicielle & Systèmes Distribués',
          bulletinPath: '/docs/bulletin-bloc1.pdf',
          transcriptPath: null,
          sections: [
            {
              label: 'Enseignements fondamentaux',
              courses: [
                {
                  id: 'c1',
                  code: 'INF801',
                  name: 'Architecture Logicielle & Microservices',
                  notebookPath: null,
                  evaluations: [
                    { label: 'Projet Backend & CI/CD', score: '18.5', scale: '/20', weight: '50%', children: [] },
                    { label: 'Partiel Écrit', score: '16.0', scale: '/20', weight: '50%', children: [] }
                  ]
                },
                {
                  id: 'c2',
                  code: 'INF802',
                  name: 'Systèmes Distribués & Consensus',
                  notebookPath: null,
                  evaluations: [
                    { label: 'TP Noté Raft & Paxos', score: '19.0', scale: '/20', weight: '40%', children: [] },
                    { label: 'Examen Final', score: '16.5', scale: '/20', weight: '60%', children: [] }
                  ]
                },
                {
                  id: 'c3',
                  code: 'INF803',
                  name: 'Cloud Computing & Kubernetes',
                  notebookPath: null,
                  evaluations: [
                    { label: 'TP Terraform & Helm', score: '16.5', scale: '/20', weight: '100%', children: [] }
                  ]
                }
              ]
            }
          ]
        },
        {
          id: 'b2',
          label: 'Bloc 2 : Data Science & Intelligence Artificielle',
          bulletinPath: null,
          transcriptPath: null,
          sections: [
            {
              label: 'Machine Learning & Données',
              courses: [
                {
                  id: 'c4',
                  code: 'DAT801',
                  name: 'Intelligence Artificielle & Deep Learning',
                  notebookPath: null,
                  evaluations: [
                    { label: 'Examen Final - CNN & Transformers', score: '17.0', scale: '/20', weight: '60%', children: [] },
                    { label: 'Mini-Projet PyTorch', score: '18.0', scale: '/20', weight: '40%', children: [] }
                  ]
                },
                {
                  id: 'c5',
                  code: 'DAT802',
                  name: 'Bases de Données NoSQL & Big Data',
                  notebookPath: null,
                  evaluations: [
                    { label: 'Contrôle Continu', score: '17.5', scale: '/20', weight: '100%', children: [] }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  ],
  absencePeriods: [],
  questionnaires: [],
  markupRecognized: true,
  stale: false
};

const mockAbsencesPage: PortalPage = {
  resource: 'absences',
  fetchedAt: Date.now(),
  title: 'Assiduité et absences',
  headings: ['Année 2025-2026'],
  tables: [],
  fields: [],
  documents: [
    {
      kind: 'absenceReport',
      label: 'Relevé des absences S8.pdf',
      requestPath: '/docs/absences-s8.pdf',
      suggestedFilename: 'Releve_Absences_S8.pdf'
    }
  ],
  gradePeriods: [],
  absencePeriods: [
    {
      id: 'abs-2025-2026',
      label: '2025 - 2026 (Semestre 8)',
      blocks: [
        {
          id: 'abs-b1',
          label: 'Semestre 8',
          reportPath: '/docs/absences-s8.pdf',
          entries: [
            {
              id: 'abs-1',
              date: '14/02/2026',
              time: '08:30 - 10:30',
              course: 'Algorithmique Avancée',
              duration: '2h00',
              excused: true,
              reason: 'Certificat médical transmis'
            }
          ]
        }
      ]
    }
  ],
  questionnaires: [],
  markupRecognized: true,
  stale: false
};

export function getDemoResponse(cmd: string, args?: Record<string, unknown>): unknown {
  if (cmd === 'normalize_portal_url') {
    return { portalUrl: 'https://ecole-ingenieurs.myintranet.online' };
  }
  const demoSession = {
    portalUrl: 'https://ecole-ingenieurs.myintranet.online',
    username: 'alexandre.martin@ecole-ingenieurs.fr',
    credentialsSaved: true,
    sundaysVisible: false
  };
  if (cmd === 'saved_identity') {
    return {
      portalUrl: demoSession.portalUrl,
      username: demoSession.username,
      hasSnapshots: false
    };
  }
  if (cmd === 'login') {
    return demoSession;
  }
  if (cmd === 'restore_session') {
    return {
      status: 'restored',
      session: demoSession,
      identity: {
        portalUrl: demoSession.portalUrl,
        username: demoSession.username,
        hasSnapshots: false
      }
    };
  }
  // The update flow, so the notice and the card can be driven in a browser.
  if (cmd === 'default_update_channel') {
    return 'beta';
  }
  if (cmd === 'check_for_update') {
    return {
      available: true,
      currentVersion: '0.1.1-beta.8',
      latestVersion: '0.1.1-beta.9',
      notes: 'Demo release notes.',
      publishedAt: new Date().toISOString(),
      delivery: 'inApp',
      channel: args?.channel ?? 'beta',
      downloadUrl: null,
      storeUrl: null
    };
  }
  if (cmd === 'install_update') {
    return { handedOff: true, permissionRequired: false };
  }
  if (cmd === 'get_planning_settings') {
    return { sundaysVisible: false };
  }
  if (cmd === 'get_schedule') {
    return {
      events: getWeekEvents(),
      fetchedAt: Date.now(),
      stale: false
    };
  }
  if (cmd === 'sync_grades') {
    return {
      grades: mockGrades,
      stale: false
    };
  }
  if (cmd === 'get_portal_resource') {
    const resource = args?.resource;
    if (resource === 'grades') return mockPortalGradesPage;
    if (resource === 'absences') return mockAbsencesPage;
    return {
      resource: resource || 'grades',
      fetchedAt: Date.now(),
      title: 'Portail Étudiant',
      headings: [],
      tables: [],
      fields: [],
      documents: [],
      gradePeriods: [],
      absencePeriods: [],
      questionnaires: [],
      markupRecognized: true,
      stale: false
    };
  }
  return undefined;
}
