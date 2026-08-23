[← Documentation index](README.md)

# Rust backend API

This document describes the client-facing contract exposed by the Tauri backend. The Rust process owns portal authentication, cookies, HTML parsing, URL validation, and document downloads. Client code must not call Aimaira routes directly or parse portal HTML.

## Command summary

| Command | Input | Output |
|---|---|---|
| `normalize_portal_url` | `{ portalUrl: string }` | `{ portalUrl: string }` |
| `login` | `{ request: LoginRequest }` | `LoginResult` |
| `restore_session` | none | `LoginResult \| null` |
| `logout` | none | `null` |
| `get_schedule` | `{ request: ScheduleRequest }` | `ScheduleResult` |
| `get_planning_settings` | none | `PlanningSettingsResult` |
| `get_portal_resource` | `{ resource: PortalResource, force?: boolean }` | `PortalPage` |
| `get_questionnaire_detail` | `{ request: { responsePath: string } }` | `QuestionnaireDetail` |
| `sync_grades` | `{ force?: boolean }` | `GradeSyncResult` |
| `mark_grade_alerts_read` | none | `null` |
| `download_portal_document` | `{ request: { requestPath: string } }` | binary PDF response |
| `check_for_update` | none | `UpdateInfo` |
| `install_update` | none | `InstallOutcome` |
| `update_feed_base` | none | `string` |

Every command in this table is reachable from the webview, so the surface stays
minimal on purpose: a command that no client calls is removed rather than left
exposed.

Every command error is serialized as `{ code: string }`. Clients should branch on `code`, never on transport diagnostics.

## Authentication

```ts
type LoginRequest = {
  portalUrl: string;
  username: string;
  password: string;
  remember: boolean;
};

type LoginResult = {
  portalUrl: string;
  username: string;
  credentialsSaved: boolean;
  sundaysVisible: boolean;
};

type PlanningSettingsResult = {
  sundaysVisible: boolean;
};
```

`get_planning_settings` always re-reads the settings from the portal and refreshes
the copy cached on the session, so there is no separate refresh command.

`restore_session` reads the last saved identity and password from the operating system credential store, then performs a normal portal login to create a fresh in-memory cookie jar. It returns `null` when no saved credentials exist or when they are no longer accepted by Aimaira. Cookies remain private to Rust. `logout` drops the in-memory cookie jar and removes the saved credentials.

## Schedule

```ts
type ScheduleRequest = {
  start: string; // ISO start instant
  duration: number; // integer from 1 through 42
};

type CalendarEvent = {
  id: string;
  startsAt: string;
  endsAt: string;
  planification: string;
  description: string;
  kind: string;
  externalComment: string;
  tempoUrl: string | null;
};

type ScheduleResult = {
  events: CalendarEvent[];
  fetchedAt: number; // Unix epoch milliseconds
};
```

Portal HTML fragments are converted to plain text before serialization. Date strings preserve the portal value because Aimaira returns local ISO datetimes without a timezone.

## Grade synchronization

`sync_grades` fetches the authenticated `/Note` resource and accepts only tables with a recognized grade column. The first successful call for an account stores the current result set silently. Later calls return grades whose deterministic fingerprint was not present before as `unreadAlerts`. The local SQLite database is scoped to the device application data directory; it contains no portal password or cookie.

```ts
type Grade = {
  id: string;
  subject: string;
  label: string;
  score: string;
  scale: string | null;
  coefficient: string | null;
  average: string | null;
};

type GradeSyncResult = {
  grades: Grade[];
  unreadAlerts: Grade[];
  initialized: boolean;
};
```

Call `mark_grade_alerts_read` after displaying the notification drawer. BetterAimaira never polls grades in the background and does not send system push notifications.

## Read-only portal resources

`get_portal_resource` supports `"grades"`, `"absences"`, `"profile"`, `"documents"`, and `"questionnaires"`. Aimaira serves these features as HTML pages rather than a JSON API. Rust converts each page to a semantic contract:

