export type CalendarEvent = {
  id: string;
  startsAt: string;
  endsAt: string;
  planification: string;
  description: string;
  kind: string;
  externalComment: string;
  tempoUrl: string | null;
};

export type ScheduleResult = {
  events: CalendarEvent[];
  fetchedAt: number;
  /**
   * The portal could not be reached and this came off the local snapshot.
   * `fetchedAt` is then the age of that snapshot, not the age of this call, so
   * the freshness label states when the data was really read.
   */
  stale: boolean;
};

export type ScheduleErrorCode =
  | 'session_expired'
  | 'planning_unavailable'
  | 'planning_invalid_response'
  | 'invalid_schedule_range'
  | 'internal_error';

export type ScheduleState =
  | { kind: 'loading' }
  | { kind: 'ready'; events: CalendarEvent[]; fetchedAt: number; cacheKey: string; stale: boolean }
  | { kind: 'error'; code: ScheduleErrorCode };

export type Grade = {
  id: string;
  subject: string;
  label: string;
  score: string;
  scale: string | null;
  coefficient: string | null;
  average: string | null;
};

export type GradeSyncResult = {
  grades: Grade[];
  /** Served from the stored snapshot because the portal was unreachable. */
  stale: boolean;
};

export type GradeSyncErrorCode =
  | 'session_expired'
  | 'grades_unavailable'
  | 'grades_invalid_response'
  | 'grade_storage_unavailable'
  | 'internal_error';

export type PortalResource = 'grades' | 'absences' | 'profile' | 'documents' | 'questionnaires';

export type PortalTable = {
  context: string[];
  caption: string | null;
  headers: string[];
  rows: string[][];
};

export type PortalField = {
  label: string;
  value: string;
};

export type PortalDocumentKind =
  | 'absenceReport'
  | 'gradeBulletin'
  | 'gradeTranscript'
  | 'enrollmentCertificate'
  | 'schoolCertificate'
  | 'schoolTranscript'
  | 'gradeReport';

export type PortalDocument = {
  kind: PortalDocumentKind;
  label: string;
  requestPath: string;
  suggestedFilename: string | null;
};

export type DocumentDownloadResult = {
  /** Absolute path of the saved file, so the view can say where it went. */
  path: string;
  /** False when the file is on disk but the system refused to display it. */
  opened: boolean;
};

export type GradeEvaluation = {
  label: string;
  score: string | null;
  scale: string | null;
  /** Portal wording, kept raw: `50,00%` for a weighting, `Pondération : 20,00` for a sub-evaluation. */
  weight: string | null;
  children: GradeEvaluation[];
};

export type GradeCourse = {
  id: string;
  code: string | null;
  name: string;
  notebookPath: string | null;
  evaluations: GradeEvaluation[];
};

export type GradeSection = {
  label: string | null;
  courses: GradeCourse[];
};

export type GradeBlock = {
  id: string;
  label: string;
  bulletinPath: string | null;
  transcriptPath: string | null;
  sections: GradeSection[];
};

export type GradePeriod = {
  id: string;
  label: string;
  blocks: GradeBlock[];
};

export type AbsenceEntry = {
  id: string;
  date: string;
  /** The portal packs the start time into the date cell; the backend splits it out. */
  time: string | null;
  course: string;
  /** Raw portal wording: `3,25` counts hours, `1h30` does not. */
  duration: string | null;
  /** `null` while the school has not ruled on the justification yet. */
  excused: boolean | null;
  reason: string | null;
};

export type AbsenceBlock = {
  id: string;
  label: string;
  reportPath: string | null;
  entries: AbsenceEntry[];
};

export type AbsencePeriod = {
  id: string;
  label: string;
  blocks: AbsenceBlock[];
};

export type QuestionnaireSummary = {
  id: string;
  title: string;
  context: string;
  deadline: string | null;
  status: string;
  completed: boolean;
  responsePath: string;
};

export type QuestionnaireOption = {
  value: string;
  label: string;
};

export type QuestionnaireQuestion = {
  id: string;
  kind: string;
  title: string;
  description: string | null;
  required: boolean;
  options: QuestionnaireOption[];
  answers: string[];
};

export type QuestionnairePage = {
  id: string;
  title: string | null;
  questions: QuestionnaireQuestion[];
};

export type QuestionnaireDetail = {
  title: string;
  completed: boolean;
  pages: QuestionnairePage[];
};

export type PortalPage = {
  resource: PortalResource;
  fetchedAt: number;
  title: string;
  headings: string[];
  tables: PortalTable[];
  fields: PortalField[];
  documents: PortalDocument[];
  /** Only filled for the grades page: year → block → course, as the portal groups them. */
  gradePeriods: GradePeriod[];
  /** Only filled for the absences page: year → block → missed sessions. */
  absencePeriods: AbsencePeriod[];
  /** Only filled for the questionnaires page. */
  questionnaires: QuestionnaireSummary[];
  markupRecognized: boolean;
  /**
   * The portal could not be reached and this page came off the local snapshot.
   * `fetchedAt` then dates the snapshot, which is what the freshness label states.
   */
  stale: boolean;
};

export type PortalResourceErrorCode =
  | 'session_expired'
  | 'grades_unavailable'
  | 'absences_unavailable'
  | 'profile_unavailable'
  | 'documents_unavailable'
  | 'questionnaires_unavailable'
  | 'questionnaire_invalid_response'
  | 'invalid_questionnaire_request'
  | 'internal_error';

export type PortalResourceState =
  | { kind: 'loading' }
  | { kind: 'ready'; page: PortalPage }
  | { kind: 'error'; code: PortalResourceErrorCode };

export type ScheduleView = 'today' | 'schedule' | 'grades' | 'absences' | 'more';
export type MoreSection = 'profile' | 'documents' | 'questionnaires';
export type CalendarScope = 'day' | 'week' | 'month';
