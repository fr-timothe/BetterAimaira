[← Documentation index](README.md)

# Rust backend API

This document describes the client-facing contract exposed by the Tauri backend. The Rust process owns portal authentication, cookies, HTML parsing, URL validation, and document downloads. Client code must not call Aimaira routes directly or parse portal HTML.

## Command summary

| Command | Input | Output |
|---|---|---|
| `normalize_portal_url` | `{ portalUrl: string }` | `{ portalUrl: string }` |
| `login` | `{ request: LoginRequest }` | `LoginResult` |
| `restore_session` | none | `RestoreResult` |
| `saved_identity` | none | `SavedIdentity \| null` |
| `logout` | none | `null` |
| `get_schedule` | `{ request: ScheduleRequest }` | `ScheduleResult` |
| `get_planning_settings` | none | `PlanningSettingsResult` |
| `get_portal_resource` | `{ resource: PortalResource, force?: boolean }` | `PortalPage` |
| `get_questionnaire_detail` | `{ request: { responsePath: string } }` | `QuestionnaireDetail` |
| `sync_grades` | `{ force?: boolean }` | `GradeSyncResult` |
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
the copy cached on the session, so there is no separate refresh command. It is
also the only command that reads them: `sundaysVisible` on a `LoginResult` is the
default and never the portal's value, because neither `login` nor
`restore_session` loads `/Calendar`. A client that honours the setting has to ask
for it after every session it opens, a replayed one included.

```ts
type SavedIdentity = {
  portalUrl: string;
  username: string;
  hasSnapshots: boolean; // something is stored for this account, see Offline snapshots
};

type RestoreResult = {
  status: 'restored' | 'no_credentials' | 'credentials_rejected';
  session: LoginResult | null; // set only when status is `restored`
  identity: SavedIdentity | null; // absent only when nothing was saved
};
```

`saved_identity` reads the saved identity alone and never touches the password
entry or the network, so the client can decide between a startup wait and the
login form before the slow work begins. It answers `null` when no account is
saved and fails with `credential_store` when the platform store is unavailable.
`hasSnapshots` says whether the account has anything on disk to show before the
portal has been reached; it only ever widens what the client may offer, so a
storage failure reports `false` rather than failing the call.

`restore_session` reads the last saved identity and password from the operating
system credential store, then performs a normal portal login to create a fresh
in-memory cookie jar. It is not a startup-only command: the client also calls it
in the middle of a session, to replay the saved password behind a read that came
back `session_expired`, which
[Architecture](ARCHITECTURE.md#replaying-the-password-behind-a-failed-read)
describes. Either way the call is a real sign-in, made with a stored password and
no gesture from the reader — the same sign-in the "remember me" checkbox buys at
every cold start — so at a school that counts concurrent sessions or emails a
notice on each login, one expiry now costs one extra portal login.

`status` separates the two outcomes that are not a restored session:
`no_credentials` when nothing is saved, and `credentials_rejected` when Aimaira
refused the saved password. A rejection removes the password entry and keeps the
identity, so `identity` still carries the account and the client can open a
pre-filled login form. Keeping it is what also keeps the stored snapshots
reachable: the identity is what names them while no session is open, so dropping
it over a wrong password would orphan every page and range already on disk. The
consequence to expect is that a rejection does not survive a restart — the next
cold start finds an identity and no password, so `restore_session` answers
`no_credentials`, and the form opens pre-filled with nothing to report. Every
other failure is a normal command error, for example `portal_unreachable`, which
the client retries. Cookies remain private to Rust. `logout` is the deliberate
sign-out and the only path that forgets the account: it drops the in-memory
cookie jar and removes both the password and the identity.

## Offline snapshots

A read the portal cannot answer falls back to the copy on disk written by the last
successful fetch. `PortalPage`, `ScheduleResult` and `GradeSyncResult` each carry
`stale`, raised only on that path. A stale payload keeps the `fetchedAt` of the
fetch it came from and never the moment it was read back, so the client can state
the real age of what is on screen instead of presenting old content as current.

That makes three tiers in front of a portal resource: a five-minute in-session
memory copy, which `force` skips, then the portal, then the snapshot.
`get_schedule` has no memory tier of its own and always asks the portal first.
Rust never memoises a stale answer, and the client should not either: the next
read is what reaches the portal again once it is back.

Snapshots are filed under an account key derived from the portal address and the
username. The key comes from the live session when one is open and from the saved
identity otherwise — that fallback is what lets a cold start with no session at
all find its own rows. The saved identity therefore outlives a password the portal
rejected, and only `logout` removes it; see Authentication above.

One failure is never answered from disk. `session_expired` covers two situations:
no session is open, which is the cold offline start this exists for, and the
portal redirected a live session to its login page. Only the first falls back.
Replaying a snapshot for the second would hide the expiry behind data that can no
longer change, leaving the reader on frozen content with nothing offering the
sign-in that [DESIGN.md](../DESIGN.md) makes the required action for an expired
session. It would also suppress the recovery: the error surfacing is what lets
the client replay the saved password through `restore_session` and retry the read
before the reader is asked for anything. Every other code, `portal_unreachable`
and the per-resource failures included, falls back either way.

Writing a snapshot never fails a call: a result the caller already holds is
returned whether or not it reached disk.

The snapshots live in one application-data SQLite database, the stored grades
included. It holds no password and no cookie, it is not encrypted, and `logout`
clears the credentials and the session without deleting the stored rows.

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
  stale: boolean; // replayed from disk, see Offline snapshots
};
```

Portal HTML fragments are converted to plain text before serialization. Date strings preserve the portal value because Aimaira returns local ISO datetimes without a timezone.

Each requested range is snapshotted on its own, under the start instant and
duration it was asked with. A range that was never fetched has nothing to replay,
so an unreachable portal still fails for it.

## Grade synchronization

`sync_grades` fetches the authenticated `/Note` resource, accepts only tables with
a recognized grade column, and answers with the current school year flattened into
one list. That same list is what goes to disk, replacing the rows held for the
account, so the online answer and the one replayed offline are the same list
rather than two that disagree. Older school years are read through
`get_portal_resource`, which returns all of them in `gradePeriods`. The local
SQLite database is scoped to the device application data directory; it contains no
portal password or cookie.

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
  stale: boolean; // replayed from disk, see Offline snapshots
};
```

When the grades page itself came from a snapshot, `sync_grades` answers `stale`
with the stored grades and writes nothing back: a page the app could not refresh
is no evidence of what the portal holds now, so the copy on disk stays as the last
successful fetch left it.

BetterAimaira never polls grades in the background and does not send system push notifications.

## Read-only portal resources

`get_portal_resource` supports `"grades"`, `"absences"`, `"profile"`, `"documents"`, and `"questionnaires"`. Aimaira serves these features as HTML pages rather than a JSON API. Rust converts each page to a semantic contract:

```ts
type PortalResource = "grades" | "absences" | "profile" | "documents" | "questionnaires";

type PortalPage = {
  resource: PortalResource;
  fetchedAt: number; // Unix epoch milliseconds
  stale: boolean; // replayed from disk, see Offline snapshots
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

`sync_grades` flattens the same structure for the current school year,
sub-evaluations included, so a mark is carried wherever the portal placed it.

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