```ts
type PortalResource = "grades" | "absences" | "profile" | "documents" | "questionnaires";

type PortalPage = {
  resource: PortalResource;
  fetchedAt: number; // Unix epoch milliseconds
  title: string;
  headings: string[];
  tables: PortalTable[];
  fields: PortalField[];
  documents: PortalDocument[];
  gradePeriods: GradePeriod[]; // grades page only, empty elsewhere
  absencePeriods: AbsencePeriod[]; // absences page only, empty elsewhere
  questionnaires: QuestionnaireSummary[]; // questionnaires page only, empty elsewhere
  markupRecognized: boolean;
};

type PortalTable = {
  context: string[];
  caption: string | null;
  headers: string[];
  rows: string[][];
};

type PortalField = {
  label: string;
  value: string;
};

type PortalDocumentKind =
  | "absenceReport"
  | "gradeBulletin"
  | "gradeTranscript"
  | "enrollmentCertificate"
  | "schoolCertificate"
  | "schoolTranscript"
  | "gradeReport";

type PortalDocument = {
  kind: PortalDocumentKind;
  label: string;
  requestPath: string;
  suggestedFilename: string | null;
};
```

`context` contains the active heading hierarchy around a table, such as academic year, sequence, and course. Cell values preserve localized source strings; the client displays them as returned and does not infer numeric meaning without a dedicated normalized field.

### Grades

The grades page carries no table at all: Aimaira nests school year, block, learning
track, course, and evaluation in Bootstrap accordions. `gradePeriods` mirrors that
hierarchy so the client renders the same grouping instead of a flat list.

```ts
type GradePeriod = {
  id: string;
  label: string; // school year, e.g. "2025/2026"
  blocks: GradeBlock[];
};

type GradeBlock = {
  id: string;
  label: string; // e.g. "2025/26 LI-B-ESCEN N1 - BLOC 1 (01/09/2025 - 31/07/2026)"
  bulletinPath: string | null; // feed to download_portal_document
  transcriptPath: string | null;
  sections: GradeSection[];
};

type GradeSection = {
  label: string | null; // learning track heading, absent for the default group
  courses: GradeCourse[];
};

type GradeCourse = {
  id: string;
  code: string | null; // e.g. "C00004543-1"
  name: string;
  notebookPath: string | null; // course notebook, not a document download
  evaluations: GradeEvaluation[];
};

type GradeEvaluation = {
  label: string;
  score: string | null; // localized, e.g. "18,90"
  scale: string | null; // e.g. "20"
  weight: string | null; // "30,00%" for a share, "Pondération : 20,00" for a child
  children: GradeEvaluation[]; // sub-evaluations already rolled up into the parent
};
```

Averages are not returned: the portal publishes none, and a course average is only
as meaningful as the weightings it comes from. The client computes them and labels
them as indicative.

`sync_grades` flattens the same structure, sub-evaluations included, so a new mark
raises an alert wherever the portal placed it.

### Absences

The absences page reuses the grades layout: school year and block Bootstrap
accordions, each block carrying its own attendance report link and a single table
of missed sessions. `absencePeriods` mirrors that hierarchy, so the client groups
absences by year and block instead of flattening every table into one list.

```ts
type AbsencePeriod = {
  id: string;
  label: string; // school year, e.g. "2025/2026"
  blocks: AbsenceBlock[];
};

type AbsenceBlock = {
  id: string;
  label: string; // e.g. "2025/26 LI-B-ESCEN N1 - BLOC 1 (01/09/2025 - 31/07/2026)"
  reportPath: string | null; // feed to download_portal_document
  entries: AbsenceEntry[];
};

type AbsenceEntry = {
  id: string;
  date: string; // localized, e.g. "29/09/2025"
  time: string | null; // start time, split out of the portal's date cell
  course: string;
  duration: string | null; // localized hours count, e.g. "3,25"
  excused: boolean | null; // null while the school has not ruled on it
  reason: string | null; // portal `Motif` column, e.g. "Motif administratif"
};
```

The portal names the status column `Excusée` and answers `Oui` / `Non`, leaving it
empty while a justification is still being handled. `excused: null` therefore means
pending, never refused — the client shows those as awaiting review rather than as
unexcused hours.

No attendance rate is returned: the portal publishes no total of scheduled hours, so
a rate would be invented. The client totals the missed hours it is given instead.

### Questionnaires

The authenticated `/Questionnaire` page lists both pending and completed surveys.
Rust keeps the opaque response path returned by Aimaira and rejects any detail path
outside the same-origin `/Questionnaire/Reponse` route.

```ts
type QuestionnaireSummary = {
  id: string;
  title: string;
  context: string;
  deadline: string | null;
  status: string;
  completed: boolean;
  responsePath: string; // feed to get_questionnaire_detail
};

type QuestionnaireDetail = {
  title: string;
  completed: boolean;
  pages: QuestionnairePage[];
};

type QuestionnairePage = {
  id: string;
  title: string | null;
  questions: QuestionnaireQuestion[];
};

type QuestionnaireQuestion = {
  id: string;
  kind: string;
  title: string;
  description: string | null;
  required: boolean;
  options: Array<{ value: string; label: string }>;
  answers: string[];
};
```

Aimaira embeds SurveyJS configuration and saved responses as JSON in the detail
page. Rust parses those objects and serializes plain text only. BetterAimaira does
not expose a questionnaire submission command.

`markupRecognized: false` means Rust found no supported table, labeled field, or document. This differs from an authenticated resource containing a recognized empty table.

## Document downloads

Pass only a `requestPath` returned by `get_portal_resource` to `download_portal_document`. The backend rejects absolute URLs, cross-origin URLs, payment documents, write routes, unknown paths, non-PDF responses, and files larger than 25 MiB.

Tauri returns the command body through its raw binary response path. Treat it as an `ArrayBuffer` and use the matching `PortalDocument` metadata for the visible label and suggested filename.

## Updates

Three commands, one per step of the update flow. The implementation differs by
platform, the contract does not.

```ts
type UpdateDelivery = "inApp" | "androidPackage" | "altStore";

type UpdateInfo = {
  available: boolean;
  currentVersion: string;
  latestVersion: string | null;
  notes: string | null;
  publishedAt: string | null;
  delivery: UpdateDelivery;
  downloadUrl: string | null;
  storeUrl: string | null;
};

type InstallOutcome = {
  handedOff: boolean;
  permissionRequired: boolean;
};
```

`install_update` never returns on desktop: the signed bundle is installed and the
process restarts. On Android it returns once the system package installer has the
APK, or with `permissionRequired` when the user still has to allow installs from
this app. On iOS it opens the AltStore source deep link.

Progress events during a download:

| Event | Payload |
|---|---|
| `update://download-progress` | `{ downloaded: number, total: number \| null }` |
| `update://downloaded` | none |

## Stable error codes

| Code | Meaning |
|---|---|
| `invalid_portal_url` | Portal URL is malformed or contains embedded credentials. |
| `insecure_portal_url` | Portal does not use HTTPS. |
| `missing_credentials` | Username or password is empty. |
| `invalid_credentials` | Aimaira returned its login page after submission. |
| `portal_unreachable` | Login request or portal response failed. |
| `portal_not_aimaira` | Login page lacks the expected anti-CSRF field. |
| `credential_store` | Native credential store could not be read, written, or cleared. |
| `session_expired` | An authenticated route redirected to the login page. |
| `invalid_schedule_range` | Schedule start is empty or duration is outside 1 through 42. |
| `planning_unavailable` | Schedule request failed. |
| `planning_invalid_response` | Schedule payload is neither empty nor valid event JSON. |
| `grades_unavailable` | Grades page request failed. |
| `grades_invalid_response` | Grades page has recognized markup but no supported grade rows. |
| `grade_storage_unavailable` | Local grade synchronization database could not be used. |
| `absences_unavailable` | Absences page request failed. |
| `profile_unavailable` | Profile page request failed. |
| `documents_unavailable` | Documents page request failed. |
| `questionnaires_unavailable` | Questionnaire list or detail request failed. |
| `questionnaire_invalid_response` | SurveyJS data is missing or unsupported. |
| `invalid_questionnaire_request` | Detail path is outside the read-only questionnaire route. |
| `invalid_document_request` | Download path is not in the read-only allowlist. |
| `document_unavailable` | PDF request failed. |
| `document_invalid_response` | Response is not a valid PDF. |
| `document_too_large` | PDF exceeds the 25 MiB IPC limit. |
| `update_check_failed` | Update feed request failed or the updater could not start. |
| `update_manifest_invalid` | Update manifest parsed but carries no usable entry for this platform. |
| `update_not_available` | Install was requested while no newer version is published. |
| `update_download_failed` | Update payload request or local write failed. |
| `update_install_failed` | Bundle install, or the Android package installer handover, failed. |
| `update_store_unavailable` | AltStore deep link could not be opened. |
| `internal_error` | Backend state lock or system time failed. |

## Verification

Run from `src-tauri`:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

Parser tests use synthetic anonymized HTML. No portal captures, credentials, cookies, or personal data belong in the repository.
